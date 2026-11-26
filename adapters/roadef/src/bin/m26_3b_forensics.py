import pandas as pd
import numpy as np
from collections import defaultdict

def main():
    print("=== M26.3B Archive Forensics ===")
    
    try:
        df = pd.read_csv("m26_3_passive_logs.csv")
    except FileNotFoundError:
        print("Error: m26_3_passive_logs.csv not found.")
        return
        
    print(f"Loaded {len(df)} observations.")
    
    # 1. Assign global deciles (1-indexed: 1 = lowest pressure, 10 = highest)
    df['decile'] = pd.qcut(df['pressure_score'].rank(method='first'), 10, labels=False) + 1
    
    # 2. Phase 1: Descendant Elite Lift
    print("\n--- Phase 1: Descendant Elite Lift ---")
    d1_df = df[df['decile'] == 1]
    p_elite_d1 = d1_df['became_champion'].mean()
    p_elite_random = df['became_champion'].mean()
    lift = p_elite_d1 / p_elite_random if p_elite_random > 0 else 0.0
    
    print(f"P(EliteDescendant | D1):     {p_elite_d1*100:.4f}%")
    print(f"P(EliteDescendant | Random): {p_elite_random*100:.4f}%")
    print(f"Descendant Elite Lift:       {lift:.2f}x")
    
    # 3. Phase 2: Ancestor Decile Analysis
    print("\n--- Phase 2: Ancestor Decile Analysis ---")
    champs = df[df['became_champion']]
    print(f"Found {len(champs)} champion nodes. Extracting ancestors...")
    
    ancestor_decile_counts = defaultdict(int)
    total_ancestors = 0
    
    # Group by instance to avoid cross-instance parent tracking
    for instance_id in sorted(df['instance'].unique()):
        inst_df = df[df['instance'] == instance_id].reset_index(drop=True)
        # Find indices of champions in inst_df
        champ_indices = inst_df[inst_df['became_champion']].index.tolist()
        
        # Convert columns to numpy arrays for O(1) access
        search_depths = inst_df['search_depth'].values
        deciles = inst_df['decile'].values
        
        for idx in champ_indices:
            curr_depth = search_depths[idx]
            
            # Walk backwards to trace ancestors
            for j in range(idx - 1, -1, -1):
                if search_depths[j] == curr_depth - 1:
                    ancestor_decile_counts[deciles[j]] += 1
                    total_ancestors += 1
                    curr_depth -= 1
                    if curr_depth == 0:
                        break
                        
    print(f"Traced a total of {total_ancestors} ancestor nodes.")
    print("\nDistribution of Ancestor Deciles for Champion Nodes:")
    for d in range(1, 11):
        cnt = ancestor_decile_counts[d]
        pct = (cnt / total_ancestors * 100.0) if total_ancestors > 0 else 0.0
        print(f"  Ancestors from D{d:<2}: {cnt:>5} ({pct:.2f}%)")
        
    d1_3_sum = sum(ancestor_decile_counts[d] for d in [1, 2, 3])
    d1_3_pct = (d1_3_sum / total_ancestors * 100.0) if total_ancestors > 0 else 0.0
    print(f"Total Ancestors in D1-D3: {d1_3_pct:.2f}%")
    
    # 4. Phase 3: Context Persistence
    print("\n--- Phase 3: Context Persistence ---")
    lifetime_df = df.groupby('context').agg(
        instances_seen=('instance', 'nunique'),
        observations=('node_id', 'count')
    ).sort_values('instances_seen', ascending=False)
    
    print("Longest-lived contexts across instances:")
    for ctx, row in lifetime_df.head(5).iterrows():
        print(f"  {ctx:<45} -> {int(row['instances_seen'])}/7 instances ({int(row['observations'])} obs)")

if __name__ == "__main__":
    main()
