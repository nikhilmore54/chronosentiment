#!/usr/bin/env python3
"""Crypto OF paper QUALITY vs WEAK diagnostic (decision-time depth only).

QUALITY bar (predeclared): net_return_bps >= 15 OR (hold >= 10m and net > 0).
Does not change OF ±0.30. Books never pooled with CRYPTO_PLAN.
"""

from __future__ import annotations

import argparse
import json
import statistics
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple


QUALITY_BPS = 15.0
QUALITY_HOLD_S = 600.0


def _f(v: Any) -> Optional[float]:
    try:
        if v is None:
            return None
        return float(v)
    except (TypeError, ValueError):
        return None


def is_quality(row: Dict[str, Any]) -> bool:
    bps = _f(row.get("net_return_bps"))
    net = _f(row.get("net_pnl_usdt"))
    hold = _f(row.get("holding_duration_seconds"))
    if bps is not None and bps >= QUALITY_BPS:
        return True
    if hold is not None and hold >= QUALITY_HOLD_S and net is not None and net > 0:
        return True
    return False


def wl_net(rows: List[Dict[str, Any]]) -> Dict[str, Any]:
    wins = [r for r in rows if (_f(r.get("net_pnl_usdt")) or 0) > 0]
    losses = [r for r in rows if (_f(r.get("net_pnl_usdt")) or 0) <= 0]
    nets = [_f(r.get("net_pnl_usdt")) or 0.0 for r in rows]
    avg_w = (
        statistics.mean([_f(r.get("net_pnl_usdt")) or 0.0 for r in wins]) if wins else None
    )
    avg_l = (
        statistics.mean([abs(_f(r.get("net_pnl_usdt")) or 0.0) for r in losses])
        if losses
        else None
    )
    payoff = (
        (avg_w / avg_l) if avg_w is not None and avg_l and avg_l > 0 else None
    )
    return {
        "n": len(rows),
        "wins": len(wins),
        "losses": len(losses),
        "wl": f"{len(wins)}/{len(losses)}" if rows else "n/a",
        "net_sum": round(sum(nets), 4),
        "avg_win": round(avg_w, 4) if avg_w is not None else None,
        "avg_abs_loss": round(avg_l, 4) if avg_l is not None else None,
        "payoff": round(payoff, 3) if payoff is not None else None,
    }


def depth_field(row: Dict[str, Any], key: str) -> Optional[float]:
    dq = row.get("depth_quality_entry") or row.get("depth_quality") or {}
    if not isinstance(dq, dict):
        return None
    return _f(dq.get(key))


def cohort_depth_stats(rows: List[Dict[str, Any]], key: str) -> Dict[str, Any]:
    xs = [v for v in (depth_field(r, key) for r in rows) if v is not None]
    if not xs:
        return {"n": 0, "median": None, "p25": None, "p75": None}
    xs_s = sorted(xs)
    n = len(xs_s)
    return {
        "n": n,
        "median": round(xs_s[n // 2], 4),
        "p25": round(xs_s[max(0, n // 4)], 4),
        "p75": round(xs_s[min(n - 1, (3 * n) // 4)], 4),
        "min": round(xs_s[0], 4),
        "max": round(xs_s[-1], 4),
    }


def suggest_floor(
    quality: List[Dict[str, Any]], weak: List[Dict[str, Any]], key: str
) -> Optional[Dict[str, Any]]:
    """Suggest floor near QUALITY p25 if it still keeps coverage >= 40% of all."""
    q = cohort_depth_stats(quality, key)
    if not q["n"] or q["p25"] is None:
        return None
    floor = float(q["p25"])
    all_rows = quality + weak
    retained = [
        r
        for r in all_rows
        if (depth_field(r, key) or 0.0) >= floor
    ]
    cov = len(retained) / len(all_rows) if all_rows else 0.0
    return {
        "field": key,
        "suggested_floor": floor,
        "quality_p25": q["p25"],
        "coverage_if_applied": round(cov, 4),
        "meets_coverage_floor_40pct": cov >= 0.40,
    }


def load_outcomes(path: Path) -> List[Dict[str, Any]]:
    rows: List[Dict[str, Any]] = []
    if not path.exists():
        return rows
    for line in path.read_text().splitlines():
        if not line.strip():
            continue
        try:
            rows.append(json.loads(line))
        except json.JSONDecodeError:
            continue
    return rows


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--outcomes",
        default=(
            "live_capture/shadow_validation/crypto/trade_plan_overlay_v1/"
            "of_trade_outcomes.jsonl"
        ),
    )
    args = ap.parse_args()
    rows = load_outcomes(Path(args.outcomes))
    closed = [r for r in rows if r.get("net_pnl_usdt") is not None]
    quality = [r for r in closed if is_quality(r)]
    weak = [r for r in closed if not is_quality(r)]
    keys = [
        "same_side_touch_notional_usdt",
        "combined_touch_notional_usdt",
        "l1_imbalance",
        "same_side_touch_qty",
    ]
    suggestions = []
    for k in keys:
        sug = suggest_floor(quality, weak, k)
        if sug:
            suggestions.append(sug)
    out = {
        "protocol": "RP-608-OF-PAPER-QUALITY-DIAG-001",
        "quality_rule": {
            "net_return_bps_ge": QUALITY_BPS,
            "or_hold_s_ge_and_net_gt_0": QUALITY_HOLD_S,
        },
        "n_closed": len(closed),
        "quality": wl_net(quality),
        "weak": wl_net(weak),
        "depth_quality": {
            "quality": {k: cohort_depth_stats(quality, k) for k in keys},
            "weak": {k: cohort_depth_stats(weak, k) for k in keys},
        },
        "floor_suggestions": suggestions,
    }
    print(json.dumps(out, indent=2))
    print("\n# W/L-first (OF book only)")
    print(
        f"QUALITY n={out['quality']['n']} W/L={out['quality']['wl']} "
        f"net={out['quality']['net_sum']}"
    )
    print(
        f"WEAK    n={out['weak']['n']} W/L={out['weak']['wl']} "
        f"net={out['weak']['net_sum']}"
    )
    for s in suggestions:
        print(
            f"floor {s['field']}={s['suggested_floor']} "
            f"cov={s['coverage_if_applied']} ok40={s['meets_coverage_floor_40pct']}"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
