import re

with open("core/src/ga.rs", "r") as f:
    text = f.read()

struct_block = re.search(r'pub struct StrategyEvaluation \{.*?\n\}', text, re.DOTALL).group(0)
print(struct_block)
