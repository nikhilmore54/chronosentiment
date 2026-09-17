import sys

with open("/Users/nikhil/ChronoSentiment_MEGA_FINAL/scripts/research/run_multi_engine.py", "r") as f:
    content = f.read()

content = content.replace("./target/release/examples/live_observatory", "../target/release/examples/live_observatory")

with open("/Users/nikhil/ChronoSentiment_MEGA_FINAL/scripts/research/run_multi_engine.py", "w") as f:
    f.write(content)
