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
    "H0_INTRADAY": "15m bars · 60m target / 180m max · session square-off",
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
    for t in swing_items:
        recommendations.append(
            _rec_card(
                market="NSE",
                horizon_id="H1_SWING",
                horizon_class="MULTI_DAY",
                horizon_term=TERM_NSE["H1_SWING"],
                symbol=str(t.get("symbol") or ""),
                side=str(t.get("side") or "LONG"),
                decision="RECOMMEND_SWING",
                entry_price=t.get("entry_price") or t.get("reference_price"),
                stop=t.get("invalidation_price") or t.get("stop_loss_price"),
                target=t.get("target_price"),
                status=str(t.get("status") or "OPEN"),
                thesis=str(t.get("thesis_tag") or t.get("thesis") or ""),
                source="RP-609",
                extra={"trade_id": t.get("trade_id"), "product_type": "CNC"},
            )
        )

    for plan in (eng.portfolio.get("plans") or {}).values():
        leg_syms = []
        for leg in plan.get("legs") or []:
            s = str(leg.get("symbol") or "").strip()
            if s and s not in leg_syms:
                leg_syms.append(s)
        recommendations.append(
            _rec_card(
                market="NSE",
                horizon_id="H2_PORTFOLIO_BUILD",
                horizon_class="MULTI_WEEK",
                horizon_term=TERM_NSE["H2_PORTFOLIO_BUILD"],
                symbol=(leg_syms[0] if len(leg_syms) == 1 else str(plan.get("plan_id") or "PORTFOLIO")),
                side="LONG",
                decision="PORTFOLIO_BUILD",
                entry_price=None,
                status=str(plan.get("status") or "OPEN"),
                thesis=str(plan.get("plan_id") or ""),
                source="RP-609",
                extra={
                    "tranches_filled": plan.get("tranches_filled"),
                    "n_tranches": plan.get("n_tranches"),
                    "product_type": "CNC",
                    "plan_id": plan.get("plan_id"),
                    "symbols": leg_syms,
                },
            )
        )

    lt_open = eng.long_term.get("open") or eng.long_term.get("holds") or {}
    if isinstance(lt_open, dict):
        lt_items = list(lt_open.values())
    else:
        lt_items = list(lt_open)
    for h in lt_items:
        st = str(h.get("status") or "HOLDING")
        if st in {"CLOSED", "THESIS_INVALIDATED"}:
            continue
        recommendations.append(
            _rec_card(
                market="NSE",
                horizon_id="H3_LONG_TERM",
                horizon_class="LONG_TERM",
                horizon_term=TERM_NSE["H3_LONG_TERM"],
                symbol=str(h.get("symbol") or ""),
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


def _count_by_class(recs: List[Dict[str, Any]]) -> Dict[str, int]:
    out = {"INTRADAY": 0, "MULTI_DAY": 0, "MULTI_WEEK": 0, "LONG_TERM": 0}
    for r in recs:
        cls = str(r.get("horizon_class") or "INTRADAY")
        out[cls] = out.get(cls, 0) + 1
    return out


def get_multi_horizon_dashboard_state() -> Dict[str, Any]:
    nse = _nse_status()
    crypto = _crypto_status()
    all_recs = list(nse.get("recommendations") or []) + list(crypto.get("recommendations") or [])
    return {
        "live_order_authority": 0.0,
        "updated_utc": _iso_now(),
        "classes": ["INTRADAY", "MULTI_DAY", "MULTI_WEEK", "LONG_TERM"],
        "nse": nse,
        "crypto": crypto,
        "recommendations": all_recs,
        "counts_by_class": _count_by_class(all_recs),
        "note": (
            "H0/INTRADAY live books remain owned by RP-608 daemons. "
            "H1–H3 are RP-609/610 observation-only paper books (gates CLOSED)."
        ),
    }
