import re

with open("core/src/ga.rs", "r") as f:
    ga_text = f.read()

# Fix evaluation_flag missing in StrategyEvaluation { ... }
old_strat_eval = """            avg_hold_time: 0.0,
            consistency_score: 0.0,
            recent_performance: 0.0,"""

new_strat_eval = """            avg_hold_time: 0.0,
            consistency_score: 0.0,
            recent_performance: 0.0,
            evaluation_flag: None,"""
ga_text = ga_text.replace(old_strat_eval, new_strat_eval)
# Also remove any duplicates of evaluation_flag we might have accidentally created
ga_text = ga_text.replace("            evaluation_flag: None,\n            evaluation_flag: None,", "            evaluation_flag: None,")


# Fix entry_offset missing
# 10832 |         let strategy = Strategy { (missing entry_offset)
def fix_strat(match):
    s = match.group(0)
    if "entry_offset" not in s:
        return s.replace("archetype: 0,", "archetype: 0, entry_offset: 0,")
    return s

ga_text = re.sub(r'(let strategy = Strategy \{.*?\}\;)', fix_strat, ga_text, flags=re.DOTALL)


# Fix evaluate_strategy missing 8th argument
ga_text = ga_text.replace("evaluate_strategy(&strategy, &pair, &config, 0, 0.0, 0, 0.0)", "evaluate_strategy(&strategy, &pair, &config, 0, 0.0, 0, 0.0, 1.0)")


# Fix run_ga_evolution missing 3rd arg in tests
ga_text = ga_text.replace("let ga_result = run_ga_evolution(config, &scenarios_vec);", "let (ga_result, _) = run_ga_evolution(config, &scenarios_vec, &GlobalEvoState::default());\n        let ga_result = ga_result;")


# Fix ga_result.global_best misses in tests
ga_text = ga_text.replace("ga_result.global_best.fitness", "ga_result.0.global_best.fitness")
ga_text = ga_text.replace("ga_result.global_best.avg_pnl", "ga_result.0.global_best.avg_pnl")
ga_text = ga_text.replace("ga_result.global_best", "ga_result.0.global_best")
ga_text = ga_text.replace("ga_result.0.0.global_best", "ga_result.0.global_best") # clean up if duplicate
ga_text = ga_text.replace("ga_result.final_generation_best", "ga_result.0.final_generation_best")
ga_text = ga_text.replace("ga_result.0.0.final_generation_best", "ga_result.0.final_generation_best")

ga_text = ga_text.replace("let ga_result = ga_result.0;", "let ga_result = ga_result;")


# Fix ExecutionMetrics missing fields (again)
ga_text = ga_text.replace("""            execution_metrics: ExecutionMetrics {
                fill_efficiency: 0.0,
                capture_efficiency: 0.0,
            },""", """            execution_metrics: ExecutionMetrics {
                fill_efficiency: 0.0,
                capture_efficiency: 0.0,
                fill_rate: 0.0, liquidity_starved_count: 0, queue_blocked_count: 0, avg_slippage: 0.0, latency_impact: 0.0, total_attempts: 0,
            },""")

with open("core/src/ga.rs", "w") as f:
    f.write(ga_text)

