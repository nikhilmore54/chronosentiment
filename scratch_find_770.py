import json, csv, os, glob

dirs_to_search = ['datasets', 'fixtures', 'archive', 'ui/src']

def check_json(file_path):
    try:
        with open(file_path, 'r') as f:
            data = json.load(f)
            if isinstance(data, dict):
                shifts = data.get('shifts', [])
                workers = data.get('workers', [])
                if len(shifts) == 770 or len(workers) == 33:
                    print(f"[JSON Match] {file_path} - shifts: {len(shifts)}, workers: {len(workers)}")
    except:
        pass

def check_csv(file_path):
    try:
        with open(file_path, 'r') as f:
            lines = f.readlines()
            # simple line count check
            if len(lines) == 770 or len(lines) == 771 or len(lines) == 33 or len(lines) == 34:
                print(f"[CSV Match] {file_path} - lines: {len(lines)}")
    except:
        pass

for d in dirs_to_search:
    for root, _, files in os.walk(d):
        for file in files:
            file_path = os.path.join(root, file)
            if file.endswith('.json'):
                check_json(file_path)
            elif file.endswith('.csv'):
                check_csv(file_path)
