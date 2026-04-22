import re

with open("core/src/pipeline.rs", "r") as f:
    pipe_text = f.read()

# Fix the if let Some block completely
find1 = """    let global_best = { let best = ga_result.global_best.clone();
        to_unified(
            best.clone(),
            evaluate_on_exec(&best.strategy),
        )
    } else {
        // Fallback or handle null case appropriately for your pipeline
        to_unified(
            ga::StrategyEvaluation::new_legacy("EMPTY".to_string(), ga::Strategy::from_seed(0), 0.0, 0.0, 0, 0, 0.0),
            evaluate_on_exec(&ga::Strategy::default())
        )
    };"""

replace1 = """    let best = ga_result.global_best.clone();
    let global_best = to_unified(
        best.clone(),
        evaluate_on_exec(&best.strategy),
    );"""
pipe_text = pipe_text.replace(find1, replace1)

find2 = """    let final_generation_best = { let best = ga_result.final_generation_best.clone();
        to_unified(
            best.clone(),
            evaluate_on_exec(&best.strategy),
        )
    } else {
        to_unified(
            ga::StrategyEvaluation::new_legacy("EMPTY".to_string(), ga::Strategy::from_seed(0), 0.0, 0.0, 0, 0, 0.0),
            evaluate_on_exec(&ga::Strategy::default())
        )
    };"""
replace2 = """    let best2 = ga_result.final_generation_best.clone();
    let final_generation_best = to_unified(
        best2.clone(),
        evaluate_on_exec(&best2.strategy),
    );"""
pipe_text = pipe_text.replace(find2, replace2)

pipe_text = pipe_text.replace("use crate::safe_log;\n", "")

with open("core/src/pipeline.rs", "w") as f:
    f.write(pipe_text)

with open("core/src/ga.rs", "r") as f:
    ga_text = f.read()

ga_text = ga_text.replace("crate::safe_log!(", "println!(")

# evaluation_flag missing at ~ 1977
old_init = """            avg_hold_time: 0.0,
            consistency_score: 0.0,
            recent_performance: 0.0,"""
new_init = """            avg_hold_time: 0.0,
            consistency_score: 0.0,
            recent_performance: 0.0,
            evaluation_flag: None,"""
ga_text = ga_text.replace(old_init, new_init)

with open("core/src/ga.rs", "w") as f:
    f.write(ga_text)

