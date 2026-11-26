#!/usr/bin/env python3
"""
Paper Trader v0.1
==================
Executes the complete operational loop for a given trading date:

  Decision arrives (IC v1 ACT)
      ↓
  Symbol profile lookup (symbol_movement_profiler output)
      ↓
  Coverage / quality gate
      ↓
  Candidate stop from Stop Decision Engine (MAE_H300_p75)
      ↓
  Position opened at bar 13 (10:15 IST, H60 boundary)
      ↓
  1-minute bar monitoring from bar 13 onward
      ↓
  Stop / target / horizon event detection (in priority order)
      ↓
  Position lifecycle recorded

Event priority per bar:
  1. TARGET hit (high/low touches adaptive_target)
  2. STOP hit (high/low touches candidate_stop)
  3. HORIZON (bar 60 = H300 = ~14:15 IST)

The candidate stop is the engine's P75-based stop, not the adaptive_risk.
If coverage_quality == UNAVAILABLE, falls back to adaptive_risk.

Outputs per position:
  ticker, direction, entry_price, candidate_stop, adaptive_target,
  stop_confidence, coverage_quality, n_obs,
  exit_price, exit_time_ist, exit_reason,
  realized_return, max_adverse_excursion, max_favourable_excursion,
  bars_held, h300_counterfactual_ret

The h300_counterfactual_ret is always computed (no-stop replay) so the
consequence of the candidate stop can be evaluated against the no-stop path.

IC v1, Backtest v2, and Cockpit are NOT modified.

Usage:
  python3 scripts/paper_trader.py [--date YYYYMMDD]
  python3 scripts/paper_trader.py              # defaults to 2026-09-08
"""

import csv
import json
import sys
from pathlib import Path
from datetime import datetime, timezone, timedelta
from dataclasses import dataclass, asdict
from typing import Optional

# ── Paths ─────────────────────────────────────────────────────────────────────

REPO_ROOT   = Path(__file__).resolve().parent.parent
PROFILE_CSV = REPO_ROOT / "datasets" / "symbol_movement_profiles.csv"
LEDGER_DIR  = REPO_ROOT / "live_capture" / "ledger" / "entries"
BARS_DIR    = REPO_ROOT / "intraday_capture" / "yahoo_cache_1m"
OUT_DIR     = REPO_ROOT / "datasets"

IST = timezone(timedelta(hours=5, minutes=30))
USABLE_MIN_OBS = 5


# ── Profile loader ─────────────────────────────────────────────────────────────

def load_profiles(path: Path) -> dict[tuple[str, str], dict]:
    profiles: dict[tuple[str, str], dict] = {}
    if not path.exists():
        return profiles
    with open(path, newline="") as f:
        for row in csv.DictReader(f):
            def fv(col):
                try: return float(row[col])
                except: return None
            profiles[(row["ticker"], row["direction"])] = {
                "n_obs":         int(row["n_obs"]),
                "mae_h300_p50":  fv("mae_h300_p50"),
                "mae_h300_p75":  fv("mae_h300_p75"),
                "mae_h300_p90":  fv("mae_h300_p90"),
                "mae_h60_p50":   fv("mae_h60_p50"),
                "mfe_h300_p50":  fv("mfe_h300_p50"),
                "win_rate_h300": fv("win_rate_h300"),
            }
    return profiles


# ── Candidate stop from profile ───────────────────────────────────────────────

def candidate_stop_pct(profile: Optional[dict]) -> tuple[Optional[float], str, str]:
    """
    Returns (stop_pct, confidence, coverage_quality).
    stop_pct is negative (adverse direction from entry).
    """
    if profile is None:
        return None, "NONE", "UNAVAILABLE"
    n = profile["n_obs"]
    quality = "USABLE" if n >= USABLE_MIN_OBS else "LIMITED"
    confidence = "MEDIUM" if quality == "USABLE" else "LOW"
    p75 = profile.get("mae_h300_p75")
    if p75 is None or p75 >= 0:
        p50 = profile.get("mae_h300_p50")
        if p50 is None or p50 >= 0:
            return -0.005, confidence, quality  # 0.5% floor
        return p50, confidence, quality
    return p75, confidence, quality


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


# ── IC v1 frozen port ─────────────────────────────────────────────────────────

def signed_ret(close: float, entry: float, direction: str) -> float:
    r = (close - entry) / entry
    return r if direction == "LONG" else -r


def classify_h60(direction: str, h60_ret: Optional[float]) -> str:
    if h60_ret is None: return "WAIT"
    s = h60_ret if direction == "LONG" else -h60_ret
    return "ENTER" if s > 0.003 else ("WAIT" if s > -0.003 else "AVOID")


def compute_oqs(direction, h15, h60, mfe, mae, mp) -> int:
    sc = 0
    if h15 is not None:
        s = h15 if direction == "LONG" else -h15
        sc += 25 if s > 0.005 else (18 if s > 0.002 else (10 if s > 0 else (4 if s > -0.002 else 0)))
    if mfe is not None:
        sc += 25 if mfe > 0.010 else (18 if mfe > 0.005 else (12 if mfe > 0.002 else (6 if mfe > 0 else 0)))
    if mp is not None:
        sc += 25 if mp >= 0.75 else (18 if mp >= 0.60 else (10 if mp >= 0.45 else (4 if mp >= 0.30 else 0)))
    if mae is not None:
        a = abs(mae)
        sc += 25 if a < 0.002 else (18 if a < 0.005 else (10 if a < 0.010 else (4 if a < 0.015 else 0)))
    return min(sc, 100)


def classify_entry(direction, q, h60c) -> str:
    if direction == "SHORT":
        if h60c == "ENTER": return "ENTER" if q >= 70 else ("WAIT-HIGH" if q >= 50 else ("WAIT-MID" if q >= 30 else "WAIT-LOW"))
        elif h60c == "WAIT": return "WAIT-HIGH" if q >= 50 else ("WAIT-MID" if q >= 30 else "WAIT-LOW")
        else: return "AVOID"
    else:
        if h60c == "ENTER": return "ENTER" if q >= 70 else ("WAIT-HIGH" if q >= 50 else ("WAIT-MID" if q >= 30 else "WAIT-LOW"))
        elif h60c == "WAIT": return "WAIT-HIGH" if q >= 70 else ("WAIT-MID" if q >= 50 else ("WAIT-LOW" if q >= 30 else "AVOID"))
        else: return "AVOID"


def entry_action(state: str) -> str:
    return "ACT" if state in ("ENTER","WAIT-HIGH","ENTER-LATE","WAIT-LATE") else ("MONITOR" if state in ("WAIT-MID","WAIT-LOW") else "AVOID")


# ── Position lifecycle ────────────────────────────────────────────────────────

@dataclass
class PositionLifecycle:
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
    # Exit
    exit_price: Optional[float]
    exit_time_ist: Optional[str]
    exit_reason: str          # TARGET | STOP | HORIZON
    realized_return: Optional[float]
    max_adverse_excursion: float
    max_favourable_excursion: float
    bars_held: int
    # Counterfactual: no-stop H300 return
    h300_counterfactual_ret: Optional[float]
    # Consequence: cost/saving of candidate stop vs no-stop
    stop_consequence_pp: Optional[float]  # realized - counterfactual (negative = stop cost money)


def walk_position(direction: str, entry: float, target: float, stop_price: Optional[float],
                  sbars: list[dict], start_bar: int = 12
                  ) -> tuple[float, str, str, float, float, int]:
    """
    Walk forward from start_bar.
    Returns (exit_price, exit_time_ist, exit_reason, mae, mfe, bars_held).
    exit_reason: TARGET | STOP | HORIZON
    If stop_price is None, only TARGET and HORIZON are checked.
    """
    mae = 0.0
    mfe = 0.0
    bars_held = 0

    for i, bar in enumerate(sbars[start_bar:]):
        bars_held += 1
        low, high = bar["low"], bar["high"]
        ts = datetime.fromtimestamp(bar["timestamp"], tz=IST).strftime("%H:%M")

        # Track MAE/MFE
        if direction == "LONG":
            mae = min(mae, (low  - entry) / entry)
            mfe = max(mfe, (high - entry) / entry)
        else:
            mae = min(mae, (entry - high) / entry)
            mfe = max(mfe, (entry - low)  / entry)

        # Event detection: TARGET first, then STOP
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

        # H300 horizon = bar 60 (index 59 from session start)
        if start_bar + i >= 59:
            break

    # Horizon exit
    horizon_bar = sbars[min(59, len(sbars) - 1)] if sbars else None
    if horizon_bar:
        ts = datetime.fromtimestamp(horizon_bar["timestamp"], tz=IST).strftime("%H:%M")
        return horizon_bar["close"], ts, "HORIZON", mae, mfe, bars_held
    return entry, "15:14", "HORIZON", mae, mfe, bars_held


def run_paper_trader(date_str: str, profiles: dict) -> list[PositionLifecycle]:
    """
    Run the paper trader for a single date.
    Returns list of PositionLifecycle records for all ACT decisions.
    """
    ledger_pattern = f"LIVE-005-{date_str}-*.json"
    entries = sorted(LEDGER_DIR.glob(ledger_pattern))
    if not entries:
        print(f"[paper_trader] No ledger entries for {date_str}")
        return []

    positions: list[PositionLifecycle] = []

    for ep in entries:
        e = json.load(open(ep))
        ticker    = e.get("ticker", "")
        direction = e.get("direction", "")
        ref       = e.get("reference_price")
        target    = e.get("adaptive_target")
        risk      = e.get("adaptive_risk")
        if None in (ref, target, risk):
            continue

        sbars = get_session_bars(ticker, date_str)
        if len(sbars) < 12:
            continue

        # IC v1 classification at H60 boundary
        def ret(c): return signed_ret(c, ref, direction)
        h60b = sbars[:12]
        h15r = ret(sbars[2]["close"]) if len(sbars) >= 3 else None
        h60r = ret(sbars[11]["close"]) if len(sbars) >= 12 else None
        mfeh = max(ret(b["close"]) for b in h60b) if h60b else None
        maeh = min(ret(b["close"]) for b in h60b) if h60b else None
        mpp  = sum(1 for b in h60b if ret(b["close"]) > 0) / len(h60b) if h60b else None

        h60c  = classify_h60(direction, h60r)
        q     = compute_oqs(direction, h15r, h60r, mfeh, maeh, mpp)
        state = classify_entry(direction, q, h60c)
        if entry_action(state) != "ACT":
            continue

        # Candidate stop from profile
        profile = profiles.get((ticker, direction))
        cand_pct, confidence, quality = candidate_stop_pct(profile)
        n_obs = profile["n_obs"] if profile else 0

        if cand_pct is not None:
            if direction == "LONG":
                cand_price = ref * (1 + cand_pct)
            else:
                cand_price = ref * (1 - abs(cand_pct))
        else:
            # Fallback to adaptive_risk
            cand_price = risk
            cand_pct   = (risk - ref) / ref if direction == "LONG" else -(abs(risk - ref) / ref)

        # Walk position with candidate stop
        exit_price, exit_time, exit_reason, mae, mfe, bars_held = walk_position(
            direction, ref, target, cand_price, sbars, start_bar=12
        )

        realized_ret = None
        if exit_price is not None:
            realized_ret = signed_ret(exit_price, ref, direction)

        # Counterfactual: no-stop H300 (target or horizon only)
        cf_exit, _cf_time, _cf_reason, _cf_mae, _cf_mfe, _cf_bars = walk_position(
            direction, ref, target, None, sbars, start_bar=12
        )
        cf_ret = signed_ret(cf_exit, ref, direction) if cf_exit is not None else None

        # Consequence: how much did the stop cost/save vs no-stop?
        stop_consequence = None
        if realized_ret is not None and cf_ret is not None:
            stop_consequence = realized_ret - cf_ret  # negative = stop cost money

        positions.append(PositionLifecycle(
            ticker=ticker,
            direction=direction,
            date=date_str,
            entry_price=ref,
            adaptive_target=target,
            adaptive_risk=risk,
            candidate_stop_pct=cand_pct,
            candidate_stop_price=round(cand_price, 2),
            stop_confidence=confidence,
            coverage_quality=quality,
            n_obs=n_obs,
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


# ── Main ──────────────────────────────────────────────────────────────────────

def run() -> None:
    # Parse date from argv or default to 8-Sep
    date_str = "20260908"
    for arg in sys.argv[1:]:
        if arg.startswith("--date="):
            date_str = arg.split("=")[1]
        elif arg == "--date" and len(sys.argv) > sys.argv.index(arg) + 1:
            date_str = sys.argv[sys.argv.index(arg) + 1]

    profiles = load_profiles(PROFILE_CSV)
    print(f"[paper_trader] Profiles loaded: {len(profiles)}")
    print(f"[paper_trader] Running paper trader for date: {date_str}")
    print()

    positions = run_paper_trader(date_str, profiles)
    print(f"[paper_trader] Positions opened: {len(positions)}")
    print()

    if not positions:
        print("[paper_trader] No positions to report.")
        return

    # ── Print position lifecycle table ────────────────────────────────────────
    print("=" * 130)
    print(f"Paper Trader v0.1 — {date_str}  (N={len(positions)})")
    print("=" * 130)
    print(f"{'ticker':25} {'dir':5} {'qual':11} {'conf':6}  "
          f"{'cand_stop':>10}  {'exit_time':>9}  {'exit':>7}  "
          f"{'ret':>8}  {'cf_ret':>8}  {'consequence':>12}  {'MAE':>8}  {'bars':>4}")
    print("-" * 130)

    stops   = [p for p in positions if p.exit_reason == "STOP"]
    targets = [p for p in positions if p.exit_reason == "TARGET"]
    horizons= [p for p in positions if p.exit_reason == "HORIZON"]

    for p in sorted(positions, key=lambda x: x.ticker):
        cand_str = f"{p.candidate_stop_pct*100:+.3f}%" if p.candidate_stop_pct is not None else "   N/A"
        ret_str  = f"{p.realized_return*100:+.3f}%"    if p.realized_return is not None else "   N/A"
        cf_str   = f"{p.h300_counterfactual_ret*100:+.3f}%" if p.h300_counterfactual_ret is not None else "   N/A"
        cons_str = f"{p.stop_consequence_pp*100:+.3f}pp" if p.stop_consequence_pp is not None else "   N/A"
        mae_str  = f"{p.max_adverse_excursion*100:+.3f}%"

        marker = " ◄" if p.ticker == "JUBLFOOD_NS" else ""

        print(f"{p.ticker:25} {p.direction:5} {p.coverage_quality:11} {p.stop_confidence:6}  "
              f"{cand_str:>10}  {p.exit_time_ist:>9}  {p.exit_reason:>7}  "
              f"{ret_str:>8}  {cf_str:>8}  {cons_str:>12}  {mae_str:>8}  {p.bars_held:>4}{marker}")

    print("-" * 130)

    # Summary
    import statistics
    rets = [p.realized_return for p in positions if p.realized_return is not None]
    cfs  = [p.h300_counterfactual_ret for p in positions if p.h300_counterfactual_ret is not None]
    cons = [p.stop_consequence_pp for p in positions if p.stop_consequence_pp is not None]
    wins = [r for r in rets if r > 0]

    print(f"  Exits: STOP={len(stops)}  TARGET={len(targets)}  HORIZON={len(horizons)}")
    if rets:
        print(f"  Realized:      mean={statistics.mean(rets)*100:+.3f}%  "
              f"median={statistics.median(rets)*100:+.3f}%  "
              f"win={len(wins)/len(rets)*100:.0f}%  "
              f"total={sum(rets)*100:+.3f}%")
    if cfs:
        cf_wins = [r for r in cfs if r > 0]
        print(f"  No-stop H300:  mean={statistics.mean(cfs)*100:+.3f}%  "
              f"median={statistics.median(cfs)*100:+.3f}%  "
              f"win={len(cf_wins)/len(cfs)*100:.0f}%  "
              f"total={sum(cfs)*100:+.3f}%")
    if cons:
        print(f"  Stop consequence (realized - no-stop): "
              f"mean={statistics.mean(cons)*100:+.3f}pp  "
              f"total={sum(cons)*100:+.3f}pp")
    print()

    # JUBLFOOD case study
    jublfood = next((p for p in positions if p.ticker == "JUBLFOOD_NS"), None)
    if jublfood:
        print("JUBLFOOD_NS SHORT — position lifecycle:")
        print(f"  entry:          ₹{jublfood.entry_price:.2f}")
        print(f"  candidate_stop: ₹{jublfood.candidate_stop_price:.2f}  ({jublfood.candidate_stop_pct*100:+.3f}%)")
        print(f"  adaptive_risk:  ₹{jublfood.adaptive_risk:.2f}")
        print(f"  confidence:     {jublfood.stop_confidence}  ({jublfood.coverage_quality}, n={jublfood.n_obs})")
        print(f"  exit_time:      {jublfood.exit_time_ist}  ({jublfood.exit_reason})")
        print(f"  exit_price:     ₹{jublfood.exit_price:.2f}")
        print(f"  realized_ret:   {jublfood.realized_return*100:+.3f}%")
        print(f"  h300_cf_ret:    {jublfood.h300_counterfactual_ret*100:+.3f}%")
        print(f"  consequence:    {jublfood.stop_consequence_pp*100:+.3f}pp  "
              f"({'stop cost money' if jublfood.stop_consequence_pp < 0 else 'stop saved money'})")
        print(f"  MAE:            {jublfood.max_adverse_excursion*100:+.3f}%")
        print(f"  bars_held:      {jublfood.bars_held}")
        print()

    # Write CSV
    out_path = OUT_DIR / f"paper_trader_{date_str}.csv"
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    fields = list(asdict(positions[0]).keys())
    with open(out_path, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=fields)
        w.writeheader()
        for p in positions:
            row = asdict(p)
            w.writerow({k: (f"{v:.6f}" if isinstance(v, float) else str(v))
                         for k, v in row.items()})
    print(f"[paper_trader] Output CSV: {out_path}")
    print()
    print("[paper_trader] Done. IC v1, Backtest v2, and Cockpit are untouched.")


if __name__ == "__main__":
    run()