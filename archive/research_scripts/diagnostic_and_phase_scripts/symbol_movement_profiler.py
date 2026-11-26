#!/usr/bin/env python3
"""
Symbol Historical Movement Profiler
=====================================
Builds a per-symbol, per-direction movement profile from all available
1-minute bar data, anchored to the reference_price from ledger entries.

For each (symbol, direction, date) triple where both a ledger entry and
1-minute bars exist, the full intraday path is walked from bar 1 (09:15 IST)
to bar 75 (H300, ~14:15 IST) and the following are extracted:

  mae_h60    — max adverse excursion by H60  (bars 1-12)
  mae_h120   — max adverse excursion by H120 (bars 1-24)
  mae_h180   — max adverse excursion by H180 (bars 1-36)
  mae_h300   — max adverse excursion by H300 (bars 1-60)
  mfe_h60    — max favourable excursion by H60
  mfe_h120   — max favourable excursion by H120
  mfe_h300   — max favourable excursion by H300
  ret_h60    — return at H60 close
  ret_h120   — return at H120 close
  ret_h300   — return at H300 close
  recovery   — did price recover from MAE_H300 to positive territory?
  time_to_recovery_bars — bars from entry until first positive return (None if never)

The profile is then aggregated per (symbol, direction) across all available days:

  n_obs              — number of observations
  mae_h300_p25/p50/p75/p90/p95  — adverse excursion percentiles at H300
  mae_h60_p50/p75    — adverse excursion percentiles at H60
  mfe_h300_p50       — typical favourable excursion
  recovery_rate      — fraction of observations that recovered to positive
  mean_ret_h300      — mean H300 return
  win_rate_h300      — fraction with positive H300 return

Information boundary: the profiler uses the full path as a historical
observation. It does NOT use future information at decision time — the
profile is pre-computed and consumed as a static lookup at entry.

IC v1, Backtest v2, and Cockpit are NOT modified.

Outputs:
  datasets/symbol_movement_profiles.csv   — per-(symbol, direction) profile
  datasets/symbol_movement_observations.csv — per-(symbol, direction, date) raw rows

Usage:
  python3 scripts/symbol_movement_profiler.py
"""

import json
import csv
import statistics
from pathlib import Path
from datetime import datetime, timezone, timedelta
from dataclasses import dataclass, asdict
from typing import Optional

# ── Paths ─────────────────────────────────────────────────────────────────────

REPO_ROOT   = Path(__file__).resolve().parent.parent
LEDGER_DIR  = REPO_ROOT / "live_capture" / "ledger" / "entries"
BARS_DIR    = REPO_ROOT / "intraday_capture" / "yahoo_cache_1m"
OUT_OBS     = REPO_ROOT / "datasets" / "symbol_movement_observations.csv"
OUT_PROFILE = REPO_ROOT / "datasets" / "symbol_movement_profiles.csv"

# ── IST ───────────────────────────────────────────────────────────────────────

IST = timezone(timedelta(hours=5, minutes=30))


def session_open(date_str: str) -> int:
    """Return Unix timestamp of 09:15 IST for the given YYYYMMDD date string."""
    y, m, d = int(date_str[:4]), int(date_str[4:6]), int(date_str[6:8])
    return int(datetime(y, m, d, 9, 15, 0, tzinfo=IST).timestamp())


def session_close(date_str: str) -> int:
    y, m, d = int(date_str[:4]), int(date_str[4:6]), int(date_str[6:8])
    return int(datetime(y, m, d, 15, 30, 0, tzinfo=IST).timestamp())


# ── Bar loading ───────────────────────────────────────────────────────────────

_bar_cache: dict[str, list[dict]] = {}


def load_bars(ticker_ns: str) -> list[dict]:
    if ticker_ns in _bar_cache:
        return _bar_cache[ticker_ns]
    filename = ticker_ns.replace("_NS", ".NS") + ".json"
    path = BARS_DIR / filename
    if not path.exists():
        _bar_cache[ticker_ns] = []
        return []
    with open(path) as f:
        bars = json.load(f)
    bars.sort(key=lambda b: b["timestamp"])
    _bar_cache[ticker_ns] = bars
    return bars


def get_session_bars(ticker_ns: str, date_str: str) -> list[dict]:
    all_bars = load_bars(ticker_ns)
    open_ts  = session_open(date_str)
    close_ts = session_close(date_str)
    return [b for b in all_bars if open_ts <= b["timestamp"] <= close_ts]


# ── Path analysis ─────────────────────────────────────────────────────────────

@dataclass
class Observation:
    ticker: str
    direction: str
    date: str
    entry_price: float
    # Returns at horizons
    ret_h60:  Optional[float]
    ret_h120: Optional[float]
    ret_h300: Optional[float]
    # Max adverse excursion (worst direction-adjusted return seen up to horizon)
    mae_h60:  Optional[float]
    mae_h120: Optional[float]
    mae_h180: Optional[float]
    mae_h300: Optional[float]
    # Max favourable excursion
    mfe_h60:  Optional[float]
    mfe_h120: Optional[float]
    mfe_h300: Optional[float]
    # Recovery
    recovered_to_positive: bool
    time_to_recovery_bars: Optional[int]
    # Session bar count
    n_bars: int


def analyse_path(direction: str, entry_price: float, sbars: list[dict]) -> Observation:
    """
    Walk the full session path from bar 0 and compute movement statistics.
    sbars: session bars sorted by timestamp, bar 0 = 09:15 IST.
    """
    def signed_ret(close: float) -> float:
        raw = (close - entry_price) / entry_price
        return raw if direction == "LONG" else -raw

    mae = 0.0
    mfe = 0.0

    ret_h60 = ret_h120 = ret_h300 = None
    mae_h60 = mae_h120 = mae_h180 = mae_h300 = None
    mfe_h60 = mfe_h120 = mfe_h300 = None
    time_to_recovery: Optional[int] = None

    for i, bar in enumerate(sbars):
        bar_num = i + 1  # 1-indexed

        # Track MAE/MFE using bar extremes
        if direction == "LONG":
            adverse   = (bar["low"]  - entry_price) / entry_price
            favorable = (bar["high"] - entry_price) / entry_price
        else:
            adverse   = (entry_price - bar["high"]) / entry_price
            favorable = (entry_price - bar["low"])  / entry_price

        mae = min(mae, adverse)
        mfe = max(mfe, favorable)

        close_ret = signed_ret(bar["close"])

        # Recovery tracking
        if time_to_recovery is None and close_ret > 0:
            time_to_recovery = bar_num

        # Snapshot at horizons (bar 12 = H60, bar 24 = H120, bar 36 = H180, bar 60 = H300)
        if bar_num == 12:
            ret_h60  = close_ret
            mae_h60  = mae
            mfe_h60  = mfe
        elif bar_num == 24:
            ret_h120 = close_ret
            mae_h120 = mae
            mfe_h120 = mfe
        elif bar_num == 36:
            mae_h180 = mae
        elif bar_num == 60:
            ret_h300 = close_ret
            mae_h300 = mae
            mfe_h300 = mfe
            break  # H300 is our horizon ceiling

    # If session ended before H300, use last available bar
    if ret_h300 is None and sbars:
        last = sbars[min(59, len(sbars) - 1)]
        ret_h300 = signed_ret(last["close"])
        mae_h300 = mae
        mfe_h300 = mfe

    recovered = (ret_h300 is not None and ret_h300 > 0)

    return Observation(
        ticker=None,   # filled by caller
        direction=direction,
        date=None,     # filled by caller
        entry_price=entry_price,
        ret_h60=ret_h60,
        ret_h120=ret_h120,
        ret_h300=ret_h300,
        mae_h60=mae_h60,
        mae_h120=mae_h120,
        mae_h180=mae_h180,
        mae_h300=mae_h300,
        mfe_h60=mfe_h60,
        mfe_h120=mfe_h120,
        mfe_h300=mfe_h300,
        recovered_to_positive=recovered,
        time_to_recovery_bars=time_to_recovery,
        n_bars=len(sbars),
    )


# ── Percentile helper ─────────────────────────────────────────────────────────

def pct(values: list[float], p: float) -> float:
    """Return the p-th percentile (0–100) of values."""
    if not values:
        return float("nan")
    s = sorted(values)
    idx = (p / 100) * (len(s) - 1)
    lo, hi = int(idx), min(int(idx) + 1, len(s) - 1)
    frac = idx - lo
    return s[lo] + frac * (s[hi] - s[lo])


# ── Main ──────────────────────────────────────────────────────────────────────

def run() -> None:
    # ── Collect all (ticker, direction, date, entry_price) from ledger ────────
    ledger_records: dict[tuple[str, str, str], float] = {}  # (ticker, direction, date) → entry_price

    for ep in sorted(LEDGER_DIR.glob("LIVE-*.json")):
        parts = ep.stem.split("-")
        date = next((p for p in parts if len(p) == 8 and p.isdigit()), None)
        if date is None:
            continue
        e = json.load(open(ep))
        ticker    = e.get("ticker", "")
        direction = e.get("direction", "")
        ref_price = e.get("reference_price")
        if not ticker or not direction or ref_price is None:
            continue
        key = (ticker, direction, date)
        # Keep first entry per (ticker, direction, date) if duplicates exist
        if key not in ledger_records:
            ledger_records[key] = ref_price

    print(f"[profiler] Ledger records loaded: {len(ledger_records)}")

    # ── Walk paths and build observations ─────────────────────────────────────
    observations: list[Observation] = []
    skipped_no_bars = 0

    for (ticker, direction, date), entry_price in sorted(ledger_records.items()):
        sbars = get_session_bars(ticker, date)
        if len(sbars) < 12:
            skipped_no_bars += 1
            continue

        obs = analyse_path(direction, entry_price, sbars)
        obs.ticker    = ticker
        obs.date      = date
        observations.append(obs)

    print(f"[profiler] Observations built: {len(observations)}  (skipped no-bars: {skipped_no_bars})")

    # ── Write observations CSV ────────────────────────────────────────────────
    OUT_OBS.parent.mkdir(parents=True, exist_ok=True)
    if observations:
        fields = list(asdict(observations[0]).keys())
        with open(OUT_OBS, "w", newline="") as f:
            w = csv.DictWriter(f, fieldnames=fields)
            w.writeheader()
            for obs in observations:
                row = asdict(obs)
                w.writerow({k: (f"{v:.6f}" if isinstance(v, float) else str(v))
                             for k, v in row.items()})
        print(f"[profiler] Observations CSV: {OUT_OBS}")

    # ── Aggregate per (symbol, direction) ─────────────────────────────────────
    from collections import defaultdict
    groups: dict[tuple[str, str], list[Observation]] = defaultdict(list)
    for obs in observations:
        groups[(obs.ticker, obs.direction)].append(obs)

    profile_rows = []

    print()
    print("=" * 100)
    print(f"Symbol Movement Profiles — {len(groups)} (symbol, direction) pairs")
    print("=" * 100)
    print(f"{'symbol':25} {'dir':5} {'n':>3}  "
          f"{'MAE_H300_p50':>13} {'MAE_H300_p75':>13} {'MAE_H300_p90':>13}  "
          f"{'MAE_H60_p50':>12} {'MAE_H60_p75':>12}  "
          f"{'MFE_H300_p50':>13}  "
          f"{'win%':>5}  {'rec%':>5}")
    print("-" * 100)

    for (ticker, direction) in sorted(groups.keys()):
        obs_list = groups[(ticker, direction)]
        n = len(obs_list)

        mae_h300_vals = [o.mae_h300 for o in obs_list if o.mae_h300 is not None]
        mae_h60_vals  = [o.mae_h60  for o in obs_list if o.mae_h60  is not None]
        mfe_h300_vals = [o.mfe_h300 for o in obs_list if o.mfe_h300 is not None]
        ret_h300_vals = [o.ret_h300 for o in obs_list if o.ret_h300 is not None]

        mae_h300_p25 = pct(mae_h300_vals, 25)
        mae_h300_p50 = pct(mae_h300_vals, 50)
        mae_h300_p75 = pct(mae_h300_vals, 75)
        mae_h300_p90 = pct(mae_h300_vals, 90)
        mae_h300_p95 = pct(mae_h300_vals, 95)
        mae_h60_p50  = pct(mae_h60_vals,  50)
        mae_h60_p75  = pct(mae_h60_vals,  75)
        mfe_h300_p50 = pct(mfe_h300_vals, 50)

        win_rate     = sum(1 for v in ret_h300_vals if v > 0) / n if n > 0 else 0.0
        rec_rate     = sum(1 for o in obs_list if o.recovered_to_positive) / n if n > 0 else 0.0
        mean_ret     = statistics.mean(ret_h300_vals) if ret_h300_vals else float("nan")

        ttr_vals = [o.time_to_recovery_bars for o in obs_list if o.time_to_recovery_bars is not None]
        mean_ttr = statistics.mean(ttr_vals) if ttr_vals else float("nan")

        print(f"{ticker:25} {direction:5} {n:>3}  "
              f"{mae_h300_p50*100:>+12.3f}%  {mae_h300_p75*100:>+12.3f}%  {mae_h300_p90*100:>+12.3f}%  "
              f"{mae_h60_p50*100:>+11.3f}%  {mae_h60_p75*100:>+11.3f}%  "
              f"{mfe_h300_p50*100:>+12.3f}%  "
              f"{win_rate*100:>4.0f}%  {rec_rate*100:>4.0f}%")

        profile_rows.append({
            "ticker": ticker,
            "direction": direction,
            "n_obs": n,
            "mae_h300_p25": mae_h300_p25,
            "mae_h300_p50": mae_h300_p50,
            "mae_h300_p75": mae_h300_p75,
            "mae_h300_p90": mae_h300_p90,
            "mae_h300_p95": mae_h300_p95,
            "mae_h60_p50":  mae_h60_p50,
            "mae_h60_p75":  mae_h60_p75,
            "mfe_h300_p50": mfe_h300_p50,
            "win_rate_h300": win_rate,
            "recovery_rate": rec_rate,
            "mean_ret_h300": mean_ret,
            "mean_time_to_recovery_bars": mean_ttr,
        })

    print("-" * 100)
    print(f"  {len(profile_rows)} profiles across {len(observations)} observations")
    print()

    # ── Write profile CSV ─────────────────────────────────────────────────────
    if profile_rows:
        with open(OUT_PROFILE, "w", newline="") as f:
            w = csv.DictWriter(f, fieldnames=list(profile_rows[0].keys()))
            w.writeheader()
            for r in profile_rows:
                w.writerow({k: (f"{v:.6f}" if isinstance(v, float) else str(v))
                             for k, v in r.items()})
        print(f"[profiler] Profile CSV: {OUT_PROFILE}")

    # ── JUBLFOOD case study: show its profile vs the 8-Sep stop event ─────────
    jublfood_short = [r for r in profile_rows
                      if r["ticker"] == "JUBLFOOD_NS" and r["direction"] == "SHORT"]
    if jublfood_short:
        p = jublfood_short[0]
        print()
        print("JUBLFOOD_NS SHORT — movement profile vs 8-Sep stop event:")
        print(f"  n_obs:            {p['n_obs']}")
        print(f"  MAE_H300 p50:     {p['mae_h300_p50']*100:+.3f}%")
        print(f"  MAE_H300 p75:     {p['mae_h300_p75']*100:+.3f}%")
        print(f"  MAE_H300 p90:     {p['mae_h300_p90']*100:+.3f}%")
        print(f"  MAE_H60  p50:     {p['mae_h60_p50']*100:+.3f}%")
        print(f"  MAE_H60  p75:     {p['mae_h60_p75']*100:+.3f}%")
        print(f"  win_rate_h300:    {p['win_rate_h300']*100:.0f}%")
        print(f"  recovery_rate:    {p['recovery_rate']*100:.0f}%")
        print()
        print("  8-Sep event:")
        print(f"    entry=472.80  0.50× stop hit at -1.689%  H300 cf=-0.296%")
        print(f"    The 0.50× stop distance was: {0.50 * (472.80 - 464.80) / 472.80 * 100:.3f}%")
        print(f"    Compare to MAE_H300 p50 above — was the stop inside normal noise?")

    print()
    print("[profiler] Done. IC v1, Backtest v2, and Cockpit are untouched.")


if __name__ == "__main__":
    run()