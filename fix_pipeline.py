with open("financial/strategies/src/pipeline.rs", "r") as f:
    text = f.read()

import re
# Find the evaluate_on_real_data definition
match = re.search(r"pub fn evaluate_on_real_data\([\s\S]*?\) -> Vec<MetricAggregation> \{", text)
if match:
    start_idx = match.end()
    # Find #[cfg(test)]
    test_match = re.search(r"#\[cfg\(test\)\]", text[start_idx:])
    if test_match:
        end_idx = start_idx + test_match.start()
        new_text = text[:start_idx] + "\n    unimplemented!()\n}\n\n" + text[end_idx:]
        with open("financial/strategies/src/pipeline.rs", "w") as f:
            f.write(new_text)
