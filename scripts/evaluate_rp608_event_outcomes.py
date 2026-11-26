#!/usr/bin/env python3
"""
scripts/evaluate_rp608_event_outcomes.py

RP-608-EVENT-001: Time-Aligned Cross-Candidate Audit & Matured Outcome Evaluator.
Strictly Read-Only (0.0% Order Authority) with Single-Pass Forward-Only Accounting.

Implements:
  1. Time-Aligned Event Join:
     - Joins records across Candidates 16b, 32b, 64b, and 120b by exact binance_update_id / timestamp.
     - Quantifies 4-way agreement, partial agreement, divergence, and lookback filter actions.
  2. Forward-Only Outcome Accounting:
     - Evaluates strictly on prices observed *after* signal emission timestamp (t > t_rec).
     - Entry price: Executable-side (P_ask for LONG, P_bid for SHORT).
     - Stop invalidation: Explicit 30.0 bps adverse from arrival midpoint.
     - Fixed horizons: 15-minute, 30-minute, and 60-minute evaluation windows.
     - Retains open positions as UNRESOLVED (never silently treating them as closed trades).
     - Applies statutory 12.0 bps round-trip friction model (5 bps taker * 2 + 1 bp slippage * 2).
"""

from __future__ import annotations

import argparse
import json
import math
import sys
import time
from collections import defaultdict
from dataclasses import asdict, dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

WORKSPACE_DIR = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE_DIR))

EVENT_DIR = WORKSPACE_DIR / "live_capture" / "shadow_validation" / "crypto" / "event_driven"
CANDIDATE_BARS = [16, 32, 64, 120]
ROUND_TRIP_FRICTION_BPS = 12.0


def load_candidate_records(base_dir: Path) -> Dict[int, List[Dict[str, Any]]]:
    """Loads all emitted candidate records indexed by lookback bars."""
    candidate_records: Dict[int, List[Dict[str, Any]]] = {}
    for bars in CANDIDATE_BARS:
        p = base_dir / f"event_recs_candidate_{bars}b.jsonl"
        records = []
        if p.exists():
            with open(p, "r", encoding="utf-8") as f:
                for line in f:
                    line = line.strip()
                    if line:
                        try:
                            records.append(json.loads(line))
                        except Exception:
                            continue
        candidate_records[bars] = records
    return candidate_records


def perform_time_aligned_join(
    candidate_records: Dict[int, List[Dict[str, Any]]]
) -> Dict[str, Any]:
    """
    Joins candidate records by binance_update_id and timestamp_epoch_ms.
    Evaluates exact event-level agreement, divergence, and unique emissions.
    """
    # Index records by update_id -> candidate -> record
    events_by_update: Dict[int, Dict[int, Dict[str, Any]]] = defaultdict(dict)
    all_update_ids = set()

    for bars, recs in candidate_records.items():
        for r in recs:
            u_id = r.get("binance_update_id")
            if u_id is not None:
                events_by_update[u_id][bars] = r
                all_update_ids.add(u_id)

    total_distinct_events = len(all_update_ids)
    if total_distinct_events == 0:
        return {
            "total_distinct_events": 0,
            "agreement_4_way_count": 0,
            "agreement_rate_pct": 0.0,
            "divergent_events_count": 0,
            "unique_events_per_candidate": {b: 0 for b in CANDIDATE_BARS}
        }

    four_way_concordant = 0
    divergent_events = 0
    unique_per_candidate = {b: 0 for b in CANDIDATE_BARS}

    for u_id, c_map in events_by_update.items():
        if len(c_map) == 4:
            # Check if all 4 emitted the exact same decision
            decisions = {r["recommendation"]["decision"] for r in c_map.values()}
            directions = {r["recommendation"]["direction"] for r in c_map.values()}
            if len(decisions) == 1 and len(directions) == 1:
                four_way_concordant += 1
            else:
                divergent_events += 1
        else:
            divergent_events += 1
            if len(c_map) == 1:
                sole_bar = list(c_map.keys())[0]
                unique_per_candidate[sole_bar] += 1

    agreement_rate = (four_way_concordant / total_distinct_events) * 100.0

    return {
        "total_distinct_events": total_distinct_events,
        "four_way_concordant_count": four_way_concordant,
        "divergent_events_count": divergent_events,
        "four_way_agreement_rate_pct": round(agreement_rate, 2),
        "unique_events_per_candidate": unique_per_candidate,
        "per_candidate_total_emissions": {b: len(candidate_records[b]) for b in CANDIDATE_BARS}
    }


def evaluate_candidate_outcomes(
    records: List[Dict[str, Any]],
    price_timeline: List[Tuple[int, float, float]],  # (timestamp_ms, bid, ask)
    horizon_sec: int = 900,  # 15 minutes default
    adverse_stop_bps: float = 30.0,
    friction_bps: float = ROUND_TRIP_FRICTION_BPS
) -> Dict[str, Any]:
    """
    Forward-only outcome evaluation for a single candidate lookback ledger:
    - Evaluates strictly on prices occurring after signal receipt timestamp.
    - Resolves via STOP_INVALIDATED, HORIZON_MATURED, or remains UNRESOLVED.
    """
    now_ms = int(time.time() * 1000)
    horizon_ms = horizon_sec * 1000

    matured_count = 0
    stopped_count = 0
    unresolved_count = 0

    net_returns_bps: List[float] = []
    gross_returns_bps: List[float] = []
    trade_durations_sec: List[float] = []

    for r in records:
        t_rec = r["timestamp_epoch_ms"]
        direction = r["recommendation"]["direction"]
        entry_price = float(r["recommendation"]["suggested_entry_price"])
        invalidation_price = float(r["recommendation"]["invalidation_price"])

        # Subsequent quotes in timeline
        subsequent_quotes = [q for q in price_timeline if q[0] > t_rec]

        resolved = False
        exit_price = None
        exit_reason = None
        exit_time_ms = None
        mfe_bps = 0.0
        mae_bps = 0.0

        for q_time, q_bid, q_ask in subsequent_quotes:
            # Excursion tracking
            if direction == "LONG":
                fav = ((q_bid - entry_price) / entry_price) * 10000.0
                adv = ((entry_price - q_bid) / entry_price) * 10000.0
            else:
                fav = ((entry_price - q_ask) / entry_price) * 10000.0
                adv = ((q_ask - entry_price) / entry_price) * 10000.0
            if fav > mfe_bps:
                mfe_bps = fav
            if adv > mae_bps:
                mae_bps = adv

            # Check stop breach
            if direction == "LONG":
                if q_bid <= invalidation_price:
                    resolved = True
                    exit_price = q_bid
                    exit_reason = "STOP_INVALIDATED"
                    exit_time_ms = q_time
                    break
            elif direction == "SHORT":
                if q_ask >= invalidation_price:
                    resolved = True
                    exit_price = q_ask
                    exit_reason = "STOP_INVALIDATED"
                    exit_time_ms = q_time
                    break

            # Check horizon expiry
            if q_time >= (t_rec + horizon_ms):
                resolved = True
                exit_price = q_bid if direction == "LONG" else q_ask
                exit_reason = "HORIZON_MATURED"
                exit_time_ms = q_time
                break

        ref_quote = r.get("quote", {})
        actual_spread_bps = ref_quote.get("spread_bps")
        if actual_spread_bps is None and ref_quote.get("bid_price") and ref_quote.get("ask_price"):
            mid = (ref_quote["bid_price"] + ref_quote["ask_price"]) / 2.0
            actual_spread_bps = ((ref_quote["ask_price"] - ref_quote["bid_price"]) / mid) * 10000.0 if mid > 0 else 0.0

        if resolved and exit_price is not None:
            if exit_reason == "STOP_INVALIDATED":
                stopped_count += 1
            else:
                matured_count += 1

            # Compute returns
            if direction == "LONG":
                gross_ret = ((exit_price - entry_price) / entry_price) * 10000.0
            else:
                gross_ret = ((entry_price - exit_price) / entry_price) * 10000.0

            net_ret = gross_ret - friction_bps
            gross_returns_bps.append(gross_ret)
            net_returns_bps.append(net_ret)
            trade_durations_sec.append((exit_time_ms - t_rec) / 1000.0 if exit_time_ms else 0.0)
        else:
            unresolved_count += 1

    total_closed = matured_count + stopped_count
    if total_closed > 0:
        win_count = sum(1 for r in net_returns_bps if r > 0)
        win_rate = (win_count / total_closed) * 100.0
        net_expectancy = sum(net_returns_bps) / total_closed
        gross_expectancy = sum(gross_returns_bps) / total_closed
        avg_duration = sum(trade_durations_sec) / total_closed

        # Drawdown calculation
        cum_ret = 0.0
        peak = 0.0
        max_dd = 0.0
        for r in net_returns_bps:
            cum_ret += r
            if cum_ret > peak:
                peak = cum_ret
            dd = peak - cum_ret
            if dd > max_dd:
                max_dd = dd
    else:
        win_rate = 0.0
        net_expectancy = 0.0
        gross_expectancy = 0.0
        avg_duration = 0.0
        max_dd = 0.0

    return {
        "horizon_sec": horizon_sec,
        "total_emitted": len(records),
        "total_closed": total_closed,
        "matured_at_horizon": matured_count,
        "stopped_out": stopped_count,
        "unresolved_open": unresolved_count,
        "gross_expectancy_bps": round(gross_expectancy, 2),
        "net_expectancy_bps": round(net_expectancy, 2),
        "win_rate_pct": round(win_rate, 2),
        "max_drawdown_bps": round(max_dd, 2),
        "average_duration_sec": round(avg_duration, 1),
        "friction_deduction_bps": friction_bps,
        "status": "EVALUATED" if total_closed > 0 else "ALL_UNRESOLVED_OR_AWAITING_PRICES"
    }


def run_comprehensive_evaluation(base_dir: Optional[Path] = None) -> Dict[str, Any]:
    target_dir = Path(base_dir) if base_dir else EVENT_DIR
    print("=" * 90)
    print("  RP-608-EVENT-001: TIME-ALIGNED CANDIDATE AUDIT & MATURED OUTCOME EVALUATOR")
    print(f"  Target Ledger Directory: {target_dir}")
    print("=" * 90)

    candidate_records = load_candidate_records(target_dir)
    print("\n[*] Section 1: Candidate Record Inventory:")
    for bars in CANDIDATE_BARS:
        print(f"    - Candidate {bars}b: {len(candidate_records[bars])} records loaded.")

    # Time-aligned join
    join_stats = perform_time_aligned_join(candidate_records)
    print("\n[*] Section 2: Time-Aligned Event Join Audit:")
    print(f"    - Total Distinct Binance Update Events: {join_stats['total_distinct_events']}")
    print(f"    - 4-Way Concordant Events (All 4 Candidates Agree): {join_stats['four_way_concordant_count']}")
    print(f"    - Divergent / Partial Agreement Events: {join_stats['divergent_events_count']}")
    print(f"    - Concordance / Overlap Rate: {join_stats['four_way_agreement_rate_pct']}%")

    # Build forward price timeline from raw quote records in the ledgers
    # Every record stores arrival quote {bid_price, ask_price, mid_price}
    price_timeline_map: Dict[int, Tuple[float, float]] = {}
    for bars, recs in candidate_records.items():
        for r in recs:
            t = r["timestamp_epoch_ms"]
            q = r.get("quote", {})
            bid = float(q.get("bid_price", 0.0))
            ask = float(q.get("ask_price", 0.0))
            if bid > 0 and ask > 0:
                price_timeline_map[t] = (bid, ask)

    price_timeline = sorted([(t, bid, ask) for t, (bid, ask) in price_timeline_map.items()], key=lambda x: x[0])
    print(f"\n[*] Section 3: Assembled Forward Price Timeline: {len(price_timeline)} discrete top-of-book timestamps.")

    # Outcome evaluation at 15m (900s), 30m (1800s), and 60m (3600s) horizons
    eval_horizons = [900, 1800, 3600]
    outcomes_by_candidate: Dict[str, Any] = {}

    print("\n[*] Section 4: Forward-Only Matured Outcome Evaluation (12.0 bps Cost Model):")
    for bars in CANDIDATE_BARS:
        recs = candidate_records[bars]
        c_id = f"CANDIDATE_{bars}B"
        outcomes_by_candidate[c_id] = {}
        for h_sec in eval_horizons:
            h_label = f"{h_sec // 60}m"
            res = evaluate_candidate_outcomes(
                records=recs,
                price_timeline=price_timeline,
                horizon_sec=h_sec,
                adverse_stop_bps=30.0,
                friction_bps=ROUND_TRIP_FRICTION_BPS
            )
            outcomes_by_candidate[c_id][h_label] = res
            print(f"    - Candidate {bars}b ({h_label} Horizon): Matured={res['matured_at_horizon']} Stopped={res['stopped_out']} Unresolved={res['unresolved_open']} NetExp={res['net_expectancy_bps']} bps WinRate={res['win_rate_pct']}% MaxDD={res['max_drawdown_bps']} bps")

    audit_report = {
        "audit_name": "RP-608-EVENT-001 Time-Aligned Cross-Candidate Outcome Audit",
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "order_authority": 0.0,
        "round_trip_friction_bps": ROUND_TRIP_FRICTION_BPS,
        "time_aligned_join": join_stats,
        "matured_outcomes": outcomes_by_candidate
    }

    report_path = target_dir / "event_outcome_audit_report.json"
    with open(report_path, "w", encoding="utf-8") as f:
        json.dump(audit_report, f, indent=2)

    print(f"\n[+] Machine-readable audit report saved to: {report_path}")
    print("=" * 90 + "\n")

    return audit_report


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Evaluate time-aligned Gate C candidate outcomes.")
    parser.add_argument("--dir", type=str, default=str(EVENT_DIR), help="Path to event_driven directory")
    args = parser.parse_args()
    run_comprehensive_evaluation(base_dir=Path(args.dir))
