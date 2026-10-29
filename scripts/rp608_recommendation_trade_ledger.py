#!/usr/bin/env python3
"""
RP-608 Prospective V1 — recommendation ↔ simulated-trade join ledger.

CRYPTO: links Policy B recommendations to trade-plan overlay dispositions
(PLAN_OPENED / SUPPRESSED / NO_TRADE / NO_PLAN) and overlay outcomes.

NSE: links Prospective recommendation tape rows to Estimand A shadow trades
in rp608_nse_trade_journal.jsonl (TRADE_OPENED / SUPPRESSED / NO_TRADE / NO_PLAN).

0.0% live order authority. Append-only under prospective_v1/journals/.
"""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

from scripts.rp608_prospective_v1 import (
    IST,
    Market,
    PROTOCOL_ID,
    START_RULE_VERSION,
    assert_write_allowed,
    ensure_namespace,
    is_market_started,
    load_start_event,
)
from scripts.rp608_prospective_v1_observer import journals_dir

LIVE_ORDER_AUTHORITY = 0.0
LABEL = "prospective_recommendation_trade_link"
CAPTURE_LIVE_AT_CLOSE = "live_at_close"
CAPTURE_BACKFILLED = "backfilled"


def _ms_to_utc_iso(ms: Optional[int]) -> Optional[str]:
    if ms is None:
        return None
    try:
        return datetime.fromtimestamp(int(ms) / 1000.0, tz=timezone.utc).isoformat()
    except (TypeError, ValueError, OSError, OverflowError):
        return None


def _parse_utc(value: Optional[str]) -> Optional[datetime]:
    if not value:
        return None
    try:
        s = str(value).strip()
        if s.endswith("Z"):
            s = s[:-1] + "+00:00"
        dt = datetime.fromisoformat(s)
        if dt.tzinfo is None:
            dt = dt.replace(tzinfo=timezone.utc)
        return dt.astimezone(timezone.utc)
    except Exception:
        return None


def start_event_provenance(market: Market) -> Dict[str, Any]:
    """Authoritative commencement stamps for join-row governance."""
    ev = load_start_event(market) or {}
    return {
        "start_event_utc": ev.get("start_event_utc"),
        "eligible_session_boundary": ev.get("eligible_session_boundary"),
        "eligible_session_open_utc": ev.get("eligible_session_open_utc"),
        "commencement_state": ev.get("state"),
    }


def resolve_source_event_time(
    *,
    close_time_ms: Optional[int] = None,
    source_recorded_utc: Optional[str] = None,
    source_event_ms: Optional[int] = None,
    wall_utc: Optional[datetime] = None,
) -> Dict[str, Any]:
    """
    Prefer source event time over writer wall clock.

    Priority: source_event_ms → close_time_ms → source_recorded_utc → wall clock.
    (Bar/exit exchange time beats tape writer stamps, which may themselves be backfills.)
    """
    wall = wall_utc or datetime.now(timezone.utc)
    dt = None
    source = None
    iso = _ms_to_utc_iso(source_event_ms)
    dt = _parse_utc(iso)
    if dt is not None:
        source = "source_event_ms"
    if dt is None:
        iso = _ms_to_utc_iso(close_time_ms)
        dt = _parse_utc(iso)
        if dt is not None:
            source = "close_time_ms"
    if dt is None:
        dt = _parse_utc(source_recorded_utc)
        if dt is not None:
            source = "source_recorded_utc"
    if dt is None:
        dt = wall
        source = "wall_clock_fallback"
    return {
        "recorded_utc": dt.isoformat(),
        "recorded_ist": dt.astimezone(IST).isoformat(),
        "recorded_time_source": source,
        "writer_wall_utc": wall.isoformat(),
        "source_recorded_utc": source_recorded_utc,
    }


def capture_stamp(
    *,
    market: Market,
    capture_provenance: str,
    close_time_ms: Optional[int] = None,
    source_recorded_utc: Optional[str] = None,
    source_event_ms: Optional[int] = None,
) -> Dict[str, Any]:
    """Common governance + timing fields for every join row."""
    wall = datetime.now(timezone.utc)
    out: Dict[str, Any] = {
        **start_event_provenance(market),
        **resolve_source_event_time(
            close_time_ms=close_time_ms,
            source_recorded_utc=source_recorded_utc,
            source_event_ms=source_event_ms,
            wall_utc=wall,
        ),
        "capture_provenance": capture_provenance,
        "live_order_authority": LIVE_ORDER_AUTHORITY,
    }
    if capture_provenance == CAPTURE_BACKFILLED:
        out["rebuild"] = True
        out["rebuilt_at_utc"] = wall.isoformat()
    else:
        out["rebuild"] = False
    return out

DEFAULT_EVENTS = Path(
    "live_capture/shadow_validation/crypto/trade_plan_overlay_v1/trade_plan_events.jsonl"
)
DEFAULT_OUTCOMES = Path(
    "live_capture/shadow_validation/crypto/trade_plan_overlay_v1/trade_plan_outcomes.jsonl"
)
DEFAULT_NSE_TRADE_JOURNAL = Path(
    "live_capture/shadow_validation/nse/rp608_nse_trade_journal.jsonl"
)


def recommendation_trade_journal_path(market: Market = Market.CRYPTO) -> Path:
    return journals_dir() / f"{market.value}_recommendation_trades.jsonl"


def classify_trade_link(
    *,
    decision: str,
    bar_idx: int,
    plan: Optional[Dict[str, Any]],
) -> str:
    """Map Policy B decision + overlay result to a stable research label."""
    if decision not in ("LONG", "SHORT", "RECOMMEND_LONG", "RECOMMEND_SHORT"):
        return "NO_TRADE"
    if not plan:
        return "NO_PLAN"
    src = plan.get("source_bar_idx")
    if src is not None and int(src) == int(bar_idx) and plan.get("state") in (
        "OPEN",
        "CLOSED_TP",
        "CLOSED_SL",
        "TIME_EXIT",
    ):
        return "PLAN_OPENED"
    if plan.get("state") == "OPEN":
        return "SUPPRESSED"
    return "NO_PLAN"


def classify_nse_trade_link(
    *,
    decision: str,
    decision_code: Optional[str] = None,
    opened_trade: Optional[Dict[str, Any]] = None,
    blocking_trade: Optional[Dict[str, Any]] = None,
) -> str:
    """
    Map NSE recommendation + Estimand A lifecycle to a stable research label.

    TRADE_OPENED wins even if decision is NO_TRADE (entry path is signal-gated;
    RECOMMEND_LONG also requires a valid quote — they can diverge).
    """
    if opened_trade is not None:
        return "TRADE_OPENED"
    if decision_code == "EXISTING_POSITION_ACTIVE" or blocking_trade is not None:
        return "SUPPRESSED"
    if decision in ("RECOMMEND_LONG", "RECOMMEND_SHORT", "LONG", "SHORT"):
        return "NO_PLAN"
    return "NO_TRADE"


def append_recommendation_trade_link(
    *,
    market: Market,
    symbol: str,
    session_date: str,
    bar_idx: int,
    decision: str,
    secondary_decision: Optional[str] = None,
    close_time_ms: Optional[int] = None,
    reference_price: Optional[float] = None,
    is_actionable: Optional[bool] = None,
    regime: Optional[str] = None,
    plan: Optional[Dict[str, Any]] = None,
    trade_link: Optional[str] = None,
    outcome: Optional[Dict[str, Any]] = None,
    extra: Optional[Dict[str, Any]] = None,
) -> Optional[Dict[str, Any]]:
    """Append one CRYPTO recommendation↔trade-plan join row (observational only)."""
    if market != Market.CRYPTO:
        return None
    if not is_market_started(market):
        return None

    actionable = is_actionable
    if actionable is None:
        actionable = decision in ("LONG", "SHORT", "RECOMMEND_LONG", "RECOMMEND_SHORT")

    link = trade_link or classify_trade_link(decision=decision, bar_idx=bar_idx, plan=plan)
    record: Dict[str, Any] = {
        "protocol_id": PROTOCOL_ID,
        "start_rule_version": START_RULE_VERSION,
        "label": LABEL,
        "event_type": "RECOMMENDATION_TRADE_LINK",
        "market": market.value,
        "symbol": symbol,
        "session_date": session_date,
        "bar_idx": bar_idx,
        "close_time_ms": close_time_ms,
        **capture_stamp(
            market=market,
            capture_provenance=CAPTURE_LIVE_AT_CLOSE,
            close_time_ms=close_time_ms,
        ),
        "decision": decision,
        "secondary_decision": secondary_decision,
        "is_actionable": bool(actionable),
        "reference_price": reference_price,
        "regime": regime,
        "trade_link": link,
        "plan_id": (plan or {}).get("plan_id"),
        "plan_source_bar_idx": (plan or {}).get("source_bar_idx"),
        "plan_direction": (plan or {}).get("direction"),
        "plan_state": (plan or {}).get("state"),
        "entry_price": (plan or {}).get("entry_price"),
        "stop_loss_price": (plan or {}).get("stop_loss_price"),
        "profit_target_price": (plan or {}).get("profit_target_price"),
    }
    if link == "SUPPRESSED" and plan:
        record["blocking_plan_id"] = plan.get("plan_id")
        record["blocking_plan_source_bar_idx"] = plan.get("source_bar_idx")
    if outcome:
        record["outcome_attached"] = True
        record["outcome_state"] = outcome.get("state")
        record["exit_reason"] = outcome.get("exit_reason")
        record["exit_price"] = outcome.get("exit_price")
        record["net_pnl_usdt"] = outcome.get("net_pnl_usdt")
        record["gross_pnl_usdt"] = outcome.get("gross_pnl_usdt")
        record["holding_duration_seconds"] = outcome.get("holding_duration_seconds")
    else:
        record["outcome_attached"] = False
    if extra:
        record["extra"] = extra

    ensure_namespace()
    path = recommendation_trade_journal_path(market)
    assert_write_allowed(path)
    with path.open("a", encoding="utf-8") as fh:
        fh.write(json.dumps(record, separators=(",", ":")) + "\n")
    return record


def append_nse_recommendation_trade_link(
    *,
    symbol: str,
    session_date: str,
    bar_idx: int,
    decision: str,
    decision_code: Optional[str] = None,
    close_time_ms: Optional[int] = None,
    reference_price: Optional[float] = None,
    is_actionable: Optional[bool] = None,
    regime: Optional[str] = None,
    session_idx: Optional[int] = None,
    opened_trade: Optional[Dict[str, Any]] = None,
    blocking_trade: Optional[Dict[str, Any]] = None,
    closed_trade: Optional[Dict[str, Any]] = None,
    trade_link: Optional[str] = None,
    extra: Optional[Dict[str, Any]] = None,
) -> Optional[Dict[str, Any]]:
    """Append one NSE recommendation↔Estimand A trade join row."""
    market = Market.NSE
    if not is_market_started(market):
        return None

    actionable = is_actionable
    if actionable is None:
        actionable = decision in ("RECOMMEND_LONG", "RECOMMEND_SHORT", "LONG", "SHORT")

    link = trade_link or classify_nse_trade_link(
        decision=decision,
        decision_code=decision_code,
        opened_trade=opened_trade,
        blocking_trade=blocking_trade,
    )
    trade = opened_trade or closed_trade or blocking_trade or {}
    record: Dict[str, Any] = {
        "protocol_id": PROTOCOL_ID,
        "start_rule_version": START_RULE_VERSION,
        "label": LABEL,
        "event_type": "RECOMMENDATION_TRADE_LINK",
        "market": market.value,
        "symbol": symbol,
        "session_date": session_date,
        "session_idx": session_idx if session_idx is not None else trade.get("session_idx"),
        "bar_idx": bar_idx,
        "close_time_ms": close_time_ms,
        **capture_stamp(
            market=market,
            capture_provenance=CAPTURE_LIVE_AT_CLOSE,
            close_time_ms=close_time_ms,
        ),
        "decision": decision,
        "decision_code": decision_code,
        "is_actionable": bool(actionable),
        "reference_price": reference_price,
        "regime": regime,
        "trade_link": link,
        "estimand": trade.get("estimand") or "ESTIMAND_A",
        "trade_id": trade.get("trade_id"),
        "plan_id": trade.get("trade_id"),  # alias for cross-market consumers
        "plan_source_bar_idx": trade.get("entry_bar_idx"),
        "plan_direction": trade.get("side"),
        "plan_state": "CLOSED" if closed_trade else ("OPEN" if (opened_trade or blocking_trade) else None),
        "entry_price": trade.get("entry_price"),
        "side": trade.get("side"),
        "quantity": trade.get("quantity"),
    }
    if link == "SUPPRESSED" and blocking_trade:
        record["blocking_trade_id"] = blocking_trade.get("trade_id")
        record["blocking_plan_id"] = blocking_trade.get("trade_id")
        record["blocking_plan_source_bar_idx"] = blocking_trade.get("entry_bar_idx")
    if closed_trade:
        record["outcome_attached"] = True
        record["outcome_state"] = closed_trade.get("exit_reason")
        record["exit_reason"] = closed_trade.get("exit_reason")
        record["exit_price"] = closed_trade.get("exit_price")
        record["exit_bar_idx"] = closed_trade.get("exit_bar_idx")
        record["net_pnl_inr"] = closed_trade.get("net_pnl_inr")
        record["gross_pnl_inr"] = closed_trade.get("gross_pnl_inr")
        record["net_return_bps"] = closed_trade.get("net_return_bps")
        record["gross_return_bps"] = closed_trade.get("gross_return_bps")
    else:
        record["outcome_attached"] = False
    if extra:
        record["extra"] = extra

    ensure_namespace()
    path = recommendation_trade_journal_path(market)
    assert_write_allowed(path)
    with path.open("a", encoding="utf-8") as fh:
        fh.write(json.dumps(record, separators=(",", ":")) + "\n")
    return record


def append_trade_outcome_link(
    *,
    market: Market,
    outcome: Dict[str, Any],
) -> Optional[Dict[str, Any]]:
    """Append a TRADE_CLOSED join event when a CRYPTO simulated plan resolves."""
    if market != Market.CRYPTO:
        return None
    if not is_market_started(market):
        return None

    exit_ms = outcome.get("exit_time_ms")
    record: Dict[str, Any] = {
        "protocol_id": PROTOCOL_ID,
        "start_rule_version": START_RULE_VERSION,
        "label": LABEL,
        "event_type": "TRADE_CLOSED",
        "market": market.value,
        "symbol": "BTCUSDT",
        "close_time_ms": exit_ms,
        **capture_stamp(
            market=market,
            capture_provenance=CAPTURE_LIVE_AT_CLOSE,
            close_time_ms=exit_ms if exit_ms is not None else outcome.get("entry_time_ms"),
            source_event_ms=exit_ms if exit_ms is not None else None,
        ),
        "bar_idx": outcome.get("source_bar_idx"),
        "plan_id": outcome.get("plan_id"),
        "plan_source_bar_idx": outcome.get("source_bar_idx"),
        "plan_direction": outcome.get("direction"),
        "plan_state": outcome.get("state"),
        "trade_link": "PLAN_CLOSED",
        "entry_price": outcome.get("entry_price"),
        "exit_price": outcome.get("exit_price"),
        "exit_reason": outcome.get("exit_reason"),
        "net_pnl_usdt": outcome.get("net_pnl_usdt"),
        "gross_pnl_usdt": outcome.get("gross_pnl_usdt"),
        "holding_duration_seconds": outcome.get("holding_duration_seconds"),
        "outcome_attached": True,
    }
    ensure_namespace()
    path = recommendation_trade_journal_path(market)
    assert_write_allowed(path)
    with path.open("a", encoding="utf-8") as fh:
        fh.write(json.dumps(record, separators=(",", ":")) + "\n")
    return record


def append_nse_trade_closed(*, trade: Dict[str, Any]) -> Optional[Dict[str, Any]]:
    """Append TRADE_CLOSED when an NSE Estimand A shadow trade resolves."""
    market = Market.NSE
    if not is_market_started(market):
        return None

    exit_ms = trade.get("exit_timestamp_utc_ms")
    record: Dict[str, Any] = {
        "protocol_id": PROTOCOL_ID,
        "start_rule_version": START_RULE_VERSION,
        "label": LABEL,
        "event_type": "TRADE_CLOSED",
        "market": market.value,
        "symbol": trade.get("symbol"),
        "session_idx": trade.get("session_idx"),
        "close_time_ms": exit_ms,
        **capture_stamp(
            market=market,
            capture_provenance=CAPTURE_LIVE_AT_CLOSE,
            close_time_ms=exit_ms if exit_ms is not None else trade.get("entry_timestamp_utc_ms"),
            source_event_ms=exit_ms if exit_ms is not None else trade.get("entry_timestamp_utc_ms"),
        ),
        "bar_idx": trade.get("entry_bar_idx"),
        "estimand": trade.get("estimand") or "ESTIMAND_A",
        "trade_id": trade.get("trade_id"),
        "plan_id": trade.get("trade_id"),
        "plan_source_bar_idx": trade.get("entry_bar_idx"),
        "plan_direction": trade.get("side"),
        "plan_state": "CLOSED",
        "trade_link": "TRADE_CLOSED",
        "entry_price": trade.get("entry_price"),
        "exit_price": trade.get("exit_price"),
        "exit_bar_idx": trade.get("exit_bar_idx"),
        "exit_reason": trade.get("exit_reason"),
        "net_pnl_inr": trade.get("net_pnl_inr"),
        "gross_pnl_inr": trade.get("gross_pnl_inr"),
        "net_return_bps": trade.get("net_return_bps"),
        "gross_return_bps": trade.get("gross_return_bps"),
        "quantity": trade.get("quantity"),
        "outcome_attached": True,
    }
    ensure_namespace()
    path = recommendation_trade_journal_path(market)
    assert_write_allowed(path)
    with path.open("a", encoding="utf-8") as fh:
        fh.write(json.dumps(record, separators=(",", ":")) + "\n")
    return record


def _load_jsonl(path: Path) -> List[Dict[str, Any]]:
    if not path.exists():
        return []
    rows: List[Dict[str, Any]] = []
    with path.open(encoding="utf-8") as fh:
        for line in fh:
            line = line.strip()
            if line:
                rows.append(json.loads(line))
    return rows


def _bar_from_plan_id(plan_id: Optional[str]) -> Optional[int]:
    if not plan_id:
        return None
    try:
        return int(str(plan_id).split("-")[1])
    except Exception:
        return None


def rebuild_crypto_ledger(
    *,
    recommendations_path: Path,
    events_path: Path,
    outcomes_path: Path,
    out_path: Optional[Path] = None,
) -> Path:
    """
    Rebuild the join ledger from existing rec tape + overlay events/outcomes.
    Overwrites out_path (research rebuild; not a live append).
    """
    recs = _load_jsonl(recommendations_path)
    events = _load_jsonl(events_path)
    outcomes = _load_jsonl(outcomes_path)

    created = {
        _bar_from_plan_id(e.get("plan_id")): e
        for e in events
        if e.get("event_type") == "PLAN_CREATED" and _bar_from_plan_id(e.get("plan_id")) is not None
    }
    supp_by_bar = {
        int(e["suppressed_card_bar_idx"]): e
        for e in events
        if e.get("event_type") == "SUPPRESSED_OPEN_PLAN" and e.get("suppressed_card_bar_idx") is not None
    }
    outs_by_bar = {o.get("source_bar_idx"): o for o in outcomes if o.get("source_bar_idx") is not None}
    outs_by_plan = {o.get("plan_id"): o for o in outcomes if o.get("plan_id")}

    ensure_namespace()
    target = out_path or recommendation_trade_journal_path(Market.CRYPTO)
    assert_write_allowed(target)

    rows: List[Dict[str, Any]] = []
    for r in recs:
        b = int(r["bar_idx"])
        decision = str(r.get("decision") or "NO_TRADE")
        actionable = bool(r.get("is_actionable"))
        plan_snapshot: Dict[str, Any] = {}
        outcome = None
        if not actionable or decision not in ("LONG", "SHORT"):
            link = "NO_TRADE"
        elif b in created:
            link = "PLAN_OPENED"
            c = created[b]
            plan_snapshot = {
                "plan_id": c.get("plan_id"),
                "source_bar_idx": b,
                "direction": c.get("direction"),
                "state": "OPEN",
                "entry_price": c.get("entry_price"),
                "stop_loss_price": c.get("stop_loss_price"),
                "profit_target_price": c.get("profit_target_price"),
            }
            outcome = outs_by_bar.get(b) or outs_by_plan.get(c.get("plan_id"))
            if outcome:
                plan_snapshot["state"] = outcome.get("state")
        elif b in supp_by_bar:
            link = "SUPPRESSED"
            s = supp_by_bar[b]
            plan_snapshot = {
                "plan_id": s.get("plan_id"),
                "source_bar_idx": _bar_from_plan_id(s.get("plan_id")),
                "direction": s.get("direction") or s.get("suppressed_direction"),
                "state": "OPEN",
                "entry_price": s.get("entry_price"),
                "stop_loss_price": s.get("stop_loss_price"),
                "profit_target_price": s.get("profit_target_price"),
            }
        else:
            link = "NO_PLAN"

        close_ms = r.get("close_time_ms")
        row: Dict[str, Any] = {
            "protocol_id": PROTOCOL_ID,
            "start_rule_version": START_RULE_VERSION,
            "label": LABEL,
            "event_type": "RECOMMENDATION_TRADE_LINK",
            "market": "CRYPTO",
            "symbol": r.get("symbol") or "BTCUSDT",
            "session_date": r.get("session_date"),
            "bar_idx": b,
            "close_time_ms": close_ms,
            **capture_stamp(
                market=Market.CRYPTO,
                capture_provenance=CAPTURE_BACKFILLED,
                close_time_ms=close_ms,
                source_recorded_utc=r.get("recorded_utc"),
            ),
            "decision": decision,
            "secondary_decision": r.get("secondary_decision"),
            "is_actionable": actionable,
            "reference_price": r.get("reference_price"),
            "regime": r.get("regime"),
            "trade_link": link,
            "plan_id": plan_snapshot.get("plan_id"),
            "plan_source_bar_idx": plan_snapshot.get("source_bar_idx"),
            "plan_direction": plan_snapshot.get("direction"),
            "plan_state": plan_snapshot.get("state"),
            "entry_price": plan_snapshot.get("entry_price"),
            "stop_loss_price": plan_snapshot.get("stop_loss_price"),
            "profit_target_price": plan_snapshot.get("profit_target_price"),
        }
        if link == "SUPPRESSED":
            row["blocking_plan_id"] = plan_snapshot.get("plan_id")
            row["blocking_plan_source_bar_idx"] = plan_snapshot.get("source_bar_idx")
        if outcome:
            row["outcome_attached"] = True
            row["outcome_state"] = outcome.get("state")
            row["exit_reason"] = outcome.get("exit_reason")
            row["exit_price"] = outcome.get("exit_price")
            row["net_pnl_usdt"] = outcome.get("net_pnl_usdt")
            row["gross_pnl_usdt"] = outcome.get("gross_pnl_usdt")
            row["holding_duration_seconds"] = outcome.get("holding_duration_seconds")
        else:
            row["outcome_attached"] = False
        rows.append(row)

        if outcome and link == "PLAN_OPENED":
            exit_ms = outcome.get("exit_time_ms")
            rows.append(
                {
                    "protocol_id": PROTOCOL_ID,
                    "start_rule_version": START_RULE_VERSION,
                    "label": LABEL,
                    "event_type": "TRADE_CLOSED",
                    "market": "CRYPTO",
                    "symbol": r.get("symbol") or "BTCUSDT",
                    "close_time_ms": exit_ms,
                    **capture_stamp(
                        market=Market.CRYPTO,
                        capture_provenance=CAPTURE_BACKFILLED,
                        close_time_ms=exit_ms if exit_ms is not None else close_ms,
                        source_event_ms=exit_ms,
                        source_recorded_utc=r.get("recorded_utc") if exit_ms is None else None,
                    ),
                    "bar_idx": b,
                    "plan_id": outcome.get("plan_id"),
                    "plan_source_bar_idx": outcome.get("source_bar_idx"),
                    "plan_direction": outcome.get("direction"),
                    "plan_state": outcome.get("state"),
                    "trade_link": "PLAN_CLOSED",
                    "entry_price": outcome.get("entry_price"),
                    "exit_price": outcome.get("exit_price"),
                    "exit_reason": outcome.get("exit_reason"),
                    "net_pnl_usdt": outcome.get("net_pnl_usdt"),
                    "gross_pnl_usdt": outcome.get("gross_pnl_usdt"),
                    "holding_duration_seconds": outcome.get("holding_duration_seconds"),
                    "outcome_attached": True,
                }
            )

    tmp = target.with_suffix(target.suffix + ".tmp")
    with tmp.open("w", encoding="utf-8") as fh:
        for row in rows:
            fh.write(json.dumps(row, separators=(",", ":")) + "\n")
    tmp.replace(target)
    return target


def _nse_trade_key(symbol: str, entry_bar_idx: int, session_idx: Optional[int]) -> Tuple[str, int, Optional[int]]:
    return (str(symbol), int(entry_bar_idx), int(session_idx) if session_idx is not None else None)


def rebuild_nse_ledger(
    *,
    recommendations_path: Path,
    trade_journal_path: Path,
    out_path: Optional[Path] = None,
) -> Path:
    """
    Rebuild NSE join ledger from Prospective rec tape + Estimand A trade journal.
    Overwrites out_path (research rebuild; not a live append).
    """
    recs = _load_jsonl(recommendations_path)
    trades = _load_jsonl(trade_journal_path)

    by_entry: Dict[Tuple[str, int, Optional[int]], Dict[str, Any]] = {}
    by_symbol_bar: Dict[Tuple[str, int], Dict[str, Any]] = {}
    for t in trades:
        if t.get("entry_bar_idx") is None or not t.get("symbol"):
            continue
        key = _nse_trade_key(t["symbol"], int(t["entry_bar_idx"]), t.get("session_idx"))
        by_entry[key] = t
        by_symbol_bar[(str(t["symbol"]), int(t["entry_bar_idx"]))] = t

    ensure_namespace()
    target = out_path or recommendation_trade_journal_path(Market.NSE)
    assert_write_allowed(target)

    rows: List[Dict[str, Any]] = []
    emitted_trade_ids = set()

    for r in recs:
        b = int(r["bar_idx"])
        symbol = str(r.get("symbol") or "")
        decision = str(r.get("decision") or "NO_TRADE")
        actionable = bool(r.get("is_actionable"))
        extra = r.get("extra") if isinstance(r.get("extra"), dict) else {}
        decision_code = r.get("decision_code") or extra.get("decision_code")
        session_idx = extra.get("session_idx")
        if session_idx is None:
            session_idx = r.get("session_idx")

        trade = None
        if session_idx is not None:
            trade = by_entry.get(_nse_trade_key(symbol, b, int(session_idx)))
        if trade is None:
            trade = by_symbol_bar.get((symbol, b))

        if trade is not None:
            link = "TRADE_OPENED"
            opened = trade
            blocking = None
            closed = trade if trade.get("exit_reason") is not None else None
        elif decision_code == "EXISTING_POSITION_ACTIVE":
            link = "SUPPRESSED"
            opened = None
            blocking = {"trade_id": None, "entry_bar_idx": None, "side": "LONG", "estimand": "ESTIMAND_A"}
            closed = None
        elif decision in ("RECOMMEND_LONG", "RECOMMEND_SHORT"):
            link = "NO_PLAN"
            opened = blocking = closed = None
        else:
            link = "NO_TRADE"
            opened = blocking = closed = None

        close_ms = r.get("close_time_ms")
        row: Dict[str, Any] = {
            "protocol_id": PROTOCOL_ID,
            "start_rule_version": START_RULE_VERSION,
            "label": LABEL,
            "event_type": "RECOMMENDATION_TRADE_LINK",
            "market": "NSE",
            "symbol": symbol,
            "session_date": r.get("session_date"),
            "session_idx": session_idx,
            "bar_idx": b,
            "close_time_ms": close_ms,
            **capture_stamp(
                market=Market.NSE,
                capture_provenance=CAPTURE_BACKFILLED,
                close_time_ms=close_ms,
                source_recorded_utc=r.get("recorded_utc"),
            ),
            "decision": decision,
            "decision_code": decision_code,
            "is_actionable": actionable,
            "reference_price": r.get("reference_price"),
            "regime": r.get("regime"),
            "trade_link": link,
            "estimand": (trade or {}).get("estimand") or "ESTIMAND_A",
            "trade_id": (trade or {}).get("trade_id"),
            "plan_id": (trade or {}).get("trade_id"),
            "plan_source_bar_idx": (trade or {}).get("entry_bar_idx"),
            "plan_direction": (trade or {}).get("side"),
            "plan_state": ("CLOSED" if closed else ("OPEN" if opened else None)),
            "entry_price": (trade or {}).get("entry_price"),
            "side": (trade or {}).get("side"),
            "quantity": (trade or {}).get("quantity"),
            "outcome_attached": bool(closed),
        }
        if closed:
            row["outcome_state"] = closed.get("exit_reason")
            row["exit_reason"] = closed.get("exit_reason")
            row["exit_price"] = closed.get("exit_price")
            row["exit_bar_idx"] = closed.get("exit_bar_idx")
            row["net_pnl_inr"] = closed.get("net_pnl_inr")
            row["gross_pnl_inr"] = closed.get("gross_pnl_inr")
            row["net_return_bps"] = closed.get("net_return_bps")
            row["gross_return_bps"] = closed.get("gross_return_bps")
        rows.append(row)

        if closed and closed.get("trade_id") and closed["trade_id"] not in emitted_trade_ids:
            emitted_trade_ids.add(closed["trade_id"])
            exit_ms = closed.get("exit_timestamp_utc_ms")
            rows.append(
                {
                    "protocol_id": PROTOCOL_ID,
                    "start_rule_version": START_RULE_VERSION,
                    "label": LABEL,
                    "event_type": "TRADE_CLOSED",
                    "market": "NSE",
                    "symbol": closed.get("symbol"),
                    "session_idx": closed.get("session_idx"),
                    "close_time_ms": exit_ms,
                    **capture_stamp(
                        market=Market.NSE,
                        capture_provenance=CAPTURE_BACKFILLED,
                        close_time_ms=exit_ms if exit_ms is not None else closed.get("entry_timestamp_utc_ms"),
                        source_event_ms=exit_ms if exit_ms is not None else closed.get("entry_timestamp_utc_ms"),
                    ),
                    "bar_idx": closed.get("entry_bar_idx"),
                    "estimand": closed.get("estimand") or "ESTIMAND_A",
                    "trade_id": closed.get("trade_id"),
                    "plan_id": closed.get("trade_id"),
                    "plan_source_bar_idx": closed.get("entry_bar_idx"),
                    "plan_direction": closed.get("side"),
                    "plan_state": "CLOSED",
                    "trade_link": "TRADE_CLOSED",
                    "entry_price": closed.get("entry_price"),
                    "exit_price": closed.get("exit_price"),
                    "exit_bar_idx": closed.get("exit_bar_idx"),
                    "exit_reason": closed.get("exit_reason"),
                    "net_pnl_inr": closed.get("net_pnl_inr"),
                    "gross_pnl_inr": closed.get("gross_pnl_inr"),
                    "net_return_bps": closed.get("net_return_bps"),
                    "gross_return_bps": closed.get("gross_return_bps"),
                    "quantity": closed.get("quantity"),
                    "outcome_attached": True,
                }
            )

    tmp = target.with_suffix(target.suffix + ".tmp")
    with tmp.open("w", encoding="utf-8") as fh:
        for row in rows:
            fh.write(json.dumps(row, separators=(",", ":")) + "\n")
    tmp.replace(target)
    return target


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    sub = ap.add_subparsers(dest="cmd", required=True)
    reb = sub.add_parser("rebuild-crypto", help="Rebuild CRYPTO join ledger from existing journals")
    reb.add_argument("--recommendations", type=Path, default=None)
    reb.add_argument("--events", type=Path, default=DEFAULT_EVENTS)
    reb.add_argument("--outcomes", type=Path, default=DEFAULT_OUTCOMES)
    reb.add_argument("--out", type=Path, default=None)

    reb_n = sub.add_parser("rebuild-nse", help="Rebuild NSE join ledger from rec tape + Estimand A journal")
    reb_n.add_argument("--recommendations", type=Path, default=None)
    reb_n.add_argument("--trades", type=Path, default=DEFAULT_NSE_TRADE_JOURNAL)
    reb_n.add_argument("--out", type=Path, default=None)
    args = ap.parse_args()

    from scripts.rp608_prospective_v1_observer import recommendation_journal_path

    if args.cmd == "rebuild-crypto":
        rec_path = args.recommendations or recommendation_journal_path(Market.CRYPTO)
        out = rebuild_crypto_ledger(
            recommendations_path=rec_path,
            events_path=args.events,
            outcomes_path=args.outcomes,
            out_path=args.out,
        )
        n = sum(1 for _ in out.open()) if out.exists() else 0
        print(f"rebuilt {out} rows={n}")
        return 0

    if args.cmd == "rebuild-nse":
        rec_path = args.recommendations or recommendation_journal_path(Market.NSE)
        out = rebuild_nse_ledger(
            recommendations_path=rec_path,
            trade_journal_path=args.trades,
            out_path=args.out,
        )
        n = sum(1 for _ in out.open()) if out.exists() else 0
        print(f"rebuilt {out} rows={n}")
        return 0
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
