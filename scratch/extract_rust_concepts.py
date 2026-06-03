import os
import re
import json

def extract_concepts():
    concepts = set()
    pattern = re.compile(r'\b(struct|enum|trait|type)\s+([A-Z][a-zA-Z0-9_]*)\b')
    for root, dirs, files in os.walk('.'):
        if 'target' in root or '.git' in root or '.venv' in root:
            continue
        for file in files:
            if file.endswith('.rs'):
                path = os.path.join(root, file)
                try:
                    with open(path, 'r', encoding='utf-8') as f:
                        for line in f:
                            match = pattern.search(line)
                            if match:
                                concepts.add(match.group(2))
                except Exception as e:
                    pass
    
    print(json.dumps(sorted(list(concepts)), indent=2))

if __name__ == '__main__':
    extract_concepts()
