#!/usr/bin/env python3
"""
emit_prospective_to_server.py
─────────────────────────────
Reads the existing prospective ledger and POSTs each sealed decision to the
Coralys Decision Server (POST /decisions).

Usage:
    python3 scripts/emit_prospective_to_server.py [--url http://localhost:3001] [--dry-run]

Defaults:
    --url   http://localhost:3001
    --ledger product_validation/CS-P-006/observatory/prospective/ledger.json

This script is idempotent: the server returns 409 for duplicate decision_ids,
which is treated as success (already exists).

Architecture note:
    This script does NOT reconstruct decisions from C3-002. It reads the
    already-certified prospective ledger produced by csp006_p_prospective.
    The ledger is the authoritative source; this script is a one-time
    population tool for the Decision Server ledger.
"""

import argparse
import json
import sys
import urllib.request
import urllib.error
from datetime import datetime, timezone, timedelta

# ─── Canonical hashes (must match coralys-decision/src/adapter.rs) ────────────
C3_002_POLICY_ARTIFACT_HASH = "5a43b9df97daa76d85edd7f7ef1c12c3a230ef292f7ecfa98ef9587647392121"
CORALYS_EXEC_ARTIFACT_HASH  = "3876ffa232f75068636aa058c6775671ac2f935ad2751c1253edd49e0770883f"

# ─── Helpers ──────────────────────────────────────────────────────────────────

def next_trading_session(decision_time_str: str) -> str:
    """Return the next NSE trading session date (YYYY-MM-DD) after decision_time."""
    # Parse RFC3339 / ISO8601
    dt_str = decision_time_str.replace("+00:00", "+0000").replace("Z", "+0000")
    try:
        dt = datetime.strptime(dt_str, "%Y-%m-%dT%H:%M:%S%z")
    except ValueError:
        dt = datetime.strptime(dt_str, "%Y-%m-%dT%H:%M:%S.%f%z")
    d = dt.date() + timedelta(days=1)
    while d.weekday() >= 5:  # 5=Sat, 6=Sun
        d += timedelta(days=1)
    return d.strftime("%Y-%m-%d")


def map_action(action: str) -> str:
    """Map prospective ledger action to IngestRequest direction."""
    a = action.upper()
    if a == "LONG":
        return "LONG"
    if a == "SHORT":
        return "SHORT"
    return "NO_TRADE"


def post_decision(base_url: str, decision: dict, dry_run: bool) -> tuple[int, str]:
    """POST a single decision to the server. Returns (http_status, body)."""
    direction = map_action(decision["action"])
    decision_ts = decision["decision_time"]
    # Normalise to UTC Z format
    decision_ts_norm = decision_ts.replace("+00:00", "Z")
    data_snapshot_id = f"yahoo-daily-{decision_ts_norm.replace(':', '').replace('-', '')}"
    effective_session = next_trading_session(decision_ts)

    body = {
        "decision_id":                   decision["decision_id"],
        "instrument":                    decision["instrument"],
        "decision_timestamp":            decision_ts_norm,
        "direction":                     direction,
        "trend":                         decision["state"]["trend"],
        "momentum":                      decision["state"]["momentum"],
        "volatility":                    decision["state"]["volatility"],
        "target_price":                  None,
        "policy_artifact_hash":          C3_002_POLICY_ARTIFACT_HASH,
        "execution_artifact_hash":       CORALYS_EXEC_ARTIFACT_HASH,
        "decision_pipeline":             "C3-002",
        "data_snapshot_id":              data_snapshot_id,
        "certified_timestamp":           decision_ts_norm,
        "reference_risk_boundary_price": None,
        "reference_risk_boundary_type":  "CORALYS_V0_ATR_TMV",
        "atr_14":                        None,
        "reference_price":               None,
        "effective_session":             effective_session,
    }

    if dry_run:
        print(f"  [DRY-RUN] would POST {decision['instrument']} {direction} {effective_session}")
        return 200, "dry-run"

    url = f"{base_url}/decisions"
    payload = json.dumps(body).encode("utf-8")
    req = urllib.request.Request(
        url,
        data=payload,
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    try:
        with urllib.request.urlopen(req, timeout=10) as resp:
            return resp.status, resp.read().decode("utf-8")
    except urllib.error.HTTPError as e:
        return e.code, e.read().decode("utf-8")
    except urllib.error.URLError as e:
        return 0, str(e)


# ─── Main ─────────────────────────────────────────────────────────────────────

def main():
    parser = argparse.ArgumentParser(description="Emit prospective ledger decisions to Coralys Decision Server")
    parser.add_argument("--url",     default="http://localhost:3001", help="Decision Server base URL")
    parser.add_argument("--ledger",  default="product_validation/CS-P-006/observatory/prospective/ledger.json")
    parser.add_argument("--dry-run", action="store_true", help="Print what would be sent without POSTing")
    args = parser.parse_args()

    with open(args.ledger) as f:
        ledger = json.load(f)

    decisions = ledger.get("decisions", [])
    print(f"Ledger: {args.ledger}")
    print(f"Decisions to emit: {len(decisions)}")
    print(f"Server: {args.url}")
    if args.dry_run:
        print("Mode: DRY-RUN")
    print()

    ok = 0
    already = 0
    errors = 0

    for dec in decisions:
        ticker = dec["instrument"]
        action = dec["action"]
        status, body = post_decision(args.url, dec, args.dry_run)

        if args.dry_run:
            ok += 1
            continue

        if status in (200, 201):
            ok += 1
            print(f"  OK      {ticker:25s} {action:8s} → {status}")
        elif status == 409:
            already += 1
            print(f"  EXISTS  {ticker:25s} {action:8s} → 409 (already in ledger)")
        else:
            errors += 1
            print(f"  ERROR   {ticker:25s} {action:8s} → {status}: {body[:120]}")

    print()
    print(f"Result: ok={ok} already={already} errors={errors} total={len(decisions)}")

    if errors > 0:
        print(f"\nFAIL: {errors} decision(s) failed to emit.")
        sys.exit(1)
    else:
        print("PASS")


if __name__ == "__main__":
    main()