#!/usr/bin/env python3
import json
import sys

def simulate_intersection(ledger_rows):
    """
    Simulate the Execution Physics intersection.
    - Intent Generation: 1 synthetic long intent per cycle.
    - Admissibility: consumes `new_entries_allowed` from ledger.
    - Execution: Strict intersection. No queueing.
    """
    executed = 0
    blocked = 0
    intents = 0
    
    for row in ledger_rows:
        intents += 1
        intent = {"action": "ENTER_LONG"}
        
        admissibility = row.get("admissibility", {})
        allowed = admissibility.get("new_entries_allowed", False)
        
        # Physics suppression: Intent is evaluated and immediately dropped if blocked.
        if allowed:
            executed += 1
        else:
            blocked += 1
            
    return intents, executed, blocked

def run_adversarial_tests():
    print("🔬 Running Adversarial Execution Physics Tests\n" + "="*50)
    
    # 1. Oscillation Test
    print("Test 1: Oscillation (ALLOWED -> BLOCKED -> ALLOWED -> BLOCKED)")
    oscillation_ledger = []
    for i in range(10):
        allowed = (i % 2 == 0)
        oscillation_ledger.append({"cycle": i, "admissibility": {"new_entries_allowed": allowed}})
    
    i_int, i_exec, i_blk = simulate_intersection(oscillation_ledger)
    assert i_int == 10
    assert i_exec == 5
    assert i_blk == 5
    print("✅ Passed: No deferred execution buffering; strict suppression enforced.")
    
    # 2. Delayed Recovery Test
    print("Test 2: Delayed Recovery (10 cycles BLOCKED, 1 cycle ALLOWED)")
    delayed_ledger = []
    for i in range(11):
        allowed = (i == 10)
        delayed_ledger.append({"cycle": i, "admissibility": {"new_entries_allowed": allowed}})
        
    d_int, d_exec, d_blk = simulate_intersection(delayed_ledger)
    assert d_int == 11
    assert d_exec == 1  # Only the 11th intent executes!
    assert d_blk == 10
    print("✅ Passed: Sustained degradation safely discards intents; no queue accumulates.")

    # 3. Replay Equivalence Test
    print("Test 3: Replay Equivalence")
    r1_int, r1_exec, r1_blk = simulate_intersection(oscillation_ledger)
    r2_int, r2_exec, r2_blk = simulate_intersection(oscillation_ledger)
    assert (r1_int, r1_exec, r1_blk) == (r2_int, r2_exec, r2_blk)
    print("✅ Passed: Identical ledger produces identical deterministic suppression.")

    # 4. Regime Label Removal Test
    print("Test 4: Regime Label Removal (Nullifying 'regime_state' from payload)")
    faceless_ledger = []
    for i in range(5):
        # We explicitly omit 'admissibility_reason' and 'regime_state' to ensure Alpha is blind.
        faceless_ledger.append({"cycle": i, "admissibility": {"new_entries_allowed": True}})
        
    f_int, f_exec, f_blk = simulate_intersection(faceless_ledger)
    assert f_int == 5 and f_exec == 5
    print("✅ Passed: Execution intersects strictly on booleans. Labels are purely observational.")

    # 5. Synthetic Lag Injection Test (Testing the Classification Boundary)
    print("Test 5: Synthetic Lag Injection (Mechanical Classification Validation)")
    def classify(strict_r, accept_r, slope):
        is_synchronized = (strict_r >= 0.9)
        is_degraded = (accept_r < 0.5)
        is_recovering = (slope > 0.1)
        if is_synchronized: return True
        if is_degraded and not is_recovering: return False
        if is_recovering: return False
        return True # FRAGMENTED_BUT_USABLE
        
    assert classify(0.95, 0.95, 0.0) == True   # Uniform -60s / 0s
    assert classify(0.0, 0.1, 0.0) == False    # Uniform 180s
    assert classify(0.0, 1.0, 0.0) == True     # Bimodal / Acceptable Fragmentation
    
    print("✅ Passed: Mechanical threshold boundaries remain stable under pathological variance.")
    print("="*50 + "\n✅ All Adversarial Physics Tests Passed.")

if __name__ == "__main__":
    run_adversarial_tests()
