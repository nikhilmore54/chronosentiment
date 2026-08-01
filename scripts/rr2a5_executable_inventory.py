#!/usr/bin/env python3
"""
RR2-A.5: Executable Inventory
For every binary entry point (main.rs, src/bin/*.rs, explicit [[bin]]),
records:
- Binary name
- Owning crate / package
- Workspace status
- Entry point path
- LOC
- Purpose classification (Production/Benchmark/Experiment/Validation/Tool/Migration/Deprecated/Unknown)
- Classification rationale (naming pattern used)
- Deprecated flag

Purpose classification heuristics (in priority order):
1. In src/bin/deprecated/ -> Deprecated
2. Name matches m[0-9]+_ prefix -> Experiment (milestone series)
3. Name matches bdd_ prefix -> Benchmark (BDD-style)
4. Name contains benchmark/bench -> Benchmark
5. Name contains validation/validate/acceptance/governance -> Validation
6. Name contains campaign -> Tool (campaign runner)
7. Name contains sanity/check/verify/audit/forensics -> Validation
8. Name contains pilot/telemetry/observatory -> Experiment
9. Name contains cli/server/ingest -> Production
10. Name contains migration/migrate -> Migration
11. main.rs in a service crate -> Production
12. main.rs in a library crate -> Unknown (unexpected)
13. Otherwise -> Unknown

Output: docs/governance/rr2a5_executable_inventory.csv
"""

import os
import re
import csv

WORKSPACE_ROOT = "/Users/nikhil/ChronoSentiment_MEGA_FINAL"

# (member_path, package_name, workspace_status)
CRATES = [
    ("infrastructure/core",             "chronosentiment_core",          "Member"),
    ("infrastructure/optimization",     "chronosentiment_optimization",  "Member"),
    ("infrastructure/observatory/api",  "api",                           "Member"),
    ("financial/ese",                   "cs-ingest",                     "Member"),
    ("financial/strategies",            "chronosentiment_strategies",    "Member"),
    ("financial/core",                  "chronosentiment_financial_core","Member"),
    ("coralys-moga",                    "coralys-moga",                  "Member"),
    ("coralys-simulation",              "coralys-simulation",            "Member"),
    ("coralys-ecology",                 "coralys-ecology",               "Member"),
    ("coralys-decision",                "coralys-decision",              "Member"),
    ("coralys-recommendation",          "coralys-recommendation",        "Member"),
    ("coralys-infrastructure",          "coralys-infrastructure",        "Member"),
    ("adapters/ultracrew",              "ultracrew",                     "Member"),
    ("adapters/chronosentiment",        "chronosentiment_adapter",       "Member"),
    ("adapters/cvrp",                   "cvrp",                          "Member"),
    ("adapters/cvd001",                 "cvd001",                        "Member"),
    ("coralys-v2",                      "coralys-v2",                    "Member"),
    ("coralys-core",                    "coralys-core",                  "Member"),
    ("coralys-eval",                    "coralys-eval",                  "Member"),
    ("coralys-matching",                "coralys-matching",              "Member"),
    ("adapters/roadef",                 "roadef",                        "Member"),
    ("adapters/airline",                "coralys-airline",               "Member"),
    ("adapters/gerad",                  "coralys-gerad",                 "Member"),
    ("coralys-planning",                "coralys-planning",              "Member"),
    ("services/ultracrew_server",       "ultracrew_server",              "Member"),
    ("services/cvrp_server",            "cvrp_server",                   "Member"),
    ("coralys-policy",                  "coralys-policy",                "Orphan"),
]

# Service crates where main.rs is Production
SERVICE_CRATES = {
    "infrastructure/observatory/api",
    "financial/ese",
    "services/ultracrew_server",
    "services/cvrp_server",
}

def count_loc(path):
    try:
        with open(path, "r", encoding="utf-8", errors="replace") as f:
            return sum(1 for line in f if line.strip())
    except Exception:
        return 0

def classify_binary(name, path, member, is_deprecated=False):
    """Return (purpose, rationale)."""
    if is_deprecated:
        return "Deprecated", "in src/bin/deprecated/"

    n = name.lower()

    # Milestone experiment series: m8, m9, m11, m22, m25, m26, m27, m30, m31 etc.
    if re.match(r'^m\d+', n):
        return "Experiment", "m-series milestone prefix"

    # BDD tests
    if n.startswith("bdd_"):
        return "Benchmark", "bdd_ prefix"

    # Benchmark
    if any(x in n for x in ["benchmark", "_bench", "bench_", "campaign_report"]):
        return "Benchmark", "benchmark keyword"

    # Validation / audit
    if any(x in n for x in ["validation", "validate", "acceptance", "governance",
                              "sanity", "check_", "verify", "audit", "forensics",
                              "compliance", "certif"]):
        return "Validation", "validation/audit keyword"

    # Campaign runners
    if "campaign" in n:
        return "Tool", "campaign runner"

    # Pilot / telemetry / observatory
    if any(x in n for x in ["pilot", "telemetry", "observatory", "passive_", "active_"]):
        return "Experiment", "pilot/telemetry keyword"

    # Production CLI / server / ingest
    if any(x in n for x in ["cli", "-cli", "_cli", "server", "ingest", "daemon"]):
        return "Production", "cli/server/ingest keyword"

    # Migration
    if any(x in n for x in ["migration", "migrate", "import"]):
        return "Migration", "migration keyword"

    # main.rs
    if name == "main":
        if member in SERVICE_CRATES:
            return "Production", "main.rs in service crate"
        return "Unknown", "main.rs in non-service crate"

    # Ecology / ablation / response / cost / memory / history / horizon
    if any(x in n for x in ["ecology", "ablation", "response_curve", "cost_curve",
                              "memory_depth", "horizon", "history", "natural_history",
                              "ancestry", "synthetic", "seed_", "basin", "manifold",
                              "partition", "elite", "epistasis", "invariant",
                              "backbone", "decoder", "recogniz", "reconstruct",
                              "reachability", "repair_atlas", "atlas", "story"]):
        return "Experiment", "experiment keyword"

    # Comparison / search / config
    if any(x in n for x in ["compare", "search_config", "config_sweep", "eval_profiler",
                              "tiny_solver", "find_", "seed_ecology", "shadow",
                              "discriminative", "failure_density", "survival",
                              "learner", "comparator", "instrumentation",
                              "engagement", "policy_seed", "archive"]):
        return "Experiment", "experiment/analysis keyword"

    # Registry / profiler / dual_path
    if any(x in n for x in ["registry", "profiler", "dual_path", "bridge"]):
        return "Tool", "utility/tool keyword"

    return "Unknown", "no pattern matched"

def read_toml(path):
    try:
        with open(path, "r") as f:
            return f.read()
    except Exception:
        return ""

def get_explicit_bins(toml_content, member_abs):
    """Extract explicit [[bin]] entries with name and path."""
    results = []
    # Match [[bin]] blocks
    blocks = re.split(r'\[\[bin\]\]', toml_content)[1:]
    for block in blocks:
        # Stop at next section
        block = re.split(r'\[', block)[0]
        name_m = re.search(r'name\s*=\s*"([^"]+)"', block)
        path_m = re.search(r'path\s*=\s*"([^"]+)"', block)
        if name_m:
            bin_name = name_m.group(1)
            if path_m:
                bin_path = os.path.join(member_abs, path_m.group(1))
            else:
                bin_path = os.path.join(member_abs, "src", "bin", bin_name + ".rs")
            results.append((bin_name, bin_path))
    return results

rows = []

for member, package, ws_status in CRATES:
    member_abs = os.path.join(WORKSPACE_ROOT, member)
    toml_path = os.path.join(member_abs, "Cargo.toml")
    toml_content = read_toml(toml_path)

    seen_paths = set()

    # 1. main.rs
    main_rs = os.path.join(member_abs, "src", "main.rs")
    if os.path.exists(main_rs):
        purpose, rationale = classify_binary("main", main_rs, member)
        loc = count_loc(main_rs)
        rel = os.path.relpath(main_rs, WORKSPACE_ROOT)
        rows.append({
            "binary_name": "main",
            "member": member,
            "package": package,
            "workspace_status": ws_status,
            "entry_point": rel,
            "loc": loc,
            "purpose": purpose,
            "rationale": rationale,
            "deprecated": False,
        })
        seen_paths.add(main_rs)

    # 2. Explicit [[bin]] entries
    explicit_bins = get_explicit_bins(toml_content, member_abs)
    for bin_name, bin_path in explicit_bins:
        if bin_path in seen_paths:
            continue
        seen_paths.add(bin_path)
        if not os.path.exists(bin_path):
            rows.append({
                "binary_name": bin_name,
                "member": member,
                "package": package,
                "workspace_status": ws_status,
                "entry_point": f"MISSING:{bin_path}",
                "loc": 0,
                "purpose": "Unknown",
                "rationale": "file not found",
                "deprecated": False,
            })
            continue
        purpose, rationale = classify_binary(bin_name, bin_path, member)
        loc = count_loc(bin_path)
        rel = os.path.relpath(bin_path, WORKSPACE_ROOT)
        rows.append({
            "binary_name": bin_name,
            "member": member,
            "package": package,
            "workspace_status": ws_status,
            "entry_point": rel,
            "loc": loc,
            "purpose": purpose,
            "rationale": rationale,
            "deprecated": False,
        })

    # 3. Implicit src/bin/*.rs
    bin_dir = os.path.join(member_abs, "src", "bin")
    if os.path.isdir(bin_dir):
        for fname in sorted(os.listdir(bin_dir)):
            if not fname.endswith(".rs"):
                continue
            fpath = os.path.join(bin_dir, fname)
            if fpath in seen_paths:
                continue
            seen_paths.add(fpath)
            bin_name = fname[:-3]
            purpose, rationale = classify_binary(bin_name, fpath, member)
            loc = count_loc(fpath)
            rel = os.path.relpath(fpath, WORKSPACE_ROOT)
            rows.append({
                "binary_name": bin_name,
                "member": member,
                "package": package,
                "workspace_status": ws_status,
                "entry_point": rel,
                "loc": loc,
                "purpose": purpose,
                "rationale": rationale,
                "deprecated": False,
            })

    # 4. Deprecated bins
    dep_dir = os.path.join(member_abs, "src", "bin", "deprecated")
    if os.path.isdir(dep_dir):
        for fname in sorted(os.listdir(dep_dir)):
            if not fname.endswith(".rs"):
                continue
            fpath = os.path.join(dep_dir, fname)
            bin_name = fname[:-3]
            loc = count_loc(fpath)
            rel = os.path.relpath(fpath, WORKSPACE_ROOT)
            rows.append({
                "binary_name": bin_name,
                "member": member,
                "package": package,
                "workspace_status": ws_status,
                "entry_point": rel,
                "loc": loc,
                "purpose": "Deprecated",
                "rationale": "in src/bin/deprecated/",
                "deprecated": True,
            })

out_path = os.path.join(WORKSPACE_ROOT, "docs", "governance", "rr2a5_executable_inventory.csv")
fieldnames = [
    "binary_name", "member", "package", "workspace_status",
    "entry_point", "loc", "purpose", "rationale", "deprecated",
]

with open(out_path, "w", newline="") as f:
    writer = csv.DictWriter(f, fieldnames=fieldnames)
    writer.writeheader()
    writer.writerows(rows)

print(f"Written {len(rows)} rows to {out_path}")

# Summary by purpose
from collections import Counter
purpose_counts = Counter(r["purpose"] for r in rows)
print("\nPurpose Summary:")
for k, v in sorted(purpose_counts.items(), key=lambda x: -x[1]):
    print(f"  {k}: {v}")

# Summary by member
print("\nExecutables per member:")
member_counts = Counter(r["member"] for r in rows)
for member, count in sorted(member_counts.items(), key=lambda x: -x[1]):
    purposes = Counter(r["purpose"] for r in rows if r["member"] == member)
    purpose_str = ", ".join(f"{k}={v}" for k, v in sorted(purposes.items()))
    print(f"  {member}: {count} ({purpose_str})")

# Total LOC in executables
total_loc = sum(r["loc"] for r in rows)
print(f"\nTotal LOC across all executables: {total_loc}")
prod_loc = sum(r["loc"] for r in rows if r["purpose"] == "Production")
exp_loc = sum(r["loc"] for r in rows if r["purpose"] == "Experiment")
dep_loc = sum(r["loc"] for r in rows if r["purpose"] == "Deprecated")
print(f"  Production LOC: {prod_loc}")
print(f"  Experiment LOC: {exp_loc}")
print(f"  Deprecated LOC: {dep_loc}")
