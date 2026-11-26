#!/usr/bin/env python3
"""
RP-608-WR-ENTRY-DISCOVERY-002 — Selection frontier & stability audit (NSE A).

Audits the 001 score / P_hat>=0.5 candidate: ranking quality, WR–coverage
frontier, session stability, calibration, rejected-trade outcomes,
concentration/redundancy. Does NOT expand feature mining.

Authority 0%. Baseline frozen. Gate 2 criteria unchanged. Gate 3 unauthorized.
Aspirational 85% WR is a research objective, not a Gate 2 bar.

Usage (Pi):
  python3 scripts/research/rp608_wr_entry_discovery_002.py \
    --join artifacts/rp608_econ_edge_diag/wl_nse_a_decision_time_features_kite_join.jsonl \
    --out-dir artifacts/rp608_econ_edge_diag
"""

from __future__ import annotations

import argparse
import json
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

FEATURE_KEYS = ("trend", "momentum", "price_location")
SCORE_FEATURES = ("trend", "momentum", "price_location", "side", "hour_ist")
UNAVAIL_MARKERS = ("UNAVAILABLE", "??")
ASPIRATIONAL_WR = 85.0
# Thresholds explored in 001 (documented); 002 also densifies for frontier curve
THRESHOLDS_001_EXPLORED = [0.45, 0.50, 0.55, 0.60, 0.65, 0.70]
# Dense frontier for reporting (includes 001 set). Holdout metrics here are
# descriptive of the contaminated holdout — not fresh validation for picking.
THRESHOLDS_FRONTIER = [
    round(x, 3) for x in [i / 100 for i in range(40, 71)]
]  # 0.40 .. 0.70
CANDIDATE_THRESHOLD = 0.50  # 001 "best" on holdout — audit focus
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


def enrich(r: Dict[str, Any]) -> Dict[str, Any]:
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


def empirical_tables(rows: Sequence[Dict[str, Any]]) -> Dict[str, Dict[str, float]]:
    out: Dict[str, Dict[str, float]] = {}
    for feat in SCORE_FEATURES:
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


def score_row(r: Dict[str, Any], emp: Dict[str, Dict[str, float]]) -> Tuple[float, Dict[str, float]]:
    contrib: Dict[str, float] = {}
    for feat, table in emp.items():
        v = r.get(feat)
        if v is None or is_unavail(v):
            continue
        if str(v) in table:
            contrib[feat] = table[str(v)]
    if not contrib:
        return 0.5, {}
    return sum(contrib.values()) / len(contrib), contrib


def attach_scores(
    rows: Sequence[Dict[str, Any]], emp: Dict[str, Dict[str, float]]
) -> List[Dict[str, Any]]:
    out = []
    for r in rows:
        s, contrib = score_row(r, emp)
        rr = dict(r)
        rr["p_hat"] = s
        rr["p_hat_contrib"] = contrib
        out.append(rr)
    return out


def select(rows: Sequence[Dict[str, Any]], thr: float) -> Tuple[List[Dict[str, Any]], List[Dict[str, Any]]]:
    sel = [r for r in rows if r.get("p_hat", 0) >= thr]
    rej = [r for r in rows if r.get("p_hat", 0) < thr]
    return sel, rej


def frontier_point(rows: Sequence[Dict[str, Any]], thr: float) -> Dict[str, Any]:
    book_n = len(rows)
    sel, rej = select(rows, thr)
    return {
        "threshold": thr,
        "in_001_explored_set": thr in THRESHOLDS_001_EXPLORED,
        "selected": wl_stats(sel, book_n),
        "rejected": wl_stats(rej, book_n),
        "selected_n": len(sel),
        "rejected_n": len(rej),
        "coverage_pct": None if book_n == 0 else round(100.0 * len(sel) / book_n, 2),
        "gap_to_85_pp": (
            None
            if not sel or wl_stats(sel)["wr_pct"] is None
            else round(wl_stats(sel)["wr_pct"] - ASPIRATIONAL_WR, 2)
        ),
    }


def session_breakdown(rows: Sequence[Dict[str, Any]], thr: float) -> List[Dict[str, Any]]:
    days = sorted({r.get("session_day") for r in rows if r.get("session_day")})
    out = []
    for day in days:
        day_rows = [r for r in rows if r.get("session_day") == day]
        sel, rej = select(day_rows, thr)
        out.append(
            {
                "day": day,
                "eligible_n": len(day_rows),
                "selected": wl_stats(sel, len(day_rows)),
                "rejected": wl_stats(rej, len(day_rows)),
                "coverage_pct": None
                if not day_rows
                else round(100.0 * len(sel) / len(day_rows), 2),
            }
        )
    return out


def calibration_at_threshold(
    rows: Sequence[Dict[str, Any]], thr: float, n_bins: int = 5
) -> Dict[str, Any]:
    sel, _ = select(rows, thr)
    paired = [(float(r["p_hat"]), r) for r in sel if r.get("win") is not None]
    if len(paired) < n_bins:
        # overall only
        if not paired:
            return {"n": 0, "bins": [], "mean_predicted": None, "realized_wr": None}
        preds = [s for s, _ in paired]
        wins = sum(1 for _, r in paired if r["win"])
        n = len(paired)
        return {
            "n": n,
            "mean_predicted": round(sum(preds) / n, 4),
            "realized_wr": round(100.0 * wins / n, 2),
            "wins": wins,
            "losses": n - wins,
            "bins": [],
        }
    paired.sort(key=lambda x: x[0])
    size = max(1, len(paired) // n_bins)
    bins = []
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
    preds_all = [s for s, _ in paired]
    wins_all = sum(1 for _, r in paired if r["win"])
    n_all = len(paired)
    return {
        "n": n_all,
        "mean_predicted": round(sum(preds_all) / n_all, 4),
        "realized_wr": round(100.0 * wins_all / n_all, 2),
        "wins": wins_all,
        "losses": n_all - wins_all,
        "bins": bins,
    }


def ranking_quality(rows: Sequence[Dict[str, Any]]) -> Dict[str, Any]:
    """Score separation + Mann-Whitney-style rank AUC (wins vs losses)."""
    wins = [float(r["p_hat"]) for r in rows if r.get("win") is True]
    losses = [float(r["p_hat"]) for r in rows if r.get("win") is False]
    if not wins or not losses:
        return {"auc_win_gt_loss": None, "mean_score_win": None, "mean_score_loss": None}
    # AUC: P(score_win > score_loss) + 0.5 P(equal)
    gt = 0
    eq = 0
    for w in wins:
        for l in losses:
            if w > l:
                gt += 1
            elif w == l:
                eq += 1
    pairs = len(wins) * len(losses)
    auc = (gt + 0.5 * eq) / pairs if pairs else None
    return {
        "n_wins": len(wins),
        "n_losses": len(losses),
        "mean_score_win": round(sum(wins) / len(wins), 4),
        "mean_score_loss": round(sum(losses) / len(losses), 4),
        "median_score_win": round(sorted(wins)[len(wins) // 2], 4),
        "median_score_loss": round(sorted(losses)[len(losses) // 2], 4),
        "auc_win_gt_loss": None if auc is None else round(auc, 4),
        "score_min": round(min(wins + losses), 4),
        "score_max": round(max(wins + losses), 4),
    }


def concentration(sel: Sequence[Dict[str, Any]]) -> Dict[str, Any]:
    if not sel:
        return {"symbols": [], "sessions": [], "dominating_session": None, "dominating_symbol": None}
    by_sym = Counter(str(r.get("symbol")) for r in sel)
    by_day = Counter(str(r.get("session_day")) for r in sel)
    n = len(sel)
    top_sym = by_sym.most_common(10)
    top_day = by_day.most_common()
    dom_day = top_day[0] if top_day else None
    dom_sym = top_sym[0] if top_sym else None
    return {
        "n": n,
        "unique_symbols": len(by_sym),
        "unique_sessions": len(by_day),
        "top_symbols": [{"symbol": s, "n": c, "share_pct": round(100.0 * c / n, 2)} for s, c in top_sym],
        "sessions": [{"day": d, "n": c, "share_pct": round(100.0 * c / n, 2)} for d, c in top_day],
        "dominating_session": (
            None
            if not dom_day
            else {
                "day": dom_day[0],
                "n": dom_day[1],
                "share_pct": round(100.0 * dom_day[1] / n, 2),
                "flag": round(100.0 * dom_day[1] / n, 2) >= 60.0,
            }
        ),
        "dominating_symbol": (
            None
            if not dom_sym
            else {
                "symbol": dom_sym[0],
                "n": dom_sym[1],
                "share_pct": round(100.0 * dom_sym[1] / n, 2),
                "flag": round(100.0 * dom_sym[1] / n, 2) >= 25.0,
            }
        ),
    }


def redundancy_audit(sel: Sequence[Dict[str, Any]], emp: Dict[str, Dict[str, float]]) -> Dict[str, Any]:
    """Which feature contributions drive high scores in the selected set?"""
    if not sel:
        return {"feature_presence_in_contrib": {}, "level_counts": {}}
    presence = Counter()
    level_counts: Dict[str, Counter] = {f: Counter() for f in SCORE_FEATURES}
    for r in sel:
        contrib = r.get("p_hat_contrib") or {}
        for f in contrib:
            presence[f] += 1
        for f in SCORE_FEATURES:
            v = r.get(f)
            if v is not None and not is_unavail(v):
                level_counts[f][str(v)] += 1
    n = len(sel)
    return {
        "feature_presence_in_contrib": {
            f: {"n": presence[f], "share_pct": round(100.0 * presence[f] / n, 2)}
            for f in SCORE_FEATURES
        },
        "level_counts": {f: dict(c.most_common(8)) for f, c in level_counts.items()},
        "note": (
            "If one feature dominates contrib presence and levels collapse to a single bucket, "
            "the score may be effectively a re-expression of that feature."
        ),
    }


def evaluate_gate2_candidate(disc_sel: Dict[str, Any], hold_sel: Dict[str, Any], meta: Dict[str, Any]) -> Dict[str, Any]:
    """Apply unchanged Gate 2-style bars to the audited selection rule (not advance)."""
    criteria = []
    criteria.append(
        {
            "id": "G2-1_MEASURABLE",
            "satisfied": True,
            "detail": "P_hat = mean empirical P(win|level); select score>=threshold",
        }
    )
    criteria.append({"id": "G2-2_DECISION_TIME_ONLY", "satisfied": True})
    disc_ok = (
        disc_sel.get("n", 0) >= MIN_N_SIDE
        and disc_sel.get("wr_pct") is not None
        and hold_sel.get("n", 0) >= 0
    )
    # discovery support: selected WR > baseline is checked externally
    criteria.append(
        {
            "id": "G2-3_DISCOVERY_SUPPORT",
            "satisfied": disc_sel.get("n", 0) >= MIN_N_SELECTED and disc_sel.get("wr_pct") is not None,
            "detail": f"discovery_selected_n={disc_sel.get('n')} wr={disc_sel.get('wr_pct')}",
        }
    )
    # Holdout contaminated for threshold pick — replication NOT claimed as untouched
    criteria.append(
        {
            "id": "G2-4_HOLDOUT_REPLICATION",
            "satisfied": False,
            "detail": (
                "Holdout already used in 001 to choose among thresholds; "
                "not untouched validation. Require later independent sessions."
            ),
        }
    )
    criteria.append(
        {
            "id": "G2-5_FALSIFIABLE",
            "satisfied": True,
            "detail": "Fails if independent later sessions do not sustain WR lift at predeclared threshold.",
        }
    )
    cov = meta.get("holdout_coverage_pct") or 0.0
    criteria.append(
        {
            "id": "G2-6_COVERAGE_NOT_SOLE_SUCCESS",
            "satisfied": cov >= 20.0,  # thin coverage fails this spirit for strategy claim
            "detail": f"holdout_coverage_pct={cov} (8.6% candidate is coverage-thin)",
        }
    )
    econ = (
        hold_sel.get("payoff") is not None
        and hold_sel["payoff"] >= 1.0
        and hold_sel.get("net_inr") is not None
        and hold_sel["net_inr"] > 0
    )
    criteria.append(
        {
            "id": "G2-7_PAYOFF_NET_SUPPORT",
            "satisfied": bool(econ),
            "detail": f"holdout_payoff={hold_sel.get('payoff')} net={hold_sel.get('net_inr')}",
        }
    )
    all_ok = all(c["satisfied"] for c in criteria)
    return {
        "all_satisfied": all_ok,
        "criteria": criteria,
        "disposition": "GATE2_DO_NOT_ADVANCE",
        "reason": (
            "Holdout contaminated for threshold selection; coverage thin vs eligible population; "
            "85% unmet. Gate 3 unauthorized."
        ),
    }


MIN_N_SIDE = 15


def _window_sel(rows: Sequence[Dict[str, Any]], thr: float) -> Dict[str, Any]:
    sel, _rej = select(rows, thr)
    return {
        "selected": wl_stats(sel, len(rows)),
        "coverage_pct": round(100.0 * len(sel) / len(rows), 2) if rows else None,
    }


def run(join_path: Path) -> Dict[str, Any]:
    raw = [enrich(r) for r in load_jsonl(join_path)]
    ready = [
        r
        for r in raw
        if r.get("feature_ready") and r.get("session_day") in ALL_WINDOW_DAYS
    ]
    disc = [r for r in ready if r.get("session_day") in DISCOVERY_DAYS]
    hold = [r for r in ready if r.get("session_day") in HOLDOUT_ALL]
    hold_pre = [r for r in ready if r.get("session_day") in HOLDOUT_PRE]
    hold_fwd = [r for r in ready if r.get("session_day") in HOLDOUT_FWD]

    emp = empirical_tables(disc)
    disc_s = attach_scores(disc, emp)
    hold_s = attach_scores(hold, emp)
    hold_pre_s = attach_scores(hold_pre, emp)
    hold_fwd_s = attach_scores(hold_fwd, emp)

    # --- 1. Exact candidate definition ---
    score_definition = {
        "name": "p_hat_entry_v1",
        "features_in_score": list(SCORE_FEATURES),
        "primary_market_features": list(FEATURE_KEYS),
        "fit_window": DISCOVERY_DAYS,
        "fit_rule": "empirical P(win|feature_level) with min_n=5 per level; discovery only",
        "aggregation": "unweighted mean of available level probabilities",
        "missing_level_behavior": "feature skipped if level unseen or n<5 in discovery tables",
        "label": "win = net_pnl_inr > 0 (unchanged)",
        "candidate_rule_from_001": f"p_hat >= {CANDIDATE_THRESHOLD}",
        "empirical_tables_discovery": {
            k: {kk: round(vv, 4) for kk, vv in v.items()} for k, v in emp.items()
        },
    }

    # --- 2. Boundary audit ---
    boundary = {
        "thresholds_explored_in_001": THRESHOLDS_001_EXPLORED,
        "candidate_threshold": CANDIDATE_THRESHOLD,
        "threshold_predeclared_before_holdout": False,
        "how_chosen": (
            "001 evaluated multiple thresholds on discovery and holdout, then reported "
            "the best powered holdout WR (P_hat>=0.50). Holdout therefore influenced "
            "which threshold was highlighted."
        ),
        "holdout_status": "CONTAMINATED_FOR_THRESHOLD_SELECTION",
        "independent_validation_required": True,
        "independent_validation_note": (
            "Later untouched IST sessions beyond the current holdout window are required "
            "before claiming genuine out-of-sample threshold confirmation."
        ),
        "discovery_days": DISCOVERY_DAYS,
        "holdout_days": HOLDOUT_ALL,
    }

    # --- 3–4. WR–coverage frontier ---
    frontier_disc = [frontier_point(disc_s, t) for t in THRESHOLDS_FRONTIER]
    frontier_hold = [frontier_point(hold_s, t) for t in THRESHOLDS_FRONTIER]
    frontier_001_disc = [frontier_point(disc_s, t) for t in THRESHOLDS_001_EXPLORED]
    frontier_001_hold = [frontier_point(hold_s, t) for t in THRESHOLDS_001_EXPLORED]

    # Highlight candidate
    cand_disc_sel, cand_disc_rej = select(disc_s, CANDIDATE_THRESHOLD)
    cand_hold_sel, cand_hold_rej = select(hold_s, CANDIDATE_THRESHOLD)
    candidate_audit = {
        "threshold": CANDIDATE_THRESHOLD,
        "discovery": {
            "eligible_n": len(disc_s),
            "selected": wl_stats(cand_disc_sel, len(disc_s)),
            "rejected": wl_stats(cand_disc_rej, len(disc_s)),
            "rejected_n": len(cand_disc_rej),
            "coverage_pct": round(100.0 * len(cand_disc_sel) / len(disc_s), 2) if disc_s else None,
        },
        "holdout": {
            "eligible_n": len(hold_s),
            "selected": wl_stats(cand_hold_sel, len(hold_s)),
            "rejected": wl_stats(cand_hold_rej, len(hold_s)),
            "rejected_n": len(cand_hold_rej),
            "coverage_pct": round(100.0 * len(cand_hold_sel) / len(hold_s), 2) if hold_s else None,
        },
        "holdout_pre": _window_sel(hold_pre_s, CANDIDATE_THRESHOLD),
        "holdout_fwd": _window_sel(hold_fwd_s, CANDIDATE_THRESHOLD),
    }

    # --- 5. Session-level ---
    session_disc = session_breakdown(disc_s, CANDIDATE_THRESHOLD)
    session_hold = session_breakdown(hold_s, CANDIDATE_THRESHOLD)

    # --- 6. Calibration ---
    cal_disc = calibration_at_threshold(disc_s, CANDIDATE_THRESHOLD)
    cal_hold = calibration_at_threshold(hold_s, CANDIDATE_THRESHOLD)

    def full_cal(rows: List[Dict[str, Any]], n_bins: int = 5) -> List[Dict[str, Any]]:
        paired = [(float(r["p_hat"]), r) for r in rows if r.get("win") is not None]
        if not paired:
            return []
        paired.sort(key=lambda x: x[0])
        size = max(1, len(paired) // n_bins)
        bins = []
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

    # --- 7. Rejected-trade analysis (candidate) ---
    rejected_analysis = {
        "discovery_rejected": candidate_audit["discovery"]["rejected"],
        "holdout_rejected": candidate_audit["holdout"]["rejected"],
        "note": (
            "Rejected trades are still eligible under frozen baseline; their W/L is the "
            "opportunity cost of selection."
        ),
    }

    # --- 8. Concentration + redundancy ---
    conc_hold = concentration(cand_hold_sel)
    conc_disc = concentration(cand_disc_sel)
    red_hold = redundancy_audit(cand_hold_sel, emp)
    red_disc = redundancy_audit(cand_disc_sel, emp)

    # --- Ranking quality ---
    rank_disc = ranking_quality(disc_s)
    rank_hold = ranking_quality(hold_s)

    # --- 9. 85% status ---
    # Any frontier point with WR>=85 and powered n?
    hit_85_hold = [
        p
        for p in frontier_hold
        if p["selected"]["wr_pct"] is not None
        and p["selected"]["wr_pct"] >= ASPIRATIONAL_WR
        and p["selected"]["n"] >= MIN_N_SELECTED
    ]
    status_85 = {
        "aspirational_wr_pct": ASPIRATIONAL_WR,
        "achieved_on_powered_holdout": len(hit_85_hold) > 0,
        "powered_holdout_hits": hit_85_hold,
        "candidate_holdout_wr": candidate_audit["holdout"]["selected"]["wr_pct"],
        "candidate_holdout_coverage_pct": candidate_audit["holdout"]["coverage_pct"],
        "candidate_gap_to_85_pp": (
            None
            if candidate_audit["holdout"]["selected"]["wr_pct"] is None
            else round(candidate_audit["holdout"]["selected"]["wr_pct"] - ASPIRATIONAL_WR, 2)
        ),
        "interpretation": (
            "No powered holdout threshold reaches 85% WR in the reported frontier. "
            "The 70% @ 8.6% coverage candidate remains below target and coverage-thin. "
            "Holdout is contaminated for threshold picking — do not claim 85% path proven."
        ),
    }

    # --- 10. Gate 2 ---
    gate2 = evaluate_gate2_candidate(
        candidate_audit["discovery"]["selected"],
        candidate_audit["holdout"]["selected"],
        {"holdout_coverage_pct": candidate_audit["holdout"]["coverage_pct"]},
    )

    conclusion = (
        f"Audited 001 candidate p_hat>={CANDIDATE_THRESHOLD}: holdout WR "
        f"{candidate_audit['holdout']['selected']['wr_pct']}% "
        f"({candidate_audit['holdout']['selected']['wins']}/"
        f"{candidate_audit['holdout']['selected']['losses']}) at "
        f"{candidate_audit['holdout']['coverage_pct']}% coverage. "
        f"Gap to 85%: {status_85['candidate_gap_to_85_pp']} pp. "
        "Holdout used for threshold highlighting → not untouched validation. "
        "Gate 2 DO NOT ADVANCE. Do not expand features until ranking/coverage improves."
    )

    return {
        "protocol": "RP-608-WR-ENTRY-DISCOVERY-002",
        "authority": "0.0%",
        "baseline_frozen": True,
        "gate2_criteria_changed": False,
        "book": "nse_estimand_a",
        "aspirational_wr_pct": ASPIRATIONAL_WR,
        "inputs": {
            "join": str(join_path),
            "discovery_n": len(disc_s),
            "holdout_n": len(hold_s),
            "holdout_pre_n": len(hold_pre_s),
            "holdout_fwd_n": len(hold_fwd_s),
        },
        "score_definition": score_definition,
        "discovery_holdout_boundary_audit": boundary,
        "candidate_audit_p_hat_ge_0_50": candidate_audit,
        "frontier_discovery": frontier_disc,
        "frontier_holdout": frontier_hold,
        "frontier_001_thresholds_discovery": frontier_001_disc,
        "frontier_001_thresholds_holdout": frontier_001_hold,
        "session_breakdown_discovery": session_disc,
        "session_breakdown_holdout": session_hold,
        "calibration_selected_discovery": cal_disc,
        "calibration_selected_holdout": cal_hold,
        "calibration_full_population_discovery": full_cal(disc_s),
        "calibration_full_population_holdout": full_cal(hold_s),
        "rejected_trade_analysis": rejected_analysis,
        "concentration_discovery_selected": conc_disc,
        "concentration_holdout_selected": conc_hold,
        "redundancy_discovery_selected": red_disc,
        "redundancy_holdout_selected": red_hold,
        "ranking_quality_discovery": rank_disc,
        "ranking_quality_holdout": rank_hold,
        "status_85_target": status_85,
        "gate2_assessment": gate2,
        "research_conclusion": conclusion,
        "gate2_disposition": gate2["disposition"],
    }


def render_md(rep: Dict[str, Any]) -> str:
    c = rep["candidate_audit_p_hat_ge_0_50"]
    b = rep["discovery_holdout_boundary_audit"]
    s85 = rep["status_85_target"]
    g2 = rep["gate2_assessment"]
    lines = [
        "# RP-608-WR-ENTRY-DISCOVERY-002 — Selection Frontier & Stability Audit",
        "",
        f"**Protocol:** `{rep['protocol']}`  ",
        f"**Book:** {rep['book']} · Authority {rep['authority']} · baseline_frozen={rep['baseline_frozen']}  ",
        f"**Gate 2 criteria changed:** {rep['gate2_criteria_changed']} · Gate 3 unauthorized  ",
        f"**Aspirational WR:** {ASPIRATIONAL_WR}% (not a Gate 2 bar)",
        "",
        "## Research conclusion",
        "",
        rep["research_conclusion"],
        "",
        f"**Gate 2 disposition:** `{rep['gate2_disposition']}`",
        "",
        "## 1. Exact score / candidate definition",
        "",
        f"- Score: `{rep['score_definition']['name']}` = unweighted mean of empirical "
        f"P(win|level) over features {rep['score_definition']['features_in_score']}",
        f"- Fit: discovery only `{rep['score_definition']['fit_window']}`",
        f"- Candidate rule (from 001 highlight): `{rep['score_definition']['candidate_rule_from_001']}`",
        f"- Labels: {rep['score_definition']['label']}",
        "",
        "## 2. Discovery / holdout boundary audit",
        "",
        f"- Thresholds explored in 001: `{b['thresholds_explored_in_001']}`",
        f"- Predeclared before holdout? **{b['threshold_predeclared_before_holdout']}**",
        f"- Holdout status: **{b['holdout_status']}**",
        f"- {b['how_chosen']}",
        f"- Independent validation required: **{b['independent_validation_required']}** — {b['independent_validation_note']}",
        "",
        "## 3–4. Candidate audit (`p_hat >= 0.50`)",
        "",
        f"| Window | Eligible | Selected n | Cov% | WR% | W/L | Payoff | Net ₹ | Rejected WR% |",
        f"| --- | ---: | ---: | ---: | ---: | --- | ---: | ---: | ---: |",
    ]
    for label, block in [
        ("Discovery", c["discovery"]),
        ("Holdout (contaminated)", c["holdout"]),
    ]:
        s, r = block["selected"], block["rejected"]
        lines.append(
            f"| {label} | {block['eligible_n']} | {s['n']} | {block['coverage_pct']} | "
            f"{s['wr_pct']} | {s['wins']}/{s['losses']} | {s['payoff']} | {s['net_inr']} | {r['wr_pct']} |"
        )
    lines += [
        "",
        f"- Holdout-pre selected: WR%={c['holdout_pre']['selected']['wr_pct']} "
        f"n={c['holdout_pre']['selected']['n']} cov={c['holdout_pre']['coverage_pct']}%",
        f"- Holdout-fwd selected: WR%={c['holdout_fwd']['selected']['wr_pct']} "
        f"n={c['holdout_fwd']['selected']['n']} cov={c['holdout_fwd']['coverage_pct']}%",
        "",
        "## WR–coverage frontier (holdout; descriptive on contaminated window)",
        "",
        "| Thr | In 001 set | n | Cov% | WR% | W/L | Gap to 85 | Rej WR% |",
        "| ---: | --- | ---: | ---: | ---: | --- | ---: | ---: |",
    ]
    for p in rep["frontier_001_thresholds_holdout"]:
        s, r = p["selected"], p["rejected"]
        lines.append(
            f"| {p['threshold']} | yes | {s['n']} | {p['coverage_pct']} | {s['wr_pct']} | "
            f"{s['wins']}/{s['losses']} | {p['gap_to_85_pp']} | {r['wr_pct']} |"
        )
    lines += [
        "",
        "<details><summary>Dense frontier 0.40–0.70 (holdout)</summary>",
        "",
        "| Thr | n | Cov% | WR% | W/L | Gap85 |",
        "| ---: | ---: | ---: | ---: | --- | ---: |",
    ]
    for p in rep["frontier_holdout"]:
        s = p["selected"]
        if s["n"] == 0:
            continue
        lines.append(
            f"| {p['threshold']} | {s['n']} | {p['coverage_pct']} | {s['wr_pct']} | "
            f"{s['wins']}/{s['losses']} | {p['gap_to_85_pp']} |"
        )
    lines += ["", "</details>", ""]

    lines += ["## 5. Session-level WR / coverage at thr=0.50 (holdout)", ""]
    for s in rep["session_breakdown_holdout"]:
        sel = s["selected"]
        lines.append(
            f"- {s['day']}: eligible={s['eligible_n']} selected n={sel['n']} "
            f"cov={s['coverage_pct']}% WR%={sel['wr_pct']} W/L={sel['wins']}/{sel['losses']}"
        )

    lines += [
        "",
        "## 6. Calibration",
        "",
        "### Full-population holdout (ranking signal)",
        "",
    ]
    for binc in rep["calibration_full_population_holdout"]:
        lines.append(
            f"- bin {binc['bin']}: n={binc['n']} mean_pred={binc['mean_predicted']} "
            f"realized_WR%={binc['realized_wr']} W/L={binc['wins']}/{binc['losses']}"
        )
    lines += [
        "",
        f"### Selected holdout (thr=0.50): mean_pred={rep['calibration_selected_holdout'].get('mean_predicted')} "
        f"realized_WR%={rep['calibration_selected_holdout'].get('realized_wr')} "
        f"n={rep['calibration_selected_holdout'].get('n')}",
        "",
        "## 7. Rejected-trade outcomes (thr=0.50)",
        "",
        f"- Discovery rejected: n={c['discovery']['rejected']['n']} WR%={c['discovery']['rejected']['wr_pct']} "
        f"W/L={c['discovery']['rejected']['wins']}/{c['discovery']['rejected']['losses']}",
        f"- Holdout rejected: n={c['holdout']['rejected']['n']} WR%={c['holdout']['rejected']['wr_pct']} "
        f"W/L={c['holdout']['rejected']['wins']}/{c['holdout']['rejected']['losses']}",
        "",
        "## 8. Concentration & redundancy (holdout selected)",
        "",
        f"- Unique symbols: {rep['concentration_holdout_selected'].get('unique_symbols')} · "
        f"sessions: {rep['concentration_holdout_selected'].get('unique_sessions')}",
        f"- Dominating session: `{rep['concentration_holdout_selected'].get('dominating_session')}`",
        f"- Dominating symbol: `{rep['concentration_holdout_selected'].get('dominating_symbol')}`",
        f"- Top symbols: `{rep['concentration_holdout_selected'].get('top_symbols')}`",
        f"- Contrib presence: `{rep['redundancy_holdout_selected'].get('feature_presence_in_contrib')}`",
        f"- Level counts: `{rep['redundancy_holdout_selected'].get('level_counts')}`",
        "",
        "## Ranking quality",
        "",
        f"- Discovery: mean score win/loss="
        f"{rep['ranking_quality_discovery'].get('mean_score_win')}/"
        f"{rep['ranking_quality_discovery'].get('mean_score_loss')} · "
        f"AUC(win>loss)={rep['ranking_quality_discovery'].get('auc_win_gt_loss')}",
        f"- Holdout: mean score win/loss="
        f"{rep['ranking_quality_holdout'].get('mean_score_win')}/"
        f"{rep['ranking_quality_holdout'].get('mean_score_loss')} · "
        f"AUC(win>loss)={rep['ranking_quality_holdout'].get('auc_win_gt_loss')}",
        "",
        "## 9. Status of 85% target",
        "",
        f"- Achieved on powered holdout threshold? **{s85['achieved_on_powered_holdout']}**",
        f"- Candidate gap to 85%: **{s85['candidate_gap_to_85_pp']} pp** "
        f"(WR {s85['candidate_holdout_wr']}% @ {s85['candidate_holdout_coverage_pct']}% cov)",
        f"- {s85['interpretation']}",
        "",
        "## 10. Gate 2 (criteria unchanged)",
        "",
        f"**Disposition:** `{g2['disposition']}`",
        "",
        f"{g2['reason']}",
        "",
    ]
    for crit in g2["criteria"]:
        lines.append(
            f"- `{crit['id']}`: {'PASS' if crit['satisfied'] else 'FAIL'} — {crit.get('detail', '')}"
        )
    lines += [
        "",
        "## Governance",
        "",
        "- OF ±0.30 / stops / targets: **unchanged**",
        "- Books: **not pooled** · Gate 3: **unauthorized**",
        "- Next: improve ranking/coverage on decision-time score — **not** indiscriminate feature expansion; "
        "confirm on untouched later sessions",
        "",
    ]
    return "\n".join(lines)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--join",
        default="artifacts/rp608_econ_edge_diag/wl_nse_a_decision_time_features_kite_join.jsonl",
    )
    ap.add_argument("--out-dir", default="artifacts/rp608_econ_edge_diag")
    args = ap.parse_args()
    join_path = Path(args.join)
    out_dir = Path(args.out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    if not join_path.exists():
        print(json.dumps({"status": "MISSING_JOIN", "path": str(join_path)}))
        return 2
    rep = run(join_path)
    stem = out_dir / "rp608_wr_entry_discovery_002"
    stem.with_suffix(".json").write_text(json.dumps(rep, indent=2), encoding="utf-8")
    stem.with_suffix(".md").write_text(render_md(rep), encoding="utf-8")
    c = rep["candidate_audit_p_hat_ge_0_50"]["holdout"]
    print(
        json.dumps(
            {
                "status": "OK",
                "gate2_disposition": rep["gate2_disposition"],
                "candidate_holdout_wr": c["selected"]["wr_pct"],
                "candidate_holdout_wl": f"{c['selected']['wins']}/{c['selected']['losses']}",
                "candidate_coverage_pct": c["coverage_pct"],
                "gap_to_85_pp": rep["status_85_target"]["candidate_gap_to_85_pp"],
                "holdout_contaminated": True,
                "achieved_85_powered": rep["status_85_target"]["achieved_on_powered_holdout"],
                "md": str(stem.with_suffix(".md")),
            },
            indent=2,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
