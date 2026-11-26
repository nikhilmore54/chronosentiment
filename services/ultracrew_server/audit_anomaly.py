import json

def run_audit():
    with open('m9a_ultracrew_state_large.json') as f:
        data = json.load(f)
        
    # Find n030w4 seed 1
    run = next(r for r in data if r['instance'] == 'n030w4' and r['seed'] == 1)
    history = run['history']
    
    print("=== AUDIT 1: First 50 Generations ===")
    print(f"{'Gen':<5} | {'Best Fitness':<15} | {'Imp Magnitude':<15}")
    for h in history[:50]:
        print(f"{h['generation']:<5} | {h['best_fitness']:<15} | {h['improvement_magnitude']:<15}")
        
    print("\n=== AUDIT 2: Count of Improvements ===")
    imp_count = sum(1 for h in history if h['improvement_magnitude'] > 0)
    print(f"Total generations: {len(history)}")
    print(f"Generations with improvement_magnitude > 0: {imp_count}")
    
    print("\n=== AUDIT 3: Best Fitness Curve Analysis ===")
    unique_fitnesses = set(h['best_fitness'] for h in history)
    print(f"Number of unique best_fitness values over 10,000 gens: {len(unique_fitnesses)}")
    
if __name__ == "__main__":
    run_audit()
