import os
import re

def fix_reco():
    reco_path = "financial/strategies/src/reco.rs"
    if not os.path.exists(reco_path): return
    with open(reco_path, "r") as f:
        code = f.read()
    
    code = code.replace(".behavioral_signature.axes", ".behavioral_signature")
    code = code.replace(".strategy", ".candidate")
    code = code.replace(".execution_metrics.capture_efficiency", ".capture_eff")
    code = code.replace(".execution_metrics", "")
    code = code.replace("eval.capture_eff", "eval.short_term_capture_eff")
    code = code.replace("best_metrics.capture_efficiency", "best_metrics.capture_eff")
    
    with open(reco_path, "w") as f:
        f.write(code)

def fix_strategy_id():
    path = "financial/strategies/src/strategy_id.rs"
    if not os.path.exists(path): return
    with open(path, "r") as f:
        code = f.read()
        
    code = code.replace("lineage: 0,", "")
    
    with open(path, "w") as f:
        f.write(code)

def fix_pipeline():
    path = "financial/strategies/src/pipeline.rs"
    if not os.path.exists(path): return
    with open(path, "r") as f:
        code = f.read()
        
    # We will comment out the ga:: calls that are breaking, they will be rewritten in D3
    code = re.sub(r'ga::ScenarioPair\s*\{', '// ga::ScenarioPair {', code)
    code = re.sub(r'let mut train_scenarios: Vec<ga::ScenarioPair<\'_>> = Vec::new\(\);', '// train_scenarios', code)
    code = re.sub(r'train_scenarios\.push\(', '// train_scenarios.push(', code)
    code = re.sub(r'let \(ga_result, _asset_states\) = ga::run_ga_evolution.*?;', 'let ga_result = crate::domain::StrategyEvaluation::default();', code, flags=re.DOTALL)
    code = re.sub(r'if let Some\(report\) = ga::evaluate_and_aggregate.*?\{', 'if false {', code, flags=re.DOTALL)
    code = re.sub(r'let one_scenario = \[', '// let one_scenario = [', code)
    
    code = code.replace("config.initial_queue_threshold = 200;", "")
    
    with open(path, "w") as f:
        f.write(code)
        
if __name__ == "__main__":
    fix_reco()
    fix_strategy_id()
    fix_pipeline()
