import re

with open("core/src/ga.rs", "r") as f:
    ga_text = f.read()

struct_block = re.search(r'pub struct StrategyEvaluation \{.*?\n\}', ga_text, re.DOTALL).group(0)
fields = re.findall(r'    (?:#\[.*?\]\n\s*)?pub ([a-zA-Z0-9_]+): ([^,\n]+),', struct_block)

assignments = []
for field, typ in fields:
    if typ == "String": val = 'String::new()'
    elif typ == "f64" or typ == "f32": val = '0.0'
    elif typ == "usize" or typ == "u64" or typ == "i32" or typ == "u8": val = '0'
    elif typ == "bool": val = 'false'
    elif typ == "Strategy": val = 'Strategy::from_seed(0)'
    elif typ.startswith("Vec"): val = 'Vec::new()'
    elif typ.startswith("Option"): val = 'None'
    elif typ == "AcceptanceMode": val = 'AcceptanceMode::Dominance'
    elif typ == "BehavioralSignature": val = 'BehavioralSignature { fingerprint: 0, axes: (0,0,0,0) }'
    elif typ == "ScenarioCapability": val = 'ScenarioCapability::Executable'
    elif typ == "ExecutionMetrics": val = 'ExecutionMetrics { fill_efficiency: 0.0, capture_efficiency: 0.0, fill_rate: 0.0, avg_slippage: 0.0, latency_impact: 0.0, queue_blocked_count: 0, liquidity_starved_count: 0, total_attempts: 0 }'
    elif typ == "ScenarioExecutionSignature": val = 'ScenarioExecutionSignature { avg_queue_ahead: 0.0, avg_latency: 0.0, fill_ratio: 0.0, participation: 0.0, execution_variance: 0.0 }'
    elif typ == "[f64; 6]": val = '[0.0; 6]'
    else: val = f'{typ}::default()'
    
    if field == "strategy_id": assignments.append('strategy_id: "FLAGGED".to_string(),')
    elif field == "evaluation_flag": assignments.append('evaluation_flag: Some(flag.to_string()),')
    elif field == "fitness": assignments.append('fitness: -0.03,')
    else: assignments.append(f'{field}: {val},')

assignments_str = "\n            ".join(assignments)

new_legacy = f"""    pub fn new_legacy_with_flag(flag: &str) -> Self {{
        Self {{
            {assignments_str}
        }}
    }}"""

# We replace the buggy new_legacy_with_flag inside `ga.rs` completely
ga_text = re.sub(r'    pub fn new_legacy_with_flag\(flag: &str\) -> Self \{.*?\n    \}', new_legacy, ga_text, flags=re.DOTALL)

with open("core/src/ga.rs", "w") as f:
    f.write(ga_text)

