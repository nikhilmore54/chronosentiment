# V-006 — Serialization Law Declaration

**Status:** ACTIVE — constitutional law (no migration authorized)  
**Governance class:** replay substrate identity governance  
**Evidence basis:** `fixtures/chronology_serialization/ratification_report.json` (Phase B generated proof)  
**Prerequisites:** `V006_MANIFEST_DIALECT_POLICY.md`, `V006_CHRONOLOGY_BYTE_FIXTURES.md`, `V006_PRODUCER_RATIFICATION.md`  
**Constitutional reference:** `.cursor/rules/chronosentiment-core.mdc`, `AUTHORITY_MAP.md` V-006

---

## Purpose

Formally declare chronology serialization law after empirical proof that:

```text
semantic equivalence ≠ substrate equivalence
```

Phase B demonstrated identical tick field semantics can produce **different lawful substrate bytes** and therefore **different `chronology_hash` values**. This document establishes constitutional rules before any multi-producer convergence.

---

## Foundational Law

### L-1 — Substrate identity is byte-bound

`chronology_hash` binds **raw JSONL line bytes**, not parsed tick semantics.

| Claim | Status |
|-------|--------|
| Byte-identical JSONL → identical hash | **law** |
| Semantically identical JSONL → identical hash | **not guaranteed** |
| Semantic equivalence substitutes for substrate equivalence | **prohibited** for hash/certification purposes |

### L-2 — Parsed semantics are secondary evidence

Tick field values (`symbol`, `timestamp`, `price`, `volume`, `is_buyer_maker`) are interpretive. They may be used for analysis, catalog display, and cross-producer comparison — but **must not override byte-bound substrate identity**.

### L-3 — Historical bytes are lawful without canonical emission rights

Persisted chronology bytes are **lawful replay evidence** even when the emitting producer lineage is non-canonical, provisional, or cross-language divergent.

```text
historical visibility ≠ future emission authority
```

---

## Serialization Format Law (Forward-Looking)

### Target canonical tick emission format (Rust lineage)

For **new** chronology writes seeking unified producer ratification:

| Property | Law |
|----------|-----|
| Encoding | UTF-8 |
| Object serialization | compact JSON (no interior whitespace) |
| Key order | struct declaration order: `symbol`, `timestamp`, `price`, `volume`, `is_buyer_maker` |
| Line terminator | single `\n` (LF) per record |
| Float representation | shortest `serde_json` emission for parsed `f64` |
| Boolean representation | JSON literals `true` / `false` (unquoted) |

**Empirical basis:** Rust `serde_json::to_string` round-trip is **byte-identical** on Binance-class lawful fixtures (Dialect A and B tick layers, Phase B).

### Named emission lineage: `compact_rust_v1`

This label identifies the forward canonical byte format candidate. It is **not yet ratified** as producer authority — it is declared serialization law only.

---

## Cross-Language Serialization Law

### Python spaced JSON (`python_json_default_v1`)

Observed in Yahoo Dialect C fixtures (e.g. `yahoo_dialect_c_btcusd_crossfeed`):

```text
{"symbol": "BTC-USD", "timestamp": ..., "price": ..., ...}
```

vs Rust compact:

```text
{"symbol":"BTC-USD","timestamp":...,...}
```

| Property | Classification |
|----------|----------------|
| Historical admissibility | **lawful** — persisted Yahoo universes remain valid evidence |
| Forward canonical emission | **not lawful** for unified producer ratification |
| Cross-producer byte unification | **prohibited without explicit replay-scope declaration** |
| Hash comparability with Rust emission | **will differ** even when parsed tick values identical |

**Declared finding:** Python spacing drift is **serialization-law divergence**, not a parsing bug and not a semantic defect.

### Cross-language byte equivalence requirement

| Scope | Requirement |
|-------|-------------|
| Historical persisted universes | **not required** — each universe retains its own byte lineage |
| Future multi-producer consolidation | **required** — all ratified producers must emit under declared forward serialization law |
| Retroactive re-emission of historical universes | **replay-sensitive substrate migration** — hash rotation mandatory |

---

## Equivalence Class Law

| Class | Definition | Permitted use |
|-------|------------|---------------|
| **Byte-equivalent** | Identical JSONL line bytes | hash preservation, producer ratification, bounded convergence |
| **Semantically equivalent** | Parsed tick values match; bytes differ | historical analysis only — **not** hash/certification substitution |
| **Identity mutation** | tick set, ordering, or values change | new universe; new hash; explicit scope required |

**Constitutional rule:** Only **byte-equivalent** emission satisfies producer ratification on existing fixture corpora.

---

## Producer Ratification Serialization Requirements

Before any producer is promoted to canonical emitter:

1. **Byte fixture proof mandatory** — must pass `verify_chronology_producer_ratification.py` on applicable fixture corpus.
2. **No semantic-only proof** — parsed-value comparison alone is insufficient.
3. **Declared emission lineage required** — producer must declare which serialization law it emits (`compact_rust_v1`, etc.).
4. **Cross-language producers** must either:
   - adopt forward canonical byte law, or
   - remain historically admitted but **non-ratified** alternate lineage.

---

## Manifest Serialization (Separate Layer)

Manifest JSON is **evidentiary metadata** — not included in `chronology_hash`.

Manifest serialization law is governed separately by `V006_MANIFEST_DIALECT_POLICY.md`. Manifest pretty-print, field presence, and timestamp unit choices affect catalog identity and forward emission law — not tick substrate bytes.

**Important:** correcting manifest metadata (e.g. seconds → ms bounds) without JSONL re-emission is a **manifest metadata migration**, not a tick substrate migration — but may still require replay-scope declaration for catalog interpretation.

---

## Non-Claims

This declaration does **not**:

- ratify any producer,
- authorize `capture_types.rs`,
- require rewriting historical Yahoo spaced JSON,
- declare Rust as the only lawful language forever,
- authorize semantic-equivalence certification shortcuts.

---

## Ledger Alignment

Forward serialization law: **`compact_rust_v1` declared; not ratified.**  
Historical Python spaced JSON: **lawfully admitted; non-canonical for forward emission.**

Next artifact: `V006_PHASE_C_CLASSIFICATION.md` — formal drift classification and ratification posture.
