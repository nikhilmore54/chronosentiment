#!/usr/bin/env python3
"""
REC-001-H Evidence Quality Report

Reads all JSONL files from datasets/recommendation/historical/ and produces
a per-ticker evidence quality table showing:
  - total records
  - COMPLETE records (outcome != INSUFFICIENT_DATA and sessions_available == 10)
  - LONG / SHORT / NO_TRADE split
  - state-bucket counts: Bullish+Positive, Bullish+Negative, Bearish+Positive, Bearish+Negative, Other
  - target rate (TARGET_BEFORE_RISK / COMPLETE)
  - median MFE at session 5 and session 10
  - minimum analogue bucket size (smallest of the 4 state buckets for LONG decisions)

Usage:
    python3 scripts/rec001h_evidence_quality.py [--dir datasets/recommendation/historical]

Output:
    Console table + datasets/recommendation/historical/evidence_quality_report.csv
"""

import json
import os
import sys
import csv
import statistics
from collections import defaultdict
from pathlib import Path


def parse_args():
    import argparse
    p = argparse.ArgumentParser()
    p.add_argument("--dir", default="datasets/recommendation/historical",
                   help="Directory containing JSONL files")
    p.add_argument("--min-complete", type=int, default=15,
                   help="Minimum COMPLETE records to flag as viable for ticker-specific evidence")
    return p.parse_args()


def state_bucket(trend: str, momentum: str) -> str:
    """Map (trend, momentum) to a 4-bucket label.

    Actual Coralys label values (from JSONL inspection):
      trend:    'Bullish' | 'Bearish' | 'absent'
      momentum: 'Positive' | 'Negative'

    Mapping:
      Bullish + Positive  → Bull+Pos
      Bullish + Negative  → Bull+Neg
      Bearish + Positive  → Bear+Pos
      Bearish + Negative  → Bear+Neg
      absent  + *         → Absent+{Pos|Neg}  (trend absent — no directional signal)
    """
    t = trend.strip()
    m = momentum.strip()

    if t == "Bullish":
        trend_label = "Bull"
    elif t == "Bearish":
        trend_label = "Bear"
    else:
        trend_label = "Absent"  # 'absent' or any unknown value

    if m == "Positive":
        mom_label = "Pos"
    elif m == "Negative":
        mom_label = "Neg"
    else:
        mom_label = "Neutral"

    return f"{trend_label}+{mom_label}"


def analyse_ticker(records: list) -> dict:
    total = len(records)
    complete = [r for r in records if r.get("sessions_available", 0) >= 10
                and r.get("outcome") not in ("INSUFFICIENT_DATA", "NO_GEOMETRY")]
    n_complete = len(complete)

    n_long = sum(1 for r in records if r.get("direction") == "LONG")
    n_short = sum(1 for r in records if r.get("direction") == "SHORT")
    n_notrade = sum(1 for r in records if r.get("direction") == "NO_TRADE")

    # Actual C3-002 state→direction mapping (verified from JSONL data):
    #   Bullish+Positive → LONG   (trend-following)
    #   Bullish+Negative → SHORT  (momentum divergence from bullish trend)
    #   Bearish+Positive → LONG   (counter-trend momentum bounce)
    #   Bearish+Negative → LONG   (counter-trend oversold bounce)
    #   absent+Positive  → LONG   (momentum only, no trend signal)
    #   absent+Negative  → SHORT  (momentum only, no trend signal)
    #
    # So LONG states: Bull+Pos, Bear+Pos, Bear+Neg, Absent+Pos
    # SHORT states:   Bull+Neg, Absent+Neg
    #
    # Track by direction (not by assumed state→direction mapping).
    long_complete = [r for r in complete if r.get("direction") == "LONG"]
    short_complete = [r for r in complete if r.get("direction") == "SHORT"]

    # Count by state bucket within each direction
    long_bucket_counts = defaultdict(int)
    long_bucket_target = defaultdict(list)
    for r in long_complete:
        b = state_bucket(r.get("trend", ""), r.get("momentum", ""))
        long_bucket_counts[b] += 1
        long_bucket_target[b].append(1 if r.get("outcome") == "TARGET_BEFORE_RISK" else 0)

    short_bucket_counts = defaultdict(int)
    short_bucket_target = defaultdict(list)
    for r in short_complete:
        b = state_bucket(r.get("trend", ""), r.get("momentum", ""))
        short_bucket_counts[b] += 1
        short_bucket_target[b].append(1 if r.get("outcome") == "TARGET_BEFORE_RISK" else 0)

    # Overall target rate (LONG complete only — LONG is the primary recommendation direction)
    n_target = sum(1 for r in long_complete if r.get("outcome") == "TARGET_BEFORE_RISK")
    target_rate = n_target / len(long_complete) if long_complete else None

    # SHORT target rate
    n_short_target = sum(1 for r in short_complete if r.get("outcome") == "TARGET_BEFORE_RISK")
    short_target_rate = n_short_target / len(short_complete) if short_complete else None

    # Median MFE at session 5 and 10 (LONG complete)
    mfe5 = [r["mfe_pct"][4] for r in long_complete if len(r.get("mfe_pct", [])) >= 5]
    mfe10 = [r["mfe_pct"][9] for r in long_complete if len(r.get("mfe_pct", [])) >= 10]
    median_mfe5 = statistics.median(mfe5) if mfe5 else None
    median_mfe10 = statistics.median(mfe10) if mfe10 else None

    # LONG state buckets (all 4 states that produce LONG)
    long_state_buckets = ["Bull+Pos", "Bear+Pos", "Bear+Neg", "Absent+Pos"]
    min_long_bucket = min(long_bucket_counts.get(b, 0) for b in ["Bull+Pos", "Bear+Pos", "Bear+Neg"])

    # SHORT state buckets (states that produce SHORT)
    short_state_buckets = ["Bull+Neg", "Absent+Neg"]
    min_short_bucket = long_bucket_counts.get("Bull+Neg", 0)  # only one main SHORT state

    # Per-bucket target rates
    def bucket_rate(counts_dict, target_dict, b):
        vals = target_dict.get(b, [])
        return sum(vals) / len(vals) if vals else None

    return {
        "total": total,
        "complete": n_complete,
        "long": n_long,
        "short": n_short,
        "no_trade": n_notrade,
        "long_complete": len(long_complete),
        "short_complete": len(short_complete),
        # LONG state buckets
        "bull_pos": long_bucket_counts.get("Bull+Pos", 0),
        "bear_pos": long_bucket_counts.get("Bear+Pos", 0),
        "bear_neg_long": long_bucket_counts.get("Bear+Neg", 0),
        "absent_long": long_bucket_counts.get("Absent+Pos", 0),
        "min_long_bucket": min_long_bucket,
        # SHORT state buckets
        "bull_neg": short_bucket_counts.get("Bull+Neg", 0),
        "absent_short": short_bucket_counts.get("Absent+Neg", 0),
        "min_short_bucket": short_bucket_counts.get("Bull+Neg", 0),
        # Rates
        "target_rate": target_rate,
        "short_target_rate": short_target_rate,
        "median_mfe5": median_mfe5,
        "median_mfe10": median_mfe10,
        "bull_pos_rate": bucket_rate(long_bucket_counts, long_bucket_target, "Bull+Pos"),
        "bear_pos_rate": bucket_rate(long_bucket_counts, long_bucket_target, "Bear+Pos"),
        "bear_neg_long_rate": bucket_rate(long_bucket_counts, long_bucket_target, "Bear+Neg"),
        "bull_neg_rate": bucket_rate(short_bucket_counts, short_bucket_target, "Bull+Neg"),
    }


def fmt_pct(v):
    if v is None:
        return "—"
    return f"{v*100:.1f}%"


def fmt_f(v, decimals=2):
    if v is None:
        return "—"
    return f"{v:.{decimals}f}"


def main():
    args = parse_args()
    hist_dir = Path(args.dir)
    if not hist_dir.exists():
        print(f"ERROR: {hist_dir} does not exist", file=sys.stderr)
        sys.exit(1)

    jsonl_files = sorted(hist_dir.glob("*.jsonl"))
    if not jsonl_files:
        print(f"ERROR: no .jsonl files in {hist_dir}", file=sys.stderr)
        sys.exit(1)

    print(f"Reading {len(jsonl_files)} JSONL files from {hist_dir}...")

    rows = []
    for path in jsonl_files:
        ticker = path.stem.replace("_NS", ".NS").replace("_BSE", ".BSE")
        records = []
        with open(path) as f:
            for line in f:
                line = line.strip()
                if line:
                    try:
                        records.append(json.loads(line))
                    except json.JSONDecodeError:
                        pass
        if not records:
            continue
        stats = analyse_ticker(records)
        stats["ticker"] = ticker
        rows.append(stats)

    # Initial sort by ticker name; sections will re-sort as needed
    rows.sort(key=lambda r: r["ticker"])

    # Print summary table
    print()
    print("=" * 140)
    print("REC-001-H Evidence Quality Report")
    print("=" * 140)
    # Sort by ticker name
    rows.sort(key=lambda r: r["ticker"])

    # LONG section
    # Actual LONG-producing states: Bull+Pos, Bear+Pos, Bear+Neg (counter-trend), Absent+Pos
    print()
    print("── LONG decisions (Bull+Pos, Bear+Pos, Bear+Neg states) ──")
    header = (
        f"{'Ticker':<20} {'Total':>6} {'Compl':>6} {'Long':>5} "
        f"{'B+P':>5} {'Be+P':>5} {'Be+N':>5} {'AbsL':>4} "
        f"{'MinL':>6} {'TgtRate':>8} {'MFE5':>6} {'MFE10':>6} "
        f"{'B+P%':>7} {'Be+P%':>7} {'Be+N%':>7}"
    )
    print(header)
    print("-" * 110)

    viable = 0
    sparse = 0
    for r in rows:
        flag = ""
        if r["min_long_bucket"] < args.min_complete:
            flag = " ⚠"
            sparse += 1
        else:
            viable += 1
        line = (
            f"{r['ticker']:<20} {r['total']:>6} {r['complete']:>6} {r['long']:>5} "
            f"{r['bull_pos']:>5} {r['bear_pos']:>5} {r['bear_neg_long']:>5} {r['absent_long']:>4} "
            f"{r['min_long_bucket']:>6} {fmt_pct(r['target_rate']):>8} "
            f"{fmt_f(r['median_mfe5']):>6} {fmt_f(r['median_mfe10']):>6} "
            f"{fmt_pct(r['bull_pos_rate']):>7} {fmt_pct(r['bear_pos_rate']):>7} "
            f"{fmt_pct(r['bear_neg_long_rate']):>7}{flag}"
        )
        print(line)

    print("-" * 110)
    print(f"Total tickers: {len(rows)} | Viable LONG (min_long_bucket ≥ {args.min_complete}): {viable} | Sparse (⚠): {sparse}")

    # SHORT section
    # Actual SHORT-producing state: Bull+Neg only (Bullish trend + Negative momentum)
    print()
    print("── SHORT decisions (Bull+Neg state only) ──")
    header2 = (
        f"{'Ticker':<20} {'Short':>5} {'B+N':>5} {'AbsS':>5} {'MinS':>6} {'B+N%':>7}"
    )
    print(header2)
    print("-" * 60)
    rows_by_short = sorted(rows, key=lambda r: -r["min_short_bucket"])
    viable_s = 0
    sparse_s = 0
    for r in rows_by_short:
        flag = ""
        if r["min_short_bucket"] < args.min_complete:
            flag = " ⚠"
            sparse_s += 1
        else:
            viable_s += 1
        line2 = (
            f"{r['ticker']:<20} {r['short']:>5} {r['bull_neg']:>5} {r['absent_short']:>5} "
            f"{r['min_short_bucket']:>6} {fmt_pct(r['bull_neg_rate']):>7}{flag}"
        )
        print(line2)
    print("-" * 60)
    print(f"Total tickers: {len(rows)} | Viable SHORT (min_short_bucket ≥ {args.min_complete}): {viable_s} | Sparse (⚠): {sparse_s}")
    print()

    # Aggregate stats
    all_target_rates = [r["target_rate"] for r in rows if r["target_rate"] is not None]
    all_min_long = [r["min_long_bucket"] for r in rows]
    all_min_short = [r["min_short_bucket"] for r in rows]
    all_totals = [r["total"] for r in rows]
    print()
    print(f"Aggregate LONG target rate (mean across tickers): {sum(all_target_rates)/len(all_target_rates)*100:.1f}%")
    print(f"Min LONG bucket — median: {statistics.median(all_min_long):.0f}, min: {min(all_min_long)}, max: {max(all_min_long)}")
    print(f"Min SHORT bucket — median: {statistics.median(all_min_short):.0f}, min: {min(all_min_short)}, max: {max(all_min_short)}")
    print(f"Total records — median: {statistics.median(all_totals):.0f}, min: {min(all_totals)}, max: {max(all_totals)}")
    print(f"Total records across all tickers: {sum(all_totals):,}")
    print()

    # Write CSV
    csv_path = hist_dir / "evidence_quality_report.csv"
    fieldnames = [
        "ticker", "total", "complete", "long", "short", "no_trade",
        "long_complete", "short_complete",
        # LONG state buckets (all produce LONG in C3-002)
        "bull_pos", "bear_pos", "bear_neg_long", "absent_long", "min_long_bucket",
        # SHORT state buckets (Bull+Neg only in C3-002)
        "bull_neg", "absent_short", "min_short_bucket",
        # Rates
        "target_rate", "short_target_rate", "median_mfe5", "median_mfe10",
        "bull_pos_rate", "bear_pos_rate", "bear_neg_long_rate", "bull_neg_rate",
    ]
    with open(csv_path, "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames, extrasaction="ignore")
        writer.writeheader()
        writer.writerows(rows)
    print(f"CSV written: {csv_path}")


if __name__ == "__main__":
    main()