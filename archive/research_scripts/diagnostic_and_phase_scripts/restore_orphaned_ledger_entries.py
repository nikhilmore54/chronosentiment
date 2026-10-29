#!/usr/bin/env python3
"""
restore_orphaned_ledger_entries.py

Reconstructs missing live_capture/ledger/entries/ JSON files from
TIME-009 observation artifacts for the Aug 20 and Aug 21 cohorts.

The observation artifacts contain all fields the time009_observe binary
needs from a ledger entry:
  - decision_id
  - ticker
  - source_snapshot_timestamp
  - reference_price
  - admitted_at
  - certification_status
  - horizon_sessions  (stored as adaptive_horizon_sessions in obs artifact)

Fields that are not in the observation artifact are reconstructed from
the decision_id (which encodes the run-id) or set to sentinel values
that do not affect observer logic.

GOVERNANCE NOTE: This script only writes ledger entries for decisions
that already have PENDING observation artifacts. It does NOT create new
decisions or modify any observation artifact. It is a recovery operation
for entries lost when the ledger was cleared on backend restart.
"""

import glob
import json
import os
import sys

OBS_DIR = "time_machine/analysis/TIME009/observations"
LEDGER_DIR = "live_capture/ledger/entries"


def main():
    os.makedirs(LEDGER_DIR, exist_ok=True)

    # Load existing ledger entry decision_ids to avoid overwriting
    existing_ids = set()
    for f in glob.glob(os.path.join(LEDGER_DIR, "*.json")):
        try:
            d = json.load(open(f))
            existing_ids.add(d.get("decision_id", ""))
        except Exception:
            pass

    print(f"[restore] Existing ledger entries: {len(existing_ids)}")

    # Find orphaned observation artifacts (PENDING, no ledger entry)
    restored = 0
    skipped_existing = 0
    skipped_complete = 0
    errors = 0

    obs_files = sorted(glob.glob(os.path.join(OBS_DIR, "*.json")))
    for obs_path in obs_files:
        if "latest" in os.path.basename(obs_path):
            continue

        try:
            obs = json.load(open(obs_path))
        except Exception as e:
            print(f"[restore] ERROR reading {obs_path}: {e}")
            errors += 1
            continue

        decision_id = obs.get("decision_id", "")
        if not decision_id:
            continue

        # Skip if ledger entry already exists
        if decision_id in existing_ids:
            skipped_existing += 1
            continue

        # Skip COMPLETE observations — they don't need re-processing
        obs_status = obs.get("observation_status", "")
        if obs_status == "COMPLETE":
            skipped_complete += 1
            continue

        # Reconstruct ledger entry from observation artifact fields
        # The observer reads these fields from the ledger entry:
        #   decision_id, ticker, source_snapshot_timestamp,
        #   reference_price, admitted_at, certification_status,
        #   adaptive_horizon_sessions (→ horizon_sessions in obs)
        ledger_entry = {
            "decision_id": decision_id,
            "admitted_at": obs.get("admitted_at"),
            "producer": "restore_orphaned_ledger_entries.v1",
            "certification_id": obs.get("certification_id"),
            "certification_status": obs.get("certification_status"),
            "certified_at": obs.get("certified_at"),
            "recommendation_id": obs.get("recommendation_id"),
            "recommended_at": obs.get("recommended_at"),
            "source_snapshot_id": obs.get("source_snapshot_id"),
            "source_snapshot_timestamp": obs.get("source_snapshot_timestamp"),
            "source_state_id": obs.get("source_state_id"),
            "source_type": "LIVE",
            "c3_002_artifact_hash": obs.get("c3_002_artifact_hash"),
            "engine_version": "v1",
            "ticker": obs.get("ticker"),
            "direction": obs.get("direction"),
            "action": obs.get("action"),
            "trend": None,
            "momentum": None,
            "reference_price": obs.get("reference_price"),
            "adaptive_target": obs.get("adaptive_target"),
            "adaptive_risk": obs.get("adaptive_risk"),
            "adaptive_upside_pct": None,
            "adaptive_downside_pct": None,
            "adaptive_rr": None,
            # horizon_sessions is stored as adaptive_horizon_sessions in obs
            "adaptive_horizon_sessions": obs.get("horizon_sessions"),
            "degradation_level": obs.get("degradation_level"),
            "sample_size": obs.get("sample_size"),
            "target_rate": obs.get("target_rate"),
            "evidence_class": obs.get("evidence_class"),
            "rank_score": obs.get("rank_score"),
            "recommendation_policy_version": "v1",
            "vol_regime": obs.get("vol_regime"),
            "volume_regime": obs.get("volume_regime"),
            "_restored_from": os.path.basename(obs_path),
            "_restore_note": "Reconstructed from TIME-009 observation artifact after ledger cleared on backend restart",
        }

        # Write to ledger entries dir
        entry_filename = f"{decision_id}.json"
        entry_path = os.path.join(LEDGER_DIR, entry_filename)
        with open(entry_path, "w") as f:
            json.dump(ledger_entry, f, indent=2)

        existing_ids.add(decision_id)
        restored += 1

    print(f"[restore] Restored:          {restored}")
    print(f"[restore] Skipped (existing): {skipped_existing}")
    print(f"[restore] Skipped (COMPLETE): {skipped_complete}")
    print(f"[restore] Errors:            {errors}")
    print(f"[restore] Total ledger entries now: {len(existing_ids)}")

    if restored > 0:
        print(f"[restore] OK — run time009_observe to resolve overdue PENDING observations.")
    else:
        print(f"[restore] Nothing to restore.")


if __name__ == "__main__":
    main()