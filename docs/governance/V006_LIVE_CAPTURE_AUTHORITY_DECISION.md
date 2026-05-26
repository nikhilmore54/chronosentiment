# V-006 — Live Capture Authority Decision

**Status:** DECIDED — policy binding; no producer migration authorized  
**Governance class:** replay substrate identity governance (manifest layer)  
**Prerequisites:** `docs/governance/V006_CAPTURE_SCHEMA_SCOPE.md`, `docs/governance/V006_MANIFEST_DIALECT_POLICY.md`  
**Constitutional reference:** `.cursor/rules/chronosentiment-core.mdc`, `AUTHORITY_MAP.md` V-006

---

## Question

```text
What is future live capture emission legally supposed to be?
```

This is not “which Rust struct wins.” It determines future chronology lineage, hash stability, manifest doctrine, and normalization policy for the live capture lane.

---

## Evidence Summary

### On-disk lawful lineage (Dialect B)

Location: `core/chronology/live_capture/`  
Inventory: **12 rotation pairs** (`btcusdt_*.{jsonl,_manifest.json}`)

| Property | Observed value |
|----------|----------------|
| Manifest dialect | **Dialect B (slim 6-field)** |
| Tick JSON | 5-field NormalizedTick (`symbol`, `timestamp`, `price`, `volume`, `is_buyer_maker`) |
| Tick timestamps | **milliseconds** (e.g. `1779545160980`) |
| Manifest bounds | **seconds** (e.g. `capture_start: 1779545161`) — contract defect per timestamp doctrine |
| `chronology_hash` | **verified** — SHA-256 over JSONL line bytes matches manifest (sampled pairs pass) |
| `capture_method` | **absent** — not attributable to current `capture_daemon.rs` emission |

### Current code (`core/src/bin/capture_daemon.rs`)

| Property | Current code behavior |
|----------|----------------------|
| Output path | `chronology/live_capture/` (**not** `core/chronology/live_capture/`) |
| Manifest dialect | **Dialect A shape** (10-field: `source`, `resolution`, `capture_method`, …) |
| Manifest bounds | **seconds** (`current_rotation_start`, `now` from `SystemTime::as_secs()`) |
| Tick JSON | 5-field NormalizedTick; ms tick timestamps from Binance `E` |
| Producer ratification | **none** — code has not been proven to emit byte-identical artifacts |

### Authority fracture (confirmed)

```text
runtime producer authority (capture_daemon.rs)
≠
persisted chronology authority (core/chronology/live_capture/*)
```

The on-disk archive is **hash-valid lawful evidence**. The current bin source is **provisional** and **not demonstrated** as the producer of the persisted lineage.

---

## Decisions

### D-1 — Persisted live capture lineage is lawful historical evidence

**Decision:** All 12 Dialect B universes under `core/chronology/live_capture/` remain **lawful, admissible, non-rewritable** chronology evidence.

**Rationale:** `chronology_hash` binds JSONL bytes; manifests verify integrity. Historical visibility must not be collapsed into future canonical preference.

**Non-claim:** This does not certify that `capture_daemon.rs` as currently written produced these artifacts.

---

### D-2 — Dialect B is historically admitted; not future canonical live law

**Decision:** Dialect B (slim 6-field live manifests) is **historically admitted only**. Future live capture emission **must not** target Dialect B as canonical output.

**Future canonical live manifest law:** **Dialect A (full 10-field)**, aligned with `historical_importer` and `V006_MANIFEST_DIALECT_POLICY.md` target emission law.

**Rationale:** Preserves V-001 asymmetry pattern:

```text
historical visibility ≠ future canonical authority
```

Dialect B’s missing provenance fields (`source`, `resolution`, `capture_method`, `import_timestamp`) are unacceptable for forward canonical emission where Binance live semantics apply.

---

### D-3 — Tick JSONL emission law unchanged for live capture

**Decision:** Future live capture tick substrate remains **5-field NormalizedTick JSONL** with **millisecond** tick timestamps.

**Rationale:** Tick layer is already effectively canonicalized; `chronology_hash` binds these bytes. Changing tick serialization requires replay-sensitive substrate migration classification.

**Constraint:** Any producer ratification must prove **byte-stable** tick emission against frozen fixtures before code promotion.

---

### D-4 — Timestamp correction obligation for future live emission

**Decision:** Future live manifest `capture_start` / `capture_end` **must use milliseconds**, correcting the seconds defect present in Dialect B historical bounds.

**Historical interpretation (Dialect B):** bounds remain **seconds-based rotation metadata**; tools must classify dialect before window arithmetic.

**Rationale:** Aligns live forward emission with `LIVE_CAPTURE_ISOLATION_CONTRACT_v1.md` §5 and `V006_MANIFEST_DIALECT_POLICY.md` timestamp doctrine.

**Scope note:** Correcting Dialect B bounds on disk is a **manifest metadata migration** (JSONL unchanged → hash unchanged) but alters catalog window semantics — **not authorized** in this decision; requires separate scoped tranche if ever attempted.

---

### D-5 — `capture_daemon.rs` is provisional; not canonical live producer

**Decision:** `core/src/bin/capture_daemon.rs` in its current form is **not ratified** as canonical live capture authority.

**Blocking defects (must be resolved before ratification):**

| Defect | Required resolution |
|--------|---------------------|
| Output path drift (`chronology/` vs `core/chronology/`) | declare canonical live output root |
| Manifest dialect mismatch vs lawful on-disk lineage | emit Dialect A per D-2 |
| Seconds manifest bounds | emit ms bounds per D-4 |
| No byte-emission proof | golden fixture corpus + hash verification |
| Unknown equivalence to historical Dialect B producer | document lineage or accept new producer identity |

**Rationale:** Code cannot overwrite historical replay truth. Producer promotion requires evidence, not struct alignment alone.

---

### D-6 — Canonical live output root

**Decision:** Canonical persisted live capture root is:

```text
core/chronology/live_capture/
```

All future ratified live producers must write under this root (not `chronology/live_capture/`).

**Rationale:** Matches persisted lawful archive location and catalog indexing under `core/chronology/`.

---

### D-7 — Manifest role reaffirmed for live lane

**Decision:** Live manifests remain **evidentiary metadata + hash pointer**, not replay execution substrate.

**Rationale:** Prevents manifest fields from becoming implicit replay authority; tick JSONL retains substrate identity binding.

---

## Future Live Capture Emission Profile (Target Law)

When a producer is ratified, forward live capture **must** emit:

| Layer | Target |
|-------|--------|
| Tick JSONL | 5-field NormalizedTick; ms timestamps; streaming SHA-256 hash over line bytes |
| Manifest | Dialect A full 10-field |
| `capture_method` | `"capture_daemon"` (or successor name explicitly declared at ratification) |
| `source` | `"Binance Live Stream"` |
| `resolution` | `"aggTrade"` |
| `capture_start` / `capture_end` | **milliseconds** |
| Output directory | `core/chronology/live_capture/` |

**Not authorized by this decision:** code changes, struct extraction, or live capture reruns.

---

## Classification of Pending Work

| Work item | Tranche class | Authorized now? |
|-----------|---------------|-----------------|
| Document-only decisions (this artifact) | governance | **yes** |
| Create byte golden fixtures from Dialect B samples | governance prep | next gate |
| Align `capture_daemon.rs` to target profile without tick byte change | bounded convergence candidate | **no** — requires fixtures + ratification doc |
| Re-emit or rewrite Dialect B manifests to Dialect A | replay-sensitive substrate migration | **no** |
| Rewrite Dialect B seconds → ms bounds on disk | manifest metadata migration | **no** |
| `capture_types.rs` extraction | blocked | **no** |

---

## Relationship to Other V-006 Artifacts

| Artifact | Relationship |
|----------|--------------|
| `V006_MANIFEST_DIALECT_POLICY.md` | this decision operationalizes live lane of target canonical emission law |
| `V006_CHRONOLOGY_SERIALIZATION_FIXTURES/` | **next required gate** before producer ratification |
| `V006_PRODUCER_RATIFICATION.md` | must adjudicate `capture_daemon.rs` against this target profile |
| `binance_adapter` NormalizedTick wiring | **out of scope** — separate integration track |

---

## Non-Claims

This decision does **not**:

- identify the historical tool/version that produced Dialect B artifacts,
- modify `capture_daemon.rs`,
- rewrite `core/chronology/live_capture/*`,
- rotate any `chronology_hash`,
- authorize `capture_types.rs`,
- declare replay equivalence for any future live producer change.

---

## Ledger Update

`AUTHORITY_MAP.md` V-006: **LIVE CAPTURE AUTHORITY DECIDED** — forward law = Dialect A + ms bounds + `core/chronology/live_capture/`; Dialect B preserved as historical evidence; `capture_daemon.rs` provisional.

Cadence:

```text
observe → classify → declare law → freeze semantics → migrate only under declared scope
```

Live capture is now at **freeze semantics** for the manifest layer. Next gate: byte fixtures, then producer ratification.
