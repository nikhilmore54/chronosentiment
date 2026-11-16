"""
RP-608-OFX — volume / aggressor-direction mark (research-only).

Uses signed executed-volume imbalance:
  volume_imbalance = signed_delta_qty / total_classified_qty

NOT baseline L1 OF (±0.30 resting book). Do not import into
classify_order_flow_support or OF paper activation without Gate authorization.
"""

from __future__ import annotations

from typing import Any, Dict, List, Optional, Sequence, Tuple

from .aggregate import ExecutedFlowWindowResult

# OFX research threshold (volume aggressor imbalance) — independent of baseline L1 ±0.30
OFX_VOLUME_IMBALANCE_THRESHOLD = 0.30
OFX_VOLUME_LABEL = "OFX_VOLUME_FLOW_SUPPORTED"
OFX_VOLUME_EXIT_LABEL = "OFX_VOLUME_FLOW_EXIT"
# Opposing-volume match exit: buy-entry closed by ~equal sell volume (and vice versa).
# Relative |opp − entry| / entry must fall within [MATCH_DEV_MIN, MATCH_DEV_MAX].
# Band −10%..+10% ⇒ |dev| ≤ 0.10 (exact match through ±10% inclusive).
OFX_VOLUME_MATCH_DEV_MIN = 0.00
OFX_VOLUME_MATCH_DEV_MAX = 0.10
OFX_VOLUME_MATCH_EXIT_LABEL = "OFX_VOLUME_MATCH_EXIT"
MIN_CLASSIFIED_QTY = 1e-12
# OFX paper entry surge (crypto + NSE): entry tick qty ≥ 3× mean of prior 20 ticks.
# Raw completed-tick sizes only — no L1 OF / imbalance / other order-flow signals.
OFX_ENTRY_SURGE_MULTIPLE = 3.0
OFX_ENTRY_SURGE_LOOKBACK_TICKS = 20
OFX_ENTRY_SURGE_LABEL = "OFX_ENTRY_VOLUME_SURGE"
# Aggressor tempo (crypto DIRECT / NSE DERIVED): staggered vs clubbed same-side flow.
# Top-1 share of same-side qty in the lookback window — research-only, not L1 OF.
OFX_TEMPO_LOOKBACK_TICKS = 20
OFX_TEMPO_CLUBBED_TOP_SHARE = 0.45
OFX_TEMPO_STAGGERED_TOP_SHARE = 0.22
OFX_TEMPO_MIN_SIDE_TICKS = 4
OFX_TEMPO_CLUBBED = "CLUBBED"
OFX_TEMPO_STAGGERED = "STAGGERED"
OFX_TEMPO_MIXED = "MIXED"
OFX_TEMPO_UNAVAILABLE = "UNAVAILABLE"
OFX_CLUBBED_OPPOSING_EXIT_LABEL = "OFX_CLUBBED_OPPOSING_COMPLETION"
OFX_CLUBBED_OPPOSING_EXIT_REASON = "CLUBBED_OPPOSING_COMPLETION"
# Path analysis 2026-11-15 (crypto OFX paper, n=170): MIXED entries WR 26.7% / net drag;
# clubbed overshoot exits (dev >> ±10%) dominated losses. Research-only gates — not baseline OF.
OFX_SUPPRESS_MIXED_ENTRY_TEMPO = True
# Clubbed completion may overshoot the ±10% band, but not unboundedly (e.g. 58× dumps).
OFX_CLUBBED_MAX_OVERSHOOT = 0.50  # opp <= entry × (1 + 0.50)
# Prefer holding through early clubbed noise; ±10% in-band match keeps standard min-hold.
OFX_CLUBBED_MIN_HOLD_MS = 900_000  # 15 minutes
# First-minute adverse without recovery → flatten (bad-entry proxy).
OFX_EARLY_ADVERSE_BPS = 8.0
OFX_EARLY_ADVERSE_WINDOW_MS = 60_000
OFX_EARLY_ADVERSE_EXIT_REASON = "EARLY_ADVERSE_ABORT"


def entry_aggressor_qty_for_side(
    side: str, *, buy_aggressor_qty: float, sell_aggressor_qty: float
) -> Optional[float]:
    """LONG opens on buy aggressor qty; SHORT on sell aggressor qty."""
    s = str(side or "").upper()
    if s == "LONG":
        return float(buy_aggressor_qty)
    if s == "SHORT":
        return float(sell_aggressor_qty)
    return None


def _normalize_tick_rows(
    trades: Sequence[Any], *, as_of_ms: Optional[int] = None
) -> List[Tuple[int, float, str]]:
    """[(timestamp_ms, qty, aggressor)] sorted ascending; raw sizes only."""
    rows: List[Tuple[int, float, str]] = []
    for t in trades or ():
        try:
            if hasattr(t, "timestamp_ms"):
                ts = int(t.timestamp_ms)
                qty = float(t.quantity)
                agg = str(getattr(getattr(t, "side", None), "aggressor", "") or "").upper()
            elif isinstance(t, dict):
                ts = int(
                    t.get("timestamp_ms")
                    or t.get("trade_time_ms")
                    or t.get("receipt_utc_ms")
                    or t.get("exchange_timestamp_utc_ms")
                    or 0
                )
                qty = float(t.get("quantity") or t.get("last_traded_quantity") or t.get("q") or 0.0)
                side = t.get("side")
                if isinstance(side, dict):
                    agg = str(side.get("aggressor") or "").upper()
                else:
                    agg = str(
                        t.get("aggressor")
                        or t.get("entry_aggressor_side")
                        or ""
                    ).upper()
            else:
                continue
        except (TypeError, ValueError, AttributeError):
            continue
        if ts <= 0 or qty <= MIN_CLASSIFIED_QTY:
            continue
        if as_of_ms is not None and ts > int(as_of_ms):
            continue
        if agg not in ("BUY", "SELL"):
            # Still count UNKNOWN ticks toward average baseline? No — need completed
            # classified prints only for a stable avg; skip unclassified.
            continue
        rows.append((ts, qty, agg))
    rows.sort(key=lambda r: (r[0], r[1]))
    return rows


def evaluate_ofx_entry_volume_surge(
    *,
    entry_volume: float,
    preceding_tick_volumes: Sequence[float],
    lookback: int = OFX_ENTRY_SURGE_LOOKBACK_TICKS,
    multiple: float = OFX_ENTRY_SURGE_MULTIPLE,
) -> Dict[str, Any]:
    """
    True when entry_volume >= multiple × mean(preceding lookback tick volumes).

    Uses raw completed-tick quantities only. Does **not** consult L1 OF,
    resting imbalance, reverse-OF, or any other order-flow signal.
    """
    lb = max(1, int(lookback))
    mult = float(multiple)
    try:
        entry_q = float(entry_volume)
    except (TypeError, ValueError):
        entry_q = 0.0
    preceding = []
    for v in preceding_tick_volumes or ():
        try:
            q = float(v)
        except (TypeError, ValueError):
            continue
        if q > MIN_CLASSIFIED_QTY:
            preceding.append(q)
    base = {
        "passed": False,
        "ofx_entry_surge_label": None,
        "entry_volume": round(entry_q, 8) if entry_q else entry_q,
        "preceding_tick_count": len(preceding),
        "lookback_required": lb,
        "surge_multiple": mult,
        "avg_preceding_volume": None,
        "surge_threshold": None,
        "surge_ratio": None,
        "order_flow_signals_excluded": True,
        "baseline_of_inert": True,
    }
    if entry_q <= MIN_CLASSIFIED_QTY:
        return {**base, "reason": "entry_volume_unavailable"}
    if len(preceding) < lb:
        return {**base, "reason": "insufficient_preceding_ticks"}
    window = preceding[-lb:]
    avg = sum(window) / float(lb)
    if avg <= MIN_CLASSIFIED_QTY:
        return {**base, "reason": "avg_volume_unavailable", "avg_preceding_volume": avg}
    threshold = mult * avg
    ratio = entry_q / avg
    passed = entry_q + 1e-12 >= threshold
    return {
        **base,
        "passed": bool(passed),
        "ofx_entry_surge_label": OFX_ENTRY_SURGE_LABEL if passed else None,
        "avg_preceding_volume": round(avg, 8),
        "surge_threshold": round(threshold, 8),
        "surge_ratio": round(ratio, 6),
        "preceding_tick_count": len(window),
        "reason": "surge_pass" if passed else "surge_below_multiple",
    }


def ofx_entry_surge_from_trades(
    trades: Sequence[Any],
    *,
    direction: str,
    lookback: int = OFX_ENTRY_SURGE_LOOKBACK_TICKS,
    multiple: float = OFX_ENTRY_SURGE_MULTIPLE,
    as_of_ms: Optional[int] = None,
) -> Dict[str, Any]:
    """
    Latest completed tick (matching LONG→BUY / SHORT→SELL) must be ≥ 3×
    the average quantity of the preceding 20 completed ticks.
    """
    want = "BUY" if str(direction or "").upper() == "LONG" else (
        "SELL" if str(direction or "").upper() == "SHORT" else None
    )
    empty = {
        "passed": False,
        "ofx_entry_surge_label": None,
        "entry_volume": None,
        "entry_aggressor_side": want,
        "entry_tick_timestamp_ms": None,
        "preceding_tick_count": 0,
        "lookback_required": int(lookback),
        "surge_multiple": float(multiple),
        "avg_preceding_volume": None,
        "surge_threshold": None,
        "surge_ratio": None,
        "order_flow_signals_excluded": True,
        "baseline_of_inert": True,
        "reason": "invalid_direction",
    }
    if want is None:
        return empty

    rows = _normalize_tick_rows(trades, as_of_ms=as_of_ms)
    lb = max(1, int(lookback))
    if len(rows) < lb + 1:
        return {
            **empty,
            "preceding_tick_count": max(0, len(rows) - 1),
            "reason": "insufficient_preceding_ticks",
        }

    # Walk back to the latest tick on the entry aggressor side
    entry_idx = None
    for i in range(len(rows) - 1, -1, -1):
        if rows[i][2] == want:
            entry_idx = i
            break
    if entry_idx is None:
        return {**empty, "reason": "no_side_matching_entry_tick"}
    if entry_idx < lb:
        return {
            **empty,
            "entry_volume": rows[entry_idx][1],
            "entry_tick_timestamp_ms": rows[entry_idx][0],
            "preceding_tick_count": entry_idx,
            "reason": "insufficient_preceding_ticks",
        }

    entry_ts, entry_qty, entry_agg = rows[entry_idx]
    preceding_vols = [q for _, q, _ in rows[entry_idx - lb : entry_idx]]
    gate = evaluate_ofx_entry_volume_surge(
        entry_volume=entry_qty,
        preceding_tick_volumes=preceding_vols,
        lookback=lb,
        multiple=multiple,
    )
    return {
        **gate,
        "entry_aggressor_side": entry_agg,
        "entry_tick_timestamp_ms": entry_ts,
        "entry_volume": round(entry_qty, 8),
    }


def opposite_aggressor_label(side: str) -> Optional[str]:
    s = str(side or "").upper()
    if s == "LONG":
        return "SELL"
    if s == "SHORT":
        return "BUY"
    return None


def classify_ofx_aggressor_tempo(
    trades: Sequence[Any],
    *,
    aggressor_side: str,
    lookback: int = OFX_TEMPO_LOOKBACK_TICKS,
    clubbed_top_share: float = OFX_TEMPO_CLUBBED_TOP_SHARE,
    staggered_top_share: float = OFX_TEMPO_STAGGERED_TOP_SHARE,
    min_side_ticks: int = OFX_TEMPO_MIN_SIDE_TICKS,
    as_of_ms: Optional[int] = None,
) -> Dict[str, Any]:
    """
    Classify same-side aggressor tempo as STAGGERED / CLUBBED / MIXED.

    Uses raw executed (or derived) tick sizes only. Institutional-style
    staggered flow → low top-1 share; clubbed dump/lift → high top-1 share.
    """
    want = str(aggressor_side or "").upper()
    base = {
        "tempo": OFX_TEMPO_UNAVAILABLE,
        "aggressor_side": want if want in ("BUY", "SELL") else None,
        "side_tick_count": 0,
        "side_qty_sum": 0.0,
        "top1_qty": None,
        "top1_share": None,
        "lookback_ticks": int(lookback),
        "clubbed_top_share": float(clubbed_top_share),
        "staggered_top_share": float(staggered_top_share),
        "baseline_of_inert": True,
        "reason": "invalid_aggressor_side",
    }
    if want not in ("BUY", "SELL"):
        return base
    rows = _normalize_tick_rows(trades, as_of_ms=as_of_ms)
    lb = max(1, int(lookback))
    window = rows[-lb:] if rows else []
    side_qtys = [q for _, q, agg in window if agg == want]
    n = len(side_qtys)
    total = float(sum(side_qtys))
    if n < max(1, int(min_side_ticks)) or total <= MIN_CLASSIFIED_QTY:
        return {
            **base,
            "side_tick_count": n,
            "side_qty_sum": round(total, 8),
            "reason": "insufficient_side_ticks",
        }
    top1 = float(max(side_qtys))
    share = top1 / total
    if share + 1e-12 >= float(clubbed_top_share):
        tempo = OFX_TEMPO_CLUBBED
        reason = "top1_share_clubbed"
    elif share <= float(staggered_top_share) + 1e-12:
        tempo = OFX_TEMPO_STAGGERED
        reason = "top1_share_staggered"
    else:
        tempo = OFX_TEMPO_MIXED
        reason = "top1_share_mixed"
    return {
        **base,
        "tempo": tempo,
        "side_tick_count": n,
        "side_qty_sum": round(total, 8),
        "top1_qty": round(top1, 8),
        "top1_share": round(share, 6),
        "reason": reason,
    }


def ofx_entry_qty_for_tempo(
    *,
    surge_entry_volume: float,
    entry_side_tempo: Dict[str, Any],
) -> Dict[str, Any]:
    """
    Stamp entry aggressor qty from tempo:
      STAGGERED → sum of same-side lookback qty (distributed buy/sell program)
      CLUBBED / MIXED / other → surge tick qty (burst print)
    """
    try:
        surge_q = float(surge_entry_volume)
    except (TypeError, ValueError):
        surge_q = 0.0
    tempo = str((entry_side_tempo or {}).get("tempo") or OFX_TEMPO_UNAVAILABLE)
    side_sum = float((entry_side_tempo or {}).get("side_qty_sum") or 0.0)
    if tempo == OFX_TEMPO_STAGGERED and side_sum > MIN_CLASSIFIED_QTY:
        qty = side_sum
        source = "staggered_same_side_sum"
    else:
        qty = surge_q
        source = "surge_tick"
    return {
        "entry_aggressor_qty": round(float(qty), 8) if qty else 0.0,
        "entry_qty_source": source,
        "entry_side_tempo": tempo,
        "surge_entry_volume": round(surge_q, 8) if surge_q else surge_q,
        "staggered_side_qty_sum": round(side_sum, 8),
        "baseline_of_inert": True,
    }


def should_suppress_ofx_entry_for_tempo(
    *,
    direction: str,
    entry_side_tempo: Dict[str, Any],
    opposite_side_tempo: Dict[str, Any],
    suppress_mixed_entry: bool = OFX_SUPPRESS_MIXED_ENTRY_TEMPO,
) -> Dict[str, Any]:
    """
    Suppress fresh OFX entry when:
      - entry-side tempo is MIXED (path analysis: dominant net drag), or
      - clubbed opposite flow dominates a thin/staggered entry side
        (typical cover/reverse of a dump or lift, not a new program).
    """
    d = str(direction or "").upper()
    entry_t = str((entry_side_tempo or {}).get("tempo") or "")
    opp_t = str((opposite_side_tempo or {}).get("tempo") or "")
    entry_sum = float((entry_side_tempo or {}).get("side_qty_sum") or 0.0)
    opp_sum = float((opposite_side_tempo or {}).get("side_qty_sum") or 0.0)
    suppress = False
    reason = "tempo_entry_ok"
    if d in ("LONG", "SHORT") and bool(suppress_mixed_entry) and entry_t == OFX_TEMPO_MIXED:
        suppress = True
        reason = "mixed_entry_tempo_suppressed"
    elif d in ("LONG", "SHORT") and opp_t == OFX_TEMPO_CLUBBED:
        if entry_t in (OFX_TEMPO_STAGGERED, OFX_TEMPO_MIXED, OFX_TEMPO_UNAVAILABLE):
            if opp_sum > MIN_CLASSIFIED_QTY and opp_sum >= max(entry_sum, MIN_CLASSIFIED_QTY) * 1.25:
                suppress = True
                reason = "clubbed_opposite_dominates_entry_side"
    return {
        "suppress": bool(suppress),
        "reason": reason,
        "direction": d if d in ("LONG", "SHORT") else None,
        "entry_side_tempo": entry_t or None,
        "opposite_side_tempo": opp_t or None,
        "entry_side_qty_sum": round(entry_sum, 8),
        "opposite_side_qty_sum": round(opp_sum, 8),
        "suppress_mixed_entry": bool(suppress_mixed_entry),
        "baseline_of_inert": True,
    }


def is_ofx_clubbed_opposing_completion(
    *,
    entry_aggressor_qty: float,
    opposite_aggressor_qty: float,
    opposite_tempo: str,
    match_dev_max: float = OFX_VOLUME_MATCH_DEV_MAX,
    max_overshoot: float = OFX_CLUBBED_MAX_OVERSHOOT,
) -> Dict[str, Any]:
    """
    Exit path when opposing flow is CLUBBED and has filled at least
    entry×(1−match_dev_max), without unbounded overshoot.

    Allows completion when a dump/lift modestly overshoots the ±band that a
    staggered entry would otherwise never match — but rejects extreme overshoots
    that path analysis tied to giveback losses.
    """
    try:
        entry_q = float(entry_aggressor_qty)
        opp_q = float(opposite_aggressor_qty)
    except (TypeError, ValueError):
        entry_q, opp_q = 0.0, 0.0
    hi = max(0.0, float(match_dev_max))
    try:
        cap = max(0.0, float(max_overshoot))
    except (TypeError, ValueError):
        cap = float(OFX_CLUBBED_MAX_OVERSHOOT)
    floor = entry_q * (1.0 - hi) if entry_q > MIN_CLASSIFIED_QTY else 0.0
    ceiling = entry_q * (1.0 + cap) if entry_q > MIN_CLASSIFIED_QTY else 0.0
    tempo = str(opposite_tempo or "")
    if tempo != OFX_TEMPO_CLUBBED:
        reason = "opposite_tempo_not_clubbed"
        ok = False
    elif not (entry_q > MIN_CLASSIFIED_QTY and floor > MIN_CLASSIFIED_QTY):
        reason = "clubbed_opposite_below_entry_floor"
        ok = False
    elif opp_q + 1e-12 < floor:
        reason = "clubbed_opposite_below_entry_floor"
        ok = False
    elif ceiling > MIN_CLASSIFIED_QTY and opp_q > ceiling + 1e-12:
        reason = "clubbed_opposite_overshoot_exceeds_cap"
        ok = False
    else:
        reason = OFX_CLUBBED_OPPOSING_EXIT_REASON
        ok = True
    return {
        "exit": bool(ok),
        "ofx_volume_exit_label": OFX_CLUBBED_OPPOSING_EXIT_LABEL if ok else None,
        "reason": reason,
        "entry_aggressor_qty": round(entry_q, 8),
        "opposite_aggressor_qty": round(opp_q, 8),
        "completion_floor_qty": round(floor, 8),
        "completion_ceiling_qty": round(ceiling, 8),
        "max_overshoot": round(cap, 6),
        "opposite_tempo": tempo or None,
        "match_dev_max": hi,
        "baseline_of_inert": True,
    }


def is_ofx_opposing_volume_match(
    *,
    entry_aggressor_qty: float,
    opposite_aggressor_qty: float,
    match_dev_min: float = OFX_VOLUME_MATCH_DEV_MIN,
    match_dev_max: float = OFX_VOLUME_MATCH_DEV_MAX,
) -> Dict[str, Any]:
    """
    True only when opposing aggressor qty is inside the −10%..+10% band
    (defaults; callers may pass explicit match_dev_*).

    LONG (buy entry qty Q) closes when sell aggressor qty S satisfies:
      |S − Q| / Q <= match_dev_max
    SHORT (sell entry qty Q) closes when buy aggressor qty B satisfies:
      |B − Q| / Q <= match_dev_max

    Equivalently opp ∈ [entry×(1−hi), entry×(1+hi)]. Overshoot (|dev| > hi) is
    NOT a match. No imbalance-flip shortcut.
    """
    try:
        entry_q = float(entry_aggressor_qty)
        opp_q = float(opposite_aggressor_qty)
    except (TypeError, ValueError):
        entry_q, opp_q = 0.0, 0.0
    lo = max(0.0, float(match_dev_min))
    hi = max(lo, float(match_dev_max))
    pct = int(round(hi * 100))
    if entry_q <= MIN_CLASSIFIED_QTY:
        return {
            "match": False,
            "ofx_volume_exit_label": None,
            "entry_aggressor_qty": entry_q,
            "opposite_aggressor_qty": opp_q,
            "match_deviation": None,
            "match_dev_min": lo,
            "match_dev_max": hi,
            "match_tolerance_band": [-hi, hi],
            "reason": "entry_qty_unavailable",
            "baseline_of_inert": True,
        }
    deviation = abs(opp_q - entry_q) / entry_q
    # Strict symmetric band: |dev| in [lo, hi] (default lo=0, hi=0.10 → −10%..+10%).
    # Tiny epsilon so float edges like 1.10 / 1.0 count as inclusive +10%.
    eps = 1e-12
    in_band = (lo - eps) <= deviation <= (hi + eps)
    overshoot = deviation > (hi + eps)
    undershoot = deviation < (lo - eps)  # only when lo > 0
    matched = bool(in_band)
    if matched:
        reason = f"matched_in_pm{pct}_band"
    elif overshoot:
        reason = f"overshoot_outside_pm{pct}_band"
    elif opp_q <= 0:
        reason = "awaiting_opposing_volume"
    else:
        reason = f"undershoot_outside_pm{pct}_band"
    return {
        "match": matched,
        "ofx_volume_exit_label": OFX_VOLUME_MATCH_EXIT_LABEL if matched else None,
        "entry_aggressor_qty": round(entry_q, 8),
        "opposite_aggressor_qty": round(opp_q, 8),
        "match_deviation": round(deviation, 6),
        "match_signed_deviation": round((opp_q - entry_q) / entry_q, 6),
        "match_in_tolerance_band": bool(in_band),
        "match_overshoot": bool(overshoot),
        "match_undershoot": bool(undershoot and not matched),
        "match_deviation_in_band": bool(in_band),
        "match_deviation_in_pm6_band": bool(in_band),  # legacy alias
        "match_dev_min": lo,
        "match_dev_max": hi,
        "match_tolerance_band": [-hi, hi],
        "reason": reason,
        "baseline_of_inert": True,
    }


def is_ofx_volume_match_exit(
    active_side: str,
    *,
    entry_aggressor_qty: float,
    opposite_aggressor_qty_since_entry: float,
    match_dev_min: float = OFX_VOLUME_MATCH_DEV_MIN,
    match_dev_max: float = OFX_VOLUME_MATCH_DEV_MAX,
) -> Dict[str, Any]:
    """Paper-book exit: opposing aggressor qty matches entry aggressor qty (−10%..+10%)."""
    opp_label = opposite_aggressor_label(active_side)
    match = is_ofx_opposing_volume_match(
        entry_aggressor_qty=entry_aggressor_qty,
        opposite_aggressor_qty=opposite_aggressor_qty_since_entry,
        match_dev_min=match_dev_min,
        match_dev_max=match_dev_max,
    )
    return {
        "exit": bool(match.get("match")),
        "ofx_volume_exit_label": match.get("ofx_volume_exit_label"),
        "opposite_side": (
            "SHORT" if str(active_side).upper() == "LONG" else (
                "LONG" if str(active_side).upper() == "SHORT" else None
            )
        ),
        "opposite_aggressor": opp_label,
        **match,
    }


def volume_imbalance_from_result(result: ExecutedFlowWindowResult) -> Optional[float]:
    tot = float(result.total_classified_qty or 0.0)
    if tot <= MIN_CLASSIFIED_QTY:
        return None
    return float(result.signed_delta_qty) / tot


def classify_ofx_volume_support(
    side: str,
    result: ExecutedFlowWindowResult,
    *,
    imbalance_threshold: float = OFX_VOLUME_IMBALANCE_THRESHOLD,
) -> Dict[str, Any]:
    """
    Mark LONG/SHORT when executed aggressor volume agrees with side.

    LONG: volume_imbalance >= +threshold (buy-aggressor dominant)
    SHORT: volume_imbalance <= -threshold (sell-aggressor dominant)
    """
    empty = {
        "ofx_volume_supported": False,
        "ofx_volume_label": None,
        "ofx_volume_imbalance": None,
        "signed_delta_qty": result.signed_delta_qty,
        "buy_aggressor_qty": result.buy_aggressor_qty,
        "sell_aggressor_qty": result.sell_aggressor_qty,
        "total_classified_qty": result.total_classified_qty,
        "classification_coverage": result.classification_coverage,
        "data_quality": result.data_quality,
        "status": result.status,
        "construct": "EXECUTED_AGGRESSOR_VOLUME",
        "baseline_of_inert": True,
    }
    if result.data_quality == "UNAVAILABLE" or result.n_classified <= 0:
        return {**empty, "ofx_volume_source": "unavailable"}

    imb = volume_imbalance_from_result(result)
    if imb is None:
        return {**empty, "ofx_volume_source": "no_classified_qty"}

    side_u = str(side or "").upper()
    thr = float(imbalance_threshold)
    if side_u == "LONG":
        ok = imb >= thr
    elif side_u == "SHORT":
        ok = imb <= -thr
    else:
        return {
            **empty,
            "ofx_volume_imbalance": round(imb, 6),
            "ofx_volume_source": result.source,
        }

    return {
        "ofx_volume_supported": bool(ok),
        "ofx_volume_label": OFX_VOLUME_LABEL if ok else None,
        "ofx_volume_imbalance": round(imb, 6),
        "signed_delta_qty": result.signed_delta_qty,
        "buy_aggressor_qty": result.buy_aggressor_qty,
        "sell_aggressor_qty": result.sell_aggressor_qty,
        "total_classified_qty": result.total_classified_qty,
        "classification_coverage": result.classification_coverage,
        "data_quality": result.data_quality,
        "status": result.status,
        "ofx_volume_source": result.source,
        "ofx_volume_imbalance_threshold": thr,
        "construct": "EXECUTED_AGGRESSOR_VOLUME",
        "baseline_of_inert": True,
        "side": side_u,
    }


def is_ofx_volume_exit_signal(
    active_side: str,
    result: ExecutedFlowWindowResult,
    *,
    imbalance_threshold: float = OFX_VOLUME_IMBALANCE_THRESHOLD,
) -> Dict[str, Any]:
    """Opposite-side volume aggression confirm (research observation only)."""
    opp = "SHORT" if str(active_side).upper() == "LONG" else (
        "LONG" if str(active_side).upper() == "SHORT" else ""
    )
    if not opp:
        return {
            "exit": False,
            "ofx_volume_exit_label": None,
            "opposite_side": None,
            "ofx_volume_imbalance": None,
        }
    mark = classify_ofx_volume_support(opp, result, imbalance_threshold=imbalance_threshold)
    exited = bool(mark.get("ofx_volume_supported"))
    return {
        "exit": exited,
        "ofx_volume_exit_label": OFX_VOLUME_EXIT_LABEL if exited else None,
        "opposite_side": opp,
        "ofx_volume_imbalance": mark.get("ofx_volume_imbalance"),
        "ofx_volume_source": mark.get("ofx_volume_source"),
        "mark": mark,
        "baseline_of_inert": True,
    }


def inspect_ofx_volume_book(
    result: ExecutedFlowWindowResult,
    *,
    primary_side: str = "LONG",
    imbalance_threshold: float = OFX_VOLUME_IMBALANCE_THRESHOLD,
) -> Dict[str, Any]:
    side = str(primary_side or "LONG").upper()
    if side not in ("LONG", "SHORT"):
        side = "LONG"
    long_m = classify_ofx_volume_support("LONG", result, imbalance_threshold=imbalance_threshold)
    short_m = classify_ofx_volume_support("SHORT", result, imbalance_threshold=imbalance_threshold)
    primary = long_m if side == "LONG" else short_m
    return {
        "ofx_volume_supported": bool(primary.get("ofx_volume_supported")),
        "ofx_volume_label": primary.get("ofx_volume_label"),
        "ofx_volume_imbalance": primary.get("ofx_volume_imbalance"),
        "ofx_volume_source": primary.get("ofx_volume_source"),
        "ofx_volume_side": side,
        "ofx_volume_long_supported": bool(long_m.get("ofx_volume_supported")),
        "ofx_volume_short_supported": bool(short_m.get("ofx_volume_supported")),
        "ofx_volume_inspection": True,
        "ofx_volume_imbalance_threshold": float(imbalance_threshold),
        "signed_delta_qty": result.signed_delta_qty,
        "buy_aggressor_qty": result.buy_aggressor_qty,
        "sell_aggressor_qty": result.sell_aggressor_qty,
        "construct": "EXECUTED_AGGRESSOR_VOLUME",
        "baseline_of_inert": True,
        "candle_independent": True,
        "note": "Not baseline L1 OF_SUP; research-only volume aggressor mark",
    }
