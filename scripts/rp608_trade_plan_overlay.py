#!/usr/bin/env python3
"""
scripts/rp608_trade_plan_overlay.py

RP-608 Trade-Plan Overlay v1: Shadow Execution & Lifecycle State Engine.
Implements Protocol: RP-608-TRADE-PLAN-OVERLAY-001
Strictly Read-Only · Zero Order Authority (0.0%)

Generates and monitors complete trade plans from Policy B recommendations:
  - Eligible Entry: Ask for LONG, Bid for SHORT
  - Stop Loss:  1.0 x ATR(14)
  - Profit Target: 1.5 x ATR(14)
  - Holding Horizon: 60 minutes (3,600s) from simulated entry
  - Net Reward/Risk: Post-12.0 bps round-trip friction and spread crossing
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import sys
import time
import urllib.request
from typing import Any, Dict, List, Optional, Tuple

WORKSPACE_DIR = Path(__file__).resolve().parent.parent
CRYPTO_DIR = WORKSPACE_DIR / "live_capture" / "shadow_validation" / "crypto"
OVERLAY_DIR = CRYPTO_DIR / "trade_plan_overlay_v1"
OVERLAY_DIR.mkdir(parents=True, exist_ok=True)

DECISION_CARDS_FILE = CRYPTO_DIR / "rp608_decision_cards.jsonl"
STATE_FILE = OVERLAY_DIR / "overlay_state.json"
EVENTS_FILE = OVERLAY_DIR / "trade_plan_events.jsonl"
OUTCOMES_FILE = OVERLAY_DIR / "trade_plan_outcomes.jsonl"
CONFIG_FILE = OVERLAY_DIR / "overlay_config.json"

# Fixed Governance Parameters (Protocol RP-608-TRADE-PLAN-OVERLAY-001)
CONFIG: Dict[str, Any] = {
    "protocol_id": "RP-608-TRADE-PLAN-OVERLAY-001",
    "version": "1.2.0",
    "instrument": "BTC/USDT",
    "signal_policy": "B",
    # Signals open on tick bookTicker decisions (RP-608-SHADOW-TICK-001);
    # ATR(14) SL/TP geometry still frozen from the 15m bar buffer at open;
    # TP/SL lifecycle remains continuous L1 (not 15m-gated).
    "lifecycle_monitor": "continuous_l1_bookTicker",
    "signal_cadence": "tick_book_ticker",
    "atr_geometry_source": "completed_15m_atr14",
    "stop_multiplier": 1.0,
    "target_multiplier": 1.5,
    "max_holding_seconds": 3600,  # 60 minutes
    "simulated_notional_usdt": 1000.0,
    "price_tick_size": 0.01,
    "taker_fee_bps": 5.0,
    "slippage_bps": 1.0,
    "round_trip_friction_bps": 12.0,  # 2 * (5.0 + 1.0) bps
    "max_quote_age_ms": 500,
    "order_authority_pct": 0.0,
}

CONFIG_HASH = hashlib.sha256(json.dumps(CONFIG, sort_keys=True).encode("utf-8")).hexdigest()
CONFIG["config_hash"] = CONFIG_HASH


def save_config_if_missing():
    if not CONFIG_FILE.exists():
        with open(CONFIG_FILE, "w", encoding="utf-8") as f:
            json.dump(CONFIG, f, indent=2)


def round_tick(price: float, tick_size: float = 0.01) -> float:
    """Rounds price to nearest instrument tick increment."""
    return round(round(price / tick_size) * tick_size, 2)


def fetch_live_btcusdt_book(timeout_sec: float = 2.0) -> Optional[Dict[str, Any]]:
    """
    Live Binance BTCUSDT bookTicker (REST). Used for TP/SL/MTM — never the
    entry-bar decision-card quote, which goes stale within seconds.
    """
    url = "https://api.binance.com/api/v3/ticker/bookTicker?symbol=BTCUSDT"
    try:
        req = urllib.request.Request(url, headers={"User-Agent": "rp608-trade-plan-overlay/1.0"})
        with urllib.request.urlopen(req, timeout=timeout_sec) as resp:
            data = json.loads(resp.read().decode("utf-8"))
        bid = float(data["bidPrice"])
        ask = float(data["askPrice"])
        if bid <= 0 or ask <= 0 or ask < bid:
            return None
        mid = (bid + ask) / 2.0
        now_ms = int(time.time() * 1000)
        return {
            "best_bid_price": bid,
            "best_ask_price": ask,
            "best_bid_qty": float(data.get("bidQty") or 0.0),
            "best_ask_qty": float(data.get("askQty") or 0.0),
            "spread_bps": ((ask - bid) / mid) * 10000.0 if mid > 0 else 0.0,
            "local_wallclock_utc_ms": now_ms,
            "quote_source": "binance_rest_bookTicker",
        }
    except Exception:
        return None


def normalize_book_quote(quote: Optional[Dict[str, Any]]) -> Optional[Dict[str, Any]]:
    """Accept daemon bookTicker or REST shapes; require valid bid/ask."""
    if not quote:
        return None
    try:
        bid = float(quote.get("best_bid_price") or quote.get("b") or 0.0)
        ask = float(quote.get("best_ask_price") or quote.get("a") or 0.0)
    except (TypeError, ValueError):
        return None
    if bid <= 0.0 or ask <= 0.0 or ask < bid:
        return None
    out = dict(quote)
    out["best_bid_price"] = bid
    out["best_ask_price"] = ask
    return out


def calculate_trade_plan(
    decision_card: Dict[str, Any],
    quote: Optional[Dict[str, Any]] = None,
    notional_usdt: float = 1000.0
) -> Optional[Dict[str, Any]]:
    """
    Formulates a deterministic Trade Plan from a Policy B decision card.
    """
    direction = decision_card.get("policy_b_action")
    if direction not in ("LONG", "SHORT"):
        return None

    atr = float(decision_card.get("atr_14", 0.0))
    ref_price = float(decision_card.get("reference_price", 0.0))
    if atr <= 0.0 or ref_price <= 0.0:
        return None

    # Use execution quote if provided, otherwise fallback to reference price
    exec_quote = quote or decision_card.get("execution_quote") or {}
    bid = float(exec_quote.get("best_bid_price", ref_price))
    ask = float(exec_quote.get("best_ask_price", ref_price))

    # Bid/Ask-aware entry
    if direction == "LONG":
        entry_price = round_tick(ask if ask > 0 else ref_price)
        stop_price = round_tick(entry_price - (CONFIG["stop_multiplier"] * atr))
        target_price = round_tick(entry_price + (CONFIG["target_multiplier"] * atr))
        invalidation_price = stop_price
    else:  # SHORT
        entry_price = round_tick(bid if bid > 0 else ref_price)
        stop_price = round_tick(entry_price + (CONFIG["stop_multiplier"] * atr))
        target_price = round_tick(entry_price - (CONFIG["target_multiplier"] * atr))
        invalidation_price = stop_price

    # Validate price monotonicity
    if direction == "LONG" and not (stop_price < entry_price < target_price):
        return None
    if direction == "SHORT" and not (target_price < entry_price < stop_price):
        return None

    # Simulated Sizing
    qty = round(notional_usdt / entry_price, 6)

    # Distances
    gross_risk_dist = abs(entry_price - stop_price)
    gross_reward_dist = abs(target_price - entry_price)
    gross_risk_usdt = gross_risk_dist * qty
    gross_reward_usdt = gross_reward_dist * qty
    gross_rr_ratio = round(gross_reward_dist / gross_risk_dist, 2) if gross_risk_dist > 0 else 0.0

    # Friction accounting: 6 bps entry + 6 bps exit = 12 bps total on notional
    entry_friction_usdt = round(notional_usdt * (6.0 / 10000.0), 4)
    exit_friction_target_usdt = round((target_price * qty) * (6.0 / 10000.0), 4)
    exit_friction_stop_usdt = round((stop_price * qty) * (6.0 / 10000.0), 4)

    total_target_friction_usdt = round(entry_friction_usdt + exit_friction_target_usdt, 4)
    total_stop_friction_usdt = round(entry_friction_usdt + exit_friction_stop_usdt, 4)

    net_target_profit_usdt = round(gross_reward_usdt - total_target_friction_usdt, 2)
    net_stop_loss_usdt = round(gross_risk_usdt + total_stop_friction_usdt, 2)
    net_rr_ratio = round(net_target_profit_usdt / net_stop_loss_usdt, 2) if net_stop_loss_usdt > 0 else 0.0

    # Timestamps
    signal_close_ms = int(decision_card.get("close_time_ms", time.time() * 1000))
    entry_time_ms = int(exec_quote.get("exchange_event_time_ms", signal_close_ms))
    expiry_time_ms = entry_time_ms + (CONFIG["max_holding_seconds"] * 1000)

    event_idx = decision_card.get("decision_event_idx", decision_card.get("bar_idx", 0))
    plan_id = f"PLAN-{int(event_idx):06d}-{direction}-{entry_time_ms}"

    return {
        "plan_id": plan_id,
        "protocol_id": CONFIG["protocol_id"],
        "config_hash": CONFIG_HASH,
        "signal_cadence": CONFIG.get("signal_cadence", "tick_book_ticker"),
        "source_bar_idx": decision_card.get("bar_idx"),
        "source_decision_event_idx": decision_card.get("decision_event_idx"),
        "source_decision_clock": decision_card.get("decision_clock", CONFIG.get("signal_cadence")),
        "source_timestamp_utc": decision_card.get("timestamp_utc"),
        "direction": direction,
        "reference_price": ref_price,
        "atr_14": atr,
        "entry_price": entry_price,
        "stop_loss_price": stop_price,
        "profit_target_price": target_price,
        "invalidation_price": invalidation_price,
        "simulated_quantity": qty,
        "simulated_notional_usdt": notional_usdt,
        "gross_risk_usdt": round(gross_risk_usdt, 2),
        "gross_reward_usdt": round(gross_reward_usdt, 2),
        "gross_rr_ratio": gross_rr_ratio,
        "round_trip_friction_bps": CONFIG["round_trip_friction_bps"],
        "est_friction_usdt": total_target_friction_usdt,
        "net_target_profit_usdt": net_target_profit_usdt,
        "net_stop_loss_usdt": net_stop_loss_usdt,
        "net_rr_ratio": net_rr_ratio,
        "max_holding_seconds": CONFIG["max_holding_seconds"],
        "entry_time_ms": entry_time_ms,
        "expiry_time_ms": expiry_time_ms,
        "state": "OPEN",
        "data_quality_status": "VALID",
        "order_authority": "0.0% (SIMULATED)",
        "horizon_class": "INTRADAY",
        "horizon_id": "H0_INTRADAY_60M",
        "horizon_label": "Intraday",
        "horizon_term": "≤60 minutes (trade-plan overlay)",
    }


class TradePlanOverlayManager:
    """Manages lifecycle states, state persistence, and outcome recording."""

    def __init__(self, base_dir: Optional[Path] = None):
        self.base_dir = Path(base_dir) if base_dir else OVERLAY_DIR
        self.base_dir.mkdir(parents=True, exist_ok=True)
        self.state_file = self.base_dir / "overlay_state.json"
        self.events_file = self.base_dir / "trade_plan_events.jsonl"
        self.outcomes_file = self.base_dir / "trade_plan_outcomes.jsonl"
        self.config_file = self.base_dir / "overlay_config.json"
        self._save_config_if_missing()
        self.state: Dict[str, Any] = self._load_state()

    def _save_config_if_missing(self):
        if not self.config_file.exists():
            with open(self.config_file, "w", encoding="utf-8") as f:
                json.dump(CONFIG, f, indent=2)

    def _load_state(self) -> Dict[str, Any]:
        if self.state_file.exists():
            try:
                with open(self.state_file, "r", encoding="utf-8") as f:
                    return json.load(f)
            except Exception:
                pass
        return {
            "schema_version": "1.0.0",
            "active_plan": None,
            "total_plans_created": 0,
            "closed_plans_count": 0,
            "suppressed_signals_count": 0,
            "outcomes_summary": {
                "tp_hits": 0,
                "sl_hits": 0,
                "time_exits": 0,
                "ambiguous_hits": 0,
                "cumulative_net_pnl_usdt": 0.0
            }
        }

    def _save_state(self):
        tmp_file = self.state_file.with_suffix(".tmp")
        with open(tmp_file, "w", encoding="utf-8") as f:
            json.dump(self.state, f, indent=2)
            f.flush()
            os.fsync(f.fileno())
        tmp_file.replace(self.state_file)

    def _log_event(self, event_type: str, plan_data: Dict[str, Any], extra: Optional[Dict[str, Any]] = None):
        evt = {
            "timestamp_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
            "timestamp_ms": int(time.time() * 1000),
            "event_type": event_type,
            "plan_id": plan_data.get("plan_id"),
            "direction": plan_data.get("direction"),
            "entry_price": plan_data.get("entry_price"),
            "stop_loss_price": plan_data.get("stop_loss_price"),
            "profit_target_price": plan_data.get("profit_target_price"),
            "state": plan_data.get("state")
        }
        if extra:
            evt.update(extra)
        with open(self.events_file, "a", encoding="utf-8") as f:
            f.write(json.dumps(evt) + "\n")

    def _log_outcome(self, outcome_data: Dict[str, Any]):
        with open(self.outcomes_file, "a", encoding="utf-8") as f:
            f.write(json.dumps(outcome_data) + "\n")

    def process_decision_card(self, card: Dict[str, Any], current_quote: Optional[Dict[str, Any]] = None) -> Optional[Dict[str, Any]]:
        """Evaluates a new decision card for Trade Plan creation or suppression."""
        # Strict Boundary Check: Exclude pre-window startup cards (< 05:30 IST / 1792972800000 ms)
        close_time_ms = int(card.get("close_time_ms", 0))
        if close_time_ms < 1792972800000:
            return None

        direction = card.get("policy_b_action")
        if direction not in ("LONG", "SHORT"):
            # If no new signal, preserve and return existing active plan
            active = self.state.get("active_plan")
            if active and active.get("state") == "OPEN":
                return active
            return None

        # Check concurrency: 1 plan max
        if self.state.get("active_plan"):
            active = self.state["active_plan"]
            if active.get("state") == "OPEN":
                self.state["suppressed_signals_count"] += 1
                self._log_event("SUPPRESSED_OPEN_PLAN", active, {
                    "suppressed_card_bar_idx": card.get("bar_idx"),
                    "suppressed_direction": direction
                })
                self._save_state()
                return active

        # Create new plan
        plan = calculate_trade_plan(card, quote=current_quote)
        if not plan:
            return None

        self.state["active_plan"] = plan
        self.state["total_plans_created"] += 1
        self._log_event("PLAN_CREATED", plan)
        self._save_state()
        return plan

    def evaluate_market_tick(
        self,
        quote: Dict[str, Any],
        *,
        reload_state: bool = True,
        now_ms: Optional[int] = None,
    ) -> Optional[Dict[str, Any]]:
        """
        Monitors open trade plan against live top-of-book (continuous, not 15m).

        Executable-side triggers (never the stale entry-bar decision quote):
          LONG  TP: bid >= target · SL: bid <= stop
          SHORT TP: ask <= target · SL: ask >= stop

        Hit-then-reverse invariant: the first observed touch of TP/SL closes the
        plan immediately. A later reverse inside the same 15m bar cannot unwind
        a fill that already latched. Optional low_price/high_price on the quote
        (e.g. forming kline extremes) count as path-through observations.
        """
        if reload_state:
            self.state = self._load_state()
        plan = self.state.get("active_plan")
        if not plan or plan.get("state") != "OPEN":
            return None

        q = normalize_book_quote(quote)
        if not q:
            return None

        direction = plan["direction"]
        sl = float(plan["stop_loss_price"])
        tp = float(plan["profit_target_price"])
        qty = plan["simulated_quantity"]
        entry_price = plan["entry_price"]
        entry_ms = plan["entry_time_ms"]
        expiry_ms = plan["expiry_time_ms"]

        now_ms = int(now_ms if now_ms is not None else time.time() * 1000)
        bid = float(q["best_bid_price"])
        ask = float(q["best_ask_price"])

        # Optional OHLC extremes (forming kline) — path-through inside the 15m bar.
        low = float(q["low_price"]) if q.get("low_price") is not None else min(bid, ask)
        high = float(q["high_price"]) if q.get("high_price") is not None else max(bid, ask)

        # Track observed executable extremes (audit + reverse-proof latch).
        exec_px = bid if direction == "LONG" else ask
        if direction == "LONG":
            fav = max(bid, high)
            adv = min(bid, low)
            prev_mfe = plan.get("mfe_exec_price")
            prev_mae = plan.get("mae_exec_price")
            plan["mfe_exec_price"] = round_tick(max(float(prev_mfe), fav) if prev_mfe is not None else fav)
            plan["mae_exec_price"] = round_tick(min(float(prev_mae), adv) if prev_mae is not None else adv)
        else:
            fav = min(ask, low)  # favorable for SHORT = lower cover ask
            adv = max(ask, high)
            prev_mfe = plan.get("mfe_exec_price")
            prev_mae = plan.get("mae_exec_price")
            plan["mfe_exec_price"] = round_tick(min(float(prev_mfe), fav) if prev_mfe is not None else fav)
            plan["mae_exec_price"] = round_tick(max(float(prev_mae), adv) if prev_mae is not None else adv)
        plan["last_mark_price"] = round_tick(exec_px)
        plan["last_mark_time_ms"] = now_ms
        plan["lifecycle_monitor"] = CONFIG.get("lifecycle_monitor", "continuous_l1_bookTicker")
        self.state["active_plan"] = plan

        hit_tp = False
        hit_sl = False
        exit_price = 0.0
        terminal_state = None
        exit_reason = None

        if direction == "LONG":
            if bid >= tp or high >= tp or float(plan["mfe_exec_price"]) >= tp:
                hit_tp = True
            if bid <= sl or low <= sl or float(plan["mae_exec_price"]) <= sl:
                hit_sl = True
            exit_price = sl if hit_sl else (tp if hit_tp else bid)
        else:  # SHORT — buy-to-cover on ask
            if ask <= tp or low <= tp or float(plan["mfe_exec_price"]) <= tp:
                hit_tp = True
            if ask >= sl or high >= sl or float(plan["mae_exec_price"]) >= sl:
                hit_sl = True
            exit_price = sl if hit_sl else (tp if hit_tp else ask)

        if hit_tp and hit_sl:
            terminal_state = "CLOSED_SL"
            exit_reason = "AMBIGUOUS_STOP_TARGET"
            self.state["outcomes_summary"]["ambiguous_hits"] += 1
        elif hit_tp:
            terminal_state = "CLOSED_TP"
            exit_reason = "PROFIT_TARGET_HIT"
            self.state["outcomes_summary"]["tp_hits"] += 1
        elif hit_sl:
            terminal_state = "CLOSED_SL"
            exit_reason = "STOP_LOSS_HIT"
            self.state["outcomes_summary"]["sl_hits"] += 1
        elif now_ms >= expiry_ms:
            terminal_state = "TIME_EXIT"
            exit_reason = "MAX_HORIZON_60M_EXPIRED"
            self.state["outcomes_summary"]["time_exits"] += 1

        if terminal_state:
            plan["state"] = terminal_state
            plan["exit_price"] = round_tick(exit_price)
            plan["exit_time_ms"] = now_ms
            plan["holding_duration_seconds"] = round((now_ms - entry_ms) / 1000.0, 1)

            if direction == "LONG":
                gross_pnl = (plan["exit_price"] - entry_price) * qty
            else:
                gross_pnl = (entry_price - plan["exit_price"]) * qty

            friction = (plan["simulated_notional_usdt"] + (plan["exit_price"] * qty)) * (6.0 / 10000.0)
            net_pnl = round(gross_pnl - friction, 2)

            plan["gross_pnl_usdt"] = round(gross_pnl, 2)
            plan["friction_usdt"] = round(friction, 4)
            plan["net_pnl_usdt"] = net_pnl
            plan["exit_reason"] = exit_reason

            self.state["closed_plans_count"] += 1
            self.state["outcomes_summary"]["cumulative_net_pnl_usdt"] += net_pnl
            self.state["last_closed_plan"] = dict(plan)
            self.state["active_plan"] = None  # reverse after latch cannot reopen this plan

            self._log_event(
                terminal_state,
                plan,
                {
                    "exit_price": plan["exit_price"],
                    "net_pnl": net_pnl,
                    "reason": exit_reason,
                    "mfe_exec_price": plan.get("mfe_exec_price"),
                    "mae_exec_price": plan.get("mae_exec_price"),
                    "quote_source": q.get("quote_source"),
                },
            )
            self._log_outcome(plan)
            self._save_state()
            return plan

        return plan

    def compute_mtm_valuation(self, plan: Optional[Dict[str, Any]], current_quote: Optional[Dict[str, Any]], now_ms: Optional[int] = None) -> Dict[str, Any]:
        """Calculates executable-side mark-to-market valuation after friction."""
        if not plan:
            return {}

        now_ms = now_ms if now_ms is not None else int(time.time() * 1000)
        direction = plan.get("direction", "LONG")
        entry_p = float(plan.get("entry_price", 0.0))
        qty = float(plan.get("simulated_quantity", 0.0))
        notional = float(plan.get("simulated_notional_usdt", 1000.0))

        q = current_quote or {}
        bid_p = float(q.get("best_bid_price", entry_p))
        ask_p = float(q.get("best_ask_price", entry_p))

        # Executable exit quote: bid for LONG, ask for SHORT
        exec_exit_p = bid_p if direction == "LONG" else ask_p

        if plan.get("state") in ("CLOSED_TP", "CLOSED_SL", "TIME_EXIT"):
            exec_exit_p = float(plan.get("exit_price", exec_exit_p))
            net_pnl = float(plan.get("net_pnl_usdt", 0.0))
            net_ret_bps = (net_pnl / notional) * 10000.0 if notional > 0 else 0.0
            return {
                "entry_price": entry_p,
                "current_bid": bid_p,
                "current_ask": ask_p,
                "executable_exit_price": exec_exit_p,
                "unrealized_gross_pnl_usdt": plan.get("gross_pnl_usdt", 0.0),
                "unrealized_net_pnl_usdt": net_pnl,
                "unrealized_net_return_bps": round(net_ret_bps, 2),
                "time_remaining_seconds": 0,
                "is_expired": True
            }

        # Open position mark-to-market
        if direction == "LONG":
            gross_pnl = (exec_exit_p - entry_p) * qty
        else:
            gross_pnl = (entry_p - exec_exit_p) * qty

        # Round-trip 12 bps friction on notional
        friction = (notional + (exec_exit_p * qty)) * (6.0 / 10000.0)
        net_pnl = gross_pnl - friction
        net_ret_bps = (net_pnl / notional) * 10000.0 if notional > 0 else 0.0

        time_remaining = max(0, int((plan.get("expiry_time_ms", now_ms) - now_ms) / 1000))

        return {
            "entry_price": entry_p,
            "current_bid": bid_p,
            "current_ask": ask_p,
            "executable_exit_price": exec_exit_p,
            "unrealized_gross_pnl_usdt": round(gross_pnl, 2),
            "unrealized_net_pnl_usdt": round(net_pnl, 2),
            "unrealized_net_return_bps": round(net_ret_bps, 2),
            "time_remaining_seconds": time_remaining,
            "is_expired": (time_remaining == 0)
        }

    def sync_lifecycle(self, current_quote: Optional[Dict[str, Any]] = None, now_ms: Optional[int] = None) -> Dict[str, Any]:
        """
        Reconciles lifecycle state against latest quote and current time.
        Transitions expired or hit plans to terminal states.

        Prefer a live bookTicker; fall back to provided quote only if live fetch fails.
        Never evaluate TP/SL solely against a stale decision-card execution quote when
        a fresher market book is available.
        """
        now_ms = now_ms if now_ms is not None else int(time.time() * 1000)
        self.state = self._load_state()
        active = self.state.get("active_plan")

        live = fetch_live_btcusdt_book()
        eval_quote = normalize_book_quote(live) or normalize_book_quote(current_quote)

        if active and active.get("state") == "OPEN":
            if eval_quote:
                resolved = self.evaluate_market_tick(
                    eval_quote, reload_state=False, now_ms=now_ms
                )
                if resolved and resolved.get("state") != "OPEN":
                    return {
                        "active_plan": None,
                        "display_plan": resolved,
                        "is_active": False,
                        "state": resolved["state"],
                        "valuation": self.compute_mtm_valuation(resolved, eval_quote, now_ms),
                        "quote_source": eval_quote.get("quote_source") or "live_or_provided",
                    }

            # Refresh after tick (evaluate reloads/persists state)
            active = self.state.get("active_plan")
            if not active or active.get("state") != "OPEN":
                last_closed = self.state.get("last_closed_plan")
                return {
                    "active_plan": None,
                    "display_plan": last_closed,
                    "is_active": False,
                    "state": (last_closed or {}).get("state", "NO_PLAN"),
                    "valuation": self.compute_mtm_valuation(last_closed, eval_quote, now_ms) if last_closed else None,
                }

            # Check direct expiry
            if now_ms >= active.get("expiry_time_ms", 0):
                q = eval_quote or {
                    "best_bid_price": active["entry_price"],
                    "best_ask_price": active["entry_price"],
                }
                resolved = self.evaluate_market_tick(q, reload_state=False, now_ms=now_ms)
                return {
                    "active_plan": None,
                    "display_plan": resolved,
                    "is_active": False,
                    "state": resolved.get("state", "TIME_EXIT") if resolved else "TIME_EXIT",
                    "valuation": self.compute_mtm_valuation(resolved, q, now_ms),
                }

            # Still genuinely open — MTM against live book
            return {
                "active_plan": active,
                "display_plan": active,
                "is_active": True,
                "state": "OPEN",
                "valuation": self.compute_mtm_valuation(active, eval_quote, now_ms),
                "quote_source": (eval_quote or {}).get("quote_source") or "provided",
            }

        last_closed = self.state.get("last_closed_plan")
        return {
            "active_plan": None,
            "display_plan": last_closed,
            "is_active": False,
            "state": last_closed.get("state", "NO_PLAN") if last_closed else "NO_PLAN",
            "valuation": self.compute_mtm_valuation(last_closed, eval_quote or current_quote, now_ms) if last_closed else None,
        }


def run_standalone_sync():
    """Reads latest decision card from disk and synchronizes the Trade Plan overlay."""
    mgr = TradePlanOverlayManager()
    if not DECISION_CARDS_FILE.exists():
        print("[-] No decision cards file found.")
        return

    latest_card = None
    with open(DECISION_CARDS_FILE, "r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if line:
                try:
                    latest_card = json.loads(line)
                except Exception:
                    continue

    if not latest_card:
        print("[-] No valid cards in log.")
        return

    print("=" * 80)
    print("  RP-608 TRADE-PLAN OVERLAY v1 — SYNCHRONIZATION")
    print(f"  Source Card: Bar #{latest_card.get('bar_idx')} ({latest_card.get('timestamp_utc')})")
    print("=" * 80)

    plan = mgr.process_decision_card(latest_card)
    if plan:
        print(f"[+] Active Trade Plan: {plan['plan_id']}")
        print(f"    Direction:      {plan['direction']}")
        print(f"    Eligible Entry: ${plan['entry_price']:,.2f}")
        print(f"    Stop Loss:      ${plan['stop_loss_price']:,.2f} (-${plan['gross_risk_usdt']:,.2f})")
        print(f"    Profit Target:  ${plan['profit_target_price']:,.2f} (+${plan['gross_reward_usdt']:,.2f})")
        print(f"    Reward/Risk:    Gross {plan['gross_rr_ratio']}:1 | Net {plan['net_rr_ratio']}:1")
        print(f"    Est Friction:   ${plan['est_friction_usdt']:.2f} ({plan['round_trip_friction_bps']} bps)")
        print(f"    Horizon:        {plan['max_holding_seconds']}s (60m) [Expires: {time.strftime('%H:%M:%S UTC', time.gmtime(plan['expiry_time_ms'] / 1000))}]")
        print(f"    State:          {plan['state']}")
    else:
        print("[*] No active trade plan issued (Policy B is NO_TRADE or criteria unfulfilled).")


if __name__ == "__main__":
    run_standalone_sync()
