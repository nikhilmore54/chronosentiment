import re

with open("core/src/ga.rs", "r") as f:
    ga_text = f.read()

# 1. Allow unused_parens and unused_assignments and unreachable_patterns
ga_text = ga_text.replace("#![allow(unused_variables, unused_mut, unused_imports, dead_code, unreachable_code, unused_assignments)]", "#![allow(unused_variables, unused_mut, unused_imports, dead_code, unreachable_code, unused_assignments, unused_parens, unreachable_patterns)]")

# 2. Fix remaining Strategy missing fields
# Find all occurrences of "participation_threshold: " and add exec_aggression if missing
def fix_strategy(match):
    s = match.group(0)
    if "exec_aggression" not in s:
        return s.replace("}", "    exec_aggression: 50, latency_bias: 10, fill_threshold: 50,\n        }")
    return s
ga_text = re.sub(r'Strategy \{[^\}]*?participation_threshold: [^\}]*?\}', fix_strategy, ga_text)

# Also fix the empty ones
ga_text = re.sub(r'(Strategy \{)(\s*\})', r'\1\n            exec_aggression: 50, latency_bias: 10, fill_threshold: 50,\n        }', ga_text)


# 3. Why is new_legacy missing?
# Maybe StrategyEvaluation block has two impl blocks, and my script didn't touch it, but wait:
# I didn't replace `new_legacy`. Let's check `pipeline.rs` instead.
with open("core/src/ga.rs", "w") as f:
    f.write(ga_text)

with open("core/src/edge_decay.rs", "r") as f:
    edge_text = f.read()
edge_text = re.sub(r'Strategy \{[^\}]*?participation_threshold: [^\}]*?\}', fix_strategy, edge_text)
# the ones without participation_threshold:
def fix_strat_empty(match):
    s = match.group(0)
    if "exec_aggression" not in s:
        return s.replace("}", "    exec_aggression: 50, latency_bias: 10, fill_threshold: 50,\n        }")
    return s

edge_text = re.sub(r'Strategy \{([^{}]*)\}', fix_strat_empty, edge_text)
with open("core/src/edge_decay.rs", "w") as f:
    f.write(edge_text)


with open("core/src/pipeline.rs", "r") as f:
    pipe_text = f.read()

# Fix pipeline.rs mismatched types because GaResult `global_best` is an `Option` in pipeline.rs but not in ga.rs.
# Let's fix pipeline.rs to match ga.rs (global_best is StrategyEvaluation, not Option)
pipe_text = pipe_text.replace("global_best: None,", 'global_best: ga::StrategyEvaluation::new_legacy("EMPTY".to_string(), ga::Strategy::from_seed(0), 0.0, 0.0, 0, 0, 0.0),')
pipe_text = pipe_text.replace("final_generation_best: None,", 'final_generation_best: ga::StrategyEvaluation::new_legacy("EMPTY".to_string(), ga::Strategy::from_seed(0), 0.0, 0.0, 0, 0, 0.0),')

pipe_text = pipe_text.replace("if let Some(best) = ga_result.global_best.clone() {", "let best = ga_result.global_best.clone(); {")
pipe_text = pipe_text.replace("if let Some(best) = ga_result.final_generation_best.clone() {", "let best = ga_result.final_generation_best.clone(); {")

pipe_text = pipe_text.replace(".or(ga_result.global_best.clone());", ".or(Some(ga_result.global_best.clone()));")
# line 2930: .or(ga_result.global_best.as_ref());
pipe_text = pipe_text.replace(".or(ga_result.global_best.as_ref());", ".or(Some(&ga_result.global_best));")

with open("core/src/pipeline.rs", "w") as f:
    f.write(pipe_text)

