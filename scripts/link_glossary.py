#!/usr/bin/env python3
"""One‑off script to replace local definitions of constitutional terms with glossary links.

Usage (run from the repository root)::
    python scripts/link_glossary.py

It reads the glossary table in `docs/constitution/glossary.md`, builds a map of term → anchor
(e.g., `authority` → `#authority`), then scans the target files listed in `TARGETS` and rewrites
any plain occurrence of a term that is not already a markdown link.

The script is *observational* – it prints a diff‑style summary of changes and writes the files
in‑place. It does **not** commit automatically; you should review the PR before merging.
"""
import re
from pathlib import Path

GLOSSARY_PATH = Path("docs/constitution/glossary.md")
TARGETS = [
    Path("AUTHORITY_MAP.md"),
    Path("docs/constitution/architecture.md"),
    Path("docs/constitution/topology.md"),
]

def load_glossary():
    terms = {}
    if not GLOSSARY_PATH.is_file():
        print(f"Glossary not found at {GLOSSARY_PATH}")
        return terms
    with GLOSSARY_PATH.open(encoding="utf-8") as f:
        lines = f.readlines()
    in_table = False
    for line in lines:
        line = line.strip()
        if line.startswith("| Term") and not in_table:
            in_table = True
            continue
        if in_table:
            if line.startswith("|---"):
                continue
            if not line.startswith("|"):
                break
            cols = [c.strip() for c in line.strip("|").split("|")]
            if len(cols) >= 1:
                term = cols[0]
                # Generate markdown anchor (GitHub style – lower‑case, hyphens, remove punctuation)
                anchor = re.sub(r"[^a-z0-9\-]", "", term.lower().replace(" ", "-"))
                terms[term] = f"{term}(../glossary.md#{anchor})"
    return terms

def replace_in_file(path: Path, term_map: dict):
    original = path.read_text(encoding="utf-8")
    new_content = original
    changes = []
    for term, link in term_map.items():
        # regex to match the term as a whole word not already inside a markdown link
        pattern = rf"(?<!\[){re.escape(term)}(?!\])"
        if re.search(pattern, new_content):
            new_content, cnt = re.subn(pattern, f"[{link}]", new_content)
            if cnt:
                changes.append((term, cnt))
    if changes:
        path.write_text(new_content, encoding="utf-8")
        print(f"Updated {path}:")
        for term, cnt in changes:
            print(f"  - {term}: {cnt} replacement(s)")
    else:
        print(f"No changes needed in {path}")

def main():
    term_map = load_glossary()
    if not term_map:
        print("No terms loaded; aborting.")
        return
    for target in TARGETS:
        if target.is_file():
            replace_in_file(target, term_map)
        else:
            print(f"Target file {target} does not exist; skipping.")

if __name__ == "__main__":
    main()
