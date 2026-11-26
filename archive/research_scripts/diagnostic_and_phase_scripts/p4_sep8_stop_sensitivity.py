#!/usr/bin/env python3
"""
P4 — 8-Sep-2026 Stop-Loss Sensitivity Analysis
================================================
Replays the 31 ACT decisions from the 8-Sep paper-trading replay
across a matrix of stop-distance multipliers, using the existing
adaptive_risk as the anchor.

For each multiplier k, the adjusted stop is:
  LONG:  SL_k = entry - k × (entry - adaptive_risk)
  SHORT: SL_k = entry + k × (adaptive_risk - entry)

Multipliers tested: 0.50, 0.75, 0.90, 1.00 (baseline), 1.10, 1.25, 1.50, 2.00

For each (decision, multiplier) pair, the 1-minute path is replayed from
bar 13 (10:15 IST) onward. The target (adaptive_target) is held constant.

Metrics reported per multiplier:
  stop_hit_pct       — fraction of trades where stop was triggered
  target_hit_pct     — fraction of trades where target was reached
  horizon_pct        — fraction of trades that ran to H300
  false_stop_pct     — stop hit, but stock subsequently recovered to target
  win_rate           — fraction of trades with positive realized return
  mean_ret           — mean realized return
  median_ret         — median realized return
  profit_factor      — gross wins / gross losses
  total_pnl_pct      — sum of realized returns (equal-weight)
  mean_mae           — mean maximum adverse excursion before exit

IC v1, Backtest v2, and Cockpit are NOT modified.
The base replay (p4_sep8_replay.py) is NOT modified.

Outputs:
  datasets/p4_sep8_stop_sensitivity.csv   — per-decision × per-multiplier rows
  datasets/p4_sep8_stop_sensitivity_summary.csv — per-multiplier summary

Usage:
  python3 scripts/p4_sep8_stop_sensitivity.py
"""

import json
import csv
import sys
import statistics
from pathlib import Path
from datetime import datetime, timezone, timedelta
from dataclasses import dataclass, asdict
from typing import Optional

# ── Paths ─────────────────────────────────────────────────────────────────────

REPO_ROOT    = Path(__file__).resolve().parent.parent
LEDGER_DIR   = REPO_ROOT / "live_capture" / "ledger" / "entries"
BARS_1M_DIR  = REPO_ROOT / "intraday_capture" / "yahoo_cache_1m"
OUT_DETAIL   = REPO_ROOT / "datasets" / "p4_sep8_stop_sensitivity.csv"
OUT_SUMMARY  = REPO_ROOT / "datasets" / "p4_sep8_stop_sensitivity_summary.csv"

# ── IST offset ────────────────────────────────────────────────────────────────

IST = timezone(timedelta(hours=5, minutes=30))
OPEN_IST  = datetime(2026, 9, 8, 9, 15, 0, tzinfo=IST)
CLOSE_IST = datetime(2026, 9, 8, 15, 30, 0, tzinfo=IST)

# ── Stop multipliers ──────────────────────────────────────────────────────────

MULTIPLIERS = [0.50, 0.75, 0.90, 1.00, 1.10, 1.25, 1.50, 2.00]

# ── Frozen IC v1 — Python port ────────────────────────────────────────────────

def classify_at_entry(direction: str, oqs: int, h60_class: str) -> str:
    if direction == "SHORT":
        if h60_class == "ENTER":
            if oqs >= 70: return "ENTER"
            elif oqs >= 50: return "WAIT-HIGH"
            elif oqs >= 30: return "WAIT-MID"
            else: return "WAIT-LOW"
        elif h60_class == "WAIT":
            if oqs >= 70: return "WAIT-HIGH"
            elif oqs >= 50: return "WAIT-HIGH"
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


def entry_action(state: str) -> str:
    if state in ("ENTER", "WAIT-HIGH", "ENTER-LATE", "WAIT-LATE"): return "ACT"
    elif state in ("WAIT-MID", "WAIT-LOW"): return "MONITOR"
    else: return "AVOID"


def classify_h60(direction: str, h60_ret: Optional[float]) -> str:
    if h60_ret is None: return "WAIT"
    signed = h60_ret if direction == "LONG" else -h60_ret
    if signed > 0.003: return "ENTER"
    elif signed > -0.003: return "WAIT"
    else: return "AVOID"


def compute_time_safe_oqs(direction: str, h15_ret, h60_ret, mfe_h60, mae_h60,
                           momentum_persistence) -> int:
    score = 0
    if h15_ret is not None:
        signed = h15_ret if direction == "LONG" else -h15_ret
        if signed > 0.005: score += 25
        elif signed > 0.002: score += 18
        elif signed > 0.0: score += 10
        elif signed > -0.002: score += 4
    if mfe_h60 is not None:
        if mfe_h60 > 0.010: score += 25
        elif mfe_h60 > 0.005: score += 18
        elif mfe_h60 > 0.002: score += 12
        elif mfe_h60 > 0.0: score += 6
    if momentum_persistence is not None:
        if momentum_persistence >= 0.75: score += 25
        elif momentum_persistence >= 0.60: score += 18
        elif momentum_persistence >= 0.45: score += 10
        elif momentum_persistence >= 0.30: score += 4
    if mae_h60 is not None:
        abs_mae = abs(mae_h60)
        if abs_mae < 0.002: score += 25
        elif abs_mae < 0.005: score += 18
        elif abs_mae < 0.010: score += 10
        elif abs_mae < 0.015: score += 4
    return min(score, 100)


# ── Bar utilities ─────────────────────────────────────────────────────────────

def load_bars(ticker_ns: str) -> list[dict]:
    filename = ticker_ns.replace("_NS", ".NS") + ".json"
    path = BARS_1M_DIR / filename
    if not path.exists():
        return []
    with open(path) as f:
        bars = json.load(f)
    bars.sort(key=lambda b: b["timestamp"])
    return bars


def session_bars(bars: list[dict]) -> list[dict]:
    open_ts  = int(OPEN_IST.timestamp())
    close_ts = int(CLOSE_IST.timestamp())
    return [b for b in bars if open_ts <= b["timestamp"] <= close_ts]


def path_features(direction: str, entry_price: float, sbars: list[dict]) -> dict:
    def ret(close): return (close - entry_price) / entry_price if direction == "LONG" else (entry_price - close) / entry_price
    h60_bars = sbars[:12]
    h15_ret = ret(sbars[2]["close"]) if len(sbars) >= 3 else None
    h30_ret = ret(sbars[5]["close"]) if len(sbars) >= 6 else None
    h60_ret = ret(sbars[11]["close"]) if len(sbars) >= 12 else None
    mfe_h60 = max(ret(b["close"]) for b in h60_bars) if h60_bars else None
    mae_h60 = min(ret(b["close"]) for b in h60_bars) if h60_bars else None
    mp = sum(1 for b in h60_bars if ret(b["close"]) > 0) / len(h60_bars) if h60_bars else None
    return {"h15_ret": h15_ret, "h30_ret": h30_ret, "h60_ret": h60_ret,
            "mfe_h60": mfe_h60, "mae_h60": mae_h60, "momentum_persistence": mp}


def replay_with_stop(direction: str, entry_price: float, target: float, stop: float,
                     sbars: list[dict], start_bar: int = 12
                     ) -> tuple[float, str, str, float]:
    """
    Walk forward from start_bar with the given stop and target.
    Returns (exit_price, exit_time_ist, exit_reason, max_adverse_excursion).
    exit_reason: TARGET | RISK | HORIZON
    max_adverse_excursion: worst direction-adjusted return seen before exit
    """
    def ret(close): return (close - entry_price) / entry_price if direction == "LONG" else (entry_price - close) / entry_price

    mae = 0.0
    for bar in sbars[start_bar:]:
        low, high = bar["low"], bar["high"]
        ts = datetime.fromtimestamp(bar["timestamp"], tz=IST).strftime("%H:%M")
        # Track MAE (worst adverse move)
        if direction == "LONG":
            mae = min(mae, ret(low))
        else:
            mae = min(mae, ret(high))

        if direction == "SHORT":
            if low <= target:
                return target, ts, "TARGET", mae
            if high >= stop:
                return stop, ts, "RISK", mae
        else:
            if high >= target:
                return target, ts, "TARGET", mae
            if low <= stop:
                return stop, ts, "RISK", mae

    # Horizon
    if len(sbars) >= 60:
        bar = sbars[59]
    elif sbars:
        bar = sbars[-1]
    else:
        return entry_price, "15:14", "HORIZON", mae
    ts = datetime.fromtimestamp(bar["timestamp"], tz=IST).strftime("%H:%M")
    return bar["close"], ts, "HORIZON", mae


def check_false_stop(direction: str, entry_price: float, target: float,
                     stop_hit_bar_idx: int, sbars: list[dict]) -> bool:
    """
    After a stop is hit at stop_hit_bar_idx, check whether the stock
    subsequently recovered to reach the target before session end.
    Returns True if it would have reached target after the stop.
    """
    for bar in sbars[stop_hit_bar_idx:]:
        if direction == "SHORT":
            if bar["low"] <= target:
                return True
        else:
            if bar["high"] >= target:
                return True
    return False


# ── Main ──────────────────────────────────────────────────────────────────────

@dataclass
class DetailRow:
    ticker: str
    direction: str
    entry_price: float
    adaptive_target: float
    adaptive_risk: float
    risk_distance: float
    multiplier: float
    adjusted_stop: float
    exit_price: Optional[float]
    exit_time_ist: Optional[str]
    exit_reason: str
    realized_return: Optional[float]
    max_adverse_excursion: float
    false_stop: bool
    oqs: int
    entry_state: str
    # Counterfactual: what would H300 return have been with no stop at all?
    h300_counterfactual_ret: Optional[float] = None


def run() -> None:
    # Load 8-Sep ledger entries
    sep8_entries = sorted(LEDGER_DIR.glob("LIVE-005-20260908-*.json"))
    print(f"[sensitivity] 8-Sep ledger entries: {len(sep8_entries)}")

    # Filter to ACT decisions only (same as base replay)
    act_records = []
    for ep in sep8_entries:
        e = json.load(open(ep))
        ticker_ns = e.get("ticker", "")
        direction = e.get("direction", "")
        ref_price = e.get("reference_price")
        adaptive_target = e.get("adaptive_target")
        adaptive_risk = e.get("adaptive_risk")

        if ref_price is None or adaptive_target is None or adaptive_risk is None:
            continue

        all_bars = load_bars(ticker_ns)
        sbars = session_bars(all_bars)
        if len(sbars) < 12:
            continue

        pf = path_features(direction, ref_price, sbars)
        h60_cls = classify_h60(direction, pf["h60_ret"])
        oqs = compute_time_safe_oqs(direction, pf["h15_ret"], pf["h60_ret"],
                                     pf["mfe_h60"], pf["mae_h60"], pf["momentum_persistence"])
        state = classify_at_entry(direction, oqs, h60_cls)
        action = entry_action(state)

        if action != "ACT":
            continue

        act_records.append({
            "ticker": ticker_ns,
            "direction": direction,
            "entry_price": ref_price,
            "adaptive_target": adaptive_target,
            "adaptive_risk": adaptive_risk,
            "sbars": sbars,
            "oqs": oqs,
            "entry_state": state,
        })

    print(f"[sensitivity] ACT decisions: {len(act_records)}")
    print()

    detail_rows: list[DetailRow] = []

    for rec in act_records:
        direction = rec["direction"]
        entry = rec["entry_price"]
        target = rec["adaptive_target"]
        orig_risk = rec["adaptive_risk"]
        sbars = rec["sbars"]

        # Risk distance from entry to original stop
        if direction == "LONG":
            risk_dist = entry - orig_risk   # positive = distance below entry
        else:
            risk_dist = orig_risk - entry   # positive = distance above entry

        if risk_dist <= 0:
            # Degenerate: skip
            continue

        # ── Counterfactual: H300 return with no stop (target-or-horizon only) ──
        # Use a stop so far away it can never be hit (100× risk distance).
        # This gives the "what if we never stopped out" H300 outcome.
        if direction == "LONG":
            infinite_stop = entry - 100.0 * risk_dist
        else:
            infinite_stop = entry + 100.0 * risk_dist

        cf_exit_price, _cf_time, _cf_reason, _cf_mae = replay_with_stop(
            direction, entry, target, infinite_stop, sbars, start_bar=12
        )
        if cf_exit_price is not None:
            if direction == "LONG":
                h300_cf_ret: Optional[float] = (cf_exit_price - entry) / entry
            else:
                h300_cf_ret = (entry - cf_exit_price) / entry
        else:
            h300_cf_ret = None

        for k in MULTIPLIERS:
            if direction == "LONG":
                adjusted_stop = entry - k * risk_dist
            else:
                adjusted_stop = entry + k * risk_dist

            exit_price, exit_time, exit_reason, mae = replay_with_stop(
                direction, entry, target, adjusted_stop, sbars, start_bar=12
            )

            realized_ret = None
            if exit_price is not None:
                if direction == "LONG":
                    realized_ret = (exit_price - entry) / entry
                else:
                    realized_ret = (entry - exit_price) / entry

            # False stop: stop hit, but stock later reached target
            false_stop = False
            if exit_reason == "RISK":
                # Find which bar the stop was hit
                stop_bar_idx = 12
                for i, bar in enumerate(sbars[12:], start=12):
                    if direction == "SHORT" and bar["high"] >= adjusted_stop:
                        stop_bar_idx = i + 1
                        break
                    elif direction == "LONG" and bar["low"] <= adjusted_stop:
                        stop_bar_idx = i + 1
                        break
                false_stop = check_false_stop(direction, entry, target, stop_bar_idx, sbars)

            detail_rows.append(DetailRow(
                ticker=rec["ticker"],
                direction=direction,
                entry_price=entry,
                adaptive_target=target,
                adaptive_risk=orig_risk,
                risk_distance=risk_dist,
                multiplier=k,
                adjusted_stop=adjusted_stop,
                exit_price=exit_price,
                exit_time_ist=exit_time,
                exit_reason=exit_reason,
                realized_return=realized_ret,
                max_adverse_excursion=mae,
                false_stop=false_stop,
                oqs=rec["oqs"],
                entry_state=rec["entry_state"],
                h300_counterfactual_ret=h300_cf_ret,
            ))

    # ── Write detail CSV ──────────────────────────────────────────────────────
    OUT_DETAIL.parent.mkdir(parents=True, exist_ok=True)
    fields = list(asdict(detail_rows[0]).keys()) if detail_rows else []
    with open(OUT_DETAIL, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=fields)
        w.writeheader()
        for r in detail_rows:
            row = asdict(r)
            w.writerow({k: (f"{v:.6f}" if isinstance(v, float) else str(v)) for k, v in row.items()})
    print(f"[sensitivity] Detail CSV: {OUT_DETAIL}")

    # ── Summary per multiplier ────────────────────────────────────────────────
    summary_rows = []
    print()
    print("=" * 80)
    print("Stop-Loss Sensitivity — 8-Sep-2026 ACT decisions (N=31)")
    print("=" * 80)
    print(f"{'k':>6}  {'stop%':>7}  {'tgt%':>6}  {'hor%':>6}  {'false%':>7}  "
          f"{'win%':>6}  {'mean':>8}  {'med':>8}  {'PF':>6}  {'totPnL':>8}  {'MAE':>8}")
    print("-" * 80)

    for k in MULTIPLIERS:
        rows = [r for r in detail_rows if abs(r.multiplier - k) < 0.001]
        n = len(rows)
        if n == 0:
            continue

        stop_hit   = [r for r in rows if r.exit_reason == "RISK"]
        target_hit = [r for r in rows if r.exit_reason == "TARGET"]
        horizon    = [r for r in rows if r.exit_reason == "HORIZON"]
        false_stop = [r for r in rows if r.false_stop]

        rets = [r.realized_return for r in rows if r.realized_return is not None]
        wins = [r for r in rows if r.realized_return is not None and r.realized_return > 0]
        losses = [r for r in rows if r.realized_return is not None and r.realized_return <= 0]

        gross_win  = sum(r.realized_return for r in wins)
        gross_loss = abs(sum(r.realized_return for r in losses))
        pf = gross_win / gross_loss if gross_loss > 0 else float("inf")

        mean_ret   = statistics.mean(rets) if rets else 0.0
        median_ret = statistics.median(rets) if rets else 0.0
        total_pnl  = sum(rets)
        mean_mae   = statistics.mean(r.max_adverse_excursion for r in rows)

        marker = " ←" if abs(k - 1.0) < 0.001 else ""

        print(f"{k:>6.2f}  {len(stop_hit)/n*100:>6.1f}%  {len(target_hit)/n*100:>5.1f}%  "
              f"{len(horizon)/n*100:>5.1f}%  {len(false_stop)/n*100:>6.1f}%  "
              f"{len(wins)/n*100:>5.1f}%  {mean_ret*100:>+7.3f}%  {median_ret*100:>+7.3f}%  "
              f"{pf:>6.2f}  {total_pnl*100:>+7.3f}%  {mean_mae*100:>+7.3f}%{marker}")

        summary_rows.append({
            "multiplier": k,
            "n": n,
            "stop_hit": len(stop_hit),
            "target_hit": len(target_hit),
            "horizon": len(horizon),
            "false_stop": len(false_stop),
            "stop_hit_pct": len(stop_hit) / n,
            "target_hit_pct": len(target_hit) / n,
            "horizon_pct": len(horizon) / n,
            "false_stop_pct": len(false_stop) / n,
            "win_rate": len(wins) / n,
            "mean_ret": mean_ret,
            "median_ret": median_ret,
            "profit_factor": pf if pf != float("inf") else 999.0,
            "total_pnl": total_pnl,
            "mean_mae": mean_mae,
        })

    print("-" * 80)
    print("  k=1.00 ← baseline (adaptive_risk from ledger)")
    print()

    # ── Write summary CSV ─────────────────────────────────────────────────────
    if summary_rows:
        with open(OUT_SUMMARY, "w", newline="") as f:
            w = csv.DictWriter(f, fieldnames=list(summary_rows[0].keys()))
            w.writeheader()
            for r in summary_rows:
                w.writerow({k: (f"{v:.6f}" if isinstance(v, float) else str(v))
                             for k, v in r.items()})
        print(f"[sensitivity] Summary CSV: {OUT_SUMMARY}")

    # ── False-stop case studies ───────────────────────────────────────────────
    baseline_false = [r for r in detail_rows
                      if abs(r.multiplier - 1.0) < 0.001 and r.false_stop]
    if baseline_false:
        print()
        print("False stops at baseline (k=1.00) — stop hit, stock later reached target:")
        for r in baseline_false:
            print(f"  {r.ticker:25} {r.direction:5}  "
                  f"entry={r.entry_price:.2f}  stop={r.adjusted_stop:.2f}  "
                  f"target={r.adaptive_target:.2f}  ret={r.realized_return*100:+.2f}%")

    # ── Counterfactual case studies — stop-hit rows only ─────────────────────
    stop_hit_rows = [r for r in detail_rows if r.exit_reason == "RISK"]
    if stop_hit_rows:
        print()
        print("Counterfactual case studies — stop-hit trades (actual vs no-stop H300):")
        print(f"  {'ticker':25} {'dir':5} {'k':>5}  {'actual_ret':>11}  {'h300_cf_ret':>12}  {'delta':>8}  false_stop")
        print("  " + "-" * 80)
        for r in sorted(stop_hit_rows, key=lambda x: (x.ticker, x.multiplier)):
            cf = r.h300_counterfactual_ret
            cf_str  = f"{cf*100:+.3f}%" if cf is not None else "    N/A"
            act_str = f"{r.realized_return*100:+.3f}%" if r.realized_return is not None else "    N/A"
            delta_str = (f"{(cf - r.realized_return)*100:+.3f}%"
                         if cf is not None and r.realized_return is not None else "    N/A")
            print(f"  {r.ticker:25} {r.direction:5} {r.multiplier:>5.2f}  "
                  f"{act_str:>11}  {cf_str:>12}  {delta_str:>8}  {r.false_stop}")

    print()
    print("[sensitivity] Done. IC v1, Backtest v2, and Cockpit are untouched.")


if __name__ == "__main__":
    run()
