import re

def parse_blocks(filepath):
    with open(filepath, 'r') as f:
        lines = f.readlines()
        
    blocks = []
    current_comments = []
    current_block = []
    in_block = False
    
    for line in lines:
        if not in_block:
            if line.startswith("//") or line.startswith("#[") or line.strip() == "":
                current_comments.append(line)
            else:
                in_block = True
                current_block.extend(current_comments)
                current_comments = []
                current_block.append(line)
        else:
            current_block.append(line)
            if line.startswith("}"):
                blocks.append("".join(current_block))
                current_block = []
                in_block = False
                
    if current_comments or current_block:
        blocks.append("".join(current_comments + current_block))
        
    return blocks

def classify(block):
    clean_block = "\n".join(line for line in block.split("\n") if not line.strip().startswith("//"))
    
    # We only want to keep the absolute bare minimum for the optimization crate
    opt_structs = [
        "Candidate", "CandidateAnnotation", "CandidateEvaluation", "GaConfig", "GaResult", 
        "DistributionStats", "EvaluationMetrics", "PopulationStats", 
        "ElitePopulationBundle", "PercentileBuffer", "GaDiversityMode", "FitnessMode"
    ]
    for s in opt_structs:
        if re.search(r'^(pub )?(struct|impl|enum)\s+' + s + r'\b', clean_block, re.MULTILINE):
            return "core"
        if re.search(r'^impl.*for\s+' + s + r'\b', clean_block, re.MULTILINE):
            return "core"
            
    opt_funcs = [
        "run_ga_evolution", "mutate_strategy", "mutate_candidate", "crossover", "random_strategy", "initialize_population",
        "ga_debug_enabled", "ga_log_progress", "percentilef64", "clamp01", "compute_std_dev",
        "deduplicate_population", "blend_u64", "tournament_selection_diverse", "tournament_selection_diverse_with_idx",
        "tournament_selection_failure_mode", "cosine_dist", "unit_normalize", "compute_shannon_entropy",
        "percentile", "apply_ga_top_k_selection", "calculate_population_diversity", "save_elite_population",
        "calculate_genotype_distance_normalized", "calculate_behavioral_distance", 
        "ga_scenario_rank_score", "ga_top_k_pick_diverse", "calculate_genotype_distance",
        "stable_deterministic_hash", "scenario_execution_signature_l1", "pearson_correlation"
    ]
    for fn in opt_funcs:
        if re.search(r'^(pub )?fn\s+' + fn + r'\b', clean_block, re.MULTILINE):
            return "core"

    return "discard"

def main():
    source_file = "infrastructure/optimization/src/evolution_engine.rs"
    blocks = parse_blocks(source_file)
    print(f"Parsed {len(blocks)} blocks.")
    
    core_blocks = []
    discarded_blocks = []
    
    for b in blocks:
        if classify(b) == "core":
            core_blocks.append(b)
        else:
            discarded_blocks.append(b)
            
    header = """use serde::{Serialize, Deserialize};
use rand::Rng;
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::collections::{HashMap, HashSet, BTreeMap, VecDeque};
use std::fs;
use std::path::Path;
use chrono::Utc;

pub trait FitnessEvaluator<T> {
    type Evaluation;
    fn evaluate(&self, candidate: &T) -> Self::Evaluation;
}

"""
    
    with open(source_file, "w") as f:
        f.write(header + "".join(core_blocks))
        
    print(f"Kept {len(core_blocks)} blocks, discarded {len(discarded_blocks)} blocks.")

if __name__ == "__main__":
    main()
