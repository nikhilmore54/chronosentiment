# Replay Window Declaration - Batch 003

## Status

**Status:** ACTIVE - window declaration before replay materialization
**Date:** 2026-05-26
**Baseline anchor:** `replay-governance-baseline-v1`
**Governing contract:** `docs/governance/REPLAY_ARCHIVE_RESTORATION_CONTRACT.md`
**Restoration plan:** `docs/governance/REPLAY_ARCHIVE_RESTORATION_PLAN.md`
**Target cohort pair:** batch `003`, `live` vs `replay_equiv`

## Purpose

This declaration makes replay comparability an explicit precondition for V-001
ratification. The prior comparator run evaluated non-overlapping chronology
windows, so its failure cannot be used as evidence of strategy identity
divergence.

The chosen window below is the only admissible target for the next batch `003`
restoration attempt unless this declaration is replaced by a newer governed
record.

## Declared Certified Window

The certified restoration window is bounded by the frozen substrate manifest:

```text
window_start_ts = 1778643900
window_end_ts   = 1779181800
```

Source:

```text
state_archive/candles/batch_003/manifest.json
```

Frozen substrate identity:

```text
timeline_fingerprint = 7dac2292bf15e994
substrate_hash       = e4ccc5b72b9b90c5
timeline_intervals   = 366
symbols_frozen       = 500
```

## Rationale

The frozen substrate is the only existing batch `003` evidence with a complete
timeline fingerprint and substrate hash. It is therefore the safest anchor for
restoring a certifiable replay cohort pair.

The existing live-step window is excluded from ratification because it is outside
the frozen substrate:

```text
live committed window = 1779183000 -> 1779357300
frozen substrate      = 1778643900 -> 1779181800
replay barriers       = 1778643900 -> 1778658600
```

No committed live-step timestamp falls inside the frozen substrate window.
Comparing the current live-step window against the current replay archive would
test chronology misalignment, not strategy identity equivalence.

## Excluded Chronology Ranges

The following chronology ranges are excluded from V-001 replay ratification for
batch `003`:

- Existing live committed step range: `1779183000` to `1779357300`.
- Existing live Layer 1 barrier range: `1779270900` to `1779357300`.
- Any replay archive material outside `1778643900` to `1779181800`.

These exclusions are not semantic exclusions. They only prevent non-comparable
chronology from entering the V-001 ratification comparison.

## Replay Materialization Inputs

The next restoration attempt must use:

- Cohort file: `cohorts/batch_003.txt`
- Frozen substrate manifest: `state_archive/candles/batch_003/manifest.json`
- Batch id: `003`
- Live label: `live`
- Replay label: `replay_equiv`
- Declared window: `1778643900` to `1779181800`

## Pre-Materialization Blocker

A read-only `cs-ingest timeline` probe found that the current candle files do
not match `state_archive/candles/batch_003/manifest.json`:

```text
manifest fingerprint = 7dac2292bf15e994
loaded fingerprint   = 0116a4aadaf1e82f
manifest intervals   = 366
loaded intervals     = 516
```

This means the declared certified window remains governed by the manifest, but
the current candle directory cannot yet be used as its materialization source.
The manifest/content mismatch must be resolved before replay archive writes.

A follow-up read-only timestamp extraction showed the mismatch is additive:

```text
manifest-bounded slice fingerprint = 7dac2292bf15e994
manifest-bounded slice intervals   = 366
manifest-bounded slice bars        = 136145
full current candle intervals      = 516
extra intervals after manifest     = 150
```

The original certified chronology is therefore recoverable as the declared
window slice. Materialization must use that bounded slice rather than the full
current candle root.

The replay side has been materialized for this declared window under:

```text
state_archive/batches/batch_003/runs/replay_equiv_manifest_7dac
```

Its persisted Layer 1 barrier timestamps match the declared manifest window:

```text
barrier timestamp fingerprint = 7dac2292bf15e994
barrier timestamp count       = 366
barrier timestamp range       = 1778643900 -> 1779181800
```

This is restoration progress only. It does not ratify V-001 because the archive
still lacks a replay manifest and there is not yet a matching live-side archive
for the declared window.

A matching bounded live-side archive was then materialized under:

```text
state_archive/batches/batch_003/runs/live_manifest_7dac
```

The bounded comparator run passed for:

```text
live_manifest_7dac vs replay_equiv_manifest_7dac
```

within the declared window:

```text
1778643900 -> 1779181800
```

Persisted report:

```text
state_archive/batches/batch_003/runs/live_manifest_7dac/metadata/replay_equivalence_report.json
```

The report classifies the reconstructed bounded pair as replay-equivalent with
matching substrate hash:

```text
substrate_hash_certified = true
live_substrate_hash      = 5f463caa0583acc354eb75b6897fd1658a9df97586a0f1b987199adc63fcb434
replay_substrate_hash    = 5f463caa0583acc354eb75b6897fd1658a9df97586a0f1b987199adc63fcb434
```

This certifies the reconstructed bounded pair only. It does not reclassify the
original non-overlapping `live` archive.

`docs/governance/V001_RATIFICATION_DECISION.md` accepts this bounded pair as the
lawful V-001 certification substrate.

Materialization must preserve V-001 semantic freeze:

- No strategy identity parser behavior changes.
- No compatibility doctrine changes.
- No fixture taxonomy expansion.
- No routing changes.
- No admissibility broadening.

## Expected Archive Outputs

Before rerunning comparator certification, both sides of the cohort pair must
provide:

- A manifest binding batch id, cohort, run label, timeline fingerprint, and
  substrate identity.
- Layer 1 barrier files for the declared window.
- A computable substrate hash over the declared window.
- A persisted comparator report.

If either side cannot provide these outputs, the result remains
`ratification_blocked`.

## Comparator Scope

The comparator must be scoped to the declared certified window. If the existing
tooling cannot express the exact window directly, the materialized archives must
contain only comparable barrier timestamps for the ratification run, or a
bounded exclusion record must be created before certification.

Comparator command after materialization:

```bash
python3 scripts/compare_replay_equivalence.py \
  --batch-id 3 \
  --live-label live_manifest_7dac \
  --replay-label replay_equiv_manifest_7dac \
  --ts-min 1778643900 \
  --ts-max 1779181800
```

The persisted comparator report must be classified under
`docs/governance/REPLAY_ARCHIVE_RESTORATION_CONTRACT.md`.

## Ratification Rule

V-001 may move toward `CRITICAL / STABLE` only if this declared window produces
one of:

- `replay_equivalent`
- `strategy_identity_neutral_delta`

The following outcomes block promotion:

- `ratification_blocked`
- `replay_divergent`

## Non-Goals

This declaration does not authorize replay engine redesign, generalized archive
infrastructure, parser changes, API surface changes, fixture expansion, or
manifest schema redesign.
