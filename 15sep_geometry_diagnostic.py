#!/usr/bin/env python3
"""15sep_geometry_diagnostic.py

Reconstructs all HORIZON trades from the Sep‑2026 paper‑trader ledger and joins them
to the minute‑level intraday observations (p4_intraday_flat.csv).  All timestamps are
interpreted in the Asia/Kolkata (IST) timezone as required by the dataset.

The script outputs a markdown report `15sep_geometry_diagnostic.md` containing:

* Total number of HORIZON rows discovered (should be 54 for the Sep‑15 batch).
* Number of matched intraday series.
* Summary statistics (average MFE, MAE, realized return, etc.).
* A per‑trade table with the most relevant metrics.

The implementation purposefully avoids any changes to source datasets or runtime
logic – it only reads existing CSV files.
"""

import csv
import os
from collections import defaultdict
from datetime import datetime, timezone
try:
    from zoneinfo import ZoneInfo  # Python 3.9+
except ImportError:
    from backports.zoneinfo import ZoneInfo  # type: ignore

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------
ROOT = os.path.abspath(os.path.dirname(__file__))
DATASET_DIR = os.path.join(ROOT, "datasets")
INTRADAY_FILE = os.path.join(DATASET_DIR, "p4_intraday_flat.csv")
OUTPUT_MD = os.path.join(ROOT, "15sep_geometry_diagnostic.md")
IST = ZoneInfo("Asia/Kolkata")

# ---------------------------------------------------------------------------
# Helper functions
# ---------------------------------------------------------------------------

def parse_float(val: str) -> float:
    try:
        return float(val)
    except ValueError:
        return float('nan')

def load_ledger() -> list[dict]:
    """Load all ledger rows with exit_reason == HORIZON.

    Returns a list of dictionaries with the columns we need.
    """
    ledger_rows = []
    for fname in os.listdir(DATASET_DIR):
        if not fname.startswith('paper_trader_v2_') or not fname.endswith('.csv'):
            continue
        path = os.path.join(DATASET_DIR, fname)
        with open(path, newline='') as f:
            reader = csv.DictReader(f)
            for row in reader:
                if row.get('exit_reason') != 'HORIZON':
                    continue
                ledger_rows.append({
                    'ticker': row['ticker'].replace('_NS', ''),
                    'direction': row['direction'],
                    'date': row['date'],
                    'entry_price': parse_float(row['entry_price']),
                    'exit_price': parse_float(row['exit_price']),
                    'realized_return': parse_float(row['realized_return']),
                    'exit_time_ist': row['exit_time_ist'],
                })
    return ledger_rows

def load_intraday() -> dict:
    """Load intraday flat data indexed by (date, ticker, decision_id)."""
    index = defaultdict(list)
    with open(INTRADAY_FILE, newline='') as f:
        reader = csv.DictReader(f)
        for row in reader:
            cohort_date = row['cohort_date'].replace('-', '')  # YYYYMMDD
            ticker = row['ticker'].replace('_NS', '')
            decision_id = row['decision_id']
            try:
                snapshot_unix = int(row['source_snapshot_unix'])
                snapshot_dt = datetime.fromtimestamp(snapshot_unix, tz=timezone.utc).astimezone(IST)
                snapshot_str = snapshot_dt.strftime('%H:%M')
            except Exception:
                snapshot_str = ''
            index[(cohort_date, ticker, decision_id)].append({
                'intraday_return': parse_float(row.get('intraday_return', '')),
                'snapshot': snapshot_str,
                'entry_price': parse_float(row.get('entry_price', '')),
                'exit_price': parse_float(row.get('exit_price', '')),
                'daily_exit_reason': row.get('daily_exit_reason', ''),
            })
    return index

def compute_metrics(ledger_row: dict, intraday_series: list[dict]) -> dict:
    if not intraday_series:
        return {'matched': False}
    entry_price = ledger_row['entry_price']
    returns = [r['intraday_return'] for r in intraday_series if r['intraday_return'] == r['intraday_return']]
    if not returns:
        mfe = mae = 0.0
    else:
        mfe = max(returns)
        mae = min(returns)
    return {
        'matched': True,
        'ticker': ledger_row['ticker'],
        'direction': ledger_row['direction'],
        'date': ledger_row['date'],
        'entry_price': entry_price,
        'exit_price': ledger_row['exit_price'],
        'realized_return': ledger_row['realized_return'],
        'mfe': mfe,
        'mae': mae,
        'mfe_pct': mfe / entry_price if entry_price != 0 else 0,
        'mae_pct': mae / entry_price if entry_price != 0 else 0,
    }

def generate_markdown(all_metrics: list[dict], total_horizon: int, matched: int, mismatches: int) -> str:
    lines = []
    lines.append('# 15‑Sep Geometry Diagnostic')
    lines.append('')
    lines.append(f'**Total HORIZON rows discovered:** {total_horizon}')
    lines.append(f'**Matched intraday series:** {matched}')
    lines.append(f'**Price mismatches / missing series:** {mismatches}')
    lines.append('')
    mfe_vals = [m['mfe'] for m in all_metrics if m.get('matched')]
    mae_vals = [m['mae'] for m in all_metrics if m.get('matched')]
    ret_vals = [m['realized_return'] for m in all_metrics if m.get('matched')]
    def avg(lst):
        return sum(lst) / len(lst) if lst else 0
    lines.append('## Summary Statistics')
    lines.append(f'- Average MFE: {avg(mfe_vals):.6f}')
    lines.append(f'- Average MAE: {avg(mae_vals):.6f}')
    lines.append(f'- Average realized return: {avg(ret_vals):.6f}')
    lines.append('')
    lines.append('## Per‑Trade Metrics')
    header = ['Ticker', 'Direction', 'Date', 'Entry', 'Exit', 'Ret', 'MFE', 'MAE', 'MFE%', 'MAE%']
    lines.append('| ' + ' | '.join(header) + ' |')
    lines.append('|' + '---|' * len(header))
    for m in all_metrics:
        if not m.get('matched'):
            continue
        rows = [
            m['ticker'],
            m['direction'],
            m['date'],
            f"{m['entry_price']:.4f}",
            f"{m['exit_price']:.4f}",
            f"{m['realized_return']:.6f}",
            f"{m['mfe']:.6f}",
            f"{m['mae']:.6f}",
            f"{m['mfe_pct']:.2%}",
            f"{m['mae_pct']:.2%}",
        ]
        lines.append('| ' + ' | '.join(rows) + ' |')
    return '\n'.join(lines)

def main():
    ledger = load_ledger()
    total_horizon = len(ledger)
    intraday_index = load_intraday()
    all_metrics = []
    matched = 0
    mismatches = 0
    for row in ledger:
        matches = []
        for (d_date, d_ticker, _), series in intraday_index.items():
            if d_date == row['date'] and d_ticker == row['ticker']:
                matches.extend(series)
        if not matches:
            mismatches += 1
            all_metrics.append({'matched': False})
            continue
        metrics = compute_metrics(row, matches)
        if metrics.get('matched'):
            matched += 1
        else:
            mismatches += 1
        all_metrics.append(metrics)
    md_content = generate_markdown(all_metrics, total_horizon, matched, mismatches)
    with open(OUTPUT_MD, 'w') as f:
        f.write(md_content)
    print(f'Report written to {OUTPUT_MD}')

if __name__ == "__main__":
    main()
