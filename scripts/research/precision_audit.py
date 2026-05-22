import subprocess
import re
import pandas as pd
import numpy as np

# Configuration
TESTS = [
    {'delay': 1, 'min': 1.7, 'max': 1.9, 'name': 'Aggressive (Delay=1, Band=1.7-1.9)'},
    {'delay': 2, 'min': 1.7, 'max': 1.9, 'name': 'Balanced (Delay=2, Band=1.7-1.9)'},
    {'delay': 2, 'min': 1.8, 'max': 2.2, 'name': 'High-Con (Delay=2, Band=1.8-2.2)'}
]
SEEDS = [80, 81, 82]
DURATION = 150

def run_precision_sim(test, seed):
    env = f"TRAP_POLICY=6 STRUC_DELAY={test['delay']} STRUC_INTENSITY_MIN={test['min']} STRUC_INTENSITY_MAX={test['max']}"
    cmd = f"python3 scripts/mock_streamer.py --duration {DURATION} --seed {seed} | {env} ./target/debug/examples/live_engine"
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
    
    m = re.search(r"avg_pnl=([-+]?\d*\.\d+)", result.stderr)
    n = re.search(r"closed_trades=(\d+)", result.stderr)
    if m and n:
        return float(m.group(1)) * 10000, int(n.group(1))
    return 0.0, 0

def main():
    print("🔬 PRECISION SNIPER AUDIT (Tuning the Extraction Window)")
    print("-" * 75)
    
    for t in TESTS:
        print(f"\n🧪 Testing: {t['name']}")
        pnls = []
        ns = []
        for s in SEEDS:
            pnl, n = run_precision_sim(t, s)
            pnls.append(pnl)
            ns.append(n)
        
        mean_pnl = np.mean(pnls)
        total_n = np.sum(ns)
        status = "🏆 PROFITABLE" if mean_pnl > 0 else "❌ LOSS"
        print(f"   Mean PnL: {mean_pnl:+.2f} bps | Total Trades: {total_n} | Status: {status}")

if __name__ == "__main__":
    main()
