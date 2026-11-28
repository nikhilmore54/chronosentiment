#!/usr/bin/env python3
"""
RP-608-SCI-ADAPTER-REC — event-level OBSERVE classifier (authority 0%).

Per Anuj empirical loop: classify each closed plan into failure categories
before proposing a FIX. Books never pooled. Does not retune armed soaks.

Usage:
  python3 scripts/research/rp608_sci_adapter_rec_event_observe.py
  python3 scripts/research/rp608_sci_adapter_rec_event_observe.py --book SCI --json
"""

from __future__ import annotations

import argparse
import json
from collections import Counter
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

WORKSPACE = Path(__file__).resolve().parents[2]
CRYPTO_ROOT = WORKSPACE / "live_capture/shadow_validation/crypto"

BOOKS = {
    "SCI": "trade_plan_overlay_sci_adapter_rec_v1",
    "SCI_CLEAN": "trade_plan_overlay_sci_adapter_rec_clean_v1",
    "SCI_ETH": "trade_plan_overlay_sci_adapter_rec_eth_v1",
}

# Meaningful-movement floor for path-support hypothesis only (not Gate 5 / exit change).
MFE_FLOOR_USDT = 1.0


def _load_jsonl(path: Path) -> List[Dict[str, Any]]:
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


def classify(o: Dict[str, Any]) -> Tuple[str, List[str]]:
    """Return (primary, all_tags). WIN if net>=0."""
    net = _f(o, "net_pnl_usdt")
    if net >= 0.0:
        return "WIN", ["WIN"]

    gross = _f(o, "gross_pnl_usdt")
    fric = _f(o, "friction_usdt", "est_friction_usdt")
    mfe = _f(o, "gate2x_peak_mfe_usdt")
    direction = str(o.get("direction") or "")
    entry = _f(o, "entry_price")
    exitp = _f(o, "exit_price")
    px_up = exitp > entry
    dir_ok = (direction == "LONG" and px_up) or (direction == "SHORT" and not px_up)
    path_cleared = mfe >= (fric + MFE_FLOOR_USDT)
    tags: List[str] = []

    # Execution: friction dominates otherwise-useful or already-negative gross.
    if (gross > 0 and net < 0) or (gross <= 0 and fric > abs(gross)):
        tags.append("EXECUTION_FRICTION")

    if (not dir_ok) and gross < 0:
        tags.append("DIRECTIONAL_FAILURE")

    if path_cleared and net < 0:
        tags.append("TIMING_EXIT_GEOMETRY")

    if str(o.get("exit_reason") or "") == "MAX_HORIZON_60M_EXPIRED" and net < 0:
        tags.append("OUTCOME_HORIZON_MISMATCH")

    sci = o.get("sci_adapter_rec") if isinstance(o.get("sci_adapter_rec"), dict) else {}
    p_hat = _f(sci, "p_hat")
    if sci.get("band") == "ENTER" and p_hat < 0.40 and net < 0:
        # Soft selection flag — ENTER band itself is often <0.40; use as co-evidence only.
        tags.append("SELECTION_WEAK_ENTER")

    if str(o.get("data_quality_status") or "VALID") not in ("VALID", ""):
        tags.append("DATA_MEASUREMENT")

    if not tags:
        tags.append("UNCLASSIFIED_LOSS")

    # Primary priority: friction → directional → timing → horizon → selection → other
    priority = [
        "EXECUTION_FRICTION",
        "DIRECTIONAL_FAILURE",
        "TIMING_EXIT_GEOMETRY",
        "OUTCOME_HORIZON_MISMATCH",
        "SELECTION_WEAK_ENTER",
        "DATA_MEASUREMENT",
        "UNCLASSIFIED_LOSS",
    ]
    primary = next((p for p in priority if p in tags), tags[0])
    return primary, tags


def observe(book: str) -> Dict[str, Any]:
    rel = BOOKS[book]
    root = CRYPTO_ROOT / rel
    outs = _load_jsonl(root / "trade_plan_outcomes.jsonl")
    primary_c: Counter[str] = Counter()
    all_c: Counter[str] = Counter()
    exit_c: Counter[str] = Counter()
    side_c: Counter[str] = Counter()
    rows: List[Dict[str, Any]] = []
    wins = 0
    for o in outs:
        primary, tags = classify(o)
        primary_c[primary] += 1
        for t in tags:
            all_c[t] += 1
        exit_c[str(o.get("exit_reason") or "NONE")] += 1
        side_c[str(o.get("direction") or "NONE")] += 1
        net = _f(o, "net_pnl_usdt")
        if net >= 0:
            wins += 1
        sci = o.get("sci_adapter_rec") if isinstance(o.get("sci_adapter_rec"), dict) else {}
        rows.append(
            {
                "plan_id": o.get("plan_id"),
                "direction": o.get("direction"),
                "exit_reason": o.get("exit_reason"),
                "gross_pnl_usdt": round(_f(o, "gross_pnl_usdt"), 4),
                "friction_usdt": round(_f(o, "friction_usdt"), 4),
                "net_pnl_usdt": round(net, 4),
                "peak_mfe_usdt": round(_f(o, "gate2x_peak_mfe_usdt"), 4),
                "p_hat": round(_f(sci, "p_hat"), 4) if sci else None,
                "band": sci.get("band") if sci else None,
                "primary": primary,
                "tags": tags,
            }
        )

    n = len(outs)
    losses = n - wins
    sum_g = sum(_f(o, "gross_pnl_usdt") for o in outs)
    sum_f = sum(_f(o, "friction_usdt") for o in outs)
    sum_n = sum(_f(o, "net_pnl_usdt") for o in outs)
    path_clear_losses = sum(
        1
        for o in outs
        if _f(o, "net_pnl_usdt") < 0
        and _f(o, "gate2x_peak_mfe_usdt") >= _f(o, "friction_usdt") + MFE_FLOOR_USDT
    )
    return {
        "generated_utc": datetime.now(timezone.utc).isoformat(),
        "book": book,
        "overlay_dir": str(root),
        "n_closed": n,
        "wins": wins,
        "losses": losses,
        "wl_ratio": (wins / losses) if losses else None,
        "sum_gross_usdt": round(sum_g, 4),
        "sum_friction_usdt": round(sum_f, 4),
        "sum_net_usdt": round(sum_n, 4),
        "primary_counts": dict(primary_c),
        "tag_counts": dict(all_c),
        "exit_reason_counts": dict(exit_c),
        "side_counts": dict(side_c),
        "losses_with_mfe_clearing_fric_plus_floor": path_clear_losses,
        "mfe_floor_usdt": MFE_FLOOR_USDT,
        "events": rows,
    }


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--book", default="SCI", choices=sorted(BOOKS.keys()))
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()
    rep = observe(args.book)
    if args.json:
        # omit full event dump for stdout size unless small
        slim = {k: v for k, v in rep.items() if k != "events"}
        slim["events_n"] = len(rep["events"])
        print(json.dumps(slim, indent=2, sort_keys=True))
        return 0
    print(
        f"[RP-608-SCI-EVENT-OBSERVE] book={rep['book']} n={rep['n_closed']} "
        f"W/L={rep['wins']}/{rep['losses']} net={rep['sum_net_usdt']} "
        f"fric={rep['sum_friction_usdt']} gross={rep['sum_gross_usdt']}"
    )
    print(f"  primary: {rep['primary_counts']}")
    print(f"  tags:    {rep['tag_counts']}")
    print(f"  exits:   {rep['exit_reason_counts']}")
    print(f"  sides:   {rep['side_counts']}")
    print(
        f"  losses with MFE≥fric+{MFE_FLOOR_USDT}: "
        f"{rep['losses_with_mfe_clearing_fric_plus_floor']}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
