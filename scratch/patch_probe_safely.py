import re

with open("core/src/ga.rs", "r") as f:
    text = f.read()

# 1. Update initializes to include is_probe: false
# Targeting: ConvictionOutcome { ... }
# Use a non-greedy match and verify it looks like a struct init
def add_is_probe_safe(match):
    content = match.group(1)
    if "is_probe" not in content:
        # Check if it has fields like conviction_score or is_valid
        if "conviction_score" in content or "is_valid" in content:
            # Add before the last closing brace
            return "ConvictionOutcome {" + content.rstrip().rstrip(",") + ",\n            is_probe: false,\n        }"
    return match.group(0)

# Replace all ConvictionOutcome initializations
text = re.sub(r'ConvictionOutcome\s*\{([\s\S]*?)\}', add_is_probe_safe, text)

with open("core/src/ga.rs", "w") as f:
    f.write(text)
