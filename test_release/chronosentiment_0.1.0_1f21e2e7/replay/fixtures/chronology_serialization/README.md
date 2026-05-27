# Chronology Serialization Fixtures

Executable byte-evidence base for V-006 capture schema governance.
These fixtures freeze **lawful persisted chronology bytes** before any producer
consolidation, struct extraction, or emission-path ratification.

## Purpose

Chronology identity is bound to **emitted JSONL line bytes**, not Rust struct
field alignment. These fixtures prove:

- lawful dialect samples exist as frozen byte artifacts,
- `chronology_hash` streaming algorithm is reproducible,
- future producer changes can be checked for byte mutation before ratification.

This corpus is **not** a parser authority and **not** authorization to migrate
producers. It is evidence infrastructure only.

## Fixture Layout

Each fixture directory contains:

| File | Role |
|------|------|
| `fixture_meta.json` | fixture identity, dialect class, source lineage, excerpt hash |
| `substrate_excerpt.jsonl` | byte-frozen tick JSONL excerpt (exact persisted bytes) |
| `manifest.json` | byte-frozen manifest sample from lawful source universe |
| `first_line.hex` | hex encoding of first JSONL line bytes (ordering guarantee) |

Repository-level index:

| File | Role |
|------|------|
| `reference_hashes.json` | cross-checks streaming `chronology_hash` vs full source files |

## Hash Algorithm (Frozen)

```text
chronology_hash = SHA256( line_1_bytes || line_2_bytes || ... || line_n_bytes )
```

Where each `line_k_bytes` includes the trailing `\n` newline as persisted.

**Not** the same as `SHA256(entire_file_bytes)` unless the file has no trailing
partial line anomalies.

## Dialect Coverage

| Fixture ID | Dialect | Tick law | Manifest law |
|------------|---------|----------|--------------|
| `live_dialect_b_btcusdt_1779545161` | B | 5-field NormalizedTick; ms tick timestamps | slim 6-field; **seconds** bounds |
| `historical_dialect_a_cpi_shock_tick` | A | 5-field NormalizedTick; ms tick timestamps | full 10-field; ms bounds |
| `yahoo_dialect_c_btcusd_crossfeed` | C | 5-field NormalizedTick; ms tick timestamps | slim + `provenance`; ms bounds |

## Timestamp Interpretation Guarantees

| Field | Dialect A/C | Dialect B (historical live) |
|-------|-------------|----------------------------|
| tick `timestamp` | milliseconds | milliseconds |
| manifest `capture_start` / `capture_end` | milliseconds | **seconds** (historical defect; admissible) |

Tools must not infer manifest bound units without dialect classification.

## Serialization Guarantees Under Test

Fixtures freeze the following byte-level properties:

1. JSON key order as emitted by source producers (`serde_json` default object order)
2. Float formatting as emitted (e.g. `75108.16`, not padded variants)
3. Trailing newline per JSONL line
4. Line ordering preserved from source chronology
5. 5-field tick schema (`symbol`, `timestamp`, `price`, `volume`, `is_buyer_maker`)

## Verification

```bash
python3 scripts/verify_chronology_byte_fixtures.py
python3 scripts/verify_chronology_producer_ratification.py --producer capture_daemon
```

Expected: all fixtures pass excerpt hash verification and reference hash checks.

## Governance References

- `docs/governance/V006_CAPTURE_SCHEMA_SCOPE.md`
- `docs/governance/V006_MANIFEST_DIALECT_POLICY.md`
- `docs/governance/V006_LIVE_CAPTURE_AUTHORITY_DECISION.md`
- `docs/governance/V006_CHRONOLOGY_BYTE_FIXTURES.md`

## Non-Goals

- Does not authorize `capture_types.rs`
- Does not rewrite persisted chronology
- Does not ratify `capture_daemon.rs`
- Does not prove full-universe hash equivalence (excerpt-scoped only)
