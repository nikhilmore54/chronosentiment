#!/usr/bin/env python3
"""Gate 2E allow_only twin attribution — PRIMARY vs BASELINE (never pooled).

Reports PLAN_CREATED vs SUPPRESSED_TRADE_PATTERN since allow_only cut.
W/L-first remains in rp608_volume_pattern_twin_compare.py.
"""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone, timedelta
from pathlib import Path
from typing import Any, Dict, Optional

IST = timezone(timedelta(hours=5, minutes=30))
DEFAULT_ALLOW_ONLY_UTC = "2026-11-21T05:34:00Z"


def _parse_utc(s: str) -> datetime:
    dt = datetime.fromisoformat(s.replace("Z", "+00:00"))
    if dt.tzinfo is None:
        dt = dt.replace(tzinfo=timezone.utc)
    return dt


def _event_ts(row: Dict[str, Any]) -> Optional[datetime]:
    u = row.get("timestamp_utc")
    if u:
        try:
            return _parse_utc(str(u))
        except Exception:
            pass
    ms = row.get("timestamp_ms")
    if ms is not None:
        try:
            return datetime.fromtimestamp(int(ms) / 1000, tz=timezone.utc)
        except Exception:
            pass
    return None


def count_entry_events(events_path: Path, since: datetime) -> Dict[str, Any]:
    created = 0
    suppressed = 0
    last_suppress: Optional[Dict[str, Any]] = None
    last_create: Optional[Dict[str, Any]] = None
    if not events_path.exists():
        return {
            "plan_created": 0,
            "suppressed_trade_pattern": 0,
            "last_create": None,
            "last_suppress": None,
        }
    with events_path.open() as f:
        for line in f:
            if not line.strip():
                continue
            try:
                row = json.loads(line)
            except json.JSONDecodeError:
                continue
            ts = _event_ts(row)
            if ts is None or ts < since:
                continue
            et = row.get("event_type")
            if et == "PLAN_CREATED":
                created += 1
                last_create = {
                    "timestamp_utc": row.get("timestamp_utc"),
                    "plan_id": row.get("plan_id"),
                    "prediction": (row.get("trade_pattern_match") or {}).get(
                        "prediction"
                    ),
                }
            elif et == "SUPPRESSED_TRADE_PATTERN":
                suppressed += 1
                last_suppress = {
                    "timestamp_utc": row.get("timestamp_utc"),
                    "prediction": row.get("prediction")
                    or (row.get("trade_pattern_match") or {}).get("prediction"),
                    "gate_mode": row.get("gate_mode"),
                }
    return {
        "plan_created": created,
        "suppressed_trade_pattern": suppressed,
        "last_create": last_create,
        "last_suppress": last_suppress,
    }


def book_open_flat(state_path: Path) -> Dict[str, Any]:
    if not state_path.exists():
        return {"state": "UNAVAILABLE"}
    try:
        s = json.loads(state_path.read_text())
    except Exception:
        return {"state": "UNAVAILABLE"}
    ap = s.get("active_plan") or {}
    lc = s.get("last_closed_plan") or {}
    if ap.get("state") == "OPEN":
        return {
            "state": "OPEN",
            "plan_id": ap.get("plan_id"),
            "direction": ap.get("direction"),
            "entry_price": ap.get("entry_price"),
            "net_pnl_usdt": (ap.get("valuation") or {}).get("unrealized_net_pnl_usdt"),
            "prediction": (ap.get("trade_pattern_match") or {}).get("prediction"),
        }
    return {
        "state": "FLAT",
        "last_plan_id": lc.get("plan_id"),
        "last_exit_reason": lc.get("exit_reason"),
        "last_net_pnl_usdt": lc.get("net_pnl_usdt"),
    }


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--crypto-root",
        default="live_capture/shadow_validation/crypto",
    )
    ap.add_argument(
        "--allow-only-utc",
        default=DEFAULT_ALLOW_ONLY_UTC,
        help="ISO UTC when PRIMARY allow_only went live",
    )
    args = ap.parse_args()
    since = _parse_utc(args.allow_only_utc)
    root = Path(args.crypto_root)
    primary_ev = root / "trade_plan_overlay_v1" / "trade_plan_events.jsonl"
    baseline_ev = (
        root / "trade_plan_overlay_baseline_parallel_v1" / "trade_plan_events.jsonl"
    )
    out = {
        "allow_only_since_utc": since.isoformat(),
        "primary": {
            **count_entry_events(primary_ev, since),
            **book_open_flat(root / "trade_plan_overlay_v1" / "overlay_state.json"),
            "book": "CRYPTO_PLAN",
            "gate": "allow_only",
        },
        "baseline": {
            **count_entry_events(baseline_ev, since),
            **book_open_flat(
                root
                / "trade_plan_overlay_baseline_parallel_v1"
                / "overlay_state.json"
            ),
            "book": "CRYPTO_PLAN_BASELINE_PARALLEL",
            "gate": "0",
        },
    }
    print(json.dumps(out, indent=2))
    p, b = out["primary"], out["baseline"]
    print("\n# Gate2E twin (never pooled) · since allow_only")
    print(
        f"PRIMARY  state={p.get('state')} create={p['plan_created']} "
        f"suppress={p['suppressed_trade_pattern']}"
    )
    print(
        f"BASELINE state={b.get('state')} create={b['plan_created']} "
        f"suppress={b['suppressed_trade_pattern']}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
