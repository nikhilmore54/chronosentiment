import json
from collections import defaultdict

def validate_run():
    with open('m9a_validation.json') as f:
        data = json.load(f)
        
    history = data[0]['history']
    
    print("=== Check A & B: Improvement Magnitude vs Best Fitness ===")
    violations = 0
    non_improvements = 0
    improvements = 0
    
    last_best = None
    for h in history:
        gen = h['generation']
        fit = h['best_fitness']
        mag = h['improvement_magnitude']
        
        if last_best is None:
            last_best = fit
            continue
            
        is_imp_fit = fit < last_best
        if mag > 0 and not is_imp_fit:
            violations += 1
            
        if mag > 0:
            improvements += 1
        else:
            non_improvements += 1
            
        if is_imp_fit:
            last_best = fit
            
    print(f"Total Generations (after Gen 1): {len(history)-1}")
    print(f"Improvements: {improvements}")
    print(f"Non-improvements: {non_improvements}  <- Check B")
    print(f"Violations (mag > 0 but fitness didn't improve): {violations}  <- Check A")
    
    print("\n=== Check C: Droughts ===")
    gens = [h['generation'] for h in history if h['improvement_magnitude'] > 0]
    if gens:
        droughts = [gens[i] - gens[i-1] for i in range(1, len(gens))]
        print(f"Sample droughts (first 20): {droughts[:20]}")
        print(f"Max drought: {max(droughts) if droughts else 0}")
    
    print("\n=== Check D: Opportunity Table (Window=200) ===")
    WINDOW = 200
    opportunities = {'total': 0, 'with_improvement': 0, 'without_improvement': 0}
    current_op = None
    
    for i, h in enumerate(history):
        if h['dominant_operator'] != current_op:
            if current_op is not None:
                opportunities['total'] += 1
                window = history[i:i+WINDOW]
                if any(w['improvement_magnitude'] > 0 for w in window):
                    opportunities['with_improvement'] += 1
                else:
                    opportunities['without_improvement'] += 1
            current_op = h['dominant_operator']
            
    print(f"Total Regime Transitions Identified: {opportunities['total']}")
    if opportunities['total'] > 0:
        imp_rate = opportunities['with_improvement'] / opportunities['total'] * 100
        print(f"Transitions yielding Improvement: {opportunities['with_improvement']} ({imp_rate:.1f}%)")
        print(f"Transitions yielding NO Improvement: {opportunities['without_improvement']} ({100 - imp_rate:.1f}%)  <- Check D")

if __name__ == "__main__":
    validate_run()
