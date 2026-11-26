#!/usr/bin/env python3
"""
RP-608 Crypto plan — live trade pattern matcher.

ENTRY: ALLOW / STANDASIDE_ADVERSE_PATTERN / ABSTAIN_INSUFFICIENT /
       ABSTAIN_NEGATIVE_EXPECTANCY
EXIT:  HOLD / EXIT_REVERSE_PATTERN / ABSTAIN_INSUFFICIENT

Gate2E selection (2026-11-23): NEUTRAL/ALLOW only when train net_mean > 0.
Negative or missing hist net → ABSTAIN_NEGATIVE_EXPECTANCY (fail-closed under
allow_only). Does not change OF ±0.30 / trail / stops (Gate 5).

Env RP608_CRYPTO_PATTERN_GATE:
  0 / off              → stamp only
  1 / adverse / true   → block STANDASIDE_ADVERSE only
  2 / allow_only       → block unless ALLOW (no trade > bad trade)

RP608_CRYPTO_PATTERN_EXIT_GATE=1 → paper early exit on reverse (default 0; Gate 5 frozen)

RP608_CRYPTO_GATE2E_SHADOW_THIN_WR (default 1 / ON):
  Stamp-only SHADOW_ALLOW_IF_N10_WR_GT_035 when allow_only suppresses ABSTAIN
  on thin cells with n≥10 and hist_wr>0.35. Never opens trades; never loosens
  live PRIMARY gate. Join later to BASELINE outcomes (books never pooled).

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
    default_baseline_library_path,
    default_library_path,
    dist_bps_bucket,
    hold_bucket,
    imb_bucket,
    mfe_bps_from_plan,
    mfe_bucket,
    rr_bucket,
    utc_session_phase,
    vol_surge_bucket,
)

# Always required for parent-cell match (never in ENTRY_BACKOFF_DROP_ORDER).
_CORE_ENTRY_AXES = ("utc_session_phase", "direction")


def normalize_entry_gate_mode(raw: Optional[str]) -> str:
    """Return off | adverse | allow_only."""
    v = (raw or "0").strip().lower()
    if v in {"0", "off", "false", "no", ""}:
        return "off"
    if v in {"2", "allow_only", "allow-only", "strict"}:
        return "allow_only"
    if v in {"1", "true", "on", "yes", "adverse"}:
        return "adverse"
    return "off"


def entry_gate_mode(override: Optional[str] = None) -> str:
    if override is not None:
        return normalize_entry_gate_mode(override)
    return normalize_entry_gate_mode(os.environ.get("RP608_CRYPTO_PATTERN_GATE", "0"))


def entry_gate_enabled(override: Optional[str] = None) -> bool:
    """True when any blocking mode is active (adverse or allow_only)."""
    return entry_gate_mode(override) != "off"


def entry_blocks_prediction(prediction: str, mode: str) -> bool:
    m = normalize_entry_gate_mode(mode)
    if m == "off":
        return False
    if m == "adverse":
        return prediction == "STANDASIDE_ADVERSE_PATTERN"
    if m == "allow_only":
        return prediction != "ALLOW"
    return False


def entry_prediction_from_label(label: str, train: Optional[Dict[str, Any]] = None) -> str:
    """Map library label (+ train net) → live prediction.

    NEUTRAL is not sufficient for ALLOW: mined net_mean must be strictly > 0.
    (RP-608 Gate2E 2026-11-23 — stop opening negative-expectancy BACKOFF parents.)
    """
    lab = str(label or "INSUFFICIENT")
    if lab == "ADVERSE":
        return "STANDASIDE_ADVERSE_PATTERN"
    if lab == "INSUFFICIENT":
        return "ABSTAIN_INSUFFICIENT"
    tr = train or {}
    raw = tr.get("net_mean")
    try:
        net = float(raw) if raw is not None else None
    except (TypeError, ValueError):
        net = None
    if net is None or net <= 0.0:
        return "ABSTAIN_NEGATIVE_EXPECTANCY"
    return "ALLOW"


def exit_gate_enabled() -> bool:
    raw = os.environ.get("RP608_CRYPTO_PATTERN_EXIT_GATE", "0").strip().lower()
    return raw in {"1", "true", "on", "yes"}


# Gate 2E observational shadow (stamp-only; authority 0%; never opens trades).
# Counterfactual: thin cells with n≥10 and hist_wr>0.35 that allow_only still
# suppresses as ABSTAIN (library adverse floor remains n≥30). Join later to
# BASELINE outcomes — do not loosen live RP608_CRYPTO_PATTERN_GATE=allow_only.
SHADOW_ALLOW_IF_N10_WR_GT_035 = "SHADOW_ALLOW_IF_N10_WR_GT_035"
SHADOW_THIN_WR_MIN_N = 10
SHADOW_THIN_WR_MIN_WR = 0.35
# Default ON: stamp-only is safe / observational. Set 0 to disable.
_SHADOW_THIN_WR_ENV = "RP608_CRYPTO_GATE2E_SHADOW_THIN_WR"


def gate2e_shadow_thin_wr_enabled(override: Optional[str] = None) -> bool:
    """Env RP608_CRYPTO_GATE2E_SHADOW_THIN_WR — default ON (stamp-only)."""
    raw = (
        override
        if override is not None
        else os.environ.get(_SHADOW_THIN_WR_ENV, "1")
    )
    v = (raw or "1").strip().lower()
    return v not in {"0", "off", "false", "no"}


def matches_shadow_allow_if_n10_wr_gt_035(pattern_stamp: Dict[str, Any]) -> bool:
    """True when suppress is ABSTAIN/thin but cohort would pass n≥10 & wr>0.35.

    Live allow_only still blocks; this is observational only.
    """
    pred = str(pattern_stamp.get("prediction") or "")
    label = str(pattern_stamp.get("label") or "")
    # Thin-high-WR under allow_only → ABSTAIN_INSUFFICIENT (n < adverse floor 30).
    if pred not in {"ABSTAIN_INSUFFICIENT"} and label not in {"INSUFFICIENT"}:
        return False
    if pred == "ALLOW":
        return False
    n = int(pattern_stamp.get("cohort_n") or 0)
    wr_raw = pattern_stamp.get("hist_wr")
    if wr_raw is None:
        return False
    try:
        wr = float(wr_raw)
    except (TypeError, ValueError):
        return False
    return n >= SHADOW_THIN_WR_MIN_N and wr > SHADOW_THIN_WR_MIN_WR


def build_gate2e_shadow_allow_stamp(
    *,
    pattern_stamp: Dict[str, Any],
    plan_id: Any,
    direction: Any,
    suppressed_ms: int,
    book_id: str,
    plan: Optional[Dict[str, Any]] = None,
) -> Dict[str, Any]:
    """Append-only jsonl schema for SHADOW_ALLOW_IF_N10_WR_GT_035.

    Includes live Gate 2E candle volume (≠ OFX). ``pattern_id`` may show a
    BACKOFF mined ``VUNK`` cell — ``live_vol_*`` is the decision-time axis.
    """
    src = plan or {}
    live_surge = pattern_stamp.get("live_vol_surge")
    live_burst = pattern_stamp.get("live_vol_burst_20")
    if live_surge not in {"LOW", "MED", "HIGH", "UNK"}:
        live_surge = _resolve_vol_surge(plan=src, card=None)
    if live_burst is None:
        raw_b = src.get("vol_burst_20")
        try:
            live_burst = float(raw_b) if raw_b is not None else None
        except (TypeError, ValueError):
            live_burst = None
    return {
        "shadow_rule_id": SHADOW_ALLOW_IF_N10_WR_GT_035,
        "pattern_id": pattern_stamp.get("pattern_id"),
        "feature_pattern_id": pattern_stamp.get("feature_pattern_id"),
        "prediction": pattern_stamp.get("prediction"),
        "hist_wr": pattern_stamp.get("hist_wr"),
        "cohort_n": pattern_stamp.get("cohort_n"),
        "plan_id": plan_id,
        "direction": direction,
        "suppressed_ms": int(suppressed_ms),
        "book_id": book_id,
        "match_mode": pattern_stamp.get("match_mode"),
        "gate_mode": pattern_stamp.get("gate_mode"),
        "label": pattern_stamp.get("label"),
        "entry_price": src.get("entry_price"),
        "vol_burst_20": live_burst,
        "vol_surge": live_surge,
        "vol_axis": f"V{live_surge}",
        "live_vol_burst_20": live_burst,
        "live_vol_surge": live_surge,
        "authority_pct": 0.0,
        "opens_trade": False,
    }

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


def _resolve_vol_surge(
    *,
    card: Optional[Dict[str, Any]] = None,
    plan: Optional[Dict[str, Any]] = None,
    prefer_exit: bool = False,
) -> str:
    """Candle OHLCV surge bucket (not OFX aggression). Prefer latched plan stamps.

    Concrete LOW/MED/HIGH and numeric burst win over placeholder ``UNK`` so a
    plan latch of UNK cannot mask card ``vol_burst_20`` (Gate 2E ≠ OFX).
    """
    src_plan = plan or {}
    src_card = card or {}
    if prefer_exit:
        candidates = (
            src_plan.get("exit_vol_surge"),
            src_plan.get("exit_vol_burst_20"),
            src_card.get("exit_vol_burst_20"),
            src_card.get("vol_burst_20"),
            src_plan.get("vol_surge"),
            src_plan.get("vol_burst_20"),
            src_card.get("vol_burst_20"),
        )
    else:
        candidates = (
            src_plan.get("vol_surge"),
            src_plan.get("vol_burst_20"),
            src_card.get("vol_surge"),
            src_card.get("vol_burst_20"),
        )
    for raw in candidates:
        if raw in {"LOW", "MED", "HIGH"}:
            return str(raw)
        if raw is None or raw == "UNK":
            continue
        try:
            return vol_surge_bucket(float(raw))
        except (TypeError, ValueError):
            continue
    return "UNK"


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
            "vol_burst_20",
            "vol_surge",
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
        "vol_surge": _resolve_vol_surge(card=card, plan=plan, prefer_exit=False),
        "vol_burst_20": _optional_float(
            src.get("vol_burst_20")
            if src.get("vol_burst_20") is not None
            else (card or {}).get("vol_burst_20")
        ),
    }


def _optional_float(raw: Any) -> Optional[float]:
    if raw is None:
        return None
    try:
        return float(raw)
    except (TypeError, ValueError):
        return None


def parse_entry_pattern_key(key: str) -> Optional[Dict[str, Any]]:
    """Parse mined entry key → axis dict for parent-cell compare."""
    parts = str(key).split("|")
    if len(parts) < 9:
        return None
    of_tok = parts[2]
    if not of_tok.startswith("OF"):
        return None
    prior_tok = parts[8]
    if not prior_tok.startswith("P"):
        return None
    out: Dict[str, Any] = {
        "utc_session_phase": parts[0],
        "direction": parts[1],
        "of_supported": of_tok[2:] == "T",
        "imb_bucket": parts[3],
        "atr_bucket": parts[4],
        "stop_dist_bucket": parts[5][1:] if parts[5].startswith("S") else parts[5],
        "target_dist_bucket": parts[6][1:] if parts[6].startswith("T") else parts[6],
        "gross_rr_bucket": parts[7][1:] if parts[7].startswith("R") else parts[7],
        "prior_stop_same_utc_day_dir": prior_tok[1:] == "T",
    }
    if len(parts) >= 10 and parts[9].startswith("V"):
        out["vol_surge"] = parts[9][1:]
    else:
        out["vol_surge"] = "UNK"
    return out


def _feat_axis(feats: Dict[str, Any], ax: str) -> Any:
    if ax == "of_supported":
        return bool(feats.get("of_supported"))
    if ax == "prior_stop_same_utc_day_dir":
        return bool(feats.get("prior_stop_same_utc_day_dir"))
    if ax == "utc_session_phase":
        return str(feats.get("utc_session_phase") or "UNK")
    if ax == "direction":
        return str(feats.get("direction") or "UNK").upper()
    if ax == "vol_surge":
        return str(feats.get("vol_surge") or "UNK")
    return feats.get(ax)


def _parent_cell_match(
    patterns: Dict[str, Any],
    feats: Dict[str, Any],
    dropped: Sequence[str],
    *,
    min_n: int,
    include_volume: bool,
) -> Optional[Dict[str, Any]]:
    """Best library cell matching live feats on all non-dropped axes (+ core)."""
    required = list(_CORE_ENTRY_AXES)
    drop_set = set(dropped)
    for ax in (
        "of_supported",
        "imb_bucket",
        "atr_bucket",
        "stop_dist_bucket",
        "target_dist_bucket",
        "gross_rr_bucket",
        "prior_stop_same_utc_day_dir",
        "vol_surge",
    ):
        if ax in drop_set:
            continue
        if ax == "vol_surge" and not include_volume:
            continue
        required.append(ax)

    best: Optional[Dict[str, Any]] = None
    best_n = -1
    for key, rec in patterns.items():
        parsed = parse_entry_pattern_key(key)
        if not parsed:
            continue
        ok = True
        for ax in required:
            live = _feat_axis(feats, ax)
            cell = parsed.get(ax)
            if ax in {"of_supported", "prior_stop_same_utc_day_dir"}:
                if bool(live) != bool(cell):
                    ok = False
                    break
            elif str(live) != str(cell):
                ok = False
                break
        if not ok:
            continue
        n = int((rec.get("train") or {}).get("n") or 0)
        if n < min_n:
            continue
        if n > best_n:
            best_n = n
            best = rec
    return best


def _lookup_entry(
    patterns: Dict[str, Any],
    feats: Dict[str, Any],
    *,
    min_n: int = 10,
    include_volume: bool = True,
) -> tuple[Optional[Dict[str, Any]], str, List[str]]:
    exact = build_entry_pattern_key(feats, include_volume=include_volume)
    if exact in patterns:
        return patterns[exact], "EXACT", []
    dropped: List[str] = []
    drop_order = (
        ENTRY_BACKOFF_DROP_ORDER
        if include_volume
        else tuple(ax for ax in ENTRY_BACKOFF_DROP_ORDER if ax != "vol_surge")
    )
    for ax in drop_order:
        dropped.append(ax)
        rec = _parent_cell_match(
            patterns,
            feats,
            dropped,
            min_n=min_n,
            include_volume=include_volume,
        )
        if rec is not None:
            return rec, "BACKOFF", list(dropped)
    return None, "NONE", dropped


def match_entry(
    *,
    library: Dict[str, Any],
    feats: Dict[str, Any],
    gate_on: Optional[bool] = None,
    gate_mode: Optional[str] = None,
    include_volume: bool = True,
) -> Dict[str, Any]:
    patterns = library.get("entry_patterns") or {}
    exact = build_entry_pattern_key(feats, include_volume=include_volume)
    assert "symbol" not in exact.lower()
    rec, mode, dropped = _lookup_entry(
        patterns, feats, include_volume=include_volume
    )
    if gate_mode is not None:
        gmode = normalize_entry_gate_mode(gate_mode)
    elif gate_on is True:
        gmode = "adverse"
    elif gate_on is False:
        gmode = "off"
    else:
        gmode = entry_gate_mode()
    gate = gmode != "off"
    live_vol = str(feats.get("vol_surge") or "UNK")
    live_burst = feats.get("vol_burst_20")
    if rec is None:
        prediction = "ABSTAIN_INSUFFICIENT"
        blocks = entry_blocks_prediction(prediction, gmode)
        return {
            "kind": "ENTRY",
            "pattern_id": exact,
            "feature_pattern_id": exact,
            "match_mode": mode,
            "dropped_axes": dropped,
            "cohort_n": 0,
            "hist_wr": None,
            "hist_payoff": None,
            "hist_net": None,
            "label": "INSUFFICIENT",
            "prediction": prediction,
            "blocks_entry": blocks,
            "gate_enabled": gate,
            "gate_mode": gmode,
            "standaside_reason": "STANDASIDE_TRADE_PATTERN" if blocks else None,
            "symbol_banned": False,
            "live_vol_surge": live_vol,
            "live_vol_burst_20": live_burst,
        }
    train = rec.get("train") or {}
    label = str(rec.get("label") or "INSUFFICIENT")
    prediction = entry_prediction_from_label(label, train)
    blocks = entry_blocks_prediction(prediction, gmode)
    return {
        "kind": "ENTRY",
        "pattern_id": rec.get("pattern_id") or exact,
        "feature_pattern_id": exact,
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
        "gate_mode": gmode,
        "standaside_reason": "STANDASIDE_TRADE_PATTERN" if blocks else None,
        "symbol_banned": False,
        # Live candle volume at decision (may differ from BACKOFF mined V* axis)
        "live_vol_surge": live_vol,
        "live_vol_burst_20": live_burst,
    }


def match_exit(
    *,
    library: Dict[str, Any],
    plan: Dict[str, Any],
    gate_on: Optional[bool] = None,
    include_volume: bool = True,
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
        "exit_vol_surge": _resolve_vol_surge(plan=plan, prefer_exit=True),
    }
    pid = build_exit_pattern_key(feats, include_volume=include_volume)
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
