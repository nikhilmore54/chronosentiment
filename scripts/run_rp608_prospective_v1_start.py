#!/usr/bin/env python3
"""
Prescribed Prospective V1 market commencement CLI (start-rule v1.0).

Does not alter strategy/thresholds. Authorization and preflight alone cannot
start the clock — this command with --confirm is required per market.

Examples:
  python3 scripts/run_rp608_prospective_v1_start.py --market NSE --operator Anuj --confirm
  python3 scripts/run_rp608_prospective_v1_start.py --market CRYPTO --operator Anuj --confirm
  python3 scripts/run_rp608_prospective_v1_start.py --market NSE --operator Anuj --dry-run
"""

from __future__ import annotations

import argparse
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

WORKSPACE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE))

from scripts.rp608_prospective_v1 import (  # noqa: E402
    PROTOCOL_ID,
    START_RULE_VERSION,
    Market,
    commence_market,
    commencement_state,
    evaluate_start_eligibility,
    is_market_started,
    load_start_event,
    run_prestart_gates,
)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--market", required=True, choices=["NSE", "CRYPTO"])
    ap.add_argument("--operator", required=True)
    ap.add_argument("--confirm", action="store_true")
    ap.add_argument("--dry-run", action="store_true")
    ap.add_argument("--expected-deployed-rev", default=None)
    args = ap.parse_args()

    market = Market.NSE if args.market == "NSE" else Market.CRYPTO
    now = datetime.now(timezone.utc)
    gates = run_prestart_gates(
        market=market, expected_deployed_rev=args.expected_deployed_rev
    )
    eligibility = evaluate_start_eligibility(market, decision_utc=now)

    print(
        json.dumps(
            {
                "protocol_id": PROTOCOL_ID,
                "start_rule_version": START_RULE_VERSION,
                "market": market.value,
                "now_utc": now.isoformat(),
                "current_state": commencement_state(market).value,
                "already_started": is_market_started(market),
                "prestart_gate": gates.as_dict(),
                "eligibility": eligibility,
                "dry_run": args.dry_run,
            },
            indent=2,
        )
    )

    if args.dry_run:
        print(
            json.dumps(
                {
                    "would_commence": gates.ok and not is_market_started(market),
                    "note": "dry-run does not persist a start event",
                },
                indent=2,
            )
        )
        return 0 if gates.ok else 1

    result = commence_market(
        market,
        operator=args.operator,
        confirm=bool(args.confirm),
        decision_utc=now,
        expected_deployed_rev=args.expected_deployed_rev,
    )
    print(json.dumps(result, indent=2, default=str))
    if result.get("accepted"):
        # Prove persistence / reload
        reloaded = load_start_event(market)
        print(json.dumps({"persisted": reloaded is not None, "reload_state": (reloaded or {}).get("state")}, indent=2))
        return 0
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
