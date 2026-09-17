#!/usr/bin/env python3
"""
Position Reassessment / Self-Correction Engine v0.2
=====================================================
EXIT-only. Inversion disabled.

Improvement over v0.1:
  v0.1 triggered inversions on GODREJPROP and TRENT — both temporary adverse
  moves that subsequently recovered. The three-signal detector (momentum
  reversal + persistence + magnitude) was too coarse.

v0.2 replaces it with a path shape classifier that distinguishes:

  SUSTAINED_DOWNTREND  — lower highs AND lower lows across N bars
                         → thesis has structurally failed → EXIT
  TEMPORARY_DIP        — adverse but recovering (higher lows or higher highs)
                         → noise, not thesis failure → HOLD

The classifier uses a rolling window of bar highs and lows, not just closes.
This is more robust because:
  - A genuine downtrend produces lower highs AND lower lows
  - A temporary dip produces lower lows but then higher lows (recovery)
  - A spike-and-recover produces lower lows but immediately higher highs

Additional guard: EXIT is only triggered if the position has been held for
at least MIN_HOLD_BARS bars. This prevents exiting on the first adverse bar.

Inversion: DISABLED in v0.2. Will be re-evaluated after EXIT logic is proven.

Production baseline: v0.2 paper trader = +90.228% (five dates).
Target: exceed +90.228% by reducing losses from THESIS_DETERIORATION trades.

IC v1, Backtest v2, Cockpit, and all prior frozen components are NOT modified.

Usage:
  python3 scripts/position_reassessment_v2.py [--date YYYYMMDD]
  python3 scripts/position_reassessment_v2.py   # all five overlap dates
"""

import csv
import json
import sys
import statistics
from pathlib import Path
from datetime import datetime, timezone, timedelta
from dataclasses import dataclass, asdict
from typing import Optional

import importlib.util
_scripts = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location(
    "stop_decision_engine_v2", _scripts / "stop_decision_engine_v2.py"
)
_eng = importlib.util.module_from_spec(spec)
spec.loader.exec_module(_eng)

REPO_ROOT   = Path(__file__).resolve().parent.parent
PROFILE_CSV = REPO_ROOT / "datasets" / "symbol_movement_profiles.csv"
LEDGER_DIR  = REPO_ROOT / "live_capture" / "ledger" / "entries"
BARS_DIR    = REPO_ROOT / "intraday_capture" / "yahoo_cache_1m"
OUT_DIR     = REPO_ROOT / "datasets"

IST = timezone(timedelta(hours=5, minutes=30))
OVERLAP_DATES = ["20260903", "20260904", "20260907", "20260908", "20260909"]

# ── Thresholds ────────────────────────────────────────────────────────────────

DOWNTREND_WINDOW  = 5    # bars to check for lower highs AND lower lows
MIN_HOLD_BARS     = 6    # minimum bars held before EXIT can trigger
LOWER_COUNT_MIN   = 4    # out of DOWNTREND_WINDOW bars, this many must be lower


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


def load_ledger_targets(date_str: str) -> dict[str, float]:
    targets: dict[str, float] = {}
    for ep in sorted(LEDGER_DIR.glob(f"LIVE-005-{date_str}-*.json")):
        e = json.load(open(ep))
        t = e.get("adaptive_target")
        if t is not None:
            targets[e.get("ticker", "")] = t
    return targets


# ── Path shape classifier ─────────────────────────────────────────────────────

def is_sustained_downtrend(direction: str, entry: float,
                            history: list[dict], window: int = DOWNTREND_WINDOW,
                            lower_count_min: int = LOWER_COUNT_MIN) -> bool:
    """
    Returns True if the last `window` bars show a sustained downtrend
    in the adverse direction.

    For LONG: downtrend = lower highs AND lower lows (price falling)
    For SHORT: downtrend = higher lows AND higher highs (price rising = adverse)

    Requires `lower_count_min` out of `window` bars to show the pattern.
    """
    if len(history) < window + 1:
        return False

    recent = history[-(window + 1):]  # window+1 to compare consecutive pairs

    lower_highs = 0
    lower_lows  = 0

    for i in range(1, len(recent)):
        prev = recent[i - 1]
        curr = recent[i]
        if direction == "LONG":
            if curr["high"] < prev["high"]:
                lower_highs += 1
            if curr["low"] < prev["low"]:
                lower_lows += 1
        else:  # SHORT: adverse = price rising
            if curr["low"] > prev["low"]:
                lower_highs += 1   # "lower" in adverse direction = higher price for SHORT
            if curr["high"] > prev["high"]:
                lower_lows += 1

    # Both highs AND lows must be consistently moving adversely
    return lower_highs >= lower_count_min and lower_lows >= lower_count_min


# ── Position lifecycle dataclass ──────────────────────────────────────────────

@dataclass
class ReassessedPositionV2:
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
    final_tier: str
    # Exit
    exit_price: Optional[float]
    exit_time_ist: Optional[str]
    exit_reason: str          # STOP | TARGET | HORIZON | REASSESS_EXIT
    realized_return: Optional[float]
    max_adverse_excursion: float
    bars_held: int
    # Reassessment
    reassess_triggered: bool
    reassess_bar: Optional[int]
    reassess_time_ist: Optional[str]
    # Counterfactual
    h300_counterfactual_ret: Optional[float]


# ── Walk with reassessment ────────────────────────────────────────────────────

def walk_with_reassessment(
    direction: str, entry: float, target: float,
    stop_price: Optional[float], sbars: list[dict],
    start_bar: int = 12
) -> dict:
    mae = 0.0
    bars_held = 0
    history: list[dict] = []

    exit_price = exit_time = exit_reason = None
    reassess_triggered = False
    reassess_bar = reassess_time = None

    for i, bar in enumerate(sbars[start_bar:]):
        bar_idx = start_bar + i
        bars_held += 1
        ts = bar_time(bar)
        low, high = bar["low"], bar["high"]

        # MAE
        if direction == "LONG":
            mae = min(mae, (low  - entry) / entry)
        else:
            mae = min(mae, (entry - high) / entry)

        history.append(bar)

        # ── Stop / target ─────────────────────────────────────────────────────
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

        # ── Reassessment: EXIT only, no inversion ─────────────────────────────
        if bars_held >= MIN_HOLD_BARS:
            if is_sustained_downtrend(direction, entry, history):
                exit_price        = bar["close"]
                exit_time         = ts
                exit_reason       = "REASSESS_EXIT"
                reassess_triggered = True
                reassess_bar      = bar_idx + 1
                reassess_time     = ts
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

    realized = signed_ret(exit_price, entry, direction) if exit_price is not None else 0.0

    # Counterfactual: no stop, no reassessment
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
    cf_ret = signed_ret(cf_exit, entry, direction) if cf_exit else None

    return {
        "exit_price": exit_price, "exit_time": exit_time, "exit_reason": exit_reason,
        "realized": realized, "mae": mae, "bars_held": bars_held,
        "reassess_triggered": reassess_triggered,
        "reassess_bar": reassess_bar, "reassess_time": reassess_time,
        "cf_ret": cf_ret,
    }


# ── Run one date ──────────────────────────────────────────────────────────────

def run_date(date_str: str, profiles: dict) -> list[ReassessedPositionV2]:
    stop_decisions = _eng.run_date(date_str, profiles)
    if not stop_decisions:
        return []

    ledger_targets = load_ledger_targets(date_str)
    positions: list[ReassessedPositionV2] = []

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

        result = walk_with_reassessment(
            direction, ref, target, sd.candidate_stop_price, sbars, start_bar=12
        )

        positions.append(ReassessedPositionV2(
            ticker=ticker, direction=direction, date=date_str,
            entry_price=ref, adaptive_target=target, adaptive_risk=sd.adaptive_risk,
            candidate_stop_pct=sd.candidate_stop_pct,
            candidate_stop_price=sd.candidate_stop_price,
            coverage_quality=sd.coverage_quality, n_obs=sd.n_obs,
            final_tier=sd.final_tier,
            exit_price=result["exit_price"],
            exit_time_ist=result["exit_time"],
            exit_reason=result["exit_reason"],
            realized_return=result["realized"],
            max_adverse_excursion=result["mae"],
            bars_held=result["bars_held"],
            reassess_triggered=result["reassess_triggered"],
            reassess_bar=result["reassess_bar"],
            reassess_time_ist=result["reassess_time"],
            h300_counterfactual_ret=result["cf_ret"],
        ))

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
    print(f"[reassess_v2] Profiles loaded: {len(profiles)}")
    print(f"[reassess_v2] Dates: {dates}")
    print(f"[reassess_v2] Downtrend window={DOWNTREND_WINDOW}  min_hold={MIN_HOLD_BARS}  lower_count={LOWER_COUNT_MIN}")
    print()

    all_positions: list[ReassessedPositionV2] = []

    for date_str in dates:
        positions = run_date(date_str, profiles)
        all_positions.extend(positions)
        if not positions:
            continue

        reassessed = [p for p in positions if p.reassess_triggered]
        stops      = [p for p in positions if p.exit_reason == "STOP"]
        horizons   = [p for p in positions if p.exit_reason == "HORIZON"]
        targets    = [p for p in positions if p.exit_reason == "TARGET"]
        rets       = [p.realized_return for p in positions if p.realized_return is not None]
        cf_rets    = [p.h300_counterfactual_ret for p in positions if p.h300_counterfactual_ret is not None]
        wins       = sum(1 for r in rets if r > 0)

        print(f"  {date_str}  N={len(positions)}  "
              f"STOP={len(stops)}  REASSESS={len(reassessed)}  TGT={len(targets)}  HOR={len(horizons)}  "
              f"win={wins/len(rets)*100:.0f}%  "
              f"mean={statistics.mean(rets)*100:+.3f}%  "
              f"total={sum(rets)*100:+.3f}%  "
              f"cf_total={sum(cf_rets)*100:+.3f}%")

        for p in reassessed:
            print(f"    REASSESS_EXIT: {p.ticker} {p.direction}  "
                  f"bar={p.reassess_bar}  t={p.reassess_time_ist}  "
                  f"ret={p.realized_return*100:+.3f}%  "
                  f"cf={p.h300_counterfactual_ret*100:+.3f}%  "
                  f"{'SAVED' if p.realized_return > p.h300_counterfactual_ret else 'COST'} "
                  f"{abs(p.realized_return - p.h300_counterfactual_ret)*100:.3f}pp")

        out_path = OUT_DIR / f"reassessment_v2_{date_str}.csv"
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
    if all_positions:
        print("=" * 100)
        print(f"Aggregate — Position Reassessment v0.2 (EXIT-only)  (N={len(all_positions)})")
        print("=" * 100)
        all_rets     = [p.realized_return for p in all_positions if p.realized_return is not None]
        all_cf       = [p.h300_counterfactual_ret for p in all_positions if p.h300_counterfactual_ret is not None]
        all_reassess = [p for p in all_positions if p.reassess_triggered]
        all_wins     = sum(1 for r in all_rets if r > 0)

        print(f"  Total positions:    {len(all_positions)}")
        print(f"  REASSESS_EXIT:      {len(all_reassess)}")
        print(f"  Win rate:           {all_wins/len(all_rets)*100:.0f}%")
        print(f"  Mean realized ret:  {statistics.mean(all_rets)*100:+.3f}%")
        print(f"  Total realized:     {sum(all_rets)*100:+.3f}%")
        print(f"  No-stop H300 cf:    {sum(all_cf)*100:+.3f}%")
        print(f"  v0.2 paper total:   +90.228%  (production baseline)")
        print(f"  vs v0.2 paper:      {sum(all_rets)*100 - 90.228:+.3f}pp")
        print(f"  vs no-stop:         {(sum(all_rets) - sum(all_cf))*100:+.3f}pp")
        print()

        if all_reassess:
            print("  REASSESS_EXIT trades:")
            for p in all_reassess:
                delta = (p.realized_return - p.h300_counterfactual_ret) * 100
                print(f"    {p.ticker:25} {p.direction}  {p.date}  "
                      f"ret={p.realized_return*100:+.3f}%  cf={p.h300_counterfactual_ret*100:+.3f}%  "
                      f"{'SAVED' if delta > 0 else 'COST'} {abs(delta):.3f}pp")
        print()

    print("[reassess_v2] Done. IC v1, Backtest v2, Cockpit, all prior components untouched.")


if __name__ == "__main__":
    run()