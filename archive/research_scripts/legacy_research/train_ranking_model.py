import re
import pandas as pd
import numpy as np

# Configuration
LOG_FILE = "scripts/training_snapshots_rich.log"
COST_BPS = 4.0 / 10000.0

def parse_snapshots(file_path):
    print(f"📂 Parsing {file_path}...")
    data = []
    flt = r"([-+]?\d*\.?\d+(?:[eE][-+]?\d+)?)"
    pattern = re.compile(
        fr"\[SNAPSHOT\] sym=([^\s]+) eos={flt} trap={flt} exh={flt} frag={flt} mom={flt} trap_d={flt} vol_r={flt} reg=([^\s]+) r1={flt} r5={flt} r10={flt} r20={flt}"
    )
    
    with open(file_path, "r") as f:
        for line in f:
            m = pattern.search(line)
            if m:
                data.append({
                    "trap": float(m.group(3)),
                    "exh": float(m.group(4)),
                    "frag": float(m.group(5)),
                    "mom": float(m.group(6)),
                    "trap_d": float(m.group(7)),
                    "vol_r": float(m.group(8)),
                    "r20": float(m.group(13))
                })
    return pd.DataFrame(data)

def main():
    df = parse_snapshots(LOG_FILE)
    if df.empty:
        print("❌ No snapshots found.")
        return

    df['r20_net'] = df['r20'] - COST_BPS
    
    # NON-LINEAR INTERACTION FEATURES
    df['trap_exh'] = df['trap'] * df['exh']
    df['trap_mom'] = df['trap'] * df['mom']
    df['mom_vol'] = df['mom'] * df['vol_r']
    df['trap_sq'] = df['trap'] ** 2
    
    # HEURISTIC SCORING GRID SEARCH (Simulated Model)
    # We want to find weights that maximize the Top 1% PnL
    best_pnl = -999
    best_weights = {}
    
    print("🔍 Grid Searching for Alpha Pocket...")
    for w_trap_exh in [0, 1, 2]:
        for w_trap_d in [0, 1, 2]:
            for w_mom_vol in [-1, 0, 1]:
                df['score'] = (df['trap_exh'] * w_trap_exh) + (df['trap_d'] * w_trap_d) + (df['mom_vol'] * w_mom_vol)
                top_1_pct_count = int(len(df) * 0.01)
                if top_1_pct_count < 20: continue
                
                mean_pnl = df.sort_values(by='score', ascending=False).head(top_1_pct_count)['r20_net'].mean()
                if mean_pnl > best_pnl:
                    best_pnl = mean_pnl
                    best_weights = {'trap_exh': w_trap_exh, 'trap_d': w_trap_d, 'mom_vol': w_mom_vol}

    print(f"🎯 Best Scorer Found: {best_weights} -> Top 1% PnL: {best_pnl*10000:+.2f} bps")
    
    # Run Selectivity Sweep with Best Weights
    df['score'] = (df['trap_exh'] * best_weights['trap_exh']) + \
                  (df['trap_d'] * best_weights['trap_d']) + \
                  (df['mom_vol'] * best_weights['mom_vol'])
                  
    results_df = df.sort_values(by='score', ascending=False)
    
    print("\n📈 RICH SELECTIVITY SWEEP (Nonlinear Ranking)")
    print(f"{'Selectivity':15} | {'Trades':8} | {'Mean Net PnL (bps)':20} | {'Sharpe'}")
    print("-" * 65)
    
    for top_pct in [100, 50, 20, 10, 5, 2, 1, 0.5, 0.2, 0.1]:
        count = int(len(results_df) * top_pct / 100)
        if count < 10: continue
        
        top_slice = results_df.head(count)
        mean_pnl = top_slice['r20_net'].mean()
        std_pnl = top_slice['r20_net'].std()
        sharpe = mean_pnl / (std_pnl + 1e-9)
        
        status = "✅ EDGE" if mean_pnl > 0 else "❌ NO EDGE"
        print(f"Top {top_pct:6.1f}%      | {count:8} | {mean_pnl*10000:+.2f} bps ({status:8}) | {sharpe:.2f}")

if __name__ == "__main__":
    main()
