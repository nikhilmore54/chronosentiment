# Replay Archive Restoration Plan

## Status

**Status:** ACTIVE - operational restoration only
**Date:** 2026-05-26
**Baseline anchor:** `replay-governance-baseline-v1`
**Governing contract:** `docs/governance/REPLAY_ARCHIVE_RESTORATION_CONTRACT.md`
**V-001 adjudication record:** `docs/governance/V001_REPLAY_COHORT_ADJUDICATION.md`
**Window declaration:** `docs/governance/REPLAY_WINDOW_DECLARATION_BATCH_003.md`
**Target cohort pair:** batch `003`, `live` vs `replay_equiv`

## Purpose

This plan converts the V-001 replay ratification blocker from unknown replay
incompleteness into enumerated restoration obligations.

The remaining work is operational replay archive restoration, not strategy
identity redesign. The objective is to restore one certifiable replay cohort
pair so `scripts/compare_replay_equivalence.py` can produce a governed
ratification result.

## Semantic Freeze

V-001 is in semantic freeze pending replay ratification.

Until a certifiable replay substrate exists, do not change:

- `core/src/strategy_id.rs` parser behavior.
- Compatibility translation doctrine for historical underscore IDs.
- Fixture taxonomy or expected replay outcome vocabulary.
- Canonical serialization format.
- Edge-decay or API parser routing.
- Historical admissibility policy.

The restoration posture is:

```text
stable semantic authority
variable replay substrate
```

Any semantic change during restoration would make replay adjudication
non-attributable and requires a separate V-001 scope declaration.

## Current Inventory

Observed local target:

- Live archive: `state_archive/batches/batch_003/runs/live`
- Replay archive: `state_archive/batches/batch_003/runs/replay_equiv`
- Frozen substrate manifest: `state_archive/candles/batch_003/manifest.json`
- Last comparator report:
  `state_archive/batches/batch_003/runs/live/metadata/replay_equivalence_report.json`

Frozen substrate manifest evidence:

- `timeline_fingerprint`: `7dac2292bf15e994`
- `substrate_hash`: `e4ccc5b72b9b90c5`
- `timeline_intervals`: `366`
- `symbols_frozen`: `500`
- `timeline_first_ts`: `1778643900`
- `timeline_last_ts`: `1779181800`

Comparator evidence from the blocked adjudication:

- `barriers_compared`: `26`
- `ts_missing_live`: `2`
- `ts_missing_replay`: `24`
- `live_substrate_hash`:
  `008ca7a87ad3eaa085ea3d7ac474babe73573a5142df2a4da7c78d5ecce504c6`
- `replay_substrate_hash`: `none`
- `substrate_hash_certified`: `false`

## Restoration Obligation Table

| Required Artifact | Present? | Restorable? | Source | Blocking? |
|-------------------|----------|-------------|--------|-----------|
| Live archive directory | yes | n/a | `batch_003/runs/live` | no |
| Replay archive directory | partial | yes | `batch_003/runs/replay_equiv` | yes |
| Live replay manifest | no | maybe | live run regeneration or manifest reconstruction | yes |
| Replay manifest | no | yes | replay materialization | yes |
| Frozen substrate manifest | yes | n/a | `state_archive/candles/batch_003/manifest.json` | no |
| Frozen timeline fingerprint | yes | n/a | frozen substrate manifest | no |
| Live aligned Layer 1 barriers | partial | maybe | archived live barriers | yes |
| Replay aligned Layer 1 barriers | no for compared window | yes | replay regeneration | yes |
| Aligned timestamp window | partial | yes | declared replay window | yes |
| Live substrate hash | yes | n/a | comparator report | no |
| Replay substrate hash | no | yes | replay regeneration and comparator | yes |
| Persisted comparator report | yes, failing | yes | comparator output | yes until passing or declared delta |

The replay archive contains barrier files for an earlier frozen window, while
the live comparison window extends later than the frozen substrate manifest's
`timeline_last_ts`. This is a materialization mismatch, not a parser finding.

## Operational Inspection - 2026-05-26

Read-only inspection of the target archives found:

| Archive | Manifest Count | Symbol Dirs | Barrier Files | Barrier Timestamp Count | Barrier Timestamp Range |
|---------|----------------|-------------|---------------|-------------------------|-------------------------|
| `batch_003/runs/live` | 0 | 498 | 5507 | 21 | `1779270900` to `1779357300` |
| `batch_003/runs/replay_equiv` | 0 | 492 | 18518 | 50 | `1778643900` to `1778658600` |

Live step evidence:

- `live_session_steps.jsonl` contains 41 records.
- 35 records are committed barriers.
- The committed live-step window has 26 unique timestamps:
  `1779183000` to `1779357300`.
- The frozen substrate window is `1778643900` to `1779181800`.
- No committed live-step timestamp falls inside the frozen substrate window.

Restoration classification:

```text
manifest reconstruction alone is insufficient
timestamp windows are non-overlapping
cohort pair must be regenerated or restored around one declared shared window
```

The current `live` and `replay_equiv` archives are not a certifiable pair
because they bind to different effective chronology windows. Treating them as
ratification inputs would create a false replay-divergence signal unrelated to
strategy identity.

## Materialization Probe - 2026-05-26

Before writing any restoration archive, `cs-ingest` was built and the frozen
timeline was checked read-only:

```bash
./target/release/cs-ingest timeline \
  --batch-id 3 \
  --cohort cohorts/batch_003.txt \
  --candle-root state_archive/candles
```

The probe found that the frozen manifest and the current candle files disagree:

| Field | Manifest | Loaded by `cs-ingest` |
|-------|----------|-----------------------|
| `timeline_fingerprint` | `7dac2292bf15e994` | `0116a4aadaf1e82f` |
| `timeline_intervals` | `366` | `516` |
| `total_bars` | `136145` | `190334` |
| timestamp range | `1778643900` to `1779181800` | `1778643900` to `1779443700` |

Materialization is therefore blocked before replay archive writes. The current
`state_archive/candles/batch_003` directory is not manifest-consistent, so it
cannot anchor a certified replay pair until the manifest/content mismatch is
resolved.

Restoration classification:

```text
frozen substrate manifest/content mismatch
materialization blocked before archive writes
no replay equivalence classification possible
```

## Substrate Provenance Probe - 2026-05-26

Read-only timestamp extraction from `state_archive/candles/batch_003/symbols`
shows that the original manifest-bounded substrate is recoverable as a slice
inside the current candle root:

| Field | Manifest | Full Current Candle Root | Manifest-Bounded Slice |
|-------|----------|--------------------------|------------------------|
| timeline fingerprint | `7dac2292bf15e994` | `0116a4aadaf1e82f` | `7dac2292bf15e994` |
| timeline intervals | `366` | `516` | `366` |
| total bars | `136145` | `190334` | `136145` |
| timestamp range | `1778643900` to `1779181800` | `1778643900` to `1779443700` | `1778643900` to `1779181800` |
| symbols present | `500` | `500` | `500` |

Additional current candle intervals begin after the manifest window:

```text
extra interval count = 150
extra interval range = 1779335100 -> 1779443700
```

Restoration classification:

```text
current candle root is additive relative to the sealed manifest window
original 366-interval substrate is recoverable as a bounded slice
historical substrate recovery can proceed without mutating manifest.json
```

The manifest remains the authority for the certified batch `003` restoration
window. The full current candle root must not be treated as the certified
substrate unless a separate supersession record is declared.

## Bounded Replay Materialization - 2026-05-26

The manifest-bounded replay side was materialized into a new non-destructive run
label:

```bash
python3 scripts/run_nse_cohort.py \
  --batch-id 3 \
  --run-label replay_equiv_manifest_7dac \
  --fresh \
  --start-interval 0 \
  --max-intervals 366
```

This did not overwrite the existing `live` or `replay_equiv` archives.

Materialized replay archive:

```text
state_archive/batches/batch_003/runs/replay_equiv_manifest_7dac
```

Observed output:

| Field | Value |
|-------|-------|
| intervals run | `366` |
| persisted ticks | `128523` |
| corridors | `266` |
| dedupe skipped | `0` |
| duration | `49.25s` |

Read-only archive inspection found:

| Field | Value |
|-------|-------|
| symbol dirs | `498` |
| barrier files | `135413` |
| barrier timestamp count | `366` |
| barrier timestamp range | `1778643900` to `1779181800` |
| barrier timestamp fingerprint | `7dac2292bf15e994` |
| matches manifest fingerprint | `true` |
| matches manifest interval count | `true` |
| matches manifest range | `true` |

This restores the replay-side Layer 1 barrier window for the declared
manifest-bounded chronology.

Remaining blockers:

- No replay manifest was emitted for `replay_equiv_manifest_7dac`.
- No live-side archive exists for the declared window.
- No certifiable baseline/current pair exists yet.
- No comparator ratification is meaningful until both sides satisfy
  `docs/governance/REPLAY_ARCHIVE_RESTORATION_CONTRACT.md`.

Restoration classification:

```text
replay-side declared window materialized
manifest completeness still blocked
live-side declared window still unresolved
ratification still blocked
```

## Bounded Pair Comparator Run - 2026-05-26

A matching bounded live-side archive was materialized under a new
non-destructive run label:

```text
state_archive/batches/batch_003/runs/live_manifest_7dac
```

Manifests were emitted for both bounded restoration labels:

```text
state_archive/batches/batch_003/runs/live_manifest_7dac/manifests/ingestion_live_manifest_7dac.json
state_archive/batches/batch_003/runs/replay_equiv_manifest_7dac/manifests/ingestion_replay_equiv_manifest_7dac.json
```

Comparator command:

```bash
python3 scripts/compare_replay_equivalence.py \
  --batch-id 3 \
  --live-label live_manifest_7dac \
  --replay-label replay_equiv_manifest_7dac \
  --ts-min 1778643900 \
  --ts-max 1779181800
```

Persisted report:

```text
state_archive/batches/batch_003/runs/live_manifest_7dac/metadata/replay_equivalence_report.json
```

Observed report summary:

| Field | Value |
|-------|-------|
| `barriers_compared` | `366` |
| `ts_missing_live` | `0` |
| `ts_missing_replay` | `0` |
| `ts_tick_mismatch` | `0` |
| `ts_corridor_mismatch` | `0` |
| `ts_symbol_mismatch` | `0` |
| `replay_equivalence` | `true` |
| `substrate_hash_certified` | `true` |
| `live_substrate_hash` | `5f463caa0583acc354eb75b6897fd1658a9df97586a0f1b987199adc63fcb434` |
| `replay_substrate_hash` | `5f463caa0583acc354eb75b6897fd1658a9df97586a0f1b987199adc63fcb434` |

Restoration classification:

```text
bounded reconstructed pair is replay_equivalent
declared window is certifiable
no parser semantics changed
```

Governance note: this result certifies the reconstructed bounded cohort pair.
It should not be confused with evidence from the original non-overlapping
`live` archive. Promotion of V-001 still requires accepting this reconstructed
bounded pair as the ratification substrate under
`docs/governance/REPLAY_ARCHIVE_RESTORATION_CONTRACT.md`.

Ratification decision:

```text
docs/governance/V001_RATIFICATION_DECISION.md
```

The bounded reconstructed pair is accepted as the V-001 certification substrate.

## Restoration Lanes

1. Confirm the target timestamp window.
   - Decide whether batch `003` ratification should use the existing live window
     or a newly materialized window bounded by the frozen substrate.
   - Record the window before rerunning replay.
   - Do not compare the existing live-step window against the current
     `replay_equiv` archive; their timestamp ranges do not overlap.
   - Batch `003` now uses the frozen-substrate-bounded window declared in
     `docs/governance/REPLAY_WINDOW_DECLARATION_BATCH_003.md`.

2. Recreate or restore the replay side.
   - Use the existing frozen substrate manifest as the replay input anchor.
   - Materialize `batch_003/runs/replay_equiv` through the canonical replay path.
   - Ensure Layer 1 barriers exist for every declared timestamp in the target
     window.
   - Materialize only the manifest-bounded 366-interval slice; the full current
     candle root has additional intervals and does not match the sealed
     manifest fingerprint.

3. Ensure manifest completeness.
   - Produce a manifest for the live side or explicitly document why the live
     side is incremental-only and which metadata substitutes for manifest
     identity.
   - Produce a replay manifest for `replay_equiv_manifest_7dac` or the final
     selected replay run label.
   - Manifests must bind batch, cohort, run label, timeline fingerprint, and
     substrate identity.

4. Recompute replay hashes.
   - Rerun `scripts/compare_replay_equivalence.py`.
   - Require non-`none` substrate hashes for both sides unless a bounded
     exclusion is declared.

5. Persist and classify the comparator report.
   - Store the report as governed evidence.
   - Classify the outcome under
     `docs/governance/REPLAY_ARCHIVE_RESTORATION_CONTRACT.md`.

## Materialization Protocol Placeholder

Replay materialization must be reproducible before V-001 ratification can close.
The follow-up protocol should document:

- Exact binary: `cs-ingest` / `cs-ingest replay-step`.
- Exact wrapper: `scripts/run_nse_cohort.py`.
- Exact frozen substrate: `state_archive/candles/batch_003/manifest.json`.
- Exact cohort file: `cohorts/batch_003.txt`.
- Exact run labels: `live` and `replay_equiv`.
- Exact output structure under `state_archive/batches/batch_003/runs/`.
- Expected manifest path and schema fields.
- Expected comparator invocation.

Do not generalize this into a replay platform before batch `003` is certifiable.

## Ratification Command

The bounded restoration pair has been compared with:

```bash
python3 scripts/compare_replay_equivalence.py \
  --batch-id 3 \
  --live-label live_manifest_7dac \
  --replay-label replay_equiv_manifest_7dac \
  --ts-min 1778643900 \
  --ts-max 1779181800
```

Then classify the result strictly:

| Outcome | Meaning |
|---------|---------|
| `replay_equivalent` | Full replay certification. |
| `strategy_identity_neutral_delta` | Governed, declared non-identity replay delta. |
| `replay_divergent` | Complete replay evidence shows undeclared divergence. |
| `ratification_blocked` | Restoration still incomplete. |

Final classification:

```text
replay_equivalent
```

## Non-Goals

Do not use this restoration phase to perform:

- Phase 3B abstraction.
- Vocabulary neutralization.
- Multi-domain generalization.
- Replay engine redesign.
- Strategy identity fixture expansion.
- Parser feature additions.
- API surface broadening.
- Certification format redesign.

The current repository state is:

```text
semantic topology stabilized
bounded replay substrate certified
```
