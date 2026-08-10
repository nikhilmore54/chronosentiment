import os
import re
import csv
import json
import hashlib
from collections import defaultdict
from pathlib import Path

# --- Phase 0: Scope ---
EXCLUDE_DIRS = {
    'target', 'node_modules', '.venv', 'venv', '__pycache__', 
    '.git', 'archive', 'vendor', 'third_party', 'dist', 'build', 
    'coverage', '.gemini', '.idea'
}
EXCLUDE_PREFIXES = ('.baseline', 'test_release')

def is_excluded(path_str):
    parts = Path(path_str).parts
    for p in parts:
        if p in EXCLUDE_DIRS or any(p.startswith(prefix) for prefix in EXCLUDE_PREFIXES):
            return True
    return False

# --- Phase 0.5: Topology ---
TOPOLOGY = {
    'coralys-moga': {'Purpose': 'Core runtime', 'Owner': 'Coralys Core', 'Status': 'Active'},
    'adapters/ultracrew': {'Purpose': 'Airline domain', 'Owner': 'UltraCrew', 'Status': 'Active'},
    'adapters/roadef': {'Purpose': 'ROADEF adapter', 'Owner': 'ROADEF', 'Status': 'Active'},
    'adapters/chronosentiment': {'Purpose': 'Finance domain', 'Owner': 'ChronoSentiment', 'Status': 'Active'},
    'services/ultracrew_server': {'Purpose': 'UltraCrew Server', 'Owner': 'UltraCrew', 'Status': 'Active'},
    'services/cvrp_server': {'Purpose': 'CVRP Server', 'Owner': 'Coralys Research', 'Status': 'Active'},
    'apps': {'Purpose': 'User Interfaces', 'Owner': 'Coralys Core', 'Status': 'Active'},
    'docs': {'Purpose': 'Documentation', 'Owner': 'Coralys Core', 'Status': 'Active'},
}

def get_topology_info(filepath):
    for prefix, info in TOPOLOGY.items():
        if filepath.startswith(prefix):
            return info
    return {'Purpose': 'Unknown', 'Owner': 'Unknown', 'Status': 'Unknown'}

# --- Scanning & Grouping ---
def scan_repository(root_dir='.'):
    executables = []
    docs = []
    others = []
    for dirpath, dirnames, filenames in os.walk(root_dir):
        # Filter directories
        dirnames[:] = [d for d in dirnames if not is_excluded(os.path.join(dirpath, d))]
        for f in filenames:
            path = os.path.normpath(os.path.join(dirpath, f))
            if path.startswith('./'):
                path = path[2:]
            
            if f.endswith('.py') or f.endswith('.sh') or f.endswith('.ps1') or f.endswith('.bat'):
                executables.append(path)
            elif f.endswith('.md'):
                docs.append(path)
            elif f in ['Makefile', 'Cargo.toml', 'package.json'] or f.endswith('.yml') or f.endswith('.yaml'):
                others.append(path)
    return executables, docs, others

def hash_file(filepath):
    try:
        with open(filepath, 'rb') as f:
            return hashlib.md5(f.read()).hexdigest()
    except Exception:
        return None

# --- Dependency Parsing ---
def parse_imports(filepath):
    imports = set()
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            for line in f:
                if line.startswith('import '):
                    imports.add(line.split()[1].split('.')[0])
                elif line.startswith('from '):
                    imports.add(line.split()[1].split('.')[0])
    except Exception:
        pass
    return list(imports)

def extract_markdown_links(filepath):
    links = []
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()
            # simple markdown link regex
            matches = re.findall(r'\[([^\]]+)\]\(([^)]+)\)', content)
            for text, link in matches:
                # keep only internal links
                if not link.startswith('http') and not link.startswith('#'):
                    links.append(link)
    except Exception:
        pass
    return links

def build_invocation_graph(executables, search_files):
    # Mapping executable path -> list of files that invoke/reference it
    invocations = defaultdict(list)
    basenames = {os.path.basename(ex): ex for ex in executables}
    
    for sf in search_files:
        try:
            with open(sf, 'r', encoding='utf-8') as f:
                content = f.read()
            for base, ex_path in basenames.items():
                if base in content:
                    invocations[ex_path].append(sf)
        except Exception:
            continue
    return invocations

def main():
    print("Phase 0 & 1: Scanning repository...")
    executables, docs, others = scan_repository()
    
    print("Phase 2: Analyzing Artifacts...")
    invocations = build_invocation_graph(executables, others + docs + executables)
    
    # Analyze Executables
    exe_inventory = []
    exe_hashes = defaultdict(list)
    
    for i, ex in enumerate(executables):
        topo = get_topology_info(ex)
        h = hash_file(ex)
        if h: exe_hashes[h].append(ex)
        
        is_primary = False
        try:
            with open(ex, 'r', encoding='utf-8') as f:
                content = f.read()
                if '__main__' in content or ex.endswith('.sh') or ex.endswith('.bat'):
                    is_primary = True
        except: pass
        
        imports = parse_imports(ex)
        invoked_by = invocations.get(ex, [])
        
        # Categorization heuristics
        name = os.path.basename(ex).lower()
        cat = "Unknown"
        if 'test' in name: cat = 'Experiment'
        elif 'benchmark' in name or 'perf' in name: cat = 'Benchmarking'
        elif 'cleanup' in name or 'migrate' in name or 'convert' in name: cat = 'Maintenance'
        elif 'audit' in name or 'validate' in name or 'check' in name: cat = 'Diagnostics'
        elif 'generate' in name or 'build' in name: cat = 'Generator'
        elif 'analysis' in name or 'analyze' in name: cat = 'Analysis'
        
        evidence = []
        if cat != "Unknown": evidence.append(f"Filename matched {cat}")
        if invoked_by: evidence.append(f"Invoked by {len(invoked_by)} files")
        
        confidence = "Low"
        if invoked_by and cat != "Unknown":
            confidence = "High"
        elif invoked_by or cat != "Unknown":
            confidence = "Medium"
            
        exe_inventory.append({
            'Artifact_ID': f"ART-{i+1:04d}",
            'Path': ex,
            'Is_Primary': is_primary,
            'Category': cat,
            'Owner': topo['Owner'],
            'Lifecycle': 'Active' if invoked_by else 'Candidate for Removal',
            'Confidence': confidence,
            'Evidence': "; ".join(evidence),
            'Imports': imports,
            'Invoked_By': invoked_by
        })
        
    # Analyze Docs
    doc_inventory = []
    doc_links = {}
    
    for i, doc in enumerate(docs):
        topo = get_topology_info(doc)
        links = extract_markdown_links(doc)
        doc_links[doc] = links
        
        name = os.path.basename(doc).lower()
        cat = "Research"
        if 'arch-' in name: cat = 'Architecture'
        elif 'contract' in name: cat = 'Contract'
        elif 'spec' in name: cat = 'Specification'
        elif 'guide' in name or 'readme' in name: cat = 'Guide'
        elif 'report' in name: cat = 'Report'
        
        doc_inventory.append({
            'Doc_ID': f"DOC-{i+1:04d}",
            'Path': doc,
            'Type': cat,
            'Layer': 'TBD',
            'Status': 'Active Research',
            'Owner': topo['Owner'],
            'Canonical': 'Yes'
        })
        
    print("Phase 3 & 4: Generating Output Reports...")
    os.makedirs('reports', exist_ok=True)
    
    # 1. repository_inventory.csv
    with open('reports/repository_inventory.csv', 'w', newline='', encoding='utf-8') as f:
        writer = csv.writer(f)
        writer.writerow(['ID', 'Path', 'Type', 'Category', 'Owner', 'Lifecycle', 'Confidence', 'Evidence'])
        for item in exe_inventory:
            writer.writerow([item['Artifact_ID'], item['Path'], 'Executable', item['Category'], item['Owner'], item['Lifecycle'], item['Confidence'], item['Evidence']])
        for item in doc_inventory:
            writer.writerow([item['Doc_ID'], item['Path'], 'Document', item['Type'], item['Owner'], item['Status'], 'High', 'Extracted from path'])
            
    # 2. repository_summary.md
    with open('reports/repository_summary.md', 'w', encoding='utf-8') as f:
        f.write("# Repository Summary\n\n## Executables by Category\n")
        counts = defaultdict(int)
        for item in exe_inventory: counts[item['Category']] += 1
        for k, v in sorted(counts.items()): f.write(f"- {k}: {v}\n")
            
        f.write("\n## Documents by Type\n")
        dcounts = defaultdict(int)
        for item in doc_inventory: dcounts[item['Type']] += 1
        for k, v in sorted(dcounts.items()): f.write(f"- {k}: {v}\n")
            
    # 3. repository_metrics.md
    duplicates = [paths for h, paths in exe_hashes.items() if len(paths) > 1]
    orphans = sum(1 for item in exe_inventory if item['Lifecycle'] == 'Candidate for Removal')
    with open('reports/repository_metrics.md', 'w', encoding='utf-8') as f:
        f.write("# Repository Health Metrics\n\n")
        f.write(f"| Metric | Value |\n|---|---|\n")
        f.write(f"| Total Executables | {len(exe_inventory)} |\n")
        f.write(f"| Total Documents | {len(doc_inventory)} |\n")
        f.write(f"| Orphaned Executables | {orphans} |\n")
        f.write(f"| Duplicate Script Groups | {len(duplicates)} |\n")
        
    # 4. JSON Graphs
    with open('reports/dependency_graph.json', 'w', encoding='utf-8') as f:
        json.dump({'imports': {item['Artifact_ID']: item['Imports'] for item in exe_inventory}, 
                   'invocations': {item['Artifact_ID']: item['Invoked_By'] for item in exe_inventory}}, f, indent=2)
                   
    with open('reports/documentation_graph.json', 'w', encoding='utf-8') as f:
        json.dump(doc_links, f, indent=2)
        
    # 5. review_candidates.md
    with open('reports/review_candidates.md', 'w', encoding='utf-8') as f:
        f.write("# Review Candidates\n\n## Low Confidence Executables\n")
        f.write("| ID | Path | Hint Category | Evidence |\n|---|---|---|---|\n")
        for item in exe_inventory:
            if item['Confidence'] == 'Low':
                f.write(f"| {item['Artifact_ID']} | `{item['Path']}` | {item['Category']} | {item['Evidence']} |\n")
        
        f.write("\n## Duplicate Script Groups\n")
        for paths in duplicates:
            f.write(f"- " + ", ".join(paths) + "\n")

    # 6. migration_plan.md
    with open('reports/migration_plan.md', 'w', encoding='utf-8') as f:
        f.write("# Proposed Migration Plan\n\n(Generated after human review of candidates)\n")

    print("Reports successfully generated in 'reports/' directory.")

if __name__ == '__main__':
    main()
