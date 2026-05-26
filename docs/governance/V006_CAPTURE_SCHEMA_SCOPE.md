# V-006 — Capture Schema Authority Scope

**Status:** ACTIVE — observational phase (inspection only; no consolidation)  
**Governance class:** schema-governance / replay-sensitive  
**Constitutional reference:** `.cursor/rules/chronosentiment-core.mdc`, `AUTHORITY_MAP.md` V-006, `docs/research/LIVE_CAPTURE_ISOLATION_CONTRACT_v1.md`

---

## Intent

Map authority, persistence exposure, and replay coupling for the capture schema family:

```text
NormalizedTick
CaptureGap
CaptureManifest
```

before any shared `capture_types.rs` extraction is attempted.

**Non-goal for this phase:** create `core/src/capture_types.rs` or migrate producers/consumers.

---

## Domain Posture

```text
V-006 = CRITICAL domain / observational phase
```

These types sit at the boundary between capture ingestion, persisted chronology substrate, manifest identity, and certification comparability. Consolidation intensity cannot be declared until replay coupling is fully mapped.

Frozen doctrine (`LIVE_CAPTURE_ISOLATION_CONTRACT_v1.md` §5) declares:

- symbol normalization (lowercase pair-continuous)
- timestamp semantics: **Unix epoch milliseconds**
- precision: unbounded floats for price/volume
- serialization: strict JSONL, append-only by network arrival
- `capture_hash` / `chronology_hash` secures raw chronology integrity

---

## Critical Questions — Answers (Observational Pass)

| Question | Finding |
|----------|---------|
| Which `NormalizedTick` shape is canonical in persisted artifacts? | **5-field JSON object** is the de facto persisted tick authority: `symbol`, `timestamp`, `price`, `volume`, `is_buyer_maker`. Present across 38+ historical universes and live capture JSONL. Outliers exist (see below). |
| Which variants are serialized into manifests or barrier files? | **Three manifest lineages** persisted under `core/chronology/` (see inventory). `cs-ingest` frozen replay uses a separate `state_archive/.../manifest.json` + gzip bar layout — not `CaptureManifest`. Certification tier manifests (`emit_manifest_v1.py`) use `replay_identity.chronology_hash` — downstream of substrate, not producer structs. |
| Are field differences semantic or transport-local? | **Mixed.** `NormalizedTick` Rust/Python Yahoo producers are aligned at the JSON layer. `CaptureManifest` drift is **semantic at manifest identity layer** (missing `capture_method`, `source`, `resolution`, `import_timestamp`; alternate field names; timestamp unit inconsistency in live rotation bounds). |
| Does replay tooling deserialize multiple variants already? | **Yes, loosely.** `scripts/rebuild_catalog.py` reads optional fields (`capture_start`/`start_timestamp`, `total_ticks`/`total_klines`). No Rust consumer deserializes `CaptureManifest` as a typed struct. `binance_adapter::load_binance_events_from_jsonl` expects **Binance raw websocket keys** (`s`,`E`,`p`,`q`,`m`) — not `NormalizedTick` — so tick JSON and engine loader are **decoupled**. |
| Are hashes/fingerprints derived from these structs? | **`chronology_hash` is derived from JSONL line bytes** (SHA-256 over each serialized tick line including `\n`) in all Rust importers and Python Yahoo fetchers. Manifest hash binds catalog identity. `cs-ingest` timeline fingerprint uses frozen cohort bar timestamps — different substrate lineage. |
| Is `yahoo_importer` drift additive, lossy, or reinterpretive? | **Lossy at manifest layer, aligned at tick layer.** Drops canonical metadata fields; adds `provenance`. Tick JSON matches 5-field `NormalizedTick`. Python Yahoo fetchers (`scripts/yahoo_fetcher.py`) follow the same slim manifest lineage as `yahoo_importer.rs`. |

---

## Struct Lineage Comparison (Rust Bins)

### `NormalizedTick`

| Field | capture_daemon | historical_importer | yahoo_importer |
|-------|----------------|---------------------|----------------|
| `symbol` | yes | yes | yes |
| `timestamp` | yes (ms, Binance `E`) | yes (ms) | yes (ms, Yahoo sec × 1000) |
| `price` | yes | yes | yes (close) |
| `volume` | yes | yes | yes |
| `is_buyer_maker` | yes (live) | yes (tick truth; `false` for kline) | yes (always `false`) |

**Rust struct bodies are field-identical.** Semantic variance is in producer mapping, not struct definition.

### `CaptureGap`

Identical across all three bins: `gap_start`, `gap_end`, `reason`.

### `CaptureManifest`

| Field | capture_daemon | historical_importer | yahoo_importer |
|-------|----------------|---------------------|----------------|
| `source` | yes | yes | **absent** |
| `resolution` | yes | yes | **absent** |
| `capture_method` | yes | yes | **absent** |
| `import_timestamp` | yes | yes | **absent** |
| `substrate` | yes | yes | yes |
| `capture_start` | yes (**seconds**, rotation) | yes (**milliseconds**) | yes (ms) |
| `capture_end` | yes (**seconds**) | yes (ms) | yes (ms) |
| `total_ticks` | yes | yes | yes |
| `chronology_hash` | yes | yes | yes |
| `gaps` | yes | yes | yes |
| `provenance` | **absent** | **absent** | yes |

**yahoo_importer drift class:** manifest-level **lossy reinterpretation**, not tick-level reinterpretation.

---

## Persisted Artifact Survey (`core/chronology/`)

Observed manifest schema signatures (2026-05-26 inventory):

| Count | Manifest field signature | Likely lineage |
|------:|--------------------------|----------------|
| 38 | full 10-field (`source`, `resolution`, `capture_method`, `import_timestamp`, …) | `historical_importer` |
| 12 | slim 6-field (`substrate`, `capture_start`, `capture_end`, `total_ticks`, `chronology_hash`, `gaps`) | live rotation archives |
| 6 | slim + `provenance` | Python / Rust Yahoo producers |
| 1 | legacy kline manifest (`interval`, `total_klines`, `start_timestamp`, `end_timestamp`, `capture_time`) | `scripts/capture_live_chronology.py` |

Observed tick JSON key sets:

| Keys | Exposure |
|------|----------|
| `symbol`, `timestamp`, `price`, `volume`, `is_buyer_maker` | **canonical persisted tick** (historical + live + yahoo) |
| `symbol`, `timestamp`, `open`, `high`, `low`, `close`, `volume` | legacy OHLC capture (`live_capture_0001.jsonl`) |
| `price`, `timestamp`, `volume` | isolated outlier shape (non-standard) |

---

## Consumer Inventory

### Producers (write JSONL + manifest)

| Producer | Location | Manifest lineage | Tick lineage |
|----------|----------|------------------|--------------|
| `historical_importer` | `core/src/bin/historical_importer.rs` | full 10-field | 5-field NormalizedTick |
| `capture_daemon` | `core/src/bin/capture_daemon.rs` | full 10-field (code) | 5-field NormalizedTick |
| `yahoo_importer` | `core/src/bin/yahoo_importer.rs` | slim + `provenance` | 5-field NormalizedTick |
| `yahoo_fetcher.py` | `scripts/yahoo_fetcher.py` | slim + `provenance` | 5-field NormalizedTick |
| `fetch_yahoo.py` | repo root | slim + `provenance` | 5-field NormalizedTick |
| `capture_live_chronology.py` | `scripts/capture_live_chronology.py` | legacy kline manifest | OHLC 7-field (non-NormalizedTick) |

**Note:** persisted `core/chronology/live_capture/*` manifests are **slim 6-field**, not the 10-field shape `capture_daemon.rs` currently emits. Code/artifact divergence exists for live capture manifest authority.

### Serialization / catalog consumers (read manifests loosely)

| Consumer | Reads | Typed `CaptureManifest`? | Replay sensitivity |
|----------|-------|--------------------------|-------------------|
| `scripts/rebuild_catalog.py` | `*_manifest.json` | no | catalog index only |
| `core/chronology/catalog.json` | derived | no | observability |
| `scripts/verify_manifest_v1.py` | certification `manifest.json` | no (different schema) | certification coupling |
| `scripts/emit_manifest_v1.py` | substrate file hash | no | certification outputs |
| `scripts/compare_replay_equivalence.py` | `state_archive` ingestion manifests | no | replay equivalence (frozen cohort lineage) |

### Engine consumers (read tick JSONL)

| Consumer | Expected tick shape | Reads chronology capture dir? |
|----------|--------------------|------------------------------|
| `binance_adapter::load_binance_events_from_jsonl` | Binance raw (`s`,`E`,`p`,`q`,`m`) | via `tick_replay` paths — **not NormalizedTick** |
| `cs-ingest` frozen loader | gzip bar records in `state_archive` | separate lineage |
| ad-hoc Python (`extract_spy.py`, `filter_yahoo.py`, etc.) | NormalizedTick-like keys | yes, field-selective |

---

## Replay Coupling Classification

| Surface | Coupling to V-006 structs | Classification |
|---------|---------------------------|----------------|
| `chronology_hash` in persisted `*_manifest.json` | direct — hash over tick JSONL bytes | **REPLAY-SENSITIVE** — changing serialization changes hash |
| `core/chronology/historical/*` substrate files | direct — 5-field tick JSON | **REPLAY-SENSITIVE** for chronology replay from capture |
| `cs-ingest` batch replay | indirect — uses frozen NSE/OHLC bars, not capture bins | **LOW COUPLING** to Rust capture structs |
| `tick_replay` + `binance_adapter` | **misaligned** — loader does not consume NormalizedTick JSON | **INTEGRATION GAP** (pre-existing; not introduced by V-006 inspection) |
| Certification manifests (`emit_manifest_v1.py`) | consumes substrate file hash | **CERTIFICATION-COUPLED** to whatever substrate file is presented |

---

## Risk Summary

| Risk | Severity | Notes |
|------|----------|-------|
| Triplicated Rust struct definitions | medium | compile-time drift risk; JSON output currently aligned for ticks |
| Yahoo manifest lineage lossy vs historical full manifest | medium | catalog/tooling must tolerate both; identity metadata incomplete for Yahoo universes |
| Live capture manifest code ≠ persisted slim manifests | high | `capture_daemon.rs` authority does not match on-disk live archive shape |
| `capture_start`/`capture_end` seconds vs milliseconds | high | violates frozen contract for live rotation bounds; tick timestamps remain ms |
| `binance_adapter` ≠ NormalizedTick JSON | high | engine replay path not wired to persisted chronology capture format |
| OHLC legacy tick shape | medium | isolated files; not part of NormalizedTick authority |

---

## Observational Conclusions

1. **De facto persisted tick authority:** 5-field NormalizedTick JSON (not the Rust struct location).
2. **De facto persisted manifest authority for historical Binance universes:** 10-field `historical_importer` manifest.
3. **Parallel manifest dialects** exist in production data (slim live, slim+yahoo provenance, legacy kline script).
4. **Consolidation cannot be a simple struct extraction** without a declared manifest dialect policy and timestamp unit correction scope.
5. **Replay ratification** will be required before promoting any single Rust struct module to `CRITICAL / STABLE` if consolidation changes serialized bytes or manifest fields referenced by hash/catalog/certification.

---

## Next Phase Gate (Not Started)

Before `capture_types.rs` or producer migration:

1. Declare canonical manifest dialect(s) — full vs slim vs provenance-extended.
2. Resolve live capture code/artifact manifest mismatch.
3. Resolve timestamp unit policy for rotation bounds.
4. Declare whether `binance_adapter` normalization is in or out of V-006 scope (integration boundary).
5. If serialized output changes are possible, declare replay equivalence scope per `AUTHORITY_MAP.md` governance rule 8.

---

## Verification Performed (This Pass)

- Rust struct comparison across three bins
- Persisted manifest schema census (`core/chronology/`, 57 manifests)
- Tick JSON key-set census
- Consumer/producer grep across Rust + Python + tooling
- No code migration performed
