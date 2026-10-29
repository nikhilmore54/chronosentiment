#!/usr/bin/env python3
"""
Read-only experiment leaderboard from append-only JSONL registry.

Aligned with .cursor/rules/chronosentiment-core.mdc:
- deterministic ranking
- no recomputation of metrics
- no mutation of core decision artifacts
"""

from __future__ import annotations

import argparse
import json
from datetime import datetime
from pathlib import Path
from typing import Any


def load_registry(path: Path) -> list[dict[str, Any]]:
    records: list[dict[str, Any]] = []
    if not path.exists():
        return records

    with path.open("r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                records.append(json.loads(line))
            except json.JSONDecodeError:
                # Keep reader resilient to malformed lines in append-only logs.
                continue

    return records


def parse_ts(ts: str) -> datetime | None:
    try:
        return datetime.fromisoformat(ts.replace("Z", "+00:00"))
    except Exception:
        return None


def latest_per_hypothesis(records: list[dict[str, Any]]) -> list[dict[str, Any]]:
    latest: dict[str, dict[str, Any]] = {}

    for r in records:
        hypothesis_id = r.get("hypothesis_id")
        if not hypothesis_id:
            continue
        ts_raw = r.get("timestamp")
        if not isinstance(ts_raw, str) or not ts_raw:
            continue
        ts = parse_ts(ts_raw)
        if ts is None:
            continue

        prev = latest.get(hypothesis_id)
        if prev is None or ts > prev["_ts"]:
            row = dict(r)
            row["_ts"] = ts
            latest[hypothesis_id] = row

    return list(latest.values())


def rank_records(records: list[dict[str, Any]]) -> list[dict[str, Any]]:
    def sort_key(r: dict[str, Any]) -> tuple[float, float, float]:
        summary = r.get("batch_summary", {})
        avg_delta_avg_pnl = float(summary.get("avg_delta_avg_pnl", 0.0))
        avg_delta_hit_rate = float(summary.get("avg_delta_hit_rate", 0.0))
        avg_delta_max_dd = float(summary.get("avg_delta_max_dd", 0.0))
        # Primary: avg pnl uplift (higher better)
        # Tie-break 1: hit-rate uplift (higher better)
        # Tie-break 2: max-dd delta (lower better)
        return (avg_delta_avg_pnl, avg_delta_hit_rate, -avg_delta_max_dd)

    return sorted(records, key=sort_key, reverse=True)


def apply_filters(records: list[dict[str, Any]], args: argparse.Namespace) -> list[dict[str, Any]]:
    filtered: list[dict[str, Any]] = []

    for r in records:
        summary = r.get("batch_summary", {})

        if args.decision and r.get("decision") != args.decision:
            continue
        if args.state is not None:
            record_state = r.get("state", "active")
            if record_state != args.state:
                continue
        if args.min_retained_pct is not None:
            try:
                retained = float(summary.get("avg_retained_pct", 0.0))
            except (TypeError, ValueError):
                retained = 0.0
            if retained < args.min_retained_pct:
                continue
        if args.min_positive_ratio is not None:
            try:
                positive_ratio = float(summary.get("positive_ratio", 0.0))
            except (TypeError, ValueError):
                positive_ratio = 0.0
            if positive_ratio < args.min_positive_ratio:
                continue

        filtered.append(r)

    return filtered


def format_float(x: Any) -> str:
    if x is None:
        return "-"
    try:
        return f"{float(x):+.6f}"
    except (TypeError, ValueError):
        return "-"


def print_table(records: list[dict[str, Any]], top_n: int | None) -> None:
    rows = records[:top_n] if top_n else records
    headers = [
        "Rank",
        "Hypothesis",
        "AvgPnLΔ",
        "HitΔ",
        "DDΔ",
        "Ret%",
        "Decision",
        "Confidence",
    ]

    print("-" * 95)
    print(
        f"{headers[0]:<5} | {headers[1]:<20} | {headers[2]:<10} | "
        f"{headers[3]:<10} | {headers[4]:<10} | {headers[5]:<7} | "
        f"{headers[6]:<9} | {headers[7]:<10}"
    )
    print("-" * 95)

    for i, r in enumerate(rows, start=1):
        summary = r.get("batch_summary", {})
        try:
            retained = float(summary.get("avg_retained_pct", 0.0))
        except (TypeError, ValueError):
            retained = 0.0

        print(
            f"{i:<5} | "
            f"{str(r.get('hypothesis_id', '-')):<20} | "
            f"{format_float(summary.get('avg_delta_avg_pnl')):<10} | "
            f"{format_float(summary.get('avg_delta_hit_rate')):<10} | "
            f"{format_float(summary.get('avg_delta_max_dd')):<10} | "
            f"{retained:<7.2f} | "
            f"{str(r.get('decision', '-')):<9} | "
            f"{str(r.get('confidence', '-')):<10}"
        )

    print("-" * 95)


def main() -> int:
    parser = argparse.ArgumentParser(description="Show experiment leaderboard")
    parser.add_argument(
        "--registry-path",
        default="data/experiments.jsonl",
        help="path to registry JSONL",
    )
    parser.add_argument(
        "--top",
        type=int,
        default=None,
        help="show top N ranked hypotheses",
    )
    parser.add_argument(
        "--decision",
        choices=["PROMOTE", "HOLD", "REJECT"],
        default=None,
        help="optional decision filter",
    )
    parser.add_argument(
        "--min-retained-pct",
        type=float,
        default=None,
        help="optional minimum avg_retained_pct filter",
    )
    parser.add_argument(
        "--min-positive-ratio",
        type=float,
        default=None,
        help="optional minimum positive_ratio filter",
    )
    parser.add_argument(
        "--state",
        choices=["active", "validated", "archived"],
        default=None,
        help="optional lifecycle state filter (missing treated as active)",
    )
    args = parser.parse_args()

    records = load_registry(Path(args.registry_path))
    if not records:
        print("No records found.")
        return 0

    latest = latest_per_hypothesis(records)
    ranked = rank_records(latest)
    filtered = apply_filters(ranked, args)
    print_table(filtered, top_n=args.top)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
