import json
from apps.crypto_24h.observation_store import ObservationStore
from apps.crypto_24h.rolling_state_engine import RollingStateEngine
from apps.crypto_24h.crypto_ic import NativeCryptoIC
from apps.crypto_24h.admission_gate import AdmissionGate
from apps.crypto_24h.decision_brief import DecisionBrief
from apps.crypto_24h.lifecycle_manager import LifecycleManager

def run_replay(dataset_path):
    """
    Deterministic event loop processing a historical 1m stream.
    Enforces the invariant: Nothing at time t can depend on anything after t.
    """
    obs_store = ObservationStore()
    state_engine = RollingStateEngine()
    ic_layer = NativeCryptoIC()
    admission_gate = AdmissionGate()
    lifecycle_mgr = LifecycleManager()
    
    with open(dataset_path, 'r') as f:
        bars = json.load(f)
        
    full_trace = []
    
    # Process observations strictly in temporal order
    for bar in bars:
        timestamp = bar["timestamp"]
        
        # 1. Update Observation Store
        added = obs_store.add(
            timestamp, 
            bar["open"], 
            bar["high"], 
            bar["low"], 
            bar["close"], 
            bar["volume"]
        )
        if not added:
            continue
            
        # 2. Update Lifecycle (exit evaluations use the *current* observation)
        lifecycle_mgr.update(bar)
        
        # Capture lifecycle events at this tick
        newly_closed = [p for p in lifecycle_mgr.closed_positions if getattr(p, '_just_closed_at', None) == timestamp]
        for p in lifecycle_mgr.active_positions:
            if p.is_closed and not hasattr(p, '_just_closed_at'):
                p._just_closed_at = timestamp
                newly_closed.append(p)
                
        lc_events = []
        for p in newly_closed:
            lc_events.append({
                "entry_px": p.entry_px,
                "exit_px": p.exit_px,
                "exit_reason": p.exit_reason,
                "direction": p.direction
            })
            
        # 3. Synchronous State Update (uses obs <= t)
        state_snapshot = state_engine.update(obs_store)
        
        # 4. IC Evaluation (uses state at t)
        ic_output = ic_layer.evaluate(state_snapshot)
        
        # 5. Admission Gate (uses IC at t)
        decision = admission_gate.evaluate(ic_output)
        
        # 6. Decision Generation (if ACT)
        brief_dict = None
        if decision.action == "ACT":
            brief = DecisionBrief(decision, state_snapshot, ic_output)
            brief_dict = brief.to_dict()
            lifecycle_mgr.add_position(brief, entry_px=bar["close"])
            
        # Compile full trace for this observation
        step_trace = {
            "observation_id": timestamp,
            "state_snapshot": state_snapshot.to_dict() if state_snapshot else None,
            "ic_output": ic_output.to_dict() if ic_output else None,
            "admission": decision.to_dict(),
            "decision_brief": brief_dict,
            "lifecycle_events": lc_events
        }
        full_trace.append(step_trace)
            
    return full_trace

if __name__ == "__main__":
    import sys
    import hashlib
    from pathlib import Path
    workspace = Path("/Users/nikhil/ChronoSentiment_MEGA_FINAL")
    dataset = workspace / "datasets/e9/crypto/high_vol_BTCUSDT.json"
    
    if dataset.exists():
        print(f"Running deterministic replay on {dataset.name}...")
        result1 = run_replay(dataset)
        result2 = run_replay(dataset)
        
        res1_str = json.dumps(result1, sort_keys=True)
        res2_str = json.dumps(result2, sort_keys=True)
        
        hash1 = hashlib.sha256(res1_str.encode()).hexdigest()
        hash2 = hashlib.sha256(res2_str.encode()).hexdigest()
        
        print(f"Trace 1 Length: {len(result1)} ticks")
        print(f"Trace 1 SHA256: {hash1}")
        print(f"Trace 2 SHA256: {hash2}")
        
        if hash1 == hash2:
            print("DETERMINISM TEST: PASS (Full trace is byte-equivalent)")
        else:
            print("DETERMINISM TEST: FAIL")
    else:
        print(f"Dataset not found: {dataset}")
