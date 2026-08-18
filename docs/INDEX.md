# Canonical Repository Index

**Document ID:** GOV-IDX-001
**Version:** 1.72
**Status:** Active
**Created:** 2026-08-01

---

## Purpose

This is the single authoritative entry point for the repository. Every canonical artifact has exactly one entry here. If a document, crate, experiment, or evidence asset is not listed in this index, it is not canonical.

**Rule:** Before creating any new document, add it here first. If it cannot be added here, it should not be created.

---

## Current Status

| Area | Status | Last Updated |
|------|--------|-------------|
| Section 2 — Pairing Topology Mutation Evaluation | **FROZEN** 2026-08-01 | 2026-08-01 |
| Section 3 — Coralys Native Scheduler | Active — Experiment 0 spec complete | 2026-08-01 |
| Experiment Harness | Stable — all 5 modules complete | 2026-08-01 |
| GERAD Coralys v1.0 Baseline | **FROZEN** 2026-08-01 | 2026-08-01 |
| Repository Governance | Active — GOV-001 complete, GOV-002 this document | 2026-08-01 |
| CDI MVP v0.1 — Decision Server | **FROZEN BASELINE 2026-08-18** — 101-ticker universe; RecommendationEngine v1 OPERATIONAL; `/latest` deduplicates by ticker (newest wins); `/history` returns all observations; evaluated=101, Buy=14, Watch=46, NoTrade=41; 34/34 tests green | 2026-08-18 |
| CDI MVP v0.1 — Algorithm FROZEN | **ALGORITHM FROZEN 2026-08-18** — policy reconciliation complete; dormant SELL branch added (SHORT+Favourable→SELL; 0 SELLs at baseline); corrected policy semantics in REC-BASELINE-001-RECONCILIATION.md; 60/60 tests pass; no further algorithm changes until prospective evidence accumulates | 2026-08-18 |
| REC-001 — Recommendation Engine Validation | **Active — prospective observation phase** (REC-001-H COMPLETE — 101 tickers, 121,805 records; v1 engine live; G2 proven; `/latest` = one per ticker; `/history` = all observations; next: prospective observation recorder) | 2026-08-18 |
| Prospective Observation Recorder | **NEXT MILESTONE** — record T0 snapshot (ticker, state, action, target, risk, horizon, analogue_n, target_rate, score) + T+h outcome (MFE, MAE, target_reached, risk_reached, first_exit, sessions_to_outcome, outcome); observation_status=OPEN/CLOSED/INCOMPLETE; immutable T0 snapshot — never overwrite; accumulate evidence without contaminating recommendation with future information | 2026-08-18 |
| REC-001-H Evidence Quality | **COMPLETE 2026-08-18** — evidence_quality_report.csv written; C3-002 mapping verified (Bear+Neg→LONG); LONG min-bucket median=170; SHORT min-bucket median=187; LONG target rate 29.6% mean | 2026-08-18 |
| RecommendationEngine v1 | **COMPLETE 2026-08-18** — Coralys-native; analogue-population-based; adaptive R:R + horizon; first-exit semantics; graceful degradation (Exact→RelaxVol→RelaxBoth→StateOnly→NO_TRADE); see ARCH-006 | 2026-08-18 |
| HDV-001 — Historical Decision Validation | **FROZEN** 2026-08-17 — all gates PASS | 2026-08-17 |
| HDV-002 — Risk-Boundary Research | **FROZEN methodology** / validation accumulation active — opens 2026-08-18; independent of REC-001 | 2026-08-18 |
| 102-stock universe (UNIV-001) | **Versioned / active** — `datasets/universes/coralys_102_v2.json` (ACTIVE); v1 frozen/superseded; v2 replaces MCDOWELL-N.NS with UBL.NS; 102 valid tickers, 0 unavailable | 2026-08-18 |
| Recommendation evidence | **REC-001-H ticker-specific (COMPLETE)** — 101 tickers, 121,805 records; `datasets/recommendation/historical/TICKER_NS.jsonl`; leakage-free | 2026-08-18 |
| Recommendation ranking | **v1 operational** — adaptive target from 25th-pct MFE; adaptive risk from median MAE; adaptive horizon from median sessions_to_outcome; ticker-specific analogue population | 2026-08-18 |
| Volume enrichment | **Data capture permitted** — relative_volume_20 stored in REC-001-H; volume regime used in analogue matching (LOW/NORMAL/HIGH); recommendation use not yet prospectively validated | 2026-08-18 |
| Adaptive R:R | **OPERATIONAL in v1** — derived from first-exit analogue population; requires reference_price from prospective pipeline for absolute price targets | 2026-08-18 |
| chrono-ui — Trading MVP | Active — Next.js 15, port 3000; v1 endpoint `/api/recommendations/v1/latest`; adaptive geometry + degradation badges displayed | 2026-08-18 |
| hdv001-dashboard — Evidence Dashboard | **FROZEN** 2026-08-17 — Vite/React, port 5174, read-only | 2026-08-17 |
| emit_prospective_to_server.py | **Transitional tool 2026-08-18** — `scripts/emit_prospective_to_server.py`; re-emits an already-sealed prospective ledger to the Decision Server without re-fetching Yahoo data; idempotent; canonical path is `csp006_p_prospective --emit-url` which fetches fresh data and emits in one pass | 2026-08-18 |
| Yahoo incremental fetch | **COMPLETE 2026-08-18** — `adapters/chronosentiment/src/ingestion/yahoo.rs`; `fetch_historical` now checks local cache first, fetches only missing ticks (period1=last_ts+1, period2=now+1d), merges+dedupes+sorts, writes back; no full re-fetch when cache is warm | 2026-08-18 |

---

## Frozen Documents

Documents in this section must not be modified except for typographical corrections approved by the reviewer.

| ID | Path | Frozen Date | Modification Policy |
|----|------|-------------|---------------------|
| UC-R-001 §2 | `docs/research/UltraCrew_Pairing_Topology_Mutation_Evaluation.md` (Section 2) | 2026-08-01 | Numerical corrections and typographical fixes only; requires reviewer approval + version increment |
| BENCH-001 | `adapters/airline/tests/gerad_coralys.rs` | 2026-08-01 | No modifications; improvements go in `gerad_coralys_v2.rs` |

---

## Canonical Documents

### 1. Governance

| ID | Path | Title | Role |
|----|------|-------|------|
| GOV-KS-001 | `docs/governance/KNOWLEDGE_SURVEY.md` | Repository Knowledge Survey | Full typed asset inventory — authoritative |
| GOV-IDX-001 | `docs/INDEX.md` | Canonical Repository Index | This document — entry point for all canonical assets |
| GOV-DEP-001 | `docs/governance/DEPENDENCY_GRAPH.md` | Repository Dependency Graph | Typed dependency graph across all knowledge systems |
| GOV-CLN-001 | `docs/governance/CLEANUP_REGISTER.md` | Cleanup Register | Tracks every consolidation, merge, archival, deletion |

### 2. Research Programme

| ID | Path | Title | Status |
|----|------|-------|--------|
| UC-R-001 | `docs/research/UltraCrew_Pairing_Topology_Mutation_Evaluation.md` | Pairing Topology Mutation Evaluation | Section 2 FROZEN; Section 3 active |
| UC-R-002 | `docs/research/UltraCrew_Coralys_Native_Scheduler_Section3.md` | Coralys Native Scheduler — Section 3 | Active — Experiment 0 spec complete |
| UC-R-003 | `docs/research/UltraCrew_GENCOL_Pipeline_Divergence_Analysis.md` | GENCOL Pipeline Divergence Analysis | Active |
| UC-R-004 | `docs/research/UltraCrew_Layover_Threshold_Experiment.md` | Layover Threshold Experiment | Active |
| UC-R-005 | `docs/research/UltraCrew_Objective_Function_Alignment.md` | Objective Function Alignment | Active |

### 3. Experiment Infrastructure

| ID | Path | Title | Status |
|----|------|-------|--------|
| HARNESS-001 | `adapters/airline/tests/harness/schema.rs` | Experiment Schema | Stable |
| HARNESS-002 | `adapters/airline/tests/harness/logging.rs` | Event Logger | Stable |
| HARNESS-003 | `adapters/airline/tests/harness/persistence.rs` | Result Persistence | Stable |
| HARNESS-004 | `adapters/airline/tests/harness/reproducibility.rs` | Reproducibility Info | Stable |
| HARNESS-005 | `adapters/airline/tests/harness/report.rs` | Report Generator | Stable |
| HARNESS-006 | `adapters/airline/tests/harness/mod.rs` | Harness Root | Stable |

### 4. Benchmarks

| ID | Path | Title | Status |
|----|------|-------|--------|
| BENCH-001 | `adapters/airline/tests/gerad_coralys.rs` | GERAD Coralys v1.0 Baseline | **FROZEN** 2026-08-01 |
| BENCH-002 | `adapters/airline/tests/gerad_e2e.rs` | GERAD End-to-End | Active |
| BENCH-003 | `adapters/airline/tests/benchmark.rs` | General Benchmark Suite | Active |

### 5. Platform Crates

| ID | Path | Role |
|----|------|------|
| CRATE-001 | `coralys-core/` | Core EA engine |
| CRATE-002 | `coralys-decision/` | Decision layer |
| CRATE-003 | `coralys-ecology/` | Ecology / population dynamics |
| CRATE-004 | `coralys-eval/` | Fitness evaluation |
| CRATE-005 | `coralys-infrastructure/` | Shared infrastructure |
| CRATE-006 | `coralys-matching/` | Matching algorithms |
| CRATE-007 | `coralys-moga/` | Multi-objective GA |
| CRATE-008 | `coralys-planning/` | Planning layer |
| CRATE-009 | `coralys-policy/` | Policy layer |
| CRATE-010 | `coralys-recommendation/` | Recommendation engine |
| CRATE-011 | `coralys-simulation/` | Simulation layer |
| CRATE-012 | `coralys-v2/` | Coralys v2 |

### 6. Adapters

| ID | Path | Domain |
|----|------|--------|
| ADAPTER-001 | `adapters/airline/` | Airline crew pairing — canonical research adapter |
| ADAPTER-002 | `adapters/chronosentiment/` | ChronoSentiment financial signals |
| ADAPTER-003 | `adapters/roadef/` | ROADEF 2026 challenge |

### 7. Architecture

| ID | Path | Title |
|----|------|-------|
| ARCH-001 | `docs/CORALYS_PLATFORM_ARCHITECTURE.md` | Coralys Platform Architecture |
| ARCH-002 | `docs/PLATFORM_CRATE_RESPONSIBILITIES.md` | Platform Crate Responsibilities |
| ARCH-003 | `docs/Service_Boundary_Definition.md` | Service Boundary Definition |
| ARCH-004 | `docs/Event_Flow_Specification.md` | Event Flow Specification |
| ARCH-005 | `docs/ARCHITECTURE_GLOSSARY.md` | Architecture Glossary |
| ARCH-006 | `docs/ARCH-006_RECOMMENDATION_ENGINE_ARCHITECTURE.md` | Recommendation Engine Architecture — v0 vs v1 comparison; locked-in Coralys-native direction; C3-002 mapping verified; analogue-population design; graceful degradation spec; four MVP gates (G1–G4); execution order (v1 first, fetcher parallel); MarketDataFetcher refactor spec |

### 8. Evidence Governance

| ID | Path | Title |
|----|------|-------|
| STANDARD-001 | `docs/CORALYS_EVIDENCE_GOVERNANCE_STANDARD.md` | Coralys Evidence Governance Standard |
| STANDARD-002 | `docs/BENCHMARK-GOVERNANCE.md` | Benchmark Governance |
| STANDARD-003 | `docs/BENCHMARK-REFERENCE-SPECIFICATION-v1.0.md` | Benchmark Reference Specification |
| EV-GOV-001 | `docs/CHRONOSENTIMENT_EVIDENCE_PROGRAMME.md` | ChronoSentiment Evidence Programme |
| EV-GOV-002 | `docs/ULTRACREW_WORKFORCE_EVIDENCE.md` | UltraCrew Workforce Evidence |
| EV-GOV-003 | `G_GATE_v1.1_STATISTICAL_CLOSURE.md` | G-GATE v1.1 closed: B4 INCONCLUSIVE; predictive value not established; Decision A |

### 9. Product & Strategy

| ID | Path | Title |
|----|------|-------|
| BLUEPRINT-001 | `docs/ChronoSentiment_Product_Blueprint_v1.md` | ChronoSentiment Product Blueprint |
| STRATEGY-001 | `docs/CORALYS_PLATFORM_STRATEGY.md` | Coralys Platform Strategy |
| STRATEGY-002 | `docs/ChronoSentiment_Product_Strategy_v1.md` | ChronoSentiment Product Strategy |
| PRD-001 | `docs/PRD_v3_3.md` | Product Requirements Document v3.3 (superseded; historical) |
| PRD-002 | `docs/CHRONOSENTIMENT_PRD_V1.md` | ChronoSentiment Product Definition v1.0 (authoritative commercial PRD) |
| CS-P-001 | `docs/CS-P-001_DECISION_SUPPORT_PRODUCT_MODE.md` | Product mode after G-GATE v1.1 close; co-pilot / paper trading; v1.2 not opened |
| CS-P-002 | `docs/CS-P-002_DECISION_VALIDATION_PLATFORM.md` | Decision Validation Platform v1: one engine, replay/live adapters, backtest + forward paper |
| CS-P-002-R1 | `product_validation/B4_unfrozen_dev/HISTORICAL_PERFORMANCE_REPORT.md` | B4 historical product validation baseline (`unfrozen-dev`; not G-GATE; v1.0 not frozen) |
| CS-P-003 | `docs/CS-P-003_FORWARD_PAPER_VALIDATION.md` | Forward/Paper Validation v0.1: daily tick continues (confirmation, not discovery) |
| CS-P-004 | `docs/CS-P-004_HISTORICAL_RESEARCH_LABORATORY.md` | Historical Research & Robustness Laboratory: B4 reconstruction, no engine change |
| CS-P-004-R1 | `product_validation/CS-P-004_unfrozen_dev/HISTORICAL_RESEARCH_SUMMARY.md` | First B4 laboratory reports (`unfrozen-dev`; not a candidate policy; not G-GATE) |
| CS-P-004-A1 | `product_validation/CS-P-004_adapter_v0.1/HISTORICAL_RESEARCH_SUMMARY.md` | Adapter v0.1 lab re-run: unavailable confidence + preserved evidence; not a candidate policy |
| CS-P-004-E1 | `docs/CS-P-004_HISTORICAL_RESEARCH_LABORATORY.md` | Assessment Enrichment v0.1: factors at T; code + tests |
| CS-P-004-E1-S1 | `product_validation/assessment_enrichment_v0.1/` | Information-fidelity snapshot + factor availability (not B5; not a strategy experiment) |
| CS-P-TEST-001 | `docs/CS-P-TEST-001_DECISION_INTELLIGENCE_VERIFICATION_MATRIX.md` | Decision intelligence verification matrix (vision-level tests; not a snapshot) |
| CS-P-005 | `docs/CS-P-005_FACTOR_ECOLOGY_ANALYSIS.md` | Factor Ecology Analysis v0.1: states at T; no candidate policy |
| CS-P-AUDIT-001 | `docs/CS-P-AUDIT-001_ADAPTER_DISCIPLINE.md` | Read-only ChronoSentiment adapter discipline inventory (no cleanup) |
| CS-P-CLEAN-001 | `docs/CS-P-CLEAN-001_ADAPTER_QUARANTINE.md` | PR-1: research/legacy quarantine; B3/B4 generators preserved; no policy change |
| CS-P-CLEAN-002 | `docs/CS-P-CLEAN-002_EXPLICIT_POLICY_CONTRACT.md` | PR-2: explicit DecisionPolicy required; baseline fixture is not the ChronoSentiment strategy |
| CS-P-006 | `docs/CS-P-006_CORALYS_POLICY_DISCOVERY.md` | Research frozen at C.3-F; P.3–P.6 observatory slice; no Search #3 |
| CS-P-006-A | `docs/CS-P-006-A_POLICY_DISCOVERY_CONTRACT.md` | PolicyArtifact consumption contract (`csp006a.policy_artifact.1`); no search |
| CS-P-006-B | `docs/CS-P-006-B_RESEARCH_PROTOCOL.md` | Discovery protocol; S1 certified; B.1 partition frozen |
| CS-P-006-B.1 | `docs/CS-P-006-B.1_CHRONOLOGICAL_PARTITION.md` | Frozen development/selection/evaluation partition (protocol TRAIN/VAL/TEST) |
| CS-P-006-S1 | `docs/CS-P-006-S1_SEVEN_INSTRUMENT_SNAPSHOT.md` | Disposable 7-instrument research snapshot (not B4/B5); certified PASS |
| CS-P-006-V | `docs/CS-P-006-V_DECISION_VALUE.md` | Vision: prediction ≠ decision; future certified state families; not this genome |
| CS-P-006-C | `docs/CS-P-006-C_POLICY_DISCOVERY.md` | Search #1 immutable: reproducible discovery, failed generalization |
| CS-P-006-C.1 | `docs/CS-P-006-C.1_SEARCH_DIAGNOSIS.md` | Post-search diagnosis; no Search #2; TMV insufficiency not concluded |
| CS-P-006-C.2 | `docs/CS-P-006-C.2_RESEARCH_GAP_REVIEW.md` | Instrumentation & information-gap review; Search #2 not authorized |
| CS-P-006-C.2-O | `docs/CS-P-006-C.2-O_SEARCH_OBSERVABILITY.md` | Search observability; same artifact on/off; no Search #2 |
| CS-P-006-C.2-P | `docs/CS-P-006-C.2-P_POPULATION_ECOLOGY.md` | Search #1 population ecology; no Search #2 |
| CS-P-006-C.2-R | `docs/CS-P-006-C.2-R_RECOMMENDATION_OUTCOME.md` | Search #1 recommendation-vs-outcome matrix; no Search #2 |
| CS-P-006-C.2-S | `docs/CS-P-006-C.2-S_SELECTION_DECISION_VALUE.md` | Selection bottleneck + decision-value review; no Search #2 |
| CS-P-006-C.2-D | `docs/CS-P-006-C.2-D_DECISION_VALUE_LANDSCAPE.md` | Decision-value landscape of Search #1 recommendations; no Search #2 |
| CS-P-006-M | `docs/CS-P-006-M_DECISION_VALUE_MODEL.md` | Decision-value model (protocol questions); not Coralys fitness; no Search #2 |
| CS-P-006-M.1 | `docs/CS-P-006-M.1_DECISION_VALUE_SPECIFICATION.md` | Decision-value specification; continuous V; regret not fitness; no Search #2 |
| CS-P-006-N | `docs/CS-P-006-N_DECISION_VALUE_RESEARCH_HARNESS.md` | Decision-value harness; symbol matrices required; C.3 protocol opened separately |
| CS-P-006-C.3 | `docs/CS-P-006-C.3_PROTOCOL.md` | C.3 protocol authorization; Search #2 not started; same TMV |
| CS-P-006-C.3-I | `docs/CS-P-006-C.3-I_IMPLEMENTATION.md` | C.3-I implementation PASS; identity gate; Search #2 not run |
| CS-P-006-C.3-R | `docs/CS-P-006-C.3-R_SEARCH.md` | One authorized Search #2 run; Search #1 immutable; no iteration |
| CS-P-006-C.3-C | `docs/CS-P-006-C.3-C_COMPARATIVE_REVIEW.md` | Search #1 vs #2 sealed-artifact review; no Search #3 |
| CS-P-006-C.3-D | `docs/CS-P-006-C.3-D_RULE_ECOLOGY.md` | Search #2 live-rule ecology; candidate not promoted; no Search #3 |
| CS-P-006-C.3-E | `docs/CS-P-006-C.3-E_RULE_PERSISTENCE.md` | Search #2 discovered-rule persistence; no pass threshold; no Search #3 |
| CS-P-006-C.3-F | `docs/CS-P-006-C.3-F_STATE_ACTION_LANDSCAPE.md` | Certified TMV state x action value landscape; frozen conclusion; no Search #3 |
| CS-P-006-C.3-G | `docs/CS-P-006-C.3-G_REGIME_PERSISTENCE_QUESTION.md` | Regime-persistence question only; research loop stopped; no Search #3 |
| CS-P-006-P | `docs/CS-P-006-P_DECISION_OBSERVATORY.md` | Decision Observatory; evidence dashboard; no early peek; C3-002 paper-only |
| CS-P-006-P.H | `docs/CS-P-006-P.H_HISTORICAL_REPLAY.md` | Historical Observatory Replay; same engine/policy; no lookahead; not C.3-G |
| CS-P-006-P.H.1 | `docs/CS-P-006-P.H.1_DECISION_EVIDENCE_ENGINE.md` | Decision Evidence Engine; Replay v0 20 calendar days archived; not a statistical backtest |
| CS-P-006-P.H.2 | `docs/CS-P-006-P.H.2_MARKET_SESSION_HORIZON.md` | Observatory 20 market sessions; Replay v1; v0 not reinterpreted; not C.3-G |
| CS-P-006-P.H.3 | `docs/CS-P-006-P.H.3_DECISION_EVIDENCE_DASHBOARD.md` | Decision Evidence Dashboard; replay integrity ≠ strategy validation; no C.3-G |
| CS-P-006-P.E | `docs/CS-P-006-P.E_TARGETED_DECISION_EXECUTION.md` | Targeted execution contract; target sealed at T; OHLC first-exit; not C.3-G |
| CS-P-006-P.E.1 | `docs/CS-P-006-P.E.1_EXECUTION_EVIDENCE_SURFACE.md` | Decision / Execution / Evidence layers; Execution Contract v0 owns target_pct; frozen; not C.3-G |
| CS-P-006-P.E.2 | `docs/CS-P-006-P.E.2_LIVE_EXECUTION_OBSERVATION.md` | Frozen prospective lifecycle with fixed Execution Contract v0; not a 5% quality test; not C.3-G |
| CS-P-006-P.E.2.H | `docs/CS-P-006-P.E.2.H_HISTORICAL_LIFECYCLE_VALIDATION.md` | Historical P.E.2 lifecycle validation — PASS; live P.E.2 remains AWAITING_NEXT_SESSION; not a statistical backtest |
| CS-P-006-P.E.B | `docs/CS-P-006-P.E.B_CORALYS_TARGET_FROM_STATE.md` | Pointer — Coralys target discovery is CS-P-006-P.E.3; no Search #3 |
| CS-P-006-P.E.3 | `docs/CS-P-006-P.E.3_CORALYS_TARGET_DISCOVERY.md` | Coralys target from state at T; waits for CS-P-007; P.E.2 is the control; not started; no Search #3 |
| CS-P-006-P.E.3.A | `docs/CS-P-006-P.E.3.A_CORALYS_TARGET_ARTIFACT.md` | Coralys Target Artifact contract; no generator, ATR map, or search; waits for CS-P-007; no Search #3 |
| CS-P-007 | `docs/CS-P-007_STATISTICAL_STRATEGY_VALIDATION.md` | Statistical validation of frozen C3-002 + Execution Contract v0; specified not run; P.E.3 waits; no Search #3 |
| CDI-MVP-V01 | `docs/CORALYS_DECISION_INTELLIGENCE_MVP_V01.md` | Coralys Decision Intelligence MVP v0.1 — product specification; DecisionRecord schema; user-controlled execution; no allocation; supersedes Observatory product layer |
| CDI-MVP-V01-DEL | `docs/CORALYS_DECISION_INTELLIGENCE_MVP_V01_DELETION_LIST.md` | MVP v0.1 deletion/retirement list — Observatory implementation code to retire; evidence archives to keep; dependency grep required before any deletion |
| ROADMAP-001 | `docs/EP-002_ROADMAP.md` | Engineering Programme Roadmap |
| HDV-001 | `docs/HDV_001_PERIODS.md` | Historical Decision Validation — period definitions, price cache, path extractor, MAE/MFE, outcome classifier, baselines, freeze gate; all gates PASS; **FROZEN** 2026-08-17 |
| HDV-001-F | `datasets/hdv001/HDV_001_F_DETERMINATION.md` | HDV-001-F official criterion evaluation — Gate1 +6.0pp, Gate2 +12.4pp, Gate3 2/4 segments; PASS; **FROZEN** 2026-08-17 |
| HDV-002-A | `docs/HDV_002_METHODOLOGY.md` | HDV-002 Risk-Boundary Research — methodology freeze gate; accumulation opens 2026-08-18; **independent of REC-001**: REC-001 cannot modify C3-002 risk boundaries; HDV-002 cannot use REC-001 recommendation outcomes as an optimization feedback loop unless its methodology explicitly authorizes it |
| UNIV-001 | `datasets/universes/coralys_102_v1.json` | CDI 102-Stock Universe v1 — Nifty 100 + 2 liquid mid-caps; frozen 2026-08-18; 102 instruments; single source of truth for REC-001 v0; versioned stepping stone toward 6,800+ NSE/BSE universe | Active / Frozen for REC-001 v0 |
| REC-001 | `docs/REC-001_RECOMMENDATION_VALIDATION.md` | Recommendation Engine v0 — Prospective Validation & Policy — defines what REC-001 establishes, v0 policy rules (BUY/WATCH/NO_TRADE), scoring formula, prospective population, evaluation horizon, success/failure criteria; no post-hoc tuning; **independent of HDV-002** | Active — v0 prospective observation |
| REC-001-B | `datasets/universes/coralys_102_v1.json` | REC-001-B: 102-stock NSE universe (Nifty 100 + 2 liquid mid-caps); frozen 2026-08-18; canonical single source of truth; pipeline emitted 101 decisions (1 skipped: no Yahoo data); evaluated=101, BUY=60, NO_TRADE=41; v0 clustering confirmed — all Favourable/state-bucket stocks share identical scores (0.6211); architectural finding: R:R is always 2.0 in v0 (fixed C3-002 geometry); SHORT sign display bug fixed in /live, /decisions, /decisions/[id] |
| REC-001-A | (not started) | REC-001-A: Adaptive Opportunity Geometry — superseded by ARCH-006 v1 direction; adaptive R:R now part of RecommendationEngine v1 MVP; do NOT implement as a separate research programme |
| REC-001-H | `adapters/chronosentiment/src/bin/rec001h_historical_reconstruction.rs` → `datasets/recommendation/historical/` | REC-001-H: Historical Decision Reconstruction — **COMPLETE 2026-08-18**; 101 tickers processed (1 skipped: MCDOWELL-N.NS — no Yahoo data); 121,805 records; leakage-free; schema: ticker, date, direction, trend, momentum, volatility, atr_14, reference_price, open, high, low, volume, relative_volume_20, target/risk geometry, mfe_pct[10], mae_pct[10], outcome, sessions_to_outcome; volume stored but NOT used in recommendation engine v0 |
| REC-001-H-EQ | `datasets/recommendation/historical/evidence_quality_report.csv` + `scripts/rec001h_evidence_quality.py` | REC-001-H Evidence Quality Report — **COMPLETE 2026-08-18**; C3-002 mapping verified from data: Bear+Neg→LONG (counter-trend), only SHORT state is Bull+Neg; LONG min-bucket median=170 (min=25 TMCV.NS, max=239); SHORT min-bucket median=187 (min=17 TMCV.NS, max=255); LONG target rate 29.6% mean (range 23–39%); per-bucket rates vary meaningfully; evidence sufficient for ticker-specific analogue-based recommendations |
| ARCH-006 | `docs/ARCH-006_RECOMMENDATION_ENGINE_ARCHITECTURE.md` | Recommendation Engine Architecture — **locked-in 2026-08-18**; v0 vs v1 comparison; Coralys-native design; analogue-population-based evidence; adaptive R:R + horizon; MarketDataFetcher refactor spec; MVP path (not a large research programme) |

### 10. Visualisation

| ID | Path | Title | Status |
|----|------|-------|--------|
| VIZ-001 | `docs/code_map.html` | Code Map | Needs update — Rust-only; does not cover research or governance nodes |

### 11. Services & UI

| ID | Path | Title | Status |
|----|------|-------|--------|
| SVC-001 | `services/coralys_decision_server/` | Coralys Decision Intelligence API — Rust/Axum, port 3001; GET /decisions, GET /decisions/{id}, POST /decisions (ingest), POST /decisions/{id}/execution, POST /decisions/{id}/outcome, GET /recommendations/latest (Recommendation Engine v0, HDV-001 evidence) | Active |
| SVC-002 | `chrono-ui/` | Coralys Trading MVP — Next.js 15, port 3000; Decision Feed, Inspector, Execution, Outcome, Ledger, Audit; reads Decision Server at CORALYS_API_URL (:3001) | Active |
| SVC-003 | `hdv001-dashboard/` | HDV-001 Evidence Dashboard — Vite/React, port 5174; read-only frozen evidence; KPIs, gates, segmentation, MFE chart | **FROZEN** 2026-08-17 |

---

## Evidence Directories (runtime)

These directories are populated by experiments and are not individually indexed. They are governed by the harness persistence module (`HARNESS-003`).

| Path | Contents |
|------|----------|
| `results/` | Experiment results — CSV, JSON, per-run directories |
| `logs/` | Runtime logs |
| `pilot_sessions/` | Pilot session recordings |
| `snapshots/` | System snapshots |
| `reports/` | Generated markdown reports |

---

## Non-Canonical / Pending Review

The following documents exist in the repository but have not been confirmed as canonical. They require review before being added to this index or archived.

| Path | Issue |
|------|-------|
| `docs/REPOSITORY_SURVEY.md` | Superseded by `docs/governance/KNOWLEDGE_SURVEY.md` — archive candidate |
| `docs/ChronoSentiment_Personal_Blueprint_v1.md` | Scope overlap with `ChronoSentiment_Product_Blueprint_v1.md` — review needed |
| `docs/CODEBASE_ASSESSMENT.md` | Possible duplicate of `CODEBASE_ARCHITECTURE_ASSESSMENT.md` — review needed |
| `docs/EP-001_MILESTONE.md` | Possible duplicate of `docs/P001_MILESTONE.md` — review needed |
| `docs/research/RESEARCH_LINEAGE.md` | Possible duplicate of `docs/RESEARCH_LINEAGE.md` — review needed |
| `docs/research/CS-R-009_AI_Adoption_Investment_Management.md` | Stub — content not yet authored |
| `docs/research/CS-R-015_Investment_Thesis.md` | Stub — content not yet authored |
| `docs/research/CS-R-015A_Executive_Investment_Summary.md` | Stub — content not yet authored |
| `docs/ChronoSentiment_Product_Blueprint_v1.md` | Stub — content not yet authored |
| `docs/ui/uiux.md` | Stub — content not yet authored |

---

## Changelog

| Version | Date | Change |
|---------|------|--------|
| 1.72 | 2026-08-18 | Algorithm FROZEN — policy reconciliation complete; dormant SELL branch (SHORT+Favourable→SELL, 0 SELLs at baseline); REC-BASELINE-001-RECONCILIATION.md answers all 5 Q/A from source; corrected policy semantics for REC-BASELINE-002; 60/60 tests pass; no further algorithm changes until prospective evidence accumulates |
| 1.71 | 2026-08-18 | CDI MVP v0.1 FROZEN BASELINE — 101-ticker universe; RecommendationEngine v1 OPERATIONAL; /latest deduplicates by ticker; /history returns all; evaluated=101, Buy=14, Watch=46, NoTrade=41; Prospective Observation Recorder declared NEXT MILESTONE |
| 1.70 | 2026-08-18 | Yahoo incremental fetch complete — csp006_p_enrich binary; emitted_new=202; coralys_decision_server rebuilt with dedup logic |
| 1.69 | 2026-08-18 | Evidence Quality Report; C3-002 mapping verified; ARCH-006 v1.1 |
| 1.68 | 2026-08-18 | REC-001-H Phase 1–4 complete — RecommendationEngine v1; G1–G4 pass; server wiring; UI wiring; NULL f64 fix; 34/34 tests pass |
| 1.67 | 2026-08-18 | REC-001-B complete — 101 recommendations emitted; BUY=60, NO_TRADE=41; SHORT sign fix; governance: INDEX.md changes + REC-001_RECOMMENDATION_VALIDATION.md |
| 1.66 | 2026-08-18 | SVC-001 Recommendation Engine v0 added to coralys-decision (evidence.rs + engine.rs; EvidenceStore::load_from_file; GET /recommendations/latest; /live page replaced with ranked recommendation view; 13 tests pass; policy version v0 frozen with HDV-001) |
| 1.65 | 2026-08-17 | SVC-001 POST /decisions ingest endpoint added to coralys_decision_server (Rust/Axum; SealedDecisionInput; provenance verified; builds clean) |
| 1.64 | 2026-08-17 | SVC-001/SVC-002/SVC-003 Services & UI section added to index |
| 1.63 | 2026-08-17 | HDV-002-A methodology freeze gate — docs/HDV_002_METHODOLOGY.md; forward validation protocol; accumulation opens 2026-08-18 |
| 1.62 | 2026-08-17 | HDV-001-F official criterion evaluation PASS — Gate1 +6.0pp, Gate2 +12.4pp, Gate3 2/4 segments; frozen 2026-08-17 |
| 1.61 | 2026-08-17 | HDV-001 all gates PASS — period definitions, price cache (52 instruments), path extractor (1144/1144), MAE/MFE, outcome classifier, baselines, freeze gate; frozen 2026-08-17 |
| 1.60 | 2026-08-17 | SVC-003 hdv001-dashboard frozen evidence dashboard (Vite/React, port 5174, read-only) |
| 1.59 | 2026-08-17 | CDI-MVP-V01-DEL deletion/retirement list (Observatory implementation code; evidence archives kept; dependency grep required) |
| 1.58 | 2026-08-17 | CDI-MVP-V01 Coralys Decision Intelligence MVP v0.1 specification (DecisionRecord schema; user-controlled execution; no allocation; supersedes Observatory product layer) |
| 1.57 | 2026-08-15 | CS-P-007 Statistical Strategy Validation specified (not run; P.E.3 waits; no Search #3) |
| 1.56 | 2026-08-15 | CS-P-006-P.E.3.A Coralys Target Artifact contract (P.E.2.H PASS; no target algorithm; no Search #3) |
| 1.55 | 2026-08-15 | CS-P-006-P.E.2.H historical P.E.2 lifecycle validation (15 Jul clock; live P.E.2 untouched; no Search #3) |
| 1.54 | 2026-08-15 | CS-P-006-P.E.3 Coralys Target Discovery specified (P.E.2 frozen as control; no Search #3) |
| 1.53 | 2026-08-15 | CS-P-006-P.E.B Coralys target from certified state at T (P.E.1 is the +5% control; no Search #3) |
| 1.52 | 2026-08-15 | CS-P-006-P.E.2 Live Execution Observation (next cohort with Execution Contract v0; no Search #3) |
| 1.51 | 2026-08-15 | CS-P-006-P.E.1 Execution Evidence Surface (Decision ≠ Execution ≠ Evidence; no Search #3) |
| 1.50 | 2026-08-15 | CS-P-006-P.E Targeted Decision Execution (target sealed at T; OHLC first-exit; no Search #3) |
| 1.49 | 2026-08-15 | CS-P-006-P.H.3 Decision Evidence Dashboard (replay integrity ≠ strategy validation; no Search #3) |
| 1.48 | 2026-08-15 | CS-P-006-P.H.2 20 market-session Observatory horizon (Replay v1; v0 archived; no Search #3) |
| 1.47 | 2026-08-15 | CS-P-006-P.H.1 Decision Evidence Engine (20 calendar days; session rule; not a statistical backtest) |
| 1.46 | 2026-08-15 | CS-P-006-P.H Historical Observatory Replay (same engine; no lookahead; not C.3-G) |
| 1.45 | 2026-08-15 | CS-P-006-P Observatory wording (outcome not yet observed; no early peek; no Search #3) |
| 1.44 | 2026-08-15 | CS-P-006-P Observatory maturity path (countdown / OUTCOME DUE; no early peek; no Search #3) |
| 1.43 | 2026-08-15 | CS-P-006-P prospective C3-002 paper clock (OBSERVING; no outcomes at seal; no Search #3) |
| 1.42 | 2026-08-15 | CS-P-006-P.7 Observatory product screens (Decision object; OBSERVED status; no Search #3) |
| 1.41 | 2026-08-15 | CS-P-006-P.3–P.6 observatory vertical slice (immutable record; append-only outcome; no Search #3) |
| 1.40 | 2026-08-15 | CS-P-006-P Decision Observatory protocol (C3-002 paper-only; research loop stopped; no Search #3) |
| 1.39 | 2026-08-15 | CS-P-006-C.3-G regime-persistence question (experiment not authorized; no Search #3) |
| 1.38 | 2026-08-15 | CS-P-006-C.3-F certified TMV state x action landscape (no Search #3; not a strategy) |
| 1.37 | 2026-08-15 | CS-P-006-C.3-E Search #2 discovered-rule persistence (no pass threshold; no Search #3) |
| 1.36 | 2026-08-15 | CS-P-006-C.3-D Search #2 live-rule ecology (no Search #3; not promoted) |
| 1.35 | 2026-08-15 | CS-P-006-C.3-C Search #1 vs #2 comparative review (no Search #3) |
| 1.34 | 2026-08-15 | CS-P-006-C.3-R one authorized Search #2 run (Search #1 immutable; no iteration) |
| 1.33 | 2026-08-15 | CS-P-006-C.3-I implementation and identity gate (Search #2 not run) |
| 1.32 | 2026-08-15 | CS-P-006-C.3 protocol authorized (Search #2 not started; same TMV; M.1 V) |
| 1.31 | 2026-08-15 | CS-P-006-N harness implemented (symbol matrices required; C.3 not authorized) |
| 1.30 | 2026-08-15 | CS-P-006-M.1 decision-value specification + CS-P-006-N harness spec (no Search #2) |
| 1.29 | 2026-08-15 | CS-P-006-M decision-value model (protocol only; no Search #2; advantage is not fitness) |
| 1.28 | 2026-08-15 | CS-P-006-C.2-D decision-value landscape (no Search #2; no invented bands) |
| 1.27 | 2026-08-15 | CS-P-006-C.2-S selection and decision-value review (no Search #2) |
| 1.26 | 2026-08-15 | CS-P-006-C.2-R Search #1 recommendation-vs-outcome matrix (no Search #2) |
| 1.25 | 2026-08-15 | CS-P-006-C.2-P Search #1 population ecology (no Search #2; C.3 not authorized) |
| 1.24 | 2026-08-15 | CS-P-006-C.2-O search observability (no Search #2; artifact identity unchanged) |
| 1.23 | 2026-08-15 | CS-P-006-C.2 research-gap review (observability + volatility presence; no Search #2) |
| 1.22 | 2026-08-15 | CS-P-006-C.1 Search #1 diagnosis (no second search; no promotion) |
| 1.21 | 2026-08-14 | CS-P-006-C first Coralys TMV discovery (sealed PolicyArtifact; evaluation not fed back) |
| 1.20 | 2026-08-14 | CS-P-006-V decision-value vision (future certified families; not 006-C genome) |
| 1.19 | 2026-08-14 | CS-P-006-B.1 chronological partition freeze (39 timestamps; Coralys authorized) |
| 1.18 | 2026-08-14 | CS-P-006-S1 certified PASS (7-name research snapshot; dates still not frozen) |
| 1.17 | 2026-08-14 | Index CS-P-006-S1 7-instrument research snapshot (not B4/B5; no date freeze) |
| 1.16 | 2026-08-14 | Index CS-P-006-B research protocol (7-name universe; split dates not frozen) |
| 1.15 | 2026-08-14 | Index CS-P-006 / CS-P-006-A Policy Discovery Contract (no optimizer; no split dates) |
| 1.14 | 2026-08-14 | Index CS-P-CLEAN-002 explicit DecisionPolicy contract (PR-2; no CS-P-006) |
| 1.13 | 2026-08-14 | Index CS-P-CLEAN-001 adapter quarantine (PR-1; no B4 regeneration) |
| 1.12 | 2026-08-14 | Index CS-P-AUDIT-001 adapter discipline audit (read-only; no cleanup) |
| 1.11 | 2026-08-14 | Index CS-P-TEST-001 verification matrix and CS-P-005 Factor Ecology (no candidate policy) |
| 1.10 | 2026-08-14 | Index CS-P-004-E1-S1 information-fidelity snapshot (not B5; factor availability only) |
| 1.9 | 2026-08-14 | Index CS-P-004-E1 Assessment Enrichment v0.1 (code/tests only; no new dataset) |
| 1.8 | 2026-08-14 | Index CS-P-004-A1 adapter information-fidelity enhancement |
| 1.7 | 2026-08-14 | Index CS-P-004-R1 B4 historical laboratory reports |
| 1.6 | 2026-08-14 | Index CS-P-004 Historical Research Laboratory; CS-P-003 remains confirmation clock |
| 1.5 | 2026-08-14 | Index CS-P-003 Forward/Paper Validation v0.1 |
| 1.4 | 2026-08-14 | Index CS-P-002-R1 B4 historical product validation (`unfrozen-dev`) |
| 1.3 | 2026-08-14 | Index CS-P-002 Decision Validation Platform |
| 1.2 | 2026-08-14 | Index CS-P-001 product mode; PRD-002 |
| 1.1 | 2026-08-14 | Index EV-GOV-003 G-GATE v1.1 statistical closure |
| 1.0 | 2026-08-01 | Initial creation — Repository v2 Governance Artifact 2 |