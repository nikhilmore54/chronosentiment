#!/usr/bin/env python3
"""
P1 Python behavioural reference: OF ±0.30 marks linked to market events + depth.

Uses frozen scripts/rp608_order_flow_mark.py. No threshold / reverse-OF / paper changes.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional

_ROOT = Path(__file__).resolve().parents[1]
if str(_ROOT) not in sys.path:
    sys.path.insert(0, str(_ROOT))

from scripts.rp608_nse_market_event_ref import (  # noqa: E402
    MarketEventEngineRef,
    encode_full_mode_packet,
    write_p0_fixtures,
)
from scripts.rp608_order_flow_mark import (  # noqa: E402
    ORDER_FLOW_IMBALANCE_THRESHOLD,
    ORDER_FLOW_LABEL,
    classify_order_flow_support,
    inspect_order_flow_book,
    is_order_flow_exit_signal,
    l1_imbalance,
)


def l1_from_event(ev: Dict[str, Any]) -> Dict[str, Any]:
    return {
        "bid_qty": ev["bid_qty"],
        "ask_qty": ev["ask_qty"],
        "bid_price": ev["best_bid"],
        "ask_price": ev["best_ask"],
        "source": ev.get("source") or "fixture_replay",
    }


def of_mark_for_accepted(
    row: Dict[str, Any],
    *,
    primary_side: str = "LONG",
    is_current: bool,
    suppressed_reason: Optional[str],
) -> Dict[str, Any]:
    ev = row["event"]
    insp = inspect_order_flow_book(l1_from_event(ev), primary_side=primary_side)
    return {
        "ingest_seq": ev["ingest_seq"],
        "content_fp": ev["content_fp"],
        "instrument_token": ev["instrument_token"],
        "exchange_timestamp_utc_ms": ev["exchange_timestamp_utc_ms"],
        "receipt_utc_ms": ev["receipt_utc_ms"],
        "quote_source": ev.get("source"),
        "inspection": insp,
        "is_current": bool(is_current),
        "suppressed_reason": suppressed_reason,
    }


class OfLinkedEngineRef(MarketEventEngineRef):
    """Extends P0 engine with OF marks + current-OF suppression while stale."""

    def __init__(self) -> None:
        super().__init__()
        self.of_history: List[Dict[str, Any]] = []
        self.current_of: Dict[int, Dict[str, Any]] = {}

    def on_disconnect(self) -> None:
        super().on_disconnect()
        self.current_of.clear()

    def begin_reconcile(self) -> None:
        super().begin_reconcile()
        self.current_of.clear()

    def complete_reconcile(self) -> None:
        super().complete_reconcile()
        self.current_of.clear()

    def ingest_packet(self, pkt: bytes, **kwargs: Any) -> Optional[Dict[str, Any]]:
        row = super().ingest_packet(pkt, **kwargs)
        if row is None:
            return None
        tok = int(row["event"]["instrument_token"])
        stale = bool(row["observed_while_stale"])
        reason = self.health.stale_reason if stale else None
        ofm = of_mark_for_accepted(
            row, is_current=not stale, suppressed_reason=reason
        )
        row["of_mark"] = ofm
        self.of_history.append(ofm)
        if ofm["is_current"]:
            self.current_of[tok] = ofm
        else:
            self.current_of.pop(tok, None)
        return row

    def current_of_mark(self, token: int) -> Optional[Dict[str, Any]]:
        if self.health.token_stale(token):
            return None
        m = self.current_of.get(int(token))
        if m and m.get("is_current"):
            return m
        return None


def write_p1_fixtures(out_dir: Path) -> Path:
    out_dir.mkdir(parents=True, exist_ok=True)
    # Ensure P0 packets exist
    write_p0_fixtures(out_dir)
    p0 = json.loads((out_dir / "p0_kite_full_differential.json").read_text(encoding="utf-8"))

    # Boundary + support cases (no micro conflict: bid < ask)
    cases = [
        {"name": "long_strong", "bid_qty": 90, "ask_qty": 10, "side": "LONG"},
        {"name": "long_boundary_030", "bid_qty": 65, "ask_qty": 35, "side": "LONG"},
        {"name": "long_below", "bid_qty": 64, "ask_qty": 36, "side": "LONG"},
        {"name": "short_strong", "bid_qty": 10, "ask_qty": 90, "side": "SHORT"},
        {"name": "short_boundary", "bid_qty": 35, "ask_qty": 65, "side": "SHORT"},
        {"name": "neutral", "bid_qty": 50, "ask_qty": 50, "side": "LONG"},
        {"name": "empty_book", "bid_qty": 0, "ask_qty": 0, "side": "LONG"},
    ]
    classify_cases = []
    for c in cases:
        l1 = {
            "bid_qty": c["bid_qty"],
            "ask_qty": c["ask_qty"],
            "bid_price": 99.9,
            "ask_price": 100.1,
            "source": "fixture_replay",
        }
        if c["name"] == "empty_book":
            l1["bid_price"] = 0.0
            l1["ask_price"] = 0.0
        classify_cases.append(
            {
                **c,
                "imbalance": l1_imbalance(c["bid_qty"], c["ask_qty"]),
                "classify": classify_order_flow_support(c["side"], l1),
                "inspect": inspect_order_flow_book(l1, primary_side=c["side"]),
                "exit_from_long": is_order_flow_exit_signal("LONG", l1),
            }
        )

    eng = OfLinkedEngineRef()
    pkt_a = bytes.fromhex(p0["packets"]["pkt_a_hex"])
    pkt_b = bytes.fromhex(p0["packets"]["pkt_b_hex"])
    # Build a strong LONG book packet for current-OF
    buy = [(100.0 - i * 0.05, 90 if i == 0 else 10) for i in range(5)]
    sell = [(100.1 + i * 0.05, 10 if i == 0 else 10) for i in range(5)]
    pkt_long = encode_full_mode_packet(
        instrument_token=6401,
        last_price=100.05,
        volume=1,
        exchange_timestamp_sec=1_700_000_010,
        depth_buy=buy,
        depth_sell=sell,
    )
    receipt = 1_700_000_010_500
    eng.ingest_packet(pkt_long, receipt_utc_ms=receipt, symbol="TEST.NS")
    before_disconnect = eng.current_of_mark(6401)
    eng.on_disconnect()
    during_stale = eng.current_of_mark(6401)
    # Ingest while stale — recorded but not current
    eng.ingest_packet(pkt_a, receipt_utc_ms=receipt + 1000, symbol="TEST.NS")
    while_stale_acc = eng.accepted[-1]
    eng.begin_reconcile()
    during_recon = eng.current_of_mark(6401)
    eng.complete_reconcile()
    after_recon = eng.current_of_mark(6401)
    eng.ingest_packet(pkt_b, receipt_utc_ms=receipt + 2000, symbol="TEST.NS")
    after_live = eng.current_of_mark(6401)

    linked = []
    for row in eng.accepted:
        ofm = row["of_mark"]
        snap = row["depth_snapshot"]
        linked.append(
            {
                "ingest_seq": row["event"]["ingest_seq"],
                "content_fp": row["event"]["content_fp"],
                "depth_ingest_seq": snap.get("ingest_seq"),
                "depth_content_fp": snap.get("content_fp"),
                "of_ingest_seq": ofm["ingest_seq"],
                "of_content_fp": ofm["content_fp"],
                "of_is_current": ofm["is_current"],
                "order_flow_supported": ofm["inspection"]["order_flow_supported"],
                "order_flow_imbalance": ofm["inspection"]["order_flow_imbalance"],
                "depth_levels_buy": len(snap["depth_buy"]),
                "depth_levels_sell": len(snap["depth_sell"]),
            }
        )

    golden = {
        "protocol": "RP608-NSE-P1-OF-DEPTH-LINK",
        "threshold": ORDER_FLOW_IMBALANCE_THRESHOLD,
        "label": ORDER_FLOW_LABEL,
        "classify_cases": classify_cases,
        "packets": {
            **p0["packets"],
            "pkt_long_hex": pkt_long.hex(),
        },
        "engine": {
            "linked": linked,
            "before_disconnect_has_current": before_disconnect is not None,
            "during_stale_current": during_stale,
            "while_stale_of_is_current": while_stale_acc["of_mark"]["is_current"],
            "while_stale_suppressed_reason": while_stale_acc["of_mark"]["suppressed_reason"],
            "during_recon_current": during_recon,
            "after_recon_current_cleared": after_recon is None,
            "after_live_has_current": after_live is not None,
            "after_live_imbalance": (after_live or {}).get("inspection", {}).get(
                "order_flow_imbalance"
            ),
            "of_history_len": len(eng.of_history),
            "depth_history_len": len(eng.depth_history),
            "accepted_len": len(eng.accepted),
        },
    }
    path = out_dir / "p1_of_depth_differential.json"
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
    print(write_p1_fixtures(out))
