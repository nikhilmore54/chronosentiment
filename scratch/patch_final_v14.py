import re

with open("core/src/ga.rs", "r") as f:
    ga_text = f.read()

# Fix evaluation_flag duplicate
ga_text = ga_text.replace("evaluation_flag: None,\n            evaluation_flag: None,", "evaluation_flag: None,")

# Fix ga_result.0
ga_text = ga_text.replace("ga_result.0.global_best", "ga_result.global_best")
ga_text = ga_text.replace("ga_result.0.final_generation_best", "ga_result.final_generation_best")

# Fix missing ExecutionMetrics
old_metrics = """            execution_metrics: ExecutionMetrics {
                fill_efficiency: 0.0,
                capture_efficiency: 0.0,
            },"""
new_metrics = """            execution_metrics: ExecutionMetrics {
                fill_efficiency: 0.0,
                capture_efficiency: 0.0,
                fill_rate: 0.0, liquidity_starved_count: 0, queue_blocked_count: 0, avg_slippage: 0.0, latency_impact: 0.0, total_attempts: 0,
            },"""
ga_text = ga_text.replace(old_metrics, new_metrics)

with open("core/src/ga.rs", "w") as f:
    f.write(ga_text)

