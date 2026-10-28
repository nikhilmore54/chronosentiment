#!/usr/bin/env python3
"""
Prospective V1 deployment preflight — does NOT authorize the experiment.

Runs local verification of packer/fixtures/Arm A invariance, ensures
prospective_v1/ namespace isolation, and records readiness. Leaves
prospective_v1_authorization.json authorized=false.
"""

from __future__ import annotations

import json
import subprocess
import sys
import unittest
from datetime import datetime, timezone
from pathlib import Path

WORKSPACE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE))

from scripts.rp608_prospective_v1 import (  # noqa: E402
    ARTIFACT_ROOT,
    AUTH_PATH,
    GATE4_NS,
    PROTOCOL_ID,
    PROSPECTIVE_NS,
    artifact_hashes,
    ensure_namespace,
    integrity_review_actions_forbidden,
    is_authorized,
    load_authorization,
    save_clocks,
    load_clocks,
)


def _run_unittest(module: str) -> dict:
    loader = unittest.TestLoader()
    suite = loader.loadTestsFromName(module)
    stream = open("/dev/null", "w")
    result = unittest.TextTestRunner(stream=stream, verbosity=0).run(suite)
    stream.close()
    return {
        "module": module,
        "tests_run": result.testsRun,
        "failures": len(result.failures),
        "errors": len(result.errors),
        "passed": result.wasSuccessful(),
    }


def _git_rev() -> str:
    try:
        return subprocess.check_output(
            ["git", "rev-parse", "HEAD"], cwd=str(WORKSPACE), text=True
        ).strip()
    except Exception:
        # Pi deploy tree may not be a git checkout; fall back to stamped rev file.
        stamped = (
            WORKSPACE
            / "artifacts"
            / "rp608_regime_breakout"
            / "prospective_v1"
            / "DEPLOYED_REV.txt"
        )
        if stamped.exists():
            return stamped.read_text(encoding="utf-8").strip() or "UNKNOWN"
        return "UNKNOWN"


def main() -> int:
    ensure_namespace()
    clocks = load_clocks()
    save_clocks(clocks)

    auth = load_authorization()
    lifecycle = _run_unittest("tests.test_rp608_breakout_lifecycle_fixtures")
    arm_a = _run_unittest("tests.test_rp608_arm_a_invariance")
    prospective_mod = _run_unittest("tests.test_rp608_prospective_v1")

    gate4_intact = GATE4_NS.exists() and any(GATE4_NS.iterdir())
    ns_isolated = PROSPECTIVE_NS.exists() and (PROSPECTIVE_NS / "README.md").exists()
    auth_still_false = auth.get("authorized") is not True

    checks = {
        "lifecycle_fixtures": lifecycle,
        "arm_a_invariance": arm_a,
        "prospective_v1_module_tests": prospective_mod,
        "canonical_packer_hashed": "scripts/breakout_lifecycle.py" in artifact_hashes(),
        "prospective_namespace_ready": ns_isolated,
        "gate4_namespace_intact": gate4_intact,
        "authorization_remains_false": auth_still_false,
        "nse_crypto_pooling": "FORBIDDEN",
        "integrity_tuning_forbidden": list(integrity_review_actions_forbidden()),
        "session_clock_initialized": True,
        "extension_rule": "+10 per EXT; max 2; never relax criteria",
    }

    ready = all(
        [
            lifecycle["passed"],
            arm_a["passed"],
            prospective_mod["passed"],
            ns_isolated,
            gate4_intact,
            auth_still_false,
        ]
    )

    report = {
        "protocol_id": PROTOCOL_ID,
        "preflight_utc": datetime.now(timezone.utc).isoformat(),
        "git_rev": _git_rev(),
        "artifact_hashes": artifact_hashes(),
        "checks": checks,
        "preflight_pass": ready,
        "authorize_now": False,
        "authorized": False,
        "observation_counting": "NOT_STARTED",
        "recommendation": (
            "Authorize Prospective V1 only after this preflight_pass is true on the "
            "deployed revision, then set authorized=true with human provenance. "
            "Do not count sessions until session-start criteria are met. "
            "Sessions 1–5 integrity review must not tune thresholds."
        ),
        "controls": {
            "arm_a": "FROZEN",
            "holdout_sessions_41_68": "SEALED",
            "live_order_authority_pct": 0.0,
            "dynamic_exits": "FORBIDDEN",
        },
        "output_namespace": str(PROSPECTIVE_NS.relative_to(WORKSPACE)),
        "auth_path": str(AUTH_PATH.relative_to(WORKSPACE)),
    }

    out = ARTIFACT_ROOT / "prospective_v1_preflight.json"
    out.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    # Refresh auth sidecar metadata without flipping authorized
    auth = load_authorization()
    auth["preflight_artifact"] = str(out.relative_to(WORKSPACE))
    auth["preflight_pass"] = ready
    auth["preflight_utc"] = report["preflight_utc"]
    auth["git_rev_at_preflight"] = report["git_rev"]
    auth["authorized"] = False
    auth["observation_counting"] = "NOT_STARTED"
    AUTH_PATH.write_text(json.dumps(auth, indent=2) + "\n", encoding="utf-8")

    print(json.dumps({
        "wrote": str(out),
        "preflight_pass": ready,
        "authorized": False,
        "lifecycle_passed": lifecycle["passed"],
        "arm_a_passed": arm_a["passed"],
        "prospective_tests_passed": prospective_mod["passed"],
        "is_authorized_helper": is_authorized(),
    }, indent=2))
    return 0 if ready else 1


if __name__ == "__main__":
    raise SystemExit(main())
