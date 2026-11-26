# Strategy Identity Fixtures

This corpus is the executable evidence base for V-001 strategy identity consolidation.
It freezes observed parser behavior before `core/src/strategy_id.rs` becomes the
canonical parser authority.

## Purpose

Strategy identity fixtures preserve observed replay semantics. They are not just
unit tests for parser correctness. Each fixture records an input identity string,
the parser lineage that interpreted it, the parsed `Strategy` shape, and the
expected replay classification for that interpretation.

## Fixture Schema

Each `.jsonl` record uses this shape:

```json
{
  "fixture_id": "stable_snake_case_id",
  "serialized_id": "STRAT_...",
  "parser_source": "path::function",
  "provenance": {
    "type": "historically_observed | boundary_derived | malformed | synthetic",
    "source": "file or rationale",
    "observed_at": "commit, tag, or phase anchor"
  },
  "expected_parser_result": "parsed | rejected",
  "parsed_structure": {},
  "round_trip_serialization": "STRAT_...",
  "expected_classification": "bit_equivalent | semantically_equivalent | accepted_normalized | divergent_but_declared | rejected",
  "notes": "short replay-governance rationale"
}
```

`parsed_structure` is a full `Strategy` field snapshot when parsing succeeds and
`null` when parsing is expected to reject the input.

## Provenance Weight

Historically observed fixtures carry the strongest replay authority because they
prove that a serialized identity existed in repository artifacts. Boundary-derived
fixtures cover known parser split points such as the V-001 `entry_offset` index
shift. Malformed fixtures define admissibility boundaries. Synthetic fixtures are
useful for harness coverage but do not outweigh historical evidence.

## Parser Lineage

`parser_source` must name the legacy path being frozen, not the future canonical
parser. Current V-001 lineages are:

- `core/src/edge_decay.rs::parse_strategy_from_id_local`
- `services/api/src/strategy_id_parse.rs::parse_strategy_id_full`

Additional lineages may be added only as evidence consumers. They do not become
strategy identity authorities.

## Replay Outcome Model

Parser outcome is not binary. V-001 evidence preserves two independent axes:

- `admissibility`: whether the artifact entered replay scope.
- `interpretation`: which `Strategy` meaning was assigned after admission.

Future canonical parser APIs must preserve this distinction. A flat parse error
is not enough for historical replay evidence because it cannot distinguish an
artifact that was never admitted from an artifact that was historically admitted
by a legacy parser lineage.

The canonical outcome vocabulary for differential evidence is:

- `accepted_same_meaning`
- `accepted_normalized`
- `accepted_divergent_semantics`
- `rejected_historically_admitted`
- `rejected_universally`

## Divergence Taxonomy

- `bit_equivalent`: parse and round-trip preserve the serialized identity exactly.
- `semantically_equivalent`: parse preserves strategy meaning but normalizes legacy
  or abbreviated formatting.
- `accepted_normalized`: a historical non-canonical artifact remains
  replay-addressable through explicit compatibility translation and emits
  canonical `STRAT_...` serialization.
- `divergent_semantics`: the same artifact is admitted but assigned different
  strategy meaning by different parser lineages.
- `divergent_acceptance`: artifact admissibility differs across parser lineages.
- `all_rejected`: every registered parser lineage rejects the artifact.

## Differential Report

Run the verifier from the repository root:

```bash
python3 scripts/verify_strategy_identity_fixtures.py
```

The verifier checks every fixture record and writes
`fixtures/strategy_identity/differential_report.json`. That report executes every
registered legacy parser against each unique serialized ID in the frozen corpus
and classifies cross-lineage behavior as:

- `bit_equivalent`
- `semantically_equivalent`
- `divergent_semantics`
- `divergent_acceptance`
- `all_rejected`

Each case also includes an `outcome_category` using the replay outcome model
above. That category is an implementation target for `core/src/strategy_id.rs`,
not a source of parser authority.

The report is generated evidence. It does not define canonical parser law.

## Legacy Path Policy

Legacy parsers must be routed through canonical authority or retired with replay
evidence. Silent semantic coexistence is not allowed on this CRITICAL surface.

## Constitutional Invariant

The canonical parser is not a historical truth generator. Fixtures preserve what
history was observed to mean; `core/src/strategy_id.rs` will define lawful future
interpretation after this evidence base exists.
