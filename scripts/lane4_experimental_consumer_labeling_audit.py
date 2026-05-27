#!/usr/bin/env python3
"""
Lane 4 — experimental consumer labeling audit.

Read-only classification of research surfaces vs CRITICAL authority domains.
Labels exposure only — no convergence, remediation, or canonical replacement.

See: docs/governance/LANE4_EXPERIMENTAL_CONSUMER_LABELING_SCOPE.md
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[1]
AUDIT_PATH = REPO_ROOT / "fixtures" / "lane4" / "experimental_consumer_labeling_audit.json"
AUDIT_VERSION = "1.0.0"

SCAN_ROOTS = (
    REPO_ROOT / "research_experiments",
    REPO_ROOT / "scripts" / "research",
)

# Map-declared surfaces outside scripts/research (Lane 4 high-signal)
ADDITIONAL_SURFACES = (
    "scripts/signal_physics_harness.py",
    "scripts/adversarial_physics_test.py",
    "scripts/synthetic_fragmentation_injector.py",
    "scripts/survivability_surface_builder.py",
)

CLASS_PRIORITY = (
    "semantic_duplicate",
    "downstream_consumer",
    "observational_consumer",
    "orphaned_lineage",
    "detached_archive",
)

PATTERN_RULES: tuple[tuple[str, str, str], ...] = (
    (
        "semantic_duplicate",
        r"0\.5\s*\*\s*h_accel|h_accel.*0\.3|0\.3\s*\*\s*e_decay|0\.2\s*\*\s*c_vel",
        "risk scoring formula drift (map-declared)",
    ),
    (
        "semantic_duplicate",
        r"baseline_exposure|survival_gain|1\.0\s*-\s*\(\s*exposure",
        "survival gain formula drift",
    ),
    (
        "semantic_duplicate",
        r"compute_admissibility|is_synchronized\s*=|strict_ratio\s*>=\s*0\.9|classify\(0\.95",
        "admissibility threshold drift",
    ),
    (
        "semantic_duplicate",
        r"generate_intent\s*\(",
        "signal logic drift (generate_intent)",
    ),
    (
        "downstream_consumer",
        r"evaluate_strategy|fitness|genome|STRAT_|parse_strategy|strategy_id",
        "strategy / GA consumer reference",
    ),
    (
        "downstream_consumer",
        r"trace_replay|replay_hash|reduce_replay|certify_equivalence",
        "replay / certification consumer reference",
    ),
    (
        "observational_consumer",
        r"export_observatory|plot_observatory|read_only|verify_|observatory",
        "observational export/plot/verify pattern",
    ),
    (
        "orphaned_lineage",
        r"infrastructure/core/src/live_source|infrastructure/core/src/data_source/python|infrastructure/core/src/data_source/yahoo|mod kernel|kernel::run_ga",
        "former core path or deprecated stub reference",
    ),
)

MAP_BASELINE: dict[str, str] = {
    "scripts/research/policy_competition_engine.py": "semantic_duplicate",
    "scripts/research/counterfactual_replay.py": "semantic_duplicate",
    "scripts/research/controlled_ablation_harness.py": "semantic_duplicate",
    "scripts/research/adaptive_participation_layer.py": "downstream_consumer",
    "scripts/research/train_ranking_model.py": "downstream_consumer",
    "scripts/research/robustness_experiments.py": "semantic_duplicate",
    "scripts/signal_physics_harness.py": "semantic_duplicate",
    "scripts/adversarial_physics_test.py": "semantic_duplicate",
    "scripts/synthetic_fragmentation_injector.py": "semantic_duplicate",
    "scripts/survivability_surface_builder.py": "downstream_consumer",
    "research_experiments/kernel_stub_v0/kernel.rs": "detached_archive",
    "research_experiments/live_source_v0/live_source.rs": "detached_archive",
    "research_experiments/python_bridge/python.rs": "detached_archive",
    "research_experiments/yahoo_adapter_v0/yahoo.rs": "detached_archive",
}


@dataclass
class SurfaceRecord:
    path: str
    primary_label: str
    labels: list[str]
    signals: list[str]
    operational_authority: bool
    map_baseline_label: str | None
    notes: list[str]


def get_git_commit() -> str | None:
    try:
        proc = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=REPO_ROOT,
            capture_output=True,
            text=True,
            check=True,
        )
        return proc.stdout.strip()
    except (subprocess.CalledProcessError, FileNotFoundError):
        return None


def rel(path: Path) -> str:
    return path.relative_to(REPO_ROOT).as_posix()


def collect_surfaces() -> list[Path]:
    paths: set[Path] = set()
    for root in SCAN_ROOTS:
        if not root.exists():
            continue
        for path in root.rglob("*"):
            if path.is_file() and not path.name.startswith("."):
                if "__pycache__" in path.parts or path.suffix in {".pyc", ".pyo"}:
                    continue
                paths.add(path.resolve())
    for item in ADDITIONAL_SURFACES:
        path = REPO_ROOT / item
        if path.exists():
            paths.add(path.resolve())
    return sorted(paths)


def classify_file(path: Path) -> SurfaceRecord:
    relpath = rel(path)
    notes: list[str] = []
    labels: set[str] = set()
    signals: list[str] = []

    if relpath.startswith("research_experiments/"):
        labels.add("detached_archive")
        notes.append("not in Rust module tree — archived lineage only")

    try:
        text = path.read_text(errors="replace")
    except OSError as exc:
        return SurfaceRecord(
            path=relpath,
            primary_label="orphaned_lineage",
            labels=["orphaned_lineage"],
            signals=[f"read_error: {exc}"],
            operational_authority=False,
            map_baseline_label=MAP_BASELINE.get(relpath),
            notes=["unreadable surface"],
        )

    for label, regex, description in PATTERN_RULES:
        if re.search(regex, text, flags=re.IGNORECASE):
            labels.add(label)
            signals.append(description)

    if relpath.startswith("scripts/research/") and not labels:
        labels.add("downstream_consumer")
        notes.append("default research corpus classification")

    if not labels:
        labels.add("observational_consumer")
        notes.append("no CRITICAL-duplicate signals detected")

    primary = sorted(labels, key=lambda x: CLASS_PRIORITY.index(x) if x in CLASS_PRIORITY else 99)[0]
    baseline = MAP_BASELINE.get(relpath)

    if baseline and baseline not in labels and primary != baseline:
        notes.append(f"map baseline expects {baseline}; detected primary={primary}")

    return SurfaceRecord(
        path=relpath,
        primary_label=primary,
        labels=sorted(labels),
        signals=sorted(set(signals)),
        operational_authority=False,
        map_baseline_label=baseline,
        notes=notes,
    )


def summarize(records: list[SurfaceRecord]) -> dict[str, Any]:
    by_label: dict[str, int] = {}
    for record in records:
        by_label[record.primary_label] = by_label.get(record.primary_label, 0) + 1

    map_surfaces = [r for r in records if r.map_baseline_label]
    map_mismatches = [
        r.path
        for r in map_surfaces
        if r.map_baseline_label and r.map_baseline_label not in r.labels
    ]

    return {
        "surfaces_scanned": len(records),
        "by_primary_label": by_label,
        "semantic_duplicate_count": by_label.get("semantic_duplicate", 0),
        "detached_archive_count": by_label.get("detached_archive", 0),
        "map_baseline_surfaces": len(map_surfaces),
        "map_label_mismatches": map_mismatches,
        "operational_authority_surfaces": 0,
    }


def build_audit() -> dict[str, Any]:
    records = [classify_file(path) for path in collect_surfaces()]
    summary = summarize(records)

    attestations = ["BOUNDED_SURFACE_SCAN_COMPLETE", "NO_OPERATIONAL_AUTHORITY_GRANTED"]
    if summary["semantic_duplicate_count"]:
        attestations.append("SEMANTIC_DUPLICATES_LABELED")
    if summary["detached_archive_count"]:
        attestations.append("DETACHED_ARCHIVES_LABELED")
    if not summary["map_label_mismatches"]:
        attestations.append("MAP_BASELINE_ALIGNED")

    return {
        "audit_version": AUDIT_VERSION,
        "lane": "Lane 4 — experimental consumer labeling audit",
        "layer": "observational",
        "scope_doc": "docs/governance/LANE4_EXPERIMENTAL_CONSUMER_LABELING_SCOPE.md",
        "lineage_doc": "docs/RESEARCH_LINEAGE.md",
        "core_rule": "documented ≠ operational authority",
        "classification_taxonomy": list(CLASS_PRIORITY),
        "summary": summary,
        "surfaces": [asdict(r) for r in records],
        "attestations": attestations,
        "convergence_authorized": False,
        "git_commit": get_git_commit(),
        "status": "AUDIT_COMPLETE",
    }


def check_reference(current: dict[str, Any], reference_path: Path) -> list[str]:
    if not reference_path.exists():
        return ["REFERENCE_MISSING"]
    reference = json.loads(reference_path.read_text())
    errors: list[str] = []
    if reference.get("summary") != current.get("summary"):
        errors.append("SUMMARY_DRIFT")
    ref_commit = reference.get("git_commit")
    cur_commit = current.get("git_commit")
    if ref_commit == cur_commit and reference.get("surfaces") != current.get("surfaces"):
        errors.append("SURFACES_DRIFT")
    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description="Lane 4 experimental consumer labeling audit")
    parser.add_argument(
        "--check-reference",
        action="store_true",
        help="Fail if summary/surfaces drift at same git commit vs committed audit",
    )
    parser.add_argument(
        "--output",
        default=str(AUDIT_PATH),
        help="Audit JSON output path",
    )
    args = parser.parse_args()

    print("Lane 4 — experimental consumer labeling audit", flush=True)
    print("Layer: observational — label only, no convergence\n", flush=True)

    audit = build_audit()
    output_path = Path(args.output)

    failures: list[str] = []
    if args.check_reference:
        failures = check_reference(audit, output_path)

    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(audit, indent=2, sort_keys=True) + "\n")

    print(json.dumps({"status": audit["status"], "summary": audit["summary"]}, indent=2), flush=True)
    print(f"Audit: {output_path.relative_to(REPO_ROOT)}", flush=True)

    if failures:
        print(f"[FAIL] Reference check: {failures}", file=sys.stderr)
        return 1

    print("[OK] Experimental consumer labeling audit complete.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
