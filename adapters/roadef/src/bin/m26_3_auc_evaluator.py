import pandas as pd
import numpy as np
from sklearn.metrics import roc_auc_score, precision_recall_curve, auc
from scipy.stats import spearmanr, kendalltau

def evaluate_df(df):
    # 1. Binary Prediction (Failure)
    y_true_failure = (~df['subtree_survived']).astype(int)
    y_scores = df['pressure_score']
    
    roc_auc = np.nan
    if len(y_true_failure.unique()) > 1:
        roc_auc = roc_auc_score(y_true_failure, y_scores)
        
    # 2. Ranking Correlations (Pressure -> Survival Depth)
    # Since higher pressure should lead to earlier failure (lower survival depth),
    # we expect a negative correlation.
    spearman_corr, _ = spearmanr(df['pressure_score'], df['survival_depth'])
    kendall_corr, _ = kendalltau(df['pressure_score'], df['survival_depth'])
    
    # 3. Champion Probability
    df = df.copy()
    df['pressure_bucket'] = pd.cut(df['pressure_score'], bins=np.arange(0, 1.1, 0.1), include_lowest=True)
    df['is_champion'] = df['became_champion'].astype(int)
    champ_stats = df.groupby('pressure_bucket', observed=False).agg(
        obs_count=('node_id', 'count'),
        champ_rate=('is_champion', 'mean'),
        champs=('is_champion', 'sum')
    ).dropna()
    
    # Find best bucket
    best_bucket = None
    max_rate = -1
    for idx, row in champ_stats.iterrows():
        if row['obs_count'] > 0 and row['champ_rate'] > max_rate:
            max_rate = row['champ_rate']
            best_bucket = str(idx)
            
    return roc_auc, spearman_corr, kendall_corr, best_bucket, champ_stats['champs'].sum()

def main():
    print("=== M26.3A.1 & M26.3A.2 Evaluator ===")
    
    try:
        df = pd.read_csv("m26_3_passive_logs.csv")
    except FileNotFoundError:
        print("Error: m26_3_passive_logs.csv not found.")
        return
        
    # Compute O(N) survival depth
    depths = df['search_depth'].values
    n_obs = len(df)
    survival_depths = np.zeros(n_obs, dtype=int)
    
    stack = [] # (index, search_depth, max_depth_seen)
    for i in range(n_obs):
        d_i = depths[i]
        while stack and stack[-1][1] >= d_i:
            idx, dep, max_d = stack.pop()
            survival_depths[idx] = max_d
            if stack:
                parent_idx, parent_dep, parent_max_d = stack[-1]
                stack[-1] = (parent_idx, parent_dep, max(parent_max_d, max_d))
        stack.append((i, d_i, d_i))
        
    while stack:
        idx, dep, max_d = stack.pop()
        survival_depths[idx] = max_d
        if stack:
            parent_idx, parent_dep, parent_max_d = stack[-1]
            stack[-1] = (parent_idx, parent_dep, max(parent_max_d, max_d))
            
    df['survival_depth'] = survival_depths
    
    print(f"Total Observations: {len(df)}")
    print(f"Unique Contexts: {df['context'].nunique()}")
    print(f"Total Champions: {df['became_champion'].sum()}")
    print(f"Total Failures:  {(~df['subtree_survived']).sum()}")
    
    # Overall Evaluation
    print("\n--- Layer 1: Overall Binary Prediction ---")
    roc_auc, spearman, kendall, best_bucket, total_champs = evaluate_df(df)
    print(f"Overall ROC-AUC (Pressure -> Failure): {roc_auc:.4f}")
    
    print("\n--- Layer 2: M26.3A.2 Context Stability Audit ---")
    print(f"{'Instance':<10} | {'ROC-AUC':<8} | {'Spearman':<9} | {'Kendall':<8} | {'Best Champ Bucket':<18} | {'Champs'}")
    print("-" * 75)
    
    all_bucket_orderings = []
    
    for instance_id in sorted(df['instance'].unique()):
        idf = df[df['instance'] == instance_id]
        i_auc, i_spearman, i_kendall, i_bucket, i_champs = evaluate_df(idf)
        auc_str = f"{i_auc:.4f}" if not np.isnan(i_auc) else "NaN"
        spearman_str = f"{i_spearman:.4f}" if not np.isnan(i_spearman) else "NaN"
        kendall_str = f"{i_kendall:.4f}" if not np.isnan(i_kendall) else "NaN"
        print(f"setA-{instance_id:02} | {auc_str:<8} | {spearman_str:<9} | {kendall_str:<8} | {str(i_bucket):<18} | {int(i_champs)}")
        
        # Track pressure ordering for this instance (Failure rate per bucket)
        idf = idf.copy()
        idf['pressure_bucket'] = pd.cut(idf['pressure_score'], bins=np.arange(0, 1.1, 0.1), include_lowest=True)
        idf_fail_rates = idf.groupby('pressure_bucket', observed=False)['subtree_survived'].apply(lambda x: 1.0 - x.mean()).dropna()
        ordering = idf_fail_rates.sort_values(ascending=False).index.astype(str).tolist()
        all_bucket_orderings.append((instance_id, ordering))

    print("\n--- Layer 2.5: Pressure Ordering Stability ---")
    for instance_id, ordering in all_bucket_orderings:
        print(f"setA-{instance_id:02}: {' > '.join(ordering[:5])} ...")
        
    # 3. Context Lifetime
    print("\n--- Layer 3: Context Lifetime Audit ---")
    lifetime_df = df.groupby('context').agg(
        first_seen=('node_id', 'min'),
        last_seen=('node_id', 'max'),
        observations=('node_id', 'count'),
        instances_seen=('instance', 'nunique')
    )
    lifetime_df['lifetime'] = lifetime_df['last_seen'] - lifetime_df['first_seen']
    
    print(f"Average Lifetime: {lifetime_df['lifetime'].mean():.2f}")
    print(f"Median Lifetime:  {lifetime_df['lifetime'].median():.2f}")
    print(f"95th Pct Lifetime:{lifetime_df['lifetime'].quantile(0.95):.2f}")
    
    # Top 5 longest-lived contexts
    print("\nTop 5 longest-lived contexts:")
    top_5 = lifetime_df.sort_values(['instances_seen', 'lifetime'], ascending=[False, False]).head(5)
    print(f"{'Context':<40} | {'Instances':<9} | {'Lifetime':<10} | {'Observations'}")
    print("-" * 75)
    for ctx, row in top_5.iterrows():
        print(f"{ctx:<40} | {int(row['instances_seen']):<9} | {int(row['lifetime']):<10} | {int(row['observations'])}")
        
    # 4. Champion Ecology Analysis (A.0C)
    print("\n--- Layer 4: A.0C Champion Ecology Analysis ---")
    champ_ecology = df.groupby('context').agg(
        observations=('node_id', 'count'),
        champions=('became_champion', 'sum')
    )
    champ_ecology['champion_rate'] = (champ_ecology['champions'] / champ_ecology['observations']) * 100.0
    
    # Filter to contexts with at least 100 observations to reduce noise
    valid_ecology = champ_ecology[champ_ecology['observations'] >= 100].sort_values('champion_rate', ascending=False)
    
    print(f"{'Context':<40} | {'Obs':<8} | {'Champs':<8} | {'Champion Rate'}")
    print("-" * 75)
    for ctx, row in valid_ecology.head(10).iterrows():
        print(f"{ctx:<40} | {int(row['observations']):<8} | {int(row['champions']):<8} | {row['champion_rate']:.3f}%")
        
    print("\n(Showing top 10 contexts by champion rate with >100 observations)")
        
    # 5. Champion Ranking Audit (A.0D)
    print("\n--- Layer 5: A.0D Champion Ranking Audit ---")
    # Note: df['became_champion'].sum() gets count of champions.
    global_champs = df['became_champion'].sum()
    if global_champs > 0 and len(df['became_champion'].unique()) > 1:
        champ_roc_auc = roc_auc_score(df['became_champion'], 1.0 - df['pressure_score'])
        precision, recall, _ = precision_recall_curve(df['became_champion'], 1.0 - df['pressure_score'])
        champ_pr_auc = auc(recall, precision)
        
        print(f"ROC-AUC (Pressure -> Champion): {champ_roc_auc:.4f}")
        print(f"PR-AUC  (Pressure -> Champion): {champ_pr_auc:.4f}")
        
        # Lift by Pressure Decile
        print("\nChampion Lift by Pressure Decile:")
        print(f"{'Decile':<8} | {'Range':<15} | {'Obs':<8} | {'Champs':<8} | {'Champ Rate'}")
        print("-" * 65)
        
        # Sort by pressure ascending (decile 1 = lowest pressure, safest)
        df_sorted = df.sort_values('pressure_score', ascending=True).reset_index(drop=True)
        n_rows = len(df_sorted)
        chunk_size = int(np.ceil(n_rows / 10.0))
        
        global_rate = df_sorted['became_champion'].mean()
        
        deciles = []
        for i in range(10):
            start_idx = i * chunk_size
            end_idx = min((i + 1) * chunk_size, n_rows)
            if start_idx >= n_rows:
                break
            d = df_sorted.iloc[start_idx:end_idx]
            deciles.append(d)
            
            obs = len(d)
            champs = d['became_champion'].sum()
            rate = champs / obs if obs > 0 else 0
            
            p_min = d['pressure_score'].min()
            p_max = d['pressure_score'].max()
            
            print(f"D{i+1:<7} | {p_min:.2f} - {p_max:.2f}   | {obs:<8} | {int(champs):<8} | {rate*100:.3f}%")
            
        print(f"\nGlobal Champion Rate: {global_rate*100:.3f}%")
        if len(deciles) > 0 and deciles[0]['became_champion'].mean() > 0:
            lift = deciles[0]['became_champion'].mean() / global_rate
            print(f"D1 Champion Lift: {lift:.2f}x")
    else:
        print("Not enough champions to compute ROC-AUC and Lift metrics.")
        
    # 6. Pressure Stratification Audit (A.0E)
    print("\n--- Layer 6: A.0E Pressure Stratification Audit ---")
    if global_champs > 0:
        top_contexts = valid_ecology.head(3).index
        for ctx in top_contexts:
            ctx_df = df[df['context'] == ctx].copy()
            if len(ctx_df['became_champion'].unique()) > 1 and ctx_df['pressure_score'].nunique() > 1:
                try:
                    ctx_df['pressure_tercile'] = pd.qcut(ctx_df['pressure_score'], 3, labels=['Low', 'Mid', 'High'], duplicates='drop')
                except ValueError:
                    ctx_df['pressure_tercile'] = pd.qcut(ctx_df['pressure_score'], 3, duplicates='drop')
                stratified = ctx_df.groupby('pressure_tercile', observed=False).agg(
                    obs=('node_id', 'count'),
                    champs=('became_champion', 'sum')
                )
                stratified['rate'] = (stratified['champs'] / stratified['obs']) * 100.0
                
                print(f"\nContext: {ctx}")
                print(f"{'Pressure Band':<15} | {'Obs':<8} | {'Champs':<8} | {'Rate'}")
                print("-" * 50)
                for band, row in stratified.iterrows():
                    print(f"{str(band):<15} | {int(row['obs']):<8} | {int(row['champs']):<8} | {row['rate']:.3f}%")

if __name__ == "__main__":
    main()
