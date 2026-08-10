import re

with open("financial/strategies/src/pipeline.rs", "r") as f:
    content = f.read()

pattern = r"(let global_state = ga::GlobalEvoState::default\(\);\s+let \(ga_result, _asset_states\) = ga::run_ga_evolution)"
replacement = r"crate::market_regime::initialize_ga_delegates();\n        \1"

new_content = re.sub(pattern, replacement, content)

with open("financial/strategies/src/pipeline.rs", "w") as f:
    f.write(new_content)

print("Injected init to pipeline.")
