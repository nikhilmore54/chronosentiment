import re

files_to_fix = [
    "core/src/pnl_overlay.rs",
    "core/src/replay_evaluator.rs",
    "core/src/strategy_ranking.rs",
]
for file in files_to_fix:
    with open(file, "r") as f:
        text = f.read()
    # Fix Strategy fields
    text = re.sub(r'(participation_threshold:\s*\d+,)', r'\1\n            exec_aggression: 50, latency_bias: 10, fill_threshold: 50,', text)
    with open(file, "w") as f:
        f.write(text)

with open("core/src/ga.rs", "r") as f:
    ga_text = f.read()

# Fix Strategy missing entry_offset
# 11021 |             strategy: Strategy { ... missing entry_offset
def add_entry(match):
    s = match.group(0)
    if "entry_offset" not in s:
        return s.replace("archetype: 0,", "archetype: 0, entry_offset: 0,")
    return s
ga_text = re.sub(r'Strategy \{.*?(?:archetype: 0,).*?\}', add_entry, ga_text, flags=re.DOTALL)


# Fix ExecutionMetrics missing fields
ga_text = ga_text.replace("""            execution_metrics: ExecutionMetrics {
                fill_efficiency: 0.0,
                capture_efficiency: 0.0,
            },""", """            execution_metrics: ExecutionMetrics {
                fill_efficiency: 0.0,
                capture_efficiency: 0.0,
                fill_rate: 0.0, liquidity_starved_count: 0, queue_blocked_count: 0, avg_slippage: 0.0, latency_impact: 0.0, total_attempts: 0,
            },""")


# Fix missing arg in run_ga_evolution test harnesses
ga_text = ga_text.replace("let ga_result = run_ga_evolution(config.clone(), &scenarios_vec);", "let (ga_result, _) = run_ga_evolution(config.clone(), &scenarios_vec, &GlobalEvoState::default());\n        let ga_result = ga_result;")

# Fix evaluate_and_aggregate_with_trade_depth missing 8th arg
# it's usually: &ga_result.global_best.strategy, &config, &scenarios_vec, 0, 0.0, 0, 0.0, <missing>
ga_text = re.sub(
    r'(evaluate_and_aggregate_with_trade_depth\([\s\S]*?&scenarios_vec,\s*0,\s*0\.0,\s*0,\s*0\.0,\s*)(\))',
    r'\1 1.0\2', ga_text
)


with open("core/src/ga.rs", "w") as f:
    f.write(ga_text)
