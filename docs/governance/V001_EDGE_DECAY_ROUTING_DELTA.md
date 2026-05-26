# V-001 Edge-Decay Routing Delta

## Status

**Phase:** 3A — Strategy Identity Law Operationalization  
**Routing step:** `core/src/edge_decay.rs::parse_strategy_from_id_local`  
**Date:** 2026-05-26  
**Pre-routing evidence:** `fixtures/strategy_identity/archive/pre_routing_2026-05-26/differential_report.json`  
**Post-routing evidence:** `fixtures/strategy_identity/differential_report.json`

## Routing Action

The edge-decay parser lineage now routes through:

```text
core/src/strategy_id.rs::parse_strategy_id
```

The API parser lineage remains operationally independent:

```text
services/api/src/strategy_id_parse.rs::parse_strategy_id_full
```

No historical underscore ID support was added to the canonical candidate or the
edge-decay path.

## Replay Delta Summary

| Fixture Class | Pre-Routing Classification | Post-Routing Classification | Delta |
|---------------|----------------------------|-----------------------------|-------|
| 13-field canonical `STRAT_...` | `bit_equivalent` | `bit_equivalent` | unchanged |
| 19-field `entry_offset` boundary | `divergent_semantics` | `bit_equivalent` | expected normalization |
| Non-numeric mandatory gene | `all_rejected` | `all_rejected` | unchanged |
| Historical underscore ID | `divergent_acceptance` | `divergent_acceptance` | unchanged |

## Interpretation Delta

The known edge-decay 19-field shift is resolved for the routed path:

```text
entry_offset: 35 -> 3
direction_bias: 3 -> 55
vol_floor: 55 -> 22
mom_floor: 22 -> 23
edge_ratio: 23 -> 160
participation_threshold: 160 -> 35
```

The post-routing behavior now matches the canonical candidate and the API parser
for canonical `STRAT_...` identities.

## Admissibility Delta

No admissibility expansion was introduced for historical underscore IDs. The
historical witness:

```text
strat_BTCUSDT_jsonl_window_1_201_2_31_10
```

remains:

```text
rejected by edge-decay routed path
rejected by canonical candidate
accepted by API parser lineage
```

The replay visibility classification therefore remains
`rejected_historically_admitted`.

## Governance Result

This routing step matches the declared equivalence scope in
`docs/governance/V001_ROUTING_EQUIVALENCE_SCOPE.md`:

- `divergent_semantics` was reduced for the routed edge-decay path.
- `divergent_acceptance` was not expanded.
- No new replay classification class appeared.
- API parser routing remains pending and must be handled under a separate
  declared scope.
