import sys

with open("/Users/nikhil/ChronoSentiment_MEGA_FINAL/scripts/research/run_multi_engine.py", "r") as f:
    content = f.read()

content = content.replace("scripts/research/blue_green_log_writer.py", "research/blue_green_log_writer.py")
content = content.replace("scripts/research/fetch_stream_loop.py", "research/fetch_stream_loop.py")

with open("/Users/nikhil/ChronoSentiment_MEGA_FINAL/scripts/research/run_multi_engine.py", "w") as f:
    f.write(content)
