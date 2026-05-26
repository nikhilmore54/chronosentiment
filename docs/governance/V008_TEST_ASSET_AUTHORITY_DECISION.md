# V-008 — Test Asset Path Authority Decision (Phase B)

**Status:** DECIDED — policy binding; Phase C migration not yet authorized  
**Governance class:** Lane 1 — environmental authority hygiene (V-002-class bounded SSOT)  
**Prerequisites:** `docs/governance/V008_TEST_ASSET_AUTHORITY_SCOPE.md` (Phase A inspection complete)  
**Constitutional reference:** `.cursor/rules/chronosentiment-core.mdc`, `AUTHORITY_MAP.md` V-008

---

## Question

```text
Who owns lawful resolution of the test_assets root?
```

Phase A established **environmental authority fracture without semantic authority fracture**. Phase B declares singular path resolution doctrine before literal replacement.

---

## Governance Profile (confirmed)

| Class | Burden |
|-------|--------|
| not replay constitutional | no byte fixtures, cohort adjudication, or ratification archive |
| not semantic loader governance | `FolderCandleSource` remains sole loader |
| environmental path authority | single resolver + explicit override |

**Core distinction (binding):**

```text
shared loader
≠
shared authority resolution
```

**Lane 2:** not triggered — environment-bound, not replay-bound.

---

## Evidence Summary (from Phase A)

| Finding | Implication |
|---------|-------------|
| 4 compiled sites share identical absolute literal | implicit filesystem authority duplicated |
| Single loader (`FolderCandleSource`) | convergence target is path resolution only |
| `ga.rs` tests use manifest-relative path | existing lawful lineage — not a new pattern |
| `inspect_strategy` default scenario uses path | operational route affected — not test-only |
| No chronology / certification coupling | Lane 1 proportional cleanup sufficient |
| `DATA_SOURCE=synthetic` default | folder path inactive for most pipeline/signal paths unless opted in |

---

## Decisions

### D-1 — Canonical path resolver authority

**Decision:** **`chronosentiment_core`** owns the sole lawful test-asset root resolver.

**Canonical function (Phase C):** `resolve_test_assets_dir() -> Result<PathBuf, TestAssetsPathError>` (exact error type at implementation discretion).

**Recommended module:** `core/src/test_assets.rs` (exported from `core/src/lib.rs`).

**Rationale:** Loader (`folder_source.rs`), pipeline folder mode, and the lawful `ga.rs` test precedent all live in `core`. API and library handlers are consumers — not path authorities.

---

### D-2 — Override surface (`TEST_ASSETS_PATH`)

**Decision:** Environment variable **`TEST_ASSETS_PATH`** is the **sole runtime override** for test-asset root resolution.

| Property | Law |
|----------|-----|
| Precedence | **first** — if set and non-empty after trim, use verbatim |
| Semantics | absolute or relative path accepted; must refer to a directory |
| Authority | operational override — not persisted replay law |
| Documentation | must appear in resolver rustdoc and Phase C call-site behavior |

**Non-decision:** `DATA_FOLDER` in shell scripts remains script-local — not compiled authority.

---

### D-3 — Manifest-relative default law

**Decision:** When `TEST_ASSETS_PATH` is unset or empty, default resolution is:

```text
{CARGO_MANIFEST_DIR}/../test_assets
```

evaluated from the **`chronosentiment_core` crate manifest** (same lineage as `ga.rs` tests).

| Property | Law |
|----------|-----|
| Portability | valid for any repo checkout path |
| cwd independence | required — must not depend on process working directory |
| Fixture location | `{repo_root}/test_assets` remains on-disk authority |
| Relocation of fixtures | out of V-008 scope — resolver points at existing root |

**Rationale:** Adopts existing lawful lineage instead of inventing mid-remediation policy.

---

### D-4 — Resolution topology (binding order)

**Decision:** Lawful resolution order:

```text
TEST_ASSETS_PATH override (if non-empty)
→ manifest-relative default
→ shared resolver (single function)
→ FolderCandleSource { folder_path: resolved string }
```

| Stage | Owner |
|-------|-------|
| Path authority | `resolve_test_assets_dir()` |
| Candle loading semantics | `FolderCandleSource::load_all()` — unchanged |

**Prohibited:** call-site-local path string construction for `test_assets` root in compiled module tree after Phase C.

---

### D-5 — Prohibition on embedded absolute developer roots

**Decision:** **Embedded absolute developer-local roots are prohibited** in the compiled module tree for `test_assets` resolution.

Specifically retired literal class:

```text
/Users/nikhil/ChronoSentiment_MEGA_FINAL/test_assets
```

and any equivalent machine-bound absolute root for this purpose.

**Scope:** Phase C tranche (4 primary compiled sites). Examples, scripts, and observatory JSON are **adjacent lineage** — optional follow-on tranche, not blocking Phase C closure.

---

### D-6 — Failure and fallback semantics

**Decision:**

| Condition | Lawful behavior |
|-----------|-----------------|
| `TEST_ASSETS_PATH` set but path not a directory | resolver returns **structured error** |
| Default path not present / not readable | resolver returns **structured error** |
| Resolver error at API boundary | propagate as `ApiError::EngineError` or `InternalError` with explicit message — **no silent synthetic fallback** |
| `FolderCandleSource::load_all` empty CSV set | existing caller errors preserved |
| `DATA_SOURCE=synthetic` | folder resolver **not invoked** — unchanged mode gate |

**Panic policy:** Phase C **may** improve `FolderCandleSource` to accept pre-validated paths; converting `read_dir` panic to error is **recommended** but not required for V-008 ledger closure if callers validate via resolver first.

**Non-decision:** V-008 does not change synthetic scenario generation semantics when folder mode is off.

---

### D-7 — Phase C operational tranche (compiled sites)

**Decision:** Phase C must replace all **4 primary compiled literals** in one tranche:

| # | Location |
|---|----------|
| 1 | `services/api/src/simulate.rs` |
| 2 | `services/api/src/services/evaluation_service.rs` — `load_all_real_scenarios()` |
| 3 | `services/api/src/services/evaluation_service.rs` — `get_latest_signals()` folder branch |
| 4 | `core/src/pipeline.rs` — `evaluate_on_real_data()` folder branch |

**Consumer preservation:** `POST /inspect_strategy` default scenario selection continues to use real folder scenarios when path resolves — behavior unchanged except portability.

**Library preservation:** `handle_simulate()` continues to load folder candles through `FolderCandleSource` — loader semantics unchanged.

---

### D-8 — Replay / certification scope boundary

**Decision:** V-008 Phase C **must not**:

- move fixtures into `core/chronology/`,
- alter chronology bytes or serialization law,
- bind test-asset paths into certification fingerprints,
- expand into `state_archive` path cleanup,
- or claim `replay_response.schema.json` conformance.

**Interpretation:** Path resolution is **deterministic environment governance**, not replay constitutional law.

---

## Phase C Authorization Threshold

Phase C bounded migration may begin when this document is recorded in `AUTHORITY_MAP.md`.

**Phase C must deliver:**

1. `resolve_test_assets_dir()` in `chronosentiment_core` with D-2 / D-3 / D-6 semantics.
2. All 4 compiled call sites use resolver output — zero absolute developer literals remain in tranche.
3. `cargo check` / `cargo test` for affected crates pass.
4. `AUTHORITY_MAP.md` V-008 status → resolved.

**Phase C mechanical sequence (authorized):**

```text
extract resolver
→ replace 4 literals
→ preserve FolderCandleSource semantics
→ cargo/test verification
→ AUTHORITY_MAP closure
```

**Phase C must not deliver (without new scope doc):**

- example binary path cleanup tranche,
- observatory `archive_dir` normalization,
- fixture file moves or renames.

---

## Non-Claims

This decision does **not**:

- declare `test_assets/` fixtures replay-certified or chronology-admissible,
- resolve V-009 / V-010,
- change `DATA_SOURCE` mode semantics beyond using lawful path when `folder` is selected,
- require Lane 2 producer ratification or byte fixtures.

---

## Artifact Discipline

| Phase | Artifact | Status |
|-------|----------|--------|
| A — inspect | `V008_TEST_ASSET_AUTHORITY_SCOPE.md` | **complete** |
| B — path authority decision | this document | **complete** |
| C — bounded migration | code + tests + `AUTHORITY_MAP.md` | **complete** |

**Cadence preserved:**

```text
inspect first        ✓
classify coupling    ✓
declare authority    ✓
converge literals    → Phase C
```

---

## References

- `docs/governance/V008_TEST_ASSET_AUTHORITY_SCOPE.md`
- `core/src/folder_source.rs` — loader authority (unchanged)
- `core/src/ga.rs:13594` — manifest-relative precedent
- `core/src/pipeline.rs:1101`
- `services/api/src/simulate.rs:32`
- `services/api/src/services/evaluation_service.rs:617,743`
- `docs/governance/V007_TYPE_AUTHORITY_DECISION.md` — Phase B decision structure precedent
