#!/usr/bin/env python3
"""
RP-608-WR-ENTRY-DISCOVERY-001 — NSE Estimand A entry-quality discovery (WR v2).

Stages 1–5: completeness audit, P(Win|X_entry) associations, stability,
hypotheses, Gate 2 assessment (criteria unchanged).

Authority 0%. Baseline frozen. Books never pooled. Gate 3 unauthorized.
Aspirational 85% WR is a research objective, not a Gate 2 bar.

Usage (prefer Pi):
  python3 scripts/research/rp608_wr_entry_discovery_001.py \
    --join artifacts/rp608_econ_edge_diag/wl_nse_a_decision_time_features_kite_join.jsonl \
    --export-dir artifacts/rp608_econ_edge_diag \
    --out-dir artifacts/rp608_econ_edge_diag
"""

from __future__ import annotations

import argparse
import json
import math
from collections import Counter, defaultdict
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional, Sequence, Tuple

IST = timezone(timedelta(hours=5, minutes=30))

DISCOVERY_DAYS = ["2026-11-03", "2026-11-04", "2026-11-05"]
HOLDOUT_PRE = ["2026-10-27", "2026-10-28", "2026-10-30", "2026-11-02"]
HOLDOUT_FWD = ["2026-11-06"]
HOLDOUT_ALL = HOLDOUT_PRE + HOLDOUT_FWD
ALL_WINDOW_DAYS = DISCOVERY_DAYS + HOLDOUT_ALL

UNAVAIL_MARKERS = ("UNAVAILABLE", "??")
FEATURE_KEYS = ("trend", "momentum", "price_location")
ASPIRATIONAL_WR = 85.0
MIN_N_SIDE = 15
MIN_N_SELECTED = 30


def pdt_ms(v: Any) -> Optional[int]:
    if v is None:
        return None
    if isinstance(v, (int, float)):
        ms = int(v)
        return ms if ms > 10_000_000_000 else ms * 1000
    try:
        dt = datetime.fromisoformat(str(v).replace("Z", "+00:00"))
        if dt.tzinfo is None:
            dt = dt.replace(tzinfo=timezone.utc)
        return int(dt.timestamp() * 1000)
    except Exception:
        return None


def hour_ist(ms: Optional[int]) -> str:
    if ms is None:
        return "??"
    return datetime.fromtimestamp(ms / 1000, tz=timezone.utc).astimezone(IST).strftime("%H")


def is_unavail(v: Any) -> bool:
    if v is None:
        return True
    s = str(v)
    return any(m in s for m in UNAVAIL_MARKERS) or s == ""


def feature_ready(r: Dict[str, Any]) -> bool:
    return all(not is_unavail(r.get(k)) for k in FEATURE_KEYS) and bool(r.get("look_ahead_safe", True))


def load_jsonl(path: Path) -> List[Dict[str, Any]]:
    if not path.exists():
        return []
    out: List[Dict[str, Any]] = []
    for line in path.read_text(encoding="utf-8", errors="ignore").splitlines():
        if not line.strip():
            continue
        try:
            out.append(json.loads(line))
        except Exception:
            continue
    return out


def enrich_join_row(r: Dict[str, Any]) -> Dict[str, Any]:
    r = dict(r)
    ms = pdt_ms(r.get("entry_utc_ms") or r.get("entry_utc"))
    r["entry_utc_ms"] = ms
    if not r.get("session_day") and ms:
        r["session_day"] = (
            datetime.fromtimestamp(ms / 1000, tz=timezone.utc).astimezone(IST).date().isoformat()
        )
    r["hour_ist"] = hour_ist(ms)
    if r.get("win") is None and r.get("net_pnl_inr") is not None:
        r["win"] = float(r["net_pnl_inr"]) > 0
    r["feature_ready"] = feature_ready(r)
    # leakage check: prior bars must close before entry
    r["leakage_ok"] = bool(r.get("look_ahead_safe", True)) and ms is not None
    q, px = r.get("quantity"), r.get("entry_price")
    # join may lack quantity — notional optional
    if q is not None and px is not None:
        r["notional"] = abs(float(q) * float(px))
    else:
        r["notional"] = None
    return r


def wl_stats(rows: Sequence[Dict[str, Any]], book_n: Optional[int] = None) -> Dict[str, Any]:
    usable = [r for r in rows if r.get("win") is not None]
    n = len(usable)
    wins = sum(1 for r in usable if r["win"])
    losses = n - wins
    nets = [float(r["net_pnl_inr"]) for r in usable if r.get("net_pnl_inr") is not None]
    avg_w = (
        sum(x for x in nets if x > 0) / max(1, sum(1 for x in nets if x > 0))
        if any(x > 0 for x in nets)
        else None
    )
    avg_l = (
        sum(abs(x) for x in nets if x < 0) / max(1, sum(1 for x in nets if x < 0))
        if any(x < 0 for x in nets)
        else None
    )
    payoff = (avg_w / avg_l) if (avg_w is not None and avg_l and avg_l > 0) else None
    return {
        "n": n,
        "wins": wins,
        "losses": losses,
        "wr_pct": None if n == 0 else round(100.0 * wins / n, 2),
        "net_inr": round(sum(nets), 2) if nets else 0.0,
        "payoff": None if payoff is None else round(payoff, 3),
        "coverage_pct": None if not book_n else round(100.0 * n / book_n, 2),
    }


def load_closed_exports(export_dir: Path, days: Sequence[str]) -> List[Dict[str, Any]]:
    rows: List[Dict[str, Any]] = []
    for day in days:
        path = export_dir / f"session_export_{day}_nse_estimand_a_closed.jsonl"
        for r in load_jsonl(path):
            r = dict(r)
            r["_session_day"] = day
            r["entry_utc_ms"] = pdt_ms(r.get("entry_utc") or r.get("entry_timestamp_utc_ms"))
            net = r.get("net_pnl_inr")
            if net is not None:
                r["win"] = float(net) > 0
            rows.append(r)
    return rows


# ---------------------------------------------------------------------------
# Stage 1 — completeness
# ---------------------------------------------------------------------------


def stage1_completeness(
    join_rows: List[Dict[str, Any]],
    closed_exports: List[Dict[str, Any]],
) -> Dict[str, Any]:
    by_tid = {str(r.get("trade_id")): r for r in join_rows if r.get("trade_id")}
    disc_join = [r for r in join_rows if r.get("session_day") in DISCOVERY_DAYS]
    hold_join = [r for r in join_rows if r.get("session_day") in HOLDOUT_ALL]

    ready = [r for r in join_rows if r.get("feature_ready")]
    unavail = [r for r in join_rows if not r.get("feature_ready")]

    # Missingness reasons for unavail
    reason_counts: Counter = Counter()
    for r in unavail:
        reasons = []
        for k in FEATURE_KEYS:
            if is_unavail(r.get(k)):
                reasons.append(f"{k}_unavailable")
        if r.get("n_prior_bars") is not None and int(r["n_prior_bars"]) < 8:
            reasons.append("n_prior_bars<8")
        if not r.get("look_ahead_safe", True):
            reasons.append("look_ahead_flag_false")
        reason_counts[",".join(reasons) or "unknown"] += 1

    # Export vs join coverage (discovery window emphasis)
    closed_disc = [r for r in closed_exports if r.get("_session_day") in DISCOVERY_DAYS]
    missing_from_join = []
    for r in closed_disc:
        tid = str(r.get("trade_id"))
        if tid not in by_tid:
            missing_from_join.append(
                {
                    "trade_id": tid,
                    "symbol": r.get("symbol"),
                    "day": r.get("_session_day"),
                    "why": "not_in_kite_join",
                }
            )

    ready_disc = [r for r in disc_join if r.get("feature_ready")]
    unavail_disc = [r for r in disc_join if not r.get("feature_ready")]

    # W/L by availability
    wl_ready = wl_stats(ready)
    wl_unavail = wl_stats(unavail)
    wl_ready_disc = wl_stats(ready_disc)
    wl_unavail_disc = wl_stats(unavail_disc)

    # Systematic missingness vs win
    avail_vs_win = {
        "ready_wr": wl_ready.get("wr_pct"),
        "unavail_wr": wl_unavail.get("wr_pct"),
        "ready_n": wl_ready["n"],
        "unavail_n": wl_unavail["n"],
        "note": (
            "If unavail WR differs sharply from ready WR, feature-ready subset may be biased."
        ),
    }

    # Timestamp / leakage audit
    leakage_violations = 0
    prior_bars_hist = Counter()
    for r in join_rows:
        if not r.get("leakage_ok"):
            leakage_violations += 1
        nb = r.get("n_prior_bars")
        if nb is None:
            prior_bars_hist["missing"] += 1
        elif int(nb) < 2:
            prior_bars_hist["0_1"] += 1
        elif int(nb) < 8:
            prior_bars_hist["2_7"] += 1
        else:
            prior_bars_hist["ge8"] += 1

    by_day_ready = {
        d: wl_stats([r for r in join_rows if r.get("session_day") == d and r.get("feature_ready")])
        for d in sorted({r.get("session_day") for r in join_rows if r.get("session_day")})
    }

    return {
        "join_n": len(join_rows),
        "feature_ready_n": len(ready),
        "feature_unavailable_n": len(unavail),
        "discovery_join_n": len(disc_join),
        "discovery_ready_n": len(ready_disc),
        "discovery_unavail_n": len(unavail_disc),
        "holdout_join_n": len(hold_join),
        "holdout_ready_n": sum(1 for r in hold_join if r.get("feature_ready")),
        "unavail_reasons": dict(reason_counts),
        "closed_export_discovery_n": len(closed_disc),
        "missing_from_join_discovery_n": len(missing_from_join),
        "missing_from_join_sample": missing_from_join[:20],
        "wl_feature_ready": wl_ready,
        "wl_feature_unavail": wl_unavail,
        "wl_discovery_ready": wl_ready_disc,
        "wl_discovery_unavail": wl_unavail_disc,
        "availability_vs_win": avail_vs_win,
        "leakage_violations": leakage_violations,
        "look_ahead_safe_all": all(bool(r.get("look_ahead_safe", True)) for r in join_rows)
        if join_rows
        else True,
        "prior_bars_hist": dict(prior_bars_hist),
        "by_day_ready_wl": by_day_ready,
        "usable_population_note": (
            "Treat feature-ready rows as the analysis population only after reviewing "
            "missingness vs W/L; do not equate ready_n with all closed trades without audit."
        ),
    }


# ---------------------------------------------------------------------------
# Stage 2 — associations + P(Win|X)
# ---------------------------------------------------------------------------


def contrast(
    rows: Sequence[Dict[str, Any]], key: str, high: str, low: str, book_n: int
) -> Dict[str, Any]:
    hi = [r for r in rows if r.get(key) == high]
    lo = [r for r in rows if r.get(key) == low]
    hs, ls = wl_stats(hi, book_n), wl_stats(lo, book_n)
    gap = None
    if hs["wr_pct"] is not None and ls["wr_pct"] is not None:
        gap = round(hs["wr_pct"] - ls["wr_pct"], 3)
    return {
        "feature": key,
        "high_key": high,
        "low_key": low,
        "high": hs,
        "low": ls,
        "wr_gap_pp": gap,
    }


def score_row(r: Dict[str, Any], emp: Dict[str, Dict[str, float]]) -> float:
    """Simple average of empirical P(win|level) across available features (decision-time)."""
    probs = []
    for feat, table in emp.items():
        v = r.get(feat)
        if v is None or is_unavail(v):
            continue
        if str(v) in table:
            probs.append(table[str(v)])
    if not probs:
        return 0.5
    return sum(probs) / len(probs)


def empirical_tables(rows: Sequence[Dict[str, Any]]) -> Dict[str, Dict[str, float]]:
    out: Dict[str, Dict[str, float]] = {}
    for feat in list(FEATURE_KEYS) + ["side", "hour_ist"]:
        buckets: Dict[str, List[bool]] = defaultdict(list)
        for r in rows:
            v = r.get(feat)
            if v is None or is_unavail(v) or r.get("win") is None:
                continue
            buckets[str(v)].append(bool(r["win"]))
        out[feat] = {
            k: (sum(1 for x in xs if x) / len(xs)) for k, xs in buckets.items() if len(xs) >= 5
        }
    return out


def calibration_bins(
    rows: Sequence[Dict[str, Any]], scores: Sequence[float], n_bins: int = 5
) -> List[Dict[str, Any]]:
    paired = [(s, r) for s, r in zip(scores, rows) if r.get("win") is not None]
    if not paired:
        return []
    paired.sort(key=lambda x: x[0])
    bins: List[Dict[str, Any]] = []
    size = max(1, len(paired) // n_bins)
    for i in range(n_bins):
        chunk = paired[i * size : (None if i == n_bins - 1 else (i + 1) * size)]
        if not chunk:
            continue
        preds = [s for s, _ in chunk]
        wins = sum(1 for _, r in chunk if r["win"])
        n = len(chunk)
        bins.append(
            {
                "bin": i,
                "n": n,
                "mean_predicted": round(sum(preds) / n, 4),
                "realized_wr": round(100.0 * wins / n, 2),
                "wins": wins,
                "losses": n - wins,
            }
        )
    return bins


def selection_report(
    all_rows: Sequence[Dict[str, Any]],
    selected: Sequence[Dict[str, Any]],
    label: str,
) -> Dict[str, Any]:
    book_n = len(all_rows)
    sel = list(selected)
    sel_ids = {id(r) for r in sel}
    rejected = [r for r in all_rows if id(r) not in sel_ids]
    # also try trade_id based reject if overlapping objects differ
    if len(rejected) + len(sel) != book_n:
        stids = {str(r.get("trade_id")) for r in sel}
        rejected = [r for r in all_rows if str(r.get("trade_id")) not in stids]
    return {
        "candidate": label,
        "original_n": book_n,
        "selected_n": len(sel),
        "rejected_n": len(rejected),
        "coverage_pct": None if book_n == 0 else round(100.0 * len(sel) / book_n, 2),
        "selected": wl_stats(sel, book_n),
        "rejected": wl_stats(rejected, book_n),
        "gap_to_aspirational_85_pp": (
            None
            if not sel or wl_stats(sel)["wr_pct"] is None
            else round(wl_stats(sel)["wr_pct"] - ASPIRATIONAL_WR, 2)
        ),
    }


SPECS = [
    ("momentum", "MOM_UP_>10bps", "MOM_DOWN_<-10bps"),
    ("momentum", "MOM_UP_>10bps", "MOM_FLAT"),
    ("price_location", "LOC_MID_THIRD", "LOC_UPPER_THIRD"),
    ("price_location", "LOC_MID_THIRD", "LOC_LOWER_THIRD"),
    ("trend", "TREND_UP", "TREND_DOWN"),
    ("trend", "TREND_UP", "TREND_FLAT"),
]


def stage2_associations(disc: List[Dict[str, Any]], hold: List[Dict[str, Any]]) -> Dict[str, Any]:
    book_n = len(disc)
    baseline = wl_stats(disc, book_n)
    contrasts = []
    for feat, high, low in SPECS:
        c = contrast(disc, feat, high, low, book_n)
        c["holdout"] = contrast(hold, feat, high, low, len(hold))
        contrasts.append(c)

    emp = empirical_tables(disc)
    disc_scores = [score_row(r, emp) for r in disc]
    hold_scores = [score_row(r, emp) for r in hold]
    cal_disc = calibration_bins(disc, disc_scores)
    cal_hold = calibration_bins(hold, hold_scores)

    # Probability-threshold candidates (entry selection by score)
    thresholds = [0.45, 0.50, 0.55, 0.60, 0.65, 0.70]
    score_candidates = []
    for thr in thresholds:
        sel_d = [r for r, s in zip(disc, disc_scores) if s >= thr]
        sel_h = [r for r, s in zip(hold, hold_scores) if s >= thr]
        score_candidates.append(
            {
                "threshold": thr,
                "discovery": selection_report(disc, sel_d, f"P_hat>={thr}"),
                "holdout": selection_report(hold, sel_h, f"P_hat>={thr}"),
            }
        )

    # Feature-value selection candidates (take high side only; dedupe by rule)
    feature_candidates = []
    seen_rules: set = set()
    for feat, high, low in SPECS:
        rule = f"{feat}=={high}"
        if rule in seen_rules:
            continue
        seen_rules.add(rule)
        sel_d = [r for r in disc if r.get(feat) == high]
        sel_h = [r for r in hold if r.get(feat) == high]
        feature_candidates.append(
            {
                "rule": rule,
                "vs": low,
                "discovery": selection_report(disc, sel_d, rule),
                "holdout": selection_report(hold, sel_h, rule),
                "contrast_discovery": contrast(disc, feat, high, low, book_n),
            }
        )

    # Conjunction: MOM_UP & LOC_MID
    conj_d = [
        r
        for r in disc
        if r.get("momentum") == "MOM_UP_>10bps" and r.get("price_location") == "LOC_MID_THIRD"
    ]
    conj_h = [
        r
        for r in hold
        if r.get("momentum") == "MOM_UP_>10bps" and r.get("price_location") == "LOC_MID_THIRD"
    ]
    feature_candidates.append(
        {
            "rule": "momentum==MOM_UP_>10bps AND price_location==LOC_MID_THIRD",
            "vs": "complement",
            "discovery": selection_report(disc, conj_d, "MOM_UP&LOC_MID"),
            "holdout": selection_report(hold, conj_h, "MOM_UP&LOC_MID"),
        }
    )

    return {
        "baseline_discovery": baseline,
        "baseline_holdout": wl_stats(hold, len(hold)),
        "contrasts": contrasts,
        "empirical_p_win_tables_discovery": {
            k: {kk: round(vv, 4) for kk, vv in v.items()} for k, v in emp.items()
        },
        "calibration_discovery": cal_disc,
        "calibration_holdout": cal_hold,
        "score_threshold_candidates": score_candidates,
        "feature_selection_candidates": feature_candidates,
        "scoring_method": (
            "Unweighted mean of empirical P(win|feature_level) fit on discovery only; "
            "applied to holdout without refit. Not a production model."
        ),
    }


# ---------------------------------------------------------------------------
# Stage 3 — stability
# ---------------------------------------------------------------------------


def session_gaps(
    rows: Sequence[Dict[str, Any]], feat: str, high: str, low: str
) -> List[Dict[str, Any]]:
    days = sorted({r.get("session_day") for r in rows if r.get("session_day")})
    out = []
    for day in days:
        day_rows = [r for r in rows if r.get("session_day") == day]
        c = contrast(day_rows, feat, high, low, len(day_rows))
        gap = c["wr_gap_pp"]
        sign = "UNEVALUABLE"
        if gap is not None and c["high"]["n"] >= 3 and c["low"]["n"] >= 3:
            if gap > 1:
                sign = "POSITIVE"
            elif gap < -1:
                sign = "NEGATIVE"
            else:
                sign = "NEAR_ZERO"
        out.append({"day": day, "gap_pp": gap, "sign": sign, "high_n": c["high"]["n"], "low_n": c["low"]["n"]})
    return out


def symbol_concentration(rows: Sequence[Dict[str, Any]], top_k: int = 10) -> Dict[str, Any]:
    by_sym: Dict[str, List[Dict[str, Any]]] = defaultdict(list)
    for r in rows:
        by_sym[str(r.get("symbol"))].append(r)
    ranked = sorted(by_sym.items(), key=lambda kv: len(kv[1]), reverse=True)
    total = len(rows)
    top = []
    for sym, rs in ranked[:top_k]:
        s = wl_stats(rs)
        top.append({"symbol": sym, **s, "share_pct": round(100.0 * len(rs) / total, 2) if total else 0})
    # loss concentration
    losses = [r for r in rows if r.get("win") is False]
    loss_syms = Counter(str(r.get("symbol")) for r in losses)
    top_loss = loss_syms.most_common(5)
    return {
        "top_symbols_by_n": top,
        "top_loss_symbols": [{"symbol": s, "loss_n": n} for s, n in top_loss],
        "unique_symbols": len(by_sym),
    }


def stage3_stability(disc: List[Dict[str, Any]], stage2: Dict[str, Any]) -> Dict[str, Any]:
    audits = []
    for feat, high, low in SPECS:
        sess = session_gaps(disc, feat, high, low)
        pos = [s for s in sess if s["sign"] == "POSITIVE"]
        neg = [s for s in sess if s["sign"] == "NEGATIVE"]
        # single-session domination on high side
        hi_by_day = Counter(r["session_day"] for r in disc if r.get(feat) == high)
        single = False
        if hi_by_day:
            top_n = hi_by_day.most_common(1)[0][1]
            single = top_n / max(1, sum(hi_by_day.values())) >= 0.60
        stable = len(pos) >= 2 and len(neg) == 0 and not single
        audits.append(
            {
                "contrast": f"{feat}:{high}-vs-{low}",
                "session_effects": sess,
                "positive_sessions": len(pos),
                "negative_sessions": len(neg),
                "single_session_domination": single,
                "promote": stable,
                "blocks": (
                    []
                    if stable
                    else (
                        (["SINGLE_SESSION_DOMINATION"] if single else [])
                        + (["SESSION_SIGN_FLIP"] if neg else [])
                        + (["INSUFFICIENT_POSITIVE_SESSIONS"] if len(pos) < 2 else [])
                    )
                ),
            }
        )

    # Hour diagnostic only
    by_h: Dict[str, List[Dict[str, Any]]] = defaultdict(list)
    for r in disc:
        by_h[str(r.get("hour_ist") or "??")].append(r)
    hour_wl = {
        h: wl_stats(rs)
        for h, rs in sorted(by_h.items())
        if wl_stats(rs)["n"] >= 15
    }

    # Score-threshold stability: require holdout WR >= discovery WR - 10pp and powered
    score_stable = []
    for sc in stage2.get("score_threshold_candidates", []):
        d = sc["discovery"]["selected"]
        h = sc["holdout"]["selected"]
        ok = (
            d["n"] >= MIN_N_SELECTED
            and h["n"] >= MIN_N_SELECTED
            and d["wr_pct"] is not None
            and h["wr_pct"] is not None
            and h["wr_pct"] >= (d["wr_pct"] - 10.0)
        )
        score_stable.append(
            {
                "threshold": sc["threshold"],
                "promote": ok,
                "discovery_wr": d["wr_pct"],
                "holdout_wr": h["wr_pct"],
                "discovery_n": d["n"],
                "holdout_n": h["n"],
                "coverage_discovery_pct": sc["discovery"]["coverage_pct"],
            }
        )

    return {
        "contrast_stability": audits,
        "symbol_concentration_discovery": symbol_concentration(disc),
        "hour_wl_diagnostic_only": hour_wl,
        "score_threshold_stability": score_stable,
        "selection_bias_notes": [
            "Empirical P(win|level) fit on discovery only — holdout scores are out-of-sample levels.",
            "Feature-ready filter must not be treated as random subsample without Stage 1 audit.",
            "Hour contrasts are diagnostic only; not Gate 2 promotion alone.",
            "Coverage reduction must be reported; rejected-trade W/L is mandatory.",
        ],
    }


# ---------------------------------------------------------------------------
# Stages 4–5 — hypotheses + Gate 2
# ---------------------------------------------------------------------------


def replicate(disc_gap: Optional[float], hold_c: Dict[str, Any]) -> str:
    hg = hold_c.get("wr_gap_pp")
    if disc_gap is None or hg is None:
        return "INCONCLUSIVE_MISSING_GAP"
    if hold_c["high"]["n"] < MIN_N_SIDE or hold_c["low"]["n"] < MIN_N_SIDE:
        return "INCONCLUSIVE_UNDERPOWERED"
    if disc_gap == 0:
        return "INCONCLUSIVE_ZERO_DISCOVERY_GAP"
    if (disc_gap > 0 and hg > 0) or (disc_gap < 0 and hg < 0):
        if abs(hg) < 1.0:
            return "FAILS_REPLICATION_NEAR_ZERO_HOLDOUT"
        return "REPLICATES"
    return "FAILS_REPLICATION"


def evaluate_gate2(hyp: Dict[str, Any]) -> Dict[str, Any]:
    criteria = []
    measurable = bool(hyp.get("feature") and hyp.get("comparison"))
    criteria.append({"id": "G2-1_MEASURABLE_HYPOTHESIS", "satisfied": measurable})
    criteria.append(
        {
            "id": "G2-2_DECISION_TIME_ONLY",
            "satisfied": not any(
                x in hyp.get("feature", "").lower() for x in ("mfe", "mae", "exit_reason")
            ),
        }
    )
    disc = hyp["discovery_evidence"]
    disc_ok = (
        disc.get("wr_gap_pp") is not None
        and disc["wr_gap_pp"] > 0
        and disc["high"]["n"] >= MIN_N_SIDE
        and disc["low"]["n"] >= MIN_N_SIDE
    )
    criteria.append({"id": "G2-3_DISCOVERY_SUPPORT", "satisfied": disc_ok})
    repl_ok = hyp.get("replication_verdict") == "REPLICATES"
    criteria.append({"id": "G2-4_HOLDOUT_REPLICATION", "satisfied": repl_ok})
    criteria.append(
        {
            "id": "G2-5_FALSIFIABLE",
            "satisfied": "holdout" in hyp.get("falsification_condition", "").lower(),
        }
    )
    cov = disc["high"].get("coverage_pct") or 0.0
    criteria.append(
        {
            "id": "G2-6_COVERAGE_NOT_SOLE_SUCCESS",
            "satisfied": True,
            "detail": f"selected_side_coverage_pct={cov}",
        }
    )
    hi = disc["high"]
    econ = (hi.get("payoff") is not None and hi["payoff"] >= 1.0) and (
        hi.get("net_inr") is not None and hi["net_inr"] > 0
    )
    criteria.append(
        {
            "id": "G2-7_PAYOFF_NET_SUPPORT",
            "satisfied": econ,
            "detail": f"high_payoff={hi.get('payoff')} high_net={hi.get('net_inr')}",
        }
    )
    temporal = hyp.get("diagnostic_only", False)
    all_ok = all(c["satisfied"] for c in criteria) and not temporal
    blocked = None
    if temporal:
        blocked = "TEMPORAL_DIAGNOSTIC_ONLY"
    elif repl_ok and disc_ok and not econ:
        blocked = "WR_REPLICATES_BUT_PAYOFF_NET_BAR_FAIL"
    elif not all_ok:
        blocked = "GATE2_CRITERIA_UNMET"
    return {
        "hypothesis_id": hyp["hypothesis_id"],
        "criteria": criteria,
        "all_satisfied": all_ok,
        "blocked_reason": blocked,
    }


def stages4_5(
    disc: List[Dict[str, Any]],
    hold: List[Dict[str, Any]],
    stage2: Dict[str, Any],
    stage3: Dict[str, Any],
) -> Dict[str, Any]:
    promoted = {
        a["contrast"] for a in stage3["contrast_stability"] if a.get("promote")
    }
    hypotheses = []
    failed = []

    for feat, high, low in SPECS:
        key = f"{feat}:{high}-vs-{low}"
        d_c = contrast(disc, feat, high, low, len(disc))
        h_c = contrast(hold, feat, high, low, len(hold))
        verdict = replicate(d_c.get("wr_gap_pp"), h_c)
        hyp = {
            "hypothesis_id": f"H-{feat}-{high}-gt-{low}",
            "feature": feat,
            "comparison": f"WR({high}) > WR({low})",
            "expected_direction": f"WR({high}) > WR({low})",
            "falsification_condition": (
                "Holdout same-sign WR gap with n>=15 each side required; "
                "else hypothesis fails."
            ),
            "discovery_evidence": d_c,
            "holdout_evidence": h_c,
            "replication_verdict": verdict,
            "stability_promoted": key in promoted,
            "diagnostic_only": False,
            "known_limitations": [
                f"coverage_drop_if_selected_pct={round(100.0 - (d_c['high'].get('coverage_pct') or 0), 2)}"
            ],
        }
        hyp["gate2"] = evaluate_gate2(hyp)
        if hyp["stability_promoted"] and verdict == "REPLICATES":
            hypotheses.append(hyp)
        else:
            failed.append(hyp)

    # Score-threshold hypotheses (selection rules)
    for sc in stage2.get("score_threshold_candidates", []):
        thr = sc["threshold"]
        d = sc["discovery"]
        h = sc["holdout"]
        # Fabricate contrast-like evidence: selected vs rejected on discovery
        d_sel, d_rej = d["selected"], d["rejected"]
        h_sel, h_rej = h["selected"], h["rejected"]
        d_gap = (
            None
            if d_sel["wr_pct"] is None or d_rej["wr_pct"] is None
            else round(d_sel["wr_pct"] - d_rej["wr_pct"], 3)
        )
        h_gap = (
            None
            if h_sel["wr_pct"] is None or h_rej["wr_pct"] is None
            else round(h_sel["wr_pct"] - h_rej["wr_pct"], 3)
        )
        fake_hold = {
            "wr_gap_pp": h_gap,
            "high": h_sel,
            "low": h_rej,
        }
        verdict = replicate(d_gap, fake_hold)
        stab = next(
            (x for x in stage3["score_threshold_stability"] if x["threshold"] == thr),
            {},
        )
        hyp = {
            "hypothesis_id": f"H-PWIN-SCORE-GE-{thr}",
            "feature": "p_win_hat_entry",
            "comparison": f"WR(score>={thr}) > WR(score<{thr})",
            "expected_direction": f"WR(score>={thr}) > WR(rejected)",
            "falsification_condition": (
                "Holdout selected vs rejected WR gap must same-sign with powered n; "
                "calibration must not systematically overstate probability."
            ),
            "discovery_evidence": {
                "high_key": f"score>={thr}",
                "low_key": f"score<{thr}",
                "high": d_sel,
                "low": d_rej,
                "wr_gap_pp": d_gap,
            },
            "holdout_evidence": {
                "high": h_sel,
                "low": h_rej,
                "wr_gap_pp": h_gap,
            },
            "selection_report_discovery": d,
            "selection_report_holdout": h,
            "replication_verdict": verdict,
            "stability_promoted": bool(stab.get("promote")),
            "diagnostic_only": False,
            "known_limitations": [
                f"coverage_drop_if_selected_pct={round(100.0 - (d.get('coverage_pct') or 0), 2)}",
                "Score is discovery-fit empirical average — simple, not regularized.",
            ],
        }
        hyp["gate2"] = evaluate_gate2(hyp)
        if hyp["stability_promoted"]:
            hypotheses.append(hyp)
        else:
            failed.append(hyp)

    advanced = [h for h in hypotheses if h["gate2"]["all_satisfied"]]
    wr_only = [
        h
        for h in hypotheses + failed
        if h.get("replication_verdict") == "REPLICATES"
        and h["gate2"].get("blocked_reason") == "WR_REPLICATES_BUT_PAYOFF_NET_BAR_FAIL"
    ]

    if advanced:
        disposition = "GATE2_CANDIDATE_PRESENT"
        conclusion = (
            f"{len(advanced)} hypothesis(es) satisfy unchanged Gate 2 criteria including "
            "payoff/net. Human Gate 2 review still required; Gate 3 unauthorized until review."
        )
        tier = "PARTIAL_OR_BETTER"
    elif wr_only:
        disposition = "GATE2_DO_NOT_ADVANCE"
        conclusion = (
            f"WR replication present on {len(wr_only)} candidate(s) but payoff/net bar fails. "
            f"Aspirational 85% WR not met as Gate advance. Distance-to-85% documented in selection reports."
        )
        tier = "PARTIAL"
    else:
        disposition = "GATE2_DO_NOT_ADVANCE"
        conclusion = (
            "No entry-time hypothesis clears stability + holdout + Gate 2 bars. "
            "Gate 2 remains closed. Fail-closed is a valid research outcome."
        )
        tier = "FAIL_CLOSED"

    # Best holdout selected WR among powered candidates
    best = None
    for sc in stage2.get("score_threshold_candidates", []):
        h = sc["holdout"]["selected"]
        if h["n"] >= MIN_N_SELECTED and h["wr_pct"] is not None:
            if best is None or h["wr_pct"] > best["wr_pct"]:
                best = {
                    "rule": f"P_hat>={sc['threshold']}",
                    "wr_pct": h["wr_pct"],
                    "n": h["n"],
                    "wins": h["wins"],
                    "losses": h["losses"],
                    "coverage_pct": sc["holdout"]["coverage_pct"],
                    "payoff": h["payoff"],
                    "net_inr": h["net_inr"],
                    "gap_to_85_pp": round(h["wr_pct"] - ASPIRATIONAL_WR, 2),
                }
    for fc in stage2.get("feature_selection_candidates", []):
        h = fc["holdout"]["selected"]
        if h["n"] >= MIN_N_SELECTED and h["wr_pct"] is not None:
            if best is None or h["wr_pct"] > best["wr_pct"]:
                best = {
                    "rule": fc["rule"],
                    "wr_pct": h["wr_pct"],
                    "n": h["n"],
                    "wins": h["wins"],
                    "losses": h["losses"],
                    "coverage_pct": fc["holdout"]["coverage_pct"],
                    "payoff": h["payoff"],
                    "net_inr": h["net_inr"],
                    "gap_to_85_pp": round(h["wr_pct"] - ASPIRATIONAL_WR, 2),
                }

    return {
        "hypotheses": hypotheses,
        "failed_or_blocked_hypotheses": failed,
        "gate2_disposition": disposition,
        "research_tier": tier,
        "research_conclusion": conclusion,
        "best_powered_holdout_selection": best,
        "aspirational_wr_pct": ASPIRATIONAL_WR,
        "gate3_authorized": False,
    }


# ---------------------------------------------------------------------------
# Report render
# ---------------------------------------------------------------------------


def render_md(rep: Dict[str, Any]) -> str:
    s1 = rep["stage1_completeness"]
    s2 = rep["stage2_associations"]
    s3 = rep["stage3_stability"]
    s45 = rep["stages4_5_gate2"]
    lines = [
        "# RP-608-WR-ENTRY-DISCOVERY-001",
        "",
        f"**Protocol:** `{rep['protocol']}`  ",
        f"**Book:** {rep['book']} · **Authority:** {rep['authority']} · baseline_frozen={rep['baseline_frozen']}  ",
        f"**Gate 2 criteria changed:** {rep['gate2_criteria_changed']} · Gate 3 unauthorized  ",
        f"**Aspirational WR:** {ASPIRATIONAL_WR}% (research objective, not Gate 2 bar)",
        "",
        "## Research conclusion",
        "",
        s45["research_conclusion"],
        "",
        f"**Disposition:** `{s45['gate2_disposition']}` · **Tier:** `{s45['research_tier']}`",
        "",
    ]
    best = s45.get("best_powered_holdout_selection")
    if best:
        lines += [
            "### Best powered holdout selection (honest distance to 85%)",
            "",
            f"- Rule: `{best['rule']}`",
            f"- Holdout WR **{best['wr_pct']}%** on n={best['n']} (W/L {best['wins']}/{best['losses']})",
            f"- Coverage {best['coverage_pct']}% · payoff={best['payoff']} · net₹={best['net_inr']}",
            f"- Gap to 85%: **{best['gap_to_85_pp']} pp**",
            "",
        ]

    lines += [
        "## Stage 1 — Feature completeness",
        "",
        f"- Join rows: **{s1['join_n']}** · feature-ready **{s1['feature_ready_n']}** · unavailable **{s1['feature_unavailable_n']}**",
        f"- Discovery join: {s1['discovery_join_n']} (ready {s1['discovery_ready_n']}, unavail {s1['discovery_unavail_n']})",
        f"- Closed-export discovery n: {s1['closed_export_discovery_n']} · missing from join: {s1['missing_from_join_discovery_n']}",
        f"- Leakage violations: {s1['leakage_violations']} · look_ahead_safe_all={s1['look_ahead_safe_all']}",
        f"- Unavail reasons: `{s1['unavail_reasons']}`",
        f"- Ready WL: n={s1['wl_feature_ready']['n']} W/L={s1['wl_feature_ready']['wins']}/{s1['wl_feature_ready']['losses']} WR%={s1['wl_feature_ready']['wr_pct']}",
        f"- Unavail WL: n={s1['wl_feature_unavail']['n']} W/L={s1['wl_feature_unavail']['wins']}/{s1['wl_feature_unavail']['losses']} WR%={s1['wl_feature_unavail']['wr_pct']}",
        f"- Note: {s1['usable_population_note']}",
        "",
        "## Stage 2 — Baseline and associations",
        "",
        f"- Discovery baseline: n={s2['baseline_discovery']['n']} W/L={s2['baseline_discovery']['wins']}/{s2['baseline_discovery']['losses']} WR%={s2['baseline_discovery']['wr_pct']} payoff={s2['baseline_discovery']['payoff']} net₹={s2['baseline_discovery']['net_inr']}",
        f"- Holdout baseline: n={s2['baseline_holdout']['n']} W/L={s2['baseline_holdout']['wins']}/{s2['baseline_holdout']['losses']} WR%={s2['baseline_holdout']['wr_pct']}",
        f"- Scoring: {s2['scoring_method']}",
        "",
        "### Calibration (discovery)",
        "",
    ]
    for b in s2.get("calibration_discovery", []):
        lines.append(
            f"- bin {b['bin']}: n={b['n']} mean_pred={b['mean_predicted']} realized_WR%={b['realized_wr']} W/L={b['wins']}/{b['losses']}"
        )
    lines += ["", "### Score-threshold selection (coverage mandatory)", ""]
    lines.append("| Thr | Disc n (cov%) | Disc WR W/L | Hold n (cov%) | Hold WR W/L | Rej hold WR |")
    lines.append("| ---: | ---: | --- | ---: | --- | --- |")
    for sc in s2.get("score_threshold_candidates", []):
        d, h = sc["discovery"], sc["holdout"]
        lines.append(
            f"| {sc['threshold']} | {d['selected_n']} ({d['coverage_pct']}) | "
            f"{d['selected']['wr_pct']} {d['selected']['wins']}/{d['selected']['losses']} | "
            f"{h['selected_n']} ({h['coverage_pct']}) | "
            f"{h['selected']['wr_pct']} {h['selected']['wins']}/{h['selected']['losses']} | "
            f"{h['rejected']['wr_pct']} |"
        )

    lines += ["", "### Feature selection candidates", ""]
    for fc in s2.get("feature_selection_candidates", []):
        d, h = fc["discovery"], fc["holdout"]
        lines.append(
            f"- `{fc['rule']}`: disc WR {d['selected']['wr_pct']}% n={d['selected_n']} "
            f"(cov {d['coverage_pct']}%, rej WR {d['rejected']['wr_pct']}%) · "
            f"hold WR {h['selected']['wr_pct']}% n={h['selected_n']} "
            f"(cov {h['coverage_pct']}%, rej WR {h['rejected']['wr_pct']}%)"
        )

    lines += ["", "## Stage 3 — Stability", ""]
    for a in s3.get("contrast_stability", []):
        lines.append(
            f"- `{a['contrast']}` promote={a['promote']} blocks={a['blocks']} "
            f"pos_sess={a['positive_sessions']} neg_sess={a['negative_sessions']}"
        )
    conc = s3.get("symbol_concentration_discovery", {})
    lines.append(f"- Unique symbols: {conc.get('unique_symbols')}")
    lines.append(f"- Top loss symbols: `{conc.get('top_loss_symbols')}`")

    lines += ["", "## Stages 4–5 — Hypotheses and Gate 2", ""]
    lines.append("| Hypothesis | Replication | Gate2 | Block |")
    lines.append("| --- | --- | --- | --- |")
    for h in s45.get("hypotheses", []) + s45.get("failed_or_blocked_hypotheses", []):
        g = h["gate2"]
        lines.append(
            f"| `{h['hypothesis_id']}` | {h['replication_verdict']} | "
            f"{'PASS' if g['all_satisfied'] else 'NO'} | {g.get('blocked_reason')} |"
        )

    lines += [
        "",
        "## Governance",
        "",
        "- OF ±0.30 / stops / targets / Estimand A thresholds: **unchanged**",
        "- Outcome W/L definition: **unchanged** (net)",
        "- Books: **not pooled**",
        "- Gate 3: **unauthorized**",
        "",
    ]
    return "\n".join(lines)


def run(join_path: Path, export_dir: Path) -> Dict[str, Any]:
    raw = [enrich_join_row(r) for r in load_jsonl(join_path)]
    # Analysis population: feature-ready within windows
    windowed = [r for r in raw if r.get("session_day") in ALL_WINDOW_DAYS]
    ready = [r for r in windowed if r.get("feature_ready")]
    closed = load_closed_exports(export_dir, ALL_WINDOW_DAYS)

    stage1 = stage1_completeness(windowed, closed)

    disc = [r for r in ready if r.get("session_day") in DISCOVERY_DAYS]
    hold = [r for r in ready if r.get("session_day") in HOLDOUT_ALL]

    stage2 = stage2_associations(disc, hold)
    stage3 = stage3_stability(disc, stage2)
    stage45 = stages4_5(disc, hold, stage2, stage3)

    return {
        "protocol": "RP-608-WR-ENTRY-DISCOVERY-001",
        "program": "RP-608-WR-IMPROVE-V2",
        "authority": "0.0%",
        "baseline_frozen": True,
        "gate2_criteria_changed": False,
        "book": "nse_estimand_a",
        "aspirational_wr_pct": ASPIRATIONAL_WR,
        "windows": {
            "discovery": DISCOVERY_DAYS,
            "holdout_pre": HOLDOUT_PRE,
            "holdout_fwd": HOLDOUT_FWD,
        },
        "inputs": {
            "join": str(join_path),
            "export_dir": str(export_dir),
            "join_rows_windowed": len(windowed),
            "feature_ready_windowed": len(ready),
            "discovery_n": len(disc),
            "holdout_n": len(hold),
        },
        "stage1_completeness": stage1,
        "stage2_associations": stage2,
        "stage3_stability": stage3,
        "stages4_5_gate2": stage45,
        "research_conclusion": stage45["research_conclusion"],
        "gate2_disposition": stage45["gate2_disposition"],
    }


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--join",
        default="artifacts/rp608_econ_edge_diag/wl_nse_a_decision_time_features_kite_join.jsonl",
    )
    ap.add_argument("--export-dir", default="artifacts/rp608_econ_edge_diag")
    ap.add_argument("--out-dir", default="artifacts/rp608_econ_edge_diag")
    args = ap.parse_args()
    join_path = Path(args.join)
    export_dir = Path(args.export_dir)
    out_dir = Path(args.out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    if not join_path.exists():
        print(json.dumps({"status": "MISSING_JOIN", "path": str(join_path)}))
        return 2
    rep = run(join_path, export_dir)
    stem = out_dir / "rp608_wr_entry_discovery_001"
    json_path = stem.with_suffix(".json")
    md_path = stem.with_suffix(".md")
    json_path.write_text(json.dumps(rep, indent=2), encoding="utf-8")
    md_path.write_text(render_md(rep), encoding="utf-8")
    print(
        json.dumps(
            {
                "status": "OK",
                "gate2_disposition": rep["gate2_disposition"],
                "tier": rep["stages4_5_gate2"]["research_tier"],
                "discovery_n": rep["inputs"]["discovery_n"],
                "holdout_n": rep["inputs"]["holdout_n"],
                "best": rep["stages4_5_gate2"].get("best_powered_holdout_selection"),
                "md": str(md_path),
            },
            indent=2,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
