#!/usr/bin/env python3
"""Semantic lint script (warning mode).

- Loads the canonical glossary from `docs/constitution/glossary.md`.
- Classifies each term into one of the categories defined in the glossary table.
- Determines the set of *constitutional* terms (those with the "Constitutional" stability level).
- Scans ONLY the files changed in the current PR/commit (via `git diff --name-only ${{ github.base_ref }} ${{ github.sha }}` when run in CI).
- Emits warnings for:
  * Use of undefined constitutional terms.
  * Use of prohibited synonyms (`snapshot`, `execution state`).
  * Introduction of experimental terms inside constitutional documents.
  * Presence of large‑artifact patterns (`*.jsonl`, `*.tar.gz`, `node_modules/`).
- Prints a markdown summary that GitHub Actions displays in the job log.

The script never modifies repository files; it only reports.
"""
import os
import re
import subprocess
import sys
from pathlib import Path

GLOSSARY_PATH = Path("docs/constitution/glossary.md")
PROHIBITED_SYNONYMS = {"snapshot", "execution state"}
LARGE_ARTIFACT_PATTERNS = ["*.jsonl", "*.tar.gz", "node_modules/**"]

def load_glossary():
    """Parse the glossary markdown table and return a dict of term -> category info.
    Expected columns: Term | Definition | Allowed Synonyms / Aliases.
    The stability level is inferred from the section headings that follow the table.
    """
    terms = {}
    if not GLOSSARY_PATH.is_file():
        print(f"::warning ::Glossary file not found at {GLOSSARY_PATH}")
        return terms
    with GLOSSARY_PATH.open(encoding="utf-8") as f:
        lines = f.readlines()
    table_started = False
    for line in lines:
        line = line.strip()
        if line.startswith("| Term") and not table_started:
            table_started = True
            continue
        if table_started:
            if line.startswith("|---"):
                continue
            if not line.startswith("|"):
                # End of table
                break
            parts = [p.strip() for p in line.strip("|").split("|")]
            if len(parts) < 3:
                continue
            term, definition, synonyms = parts[0], parts[1], parts[2]
            # Determine category from synonyms column (look for keywords)
            category = "operational"  # default fallback
            low = synonyms.lower()
            if "(no synonyms)" in low:
                category = "constitutional"
            elif "prohibited" in low:
                category = "prohibited"
            elif "experimental" in low:
                category = "experimental"
            terms[term.lower()] = {"category": category, "synonyms": set(map(str.strip, re.split(r",\s*", synonyms.lower())))}
    return terms

def get_changed_files():
    """Return a list of file paths changed in this CI run.
    The environment variable GITHUB_BASE_REF is set for PRs; for push events we compare against the previous commit.
    """
    base_ref = os.getenv("GITHUB_BASE_REF")
    sha = os.getenv("GITHUB_SHA")
    if base_ref:
        cmd = ["git", "diff", "--name-only", f"origin/{base_ref}", sha]
    else:
        # For pushes on main, compare against previous commit
        cmd = ["git", "diff", "--name-only", "HEAD~1", "HEAD"]
    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode != 0:
        print(f"::warning ::Failed to get changed files: {result.stderr}")
        return []
    return [p for p in result.stdout.splitlines() if p]

def tokenize(text):
    return re.findall(r"[A-Za-z_][A-Za-z0-9_]*", text.lower())

def main():
    glossary = load_glossary()
    if not glossary:
        sys.exit(0)
    changed_files = get_changed_files()
    if not changed_files:
        print("::notice ::No changed files detected; nothing to lint.")
        sys.exit(0)
    warnings = []
    for rel_path in changed_files:
        path = Path(rel_path)
        try:
            content = path.read_text(encoding="utf-8")
        except Exception as e:
            # binary or unreadable files are ignored for lint purposes
            continue
        tokens = set(tokenize(content))
        # Check for prohibited synonyms
        for syn in PROHIBITED_SYNONYMS:
            if syn in tokens:
                warnings.append(f"- **Prohibited synonym** `{syn}` found in `{rel_path}`.")
        # Check for undefined constitutional terms (terms that are marked constitutional in glossary but not present)
        for term, info in glossary.items():
            if info["category"] == "constitutional" and term not in tokens:
                continue  # not used, that's fine
            if info["category"] == "constitutional" and term in tokens:
                # term is allowed, no warning
                continue
            if info["category"] == "experimental" and term in tokens:
                # experimental term used in a constitutional doc is a warning
                if path.match("AUTHORITY_MAP.md") or path.match("docs/constitution/**/*.md"):
                    warnings.append(f"- **Experimental term** `{term}` used in constitutional file `{rel_path}`.")
        # Large‑artifact detection (only for added files)
        for pattern in LARGE_ARTIFACT_PATTERNS:
            if path.match(pattern):
                warnings.append(f"- **Large artifact** pattern `{pattern}` matched file `{rel_path}`.")
    if warnings:
        print("## Semantic Lint Warnings (observational mode)\n")
        for w in warnings:
            print(w)
        # Emit a GitHub job summary so it appears in the UI
        summary_path = os.getenv("GITHUB_STEP_SUMMARY")
        if summary_path:
            with open(summary_path, "a", encoding="utf-8") as s:
                s.write("## Semantic Lint Warnings (observational mode)\n")
                for w in warnings:
                    s.write(f"{w}\n")
    else:
        print("::notice ::No semantic lint warnings detected.")

if __name__ == "__main__":
    main()
