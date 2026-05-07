import re
import pandas as pd
import numpy as np

# Configuration
LOG_FILE = "scripts/microstructure_research.log"
COST_BPS = 4.0 / 10000.0

def parse_snapshots(file_path):
    print(f"📂 Parsing {file_path}...")
    data = []
    flt = r"([-+]?\d*\.?\d+(?:[eE][-+]?\d+)?)"
    pattern = re.compile(
        fr"\[SNAPSHOT\] sym=([^\s]+) eos={flt} trap={flt} exh={flt} frag={flt} mom={flt} trap_d={flt} vol_r={flt} vs={flt} vl={flt} mp={flt} reg=([^\s]+) r1={flt} r5={flt} r10={flt} r20={flt}"
    )
    
    with open(file_path, "r") as f:
        for line in f:
            m = pattern.search(line)
            if m:
                vs = float(m.group(9))
                vl = float(m.group(10))
                r20 = float(m.group(16))
                if vs > 0 and vl > 0 and abs(r20) > 1e-9:
                    data.append({
                        "trap": float(m.group(3)),
                        "mom_p": float(m.group(11)),
                        "vol_s": vs,
                        "vol_l": vl,
                        "regime": m.group(12),
                        "r20": r20
                    })
    return pd.DataFrame(data)

def main():
    df = parse_snapshots(LOG_FILE)
    if df.empty:
        print("❌ No data found.")
        return

    df['r20_net'] = df['r20'] - COST_BPS
    df['vol_ratio'] = df['vol_s'] / df['vol_l']
    df['signal'] = df['trap'] * df['mom_p']
    
    # 1. GLOBAL SHUFFLE TEST
    orig_corr = df['signal'].corr(df['r20_net'])
    shuffled_y = pd.Series(np.random.permutation(df['r20_net'].values), index=df.index)
    shuff_corr = df['signal'].corr(shuffled_y)
    
    print("\n🎲 GLOBAL SHUFFLE TEST (Signal vs Outcome)")
    print("-" * 50)
    print(f"Original Correlation: {orig_corr:+.6f}")
    print(f"Shuffled Correlation: {shuff_corr:+.6f}")
    print(f"Confidence Delta: {abs(orig_corr) - abs(shuff_corr):+.6f}")
    
    # 2. STATE-SPACE AUDIT
    df['vol_slice'] = 'normal'
    df.loc[df['vol_ratio'] > 1.5, 'vol_slice'] = 'burst'
    df.loc[df['vol_ratio'] < 0.8, 'vol_slice'] = 'low'
    
    df['mp_slice'] = 'neutral'
    df.loc[df['mom_p'] > 0.6, 'mp_slice'] = 'bull_pers'
    df.loc[df['mom_p'] < -0.6, 'mp_slice'] = 'bear_pers'
    
    print("\n🔍 3D STATE-SPACE SLICING AUDIT")
    print("-" * 80)
    print(f"{'Vol Slice':10} | {'MP Slice':10} | {'Regime':10} | {'N':6} | {'Corr':8} | {'Exp (bps)'}")
    print("-" * 80)
    
    results = []
    for vs in ['burst', 'normal', 'low']:
        for mp in ['bull_pers', 'bear_pers', 'neutral']:
            for reg in ['TREND', 'RANGE', 'VOLATILE']:
                subset = df[(df['vol_slice'] == vs) & (df['mp_slice'] == mp) & (df['regime'] == reg)]
                n = len(subset)
                if n < 50: continue
                
                corr = subset['signal'].corr(subset['r20_net'])
                mean_pnl = subset['r20_net'].mean() * 10000
                
                results.append({'vs': vs, 'mp': mp, 'reg': reg, 'n': n, 'corr': corr, 'pnl': mean_pnl})
                indicator = "✅" if abs(corr) > 0.05 and mean_pnl > 0 else ""
                print(f"{vs:10} | {mp:10} | {reg:10} | {n:6} | {corr:+.4f} | {mean_pnl:+.2f} {indicator}")

    # 3. PEAK SLICE VALIDATION
    if results:
        best_slice = sorted(results, key=lambda x: abs(x['corr']) if not np.isnan(x['corr']) else -1, reverse=True)[0]
        print(f"\n🏆 PEAK SIGNAL SLICE: Vol={best_slice['vs']}, MP={best_slice['mp']}, Reg={best_slice['reg']}")
        
        peak_df = df[(df['vol_slice'] == best_slice['vs']) & (df['mp_slice'] == best_slice['mp']) & (df['regime'] == best_slice['reg'])].copy()
        peak_orig = peak_df['signal'].corr(peak_df['r20_net'])
        peak_shuff = peak_df['signal'].corr(pd.Series(np.random.permutation(peak_df['r20_net'].values), index=peak_df.index))
        
        print(f"Original Slice Corr: {peak_orig:+.6f}")
        print(f"Shuffled Slice Corr: {peak_shuff:+.6f}")
        
        # Monotonicity with Quantiles
        peak_df['bucket'] = pd.qcut(peak_df['signal'], 5, labels=False, duplicates='drop')
        monot = peak_df.groupby('bucket')['r20_net'].mean() * 10000
        
        print(f"\n📈 PEAK SLICE MONOTONICITY (5 Buckets)")
        print("-" * 30)
        for d, p in monot.items():
            status = "🟢" if p > 0 else "🔴"
            print(f"  Bucket {d}: {p:+.2f} bps {status}")

if __name__ == "__main__":
    main()
