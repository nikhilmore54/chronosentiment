#!/usr/bin/env python3
"""
RP-608-SCI-ADAPTER-REC — sealed CLEAN vs stacked comparative OBSERVE (authority 0%).

RQ (Anuj 2026-11-28): Does removing inherited Gate2E selection contamination
materially change SCI recommendation quality, coverage ownership, and after-cost
economics on the same BTC book?

Does not modify either twin. Prefer failure-distribution contrast over hit-rate.
Books never pooled.

Usage:
  python3 scripts/research/rp608_sci_adapter_rec_clean_vs_stacked_observe.py
  python3 scripts/research/rp608_sci_adapter_rec_clean_vs_stacked_observe.py --json
"""

from __future__ import annotations

import argparse
import json
from collections import Counter
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

WORKSPACE = Path(__file__).resolve().parents[2]
CRYPTO = WORKSPACE / "live_capture/shadow_validation/crypto"
SCI_DIR = CRYPTO / "trade_plan_overlay_sci_adapter_rec_v1"
CLEAN_DIR = CRYPTO / "trade_plan_overlay_sci_adapter_rec_clean_v1"

# CLEAN twin arm ≈ 2026-11-28 08:03 Asia/Calcutta
CLEAN_ARM_UTC = "2026-11-28T02:33:00+00:00"

ATTEMPT_EVENTS = {
    "PLAN_CREATED",
    "SUPPRESSED_SCI_ADAPTER_REC",
    "SUPPRESSED_GATE2E_DI_SKEW",
    "SUPPRESSED_GATE2E_PATH_INVALID",
    "SUPPRESSED_GATE2E_REGIME_ALIGN",
    "SUPPRESSED_GATE2E_REGIME_DIRECTIONAL",
}


def _load(path: Path) -> List[Dict[str, Any]]:
    if not path.exists():
        return []
    rows: List[Dict[str, Any]] = []
    for line in path.read_text(encoding="utf-8", errors="ignore").splitlines():
        if not line.strip():
            continue
        try:
            rows.append(json.loads(line))
        except json.JSONDecodeError:
            continue
    return rows


def _f(row: Dict[str, Any], *keys: str, default: float = 0.0) -> float:
    for k in keys:
        if row.get(k) is not None:
            try:
                return float(row[k])
            except (TypeError, ValueError):
                continue
    return default


def _etype(e: Dict[str, Any]) -> str:
    return str(e.get("event_type") or e.get("type") or "NONE")


def _ts(e: Dict[str, Any]) -> str:
    return str(e.get("source_timestamp_utc") or e.get("timestamp_utc") or "")


def _window(rows: List[Dict[str, Any]], arm: str) -> List[Dict[str, Any]]:
    arm_cmp = arm.replace("+00:00", "")
    out: List[Dict[str, Any]] = []
    for r in rows:
        t = _ts(r).replace("+00:00", "").replace("Z", "")
        if t and t >= arm_cmp:
            out.append(r)
    return out


def _created_ids(rows: List[Dict[str, Any]]) -> List[str]:
    return [str(e.get("plan_id")) for e in rows if _etype(e) == "PLAN_CREATED" and e.get("plan_id")]


def _outcome_stats(outs: List[Dict[str, Any]]) -> Dict[str, Any]:
    if not outs:
        return {
            "n": 0,
            "wins": 0,
            "losses": 0,
            "wl": None,
            "avg_net_win": None,
            "avg_abs_net_loss": None,
            "avg_gross": None,
            "avg_friction": None,
            "avg_net": None,
            "sum_net": 0.0,
            "sum_gross": 0.0,
            "sum_friction": 0.0,
            "payoff_asym_net": None,
            "net_expectancy": None,
            "exit_counts": {},
            "side_counts": {},
            "failure_primary": {},
        }
    wins = [o for o in outs if _f(o, "net_pnl_usdt") >= 0]
    losses = [o for o in outs if _f(o, "net_pnl_usdt") < 0]
    avg_w = (sum(_f(o, "net_pnl_usdt") for o in wins) / len(wins)) if wins else None
    avg_l = (sum(abs(_f(o, "net_pnl_usdt")) for o in losses) / len(losses)) if losses else None
    n = len(outs)
    pw, pl = len(wins) / n, len(losses) / n
    avg_net = sum(_f(o, "net_pnl_usdt") for o in outs) / n
    # Net expectancy ≡ average net per closed trade (after-cost).
    payoff = (abs(avg_w) / avg_l) if (avg_w is not None and avg_l) else None
    # Failure primary (reuse lightweight rules from event_observe)
    primary: Counter[str] = Counter()
    for o in outs:
        net = _f(o, "net_pnl_usdt")
        if net >= 0:
            primary["WIN"] += 1
            continue
        gross = _f(o, "gross_pnl_usdt")
        fric = _f(o, "friction_usdt")
        entry = _f(o, "entry_price")
        exitp = _f(o, "exit_price")
        d = str(o.get("direction") or "")
        dir_ok = (d == "LONG" and exitp > entry) or (d == "SHORT" and exitp < entry)
        if (gross > 0 and net < 0) or (gross <= 0 and fric > abs(gross)):
            primary["EXECUTION_FRICTION"] += 1
        elif (not dir_ok) and gross < 0:
            primary["DIRECTIONAL_FAILURE"] += 1
        else:
            primary["OTHER_LOSS"] += 1
    return {
        "n": n,
        "wins": len(wins),
        "losses": len(losses),
        "wl": f"{len(wins)}/{len(losses)}",
        "avg_net_win": round(avg_w, 4) if avg_w is not None else None,
        "avg_abs_net_loss": round(avg_l, 4) if avg_l is not None else None,
        "avg_gross": round(sum(_f(o, "gross_pnl_usdt") for o in outs) / n, 4),
        "avg_friction": round(sum(_f(o, "friction_usdt") for o in outs) / n, 4),
        "avg_net": round(avg_net, 4),
        "sum_net": round(sum(_f(o, "net_pnl_usdt") for o in outs), 4),
        "sum_gross": round(sum(_f(o, "gross_pnl_usdt") for o in outs), 4),
        "sum_friction": round(sum(_f(o, "friction_usdt") for o in outs), 4),
        "payoff_asym_net": round(payoff, 4) if payoff is not None else None,
        "net_expectancy": round(avg_net, 4),
        "p_win": round(pw, 4),
        "p_loss": round(pl, 4),
        "exit_counts": dict(Counter(str(o.get("exit_reason") or "NONE") for o in outs)),
        "side_counts": dict(Counter(str(o.get("direction") or "NONE") for o in outs)),
        "failure_primary": dict(primary),
    }


def _attempt_breakdown(events: List[Dict[str, Any]]) -> Dict[str, Any]:
    c = Counter(_etype(e) for e in events)
    attempt = sum(c[k] for k in ATTEMPT_EVENTS)
    created = c.get("PLAN_CREATED", 0)
    return {
        "event_counts": {k: c[k] for k in sorted(c) if k.startswith("SUPPRESS") or k == "PLAN_CREATED" or "PATH" in k},
        "eligible_attempt_n": attempt,
        "plan_created": created,
        "coverage_created_per_attempt": round(created / attempt, 4) if attempt else None,
        "di_skew": c.get("SUPPRESSED_GATE2E_DI_SKEW", 0),
        "path_invalid": c.get("SUPPRESSED_GATE2E_PATH_INVALID", 0),
        "sci_suppress": c.get("SUPPRESSED_SCI_ADAPTER_REC", 0),
        "open_plan": c.get("SUPPRESSED_OPEN_PLAN", 0),
        "di_share_of_attempt": round(c.get("SUPPRESSED_GATE2E_DI_SKEW", 0) / attempt, 4) if attempt else None,
        "path_share_of_attempt": round(c.get("SUPPRESSED_GATE2E_PATH_INVALID", 0) / attempt, 4) if attempt else None,
    }


def compare(arm_utc: str = CLEAN_ARM_UTC) -> Dict[str, Any]:
    sci_ev = _load(SCI_DIR / "trade_plan_events.jsonl")
    cl_ev = _load(CLEAN_DIR / "trade_plan_events.jsonl")
    sci_out = _load(SCI_DIR / "trade_plan_outcomes.jsonl")
    cl_out = _load(CLEAN_DIR / "trade_plan_outcomes.jsonl")

    sci_win_ev = _window(sci_ev, arm_utc)
    # CLEAN lifetime == post-arm by construction
    cl_ids = set(_created_ids(cl_ev))
    sci_ids_all = set(_created_ids(sci_ev))
    sci_ids_win = set(_created_ids(sci_win_ev))
    overlap_all = sorted(cl_ids & sci_ids_all)
    overlap_win = sorted(cl_ids & sci_ids_win)
    clean_only = sorted(cl_ids - sci_ids_all)
    stacked_win_only = sorted(sci_ids_win - cl_ids)

    sci_out_win = [
        o
        for o in sci_out
        if str(o.get("plan_id") or "") in cl_ids
        or str(o.get("source_timestamp_utc") or "").replace("+00:00", "")
        >= arm_utc.replace("+00:00", "")
    ]
    # Prefer plan-id aligned closed set for post-arm economics
    sci_out_aligned = [o for o in sci_out if str(o.get("plan_id") or "") in cl_ids]
    cl_out_aligned = [o for o in cl_out if str(o.get("plan_id") or "") in cl_ids]

    matched = []
    for o in cl_out_aligned:
        pid = o.get("plan_id")
        so = next((x for x in sci_out_aligned if x.get("plan_id") == pid), None)
        if so:
            matched.append(
                {
                    "plan_id": pid,
                    "clean_net": _f(o, "net_pnl_usdt"),
                    "stacked_net": _f(so, "net_pnl_usdt"),
                    "clean_exit": o.get("exit_reason"),
                    "stacked_exit": so.get("exit_reason"),
                    "identical_net": abs(_f(o, "net_pnl_usdt") - _f(so, "net_pnl_usdt")) < 1e-9,
                }
            )

    sci_life = _attempt_breakdown(sci_ev)
    sci_win = _attempt_breakdown(sci_win_ev)
    cl_life = _attempt_breakdown(cl_ev)

    # RQ verdict (honest, sample-aware)
    post_arm_n = len(cl_out_aligned)
    identical_economics = bool(matched) and all(m["identical_net"] for m in matched)
    di_path_in_window = (sci_win.get("di_skew", 0) + sci_win.get("path_invalid", 0)) == 0
    if post_arm_n < 5:
        rq_status = "INCONCLUSIVE_SAMPLE"
        rq_note = (
            "Post-arm closed n too small for materiality claim; "
            "twins currently synchronized on shared plan_ids when DI/PATH idle."
        )
    elif identical_economics and di_path_in_window:
        rq_status = "NO_MATERIAL_DELTA_YET"
        rq_note = "Aligned closed economics identical while Gate2E DI/PATH not firing in window."
    elif not identical_economics:
        rq_status = "MATERIAL_DELTA_OBSERVED"
        rq_note = "Aligned closed nets diverge — inspect failure mix."
    else:
        rq_status = "COVERAGE_SHIFT_PENDING_ECONOMICS"
        rq_note = "Coverage/attempt mix differs; wait for closed economics under DI/PATH pressure."

    return {
        "generated_utc": datetime.now(timezone.utc).isoformat(),
        "rq": (
            "Does removing inherited Gate2E selection contamination materially change "
            "SCI recommendation quality, coverage ownership, and after-cost economics "
            "on the same BTC book?"
        ),
        "arm_utc": arm_utc,
        "constraints": {
            "twins_modified": False,
            "pooled": False,
            "ml1d": False,
            "coralys_gate5": False,
        },
        "overlap": {
            "clean_plan_created": sorted(cl_ids),
            "stacked_plan_created_post_arm": sorted(sci_ids_win),
            "overlap_plan_ids": overlap_win,
            "overlap_n": len(overlap_win),
            "clean_only_plan_ids": clean_only,
            "stacked_win_only_plan_ids": stacked_win_only,
            "unique_event_overlap_rate": (
                round(len(overlap_win) / len(cl_ids), 4) if cl_ids else None
            ),
        },
        "coverage_ownership": {
            "stacked_lifetime": sci_life,
            "stacked_post_arm": sci_win,
            "clean_lifetime": cl_life,
            "path_invalid_attribution": {
                "stacked_lifetime": sci_life.get("path_invalid"),
                "stacked_post_arm": sci_win.get("path_invalid"),
                "clean_lifetime": cl_life.get("path_invalid"),
                "note": "PATH_INVALID retained on CLEAN DNA; DI/regime bypass only.",
            },
            "di_skew_attribution": {
                "stacked_lifetime": sci_life.get("di_skew"),
                "stacked_post_arm": sci_win.get("di_skew"),
                "clean_lifetime": cl_life.get("di_skew"),
                "note": "CLEAN should stay ~0 DI_SKEW if bypass holds.",
            },
        },
        "economics": {
            "stacked_lifetime": _outcome_stats(sci_out),
            "stacked_post_arm_aligned": _outcome_stats(sci_out_aligned),
            "clean": _outcome_stats(cl_out),
            "matched_closed": matched,
        },
        "rq_verdict": {"status": rq_status, "note": rq_note, "prefer": "failure_distribution_over_hit_rate"},
    }


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--json", action="store_true")
    ap.add_argument("--arm-utc", default=CLEAN_ARM_UTC)
    args = ap.parse_args()
    rep = compare(args.arm_utc)
    if args.json:
        print(json.dumps(rep, indent=2, sort_keys=True))
        return 0
    print(f"[CLEAN_VS_STACKED] generated={rep['generated_utc']}")
    print(f"  RQ verdict: {rep['rq_verdict']['status']} — {rep['rq_verdict']['note']}")
    ov = rep["overlap"]
    print(
        f"  overlap plan_ids={ov['overlap_n']} clean_only={len(ov['clean_only_plan_ids'])} "
        f"stacked_win_only={len(ov['stacked_win_only_plan_ids'])}"
    )
    for label, block in (
        ("stacked_lifetime", rep["economics"]["stacked_lifetime"]),
        ("stacked_aligned", rep["economics"]["stacked_post_arm_aligned"]),
        ("clean", rep["economics"]["clean"]),
    ):
        print(
            f"  {label}: W/L={block.get('wl')} n={block.get('n')} "
            f"net_exp={block.get('net_expectancy')} "
            f"fail={block.get('failure_primary')}"
        )
    cov = rep["coverage_ownership"]
    print(
        f"  PATH stacked_life={cov['path_invalid_attribution']['stacked_lifetime']} "
        f"post_arm={cov['path_invalid_attribution']['stacked_post_arm']} "
        f"clean={cov['path_invalid_attribution']['clean_lifetime']}"
    )
    print(
        f"  DI stacked_life={cov['di_skew_attribution']['stacked_lifetime']} "
        f"post_arm={cov['di_skew_attribution']['stacked_post_arm']} "
        f"clean={cov['di_skew_attribution']['clean_lifetime']}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
