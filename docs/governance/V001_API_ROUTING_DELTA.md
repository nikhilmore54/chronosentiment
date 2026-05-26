# V-001 API Routing Delta

## Status

**Phase:** 3A — Strategy Identity Law Operationalization  
**Routing step:** `services/api/src/strategy_id_parse.rs::parse_strategy_id_full`  
**Date:** 2026-05-26  
**Pre-API-routing evidence:** `fixtures/strategy_identity/archive/pre_api_routing_2026-05-26/differential_report.json`  
**Post-routing evidence:** `fixtures/strategy_identity/differential_report.json`  
**Doctrine:** `docs/governance/V001_API_ADMISSIBILITY_CONTRACT.md`

## Routing Action

The API parser now routes through:

```text
core/src/strategy_id.rs::parse_strategy_id_with_compatibility
```

Native canonical parsing remains closed over `STRAT_...` identities. Legacy
underscore support exists only as compatibility translation and does not expand
future canonical serialization law.

## Replay Delta Summary

| Fixture Class | Pre-API-Routing Outcome | Post-API-Routing Outcome | Delta |
|---------------|-------------------------|--------------------------|-------|
| 13-field canonical `STRAT_...` | `accepted_same_meaning` | `accepted_same_meaning` | unchanged |
| 19-field `entry_offset` boundary | `accepted_same_meaning` after edge-decay routing | `accepted_same_meaning` | unchanged |
| Non-numeric mandatory gene | `rejected_universally` | `rejected_universally` | unchanged |
| Historical underscore ID | `rejected_historically_admitted` | `accepted_normalized` | expected compatibility translation |

## Compatibility Translation

The historical witness:

```text
strat_BTCUSDT_jsonl_window_1_201_2_31_10
```

now remains replay-addressable through the API routed path as:

```text
STRAT_201v2v31v10v0v50v30v20v100v100v100v75v0v0v50v20v20v150v30
```

The original underscore string remains historical/source identity evidence. It
is not future canonical serialization.

## Authority Result

`core/src/strategy_id.rs` now owns:

- native canonical strategy identity parsing,
- explicit compatibility translation for historical underscore IDs,
- canonical `STRAT_...` round-trip serialization for routed paths.

The compatibility adapter is not a second parser authority. It is a governed
translation boundary under the selected `normalized_compatibility` doctrine.

## Verification

Required verification after routing:

```text
python3 scripts/verify_strategy_identity_fixtures.py
cargo check -p chronosentiment_core --lib
cargo check -p api
```

`cargo check -p api` currently emits pre-existing warnings unrelated to V-001
unused imports/variables and dead code. It exits successfully.

## Governance Result

This routing step matches the declared admissibility contract:

- Underscore IDs remain historically visible.
- Underscore IDs are replay-addressable through compatibility translation.
- Underscore IDs are excluded from future canonical serialization.
- Canonical `STRAT_...` interpretation remains unchanged.
- No new replay classification class appeared.
