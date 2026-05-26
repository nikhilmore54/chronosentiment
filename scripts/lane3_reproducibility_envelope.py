#!/usr/bin/env python3
"""
Lane 3 slice 2 — reproducibility envelope freeze.

Bounded environment attestation + deterministic replay fingerprint.
Compares two replay passes; fails on authoritative drift.

Observational residue may vary; authoritative replay meaning must not.
Does not introduce replay semantics, orchestration, or new authority surfaces.
"""

from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import os
import platform
import subprocess
import sys
from typing import Any

PROJECT_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
TIER1_SCRIPT = os.path.join(PROJECT_ROOT, "scripts", "run_tier1_observability_demo.py")
RERUN_GUARD_SCRIPT = os.path.join(PROJECT_ROOT, "scripts", "lane3_replay_rerun_guard.py")
ENVELOPE_PATH = os.path.join(PROJECT_ROOT, "fixtures", "lane3", "reproducibility_envelope.json")
ENVELOPE_VERSION = "1.0.0"


def load_module(path: str, name: str):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"Unable to load module from {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def hash_file(filepath: str) -> str | None:
    if not os.path.exists(filepath):
        return None
    digest = hashlib.sha256()
    with open(filepath, "rb") as handle:
        while chunk := handle.read(8192):
            digest.update(chunk)
    return digest.hexdigest()


def hash_dict(payload: dict[str, Any]) -> str:
    digest = hashlib.sha256()
    digest.update(json.dumps(payload, sort_keys=True, separators=(",", ":")).encode("utf-8"))
    return digest.hexdigest()


def collect_environment() -> dict[str, str]:
    cargo_lock = os.path.join(PROJECT_ROOT, "Cargo.lock")
    rust_version = subprocess.check_output(["rustc", "--version"], text=True).strip()
    return {
        "git_commit": load_module(TIER1_SCRIPT, "tier1_demo").get_git_commit(),
        "rust_version": rust_version,
        "cargo_lock_sha256": hash_file(cargo_lock) or "missing",
        "python_version": sys.version.split()[0],
        "os_fingerprint": f"{platform.system().lower()}-{platform.machine().lower()}",
    }


def build_authoritative_block(
    tier1: Any,
    rerun: Any,
    base_scenario: dict[str, Any],
    chronology_manifest: dict[str, Any],
    artifact_root: str,
) -> dict[str, Any]:
    fingerprints = rerun.collect_fingerprints(
        tier1,
        artifact_root,
        (tier1.COGNITION_A, tier1.COGNITION_B),
    )
    manifest_path = tier1.emit_manifest(base_scenario, chronology_manifest, artifact_root)
    with open(manifest_path) as handle:
        manifest_id = json.load(handle)["manifest_id"]

    block = {
        "scenario_id": base_scenario["catalog_name"],
        "chronology_hash": chronology_manifest.get("chronology_hash"),
        "replay_fingerprint": fingerprints,
        "manifest_id": manifest_id,
    }
    block["deterministic_replay_hash"] = hash_dict(
        {
            "scenario_id": block["scenario_id"],
            "chronology_hash": block["chronology_hash"],
            "replay_fingerprint": block["replay_fingerprint"],
            "manifest_id": block["manifest_id"],
        }
    )
    return block


def compare_authoritative(
    run_a: dict[str, Any],
    run_b: dict[str, Any],
    rerun_mod: Any,
) -> list[dict[str, Any]]:
    failures: list[dict[str, Any]] = []
    if run_a.get("deterministic_replay_hash") != run_b.get("deterministic_replay_hash"):
        failures.append(
            {
                "error": "DETERMINISTIC_REPLAY_HASH_DRIFT",
                "run_a": run_a.get("deterministic_replay_hash"),
                "run_b": run_b.get("deterministic_replay_hash"),
            }
        )
    if run_a.get("manifest_id") != run_b.get("manifest_id"):
        failures.append(
            {
                "error": "MANIFEST_ID_DRIFT",
                "run_a": run_a.get("manifest_id"),
                "run_b": run_b.get("manifest_id"),
            }
        )
    failures.extend(
        rerun_mod.compare_fingerprints(
            run_a.get("replay_fingerprint", {}),
            run_b.get("replay_fingerprint", {}),
        )
    )
    return failures


def run_envelope(scenario_key: str = "primary", *, check_reference: bool = False) -> int:
    tier1 = load_module(TIER1_SCRIPT, "tier1_demo")
    rerun = load_module(RERUN_GUARD_SCRIPT, "lane3_rerun")

    base_scenario = tier1.SCENARIOS[scenario_key]
    if not os.path.exists(base_scenario["substrate_file"]):
        print(f"[FAIL] Substrate not found: {base_scenario['substrate_file']}", file=sys.stderr)
        return 1

    print("Lane 3 slice 2 — reproducibility envelope freeze", flush=True)
    print("Conserved quantity: replay meaning under bounded environment assumptions\n", flush=True)

    tier1.ensure_trace_replay()
    chronology_manifest = tier1.load_chronology_manifest(base_scenario["manifest_file"])
    tier1.verify_provenance(base_scenario, chronology_manifest)

    environment = collect_environment()

    scenario_a = rerun.scenario_with_namespace(base_scenario, "lane3_envelope_a")
    scenario_b = rerun.scenario_with_namespace(base_scenario, "lane3_envelope_b")

    print("Pass 1/2 — bounded replay execution", flush=True)
    root_a = rerun.execute_replay_pass(tier1, scenario_a)
    authoritative_a = build_authoritative_block(
        tier1, rerun, base_scenario, chronology_manifest, root_a
    )

    print("Pass 2/2 — bounded replay execution", flush=True)
    root_b = rerun.execute_replay_pass(tier1, scenario_b)
    authoritative_b = build_authoritative_block(
        tier1, rerun, base_scenario, chronology_manifest, root_b
    )

    failures = compare_authoritative(authoritative_a, authoritative_b, rerun)

    envelope = {
        "envelope_version": ENVELOPE_VERSION,
        "lane": "Lane 3 slice 2 — reproducibility envelope freeze",
        "authority_map": "AUTHORITY_MAP.md — calibration preservation layer",
        "environment": environment,
        "authoritative": authoritative_a,
        "rerun_verification": {
            "status": "ENVELOPE_EQUIVALENT" if not failures else "ENVELOPE_DRIFT_DETECTED",
            "pass_a_namespace": scenario_a["substrate_namespace"],
            "pass_b_namespace": scenario_b["substrate_namespace"],
            "failures": failures,
        },
    }

    if check_reference and os.path.exists(ENVELOPE_PATH):
        with open(ENVELOPE_PATH) as handle:
            reference = json.load(handle)
        ref_env = reference.get("environment", {})
        ref_hash = reference.get("authoritative", {}).get("deterministic_replay_hash")
        cur_hash = authoritative_a.get("deterministic_replay_hash")
        same_commit = ref_env.get("git_commit") == environment["git_commit"]
        same_os = ref_env.get("os_fingerprint") == environment["os_fingerprint"]
        if same_commit and same_os and ref_hash != cur_hash:
            failures.append(
                {
                    "error": "REFERENCE_REPLAY_HASH_DRIFT",
                    "reference": ref_hash,
                    "current": cur_hash,
                }
            )
            envelope["rerun_verification"]["status"] = "ENVELOPE_DRIFT_DETECTED"
            envelope["rerun_verification"]["failures"] = failures

    os.makedirs(os.path.dirname(ENVELOPE_PATH), exist_ok=True)
    with open(ENVELOPE_PATH, "w") as handle:
        json.dump(envelope, handle, indent=2, sort_keys=True)

    print(json.dumps(envelope["rerun_verification"], indent=2), flush=True)
    print(f"Envelope: {os.path.relpath(ENVELOPE_PATH, PROJECT_ROOT)}", flush=True)

    if failures:
        print("[FAIL] Reproducibility envelope drift detected.", file=sys.stderr)
        return 1

    print("[OK] Reproducibility envelope equivalent across bounded rerun passes.")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Lane 3 reproducibility envelope freeze (environment-bounded attestation)"
    )
    parser.add_argument(
        "--scenario",
        choices=["primary"],
        default="primary",
        help="Bounded Tier 1 scenario preset (default: primary)",
    )
    parser.add_argument(
        "--check-reference",
        action="store_true",
        help="At same git commit, fail if authoritative replay hash differs from committed envelope",
    )
    args = parser.parse_args()
    try:
        return run_envelope(args.scenario, check_reference=args.check_reference)
    except subprocess.CalledProcessError as exc:
        print(f"[FAIL] Command failed: {exc}", file=sys.stderr)
        return exc.returncode or 1
    except RuntimeError as exc:
        print(f"[FAIL] {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
