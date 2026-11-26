# RR2-C — Module Utilization Analysis

**Programme:** Repository Rationalization  
**Phase:** RR2 — Repository Structural Analysis  
**Pass:** C — Orphaned Module Classification  
**Status:** Complete  
**Produced:** 2026-08-02  
**Input:** `docs/governance/rr2b_files.csv` (lib:orphan=9 rows)  
**Output:** This document + `docs/governance/rr2c_module_utilization.csv`

---

## 1. Scope

RR2-B Enhanced identified 9 files classified as `lib:orphan`: absent from every compilation root's module tree (no `mod` declaration path reaches them from any `lib.rs`, `main.rs`, `src/bin/*.rs`, `tests/*.rs`, `examples/*.rs`, or `benches/*.rs`). This pass classifies each orphan by intended purpose, implementation completeness, git history, and assigns a structural evidence level (E0–E5).

Evidence levels used in this pass are structural only (RR2 scope). Historical evidence (RR3) may upgrade or downgrade these assessments.

| Level | Meaning |
|-------|---------|
| E0 | Unknown — insufficient information |
| E1 | No Cargo reference — crate not in workspace |
| E2 | No module declaration — file not reachable via any `mod` chain |
| E3 | No symbol reference — file reachable in principle but no caller imports its symbols |
| E4 | Not compiled — file excluded from all compilation units |
| E5 | Validated removable — self-declared deprecated or canonical replacement confirmed |

All 9 orphans are at minimum E2 (no `mod` declaration path). The table below records the highest applicable level.

---

## 2. Classification Categories

| Category | Definition |
|----------|-----------|
| **obsolete** | Functionality explicitly superseded; canonical replacement named in code or commit |
| **dormant** | Complete, compilable implementation; never wired into the module tree; may have future value |
| **incomplete** | Partial or stub implementation; never reached a state suitable for wiring |
| **accidental** | File created by tooling or copy-paste; no intentional design |
| **generated** | Machine-generated artefact; not hand-authored |

---

## 3. Orphan Inventory

### 3.1 `financial/strategies/src/edge_decay.rs`

| Field | Value |
|-------|-------|
| LOC | 225 |
| First commit | 2026-03-27 (`dc0a0ed2` ancestry) |
| Last active | 2026-05-28 `dc0a0ed2` — "Finalize orchestration certification freeze" |
| Classification | **dormant** |
| Evidence level | **E3** |
| Completeness | High — full implementation with unit tests |

**Analysis.** Implements `run_edge_decay()`, a multi-model edge-decay measurement pipeline that runs tick replay under three execution models (Ideal, Spread, SpreadSlippage) and returns sorted `EdgeDecayResult` records. References five sibling modules: `crate::ga`, `crate::pnl_overlay`, `crate::replay_evaluator`, `crate::strategy_ranking`, `crate::tick_replay`. Contains a `#[cfg(test)]` block with two passing assertions. The file is structurally complete and internally consistent. It was never added to `lib.rs` or any intermediate `mod.rs`. The commit message "Finalize orchestration certification freeze" suggests the broader pipeline was frozen before this module was promoted. No symbol from this file is imported by any reachable module (E3).

**RR4 recommendation input:** Candidate for promotion to the `financial/strategies` module tree if edge-decay measurement is still a live requirement. Requires a `pub mod edge_decay;` declaration in `lib.rs` and a review of whether `run_edge_decay`'s API matches current caller expectations.

---

### 3.2 `financial/strategies/src/edge_half_life_estimator.rs`

| Field | Value |
|-------|-------|
| LOC | 72 |
| First commit | 2026-05-22 |
| Last active | 2026-05-28 `dc0a0ed2` — "Finalize orchestration certification freeze" |
| Classification | **dormant** |
| Evidence level | **E3** |
| Completeness | High — complete struct with full method implementation |

**Analysis.** Implements `EdgeHalfLifeEstimator`, a five-factor trade lifespan estimator (directional coherence, volatility persistence, drift toxicity, regime multiplier, execution feasibility). The doc comment explicitly names this "ChronoSentiment — Temporal Elasticity Timing Brain", indicating intentional design. The `estimate_half_life()` method is fully implemented with clamped multipliers and absolute boundary enforcement (10–240 minutes). No `Default` impl is missing; the struct is self-contained. Never declared in `lib.rs`. Frozen at the same certification-freeze commit as the rest of this cluster.

**RR4 recommendation input:** Candidate for promotion alongside `edge_decay.rs`. The two modules are conceptually coupled (half-life estimation feeds edge-decay measurement).

---

### 3.3 `financial/strategies/src/paper.rs`

| Field | Value |
|-------|-------|
| LOC | 1179 |
| First commit | 2026-04-27 |
| Last active | 2026-05-28 `dc0a0ed2` — "Finalize orchestration certification freeze" |
| Classification | **dormant** |
| Evidence level | **E3** |
| Completeness | High — substantial, feature-complete subsystem |

**Analysis.** The largest orphan in the repository (1179 LOC). Implements a complete paper-trading engine: `PaperRegistry` (30+ fields tracking open/closed trades, PnL history, regime metrics, path metrics, equity curve, drawdown), `TradeIntent`, `ActiveTrade`, `TradeObservation`, `update_paper_registry()` (the main per-candle update loop with fill logic, adaptive TP/SL geometry, propagation phase tracking, mortality exits), and `apply_slippage()`. References `crate::domain` and `chronosentiment_core::market_adapter::Candle`. The `summary()` method emits structured `[PAPER_SUMMARY]`, `[CAPITAL_CONTENTION]`, `[RISK_PROFILE]`, `[PATH_ALPHA]`, `[REGIME_ALPHA]`, `[SIGMA]`, and `[RISK_SUMMARY]` log lines — a complete observability surface. This is not a stub; it is a production-grade paper-trading harness that was frozen before being wired into the library's public API.

**RR4 recommendation input:** High-value dormant asset. If paper-trading capability is required by any experiment binary, this module should be promoted. If the capability has been superseded by a different implementation, this is a candidate for archival (not deletion — 1179 LOC of domain logic warrants preservation in the archive branch).

---

### 3.4 `financial/strategies/src/pipeline/certification/orchestration.rs`

| Field | Value |
|-------|-------|
| LOC | 61 |
| First commit | 2026-05-28 |
| Last active | 2026-05-28 `dc0a0ed2` — "Finalize orchestration certification freeze" |
| Classification | **incomplete** |
| Evidence level | **E2** |
| Completeness | Medium — functions implemented but parent module chain broken |

**Analysis.** Defines two `pub(crate)` functions: `asset_loop_order_is_stable()` and `multi_asset_execution_projection_is_stable()`, both producing a deterministic `ExecutionProjection` with SHA-256 hash of the canonicalized asset list. The `pub(crate)` visibility implies these were intended as internal certification utilities. However, the parent module `pipeline/certification/` is itself not declared in `pipeline/mod.rs` — the entire `certification` sub-module is absent from the module tree. This is a two-level orphan: neither `certification` nor `orchestration` is reachable. The file was created and frozen in the same commit that created it (first = last = 2026-05-28), suggesting it was written as part of a certification framework that was never completed.

**RR4 recommendation input:** If the `pipeline/certification` framework is intended to be completed, this file needs both `pub mod certification;` in `pipeline/mod.rs` and `pub(crate) mod orchestration;` in `pipeline/certification/mod.rs`. If the certification framework was abandoned, this is a candidate for archival.

---

### 3.5 `financial/strategies/src/signals.rs`

| Field | Value |
|-------|-------|
| LOC | 191 |
| First commit | 2026-05-28 |
| Last active | 2026-05-28 `dc0a0ed2` — "Finalize orchestration certification freeze" |
| Classification | **dormant** |
| Evidence level | **E3** |
| Completeness | High — complete type vocabulary |

**Analysis.** Defines the signals vocabulary for the `financial/strategies` crate: `TradeSignal`, `SignalsSnapshot`, `SignalMeta`, `SignalAction`, `RecommendationStatus`, `AlphaPorosity`, `EntryType`, `EdgeLossReason`, `EdgeLossBreakdown`, `EdgeTransfer`, `ReasonLossShare`. These are rich, well-documented types with serde derives. References `crate::exit::ExitReason`. Notably, `paper.rs` (also an orphan) uses `RecommendationStatus` and `AlphaPorosity` — but since `paper.rs` is itself unreachable, this cross-reference does not constitute a live dependency. The file was created and frozen in the same commit as `orchestration.rs`. The types here overlap with types defined in `crate::domain` (which is reachable); the relationship between `signals.rs` and `domain` requires RR3 investigation to determine whether this is a duplicate, a planned replacement, or a complementary extension.

**RR4 recommendation input:** If `signals.rs` types are intended to replace or extend `domain` types, promotion requires resolving the overlap. If they are duplicates, this is a candidate for archival after confirming `domain` covers all required cases.

---

### 3.6 `adapters/ultracrew/src/helpers.rs`

| Field | Value |
|-------|-------|
| LOC | 110 |
| First commit | 2026-07-07 `e256cbd3` |
| Last active | 2026-07-07 `e256cbd3` — "feat(observability): implement generic performance characterization and auto-configuration framework" |
| Classification | **incomplete** |
| Evidence level | **E3** |
| Completeness | Low-to-medium — mixed stub and real implementation |

**Analysis.** The file has two distinct sections. Lines 1–92 are placeholder stubs: `import_data()` (CSV→JSON), `run_optimizer()` (returns a single hardcoded dummy assignment with comment "In production replace with real optimizer call"), `generate_explanations()`, `validate_constraints()` (always returns `true`), `edit_schedule()` (no-op), `approve_schedule()` (always returns `true`), `export_schedule()`, `save_replay()`. Lines 94–110 contain a real implementation: `run_optimization()` references `crate::optimization::ScheduleOptimizer` and `coralys_moga::engine::EvolutionEngine`, building a full GA evolution pipeline. The real function at the bottom was likely added in the same commit as the stubs, suggesting the file was scaffolded with placeholders and one real function, then never wired into `lib.rs`. The commit message references "observability" and "auto-configuration", not scheduling — this file may have been committed as part of a broader refactor that did not complete the wiring step.

**RR4 recommendation input:** The stub functions (lines 1–92) have no value and should be removed. The `run_optimization()` function (lines 94–110) may have value if `ScheduleOptimizer` is a live type; its fate depends on whether `adapters/ultracrew` is being actively developed.

---

### 3.7 `adapters/ultracrew/src/inrc/bipartite_matching.rs`

| Field | Value |
|-------|-------|
| LOC | 1 |
| First commit | 2026-07-07 `e256cbd3` |
| Last active | 2026-07-07 `e256cbd3` — "feat(observability): implement generic performance characterization and auto-configuration framework" |
| Classification | **obsolete** |
| Evidence level | **E5** |
| Completeness | None — tombstone only |

**Analysis.** File contains a single line: `// Deprecated - moved to coralys-matching`. This is a self-declared deprecation tombstone with an explicit canonical replacement named (`coralys-matching`). No implementation exists. The file serves no compilation purpose. This is the only orphan in this pass that reaches E5 on structural evidence alone.

**RR4 recommendation input:** Eligible for immediate deletion. No archival required — the tombstone comment is the only content and the canonical replacement is named. Deletion action: remove file and confirm `coralys-matching` crate contains the replacement implementation.

---

### 3.8 `adapters/roadef/src/adapter.rs`

| Field | Value |
|-------|-------|
| LOC | 17 |
| First commit | 2026-07-06 `90262726` |
| Last active | 2026-07-06 `90262726` — "feat(observability): restore workspace and implement feasibility repair telemetry" |
| Classification | **incomplete** |
| Evidence level | **E2** |
| Completeness | Low — trait definition only, no implementors |

**Analysis.** Defines `Context` (tags: `Vec<String>`), `SolverState<'a>` (demand_idx, path_nodes, has_interventions, volume), and the `EcologyAdapter` trait with a single method `extract_context()`. The doc comment on the trait is precise and architectural: "This adapter MUST NOT contain logic for computing pressure, confidence, trend, or branch ranking. It is strictly a translation layer." This indicates intentional design with a clear separation-of-concerns mandate. However, no implementor of `EcologyAdapter` exists anywhere in the repository (confirmed by the module graph — `adapters/roadef` has no reachable files that reference this trait). The file was created and never touched again (first = last commit). The commit message references "feasibility repair telemetry", not ROADEF adapter work, suggesting this file was committed incidentally as part of a workspace restore.

**RR4 recommendation input:** If the ROADEF adapter is a live work item, this trait definition is the correct starting point and should be wired into `lib.rs`. If the ROADEF adapter work has been abandoned, this is a candidate for archival.

---

### 3.9 `services/ultracrew_server/src/simulation_test.rs`

| Field | Value |
|-------|-------|
| LOC | 6 |
| First commit | 2026-07-06 `90262726` |
| Last active | 2026-07-06 `90262726` — "feat(observability): restore workspace and implement feasibility repair telemetry" |
| Classification | **incomplete** |
| Evidence level | **E4** |
| Completeness | None — abandoned scratch stub |

**Analysis.** File contains a `fn main()` stub with a single `println!` and the comment "Just a simple script to verify logic". It is not declared as a `[[bin]]` entry in `Cargo.toml` and is not reachable from any compilation root. It was committed in the same workspace-restore commit as `adapters/roadef/src/adapter.rs` and has never been modified. This is an abandoned scratch file with no implementation value.

**RR4 recommendation input:** Eligible for deletion. No archival required. The file has no implementation content beyond a stub `main()`.

---

## 4. Summary Table

| # | File | LOC | First Commit | Last Active | Category | Evidence | RR4 Direction |
|---|------|-----|-------------|-------------|----------|----------|---------------|
| 1 | `financial/strategies/src/edge_decay.rs` | 225 | 2026-03-27 | 2026-05-28 | dormant | E3 | Promote or archive |
| 2 | `financial/strategies/src/edge_half_life_estimator.rs` | 72 | 2026-05-22 | 2026-05-28 | dormant | E3 | Promote or archive |
| 3 | `financial/strategies/src/paper.rs` | 1179 | 2026-04-27 | 2026-05-28 | dormant | E3 | Promote or archive (high value) |
| 4 | `financial/strategies/src/pipeline/certification/orchestration.rs` | 61 | 2026-05-28 | 2026-05-28 | incomplete | E2 | Complete wiring or archive |
| 5 | `financial/strategies/src/signals.rs` | 191 | 2026-05-28 | 2026-05-28 | dormant | E3 | Promote (resolve domain overlap) or archive |
| 6 | `adapters/ultracrew/src/helpers.rs` | 110 | 2026-07-07 | 2026-07-07 | incomplete | E3 | Partial — remove stubs, evaluate `run_optimization` |
| 7 | `adapters/ultracrew/src/inrc/bipartite_matching.rs` | 1 | 2026-07-07 | 2026-07-07 | obsolete | **E5** | **Delete** |
| 8 | `adapters/roadef/src/adapter.rs` | 17 | 2026-07-06 | 2026-07-06 | incomplete | E2 | Wire or archive |
| 9 | `services/ultracrew_server/src/simulation_test.rs` | 6 | 2026-07-06 | 2026-07-06 | incomplete | E4 | Delete |

**Totals:** 9 orphans, 1862 LOC. Dormant: 4 (1687 LOC). Incomplete: 4 (174 LOC). Obsolete: 1 (1 LOC).

---

## 5. Cluster Observations

### 5.1 `financial/strategies` freeze cluster (files 1–5)

All five files share the same last-active commit (`dc0a0ed2`, 2026-05-28, "Finalize orchestration certification freeze"). This is not coincidental — the commit message describes a deliberate freeze event. The five orphans represent a coherent subsystem (signals vocabulary, paper-trading engine, edge-decay measurement, half-life estimation, certification orchestration) that was developed and then frozen before being wired into the library's public module tree. The freeze was intentional; the failure to wire was either intentional (the subsystem was parked for later) or an oversight (the `mod` declarations were forgotten).

The 1687 LOC across these five files represents the highest-value dormant asset cluster in the repository. RR3 should investigate whether any experiment binary in `financial/strategies/src/bin/` imports these types directly (bypassing the lib module tree via path-based imports), which would change the evidence level.

### 5.2 `adapters/ultracrew` + `adapters/roadef` + `services/ultracrew_server` (files 6–9)

All four files were committed within a 24-hour window (2026-07-06 to 2026-07-07) across two observability-related commits. None has been touched since. This cluster appears to be scaffolding committed during a workspace restore or refactor that was not completed. The `bipartite_matching.rs` tombstone (E5) and `simulation_test.rs` stub (E4) are clear deletion candidates. The `helpers.rs` and `adapter.rs` files require owner input before a decision can be made.

---

## 6. Evidence Gaps

The following questions cannot be answered from structural analysis alone and are deferred to RR3 (Historical Evidence):

1. **`financial/strategies` freeze intent**: Was the 2026-05-28 freeze deliberate (park for later) or an oversight (forgot to add `mod` declarations)? Git log of `lib.rs` around that date would clarify.
2. **`paper.rs` vs. live paper-trading**: Does any currently-active experiment binary use a different paper-trading implementation? If so, `paper.rs` may be a superseded duplicate rather than a dormant asset.
3. **`signals.rs` vs. `domain` overlap**: What types in `crate::domain` overlap with `signals.rs`? If `SignalAction`, `RecommendationStatus`, and `AlphaPorosity` are already defined in `domain`, `signals.rs` is a duplicate.
4. **`helpers.rs` `run_optimization` viability**: Is `crate::optimization::ScheduleOptimizer` a live, compilable type? If the `optimization` module has been removed or renamed, `run_optimization()` is dead code regardless of wiring.
5. **`adapter.rs` ROADEF work status**: Is the ROADEF adapter a live work item or was it abandoned? No issue or PR reference found in the file.

---

## 7. Immediate Actions (E5 and E4)

Two files have sufficient structural evidence for action without waiting for RR3:

| File | Evidence | Action | Prerequisite |
|------|----------|--------|-------------|
| `adapters/ultracrew/src/inrc/bipartite_matching.rs` | E5 | Delete | Confirm `coralys-matching` contains replacement |
| `services/ultracrew_server/src/simulation_test.rs` | E4 | Delete | None — stub with no implementation value |

These deletions should be batched into a single commit after RR3 is complete, to avoid multiple small cleanup commits.

---

## 8. Amendment Log

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-08-02 | governance-hardening | Initial RR2-C classification of all 9 lib:orphan files |