import json
from collections import defaultdict
import statistics

def analyze_sanity():
    try:
        with open("m9a_ultracrew_state.json") as f:
            data = json.load(f)
    except Exception as e:
        print(f"Error loading JSON: {e}")
        return
        
    print("=== M9A.0b Telemetry Sanity Review ===")
    
    for run in data:
        instance = run["instance"]
        seed = run["seed"]
        history = run["history"]
        
        if not history:
            continue
            
        print(f"\nInstance: {instance} | Seed: {seed}")
        print(f"Total Generations: {len(history)}")
        
        # 1. Operator Dominance Distribution
        op_shares = [h["dominant_operator_share"] for h in history]
        ops = defaultdict(int)
        for h in history:
            ops[h["dominant_operator"]] += 1
            
        print(f"Dominant Operator Frequencies:")
        for op, count in ops.items():
            print(f"  {op}: {count} generations ({(count/len(history))*100:.1f}%)")
            
        print(f"Dominant Operator Share Range: {min(op_shares):.3f} - {max(op_shares):.3f}")
        
        # 2. Novelty Range
        novelties = [h["memory_novelty_proxy"] for h in history]
        print(f"Memory Novelty Proxy Range: {min(novelties):.3f} - {max(novelties):.3f}")
        
        # 3. Acceptance Rates Decay
        worse_acc_first_100 = [h["worse_acceptance_rate"] for h in history[:100]]
        worse_acc_last_100 = [h["worse_acceptance_rate"] for h in history[-100:]]
        
        print(f"Worse Acceptance Rate (First 100 mean): {statistics.mean(worse_acc_first_100):.3f}")
        print(f"Worse Acceptance Rate (Last 100 mean):  {statistics.mean(worse_acc_last_100):.3f}")
        
        # 4. Total Improvements
        improvements = [h for h in history if h["improvement_magnitude"] > 0 and h["best_fitness_age"] == 0]
        print(f"Total Breakthroughs: {len(improvements)}")

if __name__ == "__main__":
    analyze_sanity()
