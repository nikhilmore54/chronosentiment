#!/usr/bin/env python3
"""
Lane 2 prerequisite — manifest ms assumption probe.

Observational only: detect implicit millisecond semantics in bounded consumer
surfaces. No manifest rewrite, producer edit, or schema normalization.

See: docs/governance/V006_MANIFEST_MS_ASSUMPTION_PROBE_SCOPE.md
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
PROBE_PATH = REPO_ROOT / "fixtures" / "lane2" / "manifest_ms_assumption_probe.json"
INVENTORY_PATH = REPO_ROOT / "fixtures" / "lane2" / "manifest_ms_impact_inventory.json"
PROBE_VERSION = "1.0.0"
MS_THRESHOLD = 1_000_000_000_000

# Bounded consumer/producer surfaces — chronology manifest bounds only
PROBE_SURFACES: tuple[str, ...] = (
    "scripts/rebuild_catalog.py",
    "scripts/run_tier1_observability_demo.py",
    "scripts/verify_chronology_producer_ratification.py",
    "scripts/verify_chronology_byte_fixtures.py",
    "scripts/emit_manifest_v1.py",
    "core/src/bin/capture_daemon.rs",
    "core/src/bin/historical_importer.rs",
    "core/src/bin/yahoo_importer.rs",
)

PATTERN_RULES: tuple[dict[str, str], ...] = (
    {
        "pattern_id": "MS_FIELD_SUFFIX",
        "regex": r"capture_start_ms|capture_end_ms",
        "classification": "IMPLICIT_MS_SEMANTICS",
        "description": "Field name asserts millisecond semantics without dialect classification gate.",
        "dialect_b_risk": "medium",
    },
    {
        "pattern_id": "UNCONDITIONAL_MS_DIVIDE",
        "regex": r"capture_(?:start|end)[^\n]{0,80}/\s*1000|fromtimestamp\([^)]*/\s*1000\)",
        "classification": "IMPLICIT_MS_ARITHMETIC",
        "description": "Unconditional divide-by-1000 on manifest-bound timestamps.",
        "dialect_b_risk": "high",
    },
    {
        "pattern_id": "THRESHOLD_UNIT_HEURISTIC",
        "regex": r"1e11|1_000_000_000_000|MS_THRESHOLD",
        "classification": "HEURISTIC_UNIT_AWARE",
        "description": "Threshold heuristic distinguishes seconds vs milliseconds.",
        "dialect_b_risk": "low",
    },
    {
        "pattern_id": "EXPLICIT_MS_LAW_CHECK",
        "regex": r"<\s*1_000_000_000_000|non-millisecond|Dialect A fixture manifest bounds",
        "classification": "EXPLICIT_MS_LAW_CHECK",
        "description": "Explicit ms-law or dialect classification check present.",
        "dialect_b_risk": "none",
    },
    {
        "pattern_id": "PRODUCER_AS_SECS",
        "regex": r"as_secs\(\)",
        "classification": "PRODUCER_SECONDS_EMISSION",
        "description": "Producer emits manifest bounds from SystemTime seconds.",
        "dialect_b_risk": "n/a",
    },
    {
        "pattern_id": "CAPTURE_BOUNDS_REFERENCE",
        "regex": r"capture_start|capture_end",
        "classification": "BOUNDS_REFERENCE",
        "description": "References manifest capture bounds.",
        "dialect_b_risk": "context",
    },
)

ESCALATION_TRIGGERS: list[dict[str, str]] = [
    {
        "trigger": "implicit_ms_semantics_on_dialect_b_paths",
        "mandatory": "consumer dialect gate or rename observational fields — no silent normalization",
    },
    {
        "trigger": "probe_pressure_for_rewrite_utilities",
        "mandatory": "stop probe — Lane 2 tranche scope doc required",
    },
    {
        "trigger": "hidden_ms_normalization_in_new_consumer",
        "mandatory": "extend bounded surface list + re-run probe; escalate if normalization detected",
    },
]


@dataclass
class Finding:
    surface: str
    line: int
    pattern_id: str
    classification: str
    description: str
    dialect_b_risk: str
    excerpt: str


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


def load_inventory_summary() -> dict[str, Any]:
    if not INVENTORY_PATH.exists():
        return {"available": False}
    payload = json.loads(INVENTORY_PATH.read_text())
    summary = payload.get("summary", {})
    return {
        "available": True,
        "manifests_scanned": summary.get("manifests_scanned"),
        "seconds_bounds": summary.get("bounds_unit", {}).get("seconds"),
        "live_capture_seconds_bounds": summary.get("live_capture_seconds_bounds"),
        "seconds_ms_ambiguity_count": summary.get("seconds_ms_ambiguity_count"),
    }


def scan_surface(relative_path: str) -> list[Finding]:
    path = REPO_ROOT / relative_path
    if not path.exists():
        return [
            Finding(
                surface=relative_path,
                line=0,
                pattern_id="MISSING_SURFACE",
                classification="PROBE_GAP",
                description="Expected probe surface file not found.",
                dialect_b_risk="unknown",
                excerpt="",
            )
        ]

    text = path.read_text()
    lines = text.splitlines()
    findings: list[Finding] = []

    for rule in PATTERN_RULES:
        regex = re.compile(rule["regex"])
        for line_no, line in enumerate(lines, start=1):
            if regex.search(line):
                findings.append(
                    Finding(
                        surface=relative_path,
                        line=line_no,
                        pattern_id=rule["pattern_id"],
                        classification=rule["classification"],
                        description=rule["description"],
                        dialect_b_risk=rule["dialect_b_risk"],
                        excerpt=line.strip()[:160],
                    )
                )

    return findings


def classify_surface_posture(findings: list[Finding]) -> dict[str, Any]:
    classes = {f.classification for f in findings}
    has_heuristic = "HEURISTIC_UNIT_AWARE" in classes
    has_implicit_naming = "IMPLICIT_MS_SEMANTICS" in classes
    has_bad_arith = any(
        f.classification == "IMPLICIT_MS_ARITHMETIC"
        and not (">" in f.excerpt and ("1e11" in f.excerpt or "1_000_000_000_000" in f.excerpt))
        for f in findings
    )
    implicit = has_implicit_naming or (has_bad_arith and not has_heuristic)

    if implicit and has_heuristic:
        posture = "MIXED_ASSUMPTION_AND_GATE"
    elif implicit:
        posture = "IMPLICIT_MS_ASSUMPTION"
    elif "EXPLICIT_MS_LAW_CHECK" in classes:
        posture = "EXPLICIT_GATE"
    elif has_heuristic:
        posture = "HEURISTIC_UNIT_AWARE"
    elif "PRODUCER_SECONDS_EMISSION" in classes:
        posture = "PRODUCER_SECONDS_EMISSION"
    elif "BOUNDS_REFERENCE" in classes:
        posture = "BOUNDS_REFERENCE_ONLY"
    else:
        posture = "NO_BOUNDS_CONSUMPTION"

    return {
        "posture": posture,
        "implicit_ms_assumption": implicit,
        "dialect_b_sensitive": implicit
        or posture in {"HEURISTIC_UNIT_AWARE", "MIXED_ASSUMPTION_AND_GATE"},
    }


def summarize(findings: list[Finding], surfaces: dict[str, Any]) -> dict[str, Any]:
    by_class: dict[str, int] = {}
    for finding in findings:
        by_class[finding.classification] = by_class.get(finding.classification, 0) + 1

    implicit_surfaces = [
        name
        for name, meta in surfaces.items()
        if meta.get("implicit_ms_assumption")
    ]
    dialect_b_sensitive = [
        name for name, meta in surfaces.items() if meta.get("dialect_b_sensitive")
    ]

    return {
        "surfaces_scanned": len(surfaces),
        "findings_total": len(findings),
        "by_classification": by_class,
        "implicit_ms_assumption_surfaces": implicit_surfaces,
        "dialect_b_sensitive_surfaces": dialect_b_sensitive,
        "surfaces_with_explicit_gate": [
            name for name, meta in surfaces.items() if meta.get("posture") == "EXPLICIT_GATE"
        ],
        "surfaces_heuristic_aware": [
            name
            for name, meta in surfaces.items()
            if meta.get("posture") in {"HEURISTIC_UNIT_AWARE", "MIXED_ASSUMPTION_AND_GATE"}
        ],
    }


def build_probe() -> dict[str, Any]:
    all_findings: list[Finding] = []
    surface_meta: dict[str, Any] = {}

    for surface in PROBE_SURFACES:
        findings = scan_surface(surface)
        all_findings.extend(findings)
        surface_meta[surface] = {
            **classify_surface_posture(findings),
            "finding_count": len(findings),
        }

    summary = summarize(all_findings, surface_meta)
    inventory = load_inventory_summary()

    attestations: list[str] = ["BOUNDED_SURFACE_SCAN_COMPLETE"]
    if summary["implicit_ms_assumption_surfaces"]:
        attestations.append("IMPLICIT_MS_ASSUMPTIONS_DETECTED")
    if summary["surfaces_heuristic_aware"]:
        attestations.append("HEURISTIC_UNIT_GATES_PRESENT")
    if summary["surfaces_with_explicit_gate"]:
        attestations.append("EXPLICIT_MS_LAW_CHECKS_PRESENT")
    if inventory.get("available"):
        attestations.append("INVENTORY_CROSS_REFERENCE_LOADED")

    return {
        "probe_version": PROBE_VERSION,
        "lane": "Lane 2 prerequisite — manifest ms assumption probe",
        "layer": "observational",
        "scope_doc": "docs/governance/V006_MANIFEST_MS_ASSUMPTION_PROBE_SCOPE.md",
        "parent_inventory": "fixtures/lane2/manifest_ms_impact_inventory.json",
        "authority_refs": [
            "V006_MANIFEST_DIALECT_POLICY.md",
            "V006_MANIFEST_MS_IMPACT_INVENTORY_SCOPE.md",
        ],
        "question": "which consumers implicitly assume millisecond semantics?",
        "inventory_cross_reference": inventory,
        "summary": summary,
        "surface_postures": surface_meta,
        "findings": [asdict(f) for f in all_findings],
        "escalation_triggers": ESCALATION_TRIGGERS,
        "attestations": attestations,
        "tranche_authorized": False,
        "git_commit": get_git_commit(),
        "status": "PROBE_COMPLETE",
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
    if ref_commit == cur_commit and reference.get("findings") != current.get("findings"):
        errors.append("FINDINGS_DRIFT")
    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description="Lane 2 manifest ms assumption probe")
    parser.add_argument(
        "--check-reference",
        action="store_true",
        help="Fail if summary/findings drift at same git commit vs committed probe artifact",
    )
    parser.add_argument(
        "--output",
        default=str(PROBE_PATH),
        help="Probe JSON output path",
    )
    args = parser.parse_args()

    print("Lane 2 prerequisite — manifest ms assumption probe", flush=True)
    print("Layer: observational — detect implicit assumptions only\n", flush=True)

    probe = build_probe()
    output_path = Path(args.output)

    failures: list[str] = []
    if args.check_reference:
        failures = check_reference(probe, output_path)

    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(probe, indent=2, sort_keys=True) + "\n")

    print(
        json.dumps(
            {
                "status": probe["status"],
                "summary": probe["summary"],
                "implicit_ms_assumption_surfaces": probe["summary"]["implicit_ms_assumption_surfaces"],
            },
            indent=2,
        ),
        flush=True,
    )
    print(f"Probe artifact: {output_path.relative_to(REPO_ROOT)}", flush=True)

    if failures:
        print(f"[FAIL] Reference check: {failures}", file=sys.stderr)
        return 1

    print("[OK] Manifest ms assumption probe complete.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
