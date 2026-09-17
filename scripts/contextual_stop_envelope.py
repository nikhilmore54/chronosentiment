#!/usr/bin/env python3
"""
Contextual Stop Envelope v0.1
==============================
Consumes the Symbol Movement Profile produced by symbol_movement_profiler.py
and, for a given (symbol, direction), exposes the historical adverse-movement
envelope with an explicit coverage/quality gate.

This component answers ONE question:

    "What adverse movement does this symbol historically tolerate
     for this direction and horizon?"

It does NOT:
  - optimize a stop value
  - modify IC v1
  - modify Backtest v2
  - modify the Cockpit
  - use any future outcome information

Coverage gate:
  UNAVAILABLE  — no profile exists for this (symbol, direction)
  LIMITED      — n_obs < 5 (percentiles are directionally informative only)
  USABLE       — n_obs >= 5 (percentiles are reasonably stable)

Output per lookup:
  symbol
  direction
  n_obs
  coverage_quality        UNAVAILABLE | LIMITED | USABLE
  mae_h60_p50             typical adverse excursion by H60
  mae_h60_p75             upper-quartile adverse excursion by H60
  mae_h300_p50            typical adverse excursion by H300
  mae_h300_p75            upper-quartile adverse excursion by H300
  mae_h300_p90            90th-percentile adverse excursion by H300
  mfe_h300_p50            typical favourable excursion by H300
  win_rate_h300           historical win rate at H300
  recovery_rate           fraction of observations that recovered to positive
  adaptive_risk_pct       adaptive_risk distance as % of entry (from decision)
  profile_vs_adaptive     how the adaptive_risk compares to the MAE envelope

The script also runs a batch evaluation over all 8-Sep ACT decisions,
showing the envelope for each, and highlights the JUBLFOOD case study.

IC v1, Backtest v2, and Cockpit are NOT modified.

Usage:
  python3 scripts/contextual_stop_envelope.py
"""

import csv
import json
from pathlib import Path
from dataclasses import dataclass
from typing import Optional

# ── Paths ─────────────────────────────────────────────────────────────────────

REPO_ROOT    = Path(__file__).resolve().parent.parent
PROFILE_CSV  = REPO_ROOT / "datasets" / "symbol_movement_profiles.csv"
LEDGER_DIR   = REPO_ROOT / "live_capture" / "ledger" / "entries"

# ── Coverage thresholds ───────────────────────────────────────────────────────

USABLE_MIN_OBS   = 5   # n_obs >= 5 → USABLE
LIMITED_MIN_OBS  = 1   # n_obs >= 1 → LIMITED; 0 → UNAVAILABLE


# ── Profile store ─────────────────────────────────────────────────────────────

@dataclass
class MovementProfile:
    ticker: str
    direction: str
    n_obs: int
    coverage_quality: str          # UNAVAILABLE | LIMITED | USABLE
    mae_h60_p50:  Optional[float]
    mae_h60_p75:  Optional[float]
    mae_h300_p25: Optional[float]
    mae_h300_p50: Optional[float]
    mae_h300_p75: Optional[float]
    mae_h300_p90: Optional[float]
    mae_h300_p95: Optional[float]
    mfe_h300_p50: Optional[float]
    win_rate_h300: Optional[float]
    recovery_rate: Optional[float]


def load_profiles(path: Path) -> dict[tuple[str, str], MovementProfile]:
    profiles: dict[tuple[str, str], MovementProfile] = {}
    if not path.exists():
        print(f"[envelope] WARNING: profile CSV not found at {path}")
        return profiles

    with open(path, newline="") as f:
        reader = csv.DictReader(f)
        for row in reader:
            n = int(row["n_obs"])
            if n >= USABLE_MIN_OBS:
                quality = "USABLE"
            elif n >= LIMITED_MIN_OBS:
                quality = "LIMITED"
            else:
                quality = "UNAVAILABLE"

            def fv(col: str) -> Optional[float]:
                v = row.get(col, "")
                try:
                    return float(v)
                except (ValueError, TypeError):
                    return None

            ticker    = row["ticker"]
            direction = row["direction"]
            profiles[(ticker, direction)] = MovementProfile(
                ticker=ticker,
                direction=direction,
                n_obs=n,
                coverage_quality=quality,
                mae_h60_p50=fv("mae_h60_p50"),
                mae_h60_p75=fv("mae_h60_p75"),
                mae_h300_p25=fv("mae_h300_p25"),
                mae_h300_p50=fv("mae_h300_p50"),
                mae_h300_p75=fv("mae_h300_p75"),
                mae_h300_p90=fv("mae_h300_p90"),
                mae_h300_p95=fv("mae_h300_p95"),
                mfe_h300_p50=fv("mfe_h300_p50"),
                win_rate_h300=fv("win_rate_h300"),
                recovery_rate=fv("recovery_rate"),
            )
    return profiles


# ── Envelope lookup ───────────────────────────────────────────────────────────

def get_envelope(profiles: dict, ticker: str, direction: str,
                 entry_price: Optional[float] = None,
                 adaptive_risk: Optional[float] = None) -> dict:
    """
    Return the contextual stop envelope for a (ticker, direction) pair.
    Optionally annotates with adaptive_risk distance for comparison.
    """
    key = (ticker, direction)
    if key not in profiles:
        return {
            "ticker": ticker,
            "direction": direction,
            "n_obs": 0,
            "coverage_quality": "UNAVAILABLE",
            "mae_h60_p50": None,
            "mae_h60_p75": None,
            "mae_h300_p50": None,
            "mae_h300_p75": None,
            "mae_h300_p90": None,
            "mfe_h300_p50": None,
            "win_rate_h300": None,
            "recovery_rate": None,
            "adaptive_risk_pct": None,
            "profile_vs_adaptive": "NO_PROFILE",
        }

    p = profiles[key]

    # Compute adaptive_risk distance as % of entry
    adaptive_risk_pct = None
    profile_vs_adaptive = "UNKNOWN"
    if entry_price is not None and adaptive_risk is not None and entry_price > 0:
        if direction == "LONG":
            adaptive_risk_pct = (adaptive_risk - entry_price) / entry_price  # negative
        else:
            adaptive_risk_pct = (entry_price - adaptive_risk) / entry_price  # negative (distance)
            adaptive_risk_pct = -abs(adaptive_risk_pct)

        # Compare adaptive_risk distance to MAE envelope
        if p.mae_h300_p50 is not None and adaptive_risk_pct is not None:
            # Both are negative (adverse direction); more negative = wider stop
            if adaptive_risk_pct < p.mae_h300_p90:
                profile_vs_adaptive = "WIDER_THAN_P90"   # stop is beyond 90th pct MAE
            elif adaptive_risk_pct < p.mae_h300_p75:
                profile_vs_adaptive = "WIDER_THAN_P75"
            elif adaptive_risk_pct < p.mae_h300_p50:
                profile_vs_adaptive = "WIDER_THAN_P50"
            elif adaptive_risk_pct < 0:
                profile_vs_adaptive = "INSIDE_P50"       # stop is tighter than typical MAE
            else:
                profile_vs_adaptive = "AT_ENTRY"

    return {
        "ticker": ticker,
        "direction": direction,
        "n_obs": p.n_obs,
        "coverage_quality": p.coverage_quality,
        "mae_h60_p50": p.mae_h60_p50,
        "mae_h60_p75": p.mae_h60_p75,
        "mae_h300_p50": p.mae_h300_p50,
        "mae_h300_p75": p.mae_h300_p75,
        "mae_h300_p90": p.mae_h300_p90,
        "mfe_h300_p50": p.mfe_h300_p50,
        "win_rate_h300": p.win_rate_h300,
        "recovery_rate": p.recovery_rate,
        "adaptive_risk_pct": adaptive_risk_pct,
        "profile_vs_adaptive": profile_vs_adaptive,
    }


# ── 8-Sep ACT decisions ───────────────────────────────────────────────────────

def load_sep8_act_decisions() -> list[dict]:
    """
    Re-run the same ACT filter used in p4_sep8_replay.py / p4_sep8_stop_sensitivity.py
    to get the 31 ACT decisions for 8-Sep-2026.
    Returns list of dicts with ticker, direction, entry_price, adaptive_risk, adaptive_target.
    """
    from datetime import datetime, timezone, timedelta

    IST = timezone(timedelta(hours=5, minutes=30))
    BARS_DIR = REPO_ROOT / "intraday_capture" / "yahoo_cache_1m"

    def load_bars(ticker_ns: str) -> list[dict]:
        filename = ticker_ns.replace("_NS", ".NS") + ".json"
        path = BARS_DIR / filename
        if not path.exists():
            return []
        with open(path) as f:
            bars = json.load(f)
        bars.sort(key=lambda b: b["timestamp"])
        return bars

    OPEN_IST  = datetime(2026, 9, 8, 9, 15, 0, tzinfo=IST)
    CLOSE_IST = datetime(2026, 9, 8, 15, 30, 0, tzinfo=IST)
    open_ts   = int(OPEN_IST.timestamp())
    close_ts  = int(CLOSE_IST.timestamp())

    def signed_ret(close, entry, direction):
        raw = (close - entry) / entry
        return raw if direction == "LONG" else -raw

    def classify_h60(direction, h60_ret):
        if h60_ret is None: return "WAIT"
        signed = h60_ret if direction == "LONG" else -h60_ret
        if signed > 0.003: return "ENTER"
        elif signed > -0.003: return "WAIT"
        else: return "AVOID"

    def compute_oqs(direction, h15_ret, h60_ret, mfe_h60, mae_h60, mp):
        score = 0
        if h15_ret is not None:
            s = h15_ret if direction == "LONG" else -h15_ret
            if s > 0.005: score += 25
            elif s > 0.002: score += 18
            elif s > 0.0: score += 10
            elif s > -0.002: score += 4
        if mfe_h60 is not None:
            if mfe_h60 > 0.010: score += 25
            elif mfe_h60 > 0.005: score += 18
            elif mfe_h60 > 0.002: score += 12
            elif mfe_h60 > 0.0: score += 6
        if mp is not None:
            if mp >= 0.75: score += 25
            elif mp >= 0.60: score += 18
            elif mp >= 0.45: score += 10
            elif mp >= 0.30: score += 4
        if mae_h60 is not None:
            a = abs(mae_h60)
            if a < 0.002: score += 25
            elif a < 0.005: score += 18
            elif a < 0.010: score += 10
            elif a < 0.015: score += 4
        return min(score, 100)

    def classify_at_entry(direction, oqs, h60_class):
        if direction == "SHORT":
            if h60_class == "ENTER":
                if oqs >= 70: return "ENTER"
                elif oqs >= 50: return "WAIT-HIGH"
                elif oqs >= 30: return "WAIT-MID"
                else: return "WAIT-LOW"
            elif h60_class == "WAIT":
                if oqs >= 50: return "WAIT-HIGH"
                elif oqs >= 30: return "WAIT-MID"
                else: return "WAIT-LOW"
            else: return "AVOID"
        else:
            if h60_class == "ENTER":
                if oqs >= 70: return "ENTER"
                elif oqs >= 50: return "WAIT-HIGH"
                elif oqs >= 30: return "WAIT-MID"
                else: return "WAIT-LOW"
            elif h60_class == "WAIT":
                if oqs >= 70: return "WAIT-HIGH"
                elif oqs >= 50: return "WAIT-MID"
                elif oqs >= 30: return "WAIT-LOW"
                else: return "AVOID"
            else: return "AVOID"

    def entry_action(state):
        if state in ("ENTER", "WAIT-HIGH", "ENTER-LATE", "WAIT-LATE"): return "ACT"
        elif state in ("WAIT-MID", "WAIT-LOW"): return "MONITOR"
        else: return "AVOID"

    act_records = []
    for ep in sorted(LEDGER_DIR.glob("LIVE-005-20260908-*.json")):
        e = json.load(open(ep))
        ticker    = e.get("ticker", "")
        direction = e.get("direction", "")
        ref_price = e.get("reference_price")
        target    = e.get("adaptive_target")
        risk      = e.get("adaptive_risk")
        if ref_price is None or target is None or risk is None:
            continue

        all_bars = load_bars(ticker)
        sbars = [b for b in all_bars if open_ts <= b["timestamp"] <= close_ts]
        if len(sbars) < 12:
            continue

        def ret(close): return signed_ret(close, ref_price, direction)
        h60_bars = sbars[:12]
        h15_ret  = ret(sbars[2]["close"]) if len(sbars) >= 3 else None
        h60_ret  = ret(sbars[11]["close"]) if len(sbars) >= 12 else None
        mfe_h60  = max(ret(b["close"]) for b in h60_bars) if h60_bars else None
        mae_h60  = min(ret(b["close"]) for b in h60_bars) if h60_bars else None
        mp       = sum(1 for b in h60_bars if ret(b["close"]) > 0) / len(h60_bars) if h60_bars else None

        h60_cls = classify_h60(direction, h60_ret)
        oqs     = compute_oqs(direction, h15_ret, h60_ret, mfe_h60, mae_h60, mp)
        state   = classify_at_entry(direction, oqs, h60_cls)
        action  = entry_action(state)

        if action != "ACT":
            continue

        act_records.append({
            "ticker": ticker,
            "direction": direction,
            "entry_price": ref_price,
            "adaptive_target": target,
            "adaptive_risk": risk,
        })

    return act_records


# ── Main ──────────────────────────────────────────────────────────────────────

def run() -> None:
    profiles = load_profiles(PROFILE_CSV)
    print(f"[envelope] Profiles loaded: {len(profiles)}")
    print()

    act_decisions = load_sep8_act_decisions()
    print(f"[envelope] 8-Sep ACT decisions: {len(act_decisions)}")
    print()

    # ── Batch envelope table ──────────────────────────────────────────────────
    print("=" * 110)
    print("Contextual Stop Envelope — 8-Sep-2026 ACT decisions")
    print("=" * 110)
    print(f"{'ticker':25} {'dir':5} {'n':>3}  {'qual':11}  "
          f"{'MAE_H60_p50':>12} {'MAE_H300_p50':>13} {'MAE_H300_p75':>13} {'MAE_H300_p90':>13}  "
          f"{'AR_pct':>8}  {'vs_profile':>18}")
    print("-" * 110)

    quality_counts = {"USABLE": 0, "LIMITED": 0, "UNAVAILABLE": 0}
    inside_p50_count = 0

    for dec in sorted(act_decisions, key=lambda d: d["ticker"]):
        env = get_envelope(profiles, dec["ticker"], dec["direction"],
                           dec["entry_price"], dec["adaptive_risk"])

        q = env["coverage_quality"]
        quality_counts[q] = quality_counts.get(q, 0) + 1

        def fmt(v): return f"{v*100:+.3f}%" if v is not None else "     N/A"

        ar_str = fmt(env["adaptive_risk_pct"])
        vs     = env["profile_vs_adaptive"]

        if vs == "INSIDE_P50":
            inside_p50_count += 1

        marker = " ◄" if dec["ticker"] == "JUBLFOOD_NS" else ""

        print(f"{dec['ticker']:25} {dec['direction']:5} {env['n_obs']:>3}  "
              f"{q:11}  "
              f"{fmt(env['mae_h60_p50']):>12} {fmt(env['mae_h300_p50']):>13} "
              f"{fmt(env['mae_h300_p75']):>13} {fmt(env['mae_h300_p90']):>13}  "
              f"{ar_str:>8}  {vs:>18}{marker}")

    print("-" * 110)
    print(f"  Coverage: USABLE={quality_counts['USABLE']}  "
          f"LIMITED={quality_counts['LIMITED']}  "
          f"UNAVAILABLE={quality_counts['UNAVAILABLE']}")
    print(f"  adaptive_risk inside P50 MAE envelope: {inside_p50_count} decisions")
    print()

    # ── JUBLFOOD case study ───────────────────────────────────────────────────
    jublfood = next((d for d in act_decisions if d["ticker"] == "JUBLFOOD_NS"), None)
    if jublfood:
        env = get_envelope(profiles, "JUBLFOOD_NS", "SHORT",
                           jublfood["entry_price"], jublfood["adaptive_risk"])
        print("JUBLFOOD_NS SHORT — full envelope:")
        print(f"  n_obs:              {env['n_obs']}")
        print(f"  coverage_quality:   {env['coverage_quality']}")
        print(f"  MAE_H60  p50:       {env['mae_h60_p50']*100:+.3f}%")
        print(f"  MAE_H60  p75:       {env['mae_h60_p75']*100:+.3f}%")
        print(f"  MAE_H300 p50:       {env['mae_h300_p50']*100:+.3f}%")
        print(f"  MAE_H300 p75:       {env['mae_h300_p75']*100:+.3f}%")
        print(f"  MAE_H300 p90:       {env['mae_h300_p90']*100:+.3f}%")
        print(f"  MFE_H300 p50:       {env['mfe_h300_p50']*100:+.3f}%")
        print(f"  win_rate_h300:      {env['win_rate_h300']*100:.0f}%")
        print(f"  recovery_rate:      {env['recovery_rate']*100:.0f}%")
        print()
        print(f"  adaptive_risk_pct:  {env['adaptive_risk_pct']*100:+.3f}%")
        print(f"  profile_vs_adaptive:{env['profile_vs_adaptive']}")
        print()
        print("  8-Sep event summary:")
        print(f"    entry=472.80  adaptive_risk=464.80  AR_dist={env['adaptive_risk_pct']*100:+.3f}%")
        print(f"    0.50× stop at -0.846% → triggered at -1.689%")
        print(f"    H300 counterfactual: -0.296%  (cost of premature stop: +1.393 pp)")
        print()
        print(f"  Interpretation:")
        ar = env["adaptive_risk_pct"]
        p50 = env["mae_h300_p50"]
        p75 = env["mae_h300_p75"]
        if ar is not None and p50 is not None:
            if ar < p50:
                print(f"    adaptive_risk ({ar*100:+.3f}%) is WIDER than historical MAE p50 ({p50*100:+.3f}%).")
                print(f"    The baseline stop is beyond the typical adverse excursion for this symbol/direction.")
            else:
                print(f"    adaptive_risk ({ar*100:+.3f}%) is INSIDE the historical MAE p50 ({p50*100:+.3f}%).")
                print(f"    The baseline stop is tighter than the typical adverse excursion — elevated stop-out risk.")

    print()
    print("[envelope] Done. IC v1, Backtest v2, and Cockpit are untouched.")


if __name__ == "__main__":
    run()