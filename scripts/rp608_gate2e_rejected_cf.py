#!/usr/bin/env python3
"""
RP-608 Gate 2E — rejected-trade counterfactual ledger.

Protocol: RP-608-GATE2E-REJECTED-CF-001
Authority: 0% · opens_trade=false · books never pooled

PRIMARY standasides (PATTERN / DI_SKEW / REGIME_DIRECTIONAL / PATH / COOLDOWN)
are stamped to `gate2e_rejected_cf.jsonl`. CF PnL is the ungated BASELINE twin
outcome (or open mark) matched same direction within join tolerance — not a
second PRIMARY book and not pooled into PRIMARY W/L.

Env RP608_GATE2E_REJECTED_CF — default ON (stamp-only).
"""
from __future__ import annotations

import os
from typing import Any, Dict, Optional

PROTOCOL_ID = "RP-608-GATE2E-REJECTED-CF-001"
REJECT_REASONS = frozenset(
    {"PATTERN", "DI_SKEW", "REGIME_DIRECTIONAL", "PATH", "COOLDOWN", "OPEN_PLAN"}
)


def _env_bool(name: str, default: bool) -> bool:
    raw = os.environ.get(name)
    if raw is None or str(raw).strip() == "":
        return default
    return str(raw).strip().lower() not in {"0", "false", "no", "off"}


def gate2e_rejected_cf_enabled() -> bool:
    return _env_bool("RP608_GATE2E_REJECTED_CF", True)


def rejected_cf_applies_to_book(book_id: Optional[str]) -> bool:
    """PRIMARY only — BASELINE / r50 twins are the CF path, not stamped."""
    if not gate2e_rejected_cf_enabled():
        return False
    bid = str(book_id or "").upper()
    if any(
        tok in bid
        for tok in (
            "BASELINE",
            "GATE2X_R50",
            "G2X_R50",
            "FLOOR_LOCK",
            "GATE2X_FLOOR",
            "TREND_AGREE",
        )
    ):
        return False
    return True


def twin_book_for(book_id: Optional[str]) -> str:
    bid = str(book_id or "")
    mapping = {
        "CRYPTO_PLAN": "CRYPTO_PLAN_BASELINE_PARALLEL",
        "CRYPTO_PLAN_ETHUSDT": "CRYPTO_PLAN_BASELINE_PARALLEL_ETHUSDT",
        "NSE_ESTIMAND_A": "NSE_ESTIMAND_A_BASELINE",
    }
    if bid in mapping:
        return mapping[bid]
    if "BASELINE" in bid.upper():
        return bid
    return f"{bid}_BASELINE_PARALLEL" if bid else "UNKNOWN_BASELINE"


def build_rejected_cf_stamp(
    *,
    reject_reason: str,
    book_id: str,
    suppressed_ms: int,
    direction: Any = None,
    plan: Optional[Dict[str, Any]] = None,
    detail: Optional[str] = None,
    pattern_stamp: Optional[Dict[str, Any]] = None,
    regime_align: Optional[Dict[str, Any]] = None,
    extra: Optional[Dict[str, Any]] = None,
) -> Dict[str, Any]:
    reason = str(reject_reason or "").upper()
    if reason not in REJECT_REASONS:
        reason = "PATTERN"
    plan = plan or {}
    pat = pattern_stamp or {}
    ra = regime_align or {}
    stamp: Dict[str, Any] = {
        "protocol_id": PROTOCOL_ID,
        "reject_reason": reason,
        "book_id": book_id,
        "twin_book": twin_book_for(book_id),
        "symbol": plan.get("symbol"),
        "direction": direction or plan.get("direction"),
        "plan_id": plan.get("plan_id"),
        "suppressed_ms": int(suppressed_ms),
        "entry_price": plan.get("entry_price"),
        "stop_loss_price": plan.get("stop_loss_price"),
        "profit_target_price": plan.get("profit_target_price"),
        "atr_14": plan.get("atr_14"),
        "bar_idx": plan.get("bar_idx") or plan.get("entry_bar_idx"),
        "detail": detail,
        "authority_pct": 0.0,
        "opens_trade": False,
        "cf_source": "BASELINE_TWIN_JOIN",
        # pattern axes (optional)
        "pattern_id": pat.get("pattern_id"),
        "prediction": pat.get("prediction"),
        "cohort_n": pat.get("cohort_n"),
        "hist_wr": pat.get("hist_wr"),
        # regime / DI axes (optional)
        "regime_state": ra.get("regime_state"),
        "trend_dir": ra.get("trend_dir"),
        "guard": ra.get("guard"),
        "block_reason": ra.get("block_reason") or detail,
        "di_skew": ra.get("di_skew"),
        "plus_di": ra.get("plus_di"),
        "minus_di": ra.get("minus_di"),
    }
    if extra:
        for k, v in extra.items():
            if k not in stamp or stamp[k] is None:
                stamp[k] = v
    return stamp
