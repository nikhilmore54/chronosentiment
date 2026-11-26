import json
import numpy as np
from sklearn.cluster import KMeans
from collections import defaultdict
import os

def load_data():
    with open('m9a_ultracrew_state_large.json') as f:
        return json.load(f)

def run_phase_1(data, report):
    report.append("## Phase 1 — Descriptive Statistics\n")
    last_imp_gens = []
    total_imps = []
    best_fitnesses = []
    avg_hist_novs = []
    avg_dists = []
    drought_lengths = []
    
    for run in data:
        history = run['history']
        if not history: continue
        
        improvements = [h for h in history if h['improvement_magnitude'] > 0]
        last_imp_gens.append(improvements[-1]['generation'] if improvements else 0)
        total_imps.append(len(improvements))
        best_fitnesses.append(history[-1]['best_fitness'])
        
        novs = [h['history_novelty'] for h in history]
        dists = [h['distance_to_incumbent_best'] for h in history]
        avg_hist_novs.append(np.mean(novs))
        avg_dists.append(np.mean(dists))
        
        # Droughts: max generations without improvement
        if improvements:
            gens = [0] + [h['generation'] for h in improvements] + [history[-1]['generation']]
            droughts = [gens[i] - gens[i-1] for i in range(1, len(gens))]
            drought_lengths.append(np.max(droughts))

    report.append(f"- **Total Runs Analyzed**: {len(data)}")
    report.append(f"- **Last Improvement Gen**: Mean {np.mean(last_imp_gens):.1f}, Median {np.median(last_imp_gens):.1f}, Max {np.max(last_imp_gens)}")
    report.append(f"- **Total Improvements per Run**: Mean {np.mean(total_imps):.1f}, Median {np.median(total_imps):.1f}")
    report.append(f"- **Best Fitness Achieved**: Mean {np.mean(best_fitnesses):.1f}, Min {np.min(best_fitnesses)}")
    report.append(f"- **Average History Novelty**: Mean {np.mean(avg_hist_novs):.4f}")
    report.append(f"- **Average Distance to Incumbent**: Mean {np.mean(avg_dists):.1f}")
    report.append(f"- **Max Improvement Drought Length**: Mean {np.mean(drought_lengths):.1f}, Max {np.max(drought_lengths)}")
    report.append("")


def run_phase_2(data, report):
    report.append("## Phase 2 — Search Archetypes (Clustering)\n")
    # We will build feature vectors representing trajectory shapes (subsampled to 100 points)
    X = []
    valid_runs = []
    
    for run in data:
        history = run['history']
        if not history: continue
        
        # Subsample to 100 points
        indices = np.linspace(0, len(history)-1, 100, dtype=int)
        
        # Extract shapes
        nov_shape = [history[i]['history_novelty'] for i in indices]
        dist_shape = [history[i]['distance_to_incumbent_best'] for i in indices]
        acc_shape = [history[i]['acceptance_rate'] for i in indices]
        
        # Normalize shapes
        nov_norm = (nov_shape - np.mean(nov_shape)) / (np.std(nov_shape) + 1e-9)
        dist_norm = (dist_shape - np.mean(dist_shape)) / (np.std(dist_shape) + 1e-9)
        acc_norm = (acc_shape - np.mean(acc_shape)) / (np.std(acc_shape) + 1e-9)
        
        vector = np.concatenate([nov_norm, dist_norm, acc_norm])
        X.append(vector)
        valid_runs.append(run)
        
    X = np.array(X)
    n_clusters = 4
    kmeans = KMeans(n_clusters=n_clusters, random_state=42).fit(X)
    
    clusters = defaultdict(list)
    for i, label in enumerate(kmeans.labels_):
        clusters[label].append(valid_runs[i])
        
    for c_id, runs_in_cluster in clusters.items():
        report.append(f"### Cluster {c_id} (N={len(runs_in_cluster)})")
        best_fits = [r['history'][-1]['best_fitness'] for r in runs_in_cluster]
        report.append(f"- **Exemplar Trajectory**: Instance `{runs_in_cluster[0]['instance']}`, Seed `{runs_in_cluster[0]['seed']}`")
        report.append(f"- **Average Best Fitness**: {np.mean(best_fits):.1f}")
        report.append(f"- **Outcome Insight**: Does this shape produce better outcomes? (Min: {np.min(best_fits)}, Max: {np.max(best_fits)})")
        report.append("")

def run_phase_3(data, report):
    report.append("## Phase 3 — Transition Analysis\n")
    
    # 1. Gather all improvements and define 'breakthrough' threshold
    all_imps = []
    for run in data:
        history = run['history']
        imps = [h['improvement_magnitude'] for h in history if h['improvement_magnitude'] > 0]
        all_imps.extend(imps)
        
    breakthrough_threshold = np.percentile(all_imps, 90)
    report.append(f"**Breakthrough Threshold**: {breakthrough_threshold:.1f} (Top 10% magnitude)")
    
    # Dual window lists
    windows = {'improvement': {'t50': [], 't500': []}, 'breakthrough': {'t50': [], 't500': []}, 'random': {'t50': [], 't500': []}}
    
    for run in data:
        history = run['history']
        if len(history) < 500: continue
        
        # Random non-improving points
        non_imps = [i for i, h in enumerate(history) if h['improvement_magnitude'] == 0 and i >= 500]
        np.random.seed(run['seed'])
        sampled_non_imps = np.random.choice(non_imps, min(10, len(non_imps)), replace=False)
        
        for idx in sampled_non_imps:
            windows['random']['t50'].append(history[idx-50])
            windows['random']['t500'].append(history[idx-500])
            
        for i, h in enumerate(history):
            if h['improvement_magnitude'] > 0 and i >= 500:
                cat = 'breakthrough' if h['improvement_magnitude'] >= breakthrough_threshold else 'improvement'
                windows[cat]['t50'].append(history[i-50])
                windows[cat]['t500'].append(history[i-500])
                
    report.append("\n### Precursor Distribution Test")
    metrics = ['history_novelty', 'distance_to_incumbent_best', 'acceptance_rate']
    
    for cat in ['breakthrough', 'improvement', 'random']:
        report.append(f"**Category: {cat.upper()} (N={len(windows[cat]['t50'])})**")
        for m in metrics:
            t50_val = np.mean([w[m] for w in windows[cat]['t50']])
            t500_val = np.mean([w[m] for w in windows[cat]['t500']])
            report.append(f"- {m}: T-500 = {t500_val:.4f} -> T-50 = {t50_val:.4f}")
        report.append("")


def run_phase_4(data, report):
    report.append("## Phase 4 — Regime Analysis & Opportunity Table\n")
    
    # Track regime changes
    WINDOW = 200
    opportunities = {'total': 0, 'with_improvement': 0, 'without_improvement': 0}
    
    for run in data:
        history = run['history']
        current_op = None
        for i, h in enumerate(history):
            if h['dominant_operator'] != current_op:
                if current_op is not None:
                    # Regime shift occurred
                    opportunities['total'] += 1
                    
                    # Check window
                    window = history[i:i+WINDOW]
                    has_imp = any(w['improvement_magnitude'] > 0 for w in window)
                    if has_imp:
                        opportunities['with_improvement'] += 1
                    else:
                        opportunities['without_improvement'] += 1
                        
                current_op = h['dominant_operator']
                
    report.append(f"**Total Regime Transitions Identified**: {opportunities['total']}")
    if opportunities['total'] > 0:
        imp_rate = opportunities['with_improvement'] / opportunities['total'] * 100
        report.append(f"- **Transitions yielding Improvement (within {WINDOW} gens)**: {opportunities['with_improvement']} ({imp_rate:.1f}%)")
        report.append(f"- **Transitions yielding NO Improvement (Survivorship Bias check)**: {opportunities['without_improvement']} ({100 - imp_rate:.1f}%)")
    
    report.append("\n## Ledger Evaluation (H1-H7)\n")
    report.append("> **H1**: Improvements cluster near structural novelty spikes. **(TBD based on Phase 3)**")
    report.append("> **H2**: Improvements cluster near operator regime transitions. **(TBD based on Phase 4)**")
    report.append("> **H3**: Breakthroughs arise from different dynamics than ordinary improvements. **(TBD based on Phase 3)**")
    report.append("> **H4**: Acceptance-rate decay predicts breakthrough density. **(TBD based on Phase 3)**")
    report.append("> **H5**: Independent trajectories form recurring archetypal shapes. **(TBD based on Phase 2)**")
    report.append("> **H6**: Distance-to-incumbent exhibits characteristic behavior prior to breakthroughs. **(TBD based on Phase 3)**")
    report.append("> **H7**: Breakthrough probability depends more on search state than absolute generation. **(Supported if precursors are distinct from random baseline)**")
    
def main():
    print("Loading data...")
    data = load_data()
    print(f"Loaded {len(data)} trajectories.")
    
    report = ["# M9A.1 Analysis Results\n"]
    
    print("Running Phase 1...")
    run_phase_1(data, report)
    
    print("Running Phase 2...")
    run_phase_2(data, report)
    
    print("Running Phase 3...")
    run_phase_3(data, report)
    
    print("Running Phase 4...")
    run_phase_4(data, report)
    
    print("Writing report...")
    with open("m9a1_analysis_results.md", "w") as f:
        f.write("\n".join(report))
        
    print("Done!")

if __name__ == '__main__':
    main()
