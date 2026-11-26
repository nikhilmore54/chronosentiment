import json
import statistics

def analyze_m9a_0d():
    try:
        with open("m9a_ultracrew_state_0d.json") as f:
            data = json.load(f)
    except Exception as e:
        print(f"Error loading JSON: {e}")
        return
        
    print("=== M9A.0e Distance to Incumbent Results ===")
    
    for run in data:
        seed = run["seed"]
        instance = run["instance"]
        mode = run.get("mode", "Unknown")
        history = run["history"]
        
        print(f"\nInstance: {instance} | Seed: {seed} | Mode: {mode}")
        
        if not history:
            continue
            
        improvements = [h["generation"] for h in history if h["improvement_magnitude"] > 0 and h["best_fitness_age"] == 0]
        last_imp = max(improvements) if improvements else 0
        worse_acc_end = statistics.mean([h["worse_acceptance_rate"] for h in history[-50:]])
        
        print(f"  Total Breakthroughs: {len(improvements)}")
        print(f"  Last Breakthrough  : Gen {last_imp}")
        print(f"  End SA Acceptance  : {worse_acc_end*100:.1f}%")
        
        history_novelties = [h["history_novelty"] for h in history]
        print(f"  Structural Dist Rng: {min(history_novelties):.4f} - {max(history_novelties):.4f}")
        
        distances = [h.get("distance_to_incumbent_best", 0.0) for h in history]
        print(f"  Distance to Best Rng: {min(distances):.1f} - {max(distances):.1f}")
        
        # Calculate correlation manually or roughly:
        # Just print mean for now.
        print(f"  Avg Distance to Best: {statistics.mean(distances):.1f}")

if __name__ == "__main__":
    analyze_m9a_0d()
