#!/usr/bin/env python3
"""
RP-611 Phase-1 deployment orchestrator.

Runs preflight → conditional deploy stamp → first observation pass.
Preserves experiment T0. Authority 0%. No RP-608/OF/Gate5 mutation.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any, Dict

_ROOT = Path(__file__).resolve().parent.parent
if str(_ROOT) not in sys.path:
    sys.path.insert(0, str(_ROOT))

import scripts.rp611_phase1_crypto_observer as crypto_obs  # noqa: E402
import scripts.rp611_phase1_nse_observer as nse_obs  # noqa: E402
from scripts.rp611_phase1_common import (  # noqa: E402
    CRYPTO_ROOT,
    NSE_ROOT,
    load_registration,
    stamp_deployment,
)


def deploy_nse() -> Dict[str, Any]:
    pf = nse_obs.preflight()
    if not pf.get("ok"):
        receipt = stamp_deployment(
            NSE_ROOT,
            protocol_id=nse_obs.PROTOCOL_ID,
            short_id=nse_obs.SHORT_ID,
            preflight_ok=False,
            receipt={"preflight": pf, "run": None},
        )
        return {"market": "NSE", "deployed": False, "preflight": pf, "receipt": receipt}
    run = nse_obs.run_once()
    receipt = stamp_deployment(
        NSE_ROOT,
        protocol_id=nse_obs.PROTOCOL_ID,
        short_id=nse_obs.SHORT_ID,
        preflight_ok=True,
        receipt={"preflight": pf, "run": run},
    )
    reg = load_registration(NSE_ROOT)
    return {
        "market": "NSE",
        "deployed": True,
        "preflight": pf,
        "run": run,
        "t0_utc": reg.get("experiment_clock_started_utc"),
        "deployed_utc": reg.get("phase1_deployed_utc"),
        "pre_deployment_capture_gap_sec": reg.get("pre_deployment_capture_gap_sec"),
        "receipt_event": receipt.get("event_type"),
    }


def deploy_crypto() -> Dict[str, Any]:
    pf = crypto_obs.preflight()
    if not pf.get("ok"):
        receipt = stamp_deployment(
            CRYPTO_ROOT,
            protocol_id=crypto_obs.PROTOCOL_ID,
            short_id=crypto_obs.SHORT_ID,
            preflight_ok=False,
            receipt={"preflight": pf, "run": None},
        )
        return {"market": "CRYPTO", "deployed": False, "preflight": pf, "receipt": receipt}
    run = crypto_obs.run_once()
    receipt = stamp_deployment(
        CRYPTO_ROOT,
        protocol_id=crypto_obs.PROTOCOL_ID,
        short_id=crypto_obs.SHORT_ID,
        preflight_ok=True,
        receipt={"preflight": pf, "run": run},
    )
    reg = load_registration(CRYPTO_ROOT)
    return {
        "market": "CRYPTO",
        "deployed": True,
        "preflight": pf,
        "run": run,
        "t0_utc": reg.get("experiment_clock_started_utc"),
        "deployed_utc": reg.get("phase1_deployed_utc"),
        "pre_deployment_capture_gap_sec": reg.get("pre_deployment_capture_gap_sec"),
        "receipt_event": receipt.get("event_type"),
    }


def main() -> int:
    nse = deploy_nse()
    crypto = deploy_crypto()
    out = {"nse": nse, "crypto": crypto}
    print(json.dumps(out, indent=2))
    # Exit non-zero if either failed preflight/deploy
    if not nse.get("deployed") or not crypto.get("deployed"):
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
