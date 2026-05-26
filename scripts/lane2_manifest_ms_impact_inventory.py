#!/usr/bin/env python3
"""
Lane 2 prerequisite — manifest ms correction impact inventory.

Observational only: scan persisted manifests, classify bounds units, map consumer
surfaces and escalation triggers. No migration, rewrite, or producer edits.

See: docs/governance/V006_MANIFEST_MS_IMPACT_INVENTORY_SCOPE.md
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[1]
INVENTORY_PATH = REPO_ROOT / "fixtures" / "lane2" / "manifest_ms_impact_inventory.json"
INVENTORY_VERSION = "1.0.0"
MS_THRESHOLD = 1_000_000_000_000

SCAN_ROOTS = (
    REPO_ROOT / "core" / "chronology",
    REPO_ROOT / "fixtures" / "chronology_serialization",
)

MIGRATION_FIXTURES = (
    "fixtures/chronology_serialization/historical_dialect_a_cpi_shock_tick",
    "fixtures/chronology_serialization/live_dialect_b_btcusdt_1779545161",
    "fixtures/chronology_serialization/yahoo_dialect_c_btcusd_crossfeed",
)

CONSUMER_SURFACES: list[dict[str, str]] = [
    {
        "surface": "scripts/rebuild_catalog.py",
        "observation": "Uses start_ts > 1e11 heuristic to choose ms vs seconds for catalog dates.",
        "bounds_sensitive": "yes",
        "replay_coupling": "catalog interpretation only; not replay hash",
    },
    {
        "surface": "scripts/run_tier1_observability_demo.py",
        "observation": "Records capture_start/end in provenance as capture_start_ms without dialect gate.",
        "bounds_sensitive": "yes",
        "replay_coupling": "observational provenance; replay uses JSONL bytes + chronology_hash",
    },
    {
        "surface": "scripts/verify_chronology_producer_ratification.py",
        "observation": "Classifies Dialect A ms bounds vs admitted Dialect B seconds defect.",
        "bounds_sensitive": "yes",
        "replay_coupling": "none — evidence harness",
    },
    {
        "surface": "scripts/verify_chronology_byte_fixtures.py",
        "observation": "Validates dialect-shaped manifest keys on frozen fixtures.",
        "bounds_sensitive": "indirect",
        "replay_coupling": "none — fixture integrity",
    },
    {
        "surface": "scripts/emit_manifest_v1.py",
        "observation": "Binds certification manifest to substrate file hash; does not consume capture bounds.",
        "bounds_sensitive": "no",
        "replay_coupling": "downstream certification only",
    },
    {
        "surface": "core/src/bin/capture_daemon.rs",
        "observation": "Emits Dialect A-shaped manifest; capture_start/end from as_secs() (CD-2 defect).",
        "bounds_sensitive": "yes",
        "replay_coupling": "forward producer; not replay hash if JSONL unchanged",
    },
    {
        "surface": "core/src/bin/historical_importer.rs",
        "observation": "Emits Dialect A with millisecond capture_start/end.",
        "bounds_sensitive": "yes",
        "replay_coupling": "none on bounds alone",
    },
    {
        "surface": "core/src/bin/yahoo_importer.rs",
        "observation": "Emits millisecond capture_start/end (Dialect C with provenance).",
        "bounds_sensitive": "yes",
        "replay_coupling": "none on bounds alone",
    },
]

ESCALATION_TRIGGERS: list[dict[str, str]] = [
    {
        "trigger": "batch_manifest_bounds_rewrite_seconds_to_ms",
        "mandatory": "Lane 2 tranche scope doc + AUTHORITY_MAP update + catalog reinterpretation declaration",
    },
    {
        "trigger": "capture_daemon_producer_alignment",
        "mandatory": "V-006 producer ratification path; prove byte-stable JSONL; manifest-only bounds change",
    },
    {
        "trigger": "tool_assumes_ms_on_dialect_b",
        "mandatory": "consumer inventory fix or explicit dialect classification gate",
    },
    {
        "trigger": "schema_field_semantic_weight",
        "mandatory": "stop inventory expansion; Lane 2 or AUTHORITY_MAP escalation",
    },
]


@dataclass
class ManifestRecord:
    path: str
    lineage: str
    inferred_dialect: str
    bounds_unit: str
    tick_timestamp_unit: str | None
    capture_start: int | None
    capture_end: int | None
    chronology_hash_prefix: str | None
    seconds_ms_ambiguity: bool
    forward_law_violation: bool
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


def load_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text())


def classify_bounds_unit(start: Any, end: Any) -> str:
    if start is None or end is None:
        return "MISSING"
    if not isinstance(start, int) or not isinstance(end, int):
        return "NON_INTEGER"
    start_ms = start >= MS_THRESHOLD
    end_ms = end >= MS_THRESHOLD
    if start_ms and end_ms:
        return "milliseconds"
    if not start_ms and not end_ms:
        return "seconds"
    return "MIXED"


def infer_dialect(manifest: dict[str, Any], lineage: str) -> str:
    if "provenance" in manifest:
        return "C"
    if "source" in manifest and "capture_method" in manifest:
        return "A"
    if lineage == "live_capture" and set(manifest.keys()) >= {
        "substrate",
        "capture_start",
        "capture_end",
        "chronology_hash",
    }:
        return "B"
    if "capture_start" in manifest:
        return "UNKNOWN"
    return "UNKNOWN"


def infer_lineage(path: Path) -> str:
    rel = path.relative_to(REPO_ROOT).as_posix()
    if "/live_capture/" in rel or rel.startswith("core/chronology/live_capture"):
        return "live_capture"
    if "/historical/" in rel:
        return "historical"
    if "/chronology_serialization/" in rel:
        return "byte_fixture"
    if "live_capture_" in path.name:
        return "live_capture_legacy"
    return "other"


def sibling_jsonl(manifest_path: Path) -> Path | None:
    name = manifest_path.name
    if name.endswith("_manifest.json"):
        candidate = manifest_path.with_name(name.replace("_manifest.json", ".jsonl"))
        if candidate.exists():
            return candidate
    for jsonl in manifest_path.parent.glob("*.jsonl"):
        if jsonl.name != manifest_path.name:
            return jsonl
    return None


def first_tick_timestamp_unit(jsonl_path: Path) -> str | None:
    for line in jsonl_path.read_text().splitlines():
        line = line.strip()
        if not line:
            continue
        tick = json.loads(line)
        ts = tick.get("timestamp")
        if not isinstance(ts, int):
            return "NON_INTEGER"
        return "milliseconds" if ts >= MS_THRESHOLD else "seconds"
    return None


def scan_manifest(manifest_path: Path) -> ManifestRecord:
    manifest = load_json(manifest_path)
    lineage = infer_lineage(manifest_path)
    dialect = infer_dialect(manifest, lineage)
    start = manifest.get("capture_start")
    end = manifest.get("capture_end")
    bounds = classify_bounds_unit(start, end)

    jsonl = sibling_jsonl(manifest_path)
    tick_unit = first_tick_timestamp_unit(jsonl) if jsonl else None

    notes: list[str] = []
    ambiguity = bounds == "MIXED" or (
        tick_unit == "milliseconds" and bounds == "seconds"
    )
    forward_violation = bounds == "seconds" and lineage in {
        "live_capture",
        "live_capture_legacy",
    }

    if dialect == "B" and bounds == "seconds":
        notes.append("historically admitted Dialect B seconds bounds defect")
    if dialect == "A" and bounds == "seconds":
        notes.append("unexpected seconds bounds on Dialect A manifest")
        forward_violation = True
    if tick_unit == "milliseconds" and bounds == "seconds":
        notes.append("tick timestamps ms; manifest bounds seconds")
    if manifest_path.as_posix().endswith("fixtures/chronology_serialization/live_dialect_b_btcusdt_1779545161/manifest.json"):
        notes.append("frozen migration constraint fixture")

    ch = manifest.get("chronology_hash")
    ch_prefix = ch[:16] if isinstance(ch, str) and len(ch) >= 16 else None

    return ManifestRecord(
        path=manifest_path.relative_to(REPO_ROOT).as_posix(),
        lineage=lineage,
        inferred_dialect=dialect,
        bounds_unit=bounds,
        tick_timestamp_unit=tick_unit,
        capture_start=start if isinstance(start, int) else None,
        capture_end=end if isinstance(end, int) else None,
        chronology_hash_prefix=ch_prefix,
        seconds_ms_ambiguity=ambiguity,
        forward_law_violation=forward_violation,
        notes=notes,
    )


def collect_manifests() -> list[ManifestRecord]:
    records: list[ManifestRecord] = []
    seen: set[str] = set()
    for root in SCAN_ROOTS:
        if not root.exists():
            continue
        for path in sorted(root.rglob("*manifest*.json")):
            key = path.resolve().as_posix()
            if key in seen:
                continue
            seen.add(key)
            try:
                records.append(scan_manifest(path))
            except (json.JSONDecodeError, OSError) as exc:
                records.append(
                    ManifestRecord(
                        path=path.relative_to(REPO_ROOT).as_posix(),
                        lineage=infer_lineage(path),
                        inferred_dialect="UNREADABLE",
                        bounds_unit="UNREADABLE",
                        tick_timestamp_unit=None,
                        capture_start=None,
                        capture_end=None,
                        chronology_hash_prefix=None,
                        seconds_ms_ambiguity=True,
                        forward_law_violation=False,
                        notes=[f"scan_error: {exc}"],
                    )
                )
    return records


def summarize(records: list[ManifestRecord]) -> dict[str, Any]:
    def count(key: str, val: str) -> int:
        return sum(1 for r in records if getattr(r, key) == val)

    return {
        "manifests_scanned": len(records),
        "bounds_unit": {
            "milliseconds": count("bounds_unit", "milliseconds"),
            "seconds": count("bounds_unit", "seconds"),
            "mixed": count("bounds_unit", "MIXED"),
            "missing": count("bounds_unit", "MISSING"),
            "non_integer": count("bounds_unit", "NON_INTEGER"),
            "unreadable": count("bounds_unit", "UNREADABLE"),
        },
        "inferred_dialect": {
            "A": count("inferred_dialect", "A"),
            "B": count("inferred_dialect", "B"),
            "C": count("inferred_dialect", "C"),
            "unknown": count("inferred_dialect", "UNKNOWN"),
        },
        "seconds_ms_ambiguity_count": sum(1 for r in records if r.seconds_ms_ambiguity),
        "forward_law_violation_count": sum(1 for r in records if r.forward_law_violation),
        "live_capture_seconds_bounds": sum(
            1
            for r in records
            if r.lineage in {"live_capture", "live_capture_legacy"}
            and r.bounds_unit == "seconds"
        ),
        "historical_millisecond_bounds": sum(
            1 for r in records if r.lineage == "historical" and r.bounds_unit == "milliseconds"
        ),
    }


def build_inventory() -> dict[str, Any]:
    records = collect_manifests()
    summary = summarize(records)
    return {
        "inventory_version": INVENTORY_VERSION,
        "lane": "Lane 2 prerequisite — manifest ms impact inventory",
        "layer": "observational",
        "scope_doc": "docs/governance/V006_MANIFEST_MS_IMPACT_INVENTORY_SCOPE.md",
        "authority_refs": [
            "V006_MANIFEST_DIALECT_POLICY.md",
            "V006_LIVE_CAPTURE_AUTHORITY_DECISION.md",
            "V006_PHASE_C_CLASSIFICATION.md",
        ],
        "doctrine": {
            "forward_law": "manifest capture_start/capture_end must be milliseconds",
            "historical_defect": "Dialect B live manifests store seconds bounds; ticks remain ms",
            "correction_class": "manifest metadata migration — chronology_hash unchanged if JSONL unchanged",
            "ms_threshold": MS_THRESHOLD,
        },
        "summary": summary,
        "migration_constraint_fixtures": list(MIGRATION_FIXTURES),
        "consumer_surfaces": CONSUMER_SURFACES,
        "escalation_triggers": ESCALATION_TRIGGERS,
        "manifests": [asdict(r) for r in records],
        "tranche_authorized": False,
        "git_commit": get_git_commit(),
        "status": "INVENTORY_COMPLETE",
    }


def check_reference(current: dict[str, Any], reference_path: Path) -> list[str]:
    if not reference_path.exists():
        return ["REFERENCE_MISSING"]
    reference = load_json(reference_path)
    errors: list[str] = []
    if reference.get("summary") != current.get("summary"):
        errors.append("SUMMARY_DRIFT")
    ref_commit = reference.get("git_commit")
    cur_commit = current.get("git_commit")
    if ref_commit == cur_commit and reference.get("manifests") != current.get("manifests"):
        errors.append("MANIFEST_RECORDS_DRIFT")
    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description="Lane 2 manifest ms impact inventory")
    parser.add_argument(
        "--check-reference",
        action="store_true",
        help="Fail if summary/records drift at same git commit vs committed inventory",
    )
    parser.add_argument(
        "--output",
        default=str(INVENTORY_PATH),
        help="Inventory JSON output path",
    )
    args = parser.parse_args()

    print("Lane 2 prerequisite — manifest ms impact inventory", flush=True)
    print("Layer: observational — map exposure only\n", flush=True)

    inventory = build_inventory()
    output_path = Path(args.output)
    output_path.parent.mkdir(parents=True, exist_ok=True)

    failures: list[str] = []
    if args.check_reference:
        failures = check_reference(inventory, output_path)

    output_path.write_text(json.dumps(inventory, indent=2, sort_keys=True) + "\n")

    summary = inventory["summary"]
    print(json.dumps({"status": inventory["status"], "summary": summary}, indent=2), flush=True)
    print(f"Inventory: {output_path.relative_to(REPO_ROOT)}", flush=True)

    if failures:
        print(f"[FAIL] Reference check: {failures}", file=sys.stderr)
        return 1

    print("[OK] Manifest ms impact inventory complete.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
