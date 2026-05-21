#!/usr/bin/env python3
"""
Bounded ingest parity — compare Python run_nse_cohort vs cs-ingest replay-step.

Same frozen substrate, same interval window, fresh archives.

Usage:
  python3 scripts/compare_ingest_parity.py --batch-id 900 \\
    --start-interval 0 --max-intervals 5 --run

Cross-runtime parity uses legacy Python (--python-ingest) vs cs-ingest.
After promotion, `run_nse_cohort.py --from-frozen` alone is cs-ingest only.
"""

from __future__ import annotations

import argparse
import json
import math
import subprocess
import sys
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))

from archive_dedupe import iter_gzip_jsonl

COMPARE_FIELDS = [
    "ts",
    "symbol",
    "pc1",
    "pc2",
    "dist_to_centroid",
    "state",
    "entropy",
    "velocity",
    "acceleration",
    "turn_angle",
    "transition_confidence",
    "local_density",
    "corridor",
    "previous_state",
    "next_state",
    "dwell_duration",
    "instability_type",
    "survival_probability",
    "hazard_rate",
    "precursor_decay_velocity",
    "precursor_entropy_expansion",
    "precursor_density_thinning",
    "precursor_curvature_destabilization",
    "precursor_leakage_rate",
    "queue_pressure",
    "spread_elasticity",
]


def load_archive_records(archive_dir: Path) -> dict[tuple[str, int], dict]:
    out: dict[tuple[str, int], dict] = {}
    raw = archive_dir / "raw"
    if not raw.is_dir():
        return out
    for sym_dir in raw.iterdir():
        if not sym_dir.is_dir():
            continue
        symbol = sym_dir.name
        for gz in sym_dir.glob("telemetry_stream_*.jsonl.gz"):
            for _ln, rec, err in iter_gzip_jsonl(gz):
                if err or rec is None:
                    continue
                ts = int(rec["ts"])
                out[(symbol, ts)] = rec
    return out


FIELD_TOL = {
    "turn_angle": 0.05,
    "acceleration": 0.002,
    "velocity": 0.002,
    "pc1": 0.0002,
    "pc2": 0.0002,
    "dist_to_centroid": 0.0002,
    "precursor_curvature_destabilization": 0.05,
    "precursor_leakage_rate": 0.0002,
    "precursor_entropy_expansion": 0.0002,
}


def float_eq(a, b, key: str = "") -> bool:
    if a is None and b is None:
        return True
    if a is None or b is None:
        return False
    try:
        tol = FIELD_TOL.get(key, 1e-4)
        return math.isclose(float(a), float(b), rel_tol=0, abs_tol=tol)
    except (TypeError, ValueError):
        return a == b


def compare_records(py_rec: dict, rs_rec: dict) -> tuple[list[str], list[str]]:
    exact_diffs = []
    semantic_diffs = []
    for key in COMPARE_FIELDS:
        pv, rv = py_rec.get(key), rs_rec.get(key)
        if isinstance(pv, (int, float)) or isinstance(rv, (int, float)):
            if pv != rv and not float_eq(pv, rv, key):
                semantic_diffs.append(f"{key}: py={pv!r} rust={rv!r}")
            elif pv != rv:
                exact_diffs.append(f"{key}: py={pv!r} rust={rv!r}")
        elif pv != rv:
            semantic_diffs.append(f"{key}: py={pv!r} rust={rv!r}")
    # Known edge: turn_angle when velocity rounds to 0.0 (Python float residue)
    if semantic_diffs and len(semantic_diffs) == 1:
        only = semantic_diffs[0]
        if only.startswith("turn_angle:") and float_eq(
            py_rec.get("velocity", 0), 0, "velocity"
        ) and float_eq(rs_rec.get("velocity", 0), 0, "velocity"):
            semantic_diffs.clear()
    if py_rec.get("corridor") and rs_rec.get("corridor"):
        if py_rec.get("corridor_id") != rs_rec.get("corridor_id"):
            cid_diff = (
                f"corridor_id: py={py_rec.get('corridor_id')!r} "
                f"rust={rs_rec.get('corridor_id')!r}"
            )
            if py_rec.get("corridor_id") and rs_rec.get("corridor_id"):
                semantic_diffs.append(cid_diff)
            else:
                exact_diffs.append(cid_diff)
    return exact_diffs, semantic_diffs


def run_legacy_python_ingest(
    batch_id: int,
    run_label: str,
    start_interval: int,
    max_intervals: int,
) -> None:
    """Legacy Python ingest path — required for cross-runtime parity after cs-ingest promotion."""
    cmd = [
        sys.executable,
        str(ROOT / "scripts" / "run_nse_cohort.py"),
        "--batch-id",
        str(batch_id),
        "--from-frozen",
        "--python-ingest",
        "--fresh",
        "--run-label",
        run_label,
        "--start-interval",
        str(start_interval),
        "--max-intervals",
        str(max_intervals),
    ]
    subprocess.run(cmd, cwd=ROOT, check=True)


def run_rust_replay(
    batch_id: int,
    archive: Path,
    cohort: Path,
    start_interval: int,
    max_intervals: int,
) -> subprocess.CompletedProcess:
    binary = ROOT / "target" / "release" / "cs-ingest"
    if not binary.exists():
        subprocess.run(
            ["cargo", "build", "-p", "cs-ingest", "--release"],
            cwd=ROOT,
            check=True,
        )
    return subprocess.run(
        [
            str(binary),
            "replay-step",
            "--batch-id",
            str(batch_id),
            "--cohort",
            str(cohort),
            "--archive",
            str(archive),
            "--start-interval",
            str(start_interval),
            "--max-intervals",
            str(max_intervals),
            "--fresh",
        ],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    )


def compare_archives(py_dir: Path, rs_dir: Path) -> dict:
    py_recs = load_archive_records(py_dir)
    rs_recs = load_archive_records(rs_dir)
    py_keys = set(py_recs)
    rs_keys = set(rs_recs)

    report: dict = {
        "py_ticks": len(py_keys),
        "rust_ticks": len(rs_keys),
        "keys_match": py_keys == rs_keys,
        "only_python": sorted(py_keys - rs_keys)[:20],
        "only_rust": sorted(rs_keys - py_keys)[:20],
        "rounding_only": [],
        "semantic_mismatches": [],
        "semantic_mismatch_count": 0,
        "rounding_only_count": 0,
        "exact_match_count": 0,
        "per_ts": defaultdict(lambda: {"py": 0, "rust": 0, "match": 0}),
    }

    for key in py_keys & rs_keys:
        ts = key[1]
        report["per_ts"][ts]["py"] += 1
        report["per_ts"][ts]["rust"] += 1
        exact_diffs, semantic_diffs = compare_records(py_recs[key], rs_recs[key])
        if semantic_diffs:
            report["semantic_mismatch_count"] += 1
            if len(report["semantic_mismatches"]) < 30:
                report["semantic_mismatches"].append(
                    {"key": key, "diffs": semantic_diffs}
                )
        elif exact_diffs:
            report["rounding_only_count"] += 1
            if len(report["rounding_only"]) < 30:
                report["rounding_only"].append({"key": key, "diffs": exact_diffs})
            report["per_ts"][ts]["match"] += 1
        else:
            report["exact_match_count"] += 1
            report["per_ts"][ts]["match"] += 1

    report["per_ts"] = dict(sorted(report["per_ts"].items()))
    report["matched_ticks"] = (
        report["exact_match_count"] + report["rounding_only_count"]
    )
    report["semantic_valid"] = (
        report["keys_match"]
        and report["py_ticks"] == report["rust_ticks"]
        and not report["semantic_mismatches"]
    )
    report["exact_valid"] = report["semantic_valid"] and not report["rounding_only"]
    report["valid"] = report["semantic_valid"]
    return report


def main():
    ap = argparse.ArgumentParser(description="Python vs Rust ingest parity gate")
    ap.add_argument("--batch-id", type=int, default=900)
    ap.add_argument("--start-interval", type=int, default=0)
    ap.add_argument("--max-intervals", type=int, default=5)
    ap.add_argument(
        "--run",
        action="store_true",
        help="Execute both ingest paths before comparing",
    )
    ap.add_argument("--py-archive", type=Path, default=None)
    ap.add_argument("--rust-archive", type=Path, default=None)
    args = ap.parse_args()

    cohort = ROOT / f"cohorts/batch_{args.batch_id:03d}.txt"
    py_dir = args.py_archive or (
        ROOT / "state_archive" / "batches" / f"batch_{args.batch_id:03d}"
        / "runs" / "parity_py"
    )
    rs_dir = args.rust_archive or (
        ROOT / "state_archive" / "batches" / f"batch_{args.batch_id:03d}"
        / "runs" / "parity_rust"
    )

    if args.run:
        print("=" * 60)
        print("INGEST PARITY — LEGACY PYTHON (--python-ingest)")
        print("=" * 60)
        run_legacy_python_ingest(
            args.batch_id, "parity_py", args.start_interval, args.max_intervals
        )
        print("\n" + "=" * 60)
        print("INGEST PARITY — CS-INGEST (replay-step)")
        print("=" * 60)
        rust_proc = run_rust_replay(
            args.batch_id,
            rs_dir,
            cohort,
            args.start_interval,
            args.max_intervals,
        )
        rust_out = rust_proc.stdout or ""
        if "dedupe_skip" in rust_out and "persisted 0" in rust_out:
            print(
                "❌ Rust replay wrote 0 ticks (likely stale dedupe index). "
                "Use --fresh on replay-step.",
                file=sys.stderr,
            )
            sys.exit(2)

    print("\n" + "=" * 60)
    print("INGEST PARITY — COMPARE")
    print("=" * 60)
    report = compare_archives(py_dir, rs_dir)
    print(f"  Window            : start={args.start_interval} max={args.max_intervals}")
    print(f"  Legacy Python ticks : {report['py_ticks']}")
    print(f"  cs-ingest ticks     : {report['rust_ticks']}")
    print(f"  Keys identical    : {report['keys_match']}")
    print(f"  Exact match       : {report['exact_match_count']}/{report['py_ticks']}")
    print(f"  Rounding-only     : {report['rounding_only_count']}/{report['py_ticks']}")
    print(f"  Semantic diffs    : {report['semantic_mismatch_count']}/{report['py_ticks']}")
    print(f"  Semantically OK   : {report['matched_ticks']}/{report['py_ticks']}")
    print(
        f"  Verdict           : "
        f"{'SEMANTIC_PARITY' if report['semantic_valid'] else 'DIVERGENT'}"
    )
    if report.get("exact_valid"):
        print("  Exact byte fields : all matched")

    if report["only_python"]:
        print(f"  Only in Python    : {len(report['only_python'])} keys (showing 5)")
        for k in report["only_python"][:5]:
            print(f"    {k}")
    if report["only_rust"]:
        print(f"  Only in Rust      : {len(report['only_rust'])} keys (showing 5)")
        for k in report["only_rust"][:5]:
            print(f"    {k}")
    if report["semantic_mismatches"]:
        print(f"  Semantic diffs    : {len(report['semantic_mismatches'])} (sample)")
        for item in report["semantic_mismatches"][:5]:
            print(f"    {item['key']}: {item['diffs'][:3]}")
    elif report["rounding_only"]:
        print("  Note              : diffs are float-rounding only (turn_angle, acceleration)")
        for item in report["rounding_only"][:3]:
            print(f"    {item['key']}: {item['diffs'][:2]}")

    out_path = (
        ROOT / "state_archive" / "metadata" / f"ingest_parity_batch_{args.batch_id:03d}.json"
    )
    out_path.parent.mkdir(parents=True, exist_ok=True)
    with open(out_path, "w") as f:
        json.dump(
            {
                "batch_id": args.batch_id,
                "start_interval": args.start_interval,
                "max_intervals": args.max_intervals,
                "py_archive": str(py_dir),
                "rust_archive": str(rs_dir),
                **report,
            },
            f,
            indent=2,
            default=str,
        )
    print(f"  Report            : {out_path}")
    print("=" * 60)
    sys.exit(0 if report["valid"] else 1)


if __name__ == "__main__":
    main()
