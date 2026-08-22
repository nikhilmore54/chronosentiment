import json
import hashlib
import os

artifacts = [
    "archive/datasets/sunair_schedule_output.json",
    "fixtures/demo/sunair_schedule.json",
    "fixtures/demo/sunair_report.json",
    "fixtures/demo/sunair_demo.json",
    "fixtures/demo/sunair_shifts.csv",
    "fixtures/demo/sunair_workers.csv"
]

print("## Provenance Check")
print("| Artifact | Type | Shifts | Workers | Horizon | Matches UI (770/33/768)? |")
print("|----------|------|--------|---------|---------|------------------------|")

for f in artifacts:
    if not os.path.exists(f):
        continue
    
    with open(f, 'rb') as file:
        content = file.read()
        h = hashlib.sha256(content).hexdigest()
    
    shifts = "N/A"
    workers = "N/A"
    horizon = "N/A"
    is_input = "Unknown"
    
    if f.endswith('.json'):
        with open(f, 'r') as file:
            data = json.load(file)
            if "shifts" in data:
                shifts = len(data["shifts"])
            if "workers" in data:
                workers = len(data["workers"])
            if "assignments" in data:
                is_input = "OUTPUT (assignments found)"
                if shifts == "N/A":
                    shifts = len(data["assignments"])
            if "scenario" in data and "planning_horizon_hours" in data["scenario"]:
                horizon = data["scenario"]["planning_horizon_hours"]
            elif "planning_horizon_hours" in data:
                horizon = data["planning_horizon_hours"]
            
            if "shifts" in data and "workers" in data and "assignments" not in data:
                is_input = "INPUT"
    elif f.endswith('.csv'):
        with open(f, 'r') as file:
            lines = file.readlines()
            if 'shifts' in f:
                shifts = len(lines) - 1 # header
                is_input = "INPUT (CSV)"
            elif 'workers' in f:
                workers = len(lines) - 1
                is_input = "INPUT (CSV)"

    match = "No"
    if str(shifts) == "770" and str(workers) == "33":
        match = "Yes"

    print(f"| `{f}` | {is_input} | {shifts} | {workers} | {horizon} | {match} |")

