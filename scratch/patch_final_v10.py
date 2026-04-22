with open("core/src/ga.rs", "r") as f:
    ga_text = f.read()

# remove duplicate evaluation_flag
ga_text = ga_text.replace("            recent_performance: 0.0,\n            evaluation_flag: None,", "            recent_performance: 0.0,")

with open("core/src/ga.rs", "w") as f:
    f.write(ga_text)

with open("core/src/pipeline.rs", "r") as f:
    pipe_text = f.read()

# fix pipeline.rs
import re

# First block: global_best
pipe_text = re.sub(
    r'(let global_best = \{ let best = ga_result\.global_best\.clone\(\);\s+to_unified\(\s+best\.clone\(\),\s+evaluate_on_exec\(&best\.strategy\),\s+\)\s+\}) else \{\s+// Fallback or handle null case appropriately for your pipeline\s+to_unified\(\s+ga::StrategyEvaluation::new_legacy\("EMPTY"\.to_string\(\), ga::Strategy::from_seed\(0\), 0\.0, 0\.0, 0, 0, 0\.0\),\s+evaluate_on_exec\(&ga::Strategy::default\(\)\)\s+\)\s+\};',
    r'\1;', pipe_text)

# Second block: final_generation_best
pipe_text = re.sub(
    r'(let final_generation_best = \{ let best = ga_result\.final_generation_best\.clone\(\);\s+to_unified\(\s+best\.clone\(\),\s+evaluate_on_exec\(&best\.strategy\),\s+\)\s+\}) else \{\s+to_unified\(\s+ga::StrategyEvaluation::new_legacy\("EMPTY"\.to_string\(\), ga::Strategy::from_seed\(0\), 0\.0, 0\.0, 0, 0, 0\.0\),\s+evaluate_on_exec\(&ga::Strategy::default\(\)\)\s+\)\s+\};',
    r'\1;', pipe_text)


with open("core/src/pipeline.rs", "w") as f:
    f.write(pipe_text)

