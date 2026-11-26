#!/usr/bin/env python3
"""
RP-608-OFX Stage 1 — aggression → price-response observational engine.

Reads Stage 0D inferred-aggressor tick journal. Emits per-event observations
and an aggregate summary. No recommendations, orders, OF wiring, or Stage-2
mechanism labels.

Authority: 0.0%
"""

from __future__ import annotations

import argparse
import json
import sys
from collections import defaultdict
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional

WORKSPACE = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(WORKSPACE))

from research.rp608_ofx.observer.aggression_response import (  # noqa: E402
    DEFAULT_HORIZONS_MS,
    aggregate_observations,
    observe_event,
    tick_from_row,
)


def load_points_by_symbol(path: Path, *, limit: Optional[int] = None) -> Dict[str, list]:
    by_sym: Dict[str, list] = defaultdict(list)
    n = 0
    with path.open(encoding="utf-8") as fh:
        for line in fh:
            line = line.strip()
            if not line:
                continue
            try:
                row = json.loads(line)
            except json.JSONDecodeError:
                continue
            pt = tick_from_row(row)
            if pt is None:
                continue
            key = pt.symbol or f"token:{pt.instrument_token}"
            by_sym[key].append(pt)
            n += 1
            if limit is not None and n >= limit:
                break
    for sym in by_sym:
        by_sym[sym].sort(key=lambda p: p.receipt_utc_ms)
    return dict(by_sym)


def run_observer(
    journal: Path,
    *,
    out_events: Path,
    out_summary: Path,
    limit: Optional[int] = None,
    max_events: Optional[int] = None,
    horizons_ms: tuple = DEFAULT_HORIZONS_MS,
    flat_bps: float = 2.0,
) -> Dict[str, Any]:
    by_sym = load_points_by_symbol(journal, limit=limit)
    events: List[Dict[str, Any]] = []
    out_events.parent.mkdir(parents=True, exist_ok=True)
    with out_events.open("w", encoding="utf-8") as out_fh:
        for sym, points in by_sym.items():
            for i in range(len(points)):
                obs = observe_event(points, i, horizons_ms=horizons_ms, flat_bps=flat_bps)
                if obs is None:
                    continue
                events.append(obs)
                out_fh.write(json.dumps(obs, separators=(",", ":")) + "\n")
                if max_events is not None and len(events) >= max_events:
                    break
            if max_events is not None and len(events) >= max_events:
                break

    agg = aggregate_observations(events, horizons_ms=horizons_ms)
    summary = {
        "protocol_id": "RP-608-OFX",
        "phase": "STAGE_1_AGGRESSION_RESPONSE_OBSERVER",
        "generated_utc": datetime.now(timezone.utc).isoformat(),
        "authority_pct": 0.0,
        "journal_path": str(journal),
        "events_path": str(out_events),
        "n_symbols_loaded": len(by_sym),
        "n_ticks_loaded": sum(len(v) for v in by_sym.values()),
        "horizons_ms": list(horizons_ms),
        "flat_bps": flat_bps,
        "aggregate": agg,
        "governance": {
            "baseline_untouched": True,
            "stage2_mechanism_labels": "NOT_ASSIGNED",
            "recommendations": False,
            "orders": False,
            "gate_r": "UNREACHED",
            "step3_plus": "HELD",
        },
        "initial_question": (
            "When aggressive buying or selling occurs (inferred), does the subsequent "
            "price response contain stable, incremental information?"
        ),
        "answer_status": agg.get("stability_read"),
    }
    out_summary.parent.mkdir(parents=True, exist_ok=True)
    out_summary.write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
    return summary


def main() -> int:
    ap = argparse.ArgumentParser(description="RP-608-OFX Stage 1 aggression→response observer")
    ap.add_argument(
        "--journal",
        type=Path,
        default=Path("live_capture/shadow_validation/nse/ofx_tick_capture_v1/inferred_aggressor_ticks.jsonl"),
    )
    ap.add_argument(
        "--out-events",
        type=Path,
        default=Path("artifacts/rp608_ofx/stage1_aggression_response_events.jsonl"),
    )
    ap.add_argument(
        "--out-summary",
        type=Path,
        default=Path("artifacts/rp608_ofx/stage1_aggression_response_summary.json"),
    )
    ap.add_argument("--limit-ticks", type=int, default=None, help="Max ticks to read from journal")
    ap.add_argument("--max-events", type=int, default=None, help="Cap emitted observations")
    ap.add_argument("--flat-bps", type=float, default=2.0)
    args = ap.parse_args()
    if not args.journal.exists():
        print(json.dumps({"error": "JOURNAL_MISSING", "path": str(args.journal)}))
        return 2
    summary = run_observer(
        args.journal,
        out_events=args.out_events,
        out_summary=args.out_summary,
        limit=args.limit_ticks,
        max_events=args.max_events,
        flat_bps=args.flat_bps,
    )
    agg = summary.get("aggregate") or {}
    print(
        json.dumps(
            {
                "n_events": agg.get("n_events"),
                "stability_read": agg.get("stability_read"),
                "by_aggression_label": agg.get("by_aggression_label"),
                "horizons": {
                    h: {
                        k: (agg.get("horizons") or {}).get(h, {}).get(k)
                        for k in (
                            "n_with_forward",
                            "with_share",
                            "against_share",
                            "flat_share",
                            "mean_signed_return_bps",
                            "aligned_share",
                        )
                    }
                    for h in ("1000", "5000", "15000", "30000")
                },
                "out_summary": str(args.out_summary),
                "out_events": str(args.out_events),
            },
            indent=2,
        )
    )
    print(f"WROTE {args.out_summary}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
