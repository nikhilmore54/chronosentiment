#!/usr/bin/env python3
"""
Tier 1 replay-certified observability demo orchestrator.

Compose-only path bounded by docs/governance/DEMO_SCOPE.md.
Generates third-party runnable evidence — not presentation artifacts.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import subprocess
import sys
from datetime import datetime, timezone
from typing import Any

PROJECT_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CORE_DIR = os.path.join(PROJECT_ROOT, "core")
TRACE_REPLAY_BIN = os.path.join(PROJECT_ROOT, "target", "release", "trace_replay")
EMIT_SCRIPT = os.path.join(PROJECT_ROOT, "scripts", "emit_manifest_v1.py")
VERIFY_SCRIPT = os.path.join(PROJECT_ROOT, "scripts", "verify_manifest_v1.py")
EQUIV_SCRIPT = os.path.join(PROJECT_ROOT, "scripts", "certify_equivalence_v1.py")
REPORT_DIR = os.path.join(PROJECT_ROOT, "fixtures", "demo")

TOPOLOGY = "osc_50_1.0"
COGNITION_A = "rolling_50"
COGNITION_B = "event_reset"

SCENARIOS: dict[str, dict[str, Any]] = {
    "primary": {
        "substrate_namespace": "tier1_multi_stage_1m",
        "catalog_name": "2026_multi_stage_cascade_transition_1m",
        "replay_id": "2026_multi_stage_cascade_transition_1m",
        "replay_class": "multi_stage_cascade_transition",
        "authority_type": "binance_historical",
        "session_ontology": "cascade_transition_1m",
        "symbols": ["BTCUSDT"],
        "timeframe": "1m",
        "descriptors": ["tier1_observability_demo"],
        "manifest_file": os.path.join(
            CORE_DIR,
            "chronology",
            "historical",
            "2026_multi_stage_cascade_transition_1m",
            "2026_multi_stage_cascade_transition_1m_1779480000000_manifest.json",
        ),
        "substrate_file": os.path.join(
            CORE_DIR,
            "chronology",
            "historical",
            "2026_multi_stage_cascade_transition_1m",
            "btcusdt_1779480000000.jsonl",
        ),
    },
    "alternate": {
        "substrate_namespace": "tier1_impulse_shock_1m",
        "catalog_name": "2026_intraday_impulse_shock_0730_0800_utc_1m",
        "replay_id": "2026_intraday_impulse_shock_0730_0800_utc_1m",
        "replay_class": "intraday_impulse_shock",
        "authority_type": "binance_historical",
        "session_ontology": "impulse_shock_1m",
        "symbols": ["BTCUSDT"],
        "timeframe": "1m",
        "descriptors": ["tier1_observability_demo", "alternate"],
        "manifest_file": os.path.join(
            CORE_DIR,
            "chronology",
            "historical",
            "2026_intraday_impulse_shock_0730_0800_utc_1m",
            "2026_intraday_impulse_shock_0730_0800_utc_1m_1779521400000_manifest.json",
        ),
        "substrate_file": os.path.join(
            CORE_DIR,
            "chronology",
            "historical",
            "2026_intraday_impulse_shock_0730_0800_utc_1m",
            "btcusdt_1779521400000.jsonl",
        ),
    },
}


def print_header(title: str) -> None:
    print(f"\n{'=' * 60}", flush=True)
    print(f" {title.upper()}", flush=True)
    print(f"{'=' * 60}\n", flush=True)


def get_git_commit() -> str:
    try:
        return (
            subprocess.check_output(
                ["git", "rev-parse", "HEAD"],
                cwd=PROJECT_ROOT,
                stderr=subprocess.DEVNULL,
            )
            .decode("utf-8")
            .strip()
        )
    except Exception:
        return "unknown"


def hash_file(filepath: str) -> str | None:
    if not os.path.exists(filepath):
        return None
    digest = hashlib.sha256()
    with open(filepath, "rb") as handle:
        while chunk := handle.read(8192):
            digest.update(chunk)
    return digest.hexdigest()


def run_command(cmd: list[str], *, cwd: str | None = None, capture: bool = False) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=cwd or PROJECT_ROOT,
        check=True,
        text=True,
        capture_output=capture,
    )


def ensure_trace_replay() -> None:
    if os.path.exists(TRACE_REPLAY_BIN):
        return
    print_header("Building trace_replay (release)")
    run_command(["cargo", "build", "--release", "--bin", "trace_replay"], cwd=CORE_DIR)
    if not os.path.exists(TRACE_REPLAY_BIN):
        raise RuntimeError(f"Expected binary at {TRACE_REPLAY_BIN}")


def load_chronology_manifest(path: str) -> dict[str, Any]:
    with open(path) as handle:
        return json.load(handle)


def verify_provenance(scenario: dict[str, Any], chronology_manifest: dict[str, Any]) -> dict[str, Any]:
    substrate_file = scenario["substrate_file"]
    substrate_hash = hash_file(substrate_file)
    declared_hash = chronology_manifest.get("chronology_hash")
    commit_hash = get_git_commit()

    provenance = {
        "scenario": scenario["catalog_name"],
        "substrate_file": os.path.relpath(substrate_file, PROJECT_ROOT),
        "chronology_manifest": os.path.relpath(scenario["manifest_file"], PROJECT_ROOT),
        "substrate_sha256": substrate_hash,
        "declared_chronology_hash": declared_hash,
        "chronology_hash_match": substrate_hash == declared_hash,
        "capture_start_ms": chronology_manifest.get("capture_start"),
        "capture_end_ms": chronology_manifest.get("capture_end"),
        "total_ticks": chronology_manifest.get("total_ticks"),
        "git_commit": commit_hash,
    }

    print_header("Step 1 — Scenario selection + provenance")
    print(f"Scenario     : {scenario['catalog_name']}")
    print(f"Substrate    : {provenance['substrate_file']}")
    print(f"Ticks        : {provenance['total_ticks']}")
    print(f"Chronology   : {declared_hash}")
    print(f"Recomputed   : {substrate_hash}")
    print(f"Hash match   : {provenance['chronology_hash_match']}")
    print(f"Git commit   : {commit_hash}")

    if not provenance["chronology_hash_match"]:
        raise RuntimeError("Substrate hash does not match chronology manifest — stop (Lane 2 escalation)")

    return provenance


def run_trace_replay(substrate_namespace: str, substrate_file: str, cognition: str) -> str:
    cmd = [
        TRACE_REPLAY_BIN,
        "--substrate",
        substrate_namespace,
        "--substrate-file",
        os.path.abspath(substrate_file),
        "--topology",
        TOPOLOGY,
        "--cognition",
        cognition,
    ]
    print(f"  ./chrono replay {TOPOLOGY} {cognition} --substrate {substrate_namespace} --substrate-file <substrate>")
    run_command(cmd)
    return os.path.join(PROJECT_ROOT, "artifacts", substrate_namespace, TOPOLOGY, cognition)


def run_replays(scenario: dict[str, Any], *, single_cognition: str | None = None) -> str:
    print_header("Step 2 — Deterministic replay execution")
    substrate_file = scenario["substrate_file"]
    substrate_namespace = scenario["substrate_namespace"]
    cognitions = [single_cognition] if single_cognition else [COGNITION_A, COGNITION_B]
    for cognition in cognitions:
        run_trace_replay(substrate_namespace, substrate_file, cognition)
    return os.path.join(PROJECT_ROOT, "artifacts", substrate_namespace, TOPOLOGY)


def emit_manifest(scenario: dict[str, Any], chronology_manifest: dict[str, Any], artifact_dir: str) -> str:
    print_header("Step 3 — Manifest emission (provenance binding)")
    config = {
        "replay_id": scenario["replay_id"],
        "replay_class": scenario["replay_class"],
        "authority_type": scenario["authority_type"],
        "session_ontology": scenario["session_ontology"],
        "start_ts": chronology_manifest.get("capture_start"),
        "end_ts": chronology_manifest.get("capture_end"),
        "shift_offset": 0,
        "symbols": scenario["symbols"],
        "timeframe": scenario["timeframe"],
        "descriptors": scenario["descriptors"],
        "substrate_file": scenario["substrate_file"],
        "topology": TOPOLOGY,
        "cognition_a": COGNITION_A,
        "cognition_b": COGNITION_B,
        "artifact_dir": artifact_dir,
        "commit_hash": get_git_commit(),
    }
    config_path = os.path.join(artifact_dir, "tier1_demo_config.json")
    os.makedirs(artifact_dir, exist_ok=True)
    with open(config_path, "w") as handle:
        json.dump(config, handle, indent=2)

    run_command([sys.executable, EMIT_SCRIPT, config_path])
    manifest_path = os.path.join(artifact_dir, "manifest.json")
    if not os.path.exists(manifest_path):
        raise RuntimeError(f"Manifest not emitted at {manifest_path}")
    return manifest_path


def verify_manifest(manifest_path: str) -> dict[str, Any]:
    print_header("Step 4 — Passive manifest attestation")
    try:
        result = run_command([sys.executable, VERIFY_SCRIPT, manifest_path], capture=True)
        data = {"status": "ATTESTATION_PASSED", "attestations": []}
        for line in result.stdout.splitlines():
            if "[VERIFY]" in line:
                data["attestations"].append(line.split("[VERIFY]")[-1].strip())
    except subprocess.CalledProcessError as e:
        data = {"status": "ATTESTATION_FAILED", "failures": [{"error": e.stdout}]}
    
    print(f"Status: {data.get('status')}")
    for attestation in data.get("attestations", []):
        print(f"  + {attestation}")
    if data.get("status") != "ATTESTATION_PASSED":
        raise RuntimeError(f"Manifest verification failed: {data.get('failures')}")
    return data


def certify_equivalence(manifest_path: str) -> dict[str, Any]:
    print_header("Step 5 — Replay equivalence certification")
    result = run_command([sys.executable, EQUIV_SCRIPT, manifest_path], capture=True)
    data = json.loads(result.stdout)
    print(f"Status: {data.get('status')}")
    for attestation in data.get("attestations", []):
        print(f"  + {attestation}")
    if data.get("status") != "EQUIVALENCE_CERTIFIED":
        raise RuntimeError(f"Equivalence certification failed: {data.get('failures')}")
    return data


def build_causal_narrative(
    scenario: dict[str, Any],
    chronology_manifest: dict[str, Any],
    artifact_dir: str,
) -> dict[str, Any]:
    with open(os.path.join(artifact_dir, COGNITION_A, "trace_summary.json")) as handle:
        rolling = json.load(handle)
    with open(os.path.join(artifact_dir, COGNITION_B, "trace_summary.json")) as handle:
        event_reset = json.load(handle)

    rolling_persistence = rolling.get("persistence")
    event_reset_persistence = event_reset.get("persistence")

    narrative = {
        "title": "Forensic replay instrumentation — causal summary",
        "bounded_claim": (
            "Ordered chronology bytes were replayed deterministically; "
            "geometry divergence between cognition paths is attributable, not predictive."
        ),
        "sequence": [
            {
                "step": 1,
                "event": "Lawful substrate load",
                "detail": (
                    f"{chronology_manifest.get('total_ticks')} ticks from "
                    f"{scenario['catalog_name']} with verified chronology_hash."
                ),
            },
            {
                "step": 2,
                "event": "Topology fold",
                "detail": f"Applied {TOPOLOGY} over the ordered event stream.",
            },
            {
                "step": 3,
                "event": "Dual cognition replay",
                "detail": (
                    f"{COGNITION_A} persistence={rolling_persistence}; "
                    f"{COGNITION_B} persistence={event_reset_persistence}."
                ),
            },
            {
                "step": 4,
                "event": "Execution divergence (geometry)",
                "detail": (
                    "Same substrate and topology; cognition choice changes occupancy persistence — "
                    "interpretable divergence, not alpha claim."
                ),
            },
        ],
        "non_claims": [
            "No predictive edge",
            "No autonomous trading",
            "No ontology expansion",
        ],
    }

    print_header("Step 6 — Causal narrative (bounded)")
    print(narrative["bounded_claim"])
    for item in narrative["sequence"]:
        print(f"  {item['step']}. {item['event']}: {item['detail']}")

    return narrative


def write_report(
    provenance: dict[str, Any],
    manifest_path: str,
    attestation: dict[str, Any],
    equivalence: dict[str, Any],
    narrative: dict[str, Any],
) -> str:
    os.makedirs(REPORT_DIR, exist_ok=True)
    report_path = os.path.join(REPORT_DIR, "tier1_observability_report.json")
    report = {
        "demo_scope": "docs/governance/DEMO_SCOPE.md",
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "provenance": provenance,
        "manifest_path": os.path.relpath(manifest_path, PROJECT_ROOT),
        "manifest_id": json.load(open(manifest_path)).get("manifest_id"),
        "attestation": attestation,
        "equivalence": equivalence,
        "causal_narrative": narrative,
        "command_sequence": [
            "./chrono demo",
            f"python3 scripts/run_tier1_observability_demo.py --scenario primary",
        ],
    }
    with open(report_path, "w") as handle:
        json.dump(report, handle, indent=2, sort_keys=True)
    print_header("Evidence report")
    print(f"Written: {os.path.relpath(report_path, PROJECT_ROOT)}")
    return report_path


def run_smoke(scenario_key: str) -> int:
    scenario = SCENARIOS[scenario_key]
    if not os.path.exists(scenario["substrate_file"]):
        print(f"[FAIL] Substrate not found: {scenario['substrate_file']}", file=sys.stderr)
        return 1

    print_header(f"Tier 1 Smoke — {scenario_key}")
    print("Scope: docs/governance/DEMO_SCOPE.md (provenance + single replay only)\n")

    ensure_trace_replay()
    chronology_manifest = load_chronology_manifest(scenario["manifest_file"])
    verify_provenance(scenario, chronology_manifest)
    artifact_dir = run_replays(scenario, single_cognition=COGNITION_A)
    trace_summary = os.path.join(artifact_dir, COGNITION_A, "trace_summary.json")
    if not os.path.exists(trace_summary):
        raise RuntimeError(f"Expected replay artifact missing: {trace_summary}")

    with open(trace_summary) as handle:
        summary = json.load(handle)
    print(f"[OK] Smoke passed — {scenario['catalog_name']} persistence={summary.get('persistence')}")
    return 0


def run_demo(scenario_key: str = "primary") -> int:
    scenario = SCENARIOS[scenario_key]
    if not os.path.exists(scenario["substrate_file"]):
        print(f"[FAIL] Substrate not found: {scenario['substrate_file']}", file=sys.stderr)
        return 1

    print_header("Tier 1 Replay-Certified Observability Demo")
    print("Scope: docs/governance/DEMO_SCOPE.md")
    print("Invariant: demonstration must not redefine identity\n")

    ensure_trace_replay()
    chronology_manifest = load_chronology_manifest(scenario["manifest_file"])
    provenance = verify_provenance(scenario, chronology_manifest)
    artifact_dir = run_replays(scenario)
    manifest_path = emit_manifest(scenario, chronology_manifest, artifact_dir)
    attestation = verify_manifest(manifest_path)
    equivalence = certify_equivalence(manifest_path)
    narrative = build_causal_narrative(scenario, chronology_manifest, artifact_dir)
    write_report(provenance, manifest_path, attestation, equivalence, narrative)

    print("\n[OK] Tier 1 observability demo complete.")
    print("Third-party path: ./chrono demo")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Tier 1 replay-certified observability demo (DEMO_SCOPE.md)"
    )
    parser.add_argument(
        "--scenario",
        choices=sorted(SCENARIOS.keys()),
        default="primary",
        help="Bounded scenario preset (default: primary)",
    )
    parser.add_argument(
        "--smoke",
        action="store_true",
        help="CI smoke: provenance verification + single deterministic replay only",
    )
    args = parser.parse_args()
    try:
        if args.smoke:
            return run_smoke(args.scenario)
        return run_demo(args.scenario)
    except subprocess.CalledProcessError as exc:
        print(f"[FAIL] Command failed: {exc}", file=sys.stderr)
        return exc.returncode or 1
    except RuntimeError as exc:
        print(f"[FAIL] {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
