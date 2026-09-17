import sys

with open("/Users/nikhil/ChronoSentiment_MEGA_FINAL/scripts/run_live.sh", "r") as f:
    content = f.read()

content = content.replace("scripts/run_multi_engine.py", "scripts/research/run_multi_engine.py")
content = content.replace("scripts/telemetry_archive_daemon.py", "scripts/research/telemetry_archive_daemon.py")

with open("/Users/nikhil/ChronoSentiment_MEGA_FINAL/scripts/run_live.sh", "w") as f:
    f.write(content)
