#!/usr/bin/env python3
"""
RP-608-POLICY-D-PREOPEN-QA-001 — Capture-quality report (data integrity milestone).

Reads dated REST pre-open snapshots + WS continuity probe for a session date.
Does not evaluate strategy performance. Authority: 0.0%.
"""

from __future__ import annotations

import argparse
import json
import sys
from collections import Counter
from datetime import datetime
from pathlib import Path
from typing import Any, Dict, List, Optional

WORKSPACE_DIR = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(WORKSPACE_DIR))

NSE_DIR = WORKSPACE_DIR / "live_capture" / "shadow_validation" / "nse"
TEST_DIR = NSE_DIR / "test_run"
WS_DIR = NSE_DIR / "preopen_ws_continuity"


def _load_jsonl(path: Path) -> List[Dict[str, Any]]:
    rows: List[Dict[str, Any]] = []
    if not path.exists():
        return rows
    for line in path.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line:
            continue
        try:
            rows.append(json.loads(line))
        except json.JSONDecodeError:
            continue
    return rows


def _snap_paths(session_date: str) -> List[Path]:
    # Dated snapshots: preopen_discovery_snapshot_YYYYMMDDTHHMMSS.jsonl
    day = session_date.replace("-", "")
    out = sorted(TEST_DIR.glob(f"preopen_discovery_snapshot_{day}T*.jsonl"))
    return out


def summarize_rest_snap(rows: List[Dict[str, Any]], label: str) -> Dict[str, Any]:
    n = len(rows)
    prov_counts: Counter = Counter()
    for r in rows:
        fp = r.get("field_provenance") or {}
        if fp:
            for k, v in fp.items():
                prov_counts[f"{k}:{v}"] += 1
        else:
            # legacy
            prov_counts[f"indicative_source:{r.get('indicative_source')}"] += 1
    clearing = sum(1 for r in rows if r.get("indicative_clearing_price") or r.get("indicative_source") == "INDICATIVE_AUCTION_CLEARING" or r.get("indicative_source") == "INDICATIVE_CLEARING_PRICE")
    fallback = sum(1 for r in rows if r.get("indicative_source") == "FALLBACK_LAST_PRICE")
    buy_none = sum(1 for r in rows if r.get("raw_buy_quantity") is None)
    imb_none = sum(1 for r in rows if r.get("raw_total_imbalance_qty") is None)
    imb_zero = sum(1 for r in rows if r.get("raw_total_imbalance_qty") == 0)
    return {
        "label": label,
        "n_rows": n,
        "unique_symbols": len({r.get("symbol") for r in rows}),
        "indicative_clearing_n": clearing,
        "fallback_last_price_n": fallback,
        "buy_qty_missing_n": buy_none,
        "total_imbalance_missing_n": imb_none,
        "total_imbalance_zero_n": imb_zero,
        "imbalance_status": dict(Counter(r.get("imbalance_status") for r in rows)),
        "provenance_counts": dict(prov_counts),
    }


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--session-date", required=True, help="IST session YYYY-MM-DD")
    ap.add_argument(
        "--out",
        default=None,
        help="Output JSON path",
    )
    args = ap.parse_args()
    session = args.session_date
    snaps = _snap_paths(session)
    rest_summaries = []
    for p in snaps:
        rows = _load_jsonl(p)
        rest_summaries.append(summarize_rest_snap(rows, p.name))

    ws_path = WS_DIR / f"preopen_ws_continuity_{session}.json"
    ws_summary: Optional[Dict[str, Any]] = None
    if ws_path.exists():
        ws = json.loads(ws_path.read_text(encoding="utf-8"))
        ws_summary = {k: ws.get(k) for k in ws if k != "per_symbol"}
        # packet continuity buckets
        per = ws.get("per_symbol") or {}
        pkt = [int((v or {}).get("packets") or 0) for v in per.values()]
        ws_summary["packet_histogram"] = {
            "zero": sum(1 for x in pkt if x == 0),
            "1_to_10": sum(1 for x in pkt if 1 <= x <= 10),
            "gt_10": sum(1 for x in pkt if x > 10),
        }

    report = {
        "protocol_id": "RP-608-POLICY-D-PREOPEN-QA-001",
        "authority": "0.0%",
        "milestone": "DATA_INTEGRITY_NOT_STRATEGY_PERFORMANCE",
        "session_date_ist": session,
        "generated_at_utc": datetime.utcnow().isoformat() + "Z",
        "rest_snapshots_found": [p.name for p in snaps],
        "rest_summaries": rest_summaries,
        "ws_continuity_path": str(ws_path) if ws_path.exists() else None,
        "ws_summary": ws_summary,
        "acceptance_checks": {
            "coverage_119_rest": all(s.get("unique_symbols") == 119 for s in rest_summaries) if rest_summaries else False,
            "has_ws_report": ws_path.exists(),
            "has_three_rest_snaps": len(snaps) >= 3,
            "any_true_indicative_clearing": any(s.get("indicative_clearing_n", 0) > 0 for s in rest_summaries),
            "note": "Missing fields must remain missing; FALLBACK_LAST_PRICE is not indicative clearing.",
        },
    }
    out = Path(args.out) if args.out else (
        WORKSPACE_DIR / "artifacts" / "rp608_cross_session_warmup" / f"preopen_qa_{session}.json"
    )
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(json.dumps(report["acceptance_checks"], indent=2))
    print(f"[wrote] {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
