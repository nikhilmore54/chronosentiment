import sys

with open("/Users/nikhil/ChronoSentiment_MEGA_FINAL/scripts/candidate_validation_time_machine_v0_1.py", "r") as f:
    content = f.read()

content = content.replace('''    with open(p) as f:
        bars = json.load(f)''', '''    with open(p) as f:
        bars = [b for b in json.load(f) if b.get("volume", 0) > 0]''')

with open("/Users/nikhil/ChronoSentiment_MEGA_FINAL/scripts/candidate_validation_time_machine_v0_1.py", "w") as f:
    f.write(content)
