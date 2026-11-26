#!/usr/bin/env python3
"""V-006 Phase B producer ratification verifier.

Pipeline:
  fixture -> inject (chronology_serialize_probe) -> emit -> byte diff -> hash diff -> classify

Generated proof only — does not grant producer authority.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any


REPO_ROOT = Path(__file__).resolve().parents[1]
FIXTURE_ROOT = REPO_ROOT / "fixtures" / "chronology_serialization"
PROBE_BIN = "chronology_serialize_probe"

CLASS_BYTE_IDENTICAL = "byte_identical"
CLASS_HASH_SCOPE_BUG = "hash_identical_semantic_drift"
CLASS_SERIALIZATION_DRIFT = "serialization_drift"
CLASS_DIALECT_DRIFT = "dialect_drift"
CLASS_TIMESTAMP_UNIT_DRIFT = "timestamp_unit_drift"
CLASS_PATH_AUTHORITY_DRIFT = "path_authority_drift"
CLASS_RATIFICATION_BLOCKED = "ratification_blocked"

PRODUCER_PATHS = {
    "capture_daemon": "chronology/live_capture/",
    "historical_importer": "chronology/historical/",
    "yahoo_importer": "chronology/historical/",
}

LAWFUL_ROOT = "infrastructure/core/chronology/"


@dataclass
class FixtureVerdict:
    fixture_id: str
    producer: str
    tick_classification: str
    manifest_classification: str
    producer_static_classification: str
    overall: str
    probe_report: dict[str, Any]
    notes: list[str]


def load_json(path: Path) -> Any:
    return json.loads(path.read_text())


def run_probe(fixture_dir: Path, producer: str) -> dict[str, Any]:
    cmd = [
        "cargo",
        "run",
        "--quiet",
        "--bin",
        PROBE_BIN,
        "--",
        "--fixture-dir",
        str(fixture_dir),
        "--producer",
        producer,
    ]
    proc = subprocess.run(
        cmd,
        cwd=REPO_ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    if proc.returncode != 0:
        raise RuntimeError(
            f"probe failed for {fixture_dir.name}: {proc.stderr.strip() or proc.stdout.strip()}"
        )
    return json.loads(proc.stdout)


def classify_manifest(fixture_dir: Path) -> tuple[str, list[str]]:
    notes: list[str] = []
    manifest = load_json(fixture_dir / "manifest.json")
    meta = load_json(fixture_dir / "fixture_meta.json")
    dialect = meta.get("dialect")

    if dialect == "A":
        required = {
            "source",
            "resolution",
            "capture_method",
            "import_timestamp",
            "capture_start",
            "capture_end",
            "total_ticks",
            "chronology_hash",
            "gaps",
            "substrate",
        }
        if not required.issubset(manifest.keys()):
            notes.append(f"missing manifest keys: {sorted(required - set(manifest.keys()))}")
            return CLASS_DIALECT_DRIFT, notes
        start, end = manifest["capture_start"], manifest["capture_end"]
        if start < 1_000_000_000_000 or end < 1_000_000_000_000:
            notes.append("Dialect A fixture manifest bounds appear non-millisecond")
            return CLASS_TIMESTAMP_UNIT_DRIFT, notes
        return CLASS_BYTE_IDENTICAL, notes

    if dialect == "B":
        required = {
            "substrate",
            "capture_start",
            "capture_end",
            "total_ticks",
            "chronology_hash",
            "gaps",
        }
        if not required.issubset(manifest.keys()):
            notes.append(f"Dialect B missing keys: {sorted(required - set(manifest.keys()))}")
            return CLASS_DIALECT_DRIFT, notes
        notes.append("Dialect B historically admitted; not forward canonical manifest law")
        return CLASS_TIMESTAMP_UNIT_DRIFT, notes

    if dialect == "C":
        if "provenance" not in manifest:
            notes.append("Dialect C missing provenance")
            return CLASS_DIALECT_DRIFT, notes
        return CLASS_BYTE_IDENTICAL, notes

    notes.append(f"unknown dialect {dialect}")
    return CLASS_RATIFICATION_BLOCKED, notes


def classify_producer_static(producer: str, target_dialect_a_live: bool) -> tuple[str, list[str]]:
    notes: list[str] = []
    code_path = PRODUCER_PATHS.get(producer)
    if code_path is None:
        return CLASS_RATIFICATION_BLOCKED, [f"unknown producer {producer}"]

    if not code_path.startswith(LAWFUL_ROOT):
        notes.append(f"code emits under `{code_path}`; lawful root is `{LAWFUL_ROOT}`")
        path_class = CLASS_PATH_AUTHORITY_DRIFT
    else:
        path_class = CLASS_BYTE_IDENTICAL

    if producer == "capture_daemon" and target_dialect_a_live:
        notes.append("forward live law requires Dialect A + millisecond manifest bounds")
        notes.append("capture_daemon currently emits seconds bounds (Phase A defect CD-2)")
        return CLASS_TIMESTAMP_UNIT_DRIFT, notes

    return path_class, notes


def overall_classification(tick: str, manifest: str, static: str) -> str:
    classes = [tick, manifest, static]
    if CLASS_SERIALIZATION_DRIFT in classes or CLASS_HASH_SCOPE_BUG in classes:
        return CLASS_SERIALIZATION_DRIFT
    if CLASS_DIALECT_DRIFT in classes:
        return CLASS_DIALECT_DRIFT
    if CLASS_TIMESTAMP_UNIT_DRIFT in classes:
        return CLASS_TIMESTAMP_UNIT_DRIFT
    if CLASS_PATH_AUTHORITY_DRIFT in classes:
        return CLASS_PATH_AUTHORITY_DRIFT
    if all(c == CLASS_BYTE_IDENTICAL for c in classes):
        return CLASS_BYTE_IDENTICAL
    return CLASS_RATIFICATION_BLOCKED


def verify_fixture(fixture_dir: Path, producer: str) -> FixtureVerdict:
    probe = run_probe(fixture_dir, producer)
    meta = load_json(fixture_dir / "fixture_meta.json")
    tick_class = probe.get("suggested_classification", CLASS_RATIFICATION_BLOCKED)
    manifest_class, manifest_notes = classify_manifest(fixture_dir)
    static_class, static_notes = classify_producer_static(
        producer, target_dialect_a_live=(meta.get("dialect") == "B")
    )
    notes = manifest_notes + static_notes
    overall = overall_classification(tick_class, manifest_class, static_class)
    return FixtureVerdict(
        fixture_id=meta.get("fixture_id", fixture_dir.name),
        producer=producer,
        tick_classification=tick_class,
        manifest_classification=manifest_class,
        producer_static_classification=static_class,
        overall=overall,
        probe_report=probe,
        notes=notes,
    )


def main() -> int:
    parser = argparse.ArgumentParser(description="V-006 producer ratification verifier")
    parser.add_argument(
        "--producer",
        default="capture_daemon",
        choices=sorted(PRODUCER_PATHS.keys()),
    )
    parser.add_argument(
        "--fixture",
        action="append",
        help="Fixture directory name under fixtures/chronology_serialization (repeatable)",
    )
    parser.add_argument(
        "--report",
        default=str(FIXTURE_ROOT / "ratification_report.json"),
        help="Write JSON report path",
    )
    args = parser.parse_args()

    fixture_dirs = sorted(FIXTURE_ROOT.iterdir())
    if args.fixture:
        names = set(args.fixture)
        fixture_dirs = [p for p in fixture_dirs if p.name in names]

    fixture_dirs = [p for p in fixture_dirs if (p / "fixture_meta.json").exists()]
    if not fixture_dirs:
        print("No fixtures found.")
        return 1

    verdicts: list[dict[str, Any]] = []
    failures = 0
    for fixture_dir in fixture_dirs:
        verdict = verify_fixture(fixture_dir, args.producer)
        verdicts.append(
            {
                "fixture_id": verdict.fixture_id,
                "producer": verdict.producer,
                "tick_classification": verdict.tick_classification,
                "manifest_classification": verdict.manifest_classification,
                "producer_static_classification": verdict.producer_static_classification,
                "overall": verdict.overall,
                "notes": verdict.notes,
                "probe_report": verdict.probe_report,
            }
        )
        status = "PASS" if verdict.overall == CLASS_BYTE_IDENTICAL else "FAIL"
        if status == "FAIL":
            failures += 1
        print(
            f"{status}: {verdict.fixture_id} overall={verdict.overall} "
            f"tick={verdict.tick_classification} manifest={verdict.manifest_classification} "
            f"static={verdict.producer_static_classification}"
        )

    report = {
        "producer": args.producer,
        "ratification_granted": False,
        "phase": "B",
        "classification_taxonomy": [
            CLASS_BYTE_IDENTICAL,
            CLASS_HASH_SCOPE_BUG,
            CLASS_SERIALIZATION_DRIFT,
            CLASS_DIALECT_DRIFT,
            CLASS_TIMESTAMP_UNIT_DRIFT,
            CLASS_PATH_AUTHORITY_DRIFT,
            CLASS_RATIFICATION_BLOCKED,
        ],
        "verdicts": verdicts,
        "summary": {
            "fixtures_tested": len(verdicts),
            "byte_identical_count": sum(
                1 for v in verdicts if v["overall"] == CLASS_BYTE_IDENTICAL
            ),
            "failures": failures,
        },
    }

    report_path = Path(args.report)
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text(json.dumps(report, indent=2) + "\n")
    print(f"Report: {report_path}")
    print("Ratification granted: False (by design until Phase C)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
