import subprocess
import re
import pandas as pd
import numpy as np

# Configuration
PERSISTENCE_RANGE = [8, 9, 10]
INTENSITY_RANGE = [1.8, 2.0, 2.2, 2.4]
SEEDS = [80, 81, 82]
DURATION = 150

def run_sim(p, i, seed):
    env = f"TRAP_POLICY=6 STRUC_PERSISTENCE={p} STRUC_INTENSITY={i}"
    cmd = f"python3 scripts/mock_streamer.py --duration {DURATION} --seed {seed} | {env} ./target/debug/examples/live_engine"
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
    
    m = re.search(r"avg_pnl=([-+]?\d*\.\d+)", result.stderr)
    n = re.search(r"closed_trades=(\d+)", result.stderr)
    if m and n:
        return float(m.group(1)) * 10000, int(n.group(1))
    return 0.0, 0

def main():
    print("🔬 STRUCTURAL GRID SEARCH (Finding the Monetization Frontier)")
    print("-" * 75)
    print(f"{'Pers':6} | {'Inten':6} | {'Mean PnL (bps)':15} | {'N':5}")
    print("-" * 75)
    
    results = []
    for p in PERSISTENCE_RANGE:
        for i in INTENSITY_RANGE:
            pnls = []
            ns = []
            for s in SEEDS:
                pnl, n = run_sim(p, i, s)
                pnls.append(pnl)
                ns.append(n)
            
            mean_pnl = np.mean(pnls)
            total_n = np.sum(ns)
            results.append({'p': p, 'i': i, 'pnl': mean_pnl, 'n': total_n})
            
            status = "🏆 ALPHA" if mean_pnl > 0 else ""
            print(f"{p:6} | {i:6.1f} | {mean_pnl:15.2f} | {total_n:5} {status}")

    df = pd.DataFrame(results)
    best = df[df['n'] > 5].sort_values('pnl', ascending=False).iloc[0]
    print(f"\n🎯 FRONTIER OPTIMUM")
    print(f"   Persistence: {best['p']}")
    print(f"   Intensity  : {best['i']}")
    print(f"   Net PnL    : {best['pnl']:+.2f} bps")

if __name__ == "__main__":
    main()
