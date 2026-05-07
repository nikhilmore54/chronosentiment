import re
import pandas as pd
import numpy as np

# Configuration
LOG_FILE = "scripts/microstructure_research.log"

def parse_snapshots(file_path):
    print(f"📂 Parsing {file_path} for Hot-Zone Time-Scale...")
    data = []
    flt = r"([-+]?\d*\.?\d+(?:[eE][-+]?\d+)?)"
    pattern = re.compile(
        fr"\[SNAPSHOT\] sym=([^\s]+) eos={flt} trap={flt} exh={flt} frag={flt} mom={flt} trap_d={flt} vol_r={flt} vs={flt} vl={flt} mp={flt} reg=([^\s]+) r1={flt} r5={flt} r10={flt} r20={flt}"
    )
    
    with open(file_path, "r") as f:
        for line in f:
            m = pattern.search(line)
            if m:
                vs, vl = float(m.group(9)), float(m.group(10))
                if vs > 0 and vl > 0:
                    data.append({
                        "sym": m.group(1),
                        "trap": float(m.group(3)),
                        "mom_p": float(m.group(11)),
                        "vol_ratio": vs / vl,
                        "regime": m.group(12),
                        "r1": float(m.group(13)),
                        "r5": float(m.group(14)),
                        "r10": float(m.group(15)),
                        "r20": float(m.group(16))
                    })
    return pd.DataFrame(data)

def main():
    df = parse_snapshots(LOG_FILE)
    if df.empty:
        print("❌ No data found.")
        return

    df['signal'] = df['trap'] * df['mom_p']
    
    # FILTER: HOT ZONE ONLY
    df = df[(df['vol_ratio'] < 0.8) & (df['mom_p'] > 0.6) & (df['regime'] == 'RANGE')]
    
    print(f"\n🔬 Analyzing HOT-ZONE Persistence vs Horizon (N={len(df)})...")
    
    results = []
    horizons = ['r1', 'r5', 'r10', 'r20']
    
    for sym, group in df.groupby('sym'):
        if len(group) < 20: continue
        
        for window in [1, 3, 5, 10]:
            # PERSISTENCE: Rolling intensity count (Threshold 1.5 for high intensity)
            intensity = (group['signal'].abs() > 1.5).rolling(window).sum()
            
            for h in horizons:
                # We shift by 3 (The Delay)
                target = group[h].shift(-3)
                corr = intensity.corr(target)
                if not np.isnan(corr):
                    results.append({'window': window, 'horizon': h, 'corr': corr})

    if not results:
        print("❌ No results in hot zone.")
        return

    res_df = pd.DataFrame(results)
    heatmap = res_df.groupby(['window', 'horizon'])['corr'].mean().unstack()
    
    print("\n🔥 HOT-ZONE PERSISTENCE HEATMAP (Correlation at Delay=3)")
    print("-" * 60)
    print(heatmap)
    
    best_pair = res_df.groupby(['window', 'horizon'])['corr'].mean().idxmax()
    best_corr = res_df.groupby(['window', 'horizon'])['corr'].mean().max()
    
    print(f"\n🏆 HOT-ZONE OPTIMAL PAIRING")
    print(f"   Persistence Window: {best_pair[0]} ticks")
    print(f"   Target Horizon    : {best_pair[1]}")
    print(f"   Mean Correlation  : {best_corr:+.4f} (at Delay=3)")

if __name__ == "__main__":
    main()
