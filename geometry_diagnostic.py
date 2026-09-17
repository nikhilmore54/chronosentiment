#!/usr/bin/env python3
"""geometry_diagnostic.py

Generalized diagnostic for any date range (e.g., 2026‑09‑07 to 2026‑09‑11).
It reconstructs each HORIZON trade from the paper‑trader ledger, joins the
minute‑level intraday snapshots from `datasets/p4_intraday_flat.csv`, and
computes detailed geometry metrics.

All timestamps are interpreted in the Asia/Kolkata (IST) timezone as required
by the dataset.

Usage:
    python3 geometry_diagnostic.py --start 20260907 --end 20260911

This produces `geometry_diagnostic_20260907_20260911.md` with daily sections
and a summary table matching the specification.
"""

import argparse
import csv
import os
from collections import defaultdict
from datetime import datetime, timezone
import math

try:
    from zoneinfo import ZoneInfo  # Python 3.9+
except ImportError:
    from backports.zoneinfo import ZoneInfo  # type: ignore

IST = ZoneInfo("Asia/Kolkata")
ROOT = os.path.abspath(os.path.dirname(__file__))
DATASET_DIR = os.path.join(ROOT, "datasets")
LEDGER_GLOB = "paper_trader_v2_*.csv"
INTRADAY_FILE = os.path.join(DATASET_DIR, "p4_intraday_flat.csv")

def parse_float(v: str) -> float:
    try:
        return float(v)
    except Exception:
        return math.nan

def load_ledger(start_date: str, end_date: str):
    """Load ledger rows with exit_reason == HORIZON whose date falls within the range.
    Dates are strings YYYYMMDD.
    Returns list of dicts.
    """
    rows = []
    for fname in os.listdir(DATASET_DIR):
        if not fname.startswith('paper_trader_v2_') or not fname.endswith('.csv'):
            continue
        path = os.path.join(DATASET_DIR, fname)
        with open(path, newline='') as f:
            reader = csv.DictReader(f)
            for r in reader:
                if r.get('exit_reason') != 'HORIZON':
                    continue
                d = r['date']
                if d < start_date or d > end_date:
                    continue
                rows.append({
                    'ticker': r['ticker'].replace('_NS', ''),
                    'direction': r['direction'],
                    'date': d,
                    'entry_price': parse_float(r['entry_price']),
                    'target_price': parse_float(r['adaptive_target']),
                    'stop_price': parse_float(r['candidate_stop_price']),
                    'exit_price': parse_float(r['exit_price']),
                    'realized_return': parse_float(r['realized_return']),
                    'exit_time_ist': r['exit_time_ist'],
                })
    return rows

def load_intraday():
    """Load intraday snapshots indexed by (date, ticker, decision_id)."""
    index = defaultdict(list)
    with open(INTRADAY_FILE, newline='') as f:
        reader = csv.DictReader(f)
        for r in reader:
            cohort_date = r['cohort_date'].replace('-', '')  # YYYYMMDD
            ticker = r['ticker'].replace('_NS', '')
            decision_id = r['decision_id']
            try:
                snap_unix = int(r['source_snapshot_unix'])
                snap_dt = datetime.fromtimestamp(snap_unix, tz=timezone.utc).astimezone(IST)
            except Exception:
                continue
            index[(cohort_date, ticker, decision_id)].append({
                'snapshot_dt': snap_dt,
                'entry_price': parse_float(r.get('entry_price', '')),
                'exit_price': parse_float(r.get('exit_price', '')),
            })
    for key in index:
        index[key].sort(key=lambda x: x['snapshot_dt'])
    return index

def match_intraday(ledger_row, intraday_index):
    """Find intraday series that matches the ledger row.
    Matching is done by date and ticker; then we pick the series whose entry_price
    is within 0.5% of the ledger entry_price (to avoid ambiguous decision_id).
    Returns list of snapshots (dicts) or empty list.
    """
    candidates = []
    for (d, t, decision_id), series in intraday_index.items():
        if d != ledger_row['date'] or t != ledger_row['ticker']:
            continue
        if series:
            entry_price_series = series[0]['entry_price']
            if math.isnan(entry_price_series) or math.isnan(ledger_row['entry_price']):
                continue
            rel_diff = abs(entry_price_series - ledger_row['entry_price']) / ledger_row['entry_price']
            if rel_diff <= 0.005:
                candidates.append(series)
    if not candidates:
        return []
    return max(candidates, key=len)

def compute_metrics(row, series):
    """Given ledger row and intraday series, compute geometry metrics.
    Returns dict with required fields.
    """
    direction = row['direction']
    entry = row['entry_price']
    target = row['target_price']
    stop = row['stop_price']
    horizon_price = row['exit_price']
    prices = [snap['exit_price'] for snap in series if not math.isnan(snap['exit_price'])]
    timestamps = [snap['snapshot_dt'] for snap in series if not math.isnan(snap['exit_price'])]
    if not prices:
        return {}
    diffs = [p - entry if direction == 'LONG' else entry - p for p in prices]
    mfe = max(diffs)
    mae = min(diffs)
    target_dist_pct = (target - entry) / entry if direction == 'LONG' else (entry - target) / entry
    stop_dist_pct = (entry - stop) / entry if direction == 'LONG' else (stop - entry) / entry
    target_crossed = any((p >= target if direction == 'LONG' else p <= target) for p in prices)
    stop_crossed = any((p <= stop if direction == 'LONG' else p >= stop) for p in prices)
    time_to_target = None
    time_to_stop = None
    for p, ts in zip(prices, timestamps):
        if time_to_target is None and ((p >= target and direction == 'LONG') or (p <= target and direction == 'SHORT')):
            time_to_target = (ts - timestamps[0]).total_seconds() / 60.0
        if time_to_stop is None and ((p <= stop and direction == 'LONG') or (p >= stop and direction == 'SHORT')):
            time_to_stop = (ts - timestamps[0]).total_seconds() / 60.0
        if time_to_target is not None and time_to_stop is not None:
            break
    def progress(p, target_price, stop_price):
        full = target_price - stop_price if direction == 'LONG' else stop_price - target_price
        if full == 0:
            return 0.0
        return (p - stop_price) / full if direction == 'LONG' else (stop_price - p) / full
    max_target_progress = max(progress(p, target, stop) for p in prices)
    max_stop_progress = max(progress(p, target, stop) for p in prices)
    return {
        'ticker': row['ticker'],
        'direction': direction,
        'date': row['date'],
        'entry_price': entry,
        'target_price': target,
        'stop_price': stop,
        'horizon_price': horizon_price,
        'realized_return': row['realized_return'],
        'mfe': mfe,
        'mae': mae,
        'target_dist_pct': target_dist_pct,
        'stop_dist_pct': stop_dist_pct,
        'target_crossed': target_crossed,
        'stop_crossed': stop_crossed,
        'time_to_target': time_to_target,
        'time_to_stop': time_to_stop,
        'max_target_progress': max_target_progress,
        'max_stop_progress': max_stop_progress,
        'outcome': 'HORIZON',
    }

def aggregate_daily(metrics_by_day):
    days = sorted(metrics_by_day.keys())
    header = ["Metric"] + days
    lines = []
    lines.append("| " + " | ".join(header) + " |")
    lines.append("|" + "---|" * (len(header)))
    def count_cond(day, cond):
        return sum(1 for m in metrics_by_day[day] if cond(m))
    def avg(vals):
        return sum(vals) / len(vals) if vals else 0.0
    rows = [
        ("ACT predictions", lambda d: len(metrics_by_day[d])),
        ("NOT_ACT", lambda d: 0),
        ("Paper entries", lambda d: len(metrics_by_day[d])),
        ("TARGET", lambda d: avg([m['target_dist_pct'] for m in metrics_by_day[d]])),
        ("STOP", lambda d: avg([m['stop_dist_pct'] for m in metrics_by_day[d]])),
        ("HORIZON", lambda d: avg([m['realized_return'] for m in metrics_by_day[d]])),
        ("Win rate", lambda d: count_cond(d, lambda m: m['realized_return'] > 0) / max(len(metrics_by_day[d]), 1)),
        ("Mean realized return", lambda d: avg([m['realized_return'] for m in metrics_by_day[d]])),
        ("Median realized return", lambda d: (sorted([m['realized_return'] for m in metrics_by_day[d]])[len(metrics_by_day[d])//2] if metrics_by_day[d] else 0)),
        ("LONG return", lambda d: avg([m['realized_return'] for m in metrics_by_day[d] if m['direction'] == 'LONG'])),
        ("SHORT return", lambda d: avg([m['realized_return'] for m in metrics_by_day[d] if m['direction'] == 'SHORT'])),
    ]
    for metric, fn in rows:
        cols = []
        for day in days:
            try:
                val = fn(day)
                if isinstance(val, float):
                    cols.append(f"{val:.4f}")
                else:
                    cols.append(str(val))
            except Exception:
                cols.append("-")
        lines.append("| " + metric + " | " + " | ".join(cols) + " |")
    return "\n".join(lines)

def generate_report(all_metrics, start, end):
    by_day = defaultdict(list)
    for m in all_metrics:
        by_day[m['date']].append(m)
    md = []
    md.append(f"# Geometry Diagnostic {start} – {end}\n")
    md.append("## Daily Summary Table")
    md.append(aggregate_daily(by_day))
    md.append("\n## Per‑Trade Details")
    header = ["Date", "Ticker", "Dir", "Entry", "Target", "Stop", "Horizon", "Ret", "MFE", "MAE", "Target%", "Stop%", "TgtCross", "StopCross", "TgtTime(min)", "StopTime(min)"]
    md.append("| " + " | ".join(header) + " |")
    md.append("|" + "---|" * len(header))
    for m in all_metrics:
        row = [
            m['date'],
            m['ticker'],
            m['direction'],
            f"{m['entry_price']:.4f}",
            f"{m['target_price']:.4f}",
            f"{m['stop_price']:.4f}",
            f"{m['horizon_price']:.4f}",
            f"{m['realized_return']:.6f}",
            f"{m['mfe']:.6f}",
            f"{m['mae']:.6f}",
            f"{m['target_dist_pct']:.2%}",
            f"{m['stop_dist_pct']:.2%}",
            str(m['target_crossed']),
            str(m['stop_crossed']),
            f"{m['time_to_target']:.1f}" if m['time_to_target'] is not None else "-",
            f"{m['time_to_stop']:.1f}" if m['time_to_stop'] is not None else "-",
        ]
        md.append("| " + " | ".join(row) + " |")
    return "\n".join(md)

def main():
    parser = argparse.ArgumentParser(description="Geometry diagnostic over a date range.")
    parser.add_argument("--start", required=True, help="Start date YYYYMMDD")
    parser.add_argument("--end", required=True, help="End date YYYYMMDD")
    args = parser.parse_args()
    ledger = load_ledger(args.start, args.end)
    intraday_index = load_intraday()
    all_metrics = []
    missing = 0
    for row in ledger:
        series = match_intraday(row, intraday_index)
        if not series:
            missing += 1
            continue
        metrics = compute_metrics(row, series)
        if metrics:
            all_metrics.append(metrics)
    report_md = generate_report(all_metrics, args.start, args.end)
    out_path = os.path.join(ROOT, f"geometry_diagnostic_{args.start}_{args.end}.md")
    with open(out_path, "w") as f:
        f.write(report_md)
    print(f"Report written to {out_path}\nMissing matches: {missing}\nTotal HORIZON rows discovered: {len(ledger)}")

if __name__ == "__main__":
    main()
