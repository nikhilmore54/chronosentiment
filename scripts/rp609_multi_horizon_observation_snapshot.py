#!/usr/bin/env python3
"""
RP-609 multi-horizon daily observation snapshot (read-only).

Bootstraps horizon registry, counts journal activity, optionally registers
example candidates for framework exercise. Never places live orders.

Usage:
  python3 scripts/rp609_multi_horizon_observation_snapshot.py
  python3 scripts/rp609_multi_horizon_observation_snapshot.py --seed-examples
"""

from __future__ import annotations

import argparse
import json
import sys
from datetime import datetime, timezone, timedelta
from pathlib import Path
from typing import Any, Dict, List

_WORKSPACE = Path(__file__).resolve().parent.parent
if str(_WORKSPACE) not in sys.path:
    sys.path.insert(0, str(_WORKSPACE))

from scripts.rp609_indian_retail_horizons import (
    PROTOCOL_ID,
    LIVE_ORDER_AUTHORITY,
    SCHEMA_VERSION,
    append_jsonl,
    candidates_path,
    daily_log_path,
    long_term_path,
    new_candidate,
    new_long_term_hold,
    new_portfolio_build_plan,
    portfolio_path,
    swing_path,
    write_horizon_registry,
)
from scripts.rp609_multi_horizon_engine import MultiHorizonEngine

IST = timezone(timedelta(hours=5, minutes=30))
H0_TOD_LOG = (
    Path(__file__).resolve().parent.parent
    / "artifacts"
    / "rp608_regime_breakout"
    / "prospective_v1"
    / "journals"
    / "NSE_tod_observation_log.jsonl"
)


def _count_jsonl(path: Path) -> int:
    if not path.exists():
        return 0
    n = 0
    with path.open(encoding="utf-8") as f:
        for line in f:
            if line.strip():
                n += 1
    return n


def _load_jsonl(path: Path) -> List[Dict[str, Any]]:
    rows: List[Dict[str, Any]] = []
    if not path.exists():
        return rows
    with path.open(encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                rows.append(json.loads(line))
            except json.JSONDecodeError:
                continue
    return rows


def _h0_link_summary() -> Dict[str, Any]:
    rows = _load_jsonl(H0_TOD_LOG)
    if not rows:
        return {
            "tod_log_rows": 0,
            "latest_session_date": None,
            "note": "RP-608 TOD log not found yet; H0 link empty",
        }
    latest = rows[-1]
    return {
        "tod_log_rows": len(rows),
        "latest_session_date": latest.get("session_date"),
        "latest_ge11_net": (latest.get("ge_11ist") or {}).get("net_pnl_inr"),
        "latest_lt11_net": (latest.get("lt_11ist") or {}).get("net_pnl_inr"),
        "h0_decision_status": latest.get("decision_status"),
        "note": "H0 owned by RP-608; linked for contrast only",
    }


def seed_examples() -> Dict[str, int]:
    """Idempotent-ish demo seeds tagged source=framework_seed."""
    added = {"candidates": 0, "portfolio_plans": 0, "long_term": 0}
    existing = _load_jsonl(candidates_path())
    if not any(r.get("source") == "framework_seed" for r in existing):
        for row in [
            new_candidate(
                horizon_id="H1_SWING",
                symbol="HDFCBANK.NS",
                thesis_tag="swing_mean_reversion_observe",
                reference_price=1650.0,
                invalidation_price=1580.0,
                target_price=1750.0,
                max_sessions=10,
                sector="BANKING",
                notes="Framework seed only — not a live recommendation",
                source="framework_seed",
            ),
            new_candidate(
                horizon_id="H1_SWING",
                symbol="INFY.NS",
                thesis_tag="swing_trend_continuation_observe",
                reference_price=1800.0,
                invalidation_price=1720.0,
                target_price=1950.0,
                max_sessions=15,
                sector="IT",
                notes="Framework seed only — not a live recommendation",
                source="framework_seed",
            ),
        ]:
            append_jsonl(candidates_path(), row)
            added["candidates"] += 1

    plans = _load_jsonl(portfolio_path())
    if not any(r.get("plan_id") == "seed_build_001" for r in plans):
        append_jsonl(
            portfolio_path(),
            new_portfolio_build_plan(
                plan_id="seed_build_001",
                total_capital_inr=500_000.0,
                n_tranches=4,
                legs=[
                    {"symbol": "RELIANCE.NS", "sector": "ENERGY", "target_weight_pct": 8.0},
                    {"symbol": "TCS.NS", "sector": "IT", "target_weight_pct": 8.0},
                    {"symbol": "HDFCBANK.NS", "sector": "BANKING", "target_weight_pct": 8.0},
                    {"symbol": "ITC.NS", "sector": "FMCG", "target_weight_pct": 6.0},
                    {"symbol": "LT.NS", "sector": "INDUSTRIALS", "target_weight_pct": 6.0},
                ],
            ),
        )
        added["portfolio_plans"] += 1

    holds = _load_jsonl(long_term_path())
    if not any(r.get("source") == "framework_seed" for r in holds):
        row = new_long_term_hold(
            symbol="RELIANCE.NS",
            thesis="Long-term earnings power vs price — observation seed only",
            thesis_version="v0_observe",
            entry_price=2800.0,
            quantity=10,
            sector="ENERGY",
        )
        row["source"] = "framework_seed"
        append_jsonl(long_term_path(), row)
        added["long_term"] += 1
    return added


def build_snapshot(*, seed: bool = False, run_eod: bool = True) -> Dict[str, Any]:
    reg = write_horizon_registry()
    seeded = seed_examples() if seed else {"candidates": 0, "portfolio_plans": 0, "long_term": 0}
    eng = MultiHorizonEngine()
    eod = eng.run_eod() if run_eod else None
    snap = {
        "schema_version": SCHEMA_VERSION,
        "protocol_id": PROTOCOL_ID,
        "live_order_authority": LIVE_ORDER_AUTHORITY,
        "decision_status": "OBSERVE_ONLY_NO_CONCLUSION",
        "frozen_rp608_intraday": True,
        "snapshot_utc": datetime.now(timezone.utc).isoformat(),
        "snapshot_ist": datetime.now(IST).isoformat(),
        "registry_path": str(reg),
        "journal_counts": {
            "candidates": _count_jsonl(candidates_path()),
            "swing_trades": _count_jsonl(swing_path()),
            "portfolio_builds": _count_jsonl(portfolio_path()),
            "long_term_holds": _count_jsonl(long_term_path()),
        },
        "book_report": eng.report(),
        "eod": eod,
        "h0_intraday_link": _h0_link_summary(),
        "horizons_enabled_for_observation": [
            "H1_SWING",
            "H2_PORTFOLIO_BUILD",
            "H3_LONG_TERM",
        ],
        "next_collection_focus": [
            "Register real H1 swing candidates with invalidation + max_sessions",
            "Run H2 tranche plans on paper capital with sector/name caps",
            "Attach H3 thesis versions and quarterly review dates",
            "Keep RP-608 TOD multi-day observation running in parallel",
        ],
        "seeded_this_run": seeded,
    }
    # upsert one row per IST calendar date
    out = daily_log_path()
    day = datetime.now(IST).strftime("%Y-%m-%d")
    existing = [
        r
        for r in _load_jsonl(out)
        if r.get("observation_date_ist") != day and r.get("session_date") != day
    ]
    snap["observation_date_ist"] = day
    existing.append(snap)
    out.parent.mkdir(parents=True, exist_ok=True)
    with out.open("w", encoding="utf-8") as f:
        for row in existing:
            f.write(json.dumps(row, default=str) + "\n")
    return snap


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--seed-examples",
        action="store_true",
        help="Insert framework_seed demo candidates/plans/holds (not live advice)",
    )
    ap.add_argument(
        "--skip-eod",
        action="store_true",
        help="Do not run engine EOD marks/tranche fills",
    )
    args = ap.parse_args()
    snap = build_snapshot(seed=args.seed_examples, run_eod=not args.skip_eod)
    print(
        json.dumps(
            {
                "protocol_id": snap["protocol_id"],
                "decision_status": snap["decision_status"],
                "observation_date_ist": snap["observation_date_ist"],
                "journal_counts": snap["journal_counts"],
                "book_report": snap.get("book_report"),
                "eod": snap.get("eod"),
                "h0_link": snap["h0_intraday_link"],
                "seeded_this_run": snap["seeded_this_run"],
                "daily_log": str(daily_log_path()),
            },
            indent=2,
            default=str,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
