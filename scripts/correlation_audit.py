import re
import pandas as pd
import numpy as np

# Configuration
LOG_FILE = "scripts/training_snapshots_unbiased.log"
COST_BPS = 4.0 / 10000.0

def main():
    data = []
    flt = r"([-+]?\d*\.?\d+(?:[eE][-+]?\d+)?)"
    pattern = re.compile(
        fr"\[SNAPSHOT\] sym=([^\s]+) eos={flt} trap={flt} exh={flt} frag={flt} mom={flt} trap_d={flt} vol_r={flt} reg=([^\s]+) r1={flt} r5={flt} r10={flt} r20={flt}"
    )
    
    with open(LOG_FILE, "r") as f:
        for line in f:
            m = pattern.search(line)
            if m:
                data.append({
                    "trap_d": float(m.group(7)),
                    "mom": float(m.group(6)),
                    "vol_r": float(m.group(8)),
                    "r20": float(m.group(13))
                })
    df = pd.DataFrame(data)
    if df.empty:
        print("❌ No data found.")
        return

    df['r20_net'] = df['r20'] - COST_BPS
    
    # Best Scorer: {'trap_exh': 0, 'trap_d': 2, 'mom_vol': -1}
    df['score'] = (df['trap_d'] * 2.0) - (df['mom'] * df['vol_r'])
    
    # Correlation Analysis
    corr = df['score'].corr(df['r20_net'])
    print(f"📊 Signal Correlation (Score vs r20_net): {corr:.4f}")
    
    # Decile Monotonicity
    df['decile'] = pd.qcut(df['score'], 10, labels=False, duplicates='drop')
    decile_stats = df.groupby('decile')['r20_net'].mean() * 10000
    
    print("\n📈 DECILE MONOTONICITY (Mean Net PnL in bps)")
    print("-" * 40)
    for decile, pnl in decile_stats.items():
        status = "✅" if pnl > 0 else "❌"
        print(f"Decile {decile:2}: {pnl:+.2f} bps {status}")

    # Top 1% Precision
    top_1_pct = df.sort_values(by='score', ascending=False).head(int(len(df)*0.01))
    print(f"\n🎯 TOP 1% SNIPER POCKET: {top_1_pct['r20_net'].mean()*10000:+.2f} bps (n={len(top_1_pct)})")

if __name__ == "__main__":
    main()
