#!/usr/bin/env python3
"""
RP-608 Gate 2E — join SHADOW_ALLOW_IF_N10_WR_GT_035 stamps to BASELINE outcomes.

Observational only (authority 0%). Does not loosen live allow_only.
Books never pooled: PRIMARY shadows vs BASELINE closed trades only.

Usage:
  python3 scripts/research/rp608_gate2e_shadow_baseline_join.py
  python3 scripts/research/rp608_gate2e_shadow_baseline_join.py --since-hours 48 --json
"""

from __future__ import annotations

import argparse
import json
import sys
from collections import Counter
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

WORKSPACE = Path(__file__).resolve().parents[2]
CRYPTO = WORKSPACE / "live_capture" / "shadow_validation" / "crypto"
SHADOW_PATH = CRYPTO / "trade_plan_overlay_v1" / "gate2e_shadow_allows.jsonl"
BASELINE_OUTCOMES = (
    CRYPTO / "trade_plan_overlay_baseline_parallel_v1" / "trade_plan_outcomes.jsonl"
)
SHADOW_RULE = "SHADOW_ALLOW_IF_N10_WR_GT_035"
# Join window: shadow suppress time → nearest BASELINE open in same direction.
JOIN_TOLERANCE_MS = 120_000  # 2 minutes


def _load_jsonl(path: Path) -> List[Dict[str, Any]]:
    if not path.exists():
        return []
    rows: List[Dict[str, Any]] = []
    with open(path, encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                r = json.loads(line)
            except json.JSONDecodeError:
                continue
            if isinstance(r, dict):
                rows.append(r)
    return rows


def _entry_ms(row: Dict[str, Any]) -> Optional[int]:
    for k in ("entry_time_ms", "activated_ms", "open_time_ms"):
        v = row.get(k)
        if v is None:
            continue
        try:
            return int(v)
        except (TypeError, ValueError):
            continue
    return None


def _exit_ms(row: Dict[str, Any]) -> Optional[int]:
    for k in ("exit_time_ms", "closed_ms"):
        v = row.get(k)
        if v is None:
            continue
        try:
            return int(v)
        except (TypeError, ValueError):
            continue
    return None


def join_shadow_to_baseline(
    shadows: List[Dict[str, Any]],
    baselines: List[Dict[str, Any]],
    *,
    since_ms: Optional[int] = None,
    tol_ms: int = JOIN_TOLERANCE_MS,
) -> Dict[str, Any]:
    """Match each shadow stamp to nearest BASELINE open (same direction)."""
    base_idx: List[Tuple[int, Dict[str, Any]]] = []
    for b in baselines:
        em = _entry_ms(b)
        if em is None:
            continue
        if since_ms is not None and em < since_ms:
            continue
        base_idx.append((em, b))
    base_idx.sort(key=lambda x: x[0])

    joined: List[Dict[str, Any]] = []
    unmatched = 0
    wins = losses = 0
    net_sum = 0.0
    win_abs: List[float] = []
    loss_abs: List[float] = []
    vol_wl: Dict[str, Dict[str, Any]] = {}

    for s in shadows:
        if str(s.get("shadow_rule_id") or "") != SHADOW_RULE:
            continue
        sms = s.get("suppressed_ms")
        try:
            sms_i = int(sms)
        except (TypeError, ValueError):
            unmatched += 1
            continue
        if since_ms is not None and sms_i < since_ms:
            continue
        direction = str(s.get("direction") or "").upper()
        best = None
        best_abs = None
        for em, b in base_idx:
            if str(b.get("direction") or "").upper() != direction:
                continue
            d = abs(em - sms_i)
            if d > tol_ms:
                continue
            if best_abs is None or d < best_abs:
                best_abs = d
                best = b
        if best is None:
            unmatched += 1
            continue
        net = float(best.get("net_pnl_usdt") or 0.0)
        net_sum += net
        if net > 0:
            wins += 1
            win_abs.append(abs(net))
        else:
            losses += 1
            loss_abs.append(abs(net))
        # Prefer live Gate2E candle vol (≠ OFX; ≠ BACKOFF mined pattern V*)
        vol = str(
            s.get("live_vol_surge") or s.get("vol_surge") or "UNK"
        )
        if vol == "UNK":
            for pid in (
                str(s.get("feature_pattern_id") or ""),
                str(s.get("pattern_id") or ""),
            ):
                tail = pid.rsplit("|", 1)[-1] if pid else ""
                if tail.startswith("V") and len(tail) > 1 and tail[1:] != "UNK":
                    vol = tail[1:]
                    break
                if tail.startswith("V") and len(tail) > 1:
                    vol = tail[1:]
                    break
        bucket = vol_wl.setdefault(vol, {"wins": 0, "losses": 0, "net": 0.0, "n": 0})
        bucket["n"] += 1
        bucket["net"] += net
        if net > 0:
            bucket["wins"] += 1
        else:
            bucket["losses"] += 1
        joined.append(
            {
                "shadow_plan_id": s.get("plan_id"),
                "shadow_direction": direction,
                "suppressed_ms": sms_i,
                "hist_wr": s.get("hist_wr"),
                "cohort_n": s.get("cohort_n"),
                "pattern_id": s.get("pattern_id"),
                "entry_price": s.get("entry_price"),
                "vol_surge": vol,
                "backfill": bool(s.get("backfill")),
                "baseline_plan_id": best.get("plan_id"),
                "baseline_entry_ms": _entry_ms(best),
                "baseline_exit_ms": _exit_ms(best),
                "baseline_exit_reason": best.get("exit_reason") or best.get("state"),
                "baseline_net_pnl_usdt": net,
                "join_delta_ms": best_abs,
            }
        )

    n = wins + losses
    wr = (wins / n) if n else None
    avg_win = (sum(win_abs) / len(win_abs)) if win_abs else None
    avg_loss = (sum(loss_abs) / len(loss_abs)) if loss_abs else None
    payoff = (
        (avg_win / avg_loss)
        if avg_win is not None and avg_loss is not None and avg_loss > 0
        else None
    )
    return {
        "protocol": "RP-608-GATE2E-SHADOW-BASELINE-JOIN",
        "shadow_rule_id": SHADOW_RULE,
        "generated_utc": datetime.now(timezone.utc).isoformat(),
        "since_ms": since_ms,
        "join_tolerance_ms": tol_ms,
        "n_shadow_scanned": len(shadows),
        "n_joined": len(joined),
        "n_unmatched": unmatched,
        "baseline_wl": {"wins": wins, "losses": losses, "n": n, "wr": wr},
        "payoff_avg_win_over_avg_loss": payoff,
        "net_pnl_usdt_sum": round(net_sum, 4),
        "wl_by_vol_surge": {
            k: {
                **v,
                "wr": (v["wins"] / v["n"]) if v["n"] else None,
                "net": round(v["net"], 4),
            }
            for k, v in sorted(vol_wl.items())
        },
        "note": (
            "Observational counterfactual only — live PRIMARY allow_only unchanged; "
            "books never pooled; authority 0%; price+volume axes for study"
        ),
        "joined": joined,
    }


def main() -> int:
    ap = argparse.ArgumentParser(description="Join Gate2E thin-WR shadows to BASELINE outcomes")
    ap.add_argument("--shadow", type=Path, default=SHADOW_PATH)
    ap.add_argument("--baseline", type=Path, default=BASELINE_OUTCOMES)
    ap.add_argument("--since-hours", type=float, default=None)
    ap.add_argument("--tol-ms", type=int, default=JOIN_TOLERANCE_MS)
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()

    since_ms = None
    if args.since_hours is not None:
        since_ms = int(
            (datetime.now(timezone.utc).timestamp() - float(args.since_hours) * 3600) * 1000
        )

    shadows = _load_jsonl(args.shadow)
    baselines = _load_jsonl(args.baseline)
    report = join_shadow_to_baseline(
        shadows, baselines, since_ms=since_ms, tol_ms=int(args.tol_ms)
    )

    if args.json:
        json.dump(report, sys.stdout, indent=2)
        print()
        return 0

    print("RP-608 Gate2E shadow → BASELINE join (never pooled; observational)")
    print(f"  shadow={args.shadow} exists={args.shadow.exists()} n={len(shadows)}")
    print(f"  baseline={args.baseline} exists={args.baseline.exists()} n={len(baselines)}")
    print(
        f"  joined={report['n_joined']} unmatched={report['n_unmatched']} "
        f"tol_ms={report['join_tolerance_ms']}"
    )
    wl = report["baseline_wl"]
    print(
        f"  W/L={wl['wins']}/{wl['n']} wr={wl['wr']} "
        f"payoff={report['payoff_avg_win_over_avg_loss']} "
        f"net_sum={report['net_pnl_usdt_sum']}"
    )
    by_vol = report.get("wl_by_vol_surge") or {}
    if by_vol:
        print("  W/L by vol_surge (Gate2E candle axis ≠ OFX):")
        for k, v in by_vol.items():
            print(
                f"    {k}: W/L={v['wins']}/{v['n']} wr={v['wr']} net={v['net']}"
            )
    if not shadows:
        print("  (no shadow stamps yet — await thin-WR PRIMARY suppress)")
    print("  live allow_only unchanged — do not ADVANCE from this alone")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
