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
                # Filter out uninitialized rows (vs/vl == 0 or r20 == 0)
                vs = float(m.group(9))
                vl = float(m.group(10))
                r20 = float(m.group(16))
                if vs > 0 and vl > 0 and abs(r20) > 1e-9:
                    data.append({
                        "trap": float(m.group(3)),
                        "exh": float(m.group(4)),
                        "trap_d": float(m.group(7)),
                        "mom": float(m.group(6)),
                        "vol_s": vs,
                        "vol_l": vl,
                        "mom_p": float(m.group(11)),
                        "r20": r20
                    })
    return pd.DataFrame(data)

def main():
    df = parse_snapshots(LOG_FILE)
    if df.empty:
        print("❌ No valid initialized data found. Window might be too short.")
        return

    df['r20_net'] = df['r20'] - COST_BPS
    
    # NEW HYPOTHESIS TESTING
    df['vol_burst'] = df['vol_s'] / df['vol_l']
    df['trap_vol'] = df['trap'] * df['vol_burst']
    df['trap_mom_p'] = df['trap'] * df['mom_p']
    df['surge_vol'] = df['trap_d'] * df['vol_burst']
    
    print(f"\n📊 DATASET: {len(df)} rows after window initialization.")
    print("\n🔍 HYPOTHESIS CORRELATION TABLE")
    print("-" * 50)
    hypotheses = ['trap', 'trap_d', 'vol_burst', 'mom_p', 'trap_vol', 'trap_mom_p', 'surge_vol']
    results = []
    for h in hypotheses:
        c = df[h].corr(df['r20_net'])
        results.append((h, c))
        print(f"{h:15}: {c:+.4f}")
        
    # Best Interaction Analysis
    best_h = hypotheses[np.nanargmax([abs(r[1]) if not np.isnan(r[1]) else -1 for r in results])]
    print(f"\n🏆 BEST CANDIDATE: {best_h}")
    
    # Decile Monotonicity for Best Candidate
    df['decile'] = pd.qcut(df[best_h], 10, labels=False, duplicates='drop')
    decile_stats = df.groupby('decile')['r20_net'].mean() * 10000
    
    print(f"\n📈 MONOTONICITY AUDIT ({best_h})")
    print("-" * 40)
    for decile, pnl in decile_stats.items():
        status = "✅" if pnl > 0 else "❌"
        print(f"Decile {decile:2}: {pnl:+.2f} bps {status}")

if __name__ == "__main__":
    main()
