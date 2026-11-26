import json
import statistics

def analyze_m9a_0c():
    try:
        with open("m9a_ultracrew_state_0c.json") as f:
            data = json.load(f)
    except Exception as e:
        print(f"Error loading JSON: {e}")
        return
        
    print("=== M9A.0c Experiment Results ===")
    
    for run in data:
        seed = run["seed"]
        instance = run["instance"]
        global_cooling = run["global_cooling"]
        history = run["history"]
        
        mode = "GLOBAL Cooling" if global_cooling else "LOCAL Cooling"
        print(f"\nInstance: {instance} | Seed: {seed} | Mode: {mode}")
        
        if not history:
            continue
            
        # Exp A: Structural Novelty vs Workload Proxy
        workload_novelties = [h["memory_novelty_proxy"] for h in history]
        history_novelties = [h["history_novelty"] for h in history]
        
        print(f"  Exp A (Novelty):")
        print(f"    Workload Gini Range : {min(workload_novelties):.4f} - {max(workload_novelties):.4f}")
        print(f"    Structural Dist Rng : {min(history_novelties):.4f} - {max(history_novelties):.4f}")
        
        # Exp D: Revisit Rate
        revisits = [h["revisit_rate"] for h in history]
        print(f"  Exp D (Revisit Rate):")
        print(f"    Revisit Rate Mean   : {statistics.mean(revisits)*100:.1f}%")
        print(f"    Revisit Rate Max    : {max(revisits)*100:.1f}%")
        
        # Exp B: Operator Competition
        t1_attempts = sum(h["tier1_attempts"] for h in history)
        t1_accepts = sum(h["tier1_acceptances"] for h in history)
        t1_imps = sum(h["tier1_improvements"] for h in history)
        
        t2_attempts = sum(h["tier2_attempts"] for h in history)
        t2_accepts = sum(h["tier2_acceptances"] for h in history)
        t2_imps = sum(h["tier2_improvements"] for h in history)
        
        t1_acc_rate = t1_accepts / max(1, t1_attempts)
        t2_acc_rate = t2_accepts / max(1, t2_attempts)
        
        print(f"  Exp B (Operator Competition):")
        print(f"    Tier 1: {t1_attempts} attempts | {t1_accepts} accepts ({t1_acc_rate*100:.1f}%) | {t1_imps} improvements")
        print(f"    Tier 2: {t2_attempts} attempts | {t2_accepts} accepts ({t2_acc_rate*100:.1f}%) | {t2_imps} improvements")
        
        # Exp C: Cooling Schedule
        improvements = [h["generation"] for h in history if h["improvement_magnitude"] > 0 and h["best_fitness_age"] == 0]
        last_imp = max(improvements) if improvements else 0
        worse_acc_end = statistics.mean([h["worse_acceptance_rate"] for h in history[-50:]])
        
        print(f"  Exp C (Cooling Schedule):")
        print(f"    Total Breakthroughs: {len(improvements)}")
        print(f"    Last Breakthrough  : Gen {last_imp}")
        print(f"    End SA Acceptance  : {worse_acc_end*100:.1f}%")

if __name__ == "__main__":
    analyze_m9a_0c()
