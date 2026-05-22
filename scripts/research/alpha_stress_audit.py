import subprocess
import re
import pandas as pd
import numpy as np
import os

# Configuration
SEEDS = [80, 81, 82, 83, 84, 85, 86, 87, 88, 89]
DURATION = 150
COST_BPS = 4.0 / 10000.0

def run_simulation(seed):
    cmd = f"python3 scripts/mock_streamer.py --duration {DURATION} --seed {seed} | TRAP_POLICY=0 ./target/debug/examples/live_engine"
    print(f"🚀 Running Seed {seed}...")
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
    
    # Extract SNAPSHOT lines
    data = []
    flt = r"([-+]?\d*\.?\d+(?:[eE][-+]?\d+)?)"
    pattern = re.compile(
        fr"\[SNAPSHOT\] sym=([^\s]+) eos={flt} trap={flt} exh={flt} frag={flt} mom={flt} trap_d={flt} vol_r={flt} vs={flt} vl={flt} mp={flt} reg=([^\s]+) r1={flt} r5={flt} r10={flt} r20={flt}"
    )
    
    for line in result.stdout.split('\n'):
        m = pattern.search(line)
        if m:
            vs, vl, r20 = float(m.group(9)), float(m.group(10)), float(m.group(16))
            if vs > 0 and vl > 0 and abs(r20) > 1e-9:
                data.append({
                    "trap": float(m.group(3)),
                    "mom_p": float(m.group(11)),
                    "vol_r": vs / vl,
                    "regime": m.group(12),
                    "r20_net": r20 - COST_BPS
                })
    return pd.DataFrame(data)

def main():
    print("🧪 MULTI-SEED STABILITY AUDIT (Persistence Divergence)")
    print("-" * 60)
    
    all_results = []
    for s in SEEDS:
        df = run_simulation(s)
        if df.empty: continue
        
        # Define the Pocket
        df['signal'] = df['trap'] * df['mom_p']
        subset = df[(df['vol_r'] < 0.8) & (df['mom_p'] > 0.6) & (df['regime'] == 'RANGE')]
        
        n_total = len(df)
        n_pocket = len(subset)
        coverage = n_pocket / n_total if n_total > 0 else 0
        
        if n_pocket >= 20:
            corr = subset['signal'].corr(subset['r20_net'])
            pnl = subset['r20_net'].mean() * 10000
            all_results.append({'seed': s, 'corr': corr, 'pnl': pnl, 'coverage': coverage, 'n': n_pocket})
            print(f"  Seed {s}: Corr={corr:+.4f}, PnL={pnl:+.2f} bps, Cov={coverage:.1%}, N={n_pocket}")
        else:
            print(f"  Seed {s}: Insufficient samples (N={n_pocket})")

    if all_results:
        res_df = pd.DataFrame(all_results)
        print("\n📈 SUMMARY STATS")
        print("-" * 30)
        print(f"Mean Correlation: {res_df['corr'].mean():+.4f}")
        print(f"Std Correlation : {res_df['corr'].std():.4f}")
        print(f"Mean Net PnL    : {res_df['pnl'].mean():+.2f} bps")
        print(f"Mean Coverage   : {res_df['coverage'].mean():.1%}")
        
        consistency = (res_df['pnl'] > 0).mean()
        print(f"PnL Consistency : {consistency:.0%}")
        
        if consistency >= 0.8 and res_df['pnl'].mean() > 2.0:
            print("\n✅ ALPHA VALIDATED (Robust across seeds)")
        else:
            print("\n❌ ALPHA FAILED (Unstable across seeds)")

if __name__ == "__main__":
    main()
