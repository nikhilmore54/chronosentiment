#!/usr/bin/env python3
"""
RR2-B: Module Graph
For each workspace member, walks the module tree starting from lib.rs / main.rs
and all declared [[bin]] entry points. Determines:
- Which .rs files are compile-reachable (declared via `mod`)
- Which .rs files are orphaned (on disk but not reachable from any crate root)
- Which `mod` declarations reference missing files

Output: docs/governance/rr2b_module_graph.csv
"""

import os
import re
import csv

WORKSPACE_ROOT = "/Users/nikhil/ChronoSentiment_MEGA_FINAL"

WORKSPACE_MEMBERS = [
    "infrastructure/core",
    "infrastructure/optimization",
    "infrastructure/observatory/api",
    "financial/ese",
    "financial/strategies",
    "financial/core",
    "coralys-moga",
    "coralys-simulation",
    "coralys-ecology",
    "coralys-decision",
    "coralys-recommendation",
    "coralys-infrastructure",
    "adapters/ultracrew",
    "adapters/chronosentiment",
    "adapters/cvrp",
    "adapters/cvd001",
    "coralys-v2",
    "coralys-core",
    "coralys-eval",
    "coralys-matching",
    "adapters/roadef",
    "adapters/airline",
    "adapters/gerad",
    "coralys-planning",
    "services/ultracrew_server",
    "services/cvrp_server",
    "coralys-policy",  # orphan — include for completeness
]

def read_file(path):
    try:
        with open(path, "r", encoding="utf-8", errors="replace") as f:
            return f.read()
    except Exception:
        return ""

def extract_mod_declarations(content):
    """Extract module names from `mod foo;` and `pub mod foo;` declarations."""
    mods = re.findall(r'^\s*(?:pub\s+)?(?:pub\(crate\)\s+)?mod\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*;', content, re.MULTILINE)
    return mods

def resolve_mod_path(parent_file, mod_name, src_root):
    """
    Given a parent .rs file and a mod name, find the actual file.
    Rust module resolution:
    - If parent is foo/mod.rs or foo.rs, look for foo/bar.rs or foo/bar/mod.rs
    - If parent is src/lib.rs, look for src/bar.rs or src/bar/mod.rs
    """
    parent_dir = os.path.dirname(parent_file)
    parent_base = os.path.basename(parent_file)

    # If parent is mod.rs or lib.rs or main.rs, siblings are in same dir
    if parent_base in ("mod.rs", "lib.rs", "main.rs") or parent_base.endswith(".rs"):
        # Try sibling file: parent_dir/mod_name.rs
        candidate1 = os.path.join(parent_dir, mod_name + ".rs")
        # Try subdirectory: parent_dir/mod_name/mod.rs
        candidate2 = os.path.join(parent_dir, mod_name, "mod.rs")
        if os.path.exists(candidate1):
            return candidate1
        if os.path.exists(candidate2):
            return candidate2
        # For non-mod.rs parents: parent_dir/parent_stem/mod_name.rs
        parent_stem = parent_base[:-3]  # strip .rs
        if parent_stem not in ("lib", "main", "mod"):
            candidate3 = os.path.join(parent_dir, parent_stem, mod_name + ".rs")
            candidate4 = os.path.join(parent_dir, parent_stem, mod_name, "mod.rs")
            if os.path.exists(candidate3):
                return candidate3
            if os.path.exists(candidate4):
                return candidate4
    return None

def walk_module_tree(entry_file, src_root):
    """BFS walk of module tree from entry_file. Returns (reachable, missing_mods)."""
    reachable = set()
    missing_mods = []
    queue = [entry_file]
    visited = set()

    while queue:
        current = queue.pop(0)
        if current in visited:
            continue
        visited.add(current)

        if not os.path.exists(current):
            continue

        reachable.add(current)
        content = read_file(current)
        mods = extract_mod_declarations(content)

        for mod_name in mods:
            resolved = resolve_mod_path(current, mod_name, src_root)
            if resolved:
                if resolved not in visited:
                    queue.append(resolved)
            else:
                missing_mods.append((current, mod_name))

    return reachable, missing_mods

def get_all_rs_files(directory):
    """Get all .rs files under a directory."""
    result = []
    for root, dirs, files in os.walk(directory):
        # Skip target directory
        dirs[:] = [d for d in dirs if d != "target"]
        for f in files:
            if f.endswith(".rs"):
                result.append(os.path.join(root, f))
    return result

def get_entry_points(member_path, toml_content):
    """Get all entry point .rs files for a crate."""
    abs_path = os.path.join(WORKSPACE_ROOT, member_path)
    entries = []

    # lib.rs
    lib_rs = os.path.join(abs_path, "src", "lib.rs")
    if os.path.exists(lib_rs):
        entries.append(("lib", lib_rs))

    # main.rs
    main_rs = os.path.join(abs_path, "src", "main.rs")
    if os.path.exists(main_rs):
        entries.append(("binary:main", main_rs))

    # Explicit [[bin]] entries
    bin_entries = re.findall(
        r'\[\[bin\]\][^\[]*?path\s*=\s*"([^"]+)"',
        toml_content, re.DOTALL
    )
    for rel_path in bin_entries:
        abs_bin = os.path.join(abs_path, rel_path)
        if os.path.exists(abs_bin):
            entries.append(("binary:explicit", abs_bin))

    # Implicit src/bin/*.rs
    bin_dir = os.path.join(abs_path, "src", "bin")
    if os.path.isdir(bin_dir):
        for f in os.listdir(bin_dir):
            if f.endswith(".rs"):
                entries.append(("binary:implicit", os.path.join(bin_dir, f)))

    # Deprecated bins
    dep_dir = os.path.join(abs_path, "src", "bin", "deprecated")
    if os.path.isdir(dep_dir):
        for f in os.listdir(dep_dir):
            if f.endswith(".rs"):
                entries.append(("binary:deprecated", os.path.join(dep_dir, f)))

    # Test files in tests/
    tests_dir = os.path.join(abs_path, "tests")
    if os.path.isdir(tests_dir):
        for f in os.listdir(tests_dir):
            if f.endswith(".rs"):
                entries.append(("test", os.path.join(tests_dir, f)))

    return entries

rows = []

for member in WORKSPACE_MEMBERS:
    abs_member = os.path.join(WORKSPACE_ROOT, member)
    toml_path = os.path.join(abs_member, "Cargo.toml")
    toml_content = read_file(toml_path)
    src_root = os.path.join(abs_member, "src")

    # Get all .rs files on disk
    all_rs = set(get_all_rs_files(abs_member))

    # Get entry points
    entry_points = get_entry_points(member, toml_content)

    # Walk module tree from each entry point
    all_reachable = set()
    all_missing = []

    for target_type, entry_file in entry_points:
        reachable, missing = walk_module_tree(entry_file, src_root)
        all_reachable |= reachable
        all_missing.extend(missing)

    # Classify each .rs file
    for rs_file in sorted(all_rs):
        rel_path = os.path.relpath(rs_file, WORKSPACE_ROOT)

        # Determine target type
        if rs_file in all_reachable:
            # Find which entry point type reached it
            compile_status = "Reachable"
        else:
            compile_status = "Orphan"

        # Determine build classification
        rel_to_src = os.path.relpath(rs_file, abs_member)
        if rel_to_src.startswith("src/bin/deprecated/"):
            build_class = "binary:deprecated"
        elif rel_to_src.startswith("src/bin/"):
            build_class = "binary:implicit"
        elif rel_to_src == "src/lib.rs":
            build_class = "lib:root"
        elif rel_to_src == "src/main.rs":
            build_class = "binary:main"
        elif rel_to_src.startswith("src/"):
            build_class = "lib:module" if compile_status == "Reachable" else "lib:orphan"
        elif rel_to_src.startswith("tests/"):
            build_class = "test"
        elif rel_to_src.startswith("benches/"):
            build_class = "bench"
        elif rel_to_src.startswith("examples/"):
            build_class = "example"
        else:
            build_class = "unknown"

        rows.append({
            "path": rel_path,
            "member": member,
            "compile_status": compile_status,
            "build_classification": build_class,
        })

    # Record missing mod declarations
    for parent, mod_name in all_missing:
        rel_parent = os.path.relpath(parent, WORKSPACE_ROOT)
        rows.append({
            "path": f"MISSING: mod {mod_name} in {rel_parent}",
            "member": member,
            "compile_status": "Missing",
            "build_classification": "declared-but-missing",
        })

out_path = os.path.join(WORKSPACE_ROOT, "docs", "governance", "rr2b_module_graph.csv")
fieldnames = ["path", "member", "compile_status", "build_classification"]

with open(out_path, "w", newline="") as f:
    writer = csv.DictWriter(f, fieldnames=fieldnames)
    writer.writeheader()
    writer.writerows(rows)

print(f"Written {len(rows)} rows to {out_path}")

# Summary
from collections import Counter
status_counts = Counter(r["compile_status"] for r in rows)
class_counts = Counter(r["build_classification"] for r in rows)

print("\nCompile Status Summary:")
for k, v in sorted(status_counts.items()):
    print(f"  {k}: {v}")

print("\nBuild Classification Summary:")
for k, v in sorted(class_counts.items()):
    print(f"  {k}: {v}")

# Per-member orphan count
print("\nOrphaned files per member:")
orphan_by_member = Counter(r["member"] for r in rows if r["compile_status"] == "Orphan")
for member, count in sorted(orphan_by_member.items(), key=lambda x: -x[1]):
    print(f"  {member}: {count}")