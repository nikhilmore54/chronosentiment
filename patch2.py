import sys

with open("/Users/nikhil/ChronoSentiment_MEGA_FINAL/scripts/candidate_validation_time_machine_v0_1.py", "r") as f:
    content = f.read()

content = content.replace('''            if obs_exit != bs["sim_exit_reason"]:
                if obs_exit == "NO_TRADE":
                    pass # We filtered these, but just in case
                else:
                    mismatches += 1''', '''            if obs_exit != bs["sim_exit_reason"]:
                if obs_exit == "NO_TRADE":
                    pass # We filtered these, but just in case
                else:
                    print(f"Mismatch for {did}: OBS={obs_exit}, SIM={bs['sim_exit_reason']}")
                    mismatches += 1''')

with open("/Users/nikhil/ChronoSentiment_MEGA_FINAL/scripts/candidate_validation_time_machine_v0_1.py", "w") as f:
    f.write(content)
