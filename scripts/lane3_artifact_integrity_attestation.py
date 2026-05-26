#!/usr/bin/env python3
"""
Lane 3 slice 3 — artifact integrity attestation.

Observational attestation only: passive recomputation + digest-chain verification.
Integrity may attest to replay meaning; integrity may not define replay meaning.

Compose-only: verify_manifest_v1.py, Tier 1 replay path, frozen six-file universe.
No schema extension, registry, or interpretive metadata.
"""

from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import os
import platform
import shutil
import subprocess
import sys
import tempfile
from typing import Any

PROJECT_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
TIER1_SCRIPT = os.path.join(PROJECT_ROOT, "scripts", "run_tier1_observability_demo.py")
RERUN_GUARD_SCRIPT = os.path.join(PROJECT_ROOT, "scripts", "lane3_replay_rerun_guard.py")
PROBE_SCRIPT = os.path.join(PROJECT_ROOT, "scripts", "lane3_passive_recomputation_probe.py")
VERIFY_SCRIPT = os.path.join(PROJECT_ROOT, "scripts", "verify_manifest_v1.py")
ENVELOPE_PATH = os.path.join(PROJECT_ROOT, "fixtures", "lane3", "reproducibility_envelope.json")
ATTESTATION_PATH = os.path.join(PROJECT_ROOT, "fixtures", "lane3", "artifact_integrity_attestation.json")
ATTESTATION_VERSION = "1.0.0"
INTEGRITY_NAMESPACE = "lane3_integrity_attest"

# Frozen six-file universe — AUTHORITY_MAP.md digest chain (C); do not widen
COGNITIONS = ("rolling_50", "event_reset")
DETERMINISTIC_FILES = ("trace_summary.json", "trace_v1.json", "replay_hash.txt")


def load_module(path: str, name: str):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"Unable to load module from {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def hash_dict(payload: dict[str, Any]) -> str:
    digest = hashlib.sha256()
    digest.update(json.dumps(payload, sort_keys=True, separators=(",", ":")).encode("utf-8"))
    return digest.hexdigest()


def build_digest_chain(
    scenario_id: str,
    chronology_hash: str,
    replay_fingerprint: dict[str, dict[str, str | None]],
    manifest_id: str,
) -> dict[str, Any]:
    chain = {
        "scenario_id": scenario_id,
        "chronology_hash": chronology_hash,
        "replay_fingerprint": replay_fingerprint,
        "manifest_id": manifest_id,
    }
    chain["deterministic_replay_hash"] = hash_dict(
        {
            "scenario_id": chain["scenario_id"],
            "chronology_hash": chain["chronology_hash"],
            "replay_fingerprint": chain["replay_fingerprint"],
            "manifest_id": chain["manifest_id"],
        }
    )
    return chain


def ensure_artifacts(tier1: Any, rerun: Any, base_scenario: dict[str, Any]) -> str:
    topology = tier1.TOPOLOGY
    root = os.path.join(PROJECT_ROOT, "artifacts", INTEGRITY_NAMESPACE, topology)
    if all(
        os.path.exists(os.path.join(root, cog, fname))
        for cog in COGNITIONS
        for fname in DETERMINISTIC_FILES
    ) and os.path.exists(os.path.join(root, "manifest.json")):
        return root

    tier1.ensure_trace_replay()
    scenario = rerun.scenario_with_namespace(base_scenario, INTEGRITY_NAMESPACE)
    rerun.execute_replay_pass(tier1, scenario)
    chronology_manifest = tier1.load_chronology_manifest(base_scenario["manifest_file"])
    tier1.emit_manifest(base_scenario, chronology_manifest, root)
    return root


def verify_manifest_passive(manifest_path: str) -> tuple[bool, list[str], list[dict[str, Any]]]:
    proc = subprocess.run(
        [sys.executable, VERIFY_SCRIPT, manifest_path],
        cwd=PROJECT_ROOT,
        capture_output=True,
        text=True,
    )
    try:
        data = json.loads(proc.stdout)
    except json.JSONDecodeError:
        return False, [], [{"error": "VERIFY_OUTPUT_INVALID", "details": proc.stdout or proc.stderr}]
    attestations = list(data.get("attestations", []))
    failures = list(data.get("failures", []))
    ok = data.get("status") == "ATTESTATION_PASSED" and proc.returncode == 0
    return ok, attestations, failures


def execute_tamper_tests(probe_mod: Any, artifact_root: str) -> dict[str, str]:
    return probe_mod.run_tamper_tests(artifact_root)


def run_attestation(
    scenario_key: str = "primary",
    *,
    include_tamper_tests: bool = False,
    check_reference: bool = False,
) -> int:
    tier1 = load_module(TIER1_SCRIPT, "tier1_demo")
    rerun = load_module(RERUN_GUARD_SCRIPT, "lane3_rerun")
    probe = load_module(PROBE_SCRIPT, "lane3_probe")
    base_scenario = tier1.SCENARIOS[scenario_key]
    chronology_manifest = tier1.load_chronology_manifest(base_scenario["manifest_file"])

    print("Lane 3 slice 3 — artifact integrity attestation", flush=True)
    print("Layer: observational — detect/mismatch only\n", flush=True)

    artifact_root = ensure_artifacts(tier1, rerun, base_scenario)
    manifest_path = os.path.join(artifact_root, "manifest.json")

    recomputed = probe.recompute_six_file_fingerprints(artifact_root)
    with open(manifest_path) as handle:
        manifest_id = json.load(handle)["manifest_id"]

    digest_chain = build_digest_chain(
        base_scenario["catalog_name"],
        chronology_manifest.get("chronology_hash"),
        recomputed,
        manifest_id,
    )

    failures: list[dict[str, Any]] = []
    attestations: list[str] = []

    missing = [
        f"{c}/{n}"
        for c in COGNITIONS
        for n in DETERMINISTIC_FILES
        if recomputed.get(c, {}).get(n) is None
    ]
    if missing:
        failures.append({"error": "MISSING_ARTIFACT", "artifacts": missing})
    else:
        attestations.append("SIX_FILE_RECOMPUTE_COMPLETE")

    manifest_ok, manifest_atts, manifest_failures = verify_manifest_passive(manifest_path)
    if manifest_ok:
        attestations.append("MANIFEST_VERIFY_PASSED")
    else:
        failures.extend(manifest_failures or [{"error": "MANIFEST_VERIFY_FAILED"}])

    # Manifest summary binding (existing schema — passive six-file via recompute above)
    with open(manifest_path) as handle:
        manifest = json.load(handle)
    for key, expected in manifest.get("artifact_fingerprints", {}).items():
        actual = probe.hash_file(os.path.join(artifact_root, key))
        if actual != expected:
            failures.append(
                {
                    "error": "MANIFEST_SUMMARY_BINDING_MISMATCH",
                    "artifact": key,
                    "expected": expected,
                    "actual": actual,
                }
            )
    if not any(f.get("error") == "MANIFEST_SUMMARY_BINDING_MISMATCH" for f in failures):
        attestations.append("MANIFEST_SUMMARY_BINDING_MATCH")

    tamper_results: dict[str, str] = {}
    if include_tamper_tests:
        tamper_results = execute_tamper_tests(probe, artifact_root)
        if all(v == "DETECTED" for v in tamper_results.values()):
            attestations.append("TAMPER_VISIBILITY_CONFIRMED")
        else:
            failures.append({"error": "TAMPER_VISIBILITY_FAILED", "details": tamper_results})

    status = "INTEGRITY_ATTESTED" if not failures else "INTEGRITY_ATTESTATION_FAILED"

    attestation = {
        "attestation_version": ATTESTATION_VERSION,
        "lane": "Lane 3 slice 3 — artifact integrity attestation",
        "layer": "observational",
        "authority_map": "LANE 3 SLICE 3 — ADMISSIBILITY",
        "scenario_id": base_scenario["catalog_name"],
        "frozen_six_file_universe": {
            "cognitions": list(COGNITIONS),
            "files": list(DETERMINISTIC_FILES),
        },
        "digest_chain": digest_chain,
        "attestations": attestations,
        "failures": failures,
        "tamper_tests": tamper_results if include_tamper_tests else {},
        "status": status,
    }

    if check_reference and os.path.exists(ATTESTATION_PATH):
        with open(ATTESTATION_PATH) as handle:
            reference = json.load(handle)
        ref_hash = reference.get("digest_chain", {}).get("deterministic_replay_hash")
        cur_hash = digest_chain.get("deterministic_replay_hash")
        ref_commit = reference.get("git_commit")
        cur_commit = tier1.get_git_commit()
        os_fingerprint = f"{platform.system().lower()}-{platform.machine().lower()}"
        ref_os = reference.get("os_fingerprint")
        if ref_commit == cur_commit and ref_os == os_fingerprint and ref_hash != cur_hash:
            failures.append(
                {
                    "error": "REFERENCE_DIGEST_CHAIN_DRIFT",
                    "reference": ref_hash,
                    "current": cur_hash,
                }
            )
            attestation["status"] = "INTEGRITY_ATTESTATION_FAILED"
            attestation["failures"] = failures

    attestation["git_commit"] = tier1.get_git_commit()
    attestation["os_fingerprint"] = f"{platform.system().lower()}-{platform.machine().lower()}"

    os.makedirs(os.path.dirname(ATTESTATION_PATH), exist_ok=True)
    with open(ATTESTATION_PATH, "w") as handle:
        json.dump(attestation, handle, indent=2, sort_keys=True)

    print(json.dumps({"status": attestation["status"], "attestations": attestations}, indent=2), flush=True)
    print(f"Attestation: {os.path.relpath(ATTESTATION_PATH, PROJECT_ROOT)}", flush=True)

    if attestation["status"] != "INTEGRITY_ATTESTED":
        print("[FAIL] Artifact integrity attestation failed.", file=sys.stderr)
        return 1

    print("[OK] Artifact integrity attestation complete.")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description="Lane 3 artifact integrity attestation")
    parser.add_argument("--scenario", choices=["primary"], default="primary")
    parser.add_argument(
        "--run-tamper-tests",
        action="store_true",
        help="Run T1–T3 tamper visibility cases in temp dirs",
    )
    parser.add_argument(
        "--check-reference",
        action="store_true",
        help="At same git commit, fail if digest chain differs from committed attestation",
    )
    args = parser.parse_args()
    try:
        return run_attestation(
            args.scenario,
            include_tamper_tests=args.run_tamper_tests,
            check_reference=args.check_reference,
        )
    except subprocess.CalledProcessError as exc:
        print(f"[FAIL] Command failed: {exc}", file=sys.stderr)
        return exc.returncode or 1
    except RuntimeError as exc:
        print(f"[FAIL] {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
