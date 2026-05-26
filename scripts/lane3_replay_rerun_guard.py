#!/usr/bin/env python3
"""
Lane 3 — deterministic replay rerun guard.

Operational drift detection only: run Tier 1 primary replays twice,
compare deterministic artifact fingerprints, fail on divergence.

Does not introduce replay semantics, authority surfaces, or governance law.
Scope: AUTHORITY_MAP.md Lane 3 — procedural reproducibility.
"""

from __future__ import annotations

import argparse
import importlib.util
import json
import os
import subprocess
import sys
from datetime import datetime, timezone
from typing import Any

PROJECT_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
TIER1_SCRIPT = os.path.join(PROJECT_ROOT, "scripts", "run_tier1_observability_demo.py")
REPORT_DIR = os.path.join(PROJECT_ROOT, "fixtures", "lane3")

# Observational metadata excluded — not part of replay meaning.
DETERMINISTIC_ARTIFACTS = ("trace_summary.json", "replay_hash.txt", "trace_v1.json")


def load_tier1_module():
    spec = importlib.util.spec_from_file_location("tier1_demo", TIER1_SCRIPT)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"Unable to load Tier 1 module from {TIER1_SCRIPT}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def scenario_with_namespace(base: dict[str, Any], namespace: str) -> dict[str, Any]:
    return {**base, "substrate_namespace": namespace}


def artifact_root(namespace: str, topology: str) -> str:
    return os.path.join(PROJECT_ROOT, "artifacts", namespace, topology)


def collect_fingerprints(
    tier1: Any,
    root: str,
    cognitions: tuple[str, ...],
) -> dict[str, dict[str, str | None]]:
    fingerprints: dict[str, dict[str, str | None]] = {}
    for cognition in cognitions:
        cognition_dir = os.path.join(root, cognition)
        fingerprints[cognition] = {}
        for artifact_name in DETERMINISTIC_ARTIFACTS:
            path = os.path.join(cognition_dir, artifact_name)
            fingerprints[cognition][artifact_name] = tier1.hash_file(path)
    return fingerprints


def compare_fingerprints(
    run_a: dict[str, dict[str, str | None]],
    run_b: dict[str, dict[str, str | None]],
) -> list[dict[str, Any]]:
    failures: list[dict[str, Any]] = []
    for cognition in run_a:
        for artifact_name in DETERMINISTIC_ARTIFACTS:
            hash_a = run_a[cognition].get(artifact_name)
            hash_b = run_b.get(cognition, {}).get(artifact_name)
            if hash_a is None or hash_b is None:
                failures.append(
                    {
                        "error": "MISSING_DETERMINISTIC_ARTIFACT",
                        "cognition": cognition,
                        "artifact": artifact_name,
                        "run_a": hash_a,
                        "run_b": hash_b,
                    }
                )
            elif hash_a != hash_b:
                failures.append(
                    {
                        "error": "REPLAY_DRIFT",
                        "cognition": cognition,
                        "artifact": artifact_name,
                        "run_a": hash_a,
                        "run_b": hash_b,
                    }
                )
    return failures


def execute_replay_pass(tier1: Any, scenario: dict[str, Any]) -> str:
    substrate_file = scenario["substrate_file"]
    namespace = scenario["substrate_namespace"]
    for cognition in (tier1.COGNITION_A, tier1.COGNITION_B):
        tier1.run_trace_replay(namespace, substrate_file, cognition)
    return artifact_root(namespace, tier1.TOPOLOGY)


def run_guard(scenario_key: str = "primary") -> int:
    tier1 = load_tier1_module()
    base_scenario = tier1.SCENARIOS[scenario_key]
    if not os.path.exists(base_scenario["substrate_file"]):
        print(f"[FAIL] Substrate not found: {base_scenario['substrate_file']}", file=sys.stderr)
        return 1

    print("Lane 3 replay rerun guard — operational drift detection", flush=True)
    print("Conserved quantity: replay meaning (deterministic artifact equality)\n", flush=True)

    tier1.ensure_trace_replay()
    chronology_manifest = tier1.load_chronology_manifest(base_scenario["manifest_file"])
    tier1.verify_provenance(base_scenario, chronology_manifest)

    scenario_a = scenario_with_namespace(base_scenario, "lane3_rerun_a")
    scenario_b = scenario_with_namespace(base_scenario, "lane3_rerun_b")

    print("Run 1/2 — primary Tier 1 replay pass", flush=True)
    root_a = execute_replay_pass(tier1, scenario_a)
    fingerprints_a = collect_fingerprints(
        tier1,
        root_a,
        (tier1.COGNITION_A, tier1.COGNITION_B),
    )

    print("Run 2/2 — primary Tier 1 replay pass", flush=True)
    root_b = execute_replay_pass(tier1, scenario_b)
    fingerprints_b = collect_fingerprints(
        tier1,
        root_b,
        (tier1.COGNITION_A, tier1.COGNITION_B),
    )

    failures = compare_fingerprints(fingerprints_a, fingerprints_b)

    manifest_path_a = tier1.emit_manifest(base_scenario, chronology_manifest, root_a)
    manifest_path_b = tier1.emit_manifest(base_scenario, chronology_manifest, root_b)
    with open(manifest_path_a) as handle:
        mid_a = json.load(handle)["manifest_id"]
    with open(manifest_path_b) as handle:
        mid_b = json.load(handle)["manifest_id"]

    if mid_a != mid_b:
        failures.append(
            {
                "error": "MANIFEST_ID_DRIFT",
                "run_a": mid_a,
                "run_b": mid_b,
            }
        )

    report = {
        "lane": "Lane 3 — certifiability infrastructure",
        "guard": "deterministic_replay_rerun",
        "scenario": base_scenario["catalog_name"],
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "git_commit": tier1.get_git_commit(),
        "deterministic_artifacts": list(DETERMINISTIC_ARTIFACTS),
        "fingerprints_run_a": fingerprints_a,
        "fingerprints_run_b": fingerprints_b,
        "manifest_id_run_a": mid_a,
        "manifest_id_run_b": mid_b,
        "failures": failures,
        "status": "RERUN_EQUIVALENT" if not failures else "RERUN_DRIFT_DETECTED",
    }

    os.makedirs(REPORT_DIR, exist_ok=True)
    report_path = os.path.join(REPORT_DIR, "replay_rerun_guard_report.json")
    with open(report_path, "w") as handle:
        json.dump(report, handle, indent=2, sort_keys=True)

    print(json.dumps({"status": report["status"], "failures": failures}, indent=2), flush=True)
    print(f"Report: {os.path.relpath(report_path, PROJECT_ROOT)}", flush=True)

    if failures:
        print("[FAIL] Replay rerun guard detected operational drift.", file=sys.stderr)
        return 1

    print("[OK] Replay rerun guard passed — deterministic outputs equivalent across reruns.")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Lane 3 deterministic replay rerun guard (operational drift detection)"
    )
    parser.add_argument(
        "--scenario",
        choices=["primary"],
        default="primary",
        help="Bounded Tier 1 scenario preset (default: primary)",
    )
    args = parser.parse_args()
    try:
        return run_guard(args.scenario)
    except subprocess.CalledProcessError as exc:
        print(f"[FAIL] Command failed: {exc}", file=sys.stderr)
        return exc.returncode or 1
    except RuntimeError as exc:
        print(f"[FAIL] {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
