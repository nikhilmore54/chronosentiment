import sys

with open("/Users/nikhil/ChronoSentiment_MEGA_FINAL/scripts/run_live.sh", "r") as f:
    content = f.read()

content = content.replace('ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"', 'ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"')

with open("/Users/nikhil/ChronoSentiment_MEGA_FINAL/scripts/run_live.sh", "w") as f:
    f.write(content)
