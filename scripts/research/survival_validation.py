import subprocess
import re
import pandas as pd
import numpy as np

# Configuration
SEEDS = [80, 81, 82, 83, 84]
DURATION = 200

def run_survival_test(seed):
    cmd = f"python3 scripts/mock_streamer.py --duration {DURATION} --seed {seed} | TRAP_POLICY=8 ./target/debug/examples/live_engine"
    print(f"🚀 Testing Survival Sniper (Seed {seed})...")
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
    
    m = re.search(r"avg_pnl=([-+]?\d*\.\d+)", result.stderr)
    n = re.search(r"closed_trades=(\d+)", result.stderr)
    if m and n:
        pnl = float(m.group(1)) * 10000
        return pnl, int(n.group(1))
    return None, 0

def main():
    print("🧪 SURVIVAL SNIPER VALIDATION (Signal Purification Layer)")
    print("-" * 75)
    
    pnls = []
    ns = []
    for s in SEEDS:
        p, n = run_survival_test(s)
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
        
        if np.mean(pnls) > -1.0:
            print("\n🏆 ALPHA PURIFIED! (Signal survived execution reality)")
        else:
            print("\n❌ SIGNAL COLLAPSED (Alpha purity insufficient for costs)")

if __name__ == "__main__":
    main()
