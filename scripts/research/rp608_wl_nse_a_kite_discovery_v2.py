#!/usr/bin/env python3
"""
RP-608-WL-IMPROVE — NSE Estimand A discovery v2 on Kite decision-time features.

Authority 0%. Baseline frozen. Gate 2 criteria unchanged.
Book: NSE A only — never pooled.

Inputs: wl_nse_a_decision_time_features_kite_join.jsonl
Outputs: wl_nse_a_kite_discovery_v2.{md,json}
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

# Predeclared contrasts (not exhaustive mining)
SPECS = [
    ("market_state", "trend", "TREND_UP", "TREND_DOWN"),
    ("market_state", "trend", "TREND_UP", "TREND_FLAT"),
    ("market_state", "momentum", "MOM_UP_>10bps", "MOM_DOWN_<-10bps"),
    ("market_state", "momentum", "MOM_UP_>10bps", "MOM_FLAT"),
    ("market_state", "price_location", "LOC_MID_THIRD", "LOC_UPPER_THIRD"),
    ("market_state", "price_location", "LOC_MID_THIRD", "LOC_LOWER_THIRD"),
    ("directional_state", "side", "LONG", "SHORT"),
    # diagnostic only — never sole Gate2 advance
    ("temporal_descriptors", "hour_ist", None, None),
]

UNAVAIL = {
    "TREND_UNAVAILABLE",
    "MOMENTUM_UNAVAILABLE",
    "PRICE_LOCATION_UNAVAILABLE",
    None,
    "",
}


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
    dt = datetime.fromtimestamp(ms / 1000, tz=timezone.utc).astimezone(IST)
    return dt.strftime("%H")


def load_join(path: Path) -> List[Dict[str, Any]]:
    rows: List[Dict[str, Any]] = []
    for line in path.read_text(encoding="utf-8", errors="ignore").splitlines():
        if not line.strip():
            continue
        r = json.loads(line)
        day = r.get("session_day")
        if not day:
            ms = pdt_ms(r.get("entry_utc_ms") or r.get("entry_utc"))
            if ms:
                day = datetime.fromtimestamp(ms / 1000, tz=timezone.utc).astimezone(IST).date().isoformat()
        r["session_day"] = day
        r["hour_ist"] = hour_ist(pdt_ms(r.get("entry_utc_ms") or r.get("entry_utc")))
        win = r.get("win")
        if win is None and r.get("net_pnl_inr") is not None:
            win = float(r["net_pnl_inr"]) > 0
        r["win"] = bool(win) if win is not None else None
        feat_ok = (
            r.get("trend") not in UNAVAIL
            and r.get("momentum") not in UNAVAIL
            and r.get("price_location") not in UNAVAIL
        )
        r["feature_ready"] = bool(feat_ok)
        rows.append(r)
    return rows


def wl(rows: Sequence[Dict[str, Any]]) -> Dict[str, Any]:
    usable = [r for r in rows if r.get("win") is not None]
    n = len(usable)
    wins = sum(1 for r in usable if r["win"])
    losses = n - wins
    nets = [float(r["net_pnl_inr"]) for r in usable if r.get("net_pnl_inr") is not None]
    avg_win = (
        sum(x for x in nets if x > 0) / max(1, sum(1 for x in nets if x > 0))
        if any(x > 0 for x in nets)
        else None
    )
    avg_loss = (
        sum(abs(x) for x in nets if x < 0) / max(1, sum(1 for x in nets if x < 0))
        if any(x < 0 for x in nets)
        else None
    )
    payoff = None
    if avg_win is not None and avg_loss and avg_loss > 0:
        payoff = avg_win / avg_loss
    return {
        "n": n,
        "wins": wins,
        "losses": losses,
        "wr_pct": None if n == 0 else round(100.0 * wins / n, 2),
        "net_inr": round(sum(nets), 2) if nets else 0.0,
        "payoff": None if payoff is None else round(payoff, 3),
        "coverage_pct": None,
    }


def contrast(rows: Sequence[Dict[str, Any]], key: str, high: str, low: str, book_n: int) -> Dict[str, Any]:
    hi = [r for r in rows if r.get(key) == high]
    lo = [r for r in rows if r.get(key) == low]
    hs, ls = wl(hi), wl(lo)
    hs["coverage_pct"] = None if book_n == 0 else round(100.0 * hs["n"] / book_n, 2)
    ls["coverage_pct"] = None if book_n == 0 else round(100.0 * ls["n"] / book_n, 2)
    gap = None
    if hs["wr_pct"] is not None and ls["wr_pct"] is not None:
        gap = round(hs["wr_pct"] - ls["wr_pct"], 3)
    return {"high_key": high, "low_key": low, "high": hs, "low": ls, "wr_gap_pp": gap}


def session_signs(rows: Sequence[Dict[str, Any]], key: str, high: str, low: str) -> List[Dict[str, Any]]:
    by_day: Dict[str, List[Dict[str, Any]]] = defaultdict(list)
    for r in rows:
        if r.get("session_day"):
            by_day[str(r["session_day"])].append(r)
    out = []
    for day in sorted(by_day):
        c = contrast(by_day[day], key, high, low, len(by_day[day]))
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


def replicate(disc: Dict[str, Any], hold: Dict[str, Any]) -> str:
    dg, hg = disc.get("wr_gap_pp"), hold.get("wr_gap_pp")
    if dg is None or hg is None:
        return "INCONCLUSIVE_MISSING_GAP"
    if hold["high"]["n"] < 15 or hold["low"]["n"] < 15:
        return "INCONCLUSIVE_UNDERPOWERED"
    if dg == 0:
        return "INCONCLUSIVE_ZERO_DISCOVERY_GAP"
    if (dg > 0 and hg > 0) or (dg < 0 and hg < 0):
        # same sign; require holdout gap not collapsed to ~0
        if abs(hg) < 1.0:
            return "FAILS_REPLICATION_NEAR_ZERO_HOLDOUT"
        return "REPLICATES"
    return "FAILS_REPLICATION"


def hour_diagnostic(rows: Sequence[Dict[str, Any]]) -> Dict[str, Any]:
    by_h: Dict[str, List[Dict[str, Any]]] = defaultdict(list)
    for r in rows:
        by_h[str(r.get("hour_ist") or "??")].append(r)
    stats = {h: wl(v) for h, v in sorted(by_h.items()) if v}
    ranked = sorted(
        ((h, s) for h, s in stats.items() if s["n"] >= 15 and s["wr_pct"] is not None),
        key=lambda x: x[1]["wr_pct"],
        reverse=True,
    )
    if len(ranked) < 2:
        return {"status": "INSUFFICIENT", "note": "temporal diagnostic only"}
    high_h, hs = ranked[0]
    low_h, ls = ranked[-1]
    return {
        "status": "DIAGNOSTIC_ONLY",
        "high_key": high_h,
        "low_key": low_h,
        "high": hs,
        "low": ls,
        "wr_gap_pp": round(hs["wr_pct"] - ls["wr_pct"], 3),
        "note": "Hour contrasts are diagnostic; not Gate 2 promotion alone.",
    }


def evaluate_gate2(cand: Dict[str, Any]) -> Dict[str, Any]:
    criteria = []
    measurable = bool(cand.get("feature") and cand.get("high_key") and cand.get("low_key"))
    criteria.append({"id": "G2-1_MEASURABLE_HYPOTHESIS", "satisfied": measurable})
    criteria.append({"id": "G2-2_DECISION_TIME_ONLY", "satisfied": cand.get("family") != "post_entry"})
    disc = cand["discovery"]
    disc_ok = (
        disc.get("wr_gap_pp") is not None
        and disc["wr_gap_pp"] > 0
        and disc["high"]["n"] >= 15
        and disc["low"]["n"] >= 15
    )
    criteria.append({"id": "G2-3_DISCOVERY_SUPPORT", "satisfied": disc_ok, "detail": disc})
    repl_ok = cand.get("replication_verdict") == "REPLICATES"
    criteria.append(
        {
            "id": "G2-4_HOLDOUT_REPLICATION",
            "satisfied": repl_ok,
            "detail": cand.get("replication_verdict"),
        }
    )
    criteria.append(
        {
            "id": "G2-5_FALSIFIABLE",
            "satisfied": True,
            "detail": "Holdout same-sign WR gap with n>=15 each side required.",
        }
    )
    cov_drop = 100.0 - (disc["high"].get("coverage_pct") or 0.0)
    criteria.append(
        {
            "id": "G2-6_COVERAGE_NOT_SOLE_SUCCESS",
            "satisfied": True,  # enforced by requiring G2-4 for advance
            "detail": f"coverage_drop_if_selected_pct={round(cov_drop, 2)}",
        }
    )
    # G2-7 supporting economic bar (§3b): preferred (high) side must not be
    # payoff-broken / net-destroyed while claiming selection improvement.
    hi = disc["high"]
    payoff_ok = hi.get("payoff") is not None and hi["payoff"] >= 1.0
    net_ok = hi.get("net_inr") is not None and hi["net_inr"] > 0
    econ_ok = payoff_ok and net_ok
    criteria.append(
        {
            "id": "G2-7_PAYOFF_NET_SUPPORT",
            "satisfied": econ_ok,
            "detail": f"high_payoff={hi.get('payoff')} high_net_inr={hi.get('net_inr')}",
        }
    )

    temporal_block = cand.get("family") == "temporal_descriptors"
    base_ok = all(c["satisfied"] for c in criteria if c["id"] != "G2-7_PAYOFF_NET_SUPPORT")
    all_ok = base_ok and econ_ok and repl_ok and not temporal_block and disc_ok
    blocked = None
    if temporal_block:
        blocked = "TEMPORAL_DIAGNOSTIC_ONLY"
    elif repl_ok and disc_ok and not econ_ok:
        blocked = "WR_REPLICATES_BUT_PAYOFF_NET_BAR_FAIL"
    elif not all_ok:
        blocked = "GATE2_CRITERIA_UNMET"
    return {
        "candidate_id": cand["candidate_id"],
        "criteria": criteria,
        "all_satisfied": all_ok,
        "blocked_reason": blocked,
    }


def run(join_path: Path) -> Dict[str, Any]:
    all_rows = load_join(join_path)
    ready = [r for r in all_rows if r.get("feature_ready")]
    disc = [r for r in ready if r.get("session_day") in DISCOVERY_DAYS]
    hold = [r for r in ready if r.get("session_day") in HOLDOUT_ALL]
    hold_pre = [r for r in ready if r.get("session_day") in HOLDOUT_PRE]
    hold_fwd = [r for r in ready if r.get("session_day") in HOLDOUT_FWD]

    by_day = Counter(r.get("session_day") for r in ready)
    candidates: List[Dict[str, Any]] = []

    for family, feature, high, low in SPECS:
        if feature == "hour_ist":
            hd = hour_diagnostic(disc)
            if hd.get("status") != "DIAGNOSTIC_ONLY":
                continue
            high, low = hd["high_key"], hd["low_key"]
            d_contrast = {
                "high_key": high,
                "low_key": low,
                "high": hd["high"],
                "low": hd["low"],
                "wr_gap_pp": hd["wr_gap_pp"],
            }
        else:
            assert high and low
            d_contrast = contrast(disc, feature, high, low, len(disc))

        h_contrast = contrast(hold, feature, high, low, len(hold))
        h_pre = contrast(hold_pre, feature, high, low, len(hold_pre))
        h_fwd = contrast(hold_fwd, feature, high, low, len(hold_fwd))
        verdict = replicate(d_contrast, h_contrast)
        sess = session_signs(disc, feature, high, low)
        single = False
        pos = [s for s in sess if s["sign"] == "POSITIVE"]
        if pos and len(disc) > 0:
            # crude single-session domination: one day contributes >60% of high-side n
            hi_by_day = Counter(
                r["session_day"] for r in disc if r.get(feature) == high
            )
            if hi_by_day:
                top_n = hi_by_day.most_common(1)[0][1]
                single = top_n / max(1, sum(hi_by_day.values())) >= 0.60

        cid = f"CAND-{feature}-{high}-vs-{low}"
        cand = {
            "candidate_id": cid,
            "family": family,
            "feature": feature,
            "high_key": high,
            "low_key": low,
            "discovery": d_contrast,
            "holdout_all": h_contrast,
            "holdout_pre": h_pre,
            "holdout_fwd": h_fwd,
            "replication_verdict": verdict,
            "session_effects": sess,
            "single_session_domination_flag": single,
            "diagnostic_only": family == "temporal_descriptors",
        }
        cand["gate2"] = evaluate_gate2(cand)
        candidates.append(cand)

    advanced = [c for c in candidates if c["gate2"]["all_satisfied"]]
    replicates = [c for c in candidates if c["replication_verdict"] == "REPLICATES"]
    wr_only = [
        c
        for c in candidates
        if c["replication_verdict"] == "REPLICATES"
        and c["gate2"].get("blocked_reason") == "WR_REPLICATES_BUT_PAYOFF_NET_BAR_FAIL"
    ]

    if advanced:
        disposition = "GATE2_CANDIDATE_PRESENT"
        conclusion = (
            f"NSE A Kite discovery produced {len(advanced)} hypothesis(es) satisfying "
            "unchanged Gate 2 criteria including payoff/net support "
            "(human Gate 2 review still required; no live change)."
        )
    elif wr_only:
        disposition = "GATE2_DO_NOT_ADVANCE"
        conclusion = (
            f"WR holdout REPLICATES={len(replicates)} on momentum/location families, but "
            f"{len(wr_only)} preferred-side cohort(s) fail §3b payoff/net support "
            "(payoff<1 and/or net≤0). Gate 2 does not advance. Crypto Gate 2 unchanged."
        )
    elif replicates:
        disposition = "GATE2_DO_NOT_ADVANCE"
        conclusion = (
            f"REPLICATES={len(replicates)} but Gate 2 criteria not fully satisfied "
            "(or temporal-only). Gate 2 remains closed."
        )
    else:
        disposition = "GATE2_DO_NOT_ADVANCE"
        conclusion = (
            "No NSE A Kite decision-time contrast replicated on holdout under unchanged "
            "Gate 2 criteria. Gate 2 remains closed. Crypto Gate 2 disposition unchanged."
        )

    return {
        "protocol": "RP-608-WL-IMPROVE-001-NSE-A-KITE-DISCOVERY-V2",
        "authority": "0.0%",
        "baseline_frozen": True,
        "gate2_criteria_changed": False,
        "book": "nse_estimand_a",
        "join_path": str(join_path),
        "n_join_rows": len(all_rows),
        "n_feature_ready": len(ready),
        "windows": {
            "discovery": DISCOVERY_DAYS,
            "holdout_pre": HOLDOUT_PRE,
            "holdout_fwd": HOLDOUT_FWD,
        },
        "composition": {
            "discovery_n": len(disc),
            "holdout_n": len(hold),
            "holdout_pre_n": len(hold_pre),
            "holdout_fwd_n": len(hold_fwd),
            "by_day": dict(by_day),
            "discovery_wl": wl(disc),
            "holdout_wl": wl(hold),
        },
        "candidates": candidates,
        "replicates_n": len(replicates),
        "advanced_n": len(advanced),
        "gate2_disposition": disposition,
        "research_conclusion": conclusion,
    }


def render_md(rep: Dict[str, Any]) -> str:
    lines = [
        "# RP-608-WL-IMPROVE — NSE A Kite Discovery v2",
        "",
        f"**Protocol:** `{rep['protocol']}`  ",
        f"**Authority:** {rep['authority']} · baseline_frozen={rep['baseline_frozen']} · "
        f"gate2_criteria_changed={rep['gate2_criteria_changed']}  ",
        f"**Book:** {rep['book']} (never pooled)",
        "",
        "## Research conclusion",
        "",
        rep["research_conclusion"],
        "",
        f"**Gate 2 disposition:** {rep['gate2_disposition']}",
        "",
        "## Composition",
        "",
        f"- Feature-ready rows: **{rep['n_feature_ready']}** / join {rep['n_join_rows']}",
        f"- Discovery n={rep['composition']['discovery_n']} · "
        f"Holdout n={rep['composition']['holdout_n']} "
        f"(pre={rep['composition']['holdout_pre_n']}, fwd={rep['composition']['holdout_fwd_n']})",
        f"- Discovery WR%={rep['composition']['discovery_wl'].get('wr_pct')} · "
        f"Holdout WR%={rep['composition']['holdout_wl'].get('wr_pct')}",
        f"- By day: `{rep['composition']['by_day']}`",
        "",
        "## Candidates",
        "",
        "| Candidate | Disc gap pp (n_hi/n_lo) | Holdout verdict | Gate2 |",
        "| --- | ---: | --- | --- |",
    ]
    for c in rep["candidates"]:
        d = c["discovery"]
        gap = d.get("wr_gap_pp")
        lines.append(
            f"| `{c['candidate_id']}` | {gap} "
            f"({d['high']['n']}/{d['low']['n']}) | {c['replication_verdict']} | "
            f"{'PASS' if c['gate2']['all_satisfied'] else 'NO'} |"
        )
    lines += [
        "",
        "## Governance",
        "",
        "- OF ±0.30 / stops / targets / Estimand A thresholds: **unchanged**",
        "- Crypto Gate 2: **unchanged** (separate book)",
        "- Gate 3: unauthorized unless Gate 2 advances after review",
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
    stem = out_dir / "wl_nse_a_kite_discovery_v2"
    json_path = stem.with_suffix(".json")
    md_path = stem.with_suffix(".md")
    json_path.write_text(json.dumps(rep, indent=2), encoding="utf-8")
    md_path.write_text(render_md(rep), encoding="utf-8")
    print(
        json.dumps(
            {
                "status": "OK",
                "gate2_disposition": rep["gate2_disposition"],
                "replicates_n": rep["replicates_n"],
                "advanced_n": rep["advanced_n"],
                "discovery_n": rep["composition"]["discovery_n"],
                "holdout_n": rep["composition"]["holdout_n"],
                "md": str(md_path),
            },
            indent=2,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
