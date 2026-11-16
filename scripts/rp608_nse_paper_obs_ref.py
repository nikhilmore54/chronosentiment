#!/usr/bin/env python3
"""
P3 Python behavioural reference — paper-lifecycle observations from push events.

Mirrors ledger _close_allowed / reverse-OF tick logic for differential tests.
Does NOT write of_trade_outcomes.jsonl (Python ledger remains authoritative elsewhere).
"""

from __future__ import annotations

import json
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Dict, List, Optional

_ROOT = Path(__file__).resolve().parents[1]
if str(_ROOT) not in sys.path:
    sys.path.insert(0, str(_ROOT))

from scripts.rp608_order_flow_mark import (  # noqa: E402
    ORDER_FLOW_EXIT_LABEL,
    classify_order_flow_support,
    is_order_flow_exit_signal,
)

OF_PAPER_MIN_HOLD_MS = 300_000
OF_PAPER_OPPOSITE_PERSIST_MS = 30_000


def close_allowed(
    activated_ms: int,
    opposite_exit_seen_ms: Optional[int],
    now_ms: int,
    exit_signal: bool,
    *,
    min_hold_ms: int = OF_PAPER_MIN_HOLD_MS,
    persist_ms: int = OF_PAPER_OPPOSITE_PERSIST_MS,
) -> tuple[bool, str, Optional[int]]:
    held = max(0, now_ms - activated_ms)
    if held < min_hold_ms:
        return False, f"min_hold_not_met_{held}ms<{min_hold_ms}ms", opposite_exit_seen_ms
    if exit_signal:
        if opposite_exit_seen_ms is None:
            return False, f"opposite_persist_arming_need_{persist_ms}ms", now_ms
        dt = now_ms - int(opposite_exit_seen_ms)
        if dt < persist_ms:
            return False, f"opposite_persist_{dt}ms<{persist_ms}ms", opposite_exit_seen_ms
        return True, "ok", opposite_exit_seen_ms
    return True, "ok", opposite_exit_seen_ms


@dataclass
class PaperObsEngineRef:
    opens: Dict[str, Dict[str, Any]] = field(default_factory=dict)
    feed_stale: bool = False
    observations: List[Dict[str, Any]] = field(default_factory=list)

    def mirror_open(self, open_row: Dict[str, Any]) -> None:
        self.opens[str(open_row["symbol"])] = dict(open_row)

    def on_disconnect(self) -> None:
        self.feed_stale = True

    def complete_reconcile(self) -> None:
        self.feed_stale = False

    def on_depth_event(
        self,
        symbol: str,
        *,
        bid_qty: float,
        ask_qty: float,
        bid_price: float,
        ask_price: float,
        now_ms: int,
        ingest_seq: int,
        content_fp: int,
        exchange_timestamp_utc_ms: int,
        receipt_utc_ms: int,
    ) -> Optional[Dict[str, Any]]:
        prev = self.opens.get(symbol)
        if not prev or not prev.get("active"):
            return None
        if self.feed_stale:
            obs = {
                "symbol": symbol,
                "of_trade_id": prev["of_trade_id"],
                "action": "suppressed_stale",
                "deny_reason": "feed_stale_or_reconciling",
                "exit_signal": False,
                "is_current": False,
                "ingest_seq": ingest_seq,
                "content_fp": content_fp,
                "ledger_writer": "python_authoritative",
            }
            self.observations.append(obs)
            return obs

        l1 = {
            "bid_qty": bid_qty,
            "ask_qty": ask_qty,
            "bid_price": bid_price,
            "ask_price": ask_price,
            "source": "fixture_replay",
        }
        direction = str(prev.get("direction") or "LONG")
        exit_sig = is_order_flow_exit_signal(direction, l1)
        same = classify_order_flow_support(direction, l1)
        if same.get("order_flow_imbalance") is not None:
            prev["last_book_imbalance"] = same["order_flow_imbalance"]
            if same.get("order_flow_supported"):
                prev["order_flow_imbalance"] = same["order_flow_imbalance"]

        action = "hold"
        deny = None
        if exit_sig.get("exit"):
            ok, reason, seen = close_allowed(
                int(prev["activated_ms"]),
                prev.get("opposite_exit_seen_ms"),
                now_ms,
                True,
            )
            prev["opposite_exit_seen_ms"] = seen
            if ok:
                action = "close_opposite_side_order_flow"
                prev["active"] = False
                prev["close_suppressed_reason"] = None
            else:
                action = "close_suppressed"
                deny = reason
                prev["close_suppressed_reason"] = reason
        elif prev.get("opposite_exit_seen_ms") is not None:
            prev["opposite_exit_seen_ms"] = None
            prev.pop("close_suppressed_reason", None)

        self.opens[symbol] = prev
        obs = {
            "symbol": symbol,
            "of_trade_id": prev["of_trade_id"],
            "action": action,
            "deny_reason": deny,
            "exit_signal": bool(exit_sig.get("exit")),
            "opposite_side": exit_sig.get("opposite_side"),
            "order_flow_exit_label": exit_sig.get("order_flow_exit_label"),
            "same_side_supported": bool(same.get("order_flow_supported")),
            "order_flow_imbalance": same.get("order_flow_imbalance"),
            "ingest_seq": ingest_seq,
            "content_fp": content_fp,
            "exchange_timestamp_utc_ms": exchange_timestamp_utc_ms,
            "receipt_utc_ms": receipt_utc_ms,
            "now_ms": now_ms,
            "is_current": True,
            "ledger_writer": "python_authoritative",
            "ORDER_FLOW_EXIT_LABEL": ORDER_FLOW_EXIT_LABEL,
        }
        self.observations.append(obs)
        return obs


def write_p3_fixtures(out_dir: Path) -> Path:
    out_dir.mkdir(parents=True, exist_ok=True)
    act = 1_700_000_000_000
    open_row = {
        "symbol": "TEST.NS",
        "direction": "LONG",
        "of_trade_id": "NSE-OF-PAPER-TEST.NS-10-1",
        "activated_ms": act,
        "entry_price": 100.0,
        "active": True,
        "opposite_exit_seen_ms": None,
    }

    # close_allowed unit cases
    cases = []
    for name, now, seen, exit_sig in [
        ("min_hold", act + 100_000, None, True),
        ("arm", act + 400_000, None, True),
        ("persist_wait", act + 400_000, act + 400_000, True),
        ("fire", act + 400_000 + 30_000, act + 400_000, True),
    ]:
        ok, reason, new_seen = close_allowed(act, seen, now, exit_sig)
        cases.append(
            {
                "name": name,
                "now_ms": now,
                "seen": seen,
                "ok": ok,
                "reason": reason,
                "new_seen": new_seen,
            }
        )

    eng = PaperObsEngineRef()
    eng.mirror_open(open_row)

    # Same-side hold (bid heavy)
    o1 = eng.on_depth_event(
        "TEST.NS",
        bid_qty=90,
        ask_qty=10,
        bid_price=99.9,
        ask_price=100.1,
        now_ms=act + 400_000,
        ingest_seq=1,
        content_fp=111,
        exchange_timestamp_utc_ms=act + 400_000,
        receipt_utc_ms=act + 400_100,
    )
    # Reverse OF — arm
    o2 = eng.on_depth_event(
        "TEST.NS",
        bid_qty=10,
        ask_qty=90,
        bid_price=99.9,
        ask_price=100.1,
        now_ms=act + 400_000,
        ingest_seq=2,
        content_fp=222,
        exchange_timestamp_utc_ms=act + 400_000,
        receipt_utc_ms=act + 400_200,
    )
    # Reverse OF — still waiting persist
    o3 = eng.on_depth_event(
        "TEST.NS",
        bid_qty=10,
        ask_qty=90,
        bid_price=99.9,
        ask_price=100.1,
        now_ms=act + 400_000 + 10_000,
        ingest_seq=3,
        content_fp=333,
        exchange_timestamp_utc_ms=act + 410_000,
        receipt_utc_ms=act + 410_100,
    )
    # Reverse OF — fire
    o4 = eng.on_depth_event(
        "TEST.NS",
        bid_qty=10,
        ask_qty=90,
        bid_price=99.9,
        ask_price=100.1,
        now_ms=act + 400_000 + 30_000,
        ingest_seq=4,
        content_fp=444,
        exchange_timestamp_utc_ms=act + 430_000,
        receipt_utc_ms=act + 430_100,
    )

    eng2 = PaperObsEngineRef()
    eng2.mirror_open(dict(open_row))
    eng2.on_disconnect()
    stale = eng2.on_depth_event(
        "TEST.NS",
        bid_qty=10,
        ask_qty=90,
        bid_price=99.9,
        ask_price=100.1,
        now_ms=act + 500_000,
        ingest_seq=9,
        content_fp=999,
        exchange_timestamp_utc_ms=act + 500_000,
        receipt_utc_ms=act + 500_100,
    )

    golden = {
        "protocol": "RP608-NSE-P3-PAPER-OBS",
        "constants": {
            "min_hold_ms": OF_PAPER_MIN_HOLD_MS,
            "persist_ms": OF_PAPER_OPPOSITE_PERSIST_MS,
        },
        "close_allowed_cases": cases,
        "tick_sequence": [o1, o2, o3, o4],
        "stale_obs": stale,
        "governance": {
            "ledger_writer": "python_authoritative",
            "dual_write": False,
            "cutover": False,
            "authority_pct": 0.0,
        },
    }
    path = out_dir / "p3_paper_obs_differential.json"
    path.write_text(json.dumps(golden, indent=2) + "\n", encoding="utf-8")
    return path


if __name__ == "__main__":
    out = (
        _ROOT
        / "adapters"
        / "chronosentiment"
        / "tests"
        / "fixtures"
        / "nse_kite_full"
    )
    print(write_p3_fixtures(out))
