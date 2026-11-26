#!/usr/bin/env python3
"""
Generate frozen Arm A golden fixtures for NSE and crypto.
Deterministic synthetic paths; SHA-256 fingerprints for invariance tests.
"""

from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List

from scripts.rp608_arm_a_baseline import (
    ARM_A_FROZEN,
    ARM_A_LIVE_ORDER_AUTHORITY,
    NseBar,
    crypto_arm_a_decide,
    crypto_arm_a_simulate_trade,
    fingerprint,
    nse_arm_a_decide_entry,
    nse_arm_a_simulate_trade,
)

WORKSPACE_DIR = Path(__file__).resolve().parent.parent
OUT_DIR = WORKSPACE_DIR / "fixtures" / "rp608_regime_breakout"


def _synth_nse_uptrend(n: int = 24, start: float = 1000.0) -> List[NseBar]:
    bars = []
    px = start
    for i in range(1, n + 1):
        o = px
        c = px * (1.0 + 0.0025)  # +25 bps per bar
        h = max(o, c) * 1.0005
        l = min(o, c) * 0.9995
        bars.append(NseBar(bar_idx=i, open=o, high=h, low=l, close=c))
        px = c
    return bars


def _synth_nse_chop(n: int = 24, start: float = 1000.0) -> List[NseBar]:
    bars = []
    px = start
    for i in range(1, n + 1):
        o = px
        c = px * (1.0002 if i % 2 == 0 else 0.9998)
        bars.append(NseBar(bar_idx=i, open=o, high=max(o, c) * 1.0003, low=min(o, c) * 0.9997, close=c))
        px = c
    return bars


def build_nse_fixtures() -> Dict[str, Any]:
    up = _synth_nse_uptrend()
    chop = _synth_nse_chop()
    cases = []

    # Case 1: cold start
    closes = [b.close for b in up[:3]]
    d = nse_arm_a_decide_entry(closes, bar_idx=3)
    cases.append({"case_id": "nse_cold_start", "decision": d, "trade": None})

    # Case 2: sub-threshold chop at bar 10
    closes = [b.close for b in chop[:10]]
    d = nse_arm_a_decide_entry(closes, bar_idx=10)
    cases.append({"case_id": "nse_sub_threshold", "decision": d, "trade": None})

    # Case 3: qualified entry on uptrend + full trade path
    closes = [b.close for b in up[:8]]
    d = nse_arm_a_decide_entry(closes, bar_idx=8)
    trade = None
    if d["signal_entry"]:
        trade = nse_arm_a_simulate_trade(
            entry_bar_idx=8,
            entry_price=up[7].close,
            forward_bars=up[8:],
        )
    cases.append({"case_id": "nse_qualified_uptrend_trade", "decision": d, "trade": trade})

    # Case 4: stop-loss path (inject adverse move)
    entry_px = 1000.0
    adverse = [
        NseBar(9, 1000, 1001, 985, 988),
        NseBar(10, 988, 990, 980, 982),
    ]
    d = {
        "decision": "ENTER_LONG",
        "decision_code": "QUALIFIED_MOMENTUM_BREAKOUT",
        "traj15": 0.12,
        "signal_entry": True,
        "side": "LONG",
    }
    trade = nse_arm_a_simulate_trade(8, entry_px, adverse)
    cases.append({"case_id": "nse_stop_loss_path", "decision": d, "trade": trade})

    # Case 5: boundary excluded
    closes = [b.close for b in up[:22]]
    d = nse_arm_a_decide_entry(closes, bar_idx=22)
    cases.append({"case_id": "nse_boundary_excluded", "decision": d, "trade": None})

    payload = {
        "protocol": "RP-608-REGIME-BREAKOUT-001",
        "market": "NSE",
        "arm": "A",
        "arm_a_frozen": ARM_A_FROZEN,
        "live_order_authority": ARM_A_LIVE_ORDER_AUTHORITY,
        "generated_utc": datetime.now(timezone.utc).isoformat(),
        "cases": cases,
    }
    payload["fingerprint_sha256"] = fingerprint(payload["cases"])
    return payload


def build_crypto_fixtures() -> Dict[str, Any]:
    cases = []

    d = crypto_arm_a_decide("NO_TRADE", True)
    cases.append({"case_id": "crypto_policy_b_no_trade", "decision": d, "trade": None})

    d = crypto_arm_a_decide("LONG", False)
    cases.append({"case_id": "crypto_unexecutable_drop", "decision": d, "trade": None})

    d = crypto_arm_a_decide("LONG", True)
    # Favorable path to target
    fwd = [101000 + i * 80 for i in range(1, 10)]
    trade = crypto_arm_a_simulate_trade("LONG", 100000.0, fwd)
    cases.append({"case_id": "crypto_long_take_profit", "decision": d, "trade": trade})

    d = crypto_arm_a_decide("SHORT", True)
    fwd = [99000 - i * 50 for i in range(1, 10)]
    trade = crypto_arm_a_simulate_trade("SHORT", 100000.0, fwd)
    cases.append({"case_id": "crypto_short_path", "decision": d, "trade": trade})

    d = crypto_arm_a_decide("LONG", True)
    fwd = [99900 - i * 40 for i in range(1, 10)]
    trade = crypto_arm_a_simulate_trade("LONG", 100000.0, fwd)
    cases.append({"case_id": "crypto_long_stop", "decision": d, "trade": trade})

    payload = {
        "protocol": "RP-608-REGIME-BREAKOUT-001",
        "market": "CRYPTO_BTCUSDT",
        "arm": "A",
        "arm_a_frozen": ARM_A_FROZEN,
        "live_order_authority": ARM_A_LIVE_ORDER_AUTHORITY,
        "generated_utc": datetime.now(timezone.utc).isoformat(),
        "cases": cases,
    }
    payload["fingerprint_sha256"] = fingerprint(payload["cases"])
    return payload


def main() -> None:
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    nse = build_nse_fixtures()
    crypto = build_crypto_fixtures()
    nse_path = OUT_DIR / "arm_a_nse_golden.json"
    crypto_path = OUT_DIR / "arm_a_crypto_golden.json"
    nse_path.write_text(json.dumps(nse, indent=2) + "\n", encoding="utf-8")
    crypto_path.write_text(json.dumps(crypto, indent=2) + "\n", encoding="utf-8")

    manifest = {
        "protocol": "RP-608-REGIME-BREAKOUT-001",
        "phase": 0,
        "live_order_authority": "0.0%",
        "holdout": "SEALED_UNTOUCHED",
        "nse_fixture": str(nse_path.relative_to(WORKSPACE_DIR)),
        "nse_fingerprint_sha256": nse["fingerprint_sha256"],
        "crypto_fixture": str(crypto_path.relative_to(WORKSPACE_DIR)),
        "crypto_fingerprint_sha256": crypto["fingerprint_sha256"],
        "reproducible": True,
    }
    man_path = OUT_DIR / "arm_a_fixtures_manifest.json"
    man_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(manifest, indent=2))


if __name__ == "__main__":
    main()
