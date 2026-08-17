#!/usr/bin/env python3
"""
Stop-Loss Behaviour Analysis — Portfolio Replay v0.2.1

Source: historical_runs/portfolio_continuous_v021_2026-08-16/continuous_ledger.json (IMMUTABLE)

For every Coralys STOP lot, produces:
  - entry / stop / target / duration
  - MAE / MFE / session of MAE / session of MFE
  - gap-through magnitude
  - stop opportunity cost: what happened after the stop
  - classification into 6 evidence categories

Does NOT modify the ledger or regenerate the replay.
Does NOT change Coralys v0 parameters.
"""

import json
import sys
from pathlib import Path
from dataclasses import dataclass, field
from typing import Optional
import statistics

LEDGER_PATH = Path("historical_runs/portfolio_continuous_v021_2026-08-16/continuous_ledger.json")
CACHE_DIR = Path("product_validation/CS-P-006/snapshot/20260814T183851Z_7instrument/yahoo_cache")
OUTPUT_PATH = Path("historical_runs/portfolio_continuous_v021_2026-08-16/stop_loss_analysis.md")
OUTPUT_JSON = Path("historical_runs/portfolio_continuous_v021_2026-08-16/stop_loss_analysis.json")

# ─── Stop classification categories ──────────────────────────────────────────

CATEGORY_GENUINE_ADVERSE = "GENUINE_ADVERSE"
CATEGORY_GAP_THROUGH = "GAP_THROUGH"
CATEGORY_TEMPORARY_EXCURSION = "TEMPORARY_EXCURSION"
CATEGORY_STOP_TOO_TIGHT = "STOP_TOO_TIGHT"
CATEGORY_DIRECTION_FAILURE = "DIRECTION_FAILURE"
CATEGORY_PREMATURE = "PREMATURE_STOP"  # stop before target became reachable

# Thresholds
GAP_THROUGH_THRESHOLD = 0.005   # >0.5% beyond stop = gap-through
RECOVERY_THRESHOLD = 0.005      # price recovered to within 0.5% of entry = temporary excursion
TIGHT_STOP_THRESHOLD = 1.5      # stop distance < 1.5x ATR = too tight
PREMATURE_THRESHOLD = 0.0       # target was reachable after stop = premature


@dataclass
class StopDiagnostic:
    trade_id: str
    instrument: str
    direction: str
    entry_price: float
    stop_price: float
    target_price: float
    stop_pct: float
    target_pct: float
    entry_time: str
    exit_time: str
    exit_price: float
    holding_sessions: int
    allocation_inr: float
    realized_pnl_inr: float
    realized_return: float

    # TradePath fields (from ledger)
    mae_pct: Optional[float] = None
    mfe_pct: Optional[float] = None
    session_of_mae: Optional[int] = None
    session_of_mfe: Optional[int] = None
    approached_stop: bool = False
    approached_target: bool = False
    sessions_observed: int = 0

    # Post-stop analysis (from bar cache)
    post_stop_closes: list = field(default_factory=list)
    post_stop_max_adverse: Optional[float] = None   # worst close after stop (direction-normalized)
    post_stop_max_favorable: Optional[float] = None # best close after stop (direction-normalized)
    target_reachable_after_stop: bool = False
    sessions_to_recovery: Optional[int] = None      # sessions until price recovered to entry
    gap_through_magnitude: Optional[float] = None   # how far beyond stop the exit price was

    # Classification
    category: str = ""
    category_notes: str = ""

    # Opportunity cost
    opportunity_cost_pnl_inr: Optional[float] = None  # P&L if we had NOT stopped (held to horizon)
    opportunity_cost_return: Optional[float] = None


def load_ledger():
    with open(LEDGER_PATH) as f:
        return json.load(f)


def load_bar_cache():
    """Load all Yahoo bar JSON files from the cache directory."""
    cache = {}
    for json_file in CACHE_DIR.glob("*.json"):
        # stem of "HDFCBANK.NS.json" is "HDFCBANK.NS"
        instrument = json_file.name.removesuffix(".json")
        with open(json_file) as f:
            raw = json.load(f)
        bars = []
        for row in raw:
            try:
                bars.append({
                    "timestamp": int(row["timestamp"]),
                    "close": float(row["close"]),
                    "open": float(row["open"]),
                    "high": float(row["high"]),
                    "low": float(row["low"]),
                })
            except (ValueError, KeyError):
                continue
        bars.sort(key=lambda b: b["timestamp"])
        cache[instrument] = bars
    return cache


def parse_ts(ts_str: str) -> int:
    """Parse RFC3339 timestamp to unix seconds (approximate)."""
    from datetime import datetime, timezone
    # Handle +00:00 and Z
    ts_str = ts_str.replace("Z", "+00:00")
    try:
        dt = datetime.fromisoformat(ts_str)
        return int(dt.timestamp())
    except Exception:
        return 0


def bars_after(bars, after_ts: int):
    """Return bars with timestamp > after_ts, sorted ascending."""
    return [b for b in bars if b["timestamp"] > after_ts]


def direction_normalized_return(entry: float, price: float, is_long: bool) -> float:
    if entry <= 0:
        return 0.0
    if is_long:
        return (price - entry) / entry
    else:
        return (entry - price) / entry


def classify_stop(diag: StopDiagnostic) -> tuple[str, str]:
    """
    Classify a stop into one of 6 categories.
    Returns (category, notes).
    """
    is_long = diag.direction == "LONG"
    stop_distance = abs(diag.stop_pct)

    # 1. Gap-through: exit price significantly beyond stop
    if diag.gap_through_magnitude is not None and diag.gap_through_magnitude > GAP_THROUGH_THRESHOLD:
        return (
            CATEGORY_GAP_THROUGH,
            f"Exit price {diag.exit_price:.2f} was {diag.gap_through_magnitude*100:.2f}% beyond stop {diag.stop_price:.2f}. "
            f"Intraday gap prevented clean stop execution."
        )

    # 2. Premature: target was reachable after the stop
    if diag.target_reachable_after_stop:
        return (
            CATEGORY_PREMATURE,
            f"Target {diag.target_price:.2f} ({diag.target_pct*100:.2f}%) was reached after stop exit. "
            f"Stop at session {diag.holding_sessions} prevented capturing the target move."
        )

    # 3. Temporary excursion: price recovered to entry after stop
    if diag.sessions_to_recovery is not None:
        return (
            CATEGORY_TEMPORARY_EXCURSION,
            f"Price recovered to entry level within {diag.sessions_to_recovery} sessions after stop. "
            f"Stop may have been triggered by a temporary adverse excursion."
        )

    # 4. Stop too tight: stop distance < 1.5x MAE of non-stopped trades (heuristic: stop_pct < 2%)
    if stop_distance < 0.02:
        return (
            CATEGORY_STOP_TOO_TIGHT,
            f"Stop distance {stop_distance*100:.2f}% is narrow. "
            f"MAE at stop: {(diag.mae_pct or 0)*100:.2f}%. "
            f"Stop may be too tight for prevailing volatility."
        )

    # 5. Direction failure: price continued adversely after stop, no recovery
    if diag.post_stop_max_adverse is not None and diag.post_stop_max_adverse < -0.03:
        return (
            CATEGORY_DIRECTION_FAILURE,
            f"Price continued {diag.post_stop_max_adverse*100:.2f}% adversely after stop. "
            f"Stop correctly identified a direction failure."
        )

    # 6. Genuine adverse move: significant adverse move, stop was protective
    return (
        CATEGORY_GENUINE_ADVERSE,
        f"Stop at {diag.stop_price:.2f} ({stop_distance*100:.2f}% from entry). "
        f"MAE: {(diag.mae_pct or 0)*100:.2f}%. "
        f"Post-stop adverse: {(diag.post_stop_max_adverse or 0)*100:.2f}%."
    )


def analyze_stop_lot(lot: dict, bars: list) -> StopDiagnostic:
    """Build a StopDiagnostic for a single STOP lot."""
    is_long = lot["direction"] == "LONG"
    entry_ts = parse_ts(lot["entry_time"])
    exit_ts = parse_ts(lot["exit_time"])

    diag = StopDiagnostic(
        trade_id=lot["trade_id"],
        instrument=lot["instrument"],
        direction=lot["direction"],
        entry_price=lot["entry_price"],
        stop_price=lot.get("stop_price") or 0.0,
        target_price=lot["target_price"],
        stop_pct=abs(lot.get("stop_pct") or 0.0),
        target_pct=lot["target_pct"],
        entry_time=lot["entry_time"],
        exit_time=lot["exit_time"],
        exit_price=lot.get("exit_price") or lot["entry_price"],
        holding_sessions=lot.get("holding_sessions") or 0,
        allocation_inr=lot["allocation_inr"],
        realized_pnl_inr=lot.get("realized_pnl_inr") or 0.0,
        realized_return=lot.get("realized_return") or 0.0,
    )

    # TradePath fields
    tp = lot.get("trade_path")
    if tp:
        diag.mae_pct = tp.get("max_adverse_excursion_pct")
        diag.mfe_pct = tp.get("max_favorable_excursion_pct")
        diag.session_of_mae = tp.get("session_of_mae")
        diag.session_of_mfe = tp.get("session_of_mfe")
        diag.approached_stop = tp.get("approached_stop", False)
        diag.approached_target = tp.get("approached_target", False)
        diag.sessions_observed = tp.get("sessions_observed", 0)

    # Gap-through magnitude: how far exit_price is beyond stop_price
    if diag.stop_price > 0:
        if is_long:
            # For LONG: stop is below entry, exit_price should be <= stop_price
            # gap-through = how much further below stop the exit was
            gap = (diag.stop_price - diag.exit_price) / diag.entry_price
        else:
            gap = (diag.exit_price - diag.stop_price) / diag.entry_price
        diag.gap_through_magnitude = max(0.0, gap)

    # Post-stop analysis: bars after exit_ts
    post_bars = bars_after(bars, exit_ts)
    if post_bars:
        post_closes = [b["close"] for b in post_bars]
        diag.post_stop_closes = post_closes[:20]  # cap at 20 sessions

        # Post-stop max adverse and favorable (direction-normalized from entry)
        post_returns = [direction_normalized_return(diag.entry_price, c, is_long) for c in post_closes]
        if post_returns:
            diag.post_stop_max_adverse = min(post_returns)
            diag.post_stop_max_favorable = max(post_returns)

        # Target reachable after stop?
        if is_long:
            diag.target_reachable_after_stop = any(b["high"] >= diag.target_price for b in post_bars[:20])
        else:
            diag.target_reachable_after_stop = any(b["low"] <= diag.target_price for b in post_bars[:20])

        # Sessions to recovery: when did price return to entry level?
        for i, bar in enumerate(post_bars[:20]):
            ret = direction_normalized_return(diag.entry_price, bar["close"], is_long)
            if ret >= 0.0:  # price recovered to entry or better
                diag.sessions_to_recovery = i + 1
                break

        # Opportunity cost: P&L if held to horizon (20 sessions from entry) instead of stopping
        # Find the bar at entry + 20 sessions
        entry_bars_after = [b for b in bars if b["timestamp"] > entry_ts]
        if len(entry_bars_after) >= 20:
            horizon_close = entry_bars_after[19]["close"]
        elif entry_bars_after:
            horizon_close = entry_bars_after[-1]["close"]
        else:
            horizon_close = diag.entry_price

        opp_return = direction_normalized_return(diag.entry_price, horizon_close, is_long)
        diag.opportunity_cost_return = opp_return
        diag.opportunity_cost_pnl_inr = diag.allocation_inr * opp_return

    # Classify
    diag.category, diag.category_notes = classify_stop(diag)

    return diag


def render_report(diagnostics: list[StopDiagnostic], ledger: dict) -> str:
    md = []
    md.append("# Stop-Loss Behaviour Analysis — Portfolio Replay v0.2.1\n")
    md.append("**Source:** `historical_runs/portfolio_continuous_v021_2026-08-16/continuous_ledger.json` (IMMUTABLE)  ")
    md.append("**Purpose:** Understand the 70 Coralys STOP exits — not to tune the stop-loss  ")
    md.append("**Constraint:** Coralys v0 parameters are frozen. This is an observation exercise.\n")

    # Summary
    total = len(diagnostics)
    by_cat = {}
    for d in diagnostics:
        by_cat.setdefault(d.category, []).append(d)

    md.append("## Summary\n")
    md.append(f"Total STOP exits: **{total}**\n")
    md.append("| Category | Count | % | Avg realized return | Avg opportunity cost |\n")
    md.append("|----------|-------|---|---------------------|----------------------|\n")
    for cat, items in sorted(by_cat.items(), key=lambda x: -len(x[1])):
        avg_ret = statistics.mean(d.realized_return for d in items) * 100
        opp_costs = [d.opportunity_cost_return for d in items if d.opportunity_cost_return is not None]
        avg_opp = statistics.mean(opp_costs) * 100 if opp_costs else float("nan")
        md.append(f"| {cat} | {len(items)} | {len(items)/total*100:.0f}% | {avg_ret:+.2f}% | {avg_opp:+.2f}% |\n")

    # Opportunity cost summary
    all_realized = [d.realized_pnl_inr for d in diagnostics]
    all_opp = [d.opportunity_cost_pnl_inr for d in diagnostics if d.opportunity_cost_pnl_inr is not None]
    md.append(f"\n**Total realized P&L from stopped lots:** Rs.{sum(all_realized):+.2f}  \n")
    if all_opp:
        md.append(f"**Total opportunity cost (hold-to-horizon counterfactual):** Rs.{sum(all_opp):+.2f}  \n")
        md.append(f"**Net stop impact vs hold-to-horizon:** Rs.{sum(all_realized) - sum(all_opp):+.2f}  \n")

    # Premature stops detail
    premature = by_cat.get(CATEGORY_PREMATURE, [])
    if premature:
        md.append(f"\n## Premature Stops ({len(premature)}) — Target was reachable after stop\n")
        md.append("These are the most actionable: the stop prevented capturing a move that subsequently occurred.\n\n")
        md.append("| Trade ID | Instrument | Entry | Stop | Target | Exit | Hold | Realized | Opp Cost |\n")
        md.append("|----------|------------|-------|------|--------|------|------|----------|----------|\n")
        for d in sorted(premature, key=lambda x: x.realized_pnl_inr):
            md.append(
                f"| {d.trade_id} | {d.instrument} | {d.entry_price:.2f} | "
                f"{d.stop_price:.2f} ({d.stop_pct*100:.1f}%) | "
                f"{d.target_price:.2f} ({d.target_pct*100:.1f}%) | "
                f"{d.exit_price:.2f} | {d.holding_sessions}s | "
                f"Rs.{d.realized_pnl_inr:+.2f} | "
                f"Rs.{d.opportunity_cost_pnl_inr:+.2f} |\n"
            )

    # Temporary excursion detail
    temp = by_cat.get(CATEGORY_TEMPORARY_EXCURSION, [])
    if temp:
        md.append(f"\n## Temporary Excursion Stops ({len(temp)}) — Price recovered after stop\n")
        md.append("| Trade ID | Instrument | Entry | Stop | Sessions to recovery | Realized | Opp Cost |\n")
        md.append("|----------|------------|-------|------|----------------------|----------|----------|\n")
        for d in sorted(temp, key=lambda x: x.realized_pnl_inr):
            md.append(
                f"| {d.trade_id} | {d.instrument} | {d.entry_price:.2f} | "
                f"{d.stop_price:.2f} | {d.sessions_to_recovery} | "
                f"Rs.{d.realized_pnl_inr:+.2f} | "
                f"Rs.{d.opportunity_cost_pnl_inr:+.2f} |\n"
            )

    # Gap-through detail
    gaps = by_cat.get(CATEGORY_GAP_THROUGH, [])
    if gaps:
        md.append(f"\n## Gap-Through Stops ({len(gaps)}) — Exit price significantly beyond stop\n")
        md.append("| Trade ID | Instrument | Stop | Exit | Gap magnitude | Realized |\n")
        md.append("|----------|------------|------|------|---------------|----------|\n")
        for d in sorted(gaps, key=lambda x: -(x.gap_through_magnitude or 0)):
            md.append(
                f"| {d.trade_id} | {d.instrument} | {d.stop_price:.2f} | "
                f"{d.exit_price:.2f} | {(d.gap_through_magnitude or 0)*100:.2f}% | "
                f"Rs.{d.realized_pnl_inr:+.2f} |\n"
            )

    # Direction failure detail
    failures = by_cat.get(CATEGORY_DIRECTION_FAILURE, [])
    if failures:
        md.append(f"\n## Direction Failure Stops ({len(failures)}) — Stop correctly identified adverse move\n")
        md.append("| Trade ID | Instrument | Entry | Stop | Post-stop adverse | Realized |\n")
        md.append("|----------|------------|-------|------|-------------------|----------|\n")
        for d in sorted(failures, key=lambda x: x.realized_pnl_inr):
            md.append(
                f"| {d.trade_id} | {d.instrument} | {d.entry_price:.2f} | "
                f"{d.stop_price:.2f} | {(d.post_stop_max_adverse or 0)*100:.2f}% | "
                f"Rs.{d.realized_pnl_inr:+.2f} |\n"
            )

    # Genuine adverse
    genuine = by_cat.get(CATEGORY_GENUINE_ADVERSE, [])
    if genuine:
        md.append(f"\n## Genuine Adverse Stops ({len(genuine)})\n")
        md.append("| Trade ID | Instrument | Stop dist | MAE | Post-stop adverse | Realized |\n")
        md.append("|----------|------------|-----------|-----|-------------------|----------|\n")
        for d in sorted(genuine, key=lambda x: x.realized_pnl_inr):
            md.append(
                f"| {d.trade_id} | {d.instrument} | {d.stop_pct*100:.2f}% | "
                f"{(d.mae_pct or 0)*100:.2f}% | {(d.post_stop_max_adverse or 0)*100:.2f}% | "
                f"Rs.{d.realized_pnl_inr:+.2f} |\n"
            )

    # Stop too tight
    tight = by_cat.get(CATEGORY_STOP_TOO_TIGHT, [])
    if tight:
        md.append(f"\n## Stop Too Tight ({len(tight)}) — Stop distance < 2% from entry\n")
        md.append("| Trade ID | Instrument | Stop dist | MAE | Realized |\n")
        md.append("|----------|------------|-----------|-----|----------|\n")
        for d in sorted(tight, key=lambda x: x.stop_pct):
            md.append(
                f"| {d.trade_id} | {d.instrument} | {d.stop_pct*100:.2f}% | "
                f"{(d.mae_pct or 0)*100:.2f}% | Rs.{d.realized_pnl_inr:+.2f} |\n"
            )

    # Per-instrument breakdown
    md.append("\n## Per-Instrument Stop Breakdown\n")
    by_inst = {}
    for d in diagnostics:
        by_inst.setdefault(d.instrument, []).append(d)
    md.append("| Instrument | Stops | Premature | Temp excursion | Gap-through | Direction fail | Genuine | Tight | Total P&L |\n")
    md.append("|------------|-------|-----------|----------------|-------------|----------------|---------|-------|-----------|\n")
    for inst, items in sorted(by_inst.items()):
        cats = [d.category for d in items]
        total_pnl = sum(d.realized_pnl_inr for d in items)
        md.append(
            f"| {inst} | {len(items)} | "
            f"{cats.count(CATEGORY_PREMATURE)} | "
            f"{cats.count(CATEGORY_TEMPORARY_EXCURSION)} | "
            f"{cats.count(CATEGORY_GAP_THROUGH)} | "
            f"{cats.count(CATEGORY_DIRECTION_FAILURE)} | "
            f"{cats.count(CATEGORY_GENUINE_ADVERSE)} | "
            f"{cats.count(CATEGORY_STOP_TOO_TIGHT)} | "
            f"Rs.{total_pnl:+.2f} |\n"
        )

    # Interpretation
    md.append("\n## Interpretation\n\n")
    md.append("The 70 stops are not a single phenomenon. The classification above separates:\n\n")
    md.append("- **Premature stops**: the stop prevented capturing a move that subsequently occurred. "
              "These are the most actionable — they suggest the stop boundary may be too close to entry "
              "relative to the instrument's normal volatility.\n")
    md.append("- **Temporary excursion stops**: the price recovered after the stop. "
              "These suggest the stop was triggered by noise rather than a genuine adverse move.\n")
    md.append("- **Gap-through stops**: the exit price was significantly beyond the stop boundary. "
              "These are execution-quality events, not strategy failures.\n")
    md.append("- **Direction failure stops**: the stop correctly identified a losing trade. "
              "These are the stops working as intended.\n")
    md.append("- **Genuine adverse stops**: the stop was protective and the price did not recover.\n")
    md.append("- **Stop too tight**: the stop distance was very narrow, suggesting ATR-based sizing "
              "may be producing stops that are too close to entry for some instruments.\n\n")
    md.append("**Do not tune Coralys v0 based on this analysis.** "
              "The next step is to expand the universe (v0.3) and observe whether the same "
              "stop behaviour pattern persists across a larger instrument set. "
              "Only then should coralys-exec-v1 be designed.\n")

    return "".join(md)


def main():
    print("Loading ledger...")
    ledger = load_ledger()

    print("Loading bar cache...")
    cache = load_bar_cache()

    coralys_arm = ledger["coralys_arm"]
    stop_lots = [lot for lot in coralys_arm["trade_log"] if lot.get("exit_reason") == "STOP"]
    print(f"Found {len(stop_lots)} STOP lots in Coralys arm")

    diagnostics = []
    for lot in stop_lots:
        inst = lot["instrument"]
        bars = cache.get(inst, [])
        if not bars:
            print(f"  WARNING: no bars for {inst}")
        diag = analyze_stop_lot(lot, bars)
        diagnostics.append(diag)
        opp_str = f"Rs.{diag.opportunity_cost_pnl_inr:+.2f}" if diag.opportunity_cost_pnl_inr is not None else "N/A"
        print(f"  {diag.trade_id}: {diag.category} | realized={diag.realized_pnl_inr:+.2f} | opp_cost={opp_str}")

    # Write report
    report = render_report(diagnostics, ledger)
    OUTPUT_PATH.write_text(report)
    print(f"\nReport written to {OUTPUT_PATH}")

    # Write JSON
    diag_dicts = []
    for d in diagnostics:
        diag_dicts.append({
            "trade_id": d.trade_id,
            "instrument": d.instrument,
            "direction": d.direction,
            "entry_price": d.entry_price,
            "stop_price": d.stop_price,
            "target_price": d.target_price,
            "stop_pct": d.stop_pct,
            "target_pct": d.target_pct,
            "entry_time": d.entry_time,
            "exit_time": d.exit_time,
            "exit_price": d.exit_price,
            "holding_sessions": d.holding_sessions,
            "allocation_inr": d.allocation_inr,
            "realized_pnl_inr": d.realized_pnl_inr,
            "realized_return": d.realized_return,
            "mae_pct": d.mae_pct,
            "mfe_pct": d.mfe_pct,
            "session_of_mae": d.session_of_mae,
            "session_of_mfe": d.session_of_mfe,
            "approached_stop": d.approached_stop,
            "approached_target": d.approached_target,
            "gap_through_magnitude": d.gap_through_magnitude,
            "post_stop_max_adverse": d.post_stop_max_adverse,
            "post_stop_max_favorable": d.post_stop_max_favorable,
            "target_reachable_after_stop": d.target_reachable_after_stop,
            "sessions_to_recovery": d.sessions_to_recovery,
            "opportunity_cost_return": d.opportunity_cost_return,
            "opportunity_cost_pnl_inr": d.opportunity_cost_pnl_inr,
            "category": d.category,
            "category_notes": d.category_notes,
        })
    OUTPUT_JSON.write_text(json.dumps({"stop_diagnostics": diag_dicts}, indent=2))
    print(f"JSON written to {OUTPUT_JSON}")

    # Print summary
    by_cat = {}
    for d in diagnostics:
        by_cat.setdefault(d.category, []).append(d)
    print("\n=== STOP CLASSIFICATION SUMMARY ===")
    for cat, items in sorted(by_cat.items(), key=lambda x: -len(x[1])):
        print(f"  {cat}: {len(items)} ({len(items)/len(diagnostics)*100:.0f}%)")

    total_realized = sum(d.realized_pnl_inr for d in diagnostics)
    total_opp = sum(d.opportunity_cost_pnl_inr for d in diagnostics if d.opportunity_cost_pnl_inr is not None)
    print(f"\nTotal realized P&L from stops: Rs.{total_realized:+.2f}")
    print(f"Hold-to-horizon counterfactual: Rs.{total_opp:+.2f}")
    print(f"Net stop impact: Rs.{total_realized - total_opp:+.2f}")


if __name__ == "__main__":
    main()