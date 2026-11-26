#!/usr/bin/env python3
"""Audit NSE OF paper friction: recompute vs journal; component mix; double-count risk."""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any, Dict, List

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT))

from scripts.run_rp608_nse_shadow_daemon import calc_indian_equity_friction  # noqa: E402

OUTCOMES = (
    ROOT
    / "live_capture"
    / "shadow_validation"
    / "nse"
    / "of_paper_ledger_v1"
    / "of_trade_outcomes.jsonl"
)


def main() -> int:
    path = Path(sys.argv[1]) if len(sys.argv) > 1 else OUTCOMES
    rows: List[Dict[str, Any]] = []
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.strip():
            rows.append(json.loads(line))

    agg = {
        k: 0.0
        for k in (
            "brokerage",
            "stt",
            "exchange_charges",
            "sebi_charges",
            "gst",
            "stamp_duty",
            "spread_cost",
            "slippage_cost",
            "total_friction",
        )
    }
    mismatches = []
    pos_gross_neg_net = []
    entry_at_ask = 0
    exit_at_bid = 0

    print(
        "symbol qty entry exit hs_bps journal_fric recomputed delta "
        "spread slip statutory gross net"
    )
    for o in rows:
        entry = float(o["entry_price"])
        exit_px = float(o["exit_price"])
        qty = float(o.get("exit_quantity") or o.get("entry_paper_quantity") or 0)
        hs = float(o.get("half_spread_bps") or 0.0)
        fr = o.get("friction") if isinstance(o.get("friction"), dict) else {}
        recomputed = calc_indian_equity_friction(
            buy_price=entry, sell_price=exit_px, quantity=qty, half_spread_bps=hs
        )
        jtot = float(o.get("friction_inr") or fr.get("total_friction") or 0.0)
        rt = float(recomputed["total_friction"])
        delta = round(jtot - rt, 4)
        if abs(delta) > 0.05:
            mismatches.append(
                {"symbol": o.get("symbol"), "journal": jtot, "recomputed": rt, "delta": delta}
            )
        for k in agg:
            if k in fr:
                agg[k] += float(fr[k])
            else:
                agg[k] += float(recomputed.get(k) or 0.0)

        statutory = sum(
            float(recomputed[k])
            for k in (
                "brokerage",
                "stt",
                "exchange_charges",
                "sebi_charges",
                "gst",
                "stamp_duty",
            )
        )
        spread = float(recomputed["spread_cost"])
        slip = float(recomputed["slippage_cost"])
        gross = float(o.get("gross_pnl_inr") or 0.0)
        net = float(o.get("net_pnl_inr") or 0.0)
        if gross > 0 and net <= 0:
            pos_gross_neg_net.append(
                {
                    "symbol": str(o.get("symbol")).replace(".NS", ""),
                    "gross": gross,
                    "friction": jtot,
                    "net": net,
                }
            )
        ea = o.get("entry_ask")
        eb = o.get("entry_bid")
        if ea is not None and abs(entry - float(ea)) < 1e-6:
            entry_at_ask += 1
        # exit touch often only in depth_quality_exit
        dq = o.get("depth_quality_exit") or {}
        bid = dq.get("best_bid_price")
        if bid is not None and abs(exit_px - float(bid)) < 1e-6:
            exit_at_bid += 1

        sym = str(o.get("symbol") or "").replace(".NS", "")
        print(
            f"{sym} {qty:g} {entry} {exit_px} {hs:.3f} "
            f"{jtot:.2f} {rt:.2f} {delta} {spread:.2f} {slip:.2f} "
            f"{statutory:.2f} {gross} {net}"
        )

    tot = agg["total_friction"] or 1.0
    report = {
        "n_trades": len(rows),
        "component_sums_inr": {k: round(v, 2) for k, v in agg.items()},
        "spread_plus_slippage_share_pct": round(
            100.0 * (agg["spread_cost"] + agg["slippage_cost"]) / tot, 1
        ),
        "statutory_share_pct": round(
            100.0
            * (
                agg["brokerage"]
                + agg["stt"]
                + agg["exchange_charges"]
                + agg["sebi_charges"]
                + agg["gst"]
                + agg["stamp_duty"]
            )
            / tot,
            1,
        ),
        "recompute_mismatches": mismatches,
        "positive_gross_negative_net": pos_gross_neg_net,
        "entry_price_equals_ask_count": entry_at_ask,
        "exit_price_equals_bid_count": exit_at_bid,
        "double_count_risk": {
            "gross_uses_touch_prices": (
                "LONG entry at ask / exit at bid via _quote_px — gross already "
                "reflects one full spread crossing in prices when bid≠ask."
            ),
            "friction_also_adds_half_spread_both_legs": (
                "calc_indian_equity_friction spread_cost = half_spread_bps on buy "
                "and sell turnover — additional to touch-priced gross."
            ),
            "plus_fixed_2bps_slippage_each_leg": True,
            "assessment": (
                "Likely OVERSTATEMENT of execution cost relative to touch fills: "
                "spread (and possibly part of slippage) may be double-counted vs "
                "ask→bid gross. Statutory legs (brokerage/STT/exchange/SEBI/GST/stamp) "
                "are the cleaner cost baseline for OF diagnostics."
            ),
        },
        "governance": {
            "of_pm30_frozen": True,
            "reverse_of_frozen": True,
            "authority_pct": 0.0,
            "note": "Friction audit only — not a rule change.",
        },
    }
    out = path.parent / "of_paper_friction_audit.json"
    out.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(report, indent=2))
    print("wrote", out)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
