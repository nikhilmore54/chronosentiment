import json
import matplotlib.pyplot as plt
import numpy as np
from collections import defaultdict
import os

def load_data(filepath):
    if not os.path.exists(filepath):
        print(f"File {filepath} not found.")
        return []
    with open(filepath, 'r') as f:
        return json.load(f)

def analyze_regimes(history):
    # Detect regime transitions based on dominant operator changes
    regimes = []
    current_op = None
    start_gen = 0
    
    for snapshot in history:
        op = snapshot['dominant_operator']
        gen = snapshot['generation']
        
        if op != current_op:
            if current_op is not None:
                regimes.append({
                    'operator': current_op,
                    'start': start_gen,
                    'end': gen - 1,
                    'duration': gen - start_gen
                })
            current_op = op
            start_gen = gen
            
    if current_op is not None and history:
        regimes.append({
            'operator': current_op,
            'start': start_gen,
            'end': history[-1]['generation'],
            'duration': history[-1]['generation'] - start_gen + 1
        })
        
    return regimes

def plot_trajectories(reports, instance_name):
    # Filter reports for the given instance
    instance_reports = [r for r in reports if r['instance'] == instance_name]
    if not instance_reports:
        return
        
    fig, axs = plt.subplots(3, 1, figsize=(12, 15), sharex=True)
    fig.suptitle(f'SearchState Trajectories for {instance_name} (n={len(instance_reports)})', fontsize=16)
    
    # We will plot individual trajectories with high transparency to see classes (A, B, C)
    for rep in instance_reports:
        history = rep['history']
        gens = [s['generation'] for s in history]
        
        # Plot Fitness
        fitness = [s['best_fitness'] for s in history]
        axs[0].plot(gens, fitness, alpha=0.15, color='blue')
        
        # Plot Diversity
        diversity = [s['diversity'] for s in history]
        axs[1].plot(gens, diversity, alpha=0.15, color='green')
        
        # Plot Unique Successful Operators (Proxy for Operator Regime)
        unique_ops = [s['unique_successful_operators'] for s in history]
        axs[2].plot(gens, unique_ops, alpha=0.05, color='purple')
        
    axs[0].set_ylabel('Best Fitness')
    axs[0].set_title('Trajectory Clusters (Type A vs B vs C)')
    
    axs[1].set_ylabel('Diversity')
    axs[1].set_title('Diversity Collapse vs Steady State')
    
    axs[2].set_ylabel('Unique Successful Operators')
    axs[2].set_title('Operator Monoculture Phases')
    axs[2].set_xlabel('Generation')
    
    plt.tight_layout()
    plt.savefig(f'{instance_name}_trajectories.png')
    plt.close()

def extract_events(history):
    events = []
    
    # Track states to generate events
    current_op = None
    last_fitness = None
    diversity_collapsed = False
    novelty_collapsed = False
    
    for i, snapshot in enumerate(history):
        gen = snapshot['generation']
        fit = snapshot['best_fitness']
        div = snapshot['diversity']
        nov = snapshot['memory_novelty']
        op = snapshot['dominant_operator']
        
        # Improvement Event
        if last_fitness is not None and fit < last_fitness:  # Assuming minimization
            delta = last_fitness - fit
            # Find previous snapshot for 'before' state
            prev = history[i-1] if i > 0 else snapshot
            events.append({
                'type': 'improvement',
                'generation': gen,
                'delta': delta,
                'diversity_before': prev['diversity'],
                'novelty_before': prev['memory_novelty'],
                'dominant_operator': op
            })
            last_fitness = fit
        elif last_fitness is None or fit < last_fitness:
            last_fitness = fit
            
        # Regime Change Event
        if op != current_op:
            if current_op is not None:
                events.append({
                    'type': 'regime_change',
                    'generation': gen,
                    'from': current_op,
                    'to': op
                })
            current_op = op
            
        # Diversity Collapse Event
        if div < 0.001 and not diversity_collapsed:
            events.append({
                'type': 'diversity_collapse',
                'generation': gen,
                'diversity': div
            })
            diversity_collapsed = True
        elif div >= 0.005:
            diversity_collapsed = False
            
        # Novelty Collapse Event
        if nov < 0.001 and not novelty_collapsed:
            events.append({
                'type': 'novelty_collapse',
                'generation': gen,
                'novelty': nov
            })
            novelty_collapsed = True
        elif nov >= 0.005:
            novelty_collapsed = False
            
    return events

def compute_regime_stats(reports):
    regime_durations = defaultdict(list)
    transitions_per_run = []
    
    # For Recovery Window Analysis
    # We will measure distances from each improvement to the most recent regime change
    improvement_distances = []
    
    # For combination condition: Regime Change + Novelty Recovery
    # We will track what happens within X generations after a regime change
    X_WINDOW = 200
    regime_change_outcomes = {'total': 0, 'imp_only': 0, 'nov_rec_only': 0, 'both': 0, 'neither': 0}
    
    for rep in reports:
        regimes = analyze_regimes(rep['history'])
        transitions_per_run.append(len(regimes) - 1)
        for r in regimes:
            regime_durations[r['operator']].append(r['duration'])
            
        events = extract_events(rep['history'])
        improvements = [e for e in events if e['type'] == 'improvement']
        regime_changes = [e for e in events if e['type'] == 'regime_change']
        
        if improvements:
            rep['_total_improvements'] = len(improvements)
            
        # 1. Recovery Window Analysis
        for imp in improvements:
            imp_gen = imp['generation']
            # Find closest preceding regime change
            preceding_rc = [rc for rc in regime_changes if rc['generation'] < imp_gen]
            if preceding_rc:
                last_rc_gen = preceding_rc[-1]['generation']
                distance = imp_gen - last_rc_gen
                improvement_distances.append(distance)
                
        # 2. Regime Change Outcomes
        for rc in regime_changes:
            rc_gen = rc['generation']
            window_end = rc_gen + X_WINDOW
            
            # Did an improvement occur in this window? And what was the max delta?
            imps_in_window = [e for e in improvements if rc_gen < e['generation'] <= window_end]
            imp_in_window = len(imps_in_window) > 0
            max_delta = max((e['delta'] for e in imps_in_window), default=0)
            
            # Did novelty recover in this window? (e.g. novelty > 0.05)
            # Find history within window
            history_in_window = [s for s in rep['history'] if rc_gen < s['generation'] <= window_end]
            nov_recovered = any(s['memory_novelty'] > 0.05 for s in history_in_window)
            
            regime_change_outcomes['total'] += 1
            category = 'neither'
            if imp_in_window and nov_recovered:
                category = 'both'
                regime_change_outcomes['both'] += 1
            elif imp_in_window:
                category = 'imp_only'
                regime_change_outcomes['imp_only'] += 1
            elif nov_recovered:
                category = 'nov_rec_only'
                regime_change_outcomes['nov_rec_only'] += 1
            else:
                regime_change_outcomes['neither'] += 1
                
            # Track magnitude of improvement by category
            if 'deltas' not in regime_change_outcomes:
                regime_change_outcomes['deltas'] = {'both': [], 'imp_only': []}
            if imp_in_window and category in ['both', 'imp_only']:
                regime_change_outcomes['deltas'][category].append(max_delta)
            
    print(f"\nAverage operator transitions per run: {np.mean(transitions_per_run):.2f}")
    print("\nRegime Durations by Operator:")
    for op, durations in regime_durations.items():
        print(f"  {op}: median {np.median(durations):.1f} gens, max {np.max(durations)} gens")
        
    print(f"\n=== State Discovery (Durations) ===")
    # 1. Novelty Suppression Duration
    # Calculate how long novelty stays < 0.01 once it drops below that threshold
    novelty_suppression_durations = []
    for rep in reports:
        in_suppression = False
        suppression_start = 0
        for s in rep['history']:
            nov = s['memory_novelty']
            if nov < 0.01 and not in_suppression:
                in_suppression = True
                suppression_start = s['generation']
            elif nov >= 0.01 and in_suppression:
                in_suppression = False
                novelty_suppression_durations.append(s['generation'] - suppression_start)
        if in_suppression:
            novelty_suppression_durations.append(rep['history'][-1]['generation'] - suppression_start)
            
    if novelty_suppression_durations:
        print(f"Novelty Suppression Periods (<0.01): Count = {len(novelty_suppression_durations)}, Median Duration = {np.median(novelty_suppression_durations):.1f} gens, Max = {np.max(novelty_suppression_durations):.1f}")
    
    # 2. Intervals between Major Improvements (delta > median delta of all improvements)
    all_deltas = [e['delta'] for rep in reports for e in extract_events(rep['history']) if e['type'] == 'improvement']
    if all_deltas:
        median_global_delta = np.median(all_deltas)
        major_improvement_intervals = []
        for rep in reports:
            events = extract_events(rep['history'])
            major_imps = [e['generation'] for e in events if e['type'] == 'improvement' and e['delta'] > median_global_delta]
            for i in range(1, len(major_imps)):
                major_improvement_intervals.append(major_imps[i] - major_imps[i-1])
                
        if major_improvement_intervals:
            print(f"Major Improvement Intervals (delta > {median_global_delta:.1f}): Count = {len(major_improvement_intervals)}, Median Interval = {np.median(major_improvement_intervals):.1f} gens, Max = {np.max(major_improvement_intervals):.1f}")
    
    print(f"\n=== Recovery Window Analysis ===")
    if improvement_distances:
        print(f"Total improvements analyzed (with prior regime change): {len(improvement_distances)}")
        print(f"Median distance from Regime Change to Improvement: {np.median(improvement_distances):.1f} gens")
        dist_under_100 = sum(1 for d in improvement_distances if d <= 100)
        dist_under_500 = sum(1 for d in improvement_distances if d <= 500)
        print(f"Improvements within 100 gens of Regime Change: {dist_under_100} ({dist_under_100/len(improvement_distances)*100:.1f}%)")
        print(f"Improvements within 500 gens of Regime Change: {dist_under_500} ({dist_under_500/len(improvement_distances)*100:.1f}%)")
        
    print(f"\n=== Opportunity Table (Window = {X_WINDOW} gens) ===")
    print(f"{'State':<15} | {'Count':<7} | {'Imp Rate':<10} | {'Median Delta':<12} | {'Max Delta':<10}")
    print("-" * 65)
    
    # Count opportunities
    rc_no_nr_count = regime_change_outcomes['imp_only'] + regime_change_outcomes['neither']
    rc_nr_count = regime_change_outcomes['both'] + regime_change_outcomes['nov_rec_only']
    
    # Count successes
    rc_no_nr_success = regime_change_outcomes['imp_only']
    rc_nr_success = regime_change_outcomes['both']
    
    # Deltas
    rc_no_nr_deltas = regime_change_outcomes.get('deltas', {}).get('imp_only', [])
    rc_nr_deltas = regime_change_outcomes.get('deltas', {}).get('both', [])
    
    for label, count, success, deltas in [
        ('RC Only (No NR)', rc_no_nr_count, rc_no_nr_success, rc_no_nr_deltas),
        ('RC + NR', rc_nr_count, rc_nr_success, rc_nr_deltas)
    ]:
        if count == 0:
            print(f"{label:<15} | {count:<7} | {'N/A':<10} | {'N/A':<12} | {'N/A':<10}")
            continue
        
        rate = f"{success/count*100:.1f}%"
        med_delta = f"{np.median(deltas):.1f}" if deltas else "N/A"
        max_delta = f"{np.max(deltas):.1f}" if deltas else "N/A"
        print(f"{label:<15} | {count:<7} | {rate:<10} | {med_delta:<12} | {max_delta:<10}")

if __name__ == "__main__":
    data = load_data("m9a_ultracrew_state.json")
    if data:
        instances = set(r['instance'] for r in data)
        print(f"Loaded {len(data)} runs across {len(instances)} instances.")
        
        for inst in instances:
            print(f"\nAnalyzing instance {inst}...")
            inst_data = [r for r in data if r['instance'] == inst]
            plot_trajectories(inst_data, inst)
            compute_regime_stats(inst_data)

