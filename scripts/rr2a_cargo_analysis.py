#!/usr/bin/env python3
"""
RR2-A: Cargo Analysis
Enumerates every workspace member Cargo.toml and extracts:
- Package name
- Workspace membership
- Library target (src/lib.rs or [lib])
- Binary targets ([[bin]] entries + implicit src/bin/*.rs)
- Example targets ([[example]])
- Bench targets ([[bench]])
- Test targets ([[test]])
- Dependencies (direct)

Output: docs/governance/rr2a_cargo_analysis.csv
"""

import os
import csv
import glob
import re

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
]

NON_WORKSPACE = [
    "coralys-policy",
    "research_experiments",
]

def read_toml_raw(path):
    try:
        with open(path, "r") as f:
            return f.read()
    except Exception:
        return ""

def extract_package_name(content):
    m = re.search(r'^\s*name\s*=\s*"([^"]+)"', content, re.MULTILINE)
    return m.group(1) if m else "unknown"

def count_section(content, header):
    return len(re.findall(r'^\s*' + re.escape(header), content, re.MULTILINE))

def extract_bin_names(content):
    """Extract [[bin]] name entries."""
    names = re.findall(r'\[\[bin\]\][^\[]*?name\s*=\s*"([^"]+)"', content, re.DOTALL)
    return names

def extract_deps(content):
    """Extract direct dependency names from [dependencies] section."""
    in_deps = False
    deps = []
    for line in content.splitlines():
        stripped = line.strip()
        if stripped.startswith("[dependencies]"):
            in_deps = True
            continue
        if stripped.startswith("[") and in_deps:
            in_deps = False
        if in_deps and stripped and not stripped.startswith("#"):
            m = re.match(r'^([a-zA-Z0-9_\-]+)\s*[=\.]', stripped)
            if m:
                deps.append(m.group(1))
    return deps

def get_implicit_bins(member_path):
    """Get .rs files in src/bin/ (implicit binary targets)."""
    bin_dir = os.path.join(WORKSPACE_ROOT, member_path, "src", "bin")
    if not os.path.isdir(bin_dir):
        return []
    files = []
    for f in os.listdir(bin_dir):
        if f.endswith(".rs"):
            files.append(f[:-3])  # strip .rs
    return sorted(files)

def get_deprecated_bins(member_path):
    """Get .rs files in src/bin/deprecated/."""
    dep_dir = os.path.join(WORKSPACE_ROOT, member_path, "src", "bin", "deprecated")
    if not os.path.isdir(dep_dir):
        return []
    files = []
    for f in os.listdir(dep_dir):
        if f.endswith(".rs"):
            files.append(f[:-3])
    return sorted(files)

def has_lib(member_path, content):
    lib_rs = os.path.join(WORKSPACE_ROOT, member_path, "src", "lib.rs")
    explicit = bool(re.search(r'^\s*\[lib\]', content, re.MULTILINE))
    return os.path.exists(lib_rs) or explicit

def has_main(member_path):
    return os.path.exists(os.path.join(WORKSPACE_ROOT, member_path, "src", "main.rs"))

rows = []

all_members = [(m, "Member") for m in WORKSPACE_MEMBERS] + [(m, "Orphan") for m in NON_WORKSPACE]

for member, ws_status in all_members:
    toml_path = os.path.join(WORKSPACE_ROOT, member, "Cargo.toml")
    if not os.path.exists(toml_path):
        rows.append({
            "member": member,
            "package": "MISSING",
            "workspace_status": ws_status,
            "has_lib": False,
            "has_main": False,
            "explicit_bins": 0,
            "explicit_bin_names": "",
            "implicit_bins": 0,
            "implicit_bin_names": "",
            "deprecated_bins": 0,
            "deprecated_bin_names": "",
            "examples": 0,
            "benches": 0,
            "tests_declared": 0,
            "direct_deps": "",
            "notes": "Cargo.toml missing",
        })
        continue

    content = read_toml_raw(toml_path)
    pkg = extract_package_name(content)
    explicit_bin_names = extract_bin_names(content)
    implicit_bin_names = get_implicit_bins(member)
    deprecated_bin_names = get_deprecated_bins(member)
    deps = extract_deps(content)

    rows.append({
        "member": member,
        "package": pkg,
        "workspace_status": ws_status,
        "has_lib": has_lib(member, content),
        "has_main": has_main(member),
        "explicit_bins": len(explicit_bin_names),
        "explicit_bin_names": "|".join(explicit_bin_names),
        "implicit_bins": len(implicit_bin_names),
        "implicit_bin_names": "|".join(implicit_bin_names),
        "deprecated_bins": len(deprecated_bin_names),
        "deprecated_bin_names": "|".join(deprecated_bin_names),
        "examples": count_section(content, "[[example]]"),
        "benches": count_section(content, "[[bench]]"),
        "tests_declared": count_section(content, "[[test]]"),
        "direct_deps": "|".join(deps),
        "notes": "",
    })

out_path = os.path.join(WORKSPACE_ROOT, "docs", "governance", "rr2a_cargo_analysis.csv")
fieldnames = [
    "member", "package", "workspace_status",
    "has_lib", "has_main",
    "explicit_bins", "explicit_bin_names",
    "implicit_bins", "implicit_bin_names",
    "deprecated_bins", "deprecated_bin_names",
    "examples", "benches", "tests_declared",
    "direct_deps", "notes",
]

with open(out_path, "w", newline="") as f:
    writer = csv.DictWriter(f, fieldnames=fieldnames)
    writer.writeheader()
    writer.writerows(rows)

print(f"Written {len(rows)} rows to {out_path}")

# Print summary to stdout for review
print()
print(f"{'Member':<45} {'Package':<35} {'WS':<8} {'lib':<5} {'main':<5} {'bins':<5} {'impl':<5} {'dep':<5}")
print("-" * 130)
for r in rows:
    print(f"{r['member']:<45} {r['package']:<35} {r['workspace_status']:<8} "
          f"{str(r['has_lib']):<5} {str(r['has_main']):<5} "
          f"{r['explicit_bins']:<5} {r['implicit_bins']:<5} {r['deprecated_bins']:<5}")
