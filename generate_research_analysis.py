import json
import os
import re
import csv
from collections import defaultdict

# Load the 93 material docs
with open("material_docs_summary.txt", "r") as f:
    lines = f.readlines()
    paths = [l.split("PATH: ")[1].strip() for l in lines if l.startswith("PATH: ")]

docs = []
for p in paths:
    if not os.path.exists(p): continue
    with open(p, "r", encoding="utf-8") as f:
        content = f.read()
    title_match = re.search(r"^#\s+(.+)$", content, re.MULTILINE)
    title = title_match.group(1).strip() if title_match else os.path.basename(p)
    docs.append({
        "path": p,
        "title": title,
        "content": content,
        "lower": content.lower()
    })

def extract_section(doc, regex_list):
    content = doc["content"]
    lower = doc["lower"]
    for r in regex_list:
        # Try finding a header matching the term
        match = re.search(r"(?i)^#{2,4}\s*([^#\n]*?" + r + r"[^#\n]*)\n([\s\S]*?)(?=^#{2,4} |\Z)", content, re.MULTILINE)
        if match:
            text = match.group(2).strip()
            if len(text) > 20: return text[:200].replace('\n', ' ') + "..."
            
    # Try finding explicit sentence
    for r in regex_list:
        match = re.search(r"(?i)([^.]*" + r + r"[^.]*\.)", content)
        if match:
            return match.group(1).replace('\n', ' ').strip()
    return ""

def has_keywords(lower, keywords):
    return any(k in lower for k in keywords)

analysis = []

for d in docs:
    c = d["content"]
    cl = d["lower"]
    
    q1 = extract_section(d, ["research question", "objective", "problem statement", r"\?"])
    q2 = extract_section(d, ["hypothesis", "hypothesize", "conjecture"])
    q3 = extract_section(d, ["novelty", "contribution", "we introduce", "novel", "new method"])
    q4 = extract_section(d, ["methodology", "method", "approach", "proposed"])
    q5 = "Yes" if re.search(r"\$[^$]+\$|\$\$|\\begin\{equation\}", c) or "definition" in cl else "No"
    q6 = extract_section(d, ["experiment", "design", "setup", "configuration"])
    q7 = extract_section(d, ["dataset", "data", "benchmark", "instance", "roadef", "corpus"])
    q8 = extract_section(d, ["baseline", "control", "stand-aside"])
    q9 = extract_section(d, ["independent variable", "factor", "controlled"])
    q10 = extract_section(d, ["dependent variable", "metric", "objective", "fitness"])
    q11 = extract_section(d, ["ablation", "variant", "sensitivity analysis"])
    q12 = extract_section(d, ["results", "findings", "outcome", "performance"])
    
    # Results check (tables or numbers)
    has_results = bool(re.search(r"\|.*\|.*\|", c)) or has_keywords(cl, ["results", "findings", "gap:", "passed", "failed", "median gap"])
    
    q13 = "Yes" if has_keywords(cl, ["failed", "negative result", "worse than", "degraded", "did not improve"]) else "No"
    q14 = extract_section(d, ["limitation", "threats to validity", "future work"])
    q15 = "Yes" if has_keywords(cl, ["reproducibility", "seed:", "environment:", "hash:", "commit:", "provenance"]) else "No"
    
    is_gov = has_keywords(cl, ["constitution", "governance", "charter", "status: active", "law declaration"])
    is_orig = "Yes" if (has_results and "method" in cl) else "No"
    q16 = "Original Research" if (is_orig=="Yes" and not is_gov) else "Project/Governance"
    
    q17 = "Yes" if has_results else "No"
    
    # Extract links for supporting evidence
    links = re.findall(r"\[.*?\]\((.*?\.md)\)", c)
    q18 = ", ".join(set(links)) if links else "None"
    
    # Programme
    scores = {
        "A. Coralys evolutionary optimization": sum(cl.count(k) for k in ["evolution", "moga", "genetic", "crossover"]),
        "B. Coralys computational architecture": sum(cl.count(k) for k in ["coralys", "architecture", "engine", "scheduler", "platform"]),
        "C. Decision ecology / rule ecology": sum(cl.count(k) for k in ["ecology", "rule", "population", "diversity"]),
        "D. Historical replay methodology": sum(cl.count(k) for k in ["replay", "historical", "observatory", "ledger"]),
        "E. Temporal integrity / point-in-time decision research": sum(cl.count(k) for k in ["temporal", "point-in-time", "integrity", "chronology"]),
        "F. Prospective decision validation": sum(cl.count(k) for k in ["prospective", "validation", "execution", "live", "paper trading"]),
        "G. Evidence governance / provenance": sum(cl.count(k) for k in ["evidence", "governance", "provenance", "hash", "gate", "certification"]),
        "H. Decision intelligence": sum(cl.count(k) for k in ["decision", "intelligence", "value", "ai", "portfolio"])
    }
    best_prog = max(scores, key=scores.get)
    if scores[best_prog] < 2: best_prog = "I. Other / not research-paper material"
    q19 = best_prog
    
    # Role
    if q16 == "Original Research" and has_results:
        role = "PRIMARY contribution"
    elif has_results or "benchmark" in d["path"]:
        role = "SUPPORTING evidence"
    elif not is_gov and (q1 or q2 or q4):
        role = "BACKGROUND only"
    else:
        role = "NOT suitable for research paper"
        
    analysis.append({
        "path": d["path"],
        "title": d["title"],
        "q1": q1, "q2": q2, "q3": q3, "q4": q4, "q5": q5,
        "q6": q6, "q7": q7, "q8": q8, "q9": q9, "q10": q10,
        "q11": q11, "q12": q12, "q13": q13, "q14": q14, "q15": q15,
        "q16": q16, "q17": q17, "q18": q18, "q19": q19, "role": role
    })

# --- OUTPUT GENERATION ---

# 1. CSV
csv_path = "/Users/nikhil/.gemini/antigravity-ide/brain/56bf6219-8a81-4cb7-bf4a-84ff1f284672/research_analysis.csv"
with open(csv_path, "w", newline="", encoding="utf-8") as f:
    writer = csv.DictWriter(f, fieldnames=analysis[0].keys())
    writer.writeheader()
    writer.writerows(analysis)

# 2. Markdown research-source matrix
md_lines = ["# Research Source Matrix\n", "| Document | Programme | Role | Empirical? | Orig? |", "|---|---|---|---|---|"]
for a in analysis:
    md_lines.append(f"| {os.path.basename(a['path'])} | {a['q19'][:15]}... | {a['role']} | {a['q17']} | {a['q16']} |")

# 3. Grouped by programme
grouped = defaultdict(list)
for a in analysis: grouped[a['q19']].append(a)

md_lines.append("\n# Documents by Research Programme\n")
for prog, docs in sorted(grouped.items()):
    md_lines.append(f"## {prog}")
    for a in docs:
        md_lines.append(f"- **{a['title']}** ({a['role']})")

# 4. Actual empirical evidence
md_lines.append("\n# Empirical Evidence Documents\n")
for a in analysis:
    if a['q17'] == "Yes": md_lines.append(f"- {a['path']}")

# 5. Negative/failed
md_lines.append("\n# Negative or Failed Results\n")
for a in analysis:
    if a['q13'] == "Yes": md_lines.append(f"- {a['path']}")

# 6. Unsupported claims
md_lines.append("\n# Unsupported Claims (Research with no evidence)\n")
for a in analysis:
    if a['q16'] == "Original Research" and a['q17'] == "No":
        md_lines.append(f"- {a['path']}")

# 7. Duplicated/derivative
md_lines.append("\n# Duplicated or Derivative Material\n")
titles = [a['title'] for a in analysis]
seen = set()
for a in analysis:
    if a['title'] in seen: md_lines.append(f"- {a['path']} (Duplicate title: {a['title']})")
    seen.add(a['title'])

# Write markdown report
md_path = "/Users/nikhil/.gemini/antigravity-ide/brain/56bf6219-8a81-4cb7-bf4a-84ff1f284672/research_outputs.md"
with open(md_path, "w", encoding="utf-8") as f:
    f.write("\n".join(md_lines))

print("Completed processing.")
