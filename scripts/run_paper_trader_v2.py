#!/usr/bin/env python3
"""
Paper Trader v0.2 Runner
=========================
Runs the complete paper-trading loop using Stop Decision Engine v0.2 stops
across all five overlap dates. Paper Trader v0.1 is NOT modified.

This script:
  1. Generates v0.2 stop decisions for each date (via stop_decision_engine_v2)
  2. Runs the position lifecycle loop using those stops
  3. Produces per-date and aggregate comparison tables:
       v0.2 (contextual stops) vs no-stop H300 baseline

Dates: 20260903, 20260904, 20260907, 20260908, 20260909

IC v1, Backtest v2, Cockpit, Symbol Movement Profiler, Contextual Stop
Envelope, Stop Decision Engine v0.1, and Paper Trader v0.1 are NOT modified.

Usage:
  python3 scripts/run_paper_trader_v2.py
  python3 scripts/run_paper_trader_v2.py --date 20260908   # single date
"""

import csv
import json
import sys
import statistics
from pathlib import Path
from datetime import datetime, timezone, timedelta
from dataclasses import dataclass, asdict
from typing import Optional

# ── Import v0.2 engine ────────────────────────────────────────────────────────
# Add scripts/ to path so we can import the engine module
import importlib.util, os
_scripts = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location(
    "stop_decision_engine_v2",
    _scripts / "stop_decision_engine_v2.py"
)
_eng = importlib.util.module_from_spec(spec)
spec.loader.exec_module(_eng)

# ── Paths ─────────────────────────────────────────────────────────────────────

REPO_ROOT   = Path(__file__).resolve().parent.parent
PROFILE_CSV = REPO_ROOT / "datasets" / "symbol_movement_profiles.csv"
LEDGER_DIR  = REPO_ROOT / "live_capture" / "ledger" / "entries"
BARS_DIR    = REPO_ROOT / "intraday_capture" / "yahoo_cache_1m"
OUT_DIR     = REPO_ROOT / "datasets"

IST = timezone(timedelta(hours=5, minutes=30))

OVERLAP_DATES = ["20260903", "20260904", "20260907", "20260908", "20260909"]


# ── Bar utilities (same as paper_trader.py) ───────────────────────────────────

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


def signed_ret(close, entry, direction):
    r = (close - entry) / entry
    return r if direction == "LONG" else -r


# ── Position walk (same logic as paper_trader.py) ─────────────────────────────

def walk_position(direction, entry, target, stop_price, sbars, start_bar=12):
    mae = mfe = 0.0
    bars_held = 0
    for i, bar in enumerate(sbars[start_bar:]):
        bars_held += 1
        low, high = bar["low"], bar["high"]
        ts = datetime.fromtimestamp(bar["timestamp"], tz=IST).strftime("%H:%M")
        if direction == "LONG":
            mae = min(mae, (low  - entry) / entry)
            mfe = max(mfe, (high - entry) / entry)
        else:
            mae = min(mae, (entry - high) / entry)
            mfe = max(mfe, (entry - low)  / entry)
        if direction == "LONG":
            if high >= target:
                return target, ts, "TARGET", mae, mfe, bars_held
            if stop_price is not None and low <= stop_price:
                return stop_price, ts, "STOP", mae, mfe, bars_held
        else:
            if low <= target:
                return target, ts, "TARGET", mae, mfe, bars_held
            if stop_price is not None and high >= stop_price:
                return stop_price, ts, "STOP", mae, mfe, bars_held
        if start_bar + i >= 59:
            break
    horizon_bar = sbars[min(59, len(sbars) - 1)] if sbars else None
    if horizon_bar:
        ts = datetime.fromtimestamp(horizon_bar["timestamp"], tz=IST).strftime("%H:%M")
        return horizon_bar["close"], ts, "HORIZON", mae, mfe, bars_held
    return entry, "15:14", "HORIZON", mae, mfe, bars_held


# ── Position lifecycle dataclass ──────────────────────────────────────────────

@dataclass
class PositionV2:
    ticker: str
    direction: str
    date: str
    entry_price: float
    adaptive_target: float
    adaptive_risk: float
    candidate_stop_pct: Optional[float]
    candidate_stop_price: Optional[float]
    stop_confidence: str
    coverage_quality: str
    n_obs: int
    path_context: str
    final_tier: str
    exit_price: Optional[float]
    exit_time_ist: Optional[str]
    exit_reason: str
    realized_return: Optional[float]
    max_adverse_excursion: float
    max_favourable_excursion: float
    bars_held: int
    h300_counterfactual_ret: Optional[float]
    stop_consequence_pp: Optional[float]


# ── Run one date ──────────────────────────────────────────────────────────────

def load_ledger_targets(date_str: str) -> dict[str, float]:
    """Return ticker → adaptive_target from ledger entries for the given date."""
    targets: dict[str, float] = {}
    for ep in sorted(LEDGER_DIR.glob(f"LIVE-005-{date_str}-*.json")):
        e = json.load(open(ep))
        ticker = e.get("ticker", "")
        t = e.get("adaptive_target")
        if ticker and t is not None:
            targets[ticker] = t
    return targets


def run_date(date_str: str, profiles: dict) -> list[PositionV2]:
    # Generate v0.2 stop decisions
    stop_decisions = _eng.run_date(date_str, profiles)
    if not stop_decisions:
        return []

    # Load adaptive_target from ledger (not stored in StopDecisionV2)
    ledger_targets = load_ledger_targets(date_str)

    positions: list[PositionV2] = []

    for sd in stop_decisions:
        ticker    = sd.ticker
        direction = sd.direction
        ref       = sd.entry_price
        target    = ledger_targets.get(ticker)
        risk      = sd.adaptive_risk
        if target is None:
            continue

        sbars = get_session_bars(ticker, date_str)
        if len(sbars) < 12:
            continue

        cand_price = sd.candidate_stop_price

        # Walk with v0.2 candidate stop
        exit_price, exit_time, exit_reason, mae, mfe, bars_held = walk_position(
            direction, ref, target, cand_price, sbars, start_bar=12
        )
        realized_ret = signed_ret(exit_price, ref, direction) if exit_price is not None else None

        # Counterfactual: no-stop H300
        cf_exit, _, _, _, _, _ = walk_position(direction, ref, target, None, sbars, start_bar=12)
        cf_ret = signed_ret(cf_exit, ref, direction) if cf_exit is not None else None

        stop_consequence = None
        if realized_ret is not None and cf_ret is not None:
            stop_consequence = realized_ret - cf_ret

        positions.append(PositionV2(
            ticker=ticker, direction=direction, date=date_str,
            entry_price=ref, adaptive_target=target, adaptive_risk=risk,
            candidate_stop_pct=sd.candidate_stop_pct,
            candidate_stop_price=cand_price,
            stop_confidence=sd.confidence,
            coverage_quality=sd.coverage_quality,
            n_obs=sd.n_obs,
            path_context=sd.path_context,
            final_tier=sd.final_tier,
            exit_price=exit_price,
            exit_time_ist=exit_time,
            exit_reason=exit_reason,
            realized_return=realized_ret,
            max_adverse_excursion=mae,
            max_favourable_excursion=mfe,
            bars_held=bars_held,
            h300_counterfactual_ret=cf_ret,
            stop_consequence_pp=stop_consequence,
        ))

    return positions


# ── Print date summary ────────────────────────────────────────────────────────

def print_date_summary(date_str: str, positions: list[PositionV2]) -> None:
    if not positions:
        print(f"  {date_str}: no positions")
        return

    stops   = [p for p in positions if p.exit_reason == "STOP"]
    targets = [p for p in positions if p.exit_reason == "TARGET"]
    horizons= [p for p in positions if p.exit_reason == "HORIZON"]
    rets    = [p.realized_return for p in positions if p.realized_return is not None]
    cfs     = [p.h300_counterfactual_ret for p in positions if p.h300_counterfactual_ret is not None]
    cons    = [p.stop_consequence_pp for p in positions if p.stop_consequence_pp is not None]
    wins    = [r for r in rets if r > 0]
    cf_wins = [r for r in cfs if r > 0]

    print(f"  {date_str}  N={len(positions):>2}  "
          f"STOP={len(stops):>2}  TGT={len(targets):>2}  HOR={len(horizons):>2}  "
          f"win={len(wins)/len(rets)*100:.0f}%  "
          f"mean={statistics.mean(rets)*100:+.3f}%  "
          f"total={sum(rets)*100:+.3f}%  "
          f"cf_total={sum(cfs)*100:+.3f}%  "
          f"consequence={sum(cons)*100:+.3f}pp")


# ── Main ──────────────────────────────────────────────────────────────────────

def run() -> None:
    # Parse args
    single_date = None
    for i, arg in enumerate(sys.argv[1:]):
        if arg.startswith("--date="):
            single_date = arg.split("=")[1]
        elif arg == "--date" and i + 2 < len(sys.argv):
            single_date = sys.argv[i + 2]

    dates = [single_date] if single_date else OVERLAP_DATES

    profiles = _eng.load_profiles(PROFILE_CSV)
    print(f"[paper_v2] Profiles loaded: {len(profiles)}")
    print(f"[paper_v2] Dates: {dates}")
    print()

    all_positions: list[PositionV2] = []

    for date_str in dates:
        positions = run_date(date_str, profiles)
        all_positions.extend(positions)

        if not positions:
            continue

        # Per-date detail table
        print(f"{'─'*120}")
        print(f"  {date_str}  (N={len(positions)})")
        print(f"{'─'*120}")
        print(f"  {'ticker':25} {'dir':5} {'tier':15}  {'ctx':13}  "
              f"{'cand':>8}  {'exit':>9}  {'reason':>7}  "
              f"{'ret':>8}  {'cf':>8}  {'cons':>9}  {'MAE':>8}")
        for p in sorted(positions, key=lambda x: x.ticker):
            cand_s = f"{p.candidate_stop_pct*100:+.3f}%" if p.candidate_stop_pct is not None else "   N/A"
            ret_s  = f"{p.realized_return*100:+.3f}%"    if p.realized_return is not None else "   N/A"
            cf_s   = f"{p.h300_counterfactual_ret*100:+.3f}%" if p.h300_counterfactual_ret is not None else "   N/A"
            cons_s = f"{p.stop_consequence_pp*100:+.3f}pp" if p.stop_consequence_pp is not None else "   N/A"
            mae_s  = f"{p.max_adverse_excursion*100:+.3f}%"
            marker = " ◄" if p.ticker == "JUBLFOOD_NS" else ""
            print(f"  {p.ticker:25} {p.direction:5} {p.final_tier:15}  {p.path_context:13}  "
                  f"{cand_s:>8}  {p.exit_time_ist:>9}  {p.exit_reason:>7}  "
                  f"{ret_s:>8}  {cf_s:>8}  {cons_s:>9}  {mae_s:>8}{marker}")
        print()

        # Write per-date CSV
        out_path = OUT_DIR / f"paper_trader_v2_{date_str}.csv"
        fields = list(asdict(positions[0]).keys())
        with open(out_path, "w", newline="") as f:
            w = csv.DictWriter(f, fieldnames=fields)
            w.writeheader()
            for p in positions:
                row = asdict(p)
                w.writerow({k: (f"{v:.6f}" if isinstance(v, float) else str(v))
                             for k, v in row.items()})
        print(f"  → {out_path.name}")
        print()

    # ── Aggregate summary ─────────────────────────────────────────────────────
    if len(dates) > 1 and all_positions:
        print("=" * 100)
        print(f"Aggregate summary — v0.2 contextual stops  (all dates, N={len(all_positions)})")
        print("=" * 100)
        print(f"  {'date':10}  {'N':>3}  {'STOP':>5}  {'TGT':>4}  {'HOR':>4}  "
              f"{'win%':>5}  {'mean':>8}  {'total':>8}  {'cf_total':>9}  {'consequence':>12}")
        print(f"  {'─'*95}")

        for date_str in dates:
            day_pos = [p for p in all_positions if p.date == date_str]
            if not day_pos:
                continue
            stops   = sum(1 for p in day_pos if p.exit_reason == "STOP")
            targets = sum(1 for p in day_pos if p.exit_reason == "TARGET")
            horizons= sum(1 for p in day_pos if p.exit_reason == "HORIZON")
            rets    = [p.realized_return for p in day_pos if p.realized_return is not None]
            cfs     = [p.h300_counterfactual_ret for p in day_pos if p.h300_counterfactual_ret is not None]
            cons    = [p.stop_consequence_pp for p in day_pos if p.stop_consequence_pp is not None]
            wins    = sum(1 for r in rets if r > 0)
            print(f"  {date_str:10}  {len(day_pos):>3}  {stops:>5}  {targets:>4}  {horizons:>4}  "
                  f"{wins/len(rets)*100:>4.0f}%  "
                  f"{statistics.mean(rets)*100:>+7.3f}%  "
                  f"{sum(rets)*100:>+7.3f}%  "
                  f"{sum(cfs)*100:>+8.3f}%  "
                  f"{sum(cons)*100:>+11.3f}pp")

        print(f"  {'─'*95}")
        all_rets  = [p.realized_return for p in all_positions if p.realized_return is not None]
        all_cfs   = [p.h300_counterfactual_ret for p in all_positions if p.h300_counterfactual_ret is not None]
        all_cons  = [p.stop_consequence_pp for p in all_positions if p.stop_consequence_pp is not None]
        all_stops = sum(1 for p in all_positions if p.exit_reason == "STOP")
        all_tgts  = sum(1 for p in all_positions if p.exit_reason == "TARGET")
        all_hors  = sum(1 for p in all_positions if p.exit_reason == "HORIZON")
        all_wins  = sum(1 for r in all_rets if r > 0)
        print(f"  {'TOTAL':10}  {len(all_positions):>3}  {all_stops:>5}  {all_tgts:>4}  {all_hors:>4}  "
              f"{all_wins/len(all_rets)*100:>4.0f}%  "
              f"{statistics.mean(all_rets)*100:>+7.3f}%  "
              f"{sum(all_rets)*100:>+7.3f}%  "
              f"{sum(all_cfs)*100:>+8.3f}%  "
              f"{sum(all_cons)*100:>+11.3f}pp")
        print()
        print(f"  v0.2 total PnL:      {sum(all_rets)*100:+.3f}%")
        print(f"  No-stop H300 total:  {sum(all_cfs)*100:+.3f}%")
        print(f"  Stop consequence:    {sum(all_cons)*100:+.3f}pp  "
              f"({'stops cost money' if sum(all_cons) < 0 else 'stops saved money'})")
        print()

    print("[paper_v2] Done. IC v1, Backtest v2, Cockpit, Profiler, Envelope, Paper Trader v0.1 untouched.")


if __name__ == "__main__":
    run()