# V-001 Routing Equivalence Scope

## Status

**Phase:** 3A — Strategy Identity Law Operationalization  
**Gate:** Edge-decay routing contract  
**Date:** 2026-05-26  
**Pre-routing evidence archive:** `fixtures/strategy_identity/archive/pre_routing_2026-05-26/differential_report.json`
**Post-routing delta:** `docs/governance/V001_EDGE_DECAY_ROUTING_DELTA.md`

## Purpose

This document declares the replay-equivalence scope for routing legacy strategy
identity parser call sites through `core/src/strategy_id.rs`. It must exist before
any operational parser routing change.

## Current Operational State

Operational parser authority is still split:

| Legacy Path | Current Role |
|-------------|--------------|
| `core/src/edge_decay.rs::parse_strategy_from_id_local` | Routed through `core/src/strategy_id.rs::parse_strategy_id` |
| `services/api/src/strategy_id_parse.rs::parse_strategy_id_full` | API inspect/compare parser lineage |
| `core/src/strategy_id.rs::parse_strategy_id` | Canonical parser candidate and edge-decay routing target |

The pre-routing differential report records the final known dual-lineage state
before the first production call site was routed.

## First Routing Candidate

The first routing change should route one legacy path at a time. The initial
candidate is:

```text
core/src/edge_decay.rs::parse_strategy_from_id_local
```

Rationale: the edge-decay local parser is the line with known 19-field
`entry_offset` interpretation drift relative to `ga::strategy_to_id` and the
API parser.

## Expected Admissibility Changes

| Artifact Class | Expected Change |
|----------------|-----------------|
| Canonical 13-field `STRAT_...` IDs | No admissibility change expected |
| Canonical 19-field `STRAT_...` IDs | No admissibility change expected |
| Malformed mandatory core genes | No admissibility change expected |
| Historical underscore IDs | No edge-decay admissibility expansion expected in first routing step |

If the routed path begins admitting historical underscore IDs, that is an
admissibility expansion and must be declared before merge.

## Expected Interpretation Changes

| Artifact Class | Expected Change |
|----------------|-----------------|
| 13-field `STRAT_...` IDs | `accepted_same_meaning` |
| 19-field `STRAT_...` IDs | Edge-decay interpretation should normalize to canonical/API meaning |
| Malformed IDs | `rejected_universally` |
| Historical underscore IDs | Remain `rejected_historically_admitted` unless explicitly migrated |

The known 19-field edge-decay shift is expected to move from divergent local
meaning to canonical `entry_offset` at position 13.

## Expected Replay Classification Delta

| Current Classification | Expected Post-Routing Classification |
|------------------------|--------------------------------------|
| `bit_equivalent` | unchanged |
| `divergent_semantics` | reduced or eliminated for routed edge-decay path |
| `all_rejected` | unchanged |
| `divergent_acceptance` | unchanged unless underscore legacy support is intentionally added |

Any new `divergent_acceptance` or `divergent_semantics` classification is an
undeclared replay-governance failure unless predeclared in this document.

## Expected Round-Trip Behavior

Canonical round-trip serialization must use `ga::strategy_to_id` ordering:

```text
entry_offset at position 13
direction_bias at position 14
vol_floor at position 15
mom_floor at position 16
edge_ratio at position 17
participation_threshold at position 18
```

Normalized 13-field `STRAT_...` IDs may expand to the full canonical 19-field
serialization with default extension genes.

## Historical Artifact Visibility

The historical witness:

```text
strat_BTCUSDT_jsonl_window_1_201_2_31_10
```

is historically admitted by the API parser lineage and rejected by the edge-decay
lineage and the current canonical candidate. First routing must not erase this
fact. If future routing admits or rejects this artifact differently, the
post-routing report must classify the change as a replay visibility delta.

## Required Post-Routing Evidence

After any routing step:

1. Run `python3 scripts/verify_strategy_identity_fixtures.py`.
2. Regenerate `fixtures/strategy_identity/differential_report.json`.
3. Compare the post-routing report against the pre-routing archive.
4. Declare whether each changed case is:
   - `accepted_same_meaning`
   - `accepted_normalized`
   - `accepted_divergent_semantics`
   - `rejected_historically_admitted`
   - `rejected_universally`
5. Update `AUTHORITY_MAP.md` with the routing status.

## Non-Goals

This gate does not authorize broad parser cleanup, API error unification, or
historical underscore ID support. Those require separate declared replay scope.
