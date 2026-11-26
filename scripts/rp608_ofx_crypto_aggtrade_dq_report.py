#!/usr/bin/env python3
"""
RP-608-OFX — DQ / continuity report for crypto aggTrade shadow journal.

Feeds the existing executed_flow aggregator (does not rebuild it).
Research-only · authority 0% · baseline OF inert.
"""

from __future__ import annotations

import argparse
import json
import sys
from collections import Counter
from pathlib import Path
from typing import Any, Dict, List, Optional

WORKSPACE = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(WORKSPACE))

from research.rp608_ofx.executed_flow import (  # noqa: E402
    aggregate_executed_flow,
    trades_from_aggtrade_records,
)


def load_rows(path: Path, *, limit: Optional[int] = None) -> List[Dict[str, Any]]:
    rows: List[Dict[str, Any]] = []
    if not path.exists():
        return rows
    with path.open(encoding="utf-8") as f:
        for i, line in enumerate(f):
            if limit is not None and i >= limit:
                break
            line = line.strip()
            if not line:
                continue
            try:
                rows.append(json.loads(line))
            except json.JSONDecodeError:
                rows.append({"_parse_error": True, "raw": line[:200]})
    return rows


def dq_report(rows: List[Dict[str, Any]], *, gap_flag_ms: int = 5_000) -> Dict[str, Any]:
    parse_fail = sum(1 for r in rows if r.get("_parse_error"))
    good = [r for r in rows if not r.get("_parse_error")]
    ids = [r.get("agg_trade_id") for r in good if r.get("agg_trade_id") is not None]
    id_counts = Counter(ids)
    dupes = sum(1 for _, c in id_counts.items() if c > 1)

    times = [int(r["trade_time_ms"]) for r in good if r.get("trade_time_ms")]
    gaps = [times[i] - times[i - 1] for i in range(1, len(times))]
    gaps_pos = [g for g in gaps if g >= 0]
    backward = sum(1 for g in gaps if g < 0)
    large_gaps = sum(1 for g in gaps_pos if g >= gap_flag_ms)

    m_ok = sum(1 for r in good if r.get("is_buyer_maker") is not None)
    buy = sum(1 for r in good if r.get("aggressor") == "BUY")
    sell = sum(1 for r in good if r.get("aggressor") == "SELL")
    qty_side_ok = 0
    qty_side_bad = 0
    for r in good:
        q = r.get("quantity")
        m = r.get("is_buyer_maker")
        ag = r.get("aggressor")
        try:
            qf = float(q)
        except (TypeError, ValueError):
            qty_side_bad += 1
            continue
        if qf <= 0 or m is None:
            qty_side_bad += 1
            continue
        expect = "SELL" if m else "BUY"
        if ag == expect:
            qty_side_ok += 1
        else:
            qty_side_bad += 1

    status = "OK"
    if not good:
        status = "EMPTY"
    elif parse_fail or dupes or backward or qty_side_bad:
        status = "ISSUES"

    return {
        "n_rows": len(rows),
        "n_parse_failures": parse_fail,
        "n_good": len(good),
        "n_unique_agg_ids": len(id_counts),
        "n_duplicate_agg_id_rows": dupes,
        "n_trade_time_backward": backward,
        "n_gaps_ge_flag_ms": large_gaps,
        "gap_flag_ms": gap_flag_ms,
        "max_gap_ms": max(gaps_pos) if gaps_pos else None,
        "median_gap_ms": sorted(gaps_pos)[len(gaps_pos) // 2] if gaps_pos else None,
        "maker_flag_present": m_ok,
        "buy_aggressor_rows": buy,
        "sell_aggressor_rows": sell,
        "qty_side_mapping_ok": qty_side_ok,
        "qty_side_mapping_bad": qty_side_bad,
        "status": status,
    }


def sample_aggregate(rows: List[Dict[str, Any]], *, symbol: str) -> Optional[Dict[str, Any]]:
    good = [r for r in rows if not r.get("_parse_error")]
    if not good:
        return None
    # Map journal rows to aggTrade-shaped dicts for existing parser
    shaped = []
    for r in good:
        raw = r.get("raw_ref") or {}
        shaped.append(
            {
                "s": r.get("symbol") or raw.get("s") or symbol,
                "a": r.get("agg_trade_id") if r.get("agg_trade_id") is not None else raw.get("a"),
                "p": r.get("price") if r.get("price") is not None else raw.get("p"),
                "q": r.get("quantity") if r.get("quantity") is not None else raw.get("q"),
                "m": r.get("is_buyer_maker") if r.get("is_buyer_maker") is not None else raw.get("m"),
                "E": r.get("event_time_ms") or raw.get("E"),
                "T": r.get("trade_time_ms") or raw.get("T"),
            }
        )
    trades = trades_from_aggtrade_records(shaped, default_symbol=symbol)
    if not trades:
        return None
    t0 = min(t.timestamp_ms for t in trades)
    t1 = max(t.timestamp_ms for t in trades) + 1
    result = aggregate_executed_flow(
        trades,
        instrument_id=symbol.upper(),
        window_start_ms=t0,
        window_end_ms=t1,
        source="binance_aggTrade_journal",
    )
    return result.to_dict()


def main() -> int:
    ap = argparse.ArgumentParser(description="OFX crypto aggTrade journal DQ report")
    ap.add_argument(
        "--journal",
        type=Path,
        default=Path("live_capture/shadow_validation/crypto/ofx_aggtrade_capture_v1/aggtrades.jsonl"),
    )
    ap.add_argument("--out", type=Path, default=Path("artifacts/rp608_ofx/crypto_aggtrade_dq_report.json"))
    ap.add_argument("--symbol", default="BTCUSDT")
    ap.add_argument("--gap-flag-ms", type=int, default=5000)
    ap.add_argument("--limit", type=int, default=None)
    args = ap.parse_args()

    rows = load_rows(args.journal, limit=args.limit)
    report: Dict[str, Any] = {
        "protocol_id": "RP-608-OFX",
        "phase": "CRYPTO_AGGTRADE_DQ_REPORT",
        "journal": str(args.journal),
        "journal_exists": args.journal.exists(),
        "authority_pct": 0.0,
        "baseline_of_inert": True,
        "dq": dq_report(rows, gap_flag_ms=args.gap_flag_ms),
        "sample_executed_flow_window": sample_aggregate(rows, symbol=args.symbol),
        "notes": [
            "Uses existing research.rp608_ofx.executed_flow aggregator — library not rebuilt.",
            "Does not modify baseline OF, H2, Gate 2, or execution authority.",
        ],
    }
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    md = args.out.with_suffix(".md")
    dq = report["dq"]
    md.write_text(
        "\n".join(
            [
                "# RP-608-OFX — Crypto aggTrade DQ Report",
                "",
                f"**Journal:** `{args.journal}`",
                f"**Status:** `{dq['status']}`",
                f"**Authority:** 0.0% · baseline OF inert",
                "",
                "```json",
                json.dumps(dq, indent=2),
                "```",
                "",
            ]
        ),
        encoding="utf-8",
    )
    print(json.dumps({"status": dq["status"], "out": str(args.out), "n": dq["n_rows"]}, indent=2))
    return 0 if dq["status"] in ("OK", "EMPTY") else 1


if __name__ == "__main__":
    raise SystemExit(main())
