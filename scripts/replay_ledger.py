#!/usr/bin/env python3
"""
DEPRECATED — price-less replay only.

This script replays the sealed prospective ledger into the running
coralys_decision_server WITHOUT reference_price or atr_14.  All
recommendation cards will show '—' for LTP / Adap. Target / Adap. Risk / R:R.

USE THIS INSTEAD (price-enriched replay via Yahoo cache):
-----------------------------------------------------------------
  CHRONO_YAHOO_CACHE_DIR=product_validation/CS-P-006/snapshot/20260814T183851Z_100instrument/yahoo_cache \\
    cargo run -p chronosentiment_adapter --bin csp006_p_enrich -- \\
    --ledger product_validation/CS-P-006/observatory/prospective/ledger.json \\
    --emit-url http://localhost:3001 \\
    --now 2026-08-17T03:45:00Z

That command fetches reference_price + atr_14 from the warm Yahoo cache
and posts all 202 decisions with real prices (emitted_new=202, errors=0).
-----------------------------------------------------------------

This script is retained only as a lightweight smoke-test / field-mapping
reference.  Do NOT use it to populate the live UI.
"""
import json
import urllib.request
import urllib.error
import sys

LEDGER = "product_validation/CS-P-006/observatory/prospective/ledger.json"
URL = "http://localhost:3001/decisions"

print("WARNING: replay_ledger.py posts decisions WITHOUT reference_price/atr_14.")
print("         UI cards will show '—' for all price fields.")
print("         Use csp006_p_enrich for price-enriched replay (see module docstring).")
print()

with open(LEDGER) as f:
    data = json.load(f)

decisions = data if isinstance(data, list) else data.get("decisions", [])
print(f"Replaying {len(decisions)} decisions (price-less)...")

ok = skip = err = 0
for d in decisions:
    state = d.get("state", {})
    body = {
        "decision_id": d["decision_id"],
        "instrument": d["instrument"],
        "decision_timestamp": d["decision_time"],   # ledger uses decision_time
        "direction": d["action"],                   # ledger uses action (LONG/SHORT/NO_TRADE)
        "trend": state.get("trend", "absent"),
        "momentum": state.get("momentum", "absent"),
        "volatility": state.get("volatility", "absent"),
        "target_price": None,
        "policy_artifact_hash": d.get("policy_artifact_sha256", ""),
        "execution_artifact_hash": None,
        "decision_pipeline": d.get("policy_id", "C3-002"),
        "data_snapshot_id": d.get("engine_version", "unfrozen-dev"),
        "certified_timestamp": d["decision_time"],
        "reference_risk_boundary_price": None,
        "reference_risk_boundary_type": "ATR",
        # reference_price and atr_14 intentionally omitted — use csp006_p_enrich instead
    }
    req = urllib.request.Request(
        URL,
        data=json.dumps(body).encode(),
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    try:
        with urllib.request.urlopen(req, timeout=5):
            ok += 1
    except urllib.error.HTTPError as e:
        if e.code == 409:
            skip += 1
        else:
            err += 1
            print(f"ERR {e.code}: {d['instrument']} — {e.read()[:120]}")
    except Exception as ex:
        err += 1
        print(f"EX: {d['instrument']} — {ex}")

print(f"Done: ok={ok} skip(409)={skip} err={err}")
print()
print("Reminder: run csp006_p_enrich to populate reference_price + atr_14.")