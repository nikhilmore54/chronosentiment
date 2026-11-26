#!/usr/bin/env python3
"""
RP-609 multi-horizon observation engine (CNC paper book).

Deterministic lifecycle for:
  H1 swing, H2 portfolio build, H3 long-term hold.

live_order_authority = 0.0 always. Does not mutate RP-608.
"""

from __future__ import annotations

import json
import uuid
from copy import deepcopy
from datetime import datetime, timezone, timedelta
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

from scripts.rp609_indian_retail_horizons import (
    ARTIFACT_ROOT,
    FRICTION_MODEL_ID,
    HORIZONS,
    JOURNALS,
    LIVE_ORDER_AUTHORITY,
    PROTOCOL_ID,
    SCHEMA_VERSION,
    append_jsonl,
    candidates_path,
    daily_log_path,
    estimate_cnc_friction_inr,
    long_term_path,
    portfolio_path,
    swing_path,
    tax_lot_hint_from_hold_sessions,
    write_horizon_registry,
)

IST = timezone(timedelta(hours=5, minutes=30))
WORKSPACE = Path(__file__).resolve().parent.parent
DEFAULT_NSE_DIR = WORKSPACE / "live_capture" / "shadow_validation" / "nse"


def _utc_now() -> str:
    return datetime.now(timezone.utc).isoformat()


def _ist_today() -> str:
    return datetime.now(IST).strftime("%Y-%m-%d")


def _read_json(path: Path, default: Any) -> Any:
    if not path.exists():
        return deepcopy(default)
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return deepcopy(default)


def _write_json(path: Path, payload: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_suffix(path.suffix + ".tmp")
    tmp.write_text(json.dumps(payload, indent=2, default=str), encoding="utf-8")
    tmp.replace(path)


def swing_book_path() -> Path:
    return JOURNALS / "RP609_swing_book.json"


def portfolio_book_path() -> Path:
    return JOURNALS / "RP609_portfolio_book.json"


def long_term_book_path() -> Path:
    return JOURNALS / "RP609_long_term_book.json"


def empty_swing_book() -> Dict[str, Any]:
    return {
        "schema_version": SCHEMA_VERSION,
        "protocol_id": PROTOCOL_ID,
        "live_order_authority": LIVE_ORDER_AUTHORITY,
        "open": {},
        "closed": [],
    }


def empty_portfolio_book() -> Dict[str, Any]:
    return {
        "schema_version": SCHEMA_VERSION,
        "protocol_id": PROTOCOL_ID,
        "live_order_authority": LIVE_ORDER_AUTHORITY,
        "plans": {},
    }


def empty_lt_book() -> Dict[str, Any]:
    return {
        "schema_version": SCHEMA_VERSION,
        "protocol_id": PROTOCOL_ID,
        "live_order_authority": LIVE_ORDER_AUTHORITY,
        "open": {},
        "closed": [],
    }


def normalize_symbol(symbol: str) -> str:
    s = str(symbol).strip().upper()
    return s if s.endswith(".NS") else f"{s}.NS"


def load_price_map(nse_dir: Path) -> Dict[str, float]:
    """Best-effort last mark: bar history close, else latest eval reference_price."""
    prices: Dict[str, float] = {}
    bh = nse_dir / "rp608_nse_bar_history.json"
    if bh.exists():
        try:
            raw = json.loads(bh.read_text(encoding="utf-8"))
            for sym, bars in (raw or {}).items():
                if not isinstance(bars, list) or not bars:
                    continue
                last = bars[-1]
                if isinstance(last, dict) and last.get("close") is not None:
                    prices[normalize_symbol(sym)] = float(last["close"])
        except Exception:
            pass
    ev = nse_dir / "rp608_nse_latest_evaluations.json"
    if ev.exists():
        try:
            payload = json.loads(ev.read_text(encoding="utf-8"))
            evaluations = payload.get("evaluations") or payload
            if isinstance(evaluations, dict):
                for sym, row in evaluations.items():
                    if not isinstance(row, dict):
                        continue
                    px = row.get("reference_price") or row.get("entry_price")
                    if px is None:
                        continue
                    key = normalize_symbol(sym)
                    prices.setdefault(key, float(px))
        except Exception:
            pass
    return prices


def _pnl_long(entry: float, exit_px: float, qty: float) -> Dict[str, float]:
    gross = (exit_px - entry) * qty
    fr = estimate_cnc_friction_inr(
        buy_price=entry, sell_price=exit_px, quantity=qty
    )
    net = gross - float(fr["total_friction"])
    ret_bps = ((exit_px - entry) / entry * 10000.0) if entry else 0.0
    return {
        "gross_pnl_inr": round(gross, 2),
        "net_pnl_inr": round(net, 2),
        "gross_return_bps": round(ret_bps, 2),
        "friction": fr,
    }


class MultiHorizonEngine:
    def __init__(self, nse_dir: Optional[Path] = None) -> None:
        write_horizon_registry()
        JOURNALS.mkdir(parents=True, exist_ok=True)
        self.nse_dir = Path(nse_dir) if nse_dir else DEFAULT_NSE_DIR
        self.swing = _read_json(swing_book_path(), empty_swing_book())
        self.portfolio = _read_json(portfolio_book_path(), empty_portfolio_book())
        self.long_term = _read_json(long_term_book_path(), empty_lt_book())

    def persist(self) -> None:
        _write_json(swing_book_path(), self.swing)
        _write_json(portfolio_book_path(), self.portfolio)
        _write_json(long_term_book_path(), self.long_term)

    # ----- H1 Swing -----
    def register_swing_candidate(
        self,
        *,
        symbol: str,
        thesis_tag: str,
        reference_price: float,
        invalidation_price: float,
        target_price: float,
        quantity: float,
        max_sessions: int = 10,
        sector: Optional[str] = None,
        notes: Optional[str] = None,
        auto_open: bool = True,
        session_date: Optional[str] = None,
    ) -> Dict[str, Any]:
        sym = normalize_symbol(symbol)
        h = HORIZONS["H1_SWING"]
        max_sessions = int(max(h["hold_sessions_min"], min(h["hold_sessions_max"], max_sessions)))
        cand_id = f"h1cand_{uuid.uuid4().hex[:10]}"
        trade_id = f"h1_{uuid.uuid4().hex[:10]}"
        sess = session_date or _ist_today()
        candidate = {
            "schema_version": SCHEMA_VERSION,
            "protocol_id": PROTOCOL_ID,
            "event_type": "CANDIDATE_REGISTERED",
            "candidate_id": cand_id,
            "trade_id": trade_id,
            "horizon_id": "H1_SWING",
            "product_type": "CNC",
            "live_order_authority": LIVE_ORDER_AUTHORITY,
            "symbol": sym,
            "tradingsymbol": sym.replace(".NS", ""),
            "sector": sector,
            "thesis_tag": thesis_tag,
            "reference_price": float(reference_price),
            "invalidation_price": float(invalidation_price),
            "target_price": float(target_price),
            "quantity": float(quantity),
            "max_sessions": max_sessions,
            "notes": notes,
            "status": "CANDIDATE",
            "recorded_utc": _utc_now(),
            "session_date": sess,
        }
        append_jsonl(candidates_path(), candidate)

        if not auto_open:
            self.persist()
            return candidate

        return self.open_swing_from_candidate(candidate, session_date=sess)

    def open_swing_from_candidate(
        self, candidate: Dict[str, Any], *, session_date: Optional[str] = None
    ) -> Dict[str, Any]:
        sess = session_date or candidate.get("session_date") or _ist_today()
        entry = float(candidate["reference_price"])
        qty = float(candidate["quantity"])
        trade = {
            "schema_version": SCHEMA_VERSION,
            "protocol_id": PROTOCOL_ID,
            "event_type": "SWING_OPEN",
            "live_order_authority": LIVE_ORDER_AUTHORITY,
            "horizon_id": "H1_SWING",
            "product_type": "CNC",
            "trade_id": candidate.get("trade_id") or f"h1_{uuid.uuid4().hex[:10]}",
            "candidate_id": candidate.get("candidate_id"),
            "symbol": candidate["symbol"],
            "tradingsymbol": candidate.get("tradingsymbol"),
            "sector": candidate.get("sector"),
            "thesis_tag": candidate.get("thesis_tag"),
            "side": "LONG",
            "entry_price": entry,
            "quantity": qty,
            "invalidation_price": float(candidate["invalidation_price"]),
            "target_price": float(candidate["target_price"]),
            "max_sessions": int(candidate["max_sessions"]),
            "sessions_held": 0,
            "entry_session_date": sess,
            "last_mark_session_date": sess,
            "last_mark_price": entry,
            "mfe_price": entry,
            "mae_price": entry,
            "mfe_bps": 0.0,
            "mae_bps": 0.0,
            "status": "OPEN",
            "marks": [{"session_date": sess, "price": entry, "event": "ENTRY"}],
            "friction_model_id": FRICTION_MODEL_ID,
            "recorded_utc": _utc_now(),
        }
        self.swing.setdefault("open", {})[trade["trade_id"]] = trade
        append_jsonl(swing_path(), {**trade, "event_type": "SWING_OPEN"})
        self.persist()
        return trade

    def _update_excursions(self, pos: Dict[str, Any], px: float) -> None:
        ep = float(pos["entry_price"])
        mfe_p = max(float(pos.get("mfe_price") or ep), px)
        mae_p = min(float(pos.get("mae_price") or ep), px)
        pos["mfe_price"] = mfe_p
        pos["mae_price"] = mae_p
        pos["mfe_bps"] = round((mfe_p - ep) / ep * 10000.0, 2) if ep else 0.0
        pos["mae_bps"] = round((ep - mae_p) / ep * 10000.0, 2) if ep else 0.0

    def mark_swing(
        self,
        trade_id: str,
        *,
        price: float,
        session_date: Optional[str] = None,
    ) -> Dict[str, Any]:
        pos = self.swing.get("open", {}).get(trade_id)
        if not pos:
            raise KeyError(f"swing not open: {trade_id}")
        sess = session_date or _ist_today()
        px = float(price)
        # Session increment when mark date advances
        if pos.get("last_mark_session_date") and sess > str(pos["last_mark_session_date"]):
            pos["sessions_held"] = int(pos.get("sessions_held") or 0) + 1
        pos["last_mark_session_date"] = sess
        pos["last_mark_price"] = px
        self._update_excursions(pos, px)
        pos.setdefault("marks", []).append(
            {"session_date": sess, "price": px, "event": "MARK"}
        )

        exit_reason = None
        if px <= float(pos["invalidation_price"]):
            exit_reason = "INVALIDATION"
        elif px >= float(pos["target_price"]):
            exit_reason = "TARGET"
        elif int(pos.get("sessions_held") or 0) >= int(pos["max_sessions"]):
            exit_reason = "MAX_SESSIONS"

        if exit_reason:
            return self.close_swing(trade_id, price=px, session_date=sess, reason=exit_reason)

        append_jsonl(
            swing_path(),
            {
                "event_type": "SWING_MARK",
                "trade_id": trade_id,
                "session_date": sess,
                "price": px,
                "sessions_held": pos["sessions_held"],
                "mfe_bps": pos["mfe_bps"],
                "mae_bps": pos["mae_bps"],
                "live_order_authority": LIVE_ORDER_AUTHORITY,
                "recorded_utc": _utc_now(),
            },
        )
        self.persist()
        return pos

    def close_swing(
        self,
        trade_id: str,
        *,
        price: float,
        reason: str,
        session_date: Optional[str] = None,
    ) -> Dict[str, Any]:
        pos = self.swing.get("open", {}).pop(trade_id, None)
        if not pos:
            raise KeyError(f"swing not open: {trade_id}")
        sess = session_date or _ist_today()
        px = float(price)
        self._update_excursions(pos, px)
        pnl = _pnl_long(float(pos["entry_price"]), px, float(pos["quantity"]))
        pos.update(
            {
                "event_type": "SWING_CLOSE",
                "status": "CLOSED",
                "exit_price": px,
                "exit_session_date": sess,
                "exit_reason": reason,
                "tax_lot_hint": tax_lot_hint_from_hold_sessions(int(pos.get("sessions_held") or 0)),
                "closed_utc": _utc_now(),
                **pnl,
            }
        )
        self.swing.setdefault("closed", []).append(pos)
        append_jsonl(swing_path(), pos)
        self.persist()
        return pos

    # ----- H2 Portfolio build -----
    def register_portfolio_plan(
        self,
        *,
        plan_id: str,
        total_capital_inr: float,
        legs: List[Dict[str, Any]],
        n_tranches: int = 4,
        max_name_pct: float = 10.0,
        max_sector_pct: float = 30.0,
        session_date: Optional[str] = None,
    ) -> Dict[str, Any]:
        if plan_id in (self.portfolio.get("plans") or {}):
            raise ValueError(f"plan already exists: {plan_id}")
        sess = session_date or _ist_today()
        # Validate caps on target weights
        for leg in legs:
            w = float(leg.get("target_weight_pct") or 0.0)
            if w > max_name_pct + 1e-9:
                raise ValueError(
                    f"{leg.get('symbol')} weight {w}% exceeds max_name_pct {max_name_pct}%"
                )
        by_sector: Dict[str, float] = {}
        for leg in legs:
            sec = str(leg.get("sector") or "UNKNOWN")
            by_sector[sec] = by_sector.get(sec, 0.0) + float(leg.get("target_weight_pct") or 0.0)
        for sec, w in by_sector.items():
            if w > max_sector_pct + 1e-9:
                raise ValueError(f"sector {sec} weight {w}% exceeds max_sector_pct {max_sector_pct}%")

        plan = {
            "schema_version": SCHEMA_VERSION,
            "protocol_id": PROTOCOL_ID,
            "event_type": "PORTFOLIO_BUILD_PLAN",
            "live_order_authority": LIVE_ORDER_AUTHORITY,
            "horizon_id": "H2_PORTFOLIO_BUILD",
            "product_type": "CNC",
            "plan_id": plan_id,
            "status": "ACTIVE",
            "total_capital_inr": float(total_capital_inr),
            "n_tranches": int(n_tranches),
            "tranches_filled": 0,
            "caps": {
                "max_name_pct": float(max_name_pct),
                "max_sector_pct": float(max_sector_pct),
            },
            "legs": [
                {
                    "symbol": normalize_symbol(leg["symbol"]),
                    "sector": leg.get("sector"),
                    "target_weight_pct": float(leg.get("target_weight_pct") or 0.0),
                    "qty_filled": 0.0,
                    "notional_filled": 0.0,
                    "avg_price": None,
                }
                for leg in legs
            ],
            "fills": [],
            "created_session_date": sess,
            "last_tranche_session_date": None,
            "recorded_utc": _utc_now(),
        }
        self.portfolio.setdefault("plans", {})[plan_id] = plan
        append_jsonl(portfolio_path(), plan)
        self.persist()
        return plan

    def advance_portfolio_tranche(
        self,
        plan_id: str,
        *,
        prices: Dict[str, float],
        session_date: Optional[str] = None,
    ) -> Dict[str, Any]:
        plan = (self.portfolio.get("plans") or {}).get(plan_id)
        if not plan:
            raise KeyError(f"unknown plan: {plan_id}")
        if plan.get("status") != "ACTIVE":
            return plan
        if int(plan.get("tranches_filled") or 0) >= int(plan["n_tranches"]):
            plan["status"] = "BUILT"
            self.persist()
            return plan

        sess = session_date or _ist_today()
        # One tranche per session date
        if plan.get("last_tranche_session_date") == sess:
            return plan

        tranche_idx = int(plan.get("tranches_filled") or 0) + 1
        tranche_capital = float(plan["total_capital_inr"]) / float(plan["n_tranches"])
        fills = []
        for leg in plan["legs"]:
            sym = leg["symbol"]
            px = prices.get(sym)
            if px is None or px <= 0:
                continue
            notional = tranche_capital * (float(leg["target_weight_pct"]) / 100.0)
            qty = max(1.0, float(int(notional / px))) if notional >= px else 0.0
            if qty <= 0:
                continue
            spent = qty * px
            prev_n = float(leg.get("qty_filled") or 0.0)
            prev_notional = float(leg.get("notional_filled") or 0.0)
            new_n = prev_n + qty
            new_notional = prev_notional + spent
            leg["qty_filled"] = new_n
            leg["notional_filled"] = round(new_notional, 2)
            leg["avg_price"] = round(new_notional / new_n, 4) if new_n else None
            fill = {
                "tranche": tranche_idx,
                "session_date": sess,
                "symbol": sym,
                "price": float(px),
                "quantity": qty,
                "notional": round(spent, 2),
            }
            fills.append(fill)
            plan.setdefault("fills", []).append(fill)

        plan["tranches_filled"] = tranche_idx
        plan["last_tranche_session_date"] = sess
        if tranche_idx >= int(plan["n_tranches"]):
            plan["status"] = "BUILT"
        event = {
            "event_type": "PORTFOLIO_TRANCHE_FILL",
            "plan_id": plan_id,
            "tranche": tranche_idx,
            "session_date": sess,
            "fills": fills,
            "status": plan["status"],
            "live_order_authority": LIVE_ORDER_AUTHORITY,
            "recorded_utc": _utc_now(),
        }
        append_jsonl(portfolio_path(), event)
        self.persist()
        return plan

    # ----- H3 Long-term -----
    def open_long_term(
        self,
        *,
        symbol: str,
        thesis: str,
        thesis_version: str,
        entry_price: float,
        quantity: float,
        sector: Optional[str] = None,
        review_cadence_sessions: int = 60,
        session_date: Optional[str] = None,
    ) -> Dict[str, Any]:
        sym = normalize_symbol(symbol)
        sess = session_date or _ist_today()
        hold_id = f"h3_{uuid.uuid4().hex[:10]}"
        row = {
            "schema_version": SCHEMA_VERSION,
            "protocol_id": PROTOCOL_ID,
            "event_type": "LONG_TERM_HOLD_OPEN",
            "live_order_authority": LIVE_ORDER_AUTHORITY,
            "horizon_id": "H3_LONG_TERM",
            "product_type": "CNC",
            "hold_id": hold_id,
            "symbol": sym,
            "tradingsymbol": sym.replace(".NS", ""),
            "sector": sector,
            "thesis": thesis,
            "thesis_version": thesis_version,
            "side": "LONG",
            "entry_price": float(entry_price),
            "quantity": float(quantity),
            "entry_session_date": sess,
            "sessions_held": 0,
            "last_mark_session_date": sess,
            "last_mark_price": float(entry_price),
            "mfe_bps": 0.0,
            "mae_bps": 0.0,
            "review_cadence_sessions": int(review_cadence_sessions),
            "next_review_session_count": int(review_cadence_sessions),
            "status": "HOLDING",
            "tax_lot_hint": "STCG_LTE_12M_HINT",
            "recorded_utc": _utc_now(),
        }
        self.long_term.setdefault("open", {})[hold_id] = row
        append_jsonl(long_term_path(), row)
        self.persist()
        return row

    def mark_long_term(
        self,
        hold_id: str,
        *,
        price: float,
        session_date: Optional[str] = None,
    ) -> Dict[str, Any]:
        pos = self.long_term.get("open", {}).get(hold_id)
        if not pos:
            raise KeyError(f"long-term not open: {hold_id}")
        sess = session_date or _ist_today()
        px = float(price)
        if pos.get("last_mark_session_date") and sess > str(pos["last_mark_session_date"]):
            pos["sessions_held"] = int(pos.get("sessions_held") or 0) + 1
        pos["last_mark_session_date"] = sess
        pos["last_mark_price"] = px
        ep = float(pos["entry_price"])
        fav = (px - ep) / ep * 10000.0 if ep else 0.0
        adv = (ep - px) / ep * 10000.0 if ep else 0.0
        pos["mfe_bps"] = round(max(float(pos.get("mfe_bps") or 0.0), fav), 2)
        pos["mae_bps"] = round(max(float(pos.get("mae_bps") or 0.0), adv), 2)
        pos["tax_lot_hint"] = tax_lot_hint_from_hold_sessions(int(pos.get("sessions_held") or 0))
        unreal = _pnl_long(ep, px, float(pos["quantity"]))
        pos["unrealized_net_pnl_inr"] = unreal["net_pnl_inr"]
        pos["unrealized_gross_pnl_inr"] = unreal["gross_pnl_inr"]
        if int(pos.get("sessions_held") or 0) >= int(pos.get("next_review_session_count") or 60):
            pos["status"] = "REVIEW_DUE"
        append_jsonl(
            long_term_path(),
            {
                "event_type": "LONG_TERM_MARK",
                "hold_id": hold_id,
                "session_date": sess,
                "price": px,
                "sessions_held": pos["sessions_held"],
                "status": pos["status"],
                "unrealized_net_pnl_inr": pos.get("unrealized_net_pnl_inr"),
                "live_order_authority": LIVE_ORDER_AUTHORITY,
                "recorded_utc": _utc_now(),
            },
        )
        self.persist()
        return pos

    def complete_long_term_review(
        self, hold_id: str, *, decision: str = "HOLD_CONTINUE"
    ) -> Dict[str, Any]:
        pos = self.long_term.get("open", {}).get(hold_id)
        if not pos:
            raise KeyError(f"long-term not open: {hold_id}")
        if decision == "THESIS_INVALIDATED":
            px = float(pos.get("last_mark_price") or pos["entry_price"])
            pnl = _pnl_long(float(pos["entry_price"]), px, float(pos["quantity"]))
            closed = {
                **pos,
                "event_type": "LONG_TERM_CLOSE",
                "status": "CLOSED",
                "exit_reason": "THESIS_INVALIDATED",
                "exit_price": px,
                "closed_utc": _utc_now(),
                **pnl,
            }
            self.long_term["open"].pop(hold_id)
            self.long_term.setdefault("closed", []).append(closed)
            append_jsonl(long_term_path(), closed)
            self.persist()
            return closed
        # continue
        pos["status"] = "HOLDING"
        pos["next_review_session_count"] = int(pos.get("sessions_held") or 0) + int(
            pos.get("review_cadence_sessions") or 60
        )
        append_jsonl(
            long_term_path(),
            {
                "event_type": "LONG_TERM_REVIEW",
                "hold_id": hold_id,
                "decision": decision,
                "next_review_session_count": pos["next_review_session_count"],
                "live_order_authority": LIVE_ORDER_AUTHORITY,
                "recorded_utc": _utc_now(),
            },
        )
        self.persist()
        return pos

    # ----- EOD orchestration -----
    def run_eod(
        self,
        *,
        session_date: Optional[str] = None,
        prices: Optional[Dict[str, float]] = None,
    ) -> Dict[str, Any]:
        sess = session_date or _ist_today()
        pxmap = prices if prices is not None else load_price_map(self.nse_dir)
        swing_results = []
        for trade_id in list((self.swing.get("open") or {}).keys()):
            pos = self.swing["open"][trade_id]
            sym = pos["symbol"]
            if sym not in pxmap:
                continue
            swing_results.append(self.mark_swing(trade_id, price=pxmap[sym], session_date=sess))

        portfolio_results = []
        for plan_id, plan in list((self.portfolio.get("plans") or {}).items()):
            if plan.get("status") == "ACTIVE":
                portfolio_results.append(
                    self.advance_portfolio_tranche(plan_id, prices=pxmap, session_date=sess)
                )

        lt_results = []
        for hold_id in list((self.long_term.get("open") or {}).keys()):
            pos = self.long_term["open"][hold_id]
            sym = pos["symbol"]
            if sym not in pxmap:
                continue
            lt_results.append(self.mark_long_term(hold_id, price=pxmap[sym], session_date=sess))

        summary = {
            "schema_version": SCHEMA_VERSION,
            "protocol_id": PROTOCOL_ID,
            "event_type": "EOD_SNAPSHOT",
            "live_order_authority": LIVE_ORDER_AUTHORITY,
            "decision_status": "OBSERVE_ONLY_NO_CONCLUSION",
            "session_date": sess,
            "recorded_utc": _utc_now(),
            "n_prices": len(pxmap),
            "swing_open": len(self.swing.get("open") or {}),
            "swing_closed_total": len(self.swing.get("closed") or {}),
            "swing_marked": len(swing_results),
            "portfolio_active": sum(
                1 for p in (self.portfolio.get("plans") or {}).values() if p.get("status") == "ACTIVE"
            ),
            "portfolio_built": sum(
                1 for p in (self.portfolio.get("plans") or {}).values() if p.get("status") == "BUILT"
            ),
            "long_term_open": len(self.long_term.get("open") or {}),
            "long_term_review_due": sum(
                1
                for p in (self.long_term.get("open") or {}).values()
                if p.get("status") == "REVIEW_DUE"
            ),
            "source_nse_dir": str(self.nse_dir),
        }
        # upsert daily log by session_date
        existing = []
        if daily_log_path().exists():
            for line in daily_log_path().read_text(encoding="utf-8").splitlines():
                if not line.strip():
                    continue
                try:
                    row = json.loads(line)
                except json.JSONDecodeError:
                    continue
                if row.get("session_date") != sess and row.get("observation_date_ist") != sess:
                    existing.append(row)
        existing.append(summary)
        daily_log_path().parent.mkdir(parents=True, exist_ok=True)
        with daily_log_path().open("w", encoding="utf-8") as f:
            for row in existing:
                f.write(json.dumps(row, default=str) + "\n")
        self.persist()
        return summary

    def report(self) -> Dict[str, Any]:
        closed = self.swing.get("closed") or []
        nets = [float(t.get("net_pnl_inr") or 0.0) for t in closed]
        return {
            "protocol_id": PROTOCOL_ID,
            "live_order_authority": LIVE_ORDER_AUTHORITY,
            "swing": {
                "open": len(self.swing.get("open") or {}),
                "closed": len(closed),
                "closed_net_pnl_inr": round(sum(nets), 2) if nets else 0.0,
                "open_ids": list((self.swing.get("open") or {}).keys()),
            },
            "portfolio": {
                "plans": {
                    pid: {
                        "status": p.get("status"),
                        "tranches_filled": p.get("tranches_filled"),
                        "n_tranches": p.get("n_tranches"),
                    }
                    for pid, p in (self.portfolio.get("plans") or {}).items()
                }
            },
            "long_term": {
                "open": len(self.long_term.get("open") or {}),
                "review_due": [
                    hid
                    for hid, p in (self.long_term.get("open") or {}).items()
                    if p.get("status") == "REVIEW_DUE"
                ],
            },
        }
