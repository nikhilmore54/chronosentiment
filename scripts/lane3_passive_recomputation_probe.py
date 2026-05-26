#!/usr/bin/env python3
"""
Lane 3 Slice 3 — passive recomputation admissibility probe.

Disposable empirical sufficiency test (not Slice 3 implementation).
Answers: can passive recomputation close continuity visibility without schema pressure?

Does NOT modify authoritative surfaces. Read-only + temp-dir tamper cases only.
"""

from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import os
import shutil
import subprocess
import sys
import tempfile
from typing import Any

PROJECT_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
TIER1_SCRIPT = os.path.join(PROJECT_ROOT, "scripts", "run_tier1_observability_demo.py")
RERUN_GUARD_SCRIPT = os.path.join(PROJECT_ROOT, "scripts", "lane3_replay_rerun_guard.py")
VERIFY_SCRIPT = os.path.join(PROJECT_ROOT, "scripts", "verify_manifest_v1.py")
ENVELOPE_PATH = os.path.join(PROJECT_ROOT, "fixtures", "lane3", "reproducibility_envelope.json")
REPORT_PATH = os.path.join(PROJECT_ROOT, "fixtures", "lane3", "passive_recomputation_probe_report.json")

# Frozen six-file universe (AUTHORITY_MAP.md — do not widen)
COGNITIONS = ("rolling_50", "event_reset")
DETERMINISTIC_FILES = ("trace_summary.json", "trace_v1.json", "replay_hash.txt")
PROBE_NAMESPACE = "lane3_probe_base"


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


def recompute_six_file_fingerprints(artifact_root: str) -> dict[str, dict[str, str | None]]:
    fingerprints: dict[str, dict[str, str | None]] = {}
    for cognition in COGNITIONS:
        fingerprints[cognition] = {}
        for name in DETERMINISTIC_FILES:
            fingerprints[cognition][name] = hash_file(
                os.path.join(artifact_root, cognition, name)
            )
    return fingerprints


def compare_fingerprint_maps(
    expected: dict[str, dict[str, str | None]],
    actual: dict[str, dict[str, str | None]],
) -> list[dict[str, Any]]:
    failures: list[dict[str, Any]] = []
    for cognition in COGNITIONS:
        for name in DETERMINISTIC_FILES:
            exp = expected.get(cognition, {}).get(name)
            got = actual.get(cognition, {}).get(name)
            if exp is None or got is None:
                failures.append(
                    {
                        "error": "MISSING_ARTIFACT",
                        "cognition": cognition,
                        "artifact": name,
                        "expected": exp,
                        "actual": got,
                    }
                )
            elif exp != got:
                failures.append(
                    {
                        "error": "INTEGRITY_MISMATCH",
                        "cognition": cognition,
                        "artifact": name,
                        "expected": exp,
                        "actual": got,
                    }
                )
    return failures


def ensure_probe_artifacts(tier1: Any, rerun: Any, base_scenario: dict[str, Any]) -> str:
    topology = tier1.TOPOLOGY
    root = os.path.join(PROJECT_ROOT, "artifacts", PROBE_NAMESPACE, topology)
    manifest_path = os.path.join(root, "manifest.json")
    if all(
        os.path.exists(os.path.join(root, cog, fname))
        for cog in COGNITIONS
        for fname in DETERMINISTIC_FILES
    ) and os.path.exists(manifest_path):
        return root

    tier1.ensure_trace_replay()
    scenario = rerun.scenario_with_namespace(base_scenario, PROBE_NAMESPACE)
    rerun.execute_replay_pass(tier1, scenario)
    chronology_manifest = tier1.load_chronology_manifest(base_scenario["manifest_file"])
    tier1.emit_manifest(base_scenario, chronology_manifest, root)
    return root


def run_tamper_tests(artifact_root: str) -> dict[str, str]:
    results: dict[str, str] = {}

    # T1 — single-byte edit in trace_v1.json
    with tempfile.TemporaryDirectory(prefix="lane3_probe_t1_") as tmp:
        shutil.copytree(artifact_root, os.path.join(tmp, "artifacts"), dirs_exist_ok=True)
        tamper_root = os.path.join(tmp, "artifacts")
        target = os.path.join(tamper_root, "rolling_50", "trace_v1.json")
        with open(target, "r+b") as handle:
            handle.seek(0, os.SEEK_END)
            handle.write(b" ")
        baseline = recompute_six_file_fingerprints(artifact_root)
        tampered = recompute_six_file_fingerprints(tamper_root)
        results["T1_trace_byte_mutation"] = (
            "DETECTED" if compare_fingerprint_maps(baseline, tampered) else "NOT_DETECTED"
        )

    # T2 — manifest_id tamper (unsigned)
    with tempfile.TemporaryDirectory(prefix="lane3_probe_t2_") as tmp:
        tamper_root = os.path.join(tmp, "artifacts")
        shutil.copytree(artifact_root, tamper_root, dirs_exist_ok=True)
        manifest_path = os.path.join(tamper_root, "manifest.json")
        with open(manifest_path) as handle:
            manifest = json.load(handle)
        manifest["manifest_id"] = "0" * 64
        with open(manifest_path, "w") as handle:
            json.dump(manifest, handle, indent=4, sort_keys=True)
        proc = subprocess.run(
            [sys.executable, VERIFY_SCRIPT, manifest_path],
            cwd=PROJECT_ROOT,
            capture_output=True,
            text=True,
        )
        try:
            verify_data = json.loads(proc.stdout)
            failed = verify_data.get("status") != "ATTESTATION_PASSED"
        except json.JSONDecodeError:
            failed = proc.returncode != 0
        results["T2_manifest_id_tamper"] = "DETECTED" if failed else "NOT_DETECTED"

    # T3 — delete deterministic artifact
    with tempfile.TemporaryDirectory(prefix="lane3_probe_t3_") as tmp:
        tamper_root = os.path.join(tmp, "artifacts")
        shutil.copytree(artifact_root, tamper_root, dirs_exist_ok=True)
        os.remove(os.path.join(tamper_root, "event_reset", "replay_hash.txt"))
        baseline = recompute_six_file_fingerprints(artifact_root)
        tampered = recompute_six_file_fingerprints(tamper_root)
        results["T3_missing_artifact"] = (
            "DETECTED" if compare_fingerprint_maps(baseline, tampered) else "NOT_DETECTED"
        )

    return results


def run_probe(scenario_key: str = "primary") -> int:
    tier1 = load_module(TIER1_SCRIPT, "tier1_demo")
    rerun = load_module(RERUN_GUARD_SCRIPT, "lane3_rerun")
    base_scenario = tier1.SCENARIOS[scenario_key]

    print("Lane 3 Slice 3 — passive recomputation admissibility probe", flush=True)
    print("Empirical gate: passive recomputation sufficient without semantic widening?\n", flush=True)

    artifact_root = ensure_probe_artifacts(tier1, rerun, base_scenario)
    recomputed = recompute_six_file_fingerprints(artifact_root)

    failures: list[dict[str, Any]] = []

    # Self-consistency: recompute twice from same tree
    failures.extend(compare_fingerprint_maps(recomputed, recompute_six_file_fingerprints(artifact_root)))

    # Bind against committed envelope authoritative block (if present)
    envelope_match = None
    if os.path.exists(ENVELOPE_PATH):
        with open(ENVELOPE_PATH) as handle:
            envelope = json.load(handle)
        envelope_fps = envelope.get("authoritative", {}).get("replay_fingerprint", {})
        env_failures = compare_fingerprint_maps(envelope_fps, recomputed)
        envelope_match = len(env_failures) == 0
        if env_failures:
            failures.extend([{**f, "context": "envelope_reference"} for f in env_failures])

    # Manifest binding — existing schema fingerprints trace_summary only
    manifest_path = os.path.join(artifact_root, "manifest.json")
    with open(manifest_path) as handle:
        manifest = json.load(handle)
    manifest_fps = manifest.get("artifact_fingerprints", {})
    summary_only = all("trace_summary.json" in key for key in manifest_fps.keys())
    summary_failures: list[dict[str, Any]] = []
    for key, expected_hash in manifest_fps.items():
        rel = key  # e.g. rolling_50/trace_summary.json
        path = os.path.join(artifact_root, rel)
        actual = hash_file(path)
        if actual != expected_hash:
            summary_failures.append(
                {
                    "error": "MANIFEST_SUMMARY_BINDING_MISMATCH",
                    "artifact": rel,
                    "expected": expected_hash,
                    "actual": actual,
                }
            )
    failures.extend(summary_failures)
    manifest_summary_binding_ok = len(summary_failures) == 0

    # Passive six-file coverage without schema extension
    passive_six_file_sufficient = (
        len([f for f in failures if f.get("error") != "MANIFEST_SUMMARY_BINDING_MISMATCH"]) == 0
        and all(
            recomputed[c][n] is not None for c in COGNITIONS for n in DETERMINISTIC_FILES
        )
    )

    tamper_results = run_tamper_tests(artifact_root)
    tamper_ok = all(v == "DETECTED" for v in tamper_results.values())

    semantic_widening_pressure = not (
        passive_six_file_sufficient and summary_only and manifest_summary_binding_ok
    )

    probe_failures = failures.copy()
    if not tamper_ok:
        probe_failures.append({"error": "TAMPER_VISIBILITY_FAILED", "details": tamper_results})

    status = (
        "PASS_PASSIVE_RECOMPUTATION_SUFFICIENT"
        if not probe_failures and passive_six_file_sufficient and tamper_ok
        else "FAIL_SEMANTIC_WIDENING_OR_INSUFFICIENT"
    )

    report = {
        "probe": "lane3_passive_recomputation_admissibility",
        "authority_map": "LANE 3 SLICE 3 — ADMISSIBILITY (pre-implementation)",
        "artifact_root": os.path.relpath(artifact_root, PROJECT_ROOT),
        "frozen_six_file_universe": {
            "cognitions": list(COGNITIONS),
            "files": list(DETERMINISTIC_FILES),
        },
        "recomputed_fingerprints": recomputed,
        "envelope_reference_match": envelope_match,
        "manifest_summary_binding_ok": manifest_summary_binding_ok,
        "manifest_fingerprints_trace_summary_only": summary_only,
        "passive_six_file_sufficient_without_schema": passive_six_file_sufficient,
        "semantic_widening_pressure": semantic_widening_pressure,
        "tamper_tests": tamper_results,
        "failures": probe_failures,
        "conclusion": (
            "PASS: passive recomputation sufficient — Slice 3 implementation mechanically justified"
            if status.startswith("PASS")
            else "FAIL: stop Slice 3 or escalate — passive recomputation insufficient or tamper visibility failed"
        ),
        "status": status,
    }

    os.makedirs(os.path.dirname(REPORT_PATH), exist_ok=True)
    with open(REPORT_PATH, "w") as handle:
        json.dump(report, handle, indent=2, sort_keys=True)

    print(json.dumps({"status": status, "tamper_tests": tamper_results}, indent=2), flush=True)
    print(f"Report: {os.path.relpath(REPORT_PATH, PROJECT_ROOT)}", flush=True)

    if status.startswith("PASS"):
        print("[OK] Empirical admissibility gate: passive recomputation sufficient.")
        return 0

    print("[FAIL] Empirical admissibility gate not satisfied.", file=sys.stderr)
    return 1


def main() -> int:
    parser = argparse.ArgumentParser(description="Lane 3 passive recomputation admissibility probe")
    parser.add_argument("--scenario", choices=["primary"], default="primary")
    args = parser.parse_args()
    try:
        return run_probe(args.scenario)
    except subprocess.CalledProcessError as exc:
        print(f"[FAIL] Command failed: {exc}", file=sys.stderr)
        return exc.returncode or 1
    except RuntimeError as exc:
        print(f"[FAIL] {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
