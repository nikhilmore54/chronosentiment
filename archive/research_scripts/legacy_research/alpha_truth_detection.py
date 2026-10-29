import re
import pandas as pd
import numpy as np

# Configuration
LOG_FILE = "scripts/microstructure_research.log"

def parse_snapshots(file_path):
    print(f"📂 Parsing {file_path} for Truth Detection...")
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
    
    # FILTER: Target Regime (Low Vol Range + Bull Persistence)
    # We test the signal specifically in the hot zone we found
    subset = df[(df['vol_ratio'] < 0.8) & (df['mom_p'] > 0.6) & (df['regime'] == 'RANGE')].copy()
    
    if len(subset) < 100:
        print(f"⚠️ Subset too small (N={len(subset)}). Using global for testing.")
        subset = df.copy()

    print(f"\n🧪 ALPHA TRUTH DETECTION (N={len(subset)})")
    print("-" * 60)
    
    # 1. HORIZON SWEEP (At Shift=0)
    print("\n[HORIZON SWEEP] Does the signal peak early?")
    horizons = ['r1', 'r5', 'r10', 'r20']
    for h in horizons:
        corr = subset['signal'].corr(subset[h])
        print(f"  Horizon {h:3}: {corr:+.4f}")
        
    # 2. TIME-SHIFT ATTACK
    print("\n[TIME-SHIFT ATTACK] Is it a leading or lagging signal?")
    print(f"{'Shift':10} | {'Corr (r1)':10} | {'Corr (r20)':10}")
    print("-" * 40)
    
    # Group by symbol to avoid cross-symbol shifting
    for shift in [0, 1, 3, 5, 10]:
        corrs_r1 = []
        corrs_r20 = []
        for sym, group in subset.groupby('sym'):
            if len(group) <= shift: continue
            # Shift features BACK (asking if PAST features predict CURRENT outcome)
            s_signal = group['signal'].shift(shift)
            c1 = s_signal.corr(group['r1'])
            c20 = s_signal.corr(group['r20'])
            if not np.isnan(c1): corrs_r1.append(c1)
            if not np.isnan(c20): corrs_r20.append(c20)
            
        mean_c1 = np.mean(corrs_r1) if corrs_r1 else 0
        mean_c20 = np.mean(corrs_r20) if corrs_r20 else 0
        print(f"{shift:2} ticks  | {mean_c1:+.4f}    | {mean_c20:+.4f}")

    print("\n🎯 INTERPRETATION:")
    print("If Shift 5-10 remains high (> 0.1) -> You have a LEADING edge.")
    print("If Shift 1-3 collapses -> You have a coincident artifact.")

if __name__ == "__main__":
    main()
