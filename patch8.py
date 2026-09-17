import sys

with open("/Users/nikhil/ChronoSentiment_MEGA_FINAL/scripts/time_machine_audit.py", "r") as f:
    content = f.read()

content = content.replace('evidences[row["evidence_row_id"]] = row', 'did_mapped = row["evidence_row_id"].replace("TIME006", "TIME004")\n                    evidences[did_mapped] = row')

with open("/Users/nikhil/ChronoSentiment_MEGA_FINAL/scripts/time_machine_audit.py", "w") as f:
    f.write(content)
