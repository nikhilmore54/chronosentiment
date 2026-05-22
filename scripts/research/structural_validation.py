import subprocess
import re
import pandas as pd
import numpy as np

# Configuration
SEEDS = [80, 81, 82, 83, 84]
DURATION = 200

def run_structural_test(seed):
    cmd = f"python3 scripts/mock_streamer.py --duration {DURATION} --seed {seed} | TRAP_POLICY=6 ./target/debug/examples/live_engine"
    print(f"🚀 Testing Structural Sniper (Seed {seed})...")
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
    
    m = re.search(r"avg_pnl=([-+]?\d*\.\d+)", result.stderr)
    if m:
        pnl = float(m.group(1)) * 10000
        return pnl
    return None

def main():
    print("🧪 STRUCTURAL SNIPER VALIDATION (Persistence + Intensity + Delay)")
    print("-" * 65)
    
    pnls = []
    for s in SEEDS:
        p = run_structural_test(s)
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
        
        if np.mean(pnls) > 0.5:
             print("\n🏆 ALPHA DISCOVERED! (Monetizable structural edge)")
        else:
             print("\n❌ NO MONETIZABLE ALPHA (Execution costs still dominant)")

if __name__ == "__main__":
    main()
