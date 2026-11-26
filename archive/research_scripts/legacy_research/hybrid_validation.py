import subprocess
import re
import pandas as pd
import numpy as np

# Configuration
SEEDS = [80, 81, 82]
DURATION = 150

def run_hybrid_test(seed):
    cmd = f"python3 scripts/mock_streamer.py --duration {DURATION} --seed {seed} | TRAP_POLICY=5 ./target/debug/examples/live_engine"
    print(f"🚀 Testing Hybrid Strategy (Seed {seed})...")
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
    
    m = re.search(r"avg_pnl=([-+]?\d*\.\d+)", result.stderr)
    if m:
        pnl = float(m.group(1)) * 10000
        return pnl
    return None

def main():
    print("🧪 HYBRID DIRECTIONAL TIMING VALIDATION (Direction + Timing + Delay)")
    print("-" * 65)
    
    pnls = []
    for s in SEEDS:
        p = run_hybrid_test(s)
        if p is not None:
            pnls.append(p)
            status = "✅" if p > 0 else "❌"
            print(f"  Seed {s}: {p:+.2f} bps {status}")
            
    if pnls:
        print("\n📈 AGGREGATE RESULTS")
        print("-" * 30)
        print(f"Mean PnL   : {np.mean(pnls):+.2f} bps")
        print(f"PnL Std    : {np.std(pnls):.2f}")
        print(f"Consistency: {sum(p > 0 for p in pnls)/len(pnls):.0%}")

if __name__ == "__main__":
    main()
