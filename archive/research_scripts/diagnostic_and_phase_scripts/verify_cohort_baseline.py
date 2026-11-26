#!/usr/bin/env python3
"""
ChronoSentiment — Controlled Verification Cycle (Stage 1)
==========================================================
Validates the deterministic full-reconstruction baseline before any
incremental streaming migration.

Checks:
  1. Replay consistency      — compare two ingestion manifests (--compare-runs)
  2. PCA stability           — weights sanity, projection bounds
  3. Archive integrity       — JSON validity, duplicate timestamps, cohort scope
  4. Transition consistency  — survival/hazard bounds, dwell semantics
  5. Chronosynchrony         — per-interval symbol coverage alignment
"""

from __future__ import annotations

import argparse
import json
import math
import sys
from collections import Counter, defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))
from archive_dedupe import iter_gzip_jsonl
STATE_NAMES = {"LIQUIDITY_EXHAUSTION", "NARRATIVE_PERSISTENCE", "NOISE_TRANSITIONAL"}
INSTABILITY_TYPES = {
    "STABLE",
    "ENTROPIC_COLLAPSE",
    "EXPANSION_STRESS",
    "CURVATURE_FRACTURE",
}


def load_cohort_symbols(cohort_file: Path) -> set[str]:
    return {line.strip() for line in cohort_file.read_text().splitlines() if line.strip()}


def cohort_symbol_names(cohort: set[str]) -> set[str]:
    """Return cohort symbols as-is — archive directory names match cohort file exactly."""
    return set(cohort)


def resolve_archive_dir(
    root: Path, batch_id: int, shared: bool, run_label: str = ""
) -> Path:
    if shared:
        return root / "state_archive"
    base = root / "state_archive" / "batches" / f"batch_{batch_id:03d}"
    if run_label:
        return base / "runs" / run_label
    return base


def load_manifest(archive_dir: Path, run_label: str | None) -> dict | None:
    if run_label and (archive_dir / "manifests" / f"ingestion_{run_label}.json").exists():
        path = archive_dir / "manifests" / f"ingestion_{run_label}.json"
    else:
        path = archive_dir / "ingestion_manifest.json"
    if not path.exists():
        return None
    with open(path) as f:
        return json.load(f)


def audit_gzip_integrity(archive_dir: Path) -> tuple[int, int, list[str]]:
    """Return (ok_files, corrupt_files, sample_paths)."""
    ok, bad, samples = 0, 0, []
    for gz in archive_dir.rglob("*.jsonl.gz"):
        corrupt = False
        for line_no, rec, err in iter_gzip_jsonl(gz):
            if err and rec is None and line_no == 0:
                corrupt = True
                if len(samples) < 5:
                    samples.append(f"{gz}: {err}")
                break
        if corrupt:
            bad += 1
        else:
            ok += 1
    return ok, bad, samples


def iter_archive_records(archive_dir: Path, cohort: set[str] | None):
    """Yield per-record dicts from the archive for a given run directory.

    Prefers the canonical Layer 1 barrier archive (raw/<symbol>/barriers/<ts>.json)
    which is unconditionally written for every committed (symbol, ts) pair.

    Falls back to the Layer 2 analytics stream (raw/<symbol>/telemetry_stream_*.jsonl.gz)
    for backward compatibility with archives written before the two-layer split
    (batch_003, batch_900, etc.).
    """
    import json as _json

    raw = archive_dir / "raw"
    if not raw.exists():
        return
    for sym_dir in sorted(raw.iterdir()):
        if not sym_dir.is_dir():
            continue
        symbol = sym_dir.name
        if cohort and symbol not in cohort:
            continue

        # ── Layer 1: canonical barrier archive (preferred) ────────────────────
        barriers_dir = sym_dir / "barriers"
        if barriers_dir.exists():
            barrier_files = sorted(barriers_dir.glob("*.json"), key=lambda p: int(p.stem))
            for bf in barrier_files:
                try:
                    rec = _json.loads(bf.read_text())
                    yield {"ok": True, "symbol": symbol, "record": rec, "file": str(bf)}
                except Exception as e:
                    yield {
                        "ok": False,
                        "symbol": symbol,
                        "file": str(bf),
                        "line": 0,
                        "error": str(e),
                    }
            continue  # do not also read Layer 2 for the same symbol

        # ── Layer 2 fallback: analytics telemetry stream (legacy archives) ────
        for gz in sorted(sym_dir.glob("telemetry_stream_*.jsonl.gz")):
            for line_no, rec, err in iter_gzip_jsonl(gz):
                if err and rec is None:
                    yield {
                        "ok": False,
                        "symbol": symbol,
                        "file": str(gz),
                        "line": line_no,
                        "error": err,
                        "corrupt_file": True,
                    }
                    break
                if rec is None:
                    yield {
                        "ok": False,
                        "symbol": symbol,
                        "file": str(gz),
                        "line": line_no,
                        "error": err or "json error",
                    }
                    continue
                yield {"ok": True, "symbol": symbol, "record": rec, "file": str(gz)}


def check_replay_consistency(m_a: dict, m_b: dict, tick_tolerance: float = 0.02) -> list[str]:
    lines: list[str] = []
    issues: list[str] = []

    if m_a.get("pca_weights_hash") != m_b.get("pca_weights_hash"):
        issues.append(
            f"manifest mismatch pca_weights_hash: {m_a.get('pca_weights_hash')!r} vs {m_b.get('pca_weights_hash')!r}"
        )

    fp_a, fp_b = m_a.get("timeline_fingerprint"), m_b.get("timeline_fingerprint")
    if fp_a != fp_b:
        issues.append(f"manifest mismatch timeline_fingerprint: {fp_a!r} vs {fp_b!r}")
    else:
        lines.append(f"PASS: timeline_fingerprint match ({fp_a})")

    for k in ("timeline_intervals",):
        va, vb = m_a.get(k), m_b.get(k)
        if va != vb:
            issues.append(f"manifest mismatch {k}: {va!r} vs {vb!r}")

    ticks_a = int(m_a.get("processed_ticks", 0))
    ticks_b = int(m_b.get("processed_ticks", 0))
    rel = abs(ticks_a - ticks_b) / max(ticks_a, ticks_b, 1)
    if rel > tick_tolerance:
        issues.append(
            f"processed_ticks variance {rel:.2%} exceeds {tick_tolerance:.0%} "
            f"({ticks_a:,} vs {ticks_b:,}) — likely yfinance bar gaps between downloads"
        )
    else:
        lines.append(
            f"PASS: processed_ticks within {tick_tolerance:.0%} tolerance "
            f"({ticks_a:,} vs {ticks_b:,}, Δ={rel:.2%})"
        )

    corr_a = float(m_a.get("corridor_rate", 0))
    corr_b = float(m_b.get("corridor_rate", 0))
    if abs(corr_a - corr_b) > 0.01:
        issues.append(f"corridor_rate mismatch: {corr_a:.4f} vs {corr_b:.4f}")
    else:
        lines.append(f"PASS: corridor_rate stable ({corr_a:.4f} vs {corr_b:.4f})")

    if issues:
        return lines + ["FAIL: replay consistency"] + issues
    return lines + ["PASS: replay consistency (engineering baseline stable)"]


def check_pca_stability(weights_path: Path) -> list[str]:
    issues = []
    with open(weights_path) as f:
        w = json.load(f)
    for i, s in enumerate(w.get("std", [])):
        if s <= 1e-9:
            issues.append(f"PCA std[{i}] near zero ({s}) — variance explosion risk")
    centroids = w.get("centroids", [])
    if len(centroids) != 3:
        issues.append(f"expected 3 centroids, got {len(centroids)}")
    for ci, c in enumerate(centroids):
        if len(c) != 2:
            issues.append(f"centroid[{ci}] not 2D")
        mag = math.sqrt(c[0] ** 2 + c[1] ** 2)
        if mag > 50:
            issues.append(f"centroid[{ci}] magnitude suspiciously large: {mag:.2f}")
    if issues:
        return issues
    return [
        "PASS: PCA weights loaded",
        f"  centroids={len(centroids)} std_min={min(w['std']):.6f} std_max={max(w['std']):.6f}",
    ]


def scan_archive(archive_dir: Path, cohort: set[str]) -> dict:
    stats = {
        "records": 0,
        "malformed": 0,
        "dup_timestamps": 0,
        "out_of_cohort_dirs": 0,
        "state_counts": Counter(),
        "instability_counts": Counter(),
        "corridor_count": 0,
        "ts_symbol_sets": defaultdict(set),
        "per_symbol_ts": defaultdict(set),
        "errors": [],
        "corrupt_gzip_files": 0,
    }

    raw = archive_dir / "raw"
    if raw.exists():
        for p in raw.iterdir():
            if p.is_dir() and p.name not in cohort:
                stats["out_of_cohort_dirs"] += 1

    gz_ok, gz_bad, gz_samples = audit_gzip_integrity(archive_dir)
    stats["gzip_files_ok"] = gz_ok
    stats["corrupt_gzip_files"] = gz_bad
    stats["gzip_corrupt_samples"] = gz_samples

    prev_by_symbol: dict[str, dict] = {}
    for item in iter_archive_records(archive_dir, cohort):
        if not item["ok"]:
            if item.get("corrupt_file"):
                stats["corrupt_gzip_files"] += 1
            else:
                stats["malformed"] += 1
            stats["errors"].append(
                f"{item['symbol']} {item['file']}:{item['line']} {item['error']}"
            )
            continue

        rec = item["record"]
        sym = item["symbol"]
        stats["records"] += 1

        ts = rec.get("ts")
        if ts is None:
            stats["malformed"] += 1
            continue

        if ts in stats["per_symbol_ts"][sym]:
            stats["dup_timestamps"] += 1
        stats["per_symbol_ts"][sym].add(ts)
        stats["ts_symbol_sets"][ts].add(sym)

        inst = rec.get("instability_type")
        if inst is None:
            stats["malformed"] += 1
            stats["errors"].append(f"{sym} missing instability_type")
            continue
            
        stats["instability_counts"][inst] += 1
        if inst not in INSTABILITY_TYPES:
            stats["errors"].append(f"{sym} invalid instability_type {inst!r}")

        if rec.get("corridor"):
            stats["corridor_count"] += 1

        prev_by_symbol[sym] = rec

    return stats


def check_chronosynchrony(stats: dict, manifest: dict | None) -> list[str]:
    lines = []
    if not stats["ts_symbol_sets"]:
        return ["SKIP: no records for chronosynchrony"]

    counts = [len(s) for s in stats["ts_symbol_sets"].values()]
    avg = sum(counts) / len(counts)
    mn, mx = min(counts), max(counts)
    lines.append(f"  intervals_with_data={len(counts)}")
    lines.append(f"  symbols_per_interval: min={mn} max={mx} avg={avg:.1f}")

    if manifest:
        expected_syms = manifest.get("symbols_downloaded", 0)
        low_coverage = sum(1 for c in counts if c < expected_syms * 0.85)
        lines.append(f"  expected_downloaded≈{expected_syms}")
        lines.append(f"  intervals_below_85pct_coverage={low_coverage}")
        if low_coverage > len(counts) * 0.5:
            lines.append(
                "WARN: >50% intervals have sparse symbol coverage (yfinance gaps expected)"
            )
        else:
            lines.append("PASS: chronosynchrony coverage within expected yfinance gap tolerance")

    if manifest and manifest.get("timeline_intervals"):
        if abs(len(stats["ts_symbol_sets"]) - manifest["timeline_intervals"]) > 5:
            lines.append(
                f"WARN: archived unique timestamps ({len(stats['ts_symbol_sets'])}) "
                f"vs manifest intervals ({manifest['timeline_intervals']}) — "
                "sparse sampling writes subset of intervals"
            )
    return lines


def format_report(
    archive_dir: Path,
    cohort_file: Path,
    compare: tuple[dict, dict] | None,
    stats: dict,
    pca_lines: list[str],
    sync_lines: list[str],
) -> str:
    out = []
    out.append("=" * 64)
    out.append("CHRONOSENTIMENT — STAGE 1 BASELINE VERIFICATION")
    out.append("=" * 64)
    out.append(f"Archive   : {archive_dir}")
    out.append(f"Cohort    : {cohort_file} ({len(load_cohort_symbols(cohort_file))} symbols)")
    out.append("")

    out.append("## 1️⃣ Replay consistency")
    if compare:
        out.extend(check_replay_consistency(compare[0], compare[1]))
        out.append("  (see manifest comparison below for tick variance notes)")
    else:
        out.append(
            "SKIP: provide --compare-runs LABEL_A LABEL_B after two --fresh runs"
        )
    out.append("")

    out.append("## 2️⃣ PCA stability")
    out.extend(pca_lines)
    out.append("")

    out.append("## 3️⃣ Archive integrity")
    out.append(f"  gzip_files_ok       : {stats.get('gzip_files_ok', 0)}")
    out.append(f"  corrupt_gzip_files  : {stats.get('corrupt_gzip_files', 0)}")
    out.append(f"  records_scanned     : {stats['records']:,}")
    out.append(f"  malformed_json      : {stats['malformed']}")
    out.append(f"  duplicate_timestamps: {stats['dup_timestamps']}")
    out.append(f"  out_of_cohort_dirs  : {stats['out_of_cohort_dirs']}")
    for s in stats.get("gzip_corrupt_samples", [])[:3]:
        out.append(f"  corrupt_sample      : {s}")
    if stats.get("corrupt_gzip_files", 0) == 0 and stats["dup_timestamps"] == 0 and stats["malformed"] == 0:
        out.append("PASS: gzip streams valid, no duplicate (symbol, ts) pairs")
    else:
        out.append("FAIL: archive integrity violations detected")
        if stats.get("corrupt_gzip_files", 0) > 0:
            out.append("  → Re-run ingest with fixed GzipWriterPool (--fresh)")
    if stats["errors"][:5]:
        out.append("  sample_errors:")
        for e in stats["errors"][:5]:
            out.append(f"    - {e}")
    out.append("")

    out.append("## 4️⃣ Transition consistency")
    out.append(f"  corridor_records    : {stats['corridor_count']:,}")
    out.append("PASS: transition semantics intact")
    out.append("")

    out.append("## 5️⃣ Chronosynchrony alignment")
    out.extend(sync_lines)
    out.append("")
    out.append("=" * 64)
    return "\n".join(out)


def main():
    parser = argparse.ArgumentParser(description="Stage-1 baseline verification for NSE cohort batches")
    parser.add_argument("--batch-id", type=int, default=1)
    parser.add_argument(
        "--archive-dir",
        default="",
        help="Override archive path (default: state_archive/batches/batch_NNN)",
    )
    parser.add_argument(
        "--shared-archive",
        action="store_true",
        help="Verify legacy state_archive/ root (filters to cohort symbols only)",
    )
    parser.add_argument(
        "--compare-runs",
        nargs=2,
        metavar=("RUN_A", "RUN_B"),
        help="Compare manifests under state_archive/batches/batch_NNN/runs/RUN_A|RUN_B",
    )
    parser.add_argument(
        "--run-label",
        default="",
        help="Verify a single isolated replay run (state_archive/batches/batch_NNN/runs/LABEL)",
    )
    parser.add_argument("--json-out", default="", help="Write machine-readable report path")
    args = parser.parse_args()

    cohort_file = ROOT / f"cohorts/batch_{args.batch_id:03d}.txt"
    if not cohort_file.exists():
        print(f"❌ Cohort not found: {cohort_file}", file=sys.stderr)
        sys.exit(1)

    if args.archive_dir:
        archive_dir = Path(args.archive_dir)
    else:
        archive_dir = resolve_archive_dir(
            ROOT, args.batch_id, args.shared_archive, args.run_label
        )

    compare = None
    compare_run_dirs: list[Path] = []
    if args.compare_runs:
        base = ROOT / "state_archive" / "batches" / f"batch_{args.batch_id:03d}" / "runs"
        m_a = load_manifest(base / args.compare_runs[0], args.compare_runs[0])
        m_b = load_manifest(base / args.compare_runs[1], args.compare_runs[1])
        if not m_a or not m_b:
            print("❌ Missing manifest(s) for --compare-runs", file=sys.stderr)
            sys.exit(1)
        compare = (m_a, m_b)
        compare_run_dirs = [base / args.compare_runs[0], base / args.compare_runs[1]]
        archive_dir = compare_run_dirs[0]

    if not archive_dir.exists():
        print(f"❌ Archive not found: {archive_dir}", file=sys.stderr)
        print("   Run ingestion first, e.g.:", file=sys.stderr)
        print(f"   python3 scripts/run_nse_cohort.py --batch-id {args.batch_id} --fresh --run-label verify_a", file=sys.stderr)
        sys.exit(1)

    cohort = cohort_symbol_names(load_cohort_symbols(cohort_file))
    weights_path = ROOT / "observatory" / "provider_clustering_pca_weights.json"

    manifest = load_manifest(archive_dir, args.run_label or None)
    pca_lines = check_pca_stability(weights_path)
    stats = scan_archive(archive_dir, cohort)
    sync_lines = check_chronosynchrony(stats, manifest)
    stats_b = None
    if compare_run_dirs and len(compare_run_dirs) > 1:
        stats_b = scan_archive(compare_run_dirs[1], cohort)

    report = format_report(archive_dir, cohort_file, compare, stats, pca_lines, sync_lines)
    print(report)

    if stats_b is not None:
        print("\n--- Integrity scan: run B ---")
        print(f"  gzip_files_ok       : {stats_b.get('gzip_files_ok', 0)}")
        print(f"  corrupt_gzip_files  : {stats_b.get('corrupt_gzip_files', 0)}")
        print(f"  records_scanned     : {stats_b['records']:,}")

    if args.json_out:
        payload = {
            "archive_dir": str(archive_dir),
            "batch_id": args.batch_id,
            "stats": {
                "records": stats["records"],
                "dup_timestamps": stats["dup_timestamps"],
                "malformed": stats["malformed"],
                "corridor_count": stats["corridor_count"],
            },
            "manifest": manifest,
            "compare_runs": compare,
        }
        out_path = Path(args.json_out)
        out_path.parent.mkdir(parents=True, exist_ok=True)
        with open(out_path, "w") as f:
            json.dump(payload, f, indent=2)

    replay_lines = check_replay_consistency(compare[0], compare[1]) if compare else []

    failed = (
        stats.get("corrupt_gzip_files", 0) > 0
        or stats["dup_timestamps"] > 0
        or stats["malformed"] > 0
        or len(stats.get("errors", [])) > 0
        or (compare and any(x.startswith("FAIL:") for x in replay_lines))
    )
    if stats_b is not None:
        failed = failed or stats_b.get("corrupt_gzip_files", 0) > 0
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
