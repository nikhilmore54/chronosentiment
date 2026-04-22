import re

files_to_fix = [
    "services/api/src/services/evaluation_service.rs",
    "services/api/src/strategy_id_parse.rs",
]
for file in files_to_fix:
    with open(file, "r") as f:
        text = f.read()
    # Fix Strategy fields
    # Look for participation_threshold: <number>, and add exec genes
    text = re.sub(r'(participation_threshold:\s*\d+,)', r'\1\n            exec_aggression: 50, latency_bias: 10, fill_threshold: 50,', text)
    with open(file, "w") as f:
        f.write(text)

