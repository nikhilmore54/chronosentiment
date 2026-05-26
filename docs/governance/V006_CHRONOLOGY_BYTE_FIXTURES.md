# V-006 — Chronology Byte Fixtures

**Status:** ACTIVE — byte evidence frozen; producer ratification not authorized  
**Governance class:** replay substrate identity governance (evidence layer)  
**Prerequisites:** `V006_CAPTURE_SCHEMA_SCOPE.md`, `V006_MANIFEST_DIALECT_POLICY.md`, `V006_LIVE_CAPTURE_AUTHORITY_DECISION.md`  
**Constitutional reference:** `.cursor/rules/chronosentiment-core.mdc`, `AUTHORITY_MAP.md` V-006

---

## Purpose

Establish frozen byte-level evidence so future producer convergence must prove:

```text
future producer behavior does not mutate lawful substrate bytes
```

before any:

- `capture_types.rs` extraction,
- `capture_daemon.rs` ratification,
- manifest emission changes,
- replay-sensitive substrate migration.

```text
fixtures before extraction
not shared structs before byte guarantees
```

---

## Evidence Chain Position

```text
observe   → V006_CAPTURE_SCHEMA_SCOPE.md
classify  → replay substrate identity governance
declare law → V006_MANIFEST_DIALECT_POLICY.md
freeze semantics → V006_LIVE_CAPTURE_AUTHORITY_DECISION.md
freeze bytes → this document + fixtures/chronology_serialization/
migrate   → blocked
ratify    → blocked pending producer byte proof against fixtures
```

---

## Frozen Corpus

**Location:** `fixtures/chronology_serialization/`  
**Protocol:** `fixtures/chronology_serialization/README.md`  
**Verifier:** `scripts/verify_chronology_byte_fixtures.py`

### Fixture records

| Fixture ID | Dialect | Source universe | Excerpt lines | Purpose |
|------------|---------|-----------------|---------------|---------|
| `live_dialect_b_btcusdt_1779545161` | B | `core/chronology/live_capture/` | 5 | lawful historical live bytes; seconds manifest bounds |
| `historical_dialect_a_cpi_shock_tick` | A | `core/chronology/historical/2024_cpi_shock_1h_tick/` | 5 | lawful Binance historical full manifest lineage |
| `yahoo_dialect_c_btcusd_crossfeed` | C | `core/chronology/historical/2026_recent_crossfeed_1h_yahoo_1m/` | 5 | lawful alternate-source provenance dialect |

Each fixture includes:

- `substrate_excerpt.jsonl` — exact byte excerpt from lawful source
- `manifest.json` — dialect sample from same lineage
- `fixture_meta.json` — excerpt `chronology_hash`, source pointers, dialect class
- `first_line.hex` — first-line byte guarantee

### Reference integrity

`fixtures/chronology_serialization/reference_hashes.json` records streaming
`chronology_hash` verification against full lawful source files (cross-check
that excerpt algorithm matches production manifest binding).

---

## Frozen Hash Law

```text
chronology_hash = SHA256( concat(line_bytes including trailing \n) )
```

Properties under fixture protection:

| Property | Frozen by |
|----------|-----------|
| JSON key order | excerpt bytes + `first_line.hex` |
| Float formatting | excerpt bytes |
| Newline termination | excerpt bytes |
| Line ordering | excerpt bytes |
| 5-field tick schema | verifier schema check |
| Dialect manifest shape | manifest.json per fixture |

---

## Timestamp Interpretation Fixtures

Fixtures encode dialect-specific bound semantics:

| Dialect | tick `timestamp` | manifest bounds |
|---------|-------------------|-----------------|
| A | milliseconds | milliseconds |
| B | milliseconds | **seconds** (historical defect; admissible) |
| C | milliseconds | milliseconds |

Future tools must classify dialect before window arithmetic. Fixtures do not
authorize retroactive conversion of Dialect B bounds.

---

## Producer Ratification Protocol (Future)

Before any producer is promoted to canonical emitter:

1. Run candidate producer against fixture input conditions (or replay excerpt sources).
2. Compare emitted JSONL bytes to fixture excerpts **or** prove intentional new-universe hash rotation under replay-scope declaration.
3. Run `python3 scripts/verify_chronology_byte_fixtures.py` — must remain PASS for frozen corpus.
4. Record outcome in `V006_PRODUCER_RATIFICATION.md` (not yet created).

**Pass criterion for bounded schema convergence tranche:**

```text
byte-identical tick line emission on fixture excerpts
```

**Fail → classify as replay-sensitive substrate migration.**

---

## Explicit Non-Authorization

This fixture pass does **not**:

- ratify `capture_daemon.rs`,
- create `capture_types.rs`,
- modify persisted chronology under `core/chronology/`,
- declare replay equivalence for producer changes,
- wire `binance_adapter` to NormalizedTick JSON,
- rewrite Dialect B manifests to Dialect A.

---

## Verification Performed

```bash
python3 scripts/verify_chronology_byte_fixtures.py
python3 scripts/verify_chronology_producer_ratification.py --producer capture_daemon
```

Expected: byte fixtures PASS; ratification report generated with `ratification_granted: false`.

---

## Next Gate (Not Started)

| Artifact | Purpose |
|----------|---------|
| `V006_PRODUCER_RATIFICATION.md` | adjudicate `capture_daemon.rs` vs target emission profile with fixture byte proof |
| Producer alignment work | only after ratification doc + PASS on excerpt byte comparison |

---

## Ledger Alignment

`AUTHORITY_MAP.md` V-006: **BYTE FIXTURES FROZEN** — migration still blocked; producer ratification is next decision surface.
