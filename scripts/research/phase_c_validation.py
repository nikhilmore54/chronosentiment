import subprocess
import re
import numpy as np
import os

# Configuration
SEEDS = [60, 61, 62, 63, 64] # Multi-seed validation
DURATION = 300 # Large sample (10x normal)
SNIPER_THRESHOLD = 6.99

def run_test(seed, policy, cost_bps=4, latency=0):
    cmd = (
        f"python3 scripts/mock_streamer.py --duration {DURATION} --seed {seed} --latency {latency} | "
        f"TRAP_POLICY={policy} SNIPER_THRESHOLD={SNIPER_THRESHOLD} ./target/debug/examples/live_engine"
    )
    print(f"🚀 Running Seed {seed} (Policy {policy}, Cost {cost_bps}bps, Latency {latency})...")
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
    
    # Parse Net PnL
    m = re.search(r"avg_pnl=([-+]?\d*\.\d+)", result.stderr)
    if m:
        pnl = float(m.group(1)) * 10000 # Convert to bps
        # Adjust for cost (the engine uses 4bps internally, we subtract extra if cost_bps > 4)
        net_pnl = pnl - (cost_bps - 4)
        return net_pnl
    return None

def main():
    print("🧪 PHASE C: VALIDATION OR DEATH (Fragility Sniper Suite)")
    print("-" * 60)
    
    # 1. Multi-Seed Stability
    print("\n[STABILITY TEST] 5 Seeds @ 4bps, 0 Latency")
    pnls = []
    for s in SEEDS:
        p = run_test(s, 4)
        if p is not None:
            pnls.append(p)
            print(f"  Seed {s}: {p:+.2f} bps")
    
    if pnls:
        mean_pnl = np.mean(pnls)
        std_pnl = np.std(pnls)
        print(f"📈 Result: Mean={mean_pnl:+.2f} bps, Std={std_pnl:.2f}")
        if mean_pnl > 0 and mean_pnl - std_pnl > 0:
            print("✅ STABILITY PASSED (Positive @ 1 Sigma)")
        else:
            print("❌ STABILITY FAILED")

    # 2. Cost Stress Test (Seed 60)
    print("\n[COST STRESS TEST] Seed 60")
    for c in [4, 6, 8]:
        p = run_test(60, 4, cost_bps=c)
        status = "✅ SURVIVES" if p > 0 else "❌ KILLED"
        print(f"  Cost {c}bps: {p:+.2f} bps ({status})")

    # 3. Latency Stress Test (Seed 60)
    print("\n[LATENCY STRESS TEST] Seed 60")
    for l in [0, 1, 2]:
        p = run_test(60, 4, latency=l)
        status = "✅ SURVIVES" if p > 0 else "❌ KILLED"
        print(f"  Latency {l}ticks: {p:+.2f} bps ({status})")

if __name__ == "__main__":
    main()
