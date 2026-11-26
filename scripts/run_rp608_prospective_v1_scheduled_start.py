#!/usr/bin/env python3
"""
Armed scheduled commencement for RP-608 Prospective V1 (start-rule v1.0).

Does NOT start on authorization or cron alone: requires a valid arm file.
Invokes commence_market only when:
  - arm.armed is true
  - market listed and not yet STARTED
  - pre-start gates PASS
  - now is strictly before eligible open, and within lead_minutes of that open
Late / mid-session starts are refused by commence_market (protocol §10).

Typical cron (LCUtest): every 5 minutes.
  */5 * * * * cd /home/ultraspeed/ChronoSentiment_MEGA_FINAL && \\
    /usr/bin/python3 scripts/run_rp608_prospective_v1_scheduled_start.py \\
    >> artifacts/rp608_regime_breakout/prospective_v1/scheduled_start.cron.log 2>&1
"""

from __future__ import annotations

import argparse
import json
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List

WORKSPACE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE))

from scripts.rp608_prospective_v1 import (  # noqa: E402
    PROTOCOL_ID,
    PROSPECTIVE_NS,
    START_RULE_VERSION,
    Market,
    commence_market,
    ensure_namespace,
    evaluate_start_eligibility,
    is_market_started,
    run_prestart_gates,
)

ARM_PATH = PROSPECTIVE_NS / "scheduled_start_arm.json"
LOG_PATH = PROSPECTIVE_NS / "scheduled_start.jsonl"


def _now() -> datetime:
    return datetime.now(timezone.utc)


def load_arm(path: Path = ARM_PATH) -> Dict[str, Any]:
    if not path.exists():
        return {"armed": False, "reason": "arm_file_missing"}
    return json.loads(path.read_text(encoding="utf-8"))


def append_log(record: Dict[str, Any]) -> None:
    ensure_namespace()
    LOG_PATH.parent.mkdir(parents=True, exist_ok=True)
    with LOG_PATH.open("a", encoding="utf-8") as fh:
        fh.write(json.dumps(record) + "\n")


def write_arm(arm: Dict[str, Any], path: Path = ARM_PATH) -> None:
    ensure_namespace()
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(arm, indent=2) + "\n", encoding="utf-8")


def maybe_disarm_if_complete(arm: Dict[str, Any]) -> Dict[str, Any]:
    markets = [str(m).upper() for m in (arm.get("markets") or []) if str(m).upper() in ("NSE", "CRYPTO")]
    if not markets:
        return arm
    if all(is_market_started(Market[m]) for m in markets):
        arm = dict(arm)
        arm["armed"] = False
        arm["disarmed_utc"] = _now().isoformat()
        arm["disarm_reason"] = "all_listed_markets_started"
        write_arm(arm)
    return arm


def evaluate_market_window(
    market: Market,
    *,
    now: datetime,
    lead_minutes: int,
) -> Dict[str, Any]:
    el = evaluate_start_eligibility(market, decision_utc=now)
    open_utc = datetime.fromisoformat(el["eligible_session_open_utc"])
    seconds_to_open = (open_utc - now).total_seconds()
    in_window = 0 < seconds_to_open <= lead_minutes * 60
    return {
        "market": market.value,
        "eligibility": el,
        "open_utc": open_utc.isoformat(),
        "seconds_to_open": seconds_to_open,
        "in_prestart_window": in_window,
        "too_early": seconds_to_open > lead_minutes * 60,
        "too_late_or_at_open": seconds_to_open <= 0,
    }


def run_once(*, dry_run: bool = False, arm_path: Path = ARM_PATH) -> Dict[str, Any]:
    now = _now()
    arm = load_arm(arm_path)
    summary: Dict[str, Any] = {
        "protocol_id": PROTOCOL_ID,
        "start_rule_version": START_RULE_VERSION,
        "now_utc": now.isoformat(),
        "dry_run": dry_run,
        "armed": bool(arm.get("armed")),
        "actions": [],
    }

    if not arm.get("armed"):
        summary["result"] = "skipped_not_armed"
        append_log(summary)
        return summary

    operator = str(arm.get("operator") or "").strip()
    if not operator:
        summary["result"] = "skipped_operator_missing"
        append_log(summary)
        return summary

    lead = int(arm.get("lead_minutes") or 90)
    markets: List[str] = [str(m).upper() for m in (arm.get("markets") or ["NSE", "CRYPTO"])]
    expected_rev = arm.get("expected_deployed_rev")

    for name in markets:
        if name not in ("NSE", "CRYPTO"):
            summary["actions"].append({"market": name, "result": "skipped_unknown_market"})
            continue
        market = Market[name]
        if is_market_started(market):
            summary["actions"].append({"market": name, "result": "already_started"})
            continue

        window = evaluate_market_window(market, now=now, lead_minutes=lead)
        gates = run_prestart_gates(market=market, expected_deployed_rev=expected_rev)

        action: Dict[str, Any] = {
            "market": name,
            "window": window,
            "prestart_ok": gates.ok,
            "prestart_failures": gates.failures,
        }

        if not gates.ok:
            action["result"] = "blocked_prestart"
            summary["actions"].append(action)
            continue
        if window["too_early"]:
            action["result"] = "waiting_for_window"
            summary["actions"].append(action)
            continue
        if window["too_late_or_at_open"]:
            action["result"] = "missed_window_defer_next_eligible"
            summary["actions"].append(action)
            continue
        if not window["in_prestart_window"]:
            action["result"] = "waiting"
            summary["actions"].append(action)
            continue

        if dry_run:
            action["result"] = "would_commence"
            summary["actions"].append(action)
            continue

        # Explicit scheduled confirm — authorized by arm file provenance
        res = commence_market(
            market,
            operator=f"{operator} (scheduled)",
            confirm=True,
            decision_utc=now,
            expected_deployed_rev=expected_rev,
        )
        action["commence"] = {
            "accepted": res.get("accepted"),
            "reason": res.get("reason"),
            "state": res.get("state"),
            "idempotent": res.get("idempotent"),
            "eligible_session_boundary": (res.get("start_event") or {}).get("eligible_session_boundary"),
        }
        action["result"] = "commenced" if res.get("accepted") else "commence_rejected"
        summary["actions"].append(action)

    arm = maybe_disarm_if_complete(arm)
    summary["armed_after"] = bool(arm.get("armed"))
    summary["result"] = "ok"
    append_log(summary)
    return summary


def arm_command(args: argparse.Namespace) -> int:
    ensure_namespace()
    payload = {
        "armed": True,
        "armed_utc": _now().isoformat(),
        "armed_by": args.armed_by,
        "operator": args.operator,
        "markets": args.markets,
        "lead_minutes": args.lead_minutes,
        "expected_deployed_rev": args.expected_deployed_rev,
        "intent": args.intent,
        "protocol_id": PROTOCOL_ID,
        "start_rule_version": START_RULE_VERSION,
        "note": (
            "Scheduler may invoke commence_market with confirm=True only inside "
            "the pre-boundary window. Late starts are rejected."
        ),
    }
    write_arm(payload)
    print(json.dumps({"armed": True, "path": str(ARM_PATH), "arm": payload}, indent=2))
    return 0


def disarm_command(args: argparse.Namespace) -> int:
    arm = load_arm()
    arm["armed"] = False
    arm["disarmed_utc"] = _now().isoformat()
    arm["disarm_reason"] = args.reason
    arm["disarmed_by"] = args.disarmed_by
    write_arm(arm)
    print(json.dumps({"armed": False, "arm": arm}, indent=2))
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    sub = ap.add_subparsers(dest="cmd")

    run_p = sub.add_parser("run", help="Evaluate windows and optionally commence (default)")
    run_p.add_argument("--dry-run", action="store_true")

    arm_p = sub.add_parser("arm", help="Arm scheduled starts (human one-time)")
    arm_p.add_argument("--armed-by", required=True)
    arm_p.add_argument("--operator", required=True)
    arm_p.add_argument("--markets", nargs="+", default=["CRYPTO", "NSE"])
    arm_p.add_argument("--lead-minutes", type=int, default=90)
    arm_p.add_argument("--expected-deployed-rev", required=True)
    arm_p.add_argument(
        "--intent",
        default=(
            "Automate Prospective V1 market starts inside the pre-boundary window "
            "so operator need not be present; does not raise live authority."
        ),
    )

    dis_p = sub.add_parser("disarm", help="Disable scheduled starts")
    dis_p.add_argument("--disarmed-by", required=True)
    dis_p.add_argument("--reason", default="operator_disarm")

    # Default to run if no subcommand (cron-friendly)
    args, unknown = ap.parse_known_args()
    if args.cmd is None:
        # allow bare invocation + optional --dry-run
        bare = argparse.ArgumentParser()
        bare.add_argument("--dry-run", action="store_true")
        bare_args = bare.parse_args()
        out = run_once(dry_run=bare_args.dry_run)
        print(json.dumps(out, indent=2))
        return 0

    if args.cmd == "arm":
        return arm_command(args)
    if args.cmd == "disarm":
        return disarm_command(args)
    if args.cmd == "run":
        out = run_once(dry_run=bool(args.dry_run))
        print(json.dumps(out, indent=2))
        return 0
    ap.print_help()
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
