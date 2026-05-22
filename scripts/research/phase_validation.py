import subprocess
import re
import pandas as pd
import numpy as np

# Configuration
SEEDS = [80, 81, 82, 83, 84]
DURATION = 150

def run_phase_test(seed):
    cmd = f"python3 scripts/mock_streamer.py --duration {DURATION} --seed {seed} | TRAP_POLICY=7 ./target/debug/examples/live_engine"
    print(f"🚀 Testing Phase-Aligned Sniper (Seed {seed})...")
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
    
    m = re.search(r"avg_pnl=([-+]?\d*\.\d+)", result.stderr)
    n = re.search(r"closed_trades=(\d+)", result.stderr)
    if m and n:
        pnl = float(m.group(1)) * 10000
        return pnl, int(n.group(1))
    return None, 0

def main():
    print("🧪 PHASE-ALIGNED SNIPER VALIDATION (Persistence + Intensity + Plateau Detection)")
    print("-" * 75)
    
    pnls = []
    ns = []
    for s in SEEDS:
        p, n = run_phase_test(s)
        if p is not None:
            pnls.append(p)
            ns.append(n)
            status = "✅" if p > -1.0 else "❌"
            print(f"  Seed {s}: {p:+.2f} bps | N={n} {status}")
            
    if pnls:
        print("\n📈 AGGREGATE RESULTS")
        print("-" * 30)
        print(f"Mean PnL   : {np.mean(pnls):+.2f} bps")
        print(f"Total Trades: {np.sum(ns)}")
        print(f"Consistency: {sum(p > -1.0 for p in pnls)/len(pnls):.0%}")

if __name__ == "__main__":
    main()
