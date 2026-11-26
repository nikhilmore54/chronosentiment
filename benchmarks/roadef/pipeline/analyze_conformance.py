import json
import statistics
import sys
from collections import defaultdict

def main():
    try:
        with open("benchmarks/roadef/pipeline/pipeline_ab_report.json", "r") as f:
            data = json.load(f)
    except FileNotFoundError:
        print("Report JSON not found.")
        return

    arm_a = { (r["instance"], r.get("seed", 42)): r for r in data["arm_a"] }
    arm_b = { (r["instance"], r.get("seed", 42)): r for r in data["arm_b"] }

    pairs = set(arm_a.keys()).intersection(set(arm_b.keys()))

    if not pairs:
        print("No completed pairs found.")
        return

    obj_deltas = []
    runtime_deltas = []
    ifr_deltas = []
    
    wins = 0
    ties = 0
    losses = 0
    
    a_invariants = 0
    b_invariants = 0
    
    # We would need operator error parsing from logs for repair failures, but for now 
    # we rely on the validity/invariants in the JSON.

    for pair_key in pairs:
        res_a = arm_a[pair_key]
        res_b = arm_b[pair_key]
        
        d_obj = res_b["best_obj"] - res_a["best_obj"]
        d_rt = res_b["runtime_ms"] - res_a["runtime_ms"]
        d_ifr = res_b["initial_feasibility_rate"] - res_a["initial_feasibility_rate"]
        
        obj_deltas.append(d_obj)
        runtime_deltas.append(d_rt)
        ifr_deltas.append(d_ifr)
        
        # Win logic (lower obj is better)
        if d_obj < -1e-6:
            wins += 1
        elif d_obj > 1e-6:
            losses += 1
        else:
            ties += 1
            
        if res_a.get("invariant_violation_suspected", False): a_invariants += 1
        if res_b.get("invariant_violation_suspected", False): b_invariants += 1
        
    print(f"=== ROADEF Architecture Conformance Report (Partial) ===")
    print(f"Completed Pairs: {len(pairs)} / {data.get('instances_total', 20) * 10}")
    
    print("\n--- Objective Deltas (Pipeline - Legacy) ---")
    print(f"Mean Δ:   {statistics.mean(obj_deltas):+.4f}")
    if len(obj_deltas) > 1:
        print(f"StdDev Δ: {statistics.stdev(obj_deltas):.4f}")
    
    print(f"Wins:   {wins}")
    print(f"Ties:   {ties}")
    print(f"Losses: {losses}")
    
    print("\n--- Runtime Deltas (Pipeline - Legacy) ---")
    print(f"Mean Δ:   {statistics.mean(runtime_deltas):+.0f} ms")
    
    print("\n--- Feasibility & Invariants ---")
    print(f"Legacy Invariant Violations:   {a_invariants}")
    print(f"Pipeline Invariant Violations: {b_invariants}")

if __name__ == "__main__":
    main()
