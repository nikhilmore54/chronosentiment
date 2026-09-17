import sys

with open("/Users/nikhil/ChronoSentiment_MEGA_FINAL/scripts/time_machine_audit.py", "r") as f:
    content = f.read()

content = content.replace('evidences[row["decision_id"]] = row', 'evidences[row["evidence_row_id"]] = row')

with open("/Users/nikhil/ChronoSentiment_MEGA_FINAL/scripts/time_machine_audit.py", "w") as f:
    f.write(content)
