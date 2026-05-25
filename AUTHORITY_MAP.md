# AUTHORITY_MAP.md
# ChronoSentiment — Canonical Authority Registry
# Last updated: 2026-05-25
# Purpose: Single source of truth for semantic ownership.
#          Every domain has exactly one canonical authority.
#          All other implementations are either consumers or research artifacts.

---

## OPERATIONAL AUTHORITY SURFACES

Stability classes:

| Class | Meaning |
|-------|---------|
| `CRITICAL` | Determinism / identity / certification — changes require full replay equivalence proof |
| `STABLE` | Operational and correct — evolvable with test coverage |
| `TRANSITIONAL` | Planned consolidation — current state is a known violation |
| `EXPERIMENTAL` | Non-authoritative — research or exploratory |
| `DEPRECATED` | Pending removal — do not add new callers |

| Domain | Canonical Authority | Stability Class | Notes |
|--------|--------------------|-----------------|----|
| Strategy genome definition | `core/src/ga.rs` — `struct Strategy` | `CRITICAL` | 18-gene struct including Phase D.1.21 extensions |
| Strategy genome parsing | `core/src/strategy_id.rs` **(TO BE CREATED)** | `CRITICAL` domain / `TRANSITIONAL` impl | Currently split: `edge_decay.rs:36` + `strategy_id_parse.rs:6` — **LATENT DETERMINISM RISK** — domain is CRITICAL; implementation is TRANSITIONAL until `strategy_id.rs` exists |
| GA fitness computation | `core/src/ga.rs` — `evaluate_strategy()` | `CRITICAL` | Blanket `#![allow(...)]` must be removed before this is auditable |
| Regime classification | `core/src/ga.rs` — `MarketRegime` enum | `CRITICAL` | |
| Execution simulation | `core/src/ese.rs` — `run_simulation_with_data()` | `CRITICAL` | Protected by microstructure test harness — DO NOT MODIFY without failing tests |
| Replay harness | `core/src/harness.rs` — `run_simulation_harness()` | `CRITICAL` | BLAKE3 state hashing |
| Price scaling (SSOT) | `core/src/lib.rs:83` — `pub const PRICE_SCALE: u64` | `CRITICAL` | Shadow exists in `market_adapter.rs:63` — **MUST BE REMOVED** |
| Morphology metrics | `core/src/morphology.rs` — `generate_occupancy_traces()` | `STABLE` | Deterministic LCG-based |
| Topology transforms | `core/src/topology.rs` — `TopologyField::apply()` | `STABLE` | Deterministic |
| Chronology memory physics | `core/src/cognition.rs` — `MemoryState` | `STABLE` | Deterministic |
| Synthetic scenario generation | `core/src/synthetic.rs` — `generate_deterministic_scenarios()` | `STABLE` | LCG-based, deterministic |
| Candle-to-event conversion | `core/src/market_adapter.rs` — `convert_series_to_events()` | `STABLE` | |
| Binance event ingestion | `core/src/binance_adapter.rs` — `load_binance_events_from_jsonl()` | `STABLE` | |
| Tick replay engine | `core/src/tick_replay.rs` — `TickReplayEngine` | `STABLE` | |
| Edge half-life estimation | `core/src/edge_half_life_estimator.rs` — `EdgeHalfLifeEstimator` | `STABLE` | |
| Strategy ranking (live) | `core/src/strategy_ranking.rs` — `LiveEvaluator` | `STABLE` | |
| Ensemble consensus | `core/src/ensemble.rs` — `calculate_member_weight()` | `TRANSITIONAL` | Audit found `calculate_member_weight()` uncalled — ensemble weighting implemented elsewhere; consolidation required |
| Exit evaluation | `core/src/exit.rs` — `ExitEvaluator` | `TRANSITIONAL` | `ExitEvaluator` path appears unused; only exit enums/reasons are live — evaluator path must be wired or removed |
| PnL overlay | `core/src/pnl_overlay.rs` — `run_pnl_overlay_with_config()` | `STABLE` | |
| Replay evaluation | `core/src/replay_evaluator.rs` — `run_replay_with_evaluator()` | `STABLE` | |
| Selection cap / Top-K | `core/src/selection_cap.rs` | `STABLE` | Env-var driven |
| Frozen substrate loading | `cs-ingest/src/frozen_loader.rs` — `load_frozen_cohort()` | `STABLE` | |
| Timeline alignment | `cs-ingest/src/timeline.rs` — `align_timeline()` | `STABLE` | SHA256 fingerprint matches Python |
| Chronology repair | `cs-ingest/src/repair.rs` — state machine | `TRANSITIONAL` | Audit found state machine jumps PENDING→terminal states; `FETCHED` and `VERIFIED_TS_MATCH` intermediate states not confirmed persisted — verify or correct |
| Ingest replay pipeline | `cs-ingest/src/replay.rs` — `run_replay_step()` | `STABLE` | |
| API event signatures | `services/api/src/signatures.rs` | `STABLE` | BLAKE3, sole signature authority |
| Manifest assembly | `scripts/emit_manifest_v1.py` | `STABLE TOOLING` | Certification tooling — downstream consumer of Rust outputs; **not a Rust authority surface** |
| Equivalence certification | `scripts/certify_equivalence_v1.py` | `STABLE TOOLING` | Certification tooling — orchestrates `cargo run --bin trace_replay`; **not a Rust authority surface** |
| Edge decay analysis | `core/src/edge_decay.rs` | `TRANSITIONAL` | Contains local parser copy — consolidate into `strategy_id.rs` (V-001) |
| Observatory artifact generation | `core/src/bin/trace_replay.rs` | `TRANSITIONAL` | Hardcoded `commit_hash` placeholder (V-010); also owns hardcoded topology/cognition parsing — multiple placeholder bindings, not just `commit_hash` |
| API error handling | `services/api/src/errors.rs` — `ApiError` (Axum) | `TRANSITIONAL` | Duplicate exists in `lib.rs:55` — **MUST BE REMOVED** (V-003) |
| `kernel::run_ga()` | `core/src/kernel.rs` | `DEPRECATED` | Stub returning hardcoded strings — re-exported from `lib.rs` (V-004) |
| `core/src/live_source.rs` | *(uncompiled)* | `EXPERIMENTAL` | Not in module tree — wire, archive, or delete |
| `core/src/data_source/python.rs` | *(uncompiled)* | `EXPERIMENTAL` | Not in module tree — wire, archive, or delete |
| `core/src/data_source/yahoo.rs` | *(uncompiled)* | `EXPERIMENTAL` | Not in module tree — wire, archive, or delete |

---

## KNOWN AUTHORITY VIOLATIONS (Pending Remediation)

| ID | Violation | Location | Risk | Remediation Phase |
|----|-----------|----------|------|-------------------|
| V-001 | Duplicate strategy ID parser with `entry_offset` index shift | `edge_decay.rs:36` vs `strategy_id_parse.rs:6` | **LATENT DETERMINISM RISK** — silent genome corruption on cross-parser round-trips | Phase 3 |
| V-002 | `PRICE_SCALE: f64` shadows canonical `PRICE_SCALE: u64` | `market_adapter.rs:63` | Type divergence — future value changes won't propagate | Phase 3 |
| V-003 | Dual `ApiError` definitions with variant mismatch (`InvalidInput` vs `ValidationError`) | `lib.rs:55` vs `errors.rs:5` | `simulate.rs` uses non-Axum error type — cannot be wired as Axum handler | Phase 3 |
| V-004 | `kernel::run_ga()` stub with hardcoded strings, re-exported from `lib.rs` | `kernel.rs:3` | Creates false authority signal | Phase 2 |
| V-005 | Blanket `#![allow(dead_code, unused_variables, ...)]` in `ga.rs` | `ga.rs:1` | Compiler cannot detect dead branches, orphaned optimization paths, or stale fitness logic | **Phase 1** |
| V-006 | `NormalizedTick`/`CaptureGap`/`CaptureManifest` triplicated across bin files with schema drift | `capture_daemon.rs`, `historical_importer.rs`, `yahoo_importer.rs` | Schema drift already present in `yahoo_importer` variant | Phase 2 |
| V-007 | `OrderState`/`PortfolioState`/`SystemState` duplicated with type divergence (`i32` vs `u64`/`f64`) | `replay.rs:16` vs `dto.rs:465` | Naming collision creates authority confusion | Phase 3 |
| V-008 | Hardcoded absolute path `/Users/nikhil/ChronoSentiment_MEGA_FINAL/test_assets` | `simulate.rs:32` | Fails on any machine other than developer's local | Phase 3 |
| V-009 | Hardcoded `config_hash: "default-config-hash"` in certification fingerprint | `certify.rs:64` | Fingerprint not bound to simulation config — undermines certification guarantee | Phase 3 |
| V-010 | Hardcoded `commit_hash: "canonical"` in observatory manifest | `trace_replay.rs:130` | Artifact not reproducibility-bound | Phase 3 |

---

## UNCOMPILED FILES (Not in Module Tree)

These files exist on disk but are **never compiled**. They have no operational authority.
Leaving them in place creates phantom authority signals and onboarding ambiguity.

| File | Reason Not Compiled | Required Disposition |
|------|--------------------|--------------------|
| `core/src/live_source.rs` | Not declared in `lib.rs` | Wire intentionally, move to `/research_experiments/live_source_v0/`, or delete |
| `core/src/data_source/python.rs` | Not declared in `data_source.rs` | Wire intentionally, move to `/research_experiments/python_bridge/`, or delete |
| `core/src/data_source/yahoo.rs` | Not declared in `data_source.rs` | Wire intentionally, move to `/research_experiments/yahoo_adapter_v0/`, or delete |

**Rule:** "Leave ambiguous" is not a valid disposition. Every file must be either compiled or explicitly archived.

---

## RESEARCH / NON-AUTHORITATIVE SURFACES

These are downstream consumers, research experiments, or exploratory scaffolding.
They do **NOT** hold operational authority over any domain listed above.
They are preserved for lineage and potential future canonicalization.

| Path | Classification | Relationship to Core |
|------|---------------|----------------------|
| `scripts/research/policy_competition_engine.py` | Research — multi-agent policy engine | Downstream consumer of GA outputs; duplicates risk scoring formula `(0.5*h_accel + 0.3*e_decay + 0.2*c_vel)` |
| `scripts/research/adaptive_participation_layer.py` | Research — position sizing engine | Downstream consumer; duplicates execution policy semantics |
| `scripts/research/counterfactual_replay.py` | Research — risk scoring | Downstream consumer; duplicates risk formula |
| `scripts/research/train_ranking_model.py` | Research — Python scoring grid search | Downstream consumer; duplicates optimization semantics |
| `scripts/research/controlled_ablation_harness.py` | Research — survival gain scoring | Downstream consumer; duplicates survival gain formula `1.0 - (exposure / baseline_exposure)` |
| `scripts/research/robustness_experiments.py` | Research — survival gain metrics | Downstream consumer |
| `scripts/signal_physics_harness.py` | Research — signal generation | DRIFT-1: reimplements `generate_intent()` signal logic |
| `scripts/adversarial_physics_test.py` | Research — admissibility classification | DRIFT-3: triplicates admissibility thresholds `(0.9, 0.5, 0.1)` |
| `scripts/synthetic_fragmentation_injector.py` | Research — admissibility | DRIFT-4: second triplication of admissibility |
| `scripts/survivability_surface_builder.py` | Research — economic divergence metrics | DRIFT-11: economic divergence metrics |
| `core/src/data_source/python.rs` | Uncompiled research artifact | Python bridge experiment — not in module tree |
| `core/src/data_source/yahoo.rs` | Uncompiled research artifact | Pre-`yahoo_importer.rs` adapter — not in module tree |
| `core/src/live_source.rs` | Uncompiled research artifact | Live source experiment — not in module tree |

---

## REMEDIATION PHASES

### Phase 1 — Visibility Recovery *(unblocks all subsequent audits)*

**STATUS: COMPLETE — 2026-05-25 — commit `881f4141`, tags `phase1-governance-visibility-restored` + `replay-governance-baseline-v1`**
**Canonical record: [`docs/PHASE1_GOVERNANCE_CHECKPOINT.md`](docs/PHASE1_GOVERNANCE_CHECKPOINT.md)**
**Figures: 3 blanket suppressors removed · 7 warnings surfaced · 11 items annotated · 0 deleted · `cargo check` exit 0**

- [x] Remove `#![allow(dead_code)]` from `core/src/ga.rs` — **one suppression at a time**
- [x] Run `cargo check 2>&1 | grep "ga.rs"` after each removal
- [x] Classify each warning: fix it, or add targeted `#[allow(dead_code)] // REASON: ...` on the specific item
- [x] Never restore the file-level blanket suppressor
- [x] Removal order: `unused_imports` → `dead_code` → `unused_variables` → `unreachable_code` → remaining

### Phase 2 — Module Tree Cleanup

- [ ] Resolve `live_source.rs`, `data_source/python.rs`, `data_source/yahoo.rs` (wire, archive, or delete)
- [ ] Remove or implement `kernel.rs` stub (`run_ga()` returns hardcoded strings)
- [ ] Remove stale comments: `main.rs:1` (`// mod core;`), `reco.rs:4` (`// use crate::{MarketEvent, Side}`)
- [ ] Create `/research_experiments/` with subdirs: `live_source_v0/`, `python_bridge/`, `yahoo_adapter_v0/`
- [ ] Create `/docs/research_archive/governance_audit_2026-05-25.md`
- [ ] Extract shared capture types from bin files into `core/src/capture_types.rs`

### Phase 3 — Identity-Critical Consolidation

- [ ] Create `core/src/strategy_id.rs` with canonical `parse_strategy_id(id: &str) -> Result<Strategy, StrategyIdError>`
- [ ] Decide canonical schema: does `entry_offset` belong at position 13?
- [ ] Export from `core/src/lib.rs`, replace both local parsers (`edge_decay.rs:36` and `strategy_id_parse.rs:6`)
- [ ] Add round-trip test: `serialize(parse(id)) == id`
- [ ] Add `InvalidInput(String)` variant to `errors.rs::ApiError`
- [ ] Remove `ApiError` from `lib.rs`
- [ ] Update `simulate.rs`, `certify.rs`, `events.rs`, `timeline.rs` to use `errors.rs::ApiError`
- [ ] Replace `PRICE_SCALE: f64` in `market_adapter.rs:63` with `crate::PRICE_SCALE as f64`
- [ ] Replace hardcoded path in `simulate.rs:32` with `std::env::var("TEST_ASSETS_PATH")`
- [ ] Replace hardcoded hashes in `certify.rs:64` and `trace_replay.rs:130`
- [ ] Rename `replay.rs` internal types: `InternalOrderState`, `InternalPortfolioState`, `InternalSystemState`
- [ ] Remove `RunGaResponse.final_gen_best` and `.generation_found` (always duplicate other fields)

### Phase 4 — Research Quarantine

- [ ] Move Python research scripts to `/research_experiments/`
- [ ] Create `RESEARCH_LINEAGE.md` documenting relationship to Rust authority surface
- [ ] Do **not** delete — preserve for reference and potential future canonicalization
- [ ] Update this `AUTHORITY_MAP.md` to reflect final dispositions

---

## GOVERNANCE RULES

1. **Every domain has exactly one canonical authority.** If you are implementing something that already appears in this map, you are either extending the canonical authority (update this map) or creating a research artifact (document it in the non-authoritative section).

2. **Uncompiled files are not authoritative.** A file that is not in the module tree has no operational authority regardless of its content.

3. **Parser drift is determinism corruption.** Any code that parses serialized genome identity must use the canonical parser in `core/src/strategy_id.rs`. No local copies.

4. **The `ga.rs` suppressor must not be restored.** Compiler visibility over the optimization engine is non-negotiable. Removal of `#![allow(...)]` from `ga.rs` is **Phase 1 gate zero** — no subsequent remediation phase may begin until this is complete and the suppressor is permanently prohibited by this rule (V-005).

5. **Research artifacts preserve lineage without granting authority.** Moving a file to `/research_experiments/` does not delete its history — it clarifies its status.

6. **`CRITICAL domain / TRANSITIONAL implementation` is a distinct recognized state.** A domain may be CRITICAL (determinism-sensitive) while its current implementation is TRANSITIONAL (split, placeholder-bound, or pre-consolidation). These are not contradictory — they mean the domain requires replay equivalence proof on consolidation, not that the current split is acceptable indefinitely.

7. **Certification tooling is not a Rust authority surface.** Scripts classified as `STABLE TOOLING` (e.g. `emit_manifest_v1.py`, `certify_equivalence_v1.py`) are acceptable consumers and orchestrators. They do not hold authority over any domain in this map and must not be treated as canonical implementations.

8. **Semantic changes to CRITICAL surfaces require replay equivalence scope declaration.** Any change touching a `CRITICAL` authority surface must declare before merge: (a) expected replay impact, (b) equivalence expectations, (c) affected certification surfaces, (d) whether prior artifacts remain canonical, (e) whether replay hashes are expected to change. "No replay impact expected" is itself a claim requiring validation — it is not a default exemption.