#!/usr/bin/env python3
"""
Replay equivalence — compare live-evolved archive vs deterministic frozen replay.

Measurement only; does not change manifold mechanics.

Usage:
  python3 scripts/compare_replay_equivalence.py --batch-id 3 \\
    --live-label live --replay-label replay_equiv

  python3 scripts/compare_replay_equivalence.py --batch-id 900 \\
    --live-label lse --replay-label replay_equiv \\
    --ts-min 1779184500 --ts-max 1779187800
"""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from collections import defaultdict
from pathlib import Path


def calculate_substrate_hash(archive_dir: Path, cohort: set[str], ts_list: list[int]) -> str:
    """Calculate a unique, deterministic SHA-256 hash of all L1 barriers in the active window."""
    sha = hashlib.sha256()
    sorted_symbols = sorted(list(cohort))
    found_count = 0
    for ts in ts_list:
        for symbol in sorted_symbols:
            path = archive_dir / "raw" / symbol / "barriers" / f"{ts}.json"
            if path.exists():
                sha.update(f"{symbol}:{ts}:".encode("utf-8"))
                sha.update(path.read_bytes())
                found_count += 1
    if found_count == 0:
        return "none"
    return sha.hexdigest()


ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))

from archive_dedupe import iter_gzip_jsonl
from candle_substrate import frozen_batch_dir
from run_nse_cohort import resolve_archive_dir
from verify_cohort_baseline import (
    cohort_symbol_names,
    iter_archive_records,
    load_cohort_symbols,
    load_manifest,
)


def per_ts_aggregate(archive_dir: Path, cohort: set[str], ts_filter: set[int] | None) -> dict[int, dict]:
    """Per global timestamp: symbol count, tick count, corridor count."""
    out: dict[int, dict] = defaultdict(lambda: {"symbols": set(), "ticks": 0, "corridors": 0})
    for item in iter_archive_records(archive_dir, cohort):
        if not item["ok"]:
            continue
        rec = item["record"]
        ts = rec.get("ts")
        if ts is None:
            continue
        ts = int(ts)
        if ts_filter is not None and ts not in ts_filter:
            continue
        row = out[ts]
        row["symbols"].add(item["symbol"])
        row["ticks"] += 1
        if rec.get("corridor"):
            row["corridors"] += 1
    # normalize sets to counts
    return {
        ts: {
            "symbol_count": len(v["symbols"]),
            "ticks": v["ticks"],
            "corridors": v["corridors"],
            "corridor_rate": round(v["corridors"] / max(v["ticks"], 1), 4),
        }
        for ts, v in out.items()
    }


def load_live_ts_window(steps_path: Path) -> tuple[set[int], list[dict]]:
    if not steps_path.exists():
        return set(), []
    steps = [json.loads(l) for l in steps_path.read_text().splitlines() if l.strip()]
    committed = [
        s
        for s in steps
        if s.get("barrier_committed", True) and (s.get("ts") is not None)
    ]
    ts_set = {int(s["ts"]) for s in committed}
    return ts_set, committed


def main() -> int:
    parser = argparse.ArgumentParser(description="Compare live vs deterministic replay equivalence")
    parser.add_argument("--batch-id", type=int, required=True)
    parser.add_argument("--live-label", default="live", help="Run label for live-evolved archive")
    parser.add_argument("--replay-label", default="replay_equiv", help="Run label for --from-frozen replay")
    parser.add_argument("--ts-min", type=int, default=0, help="Optional lower ts bound (unix)")
    parser.add_argument("--ts-max", type=int, default=0, help="Optional upper ts bound (unix)")
    parser.add_argument("--tick-tolerance", type=float, default=0.02, help="Relative tick count tolerance")
    parser.add_argument("--corridor-rate-tolerance", type=float, default=0.05, help="Abs corridor-rate delta")
    parser.add_argument("--symbol-tolerance", type=int, default=0, help="Max symbol-count delta per ts")
    args = parser.parse_args()

    cohort_file = ROOT / "cohorts" / f"batch_{args.batch_id:03d}.txt"
    if not cohort_file.exists():
        print(f"❌ {cohort_file} not found", file=sys.stderr)
        return 1

    cohort = cohort_symbol_names(load_cohort_symbols(cohort_file))
    live_dir = resolve_archive_dir(args.batch_id, False, args.live_label)
    replay_dir = resolve_archive_dir(args.batch_id, False, args.replay_label)
    frozen_manifest_path = frozen_batch_dir(args.batch_id) / "manifest.json"

    steps_path = live_dir / "metadata" / "live_session_steps.jsonl"
    live_ts, live_steps = load_live_ts_window(steps_path)

    if args.ts_min or args.ts_max:
        lo, hi = args.ts_min or 0, args.ts_max or 2**63 - 1
        live_ts = {t for t in live_ts if lo <= t <= hi}

    if not live_ts and steps_path.exists():
        print("⚠️  No committed barriers in live steps; comparing full archive overlap", file=sys.stderr)

    m_live = load_manifest(live_dir, args.live_label)
    m_replay = load_manifest(replay_dir, args.replay_label)
    frozen = json.loads(frozen_manifest_path.read_text()) if frozen_manifest_path.exists() else {}

    print("=" * 72)
    print("REPLAY EQUIVALENCE — live vs deterministic replay")
    print("=" * 72)
    print(f"  Batch           : {args.batch_id:03d}")
    print(f"  Live archive    : {live_dir}")
    print(f"  Replay archive  : {replay_dir}")
    print(f"  Frozen substrate: {frozen_manifest_path}")
    if frozen:
        print(f"  Frozen fingerprint : {frozen.get('timeline_fingerprint')}")
        print(f"  Frozen hash        : {frozen.get('substrate_hash')}")
    print(f"  Aligned ts window : {len(live_ts)} barrier(s)")
    if live_ts:
        print(f"    range {min(live_ts)} → {max(live_ts)}")
    print("=" * 72)

    issues: list[str] = []
    passes: list[str] = []

    # Manifest-level (full replay run)
    if m_live and m_replay:
        fp_l, fp_r = m_live.get("timeline_fingerprint"), m_replay.get("timeline_fingerprint")
        if fp_l and fp_r and fp_l != fp_r:
            issues.append(f"MANIFEST fingerprint mismatch: live={fp_l!r} replay={fp_r!r}")
        elif fp_l and fp_r:
            passes.append(f"MANIFEST timeline_fingerprint match ({fp_l})")

        cr_l = float(m_live.get("corridor_rate", 0))
        cr_r = float(m_replay.get("corridor_rate", 0))
        if abs(cr_l - cr_r) > args.corridor_rate_tolerance:
            issues.append(f"MANIFEST corridor_rate drift: live={cr_l:.4f} replay={cr_r:.4f}")
        else:
            passes.append(f"MANIFEST corridor_rate close ({cr_l:.4f} vs {cr_r:.4f})")
    elif not m_replay:
        issues.append(f"Missing replay manifest at {replay_dir}/manifests/")
    elif not m_live:
        print("  (live run has no full ingestion manifest — incremental only; using per-ts scan)")

    ts_filter = live_ts if live_ts else None
    live_agg = per_ts_aggregate(live_dir, cohort, ts_filter)
    replay_agg = per_ts_aggregate(replay_dir, cohort, ts_filter)

    compare_ts = sorted(live_ts if live_ts else set(live_agg) & set(replay_agg))
    if not compare_ts:
        print("❌ No overlapping timestamps to compare", file=sys.stderr)
        return 1

    # ── Phase 6: Cryptographic Substrate Hash Certification ──
    live_hash = calculate_substrate_hash(live_dir, cohort, compare_ts)
    replay_hash = calculate_substrate_hash(replay_dir, cohort, compare_ts)

    if live_hash != "none" and replay_hash != "none":
        if live_hash != replay_hash:
            issues.append(f"SUBSTRATE CRYPTOGRAPHIC HASH MISMATCH: live={live_hash} replay={replay_hash}")
        else:
            passes.append(f"SUBSTRATE CRYPTOGRAPHIC HASH MATCH ({live_hash})")
    elif live_hash == "none" and replay_hash == "none":
        print("  (no Layer 1 barrier files found in either archive; skipping cryptographic hash comparison)")
    else:
        issues.append(f"SUBSTRATE CRYPTOGRAPHIC HASH INCOMPLETE: live={live_hash} replay={replay_hash}")

    ts_tick_mismatch = 0
    ts_corr_mismatch = 0
    ts_sym_mismatch = 0
    ts_missing_live = 0
    ts_missing_replay = 0

    for ts in compare_ts:
        lv, rv = live_agg.get(ts), replay_agg.get(ts)
        if lv is None:
            ts_missing_live += 1
            issues.append(f"TS {ts}: missing in live archive")
            continue
        if rv is None:
            ts_missing_replay += 1
            issues.append(f"TS {ts}: missing in replay archive")
            continue

        sym_d = abs(lv["symbol_count"] - rv["symbol_count"])
        if sym_d > args.symbol_tolerance:
            ts_sym_mismatch += 1
            issues.append(
                f"TS {ts}: symbol_count live={lv['symbol_count']} replay={rv['symbol_count']}"
            )

        rel = abs(lv["ticks"] - rv["ticks"]) / max(lv["ticks"], rv["ticks"], 1)
        if rel > args.tick_tolerance:
            ts_tick_mismatch += 1
            issues.append(
                f"TS {ts}: tick_count live={lv['ticks']} replay={rv['ticks']} (Δ={rel:.1%})"
            )

        cr_d = abs(lv["corridor_rate"] - rv["corridor_rate"])
        if cr_d > args.corridor_rate_tolerance:
            ts_corr_mismatch += 1
            issues.append(
                f"TS {ts}: corridor_rate live={lv['corridor_rate']:.4f} "
                f"replay={rv['corridor_rate']:.4f} (Δ={cr_d:.4f})"
            )

    if ts_tick_mismatch == 0 and compare_ts:
        passes.append(f"PER-TS tick counts within {args.tick_tolerance:.0%} on {len(compare_ts)} barrier(s)")
    if ts_corr_mismatch == 0 and compare_ts:
        passes.append(
            f"PER-TS corridor_rate within {args.corridor_rate_tolerance} on {len(compare_ts)} barrier(s)"
        )
    if ts_sym_mismatch == 0 and compare_ts:
        passes.append(f"PER-TS symbol coverage matched on {len(compare_ts)} barrier(s)")

    # Live steps quorum vs archive
    if live_steps:
        print("\nLive barrier participation (from steps log):")
        for s in sorted(live_steps, key=lambda x: x.get("ts", 0)):
            ts = s.get("ts")
            agg = live_agg.get(ts, {})
            print(
                f"  ts={ts} steps={s.get('participating_symbols', s.get('symbols'))}/"
                f"{s.get('expected_symbols', '?')} "
                f"archive_ticks={agg.get('ticks', 0)} "
                f"corridor_rate={agg.get('corridor_rate', 0):.3f}"
            )

    # 1. Aggregate chronology and TRL confidence metrics across the window
    chronology_confidences = []
    feed_fragmentations = []
    provider_consensuses = []
    cohort_len = len(cohort) if cohort else 1
    
    for s in live_steps:
        # Strict extraction: The live engine is the sole authority for these fields.
        # We no longer silently reconstruct them from raw fetch stats if missing.
        bar_conf = s.get("barrier_confidence", 0.0)
        frag_ratio = s.get("fragmentation_ratio", 0.0)
        prov_cons = s.get("provider_consensus", 1.0)
            
        chronology_confidences.append(bar_conf)
        feed_fragmentations.append(frag_ratio)
        provider_consensuses.append(prov_cons)
        
    mean_chronology_confidence = round(sum(chronology_confidences) / len(chronology_confidences), 4) if chronology_confidences else 1.0
    mean_feed_fragmentation = round(sum(feed_fragmentations) / len(feed_fragmentations), 4) if feed_fragmentations else 0.0
    mean_provider_consensus = round(sum(provider_consensuses) / len(provider_consensuses), 4) if provider_consensuses else 1.0

    print("\n--- Results ---")
    for p in passes:
        print(f"  ✅ {p}")
    for issue in issues[:30]:
        print(f"  ❌ {issue}")
    if len(issues) > 30:
        print(f"  ... and {len(issues) - 30} more")

    # Display TRL Confidence Tagging summary
    print(f"\n  🛡️  TEMPORAL RELIABILITY REPLAY AUDIT:")
    print(f"     Replay Equivalence    : {'PASS' if len(issues) == 0 else 'FAIL'}")
    print(f"     Chronology Confidence : {mean_chronology_confidence:.2%}")
    print(f"     Feed Fragmentation    : {mean_feed_fragmentation:.2%}")
    print(f"     Provider Consensus    : {mean_provider_consensus:.2%}")
    print(f"     Substrate Hash Cert   : {live_hash if (live_hash == replay_hash and live_hash != 'none') else 'MISMATCH' if (live_hash != 'none' or replay_hash != 'none') else 'N/A'}")

    summary = {
        "batch_id": args.batch_id,
        "live_label": args.live_label,
        "replay_label": args.replay_label,
        "barriers_compared": len(compare_ts),
        "ts_missing_live": ts_missing_live,
        "ts_missing_replay": ts_missing_replay,
        "ts_tick_mismatch": ts_tick_mismatch,
        "ts_corridor_mismatch": ts_corr_mismatch,
        "ts_symbol_mismatch": ts_sym_mismatch,
        "pass": len(issues) == 0,
        "frozen_fingerprint": frozen.get("timeline_fingerprint"),
        # Replay Confidence Tagging
        "replay_equivalence": len(issues) == 0,
        "chronology_confidence": mean_chronology_confidence,
        "feed_fragmentation": mean_feed_fragmentation,
        "provider_consensus": mean_provider_consensus,
        # Cryptographic Substrate Hash Certification
        "substrate_hash_certified": live_hash == replay_hash if live_hash != "none" else False,
        "live_substrate_hash": live_hash,
        "replay_substrate_hash": replay_hash,
    }
    out_path = live_dir / "metadata" / "replay_equivalence_report.json"
    out_path.parent.mkdir(parents=True, exist_ok=True)
    with open(out_path, "w") as f:
        json.dump(summary, f, indent=2)
    print(f"\n  Report: {out_path}")

    if ts_missing_replay and ts_missing_replay >= len(compare_ts) // 2:
        print(
            "\n  ℹ️  Most live barriers missing in replay — re-freeze substrate after the soak "
            "(freeze must include the same bars as the live session), then re-run --from-frozen."
        )

    if issues:
        print("\nFAIL: replay equivalence — see divergences above")
        return 1
    print("\nPASS: causal chronological signatures converge on aligned ts window")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
