# Replay Archive Restoration Contract

## Status

**Status:** SATISFIED FOR V-001 - remains active for future replay ratification
**Date:** 2026-05-26
**Baseline anchor:** `replay-governance-baseline-v1`
**Triggering record:** `docs/governance/V001_REPLAY_COHORT_ADJUDICATION.md`
**Operational plan:** `docs/governance/REPLAY_ARCHIVE_RESTORATION_PLAN.md`
**Batch 003 window:** `docs/governance/REPLAY_WINDOW_DECLARATION_BATCH_003.md`
**V-001 decision:** `docs/governance/V001_RATIFICATION_DECISION.md`

## Purpose

This contract defines the minimum replay archive evidence required before a
critical authority migration can be ratified by cohort comparison.

It exists because replay certification failure modes must be governed explicitly.
An incomplete replay archive is not evidence of replay equivalence, and it is
not automatically evidence of semantic divergence. It is a substrate
certifiability failure until the missing replay evidence is restored.

## Minimum Certifiable Substrate

A replay cohort pair is certifiable only when all of the following are true:

1. A baseline/current replay cohort pair exists for the same batch and cohort
   symbol set.
2. A replay manifest exists for both sides of the comparison, including enough
   metadata to bind the run label, cohort, timeline, and substrate identity.
3. The timestamp comparison window is explicit and aligned across both sides.
4. Layer 1 barrier files exist for every compared timestamp on both sides,
   subject only to a declared and bounded exclusion list.
5. Substrate hashes are computable for both sides and persisted in the
   comparator report.
6. The comparator report is persisted as a governed artifact, not treated as
   transient terminal output.

If any condition is missing, the only valid certification result is
`ratification_blocked`.

## Failure Classification

Replay comparison outcomes must be classified before they are used to promote a
critical authority surface:

| Outcome | Meaning | Promotion Effect |
|---------|---------|------------------|
| `replay_equivalent` | Comparator passes with matching replay substrate evidence. | May support `CRITICAL / STABLE` promotion. |
| `strategy_identity_neutral_delta` | Comparator finds a bounded delta that is declared unrelated to strategy identity semantics. | May support promotion only with a dedicated delta record. |
| `ratification_blocked` | Required replay evidence is incomplete or non-computable. | Blocks promotion; does not falsify the migration. |
| `replay_divergent` | Complete replay evidence exists and shows undeclared behavioral divergence. | Blocks promotion and requires root-cause remediation. |

## Restoration Requirements

To restore a blocked replay archive, the restoration pass must produce or verify:

- Baseline and current archive directories for the same batch.
- Matching cohort files and symbol membership.
- Replay manifests for both compared run labels.
- A declared timestamp window.
- Layer 1 barrier files under each compared archive for the declared window.
- A recomputable substrate hash for each side.
- A persisted report from `scripts/compare_replay_equivalence.py`.

Restoration must not edit strategy identity parser behavior. Parser changes
require a separate V-001 scope declaration.

## V-001 Ratification Gate

For V-001, `core/src/strategy_id.rs` may not move from
`CRITICAL domain / TRANSITIONAL impl` to `CRITICAL / STABLE` until this contract
is satisfied and one of the following is recorded:

- `scripts/compare_replay_equivalence.py` passes on a complete replay cohort
  pair anchored to `replay-governance-baseline-v1`.
- A dedicated delta record classifies all replay differences as
  `strategy_identity_neutral_delta`.

For V-001, this gate is now satisfied by the bounded reconstructed batch `003`
pair accepted in `docs/governance/V001_RATIFICATION_DECISION.md`.

Final V-001 state:

```text
authority migration complete
bounded replay cohort restored
replay equivalence certified
CRITICAL / STABLE promotion accepted
```

## Non-Goals

This contract does not authorize parser redesign, API error unification,
manifest schema expansion, or replay engine behavior changes. It governs replay
archive certifiability only.
