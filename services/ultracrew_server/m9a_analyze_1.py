import json
import statistics
import os

def analyze_m9a_1():
    try:
        with open("m9a_ultracrew_state_large.json") as f:
            data = json.load(f)
    except Exception as e:
        print(f"Error loading JSON: {e}")
        return
        
    print("=== M9A.1 Large Scale SearchState Observatory ===")
    
    # Store cluster aggregates but print raw summary for now
    total_runs = len(data)
    print(f"Total Runs Analyzed: {total_runs}")
    
    total_breakthroughs = 0
    late_breakthroughs = 0 # > gen 1000
    
    for run in data:
        seed = run["seed"]
        instance = run["instance"]
        history = run["history"]
        
        if not history:
            continue
            
        improvements = [h["generation"] for h in history if h["improvement_magnitude"] > 0 and h["best_fitness_age"] == 0]
        total_breakthroughs += len(improvements)
        late_breakthroughs += len([g for g in improvements if g > 1000])

    print(f"Total Breakthroughs Found: {total_breakthroughs}")
    print(f"Late Breakthroughs (>1000): {late_breakthroughs}")
    
    print("\nDataset ready for deeper archetypal clustering!")

if __name__ == "__main__":
    analyze_m9a_1()
