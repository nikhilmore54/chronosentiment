# V-001 API Admissibility Contract

## Status

**Phase:** 3A — Strategy Identity Law Operationalization  
**Gate:** API parser routing / retirement contract  
**Date:** 2026-05-26  
**Baseline tag:** `replay-governance-baseline-v1`  
**Current evidence:** `fixtures/strategy_identity/differential_report.json`  
**Edge-decay routing delta:** `docs/governance/V001_EDGE_DECAY_ROUTING_DELTA.md`
**Selected doctrine:** Historical visibility with compatibility translation
**API routing delta:** `docs/governance/V001_API_ROUTING_DELTA.md`

## Purpose

This document declares the replay-governance scope required before routing or
retiring:

```text
services/api/src/strategy_id_parse.rs::parse_strategy_id_full
```

The API parser is the remaining independently operational historical
admissibility surface. Routing it through `core/src/strategy_id.rs` makes the
canonical parser the singular operational authority for both interpretation and
admissibility.

## Current API Admissibility Surface

The API parser currently admits two broad classes:

| Artifact Class | Current API Behavior |
|----------------|----------------------|
| `STRAT_...` IDs with 13 or more `v`-delimited fields | Parsed with `entry_offset` at position 13 |
| Underscore legacy IDs containing at least four numeric segments | Accepted by reverse numeric fallback |

The historical witness currently depends on API admissibility:

```text
strat_BTCUSDT_jsonl_window_1_201_2_31_10
```

Current differential state:

```text
API parser: accepted
canonical parser: rejected
edge-decay routed path: rejected
classification: divergent_acceptance
outcome: rejected_historically_admitted
```

## Selected Admissibility Doctrine

Historical admissibility does not automatically make an artifact part of future
canonical identity law. It does create a replay-governance obligation to preserve
visibility and, where feasible, replay addressability through explicit
compatibility translation.

The selected doctrine for underscore legacy IDs is:

```text
historically visible
non-canonical
replay-addressable through compatibility translation
excluded from future canonical serialization
```

This doctrine preserves historical evidence without allowing legacy underscore
syntax to contaminate canonical `STRAT_...` identity law.

## Policy Decisions Before Routing

### Underscore ID Policy

| Policy | Meaning |
|--------|---------|
| `replay_addressable` | Underscore IDs remain accepted through canonical or compatibility parsing |
| `normalized_compatibility` | **Selected.** Underscore IDs map to canonical `STRAT_...` form through explicit compatibility translation |
| `historically_visible_rejected` | Underscore IDs are rejected operationally but remain visible in replay evidence |
| `certification_excluded` | Underscore IDs are excluded from certification/replay manifest scope |

Selected policy: `normalized_compatibility`.

### Normalization Policy

Underscore IDs remain admissible only through a compatibility adapter. The
normalized output must be canonical `ga::strategy_to_id` serialization.

Compatibility translation must remain distinguishable from native canonical
parsing in replay evidence. A translated underscore ID is replay-addressable, but
its original serialized form is not canonical and must not be emitted as future
canonical serialization.

### Historical Visibility Policy

Historically observed artifacts must not disappear silently. Underscore legacy
IDs remain historically visible even when translated into canonical `STRAT_...`
form for replay addressing.

The original artifact string must remain visible in evidence records and
post-routing differential reports.

### Certification / Manifest Scope

Underscore legacy IDs participate in:

- API inspect/compare requests through compatibility translation.
- Historical artifact lookup as original observed strings.
- Replay manifests only as historical/source identifiers paired with canonical
  normalized identity.
- Future cohort replay comparisons only through the normalized canonical
  identity and with compatibility provenance.

Underscore legacy IDs do **not** participate as future canonical serialization
forms. Certification surfaces must record the normalized canonical identity and,
where relevant, the original historical source ID.

## Expected Replay Delta Classes

| Scenario | Expected Classification |
|----------|-------------------------|
| API preserves underscore fallback through compatibility adapter | `accepted_normalized` |
| API rejects underscore IDs but preserves evidence visibility | `rejected_historically_admitted` |
| API rejects underscore IDs and drops visibility | governance breach unless explicitly excluded |
| API changes canonical `STRAT_...` interpretation | governance breach unless separately declared |

Expected V-001 API routing result for the historical witness:

```text
strat_BTCUSDT_jsonl_window_1_201_2_31_10
```

is `accepted_normalized`, with the normalized canonical ID:

```text
STRAT_201v2v31v10v0v50v30v20v100v100v100v75v0v0v50v20v20v150v30
```

## Required Evidence Before API Routing

Before touching `services/api/src/strategy_id_parse.rs`:

1. Archive the current `fixtures/strategy_identity/differential_report.json`.
2. Confirm the `normalized_compatibility` doctrine still applies.
3. Declare certification / manifest scope for the specific routing change.
4. Declare expected replay classification deltas.
5. Identify whether `core/src/strategy_id.rs` requires a compatibility adapter.

Compatibility adapter requirement: **yes**. The adapter must be explicit and must
not make underscore syntax part of native canonical parsing.

## Required Evidence After API Routing

After routing or retiring the API parser:

1. [x] Run `python3 scripts/verify_strategy_identity_fixtures.py`.
2. [x] Run `cargo check -p chronosentiment_core --lib`.
3. [x] Run the API crate check if API code is modified.
4. [x] Regenerate `fixtures/strategy_identity/differential_report.json`.
5. [x] Compare against the pre-API-routing archive.
6. [x] Attempt replay cohort comparison against `replay-governance-baseline-v1`.
7. [x] Record replay delta in a dedicated post-routing document.
8. [x] Update `AUTHORITY_MAP.md`.

Replay cohort comparison status: blocked by incomplete batch `003`
`replay_equiv` archive. See
`docs/governance/V001_REPLAY_COHORT_ADJUDICATION.md`. This does not ratify
V-001 as `CRITICAL / STABLE`. Replay archive restoration must satisfy
`docs/governance/REPLAY_ARCHIVE_RESTORATION_CONTRACT.md` before another
ratification attempt can certify the migration.

## Non-Goals

This contract does not authorize API error unification, broad handler rewiring,
or certification manifest format changes. Those require separate declared scope.
