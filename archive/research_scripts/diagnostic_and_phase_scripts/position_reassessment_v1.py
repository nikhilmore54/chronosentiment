#!/usr/bin/env python3
"""
Position Reassessment / Self-Correction Engine v0.1
=====================================================
Implements the first version of the continuous position reassessment loop:

  Position opened (IC v1 ACT decision)
      ↓
  Each 1-minute bar:
      ↓
  Reassess: is the original thesis still valid?
      ├── HOLD   — thesis intact, continue
      ├── EXIT   — thesis deteriorating, exit flat
      └── INVERT — thesis failed AND opposite direction now supported → reverse

This is NOT a stop-loss mechanism. It is a decision-quality monitor that
asks at each bar whether the current position is still the best available
decision given the observed path so far.

Thesis assessment uses three signals:
  1. Momentum persistence (fraction of bars since entry that are favourable)
  2. Directional drift (signed return trend over last N bars)
  3. Adverse excursion vs historical profile (MAE vs symbol's MAE_H60_p50)

Inversion criteria (conservative — requires all three):
  a. Thesis has clearly failed (momentum_persistence < 0.30 for 3+ consecutive bars)
  b. Directional drift is strongly adverse (last 5-bar drift < -0.003 per bar)
  c. Opposite direction has positive drift (last 5-bar drift in opposite direction > +0.002)

Inversion position uses the same IC v1 entry logic but in the opposite direction,
with a fresh v0.2 contextual stop.

Outputs per position:
  All fields from Paper Trader v0.2, plus:
    reassessment_events     — list of (bar, action, reason) events
    inversion_occurred      — bool
    inversion_bar           — bar number when inversion triggered (None if not)
    inversion_direction     — direction of inverted position (None if not)
    inversion_entry_price   — price at inversion entry
    inversion_exit_price    — price at inversion exit
    inversion_exit_reason   — STOP | TARGET | HORIZON
    inversion_realized_ret  — return on inverted position
    combined_realized_ret   — original + inversion returns combined

IC v1, Backtest v2, Cockpit, Symbol Movement Profiler, Contextual Stop
Envelope, Stop Decision Engine v0.1/v0.2, and Paper Trader v0.1 are NOT modified.

Usage:
  python3 scripts/position_reassessment_v1.py [--date YYYYMMDD]
  python3 scripts/position_reassessment_v1.py   # all five overlap dates
"""

import csv
import json
import sys
import statistics
from pathlib import Path
from datetime import datetime, timezone, timedelta
from dataclasses import dataclass, field, asdict
from typing import Optional

# ── Import v0.2 engine ────────────────────────────────────────────────────────
import importlib.util
_scripts = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location(
    "stop_decision_engine_v2", _scripts / "stop_decision_engine_v2.py"
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

# ── Reassessment thresholds ───────────────────────────────────────────────────

MOMENTUM_FAIL_THRESHOLD   = 0.30   # mp < 0.30 for N consecutive bars → thesis failing
MOMENTUM_FAIL_BARS        = 3      # consecutive bars required
DRIFT_ADVERSE_THRESHOLD   = -0.003 # 5-bar drift per bar < this → strongly adverse
DRIFT_INVERT_THRESHOLD    = +0.002 # 5-bar drift in opposite direction > this → invert signal
DRIFT_WINDOW              = 5      # bars for drift calculation


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


def signed_ret(close, entry, direction):
    r = (close - entry) / entry
    return r if direction == "LONG" else -r


def bar_time(bar: dict) -> str:
    return datetime.fromtimestamp(bar["timestamp"], tz=IST).strftime("%H:%M")


# ── Ledger target loader ──────────────────────────────────────────────────────

def load_ledger_targets(date_str: str) -> dict[str, float]:
    targets: dict[str, float] = {}
    for ep in sorted(LEDGER_DIR.glob(f"LIVE-005-{date_str}-*.json")):
        e = json.load(open(ep))
        ticker = e.get("ticker", "")
        t = e.get("adaptive_target")
        if ticker and t is not None:
            targets[ticker] = t
    return targets


# ── Reassessment signals ──────────────────────────────────────────────────────

def momentum_persistence(direction: str, entry: float, bars: list[dict]) -> float:
    """Fraction of bars since entry where close is favourable."""
    if not bars:
        return 0.5
    favourable = sum(1 for b in bars if signed_ret(b["close"], entry, direction) > 0)
    return favourable / len(bars)


def directional_drift(direction: str, entry: float, bars: list[dict], window: int = 5) -> float:
    """Mean per-bar signed return over the last `window` bars."""
    recent = bars[-window:] if len(bars) >= window else bars
    if not recent:
        return 0.0
    rets = [signed_ret(b["close"], entry, direction) for b in recent]
    return sum(rets) / len(rets)


def opposite_drift(direction: str, entry: float, bars: list[dict], window: int = 5) -> float:
    """Directional drift in the OPPOSITE direction (positive = opposite is working)."""
    opp = "SHORT" if direction == "LONG" else "LONG"
    return directional_drift(opp, entry, bars, window)


# ── Position lifecycle dataclass ──────────────────────────────────────────────

@dataclass
class ReassessedPosition:
    ticker: str
    direction: str
    date: str
    entry_price: float
    adaptive_target: float
    adaptive_risk: float
    candidate_stop_pct: Optional[float]
    candidate_stop_price: Optional[float]
    coverage_quality: str
    n_obs: int
    # Original position exit
    exit_price: Optional[float]
    exit_time_ist: Optional[str]
    exit_reason: str          # STOP | TARGET | HORIZON | REASSESS_EXIT
    realized_return: Optional[float]
    max_adverse_excursion: float
    bars_held: int
    # Reassessment
    inversion_occurred: bool
    inversion_bar: Optional[int]
    inversion_direction: Optional[str]
    inversion_entry_price: Optional[float]
    inversion_stop_price: Optional[float]
    inversion_exit_price: Optional[float]
    inversion_exit_reason: Optional[str]
    inversion_realized_ret: Optional[float]
    # Combined
    combined_realized_ret: Optional[float]
    # Counterfactual (no stop, no inversion)
    h300_counterfactual_ret: Optional[float]


# ── Walk with reassessment ────────────────────────────────────────────────────

def walk_with_reassessment(
    direction: str, entry: float, target: float,
    stop_price: Optional[float], sbars: list[dict],
    profile: Optional[dict], profiles: dict,
    ticker: str, start_bar: int = 12
) -> ReassessedPosition:
    """
    Walk the position bar by bar, reassessing at each step.
    Returns a fully populated ReassessedPosition.
    """
    mae = mfe = 0.0
    bars_held = 0
    history: list[dict] = []  # bars seen since entry

    # Consecutive momentum-fail counter
    mp_fail_streak = 0

    exit_price = exit_time = exit_reason = None
    inversion_occurred = False
    inversion_bar = inversion_direction = None
    inversion_entry = inversion_stop = None
    inversion_exit_price = inversion_exit_reason = None
    inversion_ret = None

    for i, bar in enumerate(sbars[start_bar:]):
        bar_idx = start_bar + i  # 0-indexed from session start
        bars_held += 1
        low, high = bar["low"], bar["high"]
        ts = bar_time(bar)

        # Track MAE/MFE
        if direction == "LONG":
            mae = min(mae, (low  - entry) / entry)
            mfe = max(mfe, (high - entry) / entry)
        else:
            mae = min(mae, (entry - high) / entry)
            mfe = max(mfe, (entry - low)  / entry)

        history.append(bar)

        # ── Event detection: TARGET first, then STOP ──────────────────────────
        if direction == "LONG":
            if high >= target:
                exit_price, exit_time, exit_reason = target, ts, "TARGET"
                break
            if stop_price is not None and low <= stop_price:
                exit_price, exit_time, exit_reason = stop_price, ts, "STOP"
                break
        else:
            if low <= target:
                exit_price, exit_time, exit_reason = target, ts, "TARGET"
                break
            if stop_price is not None and high >= stop_price:
                exit_price, exit_time, exit_reason = stop_price, ts, "STOP"
                break

        # ── Reassessment (only after at least DRIFT_WINDOW bars) ─────────────
        if len(history) >= DRIFT_WINDOW:
            mp = momentum_persistence(direction, entry, history)
            drift = directional_drift(direction, entry, history, DRIFT_WINDOW)
            opp_drift = opposite_drift(direction, entry, history, DRIFT_WINDOW)

            # Momentum fail streak
            if mp < MOMENTUM_FAIL_THRESHOLD:
                mp_fail_streak += 1
            else:
                mp_fail_streak = 0

            # Inversion criteria: all three must be met
            thesis_failed   = mp_fail_streak >= MOMENTUM_FAIL_BARS
            strongly_adverse = drift < DRIFT_ADVERSE_THRESHOLD
            opp_supported   = opp_drift > DRIFT_INVERT_THRESHOLD

            if thesis_failed and strongly_adverse and opp_supported:
                # Exit original position at current close
                exit_price  = bar["close"]
                exit_time   = ts
                exit_reason = "REASSESS_EXIT"

                # Invert: enter opposite direction
                inversion_occurred  = True
                inversion_bar       = bar_idx + 1  # 1-indexed
                inversion_direction = "SHORT" if direction == "LONG" else "LONG"
                inversion_entry     = bar["close"]

                # Get v0.2 stop for inverted position
                inv_profile = profiles.get((ticker, inversion_direction))
                inv_sd = _eng.make_stop_v2(
                    ticker, inversion_direction, "", inversion_entry,
                    inversion_entry * (0.97 if inversion_direction == "LONG" else 1.03),  # placeholder AR
                    inversion_entry * (1.03 if inversion_direction == "LONG" else 0.97),  # placeholder target
                    None, inv_profile
                )
                inversion_stop = inv_sd.candidate_stop_price

                # Walk inverted position from next bar to H300
                remaining = sbars[bar_idx + 1:]
                if remaining:
                    # Use original target as inversion target (symmetric)
                    inv_target_pct = abs((target - entry) / entry)
                    if inversion_direction == "LONG":
                        inv_target = inversion_entry * (1 + inv_target_pct)
                    else:
                        inv_target = inversion_entry * (1 - inv_target_pct)

                    inv_exit_p = inv_exit_t = inv_exit_r = None
                    for j, inv_bar in enumerate(remaining):
                        inv_ts = bar_time(inv_bar)
                        if inversion_direction == "LONG":
                            if inv_bar["high"] >= inv_target:
                                inv_exit_p, inv_exit_t, inv_exit_r = inv_target, inv_ts, "TARGET"
                                break
                            if inversion_stop and inv_bar["low"] <= inversion_stop:
                                inv_exit_p, inv_exit_t, inv_exit_r = inversion_stop, inv_ts, "STOP"
                                break
                        else:
                            if inv_bar["low"] <= inv_target:
                                inv_exit_p, inv_exit_t, inv_exit_r = inv_target, inv_ts, "TARGET"
                                break
                            if inversion_stop and inv_bar["high"] >= inversion_stop:
                                inv_exit_p, inv_exit_t, inv_exit_r = inversion_stop, inv_ts, "STOP"
                                break
                        if bar_idx + 1 + j >= 59:
                            break

                    if inv_exit_p is None:
                        last = remaining[min(59 - bar_idx - 1, len(remaining) - 1)]
                        inv_exit_p = last["close"]
                        inv_exit_t = bar_time(last)
                        inv_exit_r = "HORIZON"

                    inversion_exit_price  = inv_exit_p
                    inversion_exit_reason = inv_exit_r
                    inversion_ret = signed_ret(inv_exit_p, inversion_entry, inversion_direction)
                break

        # H300 horizon
        if bar_idx >= 59:
            break

    # Horizon exit if not already exited
    if exit_price is None:
        horizon_bar = sbars[min(59, len(sbars) - 1)] if sbars else None
        if horizon_bar:
            exit_price  = horizon_bar["close"]
            exit_time   = bar_time(horizon_bar)
            exit_reason = "HORIZON"
        else:
            exit_price, exit_time, exit_reason = entry, "15:14", "HORIZON"

    realized_ret = signed_ret(exit_price, entry, direction) if exit_price is not None else None

    # Combined return
    combined = realized_ret
    if inversion_ret is not None and realized_ret is not None:
        combined = realized_ret + inversion_ret

    # Counterfactual: no stop, no inversion
    cf_exit = None
    for i, bar in enumerate(sbars[12:]):
        if 12 + i >= 59:
            cf_exit = bar["close"]
            break
        if direction == "LONG" and bar["high"] >= target:
            cf_exit = target
            break
        if direction == "SHORT" and bar["low"] <= target:
            cf_exit = target
            break
    if cf_exit is None and sbars:
        cf_exit = sbars[min(59, len(sbars) - 1)]["close"]
    cf_ret = signed_ret(cf_exit, entry, direction) if cf_exit is not None else None

    return ReassessedPosition(
        ticker=ticker, direction=direction, date="",
        entry_price=entry, adaptive_target=target, adaptive_risk=0.0,
        candidate_stop_pct=None, candidate_stop_price=stop_price,
        coverage_quality="", n_obs=0,
        exit_price=exit_price, exit_time_ist=exit_time, exit_reason=exit_reason,
        realized_return=realized_ret,
        max_adverse_excursion=mae, bars_held=bars_held,
        inversion_occurred=inversion_occurred,
        inversion_bar=inversion_bar,
        inversion_direction=inversion_direction,
        inversion_entry_price=inversion_entry,
        inversion_stop_price=inversion_stop,
        inversion_exit_price=inversion_exit_price,
        inversion_exit_reason=inversion_exit_reason,
        inversion_realized_ret=inversion_ret,
        combined_realized_ret=combined,
        h300_counterfactual_ret=cf_ret,
    )


# ── Run one date ──────────────────────────────────────────────────────────────

def run_date(date_str: str, profiles: dict) -> list[ReassessedPosition]:
    stop_decisions = _eng.run_date(date_str, profiles)
    if not stop_decisions:
        return []

    ledger_targets = {}
    for ep in sorted(LEDGER_DIR.glob(f"LIVE-005-{date_str}-*.json")):
        e = json.load(open(ep))
        t = e.get("adaptive_target")
        if t is not None:
            ledger_targets[e.get("ticker", "")] = t

    positions: list[ReassessedPosition] = []

    for sd in stop_decisions:
        ticker    = sd.ticker
        direction = sd.direction
        ref       = sd.entry_price
        target    = ledger_targets.get(ticker)
        if target is None:
            continue

        sbars = get_session_bars(ticker, date_str)
        if len(sbars) < 12:
            continue

        profile = profiles.get((ticker, direction))
        pos = walk_with_reassessment(
            direction, ref, target, sd.candidate_stop_price,
            sbars, profile, profiles, ticker, start_bar=12
        )
        pos.ticker    = ticker
        pos.direction = direction
        pos.date      = date_str
        pos.adaptive_risk = sd.adaptive_risk
        pos.candidate_stop_pct   = sd.candidate_stop_pct
        pos.candidate_stop_price = sd.candidate_stop_price
        pos.coverage_quality = sd.coverage_quality
        pos.n_obs = sd.n_obs
        positions.append(pos)

    return positions


# ── Main ──────────────────────────────────────────────────────────────────────

def run() -> None:
    single_date = None
    for i, arg in enumerate(sys.argv[1:]):
        if arg.startswith("--date="):
            single_date = arg.split("=")[1]
        elif arg == "--date" and i + 2 < len(sys.argv):
            single_date = sys.argv[i + 2]

    dates = [single_date] if single_date else OVERLAP_DATES
    profiles = _eng.load_profiles(PROFILE_CSV)
    print(f"[reassess_v1] Profiles loaded: {len(profiles)}")
    print(f"[reassess_v1] Dates: {dates}")
    print()

    all_positions: list[ReassessedPosition] = []

    for date_str in dates:
        positions = run_date(date_str, profiles)
        all_positions.extend(positions)

        if not positions:
            continue

        inversions = [p for p in positions if p.inversion_occurred]
        stops      = [p for p in positions if p.exit_reason == "STOP"]
        reassess   = [p for p in positions if p.exit_reason == "REASSESS_EXIT"]
        horizons   = [p for p in positions if p.exit_reason == "HORIZON"]
        targets    = [p for p in positions if p.exit_reason == "TARGET"]

        combined_rets = [p.combined_realized_ret for p in positions if p.combined_realized_ret is not None]
        cf_rets       = [p.h300_counterfactual_ret for p in positions if p.h300_counterfactual_ret is not None]
        wins          = sum(1 for r in combined_rets if r > 0)

        print(f"  {date_str}  N={len(positions)}  "
              f"STOP={len(stops)}  REASSESS={len(reassess)}  TGT={len(targets)}  HOR={len(horizons)}  "
              f"INVERSIONS={len(inversions)}  "
              f"win={wins/len(combined_rets)*100:.0f}%  "
              f"mean={statistics.mean(combined_rets)*100:+.3f}%  "
              f"total={sum(combined_rets)*100:+.3f}%  "
              f"cf_total={sum(cf_rets)*100:+.3f}%")

        # Show inversions
        for p in inversions:
            print(f"    INVERT: {p.ticker} {p.direction}→{p.inversion_direction}  "
                  f"bar={p.inversion_bar}  "
                  f"orig_ret={p.realized_return*100:+.3f}%  "
                  f"inv_ret={p.inversion_realized_ret*100:+.3f}%  "
                  f"combined={p.combined_realized_ret*100:+.3f}%  "
                  f"cf={p.h300_counterfactual_ret*100:+.3f}%")

        # Write CSV
        out_path = OUT_DIR / f"reassessment_v1_{date_str}.csv"
        fields = list(asdict(positions[0]).keys())
        with open(out_path, "w", newline="") as f:
            w = csv.DictWriter(f, fieldnames=fields)
            w.writeheader()
            for p in positions:
                row = asdict(p)
                w.writerow({k: (f"{v:.6f}" if isinstance(v, float) else str(v))
                             for k, v in row.items()})
        print(f"    → {out_path.name}")
        print()

    # ── Aggregate ─────────────────────────────────────────────────────────────
    if len(dates) > 1 and all_positions:
        print("=" * 100)
        print(f"Aggregate — Position Reassessment v0.1  (N={len(all_positions)})")
        print("=" * 100)
        all_combined = [p.combined_realized_ret for p in all_positions if p.combined_realized_ret is not None]
        all_cf       = [p.h300_counterfactual_ret for p in all_positions if p.h300_counterfactual_ret is not None]
        all_inv      = [p for p in all_positions if p.inversion_occurred]
        all_wins     = sum(1 for r in all_combined if r > 0)

        print(f"  Total positions:   {len(all_positions)}")
        print(f"  Inversions:        {len(all_inv)}")
        print(f"  Win rate:          {all_wins/len(all_combined)*100:.0f}%")
        print(f"  Mean combined ret: {statistics.mean(all_combined)*100:+.3f}%")
        print(f"  Total combined:    {sum(all_combined)*100:+.3f}%")
        print(f"  No-stop H300 cf:   {sum(all_cf)*100:+.3f}%")
        print(f"  vs v0.2 paper:     +90.228%  (v0.2 five-date total)")
        print(f"  vs no-stop:        {(sum(all_combined) - sum(all_cf))*100:+.3f}pp")
        print()

    print("[reassess_v1] Done. IC v1, Backtest v2, Cockpit, all prior components untouched.")


if __name__ == "__main__":
    run()