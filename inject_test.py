import re

with open("infrastructure/optimization/src/evolution_engine.rs", "r") as f:
    content = f.read()

# Find #[test] followed by pub fn ... { or fn ... { and inject crate::init_test_delegates();
pattern = r"(\#\[test\]\s+(?:async )?fn \w+\([^\)]*\)\s*\{)"
replacement = r"\1\n        crate::init_test_delegates();"

new_content = re.sub(pattern, replacement, content)

if new_content != content:
    with open("infrastructure/optimization/src/evolution_engine.rs", "w") as f:
        f.write(new_content)
    print("Injected test delegates.")
else:
    print("No changes made.")
