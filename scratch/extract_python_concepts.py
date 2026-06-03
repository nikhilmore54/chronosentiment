import os
import ast
import json

def extract_concepts():
    concepts = set()
    for root, dirs, files in os.walk('.'):
        if 'target' in root or '.git' in root or '.venv' in root:
            continue
        for file in files:
            if file.endswith('.py'):
                path = os.path.join(root, file)
                try:
                    with open(path, 'r', encoding='utf-8') as f:
                        tree = ast.parse(f.read())
                        for node in ast.walk(tree):
                            if isinstance(node, ast.ClassDef):
                                concepts.add(node.name)
                except Exception as e:
                    pass
    
    print(json.dumps(sorted(list(concepts)), indent=2))

if __name__ == '__main__':
    extract_concepts()
