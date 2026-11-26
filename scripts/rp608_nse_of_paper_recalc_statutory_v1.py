#!/usr/bin/env python3
"""
Recalculate all NSE OF paper closed trades under NSE_OF_TOUCH_PLUS_STATUTORY_V1.

Preserves original of_trade_outcomes.jsonl untouched.
Writes parallel comparison artifact + V1 summary.
"""

from __future__ import annotations

import json
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT))

from scripts.rp608_nse_equity_costs import (  # noqa: E402
    calc_nse_of_paper_accounting,
    executed_orders_from_ledger_trade,
)

LEDGER = ROOT / "live_capture" / "shadow_validation" / "nse" / "of_paper_ledger_v1"


def main() -> int:
    src = Path(sys.argv[1]) if len(sys.argv) > 1 else LEDGER / "of_trade_outcomes.jsonl"
    out_dir = src.parent
    rows: List[Dict[str, Any]] = []
    for line in src.read_text(encoding="utf-8").splitlines():
        if line.strip():
            rows.append(json.loads(line))

    recalc_path = out_dir / "of_trade_outcomes_recalc_statutory_v1.jsonl"
    summary_path = out_dir / "of_paper_accounting_v1_summary.json"

    cum_gross = cum_stat = cum_net = cum_legacy_net = cum_overhead = 0.0
    n_wins = 0
    out_rows = []
    with recalc_path.open("w", encoding="utf-8") as f:
        for o in rows:
            entry = float(o["entry_price"])
            exit_px = float(o["exit_price"])
            qty = float(o.get("exit_quantity") or o.get("entry_paper_quantity") or 0)
            direction = str(o.get("direction") or "LONG")
            legs = executed_orders_from_ledger_trade(o)
            acct = calc_nse_of_paper_accounting(
                entry_price=entry,
                exit_price=exit_px,
                quantity=qty,
                direction=direction,
                executed_orders=legs,
            )
            legacy_net = float(o.get("net_pnl_inr") or 0.0)
            legacy_fric = float(o.get("friction_inr") or 0.0)
            row = {
                "of_trade_id": o.get("of_trade_id"),
                "symbol": o.get("symbol"),
                "entry_price": entry,
                "exit_price": exit_px,
                "quantity": qty,
                "exit_reason": o.get("exit_reason"),
                "paper_close_timestamp": o.get("paper_close_timestamp"),
                "legacy_journal": {
                    "gross_pnl_inr": o.get("gross_pnl_inr"),
                    "friction_inr": legacy_fric,
                    "net_pnl_inr": legacy_net,
                },
                **acct,
                "delta_net_vs_legacy_inr": round(float(acct["all_in_net_pnl_inr"]) - legacy_net, 2),
                "delta_overhead_vs_legacy_fric_inr": round(
                    float(acct["statutory_overhead_inr"]) - legacy_fric, 2
                ),
                "source_journal_preserved": True,
            }
            f.write(json.dumps(row, separators=(",", ":")) + "\n")
            out_rows.append(row)
            cum_gross += float(acct["touch_gross_pnl_inr"])
            cum_overhead += float(acct["statutory_overhead_inr"])
            cum_stat += float(acct["statutory_net_pnl_inr"])
            cum_net += float(acct["all_in_net_pnl_inr"])
            cum_legacy_net += legacy_net
            if float(acct["all_in_net_pnl_inr"]) > 0:
                n_wins += 1

    summary = {
        "accounting_standard": "NSE_OF_TOUCH_PLUS_STATUTORY_V1",
        "generated_utc": datetime.now(timezone.utc).isoformat(),
        "n_trades": len(rows),
        "n_wins_v1": n_wins,
        "n_losses_v1": len(rows) - n_wins,
        "touch_gross_pnl_inr": round(cum_gross, 2),
        "statutory_overhead_inr": round(cum_overhead, 2),
        "statutory_net_pnl_inr": round(cum_stat, 2),
        "all_in_net_pnl_inr": round(cum_net, 2),
        "legacy_journal_net_pnl_inr": round(cum_legacy_net, 2),
        "delta_all_in_minus_legacy_net_inr": round(cum_net - cum_legacy_net, 2),
        "status": "PROVISIONAL",
        "note": (
            "Original of_trade_outcomes.jsonl unchanged. "
            "V1 = touch gross − statutory (no double spread; slippage sensitivity excluded). "
            "Bulk totals remain PROVISIONAL until production recon gate is accepted."
        ),
        "paths": {
            "source": str(src),
            "recalc": str(recalc_path),
        },
        "governance": {
            "of_pm30_frozen": True,
            "reverse_of_frozen": True,
            "authority_pct": 0.0,
        },
    }
    summary_path.write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(summary, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
