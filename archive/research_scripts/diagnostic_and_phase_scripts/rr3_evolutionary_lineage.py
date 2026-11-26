#!/usr/bin/env python3
"""
RR3 — Evolutionary Lineage Analysis
====================================
Reconstructs the historical evolution of experiment binaries across the three
research clusters: cvrp_server, ultracrew/ultracrew_server, roadef/cvrp.

For each experiment binary this script collects:
  - First commit date and hash (when the file was introduced)
  - Last active commit date and hash (most recent modification)
  - LOC (from rr2a5_executable_inventory.csv)
  - Generation number (inferred from numeric prefix in binary name)
  - Parent binary (previous generation in the same lineage)
  - Successors (next generation binaries)
  - Experiment goal (inferred from name + commit message)
  - Outcome (inferred from whether a successor exists)
  - Canonical replacement (if any)
  - Archive status

Outputs:
  docs/governance/rr3_lineage.csv          — tabular record per binary
  docs/governance/rr3_lineage_graph.md     — Experiment Lineage Graph (DAG, text)
  docs/governance/rr3_evolutionary_lineage.md — narrative analysis document

Usage:
  python3 scripts/rr3_evolutionary_lineage.py
"""

import csv
import re
import subprocess
import sys
from collections import defaultdict
from pathlib import Path

WORKSPACE = Path(__file__).parent.parent
INVENTORY_CSV = WORKSPACE / "docs/governance/rr2a5_executable_inventory.csv"
OUT_CSV = WORKSPACE / "docs/governance/rr3_lineage.csv"
OUT_GRAPH = WORKSPACE / "docs/governance/rr3_lineage_graph.md"
OUT_DOC = WORKSPACE / "docs/governance/rr3_evolutionary_lineage.md"

# ---------------------------------------------------------------------------
# Cluster membership: which packages belong to which research stream
# ---------------------------------------------------------------------------
CLUSTER_MAP = {
    "cvrp_server": "CVRP",
    "cvrp": "CVRP",
    "ultracrew": "ULTRACREW",
    "ultracrew_server": "ULTRACREW",
    "roadef": "ROADEF",
}

# ---------------------------------------------------------------------------
# Generation extraction
# Patterns:
#   m8g, m9a, m11, m12, m14, m15, m16, m17, m18, m20, m21, m21c, m21d,
#   m22, m22_0, m22a, m22b, m22c, m22c_0, m22c_1, m22c_2, m22f,
#   m23a, m25, m25_8, m25_8b, m26_1, m26_1c, m26_1d, m26_1e, m26_3,
#   m26_4a, m26_4b, m27_1, m30, m30_0b, m30_0d, m30_2, m30_2a_1,
#   m30_2a_2, m31, m31_2a
# ---------------------------------------------------------------------------
GEN_RE = re.compile(r'^m(\d+)', re.IGNORECASE)
SUBGEN_RE = re.compile(r'^m(\d+)([a-z]?)(?:_(\d+))?(?:_(\d+))?', re.IGNORECASE)


def extract_generation(name: str) -> tuple[int, str]:
    """Return (major_gen, full_gen_label) or (9999, '') if not a generation binary."""
    m = GEN_RE.match(name)
    if not m:
        return (9999, "")
    major = int(m.group(1))
    # Build a sortable label
    sm = SUBGEN_RE.match(name)
    if sm:
        label = f"m{sm.group(1)}"
        if sm.group(2):
            label += sm.group(2)
        if sm.group(3):
            label += f"_{sm.group(3)}"
        if sm.group(4):
            label += f"_{sm.group(4)}"
    else:
        label = f"m{major}"
    return (major, label)


def git_first_commit(path: str) -> tuple[str, str]:
    """Return (date, hash) of the commit that introduced this file."""
    try:
        out = subprocess.check_output(
            ["git", "log", "--follow", "--diff-filter=A", "--format=%ai %H", "--", path],
            text=True, stderr=subprocess.DEVNULL, cwd=str(WORKSPACE)
        ).strip()
        if out:
            parts = out.split()
            date = parts[0]
            h = parts[-1]
            return date, h
    except Exception:
        pass
    return "unknown", "unknown"


def git_last_commit(path: str) -> tuple[str, str]:
    """Return (date, hash) of the most recent commit touching this file."""
    try:
        out = subprocess.check_output(
            ["git", "log", "--follow", "--format=%ai %H", "--", path],
            text=True, stderr=subprocess.DEVNULL, cwd=str(WORKSPACE)
        ).strip().split("\n")[0]
        if out:
            parts = out.split()
            date = parts[0]
            h = parts[-1]
            return date, h
    except Exception:
        pass
    return "unknown", "unknown"


def git_commit_subject(hash_: str) -> str:
    """Return the one-line subject of a commit."""
    if hash_ in ("unknown", "error"):
        return ""
    try:
        return subprocess.check_output(
            ["git", "show", "-s", "--format=%s", hash_],
            text=True, stderr=subprocess.DEVNULL, cwd=str(WORKSPACE)
        ).strip()
    except Exception:
        return ""


# ---------------------------------------------------------------------------
# Goal inference from binary name
# ---------------------------------------------------------------------------
GOAL_HINTS = {
    "atlas": "Reachability / solution space mapping",
    "audit": "Structural or behavioural audit",
    "ablation": "Ablation study — component isolation",
    "ecology": "Ecology / fitness landscape characterisation",
    "recognizability": "Recognizability / decoder fidelity",
    "independence": "Decoder independence verification",
    "reconstruction": "Reconstruction guidance",
    "backbone": "Backbone causality analysis",
    "invariant": "Structural invariant verification",
    "epistasis": "Epistasis / gene interaction study",
    "feasible": "Feasibility path analysis",
    "annealed": "Simulated annealing variant",
    "novelty": "Novelty-elitism hybrid",
    "memetic": "Memetic algorithm variant",
    "fidelity": "Fidelity audit",
    "control": "Control / baseline experiment",
    "validation": "Validation against reference",
    "benchmark": "Benchmark / performance measurement",
    "basin": "Basin of attraction characterisation",
    "manifold": "Elite manifold probing",
    "partition": "Partition probing",
    "seed": "Seed / initialisation study",
    "scale": "Scale-up experiment",
    "repair": "Repair operator study",
    "bridge": "Bridge / transition experiment",
    "passive": "Passive observation / telemetry",
    "active": "Active pilot / live deployment",
    "shadow": "Shadow advisory mode",
    "discriminative": "Discriminative audit",
    "failure": "Failure density analysis",
    "survival": "Survival curve analysis",
    "learner": "Passive learner",
    "operator": "Operator telemetry",
    "observatory": "Search observatory",
    "ancestry": "Ancestry / lineage tracing",
    "history": "Natural history pilot",
    "synthetic": "Synthetic data experiment",
    "engagement": "Engagement audit",
    "sweep": "Configuration sweep",
    "curve": "Response / cost curve",
    "memory": "Memory depth study",
    "multi_week": "Multi-week ablation",
    "pilot": "Pilot deployment",
    "telemetry": "Telemetry collection",
    "profiler": "Evaluation profiler",
    "tiny": "Minimal / smoke-test solver",
    "compare": "Comparative analysis",
    "search_config": "Search configuration study",
    "find": "Search / discovery experiment",
    "story": "Narrative / demonstration run",
    "policy": "Policy seeding",
    "sa": "Simulated annealing",
}


def infer_goal(name: str) -> str:
    name_lower = name.lower()
    for kw, desc in GOAL_HINTS.items():
        if kw in name_lower:
            return desc
    return "General experiment"


# ---------------------------------------------------------------------------
# Outcome inference: if a successor exists → superseded; else → terminal
# ---------------------------------------------------------------------------
def infer_outcome(name: str, successors: list[str]) -> str:
    if successors:
        return "Superseded"
    return "Terminal (no known successor)"


# ---------------------------------------------------------------------------
# Parent inference: within a cluster, the binary with the next-lower
# generation number and the same lineage prefix is the parent.
# For sub-generations (m22a, m22b) the parent is the base (m22).
# ---------------------------------------------------------------------------
def infer_parent(name: str, gen_major: int, gen_label: str,
                 cluster_bins: list[dict]) -> str:
    """Return the name of the most likely parent binary, or '' if root."""
    if gen_major == 9999:
        return ""

    # Sub-generation: m22a → parent is m22 (if it exists)
    sm = SUBGEN_RE.match(name)
    if sm and sm.group(2):  # has a letter suffix
        base = f"m{sm.group(1)}"
        for b in cluster_bins:
            if b["binary_name"] == base or b["binary_name"].startswith(base + "_"):
                return b["binary_name"]

    # Variant: m22c_1 → parent is m22c_0 or m22c
    if sm and sm.group(3):
        sub_idx = int(sm.group(3))
        if sub_idx > 0:
            prev = f"m{sm.group(1)}{sm.group(2) or ''}_{sub_idx - 1}"
            for b in cluster_bins:
                if b["binary_name"] == prev:
                    return prev
        # Fall back to base letter
        base_letter = f"m{sm.group(1)}{sm.group(2) or ''}"
        for b in cluster_bins:
            if b["binary_name"] == base_letter:
                return base_letter

    # General: find the binary with the largest gen_major < current
    candidates = [
        b for b in cluster_bins
        if b["gen_major"] < gen_major and b["gen_major"] != 9999
    ]
    if not candidates:
        return ""
    # Pick the one with the highest gen_major
    candidates.sort(key=lambda b: b["gen_major"], reverse=True)
    return candidates[0]["binary_name"]


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
def main():
    print("RR3: Loading experiment inventory...", file=sys.stderr)

    # Load inventory
    experiments = []
    with open(INVENTORY_CSV) as f:
        for r in csv.DictReader(f):
            if r["purpose"] == "Experiment":
                experiments.append(r)

    print(f"  {len(experiments)} experiment binaries found", file=sys.stderr)

    # Collect git history for each binary
    print("RR3: Collecting git history (this may take ~30s)...", file=sys.stderr)
    records = []
    for r in experiments:
        ep = r["entry_point"]
        pkg = r["package"]
        cluster = CLUSTER_MAP.get(pkg, "OTHER")
        gen_major, gen_label = extract_generation(r["binary_name"])

        first_date, first_hash = git_first_commit(ep)
        last_date, last_hash = git_last_commit(ep)
        first_subject = git_commit_subject(first_hash)
        last_subject = git_commit_subject(last_hash)

        records.append({
            "binary_name": r["binary_name"],
            "package": pkg,
            "cluster": cluster,
            "entry_point": ep,
            "loc": int(r["loc"]),
            "gen_major": gen_major,
            "gen_label": gen_label,
            "first_commit_date": first_date,
            "first_commit_hash": first_hash,
            "first_commit_subject": first_subject,
            "last_commit_date": last_date,
            "last_commit_hash": last_hash,
            "last_commit_subject": last_subject,
            "goal": infer_goal(r["binary_name"]),
            "parent": "",       # filled in second pass
            "successors": "",   # filled in second pass
            "outcome": "",      # filled in second pass
            "canonical_replacement": "",
            "archive_status": "Pending",
            "evidence": "E3",   # all experiment bins are reachable (E3 = no external caller)
            "notes": "",
        })

    # Sort within each cluster by gen_major then name
    records.sort(key=lambda r: (r["cluster"], r["gen_major"], r["binary_name"]))

    # Second pass: infer parent and successors within each cluster
    by_cluster = defaultdict(list)
    for r in records:
        by_cluster[r["cluster"]].append(r)

    for cluster, bins in by_cluster.items():
        for r in bins:
            r["parent"] = infer_parent(
                r["binary_name"], r["gen_major"], r["gen_label"], bins
            )

        # Build successors: for each binary, find all bins whose parent is this one
        name_to_rec = {b["binary_name"]: b for b in bins}
        for r in bins:
            succs = [b["binary_name"] for b in bins if b["parent"] == r["binary_name"]]
            r["successors"] = "|".join(succs)
            r["outcome"] = infer_outcome(r["binary_name"], succs)

    # Write CSV
    fieldnames = [
        "binary_name", "package", "cluster", "entry_point", "loc",
        "gen_label", "gen_major",
        "first_commit_date", "first_commit_hash", "first_commit_subject",
        "last_commit_date", "last_commit_hash", "last_commit_subject",
        "goal", "parent", "successors", "outcome",
        "canonical_replacement", "archive_status", "evidence", "notes",
    ]
    with open(OUT_CSV, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=fieldnames, extrasaction="ignore")
        w.writeheader()
        w.writerows(records)
    print(f"RR3: Written {OUT_CSV}", file=sys.stderr)

    # Write DAG graph document
    write_dag(records, by_cluster)
    print(f"RR3: Written {OUT_GRAPH}", file=sys.stderr)

    # Write narrative document
    write_narrative(records, by_cluster)
    print(f"RR3: Written {OUT_DOC}", file=sys.stderr)

    print("RR3: Complete.", file=sys.stderr)


# ---------------------------------------------------------------------------
# DAG graph renderer
# ---------------------------------------------------------------------------
def render_dag_cluster(bins: list[dict]) -> str:
    """Render a text DAG for one cluster."""
    name_to_rec = {b["binary_name"]: b for b in bins}

    # Find roots (no parent or parent not in this cluster)
    roots = [b for b in bins if not b["parent"] or b["parent"] not in name_to_rec]

    lines = []

    def render_node(name: str, depth: int, visited: set):
        if name in visited:
            return
        visited.add(name)
        r = name_to_rec.get(name)
        if not r:
            return
        prefix = "  " * depth
        gen = r["gen_label"] if r["gen_label"] else "—"
        goal_short = r["goal"][:50]
        lines.append(f"{prefix}{name}  [{gen}]  {r['first_commit_date']}  {r['loc']} LOC")
        lines.append(f"{prefix}  goal: {goal_short}")
        succs = [s for s in r["successors"].split("|") if s]
        for i, s in enumerate(succs):
            connector = "├──" if i < len(succs) - 1 else "└──"
            lines.append(f"{prefix}  {connector} {s}")
            render_node(s, depth + 2, visited)

    visited = set()
    for root in sorted(roots, key=lambda b: (b["gen_major"], b["binary_name"])):
        render_node(root["binary_name"], 0, visited)
        lines.append("")

    # Any remaining (cycles or disconnected)
    for b in bins:
        if b["binary_name"] not in visited:
            render_node(b["binary_name"], 0, visited)
            lines.append("")

    return "\n".join(lines)


def write_dag(records: list[dict], by_cluster: dict):
    lines = [
        "# RR3 — Experiment Lineage Graph",
        "",
        "**Programme:** Repository Rationalization  ",
        "**Phase:** RR3 — Historical Evidence / Evolutionary Lineage Analysis  ",
        "**Produced:** 2026-08-02  ",
        "",
        "Each node shows: `binary_name  [generation]  first_commit_date  LOC`  ",
        "Edges represent: derived-from / superseded-by relationships inferred from",
        "generation numbering and commit chronology.",
        "",
        "> Note: Parent–successor edges are inferred from naming conventions.",
        "> Where the naming convention is ambiguous, the edge represents the most",
        "> likely evolutionary relationship. RR4 may revise individual edges.",
        "",
    ]

    cluster_order = ["CVRP", "ULTRACREW", "ROADEF"]
    cluster_titles = {
        "CVRP": "CVRP Research Stream",
        "ULTRACREW": "ULTRACREW / INRC Research Stream",
        "ROADEF": "ROADEF Research Stream",
    }

    for cluster in cluster_order:
        bins = by_cluster.get(cluster, [])
        if not bins:
            continue
        title = cluster_titles[cluster]
        lines.append(f"## {title}  ({len(bins)} experiments)")
        lines.append("")
        lines.append("```")
        lines.append(render_dag_cluster(bins))
        lines.append("```")
        lines.append("")

    with open(OUT_GRAPH, "w") as f:
        f.write("\n".join(lines))


# ---------------------------------------------------------------------------
# Narrative document
# ---------------------------------------------------------------------------
def write_narrative(records: list[dict], by_cluster: dict):
    cluster_order = ["CVRP", "ULTRACREW", "ROADEF"]
    cluster_titles = {
        "CVRP": "CVRP Research Stream",
        "ULTRACREW": "ULTRACREW / INRC Research Stream",
        "ROADEF": "ROADEF Research Stream",
    }

    total = len(records)
    total_loc = sum(r["loc"] for r in records)

    lines = [
        "# RR3 — Evolutionary Lineage Analysis",
        "",
        "**Programme:** Repository Rationalization  ",
        "**Phase:** RR3 — Historical Evidence / Evolutionary Lineage Analysis  ",
        "**Status:** Complete  ",
        "**Produced:** 2026-08-02  ",
        "**Input:** `docs/governance/rr2a5_executable_inventory.csv` (Experiment rows)  ",
        "**Outputs:**",
        "  - `docs/governance/rr3_lineage.csv` — tabular lineage record (one row per binary)",
        "  - `docs/governance/rr3_lineage_graph.md` — Experiment Lineage Graph (DAG)",
        "  - This document — narrative analysis",
        "",
        "---",
        "",
        "## 1. Scope",
        "",
        "RR3 reconstructs the evolutionary history of the 64 experiment binaries",
        "identified by RR2-A.5. These binaries represent the primary research footprint",
        "of the repository: 16,769 LOC of experimental code across five packages.",
        "",
        "The analysis answers four questions for each binary:",
        "1. When was it introduced and when was it last modified?",
        "2. What hypothesis or goal did it evaluate?",
        "3. Which binary preceded it (parent) and which followed it (successors)?",
        "4. What was its outcome — did it produce a successor, or is it terminal?",
        "",
        "---",
        "",
        "## 2. Methodology",
        "",
        "**Generation numbering.** Experiment binaries follow a consistent naming",
        "convention: `m{N}[letter][_variant]`. The numeric prefix `N` encodes the",
        "generation. Sub-generations use letter suffixes (m22a, m22b, m22c) and",
        "variant indices (m22c_0, m22c_1, m22c_2). Non-generation binaries (those",
        "without an `m` prefix) are treated as infrastructure or utility experiments.",
        "",
        "**Parent inference.** Within each cluster, the parent of generation N is",
        "the binary with the largest generation number less than N. For sub-generations,",
        "the parent is the base generation (m22a → m22). For variants, the parent is",
        "the previous variant index (m22c_1 → m22c_0).",
        "",
        "**Outcome inference.** A binary is classified as Superseded if it has at",
        "least one successor in the same cluster. It is Terminal if no successor exists.",
        "Terminal status does not imply failure — the most recent generation in a",
        "research stream is always terminal by definition.",
        "",
        "**Evidence level.** All experiment binaries are classified E3 (no external",
        "caller imports their symbols). They are reachable from compilation roots",
        "(each has its own `main()`) but are not called by any library code.",
        "",
        "---",
        "",
        "## 3. Cluster Summaries",
        "",
    ]

    for cluster in cluster_order:
        bins = by_cluster.get(cluster, [])
        if not bins:
            continue
        title = cluster_titles[cluster]
        cluster_loc = sum(b["loc"] for b in bins)
        gen_bins = [b for b in bins if b["gen_major"] != 9999]
        non_gen_bins = [b for b in bins if b["gen_major"] == 9999]
        terminal = [b for b in bins if b["outcome"].startswith("Terminal")]
        superseded = [b for b in bins if b["outcome"] == "Superseded"]

        # Date range
        dates = [b["first_commit_date"] for b in bins if b["first_commit_date"] != "unknown"]
        last_dates = [b["last_commit_date"] for b in bins if b["last_commit_date"] != "unknown"]
        date_range = f"{min(dates)} → {max(last_dates)}" if dates and last_dates else "unknown"

        # Generation range
        gen_nums = sorted(set(b["gen_major"] for b in gen_bins))
        gen_range = f"m{min(gen_nums)}–m{max(gen_nums)}" if gen_nums else "—"

        lines += [
            f"### 3.{cluster_order.index(cluster)+1} {title}",
            "",
            f"| Field | Value |",
            f"|-------|-------|",
            f"| Packages | {', '.join(sorted(set(b['package'] for b in bins)))} |",
            f"| Experiment binaries | {len(bins)} |",
            f"| Total LOC | {cluster_loc:,} |",
            f"| Generation range | {gen_range} |",
            f"| Active period | {date_range} |",
            f"| Superseded | {len(superseded)} |",
            f"| Terminal | {len(terminal)} |",
            "",
        ]

        # List terminal binaries (most recent / active)
        if terminal:
            lines.append("**Terminal binaries (most recent in their lineage):**")
            lines.append("")
            for b in sorted(terminal, key=lambda x: x["binary_name"]):
                lines.append(f"- `{b['binary_name']}` — {b['goal']} ({b['loc']} LOC, last active {b['last_commit_date']})")
            lines.append("")

        # List non-generation binaries
        if non_gen_bins:
            lines.append("**Infrastructure / utility experiments (no generation prefix):**")
            lines.append("")
            for b in sorted(non_gen_bins, key=lambda x: x["binary_name"]):
                lines.append(f"- `{b['binary_name']}` — {b['goal']} ({b['loc']} LOC)")
            lines.append("")

        lines.append("---")
        lines.append("")

    # Summary table
    lines += [
        "## 4. Full Lineage Table",
        "",
        "See `docs/governance/rr3_lineage.csv` for the machine-readable record.",
        "The table below shows the key fields for each binary.",
        "",
        "| Binary | Cluster | Gen | LOC | First Commit | Last Active | Parent | Outcome |",
        "|--------|---------|-----|-----|-------------|-------------|--------|---------|",
    ]

    for r in records:
        gen = r["gen_label"] if r["gen_label"] else "—"
        parent = r["parent"] if r["parent"] else "—"
        lines.append(
            f"| `{r['binary_name']}` | {r['cluster']} | {gen} | {r['loc']} "
            f"| {r['first_commit_date']} | {r['last_commit_date']} "
            f"| `{parent}` | {r['outcome']} |"
        )

    lines += [
        "",
        "---",
        "",
        "## 5. Evidence Gaps",
        "",
        "The following questions cannot be answered from git history alone and",
        "require owner input or code inspection in RR4:",
        "",
        "1. **Experiment goals**: The goal field is inferred from binary names.",
        "   Names like `m22f_cvrp_control` or `story1` require human interpretation",
        "   to confirm the actual hypothesis being tested.",
        "",
        "2. **Outcome quality**: Terminal status means no successor was created,",
        "   not that the experiment succeeded or failed. The actual result (positive,",
        "   negative, inconclusive) is not recoverable from structural analysis.",
        "",
        "3. **Cross-cluster dependencies**: Some experiments in one cluster may",
        "   have informed experiments in another (e.g., CVRP findings feeding",
        "   ULTRACREW design). These cross-cluster relationships are not captured",
        "   by the intra-cluster parent/successor model.",
        "",
        "4. **Canonical replacements**: Where a terminal experiment's findings",
        "   were incorporated into a platform library (e.g., coralys-moga,",
        "   coralys-matching), the canonical replacement should be recorded.",
        "   This requires owner knowledge and is deferred to RR4.",
        "",
        "---",
        "",
        "## 6. RR4 Inputs",
        "",
        "RR3 provides the following inputs to RR4 (Governance Decisions):",
        "",
        "1. **Archive candidates**: All superseded binaries (those with successors)",
        "   are candidates for archival. They represent completed research steps",
        "   whose findings have been incorporated into successor experiments.",
        "",
        "2. **Preserve candidates**: Terminal binaries in active research streams",
        "   should be preserved until the stream is declared complete.",
        "",
        "3. **Delete candidates (E5 only)**: Only `adapters/ultracrew/src/inrc/bipartite_matching.rs`",
        "   (identified in RR2-C) has E5 evidence. No experiment binary reaches E5",
        "   on structural evidence alone.",
        "",
        "4. **Lineage graph**: The DAG in `rr3_lineage_graph.md` provides the",
        "   visual foundation for RR4 archival sequencing — superseded binaries",
        "   should be archived in reverse chronological order (oldest first).",
        "",
        "---",
        "",
        "## 7. Amendment Log",
        "",
        "| Version | Date | Author | Change |",
        "|---------|------|--------|--------|",
        "| 1.0 | 2026-08-02 | governance-hardening | Initial RR3 lineage analysis for all 64 experiment binaries |",
    ]

    with open(OUT_DOC, "w") as f:
        f.write("\n".join(lines))


if __name__ == "__main__":
    main()