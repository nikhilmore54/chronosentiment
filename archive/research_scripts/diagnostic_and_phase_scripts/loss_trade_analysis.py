#!/usr/bin/env python3
"""
Loss Trade Analysis — Five-Date v0.2 Replay
=============================================
For every losing trade in the five-date v0.2 paper trader output, reconstructs
the complete decision lifecycle and classifies the loss into one of five categories:

  1. BAD_ENTRY          — adverse from bar 1, never meaningfully positive
  2. THESIS_DETERIORATION — initially positive, then reversed
  3. STOP_PREMATURE     — stop triggered, but position subsequently recovered
  4. MISSED_INVERSION   — original direction lost, opposite direction subsequently won
  5. UNAVOIDABLE        — loss with no actionable correction available

For each losing trade, computes:
  best_exit_time        — bar where exiting would have minimised loss
  best_exit_ret         — return at best exit
  recoverable_pp        — difference between actual and best exit
  invert_opportunity    — whether opposite direction produced >+0.3% after exit
  invert_peak_ret       — best return available in opposite direction after exit

Outputs:
  datasets/loss_analysis_v2.csv   — per-trade loss analysis
  Console: summary table + category breakdown

IC v1, Backtest v2, Cockpit, and all prior frozen components are NOT modified.

Usage:
  python3 scripts/loss_trade_analysis.py
"""

import csv
import json
import statistics
from pathlib import Path
from datetime import datetime, timezone, timedelta
from dataclasses import dataclass, asdict
from typing import Optional

# ── Paths ─────────────────────────────────────────────────────────────────────

REPO_ROOT  = Path(__file__).resolve().parent.parent
BARS_DIR   = REPO_ROOT / "intraday_capture" / "yahoo_cache_1m"
OUT_DIR    = REPO_ROOT / "datasets"
IST        = timezone(timedelta(hours=5, minutes=30))

OVERLAP_DATES = ["20260903", "20260904", "20260907", "20260908", "20260909"]

# Thresholds
INITIAL_POSITIVE_THRESHOLD = 0.001   # >0.1% at any point = "initially positive"
INVERT_OPPORTUNITY_MIN     = 0.003   # >0.3% in opposite direction = invert opportunity
STOP_RECOVERY_MIN          = 0.001   # position recovers >0.1% after stop = premature stop


# ── Bar utilities ─────────────────────────────────────────────────────────────

_bar_cache: dict[str, list[dict]] = {}


def load_bars(ticker_ns: str) -> list[dict]:
    if ticker_ns in _bar_cache:
        return _bar_cache[ticker_ns]
    p = BARS_DIR / (ticker_ns.replace("_NS", ".NS") + ".json")
    if not p.exists():
        _bar_cache[ticker_ns] = []
        return []
    bars = json.load(open(p))
    bars.sort(key=lambda b: b["timestamp"])
    _bar_cache[ticker_ns] = bars
    return bars


def get_session_bars(ticker_ns: str, date_str: str) -> list[dict]:
    y, m, d = int(date_str[:4]), int(date_str[4:6]), int(date_str[6:8])
    open_ts  = int(datetime(y, m, d, 9, 15, 0, tzinfo=IST).timestamp())
    close_ts = int(datetime(y, m, d, 15, 30, 0, tzinfo=IST).timestamp())
    return [b for b in load_bars(ticker_ns) if open_ts <= b["timestamp"] <= close_ts]


def bar_time(bar: dict) -> str:
    return datetime.fromtimestamp(bar["timestamp"], tz=IST).strftime("%H:%M")


def signed_ret(close, entry, direction):
    r = (close - entry) / entry
    return r if direction == "LONG" else -r


# ── Load v0.2 paper trader CSVs ───────────────────────────────────────────────

def load_v2_trades() -> list[dict]:
    trades = []
    for date_str in OVERLAP_DATES:
        path = OUT_DIR / f"paper_trader_v2_{date_str}.csv"
        if not path.exists():
            print(f"[loss_analysis] WARNING: {path.name} not found — run run_paper_trader_v2.py first")
            continue
        with open(path, newline="") as f:
            for row in csv.DictReader(f):
                row["date"] = date_str
                trades.append(row)
    return trades


# ── Loss classification ───────────────────────────────────────────────────────

@dataclass
class LossAnalysis:
    ticker: str
    direction: str
    date: str
    entry_price: float
    exit_reason: str
    realized_return: float
    h300_counterfactual_ret: float
    candidate_stop_pct: Optional[float]
    coverage_quality: str
    n_obs: int
    # Path reconstruction
    was_ever_positive: bool
    max_positive_ret: float
    max_positive_time: Optional[str]
    max_adverse_ret: float
    max_adverse_time: Optional[str]
    # Best exit
    best_exit_time: Optional[str]
    best_exit_ret: float
    recoverable_pp: float          # actual - best_exit (positive = we left money on table)
    # Inversion opportunity
    invert_opportunity: bool
    invert_peak_ret: float         # best return in opposite direction after exit
    invert_peak_time: Optional[str]
    # Classification
    loss_category: str             # BAD_ENTRY | THESIS_DETERIORATION | STOP_PREMATURE | MISSED_INVERSION | UNAVOIDABLE


def analyse_loss(trade: dict) -> Optional[LossAnalysis]:
    """Reconstruct the full path for a losing trade and classify it."""
    ticker    = trade["ticker"]
    direction = trade["direction"]
    date_str  = trade["date"]
    try:
        entry     = float(trade["entry_price"])
        realized  = float(trade["realized_return"])
        cf_ret    = float(trade["h300_counterfactual_ret"])
        cand_pct  = float(trade["candidate_stop_pct"]) if trade.get("candidate_stop_pct") not in ("None","") else None
    except (ValueError, KeyError):
        return None

    if realized >= 0:
        return None  # not a losing trade

    exit_reason = trade.get("exit_reason", "")
    quality     = trade.get("coverage_quality", "")
    n_obs       = int(trade.get("n_obs", 0))

    sbars = get_session_bars(ticker, date_str)
    if len(sbars) < 13:
        return None

    # Walk from bar 13 (H60 entry) to H300
    post_entry_bars = sbars[12:60]
    if not post_entry_bars:
        return None

    # Build full return path
    path_rets = [signed_ret(b["close"], entry, direction) for b in post_entry_bars]
    path_times = [bar_time(b) for b in post_entry_bars]

    # Was ever positive?
    max_pos_ret = max(path_rets) if path_rets else 0.0
    max_pos_idx = path_rets.index(max_pos_ret) if path_rets else 0
    was_positive = max_pos_ret > INITIAL_POSITIVE_THRESHOLD

    max_adv_ret = min(path_rets) if path_rets else 0.0
    max_adv_idx = path_rets.index(max_adv_ret) if path_rets else 0

    # Best exit: bar with highest return (minimises loss / maximises gain)
    best_exit_idx = path_rets.index(max(path_rets))
    best_exit_ret = path_rets[best_exit_idx]
    best_exit_time = path_times[best_exit_idx]
    recoverable = realized - best_exit_ret  # negative = we could have done better

    # Inversion opportunity: after the actual exit bar, does opposite direction produce >threshold?
    # Find actual exit bar index
    actual_exit_bar = len(post_entry_bars) - 1  # default = last bar
    if exit_reason == "STOP" and cand_pct is not None:
        stop_price = entry * (1 + cand_pct) if direction == "LONG" else entry * (1 - abs(cand_pct))
        for i, bar in enumerate(post_entry_bars):
            if direction == "LONG" and bar["low"] <= stop_price:
                actual_exit_bar = i
                break
            elif direction == "SHORT" and bar["high"] >= stop_price:
                actual_exit_bar = i
                break

    # Opposite direction returns after exit
    opp_direction = "SHORT" if direction == "LONG" else "LONG"
    opp_rets = []
    opp_times = []
    exit_price = post_entry_bars[actual_exit_bar]["close"]
    for bar in post_entry_bars[actual_exit_bar + 1:]:
        opp_rets.append(signed_ret(bar["close"], exit_price, opp_direction))
        opp_times.append(bar_time(bar))

    invert_peak = max(opp_rets) if opp_rets else 0.0
    invert_peak_idx = opp_rets.index(invert_peak) if opp_rets else 0
    invert_peak_time = opp_times[invert_peak_idx] if opp_times else None
    invert_opportunity = invert_peak > INVERT_OPPORTUNITY_MIN

    # Stop recovery check: if stopped, did price recover after stop?
    stop_recovery = False
    if exit_reason == "STOP" and actual_exit_bar < len(post_entry_bars) - 1:
        post_stop_rets = path_rets[actual_exit_bar + 1:]
        if post_stop_rets and max(post_stop_rets) > path_rets[actual_exit_bar] + STOP_RECOVERY_MIN:
            stop_recovery = True

    # Classification
    if exit_reason == "STOP" and stop_recovery:
        category = "STOP_PREMATURE"
    elif invert_opportunity and was_positive:
        category = "MISSED_INVERSION"
    elif invert_opportunity and not was_positive:
        category = "MISSED_INVERSION"
    elif was_positive and max_pos_idx < actual_exit_bar:
        category = "THESIS_DETERIORATION"
    elif not was_positive:
        category = "BAD_ENTRY"
    else:
        category = "UNAVOIDABLE"

    return LossAnalysis(
        ticker=ticker, direction=direction, date=date_str,
        entry_price=entry, exit_reason=exit_reason,
        realized_return=realized, h300_counterfactual_ret=cf_ret,
        candidate_stop_pct=cand_pct, coverage_quality=quality, n_obs=n_obs,
        was_ever_positive=was_positive,
        max_positive_ret=max_pos_ret,
        max_positive_time=path_times[max_pos_idx] if path_times else None,
        max_adverse_ret=max_adv_ret,
        max_adverse_time=path_times[max_adv_idx] if path_times else None,
        best_exit_time=best_exit_time,
        best_exit_ret=best_exit_ret,
        recoverable_pp=recoverable,
        invert_opportunity=invert_opportunity,
        invert_peak_ret=invert_peak,
        invert_peak_time=invert_peak_time,
        loss_category=category,
    )


# ── Main ──────────────────────────────────────────────────────────────────────

def run() -> None:
    trades = load_v2_trades()
    print(f"[loss_analysis] Total v0.2 trades loaded: {len(trades)}")

    losses: list[LossAnalysis] = []
    for trade in trades:
        la = analyse_loss(trade)
        if la is not None:
            losses.append(la)

    print(f"[loss_analysis] Losing trades: {len(losses)}")
    print()

    if not losses:
        print("[loss_analysis] No losing trades found.")
        return

    # ── Print loss table ──────────────────────────────────────────────────────
    print("=" * 130)
    print(f"Loss Trade Analysis — v0.2 Five-Date Replay  (N={len(losses)} losing trades)")
    print("=" * 130)
    print(f"  {'ticker':25} {'dir':5} {'date':10}  {'exit':8}  "
          f"{'ret':>8}  {'cf':>8}  {'best_exit':>9}  {'best_ret':>9}  {'recover':>8}  "
          f"{'invert?':>7}  {'inv_peak':>9}  {'category':>22}")
    print(f"  {'─'*125}")

    for la in sorted(losses, key=lambda x: (x.date, x.ticker)):
        inv_str  = "YES" if la.invert_opportunity else "no"
        inv_peak = f"{la.invert_peak_ret*100:+.3f}%" if la.invert_opportunity else "      —"
        print(f"  {la.ticker:25} {la.direction:5} {la.date:10}  {la.exit_reason:8}  "
              f"{la.realized_return*100:>+7.3f}%  {la.h300_counterfactual_ret*100:>+7.3f}%  "
              f"{la.best_exit_time:>9}  {la.best_exit_ret*100:>+8.3f}%  "
              f"{la.recoverable_pp*100:>+7.3f}pp  "
              f"{inv_str:>7}  {inv_peak:>9}  {la.loss_category:>22}")

    print(f"  {'─'*125}")
    print()

    # ── Category breakdown ────────────────────────────────────────────────────
    categories = {}
    for la in losses:
        categories[la.loss_category] = categories.get(la.loss_category, [])
        categories[la.loss_category].append(la)

    print("Category breakdown:")
    print(f"  {'category':25}  {'n':>3}  {'mean_ret':>9}  {'total_ret':>10}  "
          f"{'mean_recover':>13}  {'total_recover':>14}")
    print(f"  {'─'*90}")

    total_loss = sum(la.realized_return for la in losses)
    total_recoverable = sum(la.recoverable_pp for la in losses)

    for cat in ["BAD_ENTRY", "THESIS_DETERIORATION", "STOP_PREMATURE", "MISSED_INVERSION", "UNAVOIDABLE"]:
        items = categories.get(cat, [])
        if not items:
            continue
        rets = [la.realized_return for la in items]
        recs = [la.recoverable_pp for la in items]
        print(f"  {cat:25}  {len(items):>3}  "
              f"{statistics.mean(rets)*100:>+8.3f}%  {sum(rets)*100:>+9.3f}%  "
              f"{statistics.mean(recs)*100:>+12.3f}pp  {sum(recs)*100:>+13.3f}pp")

    print(f"  {'─'*90}")
    print(f"  {'TOTAL':25}  {len(losses):>3}  "
          f"{statistics.mean([la.realized_return for la in losses])*100:>+8.3f}%  "
          f"{total_loss*100:>+9.3f}%  "
          f"{statistics.mean([la.recoverable_pp for la in losses])*100:>+12.3f}pp  "
          f"{total_recoverable*100:>+13.3f}pp")
    print()

    # ── Inversion opportunity summary ─────────────────────────────────────────
    inv_losses = [la for la in losses if la.invert_opportunity]
    print(f"Inversion opportunities: {len(inv_losses)} of {len(losses)} losing trades")
    if inv_losses:
        inv_peaks = [la.invert_peak_ret for la in inv_losses]
        print(f"  Mean invert peak return: {statistics.mean(inv_peaks)*100:+.3f}%")
        print(f"  Max  invert peak return: {max(inv_peaks)*100:+.3f}%")
        print()
        print("  Top inversion opportunities:")
        for la in sorted(inv_losses, key=lambda x: -x.invert_peak_ret)[:10]:
            print(f"    {la.ticker:25} {la.direction}→{'SHORT' if la.direction=='LONG' else 'LONG'}  "
                  f"{la.date}  orig={la.realized_return*100:+.3f}%  "
                  f"inv_peak={la.invert_peak_ret*100:+.3f}% @ {la.invert_peak_time}")
    print()

    # ── Write CSV ─────────────────────────────────────────────────────────────
    out_path = OUT_DIR / "loss_analysis_v2.csv"
    fields = list(asdict(losses[0]).keys())
    with open(out_path, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=fields)
        w.writeheader()
        for la in losses:
            row = asdict(la)
            w.writerow({k: (f"{v:.6f}" if isinstance(v, float) else str(v))
                         for k, v in row.items()})
    print(f"[loss_analysis] Output CSV: {out_path}")
    print()
    print("[loss_analysis] Done. IC v1, Backtest v2, Cockpit, all prior components untouched.")


if __name__ == "__main__":
    run()