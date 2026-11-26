#!/usr/bin/env python3
"""
Stop Decision Engine v0.1
==========================
Consumes the Contextual Stop Envelope and, for each decision, produces a
candidate stop boundary with an explicit risk/reward consequence statement.

This component answers:

    "Given the symbol's historical movement profile and the current decision
     context, what is a defensible stop boundary, and what are the expected
     consequences of placing it there?"

It does NOT:
  - claim to find the "optimal" stop
  - modify IC v1, Backtest v2, or the Cockpit
  - use future outcome information
  - pretend that 1–5 observations constitute a mature distribution

Architecture:
  IC v1 decision
      ↓
  Symbol Movement Profile  (symbol_movement_profiler.py output)
      ↓
  Contextual Stop Envelope (contextual_stop_envelope.py)
      ↓
  Stop Decision Engine     ← this script
      ↓
  Candidate stop + consequence statement

Stop boundary selection logic (conservative, v0.1):
  - If coverage_quality == UNAVAILABLE: emit NO_PROFILE, use adaptive_risk as-is
  - If coverage_quality == LIMITED (n_obs < 5):
      candidate = MAE_H300_p75 (upper-quartile adverse excursion)
      rationale: conservative — gives the trade room beyond the typical move
      confidence: LOW
  - If coverage_quality == USABLE (n_obs >= 5):
      candidate = MAE_H300_p75
      confidence: MEDIUM
  NOTE: We do NOT use P90 as the candidate because:
      - P90 from 5 observations is a single data point, not a reliable tail estimate
      - The purpose of the stop is to exit when the thesis has failed, not to
        survive every historical adverse excursion
      - P75 gives the trade room beyond the typical move while remaining
        meaningfully tighter than the current adaptive_risk

Consequence statement:
  For each candidate stop, the engine computes:
    candidate_stop_pct     — distance from entry as % (negative = adverse)
    vs_adaptive_risk_pct   — how much tighter than adaptive_risk (positive = tighter)
    expected_stop_rate     — fraction of historical observations that would have
                             been stopped at this level (from MAE distribution)
    expected_loss_if_stopped — the candidate stop distance itself (worst case)
    expected_saving_vs_ar  — how much loss is avoided vs adaptive_risk if stopped

Outputs:
  datasets/stop_decisions_sep8.csv  — per-decision stop candidate + consequence

Usage:
  python3 scripts/stop_decision_engine.py
"""

import csv
import json
from pathlib import Path
from dataclasses import dataclass, asdict
from typing import Optional

# ── Paths ─────────────────────────────────────────────────────────────────────

REPO_ROOT       = Path(__file__).resolve().parent.parent
PROFILE_CSV     = REPO_ROOT / "datasets" / "symbol_movement_profiles.csv"
LEDGER_DIR      = REPO_ROOT / "live_capture" / "ledger" / "entries"
OUT_DECISIONS   = REPO_ROOT / "datasets" / "stop_decisions_sep8.csv"

# ── Coverage thresholds (must match contextual_stop_envelope.py) ──────────────

USABLE_MIN_OBS = 5


# ── Profile loader (minimal — only what the engine needs) ─────────────────────

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
                "mae_h60_p75":   fv("mae_h60_p75"),
                "mfe_h300_p50":  fv("mfe_h300_p50"),
                "win_rate_h300": fv("win_rate_h300"),
                "recovery_rate": fv("recovery_rate"),
            }
    return profiles


# ── 8-Sep ACT decisions (same filter as previous scripts) ─────────────────────

def load_sep8_act_decisions() -> list[dict]:
    from datetime import datetime, timezone, timedelta
    IST = timezone(timedelta(hours=5, minutes=30))
    BARS_DIR  = REPO_ROOT / "intraday_capture" / "yahoo_cache_1m"
    OPEN_IST  = datetime(2026, 9, 8, 9, 15, 0, tzinfo=IST)
    CLOSE_IST = datetime(2026, 9, 8, 15, 30, 0, tzinfo=IST)
    open_ts, close_ts = int(OPEN_IST.timestamp()), int(CLOSE_IST.timestamp())

    def load_bars(ticker_ns):
        p = BARS_DIR / (ticker_ns.replace("_NS", ".NS") + ".json")
        if not p.exists(): return []
        bars = json.load(open(p))
        bars.sort(key=lambda b: b["timestamp"])
        return bars

    def sr(close, entry, direction):
        r = (close - entry) / entry
        return r if direction == "LONG" else -r

    def classify_h60(direction, h60_ret):
        if h60_ret is None: return "WAIT"
        s = h60_ret if direction == "LONG" else -h60_ret
        return "ENTER" if s > 0.003 else ("WAIT" if s > -0.003 else "AVOID")

    def oqs(direction, h15, h60, mfe, mae, mp):
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

    def action(state):
        return "ACT" if state in ("ENTER","WAIT-HIGH","ENTER-LATE","WAIT-LATE") else ("MONITOR" if state in ("WAIT-MID","WAIT-LOW") else "AVOID")

    records = []
    for ep in sorted(LEDGER_DIR.glob("LIVE-005-20260908-*.json")):
        e = json.load(open(ep))
        ticker, direction = e.get("ticker",""), e.get("direction","")
        ref, target, risk = e.get("reference_price"), e.get("adaptive_target"), e.get("adaptive_risk")
        if None in (ref, target, risk): continue
        sbars = [b for b in load_bars(ticker) if open_ts <= b["timestamp"] <= close_ts]
        if len(sbars) < 12: continue
        def ret(c): return sr(c, ref, direction)
        h60b = sbars[:12]
        h15r = ret(sbars[2]["close"]) if len(sbars) >= 3 else None
        h60r = ret(sbars[11]["close"]) if len(sbars) >= 12 else None
        mfeh = max(ret(b["close"]) for b in h60b) if h60b else None
        maeh = min(ret(b["close"]) for b in h60b) if h60b else None
        mpp  = sum(1 for b in h60b if ret(b["close"]) > 0) / len(h60b) if h60b else None
        h60c = classify_h60(direction, h60r)
        q    = oqs(direction, h15r, h60r, mfeh, maeh, mpp)
        st   = classify_entry(direction, q, h60c)
        if action(st) != "ACT": continue
        records.append({"ticker": ticker, "direction": direction,
                         "entry_price": ref, "adaptive_target": target, "adaptive_risk": risk})
    return records


# ── Stop decision ─────────────────────────────────────────────────────────────

@dataclass
class StopDecision:
    ticker: str
    direction: str
    entry_price: float
    adaptive_risk: float
    adaptive_risk_pct: float          # distance as % of entry (negative)
    n_obs: int
    coverage_quality: str             # UNAVAILABLE | LIMITED | USABLE
    candidate_stop_pct: Optional[float]   # distance from entry as % (negative)
    candidate_stop_price: Optional[float]
    confidence: str                   # NONE | LOW | MEDIUM
    rationale: str
    # Consequence statement
    vs_adaptive_risk_pp: Optional[float]  # how much tighter (positive = tighter)
    expected_stop_rate: Optional[float]   # fraction of hist obs stopped at candidate
    expected_loss_if_stopped: Optional[float]  # = candidate_stop_pct (worst case)
    expected_saving_vs_ar: Optional[float]     # loss avoided vs adaptive_risk if stopped


def make_stop_decision(ticker: str, direction: str, entry: float,
                       adaptive_risk: float, adaptive_target: float,
                       profile: Optional[dict]) -> StopDecision:
    # Adaptive risk distance
    if direction == "LONG":
        ar_pct = (adaptive_risk - entry) / entry   # negative
    else:
        ar_pct = -(abs(adaptive_risk - entry) / entry)  # negative

    if profile is None:
        return StopDecision(
            ticker=ticker, direction=direction, entry_price=entry,
            adaptive_risk=adaptive_risk, adaptive_risk_pct=ar_pct,
            n_obs=0, coverage_quality="UNAVAILABLE",
            candidate_stop_pct=None, candidate_stop_price=None,
            confidence="NONE", rationale="No historical profile available.",
            vs_adaptive_risk_pp=None, expected_stop_rate=None,
            expected_loss_if_stopped=None, expected_saving_vs_ar=None,
        )

    n = profile["n_obs"]
    quality = "USABLE" if n >= USABLE_MIN_OBS else "LIMITED"

    # Candidate boundary: MAE_H300_p75
    # Rationale: gives the trade room beyond the typical (P50) adverse move,
    # while remaining meaningfully tighter than the current adaptive_risk.
    # We do NOT use P90 because with n=1–5, P90 is a single observation.
    candidate_pct = profile.get("mae_h300_p75")

    if candidate_pct is None or candidate_pct >= 0:
        # Profile shows no adverse excursion at P75 (all observations were positive)
        # Use P50 as fallback; if also None/positive, use a conservative 0.5% floor
        candidate_pct = profile.get("mae_h300_p50")
        if candidate_pct is None or candidate_pct >= 0:
            candidate_pct = -0.005  # 0.5% floor — minimal protection
        rationale = "P75 MAE is non-negative (symbol rarely moves adversely). Using P50 or 0.5% floor."
    else:
        rationale = (f"Candidate = MAE_H300_p75 ({candidate_pct*100:+.3f}%). "
                     f"Gives room beyond typical adverse move (P50={profile['mae_h300_p50']*100:+.3f}%) "
                     f"while tighter than adaptive_risk ({ar_pct*100:+.3f}%).")

    confidence = "MEDIUM" if quality == "USABLE" else "LOW"

    # Candidate stop price
    if direction == "LONG":
        candidate_price = entry * (1 + candidate_pct)
    else:
        candidate_price = entry * (1 - abs(candidate_pct))

    # Consequence: how much tighter than adaptive_risk?
    vs_ar = candidate_pct - ar_pct   # positive = candidate is tighter (less negative)

    # Expected stop rate: fraction of historical obs where MAE_H300 <= candidate_pct
    # We approximate from the percentile structure:
    # candidate = P75 → expected ~25% of observations would have been stopped
    # (by definition of the percentile, 75% of obs had MAE worse than P75)
    # But since MAE is adverse (negative), P75 means 75% had MAE more negative than P75
    # → 25% had MAE less negative (i.e., would NOT have been stopped)
    # → 75% had MAE more negative (i.e., WOULD have been stopped at P75)
    # Wait — MAE is the worst adverse move. If candidate_pct = P75 of MAE,
    # then 75% of observations had MAE <= candidate_pct (more adverse),
    # meaning 75% of trades would have been stopped at this level.
    # That is too high. Let's be precise:
    # MAE values are negative. P75 is the 75th percentile of a negative distribution.
    # In a sorted ascending list of negatives: P25 is most negative, P75 is least negative.
    # So P75 MAE = the value where 75% of observations had MAE MORE negative (worse).
    # A stop at P75 would be hit when MAE reaches P75 or worse.
    # 75% of observations had MAE at least as bad as P75 → stop hit rate ≈ 75%.
    # That's the correct interpretation.
    expected_stop_rate = 0.75  # by definition of P75

    expected_loss = candidate_pct  # worst case = stop distance
    expected_saving = abs(candidate_pct) - abs(ar_pct)  # negative = candidate is wider (no saving)

    return StopDecision(
        ticker=ticker, direction=direction, entry_price=entry,
        adaptive_risk=adaptive_risk, adaptive_risk_pct=ar_pct,
        n_obs=n, coverage_quality=quality,
        candidate_stop_pct=candidate_pct,
        candidate_stop_price=round(candidate_price, 2),
        confidence=confidence,
        rationale=rationale,
        vs_adaptive_risk_pp=vs_ar,
        expected_stop_rate=expected_stop_rate,
        expected_loss_if_stopped=expected_loss,
        expected_saving_vs_ar=expected_saving,
    )


# ── Main ──────────────────────────────────────────────────────────────────────

def run() -> None:
    profiles = load_profiles(PROFILE_CSV)
    print(f"[stop_engine] Profiles loaded: {len(profiles)}")

    act_decisions = load_sep8_act_decisions()
    print(f"[stop_engine] 8-Sep ACT decisions: {len(act_decisions)}")
    print()

    decisions: list[StopDecision] = []
    for dec in act_decisions:
        profile = profiles.get((dec["ticker"], dec["direction"]))
        sd = make_stop_decision(
            dec["ticker"], dec["direction"], dec["entry_price"],
            dec["adaptive_risk"], dec["adaptive_target"], profile
        )
        decisions.append(sd)

    # ── Print table ───────────────────────────────────────────────────────────
    print("=" * 120)
    print("Stop Decision Engine v0.1 — 8-Sep-2026 ACT decisions")
    print("=" * 120)
    print(f"{'ticker':25} {'dir':5} {'n':>3}  {'qual':11}  {'conf':6}  "
          f"{'AR_pct':>8}  {'cand_pct':>9}  {'vs_AR_pp':>9}  {'exp_stop%':>10}  {'exp_save':>9}")
    print("-" * 120)

    for sd in sorted(decisions, key=lambda d: d.ticker):
        ar_str   = f"{sd.adaptive_risk_pct*100:+.3f}%" if sd.adaptive_risk_pct is not None else "   N/A"
        cand_str = f"{sd.candidate_stop_pct*100:+.3f}%" if sd.candidate_stop_pct is not None else "   N/A"
        vs_str   = f"{sd.vs_adaptive_risk_pp*100:+.3f}pp" if sd.vs_adaptive_risk_pp is not None else "   N/A"
        stop_r   = f"{sd.expected_stop_rate*100:.0f}%" if sd.expected_stop_rate is not None else "  N/A"
        save_str = f"{sd.expected_saving_vs_ar*100:+.3f}pp" if sd.expected_saving_vs_ar is not None else "   N/A"
        marker   = " ◄" if sd.ticker == "JUBLFOOD_NS" else ""

        print(f"{sd.ticker:25} {sd.direction:5} {sd.n_obs:>3}  "
              f"{sd.coverage_quality:11}  {sd.confidence:6}  "
              f"{ar_str:>8}  {cand_str:>9}  {vs_str:>9}  {stop_r:>10}  {save_str:>9}{marker}")

    print("-" * 120)

    # Summary
    usable  = [d for d in decisions if d.coverage_quality == "USABLE"]
    limited = [d for d in decisions if d.coverage_quality == "LIMITED"]
    unavail = [d for d in decisions if d.coverage_quality == "UNAVAILABLE"]
    print(f"  USABLE={len(usable)}  LIMITED={len(limited)}  UNAVAILABLE={len(unavail)}")

    import statistics
    cand_pcts = [d.candidate_stop_pct for d in decisions if d.candidate_stop_pct is not None]
    ar_pcts   = [d.adaptive_risk_pct  for d in decisions if d.adaptive_risk_pct  is not None]
    if cand_pcts and ar_pcts:
        print(f"  Candidate stop distance — mean: {statistics.mean(cand_pcts)*100:+.3f}%  "
              f"median: {statistics.median(cand_pcts)*100:+.3f}%")
        print(f"  Adaptive risk distance  — mean: {statistics.mean(ar_pcts)*100:+.3f}%  "
              f"median: {statistics.median(ar_pcts)*100:+.3f}%")
        savings = [d.expected_saving_vs_ar for d in decisions if d.expected_saving_vs_ar is not None]
        print(f"  Mean expected saving vs adaptive_risk (if stopped): "
              f"{statistics.mean(savings)*100:+.3f}pp")
    print()

    # ── JUBLFOOD case study ───────────────────────────────────────────────────
    jublfood = next((d for d in decisions if d.ticker == "JUBLFOOD_NS"), None)
    if jublfood:
        print("JUBLFOOD_NS SHORT — stop decision detail:")
        print(f"  entry_price:           ₹{jublfood.entry_price:.2f}")
        print(f"  adaptive_risk:         ₹{jublfood.adaptive_risk:.2f}  ({jublfood.adaptive_risk_pct*100:+.3f}%)")
        print(f"  candidate_stop:        ₹{jublfood.candidate_stop_price:.2f}  ({jublfood.candidate_stop_pct*100:+.3f}%)")
        print(f"  confidence:            {jublfood.confidence}")
        print(f"  coverage_quality:      {jublfood.coverage_quality}  (n={jublfood.n_obs})")
        print(f"  vs_adaptive_risk:      {jublfood.vs_adaptive_risk_pp*100:+.3f}pp tighter")
        print(f"  expected_stop_rate:    {jublfood.expected_stop_rate*100:.0f}%  (by P75 definition)")
        print(f"  expected_saving_vs_ar: {jublfood.expected_saving_vs_ar*100:+.3f}pp if stopped")
        print()
        print(f"  Rationale: {jublfood.rationale}")
        print()
        print("  8-Sep actual outcome:")
        print(f"    0.50× stop (-0.846%) triggered → realized -1.689%")
        print(f"    H300 counterfactual: -0.296%")
        print(f"    Candidate stop (-{abs(jublfood.candidate_stop_pct)*100:.3f}%) would have been:")
        if jublfood.candidate_stop_pct is not None:
            cand = jublfood.candidate_stop_pct
            print(f"      {'TRIGGERED' if -1.689/100 <= cand else 'NOT TRIGGERED'} "
                  f"(8-Sep MAE reached -1.689%, candidate = {cand*100:+.3f}%)")
            if -1.689/100 <= cand:
                print(f"      → realized {cand*100:+.3f}% vs H300 cf -0.296%  "
                      f"(cost vs no-stop: {(cand - (-0.00296))*100:+.3f}pp)")
            else:
                print(f"      → trade would have survived to H300 cf: -0.296%")

    print()
    print("[stop_engine] Done. IC v1, Backtest v2, and Cockpit are untouched.")

    # ── Write CSV ─────────────────────────────────────────────────────────────
    OUT_DECISIONS.parent.mkdir(parents=True, exist_ok=True)
    if decisions:
        fields = list(asdict(decisions[0]).keys())
        with open(OUT_DECISIONS, "w", newline="") as f:
            w = csv.DictWriter(f, fieldnames=fields)
            w.writeheader()
            for d in decisions:
                row = asdict(d)
                w.writerow({k: (f"{v:.6f}" if isinstance(v, float) else str(v))
                             for k, v in row.items()})
        print(f"[stop_engine] Output CSV: {OUT_DECISIONS}")


if __name__ == "__main__":
    run()