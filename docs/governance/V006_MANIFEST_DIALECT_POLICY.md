# V-006 — Manifest Dialect Policy

**Status:** ACTIVE — policy declaration (no migration authorized)  
**Governance class:** replay substrate identity governance  
**Prerequisite:** `docs/governance/V006_CAPTURE_SCHEMA_SCOPE.md` (observational pass complete)  
**Constitutional reference:** `.cursor/rules/chronosentiment-core.mdc`, `AUTHORITY_MAP.md` V-006, `docs/research/LIVE_CAPTURE_ISOLATION_CONTRACT_v1.md`

---

## Purpose

Declare manifest dialect law, chronology identity invariants, and timestamp doctrine **before** any:

- `capture_types.rs` extraction,
- producer consolidation,
- persisted chronology rewrite,
- or replay ingestion wiring.

V-006 is **replay substrate identity governance**, not shared-struct cleanup. The constitutional center is:

```text
chronology_hash binds raw JSONL line bytes
```

Any change that alters serialized tick bytes — including “cosmetic” normalization — is a **substrate identity mutation** unless explicitly governed below.

---

## Core Asymmetry (Accepted)

| Layer | Posture |
|-------|---------|
| Tick JSONL persistence | **effectively canonicalized** (5-field NormalizedTick JSON) |
| Manifest persistence | **dialect-fractured** (full / slim / provenance / legacy) |
| Replay ingestion (`binance_adapter`, `cs-ingest`) | **partially disconnected** from persisted chronology capture |
| Runtime Rust structs | **deceptively aligned** (field-identical; artifact semantics are not) |

```text
shared Rust fields ≠ shared replay meaning
```

---

## Authority Hierarchy

When code and persisted lineage disagree, **persisted lineage is lawful evidence**; code is provisional until ratified.

| Conflict | Lawful authority | Non-authoritative until ratified |
|----------|------------------|----------------------------------|
| `capture_daemon.rs` 10-field manifest vs on-disk slim live manifests | **on-disk slim 6-field live manifests** (12 universes) | current `capture_daemon.rs` emission shape |
| `historical_importer` 10-field manifest | **on-disk 10-field historical manifests** (38 universes) | — |
| Yahoo slim + `provenance` | **on-disk yahoo dialect** (6 universes) | — |
| `LIVE_CAPTURE_ISOLATION_CONTRACT` ms timestamps vs live manifest bounds in seconds | **contract declares ms law**; seconds bounds are **historically admitted defects** | uncorrected producer behavior |

**Doctrine:** history constrains canonicalization; canonicalization does not retroactively redefine history.

---

## Substrate vs Manifest Roles

### Tick JSONL — replay substrate

- **Role:** primary chronology substrate; sole input to `chronology_hash`.
- **Identity binding:** **byte-level** (SHA-256 over each serialized line including trailing `\n`, in emission order).
- **Governance class:** **CRITICAL / replay-sensitive**.

### Manifest — evidentiary identity record

- **Role:** metadata + integrity pointer (`chronology_hash`) + catalog identity; **not** replay execution substrate.
- **Identity binding:** manifest fields are **evidentiary** except `chronology_hash`, which must match substrate bytes.
- **Governance class:** **CRITICAL for catalog/certification coupling**; **MEDIUM direct replay coupling** (manifest body not hashed into replay engine state).

Manifests answer: *what was captured, how, when, and what hash proves it.* They do not substitute for tick bytes in replay.

---

## Lawful Persisted Dialects (Historical Admissibility)

The following manifest dialects are **historically admissible** as persisted evidence. They remain valid without rewrite.

### Dialect A — Full historical (`historical_importer`)

**Fields (required set):**

```text
source, resolution, capture_method, import_timestamp,
substrate, capture_start, capture_end, total_ticks,
chronology_hash, gaps
```

| Property | Value |
|----------|-------|
| Prevalence | 38 manifests under `core/chronology/historical/` |
| Timestamp unit (`capture_start` / `capture_end`) | **milliseconds** |
| Status | **lawful historical authority** for Binance historical universes |

### Dialect B — Slim live rotation

**Fields (required set):**

```text
substrate, capture_start, capture_end, total_ticks,
chronology_hash, gaps
```

| Property | Value |
|----------|-------|
| Prevalence | 12 manifests under `core/chronology/live_capture/` |
| Timestamp unit (`capture_start` / `capture_end`) | **seconds** (contract defect — see Timestamp Doctrine) |
| Status | **lawful historical evidence**; **non-canonical** for future emission |

### Dialect C — Slim + provenance (Yahoo lineage)

**Fields (required set):**

```text
substrate, capture_start, capture_end, total_ticks,
chronology_hash, gaps, provenance
```

| Property | Value |
|----------|-------|
| Prevalence | 6 manifests (Rust + Python Yahoo producers) |
| Timestamp unit | **milliseconds** |
| Status | **lawfully admissible alternate-source dialect**; lossy vs Dialect A metadata |

### Dialect D — Legacy kline script (deprecated)

**Fields:** `interval`, `total_klines`, `start_timestamp`, `end_timestamp`, `capture_time`, `substrate`, `chronology_hash`

| Property | Value |
|----------|-------|
| Prevalence | 1 manifest (`capture_live_chronology.py`) |
| Tick shape | OHLC 7-field (non-NormalizedTick) |
| Status | **historically admitted; deprecated** — no new universes |

---

## Target Canonical Emission Law (Forward-Looking)

**Not retroactive.** Applies only to **new** capture writes after explicit producer ratification.

| Artifact | Target law |
|----------|------------|
| Tick JSONL | 5-field NormalizedTick JSON per observational authority |
| Manifest (Binance historical/live) | **Dialect A (full 10-field)** |
| Manifest (non-Binance / Yahoo / crossfeed) | **Dialect C** permitted when `source`/`capture_method` semantics do not apply; must not claim Dialect A fields falsely |
| Timestamp fields | **milliseconds only** (`capture_start`, `capture_end`, tick `timestamp`) |
| `chronology_hash` | streaming SHA-256 over tick JSONL line bytes as emitted |

**Blocked until producer ratification:** changing `capture_daemon.rs` or rewriting historical manifests to match target law.

---

## Chronology Hash Invariants

These invariants are **non-negotiable** for any V-006 migration tranche:

1. **`chronology_hash` verifies tick JSONL bytes**, not manifest metadata.
2. Hash input includes **exact line serialization** (serde/json field order as emitted, trailing `\n`).
3. **Byte-level equivalence required** for hash preservation. Semantic equivalence alone is insufficient.
4. Re-serialization that changes bytes (key order, float formatting, whitespace) **mutates substrate identity** even if parsed tick values are identical.
5. Manifest `total_ticks` must equal JSONL line count; mismatch is integrity failure, not dialect variance.
6. `gaps` arrays are **not** included in `chronology_hash`; gap semantics may evolve without hash impact.

### Equivalence classes (must be declared per migration tranche)

| Class | Definition | When acceptable |
|-------|------------|-----------------|
| **Byte-equivalent** | Identical JSONL file bytes | Default requirement for hash-preserving changes |
| **Semantically equivalent** | Parsed tick values identical; bytes differ | **Only** with explicit replay-scope declaration + hash rotation + catalog update |
| **Identity mutation** | Tick set or ordering changes | New universe; new hash; no silent substitution |

---

## Timestamp Unit Doctrine

**Declared law** (aligns with `LIVE_CAPTURE_ISOLATION_CONTRACT_v1.md` §5):

| Field | Unit | Scope |
|-------|------|-------|
| Tick `timestamp` | Unix epoch **milliseconds** | all tick JSONL |
| Manifest `capture_start` / `capture_end` | Unix epoch **milliseconds** | target canonical emission |
| Manifest `import_timestamp` | Unix epoch **seconds** | Dialect A only; import audit metadata |

### Historical defects (admitted, not erased)

- **Dialect B live manifests:** `capture_start` / `capture_end` stored in **seconds** while ticks use ms.
- **Interpretation rule:** bounds are **evidentiary rotation metadata**, not tick timestamps. Tools must not assume ms without dialect classification.
- **Correction policy:** fixing Dialect B bounds to ms is a **manifest metadata migration** (does not alter `chronology_hash` if JSONL unchanged) but **does alter catalog window interpretation** — requires scoped declaration before batch rewrite.

---

## Normalization Allowances

### Permitted without hash impact (manifest-only)

- Adding optional catalog fields not present at capture time (if append-only catalog index)
- Correcting manifest typos in non-hash fields **only** when accompanied by integrity re-verification against JSONL

### Prohibited without replay-scope declaration

- Re-serializing tick JSONL for “pretty print” or field reordering
- Float formatting normalization (`49860.0` vs `49860.00000`)
- Merging/splitting tick lines
- Replacing `is_buyer_maker` mocked values retroactively
- Unifying manifest dialects by deleting historical fields

### Explicitly out of V-006 manifest policy scope (separate integration track)

- Wiring `binance_adapter::load_binance_events_from_jsonl` to consume NormalizedTick JSON
- `cs-ingest` frozen cohort bar schema (`state_archive/` lineage)

**Note:** persisted chronology is **not yet** the operational replay substrate in the unified engine sense. This **lowers immediate replay risk** for manifest-only work but **raises future integration risk** if assumed replay-native without adapter declaration.

---

## Replay Substrate Scope

| Substrate lineage | V-006 coupling | Replay role today |
|-------------------|----------------|-------------------|
| `core/chronology/**/*.jsonl` (5-field ticks) | **direct** — hash-bound | cataloged chronology evidence; not engine-native via `binance_adapter` |
| `core/chronology/**/*_manifest.json` | **identity pointer** | catalog + integrity verification |
| `state_archive/candles/batch_*` | **separate** | `cs-ingest` replay equivalence (e.g. batch `003`) |
| Certification manifests (`emit_manifest_v1.py`) | **downstream** | binds to presented substrate file hash |

V-006 consolidation tranches must declare which lineage they touch. Touching hash-bound JSONL is **replay-sensitive substrate migration**. Touching Rust struct locations alone is **bounded schema convergence** only if **zero byte emission change** is proven.

---

## Migration Classification Gate

Before any consolidation work, classify the tranche:

| Tranche type | Criteria | Governance intensity |
|--------------|----------|-------------------|
| **Bounded schema convergence** | Rust struct extraction; **proven** zero change to emitted JSONL bytes and manifest bytes on golden fixtures | consumer inventory + `cargo check` + fixture byte compare |
| **Replay-sensitive substrate migration** | any tick JSONL re-emission, hash rotation, manifest dialect rewrite, timestamp correction at scale | replay-scope declaration + cohort/hash verification + `AUTHORITY_MAP` update; ratification if certification surfaces touched |

**Current posture:** neither tranche is authorized. Policy declaration only.

---

## Producer Ratification Requirements (Future)

Before promoting any producer or `capture_types.rs` to `CRITICAL / STABLE`:

1. Declare dialect emitted (A, B, C, or target canonical).
2. Prove byte-stable emission against frozen fixture corpora (to be created — not yet present).
3. Resolve `capture_daemon` code vs Dialect B artifact mismatch via explicit choice: ratify code toward A, or document B as intentional slim emission.
4. If emission changes persisted bytes → classify as replay-sensitive substrate migration.

---

## Non-Claims

This policy does **not**:

- authorize `core/src/capture_types.rs` creation,
- rewrite persisted chronology,
- unify manifest dialects retroactively,
- ratify `binance_adapter` as NormalizedTick consumer,
- declare replay equivalence for any capture consolidation,
- erase Dialect B second-based manifest bounds.

---

## Next Artifacts (Not Started)

| Artifact | Purpose |
|----------|---------|
| `V006_CHRONOLOGY_SERIALIZATION_FIXTURES/` | frozen byte-level golden JSONL + manifest pairs for emission proof |
| `V006_PRODUCER_RATIFICATION.md` | per-producer dialect + byte-stability adjudication |
| `V006_LIVE_CAPTURE_AUTHORITY_DECISION.md` | resolve code vs Dialect B on-disk mismatch |
| Replay-scope declaration | only if a tranche touches hash-bound JSONL |

---

## Ledger Alignment

`AUTHORITY_MAP.md` V-006 status: **POLICY DECLARED — consolidation blocked**.

Cadence preserved:

```text
inspect → classify → govern → migrate → ratify only if necessary
```

Observational pass: `V006_CAPTURE_SCHEMA_SCOPE.md`  
Policy pass: this document  
Migration pass: **not authorized**
