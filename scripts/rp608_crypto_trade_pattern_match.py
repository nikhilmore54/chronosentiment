#!/usr/bin/env python3
"""
RP-608 Crypto plan — live trade pattern matcher.

ENTRY: ALLOW / STANDASIDE_ADVERSE_PATTERN / ABSTAIN_INSUFFICIENT
EXIT:  HOLD / EXIT_REVERSE_PATTERN / ABSTAIN_INSUFFICIENT

Env:
  RP608_CRYPTO_PATTERN_GATE=1      → block adverse entries (default 0)
  RP608_CRYPTO_PATTERN_EXIT_GATE=1 → paper early exit on reverse (default 0; Gate 5 frozen)

No symbol bans. Books never pooled. OF ±0.30 / OFX unchanged.
"""

from __future__ import annotations

import json
import os
from pathlib import Path
from typing import Any, Dict, List, Optional, Sequence

from scripts.rp608_crypto_trade_pattern_library import (
    ENTRY_BACKOFF_DROP_ORDER,
    atr_pct_bucket,
    build_entry_pattern_key,
    build_exit_pattern_key,
    default_library_path,
    dist_bps_bucket,
    entry_key_softened,
    hold_bucket,
    imb_bucket,
    mfe_bps_from_plan,
    mfe_bucket,
    rr_bucket,
    utc_session_phase,
)


def entry_gate_enabled() -> bool:
    raw = os.environ.get("RP608_CRYPTO_PATTERN_GATE", "0").strip().lower()
    return raw in {"1", "true", "on", "yes"}


def exit_gate_enabled() -> bool:
    raw = os.environ.get("RP608_CRYPTO_PATTERN_EXIT_GATE", "0").strip().lower()
    return raw in {"1", "true", "on", "yes"}


def load_library(path: Optional[Path] = None) -> Dict[str, Any]:
    p = Path(path) if path else default_library_path()
    if not p.exists():
        return {"entry_patterns": {}, "exit_patterns": {}, "loaded": False, "path": str(p)}
    try:
        doc = json.loads(p.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return {"entry_patterns": {}, "exit_patterns": {}, "loaded": False, "path": str(p)}
    doc["loaded"] = True
    doc["path"] = str(p)
    return doc


def features_from_card_or_plan(
    *,
    card: Optional[Dict[str, Any]] = None,
    plan: Optional[Dict[str, Any]] = None,
    prior_stop_same_utc_day_dir: bool = False,
) -> Dict[str, Any]:
    src = dict(plan or {})
    if card:
        # Prefer live card stamps at decision time
        for k in (
            "direction",
            "policy_b_action",
            "order_flow_supported",
            "order_flow_imbalance",
            "atr_14",
            "reference_price",
            "close_time_ms",
            "stop_loss_price",
            "profit_target_price",
            "gross_rr_ratio",
            "entry_price",
            "entry_time_ms",
        ):
            if card.get(k) is not None and src.get(k) is None:
                src[k] = card[k]
        if card.get("policy_b_action") in ("LONG", "SHORT") and not src.get("direction"):
            src["direction"] = card["policy_b_action"]
    et = int(src.get("entry_time_ms") or src.get("close_time_ms") or 0)
    entry = float(src.get("entry_price") or src.get("reference_price") or 0)
    atr = src.get("atr_14")
    imb = src.get("order_flow_imbalance")
    return {
        "utc_session_phase": utc_session_phase(et) if et else "UNK",
        "direction": str(src.get("direction") or "UNK").upper(),
        "of_supported": bool(src.get("order_flow_supported")),
        "imb_bucket": imb_bucket(float(imb) if imb is not None else None),
        "atr_bucket": atr_pct_bucket(
            float(atr) if atr is not None else None,
            entry if entry > 0 else None,
        ),
        "stop_dist_bucket": dist_bps_bucket(
            entry, float(src["stop_loss_price"]) if src.get("stop_loss_price") is not None else None
        ),
        "target_dist_bucket": dist_bps_bucket(
            entry,
            float(src["profit_target_price"])
            if src.get("profit_target_price") is not None
            else None,
        ),
        "gross_rr_bucket": rr_bucket(
            float(src["gross_rr_ratio"]) if src.get("gross_rr_ratio") is not None else None
        ),
        "prior_stop_same_utc_day_dir": bool(prior_stop_same_utc_day_dir),
    }


def _lookup_entry(
    patterns: Dict[str, Any],
    feats: Dict[str, Any],
    *,
    min_n: int = 10,
) -> tuple[Optional[Dict[str, Any]], str, List[str]]:
    exact = build_entry_pattern_key(feats)
    if exact in patterns:
        return patterns[exact], "EXACT", []
    dropped: List[str] = []
    for ax in ENTRY_BACKOFF_DROP_ORDER:
        dropped.append(ax)
        soft = entry_key_softened(feats, dropped)
        if soft in patterns:
            rec = patterns[soft]
            if int((rec.get("train") or {}).get("n") or 0) >= min_n:
                return rec, "BACKOFF", list(dropped)
    return None, "NONE", dropped


def match_entry(
    *,
    library: Dict[str, Any],
    feats: Dict[str, Any],
    gate_on: Optional[bool] = None,
) -> Dict[str, Any]:
    patterns = library.get("entry_patterns") or {}
    exact = build_entry_pattern_key(feats)
    assert "symbol" not in exact.lower()
    rec, mode, dropped = _lookup_entry(patterns, feats)
    gate = bool(entry_gate_enabled() if gate_on is None else gate_on)
    if rec is None:
        return {
            "kind": "ENTRY",
            "pattern_id": exact,
            "match_mode": mode,
            "dropped_axes": dropped,
            "cohort_n": 0,
            "hist_wr": None,
            "hist_payoff": None,
            "hist_net": None,
            "label": "INSUFFICIENT",
            "prediction": "ABSTAIN_INSUFFICIENT",
            "blocks_entry": False,
            "gate_enabled": gate,
            "symbol_banned": False,
        }
    train = rec.get("train") or {}
    label = str(rec.get("label") or "INSUFFICIENT")
    if label == "ADVERSE":
        prediction = "STANDASIDE_ADVERSE_PATTERN"
    elif label == "INSUFFICIENT":
        prediction = "ABSTAIN_INSUFFICIENT"
    else:
        prediction = "ALLOW"
    blocks = bool(gate and prediction == "STANDASIDE_ADVERSE_PATTERN")
    return {
        "kind": "ENTRY",
        "pattern_id": rec.get("pattern_id") or exact,
        "match_mode": mode,
        "dropped_axes": dropped,
        "cohort_n": int(train.get("n") or 0),
        "hist_wr": train.get("wr"),
        "hist_payoff": train.get("payoff"),
        "hist_net": train.get("net_mean"),
        "label": label,
        "prediction": prediction,
        "blocks_entry": blocks,
        "gate_enabled": gate,
        "standaside_reason": "STANDASIDE_TRADE_PATTERN" if blocks else None,
        "symbol_banned": False,
    }


def match_exit(
    *,
    library: Dict[str, Any],
    plan: Dict[str, Any],
    gate_on: Optional[bool] = None,
) -> Dict[str, Any]:
    et = int(plan.get("entry_time_ms") or 0)
    mfe_bps = mfe_bps_from_plan(plan)
    reason = str(plan.get("exit_reason") or "")
    # Live open plan: infer reverse from path MFE without terminal reason yet
    if not reason and mfe_bps is not None and mfe_bps >= 15:
        # adverse revisit vs entry after green — approximate with mae if present
        entry = float(plan.get("entry_price") or 0)
        mae_px = plan.get("mae_exec_price")
        direction = str(plan.get("direction") or "LONG").upper()
        rev = "NONE"
        if entry > 0 and mae_px is not None:
            mae = float(mae_px)
            adverse = (mae < entry) if direction == "LONG" else (mae > entry)
            if adverse:
                rev = "REV_AFTER_GREEN"
        first_reverse = rev
    elif mfe_bps is not None and mfe_bps >= 15 and "STOP" in reason:
        first_reverse = "REV_AFTER_GREEN"
    elif "PROFIT" in reason or "TARGET" in reason:
        first_reverse = "NONE"
    elif "STOP" in reason:
        first_reverse = "REV_NO_MFE"
    elif "TIME" in reason or "EXPIRED" in reason:
        first_reverse = "TIME"
    else:
        first_reverse = "UNK"

    hold_s = plan.get("holding_duration_seconds")
    if hold_s is None and et and plan.get("last_mark_time_ms"):
        hold_s = (int(plan["last_mark_time_ms"]) - et) / 1000.0
    elif hold_s is None and et:
        import time as _time

        hold_s = (_time.time() * 1000 - et) / 1000.0

    feats = {
        "utc_session_phase": utc_session_phase(et) if et else "UNK",
        "direction": str(plan.get("direction") or "UNK").upper(),
        "mfe_bucket": mfe_bucket(mfe_bps),
        "first_reverse": first_reverse,
        "hold_bucket": hold_bucket(float(hold_s) if hold_s is not None else None),
    }
    pid = build_exit_pattern_key(feats)
    patterns = library.get("exit_patterns") or {}
    rec = patterns.get(pid)
    gate = bool(exit_gate_enabled() if gate_on is None else gate_on)
    if rec is None:
        return {
            "kind": "EXIT",
            "pattern_id": pid,
            "match_mode": "NONE",
            "cohort_n": 0,
            "hist_wr": None,
            "hist_payoff": None,
            "hist_net": None,
            "label": "INSUFFICIENT",
            "suggestion": "ABSTAIN_INSUFFICIENT",
            "apply_early_exit": False,
            "gate_enabled": gate,
            "symbol_banned": False,
        }
    train = rec.get("train") or {}
    label = str(rec.get("label") or "INSUFFICIENT")
    if label == "REVERSE_STANDASIDE_HOLD":
        suggestion = "EXIT_REVERSE_PATTERN"
    elif label == "HOLD_FAVORABLE":
        suggestion = "HOLD"
    elif label == "INSUFFICIENT":
        suggestion = "ABSTAIN_INSUFFICIENT"
    else:
        suggestion = "HOLD"
    apply = bool(
        gate and suggestion == "EXIT_REVERSE_PATTERN" and first_reverse == "REV_AFTER_GREEN"
    )
    return {
        "kind": "EXIT",
        "pattern_id": pid,
        "match_mode": "EXACT",
        "cohort_n": int(train.get("n") or 0),
        "hist_wr": train.get("wr"),
        "hist_payoff": train.get("payoff"),
        "hist_net": train.get("net_mean"),
        "label": label,
        "suggestion": suggestion,
        "apply_early_exit": apply,
        "gate_enabled": gate,
        "exit_reason_if_applied": "PATTERN_REVERSE" if apply else None,
        "symbol_banned": False,
    }
