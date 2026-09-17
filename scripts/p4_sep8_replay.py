#!/usr/bin/env python3
"""
P4 — 8-Sep-2026 Historical Paper-Trading Replay
================================================
Reconstructs exactly what the frozen IC v1 system would have predicted
for each of the 79 8-Sep-2026 upstream decisions, using only information
available at or before 10:15 IST (the H60 information boundary).

Information boundary enforced:
  BEFORE 10:15 IST (available):
    - Ledger entry fields (direction, reference_price, adaptive_target,
      adaptive_risk, adaptive_horizon_sessions, rank_score, target_rate, ...)
    - 1m bars 09:15–10:14 IST (bars 1–60)
    - h15_ret, h30_ret, h60_ret, mfe_h60, mae_h60, momentum_persistence
    - h60_classification (derived from h60_ret)
    - time_safe_oqs (derived from H60-available features only)

  AFTER 10:15 IST (post-decision observations only):
    - h120_ret, h180_ret, h300_ret
    - h120_state (from reassess_at_h120)
    - paper_exit_price, exit_time, exit_reason, realized_return

No future information is used when generating the entry decision.
IC v1, Backtest v2, and Cockpit are NOT modified.

Outputs:
  datasets/p4_sep8_replay.csv   — full replay ledger
  datasets/p4_sep8_replay.json  — same, JSON format

Usage:
  python3 scripts/p4_sep8_replay.py
"""

import json
import csv
import sys
from pathlib import Path
from datetime import datetime, timezone, timedelta
from dataclasses import dataclass, field, asdict
from typing import Optional

# ── Paths ─────────────────────────────────────────────────────────────────────

REPO_ROOT   = Path(__file__).resolve().parent.parent
LEDGER_DIR  = REPO_ROOT / "live_capture" / "ledger" / "entries"
BARS_1M_DIR = REPO_ROOT / "intraday_capture" / "yahoo_cache_1m"
OUT_CSV     = REPO_ROOT / "datasets" / "p4_sep8_replay.csv"
OUT_JSON    = REPO_ROOT / "datasets" / "p4_sep8_replay.json"

# ── IST offset ────────────────────────────────────────────────────────────────

IST = timezone(timedelta(hours=5, minutes=30))

# 8-Sep-2026 session boundaries (IST)
SEP8_DATE   = "2026-09-08"
OPEN_IST    = datetime(2026, 9, 8, 9, 15, 0, tzinfo=IST)
H60_IST     = datetime(2026, 9, 8, 10, 15, 0, tzinfo=IST)   # entry decision
H120_IST    = datetime(2026, 9, 8, 12, 15, 0, tzinfo=IST)   # reassessment
H180_IST    = datetime(2026, 9, 8, 13, 15, 0, tzinfo=IST)
H300_IST    = datetime(2026, 9, 8, 15, 15, 0, tzinfo=IST)   # session close
CLOSE_IST   = datetime(2026, 9, 8, 15, 30, 0, tzinfo=IST)

# ── Frozen IC v1 — Python port (mirrors Rust intraday_classification) ─────────
# Source of truth: adapters/chronosentiment/src/reasoning/intraday_classification/
# This port was differential-tested (459/459 match) in scripts/differential_test_v1.py.

def classify_at_entry(direction: str, oqs: int, h60_class: str) -> str:
    """
    Frozen IC v1 entry classification.
    Mirrors classify_at_entry() in Rust.
    Returns: ENTER | WAIT-HIGH | WAIT-MID | WAIT-LOW | AVOID
    """
    if direction == "SHORT":
        if h60_class == "ENTER":
            if oqs >= 70:
                return "ENTER"
            elif oqs >= 50:
                return "WAIT-HIGH"
            elif oqs >= 30:
                return "WAIT-MID"
            else:
                return "WAIT-LOW"
        elif h60_class == "WAIT":
            if oqs >= 70:
                return "WAIT-HIGH"
            elif oqs >= 50:
                return "WAIT-HIGH"
            elif oqs >= 30:
                return "WAIT-MID"
            else:
                return "WAIT-LOW"
        else:  # AVOID
            return "AVOID"
    else:  # LONG
        if h60_class == "ENTER":
            if oqs >= 70:
                return "ENTER"
            elif oqs >= 50:
                return "WAIT-HIGH"
            elif oqs >= 30:
                return "WAIT-MID"
            else:
                return "WAIT-LOW"
        elif h60_class == "WAIT":
            if oqs >= 70:
                return "WAIT-HIGH"
            elif oqs >= 50:
                return "WAIT-MID"
            elif oqs >= 30:
                return "WAIT-LOW"
            else:
                return "AVOID"
        else:  # AVOID
            return "AVOID"


def reassess_at_h120(direction: str, entry_state: str,
                     h120_ret: Optional[float], mfe_h120: Optional[float]) -> str:
    """
    Frozen IC v1 H120 reassessment.
    Mirrors reassess_at_h120() in Rust.
    Only LONG WAIT-MID is reassessed; all others pass through.
    """
    if direction == "LONG" and entry_state == "WAIT-MID":
        if h120_ret is None:
            return "WAIT-MID"
        if h120_ret > 0.003:
            return "ENTER-LATE"
        elif h120_ret > 0.0:
            return "WAIT-LATE"
        else:
            return "AVOID-LATE"
    return entry_state


def entry_action(state: str) -> str:
    """Maps entry state to action label."""
    if state in ("ENTER", "WAIT-HIGH", "ENTER-LATE", "WAIT-LATE"):
        return "ACT"
    elif state in ("WAIT-MID", "WAIT-LOW"):
        return "MONITOR"
    else:
        return "AVOID"


# ── Time-safe OQS ─────────────────────────────────────────────────────────────
# The historical OQS in p4_opportunity_dataset.json used adverse_exposure = MAE
# over H300, which is future-dependent. This time-safe version uses only
# features available at 10:15 IST (H60 boundary).

def compute_time_safe_oqs(direction: str, h15_ret: Optional[float],
                           h60_ret: Optional[float], mfe_h60: Optional[float],
                           mae_h60: Optional[float],
                           momentum_persistence: Optional[float]) -> int:
    """
    Time-safe OQS computed from H60-available features only.
    Mirrors the OQS scoring logic from p4_track_b_intraday.py but
    excludes adverse_exposure (H300 MAE) which is not available at 10:15.

    Score components (each 0–25):
      1. early_momentum   — H15 return in direction
      2. mfe_h60_tier     — MFE quality at H60
      3. momentum_persist — fraction of bars moving in direction
      4. mae_h60_control  — absence of large adverse move at H60

    Returns integer 0–100.
    """
    score = 0

    # 1. Early momentum (H15 return in direction)
    if h15_ret is not None:
        signed = h15_ret if direction == "LONG" else -h15_ret
        if signed > 0.005:
            score += 25
        elif signed > 0.002:
            score += 18
        elif signed > 0.0:
            score += 10
        elif signed > -0.002:
            score += 4
        # else 0

    # 2. MFE at H60 tier
    if mfe_h60 is not None:
        if mfe_h60 > 0.010:
            score += 25
        elif mfe_h60 > 0.005:
            score += 18
        elif mfe_h60 > 0.002:
            score += 12
        elif mfe_h60 > 0.0:
            score += 6
        # else 0

    # 3. Momentum persistence (fraction of bars in direction)
    if momentum_persistence is not None:
        if momentum_persistence >= 0.75:
            score += 25
        elif momentum_persistence >= 0.60:
            score += 18
        elif momentum_persistence >= 0.45:
            score += 10
        elif momentum_persistence >= 0.30:
            score += 4
        # else 0

    # 4. MAE control at H60 (smaller adverse move = better)
    if mae_h60 is not None:
        abs_mae = abs(mae_h60)
        if abs_mae < 0.002:
            score += 25
        elif abs_mae < 0.005:
            score += 18
        elif abs_mae < 0.010:
            score += 10
        elif abs_mae < 0.015:
            score += 4
        # else 0

    return min(score, 100)


# ── H60 classification ────────────────────────────────────────────────────────

def classify_h60(direction: str, h60_ret: Optional[float]) -> str:
    """
    Derive H60 classification from the H60 return.
    Mirrors the classification logic used in p4_track_b_intraday.py.
    """
    if h60_ret is None:
        return "WAIT"
    signed = h60_ret if direction == "LONG" else -h60_ret
    if signed > 0.003:
        return "ENTER"
    elif signed > -0.003:
        return "WAIT"
    else:
        return "AVOID"


# ── 1-minute bar utilities ────────────────────────────────────────────────────

def load_bars_for_ticker(ticker_ns: str) -> list[dict]:
    """
    Load 1-minute bars for a ticker.
    Ticker format in ledger: RELIANCE_NS → file: RELIANCE.NS.json
    """
    filename = ticker_ns.replace("_NS", ".NS") + ".json"
    path = BARS_1M_DIR / filename
    if not path.exists():
        # Try without the .NS suffix conversion
        alt = ticker_ns.replace("_", ".") + ".json"
        path = BARS_1M_DIR / alt
        if not path.exists():
            return []
    with open(path) as f:
        bars = json.load(f)
    bars.sort(key=lambda b: b["timestamp"])
    return bars


def bars_for_session(bars: list[dict], session_date: str) -> list[dict]:
    """
    Filter bars to the 8-Sep-2026 session (IST 09:15–15:30).
    """
    open_ts  = int(OPEN_IST.timestamp())
    close_ts = int(CLOSE_IST.timestamp())
    return [b for b in bars if open_ts <= b["timestamp"] <= close_ts]


def compute_path_features(direction: str, entry_price: float,
                           session_bars: list[dict]) -> dict:
    """
    Compute direction-adjusted path returns and MFE/MAE from 1m bars.
    Returns a dict of features keyed by horizon label.

    Bars are indexed from 1 (first bar after open = 09:15 bar).
    H15 = bar 3 (09:17), H30 = bar 6, H60 = bar 12, H120 = bar 24,
    H180 = bar 36, H300 = bar 60.

    All returns are direction-adjusted:
      LONG:  (close - entry) / entry
      SHORT: (entry - close) / entry
    """
    def ret(close: float) -> float:
        if direction == "LONG":
            return (close - entry_price) / entry_price
        else:
            return (entry_price - close) / entry_price

    horizons = {"H15": 3, "H30": 6, "H60": 12, "H120": 24, "H180": 36, "H300": 60}
    result: dict = {}

    for label, n in horizons.items():
        if len(session_bars) >= n:
            result[f"{label.lower()}_ret"] = ret(session_bars[n - 1]["close"])
        else:
            result[f"{label.lower()}_ret"] = None

    # MFE and MAE at H60 (bars 1–12)
    h60_bars = session_bars[:12]
    if h60_bars:
        rets_h60 = [ret(b["close"]) for b in h60_bars]
        result["mfe_h60"] = max(rets_h60)
        result["mae_h60"] = min(rets_h60)
    else:
        result["mfe_h60"] = None
        result["mae_h60"] = None

    # Momentum persistence at H60: fraction of bars moving in direction
    if h60_bars:
        in_direction = sum(1 for r in [ret(b["close"]) for b in h60_bars] if r > 0)
        result["momentum_persistence"] = in_direction / len(h60_bars)
    else:
        result["momentum_persistence"] = None

    return result


def find_exit(direction: str, entry_price: float, adaptive_target: float,
              adaptive_risk: float, session_bars: list[dict],
              start_bar: int = 12) -> tuple[Optional[float], Optional[str], Optional[str]]:
    """
    Walk forward from bar start_bar, checking each bar for target/risk hit.
    Returns (exit_price, exit_time_ist, exit_reason).
    exit_reason: TARGET | RISK | HORIZON
    """
    for bar in session_bars[start_bar:]:
        low  = bar["low"]
        high = bar["high"]
        ts   = datetime.fromtimestamp(bar["timestamp"], tz=IST)
        ts_str = ts.strftime("%H:%M")

        if direction == "SHORT":
            # Target hit if low <= adaptive_target
            if low <= adaptive_target:
                return adaptive_target, ts_str, "TARGET"
            # Risk hit if high >= adaptive_risk
            if high >= adaptive_risk:
                return adaptive_risk, ts_str, "RISK"
        else:  # LONG
            # Target hit if high >= adaptive_target
            if high >= adaptive_target:
                return adaptive_target, ts_str, "TARGET"
            # Risk hit if low <= adaptive_risk
            if low <= adaptive_risk:
                return adaptive_risk, ts_str, "RISK"

    # No exit hit — use H300 close (bar 60) or last available bar
    if len(session_bars) >= 60:
        bar = session_bars[59]
    elif session_bars:
        bar = session_bars[-1]
    else:
        return None, None, "HORIZON"

    ts = datetime.fromtimestamp(bar["timestamp"], tz=IST)
    return bar["close"], ts.strftime("%H:%M"), "HORIZON"


# ── Replay record ─────────────────────────────────────────────────────────────

@dataclass
class ReplayRecord:
    # Identity
    decision_id: str
    ticker: str
    date: str = SEP8_DATE

    # Pre-market upstream decision (04:39 IST — available before market open)
    direction: str = ""
    action_upstream: str = ""
    reference_price: Optional[float] = None
    adaptive_target: Optional[float] = None
    adaptive_risk: Optional[float] = None
    adaptive_horizon_sessions: Optional[float] = None
    rank_score: Optional[float] = None
    target_rate: Optional[float] = None
    evidence_class: str = ""
    degradation_level: str = ""
    sample_size: int = 0
    certification_status: str = ""

    # Time-safe features computed at 10:15 IST (H60 boundary)
    h15_ret: Optional[float] = None
    h30_ret: Optional[float] = None
    h60_ret: Optional[float] = None
    mfe_h60: Optional[float] = None
    mae_h60: Optional[float] = None
    momentum_persistence: Optional[float] = None
    h60_classification: str = ""
    time_safe_oqs: int = 0

    # IC v1 entry decision (10:15 IST)
    entry_state: str = ""
    entry_action: str = ""

    # Paper trade execution
    paper_entry_price: Optional[float] = None   # reference_price used as fill
    paper_exit_price: Optional[float] = None
    exit_time_ist: Optional[str] = None
    exit_reason: str = ""
    realized_return: Optional[float] = None

    # Post-decision observations (H120 onward — NOT inputs to entry decision)
    h120_ret: Optional[float] = None
    h180_ret: Optional[float] = None
    h300_ret: Optional[float] = None
    h120_state: str = ""
    h120_action: str = ""

    # Bars available
    session_bars_available: int = 0
    bars_missing: bool = False


# ── Main replay ───────────────────────────────────────────────────────────────

def run_replay() -> list[ReplayRecord]:
    # Load all 8-Sep ledger entries
    sep8_entries = sorted(LEDGER_DIR.glob("LIVE-005-20260908-*.json"))
    print(f"[replay] 8-Sep ledger entries: {len(sep8_entries)}")

    records: list[ReplayRecord] = []
    missing_bars = 0

    for entry_path in sep8_entries:
        e = json.load(open(entry_path))
        ticker_ns = e.get("ticker", "")
        did = e.get("decision_id", "")

        rec = ReplayRecord(
            decision_id=did,
            ticker=ticker_ns,
            direction=e.get("direction", ""),
            action_upstream=e.get("action", ""),
            reference_price=e.get("reference_price"),
            adaptive_target=e.get("adaptive_target"),
            adaptive_risk=e.get("adaptive_risk"),
            adaptive_horizon_sessions=e.get("adaptive_horizon_sessions"),
            rank_score=e.get("rank_score"),
            target_rate=e.get("target_rate"),
            evidence_class=e.get("evidence_class", ""),
            degradation_level=e.get("degradation_level", ""),
            sample_size=e.get("sample_size", 0),
            certification_status=e.get("certification_status", ""),
        )

        # Load 1m bars
        all_bars = load_bars_for_ticker(ticker_ns)
        session_bars = bars_for_session(all_bars, SEP8_DATE)
        rec.session_bars_available = len(session_bars)

        if len(session_bars) < 12:
            print(f"  [warn] {ticker_ns}: only {len(session_bars)} session bars — skipping IC classification")
            rec.bars_missing = True
            records.append(rec)
            missing_bars += 1
            continue

        # Entry price = reference_price from ledger (pre-market research price)
        entry_price = rec.reference_price
        if entry_price is None or entry_price <= 0:
            print(f"  [warn] {ticker_ns}: no reference_price — skipping")
            rec.bars_missing = True
            records.append(rec)
            continue

        # ── INFORMATION BOUNDARY: 10:15 IST ──────────────────────────────────
        # Compute time-safe features from bars 1–12 only (09:15–10:14 IST)

        path = compute_path_features(rec.direction, entry_price, session_bars)
        rec.h15_ret = path["h15_ret"]
        rec.h30_ret = path["h30_ret"]
        rec.h60_ret = path["h60_ret"]
        rec.mfe_h60 = path["mfe_h60"]
        rec.mae_h60 = path["mae_h60"]
        rec.momentum_persistence = path["momentum_persistence"]

        rec.h60_classification = classify_h60(rec.direction, rec.h60_ret)
        rec.time_safe_oqs = compute_time_safe_oqs(
            rec.direction, rec.h15_ret, rec.h60_ret,
            rec.mfe_h60, rec.mae_h60, rec.momentum_persistence
        )

        # ── IC v1 entry decision ──────────────────────────────────────────────
        rec.entry_state = classify_at_entry(
            rec.direction, rec.time_safe_oqs, rec.h60_classification
        )
        rec.entry_action = entry_action(rec.entry_state)

        # ── Paper trade ───────────────────────────────────────────────────────
        # Entry fill = reference_price (pre-market research price)
        # Exit: walk forward from bar 13 (10:15 IST onward)
        rec.paper_entry_price = entry_price

        if rec.adaptive_target is not None and rec.adaptive_risk is not None:
            exit_price, exit_time, exit_reason = find_exit(
                rec.direction, entry_price,
                rec.adaptive_target, rec.adaptive_risk,
                session_bars, start_bar=12
            )
            rec.paper_exit_price = exit_price
            rec.exit_time_ist = exit_time
            rec.exit_reason = exit_reason

            if exit_price is not None and entry_price > 0:
                if rec.direction == "SHORT":
                    rec.realized_return = (entry_price - exit_price) / entry_price
                else:
                    rec.realized_return = (exit_price - entry_price) / entry_price

        # ── Post-decision observations (NOT inputs to entry decision) ─────────
        rec.h120_ret = path["h120_ret"]
        rec.h180_ret = path["h180_ret"]
        rec.h300_ret = path["h300_ret"]

        # H120 reassessment (uses h120_ret — post-decision observation)
        rec.h120_state = reassess_at_h120(
            rec.direction, rec.entry_state, rec.h120_ret, None
        )
        rec.h120_action = entry_action(rec.h120_state)

        records.append(rec)

    print(f"[replay] Records processed: {len(records)}")
    print(f"[replay] Missing bars:      {missing_bars}")
    return records


# ── Output ────────────────────────────────────────────────────────────────────

def fmt(v) -> str:
    if v is None:
        return ""
    if isinstance(v, float):
        return f"{v:.6f}"
    return str(v)


def write_csv(records: list[ReplayRecord], path: Path) -> None:
    if not records:
        return
    fields = list(asdict(records[0]).keys())
    with open(path, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=fields)
        w.writeheader()
        for r in records:
            row = asdict(r)
            w.writerow({k: fmt(v) for k, v in row.items()})
    print(f"[replay] CSV written: {path}")


def write_json(records: list[ReplayRecord], path: Path) -> None:
    data = [asdict(r) for r in records]
    with open(path, "w") as f:
        json.dump(data, f, indent=2, default=str)
    print(f"[replay] JSON written: {path}")


def print_summary(records: list[ReplayRecord]) -> None:
    act = [r for r in records if r.entry_action == "ACT" and not r.bars_missing]
    avoid = [r for r in records if r.entry_action == "AVOID" and not r.bars_missing]
    monitor = [r for r in records if r.entry_action == "MONITOR" and not r.bars_missing]
    missing = [r for r in records if r.bars_missing]

    print()
    print("=" * 60)
    print("8-Sep-2026 Paper-Trading Replay — Summary")
    print("=" * 60)
    print(f"Total candidates:  {len(records)}")
    print(f"  ACT:             {len(act)}")
    print(f"  MONITOR:         {len(monitor)}")
    print(f"  AVOID:           {len(avoid)}")
    print(f"  Missing bars:    {len(missing)}")
    print()

    if act:
        print("ACT decisions:")
        for r in sorted(act, key=lambda x: x.time_safe_oqs, reverse=True):
            ret_str = f"{r.realized_return*100:+.2f}%" if r.realized_return is not None else "—"
            print(f"  {r.ticker:25} {r.direction:5} OQS={r.time_safe_oqs:3}  "
                  f"H60={r.h60_classification:5}  state={r.entry_state:10}  "
                  f"exit={r.exit_reason:8}  ret={ret_str}")

    print()
    if act:
        completed = [r for r in act if r.realized_return is not None]
        if completed:
            rets = [r.realized_return for r in completed]
            wins = [r for r in completed if r.realized_return > 0]
            print(f"ACT realized returns (N={len(completed)}):")
            print(f"  Mean:    {sum(rets)/len(rets)*100:+.3f}%")
            print(f"  Median:  {sorted(rets)[len(rets)//2]*100:+.3f}%")
            print(f"  Win rate:{len(wins)/len(completed)*100:.1f}% ({len(wins)}/{len(completed)})")

    print()
    print("Entry state distribution:")
    from collections import Counter
    states = Counter(r.entry_state for r in records if not r.bars_missing)
    for state, count in sorted(states.items(), key=lambda x: -x[1]):
        print(f"  {state:15} {count}")

    print()
    print("H60 classification distribution:")
    h60s = Counter(r.h60_classification for r in records if not r.bars_missing)
    for cls, count in sorted(h60s.items(), key=lambda x: -x[1]):
        print(f"  {cls:10} {count}")

    print()
    print("Exit reason distribution (ACT only):")
    exits = Counter(r.exit_reason for r in act)
    for reason, count in sorted(exits.items(), key=lambda x: -x[1]):
        print(f"  {reason:10} {count}")


# ── Entry point ───────────────────────────────────────────────────────────────

if __name__ == "__main__":
    print("[replay] Starting 8-Sep-2026 paper-trading replay...")
    print(f"[replay] Ledger:   {LEDGER_DIR}")
    print(f"[replay] 1m bars:  {BARS_1M_DIR}")
    print()

    records = run_replay()

    OUT_CSV.parent.mkdir(parents=True, exist_ok=True)
    write_csv(records, OUT_CSV)
    write_json(records, OUT_JSON)

    print_summary(records)
    print()
    print("[replay] Done. IC v1, Backtest v2, and Cockpit are untouched.")