with open("financial/strategies/src/pipeline.rs", "r") as f:
    text = f.read()

import re
match = re.search(r"pub fn evaluate_on_real_data\([\s\S]*?\) -> Vec<MetricAggregation> \{", text)
if match:
    start_idx = match.start()
    
    test_match = re.search(r"#\[cfg\(test\)\]", text[start_idx:])
    if test_match:
        end_idx = start_idx + test_match.start()
        
        # New function definition
        new_func = """pub fn evaluate_on_real_data(
    assets: Vec<(String, String)>,
    global_lambda: f64,
) -> Vec<MetricAggregation> {
    unimplemented!()
}

"""
        new_text = text[:start_idx] + new_func + text[end_idx:]
        with open("financial/strategies/src/pipeline.rs", "w") as f:
            f.write(new_text)
