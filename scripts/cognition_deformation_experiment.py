import json
import math
import hashlib
from pathlib import Path
import sys

def deterministic_acceptance(ts, symbol, acceptance_ratio):
    hash_int = int(hashlib.md5(f"{ts}_{symbol}".encode()).hexdigest(), 16)
    normalized = (hash_int % 1000) / 1000.0
    return normalized <= acceptance_ratio

def compute_frozen_metrics(series):
    n = len(series)
    if n < 2: return {"mean": 0.0, "var": 0.0, "ac1": None, "ent": 0.0}
    mean = sum(series) / n
    var = sum((x - mean) ** 2 for x in series)
    from collections import Counter
    transitions = [round(series[i] - series[i-1], 2) for i in range(1, n)]
    counts = Counter(transitions)
    total = sum(counts.values())
    ent = -sum((c/total) * math.log2(c/total) for c in counts.values() if c > 0)
    ac1 = None
    if var > 1e-9:
        ac1 = sum((series[i] - mean) * (series[i+1] - mean) for i in range(n - 1)) / var
    return {"mean": mean, "var": var, "ac1": ac1, "ent": ent}

def simulate_memory_deformation(df, admissibility_map, symbol, geometry_type):
    baseline_mem = []
    frag_mem = []
    occupancy_series = []
    
    for i in range(len(df)):
        ts = int(df.index[i].timestamp())
        px = float(df['Close'].iloc[i])
        
        curr_adm = admissibility_map.get(ts, {})
        if not curr_adm: continue
        
        curr_accept_ratio = curr_adm.get("observability", {}).get("acceptance_ratio", 1.0)
        curr_accepted = deterministic_acceptance(ts, symbol, curr_accept_ratio)
        
        # Baseline always accepts (perfect environment)
        baseline_mem.append(px)
        
        # Fragmented accepts based on topology
        if curr_accepted:
            frag_mem.append(px)
            
        # Apply Geometry Rules
        if geometry_type == "rolling_50":
            if len(baseline_mem) > 50: baseline_mem.pop(0)
            if len(frag_mem) > 50: frag_mem.pop(0)
            
        elif geometry_type == "rolling_100":
            if len(baseline_mem) > 100: baseline_mem.pop(0)
            if len(frag_mem) > 100: frag_mem.pop(0)
            
        elif geometry_type == "event_reset":
            # Reset memory if price drops > 0.5% from max in memory
            if len(baseline_mem) > 0 and px < max(baseline_mem) * 0.995:
                baseline_mem.clear()
            if len(frag_mem) > 0 and px < max(frag_mem) * 0.995:
                frag_mem.clear()
                
        elif geometry_type == "accumulator":
            # Infinite state accumulator, never pops
            pass
            
        # Calculate Overlap
        if len(baseline_mem) == 0:
            occupancy = 0.0
        else:
            # Align from the right (most recent)
            cmp_frag = list(frag_mem)
            while len(cmp_frag) < len(baseline_mem):
                cmp_frag.insert(0, cmp_frag[0] if cmp_frag else 0)
            overlap_count = sum(1 for a, b in zip(baseline_mem, cmp_frag) if a == b)
            occupancy = 1.0 - (overlap_count / len(baseline_mem))
            
        occupancy_series.append(occupancy)
        
    return occupancy_series

def run_orthogonal_geometries():
    print("🔬 COMPARATIVE COGNITION DEFORMATION (METROLOGY FROZEN)")
    print("Substrate: BTCUSDT (batch_10001, 72h Continuous)")
    print("Objective: Observe Deformation Geometry Across Orthogonal Memory Architectures")
    print("=" * 115)
    
    batch_id = 10001
    symbol = "BTCUSDT"
    import sys
    sys.path.append(str(Path(__file__).parent))
    from synthetic_fragmentation_injector import inject_topology
    from candle_substrate import load_frozen_cohort
    
    data, _ = load_frozen_cohort(batch_id, [symbol])
    df = data[symbol].sort_index()
    
    base_ledger = Path(f"state_archive/batches/batch_{batch_id}/runs/live/metadata/live_session_steps.jsonl")
    topologies = ["osc_P50_A100", "osc_P100_A100"]
    geometries = ["rolling_50", "rolling_100", "event_reset", "accumulator"]
    
    print(f"{'TOPOLOGY':<16} | {'MEMORY GEOMETRY':<16} | {'MEAN OCC.':<10} | {'AC(L=1)':<12} | {'TRANS. ENT.':<12}")
    print("-" * 115)
    
    for mode in topologies:
        inject_topology(base_ledger, mode)
        topo_file = f"synthetic_{mode}_steps"
        ledger_path = Path(f"state_archive/batches/batch_{batch_id}/runs/live/metadata/{topo_file}.jsonl")
        
        adm = {}
        with open(ledger_path, 'r') as f:
            for line in f:
                if line.strip():
                    row = json.loads(line)
                    adm[row['barrier_ts']] = row
                    
        for geo in geometries:
            series = simulate_memory_deformation(df, adm, symbol, geo)
            metrics = compute_frozen_metrics(series)
            
            mean_str = f"{metrics['mean']:.3f}"
            ac1_str = str(round(metrics['ac1'], 3)) if metrics['ac1'] is not None else "DEGENERATE"
            ent_str = f"{metrics['ent']:.3f}"
            
            print(f"{mode:<16} | {geo:<16} | {mean_str:<10} | {ac1_str:<12} | {ent_str:<12}")
            
    print("=" * 115)

if __name__ == "__main__":
    run_orthogonal_geometries()
