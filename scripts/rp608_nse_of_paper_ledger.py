#!/usr/bin/env python3
"""
scripts/rp608_nse_of_paper_ledger.py

RP-608-NSE-OF-PAPER-TRADE-001 — NSE Order-Flow PAPER ledger (0.0% authority).

Sibling of crypto RP-608-OF-PAPER-TRADE-001:
  - Entry: RECOMMEND_LONG + L1 OF long support (|imb| ≥ 0.30, micro agree)
  - Exit: reverse OF (SHORT support) after min-hold + opposite persist
  - Independent of Estimand A TP/SL/60m journal
  - Forced flat at mandatory square-off / session end (NSE cash market)

Does NOT change frozen OF ±0.30 classification or live order authority.
"""

from __future__ import annotations

import json
import math
import os
import time
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

from scripts.rp608_order_flow_mark import (
    ORDER_FLOW_EXIT_LABEL,
    ORDER_FLOW_LABEL,
    classify_order_flow_support,
    is_order_flow_exit_signal,
    l1_from_nse_quote,
)
from scripts.rp608_trade_plan_overlay import (
    OF_DEPTH_QUALITY_PROTOCOL_ID,
    compute_of_depth_quality,
)
from scripts.rp608_nse_equity_costs import calc_nse_of_paper_accounting

PROTOCOL_ID = "RP-608-NSE-OF-PAPER-TRADE-001"
LIVE_ORDER_AUTHORITY_PCT = 0.0
CANDLE_15M_MS = 900_000

DEFAULT_CONFIG: Dict[str, Any] = {
    "protocol_id": PROTOCOL_ID,
    "execution_mode": "PAPER",
    "live_order_authority": LIVE_ORDER_AUTHORITY_PCT,
    # Anti-churn (wall-clock; daemon polls ~15s including mid-bar)
    "of_paper_min_hold_ms": 300_000,
    "of_paper_opposite_persist_ms": 30_000,
    "of_paper_one_per_15m_bar": True,
    # PAPER open quality (does not change OF ±0.30 mark)
    "of_paper_reject_thin_opposite": True,
    "of_paper_min_entry_signal_qty": 10.0,  # shares (NSE)
    "of_paper_min_same_side_notional_inr": 5_000.0,
    # Size from entry-side L1
    "of_paper_participation_pct": 0.10,
    "of_paper_participation_pct_min": 0.05,
    "of_paper_participation_pct_max": 0.15,
    "of_paper_min_notional_inr": 2_500.0,
    "of_paper_max_notional_inr": 50_000.0,
    "of_paper_fallback_notional_inr": 10_000.0,
    "of_paper_fallback_qty": 1.0,
}


def _iso_utc_ms(ms: int) -> str:
    return time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime(ms / 1000.0))


def quote_as_book_dict(quote: Optional[Any]) -> Dict[str, Any]:
    """Normalize NSE TopOfBook / dict into compute_of_depth_quality bookTicker shape."""
    if quote is None:
        return {}
    if isinstance(quote, dict):
        return {
            "best_bid_price": quote.get("best_bid_price", quote.get("bid_price", quote.get("bidPrice"))),
            "best_ask_price": quote.get("best_ask_price", quote.get("ask_price", quote.get("askPrice"))),
            "best_bid_qty": quote.get("best_bid_qty", quote.get("bid_qty", quote.get("bidQty"))),
            "best_ask_qty": quote.get("best_ask_qty", quote.get("ask_qty", quote.get("askQty"))),
            "half_spread_bps": quote.get("half_spread_bps"),
        }
    return {
        "best_bid_price": getattr(quote, "bid_price", None),
        "best_ask_price": getattr(quote, "ask_price", None),
        "best_bid_qty": getattr(quote, "bid_qty", None),
        "best_ask_qty": getattr(quote, "ask_qty", None),
        "half_spread_bps": getattr(quote, "half_spread_bps", None),
    }


def _atomic_write_json(path: Path, payload: Dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_name(f"{path.stem}.{os.getpid()}.{time.time_ns()}.tmp")
    try:
        with open(tmp, "w", encoding="utf-8") as f:
            json.dump(payload, f, indent=2)
            f.flush()
            os.fsync(f.fileno())
        os.replace(str(tmp), str(path))
    finally:
        if tmp.exists():
            try:
                tmp.unlink()
            except OSError:
                pass


def build_nse_of_candle_mark(
    *,
    now_ms: int,
    bar_idx: Optional[int] = None,
    decision_event_idx: Optional[int] = None,
) -> Dict[str, Any]:
    open_ms = (int(now_ms) // CANDLE_15M_MS) * CANDLE_15M_MS
    close_ms = open_ms + CANDLE_15M_MS
    b_idx = int(bar_idx) if bar_idx is not None else None
    e_idx = int(decision_event_idx) if decision_event_idx is not None else None
    label = (f"15m #{b_idx}" if b_idx is not None else "15m") + (
        f" {_iso_utc_ms(open_ms)[11:16]}–{_iso_utc_ms(close_ms)[11:16]} UTC"
    )
    return {
        "bar_idx": b_idx,
        "decision_event_idx": e_idx,
        "candle_open_time_ms": open_ms,
        "candle_close_time_ms": close_ms,
        "candle_open_utc": _iso_utc_ms(open_ms),
        "candle_close_utc": _iso_utc_ms(close_ms),
        "mark_time_ms": int(now_ms),
        "mark_time_utc": _iso_utc_ms(int(now_ms)),
        "candle_label": label,
    }


def size_nse_of_paper_from_entry_signal(
    *,
    entry_price: float,
    quote: Optional[Any],
    config: Optional[Dict[str, Any]] = None,
) -> Dict[str, Any]:
    """LONG entry signal = bid qty; paper qty = participation of signal, INR clamps."""
    cfg = {**DEFAULT_CONFIG, **(config or {})}
    l1 = l1_from_nse_quote(quote)
    try:
        bid_qty = float(l1.get("bid_qty") or 0.0)
    except (TypeError, ValueError):
        bid_qty = 0.0
    try:
        ask_qty = float(l1.get("ask_qty") or 0.0)
    except (TypeError, ValueError):
        ask_qty = 0.0
    try:
        px = float(entry_price)
    except (TypeError, ValueError):
        px = 0.0

    signal_qty = bid_qty
    pct = float(cfg["of_paper_participation_pct"])
    pct = min(
        float(cfg["of_paper_participation_pct_max"]),
        max(float(cfg["of_paper_participation_pct_min"]), pct),
    )
    raw_qty = signal_qty * pct if signal_qty > 0 else 0.0
    notional = raw_qty * px if px > 0 else 0.0
    min_n = float(cfg["of_paper_min_notional_inr"])
    max_n = float(cfg["of_paper_max_notional_inr"])
    source = "participation"

    if signal_qty <= 0 or px <= 0:
        notional = float(cfg["of_paper_fallback_notional_inr"])
        qty = float(cfg["of_paper_fallback_qty"])
        if px > 0:
            qty = max(1.0, math.floor(notional / px))
            notional = qty * px
        source = "fallback"
    else:
        if notional < min_n:
            qty = max(1.0, math.floor(min_n / px)) if px > 0 else 1.0
            notional = qty * px
            source = "clamped_min_notional"
        elif notional > max_n:
            qty = max(1.0, math.floor(max_n / px)) if px > 0 else 1.0
            notional = qty * px
            source = "clamped_max_notional"
        else:
            qty = max(1.0, math.floor(raw_qty))
            notional = qty * px
            source = "participation"

    return {
        "entry_signal_qty": round(signal_qty, 4),
        "entry_bid_qty": round(bid_qty, 4),
        "entry_ask_qty": round(ask_qty, 4),
        "participation_pct": pct,
        "participation_pct_min": float(cfg["of_paper_participation_pct_min"]),
        "participation_pct_max": float(cfg["of_paper_participation_pct_max"]),
        "simulated_quantity": float(qty),
        "simulated_notional_inr": round(float(notional), 2),
        "sizing_source": source,
    }


class NseOfPaperLedger:
    """Multi-symbol NSE OF PAPER ledger (at most one open per symbol)."""

    def __init__(self, base_dir: Path, *, friction_fn=None, config: Optional[Dict[str, Any]] = None):
        self.base_dir = Path(base_dir)
        self.base_dir.mkdir(parents=True, exist_ok=True)
        self.ledger_dir = self.base_dir / "of_paper_ledger_v1"
        self.ledger_dir.mkdir(parents=True, exist_ok=True)
        self.opens_file = self.ledger_dir / "of_paper_trade_opens.jsonl"
        self.outcomes_file = self.ledger_dir / "of_trade_outcomes.jsonl"
        self.open_state_file = self.ledger_dir / "of_paper_trades_open.json"
        self.state_file = self.ledger_dir / "of_paper_state.json"
        self.config = {**DEFAULT_CONFIG, **(config or {})}
        self._friction_fn = friction_fn
        self.state = self._load_state()
        self._opens: Dict[str, Dict[str, Any]] = self._load_opens()
        self._reconcile_closed_ghosts()
        self._persist_opens()
        self._save_state()

    def _load_state(self) -> Dict[str, Any]:
        if self.state_file.exists():
            try:
                return json.loads(self.state_file.read_text(encoding="utf-8"))
            except Exception:
                pass
        return {
            "protocol_id": PROTOCOL_ID,
            "execution_mode": "PAPER",
            "live_order_authority": LIVE_ORDER_AUTHORITY_PCT,
            "enabled": True,
            "of_outcomes_summary": self._empty_summary(),
            "last_of_paper_entry_bar_by_symbol": {},
            "last_suppressed": None,
            "last_closed": None,
        }

    @staticmethod
    def _empty_summary() -> Dict[str, Any]:
        return {
            "accounting_standard": "NSE_OF_TOUCH_PLUS_STATUTORY_V1",
            "n_trades": 0,
            "n_wins": 0,
            "n_losses": 0,
            "cumulative_gross_pnl_inr": 0.0,  # touch gross
            "cumulative_statutory_overhead_inr": 0.0,
            "cumulative_statutory_net_pnl_inr": 0.0,
            "cumulative_net_pnl_inr": 0.0,  # all-in (statutory − validated slip; slip default 0)
            "cumulative_net_return_bps": 0.0,
            "cumulative_friction_inr": 0.0,  # == statutory overhead under V1
            # Legacy journal totals preserved separately when rebuilding from history
            "legacy_journal_cumulative_net_pnl_inr": None,
        }

    def _save_state(self) -> None:
        self.state["protocol_id"] = PROTOCOL_ID
        self.state["execution_mode"] = "PAPER"
        self.state["live_order_authority"] = LIVE_ORDER_AUTHORITY_PCT
        self.state["n_open"] = len(self._opens)
        self.state["open_symbols"] = sorted(self._opens.keys())
        _atomic_write_json(self.state_file, self.state)

    def _load_opens(self) -> Dict[str, Dict[str, Any]]:
        if not self.open_state_file.exists():
            return {}
        try:
            raw = json.loads(self.open_state_file.read_text(encoding="utf-8"))
        except Exception:
            return {}
        trades = raw.get("trades") if isinstance(raw, dict) else None
        out: Dict[str, Dict[str, Any]] = {}
        if isinstance(trades, dict):
            for sym, rec in trades.items():
                if isinstance(rec, dict) and rec.get("active") and rec.get("of_trade_id"):
                    out[str(sym)] = rec
        return out

    def _persist_opens(self) -> None:
        _atomic_write_json(
            self.open_state_file,
            {
                "protocol_id": PROTOCOL_ID,
                "execution_mode": "PAPER",
                "live_order_authority": LIVE_ORDER_AUTHORITY_PCT,
                "n_open": len(self._opens),
                "trades": self._opens,
            },
        )

    def _known_outcome_ids(self) -> set:
        ids: set = set()
        if not self.outcomes_file.exists():
            return ids
        try:
            with open(self.outcomes_file, "r", encoding="utf-8") as f:
                for line in f:
                    line = line.strip()
                    if not line:
                        continue
                    try:
                        row = json.loads(line)
                    except Exception:
                        continue
                    tid = row.get("of_trade_id")
                    if tid:
                        ids.add(str(tid))
        except Exception:
            return ids
        return ids

    def _reconcile_closed_ghosts(self) -> None:
        known = self._known_outcome_ids()
        drop = [s for s, t in self._opens.items() if str(t.get("of_trade_id") or "") in known]
        for s in drop:
            self._opens.pop(s, None)

    def _append_jsonl(self, path: Path, row: Dict[str, Any]) -> None:
        path.parent.mkdir(parents=True, exist_ok=True)
        with open(path, "a", encoding="utf-8") as f:
            f.write(json.dumps(row) + "\n")

    def _entry_quality_ok(self, size: Dict[str, Any], depth_q: Dict[str, Any]) -> Tuple[bool, str]:
        if self.config.get("of_paper_reject_thin_opposite", True) and depth_q.get(
            "thin_opposite_diagnostic"
        ):
            return False, "thin_opposite_diagnostic"
        min_sig = float(self.config.get("of_paper_min_entry_signal_qty", 0.0) or 0.0)
        try:
            sig = float(size.get("entry_signal_qty") or 0.0)
        except (TypeError, ValueError):
            sig = 0.0
        if min_sig > 0 and sig < min_sig:
            return False, f"entry_signal_qty_{sig}<{min_sig}"
        min_same = float(self.config.get("of_paper_min_same_side_notional_inr", 0.0) or 0.0)
        try:
            same_n = float(depth_q.get("same_side_touch_notional_usdt") or 0.0)
        except (TypeError, ValueError):
            same_n = 0.0
        # depth helper labels field *_usdt but value is price*qty in quote currency (INR here)
        if min_same > 0 and same_n < min_same:
            return False, f"same_side_notional_{same_n}<{min_same}"
        return True, "ok"

    def _close_allowed(
        self, prev: Dict[str, Any], *, now_ms: int, exit_meta: Optional[Dict[str, Any]]
    ) -> Tuple[bool, str]:
        min_hold = int(self.config.get("of_paper_min_hold_ms", 300_000))
        persist = int(self.config.get("of_paper_opposite_persist_ms", 30_000))
        act = prev.get("activated_ms") or prev.get("entry_time_ms")
        try:
            act_i = int(act) if act is not None else None
        except (TypeError, ValueError):
            act_i = None
        if act_i is not None:
            held = max(0, now_ms - act_i)
            if held < min_hold:
                return False, f"min_hold_not_met_{held}ms<{min_hold}ms"
        if exit_meta and exit_meta.get("exit"):
            seen = prev.get("opposite_exit_seen_ms")
            if seen is None:
                prev["opposite_exit_seen_ms"] = now_ms
                return False, f"opposite_persist_arming_need_{persist}ms"
            try:
                seen_i = int(seen)
            except (TypeError, ValueError):
                prev["opposite_exit_seen_ms"] = now_ms
                return False, "opposite_persist_rearm"
            if (now_ms - seen_i) < persist:
                return False, f"opposite_persist_{(now_ms - seen_i)}ms<{persist}ms"
            return True, "ok"
        return True, "ok"

    def _quote_px(self, quote: Optional[Any], side: str, *, for_exit: bool) -> Optional[float]:
        l1 = l1_from_nse_quote(quote)
        try:
            bid = float(l1.get("bid_price") or 0.0)
            ask = float(l1.get("ask_price") or 0.0)
        except (TypeError, ValueError):
            return None
        if bid <= 0 or ask <= 0:
            return None
        # LONG entry crosses ask; LONG exit hits bid
        if str(side).upper() == "LONG":
            return ask if not for_exit else bid
        return bid if not for_exit else ask

    def _half_spread_bps(self, quote: Optional[Any]) -> float:
        if quote is None:
            return 2.5
        if hasattr(quote, "half_spread_bps"):
            try:
                v = float(getattr(quote, "half_spread_bps") or 0.0)
                return v if v > 0 else 2.5
            except (TypeError, ValueError):
                return 2.5
        if isinstance(quote, dict):
            try:
                v = float(quote.get("half_spread_bps") or 0.0)
                return v if v > 0 else 2.5
            except (TypeError, ValueError):
                return 2.5
        l1 = l1_from_nse_quote(quote)
        try:
            bid = float(l1.get("bid_price") or 0.0)
            ask = float(l1.get("ask_price") or 0.0)
        except (TypeError, ValueError):
            return 2.5
        mid = 0.5 * (bid + ask)
        if mid <= 0 or ask <= bid:
            return 2.5
        return ((ask - bid) / mid) * 10000.0 / 2.0

    def try_open_from_eval(
        self,
        *,
        symbol: str,
        eval_record: Dict[str, Any],
        quote: Optional[Any],
        now_ms: Optional[int] = None,
    ) -> Optional[Dict[str, Any]]:
        """Book LONG OF paper when RECOMMEND_LONG + OF long support."""
        if not self.state.get("enabled", True):
            return None
        now = int(now_ms if now_ms is not None else time.time() * 1000)
        sym = str(symbol)
        if sym in self._opens:
            return self._opens[sym]

        if str(eval_record.get("decision") or "") != "RECOMMEND_LONG":
            return None
        if not eval_record.get("order_flow_supported") and not eval_record.get(
            "order_flow_long_supported"
        ):
            # Require primary OF support for LONG side
            l1 = l1_from_nse_quote(quote)
            of = classify_order_flow_support("LONG", l1)
            if not of.get("order_flow_supported"):
                return None
            imb = of.get("order_flow_imbalance")
            of_src = of.get("order_flow_source")
        else:
            if not (
                eval_record.get("order_flow_supported")
                or eval_record.get("order_flow_long_supported")
            ):
                return None
            # If stamp says supported but side is not LONG, skip
            side = str(eval_record.get("order_flow_side") or "LONG").upper()
            if eval_record.get("order_flow_supported") and side not in ("LONG", ""):
                if not eval_record.get("order_flow_long_supported"):
                    return None
            imb = eval_record.get("order_flow_imbalance")
            of_src = eval_record.get("order_flow_source") or "kite_l1"

        bar_idx = eval_record.get("bar_idx")
        if self.config.get("of_paper_one_per_15m_bar", True) and bar_idx is not None:
            last_map = self.state.setdefault("last_of_paper_entry_bar_by_symbol", {})
            if last_map.get(sym) is not None and int(last_map[sym]) == int(bar_idx):
                return None

        try:
            entry_px = float(
                self._quote_px(quote, "LONG", for_exit=False)
                or eval_record.get("reference_price")
                or 0.0
            )
        except (TypeError, ValueError):
            entry_px = 0.0
        if entry_px <= 0:
            return None

        size = size_nse_of_paper_from_entry_signal(
            entry_price=entry_px, quote=quote, config=self.config
        )
        qdict = quote_as_book_dict(quote)
        depth_q = compute_of_depth_quality(
            direction="LONG", quote=qdict, imbalance=imb, phase="entry"
        )

        ok_q, deny = self._entry_quality_ok(size, depth_q)
        if not ok_q:
            self.state["last_suppressed"] = {
                "reason": deny,
                "symbol": sym,
                "direction": "LONG",
                "bar_idx": bar_idx,
                "order_flow_imbalance": imb,
                "entry_signal_qty": size.get("entry_signal_qty"),
                "suppressed_ms": now,
                "protocol_id": PROTOCOL_ID,
                "note": "NSE OF PAPER open suppressed — OF ±0.30 mark unchanged",
            }
            self._save_state()
            return None

        entry_candle = build_nse_of_candle_mark(now_ms=now, bar_idx=bar_idx)
        entry_candle.update(
            {
                "signal_qty": size.get("entry_signal_qty"),
                "signal_side": "bid",
                "paper_quantity": size.get("simulated_quantity"),
                "participation_pct": size.get("participation_pct"),
                "depth_quality": depth_q,
            }
        )
        of_trade_id = f"NSE-OF-PAPER-{sym}-{bar_idx}-{now}"
        l1 = l1_from_nse_quote(quote)
        try:
            bid = float(l1.get("bid_price") or 0.0) or None
            ask = float(l1.get("ask_price") or 0.0) or None
        except (TypeError, ValueError):
            bid = ask = None

        rec = {
            "active": True,
            "protocol_id": PROTOCOL_ID,
            "execution_mode": "PAPER",
            "live_order_authority": LIVE_ORDER_AUTHORITY_PCT,
            "of_trade_id": of_trade_id,
            "symbol": sym,
            "tradingsymbol": str(sym).replace(".NS", ""),
            "exchange": "NSE",
            "direction": "LONG",
            "order_flow_label": ORDER_FLOW_LABEL,
            "order_flow_supported": True,
            "order_flow_imbalance": imb,
            "order_flow_source": of_src,
            "entry_price": entry_px,
            "entry_bid": bid,
            "entry_ask": ask,
            "entry_signal_qty": size.get("entry_signal_qty"),
            "entry_paper_quantity": size.get("simulated_quantity"),
            "simulated_quantity": size.get("simulated_quantity"),
            "simulated_notional_inr": size.get("simulated_notional_inr"),
            "participation_pct": size.get("participation_pct"),
            "sizing_source": size.get("sizing_source"),
            "depth_quality": depth_q,
            "half_spread_bps": self._half_spread_bps(quote),
            "activated_ms": now,
            "entry_time_ms": now,
            "paper_open_timestamp": _iso_utc_ms(now),
            "entry_bar_idx": bar_idx,
            "entry_candle": entry_candle,
            "session_idx": eval_record.get("session_idx"),
            "opposite_exit_seen_ms": None,
            "exit_rule": (
                "opposite_side_order_flow_confirm"
                f"+min_hold_{int(self.config['of_paper_min_hold_ms'])}ms"
                f"+persist_{int(self.config['of_paper_opposite_persist_ms'])}ms"
            ),
            "independent_of_estimand_a": True,
            "note": (
                "NSE OF PAPER (RP-608-NSE-OF-PAPER-TRADE-001). "
                "Opens on RECOMMEND_LONG+OF; closes on reverse OF or session square-off. 0% authority."
            ),
        }
        self._opens[sym] = rec
        if bar_idx is not None:
            self.state.setdefault("last_of_paper_entry_bar_by_symbol", {})[sym] = int(bar_idx)
        self.state.pop("last_suppressed", None)
        self._append_jsonl(
            self.opens_file,
            {
                "event": "NSE_OF_PAPER_OPEN",
                **{k: rec.get(k) for k in (
                    "protocol_id", "execution_mode", "live_order_authority", "of_trade_id",
                    "symbol", "tradingsymbol", "direction", "entry_price", "simulated_quantity",
                    "simulated_notional_inr", "entry_signal_qty", "order_flow_imbalance",
                    "activated_ms", "paper_open_timestamp", "entry_bar_idx", "entry_candle",
                    "depth_quality", "sizing_source",
                )},
            },
        )
        self._persist_opens()
        self._save_state()
        return rec

    def evaluate_open_tick(
        self,
        *,
        symbol: str,
        quote: Optional[Any],
        now_ms: Optional[int] = None,
        bar_idx: Optional[int] = None,
        force_reason: Optional[str] = None,
    ) -> Optional[Dict[str, Any]]:
        """Tick an open OF paper trade; reverse OF or forced square-off may close."""
        sym = str(symbol)
        prev = self._opens.get(sym)
        if not prev or not prev.get("active"):
            return None
        now = int(now_ms if now_ms is not None else time.time() * 1000)
        l1 = l1_from_nse_quote(quote)

        if force_reason:
            return self._close(
                prev,
                reason=force_reason,
                exit_meta={"exit": True, "opposite_side": "FORCED", "order_flow_imbalance": l1_imbalance_safe(l1)},
                now_ms=now,
                quote=quote,
                bar_idx=bar_idx,
            )

        exit_sig = is_order_flow_exit_signal("LONG", l1)
        of_same = classify_order_flow_support("LONG", l1)
        if of_same.get("order_flow_imbalance") is not None:
            prev["last_book_imbalance"] = of_same["order_flow_imbalance"]
            prev["last_of_eval_ms"] = now
            if of_same.get("order_flow_supported"):
                prev["order_flow_imbalance"] = of_same["order_flow_imbalance"]
            self._opens[sym] = prev

        if exit_sig.get("exit"):
            allowed, deny = self._close_allowed(prev, now_ms=now, exit_meta=exit_sig)
            if not allowed:
                prev = dict(prev)
                prev["close_suppressed_reason"] = deny
                prev["close_suppressed_ms"] = now
                prev["pending_exit_imbalance"] = exit_sig.get("order_flow_imbalance")
                self._opens[sym] = prev
                self._persist_opens()
                self._save_state()
                return prev
            return self._close(
                prev,
                reason="OPPOSITE_SIDE_ORDER_FLOW",
                exit_meta=exit_sig,
                now_ms=now,
                quote=quote,
                bar_idx=bar_idx,
            )

        if prev.get("opposite_exit_seen_ms") is not None:
            prev["opposite_exit_seen_ms"] = None
            prev.pop("close_suppressed_reason", None)
            self._opens[sym] = prev
            self._persist_opens()
        return prev

    def _close(
        self,
        prev: Dict[str, Any],
        *,
        reason: str,
        exit_meta: Optional[Dict[str, Any]],
        now_ms: int,
        quote: Optional[Any],
        bar_idx: Optional[int],
    ) -> Optional[Dict[str, Any]]:
        sym = str(prev.get("symbol") or "")
        tid = prev.get("of_trade_id")
        if tid and str(tid) in self._known_outcome_ids():
            self._opens.pop(sym, None)
            self._persist_opens()
            self._save_state()
            return None

        entry_px = float(prev.get("entry_price") or 0.0)
        qty = float(prev.get("entry_paper_quantity") or prev.get("simulated_quantity") or 0.0)
        direction = str(prev.get("direction") or "LONG").upper()
        exit_px = self._quote_px(quote, direction, for_exit=True) or entry_px

        # Touch ask-in / bid-out fills + statutory overhead (no double-counted spread).
        acct: Dict[str, Any] = {}
        if entry_px > 0 and exit_px > 0 and qty > 0:
            try:
                if callable(self._friction_fn):
                    # Prefer dedicated OF accounting when friction_fn is the V1 helper;
                    # otherwise treat callable as calc_nse_of_paper_accounting-compatible.
                    try:
                        acct = self._friction_fn(
                            entry_price=entry_px,
                            exit_price=exit_px,
                            quantity=qty,
                            direction=direction,
                        )
                    except TypeError:
                        # Legacy signature — force statutory-only (no spread/slip).
                        fr = self._friction_fn(
                            entry_px,
                            exit_px,
                            qty,
                            0.0,
                            include_spread_crossing=False,
                            slippage_bps_per_leg=0.0,
                        )
                        overhead = float(fr.get("statutory_overhead") or fr.get("total_friction") or 0.0)
                        gross = (exit_px - entry_px) * qty
                        acct = {
                            "accounting_standard": "NSE_OF_TOUCH_PLUS_STATUTORY_V1",
                            "touch_gross_pnl_inr": round(gross, 2),
                            "touch_gross_return_bps": round(
                                ((exit_px - entry_px) / entry_px) * 10000.0, 2
                            ),
                            "brokerage": fr.get("brokerage"),
                            "stt": fr.get("stt"),
                            "exchange_charges": fr.get("exchange_charges"),
                            "sebi_charges": fr.get("sebi_charges"),
                            "stamp_duty": fr.get("stamp_duty"),
                            "gst": fr.get("gst"),
                            "statutory_overhead_inr": round(overhead, 2),
                            "statutory_net_pnl_inr": round(gross - overhead, 2),
                            "validated_incremental_slippage_inr": 0.0,
                            "all_in_net_pnl_inr": round(gross - overhead, 2),
                            "spread_crossing_charged": False,
                            "friction": fr,
                            "friction_inr": round(overhead, 2),
                            "net_pnl_inr": round(gross - overhead, 2),
                            "net_return_bps": round(
                                ((gross - overhead) / (entry_px * qty)) * 10000.0, 2
                            ),
                            "win_label": 1 if (gross - overhead) > 0 else 0,
                        }
                else:
                    acct = calc_nse_of_paper_accounting(
                        entry_price=entry_px,
                        exit_price=exit_px,
                        quantity=qty,
                        direction=direction,
                    )
            except Exception:
                acct = {}

        if not acct:
            gross_pnl = (exit_px - entry_px) * qty
            gross_bps = ((exit_px - entry_px) / entry_px) * 10000.0 if entry_px > 0 else 0.0
            fric_inr = 0.0
            net_pnl = gross_pnl
            net_bps = gross_bps
            friction: Dict[str, Any] = {}
        else:
            gross_pnl = float(acct.get("touch_gross_pnl_inr") or 0.0)
            gross_bps = float(acct.get("touch_gross_return_bps") or 0.0)
            fric_inr = float(acct.get("friction_inr") or acct.get("statutory_overhead_inr") or 0.0)
            net_pnl = float(acct.get("all_in_net_pnl_inr") or acct.get("net_pnl_inr") or 0.0)
            net_bps = float(acct.get("net_return_bps") or 0.0)
            friction = acct.get("friction") if isinstance(acct.get("friction"), dict) else acct

        exit_depth = compute_of_depth_quality(
            direction="LONG",
            quote=quote_as_book_dict(quote),
            imbalance=(exit_meta or {}).get("order_flow_imbalance"),
            phase="exit",
        )
        hold_s = None
        act = prev.get("activated_ms")
        if act is not None:
            try:
                hold_s = round((now_ms - int(act)) / 1000.0, 3)
            except (TypeError, ValueError):
                hold_s = None
        exit_depth["signal_persistence_hold_seconds"] = hold_s
        opp_seen = prev.get("opposite_exit_seen_ms")
        if opp_seen is not None:
            try:
                exit_depth["opposite_persist_ms_at_exit"] = max(0, now_ms - int(opp_seen))
            except (TypeError, ValueError):
                pass

        exit_candle = build_nse_of_candle_mark(
            now_ms=now_ms,
            bar_idx=bar_idx if bar_idx is not None else prev.get("entry_bar_idx"),
        )
        exit_candle.update(
            {
                "paper_quantity": qty,
                "qty_matched_to_entry": True,
                "depth_quality": exit_depth,
                "signal_side": "ask",
            }
        )

        closed = {
            **prev,
            "active": False,
            "exit_reason": reason,
            "order_flow_exit_label": ORDER_FLOW_EXIT_LABEL if reason.startswith("OPPOSITE") else None,
            "exit_imbalance": (exit_meta or {}).get("order_flow_imbalance"),
            "exit_opposite_side": (exit_meta or {}).get("opposite_side"),
            "exit_price": exit_px,
            "exit_quantity": qty,
            "qty_matched_entry_to_exit": True,
            "signal_exit_ms": now_ms,
            "paper_close_ms": now_ms,
            "signal_exit_timestamp": _iso_utc_ms(now_ms),
            "paper_close_timestamp": _iso_utc_ms(now_ms),
            "paper_close_lag_ms": 0,
            "holding_duration_seconds": hold_s,
            "exit_bar_idx": bar_idx,
            "exit_candle": exit_candle,
            "depth_quality_entry": prev.get("depth_quality"),
            "depth_quality_exit": exit_depth,
            "depth_quality_protocol_id": OF_DEPTH_QUALITY_PROTOCOL_ID,
            "gross_pnl_inr": round(gross_pnl, 2),
            "gross_return_bps": round(gross_bps, 2),
            "friction": friction,
            "friction_inr": round(fric_inr, 2),
            "net_pnl_inr": round(net_pnl, 2),
            "net_return_bps": round(net_bps, 2),
            "win_label": 1 if net_pnl > 0 else 0,
            "accounting_standard": acct.get("accounting_standard") or "NSE_OF_TOUCH_PLUS_STATUTORY_V1",
            "touch_gross_pnl_inr": acct.get("touch_gross_pnl_inr", round(gross_pnl, 2)),
            "brokerage": acct.get("brokerage"),
            "stt": acct.get("stt"),
            "exchange_charges": acct.get("exchange_charges"),
            "sebi_charges": acct.get("sebi_charges"),
            "stamp_duty": acct.get("stamp_duty"),
            "gst": acct.get("gst"),
            "statutory_overhead_inr": acct.get("statutory_overhead_inr", round(fric_inr, 2)),
            "statutory_net_pnl_inr": acct.get("statutory_net_pnl_inr", round(net_pnl, 2)),
            "validated_incremental_slippage_inr": acct.get(
                "validated_incremental_slippage_inr", 0.0
            ),
            "all_in_net_pnl_inr": acct.get("all_in_net_pnl_inr", round(net_pnl, 2)),
            "sensitivity_2bps_slippage_inr": acct.get("sensitivity_2bps_slippage_inr"),
            "sensitivity_all_in_net_pnl_inr": acct.get("sensitivity_all_in_net_pnl_inr"),
            "spread_crossing_charged": False,
            "protocol_id": PROTOCOL_ID,
            "execution_mode": "PAPER",
            "live_order_authority": LIVE_ORDER_AUTHORITY_PCT,
        }

        if not self._log_outcome(closed):
            # already logged
            self._opens.pop(sym, None)
            self._persist_opens()
            self._save_state()
            return None

        summary = self.state.setdefault("of_outcomes_summary", self._empty_summary())
        summary["accounting_standard"] = "NSE_OF_TOUCH_PLUS_STATUTORY_V1"
        summary["n_trades"] = int(summary.get("n_trades") or 0) + 1
        if net_pnl > 0:
            summary["n_wins"] = int(summary.get("n_wins") or 0) + 1
        else:
            summary["n_losses"] = int(summary.get("n_losses") or 0) + 1
        summary["cumulative_gross_pnl_inr"] = round(
            float(summary.get("cumulative_gross_pnl_inr") or 0.0) + gross_pnl, 2
        )
        summary["cumulative_statutory_overhead_inr"] = round(
            float(summary.get("cumulative_statutory_overhead_inr") or 0.0) + fric_inr, 2
        )
        statutory_net = float(acct.get("statutory_net_pnl_inr") or net_pnl)
        summary["cumulative_statutory_net_pnl_inr"] = round(
            float(summary.get("cumulative_statutory_net_pnl_inr") or 0.0) + statutory_net, 2
        )
        summary["cumulative_net_pnl_inr"] = round(
            float(summary.get("cumulative_net_pnl_inr") or 0.0) + net_pnl, 2
        )
        summary["cumulative_net_return_bps"] = round(
            float(summary.get("cumulative_net_return_bps") or 0.0) + net_bps, 2
        )
        summary["cumulative_friction_inr"] = round(
            float(summary.get("cumulative_friction_inr") or 0.0) + fric_inr, 2
        )
        self.state["last_closed"] = closed
        self._opens.pop(sym, None)
        self._persist_opens()
        self._save_state()
        return None

    def _log_outcome(self, row: Dict[str, Any]) -> bool:
        tid = row.get("of_trade_id")
        if tid and str(tid) in self._known_outcome_ids():
            return False
        self._append_jsonl(self.outcomes_file, row)
        return True

    def square_off_all(
        self,
        *,
        quotes_by_symbol: Dict[str, Any],
        now_ms: Optional[int] = None,
        bar_idx: Optional[int] = None,
        reason: str = "SESSION_SQUARE_OFF",
    ) -> int:
        now = int(now_ms if now_ms is not None else time.time() * 1000)
        n = 0
        for sym in list(self._opens.keys()):
            self.evaluate_open_tick(
                symbol=sym,
                quote=quotes_by_symbol.get(sym),
                now_ms=now,
                bar_idx=bar_idx,
                force_reason=reason,
            )
            n += 1
        return n

    def open_snapshot(self) -> List[Dict[str, Any]]:
        return [dict(v) for v in self._opens.values()]

    def summary(self) -> Dict[str, Any]:
        return {
            "protocol_id": PROTOCOL_ID,
            "execution_mode": "PAPER",
            "live_order_authority": LIVE_ORDER_AUTHORITY_PCT,
            "enabled": bool(self.state.get("enabled", True)),
            "n_open": len(self._opens),
            "open_symbols": sorted(self._opens.keys()),
            "of_outcomes_summary": dict(self.state.get("of_outcomes_summary") or self._empty_summary()),
            "last_closed": self.state.get("last_closed"),
            "last_suppressed": self.state.get("last_suppressed"),
        }

    def load_recent_outcomes(self, n: int = 20) -> List[Dict[str, Any]]:
        if not self.outcomes_file.exists():
            return []
        rows: List[Dict[str, Any]] = []
        try:
            with open(self.outcomes_file, "r", encoding="utf-8") as f:
                for line in f:
                    line = line.strip()
                    if not line:
                        continue
                    try:
                        rows.append(json.loads(line))
                    except Exception:
                        continue
        except Exception:
            return []
        return rows[-max(0, int(n)) :]


def l1_imbalance_safe(l1: Dict[str, Any]) -> Optional[float]:
    try:
        b = float(l1.get("bid_qty") or 0.0)
        a = float(l1.get("ask_qty") or 0.0)
    except (TypeError, ValueError):
        return None
    tot = b + a
    if tot <= 0:
        return None
    return round((b - a) / tot, 6)
