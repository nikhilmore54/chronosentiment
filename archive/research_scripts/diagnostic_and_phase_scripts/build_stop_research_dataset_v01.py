#!/usr/bin/env python3
"""
build_stop_research_dataset_v01.py
===================================
Coralys Stop Research Dataset v0.1

PURPOSE
-------
Construct a pure evidence/data layer — one row per Coralys decision (1144 total).
This is NOT a stop-optimisation script. No stop parameters are calculated or tuned.
Stop-policy discovery is a separate experiment after this dataset is validated.

INPUTS
------
  decision_realization_ledger.json          — 1144 decisions, A/B/C realization flags
  v04_1_B_50_1m_equal/continuous_ledger.json  — B trade log (1144 trades, MAE/MFE, state)
  v04_1_C_50_1m_maxlot/continuous_ledger.json — C trade log (728 trades, MAE/MFE, state)
  v04_1_B_50_1m_equal/stop_loss_analysis.json — B stop diagnostics (574 stopped trades)
  v04_1_C_50_1m_maxlot/stop_loss_analysis.json — C stop diagnostics

OUTPUTS
-------
  datasets/stop_research_dataset_v01.json
  datasets/stop_research_dataset_v01.csv
  datasets/DATASET_REPORT.md

OUTCOME STATES (per config)
---------------------------
  not_realized        — decision existed but capital regime prevented realization
  realized_stopped    — realized and exited via stop
  realized_target     — realized and exited via target
  realized_horizon    — realized and exited at horizon (time-based)

DESIGN NOTES
------------
  - A is retained as capital-gating/realization baseline only (A_realized=False for all)
  - B and C supply stop diagnostics where realized
  - All 1144 decisions are included regardless of realization state
  - The join between ledger and stop diagnostics uses trade_id
  - Fields unavailable from existing data (bar cache, ATR, full OHLC path) are
    scaffolded as null — they are placeholders for future enrichment passes
"""

import json
import csv
import os
import sys
from datetime import datetime, timezone
from collections import defaultdict

# ---------------------------------------------------------------------------
# Paths
# ---------------------------------------------------------------------------
BASE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
EXPERIMENT_DIR = os.path.join(
    BASE,
    "historical_runs",
    "portfolio_v04_1_capital_allocation_experiment",
)

LEDGER_PATH = os.path.join(EXPERIMENT_DIR, "decision_realization_ledger.json")

B_DIR = os.path.join(EXPERIMENT_DIR, "v04_1_B_50_1m_equal")
C_DIR = os.path.join(EXPERIMENT_DIR, "v04_1_C_50_1m_maxlot")

B_LEDGER = os.path.join(B_DIR, "continuous_ledger.json")
C_LEDGER = os.path.join(C_DIR, "continuous_ledger.json")
B_STOPS  = os.path.join(B_DIR, "stop_loss_analysis.json")
C_STOPS  = os.path.join(C_DIR, "stop_loss_analysis.json")

OUTPUT_DIR = os.path.join(BASE, "datasets")
os.makedirs(OUTPUT_DIR, exist_ok=True)

OUT_JSON   = os.path.join(OUTPUT_DIR, "stop_research_dataset_v01.json")
OUT_CSV    = os.path.join(OUTPUT_DIR, "stop_research_dataset_v01.csv")
OUT_REPORT = os.path.join(OUTPUT_DIR, "DATASET_REPORT.md")


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
def load_json(path: str) -> dict:
    print(f"  Loading {os.path.relpath(path, BASE)} ...", end=" ", flush=True)
    with open(path, "r", encoding="utf-8") as f:
        data = json.load(f)
    print("OK")
    return data


def outcome_state(exit_reason: str | None) -> str:
    """Map exit_reason string to canonical outcome state."""
    if exit_reason is None:
        return "not_realized"
    mapping = {
        "STOP":    "realized_stopped",
        "TARGET":  "realized_target",
        "HORIZON": "realized_horizon",
    }
    return mapping.get(exit_reason.upper(), f"realized_unknown:{exit_reason}")


def build_trade_index(continuous_ledger: dict) -> tuple[dict, dict]:
    """
    Build two dicts from the continuous ledger:
      - by_decision_id: decision_id → trade record  (for joining to ledger)
      - by_trade_id:    trade_id    → trade record  (for joining to stop diagnostics)

    The continuous ledger has two arms:
      pe2_arm      — execution arm (no stops, 572 trades)
      coralys_arm  — Coralys arm with ATR-TMV stops (1144 trades, trade_id = "coralys-*")

    We read from coralys_arm because:
      - It has all 1144 realized trades
      - Its trade_ids match stop_loss_analysis.json diagnostic keys directly
      - It carries the actual stop prices and stop-based exit reasons
    """
    by_decision: dict[str, dict] = {}
    by_trade:    dict[str, dict] = {}
    arm = continuous_ledger.get("coralys_arm", {})
    for trade in arm.get("trade_log", []):
        did = trade.get("decision_id")
        tid = trade.get("trade_id")
        if did:
            by_decision[did] = trade
        if tid:
            by_trade[tid] = trade
    return by_decision, by_trade


def build_stop_index(stop_analysis: dict) -> dict[str, dict]:
    """
    Build a dict keyed by trade_id → stop diagnostic.
    stop_loss_analysis.json uses 'diagnostics' as the list key.
    """
    index: dict[str, dict] = {}
    for diag in stop_analysis.get("diagnostics", []):
        tid = diag.get("trade_id")
        if tid:
            index[tid] = diag
    return index


def extract_trade_path(trade: dict) -> dict:
    """Extract trade_path fields with safe defaults."""
    tp = trade.get("trade_path") or {}
    return {
        "max_favorable_excursion_pct": tp.get("max_favorable_excursion_pct"),
        "max_adverse_excursion_pct":   tp.get("max_adverse_excursion_pct"),
        "session_of_mfe":              tp.get("session_of_mfe"),
        "session_of_mae":              tp.get("session_of_mae"),
        "lowest_close":                tp.get("lowest_close"),
        "highest_close":               tp.get("highest_close"),
        "approached_stop":             tp.get("approached_stop"),
        "approached_target":           tp.get("approached_target"),
        "sessions_observed":           tp.get("sessions_observed"),
    }


def extract_coralys_state(trade: dict) -> dict:
    """Extract Coralys decision state (trend/momentum/volatility) from sealed_decision."""
    sd = trade.get("sealed_decision") or {}
    state = sd.get("state") or {}
    return {
        "decision_time":  sd.get("decision_time"),
        "coralys_trend":      state.get("trend"),
        "coralys_momentum":   state.get("momentum"),
        "coralys_volatility": state.get("volatility"),
        "coralys_state_hash": state.get("state_hash"),
        "policy_id":          sd.get("policy_id"),
        "horizon_days":       sd.get("horizon_days"),
    }


def extract_stop_diag(diag: dict | None, prefix: str) -> dict:
    """
    Flatten stop diagnostic fields with a config prefix (e.g. 'B_' or 'C_').
    Returns all-null dict if diag is None (trade was not stopped).
    """
    fields = [
        "entry_time", "exit_time", "exit_price", "holding_sessions",
        "realized_pnl_inr", "allocation_inr", "gap_magnitude_pct",
        "post_stop_max_favorable_pct", "target_reached_after_stop",
        "recovered_after_stop_within_5", "continued_adverse_5_sessions",
        "stop_tightness_pct", "counterfactual_pnl_inr", "opportunity_cost_inr",
        "category",
    ]
    if diag is None:
        return {f"{prefix}{f}": None for f in fields}
    return {f"{prefix}{f}": diag.get(f) for f in fields}


# ---------------------------------------------------------------------------
# Main build
# ---------------------------------------------------------------------------
def build_dataset() -> list[dict]:
    print("\n=== Coralys Stop Research Dataset v0.1 — Build ===\n")

    # Load all inputs
    print("Loading inputs:")
    ledger      = load_json(LEDGER_PATH)
    b_cont      = load_json(B_LEDGER)
    c_cont      = load_json(C_LEDGER)
    b_stop_raw  = load_json(B_STOPS)
    c_stop_raw  = load_json(C_STOPS)

    # Build lookup indexes
    print("\nBuilding indexes:")
    b_trades, b_trades_by_tid = build_trade_index(b_cont)
    c_trades, c_trades_by_tid = build_trade_index(c_cont)
    b_stops  = build_stop_index(b_stop_raw)
    c_stops  = build_stop_index(c_stop_raw)

    print(f"  B trade index (by decision_id): {len(b_trades)} entries")
    print(f"  C trade index (by decision_id): {len(c_trades)} entries")
    print(f"  B stop index:  {len(b_stops)} entries")
    print(f"  C stop index:  {len(c_stops)} entries")

    # Process each decision
    print(f"\nProcessing {ledger['n_eligible']} decisions ...")
    records = []
    warnings = []

    for dec in ledger["records"]:
        did       = dec["decision_id"]
        instrument = dec["instrument"]
        direction  = dec["direction"]
        entry_price  = dec["entry_price"]
        target_price = dec["target_price"]
        stop_price   = dec["stop_price"]

        # Config-level realization flags
        cfg_a = dec["configs"].get("v04_1_A_50_5k_equal", {})
        cfg_b = dec["configs"].get("v04_1_B_50_1m_equal", {})
        cfg_c = dec["configs"].get("v04_1_C_50_1m_maxlot", {})

        a_realized    = cfg_a.get("realized", False)
        b_realized    = cfg_b.get("realized", False)
        c_realized    = cfg_c.get("realized", False)
        a_allocation  = cfg_a.get("allocation_inr", 0.0)
        b_allocation  = cfg_b.get("allocation_inr", 0.0)
        c_allocation  = cfg_c.get("allocation_inr", 0.0)

        # Coralys-declared stop distance (from ledger, direction-normalized)
        if direction == "LONG":
            declared_stop_distance_pct = (entry_price - stop_price) / entry_price if entry_price else None
        else:  # SHORT
            declared_stop_distance_pct = (stop_price - entry_price) / entry_price if entry_price else None

        # B trade data
        b_trade = b_trades.get(did)
        b_exit_reason = b_trade.get("exit_reason") if b_trade else None
        b_outcome = outcome_state(b_exit_reason) if b_realized else "not_realized"
        b_path    = extract_trade_path(b_trade) if b_trade else extract_trade_path({})
        b_state   = extract_coralys_state(b_trade) if b_trade else extract_coralys_state({})

        # C trade data
        c_trade = c_trades.get(did)
        c_exit_reason = c_trade.get("exit_reason") if c_trade else None
        c_outcome = outcome_state(c_exit_reason) if c_realized else "not_realized"
        c_path    = extract_trade_path(c_trade) if c_trade else extract_trade_path({})

        # Stop diagnostics — keyed by trade_id, not decision_id
        # trade_id format in stop_loss_analysis: "coralys-INSTRUMENT-seqN"
        # trade_id format in continuous_ledger:  "pe2-INSTRUMENT-seqN"
        # We need to find the matching stop diagnostic via trade_id from the ledger trade
        b_trade_id = b_trade.get("trade_id") if b_trade else None
        c_trade_id = c_trade.get("trade_id") if c_trade else None

        # Stop diagnostics use "coralys-" prefix, ledger uses "pe2-" prefix
        # Stop diagnostics — coralys_arm trade_ids already match stop diagnostic keys directly
        # (both use "coralys-INSTRUMENT-seqN" format — no translation needed)
        b_stop_diag = b_stops.get(b_trade_id) if b_trade_id else None
        c_stop_diag = c_stops.get(c_trade_id) if c_trade_id else None

        # Validate: if B outcome is stopped, we expect a stop diagnostic
        if b_outcome == "realized_stopped" and b_stop_diag is None:
            warnings.append(
                f"WARN: B decision {did} ({instrument}) outcome=stopped but no stop diagnostic found "
                f"(trade_id={b_trade_id})"
            )
        if c_outcome == "realized_stopped" and c_stop_diag is None:
            warnings.append(
                f"WARN: C decision {did} ({instrument}) outcome=stopped but no stop diagnostic found "
                f"(trade_id={c_trade_id})"
            )

        # Realized PnL from continuous ledger (not stop diagnostic — covers all exit types)
        b_realized_pnl = b_trade.get("realized_pnl_inr") if b_trade else None
        c_realized_pnl = c_trade.get("realized_pnl_inr") if c_trade else None
        b_holding_sessions = b_trade.get("holding_sessions") if b_trade else None
        c_holding_sessions = c_trade.get("holding_sessions") if c_trade else None
        b_entry_time = b_trade.get("entry_time") if b_trade else None
        b_exit_time  = b_trade.get("exit_time") if b_trade else None
        c_entry_time = c_trade.get("entry_time") if c_trade else None
        c_exit_time  = c_trade.get("exit_time") if c_trade else None

        # Assemble record
        row: dict = {
            # --- Decision identity ---
            "decision_id":   did,
            "instrument":    instrument,
            "direction":     direction,

            # --- Coralys declared prices ---
            "entry_price":   entry_price,
            "target_price":  target_price,
            "stop_price":    stop_price,
            "declared_stop_distance_pct": declared_stop_distance_pct,

            # --- Coralys decision state (from B sealed_decision; same for C) ---
            **b_state,

            # --- Config A: capital-gating baseline ---
            "A_realized":   a_realized,
            "A_allocation_inr": a_allocation,

            # --- Config B: primary stop regime (EqualWeight) ---
            "B_realized":   b_realized,
            "B_allocation_inr": b_allocation,
            "B_outcome":    b_outcome,
            "B_exit_reason": b_exit_reason,
            "B_entry_time":  b_entry_time,
            "B_exit_time":   b_exit_time,
            "B_holding_sessions": b_holding_sessions,
            "B_realized_pnl_inr": b_realized_pnl,

            # B trade path (MAE/MFE)
            "B_max_favorable_excursion_pct": b_path["max_favorable_excursion_pct"],
            "B_max_adverse_excursion_pct":   b_path["max_adverse_excursion_pct"],
            "B_session_of_mfe":              b_path["session_of_mfe"],
            "B_session_of_mae":              b_path["session_of_mae"],
            "B_lowest_close":                b_path["lowest_close"],
            "B_highest_close":               b_path["highest_close"],
            "B_approached_stop":             b_path["approached_stop"],
            "B_approached_target":           b_path["approached_target"],
            "B_sessions_observed":           b_path["sessions_observed"],

            # B stop diagnostics (null if not stopped)
            **extract_stop_diag(b_stop_diag, "B_stop_"),

            # --- Config C: primary stop regime (MaxPerLot) ---
            "C_realized":   c_realized,
            "C_allocation_inr": c_allocation,
            "C_outcome":    c_outcome,
            "C_exit_reason": c_exit_reason,
            "C_entry_time":  c_entry_time,
            "C_exit_time":   c_exit_time,
            "C_holding_sessions": c_holding_sessions,
            "C_realized_pnl_inr": c_realized_pnl,

            # C trade path (MAE/MFE)
            "C_max_favorable_excursion_pct": c_path["max_favorable_excursion_pct"],
            "C_max_adverse_excursion_pct":   c_path["max_adverse_excursion_pct"],
            "C_session_of_mfe":              c_path["session_of_mfe"],
            "C_session_of_mae":              c_path["session_of_mae"],
            "C_lowest_close":                c_path["lowest_close"],
            "C_highest_close":               c_path["highest_close"],
            "C_approached_stop":             c_path["approached_stop"],
            "C_approached_target":           c_path["approached_target"],
            "C_sessions_observed":           c_path["sessions_observed"],

            # C stop diagnostics (null if not stopped)
            **extract_stop_diag(c_stop_diag, "C_stop_"),

            # --- Scaffolded fields (future enrichment) ---
            # These require bar cache / ATR computation — not available in v0.1
            "atr_at_decision":          None,   # ATR(14) at decision_time
            "stop_distance_atr_ratio":  None,   # declared_stop_distance / ATR
            "mae_atr_ratio_B":          None,   # B MAE / ATR
            "mae_atr_ratio_C":          None,   # C MAE / ATR
            "mfe_atr_ratio_B":          None,   # B MFE / ATR
            "mfe_atr_ratio_C":          None,   # C MFE / ATR
            # Counterfactual stop simulations (ATR-based) — future experiment
            "cf_stop_0atr_outcome_B":   None,
            "cf_stop_025atr_outcome_B": None,
            "cf_stop_05atr_outcome_B":  None,
            "cf_stop_075atr_outcome_B": None,
            "cf_stop_0atr_outcome_C":   None,
            "cf_stop_025atr_outcome_C": None,
            "cf_stop_05atr_outcome_C":  None,
            "cf_stop_075atr_outcome_C": None,
            # Regime labels — future enrichment
            "market_regime":            None,
            "instrument_regime":        None,
        }

        records.append(row)

    print(f"  Built {len(records)} records")
    if warnings:
        print(f"\n  {len(warnings)} warnings:")
        for w in warnings:
            print(f"    {w}")

    return records, warnings


# ---------------------------------------------------------------------------
# Output writers
# ---------------------------------------------------------------------------
def write_json(records: list[dict], path: str) -> None:
    payload = {
        "dataset": "Coralys Stop Research Dataset v0.1",
        "built_at": datetime.now(timezone.utc).isoformat(),
        "n_decisions": len(records),
        "schema_version": "0.1",
        "note": (
            "Pure evidence/data layer. No stop parameters calculated or optimised. "
            "Scaffolded fields (ATR, counterfactuals, regime labels) are null — "
            "reserved for future enrichment passes."
        ),
        "records": records,
    }
    with open(path, "w", encoding="utf-8") as f:
        json.dump(payload, f, indent=2, default=str)
    print(f"  JSON → {os.path.relpath(path, BASE)}")


def write_csv(records: list[dict], path: str) -> None:
    if not records:
        return
    fieldnames = list(records[0].keys())
    with open(path, "w", newline="", encoding="utf-8") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(records)
    print(f"  CSV  → {os.path.relpath(path, BASE)}")


def write_report(records: list[dict], warnings: list[str], path: str) -> None:
    # Compute summary statistics
    n = len(records)

    def count(key, val):
        return sum(1 for r in records if r.get(key) == val)

    def count_not_none(key):
        return sum(1 for r in records if r.get(key) is not None)

    b_outcomes = defaultdict(int)
    c_outcomes = defaultdict(int)
    for r in records:
        b_outcomes[r.get("B_outcome", "unknown")] += 1
        c_outcomes[r.get("C_outcome", "unknown")] += 1

    b_stop_cats = defaultdict(int)
    c_stop_cats = defaultdict(int)
    for r in records:
        cat = r.get("B_stop_category")
        if cat:
            b_stop_cats[cat] += 1
        cat = r.get("C_stop_category")
        if cat:
            c_stop_cats[cat] += 1

    # Coralys state distribution
    trend_dist = defaultdict(int)
    momentum_dist = defaultdict(int)
    volatility_dist = defaultdict(int)
    for r in records:
        if r.get("coralys_trend"):
            trend_dist[r["coralys_trend"]] += 1
        if r.get("coralys_momentum"):
            momentum_dist[r["coralys_momentum"]] += 1
        if r.get("coralys_volatility"):
            volatility_dist[r["coralys_volatility"]] += 1

    now = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M UTC")

    lines = [
        "# Coralys Stop Research Dataset v0.1 — Dataset Report",
        "",
        f"**Built:** {now}  ",
        f"**Total decisions:** {n}  ",
        f"**Schema version:** 0.1  ",
        "",
        "---",
        "",
        "## Purpose",
        "",
        "Pure evidence/data construction layer. One row per Coralys decision.",
        "No stop parameters are calculated or optimised in this script.",
        "Stop-policy discovery is a separate experiment after dataset validation.",
        "",
        "---",
        "",
        "## Config Roles",
        "",
        "| Config | Role | Stop Diagnostics |",
        "|--------|------|-----------------|",
        "| A — ₹5K EqualWeight | Capital-gating / realization baseline | ❌ None (0 realizations) |",
        "| B — ₹1M EqualWeight | Primary stop regime | ✅ Yes |",
        "| C — ₹1M MaxPerLot ₹20K | Primary stop regime | ✅ Yes |",
        "",
        "---",
        "",
        "## Decision Universe",
        "",
        f"- Total certified decisions: **{n}**",
        f"- A realized: **{count('A_realized', True)}** (capital-gating baseline)",
        f"- B realized: **{count('B_realized', True)}**",
        f"- C realized: **{count('C_realized', True)}**",
        "",
        "---",
        "",
        "## Outcome States — Config B",
        "",
        "| Outcome | Count | % |",
        "|---------|-------|---|",
    ]
    for outcome, cnt in sorted(b_outcomes.items()):
        lines.append(f"| {outcome} | {cnt} | {cnt/n*100:.1f}% |")

    lines += [
        "",
        "## Outcome States — Config C",
        "",
        "| Outcome | Count | % |",
        "|---------|-------|---|",
    ]
    for outcome, cnt in sorted(c_outcomes.items()):
        lines.append(f"| {outcome} | {cnt} | {cnt/n*100:.1f}% |")

    lines += [
        "",
        "---",
        "",
        "## Stop Taxonomy — Config B",
        "",
        "| Category | Count |",
        "|----------|-------|",
    ]
    for cat, cnt in sorted(b_stop_cats.items(), key=lambda x: -x[1]):
        lines.append(f"| {cat} | {cnt} |")
    lines.append(f"| **Total stopped** | **{sum(b_stop_cats.values())}** |")

    lines += [
        "",
        "## Stop Taxonomy — Config C",
        "",
        "| Category | Count |",
        "|----------|-------|",
    ]
    for cat, cnt in sorted(c_stop_cats.items(), key=lambda x: -x[1]):
        lines.append(f"| {cat} | {cnt} |")
    lines.append(f"| **Total stopped** | **{sum(c_stop_cats.values())}** |")

    lines += [
        "",
        "---",
        "",
        "## Coralys Decision State Distribution",
        "",
        "*(Sourced from B sealed_decision; identical for C — same Coralys artifact)*",
        "",
        "### Trend",
        "",
        "| Value | Count |",
        "|-------|-------|",
    ]
    for val, cnt in sorted(trend_dist.items()):
        lines.append(f"| {val} | {cnt} |")

    lines += [
        "",
        "### Momentum",
        "",
        "| Value | Count |",
        "|-------|-------|",
    ]
    for val, cnt in sorted(momentum_dist.items()):
        lines.append(f"| {val} | {cnt} |")

    lines += [
        "",
        "### Volatility",
        "",
        "| Value | Count |",
        "|-------|-------|",
    ]
    for val, cnt in sorted(volatility_dist.items()):
        lines.append(f"| {val} | {cnt} |")

    lines += [
        "",
        "---",
        "",
        "## Scaffolded Fields (null in v0.1)",
        "",
        "These fields require bar cache / ATR computation and are reserved for future enrichment:",
        "",
        "| Field | Description |",
        "|-------|-------------|",
        "| `atr_at_decision` | ATR(14) at decision_time |",
        "| `stop_distance_atr_ratio` | Declared stop distance / ATR |",
        "| `mae_atr_ratio_B` / `_C` | MAE / ATR for each config |",
        "| `mfe_atr_ratio_B` / `_C` | MFE / ATR for each config |",
        "| `cf_stop_0atr_outcome_B/C` | Counterfactual: no stop |",
        "| `cf_stop_025atr_outcome_B/C` | Counterfactual: 0.25 ATR stop |",
        "| `cf_stop_05atr_outcome_B/C` | Counterfactual: 0.5 ATR stop |",
        "| `cf_stop_075atr_outcome_B/C` | Counterfactual: 0.75 ATR stop |",
        "| `market_regime` | Market-level regime label |",
        "| `instrument_regime` | Instrument-level regime label |",
        "",
        "---",
        "",
        "## Data Quality",
        "",
        f"- B trades with MAE data: **{count_not_none('B_max_adverse_excursion_pct')}**",
        f"- C trades with MAE data: **{count_not_none('C_max_adverse_excursion_pct')}**",
        f"- B stop diagnostics attached: **{count_not_none('B_stop_category')}**",
        f"- C stop diagnostics attached: **{count_not_none('C_stop_category')}**",
        f"- Coralys state populated: **{count_not_none('coralys_trend')}**",
    ]

    if warnings:
        lines += [
            "",
            "## Build Warnings",
            "",
            f"**{len(warnings)} warnings encountered:**",
            "",
        ]
        for w in warnings:
            lines.append(f"- {w}")

    lines += [
        "",
        "---",
        "",
        "## Next Steps",
        "",
        "1. **Validate** this dataset against known counts (574 B stops, 1144 B realizations, 728 C realizations)",
        "2. **Enrich** with ATR at decision time (requires bar cache)",
        "3. **Compute** ATR-normalised MAE/MFE ratios",
        "4. **Simulate** counterfactual stop policies (0, 0.25, 0.5, 0.75 ATR)",
        "5. **Add** regime labels (market, instrument)",
        "6. **Define** objective function before any stop parameter optimisation",
        "",
        "*Stop-policy discovery is a separate experiment after dataset validation.*",
    ]

    with open(path, "w", encoding="utf-8") as f:
        f.write("\n".join(lines) + "\n")
    print(f"  MD   → {os.path.relpath(path, BASE)}")


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------
if __name__ == "__main__":
    records, warnings = build_dataset()

    print("\nWriting outputs:")
    write_json(records, OUT_JSON)
    write_csv(records, OUT_CSV)
    write_report(records, warnings, OUT_REPORT)

    print(f"\nDone. {len(records)} decisions written to datasets/")
    if warnings:
        print(f"  {len(warnings)} warnings — review DATASET_REPORT.md")
    sys.exit(0)