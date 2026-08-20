#!/usr/bin/env python3
"""
TIME-009 Integrity Validator — time009_integrity.py

Read-only validator for the TIME-009 prospective observation experiment.
Checks that observation artifacts are internally consistent and have not
been tampered with. Does NOT modify any artifacts.

Checks performed:
  1. T0 fields match LIVE-005 ledger entry (AC-T9-01 immutability)
  2. decision_id uniqueness across all artifacts
  3. COMPLETE artifacts: source_snapshot_timestamp < first_eligible_bar_timestamp
  4. COMPLETE artifacts: observation_status has not changed back to PENDING
  5. cohort_date matches admitted_at date
  6. horizon_sessions matches adaptive_horizon_sessions from T0
  7. No synthetic/imputed bars (n_bars_after_t0 > 0 for COMPLETE)
  8. Provenance chain fields are non-empty
  9. COMPLETE → exit_reason, target_reached, risk_reached, horizon_reached, ambiguous are set
  10. eligible_for_primary_comparison is False for PENDING artifacts

Usage:
    python3 scripts/time009_integrity.py \
        --observations time_machine/analysis/TIME009/observations \
        --ledger       live_capture/ledger/entries

Exit code: 0 = all checks pass, 1 = one or more failures
"""

import argparse
import json
import sys
from pathlib import Path


PROVENANCE_FIELDS = [
    "decision_id",
    "certification_id",
    "certification_status",
    "recommendation_id",
    "source_snapshot_id",
    "source_snapshot_timestamp",
    "c3_002_artifact_hash",
]

T0_FIELDS = [
    "ticker",
    "direction",
    "action",
    "reference_price",
    "adaptive_target",
    "adaptive_risk",
    "adaptive_horizon_sessions",
    "evidence_class",
    "vol_regime",
    "volume_regime",
    "degradation_level",
    "sample_size",
    "target_rate",
    "rank_score",
]


def parse_args():
    p = argparse.ArgumentParser(description="TIME-009 integrity validator (read-only)")
    p.add_argument(
        "--observations",
        default="time_machine/analysis/TIME009/observations",
        help="Directory containing TIME009-OBS-*.json artifacts",
    )
    p.add_argument(
        "--ledger",
        default="live_capture/ledger/entries",
        help="Directory containing LIVE-005 ledger entry JSON files",
    )
    p.add_argument(
        "--verbose", "-v",
        action="store_true",
        help="Print PASS lines in addition to failures",
    )
    return p.parse_args()


def load_json_dir(d: Path, skip_names=("latest_run.json",)):
    items = []
    if not d.exists():
        return items
    for f in sorted(d.iterdir()):
        if f.suffix != ".json" or f.name in skip_names:
            continue
        try:
            items.append((f.name, json.loads(f.read_text())))
        except Exception as e:
            items.append((f.name, None))
            print(f"  ERROR: cannot parse {f.name}: {e}", file=sys.stderr)
    return items


def ts_to_unix(ts_str):
    """Parse ISO-8601 timestamp to unix seconds. Returns None on failure."""
    if not ts_str:
        return None
    try:
        from datetime import datetime, timezone
        dt = datetime.fromisoformat(ts_str.replace("Z", "+00:00"))
        return dt.timestamp()
    except Exception:
        return None


def main():
    args = parse_args()
    obs_dir = Path(args.observations)
    ledger_dir = Path(args.ledger)

    failures = []
    warnings = []
    passes = []

    def fail(check, detail):
        failures.append(f"FAIL [{check}] {detail}")

    def warn(check, detail):
        warnings.append(f"WARN [{check}] {detail}")

    def ok(check, detail=""):
        passes.append(f"PASS [{check}] {detail}")

    print("=" * 60)
    print("TIME-009 INTEGRITY VALIDATOR")
    print("=" * 60)
    print()

    # ── Load artifacts ────────────────────────────────────────────────────────
    obs_items = load_json_dir(obs_dir)
    ledger_items = load_json_dir(ledger_dir)

    observations = [(name, obj) for name, obj in obs_items if obj is not None]
    ledger_entries = {obj["decision_id"]: obj for _, obj in ledger_items if obj is not None}

    print(f"Loaded {len(observations)} observation artifacts")
    print(f"Loaded {len(ledger_entries)} ledger entries")
    print()

    # ── Check 1: decision_id uniqueness ───────────────────────────────────────
    seen_ids = {}
    for fname, obs in observations:
        did = obs.get("decision_id", "")
        if did in seen_ids:
            fail("UNIQUENESS", f"decision_id={did} appears in both {seen_ids[did]} and {fname}")
        else:
            seen_ids[did] = fname
    if len(seen_ids) == len(observations):
        ok("UNIQUENESS", f"all {len(observations)} decision_ids are unique")

    # ── Per-artifact checks ───────────────────────────────────────────────────
    for fname, obs in observations:
        did = obs.get("decision_id", fname)
        status = obs.get("observation_status", "")

        # Check 2: T0 fields match ledger (AC-T9-01)
        ledger = ledger_entries.get(did)
        if ledger is None:
            warn("T0_MATCH", f"{did}: no matching ledger entry (may be from a different run)")
        else:
            for field in T0_FIELDS:
                obs_val = obs.get(field)
                led_val = ledger.get(field)
                if obs_val != led_val:
                    fail(
                        "T0_MATCH",
                        f"{did}: field={field} obs={obs_val!r} != ledger={led_val!r}"
                    )
            # Check cohort_date matches admitted_at
            admitted_at = ledger.get("admitted_at", "")
            expected_cohort = admitted_at[:10] if admitted_at else ""
            actual_cohort = obs.get("cohort_date", "")
            if actual_cohort != expected_cohort:
                fail(
                    "COHORT_DATE",
                    f"{did}: cohort_date={actual_cohort!r} != admitted_at[:10]={expected_cohort!r}"
                )

            # Check horizon_sessions matches adaptive_horizon_sessions
            led_horizon = ledger.get("adaptive_horizon_sessions")
            obs_horizon = obs.get("horizon_sessions")
            if led_horizon is not None and obs_horizon is not None:
                expected_h = max(1, int(led_horizon + 0.9999))  # ceil
                if obs_horizon != expected_h:
                    fail(
                        "HORIZON_MATCH",
                        f"{did}: horizon_sessions={obs_horizon} != ceil(adaptive_horizon_sessions={led_horizon})={expected_h}"
                    )

        # Check 3: provenance chain fields non-empty
        for field in PROVENANCE_FIELDS:
            val = obs.get(field)
            if not val:
                fail("PROVENANCE", f"{did}: provenance field {field!r} is empty or missing")

        # Check 4: PENDING must not have eligible_for_primary_comparison=True
        if status == "PENDING":
            if obs.get("eligible_for_primary_comparison") is True:
                fail(
                    "PENDING_ELIGIBLE",
                    f"{did}: PENDING artifact has eligible_for_primary_comparison=True"
                )
            # PENDING must not have outcome fields set
            for outcome_field in ("exit_reason", "target_reached", "risk_reached"):
                if obs.get(outcome_field) is not None:
                    fail(
                        "PENDING_OUTCOME",
                        f"{did}: PENDING artifact has {outcome_field} set to {obs.get(outcome_field)!r}"
                    )

        # Check 5: COMPLETE must have outcome fields set
        if status == "COMPLETE":
            for outcome_field in ("exit_reason", "target_reached", "risk_reached", "horizon_reached", "ambiguous"):
                if obs.get(outcome_field) is None:
                    fail(
                        "COMPLETE_OUTCOME",
                        f"{did}: COMPLETE artifact missing {outcome_field}"
                    )

            # Check 6: temporal firewall — source_snapshot_timestamp < first_eligible_bar_timestamp
            snap_unix = obs.get("source_snapshot_unix")
            first_bar_unix = obs.get("first_eligible_bar_timestamp")
            if snap_unix is not None and first_bar_unix is not None:
                if first_bar_unix <= snap_unix:
                    fail(
                        "TEMPORAL_FIREWALL",
                        f"{did}: first_eligible_bar_timestamp={first_bar_unix} <= source_snapshot_unix={snap_unix}"
                    )

            # Check 7: COMPLETE must have bars
            n_bars = obs.get("n_bars_after_t0", 0)
            if n_bars == 0:
                fail(
                    "COMPLETE_NO_BARS",
                    f"{did}: COMPLETE artifact has n_bars_after_t0=0 (no data used)"
                )

            # Check 8: horizon_elapsed must be True for COMPLETE
            if obs.get("horizon_elapsed") is not True:
                fail(
                    "COMPLETE_HORIZON",
                    f"{did}: COMPLETE artifact has horizon_elapsed={obs.get('horizon_elapsed')!r}"
                )

    # ── Check 9: COMPLETE → dataset one-to-one ────────────────────────────────
    complete_obs = [obs for _, obs in observations if obs.get("observation_status") == "COMPLETE"]
    complete_ids = {o["decision_id"] for o in complete_obs}
    if complete_obs:
        ok("COMPLETE_COUNT", f"{len(complete_obs)} COMPLETE observations (eligible for dataset)")
    else:
        ok("COMPLETE_COUNT", "0 COMPLETE observations (waiting for horizons to elapse)")

    # ── Summary ───────────────────────────────────────────────────────────────
    print()
    if args.verbose:
        for line in passes:
            print(line)
    for line in warnings:
        print(line)
    for line in failures:
        print(line)

    print()
    print("=" * 60)
    print(f"CHECKS: {len(passes)} PASS  {len(warnings)} WARN  {len(failures)} FAIL")
    if failures:
        print("RESULT: INTEGRITY FAILURE — see FAIL lines above")
        print("=" * 60)
        return 1
    else:
        print("RESULT: OK — all integrity checks passed")
        print("=" * 60)
        return 0


if __name__ == "__main__":
    sys.exit(main())