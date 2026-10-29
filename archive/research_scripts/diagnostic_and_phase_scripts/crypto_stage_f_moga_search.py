import os
import pandas as pd
import numpy as np

DATA_DIR = os.path.join(os.path.dirname(__file__), '../datasets/e9/crypto')

def get_conditions(series, name, quantiles):
    thresholds = np.unique(np.quantile(series, quantiles))
    conditions = []
    
    # Depth 1 conditions
    for t in thresholds:
        conditions.append({
            'expr': f"{name} > {t:.4f}",
            'eval': lambda df, t=t, n=name: df[n] > t,
            'depth': 1
        })
        conditions.append({
            'expr': f"{name} < {t:.4f}",
            'eval': lambda df, t=t, n=name: df[n] < t,
            'depth': 1
        })
        
    for i in range(len(thresholds)):
        for j in range(i+1, len(thresholds)):
            t1 = thresholds[i]
            t2 = thresholds[j]
            conditions.append({
                'expr': f"({name} >= {t1:.4f}) & ({name} <= {t2:.4f})",
                'eval': lambda df, t1=t1, t2=t2, n=name: (df[n] >= t1) & (df[n] <= t2),
                'depth': 1
            })
            
    return conditions

def check_stability(df, condition_mask, target_col):
    # Split discovery set into 4 chronological blocks
    chunk_size = len(df) // 4
    enrichments = []
    
    baseline_p = df[target_col].mean()
    if baseline_p == 0:
        return False, 0
        
    for i in range(4):
        start = i * chunk_size
        end = (i + 1) * chunk_size if i < 3 else len(df)
        b = df.iloc[start:end]
        b_mask = condition_mask.iloc[start:end]
        
        b_n = b_mask.sum()
        if b_n < 10:
            enrichments.append(0)
            continue
            
        b_pb = b.loc[b_mask, target_col].mean()
        b_baseline = b[target_col].mean()
        
        if b_baseline > 0:
            enrichments.append(b_pb / b_baseline)
        else:
            enrichments.append(0)
            
    # Stable if median enrichment across blocks > 1.0, and positive enrichment in at least 3/4 blocks
    med_e = np.median(enrichments)
    pos_blocks = sum(1 for e in enrichments if e > 1.0)
    
    return (med_e > 1.0 and pos_blocks >= 3), med_e

def run_moga_search():
    discovery_path = os.path.join(DATA_DIR, "moga_discovery.csv")
    validation_path = os.path.join(DATA_DIR, "moga_validation.csv")
    
    print("Loading datasets...")
    disc = pd.read_csv(discovery_path)
    val = pd.read_csv(validation_path)
    
    print(f"Discovery: {len(disc)} rows | Validation: {len(val)} rows")
    
    disc_pb_prev = disc['Persistent_Build'].mean()
    val_pb_prev = val['Persistent_Build'].mean()
    
    print(f"Baseline PB Prevalence -> Discovery: {disc_pb_prev:.2%} | Validation: {val_pb_prev:.2%}")
    
    # Define search space
    q = np.linspace(0.1, 0.9, 9)
    va_conds = get_conditions(disc['volume_acceleration_60m'], 'volume_acceleration_60m', q)
    pers_conds = get_conditions(disc['persistence_240m'], 'persistence_240m', q)
    
    all_conds = []
    all_conds.extend(va_conds)
    all_conds.extend(pers_conds)
    
    # Depth 2: AND combinations
    for vc in va_conds:
        for pc in pers_conds:
            all_conds.append({
                'expr': f"({vc['expr']}) & ({pc['expr']})",
                'eval': lambda df, v=vc, p=pc: v['eval'](df) & p['eval'](df),
                'depth': 2
            })
            
    print(f"Total candidate space: {len(all_conds)} conditions")
    
    candidates = []
    
    for i, cond in enumerate(all_conds):
        mask = cond['eval'](disc)
        n = mask.sum()
        
        if n < 100:
            continue
            
        p_c = disc.loc[mask, 'Persistent_Build'].mean()
        enrichment = p_c / disc_pb_prev if disc_pb_prev > 0 else 0
        
        if enrichment <= 1.0:
            continue
            
        is_stable, med_block_e = check_stability(disc, mask, 'Persistent_Build')
        if not is_stable:
            continue
            
        # Complexity penalty calculation (Optional, but we sort by enrichment and then penalize deep trees)
        # We can subtract a small penalty for depth 2 to prefer simpler rules
        score = enrichment - (0.05 if cond['depth'] == 2 else 0.0)
        
        candidates.append({
            'expr': cond['expr'],
            'depth': cond['depth'],
            'n': n,
            'coverage': n / len(disc),
            'p_pb_c': p_c,
            'enrichment': enrichment,
            'med_block_e': med_block_e,
            'score': score,
            'eval_func': cond['eval']
        })
        
    if not candidates:
        print("No candidates passed Discovery gates.")
        return
        
    candidates.sort(key=lambda x: x['score'], reverse=True)
    best = candidates[0]
    
    print("\nBest Candidate in Discovery:")
    print(f"Expr: {best['expr']}")
    print(f"N: {best['n']} ({best['coverage']:.2%} coverage)")
    print(f"Enrichment: {best['enrichment']:.2f}x")
    
    # Validation Gate
    v_mask = best['eval_func'](val)
    v_n = v_mask.sum()
    
    v_p_c = val.loc[v_mask, 'Persistent_Build'].mean() if v_n > 0 else 0
    v_enrichment = v_p_c / val_pb_prev if val_pb_prev > 0 else 0
    
    pass_val = (v_n >= 50) and (v_enrichment > 1.0) and (v_enrichment >= 0.80 * best['enrichment'])
    
    print("\nValidation Results:")
    print(f"N: {v_n}")
    print(f"Enrichment: {v_enrichment:.2f}x")
    print(f"Passed Gate: {pass_val}")
    
    report_path = os.path.join(os.path.dirname(__file__), '../moga_v0_1_report.md')
    with open(report_path, 'w') as f:
        f.write("# Stage F-MOGA v0.1 Results\n\n")
        f.write("Evolutionary search for a compact state-conditioned description of the `Persistent_Build` phenotype.\n\n")
        
        f.write("## Discovered Candidate Rule\n")
        f.write(f"```text\n{best['expr']}\n```\n\n")
        
        f.write("## Performance Matrix\n")
        f.write("| Metric | Discovery | Validation |\n")
        f.write("|---|---|---|\n")
        f.write(f"| Candidate support N | {best['n']} | {v_n} |\n")
        f.write(f"| Candidate coverage % | {best['coverage']:.2%} | {v_n/len(val) if len(val)>0 else 0:.2%} |\n")
        f.write(f"| PB prevalence (Baseline) | {disc_pb_prev:.2%} | {val_pb_prev:.2%} |\n")
        f.write(f"| P(PB \\| C) | {best['p_pb_c']:.2%} | {v_p_c:.2%} |\n")
        f.write(f"| Enrichment | {best['enrichment']:.2f}x | {v_enrichment:.2f}x |\n")
        f.write(f"| Block enrichment median | {best['med_block_e']:.2f}x | N/A |\n")
        f.write(f"| Block sign/stability | PASS | N/A |\n")
        f.write(f"| Tree depth | {best['depth']} | {best['depth']} |\n\n")
        
        f.write("## Validation Gate Assessment\n")
        f.write("- **Requirement**: Validation N >= 50\n")
        f.write(f"- **Actual**: {v_n}\n")
        f.write("- **Requirement**: Validation Enrichment >= 0.80 × Discovery Enrichment\n")
        f.write(f"- **Actual**: {v_enrichment:.2f}x vs target {(0.80 * best['enrichment']):.2f}x\n\n")
        
        if pass_val:
            f.write("**Status**: PASS. The discovered behavioral state rule survived chronological validation.")
        else:
            f.write("**Status**: FAIL. The rule failed to survive unseen validation data.")

if __name__ == "__main__":
    run_moga_search()
