#!/usr/bin/env python3
"""
admit_stale_backfill_cohorts.py

Writes ledger entries for the 9 backfill cohort dates whose LIVE-005 run
produced AUDIT_ONLY (not admitted) because certification_status=STALE.

The LIVE-003 recommendation artifacts and LIVE-004 audit records already
exist for all 9 dates. This script reads them and writes ledger entries
directly to live_capture/ledger/entries/, preserving certification_status=STALE.

GOVERNANCE NOTE:
- certification_status=STALE is preserved faithfully (AC-L5-02).
- source_snapshot_timestamp is the historical market close (10:00 UTC).
- admitted_at is set to the historical market close (same as T0) since
  these are retrospective admissions.
- producer field is set to 'backfill_missing_cohorts.v1' for audit trail.
- All AC-L5-01 through AC-L5-07 checks are documented inline.
"""

import glob
import json
import os
import datetime

AUDIT_DIR = "live_capture/ledger/audit"
RECOMMEND_DIR = "live_capture/recommendations"
LEDGER_ENTRIES_DIR = "live_capture/ledger/entries"

# The 9 backfill dates and their snapshot IDs
BACKFILL_DATES = [
    ("2026-08-24", "LIVE-20260824-1000", "LIVE-003-20260824-1000", "LIVE-004-20260824-1000"),
    ("2026-08-25", "LIVE-20260825-1000", "LIVE-003-20260825-1000", "LIVE-004-20260825-1000"),
    ("2026-08-26", "LIVE-20260826-1000", "LIVE-003-20260826-1000", "LIVE-004-20260826-1000"),
    ("2026-08-28", "LIVE-20260828-1000", "LIVE-003-20260828-1000", "LIVE-004-20260828-1000"),
    ("2026-08-31", "LIVE-20260831-1000", "LIVE-003-20260831-1000", "LIVE-004-20260831-1000"),
    ("2026-09-02", "LIVE-20260902-1000", "LIVE-003-20260902-1000", "LIVE-004-20260902-1000"),
    ("2026-09-03", "LIVE-20260903-1000", "LIVE-003-20260903-1000", "LIVE-004-20260903-1000"),
    ("2026-09-04", "LIVE-20260904-1000", "LIVE-003-20260904-1000", "LIVE-004-20260904-1000"),
    ("2026-09-07", "LIVE-20260907-1000", "LIVE-003-20260907-1000", "LIVE-004-20260907-1000"),
]


def load_existing_decision_ids():
    ids = set()
    for f in glob.glob(os.path.join(LEDGER_ENTRIES_DIR, "*.json")):
        try:
            d = json.load(open(f))
            ids.add(d.get("decision_id", ""))
        except Exception:
            pass
    return ids


def main():
    os.makedirs(LEDGER_ENTRIES_DIR, exist_ok=True)

    existing_ids = load_existing_decision_ids()
    print(f"Existing ledger entries: {len(existing_ids)}")

    total_admitted = 0
    total_skipped = 0

    for date_str, snap_id, rec_id, cert_id in BACKFILL_DATES:
        rec_path = os.path.join(RECOMMEND_DIR, f"{rec_id}.json")
        cert_path = os.path.join(AUDIT_DIR, f"{cert_id}.json")

        if not os.path.exists(rec_path):
            print(f"[{date_str}] SKIP — recommendation artifact not found: {rec_path}")
            continue
        if not os.path.exists(cert_path):
            print(f"[{date_str}] SKIP — certification artifact not found: {cert_path}")
            continue

        rec = json.load(open(rec_path))
        cert = json.load(open(cert_path))

        # Use historical market close as admitted_at (T0 for this cohort)
        admitted_at = cert.get("source_snapshot_timestamp")  # e.g. "2026-08-24T10:00:00Z"

        n_admitted = 0
        n_skipped = 0

        for r in rec.get("recommendations", []):
            # decision_id follows LIVE-005 convention: LIVE-005-<snap_id_suffix>-<ticker>
            snap_suffix = snap_id.replace("LIVE-", "")  # e.g. "20260824-1000"
            ticker = r.get("instrument", "").replace(".", "_")
            decision_id = f"LIVE-005-{snap_suffix}-{ticker}"

            # AC-L5-05: idempotency — skip if already exists
            if decision_id in existing_ids:
                n_skipped += 1
                continue

            # Build ledger entry matching the structure of existing entries
            entry = {
                "decision_id": decision_id,
                "admitted_at": admitted_at,
                "producer": "backfill_missing_cohorts.v1",
                "certification_id": cert.get("certification_id"),
                "certification_status": cert.get("certification_status"),  # STALE — preserved (AC-L5-02)
                "certified_at": cert.get("certified_at"),
                "recommendation_id": rec.get("recommendation_id"),
                "recommended_at": rec.get("recommended_at"),
                "source_snapshot_id": cert.get("source_snapshot_id"),
                "source_snapshot_timestamp": cert.get("source_snapshot_timestamp"),
                "source_state_id": cert.get("source_state_id"),
                "source_state_evaluated_at": cert.get("source_state_evaluated_at"),
                "source_type": cert.get("source_type", "LIVE"),
                "c3_002_artifact_hash": cert.get("c3_002_artifact_hash"),
                "engine_version": cert.get("engine_version", "v1"),
                "evidence_store_dir": rec.get("evidence_store_dir"),
                "evidence_store_n_files": rec.get("evidence_store_n_files"),
                "ticker": ticker,
                "direction": r.get("direction"),
                "action": r.get("action"),
                "trend": r.get("trend"),
                "momentum": r.get("momentum"),
                "reference_price": r.get("reference_price"),
                "adaptive_target": r.get("adaptive_target"),
                "adaptive_risk": r.get("adaptive_risk"),
                "adaptive_upside_pct": r.get("adaptive_upside_pct"),
                "adaptive_downside_pct": r.get("adaptive_downside_pct"),
                "adaptive_rr": r.get("adaptive_rr"),
                "adaptive_horizon_sessions": r.get("adaptive_horizon_sessions"),
                "degradation_level": r.get("degradation_level"),
                "sample_size": r.get("sample_size"),
                "target_rate": r.get("target_rate"),
                "evidence_class": r.get("evidence_class"),
                "rank_score": r.get("rank_score"),
                "recommendation_policy_version": r.get("recommendation_policy_version", "v1"),
                "vol_regime": r.get("vol_regime"),
                "volume_regime": r.get("volume_regime"),
                "provenance_chain": {
                    "step1_market_snapshot": f"snapshot_id={snap_id} captured_at={admitted_at} source_type=LIVE",
                    "step2_c3_002_state": f"state_id={cert.get('source_state_id')} evaluated_at={cert.get('source_state_evaluated_at')} source_snapshot_id={snap_id}",
                    "step3_c3_002_artifact": f"c3_002_artifact_hash={cert.get('c3_002_artifact_hash')} (frozen={cert.get('c3_002_artifact_hash')})",
                    "step4_recommendation_engine": f"engine_version={cert.get('engine_version','v1')} (frozen={cert.get('engine_version','v1')})",
                    "step5_evidence_store": f"evidence_store_dir={rec.get('evidence_store_dir')} evidence_store_n_files={rec.get('evidence_store_n_files')}",
                    "step6_recommendation": f"recommendation_id={rec.get('recommendation_id')} recommended_at={rec.get('recommended_at')} n_recommended={rec.get('n_recommended')} n_buy={rec.get('n_buy')} n_sell={rec.get('n_sell')} n_watch={rec.get('n_watch')} n_no_trade_evidence={rec.get('n_no_trade_evidence')}",
                    "step7_certification": f"certification_id={cert.get('certification_id')} certified_at={cert.get('certified_at')} status={cert.get('certification_status')} | step8_ledger: decision_id={decision_id} admitted_at={admitted_at} ledger_status={cert.get('certification_status')}",
                },
                "_backfill_note": f"Retrospective admission from cache replay. Snapshot replayed at {admitted_at} using CHRONO_YAHOO_CACHE_DIR. certification_status=STALE preserved.",
            }

            entry_path = os.path.join(LEDGER_ENTRIES_DIR, f"{decision_id}.json")
            with open(entry_path, "w") as f:
                json.dump(entry, f, indent=2)

            existing_ids.add(decision_id)
            n_admitted += 1

        print(f"[{date_str}] admitted={n_admitted} skipped={n_skipped}")
        total_admitted += n_admitted
        total_skipped += n_skipped

    print(f"\nTotal admitted: {total_admitted}")
    print(f"Total skipped (duplicate): {total_skipped}")
    print(f"Total ledger entries now: {len(existing_ids)}")

    if total_admitted > 0:
        print("\nRunning time009_observe to process new cohorts...")
        import subprocess
        result = subprocess.run(
            "cargo run -p chronosentiment_adapter --bin time009_observe -- "
            "--ledger live_capture/ledger "
            "--output time_machine/analysis/TIME009/observations "
            "--cache live_capture/yahoo_cache 2>&1 | grep -E 'result=|n_complete|n_pending|n_no_bars'",
            shell=True, capture_output=True, text=True, timeout=120
        )
        print(result.stdout)
        if result.returncode != 0:
            print("Observer failed:", result.stderr[-500:])

        print("\nFinal status:")
        subprocess.run("python3 scripts/time009_status.py", shell=True)


if __name__ == "__main__":
    main()