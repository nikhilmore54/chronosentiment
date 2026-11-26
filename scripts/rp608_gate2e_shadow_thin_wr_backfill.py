#!/usr/bin/env python3
"""
Gate 2E — observational backfill of pre-deploy thin-WR suppresses into
`gate2e_shadow_allows.jsonl` for BASELINE join study.

Does NOT loosen live allow_only. Does NOT open trades. Authority 0%.
Books never pooled.

Volume: parse candle vol_surge bucket from pattern_id trailing `V*` axis
(PRIMARY volume-aware keys). Price path is evaluated later via join /
prospective probe — both price + volume required for interpretation.

Usage:
  python3 scripts/research/rp608_gate2e_shadow_thin_wr_backfill.py --dry-run
  python3 scripts/research/rp608_gate2e_shadow_thin_wr_backfill.py --apply
"""

from __future__ import annotations

import argparse
import json
import sys
from collections import Counter
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, Iterable, List, Optional, Set

WORKSPACE = Path(__file__).resolve().parents[2]
CRYPTO = WORKSPACE / "live_capture" / "shadow_validation" / "crypto"
EVENTS = CRYPTO / "trade_plan_overlay_v1" / "trade_plan_events.jsonl"
SHADOW = CRYPTO / "trade_plan_overlay_v1" / "gate2e_shadow_allows.jsonl"

SHADOW_RULE = "SHADOW_ALLOW_IF_N10_WR_GT_035"
MIN_N = 10
MIN_WR = 0.35


def _vol_surge_from_pattern_id(pattern_id: Optional[str]) -> str:
    """Trailing volume axis: …|VHIGH|VMED|VLOW|VUNK."""
    if not pattern_id:
        return "UNK"
    parts = str(pattern_id).split("|")
    if not parts:
        return "UNK"
    last = parts[-1]
    if last.startswith("V") and len(last) > 1:
        return last[1:]  # HIGH / MED / LOW / UNK
    return "UNK"


def _iter_suppress_events(path: Path) -> Iterable[Dict[str, Any]]:
    if not path.exists():
        return
    with open(path, "r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line or "SUPPRESSED_TRADE_PATTERN" not in line:
                continue
            try:
                row = json.loads(line)
            except json.JSONDecodeError:
                continue
            if row.get("event_type") != "SUPPRESSED_TRADE_PATTERN":
                continue
            yield row


def _existing_plan_ids(shadow_path: Path) -> Set[str]:
    out: Set[str] = set()
    if not shadow_path.exists():
        return out
    with open(shadow_path, "r", encoding="utf-8") as f:
        for line in f:
            if not line.strip():
                continue
            try:
                r = json.loads(line)
            except json.JSONDecodeError:
                continue
            pid = str(r.get("plan_id") or "")
            if pid:
                out.add(pid)
    return out


def event_to_shadow_stamp(row: Dict[str, Any]) -> Optional[Dict[str, Any]]:
    pred = str(row.get("prediction") or "")
    if pred != "ABSTAIN_INSUFFICIENT":
        return None
    try:
        n = int(row.get("cohort_n") or 0)
        wr = float(row.get("hist_wr"))
    except (TypeError, ValueError):
        return None
    if n < MIN_N or wr <= MIN_WR:
        return None
    pattern_id = row.get("pattern_id")
    vol = _vol_surge_from_pattern_id(str(pattern_id) if pattern_id else None)
    ts = int(row.get("timestamp_ms") or 0)
    return {
        "shadow_rule_id": SHADOW_RULE,
        "pattern_id": pattern_id,
        "prediction": pred,
        "hist_wr": wr,
        "cohort_n": n,
        "plan_id": row.get("plan_id"),
        "direction": row.get("direction"),
        "suppressed_ms": ts,
        "book_id": row.get("book_id") or "CRYPTO_PLAN",
        "match_mode": row.get("match_mode"),
        "gate_mode": row.get("gate_mode"),
        "label": "INSUFFICIENT",
        "authority_pct": 0.0,
        "opens_trade": False,
        # Price + volume context for study (not OFX)
        "entry_price": row.get("entry_price"),
        "stop_loss_price": row.get("stop_loss_price"),
        "profit_target_price": row.get("profit_target_price"),
        "vol_surge": vol,
        "vol_surge_axis": f"V{vol}",
        "backfill": True,
        "backfill_source": "trade_plan_events.jsonl",
        "backfill_utc": datetime.now(timezone.utc).isoformat().replace("+00:00", "Z"),
    }


def main() -> int:
    ap = argparse.ArgumentParser(description="Backfill Gate2E thin-WR shadow stamps")
    ap.add_argument("--events", type=Path, default=EVENTS)
    ap.add_argument("--shadow", type=Path, default=SHADOW)
    ap.add_argument("--dry-run", action="store_true")
    ap.add_argument("--apply", action="store_true")
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()
    if not args.dry_run and not args.apply:
        args.dry_run = True

    existing = _existing_plan_ids(args.shadow)
    candidates: List[Dict[str, Any]] = []
    vol_counts: Counter = Counter()
    scanned = 0
    for row in _iter_suppress_events(args.events) or []:
        scanned += 1
        stamp = event_to_shadow_stamp(row)
        if stamp is None:
            continue
        pid = str(stamp.get("plan_id") or "")
        if pid and pid in existing:
            continue
        vol_counts[str(stamp.get("vol_surge"))] += 1
        candidates.append(stamp)

    report = {
        "protocol": "RP-608-GATE2E-SHADOW-THIN-WR-BACKFILL",
        "scanned_suppress_events": scanned,
        "already_stamped": len(existing),
        "new_candidates": len(candidates),
        "vol_surge_breakdown": dict(vol_counts),
        "apply": bool(args.apply),
        "note": (
            "Observational backfill only; live allow_only unchanged; "
            "volume from pattern_id V* axis; join to BASELINE for W/L"
        ),
    }

    if args.apply and candidates:
        args.shadow.parent.mkdir(parents=True, exist_ok=True)
        with open(args.shadow, "a", encoding="utf-8") as f:
            for stamp in candidates:
                f.write(json.dumps(stamp, separators=(",", ":")) + "\n")
        report["written"] = len(candidates)
        report["shadow_path"] = str(args.shadow)
    else:
        report["written"] = 0

    if args.json:
        json.dump(report, sys.stdout, indent=2)
        print()
    else:
        print("Gate2E thin-WR shadow backfill (price+volume axes; never pooled)")
        print(f"  events={args.events}")
        print(f"  shadow={args.shadow}")
        print(
            f"  scanned={scanned} existing={len(existing)} "
            f"new={len(candidates)} written={report['written']}"
        )
        print(f"  vol_surge={dict(vol_counts)}")
        if args.dry_run:
            print("  dry-run only — pass --apply to append jsonl")
        if candidates[:3]:
            print("  sample_plan_ids:", [c.get("plan_id") for c in candidates[:3]])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
