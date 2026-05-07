import subprocess
import re
import pandas as pd
import numpy as np
import sys

# Configuration
SEEDS = [80, 81, 82]
DURATION = 200
STABILITY_THRESHOLDS = [1.0, 0.5, 0.1, 0.05, 0.01]

def run_stability_test(seed, threshold):
    cmd = f"python3 scripts/mock_streamer.py --duration {DURATION} --seed {seed} | REC_STABILITY_MAX={threshold} TRAP_POLICY=8 ./target/debug/examples/live_engine"
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
    
    m = re.search(r"avg_pnl=([-+]?\d*\.\d+)", result.stderr)
    n = re.search(r"closed_trades=(\d+)", result.stderr)
    if m and n:
        pnl = float(m.group(1)) * 10000
        return pnl, int(n.group(1))
    return None, 0

def main():
    print("🧪 STABILITY SNIPER VALIDATION (Path Quality Sweep)")
    print("-" * 75)
    print(f"{'Threshold':<12} | {'Mean PnL':<12} | {'Total Trades':<12} | {'Efficiency':<12}")
    print("-" * 75)
    sys.stdout.flush()
    
    for threshold in STABILITY_THRESHOLDS:
        pnls = []
        ns = []
        for s in SEEDS:
            p, n = run_stability_test(s, threshold)
            if p is not None:
                pnls.append(p)
                ns.append(n)
        
        if pnls:
            mean_pnl = np.mean(pnls)
            total_n = np.sum(ns)
            efficiency = mean_pnl / (total_n / 1000) if total_n > 0 else 0
            print(f"{threshold:<12} | {mean_pnl:>+8.2f} bps | {total_n:>12} | {efficiency:>10.2f}")
            sys.stdout.flush()

if __name__ == "__main__":
    main()
