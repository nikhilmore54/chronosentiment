#!/usr/bin/env python3
"""
Batch aggregator for deterministic per-slice decisions.

Read-only orchestration layer aligned with .cursor/rules/chronosentiment-core.mdc:
- does not mutate datasets
- does not recompute slice metrics
- delegates per-slice metrics/decision to trigger_latency_bucket_report.py --json
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

DEFAULT_HYPOTHESIS_ID_PATTERN = r"^[a-z0-9_]+$"


def run_single(
    script_path: Path,
    log_path: Path,
    mom_low_pct: float,
    min_joined: int,
    min_retained_pct: float,
    max_retained_pct: float,
    hypothesis_id: str | None,
    hypothesis_id_pattern: str | None,
    require_hypothesis_id: bool,
) -> dict[str, Any]:
    cmd = [
        sys.executable,
        str(script_path),
        "--auto-decision",
        "--json",
        "--mom-low-pct",
        str(mom_low_pct),
        "--decision-min-joined",
        str(min_joined),
        "--decision-min-retained-pct",
        str(min_retained_pct),
        "--decision-max-retained-pct",
        str(max_retained_pct),
    ]
    if hypothesis_id is not None:
        cmd.extend(["--hypothesis-id", hypothesis_id])
    if hypothesis_id_pattern is not None:
        cmd.extend(["--hypothesis-id-pattern", hypothesis_id_pattern])
    if require_hypothesis_id:
        cmd.append("--require-hypothesis-id")
    with log_path.open("r", encoding="utf-8") as f:
        proc = subprocess.run(
            cmd,
            stdin=f,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            check=False,
        )
    if proc.returncode != 0:
        raise RuntimeError(
            f"failed on {log_path}: returncode={proc.returncode}\n{proc.stderr.strip()}"
        )
    try:
        payload = json.loads(proc.stdout)
    except json.JSONDecodeError as exc:
        raise RuntimeError(f"invalid JSON from {log_path}: {exc}") from exc
    payload["source"] = str(log_path)
    return payload


def avg(vals: list[float]) -> float:
    return sum(vals) / len(vals) if vals else 0.0


def validate_hypothesis_id_pattern(
    hypothesis_id: str | None, hypothesis_id_pattern: str | None
) -> None:
    if not hypothesis_id:
        return
    pattern = hypothesis_id_pattern or DEFAULT_HYPOTHESIS_ID_PATTERN
    if not re.fullmatch(pattern, hypothesis_id):
        raise ValueError(
            f"hypothesis_id '{hypothesis_id}' does not match pattern '{pattern}'"
        )


def aggregate(results: list[dict[str, Any]]) -> dict[str, Any]:
    usable: list[dict[str, Any]] = []
    for r in results:
        baseline_count = int(r.get("metrics", {}).get("baseline", {}).get("count", 0))
        if baseline_count > 0:
            usable.append(r)
    if not usable:
        return {
            "total_slices": len(results),
            "usable_slices": 0,
            "positive_slices": 0,
            "positive_ratio": 0.0,
            "avg_delta_avg_pnl": 0.0,
            "avg_delta_hit_rate": 0.0,
            "avg_delta_max_dd": 0.0,
            "avg_retained_pct": 0.0,
        }

    positive = 0
    d_avg: list[float] = []
    d_hit: list[float] = []
    d_dd: list[float] = []
    retained: list[float] = []

    for r in usable:
        delta = r["metrics"]["cf2_vs_baseline"]
        if delta["delta_sum_pnl"] > 0.0 and delta["delta_max_dd"] <= 0.0:
            positive += 1
        d_avg.append(delta["delta_avg_pnl"])
        d_hit.append(delta["delta_hit_rate"])
        d_dd.append(delta["delta_max_dd"])
        retained.append(r["metrics"]["cf2"]["retained_pct"])

    usable_count = len(usable)
    return {
        "total_slices": len(results),
        "usable_slices": usable_count,
        "positive_slices": positive,
        "positive_ratio": (positive / usable_count) if usable_count else 0.0,
        "avg_delta_avg_pnl": avg(d_avg),
        "avg_delta_hit_rate": avg(d_hit),
        "avg_delta_max_dd": avg(d_dd),
        "avg_retained_pct": avg(retained),
    }


def batch_decision(
    batch_summary: dict[str, Any],
    min_positive_ratio: float,
) -> dict[str, Any]:
    usable = int(batch_summary["usable_slices"])
    if usable == 0:
        return {
            "decision": "REJECT",
            "confidence": "LOW",
            "reasons": ["no_usable_slices"],
        }
    reasons: list[str] = []
    if batch_summary["positive_ratio"] < min_positive_ratio:
        reasons.append("positive_ratio_below_threshold")
    if batch_summary["avg_delta_avg_pnl"] <= 0.0:
        reasons.append("avg_delta_avg_pnl_non_positive")
    if batch_summary["avg_delta_max_dd"] >= 0.0:
        reasons.append("avg_delta_max_dd_not_improved")

    if not reasons:
        return {"decision": "PROMOTE", "confidence": "HIGH", "reasons": []}
    if batch_summary["avg_delta_avg_pnl"] < 0.0:
        return {"decision": "REJECT", "confidence": "MEDIUM", "reasons": reasons}
    return {"decision": "HOLD", "confidence": "LOW", "reasons": reasons}


def append_registry_record(
    registry_path: Path,
    hypothesis_id: str | None,
    state: str,
    batch_summary: dict[str, Any],
    batch_decision_result: dict[str, Any],
) -> None:
    registry_path.parent.mkdir(parents=True, exist_ok=True)
    record = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "hypothesis_id": hypothesis_id,
        "state": state,
        "decision": batch_decision_result["decision"],
        "confidence": batch_decision_result["confidence"],
        "batch_summary": batch_summary,
    }
    with registry_path.open("a", encoding="utf-8") as f:
        f.write(json.dumps(record, ensure_ascii=True) + "\n")


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--script",
        default="scripts/trigger_latency_bucket_report.py",
        help="path to per-slice report script",
    )
    ap.add_argument(
        "--mom-low-pct",
        type=float,
        default=15.0,
        help="momentum low percentile passed through to per-slice script",
    )
    ap.add_argument(
        "--decision-min-joined",
        type=int,
        default=3,
        help="minimum baseline joined trades for per-slice PROMOTE",
    )
    ap.add_argument(
        "--decision-min-retained-pct",
        type=float,
        default=50.0,
        help="minimum retained percent for per-slice PROMOTE",
    )
    ap.add_argument(
        "--decision-max-retained-pct",
        type=float,
        default=95.0,
        help="maximum retained percent for per-slice PROMOTE",
    )
    ap.add_argument(
        "--batch-min-positive-ratio",
        type=float,
        default=0.75,
        help="minimum positive slice ratio for batch PROMOTE",
    )
    ap.add_argument(
        "--hypothesis-id",
        default=None,
        help="optional hypothesis identifier passed to each per-slice run",
    )
    ap.add_argument(
        "--hypothesis-id-pattern",
        default=None,
        help="optional regex that hypothesis_id must fully match",
    )
    ap.add_argument(
        "--allow-missing-hypothesis-id",
        action="store_true",
        help="allow missing hypothesis_id in batch (default is strict contract enforcement)",
    )
    ap.add_argument(
        "--registry-path",
        default="data/experiments.jsonl",
        help="append-only JSONL registry path for batch outcomes",
    )
    ap.add_argument(
        "--state",
        choices=["active", "validated", "archived"],
        default="active",
        help="lifecycle state metadata for appended registry record (default active)",
    )
    ap.add_argument(
        "logs",
        nargs="+",
        help="log files (one slice/run per file)",
    )
    args = ap.parse_args()

    script_path = Path(args.script).resolve()
    require_hypothesis_id = not args.allow_missing_hypothesis_id
    if require_hypothesis_id and not args.hypothesis_id:
        raise ValueError(
            "batch mode requires --hypothesis-id by default "
            "(use --allow-missing-hypothesis-id to opt out)"
        )
    validate_hypothesis_id_pattern(args.hypothesis_id, args.hypothesis_id_pattern)
    results: list[dict[str, Any]] = []
    for log in args.logs:
        log_path = Path(log).resolve()
        results.append(
            run_single(
                script_path=script_path,
                log_path=log_path,
                mom_low_pct=args.mom_low_pct,
                min_joined=args.decision_min_joined,
                min_retained_pct=args.decision_min_retained_pct,
                max_retained_pct=args.decision_max_retained_pct,
                hypothesis_id=args.hypothesis_id,
                hypothesis_id_pattern=args.hypothesis_id_pattern,
                require_hypothesis_id=require_hypothesis_id,
            )
        )

    for r in results:
        validate_hypothesis_id_pattern(
            r.get("hypothesis_id"), args.hypothesis_id_pattern
        )
    hypothesis_ids = {r.get("hypothesis_id") for r in results}
    if require_hypothesis_id and any(not hid for hid in hypothesis_ids):
        raise ValueError("missing hypothesis_id detected in strict batch mode")
    if len(hypothesis_ids) > 1:
        raise ValueError(f"mixed hypothesis_ids in batch: {sorted(hypothesis_ids, key=lambda x: '' if x is None else str(x))}")
    batch_hypothesis_id = next(iter(hypothesis_ids)) if hypothesis_ids else args.hypothesis_id

    summary = aggregate(results)
    decision = batch_decision(summary, args.batch_min_positive_ratio)
    append_registry_record(
        registry_path=Path(args.registry_path).resolve(),
        hypothesis_id=batch_hypothesis_id,
        state=args.state,
        batch_summary=summary,
        batch_decision_result=decision,
    )
    output = {
        "hypothesis_id": batch_hypothesis_id,
        "batch_summary": summary,
        "batch_decision": decision,
        "per_slice": results,
    }
    print(json.dumps(output, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
