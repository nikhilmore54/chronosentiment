#!/usr/bin/env python3
"""
One-trade production reconciliation for NSE_OF_TOUCH_PLUS_STATUTORY_V1.

Reconstructs executed-order legs from a real ledger row (never equal-split),
compares shared calculator vs an independent Zerodha-schedule hand check.

Does not rewrite of_trade_outcomes.jsonl. Does not finalize bulk V1 totals.
"""

from __future__ import annotations

import json
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Mapping, Sequence

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT))

from scripts.rp608_nse_equity_costs import (  # noqa: E402
    calc_indian_equity_friction_from_orders,
    calc_nse_of_paper_accounting,
    executed_orders_from_ledger_trade,
)

DEFAULT_TRADE = (
    ROOT / "tests" / "fixtures" / "nse_of_paper_tatasteel_production_trade.json"
)
OUT_DEFAULT = (
    ROOT
    / "artifacts"
    / "rp608_nse_of_paper"
    / "production_recon_tatasteel_v1.json"
)


def independent_zerodha_intraday_charges(
    executed_orders: Sequence[Mapping[str, Any]],
) -> Dict[str, float]:
    """
    Independent broker-schedule reconstruction.

    Literals only — does not import rates from rp608_nse_equity_costs.
    Published Zerodha equity intraday schedule (as used by V1).
    Rounding: components to 4 dp, statutory total to 2 dp (paise).
    """
    per_order: List[Dict[str, Any]] = []
    buy_turnover = 0.0
    sell_turnover = 0.0
    brokerage = 0.0
    n_buy = n_sell = 0
    for raw in executed_orders:
        side = str(raw.get("side") or "").upper()
        if "turnover" in raw and raw.get("turnover") is not None:
            turnover = float(raw["turnover"])
        else:
            turnover = float(raw.get("price") or 0.0) * float(raw.get("quantity") or 0.0)
        if turnover <= 0:
            continue
        # 0.03% per order, ₹20 cap — applied to THIS order only
        order_brok = min(turnover * 0.03 / 100.0, 20.0)
        brokerage += order_brok
        per_order.append(
            {"side": side, "turnover": round(turnover, 4), "brokerage": round(order_brok, 4)}
        )
        if side == "BUY":
            buy_turnover += turnover
            n_buy += 1
        elif side == "SELL":
            sell_turnover += turnover
            n_sell += 1
        else:
            raise ValueError(f"Unknown side in independent check: {side!r}")

    total_turnover = buy_turnover + sell_turnover
    stt = sell_turnover * 0.025 / 100.0
    exchange = total_turnover * 0.00307 / 100.0
    sebi = total_turnover * 0.0001 / 100.0  # ₹10 / crore
    stamp = buy_turnover * 0.003 / 100.0
    gst = 0.18 * (brokerage + exchange + sebi)
    statutory = brokerage + stt + exchange + sebi + stamp + gst
    return {
        "brokerage": round(brokerage, 4),
        "stt": round(stt, 4),
        "exchange_charges": round(exchange, 4),
        "sebi_charges": round(sebi, 4),
        "stamp_duty": round(stamp, 4),
        "gst": round(gst, 4),
        "statutory_overhead": round(statutory, 2),
        "buy_turnover": round(buy_turnover, 2),
        "sell_turnover": round(sell_turnover, 2),
        "n_buy_orders": n_buy,
        "n_sell_orders": n_sell,
        "per_order_brokerage": per_order,
    }


def reconcile_trade(trade: Mapping[str, Any]) -> Dict[str, Any]:
    legs = executed_orders_from_ledger_trade(trade)
    # Guard: reconstruction must not invent equal-split turnovers.
    assert all(float(o["turnover"]) > 0 for o in legs)

    shared = calc_indian_equity_friction_from_orders(
        legs, include_spread_crossing=False, slippage_bps_per_leg=0.0
    )
    independent = independent_zerodha_intraday_charges(legs)
    acct = calc_nse_of_paper_accounting(
        entry_price=float(trade["entry_price"]),
        exit_price=float(trade["exit_price"]),
        quantity=float(
            trade.get("exit_quantity")
            or trade.get("entry_paper_quantity")
            or trade.get("simulated_quantity")
            or 0.0
        ),
        direction=str(trade.get("direction") or "LONG"),
        executed_orders=legs,
    )

    component_keys = (
        "brokerage",
        "stt",
        "exchange_charges",
        "sebi_charges",
        "stamp_duty",
        "gst",
    )
    deltas = {
        k: round(float(shared[k]) - float(independent[k]), 6) for k in component_keys
    }
    overhead_delta = round(
        float(shared["statutory_overhead"]) - float(independent["statutory_overhead"]), 6
    )
    paper_vs_ind = round(
        float(acct["statutory_overhead_inr"]) - float(independent["statutory_overhead"]), 6
    )
    # Documented rounding: components ≤1e-4; statutory total exact at 2 dp (paise).
    ok = (
        abs(overhead_delta) <= 0.0
        and abs(paper_vs_ind) <= 0.0
        and all(abs(v) <= 0.0001 for v in deltas.values())
    )

    return {
        "of_trade_id": trade.get("of_trade_id"),
        "symbol": trade.get("symbol"),
        "direction": trade.get("direction"),
        "reconstructed_executed_orders": legs,
        "equal_split_used": False,
        "shared_calculator": {k: shared[k] for k in component_keys + ("statutory_overhead",)},
        "independent_broker_schedule": independent,
        "paper_accounting_v1": {
            "touch_gross_pnl_inr": acct["touch_gross_pnl_inr"],
            "statutory_overhead_inr": acct["statutory_overhead_inr"],
            "statutory_net_pnl_inr": acct["statutory_net_pnl_inr"],
            "spread_crossing_charged": acct["spread_crossing_charged"],
        },
        "legacy_journal": {
            "gross_pnl_inr": trade.get("gross_pnl_inr"),
            "friction_inr": trade.get("friction_inr"),
            "net_pnl_inr": trade.get("net_pnl_inr"),
            "note": "Legacy friction includes spread+slip additives; not V1 truth.",
        },
        "deltas_shared_minus_independent": {
            **deltas,
            "statutory_overhead": overhead_delta,
            "paper_statutory_overhead_inr": paper_vs_ind,
        },
        "rounding_policy": {
            "component_dp": 4,
            "statutory_total_dp": 2,
            "component_tolerance": 0.0001,
            "overhead_tolerance_inr": 0.0,
            "note": "Components rounded to 4 dp; statutory overhead / paper totals to 2 dp (paise).",
        },
        "pass": ok,
        "disposition": (
            "PRODUCTION_RECON_PASS"
            if ok
            else "PRODUCTION_RECON_FAIL"
        ),
        "historical_v1_bulk_totals": "PROVISIONAL",
        "generated_utc": datetime.now(timezone.utc).isoformat(),
    }


def main() -> int:
    trade_path = Path(sys.argv[1]) if len(sys.argv) > 1 else DEFAULT_TRADE
    out_path = Path(sys.argv[2]) if len(sys.argv) > 2 else OUT_DEFAULT
    trade = json.loads(trade_path.read_text(encoding="utf-8"))
    report = reconcile_trade(trade)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(report, indent=2))
    return 0 if report["pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
