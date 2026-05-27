import re

with open("infrastructure/optimization/src/lib.rs", "r") as f:
    content = f.read()

content = "pub mod edge_decay;\npub mod edge_half_life_estimator;\n" + content

with open("infrastructure/optimization/src/lib.rs", "w") as f:
    f.write(content)

with open("financial/strategies/src/lib.rs", "r") as f:
    content = f.read()

content = content.replace("pub mod edge_decay;\n", "")
content = content.replace("pub mod edge_half_life_estimator;\n", "")

with open("financial/strategies/src/lib.rs", "w") as f:
    f.write(content)
