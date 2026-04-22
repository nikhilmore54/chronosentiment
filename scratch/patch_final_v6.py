import re

with open("core/src/pipeline.rs", "r") as f:
    pipe_text = f.read()

pipe_text = pipe_text.replace("let global_best = let best = ga_result.global_best.clone(); {", "let global_best = { let best = ga_result.global_best.clone();")
pipe_text = pipe_text.replace("let final_generation_best = let best = ga_result.final_generation_best.clone(); {", "let final_generation_best = { let best = ga_result.final_generation_best.clone();")

pipe_text = "use crate::safe_log;\n" + pipe_text

with open("core/src/pipeline.rs", "w") as f:
    f.write(pipe_text)

with open("core/src/ga.rs", "r") as f:
    ga_text = f.read()

# StrategyEvaluation struct
# We need to add evaluation_flag after behavioral_signature
find_sig = "    pub behavioral_signature: BehavioralSignature,\n"
ga_text = ga_text.replace(find_sig, find_sig + "    pub evaluation_flag: Option<String>,\n", 1)

# Oh wait, we had error[E0433]: failed to resolve: could not find `safe_log` in the crate root in ga.rs line 4975
ga_text = ga_text.replace("crate::safe_log!(", "crate::safe_log!(")

with open("core/src/ga.rs", "w") as f:
    f.write(ga_text)
