#!/usr/bin/env python3
"""
RP-610 Crypto H1/H2/H3 observation engine (paper spot-sim only).

Deterministic exit rules for multi-day / multi-week / long-term holds.
Does not mutate RP-608 crypto trade-plan overlay (H0).
"""

from __future__ import annotations

import hashlib
import json
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional

from scripts.rp610_crypto_multi_horizon import (
    DEFAULT_NOTIONAL_USDT,
    DEFAULT_RT_FRICTION_BPS,
    HORIZONS,
    LIVE_ORDER_AUTHORITY,
    PROTOCOL_ID,
    SCHEMA_VERSION,
    append_jsonl,
    candidates_path,
    daily_log_path,
    day_path,
    estimate_crypto_friction_usdt,
    long_path,
    week_path,
    write_horizon_registry,
)

UTC = timezone.utc


def _parse_ts(value: Any) -> Optional[datetime]:
    if value is None:
        return None
    if isinstance(value, datetime):
        return value if value.tzinfo else value.replace(tzinfo=UTC)
    s = str(value).strip()
    if not s:
        return None
    if s.endswith("Z"):
        s = s[:-1] + "+00:00"
    try:
        dt = datetime.fromisoformat(s)
    except ValueError:
        return None
    return dt if dt.tzinfo else dt.replace(tzinfo=UTC)


def _iso(dt: datetime) -> str:
    return dt.astimezone(UTC).isoformat().replace("+00:00", "Z")


def _stable_id(*parts: Any) -> str:
    raw = "|".join(str(p) for p in parts)
    return hashlib.sha256(raw.encode("utf-8")).hexdigest()[:20]


def _load_jsonl(path: Path) -> List[Dict[str, Any]]:
    if not path.exists():
        return []
    rows: List[Dict[str, Any]] = []
    for line in path.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line:
            continue
        try:
            rows.append(json.loads(line))
        except json.JSONDecodeError:
            continue
    return rows


def build_candidate(
    *,
    symbol: str,
    side: str,
    entry_price: float,
    entry_ts: datetime,
    horizon_id: str,
    thesis: str = "",
    source_signal_id: str = "",
    notional_usdt: float = DEFAULT_NOTIONAL_USDT,
    stop_pct: Optional[float] = None,
    target_pct: Optional[float] = None,
) -> Dict[str, Any]:
    hz = HORIZONS[horizon_id]
    if hz.get("observational_link_only"):
        raise ValueError(f"{horizon_id} is H0 link-only; use RP-608 for intraday")
    side_u = side.upper()
    if side_u not in {"LONG", "SHORT"}:
        raise ValueError("side must be LONG or SHORT")
    qty = float(notional_usdt) / float(entry_price) if entry_price else 0.0
    stop = stop_pct if stop_pct is not None else hz.get("default_stop_pct")
    target = target_pct if target_pct is not None else hz.get("default_target_pct")
    hold_max = hz.get("hold_days_max")
    hold_min = int(hz.get("hold_days_min") or 0)
    max_exit_ts = None
    if hold_max is not None:
        max_exit_ts = _iso(entry_ts + timedelta(days=int(hold_max)))
    cand_id = _stable_id(PROTOCOL_ID, horizon_id, symbol, side_u, _iso(entry_ts), round(entry_price, 8))
    return {
        "schema_version": SCHEMA_VERSION,
        "protocol_id": PROTOCOL_ID,
        "candidate_id": cand_id,
        "horizon_id": horizon_id,
        "symbol": symbol.upper(),
        "side": side_u,
        "product_type": "SPOT_SIM",
        "live_order_authority": LIVE_ORDER_AUTHORITY,
        "entry_ts": _iso(entry_ts),
        "entry_price": float(entry_price),
        "quantity": qty,
        "notional_usdt": float(notional_usdt),
        "stop_pct": stop,
        "target_pct": target,
        "hold_days_min": hold_min,
        "hold_days_max": hold_max,
        "max_exit_ts": max_exit_ts,
        "thesis": thesis,
        "source_signal_id": source_signal_id,
        "status": "OPEN",
        "created_utc": _iso(datetime.now(UTC)),
    }


def admit_candidate(candidate: Dict[str, Any]) -> Dict[str, Any]:
    write_horizon_registry()
    path = candidates_path()
    existing = {r.get("candidate_id") for r in _load_jsonl(path)}
    if candidate["candidate_id"] in existing:
        return candidate
    append_jsonl(path, candidate)
    book = _book_path(candidate["horizon_id"])
    append_jsonl(book, {**candidate, "event": "ADMIT"})
    return candidate


def _book_path(horizon_id: str) -> Path:
    if horizon_id == "H1_MULTI_DAY":
        return day_path()
    if horizon_id == "H2_MULTI_WEEK":
        return week_path()
    if horizon_id == "H3_LONG_TERM":
        return long_path()
    raise ValueError(f"unsupported horizon for book: {horizon_id}")


def _pnl_usdt(side: str, entry: float, exit_px: float, qty: float) -> float:
    if side == "LONG":
        return (exit_px - entry) * qty
    return (entry - exit_px) * qty


def evaluate_open_hold(
    hold: Dict[str, Any],
    *,
    mark_ts: datetime,
    mark_price: float,
    force_close: bool = False,
    force_reason: str = "",
) -> Optional[Dict[str, Any]]:
    if hold.get("status") not in {None, "OPEN", "ADMIT"} and hold.get("event") not in {None, "ADMIT"}:
        if hold.get("status") == "CLOSED":
            return None
    entry = float(hold["entry_price"])
    qty = float(hold["quantity"])
    side = str(hold["side"]).upper()
    entry_ts = _parse_ts(hold["entry_ts"])
    if entry_ts is None:
        return None

    stop_pct = hold.get("stop_pct")
    target_pct = hold.get("target_pct")
    reason = None
    exit_px = float(mark_price)

    if force_close:
        reason = force_reason or "FORCE_CLOSE"
    else:
        if stop_pct is not None:
            stop_pct = float(stop_pct)
            if side == "LONG" and mark_price <= entry * (1.0 - stop_pct):
                reason = "INVALIDATION"
                exit_px = entry * (1.0 - stop_pct)
            elif side == "SHORT" and mark_price >= entry * (1.0 + stop_pct):
                reason = "INVALIDATION"
                exit_px = entry * (1.0 + stop_pct)
        if reason is None and target_pct is not None:
            target_pct = float(target_pct)
            if side == "LONG" and mark_price >= entry * (1.0 + target_pct):
                reason = "TARGET"
                exit_px = entry * (1.0 + target_pct)
            elif side == "SHORT" and mark_price <= entry * (1.0 - target_pct):
                reason = "TARGET"
                exit_px = entry * (1.0 - target_pct)
        max_exit = _parse_ts(hold.get("max_exit_ts"))
        if reason is None and max_exit is not None and mark_ts >= max_exit:
            reason = "MAX_HOLD_DAYS"
            exit_px = float(mark_price)
        # H3: no max hold; review cadence handled separately via mark_long_term_review

    if reason is None:
        return None

    gross = _pnl_usdt(side, entry, exit_px, qty)
    fr = estimate_crypto_friction_usdt(
        entry_price=entry,
        exit_price=exit_px,
        quantity=qty,
        rt_friction_bps=DEFAULT_RT_FRICTION_BPS,
    )
    net = gross - fr["total_friction_usdt"]
    hold_days = max(0.0, (mark_ts - entry_ts).total_seconds() / 86400.0)
    return {
        "schema_version": SCHEMA_VERSION,
        "protocol_id": PROTOCOL_ID,
        "event": "CLOSE",
        "status": "CLOSED",
        "candidate_id": hold.get("candidate_id"),
        "horizon_id": hold.get("horizon_id"),
        "symbol": hold.get("symbol"),
        "side": side,
        "product_type": "SPOT_SIM",
        "live_order_authority": LIVE_ORDER_AUTHORITY,
        "entry_ts": hold.get("entry_ts"),
        "entry_price": entry,
        "exit_ts": _iso(mark_ts),
        "exit_price": exit_px,
        "quantity": qty,
        "exit_reason": reason,
        "hold_days": round(hold_days, 4),
        "gross_pnl_usdt": round(gross, 6),
        "net_pnl_usdt": round(net, 6),
        "friction_usdt": fr["total_friction_usdt"],
        "friction_model_id": fr["friction_model_id"],
        "round_trip_friction_bps": fr["round_trip_friction_bps"],
        "closed_utc": _iso(datetime.now(UTC)),
    }


# Entry economics are immutable; plan parameters may be amended via append-only events.
_FROZEN_FIELDS = (
    "candidate_id",
    "entry_ts",
    "entry_price",
    "quantity",
    "notional_usdt",
    "side",
    "symbol",
    "product_type",
    "protocol_id",
    "parent_candidate_id",
)

_AMENDABLE_FIELDS = (
    "stop_pct",
    "target_pct",
    "hold_days_min",
    "hold_days_max",
    "max_exit_ts",
    "thesis",
    "thesis_version",
    "source_signal_id",
)


def _reconstruct_opens(rows: List[Dict[str, Any]]) -> Dict[str, Dict[str, Any]]:
    by_id: Dict[str, Dict[str, Any]] = {}
    for r in rows:
        cid = r.get("candidate_id")
        if not cid:
            continue
        event = r.get("event")
        if event in {"CLOSE", "SUPERSEDE"} or r.get("status") in {"CLOSED", "SUPERSEDED"}:
            by_id.pop(cid, None)
            continue
        if event == "MARK":
            if cid in by_id:
                by_id[cid]["last_mark_price"] = r.get("mark_price")
                by_id[cid]["last_mark_ts"] = r.get("mark_ts")
                by_id[cid]["last_mtm_gross_usdt"] = r.get("mtm_gross_usdt")
            continue
        if event == "AMEND":
            if cid not in by_id:
                continue
            base = dict(by_id[cid])
            amended = r.get("amended") or {}
            for k, v in amended.items():
                if k in _AMENDABLE_FIELDS or k == "horizon_id":
                    base[k] = v
            base["status"] = base.get("status") or "OPEN"
            base["last_amendment_id"] = r.get("amendment_id")
            base["last_amend_utc"] = r.get("amended_utc")
            base["amend_count"] = int(base.get("amend_count") or 0) + 1
            by_id[cid] = base
            continue
        if event in {"ADMIT", None} or r.get("status") in {"OPEN", "ADMIT", "HOLDING", None}:
            # Prefer first ADMIT as base; ignore later thin rows that aren't AMEND.
            if cid not in by_id or event == "ADMIT":
                by_id[cid] = dict(r)
                by_id[cid].setdefault("amend_count", 0)
    return by_id


def open_holds_from_book(horizon_id: str) -> List[Dict[str, Any]]:
    return list(_reconstruct_opens(_load_jsonl(_book_path(horizon_id))).values())


def find_open_hold(candidate_id: str) -> Optional[Dict[str, Any]]:
    for hid in ("H1_MULTI_DAY", "H2_MULTI_WEEK", "H3_LONG_TERM"):
        for hold in open_holds_from_book(hid):
            if hold.get("candidate_id") == candidate_id:
                return hold
    return None


def amend_hold(
    candidate_id: str,
    *,
    reason: str,
    stop_pct: Optional[float] = None,
    target_pct: Optional[float] = None,
    hold_days_max: Optional[int] = None,
    hold_days_min: Optional[int] = None,
    max_exit_ts: Optional[str] = None,
    thesis: Optional[str] = None,
    thesis_version: Optional[str] = None,
    source_signal_id: Optional[str] = None,
    new_horizon_id: Optional[str] = None,
    mark_price: Optional[float] = None,
    mark_ts: Optional[datetime] = None,
) -> Dict[str, Any]:
    """
    Append-only amendment. Never rewrites ADMIT.

    - Same-horizon field updates → event AMEND on the open book.
    - Horizon change → SUPERSEDE parent + ADMIT child (same entry economics).
    """
    hold = find_open_hold(candidate_id)
    if hold is None:
        raise ValueError(f"no open hold for candidate_id={candidate_id}")

    if new_horizon_id and new_horizon_id != hold.get("horizon_id"):
        return _roll_horizon(
            hold,
            new_horizon_id=new_horizon_id,
            reason=reason,
            stop_pct=stop_pct,
            target_pct=target_pct,
            hold_days_max=hold_days_max,
            hold_days_min=hold_days_min,
            thesis=thesis,
            thesis_version=thesis_version,
            source_signal_id=source_signal_id,
            mark_price=mark_price,
            mark_ts=mark_ts or datetime.now(UTC),
        )

    prior: Dict[str, Any] = {}
    amended: Dict[str, Any] = {}

    def _set(field: str, value: Any) -> None:
        if value is None:
            return
        prior[field] = hold.get(field)
        amended[field] = value

    _set("stop_pct", float(stop_pct) if stop_pct is not None else None)
    _set("target_pct", float(target_pct) if target_pct is not None else None)
    _set("hold_days_min", int(hold_days_min) if hold_days_min is not None else None)
    _set("hold_days_max", int(hold_days_max) if hold_days_max is not None else None)
    _set("thesis", thesis)
    _set("thesis_version", thesis_version)
    _set("source_signal_id", source_signal_id)

    if hold_days_max is not None or max_exit_ts is not None:
        entry_ts = _parse_ts(hold.get("entry_ts"))
        if max_exit_ts is not None:
            _set("max_exit_ts", max_exit_ts)
        elif entry_ts is not None and hold_days_max is not None:
            _set("max_exit_ts", _iso(entry_ts + timedelta(days=int(hold_days_max))))

    if not amended:
        raise ValueError("amend requires at least one changed field")

    amendment_id = _stable_id("AMEND", candidate_id, reason, _iso(datetime.now(UTC)), sorted(amended.keys()))
    row = {
        "schema_version": SCHEMA_VERSION,
        "protocol_id": PROTOCOL_ID,
        "event": "AMEND",
        "status": hold.get("status") or "OPEN",
        "amendment_id": amendment_id,
        "candidate_id": candidate_id,
        "horizon_id": hold.get("horizon_id"),
        "symbol": hold.get("symbol"),
        "reason": reason,
        "prior": prior,
        "amended": amended,
        "live_order_authority": LIVE_ORDER_AUTHORITY,
        "amended_utc": _iso(datetime.now(UTC)),
    }
    append_jsonl(_book_path(str(hold["horizon_id"])), row)
    append_jsonl(candidates_path(), {**row, "event": "AMEND_CANDIDATE"})
    effective = {**hold, **amended, "last_amendment_id": amendment_id}
    return {"amendment": row, "effective": effective}


def _roll_horizon(
    hold: Dict[str, Any],
    *,
    new_horizon_id: str,
    reason: str,
    stop_pct: Optional[float],
    target_pct: Optional[float],
    hold_days_max: Optional[int],
    hold_days_min: Optional[int],
    thesis: Optional[str],
    thesis_version: Optional[str],
    source_signal_id: Optional[str],
    mark_price: Optional[float],
    mark_ts: datetime,
) -> Dict[str, Any]:
    if new_horizon_id not in HORIZONS or HORIZONS[new_horizon_id].get("observational_link_only"):
        raise ValueError(f"cannot roll to {new_horizon_id}")
    old_hid = str(hold["horizon_id"])
    entry_ts = _parse_ts(hold["entry_ts"])
    if entry_ts is None:
        raise ValueError("hold missing entry_ts")

    child = build_candidate(
        symbol=str(hold["symbol"]),
        side=str(hold["side"]),
        entry_price=float(hold["entry_price"]),
        entry_ts=entry_ts,
        horizon_id=new_horizon_id,
        thesis=thesis if thesis is not None else str(hold.get("thesis") or ""),
        source_signal_id=source_signal_id
        if source_signal_id is not None
        else str(hold.get("source_signal_id") or ""),
        notional_usdt=float(hold.get("notional_usdt") or DEFAULT_NOTIONAL_USDT),
        stop_pct=stop_pct if stop_pct is not None else hold.get("stop_pct"),
        target_pct=target_pct if target_pct is not None else hold.get("target_pct"),
    )
    # Rebuild id so roll is unique even with same entry facts
    child["candidate_id"] = _stable_id(
        PROTOCOL_ID,
        "ROLL",
        hold["candidate_id"],
        new_horizon_id,
        _iso(mark_ts),
    )
    if hold_days_min is not None:
        child["hold_days_min"] = int(hold_days_min)
    if hold_days_max is not None:
        child["hold_days_max"] = int(hold_days_max)
        child["max_exit_ts"] = _iso(entry_ts + timedelta(days=int(hold_days_max)))
    if thesis_version is not None:
        child["thesis_version"] = thesis_version
    child["parent_candidate_id"] = hold["candidate_id"]
    child["roll_reason"] = reason
    if new_horizon_id == "H3_LONG_TERM":
        child["status"] = "HOLDING"

    supersede = {
        "schema_version": SCHEMA_VERSION,
        "protocol_id": PROTOCOL_ID,
        "event": "SUPERSEDE",
        "status": "SUPERSEDED",
        "candidate_id": hold["candidate_id"],
        "horizon_id": old_hid,
        "symbol": hold.get("symbol"),
        "supersede_reason": "HORIZON_CHANGED",
        "reason": reason,
        "child_candidate_id": child["candidate_id"],
        "new_horizon_id": new_horizon_id,
        "mark_ts": _iso(mark_ts),
        "mark_price": mark_price,
        "live_order_authority": LIVE_ORDER_AUTHORITY,
        "superseded_utc": _iso(datetime.now(UTC)),
        # No friction / no PnL close — observation continuity of same entry.
        "friction_usdt": 0.0,
        "net_pnl_usdt": None,
    }
    append_jsonl(_book_path(old_hid), supersede)
    admit_candidate(child)
    append_jsonl(
        candidates_path(),
        {
            "event": "HORIZON_ROLL",
            "parent_candidate_id": hold["candidate_id"],
            "child_candidate_id": child["candidate_id"],
            "from_horizon": old_hid,
            "to_horizon": new_horizon_id,
            "reason": reason,
            "rolled_utc": _iso(datetime.now(UTC)),
        },
    )
    return {"supersede": supersede, "child": child}


def mark_and_close_opens(
    *,
    horizon_id: str,
    mark_ts: datetime,
    mark_price: float,
    force_close: bool = False,
    force_reason: str = "",
) -> List[Dict[str, Any]]:
    closed: List[Dict[str, Any]] = []
    book = _book_path(horizon_id)
    for hold in open_holds_from_book(horizon_id):
        result = evaluate_open_hold(
            hold,
            mark_ts=mark_ts,
            mark_price=mark_price,
            force_close=force_close,
            force_reason=force_reason,
        )
        if result:
            append_jsonl(book, result)
            closed.append(result)
    return closed


def write_daily_observation(
    *,
    mark_ts: datetime,
    marks: Dict[str, float],
    notes: str = "",
) -> Dict[str, Any]:
    write_horizon_registry()
    summary: Dict[str, Any] = {
        "schema_version": SCHEMA_VERSION,
        "protocol_id": PROTOCOL_ID,
        "event": "DAILY_OBSERVATION",
        "observation_ts": _iso(mark_ts),
        "live_order_authority": LIVE_ORDER_AUTHORITY,
        "marks": marks,
        "open_counts": {},
        "closes_today": {},
        "notes": notes,
        "decision_status": "OBSERVE_ONLY_NO_CONCLUSION",
    }
    for hid in ("H1_MULTI_DAY", "H2_MULTI_WEEK", "H3_LONG_TERM"):
        opens = open_holds_from_book(hid)
        summary["open_counts"][hid] = len(opens)
        closes: List[Dict[str, Any]] = []
        for sym, px in marks.items():
            for hold in opens:
                if str(hold.get("symbol", "")).upper() != sym.upper():
                    continue
                result = evaluate_open_hold(hold, mark_ts=mark_ts, mark_price=float(px))
                if result:
                    append_jsonl(_book_path(hid), result)
                    closes.append(result)
        summary["closes_today"][hid] = len(closes)
        # mark-to-market snapshot for remaining opens
        for hold in open_holds_from_book(hid):
            sym = str(hold.get("symbol", "")).upper()
            if sym not in {k.upper() for k in marks}:
                continue
            px = float(marks[sym] if sym in marks else marks.get(sym.lower(), 0) or 0)
            if px <= 0:
                # try case-insensitive
                px = next((float(v) for k, v in marks.items() if k.upper() == sym), 0.0)
            if px <= 0:
                continue
            mtm = _pnl_usdt(str(hold["side"]).upper(), float(hold["entry_price"]), px, float(hold["quantity"]))
            append_jsonl(
                _book_path(hid),
                {
                    "event": "MARK",
                    "status": "OPEN",
                    "candidate_id": hold.get("candidate_id"),
                    "horizon_id": hid,
                    "symbol": sym,
                    "mark_ts": _iso(mark_ts),
                    "mark_price": px,
                    "mtm_gross_usdt": round(mtm, 6),
                    "live_order_authority": LIVE_ORDER_AUTHORITY,
                },
            )
    append_jsonl(daily_log_path(), summary)
    return summary


def bootstrap_demo_candidates(
    *,
    symbol: str = "BTCUSDT",
    entry_price: float = 68000.0,
    entry_ts: Optional[datetime] = None,
) -> List[Dict[str, Any]]:
    """Admit one paper hold per H1/H2/H3 for wiring / unit demos (not a signal)."""
    ts = entry_ts or datetime.now(UTC)
    out: List[Dict[str, Any]] = []
    for hid, side, thesis in (
        ("H1_MULTI_DAY", "LONG", "demo multi-day swing"),
        ("H2_MULTI_WEEK", "LONG", "demo multi-week position"),
        ("H3_LONG_TERM", "LONG", "demo long-term thesis"),
    ):
        c = build_candidate(
            symbol=symbol,
            side=side,
            entry_price=entry_price,
            entry_ts=ts,
            horizon_id=hid,
            thesis=thesis,
            source_signal_id="BOOTSTRAP_DEMO",
        )
        admit_candidate(c)
        out.append(c)
    return out


def summarize_books() -> Dict[str, Any]:
    write_horizon_registry()
    out: Dict[str, Any] = {"protocol_id": PROTOCOL_ID, "horizons": {}}
    for hid in ("H1_MULTI_DAY", "H2_MULTI_WEEK", "H3_LONG_TERM"):
        rows = _load_jsonl(_book_path(hid))
        closes = [r for r in rows if r.get("event") == "CLOSE"]
        net = sum(float(r.get("net_pnl_usdt") or 0) for r in closes)
        out["horizons"][hid] = {
            "open": len(open_holds_from_book(hid)),
            "closed": len(closes),
            "net_pnl_usdt_closed": round(net, 6),
        }
    return out
