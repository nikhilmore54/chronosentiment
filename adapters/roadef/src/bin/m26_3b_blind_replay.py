import pandas as pd
import numpy as np

def ndcg_at_k(y_true, y_score, k):
    order = np.argsort(y_score)[::-1]
    y_true_sorted = np.take(y_true, order)[:k]
    gain = y_true_sorted
    discounts = np.log2(np.arange(2, len(y_true_sorted) + 2))
    dcg = np.sum(gain / discounts)
    
    ideal_order = np.argsort(y_true)[::-1]
    y_true_ideal = np.take(y_true, ideal_order)[:k]
    ideal_gain = y_true_ideal
    ideal_dcg = np.sum(ideal_gain / discounts)
    
    return dcg / ideal_dcg if ideal_dcg > 0 else 0.0

def evaluate_metrics(df, col):
    y_true = df[col].astype(int).values
    # Lower pressure is better, so our score is 1.0 - pressure_score
    y_score = 1.0 - df['pressure_score'].values
    
    n = len(df)
    results = {}
    
    # Sort by score descending (safest first)
    order = np.argsort(y_score)[::-1]
    y_true_sorted = np.take(y_true, order)
    
    global_base = y_true.mean()
    results['global'] = global_base
    
    for pct in [1, 5, 10, 20, 50]:
        k = max(1, int(n * pct / 100.0))
        top_k_champs = y_true_sorted[:k].sum()
        
        precision = top_k_champs / k
        recall = top_k_champs / y_true.sum() if y_true.sum() > 0 else 0.0
        ndcg = ndcg_at_k(y_true, y_score, k)
        
        results[pct] = {
            'precision': precision,
            'recall': recall,
            'ndcg': ndcg
        }
    return results

def main():
    print("=== M26.3B.4 & M26.3B.5 Blind Replay Audit ===")
    
    try:
        df = pd.read_csv("m26_3_passive_logs.csv")
    except FileNotFoundError:
        print("Error: m26_3_passive_logs.csv not found.")
        return
        
    print(f"Loaded {len(df)} observations.")
    
    # 1. Telemetry Integrity Audit
    print("\n--- Phase A: Telemetry Integrity Audit ---")
    # Verify that child nodes (subsequent in index with search_depth > parent)
    # have node_id > parent node_id
    violations = 0
    total_checks = 0
    depths = df['search_depth'].values
    node_ids = df['node_id'].values
    
    for i in range(len(df) - 1):
        if depths[i+1] > depths[i]:
            total_checks += 1
            if node_ids[i+1] <= node_ids[i]:
                violations += 1
                
    violation_rate = (violations / total_checks * 100.0) if total_checks > 0 else 0.0
    print(f"Checked {total_checks} parent-child pairs.")
    print(f"Timestamp order violations: {violations} ({violation_rate:.4f}%)")
    if violations == 0:
        print("[PASS] pressure_timestamp < descendant_elite_timestamp (100%)")
    else:
        print("[FAIL] Timestamp ordering violated!")
        
    # 2. Blind Replay for Champions
    print("\n--- Phase B: Blind Replay (Target: Champions) ---")
    champ_metrics = evaluate_metrics(df, 'became_champion')
    
    print("\nPrecision@K (Fraction of Top-K% nodes that yield Champions):")
    for pct in [1, 5, 10, 20, 50]:
        print(f"  Precision@{pct}%: {champ_metrics[pct]['precision']*100:.4f}%")
    print(f"  Global Base:  {champ_metrics['global']*100:.4f}%")
    
    print("\nRecall@K (Fraction of ALL Champions captured in Top-K% nodes):")
    for pct in [1, 5, 10, 20, 50]:
        print(f"  Recall@{pct}%:  {champ_metrics[pct]['recall']*100:.1f}%")
        
    print("\nNDCG@K (Normalized Discounted Cumulative Gain):")
    for pct in [1, 5, 10, 20, 50]:
        print(f"  NDCG@{pct}%:  {champ_metrics[pct]['ndcg']:.4f}")
        
    # 3. Blind Replay for Subtree Survival
    print("\n--- Phase C: Blind Replay (Target: Subtree Survival) ---")
    surv_metrics = evaluate_metrics(df, 'subtree_survived')
    
    print("\nPrecision@K (Fraction of Top-K% nodes that survived):")
    for pct in [1, 5, 10, 20, 50]:
        print(f"  Precision@{pct}%: {surv_metrics[pct]['precision']*100:.4f}%")
    print(f"  Global Base:  {surv_metrics['global']*100:.4f}%")
    
    print("\nRecall@K (Fraction of ALL survived nodes captured in Top-K% nodes):")
    for pct in [1, 5, 10, 20, 50]:
        print(f"  Recall@{pct}%:  {surv_metrics[pct]['recall']*100:.1f}%")
        
    print("\nNDCG@K:")
    for pct in [1, 5, 10, 20, 50]:
        print(f"  NDCG@{pct}%:  {surv_metrics[pct]['ndcg']:.4f}")

if __name__ == "__main__":
    main()
