# V-001 Replay Cohort Adjudication

## Status

**Status:** ORIGINAL RUN BLOCKED; BOUNDED RESTORATION RATIFIED
**Date:** 2026-05-26
**Baseline anchor:** `replay-governance-baseline-v1`
**Command executed:** `python3 scripts/compare_replay_equivalence.py --batch-id 3 --live-label live --replay-label replay_equiv`
**Generated report:** `state_archive/batches/batch_003/runs/live/metadata/replay_equivalence_report.json`
**Restoration contract:** `docs/governance/REPLAY_ARCHIVE_RESTORATION_CONTRACT.md`
**Restoration plan:** `docs/governance/REPLAY_ARCHIVE_RESTORATION_PLAN.md`
**Window declaration:** `docs/governance/REPLAY_WINDOW_DECLARATION_BATCH_003.md`
**Bounded restoration report:** `state_archive/batches/batch_003/runs/live_manifest_7dac/metadata/replay_equivalence_report.json`
**Ratification decision:** `docs/governance/V001_RATIFICATION_DECISION.md`

## Purpose

This document records the Phase 3A replay cohort adjudication attempt for V-001
strategy identity consolidation. The goal was to determine whether routing
strategy identity parsing through `core/src/strategy_id.rs` can be promoted from
`CRITICAL domain / TRANSITIONAL impl` toward `CRITICAL / STABLE` by comparing
cohort replay behavior against the sealed governance baseline.

## Tooling Path

The available local replay tooling compares archived cohort runs, not git
revisions directly:

- `scripts/compare_replay_equivalence.py` compares `live` and `replay_equiv`
  archive labels for a batch.
- `scripts/run_nse_cohort.py` can generate isolated replay archives through
  `cs-ingest replay-step` when the release binary and frozen substrate are
  present.
- `scripts/certify_equivalence_v1.py` certifies manifest equivalence for
  historical replay manifests, but it requires a replay manifest with substrate
  metadata.

The only local batch with both `live` and `replay_equiv` labels was batch `003`.

## Result

The comparator exited non-zero. The failure is an archive-completeness blocker,
not evidence of V-001 parser divergence.

Observed summary from the generated report:

```json
{
  "batch_id": 3,
  "live_label": "live",
  "replay_label": "replay_equiv",
  "barriers_compared": 26,
  "ts_missing_live": 2,
  "ts_missing_replay": 24,
  "ts_tick_mismatch": 0,
  "ts_corridor_mismatch": 0,
  "ts_symbol_mismatch": 0,
  "pass": false,
  "frozen_fingerprint": "7dac2292bf15e994",
  "replay_equivalence": false,
  "chronology_confidence": 0.4548,
  "feed_fragmentation": 0.0,
  "provider_consensus": 0.7122,
  "substrate_hash_certified": false,
  "live_substrate_hash": "008ca7a87ad3eaa085ea3d7ac474babe73573a5142df2a4da7c78d5ecce504c6",
  "replay_substrate_hash": "none"
}
```

Primary blocker emitted by the comparator:

- Missing replay manifest at `state_archive/batches/batch_003/runs/replay_equiv/manifests/`.
- Replay side has no Layer 1 barrier hash for the aligned cohort window.
- Most live barriers are missing from the replay archive.

## Adjudication

V-001 is **not ratified as `CRITICAL / STABLE`** by this run.

The correct governance classification remains:

- Parser authority migration: complete.
- Fixture-level parser evidence: complete.
- API admissibility doctrine: complete.
- Replay cohort ratification: blocked by incomplete cohort replay archive.

Because the failure occurs before an equivalent replay substrate exists, this
run cannot classify V-001 as replay-divergent. It also cannot certify replay
equivalence. The only valid classification is **ratification blocked**.

## Required Closure Step

To complete V-001 ratification, satisfy
`docs/governance/REPLAY_ARCHIVE_RESTORATION_CONTRACT.md` by creating or
restoring a complete replay cohort pair anchored to
`replay-governance-baseline-v1`, then rerun:

```bash
python3 scripts/compare_replay_equivalence.py --batch-id 3 --live-label live --replay-label replay_equiv
```

For batch `003`, the rerun must use the certified window declared in
`docs/governance/REPLAY_WINDOW_DECLARATION_BATCH_003.md`.

Promotion of `core/src/strategy_id.rs` to `CRITICAL / STABLE` requires a passing
report or a bounded, declared, strategy-identity-neutral delta.

## Restoration Follow-Up

The original `live` vs `replay_equiv` adjudication remains blocked because those
archives are not comparable. A reconstructed bounded pair was materialized under:

```text
live_manifest_7dac
replay_equiv_manifest_7dac
```

within the declared frozen-substrate window:

```text
1778643900 -> 1779181800
```

The bounded comparator passed with:

```text
barriers_compared = 366
ts_missing_live = 0
ts_missing_replay = 0
replay_equivalence = true
substrate_hash_certified = true
```

This is evidence that the declared bounded restoration substrate is certifiable.
It does not reclassify the original non-overlapping `live` archive as
equivalent.

## Ratification Decision

`docs/governance/V001_RATIFICATION_DECISION.md` accepts the bounded
reconstructed pair as the lawful V-001 replay certification substrate.

Authority outcome:

```text
core/src/strategy_id.rs
CRITICAL / STABLE
```
