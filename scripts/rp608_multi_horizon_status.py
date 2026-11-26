#!/usr/bin/env python3
"""
Aggregate RP-608/609/610 horizon recommendation surfaces for the live dashboard.

Observation-only. live_order_authority = 0.0.
Maps protocol horizons onto display classes:
  INTRADAY | MULTI_DAY | MULTI_WEEK | LONG_TERM
"""

from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional

WORKSPACE = Path(__file__).resolve().parent.parent
NSE_DIR = WORKSPACE / "live_capture" / "shadow_validation" / "nse"
CRYPTO_DIR = WORKSPACE / "live_capture" / "shadow_validation" / "crypto"
CRYPTO_OVERLAY_STATE = CRYPTO_DIR / "trade_plan_overlay_v1" / "overlay_state.json"

# Demo / seed theses must never surface as live MH books (use RP-608 fetches instead).
_DEMO_MARKERS = (
    "demo",
    "pi_demo",
    "framework_seed",
    "seed_only",
    "observe_build_001",  # placeholder portfolio plan id
)

CLASS_MAP_NSE = {
    "H0_INTRADAY": "INTRADAY",
    "H1_SWING": "MULTI_DAY",
    "H2_PORTFOLIO_BUILD": "MULTI_WEEK",
    "H3_LONG_TERM": "LONG_TERM",
}
CLASS_MAP_CRYPTO = {
    "H0_INTRADAY_60M": "INTRADAY",
    "H1_MULTI_DAY": "MULTI_DAY",
    "H2_MULTI_WEEK": "MULTI_WEEK",
    "H3_LONG_TERM": "LONG_TERM",
}
TERM_NSE = {
    "H0_INTRADAY": "15m bars · 60m / 180m max · square-off 15:00 (close-window 15:00–15:15)",
    "H1_SWING": "2–20 sessions (CNC swing)",
    "H2_PORTFOLIO_BUILD": "5–60 sessions (staggered CNC build)",
    "H3_LONG_TERM": "≥60 sessions / ≥90d thesis hold",
}
TERM_CRYPTO = {
    "H0_INTRADAY_60M": "≤60 minutes (trade-plan overlay)",
    "H1_MULTI_DAY": "2–14 calendar days",
    "H2_MULTI_WEEK": "14–56 calendar days",
    "H3_LONG_TERM": "≥90 calendar days (thesis hold)",
}


def _iso_now() -> str:
    return datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")


def _safe_load_json(path: Path) -> Optional[Dict[str, Any]]:
    if not path.exists():
        return None
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None


def _is_demo_observation(row: Dict[str, Any]) -> bool:
    """True for framework seeds / pi demos — not fetch-backed live observation."""
    blob = " ".join(
        str(row.get(k) or "")
        for k in (
            "thesis",
            "thesis_tag",
            "source",
            "notes",
            "plan_id",
            "decision",
            "symbol",
            "trade_id",
            "candidate_id",
            "hold_id",
        )
    ).lower()
    return any(m in blob for m in _DEMO_MARKERS)


def _nse_artifact(filename: str) -> Path:
    """Prefer live NSE daemon artifacts (never test_run probes)."""
    primary = NSE_DIR / filename
    if primary.exists():
        return primary
    return NSE_DIR / "test_run" / filename


def _rp608_nse_h0_recs(marks: Dict[str, float]) -> List[Dict[str, Any]]:
    """H0 INTRADAY open books from RP-608 NSE pending active_trades (Kite fetch path)."""
    pending = _safe_load_json(_nse_artifact("rp608_nse_pending_evaluations.json")) or {}
    active = pending.get("active_trades") or {}
    if not isinstance(active, dict):
        return []
    out: List[Dict[str, Any]] = []
    for sym, t in active.items():
        if not isinstance(t, dict):
            continue
        symbol = str(t.get("symbol") or sym)
        entry = t.get("entry_price") or t.get("reference_price")
        mark = marks.get(symbol) if marks else None
        if mark is None and entry is not None:
            try:
                mark = float(entry)
            except (TypeError, ValueError):
                mark = None
        side = str(t.get("side") or "LONG").upper()
        out.append(
            _rec_card(
                market="NSE",
                horizon_id="H0_INTRADAY",
                horizon_class="INTRADAY",
                horizon_term=TERM_NSE["H0_INTRADAY"],
                symbol=symbol,
                side=side,
                decision="RECOMMEND_INTRADAY",
                entry_price=entry,
                stop=t.get("stop_price") or t.get("stop_loss_price") or t.get("invalidation_price"),
                target=t.get("target_price") or t.get("profit_target_price"),
                status=str(t.get("status") or "OPEN"),
                thesis=str(t.get("exit_reason") or t.get("thesis") or "rp608_nse_active_trade"),
                source="RP-608-NSE-FETCH",
                extra={
                    "book_kind": "INTRADAY_ACTIVE",
                    "product_type": "MIS",
                    "entry_bar_idx": t.get("entry_bar_idx"),
                    "mark_price": mark,
                    "tradingsymbol": symbol.replace(".NS", ""),
                    "data_source": "rp608_nse_pending_evaluations.active_trades",
                    "observational_only": True,
                },
            )
        )
    return out


def _rp608_crypto_h0_recs() -> List[Dict[str, Any]]:
    """H0 CRYPTO from trade-plan overlay state (bookTicker / tick path)."""
    state = _safe_load_json(CRYPTO_OVERLAY_STATE) or {}
    plan = state.get("active_plan")
    if not isinstance(plan, dict) or str(plan.get("state") or "") != "OPEN":
        return []
    direction = str(plan.get("direction") or "LONG").upper()
    return [
        _rec_card(
            market="CRYPTO",
            horizon_id="H0_INTRADAY_60M",
            horizon_class="INTRADAY",
            horizon_term=TERM_CRYPTO["H0_INTRADAY_60M"],
            symbol="BTCUSDT",
            side=direction,
            decision="RECOMMEND_INTRADAY",
            entry_price=plan.get("entry_price"),
            stop=plan.get("stop_loss_price"),
            target=plan.get("profit_target_price"),
            status="OPEN",
            thesis=str(plan.get("plan_id") or "rp608_trade_plan_overlay"),
            source="RP-608-CRYPTO-FETCH",
            extra={
                "book_kind": "INTRADAY_ACTIVE",
                "product_type": "SPOT_SIM",
                "plan_id": plan.get("plan_id"),
                "mark_price": plan.get("last_mark_price") or plan.get("reference_price"),
                "atr_14": plan.get("atr_14"),
                "signal_cadence": plan.get("signal_cadence") or plan.get("source_decision_clock"),
                "data_source": "trade_plan_overlay_v1/overlay_state.json",
                "observational_only": True,
            },
        )
    ]


def _is_universe_member(r: Dict[str, Any]) -> bool:
    return (
        r.get("book_kind") == "UNIVERSE_MEMBER"
        or r.get("decision") == "UNIVERSE_MEMBER"
        or str(r.get("status") or "") == "COVERED"
    )


def _enrich_with_fetch_marks(recs: List[Dict[str, Any]], marks: Dict[str, float]) -> List[Dict[str, Any]]:
    """Attach / refresh mark_price from RP-608 bar_history / eval fetches."""
    out: List[Dict[str, Any]] = []
    for r in recs:
        row = dict(r)
        if (row.get("market") or "") == "NSE":
            sym = str(row.get("symbol") or "")
            if sym in marks:
                row["mark_price"] = marks[sym]
                if _is_universe_member(row) and row.get("entry_price") is None:
                    row["entry_price"] = marks[sym]
        out.append(row)
    return out


def _rec_card(
    *,
    market: str,
    horizon_id: str,
    horizon_class: str,
    horizon_term: str,
    symbol: str,
    side: str,
    decision: str,
    entry_price: Any = None,
    stop: Any = None,
    target: Any = None,
    status: str = "OPEN",
    thesis: str = "",
    source: str = "",
    extra: Optional[Dict[str, Any]] = None,
) -> Dict[str, Any]:
    label = {
        "INTRADAY": "Intraday",
        "MULTI_DAY": "Multi-day",
        "MULTI_WEEK": "Multi-week",
        "LONG_TERM": "Long-term",
    }.get(horizon_class, horizon_class)
    card = {
        "market": market,
        "horizon_id": horizon_id,
        "horizon_class": horizon_class,
        "horizon_label": label,
        "horizon_term": horizon_term,
        "symbol": symbol,
        "side": side,
        "decision": decision,
        "entry_price": entry_price,
        "stop_loss_price": stop,
        "target_price": target,
        "status": status,
        "thesis": thesis,
        "source": source,
        "live_order_authority": 0.0,
    }
    if extra:
        card.update(extra)
    return card


def _nse_status() -> Dict[str, Any]:
    try:
        from scripts.rp609_indian_retail_horizons import HORIZONS, JOURNALS, write_horizon_registry
        from scripts.rp609_multi_horizon_engine import MultiHorizonEngine
    except Exception as exc:
        return {
            "available": False,
            "error": str(exc),
            "recommendations": [],
            "catalog": [],
        }

    try:
        write_horizon_registry()
    except Exception:
        pass

    eng = MultiHorizonEngine()
    report = eng.report()
    recommendations: List[Dict[str, Any]] = []

    swing_open = eng.swing.get("open") or {}
    if isinstance(swing_open, dict):
        swing_items = list(swing_open.values())
    else:
        swing_items = list(swing_open)
    marks = _nse_mark_prices()
    for t in swing_items:
        if _is_demo_observation(t if isinstance(t, dict) else {}):
            continue
        sym = str(t.get("symbol") or "")
        entry = t.get("entry_price") or t.get("reference_price")
        # Prefer fetch mark when journal entry looks stale vs bar_history
        mark = marks.get(sym)
        recommendations.append(
            _rec_card(
                market="NSE",
                horizon_id="H1_SWING",
                horizon_class="MULTI_DAY",
                horizon_term=TERM_NSE["H1_SWING"],
                symbol=sym,
                side=str(t.get("side") or "LONG"),
                decision="RECOMMEND_SWING",
                entry_price=entry,
                stop=t.get("invalidation_price") or t.get("stop_loss_price"),
                target=t.get("target_price"),
                status=str(t.get("status") or "OPEN"),
                thesis=str(t.get("thesis_tag") or t.get("thesis") or ""),
                source="RP-609",
                extra={
                    "trade_id": t.get("trade_id"),
                    "product_type": "CNC",
                    "mark_price": mark if mark is not None else t.get("last_mark_price"),
                    "data_source": "rp609_swing_book+rp608_nse_bar_history",
                },
            )
        )

    for plan in (eng.portfolio.get("plans") or {}).values():
        if _is_demo_observation(plan if isinstance(plan, dict) else {}):
            continue
        legs_out: List[Dict[str, Any]] = []
        leg_syms: List[str] = []
        for leg in plan.get("legs") or []:
            s = str(leg.get("symbol") or "").strip()
            if not s:
                continue
            if s not in leg_syms:
                leg_syms.append(s)
            avg = leg.get("avg_price")
            mark = marks.get(s)
            legs_out.append(
                {
                    "symbol": s,
                    "sector": str(leg.get("sector") or "—"),
                    "target_weight_pct": float(leg.get("target_weight_pct") or 0.0),
                    "qty_filled": float(leg.get("qty_filled") or 0.0),
                    "notional_filled": float(leg.get("notional_filled") or 0.0),
                    "avg_price": avg,
                    "mark_price": mark,
                }
            )
        filled = int(plan.get("tranches_filled") or 0)
        n_tr = int(plan.get("n_tranches") or 0)
        recommendations.append(
            _rec_card(
                market="NSE",
                horizon_id="H2_PORTFOLIO_BUILD",
                horizon_class="MULTI_WEEK",
                horizon_term=TERM_NSE["H2_PORTFOLIO_BUILD"],
                symbol=str(plan.get("plan_id") or "PORTFOLIO"),
                side="LONG",
                decision="PORTFOLIO_BUILD",
                entry_price=None,
                status=str(plan.get("status") or "OPEN"),
                thesis=str(plan.get("plan_id") or ""),
                source="RP-609",
                extra={
                    "book_kind": "PORTFOLIO_BUILD",
                    "tranches_filled": filled,
                    "n_tranches": n_tr,
                    "tranche_progress": f"{filled}/{n_tr}" if n_tr else "—",
                    "total_capital_inr": plan.get("total_capital_inr"),
                    "product_type": "CNC",
                    "plan_id": plan.get("plan_id"),
                    "symbols": leg_syms,
                    "legs": legs_out,
                    "leg_count": len(legs_out),
                    "data_source": "rp609_portfolio_book+rp608_nse_bar_history",
                },
            )
        )

    lt_open = eng.long_term.get("open") or eng.long_term.get("holds") or {}
    if isinstance(lt_open, dict):
        lt_items = list(lt_open.values())
    else:
        lt_items = list(lt_open)
    for h in lt_items:
        if _is_demo_observation(h if isinstance(h, dict) else {}):
            continue
        st = str(h.get("status") or "HOLDING")
        if st in {"CLOSED", "THESIS_INVALIDATED"}:
            continue
        sym = str(h.get("symbol") or "")
        recommendations.append(
            _rec_card(
                market="NSE",
                horizon_id="H3_LONG_TERM",
                horizon_class="LONG_TERM",
                horizon_term=TERM_NSE["H3_LONG_TERM"],
                symbol=sym,
                side=str(h.get("side") or "LONG"),
                decision="HOLD_LONG_TERM",
                entry_price=h.get("entry_price"),
                status=st,
                thesis=str(h.get("thesis") or ""),
                source="RP-609",
                extra={
                    "hold_id": h.get("hold_id"),
                    "thesis_version": h.get("thesis_version"),
                    "product_type": "CNC",
                    "mark_price": marks.get(sym) if sym else h.get("last_mark_price"),
                    "data_source": "rp609_long_term_book+rp608_nse_bar_history",
                },
            )
        )

    catalog = []
    for hid, meta in HORIZONS.items():
        catalog.append(
            {
                "horizon_id": hid,
                "horizon_class": CLASS_MAP_NSE.get(hid, "INTRADAY"),
                "horizon_label": {
                    "INTRADAY": "Intraday",
                    "MULTI_DAY": "Multi-day",
                    "MULTI_WEEK": "Multi-week",
                    "LONG_TERM": "Long-term",
                }.get(CLASS_MAP_NSE.get(hid, "INTRADAY"), hid),
                "horizon_term": TERM_NSE.get(hid, meta.get("label")),
                "label": meta.get("label"),
                "product_type": meta.get("product_type"),
                "owner_protocol": meta.get("owner_protocol"),
                "observational_link_only": bool(meta.get("observational_link_only")),
            }
        )

    gates = _safe_load_json(JOURNALS / "RP609_promotion_gates.json") or {}
    return {
        "available": True,
        "protocol_id": "RP-609-INDIAN-RETAIL-MULTI-HORIZON-001",
        "live_order_authority": 0.0,
        "updated_utc": _iso_now(),
        "journals": str(JOURNALS),
        "catalog": catalog,
        "report": report,
        "recommendations": recommendations,
        "promotion_gates": gates.get("gates") or gates,
        "counts_by_class": _count_by_class(recommendations),
    }


def _crypto_status() -> Dict[str, Any]:
    try:
        from scripts.rp610_crypto_multi_horizon import HORIZONS, JOURNALS, write_horizon_registry
        from scripts.rp610_crypto_multi_horizon_engine import open_holds_from_book, summarize_books
    except Exception as exc:
        return {
            "available": False,
            "error": str(exc),
            "recommendations": [],
            "catalog": [],
        }

    try:
        write_horizon_registry()
    except Exception:
        pass

    recommendations: List[Dict[str, Any]] = []
    for hid, cls in (
        ("H1_MULTI_DAY", "MULTI_DAY"),
        ("H2_MULTI_WEEK", "MULTI_WEEK"),
        ("H3_LONG_TERM", "LONG_TERM"),
    ):
        try:
            opens = open_holds_from_book(hid)
        except Exception:
            opens = []
        for h in opens:
            if _is_demo_observation(h if isinstance(h, dict) else {}):
                continue
            recommendations.append(
                _rec_card(
                    market="CRYPTO",
                    horizon_id=hid,
                    horizon_class=cls,
                    horizon_term=TERM_CRYPTO[hid],
                    symbol=str(h.get("symbol") or "BTCUSDT"),
                    side=str(h.get("side") or "LONG"),
                    decision=f"RECOMMEND_{cls}",
                    entry_price=h.get("entry_price"),
                    stop=h.get("stop_pct"),
                    target=h.get("target_pct"),
                    status=str(h.get("status") or "OPEN"),
                    thesis=str(h.get("thesis") or ""),
                    source="RP-610",
                    extra={
                        "candidate_id": h.get("candidate_id"),
                        "product_type": "SPOT_SIM",
                        "hold_days_min": h.get("hold_days_min"),
                        "hold_days_max": h.get("hold_days_max"),
                        "data_source": "rp610_journals",
                    },
                )
            )

    catalog = []
    for hid, meta in HORIZONS.items():
        catalog.append(
            {
                "horizon_id": hid,
                "horizon_class": CLASS_MAP_CRYPTO.get(hid, "INTRADAY"),
                "horizon_label": {
                    "INTRADAY": "Intraday",
                    "MULTI_DAY": "Multi-day",
                    "MULTI_WEEK": "Multi-week",
                    "LONG_TERM": "Long-term",
                }.get(CLASS_MAP_CRYPTO.get(hid, "INTRADAY"), hid),
                "horizon_term": TERM_CRYPTO.get(hid, meta.get("label")),
                "label": meta.get("label"),
                "product_type": meta.get("product_type"),
                "owner_protocol": meta.get("owner_protocol"),
                "observational_link_only": bool(meta.get("observational_link_only")),
            }
        )

    gates = _safe_load_json(JOURNALS / "RP610_promotion_gates.json") or {}
    try:
        summary = summarize_books()
    except Exception as exc:
        summary = {"error": str(exc)}

    return {
        "available": True,
        "protocol_id": "RP-610-CRYPTO-MULTI-HORIZON-001",
        "live_order_authority": 0.0,
        "updated_utc": _iso_now(),
        "journals": str(JOURNALS),
        "catalog": catalog,
        "summary": summary,
        "recommendations": recommendations,
        "promotion_gates": gates.get("gates") or gates,
        "counts_by_class": _count_by_class(recommendations),
    }


def _count_by_class(recs: List[Dict[str, Any]], *, actionable_only: bool = False) -> Dict[str, int]:
    out = {"INTRADAY": 0, "MULTI_DAY": 0, "MULTI_WEEK": 0, "LONG_TERM": 0}
    for r in recs:
        if actionable_only and _is_universe_member(r):
            continue
        cls = str(r.get("horizon_class") or "INTRADAY")
        out[cls] = out.get(cls, 0) + 1
    return out


def _load_nse_119_symbols() -> List[str]:
    """Canonical RP-608 NSE shadow universe (119)."""
    try:
        from scripts.run_rp608_nse_shadow_daemon import NSE_119_SYMBOLS

        return [str(s) for s in NSE_119_SYMBOLS]
    except Exception:
        return []


def _nse_mark_prices() -> Dict[str, float]:
    """Marks from RP-608 NSE fetches (bar_history kite_ws / latest evaluations)."""
    try:
        from scripts.rp609_multi_horizon_engine import load_price_map

        return load_price_map(NSE_DIR)
    except Exception:
        return {}


def _nse_symbols_in_books(recs: List[Dict[str, Any]]) -> Dict[str, List[Dict[str, Any]]]:
    """Map Yahoo-style .NS symbol → open MH book references (observation only)."""
    by_sym: Dict[str, List[Dict[str, Any]]] = {}
    for r in recs:
        if (r.get("market") or "") != "NSE":
            continue
        if _is_universe_member(r):
            # Covered later as universe members — keep actionable map separate
            continue
        refs: List[str] = []
        legs = r.get("legs") if isinstance(r.get("legs"), list) else []
        if legs:
            for leg in legs:
                s = str((leg or {}).get("symbol") or "").strip()
                if s:
                    refs.append(s)
        elif isinstance(r.get("symbols"), list) and r.get("symbols"):
            refs.extend(str(s).strip() for s in r["symbols"] if str(s).strip())
        else:
            s = str(r.get("symbol") or "").strip()
            # Skip plan_id masquerading as symbol on portfolio header cards
            if s and not (
                r.get("book_kind") == "PORTFOLIO_BUILD" or r.get("decision") == "PORTFOLIO_BUILD"
            ):
                refs.append(s)
        for sym in refs:
            by_sym.setdefault(sym, []).append(
                {
                    "horizon_id": r.get("horizon_id"),
                    "horizon_class": r.get("horizon_class"),
                    "decision": r.get("decision"),
                    "status": r.get("status"),
                    "plan_id": r.get("plan_id"),
                    "thesis": r.get("thesis"),
                }
            )
    return by_sym


def build_universe_member_recs(actionable_recs: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
    """
    Emit one Open-books row per NSE-119 scrip that is not already in an actionable H1–H3 book.
    Observation membership only — gates CLOSED, live_order_authority 0.0, no orders.
    """
    universe = _load_nse_119_symbols()
    already = set(_nse_symbols_in_books(actionable_recs).keys())
    marks = _nse_mark_prices()
    out: List[Dict[str, Any]] = []
    for sym in universe:
        if sym in already:
            continue
        px = marks.get(sym)
        out.append(
            _rec_card(
                market="NSE",
                horizon_id="H1_SWING",
                horizon_class="MULTI_DAY",
                horizon_term=TERM_NSE["H1_SWING"],
                symbol=sym,
                side="LONG",
                decision="UNIVERSE_MEMBER",
                entry_price=px,
                stop=None,
                target=None,
                status="COVERED",
                thesis="rp608_nse_119_universe",
                source="RP-608/609",
                extra={
                    "book_kind": "UNIVERSE_MEMBER",
                    "product_type": "CNC",
                    "symbols": [sym],
                    "tradingsymbol": sym.replace(".NS", ""),
                    "mark_price": px,
                    "live_order_authority": 0.0,
                    "observational_only": True,
                    "note": "In MH open books as universe coverage — not an opened swing/portfolio/hold.",
                },
            )
        )
    return out


def build_nse_universe_coverage(recs: List[Dict[str, Any]]) -> Dict[str, Any]:
    """
    119-symbol RP-608 universe vs symbols in actionable RP-609 MH books.
    Universe-member rows also appear in Open books (status COVERED).
    """
    universe = _load_nse_119_symbols()
    actionable = [r for r in recs if not _is_universe_member(r)]
    members = [r for r in recs if _is_universe_member(r)]
    in_books = _nse_symbols_in_books(actionable)
    rows: List[Dict[str, Any]] = []
    for sym in universe:
        books = list(in_books.get(sym) or [])
        if not books:
            # Still shown in Open books as COVERED member
            books = [
                {
                    "horizon_id": "H1_SWING",
                    "horizon_class": "MULTI_DAY",
                    "decision": "UNIVERSE_MEMBER",
                    "status": "COVERED",
                    "thesis": "rp608_nse_119_universe",
                }
            ]
            member_hit = True
        else:
            member_hit = False
        classes = sorted({str(b.get("horizon_class") or "") for b in books if b.get("horizon_class")})
        rows.append(
            {
                "symbol": sym,
                "tradingsymbol": sym.replace(".NS", ""),
                "in_mh_books": bool(in_books.get(sym)),
                "in_open_books_table": True,
                "universe_member_only": member_hit,
                "mh_book_count": len(in_books.get(sym) or []),
                "horizon_classes": classes,
                "books": books,
            }
        )
    in_count = sum(1 for r in rows if r["in_mh_books"])
    return {
        "protocol_id": "RP-608-NSE-SHADOW-001",
        "total_universe": len(universe),
        "in_mh_books": in_count,
        "not_in_mh_books": max(0, len(universe) - in_count),
        "universe_members_in_open_books": len(members),
        "shown_in_open_books": len(universe),
        "open_nse_book_rows": sum(1 for r in recs if (r.get("market") or "") == "NSE"),
        "live_order_authority": 0.0,
        "note": (
            "All 119 NSE scrips appear in Multi-Horizon Open books. "
            f"{in_count} have actionable H1–H3 positions; "
            f"{len(members)} are COVERED universe members (observation only, gates CLOSED)."
        ),
        "symbols": rows,
    }


def get_multi_horizon_dashboard_state() -> Dict[str, Any]:
    marks = _nse_mark_prices()
    nse = _nse_status()
    crypto = _crypto_status()

    # Canonical H0 from RP-608 fetch artifacts (not RP-609/610 demo journals).
    h0_nse = _rp608_nse_h0_recs(marks)
    h0_crypto = _rp608_crypto_h0_recs()

    nse_h1h3 = [r for r in (nse.get("recommendations") or []) if not _is_demo_observation(r)]
    crypto_h1h3 = [r for r in (crypto.get("recommendations") or []) if not _is_demo_observation(r)]

    actionable = h0_nse + nse_h1h3 + h0_crypto + crypto_h1h3
    universe_members = build_universe_member_recs(actionable)
    universe_members = _enrich_with_fetch_marks(universe_members, marks)

    nse_actionable = [r for r in actionable if (r.get("market") or "") == "NSE"]
    crypto_recs = [r for r in actionable if (r.get("market") or "") == "CRYPTO"]
    all_recs = _enrich_with_fetch_marks(nse_actionable + universe_members + crypto_recs, marks)
    nse_universe = build_nse_universe_coverage(all_recs)

    if nse.get("available"):
        nse = dict(nse)
        nse["universe"] = {
            "total_universe": nse_universe["total_universe"],
            "in_mh_books": nse_universe["in_mh_books"],
            "not_in_mh_books": nse_universe["not_in_mh_books"],
            "universe_members_in_open_books": nse_universe["universe_members_in_open_books"],
            "shown_in_open_books": nse_universe["shown_in_open_books"],
        }
        nse["recommendations"] = [r for r in all_recs if (r.get("market") or "") == "NSE"]
        nse["price_marks_from_fetches"] = len(marks)
        nse["h0_from_fetches"] = len(h0_nse)
    if crypto.get("available"):
        crypto = dict(crypto)
        crypto["recommendations"] = crypto_recs
        crypto["h0_from_fetches"] = len(h0_crypto)

    return {
        "live_order_authority": 0.0,
        "updated_utc": _iso_now(),
        "classes": ["INTRADAY", "MULTI_DAY", "MULTI_WEEK", "LONG_TERM"],
        "nse": nse,
        "crypto": crypto,
        "recommendations": all_recs,
        "counts_by_class": _count_by_class(all_recs, actionable_only=True),
        "counts_by_class_including_universe": _count_by_class(all_recs, actionable_only=False),
        "universe_member_count": len(universe_members),
        "nse_universe": nse_universe,
        "fetch_backed": {
            "nse_bar_history_marks": len(marks),
            "nse_h0_active_trades": len(h0_nse),
            "crypto_h0_overlay_open": len(h0_crypto),
            "demos_excluded": True,
        },
        "note": (
            "Multi-Horizon Books are fetch-backed: H0 from RP-608 NSE pending/active_trades "
            "+ crypto overlay; NSE marks from rp608_nse_bar_history (kite_ws). "
            "Demo/seed RP-609/610 rows are excluded. H1–H3 gates remain CLOSED (0% authority). "
            f"NSE Open books: {nse_universe.get('total_universe', 0)} scrips · "
            f"{nse_universe.get('in_mh_books', 0)} actionable · "
            f"{nse_universe.get('universe_members_in_open_books', 0)} COVERED · "
            f"{len(marks)} fetch marks."
        ),
    }
