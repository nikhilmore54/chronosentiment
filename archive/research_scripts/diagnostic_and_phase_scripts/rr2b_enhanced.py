#!/usr/bin/env python3
"""
RR2-B Enhanced: Module Graph with Files + Edge List
Extends rr2b_module_graph.py to emit two additional artifacts:

1. rr2b_files.csv — one row per .rs file:
   path, member, package, compilation_root_type, compilation_root_file,
   parent_module, module_path, reachable, reachability_root,
   incoming_edges, outgoing_edges

2. rr2b_module_edges.csv — parent→child edge list:
   parent_path, child_path, member, mod_name

A file is only considered truly orphaned if it is absent from ALL
valid compilation roots (lib, main, bin, test, example, bench).
"""

import os
import re
import csv
from collections import defaultdict

WORKSPACE_ROOT = "/Users/nikhil/ChronoSentiment_MEGA_FINAL"

WORKSPACE_MEMBERS = [
    ("infrastructure/core",             "chronosentiment_core"),
    ("infrastructure/optimization",     "chronosentiment_optimization"),
    ("infrastructure/observatory/api",  "api"),
    ("financial/ese",                   "cs-ingest"),
    ("financial/strategies",            "chronosentiment_strategies"),
    ("financial/core",                  "chronosentiment_financial_core"),
    ("coralys-moga",                    "coralys-moga"),
    ("coralys-simulation",              "coralys-simulation"),
    ("coralys-ecology",                 "coralys-ecology"),
    ("coralys-decision",                "coralys-decision"),
    ("coralys-recommendation",          "coralys-recommendation"),
    ("coralys-infrastructure",          "coralys-infrastructure"),
    ("adapters/ultracrew",              "ultracrew"),
    ("adapters/chronosentiment",        "chronosentiment_adapter"),
    ("adapters/cvrp",                   "cvrp"),
    ("adapters/cvd001",                 "cvd001"),
    ("coralys-v2",                      "coralys-v2"),
    ("coralys-core",                    "coralys-core"),
    ("coralys-eval",                    "coralys-eval"),
    ("coralys-matching",                "coralys-matching"),
    ("adapters/roadef",                 "roadef"),
    ("adapters/airline",                "coralys-airline"),
    ("adapters/gerad",                  "coralys-gerad"),
    ("coralys-planning",                "coralys-planning"),
    ("services/ultracrew_server",       "ultracrew_server"),
    ("services/cvrp_server",            "cvrp_server"),
    ("coralys-policy",                  "coralys-policy"),
]

def read_file(path):
    try:
        with open(path, "r", encoding="utf-8", errors="replace") as f:
            return f.read()
    except Exception:
        return ""

def extract_mod_declarations(content):
    return re.findall(
        r'^\s*(?:pub\s+)?(?:pub\s*\([^)]+\)\s+)?mod\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*;',
        content, re.MULTILINE
    )

def resolve_mod_path(parent_file, mod_name):
    parent_dir = os.path.dirname(parent_file)
    parent_base = os.path.basename(parent_file)
    parent_stem = parent_base[:-3]  # strip .rs

    # Standard resolution: sibling file or subdirectory/mod.rs
    candidate1 = os.path.join(parent_dir, mod_name + ".rs")
    candidate2 = os.path.join(parent_dir, mod_name, "mod.rs")
    if os.path.exists(candidate1):
        return candidate1
    if os.path.exists(candidate2):
        return candidate2

    # For non-root files: parent_dir/parent_stem/mod_name.rs
    if parent_stem not in ("lib", "main", "mod"):
        candidate3 = os.path.join(parent_dir, parent_stem, mod_name + ".rs")
        candidate4 = os.path.join(parent_dir, parent_stem, mod_name, "mod.rs")
        if os.path.exists(candidate3):
            return candidate3
        if os.path.exists(candidate4):
            return candidate4
    return None

def build_module_tree(entry_file, root_type, root_label):
    """
    BFS from entry_file. Returns:
    - file_info: dict[abs_path] -> {reachable, reachability_root, root_type,
                                     parent, module_path, outgoing_edges}
    - edges: list of (parent_abs, child_abs, mod_name)
    """
    file_info = {}
    edges = []
    queue = [(entry_file, None, "crate")]  # (file, parent, module_path)
    visited = set()

    while queue:
        current, parent, mod_path = queue.pop(0)
        if current in visited:
            continue
        visited.add(current)

        if not os.path.exists(current):
            continue

        file_info[current] = {
            "reachable": True,
            "reachability_root": root_label,
            "root_type": root_type,
            "parent": parent,
            "module_path": mod_path,
            "outgoing_edges": 0,
        }

        content = read_file(current)
        mods = extract_mod_declarations(content)
        file_info[current]["outgoing_edges"] = len(mods)

        for mod_name in mods:
            resolved = resolve_mod_path(current, mod_name)
            if resolved:
                edges.append((current, resolved, mod_name))
                if resolved not in visited:
                    child_path = mod_path + "::" + mod_name if mod_path != "crate" else "crate::" + mod_name
                    queue.append((resolved, current, child_path))

    return file_info, edges

def get_all_rs_files(member_abs):
    result = []
    for root, dirs, files in os.walk(member_abs):
        dirs[:] = [d for d in dirs if d not in ("target", ".git")]
        for f in files:
            if f.endswith(".rs"):
                result.append(os.path.join(root, f))
    return result

def get_compilation_roots(member, member_abs, toml_content):
    """Return list of (root_type, root_label, abs_path)."""
    roots = []

    lib_rs = os.path.join(member_abs, "src", "lib.rs")
    if os.path.exists(lib_rs):
        roots.append(("lib", "lib", lib_rs))

    main_rs = os.path.join(member_abs, "src", "main.rs")
    if os.path.exists(main_rs):
        roots.append(("binary:main", "main", main_rs))

    # Explicit [[bin]]
    blocks = re.split(r'\[\[bin\]\]', toml_content)[1:]
    for block in blocks:
        block_body = re.split(r'\[', block)[0]
        name_m = re.search(r'name\s*=\s*"([^"]+)"', block_body)
        path_m = re.search(r'path\s*=\s*"([^"]+)"', block_body)
        if name_m:
            bin_name = name_m.group(1)
            if path_m:
                abs_bin = os.path.join(member_abs, path_m.group(1))
            else:
                abs_bin = os.path.join(member_abs, "src", "bin", bin_name + ".rs")
            if os.path.exists(abs_bin):
                roots.append(("binary:explicit", f"bin:{bin_name}", abs_bin))

    # Implicit src/bin/*.rs
    bin_dir = os.path.join(member_abs, "src", "bin")
    if os.path.isdir(bin_dir):
        for fname in sorted(os.listdir(bin_dir)):
            if fname.endswith(".rs"):
                fpath = os.path.join(bin_dir, fname)
                bin_name = fname[:-3]
                # Skip if already covered by explicit [[bin]]
                already = any(r[2] == fpath for r in roots)
                if not already:
                    roots.append(("binary:implicit", f"bin:{bin_name}", fpath))

    # Deprecated bins
    dep_dir = os.path.join(member_abs, "src", "bin", "deprecated")
    if os.path.isdir(dep_dir):
        for fname in sorted(os.listdir(dep_dir)):
            if fname.endswith(".rs"):
                fpath = os.path.join(dep_dir, fname)
                roots.append(("binary:deprecated", f"bin:deprecated:{fname[:-3]}", fpath))

    # tests/*.rs
    tests_dir = os.path.join(member_abs, "tests")
    if os.path.isdir(tests_dir):
        for fname in sorted(os.listdir(tests_dir)):
            if fname.endswith(".rs"):
                fpath = os.path.join(tests_dir, fname)
                roots.append(("test", f"test:{fname[:-3]}", fpath))

    # tests/harness/*.rs (sub-modules of test roots — not independent roots)
    # These are reached via mod declarations in tests/*.rs, not as roots themselves

    # examples/*.rs
    ex_dir = os.path.join(member_abs, "examples")
    if os.path.isdir(ex_dir):
        for fname in sorted(os.listdir(ex_dir)):
            if fname.endswith(".rs"):
                fpath = os.path.join(ex_dir, fname)
                roots.append(("example", f"example:{fname[:-3]}", fpath))

    # benches/*.rs
    bench_dir = os.path.join(member_abs, "benches")
    if os.path.isdir(bench_dir):
        for fname in sorted(os.listdir(bench_dir)):
            if fname.endswith(".rs"):
                fpath = os.path.join(bench_dir, fname)
                roots.append(("bench", f"bench:{fname[:-3]}", fpath))

    return roots

# ---- Main pass ----

all_file_rows = []   # for rr2b_files.csv
all_edge_rows = []   # for rr2b_module_edges.csv

for member, package in WORKSPACE_MEMBERS:
    member_abs = os.path.join(WORKSPACE_ROOT, member)
    toml_path = os.path.join(member_abs, "Cargo.toml")
    toml_content = read_file(toml_path)

    all_rs = set(get_all_rs_files(member_abs))
    roots = get_compilation_roots(member, member_abs, toml_content)

    # Walk each root
    combined_file_info = {}  # abs_path -> best info (first root that reaches it)
    combined_edges = []
    incoming_count = defaultdict(int)

    for root_type, root_label, root_file in roots:
        file_info, edges = build_module_tree(root_file, root_type, root_label)
        for abs_path, info in file_info.items():
            if abs_path not in combined_file_info:
                combined_file_info[abs_path] = info
        for parent, child, mod_name in edges:
            combined_edges.append((parent, child, mod_name))
            incoming_count[child] += 1

    # Emit edge rows
    for parent, child, mod_name in combined_edges:
        all_edge_rows.append({
            "parent_path": os.path.relpath(parent, WORKSPACE_ROOT),
            "child_path": os.path.relpath(child, WORKSPACE_ROOT),
            "member": member,
            "mod_name": mod_name,
        })

    # Emit file rows — all .rs files on disk
    for rs_file in sorted(all_rs):
        rel_path = os.path.relpath(rs_file, WORKSPACE_ROOT)
        if rs_file in combined_file_info:
            info = combined_file_info[rs_file]
            reachable = True
            root_type = info["root_type"]
            root_label = info["reachability_root"]
            parent = os.path.relpath(info["parent"], WORKSPACE_ROOT) if info["parent"] else ""
            module_path = info["module_path"] or ""
            outgoing = info["outgoing_edges"]
        else:
            reachable = False
            # Classify by path even if unreachable
            rel_to_member = os.path.relpath(rs_file, member_abs)
            if rel_to_member.startswith("src/bin/deprecated/"):
                root_type = "binary:deprecated"
            elif rel_to_member.startswith("src/bin/"):
                root_type = "binary:implicit"
            elif rel_to_member == "src/lib.rs":
                root_type = "lib"
            elif rel_to_member == "src/main.rs":
                root_type = "binary:main"
            elif rel_to_member.startswith("src/"):
                root_type = "lib:orphan"
            elif rel_to_member.startswith("tests/"):
                root_type = "test"
            elif rel_to_member.startswith("examples/"):
                root_type = "example"
            elif rel_to_member.startswith("benches/"):
                root_type = "bench"
            else:
                root_type = "unknown"
            root_label = ""
            parent = ""
            module_path = ""
            outgoing = 0

        all_file_rows.append({
            "path": rel_path,
            "member": member,
            "package": package,
            "compilation_root_type": root_type,
            "reachability_root": root_label,
            "parent_module": parent,
            "module_path": module_path,
            "reachable": reachable,
            "incoming_edges": incoming_count.get(rs_file, 0),
            "outgoing_edges": outgoing,
        })

# Write rr2b_files.csv
files_out = os.path.join(WORKSPACE_ROOT, "docs", "governance", "rr2b_files.csv")
files_fields = [
    "path", "member", "package", "compilation_root_type",
    "reachability_root", "parent_module", "module_path",
    "reachable", "incoming_edges", "outgoing_edges",
]
with open(files_out, "w", newline="") as f:
    writer = csv.DictWriter(f, fieldnames=files_fields)
    writer.writeheader()
    writer.writerows(all_file_rows)
print(f"Written {len(all_file_rows)} rows to {files_out}")

# Write rr2b_module_edges.csv
edges_out = os.path.join(WORKSPACE_ROOT, "docs", "governance", "rr2b_module_edges.csv")
edges_fields = ["parent_path", "child_path", "member", "mod_name"]
with open(edges_out, "w", newline="") as f:
    writer = csv.DictWriter(f, fieldnames=edges_fields)
    writer.writeheader()
    writer.writerows(all_edge_rows)
print(f"Written {len(all_edge_rows)} rows to {edges_out}")

# Summary
from collections import Counter
reachable_count = sum(1 for r in all_file_rows if r["reachable"])
orphan_count = sum(1 for r in all_file_rows if not r["reachable"])
print(f"\nTotal files: {len(all_file_rows)}")
print(f"  Reachable: {reachable_count}")
print(f"  Orphan:    {orphan_count}")

root_type_counts = Counter(r["compilation_root_type"] for r in all_file_rows)
print("\nBy compilation root type:")
for k, v in sorted(root_type_counts.items(), key=lambda x: -x[1]):
    print(f"  {k}: {v}")

print("\nOrphaned lib modules by member:")
orphan_lib = [r for r in all_file_rows if not r["reachable"] and r["compilation_root_type"] == "lib:orphan"]
by_member = Counter(r["member"] for r in orphan_lib)
for member, count in sorted(by_member.items(), key=lambda x: -x[1]):
    files = [r["path"] for r in orphan_lib if r["member"] == member]
    for f in files:
        print(f"  {f}")

print(f"\nTotal edges: {len(all_edge_rows)}")