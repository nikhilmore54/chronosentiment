#!/usr/bin/env python3
"""
RP-608-WR-RANKING-001 — Ranking & coverage improvement + Oct 28 session study.

Frozen protocol: discovery-only p_hat; predeclared coverage bands (no holdout
threshold optimization); Oct 28 composition / within-session analysis.

Authority 0%. Baseline frozen. Gate 2 CLOSED. Gate 3 unauthorized.

Usage (Pi):
  python3 scripts/research/rp608_wr_ranking_001.py \
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

FOCUS_STRONG = "2026-10-28"
FOCUS_WEAK = ["2026-10-30", "2026-11-02"]

FEATURE_KEYS = ("trend", "momentum", "price_location")
SCORE_FEATURES = ("trend", "momentum", "price_location", "side", "hour_ist")
UNAVAIL_MARKERS = ("UNAVAILABLE", "??")
ASPIRATIONAL_WR = 85.0

# Predeclared coverage bands — NOT optimized on holdout
COVERAGE_BANDS = [
    ("ALL", None),
    ("TOP50", 0.50),
    ("TOP25", 0.25),
    ("TOP10", 0.10),
    ("SLICE_MOM_LOC", "slice"),
]


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
    r["mom_loc_slice"] = (
        r.get("momentum") == "MOM_UP_>10bps" and r.get("price_location") == "LOC_MID_THIRD"
    )
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
        "gap_to_85_pp": None if n == 0 else round(100.0 * wins / n - ASPIRATIONAL_WR, 2),
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


def score_row(r: Dict[str, Any], emp: Dict[str, Dict[str, float]]) -> float:
    probs = []
    for feat, table in emp.items():
        v = r.get(feat)
        if v is None or is_unavail(v):
            continue
        if str(v) in table:
            probs.append(table[str(v)])
    return 0.5 if not probs else sum(probs) / len(probs)


def attach_scores(rows: Sequence[Dict[str, Any]], emp: Dict[str, Dict[str, float]]) -> List[Dict[str, Any]]:
    out = []
    for r in rows:
        rr = dict(r)
        rr["p_hat"] = score_row(r, emp)
        out.append(rr)
    return out


def auc_win_gt_loss(rows: Sequence[Dict[str, Any]]) -> Optional[float]:
    wins = [float(r["p_hat"]) for r in rows if r.get("win") is True]
    losses = [float(r["p_hat"]) for r in rows if r.get("win") is False]
    if not wins or not losses:
        return None
    gt = eq = 0
    for w in wins:
        for l in losses:
            if w > l:
                gt += 1
            elif w == l:
                eq += 1
    return round((gt + 0.5 * eq) / (len(wins) * len(losses)), 4)


def ranking_summary(rows: Sequence[Dict[str, Any]]) -> Dict[str, Any]:
    wins = [float(r["p_hat"]) for r in rows if r.get("win") is True]
    losses = [float(r["p_hat"]) for r in rows if r.get("win") is False]
    return {
        "n": len(rows),
        "n_wins": len(wins),
        "n_losses": len(losses),
        "auc": auc_win_gt_loss(rows),
        "mean_score_win": None if not wins else round(sum(wins) / len(wins), 4),
        "mean_score_loss": None if not losses else round(sum(losses) / len(losses), 4),
        "score_min": None if not rows else round(min(float(r["p_hat"]) for r in rows), 4),
        "score_max": None if not rows else round(max(float(r["p_hat"]) for r in rows), 4),
        "baseline_wl": wl_stats(rows),
    }


def select_band(rows: Sequence[Dict[str, Any]], band: str, frac: Any) -> List[Dict[str, Any]]:
    if band == "ALL" or frac is None:
        return list(rows)
    if frac == "slice":
        return [r for r in rows if r.get("mom_loc_slice")]
    # top fraction by p_hat
    ranked = sorted(rows, key=lambda r: float(r.get("p_hat", 0)), reverse=True)
    k = max(1, int(round(len(ranked) * float(frac))))
    return ranked[:k]


def band_report(rows: Sequence[Dict[str, Any]], label: str) -> List[Dict[str, Any]]:
    out = []
    book_n = len(rows)
    for band, frac in COVERAGE_BANDS:
        sel = select_band(rows, band, frac)
        s = wl_stats(sel, book_n)
        out.append(
            {
                "window": label,
                "band": band,
                "selected_n": s["n"],
                "coverage_pct": s["coverage_pct"],
                "wr_pct": s["wr_pct"],
                "wins": s["wins"],
                "losses": s["losses"],
                "payoff": s["payoff"],
                "net_inr": s["net_inr"],
                "gap_to_85_pp": s["gap_to_85_pp"],
                "dominating_session_share_pct": _dom_session_share(sel),
            }
        )
    return out


def _dom_session_share(rows: Sequence[Dict[str, Any]]) -> Optional[float]:
    if not rows:
        return None
    c = Counter(str(r.get("session_day")) for r in rows)
    return round(100.0 * c.most_common(1)[0][1] / len(rows), 2)


def feature_mix(rows: Sequence[Dict[str, Any]]) -> Dict[str, Any]:
    n = len(rows)
    if n == 0:
        return {"n": 0}
    mixes = {}
    for feat in list(FEATURE_KEYS) + ["side", "hour_ist"]:
        cnt = Counter(str(r.get(feat)) for r in rows)
        mixes[feat] = {
            k: {"n": v, "pct": round(100.0 * v / n, 2)} for k, v in cnt.most_common()
        }
    slice_n = sum(1 for r in rows if r.get("mom_loc_slice"))
    return {
        "n": n,
        "wl": wl_stats(rows),
        "mom_up_loc_mid": {
            "n": slice_n,
            "pct": round(100.0 * slice_n / n, 2),
            "wl": wl_stats([r for r in rows if r.get("mom_loc_slice")]),
            "complement_wl": wl_stats([r for r in rows if not r.get("mom_loc_slice")]),
        },
        "feature_levels": mixes,
    }


def within_session_win_loss_contrast(rows: Sequence[Dict[str, Any]]) -> Dict[str, Any]:
    wins = [r for r in rows if r.get("win") is True]
    losses = [r for r in rows if r.get("win") is False]
    contrasts = {}
    for feat in list(FEATURE_KEYS) + ["side", "hour_ist"]:
        w_cnt = Counter(str(r.get(feat)) for r in wins)
        l_cnt = Counter(str(r.get(feat)) for r in losses)
        levels = sorted(set(w_cnt) | set(l_cnt))
        contrasts[feat] = [
            {
                "level": lv,
                "win_n": w_cnt.get(lv, 0),
                "win_pct": round(100.0 * w_cnt.get(lv, 0) / len(wins), 2) if wins else None,
                "loss_n": l_cnt.get(lv, 0),
                "loss_pct": round(100.0 * l_cnt.get(lv, 0) / len(losses), 2) if losses else None,
            }
            for lv in levels
        ]
    return {
        "n_wins": len(wins),
        "n_losses": len(losses),
        "mom_loc_among_wins_pct": round(
            100.0 * sum(1 for r in wins if r.get("mom_loc_slice")) / len(wins), 2
        )
        if wins
        else None,
        "mom_loc_among_losses_pct": round(
            100.0 * sum(1 for r in losses if r.get("mom_loc_slice")) / len(losses), 2
        )
        if losses
        else None,
        "feature_contrasts": contrasts,
        "ranking_within_session": ranking_summary(rows),
    }


def session_ranking_table(rows: Sequence[Dict[str, Any]]) -> List[Dict[str, Any]]:
    days = sorted({r.get("session_day") for r in rows if r.get("session_day")})
    out = []
    for day in days:
        day_rows = [r for r in rows if r.get("session_day") == day]
        mix = feature_mix(day_rows)
        out.append(
            {
                "day": day,
                "n": len(day_rows),
                "wl": wl_stats(day_rows),
                "auc": auc_win_gt_loss(day_rows) if len(day_rows) >= 20 else None,
                "auc_note": None
                if len(day_rows) >= 20
                else "n<20 — AUC not emphasized",
                "mom_loc_pct": mix.get("mom_up_loc_mid", {}).get("pct"),
                "mean_p_hat": round(sum(float(r["p_hat"]) for r in day_rows) / len(day_rows), 4)
                if day_rows
                else None,
            }
        )
    return out


def genuine_improvement_check(
    disc_bands: List[Dict[str, Any]],
    hold_bands: List[Dict[str, Any]],
    fwd_bands: List[Dict[str, Any]],
    rank_disc: Dict[str, Any],
    rank_hold: Dict[str, Any],
    rank_fwd: Dict[str, Any],
) -> Dict[str, Any]:
    """Apply frozen §F criteria — informational; does not open Gate 2."""
    checks = []

    auc_disc_ok = (rank_disc.get("auc") or 0) > 0.55
    auc_hold = rank_hold.get("auc")
    auc_fwd = rank_fwd.get("auc")
    # Prefer forward if powered; else note holdout collapse
    auc_transfer_ok = False
    transfer_detail = ""
    if rank_fwd.get("n", 0) >= 50 and auc_fwd is not None:
        auc_transfer_ok = auc_fwd > 0.55
        transfer_detail = f"forward_auc={auc_fwd}"
    elif auc_hold is not None:
        auc_transfer_ok = auc_hold > 0.55
        transfer_detail = f"holdout_auc={auc_hold} (contaminated window; not prospective)"
    checks.append(
        {
            "id": "F1_AUC",
            "satisfied": auc_disc_ok and auc_transfer_ok,
            "detail": f"discovery_auc={rank_disc.get('auc')} · {transfer_detail}",
        }
    )

    def band(rows_bands, name):
        return next((b for b in rows_bands if b["band"] == name), None)

    # Predeclared band TOP25 (>=25% by construction)
    d25, h25, f25 = band(disc_bands, "TOP25"), band(hold_bands, "TOP25"), band(fwd_bands, "TOP25")
    all_d, all_h = band(disc_bands, "ALL"), band(hold_bands, "ALL")
    lift_disc = (
        d25
        and all_d
        and d25["wr_pct"] is not None
        and all_d["wr_pct"] is not None
        and d25["wr_pct"] > all_d["wr_pct"]
    )
    # Forward preferred for lift confirmation
    lift_fwd = False
    if f25 and band(fwd_bands, "ALL") and f25["wr_pct"] is not None:
        all_f = band(fwd_bands, "ALL")
        if all_f and all_f["wr_pct"] is not None and f25.get("selected_n", 0) >= 20:
            lift_fwd = f25["wr_pct"] > all_f["wr_pct"]
    checks.append(
        {
            "id": "F2_PREDECLARED_BAND_LIFT",
            "satisfied": bool(lift_disc and lift_fwd),
            "detail": (
                f"TOP25 disc WR {None if not d25 else d25['wr_pct']} vs ALL {None if not all_d else all_d['wr_pct']}; "
                f"fwd lift={lift_fwd} (requires powered forward window)"
            ),
        }
    )

    dom_ok = d25 is not None and (d25.get("dominating_session_share_pct") or 0) < 60
    checks.append(
        {
            "id": "F3_NOT_SINGLE_SESSION",
            "satisfied": bool(dom_ok),
            "detail": f"TOP25 discovery dominating_session_share_pct={None if not d25 else d25.get('dominating_session_share_pct')}",
        }
    )

    econ_ok = (
        d25 is not None
        and d25.get("payoff") is not None
        and d25["payoff"] >= 1.0
        and d25.get("net_inr") is not None
        and d25["net_inr"] > 0
    )
    checks.append(
        {
            "id": "F4_PAYOFF_NET",
            "satisfied": bool(econ_ok),
            "detail": f"TOP25 disc payoff={None if not d25 else d25.get('payoff')} net={None if not d25 else d25.get('net_inr')}",
        }
    )

    checks.append(
        {
            "id": "F5_OUTCOME_DEF_UNCHANGED",
            "satisfied": True,
            "detail": "net W/L labels preserved; no OF/stop/target change",
        }
    )

    all_ok = all(c["satisfied"] for c in checks)
    return {
        "genuine_ranking_improvement": all_ok,
        "checks": checks,
        "note": (
            "Failing these criteria means ranking improvement is not demonstrated. "
            "Gate 2 remains CLOSED regardless."
        ),
    }


def run(join_path: Path) -> Dict[str, Any]:
    raw = [enrich(r) for r in load_jsonl(join_path)]
    ready = [
        r for r in raw if r.get("feature_ready") and r.get("session_day") in ALL_WINDOW_DAYS
    ]
    disc = [r for r in ready if r.get("session_day") in DISCOVERY_DAYS]
    hold = [r for r in ready if r.get("session_day") in HOLDOUT_ALL]
    fwd = [r for r in ready if r.get("session_day") in HOLDOUT_FWD]
    # Contaminated holdout excluding strong session for descriptive contrast
    hold_ex_oct28 = [r for r in hold if r.get("session_day") != FOCUS_STRONG]

    emp = empirical_tables(disc)
    disc_s = attach_scores(disc, emp)
    hold_s = attach_scores(hold, emp)
    fwd_s = attach_scores(fwd, emp)
    hold_ex_s = attach_scores(hold_ex_oct28, emp)

    oct28 = [r for r in hold_s if r.get("session_day") == FOCUS_STRONG]
    weak_sessions = {
        d: [r for r in hold_s if r.get("session_day") == d] for d in FOCUS_WEAK
    }

    # Chronology / untouched boundary
    # Later than contaminated holdout max day?
    max_hold = max(HOLDOUT_ALL)
    later_days = sorted(
        {
            r.get("session_day")
            for r in ready
            if r.get("session_day") and r.get("session_day") > max_hold
        }
    )
    boundary = {
        "discovery_fit_days": DISCOVERY_DAYS,
        "contaminated_holdout_days": HOLDOUT_ALL,
        "forward_in_join_days": HOLDOUT_FWD,
        "untouched_later_sessions_in_join": later_days,
        "untouched_later_available": len(later_days) > 0,
        "chronology_warning": (
            "Discovery fit (Nov 3–5) is AFTER Oct 28. Discovery-fit p_hat applied to Oct 28 "
            "is a transfer diagnostic only — not prospective prediction of Oct 28."
        ),
        "prospective_confirmation_status": (
            "NO_LATER_UNTOUCHED_SESSIONS_IN_JOIN"
            if not later_days
            else "LATER_SESSIONS_PRESENT"
        ),
    }

    # 002 candidate slice (thr=0.50) — descriptive reference only; NOT re-optimized here
    THR_002_REF = 0.50

    def slice_002(rows: Sequence[Dict[str, Any]]) -> List[Dict[str, Any]]:
        return [r for r in rows if float(r.get("p_hat", 0)) >= THR_002_REF]

    def session_contrast_block(rows: Sequence[Dict[str, Any]]) -> Dict[str, Any]:
        mix = feature_mix(rows)
        sel = slice_002(rows)
        sel_mix = feature_mix(sel)
        return {
            "full_session": {
                "wl": wl_stats(rows),
                "composition": mix,
            },
            "rp608_002_ref_slice_p_hat_ge_0_50": {
                "note": (
                    "Descriptive replay of ENTRY-DISCOVERY-002 thr=0.50 candidate. "
                    "Not a new threshold selection."
                ),
                "wl": wl_stats(sel, book_n=len(rows)),
                "mom_loc_share_pct": sel_mix.get("mom_up_loc_mid", {}).get("pct"),
                "all_mom_loc": (
                    len(sel) > 0
                    and all(r.get("mom_loc_slice") for r in sel)
                ),
            },
        }

    oct28_full = session_contrast_block(oct28)
    oct28_sel = slice_002(oct28)
    weak_blocks = {d: session_contrast_block(rs) for d, rs in weak_sessions.items()}

    # Oct 28 investigation — separate full-session vs 002 selected-slice (83.8%)
    oct28_study = {
        "session": FOCUS_STRONG,
        "distinction": (
            "83.8% (31/37) is the ENTRY-DISCOVERY-002 selected slice on Oct 28 "
            "(p_hat≥0.50), not the full Oct 28 eligible population."
        ),
        "full_session": oct28_full["full_session"],
        "wl": oct28_full["full_session"]["wl"],  # backward-compat alias = full session
        "gap_to_85_pp_full_session": None
        if not oct28
        else round(oct28_full["full_session"]["wl"]["wr_pct"] - ASPIRATIONAL_WR, 2),
        "gap_to_85_pp": None
        if not oct28_sel
        else round(wl_stats(oct28_sel)["wr_pct"] - ASPIRATIONAL_WR, 2),
        "rp608_002_selected_on_oct28": {
            **oct28_full["rp608_002_ref_slice_p_hat_ge_0_50"],
            "gap_to_85_pp": None
            if not oct28_sel
            else round(wl_stats(oct28_sel)["wr_pct"] - ASPIRATIONAL_WR, 2),
        },
        "composition": oct28_full["full_session"]["composition"],
        "within_session_win_vs_loss": within_session_win_loss_contrast(oct28),
        "within_002_slice_win_vs_loss": within_session_win_loss_contrast(oct28_sel),
        "vs_weak_sessions": weak_blocks,
        "vs_discovery_aggregate": session_contrast_block(disc_s),
        "interpretation": {
            "full_session_near_85": False,
            "selected_slice_near_85": bool(
                oct28_sel and (wl_stats(oct28_sel)["wr_pct"] or 0) >= 80
            ),
            "success_concentrated_in_mom_loc": bool(
                oct28_full["rp608_002_ref_slice_p_hat_ge_0_50"]["all_mom_loc"]
            ),
            "within_session_ranking_weak": (
                (oct28_full["full_session"]["wl"]["n"] >= 20)
                and (auc_win_gt_loss(oct28) or 0) < 0.55
            ),
            "repeatable_decision_time_pattern": "NOT_ESTABLISHED",
            "note": (
                "Oct 28 shows a near-85% selected-slice outcome under the contaminated "
                "002 threshold. Full-session WR is far below 85%. Within-session AUC and "
                "weaker sessions indicate the score has not shown reliable identification "
                "of Oct-28-like conditions prospectively."
            ),
        },
        "questions": {
            "q1_feature_mix_differs": "See full_session.composition vs weak sessions.",
            "q2_success_in_mom_loc": oct28_full["full_session"]["composition"][
                "mom_up_loc_mid"
            ],
            "q2b_002_slice_all_mom_loc": oct28_full["rp608_002_ref_slice_p_hat_ge_0_50"][
                "all_mom_loc"
            ],
            "q3_within_session_entry_diff": "See within_session_win_vs_loss.",
            "q4_discovery_score_ranks_within_oct28": ranking_summary(oct28),
            "q4b_ranks_within_002_slice": ranking_summary(oct28_sel)
            if oct28_sel
            else None,
            "q5_later_untouched": boundary["prospective_confirmation_status"],
        },
    }

    # Ranking diagnostics
    rank_disc = ranking_summary(disc_s)
    rank_hold = ranking_summary(hold_s)
    rank_hold_ex = ranking_summary(hold_ex_s)
    rank_fwd = ranking_summary(fwd_s)
    per_session = session_ranking_table(disc_s + hold_s)

    # Predeclared coverage bands
    bands_disc = band_report(disc_s, "discovery")
    bands_hold = band_report(hold_s, "holdout_contaminated")
    bands_hold_ex = band_report(hold_ex_s, "holdout_ex_oct28")
    bands_fwd = band_report(fwd_s, "forward_nov6")

    improvement = genuine_improvement_check(
        bands_disc, bands_hold, bands_fwd, rank_disc, rank_hold, rank_fwd
    )

    # Gate 2 stays closed — this unit does not advance
    gate2 = {
        "disposition": "GATE2_DO_NOT_ADVANCE",
        "reason": (
            "RANKING-001 is a protocol + diagnostic unit. Contaminated holdout remains "
            "unusable for threshold claims. Oct 28 is promising session evidence, not a "
            "validated 85% policy. Genuine ranking improvement criteria not met for Gate 2."
        ),
        "genuine_ranking_improvement": improvement["genuine_ranking_improvement"],
    }

    o28_sel_wl = wl_stats(oct28_sel)
    conclusion = (
        f"Oct 28 FULL session WR={wl_stats(oct28)['wr_pct']}% "
        f"({wl_stats(oct28)['wins']}/{wl_stats(oct28)['losses']}, n={len(oct28)}). "
        f"Oct 28 ENTRY-DISCOVERY-002 selected slice (p_hat≥0.50) WR="
        f"{o28_sel_wl['wr_pct']}% ({o28_sel_wl['wins']}/{o28_sel_wl['losses']}) "
        f"— the 83.8% signal; gap_to_85={oct28_study['gap_to_85_pp']} pp. "
        f"That slice is entirely MOM_UP∩LOC_MID; full-session MOM∩LOC share="
        f"{feature_mix(oct28).get('mom_up_loc_mid', {}).get('pct')}%. "
        f"Within-Oct28 ranking AUC={ranking_summary(oct28).get('auc')}. "
        f"Discovery AUC={rank_disc.get('auc')}; holdout AUC={rank_hold.get('auc')}; "
        f"holdout_ex_Oct28 AUC={rank_hold_ex.get('auc')}; "
        f"Nov6 forward AUC={rank_fwd.get('auc')} (n={rank_fwd.get('n')}). "
        f"Genuine ranking improvement={improvement['genuine_ranking_improvement']}. "
        f"Untouched later sessions: {boundary['prospective_confirmation_status']}. "
        "85% remains aspirational. Gate 2 CLOSED. No threshold re-scan."
    )

    return {
        "protocol": "RP-608-WR-RANKING-001",
        "authority": "0.0%",
        "baseline_frozen": True,
        "gate2_criteria_changed": False,
        "book": "nse_estimand_a",
        "aspirational_wr_pct": ASPIRATIONAL_WR,
        "frozen_protocol": {
            "score": "p_hat_entry_v1 discovery-only empirical mean P(win|level)",
            "no_holdout_threshold_optimization": True,
            "predeclared_coverage_bands": [b for b, _ in COVERAGE_BANDS],
            "genuine_improvement_criteria": "docs/research/rp608_wr_ranking_001_scoped.md §F",
        },
        "inputs": {
            "join": str(join_path),
            "discovery_n": len(disc_s),
            "holdout_n": len(hold_s),
            "forward_n": len(fwd_s),
            "oct28_n": len(oct28),
        },
        "evaluation_boundary": boundary,
        "oct28_investigation": oct28_study,
        "ranking_discovery": rank_disc,
        "ranking_holdout_contaminated": rank_hold,
        "ranking_holdout_ex_oct28": rank_hold_ex,
        "ranking_forward_nov6": rank_fwd,
        "per_session_ranking": per_session,
        "coverage_bands_discovery": bands_disc,
        "coverage_bands_holdout": bands_hold,
        "coverage_bands_holdout_ex_oct28": bands_hold_ex,
        "coverage_bands_forward": bands_fwd,
        "genuine_improvement_assessment": improvement,
        "gate2_assessment": gate2,
        "gate2_disposition": gate2["disposition"],
        "research_conclusion": conclusion,
        "score_tables_discovery": {
            k: {kk: round(vv, 4) for kk, vv in v.items()} for k, v in emp.items()
        },
    }


def render_md(rep: Dict[str, Any]) -> str:
    o28 = rep["oct28_investigation"]
    b = rep["evaluation_boundary"]
    lines = [
        "# RP-608-WR-RANKING-001 — Ranking & Coverage + Oct 28 Study",
        "",
        f"**Protocol:** `{rep['protocol']}`  ",
        f"**Book:** {rep['book']} · Authority {rep['authority']} · baseline_frozen={rep['baseline_frozen']}  ",
        f"**Gate 2:** CLOSED · criteria unchanged · Gate 3 unauthorized  ",
        f"**Aspirational WR:** {ASPIRATIONAL_WR}% (not a Gate 2 bar)",
        "",
        "## Research conclusion",
        "",
        rep["research_conclusion"],
        "",
        f"**Gate 2 disposition:** `{rep['gate2_disposition']}`",
        "",
        "## Frozen evaluation protocol",
        "",
        f"- Score: {rep['frozen_protocol']['score']}",
        f"- No holdout threshold optimization: **{rep['frozen_protocol']['no_holdout_threshold_optimization']}**",
        f"- Predeclared bands: `{rep['frozen_protocol']['predeclared_coverage_bands']}`",
        "",
        "## Evaluation boundary / chronology",
        "",
        f"- Discovery fit: `{b['discovery_fit_days']}`",
        f"- Contaminated holdout: `{b['contaminated_holdout_days']}`",
        f"- Forward in join: `{b['forward_in_join_days']}`",
        f"- Untouched later sessions: `{b['untouched_later_sessions_in_join']}` "
        f"→ **{b['prospective_confirmation_status']}**",
        f"- {b['chronology_warning']}",
        "",
        "## Oct 28 investigation",
        "",
        f"_{o28['distinction']}_",
        "",
        "### Full eligible population vs 002 selected slice",
        "",
        f"| Lens | n | WR% | W/L | Gap85 pp | Payoff | Net ₹ |",
        f"| --- | ---: | ---: | --- | ---: | ---: | ---: |",
        f"| Full Oct 28 | {o28['full_session']['wl']['n']} | {o28['full_session']['wl']['wr_pct']} | "
        f"{o28['full_session']['wl']['wins']}/{o28['full_session']['wl']['losses']} | "
        f"{o28['gap_to_85_pp_full_session']} | {o28['full_session']['wl']['payoff']} | "
        f"{o28['full_session']['wl']['net_inr']} |",
        f"| 002 slice p_hat≥0.50 | {o28['rp608_002_selected_on_oct28']['wl']['n']} | "
        f"{o28['rp608_002_selected_on_oct28']['wl']['wr_pct']} | "
        f"{o28['rp608_002_selected_on_oct28']['wl']['wins']}/"
        f"{o28['rp608_002_selected_on_oct28']['wl']['losses']} | "
        f"{o28['rp608_002_selected_on_oct28']['gap_to_85_pp']} | "
        f"{o28['rp608_002_selected_on_oct28']['wl']['payoff']} | "
        f"{o28['rp608_002_selected_on_oct28']['wl']['net_inr']} |",
        "",
        f"- Full-session `MOM_UP ∩ LOC_MID` share: "
        f"**{o28['composition']['mom_up_loc_mid']['pct']}%** "
        f"(n={o28['composition']['mom_up_loc_mid']['n']}) · "
        f"slice WR={o28['composition']['mom_up_loc_mid']['wl']['wr_pct']}% · "
        f"complement WR={o28['composition']['mom_up_loc_mid']['complement_wl']['wr_pct']}%",
        f"- 002 selected trades all MOM∩LOC: "
        f"**{o28['rp608_002_selected_on_oct28']['all_mom_loc']}** "
        f"(mom_loc share in selected="
        f"{o28['rp608_002_selected_on_oct28']['mom_loc_share_pct']}%)",
        f"- Interpretation: repeatable_decision_time_pattern="
        f"**{o28['interpretation']['repeatable_decision_time_pattern']}** — "
        f"{o28['interpretation']['note']}",
        "",
        "### vs weaker holdout sessions",
        "",
    ]
    for d, block in o28["vs_weak_sessions"].items():
        full = block["full_session"]
        m = full["composition"]["mom_up_loc_mid"]
        sel = block["rp608_002_ref_slice_p_hat_ge_0_50"]
        lines.append(
            f"- **{d}** full: WR {full['wl']['wr_pct']}% "
            f"({full['wl']['wins']}/{full['wl']['losses']}) · "
            f"MOM∩LOC share {m['pct']}% · MOM∩LOC WR {m['wl']['wr_pct']}% · "
            f"complement WR {m['complement_wl']['wr_pct']}% · "
            f"002-slice n={sel['wl']['n']} WR={sel['wl']['wr_pct']}% "
            f"({sel['wl']['wins']}/{sel['wl']['losses']})"
        )
    disc_block = o28["vs_discovery_aggregate"]
    disc_m = disc_block["full_session"]["composition"]["mom_up_loc_mid"]
    disc_sel = disc_block["rp608_002_ref_slice_p_hat_ge_0_50"]
    lines += [
        f"- **Discovery aggregate** full: WR {disc_block['full_session']['wl']['wr_pct']}% · "
        f"MOM∩LOC share {disc_m['pct']}% · "
        f"002-slice n={disc_sel['wl']['n']} WR={disc_sel['wl']['wr_pct']}%",
        "",
        "### Within Oct 28 — winners vs losers (entry features)",
        "",
        f"- Wins in MOM∩LOC: {o28['within_session_win_vs_loss']['mom_loc_among_wins_pct']}% · "
        f"Losses in MOM∩LOC: {o28['within_session_win_vs_loss']['mom_loc_among_losses_pct']}%",
        f"- Within-session ranking (discovery-fit p_hat, transfer only): "
        f"AUC={o28['within_session_win_vs_loss']['ranking_within_session'].get('auc')} · "
        f"mean score win/loss="
        f"{o28['within_session_win_vs_loss']['ranking_within_session'].get('mean_score_win')}/"
        f"{o28['within_session_win_vs_loss']['ranking_within_session'].get('mean_score_loss')}",
        f"- Within 002 selected slice ranking: "
        f"AUC={o28['within_002_slice_win_vs_loss']['ranking_within_session'].get('auc')}",
        "",
        "## Ranking quality (overall)",
        "",
        f"| Window | n | AUC | mean score win/loss | baseline WR |",
        f"| --- | ---: | ---: | --- | ---: |",
        f"| Discovery | {rep['ranking_discovery']['n']} | {rep['ranking_discovery']['auc']} | "
        f"{rep['ranking_discovery']['mean_score_win']}/{rep['ranking_discovery']['mean_score_loss']} | "
        f"{rep['ranking_discovery']['baseline_wl']['wr_pct']} |",
        f"| Holdout (contaminated) | {rep['ranking_holdout_contaminated']['n']} | "
        f"{rep['ranking_holdout_contaminated']['auc']} | "
        f"{rep['ranking_holdout_contaminated']['mean_score_win']}/{rep['ranking_holdout_contaminated']['mean_score_loss']} | "
        f"{rep['ranking_holdout_contaminated']['baseline_wl']['wr_pct']} |",
        f"| Holdout ex-Oct28 | {rep['ranking_holdout_ex_oct28']['n']} | "
        f"{rep['ranking_holdout_ex_oct28']['auc']} | "
        f"{rep['ranking_holdout_ex_oct28']['mean_score_win']}/{rep['ranking_holdout_ex_oct28']['mean_score_loss']} | "
        f"{rep['ranking_holdout_ex_oct28']['baseline_wl']['wr_pct']} |",
        f"| Forward Nov6 | {rep['ranking_forward_nov6']['n']} | {rep['ranking_forward_nov6']['auc']} | "
        f"{rep['ranking_forward_nov6']['mean_score_win']}/{rep['ranking_forward_nov6']['mean_score_loss']} | "
        f"{rep['ranking_forward_nov6']['baseline_wl']['wr_pct']} |",
        "",
        "## Predeclared coverage bands (no best-threshold pick)",
        "",
        "### Discovery",
        "",
        "| Band | n | Cov% | WR% | W/L | Payoff | Net ₹ | Dom sess % | Gap85 |",
        "| --- | ---: | ---: | ---: | --- | ---: | ---: | ---: | ---: |",
    ]
    for row in rep["coverage_bands_discovery"]:
        lines.append(
            f"| {row['band']} | {row['selected_n']} | {row['coverage_pct']} | {row['wr_pct']} | "
            f"{row['wins']}/{row['losses']} | {row['payoff']} | {row['net_inr']} | "
            f"{row['dominating_session_share_pct']} | {row['gap_to_85_pp']} |"
        )
    lines += ["", "### Holdout (contaminated — descriptive only)", ""]
    lines.append("| Band | n | Cov% | WR% | W/L | Payoff | Net ₹ | Dom sess % | Gap85 |")
    lines.append("| --- | ---: | ---: | ---: | --- | ---: | ---: | ---: | ---: |")
    for row in rep["coverage_bands_holdout"]:
        lines.append(
            f"| {row['band']} | {row['selected_n']} | {row['coverage_pct']} | {row['wr_pct']} | "
            f"{row['wins']}/{row['losses']} | {row['payoff']} | {row['net_inr']} | "
            f"{row['dominating_session_share_pct']} | {row['gap_to_85_pp']} |"
        )
    lines += ["", "### Forward Nov6", ""]
    lines.append("| Band | n | Cov% | WR% | W/L | Payoff | Net ₹ | Gap85 |")
    lines.append("| --- | ---: | ---: | ---: | --- | ---: | ---: | ---: |")
    for row in rep["coverage_bands_forward"]:
        lines.append(
            f"| {row['band']} | {row['selected_n']} | {row['coverage_pct']} | {row['wr_pct']} | "
            f"{row['wins']}/{row['losses']} | {row['payoff']} | {row['net_inr']} | {row['gap_to_85_pp']} |"
        )

    gi = rep["genuine_improvement_assessment"]
    lines += [
        "",
        "## Genuine ranking improvement criteria (§F)",
        "",
        f"**Demonstrated:** **{gi['genuine_ranking_improvement']}**",
        "",
    ]
    for c in gi["checks"]:
        lines.append(
            f"- `{c['id']}`: {'PASS' if c['satisfied'] else 'FAIL'} — {c['detail']}"
        )
    lines += [
        "",
        f"_{gi['note']}_",
        "",
        "## Gate 2",
        "",
        f"**{rep['gate2_assessment']['disposition']}** — {rep['gate2_assessment']['reason']}",
        "",
        "## Governance",
        "",
        "- Baseline / OF ±0.30 / stops / targets / horizon: **unchanged**",
        "- Books not pooled · authority 0% · no threshold re-scan on contaminated holdout",
        "- Next: accumulate untouched post-holdout sessions; improve ranking transfer — not feature sprawl",
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
    stem = out_dir / "rp608_wr_ranking_001"
    stem.with_suffix(".json").write_text(json.dumps(rep, indent=2), encoding="utf-8")
    stem.with_suffix(".md").write_text(render_md(rep), encoding="utf-8")
    o28 = rep["oct28_investigation"]
    o28_full = o28["full_session"]["wl"]
    o28_sel = o28["rp608_002_selected_on_oct28"]["wl"]
    print(
        json.dumps(
            {
                "status": "OK",
                "gate2_disposition": rep["gate2_disposition"],
                "oct28_full_wr": o28_full["wr_pct"],
                "oct28_full_wl": f"{o28_full['wins']}/{o28_full['losses']}",
                "oct28_002_slice_wr": o28_sel["wr_pct"],
                "oct28_002_slice_wl": f"{o28_sel['wins']}/{o28_sel['losses']}",
                "oct28_mom_loc_pct": o28["composition"]["mom_up_loc_mid"]["pct"],
                "auc_discovery": rep["ranking_discovery"]["auc"],
                "auc_holdout": rep["ranking_holdout_contaminated"]["auc"],
                "auc_holdout_ex_oct28": rep["ranking_holdout_ex_oct28"]["auc"],
                "auc_forward": rep["ranking_forward_nov6"]["auc"],
                "genuine_ranking_improvement": rep["genuine_improvement_assessment"][
                    "genuine_ranking_improvement"
                ],
                "untouched_later": rep["evaluation_boundary"]["prospective_confirmation_status"],
                "md": str(stem.with_suffix(".md")),
            },
            indent=2,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
