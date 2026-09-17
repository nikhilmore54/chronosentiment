#!/usr/bin/env python3
"""
Stop Decision Engine v0.2
==========================
Improves on v0.1 (universal MAE_H300_p75) by making the stop decision
genuinely contextual using three additional inputs:

1. Direction asymmetry
   LONG and SHORT symbols do not behave symmetrically. The profiler
   already stores separate profiles per direction. v0.2 uses the
   direction-specific percentile rather than a universal one.

2. Profile quality / n_obs gate
   v0.1 used P75 regardless of n_obs. v0.2 applies a tiered policy:
     USABLE  (n >= 5): use MAE_H300_p75
     LIMITED (n 2-4):  use MAE_H300_p50 (more conservative — less tail trust)
     SPARSE  (n == 1): use MAE_H60_p50  (only H60 data is meaningful)
     UNAVAILABLE:      fall back to adaptive_risk

3. Current H60 path context
   If the first hour (bars 1-12) already shows adverse movement beyond
   the symbol's historical MAE_H60_p50, the position is already in
   unusual territory. In that case, widen the stop by one tier to avoid
   stopping out on noise that is already within the observed envelope.
   Conversely, if H60 is strongly favourable, tighten by one tier to
   protect the early gain.

   Path context tiers:
     STRONG_FAVOUR  h60_ret > mfe_h300_p50 * 0.5  → tighten one tier
     NORMAL         otherwise                       → use base tier
     EARLY_ADVERSE  h60_ret < mae_h60_p50           → widen one tier

Tier ladder (from tightest to widest):
  mae_h60_p50 → mae_h300_p50 → mae_h300_p75 → mae_h300_p90 → adaptive_risk

The paper trader (v0.1) is NOT modified. This script produces a
stop_decisions_v2_{date}.csv that the paper trader can consume via
its existing interface.

IC v1, Backtest v2, Cockpit, Symbol Movement Profiler, Contextual Stop
Envelope, and Paper Trader v0.1 are NOT modified.

Usage:
  python3 scripts/stop_decision_engine_v2.py [--date YYYYMMDD]
  python3 scripts/stop_decision_engine_v2.py   # defaults to 2026-09-08
"""

import csv
import json
import sys
import statistics
from pathlib import Path
from dataclasses import dataclass, asdict
from typing import Optional
from datetime import datetime, timezone, timedelta

# ── Paths ─────────────────────────────────────────────────────────────────────

REPO_ROOT   = Path(__file__).resolve().parent.parent
PROFILE_CSV = REPO_ROOT / "datasets" / "symbol_movement_profiles.csv"
LEDGER_DIR  = REPO_ROOT / "live_capture" / "ledger" / "entries"
BARS_DIR    = REPO_ROOT / "intraday_capture" / "yahoo_cache_1m"
OUT_DIR     = REPO_ROOT / "datasets"

IST = timezone(timedelta(hours=5, minutes=30))

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
                "mae_h60_p50":   fv("mae_h60_p50"),
                "mae_h60_p75":   fv("mae_h60_p75"),
                "mae_h300_p50":  fv("mae_h300_p50"),
                "mae_h300_p75":  fv("mae_h300_p75"),
                "mae_h300_p90":  fv("mae_h300_p90"),
                "mfe_h300_p50":  fv("mfe_h300_p50"),
                "win_rate_h300": fv("win_rate_h300"),
            }
    return profiles


# ── Tier ladder ───────────────────────────────────────────────────────────────

TIER_NAMES = ["mae_h60_p50", "mae_h300_p50", "mae_h300_p75", "mae_h300_p90", "adaptive_risk"]


def get_tier_value(profile: dict, tier_name: str, adaptive_risk_pct: float) -> Optional[float]:
    if tier_name == "adaptive_risk":
        return adaptive_risk_pct
    v = profile.get(tier_name)
    if v is None or v >= 0:
        return None  # non-adverse — caller handles fallback
    return v


def select_base_tier(n_obs: int) -> str:
    """Select base tier from profile quality."""
    if n_obs >= 5:
        return "mae_h300_p75"   # USABLE
    elif n_obs >= 2:
        return "mae_h300_p50"   # LIMITED
    elif n_obs == 1:
        return "mae_h60_p50"    # SPARSE
    else:
        return "adaptive_risk"  # UNAVAILABLE


def adjust_tier(base_tier: str, context: str) -> str:
    """Shift tier up (tighter) or down (wider) based on path context."""
    idx = TIER_NAMES.index(base_tier) if base_tier in TIER_NAMES else len(TIER_NAMES) - 1
    if context == "STRONG_FAVOUR":
        idx = max(0, idx - 1)   # tighten one tier
    elif context == "EARLY_ADVERSE":
        idx = min(len(TIER_NAMES) - 1, idx + 1)  # widen one tier
    return TIER_NAMES[idx]


def path_context(direction: str, h60_ret: Optional[float],
                 profile: Optional[dict]) -> str:
    """Classify the H60 path context relative to the symbol's historical profile."""
    if h60_ret is None or profile is None:
        return "NORMAL"
    mae_h60_p50  = profile.get("mae_h60_p50")
    mfe_h300_p50 = profile.get("mfe_h300_p50")

    # h60_ret is already direction-adjusted (positive = favourable)
    if mfe_h300_p50 is not None and mfe_h300_p50 > 0:
        if h60_ret > mfe_h300_p50 * 0.5:
            return "STRONG_FAVOUR"
    if mae_h60_p50 is not None and mae_h60_p50 < 0:
        if h60_ret < mae_h60_p50:
            return "EARLY_ADVERSE"
    return "NORMAL"


# ── Stop decision ─────────────────────────────────────────────────────────────

@dataclass
class StopDecisionV2:
    ticker: str
    direction: str
    date: str
    entry_price: float
    adaptive_risk: float
    adaptive_risk_pct: float
    n_obs: int
    coverage_quality: str
    h60_ret: Optional[float]
    path_context: str
    base_tier: str
    final_tier: str
    candidate_stop_pct: Optional[float]
    candidate_stop_price: Optional[float]
    confidence: str
    rationale: str
    vs_adaptive_risk_pp: Optional[float]


def make_stop_v2(ticker: str, direction: str, date: str,
                 entry: float, adaptive_risk: float, adaptive_target: float,
                 h60_ret: Optional[float],
                 profile: Optional[dict]) -> StopDecisionV2:

    # Adaptive risk distance (negative = adverse)
    if direction == "LONG":
        ar_pct = (adaptive_risk - entry) / entry
    else:
        ar_pct = -(abs(adaptive_risk - entry) / entry)

    if profile is None:
        return StopDecisionV2(
            ticker=ticker, direction=direction, date=date,
            entry_price=entry, adaptive_risk=adaptive_risk,
            adaptive_risk_pct=ar_pct, n_obs=0,
            coverage_quality="UNAVAILABLE", h60_ret=h60_ret,
            path_context="NORMAL", base_tier="adaptive_risk",
            final_tier="adaptive_risk",
            candidate_stop_pct=ar_pct,
            candidate_stop_price=round(adaptive_risk, 2),
            confidence="NONE",
            rationale="No profile. Using adaptive_risk as-is.",
            vs_adaptive_risk_pp=0.0,
        )

    n = profile["n_obs"]
    quality = "USABLE" if n >= 5 else ("LIMITED" if n >= 2 else ("SPARSE" if n == 1 else "UNAVAILABLE"))
    confidence = "MEDIUM" if quality == "USABLE" else ("LOW" if quality in ("LIMITED","SPARSE") else "NONE")

    base_tier  = select_base_tier(n)
    ctx        = path_context(direction, h60_ret, profile)
    final_tier = adjust_tier(base_tier, ctx)

    # Resolve tier value, falling back up the ladder if None/non-adverse
    cand_pct = None
    used_tier = final_tier
    for tier in TIER_NAMES[TIER_NAMES.index(final_tier):]:
        v = get_tier_value(profile, tier, ar_pct)
        if v is not None and v < 0:
            cand_pct  = v
            used_tier = tier
            break

    if cand_pct is None:
        cand_pct  = ar_pct
        used_tier = "adaptive_risk"

    # Candidate stop price
    if direction == "LONG":
        cand_price = entry * (1 + cand_pct)
    else:
        cand_price = entry * (1 - abs(cand_pct))

    vs_ar = cand_pct - ar_pct  # positive = tighter than adaptive_risk

    h60_str = f"{h60_ret*100:+.3f}%" if h60_ret is not None else "N/A"
    rationale = (
        f"n={n} ({quality}), base={base_tier}, ctx={ctx} → final={used_tier}. "
        f"H60={h60_str}. "
        f"Candidate={cand_pct*100:+.3f}% vs AR={ar_pct*100:+.3f}%."
    )

    return StopDecisionV2(
        ticker=ticker, direction=direction, date=date,
        entry_price=entry, adaptive_risk=adaptive_risk,
        adaptive_risk_pct=ar_pct, n_obs=n,
        coverage_quality=quality, h60_ret=h60_ret,
        path_context=ctx, base_tier=base_tier,
        final_tier=used_tier,
        candidate_stop_pct=cand_pct,
        candidate_stop_price=round(cand_price, 2),
        confidence=confidence,
        rationale=rationale,
        vs_adaptive_risk_pp=vs_ar,
    )


# ── IC v1 frozen port + bar utilities ─────────────────────────────────────────

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


def classify_h60(direction, h60_ret):
    if h60_ret is None: return "WAIT"
    s = h60_ret if direction == "LONG" else -h60_ret
    return "ENTER" if s > 0.003 else ("WAIT" if s > -0.003 else "AVOID")


def compute_oqs(direction, h15, h60, mfe, mae, mp):
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


def classify_entry(direction, q, h60c):
    if direction == "SHORT":
        if h60c == "ENTER": return "ENTER" if q >= 70 else ("WAIT-HIGH" if q >= 50 else ("WAIT-MID" if q >= 30 else "WAIT-LOW"))
        elif h60c == "WAIT": return "WAIT-HIGH" if q >= 50 else ("WAIT-MID" if q >= 30 else "WAIT-LOW")
        else: return "AVOID"
    else:
        if h60c == "ENTER": return "ENTER" if q >= 70 else ("WAIT-HIGH" if q >= 50 else ("WAIT-MID" if q >= 30 else "WAIT-LOW"))
        elif h60c == "WAIT": return "WAIT-HIGH" if q >= 70 else ("WAIT-MID" if q >= 50 else ("WAIT-LOW" if q >= 30 else "AVOID"))
        else: return "AVOID"


def entry_action(state):
    return "ACT" if state in ("ENTER","WAIT-HIGH","ENTER-LATE","WAIT-LATE") else ("MONITOR" if state in ("WAIT-MID","WAIT-LOW") else "AVOID")


# ── Main ──────────────────────────────────────────────────────────────────────

def run_date(date_str: str, profiles: dict) -> list[StopDecisionV2]:
    ledger_entries = sorted(LEDGER_DIR.glob(f"LIVE-005-{date_str}-*.json"))
    if not ledger_entries:
        print(f"[stop_v2] No ledger entries for {date_str}")
        return []

    decisions: list[StopDecisionV2] = []

    for ep in ledger_entries:
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

        profile = profiles.get((ticker, direction))
        sd = make_stop_v2(ticker, direction, date_str, ref, risk, target, h60r, profile)
        decisions.append(sd)

    return decisions


def run() -> None:
    date_str = "20260908"
    for i, arg in enumerate(sys.argv[1:]):
        if arg.startswith("--date="):
            date_str = arg.split("=")[1]
        elif arg == "--date" and i + 2 < len(sys.argv):
            date_str = sys.argv[i + 2]

    profiles = load_profiles(PROFILE_CSV)
    print(f"[stop_v2] Profiles loaded: {len(profiles)}")
    print(f"[stop_v2] Date: {date_str}")
    print()

    decisions = run_date(date_str, profiles)
    print(f"[stop_v2] ACT decisions: {len(decisions)}")
    print()

    if not decisions:
        return

    # ── Print table ───────────────────────────────────────────────────────────
    print("=" * 130)
    print(f"Stop Decision Engine v0.2 — {date_str}  (N={len(decisions)})")
    print("=" * 130)
    print(f"{'ticker':25} {'dir':5} {'n':>3}  {'qual':11}  {'ctx':13}  "
          f"{'base_tier':15}  {'final_tier':15}  {'cand_pct':>9}  {'AR_pct':>8}  {'vs_AR':>8}")
    print("-" * 130)

    ctx_counts: dict[str, int] = {}
    tier_counts: dict[str, int] = {}

    for sd in sorted(decisions, key=lambda d: d.ticker):
        ctx_counts[sd.path_context] = ctx_counts.get(sd.path_context, 0) + 1
        tier_counts[sd.final_tier]  = tier_counts.get(sd.final_tier, 0) + 1

        cand_str = f"{sd.candidate_stop_pct*100:+.3f}%" if sd.candidate_stop_pct is not None else "   N/A"
        ar_str   = f"{sd.adaptive_risk_pct*100:+.3f}%"
        vs_str   = f"{sd.vs_adaptive_risk_pp*100:+.3f}pp" if sd.vs_adaptive_risk_pp is not None else "   N/A"
        marker   = " ◄" if sd.ticker == "JUBLFOOD_NS" else ""

        print(f"{sd.ticker:25} {sd.direction:5} {sd.n_obs:>3}  "
              f"{sd.coverage_quality:11}  {sd.path_context:13}  "
              f"{sd.base_tier:15}  {sd.final_tier:15}  "
              f"{cand_str:>9}  {ar_str:>8}  {vs_str:>8}{marker}")

    print("-" * 130)
    print(f"  Path context: {dict(sorted(ctx_counts.items()))}")
    print(f"  Final tiers:  {dict(sorted(tier_counts.items()))}")

    cand_pcts = [d.candidate_stop_pct for d in decisions if d.candidate_stop_pct is not None]
    ar_pcts   = [d.adaptive_risk_pct  for d in decisions if d.adaptive_risk_pct  is not None]
    if cand_pcts:
        print(f"  Candidate stop — mean: {statistics.mean(cand_pcts)*100:+.3f}%  "
              f"median: {statistics.median(cand_pcts)*100:+.3f}%")
    if ar_pcts:
        print(f"  Adaptive risk  — mean: {statistics.mean(ar_pcts)*100:+.3f}%  "
              f"median: {statistics.median(ar_pcts)*100:+.3f}%")
    print()

    # JUBLFOOD case study
    jublfood = next((d for d in decisions if d.ticker == "JUBLFOOD_NS"), None)
    if jublfood:
        print("JUBLFOOD_NS SHORT — v0.2 stop decision:")
        print(f"  n_obs:          {jublfood.n_obs}  ({jublfood.coverage_quality})")
        print(f"  h60_ret:        {jublfood.h60_ret*100:+.3f}%" if jublfood.h60_ret is not None else "  h60_ret: N/A")
        print(f"  path_context:   {jublfood.path_context}")
        print(f"  base_tier:      {jublfood.base_tier}")
        print(f"  final_tier:     {jublfood.final_tier}")
        print(f"  candidate_stop: ₹{jublfood.candidate_stop_price:.2f}  ({jublfood.candidate_stop_pct*100:+.3f}%)")
        print(f"  adaptive_risk:  ₹{jublfood.adaptive_risk:.2f}  ({jublfood.adaptive_risk_pct*100:+.3f}%)")
        print(f"  vs_adaptive:    {jublfood.vs_adaptive_risk_pp*100:+.3f}pp tighter")
        print(f"  rationale:      {jublfood.rationale}")
        print()
        print("  Compare to v0.1 (P75 universal):")
        print(f"    v0.1 candidate: -0.415%  → STOP at bar 1, realized +0.415%")
        print(f"    v0.2 candidate: {jublfood.candidate_stop_pct*100:+.3f}%")
        print()

    # ── Write CSV ─────────────────────────────────────────────────────────────
    out_path = OUT_DIR / f"stop_decisions_v2_{date_str}.csv"
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    fields = list(asdict(decisions[0]).keys())
    with open(out_path, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=fields)
        w.writeheader()
        for d in decisions:
            row = asdict(d)
            w.writerow({k: (f"{v:.6f}" if isinstance(v, float) else str(v))
                         for k, v in row.items()})
    print(f"[stop_v2] Output CSV: {out_path}")
    print()
    print("[stop_v2] Done. IC v1, Backtest v2, Cockpit, Profiler, Envelope, Paper Trader v0.1 untouched.")


if __name__ == "__main__":
    run()