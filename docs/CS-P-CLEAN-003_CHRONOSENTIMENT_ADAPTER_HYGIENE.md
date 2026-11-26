# CS-P-CLEAN-003 — ChronoSentiment adapter hygiene plan

**Document type:** Cleanup plan (authorize before execute)  
**Status:** Phase A–E **EXECUTED 2026-11-24**  

### Execution log (2026-11-24)

| Phase | Done |
| --- | --- |
| **A** | `adapters/chronosentiment/README.md` · removed empty `src/archive/` · DS_Store scrub · `rp608_position_decide` added to governed bin allowlist |
| **B** | Ledger JSONL → `archive/chronosentiment_live_capture_ledger_202609/` + SHA256SUMS · E4 fixture → `tests/fixtures/e4_shadow/` · `live_capture/README.md` |
| **C** | `legacy/quarantine/` → `archive/chronosentiment_legacy_quarantine/` · `include_str!` / discipline paths updated |
| **D** | 49 CS-P-006 / heritage / sqlx `[[test]]` → `required-features = ["research"]` · `tests/rp608_product_lifecycle_contracts.rs` on default surface |
| **E** | Deploy bins → `artifacts/deploy/chronosentiment/` · systemd / adapter client / CLI docs updated · stub `adapters/chronosentiment/bin/README.md` |
| **Verify** | `adapter_discipline_invariants` 13/13 · floor-lock twin Python 7/7 · observatory slice/maturity paths updated |  
**Date:** 2026-11-24  
**Scope:** `adapters/chronosentiment/` **only** (not repo-wide, not `scripts/`, not Pi `live_capture/shadow_validation`)  
**Parent:** CS-P-AUDIT-001 · CS-P-CLEAN-001 (quarantine already done)  
**Aligned with:** `.cursor/rules/chronosentiment-core.mdc` — determinism · research ≠ product · Coralys learns / ChronoSentiment evaluates  

**Does not:** change Gate 2X floor-lock twin economics · Gate 5 live geometry · OF ±0.30 · delete B3/B4 provenance without inventory  

---

## 1. Why now

Floor-lock / Gate2X research proved the **product** path (`src/product/gate2x_decide.rs`, `rp608_plan_lifecycle`, `bin/rp608_position_decide.*`) is load-bearing. The adapter tree around it has accrued:

| Surface | Scale | Issue |
| --- | --- | --- |
| `target/` | **~3.0 GB** | Local build cache inside adapter (should not be treated as source) |
| `Cargo.toml` `[[bin]]` | **47** entries | Noise; most `legacy-lake` / `research` gated but hard to navigate |
| `legacy/bin` | 29 | Heritage demos — OK behind feature, but Cargo.toml wall is opaque |
| `legacy/quarantine` | 18 files | Already dead to default build — candidates for `archive/` |
| `tests/` | **60** `*.rs` | Heavy CS-P-006* matrix; slow CI signal vs product Gate2X |
| `live_capture/ledger/` | **7.1 MB** | Old shadow JSONL inside the crate (Sep 2026) — not product source |
| `src/archive/` | empty | Dead directory |
| `bin/*.darwin-arm64` / `*.aarch64-linux` | 2 binaries | Checked-in deploy artifacts next to source |

CLEAN-001 already moved research out of the default compile graph. CLEAN-003 is **hygiene + navigation**, not a second quarantine redesign.

---

## 2. Keep / move / delete (default stance)

```text
KEEP (product — do not casually touch)
  src/product/{gate2x_decide,rp608_plan_lifecycle,...}
  src/bin/rp608_position_decide.rs
  artifacts/deploy/chronosentiment/rp608_position_decide.{darwin-arm64,aarch64-linux}   # until deploy pipeline owns them
  tests/adapter_discipline_invariants.rs
  features: default / legacy-lake / research (boundary stays)

MOVE (out of daily path, retain provenance)
  live_capture/ledger/*.jsonl  →  archive/ or datasets/ with README pointer
  legacy/quarantine/**         →  archive/chronosentiment_legacy_quarantine/ (optional)
  empty src/archive/           →  delete dir

DELETE / gitignore only
  adapters/chronosentiment/target/   →  ensure gitignored; never commit
  .DS_Store

DONE (CLEAN-003-D/E 2026-11-24)
  CS-P-006* / heritage / sqlx tests gated via [[test]] required-features = ["research"]
  position_decide binaries → artifacts/deploy/chronosentiment/

DEFER (needs Anuj call)
  Collapsing 47 Cargo [[bin]] into a manifest include / workspace subcrate
  (README lane index added 2026-11-24 — counts only; no Cargo split yet)
```

---

## 3. Phases (execute only after explicit OK)

### Phase A — Zero-risk hygiene (≤1 PR, no behavior change)

1. Confirm `adapters/chronosentiment/target/` is gitignored; document `cargo clean -p chronosentiment_adapter` for local reclaim (~3 GB).  
2. Remove empty `src/archive/`.  
3. Remove `.DS_Store` under the adapter.  
4. Add `adapters/chronosentiment/README.md` map: PRODUCT / LEGACY / RESEARCH / DEPLOY BINS / TESTS.  
5. Inventory table: each `src/bin/*` → used by (Pi daemon / research / deferred / unknown).

**Exit:** README + inventory committed; `cargo test -p chronosentiment_adapter` green.

### Phase B — Capture artifacts out of the crate

1. Move `live_capture/ledger/*.jsonl` to e.g. `archive/chronosentiment_live_capture_ledger_202609/` with SHA256 manifest.  
2. Leave a one-line README in `adapters/chronosentiment/live_capture/` pointing to archive (or delete empty tree).  
3. **Do not** move Pi `live_capture/shadow_validation/**` (outside this folder).

**Exit:** crate tree has no multi-MB JSONL; archive documented.

### Phase C — Legacy quarantine archive (optional)

1. `legacy/quarantine/**` → repo `archive/` (git history preserves; tree declutter).  
2. Keep `legacy/bin/**` + `required-features = ["legacy-lake"]` until a follow-up proves unused in CI.  
3. Trim Cargo.toml comments into README “legacy bin index” so the toml is skimmable.

**Exit:** default / research / legacy-lake feature matrix unchanged; CI same.

### Phase D — Test signal (optional, separate PR)

1. Classify tests: **discipline** / **product Gate2X+lifecycle** / **CS-P-006 ecology** / **sqlx** (needs DB).  
2. Gate heavy CS-P-006* behind `--features research` or a `[[test]]` required-features block so default `cargo test -p chronosentiment_adapter` stays fast.  
3. Add one focused test module for floor-lock / plan-lifecycle contracts if missing in Rust (Python twin tests stay in `tests/test_rp608_gate2x_floor_lock_twin.py`).

**Exit:** default test time down; research feature still runs full matrix.

### Phase E — Deploy bin ownership (optional)

1. Decide: checked-in `bin/rp608_position_decide.*` stay (Pi scp simplicity) **or** move to `artifacts/deploy/chronosentiment/` + document zigbuild recipe.  
2. No rename of protocol bins without Pi service unit update.

---

## 4. Hard non-goals

- No merge of Coralys ← ChronoSentiment dependency direction  
- No deletion of B3/B4 lake generators without CLEAN-001-style feature gate proof  
- No Gate 5 / OF ±0.30 / floor-lock twin parameter changes “while cleaning”  
- No repo-wide `scripts/` RP-608 Python cleanup in this ticket (separate plan if desired)  
- No `git clean -fdx` of adapter `target/` on Pi without confirming no in-use build dir for services  

---

## 5. Suggested order of authorization

| Ask Anuj | Phase | Risk |
| --- | --- | --- |
| **1** | **A** (README + empty dir + gitignore) | None |
| **2** | **B** (move old ledger JSONL) | Low — path break only if something still reads crate-local ledger |
| **3** | **C** (archive quarantine) | Low |
| **4** | **D** (test gating) | Medium — CI recipe change |
| **5** | **E** (deploy bin location) | Medium — Pi deploy docs |

Recommend: **authorize Phase A now**; B after a 30s `rg` for `live_capture/ledger` consumers; C–E as follow-ups.

---

## 6. Verification checklist (each phase)

```bash
cargo test -p chronosentiment_adapter
cargo test -p chronosentiment_adapter --features research
# product Gate2X path still builds:
cargo build -p chronosentiment_adapter --release --bin rp608_position_decide
python3 -m unittest tests.test_rp608_gate2x_floor_lock_twin
```

Pi: no adapter-folder cleanup required for floor-lock research accrual; twin journals stay under workspace `live_capture/shadow_validation/`.

---

## 7. Relation to floor-lock research

Cleanup must **not** block W1/W2 accrual. Phase A/B can run in parallel with:

```bash
./scripts/research/rp608_gate2x_floor_lock_pull_and_snapshot.sh
```

If a cleanup PR and a research snapshot conflict, prefer research journals untouched.
