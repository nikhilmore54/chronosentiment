# V-006 — Phase C Classification

**Status:** COMPLETE — classification declared; migration **NOT AUTHORIZED**; ratification **NOT GRANTED**  
**Governance class:** replay substrate identity governance  
**Prerequisites:** `V006_SERIALIZATION_LAW_DECLARATION.md`, Phase B `ratification_report.json`  
**Constitutional reference:** `.cursor/rules/chronosentiment-core.mdc`, `AUTHORITY_MAP.md` V-006

---

## Phase C Objective

```text
classification of lawful vs unlawful producer divergence
```

Not migration. Not extraction. Not convergence.

```text
freeze → prove → classify → only then authorize migration
```

---

## Constitutional Questions — Rulings

| Question | Ruling |
|----------|--------|
| Is Python spacing drift historically lawful? | **Yes** — persisted Yahoo Dialect C universes remain lawful evidence. Emission lineage: `python_json_default_v1`. |
| Is cross-language byte equivalence required? | **For historical evidence: no.** **For future unified producer authority: yes** — must emit under declared forward law (`compact_rust_v1`). |
| Can semantic equivalence substitute for byte equivalence? | **No** — prohibited for `chronology_hash`, certification, and producer ratification. |
| Is manifest ms correction replay-sensitive? | **Metadata-sensitive** — tick JSONL unchanged → hash unchanged; catalog window interpretation changes. Requires scoped declaration before batch rewrite. |
| Does future producer authority require byte fixtures? | **Yes** — mandatory per `V006_SERIALIZATION_LAW_DECLARATION.md` L-3 and ratification requirements. |
| Can Dialect B remain admitted but non-emittable? | **Yes** — lawful historical evidence; forward live emission must target Dialect A + ms bounds per `V006_LIVE_CAPTURE_AUTHORITY_DECISION.md`. |

---

## Drift Classification Matrix

Evidence source: Phase B `ratification_report.json` + Phase A inspection.

| Drift surface | Observed evidence | Classification | Historical status | Forward emission status |
|---------------|-------------------|----------------|-------------------|-------------------------|
| **Binance-class Rust tick bytes** | Dialect A/B fixtures: tick `byte_identical`, hash match on round-trip | **ratifiable substrate behavior** (candidate) | lawful | lawful candidate under `compact_rust_v1` |
| **Compact Rust JSON** | `serde_json` compact emission matches Binance persisted bytes | **lawful serialization lineage candidate** | lawful where already persisted | target forward law |
| **Python spaced JSON** | Dialect C fixture: 114-byte lines vs 105-byte Rust emission; hash mismatch | **serialization-law divergence** | **lawfully admitted** | **not lawful** for unified ratification |
| **Manifest seconds bounds (Dialect B)** | live fixture manifest bounds in seconds; ticks in ms | **chronology-interpretation defect** | **historically admitted** | **unlawful** for forward emission |
| **Output root mismatch** | code emits `chronology/`; lawful root `core/chronology/` | **authority-lineage defect** | n/a (code defect) | must correct before ratification |
| **Manifest Dialect A omission (live forward)** | `capture_daemon` targets Dialect A but live historical is Dialect B | **manifest dialect drift** | Dialect B admitted | forward must emit Dialect A |
| **Path + timestamp + manifest (composite)** | no producer passes all proof dimensions | **ratification_blocked** | — | — |

---

## Producer Ratification Posture (Phase C Decision)

| Producer | Tick serialization | Manifest law | Output lineage | Phase C decision |
|----------|-------------------|--------------|----------------|------------------|
| `capture_daemon.rs` | **candidate** (`compact_rust_v1` proven on Binance fixtures) | **non-compliant** (seconds bounds; path drift) | **non-compliant** | **NOT RATIFIED** |
| `historical_importer.rs` | **candidate** (same Rust serde path; Binance fixtures byte-identical) | **compliant** on Dialect A historical pattern | **non-compliant** (path drift) | **NOT RATIFIED** |
| `yahoo_importer.rs` | **unproven** on Yahoo fixtures via Rust probe | Dialect C compliant shape | **non-compliant** (path drift) | **NOT RATIFIED** |
| `yahoo_fetcher.py` | **alternate lineage** (`python_json_default_v1`; proven divergent from `compact_rust_v1`) | Dialect C compliant | **non-compliant** (path drift) | **NOT RATIFIED — historically admitted emitter only** |

**Universal ruling:** No producer receives canonical chronology authority in Phase C.

---

## Migration Tranche Classification

If remediation is attempted, classify before any code change:

| Proposed work | Tranche class | Authorized now? |
|---------------|---------------|-----------------|
| Extract shared Rust struct with **proven zero tick byte change** | **bounded schema convergence** | **no** — requires ratification path first |
| Align output root to `core/chronology/` only | **authority-lineage fix** | **no** — scope declaration required |
| Fix `capture_daemon` manifest bounds to ms | **manifest metadata migration** (JSONL unchanged) | **no** — catalog interpretation scope required |
| Re-emit Yahoo spaced JSON as compact Rust JSON | **replay-sensitive substrate migration** | **no** — hash rotation + scope required |
| Wire Python Yahoo to compact JSON emission | **replay-sensitive** for new writes; historical remains admitted | **no** |
| `capture_types.rs` extraction | blocked regardless | **no** |

**Phase C ruling:** All migration tranches remain **blocked** pending explicit scoped authorization artifacts per tranche.

---

## Multi-Producer Chronology Law (Declared)

The repository formally recognizes:

```text
multi-producer chronology is not lawfully unified
```

Lawful state is **pluralistic historical evidence** under declared dialect and serialization lineages — not a single implicit producer authority.

Unification requires:

1. declared forward serialization law (`compact_rust_v1`),
2. declared manifest dialect law (Dialect A / C rules),
3. byte fixture proof per producer,
4. Phase C ratification grant (not yet issued).

---

## Ratification Doctrine (Forward)

A producer may be marked **RATIFIED** only when **all** hold:

| Criterion | Required |
|-----------|----------|
| Tick byte proof | PASS on applicable fixtures |
| Forward serialization law | declared lineage match |
| Manifest forward law | Dialect A (Binance/live) or declared Dialect C (alternate source) |
| Output root | `core/chronology/...` |
| Timestamp units | ms for tick + manifest bounds (where applicable) |
| Phase C classification | no unresolved `serialization_drift`, `path_authority_drift`, or `timestamp_unit_drift` on forward law dimensions |

**Current status:** zero producers satisfy all criteria.

---

## Relationship to V-007

V-007 (`SystemState` / `OrderState` naming collision) is **type-authority governance** — lower replay substrate coupling than V-006. V-006 Phase C classification should remain closed before opening V-007 to preserve replay substrate coherence.

---

## Non-Claims

Phase C does **not**:

- authorize code changes,
- create `capture_types.rs`,
- rewrite persisted chronology,
- ratify any producer,
- declare replay equivalence for any migration tranche.

---

## Ledger Alignment

`AUTHORITY_MAP.md` V-006: **PHASE C CLASSIFICATION COMPLETE** — serialization law declared; all producers **NOT RATIFIED**; migration blocked.

Cadence:

```text
observe → classify → declare law → freeze bytes → prove → classify → [migrate only under declared scope]
```

Current position: **classify ✓** — **migrate ✗** — **ratify ✗**
