import json
import hashlib

files = [
    "archive/datasets/sunair_schedule_output.json",
    "fixtures/demo/sunair_schedule.json",
    "fixtures/demo/sunair_demo.json",
    "fixtures/demo/sunair_report.json"
]

for f in files:
    print(f"=== {f} ===")
    try:
        with open(f, 'rb') as file:
            content = file.read()
            h = hashlib.sha256(content).hexdigest()
        with open(f, 'r') as file:
            data = json.load(file)
            print(f"SHA-256: {h}")
            if "shifts" in data:
                print(f"Shifts: {len(data['shifts'])}")
                if len(data['shifts']) > 0:
                    print(f"  First ID: {data['shifts'][0].get('id')} Last ID: {data['shifts'][-1].get('id')}")
            else:
                print("No 'shifts' array.")
            
            if "workers" in data:
                print(f"Workers: {len(data['workers'])}")
            else:
                print("No 'workers' array.")
            
            if "planning_horizon_hours" in data:
                print(f"Horizon: {data['planning_horizon_hours']}")
            elif "horizon_hours" in data:
                print(f"Horizon: {data['horizon_hours']}")
            elif "scenario" in data and "planning_horizon_hours" in data["scenario"]:
                print(f"Horizon: {data['scenario']['planning_horizon_hours']}")
            else:
                print("Horizon: Not found")
                
            if "assignments" in data:
                if isinstance(data['assignments'], list):
                    print(f"Assignments (list): {len(data['assignments'])}")
                elif isinstance(data['assignments'], dict):
                    print(f"Assignments (dict keys): {len(data['assignments'])}")
    except Exception as e:
        print(f"Error: {e}")
