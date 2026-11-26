#!/usr/bin/env python3
"""
RP-608-OFX Stage 0D — quality / sync / coverage report over inferred-aggressor tick journal.

Research-only. Does not modify OF or execution. Authority 0%.
"""

from __future__ import annotations

import argparse
import json
from collections import Counter, defaultdict
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, Iterable, Optional


def _iter_rows(path: Path, limit: Optional[int] = None) -> Iterable[Dict[str, Any]]:
    n = 0
    with path.open(encoding="utf-8") as fh:
        for line in fh:
            line = line.strip()
            if not line:
                continue
            try:
                yield json.loads(line)
            except json.JSONDecodeError:
                continue
            n += 1
            if limit is not None and n >= limit:
                break


def build_report(path: Path, *, limit: Optional[int] = None) -> Dict[str, Any]:
    update_class = Counter()
    primary_label = Counter()
    trade_like_n = 0
    quote_rep_n = 0
    symbols = Counter()
    sessions = Counter()
    missing_depth = 0
    stale_quote = 0
    quote_ages = []
    exch_backwards = 0
    last_exch: Dict[int, int] = {}
    dup_exact = 0
    prev_key: Dict[int, tuple] = {}
    n = 0
    first_iso = None
    last_iso = None

    for row in _iter_rows(path, limit=limit):
        n += 1
        if first_iso is None:
            first_iso = row.get("receipt_iso")
        last_iso = row.get("receipt_iso")
        uc = row.get("update_class") or "UNKNOWN"
        update_class[uc] += 1
        if row.get("trade_like"):
            trade_like_n += 1
        else:
            quote_rep_n += 1
        prim = (row.get("aggressor_inference_primary") or {}).get("label") or "MISSING"
        primary_label[prim] += 1
        if (row.get("aggressor_inference_same_packet") or {}).get("reason") == "stale_quote":
            stale_quote += 1
        sym = row.get("symbol") or f"token:{row.get('instrument_token')}"
        symbols[sym] += 1
        sessions[row.get("connection_session_id") or "unknown"] += 1
        if int(row.get("bid_levels_qty_nonzero") or 0) < 5 or int(row.get("ask_levels_qty_nonzero") or 0) < 5:
            missing_depth += 1
        age = row.get("quote_age_ms")
        if age is not None:
            try:
                quote_ages.append(float(age))
            except (TypeError, ValueError):
                pass
        tok = int(row.get("instrument_token") or 0)
        exch = int(row.get("exchange_timestamp_sec") or 0)
        if tok and exch:
            prev = last_exch.get(tok)
            if prev is not None and exch < prev:
                exch_backwards += 1
            last_exch[tok] = exch
        key = (
            row.get("last_price"),
            row.get("last_traded_quantity"),
            row.get("last_trade_time_sec"),
            row.get("volume"),
            row.get("best_bid"),
            row.get("best_ask"),
            row.get("exchange_timestamp_sec"),
        )
        if tok in prev_key and prev_key[tok] == key:
            dup_exact += 1
        prev_key[tok] = key

    def pct(a: int, b: int) -> Optional[float]:
        return round(a / b, 4) if b else None

    age_sorted = sorted(quote_ages)
    def pctile(p: float) -> Optional[float]:
        if not age_sorted:
            return None
        idx = min(len(age_sorted) - 1, max(0, int(round((p / 100.0) * (len(age_sorted) - 1)))))
        return age_sorted[idx]

    classified = (
        primary_label.get("BUY_INITIATED_CANDIDATE", 0)
        + primary_label.get("SELL_INITIATED_CANDIDATE", 0)
    )
    ambiguous = primary_label.get("AMBIGUOUS", 0)
    unavailable = primary_label.get("UNAVAILABLE", 0) + primary_label.get("MISSING", 0)

    # Narrow Stage 0D usability heuristics (not Stage 1 authorization)
    trade_like_share = pct(trade_like_n, n)
    ambiguous_share = pct(ambiguous, n)
    gate = {
        "min_rows": n >= 500,
        "trade_like_share_in_band": trade_like_share is not None and 0.05 <= trade_like_share <= 0.98,
        "ambiguous_share_lt_0_40": ambiguous_share is not None and ambiguous_share < 0.40,
        "exch_backwards_lt_1pct": pct(exch_backwards, n) is not None and pct(exch_backwards, n) < 0.01,
        "missing_depth_lt_0_20": pct(missing_depth, n) is not None and pct(missing_depth, n) < 0.20,
    }
    gate_pass = all(gate.values())

    return {
        "protocol_id": "RP-608-OFX",
        "phase": "STAGE_0D_QUALITY_COVERAGE",
        "generated_utc": datetime.now(timezone.utc).isoformat(),
        "journal_path": str(path),
        "n_rows": n,
        "span_receipt_iso": [first_iso, last_iso],
        "n_symbols": len(symbols),
        "n_connection_sessions": len(sessions),
        "update_class_counts": dict(update_class),
        "trade_like": trade_like_n,
        "quote_repeat": quote_rep_n,
        "trade_like_share": trade_like_share,
        "quote_repeat_share": pct(quote_rep_n, n),
        "primary_infer_counts": dict(primary_label),
        "classified_buy_sell": classified,
        "ambiguous": ambiguous,
        "unavailable": unavailable,
        "classified_share": pct(classified, n),
        "ambiguous_share": ambiguous_share,
        "unavailable_share": pct(unavailable, n),
        "stale_quote_same_packet": stale_quote,
        "missing_depth_rows": missing_depth,
        "missing_depth_share": pct(missing_depth, n),
        "exch_timestamp_backwards": exch_backwards,
        "exch_timestamp_backwards_share": pct(exch_backwards, n),
        "exact_duplicate_consecutive": dup_exact,
        "quote_age_ms_p50": pctile(50),
        "quote_age_ms_p95": pctile(95),
        "top_symbols": symbols.most_common(10),
        "quality_gate_checks": gate,
        "quality_gate": "PASS" if gate_pass else "FAIL",
        "stage1_observer": "NOT_AUTHORIZED_BY_0D_ALONE" if gate_pass else "BLOCKED",
        "label": "INFERRED_AGGRESSOR_JOURNAL_QUALITY",
        "non_claims": [
            "not_exchange_aggressor_tape",
            "not_of_paper_rule_change",
            "not_stage1_mechanism_classification",
        ],
    }


def main() -> int:
    ap = argparse.ArgumentParser(description="RP-608-OFX Stage 0D quality report")
    ap.add_argument(
        "--journal",
        type=Path,
        default=Path("live_capture/shadow_validation/nse/ofx_tick_capture_v1/inferred_aggressor_ticks.jsonl"),
    )
    ap.add_argument("--out", type=Path, default=Path("artifacts/rp608_ofx/stage0d_quality_report.json"))
    ap.add_argument("--limit", type=int, default=None)
    args = ap.parse_args()
    if not args.journal.exists():
        report = {
            "protocol_id": "RP-608-OFX",
            "phase": "STAGE_0D_QUALITY_COVERAGE",
            "quality_gate": "FAIL",
            "error": "JOURNAL_MISSING",
            "journal_path": str(args.journal),
        }
    else:
        report = build_report(args.journal, limit=args.limit)
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({k: report.get(k) for k in (
        "quality_gate", "n_rows", "trade_like_share", "ambiguous_share",
        "classified_share", "n_symbols", "span_receipt_iso", "quality_gate_checks",
        "stage1_observer", "error",
    ) if k in report or report.get(k) is not None}, indent=2))
    print(f"WROTE {args.out}")
    return 0 if report.get("quality_gate") == "PASS" else 2


if __name__ == "__main__":
    raise SystemExit(main())
