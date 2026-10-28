#!/usr/bin/env python3
"""
Prescribed Prospective V1 authorization CLI.

Validates deployed revision + preflight_pass, then records human provenance.
Does NOT start Session 1 or advance the experiment clock.

Example (on LCUtest):
  python3 scripts/run_rp608_prospective_v1_authorize.py \\
    --authorized-by Anuj \\
    --intent "Authorize RP-608-REGIME-BREAKOUT-001 Prospective V1 after green Pi preflight" \\
    --expected-deployed-rev 680e18640aba2e30d06c3301005bce9c6e32cbd7 \\
    --confirm
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

WORKSPACE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE))

from scripts.rp608_prospective_v1 import (  # noqa: E402
    AUTH_PATH,
    PROTOCOL_ID,
    authorize_prospective_v1,
    is_authorized,
    load_authorization,
    load_preflight,
    read_deployed_rev,
    validate_authorization_record,
)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--authorized-by", required=True, help="Human authorizer name")
    ap.add_argument("--intent", required=True, help="Explicit authorization intent statement")
    ap.add_argument(
        "--expected-deployed-rev",
        required=True,
        help="Full or unambiguous deployed revision that passed preflight",
    )
    ap.add_argument(
        "--confirm",
        action="store_true",
        help="Required explicit confirmation flag",
    )
    ap.add_argument(
        "--dry-run",
        action="store_true",
        help="Validate gates only; do not write authorization",
    )
    args = ap.parse_args()

    deployed = read_deployed_rev()
    preflight = load_preflight()
    prev = load_authorization()

    print(
        json.dumps(
            {
                "protocol_id": PROTOCOL_ID,
                "dry_run": args.dry_run,
                "deployed_rev": deployed,
                "expected_deployed_rev": args.expected_deployed_rev,
                "preflight_pass": preflight.get("preflight_pass"),
                "previous_authorized": prev.get("authorized"),
                "previous_counting": prev.get("observation_counting"),
            },
            indent=2,
        )
    )

    if args.dry_run:
        if not args.confirm:
            print(json.dumps({"dry_run_ok": False, "reason": "confirm_required_even_for_dry_run"}))
            return 2
        if not preflight.get("preflight_pass"):
            print(json.dumps({"dry_run_ok": False, "reason": "preflight_not_pass"}))
            return 1
        print(json.dumps({"dry_run_ok": True, "would_authorize": True, "session_1": False}))
        return 0

    try:
        result = authorize_prospective_v1(
            authorized_by=args.authorized_by,
            intent=args.intent,
            expected_deployed_rev=args.expected_deployed_rev,
            confirm=bool(args.confirm),
            live_order_authority_pct=0.0,
        )
    except Exception as exc:
        print(json.dumps({"ok": False, "error": str(exc)}, indent=2))
        return 1

    auth = load_authorization()
    validation = validate_authorization_record(auth)
    print(
        json.dumps(
            {
                "ok": True,
                "wrote": str(AUTH_PATH),
                "authorized": auth.get("authorized"),
                "authorized_by": auth.get("authorized_by"),
                "authorized_at_utc": auth.get("authorized_at_utc"),
                "deployed_revision": auth.get("deployed_revision"),
                "observation_counting": auth.get("observation_counting"),
                "session_1_started": auth.get("session_1_started"),
                "clock_start": auth.get("clock_start"),
                "is_authorized_helper": is_authorized(),
                "validation": validation,
                "note": (
                    "Authorization recorded. Session 1 not started. "
                    "Integrity window 1–5 forbids tuning once counting begins."
                ),
            },
            indent=2,
        )
    )
    return 0 if validation["ok"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
