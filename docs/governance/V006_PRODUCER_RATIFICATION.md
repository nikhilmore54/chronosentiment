# V-006 — Producer Ratification

**Status:** PHASE A COMPLETE — read-only inspection; ratification **NOT GRANTED**  
**Governance class:** certification audit (not refactor scope)  
**Prerequisites:** `V006_MANIFEST_DIALECT_POLICY.md`, `V006_LIVE_CAPTURE_AUTHORITY_DECISION.md`, `V006_CHRONOLOGY_BYTE_FIXTURES.md`  
**Constitutional reference:** `.cursor/rules/chronosentiment-core.mdc`, `AUTHORITY_MAP.md` V-006

---

## Purpose

Adjudicate whether runtime producers may inherit **future chronology authority**.

```text
producer authority must be earned through byte conformance
not inferred from code ownership
```

The repository already holds:

```text
lawful bytes
lawful hashes
lawful historical dialects
lawful chronology interpretation
```

Producers must prove they emit **lawful future chronology** without mutating **lawful historical substrate semantics**.

**Non-goal:** `capture_types.rs` extraction, producer code changes, or migration in this document.

---

## Ratification Target (Primary)

| Producer | Path | Target profile |
|----------|------|----------------|
| `capture_daemon` | `core/src/bin/capture_daemon.rs` | Dialect A live emission per `V006_LIVE_CAPTURE_AUTHORITY_DECISION.md` |

### Reference producers (inspection only — not ratification subjects for live lane)

| Producer | Path | Lawful role |
|----------|------|-------------|
| `historical_importer` | `core/src/bin/historical_importer.rs` | reference for Dialect A tick/manifest emission patterns |
| `yahoo_importer` | `core/src/bin/yahoo_importer.rs` | reference for Dialect C |
| `yahoo_fetcher.py` | `scripts/yahoo_fetcher.py` | reference for Python Dialect C bytes |

---

## Phase A — Read-Only Producer Inspection (Complete)

No writes performed. Inventory of implicit serializer behavior and emission paths.

### A.1 JSON serialization path (all Rust producers)

| Property | Observed behavior | Replay risk |
|----------|-------------------|-------------|
| Serializer | `serde_json::to_string(&tick)` | **HIGH** — library controls key order, float formatting, bool encoding |
| Line assembly | `format!("{}\n", serde_json::to_string(...))` | **HIGH** — `\n` required; matches fixture hash law |
| Struct field order | declaration order: `symbol`, `timestamp`, `price`, `volume`, `is_buyer_maker` | **HIGH** — matches persisted lawful bytes in fixtures |
| Map types | none in tick struct | none |
| UTF-8 | Rust `String` / `write_all(line.as_bytes())` | **MEDIUM** — no explicit NFC normalization |

**Cross-check:** fixture excerpt line bytes match `serde_json` compact shape (Python `json.dumps(..., separators=(',', ':'))` + `\n` equivalent).

### A.2 Float formatting

| Producer | Price path | Volume path | Risk |
|----------|------------|-------------|------|
| `capture_daemon` | `String::parse()` → `f64` | same | **HIGH** — parse→serialize round-trip may differ from source string |
| `historical_importer` | API string → `f64` | same | **HIGH** — same round-trip class |
| `yahoo_importer` | `quote.close: f64` | `quote.volume as f64` | **HIGH** — Yahoo API float semantics |

Observed lawful bytes use shortest float representation (e.g. `49860.0`, `75108.16`). **No guarantee** future `serde_json` or input changes preserve bytes without fixture proof.

### A.3 Timestamp normalization

| Producer | Tick `timestamp` | Manifest `capture_start` / `capture_end` | Law alignment |
|----------|------------------|-------------------------------------------|---------------|
| `capture_daemon` | ms (`raw_trade.event_time`) | **seconds** (`SystemTime::as_secs()`) | tick ✓; manifest bounds **violate forward ms law** |
| `historical_importer` | ms | ms (`args.start_time` / `args.end_time`) | ✓ aligned with Dialect A |
| `yahoo_importer` | ms (`quote.timestamp * 1000`) | ms | ✓ |
| `yahoo_fetcher.py` | ms | ms | ✓ |

Gap records in `capture_daemon` also use **seconds** (`last_tick_time`, `now`).

### A.4 Manifest emission path

| Producer | Serializer | Pretty print | Dialect emitted |
|----------|------------|--------------|-----------------|
| `capture_daemon` | `serde_json::to_string_pretty(&manifest)` | yes | A shape (10-field) |
| `historical_importer` | `to_string_pretty` | yes | A |
| `yahoo_importer` | `to_string_pretty` | yes | C |

Manifest pretty-print affects **manifest file bytes only** — not `chronology_hash` (hash excludes manifest body).

### A.5 Directory layout

| Producer | Code output root | Lawful persisted root | Match |
|----------|------------------|----------------------|-------|
| `capture_daemon` | `chronology/live_capture/` | `core/chronology/live_capture/` | **NO** |
| `historical_importer` | `chronology/historical/{name}/` | `core/chronology/historical/{name}/` | **NO** |
| `yahoo_importer` | `chronology/historical/{name}/` | `core/chronology/historical/{name}/` | **NO** |
| `yahoo_fetcher.py` | `chronology/historical/{name}/` | `core/chronology/historical/{name}/` | **NO** |

Persisted lawful archives live under `core/chronology/`. All inspected producers emit to `chronology/` (missing `core/` prefix).

### A.6 Ordering guarantees

| Producer | Tick ordering | Notes |
|----------|---------------|-------|
| `capture_daemon` | network arrival order | websocket stream order |
| `historical_importer` | API pagination order | Binance response order; cursor advance |
| `yahoo_importer` | quote iterator order | Yahoo API order |
| `yahoo_fetcher.py` | dataframe row order | yfinance index order |

Ordering is **input-dependent** — fixture replay requires frozen input injection, not live network.

### A.7 Buffering / flush behavior

| Producer | Write pattern | Flush |
|----------|---------------|-------|
| All Rust | `file.write_all(line.as_bytes())` per tick | **no explicit `flush()`** |
| Python | `f.write(line)` per tick | stdio buffering default |

Partial-rotation crash could truncate without flush — **operational risk**, not yet fixture-tested.

### A.8 Hash pipeline

All Rust producers:

```text
line = format!("{}\n", serde_json::to_string(&tick))
hasher.update(line.as_bytes())
```

Matches frozen fixture hash law. Python Yahoo uses `hasher.update(line.encode())` — **encoding assumption UTF-8** (matches Rust for ASCII JSON).

---

## Phase A — Defect Summary (`capture_daemon`)

| ID | Defect | Severity | Blocks ratification |
|----|--------|----------|---------------------|
| CD-1 | Output path `chronology/` vs lawful `core/chronology/` | high | yes |
| CD-2 | Manifest bounds in seconds (forward law requires ms) | high | yes |
| CD-3 | Gap timestamps in seconds | medium | yes (metadata law) |
| CD-4 | No byte fixture proof against lawful excerpts | high | yes |
| CD-5 | Unknown producer of on-disk Dialect B lineage | medium | yes (lineage attribution) |
| CD-6 | Float parse→serialize round-trip unproven on fixtures | high | yes |
| CD-7 | Live network input — non-replayable without injection harness | medium | Phase B blocker |

**Phase A verdict:** `capture_daemon.rs` remains **PROVISIONAL — NOT RATIFIED**.

---

## Phase B — Fixture Replay Against Emitter (Complete)

Proof is **generated, not visually asserted**.

### Harness

| Component | Path | Role |
|-----------|------|------|
| Injection probe | `core/src/bin/chronology_serialize_probe.rs` | fixture JSONL → `NormalizedTick` → serde serialize → hash |
| Ratification verifier | `scripts/verify_chronology_producer_ratification.py` | byte diff + hash diff + constitutional classification |
| Generated report | `fixtures/chronology_serialization/ratification_report.json` | machine-readable Phase B evidence |

Pipeline:

```text
fixture → inject → emit → byte diff → hash diff → classify
```

The probe **bypasses** websocket timing, reconnect logic, async variance, exchange latency, buffering, and runtime ordering races. It exercises only:

```text
NormalizedTick → serialization path → persisted bytes → chronology_hash
```

### Classification taxonomy

| Classification | Meaning |
|----------------|---------|
| `byte_identical` | lawful producer tick emission on fixture excerpt |
| `hash_identical_semantic_drift` | hash mismatch despite line claims (scope bug) |
| `serialization_drift` | replay-sensitive producer failure (byte mismatch) |
| `dialect_drift` | manifest lineage breach |
| `timestamp_unit_drift` | chronology interpretation breach |
| `path_authority_drift` | unlawful output lineage |
| `ratification_blocked` | harness/proof incomplete |

### Phase B results (`capture_daemon`, 2026-05-26)

Command:

```bash
python3 scripts/verify_chronology_producer_ratification.py --producer capture_daemon
```

| Fixture | Tick layer | Manifest layer | Static layer | Overall |
|---------|------------|----------------|--------------|---------|
| `historical_dialect_a_cpi_shock_tick` | `byte_identical` | `byte_identical` | `path_authority_drift` | `path_authority_drift` |
| `live_dialect_b_btcusdt_1779545161` | `byte_identical` | `timestamp_unit_drift` | `timestamp_unit_drift` | `timestamp_unit_drift` |
| `yahoo_dialect_c_btcusd_crossfeed` | `serialization_drift` | `byte_identical` | `path_authority_drift` | `serialization_drift` |

**Key generated findings:**

1. **Rust serde round-trip preserves lawful Binance tick bytes** on Dialect A and B fixtures (`byte_identical` tick layer).
2. **`capture_daemon` static defects confirmed:** `path_authority_drift` + `timestamp_unit_drift` for live forward law.
3. **Python Yahoo vs Rust serde divergence:** Yahoo fixture bytes use spaced JSON (`{"symbol": "BTC-USD", ...}`); Rust emits compact JSON — **`serialization_drift`** across producer lineages even with identical field semantics.
4. **Ratification granted: false** (by design).

### Phase B verdict

Serialization determinism is **partially proven** for Rust tick emission on Binance-class fixtures. Producer ratification remains **NOT GRANTED** due to path/timestamp/manifest law defects and cross-producer serialization divergence.

---

## Phase C — Ratification Classification (Pending Phase B)

Classification determined **only by emitted bytes**, not struct alignment.

| Outcome | Condition |
|---------|-----------|
| **Bounded schema convergence** | byte-identical tick emission on fixtures; manifest conforms to forward law; no hash rotation on historical corpus |
| **Replay-sensitive substrate migration** | any tick byte change, hash rotation, or dialect rewrite on lawful historical universes |

Current expectation for naïve `capture_daemon` alignment work: likely **replay-sensitive** for manifest bounds (seconds→ms) even if tick bytes unchanged.

---

## Ratification Decision

| Producer | Decision | Phase B evidence |
|----------|----------|------------------|
| `capture_daemon.rs` | **NOT RATIFIED** | tick round-trip `byte_identical` on Binance fixtures; blocked by `path_authority_drift`, `timestamp_unit_drift`, manifest law defects |
| `historical_importer.rs` | **NOT RATIFIED** | not individually probed; shares Rust serde path; `path_authority_drift` static defect |
| `yahoo_importer.rs` | **NOT RATIFIED** | Python Yahoo fixtures show `serialization_drift` vs Rust compact JSON |
| `yahoo_fetcher.py` | **NOT RATIFIED** | emits spaced JSON; cross-producer byte divergence proven |

No producer inherits canonical chronology authority at this time.

---

## Explicit Non-Authorization

This Phase A audit does **not**:

- modify any producer,
- create `capture_types.rs`,
- run live capture,
- rewrite persisted chronology,
- declare replay equivalence,
- implement Phase B harness.

---

## Next Gate

1. **Phase C classification** for any producer alignment work (bounded convergence vs replay-sensitive substrate migration).
2. Manifest emission probe (Dialect A + ms bounds) — separate from tick serialization probe.
3. Cross-producer serialization law declaration (Rust compact vs Python spaced JSON) before multi-producer consolidation.
4. Issue ratification only if **all** proof dimensions pass under forward law.

**Still blocked:** `capture_types.rs`, producer code changes without Phase C scope declaration.

---

## Ledger Alignment

`AUTHORITY_MAP.md` V-006: **PHASE C CLASSIFICATION COMPLETE** — see `V006_SERIALIZATION_LAW_DECLARATION.md` and `V006_PHASE_C_CLASSIFICATION.md`. All producers **NOT RATIFIED**. Migration blocked pending scoped authorization per tranche.

Cadence:

```text
observe → classify → declare law → freeze bytes → prove → classify → [migrate only under declared scope]
```

Current position: **classify ✓** — **migrate ✗** — **ratify ✗**
