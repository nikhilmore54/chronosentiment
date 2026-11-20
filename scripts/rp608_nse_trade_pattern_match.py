#!/usr/bin/env python3
"""
RP-608 NSE A — live trade pattern matcher.

ENTRY (Gate 2E observational):
  Match decision-time Coralys + pre-entry candle features to mined cohorts.
  prediction ∈ {ALLOW, STANDASIDE_ADVERSE_PATTERN, ABSTAIN_INSUFFICIENT}

EXIT (Gate 2X observational):
  Match post-entry candle path to reverse cohorts.
  suggestion ∈ {HOLD, EXIT_REVERSE_PATTERN, ABSTAIN_INSUFFICIENT}
  Does NOT change frozen stops/targets unless RP608_NSE_PATTERN_EXIT_GATE=1
  (paper only; Gate 5 still required for live baseline).

No symbol / tradingsymbol in keys. Books never pooled.
"""

from __future__ import annotations

import json
import os
from datetime import datetime
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

from scripts.rp608_nse_trade_pattern_library import (
    ENTRY_BACKOFF_DROP_ORDER,
    build_entry_pattern_key,
    build_exit_pattern_key,
    default_library_path,
    entry_key_with_wildcards,
    extract_exit_path_feats,
    extract_pre_entry_candle_feats,
    gap_bucket,
    bar_tertile,
    traj15_bucket,
)


def entry_gate_enabled() -> bool:
    raw = os.environ.get("RP608_NSE_PATTERN_GATE", "0").strip().lower()
    return raw in {"1", "true", "on", "yes"}


def exit_gate_enabled() -> bool:
    """Optional paper early-exit on reverse pattern (default off; Gate 5 frozen)."""
    raw = os.environ.get("RP608_NSE_PATTERN_EXIT_GATE", "0").strip().lower()
    return raw in {"1", "true", "on", "yes"}


def load_library(path: Optional[Path] = None) -> Dict[str, Any]:
    p = Path(path) if path else default_library_path()
    if not p.exists():
        return {
            "entry_patterns": {},
            "exit_patterns": {},
            "schema": None,
            "path": str(p),
            "loaded": False,
        }
    try:
        doc = json.loads(p.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return {
            "entry_patterns": {},
            "exit_patterns": {},
            "schema": None,
            "path": str(p),
            "loaded": False,
        }
    doc["path"] = str(p)
    doc["loaded"] = True
    return doc


def features_from_coralys_and_candles(
    *,
    input_ctx: Optional[Dict[str, Any]],
    traj15: float,
    bar_idx: int,
    decision_ts_ist: Optional[datetime] = None,
    same_session_prior_stop: bool = False,
    bars_for_symbol_session: Optional[Dict[int, Dict[str, float]]] = None,
) -> Dict[str, Any]:
    ctx = dict(input_ctx or {})
    phase = ctx.get("session_phase")
    if not phase and decision_ts_ist is not None:
        from scripts.rp608_nse_coralys_input_context import session_phase_ist

        phase = session_phase_ist(decision_ts_ist)
    gap = ctx.get("opening_gap_bps")
    candle = {
        "candle_path3": "UNK",
        "signal_body": "UNK",
        "runup_3b": "UNK",
        "vol_surge": "UNK",
    }
    if bars_for_symbol_session and bar_idx:
        candle = extract_pre_entry_candle_feats(bars_for_symbol_session, int(bar_idx))
    return {
        "session_phase": phase or "UNK",
        "traj15_bucket": traj15_bucket(float(traj15)),
        "preopen_status": ctx.get("preopen_status") or "MISSING",
        "gap_bucket": gap_bucket(float(gap) if gap is not None else None),
        "jittery": bool(ctx.get("preopen_jittery")),
        "turbulent": bool(ctx.get("preopen_turbulent")),
        "trend_aligned_long": ctx.get("trend_aligned_long"),
        "prior_support": ctx.get("prior_support"),
        "same_session_prior_stop_in_similar_context": bool(same_session_prior_stop),
        "bar_tertile": bar_tertile(bar_idx),
        "candle_path3": candle.get("candle_path3") or "UNK",
        "signal_body": candle.get("signal_body") or "UNK",
        "runup_3b": candle.get("runup_3b") or "UNK",
        "vol_surge": candle.get("vol_surge") or "UNK",
    }


def _pattern_lookup(
    patterns: Dict[str, Any],
    exact_key: str,
    feats: Dict[str, Any],
    *,
    min_n: int = 10,
) -> Tuple[Optional[Dict[str, Any]], str, List[str]]:
    """Exact then backoff. Returns (pattern_rec, match_mode, dropped_axes)."""
    if exact_key in patterns:
        return patterns[exact_key], "EXACT", []
    dropped: List[str] = []
    for axis in ENTRY_BACKOFF_DROP_ORDER:
        dropped.append(axis)
        key = entry_key_with_wildcards(feats, dropped)
        # Wildcard keys are not stored; scan for compatible cohorts and aggregate
        # Prefer stored exact keys that match non-wildcard parts — simpler: try
        # rebuilding exact key with dropped axes forced to common train values
        # via progressive exact rebuild used at mine time without wildcards.
        # Practical approach: drop axis by setting to UNK / N / False as in mine
        # and look up that exact key.
        softened = dict(feats)
        for ax in dropped:
            if ax in {"vol_surge", "runup_3b", "signal_body", "candle_path3", "bar_tertile"}:
                softened[ax] = "UNK"
            elif ax in {"prior_support", "trend_aligned_long"}:
                softened[ax] = None
            elif ax == "same_session_prior_stop_in_similar_context":
                softened[ax] = False
            elif ax in {"jittery", "turbulent"}:
                softened[ax] = False
        soft_key = build_entry_pattern_key(softened)
        if soft_key in patterns:
            rec = patterns[soft_key]
            n = int((rec.get("train") or {}).get("n") or 0)
            if n >= min_n:
                return rec, "BACKOFF", list(dropped)
        # Also try literal wildcard key if miner stored rolled-up (future)
        if key in patterns:
            rec = patterns[key]
            n = int((rec.get("train") or {}).get("n") or 0)
            if n >= min_n:
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
    rec, mode, dropped = _pattern_lookup(patterns, exact, feats, min_n=10)
    if rec is None:
        stamp = {
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
            "gate_enabled": bool(entry_gate_enabled() if gate_on is None else gate_on),
            "symbol_banned": False,
        }
        return stamp

    train = rec.get("train") or {}
    label = str(rec.get("label") or "INSUFFICIENT")
    if label == "ADVERSE":
        prediction = "STANDASIDE_ADVERSE_PATTERN"
    elif label == "INSUFFICIENT":
        prediction = "ABSTAIN_INSUFFICIENT"
    else:
        prediction = "ALLOW"
    gate = bool(entry_gate_enabled() if gate_on is None else gate_on)
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
    session_phase: str,
    traj15: float,
    bars_dict: Dict[int, Dict[str, float]],
    entry_bar_idx: int,
    current_bar_idx: int,
    entry_price: float,
    gate_on: Optional[bool] = None,
) -> Dict[str, Any]:
    path = extract_exit_path_feats(
        bars_dict,
        int(entry_bar_idx),
        int(current_bar_idx),
        float(entry_price),
    )
    feats = {
        "session_phase": session_phase or "UNK",
        "traj15_bucket": traj15_bucket(float(traj15)),
        "exit_path3": path.get("exit_path3") or "UNK",
        "mfe_bucket": path.get("mfe_bucket") or "UNK",
        "first_reverse": path.get("first_reverse") or "UNK",
        "exit_vol_surge": path.get("exit_vol_surge") or "UNK",
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
            "path": path,
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
        gate
        and suggestion == "EXIT_REVERSE_PATTERN"
        and str(path.get("first_reverse") or "NONE") != "NONE"
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
        "path": path,
        "symbol_banned": False,
    }
