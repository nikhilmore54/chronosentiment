# V-008 — Test Asset Path Authority Scope (Phase A Inspection)

**Status:** INSPECTION COMPLETE — Phase C migration complete  
**Governance class:** Lane 1 — environmental authority hygiene (bounded SSOT cleanup)  
**Constitutional reference:** `.cursor/rules/chronosentiment-core.mdc`, `AUTHORITY_MAP.md` V-008  
**Cadence posture:** authority first · then cleanup · inspect before migrate

---

## Intent

Inventory how `test_assets/` is resolved across compiled surfaces, classify replay/certification coupling, and determine whether V-008 is a lightweight path-authority convergence (V-002-class) or requires escalation.

This is **not** semantic reducer governance (V-007). This is **not** replay substrate constitutional law (Lane 2).

---

## Scope Declaration

| In scope | Out of scope |
|----------|--------------|
| Hardcoded / implicit `test_assets` path resolution in **compiled module tree** | `state_archive/` absolute paths in observatory JSON (separate archive-path concern) |
| Caller graph for folder-mode data loading | Relocating or reformatting fixture files |
| Environment variable assumptions (`DATA_SOURCE`, proposed `TEST_ASSETS_PATH`) | Lane 3 schema certification |
| Replay / chronology coupling classification | Python research scripts with local absolute paths |
| Escalation threshold | Example binaries under `core/examples/` (documented as non-operational lineage) |

**Key invariant (from V-007):**

```text
authority first
then cleanup
```

even for small items.

---

## Central Question — Answer

> Is the fracture environmental path authority, or hidden replay/substrate semantics?

**Answer: environmental path authority.**

The repository repeats the same developer-local absolute root:

```text
/Users/nikhil/ChronoSentiment_MEGA_FINAL/test_assets
```

across multiple compiled call sites. Fixture **content** drives simulation inputs; path **resolution** is the governance defect. No evidence that `test_assets/` bytes participate in `chronology_hash`, frozen cohort replay, or certification fingerprints.

---

## Asset Root Inventory

### Canonical fixture location (on disk)

| Property | Value |
|----------|-------|
| Repository path | `test_assets/` (repo root) |
| File count (observed) | 33 files |
| Primary formats | `*_5m_clean.csv`, `*.NS.csv`, `btc_ohlc.csv`, `binance_ticks.jsonl`, `fixed_stream_replay.jsonl`, `strategy_store.json` |
| Loader filter | `FolderCandleSource` accepts CSV matching `_5m_clean.csv`, `.ns.csv`, `.bo.csv`; ignores `results.csv` |
| Persisted replay substrate | **no** — not under `core/chronology/` |
| Certification artifact | **no** — local dev/benchmark fixtures |

### Path resolution patterns observed

| Pattern | Example | Portability |
|---------|---------|-------------|
| Developer absolute | `/Users/nikhil/ChronoSentiment_MEGA_FINAL/test_assets` | **fails off-machine** |
| Repo-relative string | `"test_assets"` | cwd-dependent |
| Manifest-relative (good precedent) | `format!("{}/../test_assets", env!("CARGO_MANIFEST_DIR"))` in `core/src/ga.rs` tests | **portable within repo checkout** |
| Shell env override | `DATA_FOLDER=...` in `scripts/run_edge_debug.sh` | script-local |

**Fracture class:** duplicated absolute authority — not competing semantic loaders.

---

## Compiled Call-Site Inventory

### Primary V-008 fracture set (absolute path duplicated)

| # | Location | Function / context | Route / surface | Pattern |
|---|----------|-------------------|-----------------|---------|
| 1 | `services/api/src/simulate.rs:32` | `handle_simulate()` | library handler (`lib.rs`) | absolute |
| 2 | `services/api/src/services/evaluation_service.rs:743` | `load_all_real_scenarios()` | `POST /inspect_strategy` default scenario pick | absolute |
| 3 | `services/api/src/services/evaluation_service.rs:617` | `get_latest_signals()` when `DATA_SOURCE=folder` | `GET /signals/latest` | absolute |
| 4 | `core/src/pipeline.rs:1101` | `evaluate_on_real_data()` when `DATA_SOURCE=folder` | core pipeline / GA folder mode | absolute |

**Duplicate count:** 4 compiled sites, **identical string literal**.

### Consumer graph (operational)

```text
test_assets/  (repo-root fixtures)
        │
        ├─► FolderCandleSource::load_all()
        │         │
        │         ├─► simulate.rs::handle_simulate          [library]
        │         │
        │         ├─► EvaluationService::load_all_real_scenarios()
        │         │         └─► inspect_strategy_handler   [Axum — default scenario name]
        │         │
        │         ├─► EvaluationService::get_latest_signals()  [Axum — if DATA_SOURCE=folder]
        │         │
        │         └─► pipeline::evaluate_on_real_data()     [core — if DATA_SOURCE=folder]
        │
        └─► (not via FolderCandleSource) direct CSV paths in ga.rs tests — manifest-relative ✓
```

| Consumer | Uses absolute path? | Production-wired? | Default path active? |
|----------|--------------------|--------------------|----------------------|
| `handle_simulate` | yes | library only | always when called |
| `load_all_real_scenarios` | yes | yes (`inspect_strategy`) | when request omits `scenarios[]` and `strategy_id` scenario |
| `get_latest_signals` | yes | yes | only if `DATA_SOURCE=folder` (default: `synthetic`) |
| `evaluate_on_real_data` | yes | core STABLE | only if `DATA_SOURCE=folder` (default: `synthetic`) |

**Operational fragility:** `inspect_strategy` default scenario selection fails on any machine where the absolute path is invalid — even when synthetic evaluation paths work.

---

## Environment Assumptions

| Variable | Observed behavior | Default | V-008 relevance |
|----------|-------------------|---------|-----------------|
| `DATA_SOURCE` | `folder` triggers hardcoded `test_assets` in pipeline + `get_latest_signals` | `synthetic` | mode gate — does not fix path when `folder` |
| `TEST_ASSETS_PATH` | **not implemented** | — | proposed canonical override (AUTHORITY_MAP checklist) |
| `CARGO_MANIFEST_DIR` | used in `ga.rs` tests only | compile-time | **recommended SSOT precedent** |
| cwd | `"test_assets"` examples assume repo root cwd | implicit | fragile |

**Fallback behavior today:**

| Condition | Outcome |
|-----------|---------|
| Absolute path invalid | `FolderCandleSource::load_all()` panics on `read_dir` (`expect`) — hard failure |
| Empty folder / no matching CSVs | API returns `EngineError` / `InternalError` mentioning `test_assets` |
| `DATA_SOURCE=synthetic` | folder path **not consulted** for latest signals / `evaluate_on_real_data` |

No graceful repo-root discovery exists in the four fracture sites.

---

## Adjacent Lineage (documented, not V-008 tranche unless authorized)

| Path | Pattern | Classification |
|------|---------|----------------|
| `core/examples/reproduce_parity.rs:16` | absolute CSV file | example / non-operational |
| `core/examples/live_diagnostic.rs:17` | `"test_assets"` relative | example |
| `core/examples/run_pipeline.rs:5` | relative csv paths | example |
| `scripts/run_edge_debug.sh:20` | absolute `DATA_FOLDER` | dev script |
| `scripts/research/run_structured_audit_automation.py` | absolute unrelated path | research |
| `observatory/verification_*.json` | absolute `state_archive` roots | archive tooling — **not test_assets** |

---

## Good Precedent (not yet centralized)

`core/src/ga.rs` tests (e.g. `test_evaluate_strategy_multi_trade_cap_respected`):

```rust
let test_assets = format!("{}/../test_assets", env!("CARGO_MANIFEST_DIR"));
```

| Property | Value |
|----------|-------|
| Portable within checkout | yes |
| Binds to crate location | yes |
| Operational authority | test-only today |
| Recommended convergence target | **manifest-relative repo root discovery** |

---

## Replay / Certification Coupling Check

| Coupling vector | V-008 sensitivity | Notes |
|-----------------|-------------------|-------|
| `chronology_hash` / JSONL bytes | **NONE** | `test_assets/` not in chronology tree |
| `cs-ingest` frozen cohort | **NONE** | separate substrate |
| V-001 strategy identity replay | **NONE** | unrelated |
| Observatory / certification manifests | **NONE** | no test_assets path in cert fingerprints |
| Simulation determinism | **LOW** | same CSV bytes → same candles if path resolves; path drift breaks availability not hash law |
| Hidden snapshot authority | **NONE** | no persisted replay snapshots sourced from absolute path string |

**Lane escalation:** V-008 remains **Lane 1**. Lane 2 not triggered.

**Hidden coupling inspection result:** no persisted replay fixtures, chronology hash binding, or environment-dependent replay semantics discovered beyond **input availability** (folder missing → error/panic).

---

## Classification

| Label | Applies? |
|-------|----------|
| alias-equivalent paths | **no** — absolute vs relative vs manifest-relative |
| environmental authority fracture | **yes** |
| structurally divergent resolution | **yes** (4 literals + scattered adjacent patterns) |
| semantically divergent loaders | **no** — single `FolderCandleSource` |
| externally serialized replay law | **no** |
| replay-substrate-bound | **no** |

**Governance intensity:** bounded SSOT cleanup (V-002-class) — **not** constitutional ratification.

---

## Escalation Threshold

| Finding | Threshold crossed? | Next artifact |
|---------|-------------------|---------------|
| Developer-local absolute path duplication | **yes** | Phase B: path authority decision |
| Replay substrate coupling | no | — |
| Certification schema coupling | no | Lane 3 separate |
| Semantic dual loaders | no | — |

**Migration blocked until:**

1. Phase B declares single path resolution authority (env override + manifest-relative default).
2. Tranche scope lists which compiled sites migrate in Phase C (recommended: all 4 primary sites together).
3. Panic-on-missing-folder policy declared (error vs default synthetic fallback).

**Prohibited without authorization:**

- Moving fixtures into `core/chronology/` (would trigger Lane 2 review).
- Conflating with `state_archive` path cleanup in same tranche without scope doc.

---

## Recommended Phase B Posture (not authorized here)

Proportional target aligned with AUTHORITY_MAP checklist:

```text
single declared test asset authority
→ deterministic path resolution
→ no hidden local assumptions
→ cargo/test verification
→ ledger closure
```

Likely decision shape:

| Concern | Proposed direction |
|---------|-------------------|
| Canonical resolver | one function (e.g. `resolve_test_assets_dir()`) in core or shared api helper |
| Override | `TEST_ASSETS_PATH` env var |
| Default | `{repo_root}/test_assets` via manifest-relative discovery from `core` or `api` crate |
| Call sites | replace 4 literals with resolver |
| Failure mode | structured error instead of `read_dir` panic where feasible |
| Examples / scripts | optional follow-on tranche — not blocking compiled convergence |

---

## Artifact Discipline

| Phase | Artifact | Status |
|-------|----------|--------|
| A — inspect | this document | **complete** |
| B — path authority decision | `V008_TEST_ASSET_AUTHORITY_DECISION.md` | **complete** |
| C — bounded migration | compile + tests + `AUTHORITY_MAP.md` | **authorized — not started** |

---

## References

- `AUTHORITY_MAP.md` — V-008 ledger, Lane 1 cadence
- `core/src/folder_source.rs` — `FolderCandleSource` loader
- `services/api/src/simulate.rs:32` — original ledger citation
- `services/api/src/services/evaluation_service.rs:617,743`
- `core/src/pipeline.rs:1097–1107` — `DATA_SOURCE` gate + absolute path
- `core/src/ga.rs:13594` — manifest-relative test precedent
- `docs/governance/V007_TYPE_AUTHORITY_SCOPE.md` — precedent for Phase A scope doc structure
- `docs/governance/V002` resolution pattern — local shadow → canonical SSOT
