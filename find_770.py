import os
import json

for root, dirs, files in os.walk('.'):
    if '.git' in root or 'node_modules' in root or 'target' in root or 'yahoo_cache' in root or 'time_machine' in root or 'infrastructure/core/elite' in root:
        continue
    for file in files:
        if file.endswith('.json'):
            path = os.path.join(root, file)
            try:
                with open(path, 'r', encoding='utf-8') as f:
                    data = json.load(f)
                    
                    if isinstance(data, dict):
                        for k, v in data.items():
                            if isinstance(v, list) and len(v) == 770:
                                print(f"Found 770 elements in {k} of {path}")
                            if k == "total_shifts" and v == 770:
                                print(f"Found total_shifts=770 in {path}")
                            if k == "shifts" and len(v) == 770:
                                print(f"Found 770 shifts in {path}")
            except Exception:
                pass
