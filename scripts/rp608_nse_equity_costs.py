#!/usr/bin/env python3
"""
NSE equity intraday cost helpers (Zerodha-style published schedule).

Charge from *actual executed legs*, not assumed entry=buy.

Brokerage: min(0.03% × order_turnover, ₹20) per executed order.
STT:       0.025% on each SELL turnover.
Stamp:     0.003% on each BUY turnover.
Exchange / SEBI: on sum of all order turnovers.
GST:       18% × (brokerage + exchange + SEBI) only.
"""

from __future__ import annotations

from typing import Any, Dict, List, Mapping, Optional, Sequence, Union

# Statutory schedule as published PERCENT points, then ÷100 for multipliers.
BROKERAGE_PCT = 0.03
STT_SELL_PCT = 0.025
EXCHANGE_PCT = 0.00307
SEBI_PCT = 0.0001             # ₹10 / crore
STAMP_BUY_PCT = 0.003
GST_PCT = 18.0
SLIPPAGE_BPS_PER_LEG_DEFAULT = 2.0

BROKERAGE_RATE = BROKERAGE_PCT / 100.0
BROKERAGE_CAP_PER_ORDER = 20.0
BROKERAGE_CAP_PER_LEG = BROKERAGE_CAP_PER_ORDER
STT_RATE_SELL_LEG = STT_SELL_PCT / 100.0
EXCHANGE_TURNOVER_RATE = EXCHANGE_PCT / 100.0
SEBI_TURNOVER_RATE = SEBI_PCT / 100.0
STAMP_DUTY_BUY_LEG = STAMP_BUY_PCT / 100.0
GST_RATE = GST_PCT / 100.0
SLIPPAGE_RATE_PER_LEG = SLIPPAGE_BPS_PER_LEG_DEFAULT / 10_000.0

OrderLike = Mapping[str, Any]


def _norm_side(side: Any) -> str:
    s = str(side or "").strip().upper()
    if s in ("BUY", "B", "LONG_ENTRY", "SHORT_EXIT"):
        return "BUY"
    if s in ("SELL", "S", "SHORT_ENTRY", "LONG_EXIT"):
        return "SELL"
    raise ValueError(f"Unknown order side: {side!r}")


def _brokerage_on_turnover(turnover: float) -> float:
    v = float(turnover)
    if v <= 0:
        return 0.0
    return min(v * BROKERAGE_RATE, BROKERAGE_CAP_PER_ORDER)


def normalize_executed_orders(executed_orders: Sequence[OrderLike]) -> List[Dict[str, Any]]:
    out: List[Dict[str, Any]] = []
    for raw in executed_orders:
        side = _norm_side(raw.get("side"))
        if "turnover" in raw and raw.get("turnover") is not None:
            turnover = float(raw["turnover"])
        else:
            px = float(raw.get("price") or 0.0)
            qty = float(raw.get("quantity") or 0.0)
            turnover = px * qty
        if turnover <= 0:
            continue
        out.append(
            {
                "side": side,
                "turnover": turnover,
                "brokerage": _brokerage_on_turnover(turnover),
            }
        )
    return out


def calc_indian_equity_friction_from_orders(
    executed_orders: Sequence[OrderLike],
    *,
    include_spread_crossing: bool = False,
    half_spread_bps: float = 0.0,
    slippage_bps_per_leg: float = 0.0,
) -> Dict[str, float]:
    """
    Statutory (+ optional additives) from actual BUY/SELL legs.

    Example:
      [{"side": "BUY", "turnover": 100000.0},
       {"side": "SELL", "turnover": 110000.0}]
    """
    orders = normalize_executed_orders(executed_orders)
    buy_turnover = sum(o["turnover"] for o in orders if o["side"] == "BUY")
    sell_turnover = sum(o["turnover"] for o in orders if o["side"] == "SELL")
    total_turnover = buy_turnover + sell_turnover

    brokerage = sum(float(o["brokerage"]) for o in orders)
    stt = sell_turnover * STT_RATE_SELL_LEG
    exchange_charges = total_turnover * EXCHANGE_TURNOVER_RATE
    sebi_charges = total_turnover * SEBI_TURNOVER_RATE
    gst = GST_RATE * (brokerage + exchange_charges + sebi_charges)
    stamp_duty = buy_turnover * STAMP_DUTY_BUY_LEG
    statutory = brokerage + stt + exchange_charges + sebi_charges + gst + stamp_duty

    if include_spread_crossing and half_spread_bps and float(half_spread_bps) > 0:
        spread_rate = float(half_spread_bps) / 10000.0
        spread_cost = (buy_turnover + sell_turnover) * spread_rate
    else:
        spread_cost = 0.0

    slip_rate = max(0.0, float(slippage_bps_per_leg)) / 10000.0
    # One slip application per executed order turnover (leg).
    slippage_cost = sum(float(o["turnover"]) * slip_rate for o in orders)
    total_friction = statutory + spread_cost + slippage_cost

    return {
        "brokerage": round(brokerage, 4),
        "stt": round(stt, 4),
        "exchange_charges": round(exchange_charges, 4),
        "sebi_charges": round(sebi_charges, 4),
        "gst": round(gst, 4),
        "stamp_duty": round(stamp_duty, 4),
        # INR money totals: paise (2 dp). Component lines keep 4 dp for audit.
        "statutory_overhead": round(statutory, 2),
        "spread_cost": round(spread_cost, 4),
        "slippage_cost": round(slippage_cost, 4),
        "total_friction": round(total_friction, 2),
        "total_turnover": round(total_turnover, 2),
        "buy_turnover": round(buy_turnover, 2),
        "sell_turnover": round(sell_turnover, 2),
        "n_buy_orders": sum(1 for o in orders if o["side"] == "BUY"),
        "n_sell_orders": sum(1 for o in orders if o["side"] == "SELL"),
        "executed_orders": [
            {
                "side": o["side"],
                "turnover": round(float(o["turnover"]), 4),
                "brokerage": round(float(o["brokerage"]), 4),
            }
            for o in orders
        ],
        "include_spread_crossing": bool(include_spread_crossing and spread_cost > 0),
        "slippage_bps_per_leg": float(slippage_bps_per_leg),
        "effective_friction_bps": (
            round((total_friction / buy_turnover) * 10000.0, 2) if buy_turnover > 0 else 0.0
        ),
        "effective_statutory_bps": (
            round((statutory / buy_turnover) * 10000.0, 2) if buy_turnover > 0 else 0.0
        ),
    }


def calc_indian_equity_friction(
    buy_price: float,
    sell_price: float,
    quantity: float,
    half_spread_bps: float = 2.5,
    *,
    include_spread_crossing: bool = True,
    slippage_bps_per_leg: Optional[float] = None,
    n_buy_orders: int = 1,
    n_sell_orders: int = 1,
    buy_order_turnovers: Optional[Sequence[float]] = None,
    sell_order_turnovers: Optional[Sequence[float]] = None,
    executed_orders: Optional[Sequence[OrderLike]] = None,
) -> Dict[str, float]:
    """
    Compatibility wrapper.

    Prefer executed_orders=[{"side","turnover"}, ...].
    Legacy buy_price/sell_price/quantity builds one BUY + one SELL leg (LONG RT).
    """
    if executed_orders is not None:
        slip = 2.0 if slippage_bps_per_leg is None else float(slippage_bps_per_leg)
        return calc_indian_equity_friction_from_orders(
            executed_orders,
            include_spread_crossing=include_spread_crossing,
            half_spread_bps=float(half_spread_bps or 0.0),
            slippage_bps_per_leg=slip,
        )

    if int(n_buy_orders) != 1 and buy_order_turnovers is None:
        raise ValueError(
            "n_buy_orders>1 requires buy_order_turnovers or executed_orders"
        )
    if int(n_sell_orders) != 1 and sell_order_turnovers is None:
        raise ValueError(
            "n_sell_orders>1 requires sell_order_turnovers or executed_orders"
        )

    qty = float(quantity)
    buy_orders = (
        [float(x) for x in buy_order_turnovers if float(x) > 0]
        if buy_order_turnovers is not None
        else [float(buy_price) * qty]
    )
    sell_orders = (
        [float(x) for x in sell_order_turnovers if float(x) > 0]
        if sell_order_turnovers is not None
        else [float(sell_price) * qty]
    )
    legs: List[Dict[str, Any]] = (
        [{"side": "BUY", "turnover": t} for t in buy_orders]
        + [{"side": "SELL", "turnover": t} for t in sell_orders]
    )
    slip = 2.0 if slippage_bps_per_leg is None else float(slippage_bps_per_leg)
    return calc_indian_equity_friction_from_orders(
        legs,
        include_spread_crossing=include_spread_crossing,
        half_spread_bps=float(half_spread_bps or 0.0),
        slippage_bps_per_leg=slip,
    )


def paper_legs_from_direction(
    *,
    direction: str,
    entry_price: float,
    exit_price: float,
    quantity: float,
) -> List[Dict[str, Any]]:
    """
    Map paper entry/exit to actual BUY/SELL legs.

    LONG:  entry BUY @ entry_price, exit SELL @ exit_price
    SHORT: entry SELL @ entry_price, exit BUY @ exit_price
    """
    qty = float(quantity)
    entry = float(entry_price)
    exit_px = float(exit_price)
    d = str(direction or "LONG").strip().upper()
    if d == "LONG":
        return [
            {"side": "BUY", "turnover": entry * qty, "price": entry, "quantity": qty},
            {"side": "SELL", "turnover": exit_px * qty, "price": exit_px, "quantity": qty},
        ]
    if d == "SHORT":
        return [
            {"side": "SELL", "turnover": entry * qty, "price": entry, "quantity": qty},
            {"side": "BUY", "turnover": exit_px * qty, "price": exit_px, "quantity": qty},
        ]
    raise ValueError(f"direction must be LONG or SHORT, got {direction!r}")


def executed_orders_from_ledger_trade(trade: Mapping[str, Any]) -> List[Dict[str, Any]]:
    """
    Reconstruct executed-order turnovers for accounting.

    Priority:
      1. trade["executed_orders"] — already side+turnover (or price×qty)
      2. trade["fills"] / entry_fills+exit_fills — each fill is one order
      3. Single touch RT from entry_price / exit_price × quantity + direction

    Never synthesizes turnovers by dividing combined buy/sell totals by order count.
    """
    if trade.get("executed_orders"):
        return normalize_executed_orders(list(trade["executed_orders"]))

    fills = trade.get("fills")
    if fills:
        return normalize_executed_orders(list(fills))

    entry_fills = list(trade.get("entry_fills") or [])
    exit_fills = list(trade.get("exit_fills") or [])
    if entry_fills or exit_fills:
        return normalize_executed_orders(entry_fills + exit_fills)

    entry = float(trade["entry_price"])
    exit_px = float(trade["exit_price"])
    qty = float(
        trade.get("exit_quantity")
        or trade.get("entry_paper_quantity")
        or trade.get("simulated_quantity")
        or 0.0
    )
    direction = str(trade.get("direction") or "LONG")
    return normalize_executed_orders(
        paper_legs_from_direction(
            direction=direction, entry_price=entry, exit_price=exit_px, quantity=qty
        )
    )


def calc_nse_of_paper_accounting(
    *,
    entry_price: float,
    exit_price: float,
    quantity: float,
    direction: str = "LONG",
    validated_incremental_slippage_inr: float = 0.0,
    sensitivity_slippage_bps_per_leg: float = 2.0,
    executed_orders: Optional[Sequence[OrderLike]] = None,
    buy_order_turnovers: Optional[Sequence[float]] = None,
    sell_order_turnovers: Optional[Sequence[float]] = None,
) -> Dict[str, Any]:
    """
    Touch fills + statutory overhead; direction-aware legs.

    LONG  touch gross = (exit - entry) * qty
    SHORT touch gross = (entry - exit) * qty
    """
    qty = float(quantity)
    entry = float(entry_price)
    exit_px = float(exit_price)
    d = str(direction or "LONG").strip().upper()

    if executed_orders is not None:
        legs = normalize_executed_orders(executed_orders)
    elif buy_order_turnovers is not None or sell_order_turnovers is not None:
        legs = []
        for t in buy_order_turnovers or []:
            legs.append({"side": "BUY", "turnover": float(t)})
        for t in sell_order_turnovers or []:
            legs.append({"side": "SELL", "turnover": float(t)})
        legs = normalize_executed_orders(legs)
    else:
        legs = normalize_executed_orders(
            paper_legs_from_direction(
                direction=d, entry_price=entry, exit_price=exit_px, quantity=qty
            )
        )

    if d == "LONG":
        touch_gross = (exit_px - entry) * qty
    else:
        touch_gross = (entry - exit_px) * qty
    touch_gross_bps = ((touch_gross / (entry * qty)) * 10000.0) if entry > 0 and qty > 0 else 0.0

    statutory = calc_indian_equity_friction_from_orders(
        legs,
        include_spread_crossing=False,
        half_spread_bps=0.0,
        slippage_bps_per_leg=0.0,
    )
    overhead = float(statutory.get("statutory_overhead") or 0.0)
    statutory_net = touch_gross - overhead
    validated_slip = max(0.0, float(validated_incremental_slippage_inr or 0.0))
    all_in_net = statutory_net - validated_slip

    sens = calc_indian_equity_friction_from_orders(
        legs,
        include_spread_crossing=False,
        half_spread_bps=0.0,
        slippage_bps_per_leg=float(sensitivity_slippage_bps_per_leg),
    )
    sens_slip = float(sens.get("slippage_cost") or 0.0)

    return {
        "accounting_standard": "NSE_OF_TOUCH_PLUS_STATUTORY_V1",
        "direction": d,
        "touch_gross_pnl_inr": round(touch_gross, 2),
        "touch_gross_return_bps": round(touch_gross_bps, 2),
        "brokerage": statutory["brokerage"],
        "stt": statutory["stt"],
        "exchange_charges": statutory["exchange_charges"],
        "sebi_charges": statutory["sebi_charges"],
        "stamp_duty": statutory["stamp_duty"],
        "gst": statutory["gst"],
        "statutory_overhead_inr": round(overhead, 2),
        "statutory_net_pnl_inr": round(statutory_net, 2),
        "validated_incremental_slippage_inr": round(validated_slip, 2),
        "all_in_net_pnl_inr": round(all_in_net, 2),
        "sensitivity_2bps_slippage_inr": round(sens_slip, 2),
        "sensitivity_all_in_net_pnl_inr": round(statutory_net - sens_slip, 2),
        "spread_crossing_charged": False,
        "friction": statutory,
        "friction_inr": round(overhead, 2),
        "net_pnl_inr": round(all_in_net, 2),
        "net_return_bps": round(
            (all_in_net / (entry * qty) * 10000.0) if entry > 0 and qty > 0 else 0.0, 2
        ),
        "win_label": 1 if all_in_net > 0 else 0,
        "executed_orders": statutory.get("executed_orders"),
        "buy_order_turnovers": [
            o["turnover"] for o in (statutory.get("executed_orders") or []) if o["side"] == "BUY"
        ],
        "sell_order_turnovers": [
            o["turnover"] for o in (statutory.get("executed_orders") or []) if o["side"] == "SELL"
        ],
    }
