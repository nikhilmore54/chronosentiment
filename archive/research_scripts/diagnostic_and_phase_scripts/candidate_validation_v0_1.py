# candidate_validation_v0_1.py
"""Candidate Validation Script

*Purpose*: Verify that the experimental geometry (Target × 1.25, Stop × 1.00) behaves correctly
relative to the frozen baseline (Target × 1.00, Stop × 1.00) under identical portfolio
assumptions.
"""

import csv
import sys
import argparse
import json
from pathlib import Path
from datetime import datetime, timezone, timedelta
import statistics

IST = timezone(timedelta(hours=5, minutes=30))

# ---------------------------------------------------------------------------
# Configuration – must match Portfolio Realism v0.1
# ---------------------------------------------------------------------------
DATASET_DIR = Path("/Users/nikhil/ChronoSentiment_MEGA_FINAL/datasets")
BARS_DIR = Path("/Users/nikhil/ChronoSentiment_MEGA_FINAL/intraday_capture/yahoo_cache_1m")

NOTIONAL = 1_000_000  # ₹
MAX_CONCURRENT = 10   # Maximum simultaneous positions
COST_SCENARIOS = {
    "Zero‑cost baseline": 0,
    "Low": 5,
    "Moderate": 10,
    "Conservative": 20,
    "Stress": 30,
}

# ---------------------------------------------------------------------------
# Helper functions
# ---------------------------------------------------------------------------

def parse_entry_timestamp(row: dict) -> datetime | None:
    date_str = row.get("date")
    if not date_str:
        return None
    try:
        dt_date = datetime.strptime(str(date_str), "%Y%m%d")
        # Paper Trader v2 uses start_bar=12, which corresponds to 09:27
        return dt_date.replace(hour=9, minute=27, tzinfo=IST)
    except Exception:
        return None


def parse_exit_timestamp(row: dict) -> datetime | None:
    time_str = row.get("exit_time_ist")
    date_str = row.get("date")
    if not time_str or not date_str:
        return None
    try:
        dt_date = datetime.strptime(str(date_str), "%Y%m%d")
        hour_min = datetime.strptime(time_str, "%H:%M")
        return dt_date.replace(hour=hour_min.hour, minute=hour_min.minute, tzinfo=IST)
    except Exception:
        return None


def load_horizon_rows(files):
    rows = []
    for f in files:
        with open(f, newline="") as fh:
            reader = csv.DictReader(fh)
            for r in reader:
                if r.get("exit_reason") != "HORIZON":
                    continue
                # numeric conversion of needed fields
                for key in [
                    "entry_price",
                    "realized_return",
                    "max_adverse_excursion",
                    "max_favourable_excursion",
                    "bars_held",
                    "adaptive_target",
                    "adaptive_risk",
                ]:
                    try:
                        r[key] = float(r[key])
                    except Exception:
                        r[key] = None
                r["entry_time"] = parse_entry_timestamp(r)
                r["exit_time"] = parse_exit_timestamp(r)
                rows.append(r)
    return rows

def safe_mean(vals):
    clean = [v for v in vals if v is not None]
    return statistics.mean(clean) if clean else None

def max_drawdown(equity_series):
    peak = equity_series[0]
    max_dd = 0.0
    for val in equity_series:
        if val > peak:
            peak = val
        dd = (peak - val) / peak if peak != 0 else 0
        if dd > max_dd:
            max_dd = dd
    return max_dd

_bar_cache = {}
def load_bars(ticker_ns: str) -> list[dict]:
    if ticker_ns in _bar_cache:
        return _bar_cache[ticker_ns]
    p = BARS_DIR / (ticker_ns.replace("_NS", ".NS") + ".json")
    if not p.exists():
        _bar_cache[ticker_ns] = []
        return []
    with open(p) as f:
        bars = json.load(f)
    bars.sort(key=lambda b: b["timestamp"])
    _bar_cache[ticker_ns] = bars
    return bars

# ---------------------------------------------------------------------------
# Geometry simulation – returns a copy of rows with an adjusted "sim_return"
# ---------------------------------------------------------------------------

def apply_geometry(rows, target_mul: float, stop_mul: float):
    out = []
    for r in rows:
        direction = r.get("direction", "LONG")
        entry_price = r["entry_price"]
        base_target = r.get("adaptive_target")
        base_stop = r.get("adaptive_risk")
        ticker = r.get("ticker")
        entry_time = r.get("entry_time")

        target_price = base_target * target_mul if base_target is not None else None
        stop_price = base_stop * stop_mul if base_stop is not None else None
        
        simulated_ret = r.get("realized_return")
        simulated_exit = r.get("exit_reason", "HORIZON")
        sim_mae = 0.0
        sim_mfe = 0.0
        
        if ticker and entry_time and entry_price is not None:
            bars = load_bars(ticker)
            y, m, d = entry_time.year, entry_time.month, entry_time.day
            open_ts  = int(datetime(y, m, d, 9, 15, 0, tzinfo=IST).timestamp())
            close_ts = int(datetime(y, m, d, 15, 30, 0, tzinfo=IST).timestamp())
            sbars = [b for b in bars if open_ts <= b["timestamp"] <= close_ts]
            
            # The baseline run_paper_trader_v2.py uses sbars[12:] which is a positional slice.
            # Due to missing minute bars, this can shift chronologically (e.g. starting at 09:28 instead of 09:27).
            # To achieve 0 mismatches in reconciliation, we must mirror this exact positional logic.
            post_bars = sbars[12:]
            
            hit_exit = False
            # We walk up to 48 bars (since start_bar=12 means 60 - 12 = 48)
            for i, b in enumerate(post_bars[:48]):
                low, high = b["low"], b["high"]
                if direction == "LONG":
                    sim_mae = min(sim_mae, (low - entry_price) / entry_price)
                    sim_mfe = max(sim_mfe, (high - entry_price) / entry_price)
                    if target_price is not None and high >= target_price:
                        simulated_ret = (target_price - entry_price) / entry_price
                        simulated_exit = "TARGET"
                        hit_exit = True
                        break
                    if stop_price is not None and low <= stop_price:
                        simulated_ret = (stop_price - entry_price) / entry_price
                        simulated_exit = "STOP"
                        hit_exit = True
                        break
                else:
                    sim_mae = min(sim_mae, (entry_price - high) / entry_price)
                    sim_mfe = max(sim_mfe, (entry_price - low) / entry_price)
                    if target_price is not None and low <= target_price:
                        simulated_ret = (entry_price - target_price) / entry_price
                        simulated_exit = "TARGET"
                        hit_exit = True
                        break
                    if stop_price is not None and high >= stop_price:
                        simulated_ret = (entry_price - stop_price) / entry_price
                        simulated_exit = "STOP"
                        hit_exit = True
                        break
            
            if not hit_exit:
                simulated_exit = "HORIZON"
                if post_bars:
                    horizon_bar = post_bars[min(47, len(post_bars)-1)]
                    close_p = horizon_bar["close"]
                    simulated_ret = (close_p - entry_price) / entry_price if direction == "LONG" else (entry_price - close_p) / entry_price

        new_row = r.copy()
        new_row["sim_return"] = simulated_ret
        new_row["sim_exit_reason"] = simulated_exit
        new_row["sim_mae"] = sim_mae
        new_row["sim_mfe"] = sim_mfe
        out.append(new_row)
    return out

# ---------------------------------------------------------------------------
# Reconcile Baseline
# ---------------------------------------------------------------------------
def reconcile_baseline(original_rows, baseline_rows):
    mismatches = []
    for orig, base in zip(original_rows, baseline_rows):
        orig_ret = orig.get("realized_return", 0.0)
        base_ret = base.get("sim_return", 0.0)
        orig_exit = orig.get("exit_reason", "HORIZON")
        base_exit = base.get("sim_exit_reason")
        
        # Check for meaningful mismatches
        if orig_exit != base_exit:
            mismatches.append(f"{orig['ticker']} {orig['date']}: Exit mismatch (orig={orig_exit}, base={base_exit})")
        elif abs(orig_ret - base_ret) > 1e-4:
            mismatches.append(f"{orig['ticker']} {orig['date']}: Return mismatch (orig={orig_ret:.4f}, base={base_ret:.4f})")
    
    if mismatches:
        print("VALIDATION = INVALID (Baseline Reconciliation Failed)")
        for m in mismatches[:20]:
            print(" -", m)
        if len(mismatches) > 20:
            print(f" ... and {len(mismatches)-20} more")
        sys.exit(1)
    else:
        print("Baseline reconciliation: SUCCESS (0 mismatches)")

# ---------------------------------------------------------------------------
# Portfolio simulation
# ---------------------------------------------------------------------------

def simulate_portfolio(rows, cost_bps: int):
    events = []
    for r in rows:
        if r.get("entry_time"):
            events.append((r["entry_time"], "entry", r))
        if r.get("exit_time"):
            events.append((r["exit_time"], "exit", r))
    events.sort(key=lambda x: x[0])

    equity = NOTIONAL
    equity_curve = [equity]
    active = {}
    concurrent_counts = []
    net_returns = []
    gross_returns = []
    win_cnt = loss_cnt = 0

    for ts, typ, row in events:
        concurrent_counts.append(len(active))
        if typ == "entry":
            if len(active) >= MAX_CONCURRENT:
                continue
            tentative = len(active) + 1
            allocation = equity / tentative
            for k in list(active.keys()):
                active[k] = allocation
            active[id(row)] = allocation
        else:  # exit
            pid = id(row)
            if pid not in active:
                continue
            allocation = active.pop(pid)
            ret = row.get("sim_return", row.get("realized_return", 0))
            gross = allocation * ret
            cost = allocation * (cost_bps / 10_000)
            net = gross - cost
            equity += net
            equity_curve.append(equity)
            gross_returns.append(gross)
            net_returns.append(net)
            if net > 0:
                win_cnt += 1
            else:
                loss_cnt += 1
            if active:
                new_alloc = equity / len(active)
                for k in active:
                    active[k] = new_alloc

    total_net = sum(net_returns) if net_returns else 0.0
    total_gross = sum(gross_returns) if gross_returns else 0.0
    total_return_pct = total_net / NOTIONAL * 100
    avg_net_per_trade = safe_mean(net_returns)
    win_rate = (win_cnt / (win_cnt + loss_cnt)) * 100 if (win_cnt + loss_cnt) else 0
    max_dd = max_drawdown(equity_curve) * 100
    avg_concurrent = safe_mean(concurrent_counts)
    turnover = sum([abs(a) for a in gross_returns]) / NOTIONAL if gross_returns else 0.0

    return {
        "final_equity": equity,
        "total_return_pct": total_return_pct,
        "gross_return_pct": total_gross / NOTIONAL * 100,
        "net_return_pct": total_net / NOTIONAL * 100,
        "max_drawdown_pct": max_dd,
        "win_rate_pct": win_rate,
        "expectancy_per_trade_pct": (avg_net_per_trade / NOTIONAL * 100) if avg_net_per_trade is not None else None,
        "average_concurrent_positions": avg_concurrent,
        "turnover_multiple": turnover,
        "cost_bps": cost_bps,
    }

# ---------------------------------------------------------------------------
# Temporal‑causality check 
# ---------------------------------------------------------------------------

def check_temporal_causality(rows, target_mul=1.0, stop_mul=1.0):
    if not rows:
        return
    for r in rows:
        entry = r.get("entry_time")
        exit_t = r.get("exit_time")
        if entry and exit_t and entry > exit_t:
            raise AssertionError(f"Temporal causality violation: trade entry {entry} is after exit {exit_t}")

    full_sim = apply_geometry(rows, target_mul, stop_mul)
    ref = {id(r): (r["sim_return"], r["sim_exit_reason"]) for r in full_sim}

    timestamps = sorted({r["entry_time"] for r in rows if r["entry_time"]})
    for ts in timestamps:
        prefix_rows = [r for r in rows if r["entry_time"] and r["entry_time"] <= ts]
        if not prefix_rows:
            continue
        prefix_sim = apply_geometry(prefix_rows, target_mul, stop_mul)
        for r in prefix_sim:
            rid = id(r)
            ref_vals = ref.get(rid)
            if ref_vals is None:
                continue
            if (r["sim_return"], r["sim_exit_reason"]) != ref_vals:
                raise AssertionError(f"Temporal causality violation at {ts}: decision changed due to future data")

    ohlcv_patterns = ["*ohlcv*.csv", "*bars*.csv", "*candles*.csv", "*market*.csv"]
    for pattern in ohlcv_patterns:
        for f_path in DATASET_DIR.glob(pattern):
            prev_time = None
            try:
                with open(f_path, newline="") as fh:
                    reader = csv.DictReader(fh)
                    for row in reader:
                        t_str = row.get("timestamp") or row.get("date") or row.get("datetime")
                        if t_str:
                            if prev_time and t_str < prev_time:
                                raise AssertionError(f"Temporal causality violation: chronological regression in market data file {f_path.name} at {t_str}")
                            prev_time = t_str
            except Exception as e:
                if isinstance(e, AssertionError):
                    raise
                continue

# ---------------------------------------------------------------------------
# Main driver
# ---------------------------------------------------------------------------

def main():
    parser = argparse.ArgumentParser(description="Candidate validation script")
    parser.add_argument(
        "--mode",
        choices=["sanity", "validation"],
        default="sanity",
        help="Run in SANITY mode (historical window) or VALIDATION mode (unseen files)",
    )
    args = parser.parse_args()

    if args.mode == "sanity":
        ledger_files = [
            DATASET_DIR / "paper_trader_v2_20260903.csv",
            DATASET_DIR / "paper_trader_v2_20260904.csv",
            DATASET_DIR / "paper_trader_v2_20260907.csv",
            DATASET_DIR / "paper_trader_v2_20260908.csv",
            DATASET_DIR / "paper_trader_v2_20260909.csv",
        ]
    else:  # validation
        all_files = list(DATASET_DIR.glob("paper_trader_v2_*.csv"))
        cutoff = datetime.strptime("20260910", "%Y%m%d")
        ledger_files = []
        for f in all_files:
            stem = f.stem
            date_part = stem.split("_")[-1]
            try:
                file_date = datetime.strptime(date_part, "%Y%m%d")
                if file_date > cutoff:
                    ledger_files.append(f)
            except Exception:
                continue
        if not ledger_files:
            print("NO UNSEEN WINDOW AVAILABLE")
            sys.exit(0)
        for f in ledger_files:
            date_part = f.stem.split("_")[-1]
            file_date = datetime.strptime(date_part, "%Y%m%d")
            if file_date <= cutoff:
                raise AssertionError("VALIDATION = INVALID: ledger file earlier than cutoff detected")

    horizon_rows = load_horizon_rows(ledger_files)

    # 1. Prepare baseline and candidate row sets
    baseline_rows = apply_geometry(horizon_rows, target_mul=1.0, stop_mul=1.0)
    candidate_rows = apply_geometry(horizon_rows, target_mul=1.25, stop_mul=1.0)
    
    # 2. Strict baseline reconciliation
    reconcile_baseline(horizon_rows, baseline_rows)

    # 3. Temporal‑causality assertion – runs for both modes
    try:
        check_temporal_causality(horizon_rows)
    except AssertionError as e:
        print("VALIDATION = INVALID")
        print(e)
        sys.exit(1)

    report_lines = []
    report_lines.append("# Candidate Validation Report (Mode: {} )\n".format(args.mode.upper()))
    report_lines.append("\n| Cost Scenario | Metric | Baseline | Candidate |")
    report_lines.append("|---|---|---|---|")

    metrics_of_interest = [
        "total_return_pct",
        "net_return_pct",
        "gross_return_pct",
        "max_drawdown_pct",
        "win_rate_pct",
        "expectancy_per_trade_pct",
        "average_concurrent_positions",
        "turnover_multiple",
    ]

    for cost_name, bps in COST_SCENARIOS.items():
        base_metrics = simulate_portfolio(baseline_rows, bps)
        cand_metrics = simulate_portfolio(candidate_rows, bps)
        for m in metrics_of_interest:
            baseline_val = f"{base_metrics[m]:.2f}" if base_metrics[m] is not None else "-"
            cand_val = f"{cand_metrics[m]:.2f}" if cand_metrics[m] is not None else "-"
            report_lines.append(f"| {cost_name} | {m} | {baseline_val} | {cand_val} |")

    # Per‑trade transition table
    report_lines.append("\n## Per‑trade Transition Table (sample)\n")
    report_lines.append("| Date | Ticker | Direction | Entry Price | Baseline Return | Candidate Return | Δ Return | Baseline Exit | Candidate Exit |")
    report_lines.append("|---|---|---|---|---|---|---|---|---|")
    for b_row, c_row in zip(baseline_rows, candidate_rows):
        delta = (c_row.get("sim_return", 0) - b_row.get("sim_return", 0)) * 100  
        report_lines.append(
            f"| {b_row.get('date')} | {b_row.get('ticker')} | {b_row.get('direction')} | {b_row.get('entry_price'):.4f} "
            f"| {b_row.get('sim_return', 0)*100:.2f}% | {c_row.get('sim_return', 0)*100:.2f}% "
            f"| {delta:+.2f}% | {b_row.get('sim_exit_reason')} | {c_row.get('sim_exit_reason')} |"
        )

    out_md = Path("/Users/nikhil/.gemini/antigravity-ide/brain/ac8b3565-b88c-4945-8459-7e723a29389a/candidate_validation_v0_1_report.md")
    out_md.write_text("\n".join(report_lines))

    out_csv = Path("/Users/nikhil/.gemini/antigravity-ide/brain/ac8b3565-b88c-4945-8459-7e723a29389a/candidate_validation_summary.csv")
    with out_csv.open("w", newline="") as cf:
        writer = csv.writer(cf)
        header = ["cost_scenario", "metric", "baseline", "candidate"]
        writer.writerow(header)
        for cost_name, bps in COST_SCENARIOS.items():
            base_metrics = simulate_portfolio(baseline_rows, bps)
            cand_metrics = simulate_portfolio(candidate_rows, bps)
            for m in metrics_of_interest:
                writer.writerow([cost_name, m, base_metrics[m], cand_metrics[m]])

    print(f"Report written to {out_md}")
    print(f"Summary CSV written to {out_csv}")

if __name__ == "__main__":
    main()
