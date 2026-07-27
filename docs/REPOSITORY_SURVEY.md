# Repository Documentation Survey
## ChronoSentiment / Coralys / UltraCrew — Full `docs/` Inventory

**Survey Date:** July 2026
**Purpose:** Baseline for repository cleanup. Classifies every document across seven dimensions to support informed cleanup decisions.

---

## Classification Dimensions

| Dimension | Values |
|-----------|--------|
| **Type** | Constitution · Contract · Specification · Governance · Research · Evidence · Operational · Reference · Archive |
| **Product** | Coralys · ChronoSentiment · UltraCrew/INRC · Shared |
| **Lifecycle** | Draft · Baseline · Frozen · Operational · Historical · Archived |
| **Authority** | Constitutional (highest) · Contractual · Normative · Informational · Superseded |
| **Owner** | Programme · Engineering · Research · Pilot |
| **Review Trigger** | (document-specific) |
| **Action** | ✅ KEEP · 📦 ARCHIVE · 🗑️ DELETE · ❓ REVIEW |

---

## 1. Top-Level `docs/` — Architecture & Governance

| File | Type | Product | Lifecycle | Authority | Owner | Action |
|------|------|---------|-----------|-----------|-------|--------|
| `ARCHITECTURE_EVOLUTION.md` | Constitution | Coralys | **Frozen** 2026-07-22 | Constitutional | Programme | ✅ KEEP |
| `ARCHITECTURE_GLOSSARY.md` | Reference | Coralys | Baseline v1.2 | Normative | Programme | ✅ KEEP |
| `CODEBASE_ASSESSMENT.md` | Evidence | Coralys | Baseline | Normative evidence (authoritative assessment) | Engineering | ✅ KEEP |
| `CODEBASE_ARCHITECTURE_ASSESSMENT.md` | Evidence | Coralys | **Historical** | Superseded | Engineering | 📦 ARCHIVE → `docs/archive/engineering/` (Owner Decision B resolved — transitional document; responsibilities split into `CODEBASE_ASSESSMENT.md` and `ARCHITECTURE_EVOLUTION.md`) |
| `CORALYS_EVIDENCE_GOVERNANCE_STANDARD.md` | Governance | Coralys | Baseline v1.0 | Normative | Programme | ✅ KEEP |
| `EXECUTION_DIRECTIVE_2026-07-22.md` | Governance | Coralys | Operational | Normative | Programme | ✅ KEEP |
| `PLATFORM_CRATE_RESPONSIBILITIES.md` | Reference | Coralys | Operational | Normative | Engineering | ✅ KEEP |
| `RESEARCH_LINEAGE.md` | Reference | Coralys | Operational | Informational | Research | ✅ KEEP |
| `Service_Boundary_Definition.md` | Specification | Coralys | Baseline | Contractual | Engineering | ✅ KEEP |
| `System_Guarantee.md` | Specification | ChronoSentiment | Baseline | Normative | Engineering | ✅ KEEP |
| `DOMAIN_EXTENSION_GUIDE.md` | Reference | Coralys | Unknown | Informational | Engineering | ❓ REVIEW — check currency post architecture freeze |
| `OPERATIONS.md` | Operational | ChronoSentiment | Operational | Informational | Engineering | ✅ KEEP |
| `PHASE2_DECISION_PROTOCOL.md` | Governance | Coralys | **Operational** | Normative | Programme | ✅ KEEP — active governance procedure for Phase 2 authority reductions; defines how every authority reduction must be classified, validated against sealed baseline, and recorded; not a completion record; becomes Historical once all Phase 2 targets are processed and a Phase 2 Completion checkpoint is issued (Owner Decision F resolved) |

---

## 2. Top-Level `docs/` — ChronoSentiment Product

| File | Type | Product | Lifecycle | Authority | Owner | Action |
|------|------|---------|-----------|-----------|-------|--------|
| `CHRONOSENTIMENT_PRD_V1.md` | Specification | ChronoSentiment | **Frozen** (governance baseline) | Constitutional | Programme | ✅ KEEP |
| `CHRONOSENTIMENT_PRODUCT_CONCEPT.md` | Specification | ChronoSentiment | **Frozen** (governance baseline) | Constitutional | Programme | ✅ KEEP |
| `CHRONOSENTIMENT_EVIDENCE_PROGRAMME.md` | Governance | ChronoSentiment | **Frozen** (governance baseline) | Constitutional | Programme | ✅ KEEP |
| `Backend_Architecture_Blueprint.md` | Specification | ChronoSentiment | **Historical** | Superseded | Engineering | 📦 ARCHIVE → `docs/archive/engineering/` (Owner Decision A resolved — superseded by `ARCHITECTURE_EVOLUTION.md`; retains value as ChronoSentiment conceptual design lineage) |
| `Event_Flow_Specification.md` | Specification | ChronoSentiment | **Operational** | Normative | Engineering | ✅ KEEP — canonical execution-flow specification (behaviour: *how the system runs*); complementary to `Service_Boundary_Definition.md` (governance: *who may do what*); not superseded (Owner Decision C resolved) |
| `MVP_Scope_Document_v2_1.md` | Specification | ChronoSentiment | **Baseline** | Normative | Programme | ✅ KEEP — engineering scope for execution-validation MVP; not superseded by `CHRONOSENTIMENT_PRD_V1.md` (PRD broadens product vision; MVP Scope defines implementation boundaries for one core capability); ⚠️ RECONCILIATION NEEDED — MVP Scope still frames execution validation as the entire product; needs updating to reflect PRD's Financial Decision Intelligence Platform positioning (Owner Decision E resolved) |
| `SRS_v1_6.md` | Specification | ChronoSentiment | **Baseline** | Normative | Engineering | ✅ KEEP — active product requirements specification (*what ChronoSentiment must do*); not superseded by `ARCHITECTURE_EVOLUTION.md` (*how Coralys must be governed*); complementary documents at different governance levels (Owner Decision D resolved) |
| `PRD_v3_3.md` | Specification | ChronoSentiment | **Superseded** (header applied) | Superseded | Programme | 📦 ARCHIVE |
| `ChronoSentiment_Product_Strategy_v1.md` | Strategy | ChronoSentiment | **Draft** | Normative | Product | ✅ KEEP — product strategy document; market choice rationale (why investment management first), defensibility (five barriers), competitive threats (Microsoft, Bloomberg, OpenAI), long-term platform vision (four phases), adjacent markets, deliberate out-of-scope |
| `ChronoSentiment_Product_Blueprint_v1.md` | Blueprint | ChronoSentiment | **Draft** | Normative | Product | ✅ KEEP — bridge document between Phase 1A research programme and Phase 1B / MVP engineering; includes "Why Investment Organisations Forget", "Why Now?", "A Day with ChronoSentiment" narrative, Decision Workspace chapter, capability mapping table and MVP scope |
| `CORALYS_PLATFORM_ARCHITECTURE.md` | Platform Architecture | Coralys | **Baseline v1.5** | Normative | Platform / Engineering | ✅ KEEP — defines Coralys as a Knowledge Evolution Platform; platform layering diagram (Platform owns lifecycle/primitives/KG/adapter model; powered by Continuous Learning Engine); canonical architecture diagram; three-part model (Platform governs / Engine drives / Knowledge Graph preserves); 13 platform primitives (Workspace as transaction boundary, Actor, Intent, Subject, Context, Evidence, Hypothesis, Review, Timeline, Outcome, Learning, Pattern, Knowledge Graph); Learning computes / Knowledge Graph stores; Pattern Extraction and Question as future primitives; Coralys v2 observations; adapter vocabulary tables; Product Portfolio Positioning moved to `CORALYS_PLATFORM_STRATEGY.md`; complementary to `ARCHITECTURE_EVOLUTION.md` (engineering layer) |
| `CORALYS_PLATFORM_STRATEGY.md` | Platform Strategy | Coralys | **Baseline v1.1** | Normative | Strategy / Product | ✅ KEEP — product portfolio positioning; platform-product relationship; product identities (UltraCrew = Workforce Decision Engine; ChronoSentiment Enterprise = Financial Decision Intelligence Platform; ChronoSentiment Personal = Personal Investment Knowledge Platform); portfolio principle table (hidden vs visible platform); licensing and OEM considerations; three-layer architecture table (Coralys / Products / Continuous Learning Engine) |
| `ChronoSentiment_Personal_Blueprint_v1.md` | Blueprint | ChronoSentiment Personal | **Draft** | Normative | Product | ✅ KEEP — personal investment research and decision journal platform; research dossier format; AI as research assistant (not decision-maker); portfolio observations (not advice); thesis versioning; six-level feedback loop (thesis, portfolio, process, thesis evolution, research quality, investor behaviour); Personal Investment Learning Loop as the moat; implemented as domain adapter over Coralys Continuous Learning Engine. **EP-001 note:** adapter foundation now executable — evidence, hypothesis, timeline, workspace, and learning modules implemented in `adapters/chronosentiment/src/`; blueprint is no longer aspirational for these primitives |
| `EP-001_MILESTONE.md` | Milestone | Coralys / UltraCrew / ChronoSentiment | **Operational** | Normative | Engineering | ✅ KEEP — EP-001 milestone record; documents the transition from documentation-led to implementation-led; acceptance criteria, what changed, platform invariants now enforced by code, test position, and transition point to SunAir pilot and Phase 1B |
| `CS-CAT-001_Financial_Decision_Management.md` | Category Definition | ChronoSentiment | **Draft** | Normative | Product / Commercial | ✅ KEEP — defines the Financial Decision Management category; why existing systems (OMS, PMS, RMS, KMS, compliance, AI tools) do not solve the problem; full category vocabulary (Decision Workspace, Decision Record, Decision Memory, Decision Provenance, Decision Reconstruction, Decision Intelligence, Decision Archive, Decision Governance, Decision Evidence, Decision Lifecycle); why the category is emerging now |
| `CS-SUC-001_Customer_Success_Blueprint.md` | Customer Success | ChronoSentiment | **Draft** | Normative | Product / Commercial | ✅ KEEP — adoption journey from Week 1 through Year 1; five stages (Activation, Habit Formation, Governance Integration, Archive Value, Decision Intelligence); success indicators, failure modes, and design partner implications at each stage; moat accumulation model |
| `CV-001_Commercial_Validation_Playbook.md` | Playbook | ChronoSentiment | **Operational** | Normative | Commercial | ✅ KEEP — Phase 1B execution guide; includes Phase 1BA commercial intelligence workstream, staged evidence funnel, target customer ranking, buying committee map, discovery interview guide, design partner process, pricing experiment, kill criteria |
| `EL-001_Phase1B_Evidence_Ledger.md` | Evidence Ledger | ChronoSentiment | **Operational** | Normative | Commercial | ✅ KEEP — primary evidence register for Phase 1B; multi-type evidence IDs (INT/EXP/OBS/DEM/POC), hypothesis confidence tracker (H1–H7), rolling synthesis cadence, final go/no-go decision template |
| `COM-001_Commercial_Intelligence_Database.md` | Intelligence Database | ChronoSentiment | **Operational** | Normative | Commercial | ✅ KEEP — firm-level commercial intelligence database; firm dossier template, relationship stage register (stages 0–12), contact ecosystem register (13 contact types), intelligence synthesis cadence, outreach templates |

---

## 3. Top-Level `docs/` — Benchmark Programme

| File | Type | Product | Lifecycle | Authority | Owner | Action |
|------|------|---------|-----------|-----------|-------|--------|
| `BENCHMARK_RESULTS.md` | Evidence | UltraCrew/INRC | Operational (updated 2026-07-23) | Informational | Engineering | ✅ KEEP |
| `BENCHMARK-GOVERNANCE.md` | Governance | Coralys | Operational (ACTIVE) | Normative | Engineering | ✅ KEEP |
| `BENCHMARK-REFERENCE-SPECIFICATION-v1.0.md` | Specification | Coralys | Draft (Milestone 3A pre-freeze) | Normative | Engineering | ✅ KEEP |

---

## 4. Top-Level `docs/` — Phase 1 (Historical)

| File | Type | Product | Lifecycle | Authority | Owner | Action |
|------|------|---------|-----------|-----------|-------|--------|
| `PHASE1_DOMAIN_COMPARISON.md` | Research | Coralys | Historical | Superseded | Research | 📦 ARCHIVE |
| `PHASE1_GOVERNANCE_CHECKPOINT.md` | Governance | Coralys | Historical | Superseded | Programme | 📦 ARCHIVE |

---

## 5. Top-Level `docs/` — P-001 Pilot (UltraCrew / INRC)

| File | Type | Product | Lifecycle | Authority | Owner | Action |
|------|------|---------|-----------|-----------|-------|--------|
| `INRC_DEMO_GUIDE.md` | Operational | UltraCrew/INRC | Operational | Informational | Pilot | ✅ KEEP |
| `INRC_PRODUCT_EVIDENCE_PROGRAMME.md` | Governance | UltraCrew/INRC | Operational | Normative | Pilot | ✅ KEEP |
| `P001_MILESTONE.md` | Governance | UltraCrew/INRC | Operational | Normative | Pilot | ✅ KEEP |
| `P001_PILOT_RUNBOOK.md` | Operational | UltraCrew/INRC | Operational | Informational | Pilot | ✅ KEEP |
| `ULTRACREW_WORKFORCE_EVIDENCE.md` | Evidence | UltraCrew/INRC | Operational | Informational | Pilot | ✅ KEEP |
| `sunair_pilot_guide.md` | Operational | UltraCrew/INRC | Operational | Informational | Pilot | ✅ KEEP |
| `sunair_sales_playbook.md` | Operational | UltraCrew/INRC | Operational | Informational | Pilot | ✅ KEEP |

---

## 6. `docs/research/` — CS-R Series (Phase 1A Research Baseline)

All 15 CS-R documents are current research baseline artefacts. Product: ChronoSentiment. Owner: Research. Authority: Informational (evidence) / Normative (recommendations). Review trigger: Phase 1B completion.

| File | Lifecycle | Action |
|------|-----------|--------|
| `CS-R-000_Research_Evidence_Sufficiency_Matrix.md` | Baseline v1.0 | ✅ KEEP |
| `CS-R-001_Market_Landscape.md` | Baseline v2.0 | ✅ KEEP |
| `CS-R-002_Competitive_Landscape.md` | Baseline v2.0 | ✅ KEEP |
| `CS-R-003_Customer_Problem_Evidence.md` | Baseline v2.0 | ✅ KEEP |
| `CS-R-004_Regulatory_Landscape.md` | Baseline v2.0 | ✅ KEEP |
| `CS-R-005_Pricing_Analysis.md` | Baseline v2.0 | ✅ KEEP |
| `CS-R-006_Data_Landscape.md` | Baseline v2.0 | ✅ KEEP |
| `CS-R-007_Explainability_Research.md` | Baseline v2.0 | ✅ KEEP |
| `CS-R-008_Point_In_Time_Architecture_Review.md` | Baseline v2.0 | ✅ KEEP |
| `CS-R-009_AI_Adoption_Investment_Management.md` | Baseline v1.0 | ✅ KEEP |
| `CS-R-010_Investment_Workflow_Evolution.md` | Baseline v1.0 | ✅ KEEP |
| `CS-R-011_Decision_Governance_Research.md` | Baseline v1.0 | ✅ KEEP |
| `CS-R-012_Build_vs_Buy_Analysis.md` | Baseline v1.0 | ✅ KEEP |
| `CS-R-013_Technology_Readiness_Assessment.md` | Baseline v1.0 | ✅ KEEP |
| `CS-R-014_Product_Category_Creation_Study.md` | Baseline v1.0 | ✅ KEEP |
| `CS-R-015_Investment_Thesis.md` | Baseline v1.0 | ✅ KEEP |
| `CS-R-015A_Executive_Investment_Summary.md` | Baseline v1.0 | ✅ KEEP — entry-point document for CS-R-015; 2-page executive summary of the investment case |

---

## 7. `docs/research/` — Operational Contracts (Non-CS-R)

> **Current location:** `docs/research/`
> **Approved destination:** `docs/contracts/` (Owner Decision O resolved)
> **Migration status:** Pending — contracts remain active at current location until physically moved; a light classification review is recommended during migration to separate permanent platform contracts from transitional migration-specific contracts.

These are active engineering contracts governing the ChronoSentiment simulation and replay engine. Type: Contract. Product: Coralys/ChronoSentiment. Lifecycle: Operational. Authority: Contractual. Owner: Engineering.

| File | What it governs | Action |
|------|----------------|--------|
| `CHRONOLOGY_AXIOMS_CONTRACT_v1.md` | Foundational time-handling rules | ✅ KEEP |
| `CRYPTO_SUBSTRATE_CONTRACT_v1.md` | Crypto data handling rules | ✅ KEEP |
| `ECOLOGICAL_SURVIVABILITY_SURFACE_SPEC_v1.md` | Ecological survivability surface | ✅ KEEP |
| `ECOLOGY_COMPARISON_PROTOCOL_v1.md` | Ecology comparison protocol | ✅ KEEP |
| `ECONOMIC_SEMANTICS_CONTRACT_v1.md` | Financial calculation rules | ✅ KEEP |
| `EXECUTION_ECOLOGY_SPEC_v1.md` | Execution ecology specification | ✅ KEEP |
| `GENERALIZATION_BOUNDARY_CONTRACT_v1.md` | Generalisation limits | ✅ KEEP |
| `LIVE_CAPTURE_ISOLATION_CONTRACT_v1.md` | Live capture isolation rules | ✅ KEEP |
| `METROLOGY_LAYER_CONTRACT_v1.md` | Measurement and metrics rules | ✅ KEEP |
| `MORPHOLOGY_RESPONSE_ISOLATION_CONTRACT_v1.md` | Morphology response isolation | ✅ KEEP |
| `OBSERVABILITY_SEMANTICS_CONTRACT_v1.md` | Observability rules | ✅ KEEP |
| `OSCILLATORY_TOPOLOGY_DEFINITION_CONTRACT_v1.md` | Oscillatory topology definition | ✅ KEEP |
| `PERSISTENCE_SEMANTICS_CONTRACT_v1.md` | Data persistence rules | ✅ KEEP |
| `PNL_ATTRIBUTION_CONTRACT_v1.md` | PnL attribution rules | ✅ KEEP |
| `REPAIR_LABORATORY_ISOLATION_PROTOCOL_v1.md` | Repair lab isolation | ✅ KEEP |
| `REPAIR_SEMANTICS_CONTRACT_v1.md` | Repair semantics | ✅ KEEP |
| `REPLAY_EQUIVALENCE_CONTRACT_v1.md` | Replay determinism rules | ✅ KEEP |
| `REPLAY_MANIFEST_SPECIFICATION_v1.md` | Replay manifest format | ✅ KEEP |
| `SIGNAL_INTERFACE_CONTRACT_v1.md` | Signal interface rules | ✅ KEEP |
| `STATE_COHERENCE_CONTRACT_v1.md` | State consistency rules | ✅ KEEP |
| `SURFACE_HASH_CONTRACT_v1.md` | Determinism verification via hashing | ✅ KEEP |
| `SURFACE_INTERPRETATION_CONTRACT_v1.md` | Surface interpretation rules | ✅ KEEP |
| `TOPOLOGY_NEUTRALITY_CONTRACT_v1.md` | Topology neutrality rules | ✅ KEEP |
| `TOPOLOGY_PERTURBATION_CONTRACT_v1.md` | Topology perturbation rules | ✅ KEEP |
| `ARCHIVE_CURATION_CONTRACT_v1.md` | Archive curation rules | ❓ REVIEW — consider moving to `docs/contracts/` |
| `RUST_PORT_CONTRACT_v1.md` | Rust consolidation rules | ⚠️ DEFERRED — Rust port completion not evidenced; contract remains active migration governance until a completion milestone or architecture baseline formally closes the migration (Owner Decision J deferred) |

---

## 8. `docs/research/` — Other Files

| File | Type | Product | Lifecycle | Authority | Action |
|------|------|---------|-----------|-----------|--------|
| `README.md` | Reference | ChronoSentiment | Operational | Informational | ✅ KEEP |
| `RESEARCH_LOG.md` | Evidence | ChronoSentiment | Operational | Informational | ✅ KEEP |
| `DISCREPANCY_REPORT.md` | Evidence | ChronoSentiment | **Historical** | Superseded | Engineering | 📦 ARCHIVE → `docs/archive/engineering/` (Owner Decision I resolved — principal conclusions absorbed into current documentation set: product identity clarified in PRD, architectural authority in Service Boundary Definition, replay/determinism in SRS, observer-first philosophy in Architecture Evolution; remaining items are prototype UI implementation details superseded by new architecture) |
| `HANDOFF.md` | Operational | ChronoSentiment | Historical (session-specific) | Informational | 📦 ARCHIVE |
| `chrono_sentiment_full_system.md` | Specification | ChronoSentiment | **Historical** | Superseded | Engineering | 📦 ARCHIVE → `docs/archive/engineering/` (Owner Decision G resolved — monolithic early-stage design document decomposed into PRD, SRS, Event Flow Specification, Service Boundary Definition, MVP Scope; also reflects earlier product framing as trading simulation rather than Financial Decision Intelligence Platform) |
| `chrono_sentiment_user_guide.md` | Operational | ChronoSentiment | **Historical** | Superseded | Engineering | 📦 ARCHIVE → `docs/archive/engineering/` (Owner Decision H resolved — belongs to earlier product phase where ChronoSentiment was an Institutional Live Inference Engine; current product is Financial Decision Intelligence Platform; CLI operator guidance and live inference workflow no longer reflect current product identity) |

---

## 9. `docs/archive/`

All files are correctly placed. Type: Archive. Lifecycle: Archived. Authority: Historical.

| File | What it was | Action |
|------|------------|--------|
| `Execution_Thinking_Book.md` | Execution Thinking Handbook v3.3 — 11-chapter conceptual guide | ✅ KEEP IN ARCHIVE |
| `kernel_truths.md` | Kernel Truths — foundational principles | ✅ KEEP IN ARCHIVE |
| `Market_Regine_External_Influence_specification.md` | Market Regime & External Influence Spec v1.0 | ✅ KEEP IN ARCHIVE |
| `mocrostructure_dictionary.md` | Market Microstructure Dictionary v1.0 | ✅ KEEP IN ARCHIVE — **rename to `microstructure_dictionary.md`** (typo) |
| `PSD_v1_1.md` | Product/System Design v1.1 | ✅ KEEP IN ARCHIVE |
| `SDS_v2_0.md` | Strategy Definition Specification v2.0 | ✅ KEEP IN ARCHIVE |
| `short_side_diagnostic_checklist.md` | Short-side signal diagnostic checklist | ✅ KEEP IN ARCHIVE |
| `short_side_final_findings.md` | Short-side signal diagnostic final findings | ✅ KEEP IN ARCHIVE |
| `short_side_phase2_unlock_criteria.md` | Phase 2 short-side unlock criteria | ✅ KEEP IN ARCHIVE |

---

## 10. `docs/capabilities/`

Type: Specification. Product: Coralys. Lifecycle: Operational. Authority: Normative. Owner: Engineering.

| File | What it defines | Action |
|------|----------------|--------|
| `optimization.md` | Optimization capability | ✅ KEEP |
| `orchestration.md` | Orchestration capability | ✅ KEEP |
| `runtime_replay.md` | Runtime replay capability | ✅ KEEP |
| `signal_generation.md` | Signal generation capability | ✅ KEEP |
| `strategy_evaluation.md` | Strategy evaluation capability | ✅ KEEP |

---

## 11. `docs/certification/`

Type: Evidence. Product: ChronoSentiment. Lifecycle: Operational. Authority: Informational (certified evidence). Owner: Engineering.

| File | What it certifies | Action |
|------|------------------|--------|
| `orchestration_execution_order_certification.md` | Orchestration execution order | ✅ KEEP |
| `PHASE6_5_CONSEQUENCE_REPORT.md` | Phase 6.5 execution consequence | ✅ KEEP |
| `PHASE6_EXPLAINABILITY_CERTIFICATION.json` | Phase 6 explainability (JSON) | ✅ KEEP |
| `PHASE6_EXPLAINABILITY_TRACES.md` | Phase 6 explainability traces | ✅ KEEP |
| `PHASE7_FRAGILITY_MAPS.md` | Phase 7 strategy fragility | ✅ KEEP |
| `PHASE8_PORTFOLIO_INVARIANCE_SURFACE.md` | Phase 8 portfolio invariance | ✅ KEEP |
| `replay_certification_log.md` | Replay certification ledger | ✅ KEEP |
| `sweep_projection_certification.md` | Sweep projection | ✅ KEEP |
| `certified_artifacts/*.json` (10 files) | Certified strategy artifacts | ✅ KEEP |

---

## 12. `docs/constitution/`

Type: Constitution. Product: Coralys. Lifecycle: Frozen. Authority: Constitutional. Owner: Programme. Review trigger: Constitutional amendment only.

| File | What it defines | Action |
|------|----------------|--------|
| `architecture.md` | Constitutional architecture | ✅ KEEP |
| `AUTHORITY_GEOMETRY.md` | Who owns what | ✅ KEEP |
| `GENERIC_LAYER.md` | Generic layer definition | ✅ KEEP |
| `glossary.md` | Constitutional terminology | ✅ KEEP |
| `OBSERVATORY_AUTHORITY.md` | Observatory layer rules | ✅ KEEP |
| `OPERATIONAL_SOVEREIGNTY.md` | Operational independence rules | ✅ KEEP |
| `OPTIMIZATION_BOUNDARY.md` | Optimization boundary contract | ✅ KEEP |
| `SEMANTIC_REALIZATION_GATES.md` | Semantic validity conditions | ✅ KEEP |
| `topology.md` | Certified determinism surfaces | ✅ KEEP |
| `warning_policy.md` | Constitutional warning policy | ✅ KEEP |

---

## 13. `docs/contracts/`

Type: Contract. Product: ChronoSentiment. Lifecycle: Operational. Authority: Contractual. Owner: Engineering.

| File | What it contracts | Action |
|------|------------------|--------|
| `REPLAY_ATTESTATION_CONTRACT_v1.md` | Replay attestation | ✅ KEEP |
| `SCENARIO_DOMAIN_CONTRACT_v1.md` | Scenario domain | ✅ KEEP |
| `TIMELINE_VIEW_CONTRACT.md` | Timeline view model | ✅ KEEP |
| `TRADE_INSPECTOR_CONTRACT.md` | Trade inspector view model | ✅ KEEP |
| `UI_API_CONTRACT_v1.md` | UI/API interface (with pytest suite) | ✅ KEEP |

---

## 14. `docs/ga/`

Type: Specification. Product: Coralys. Lifecycle: Operational. Authority: Normative. Owner: Engineering.

| File | What it defines | Action |
|------|----------------|--------|
| `ga.md` | Genetic algorithm — population, mutation, fitness | ✅ KEEP |
| `threshold_sweep_selection_pressure.md` | Threshold sweep, gate diagnostics, selection pressure | ✅ KEEP |

---

## 15. `docs/governance/` — Active Documents

Type: Governance. Product: Coralys/ChronoSentiment. Lifecycle: Operational. Authority: Normative. Owner: Engineering/Programme.

| File | What it governs | Action |
|------|----------------|--------|
| `DEMO_SCOPE.md` | Replay-certified observability demo scope | ✅ KEEP |
| `event_taxonomy.md` | Canonical event taxonomy | ✅ KEEP |
| `governance_index.md` | Master governance index | ✅ KEEP |
| `governance_ledger_v011.md` | Governance ledger v0.11 | ✅ KEEP |
| `OBSERVATIONAL_SOAK_RESULTS.md` | Observational soak results | ✅ KEEP |
| `ontology.md` | System ontology | ✅ KEEP |
| `POST_GOVERNANCE_OBSERVATIONAL_REPLAY_SOAK.md` | Post-governance replay soak | ✅ KEEP |
| `REPLAY_ARCHIVE_RESTORATION_CONTRACT.md` | Replay archive restoration contract | ✅ KEEP |
| `REPLAY_ARCHIVE_RESTORATION_PLAN.md` | Replay archive restoration plan | ✅ KEEP |
| `REPLAY_WINDOW_DECLARATION_BATCH_003.md` | Replay window declaration batch 003 | ✅ KEEP |
| `SEMANTIC_LINT_POSTURE.md` | Semantic validation rules | ✅ KEEP |
| `semantic_registry.md` | Canonical semantic registry | ✅ KEEP |
| `soak_design.md` | Soak design for BTC/ETH/SOL | ✅ KEEP |
| `SUBSTRATE_CONTRACT_v1.md` | Phase 4.1 substrate contract | ✅ KEEP |
| `V006_SERIALIZATION_LAW_DECLARATION.md` | Serialization law (active law, not just a decision record) | ✅ KEEP |
| `V001_API_ADMISSIBILITY_CONTRACT.md` | API admissibility contract (active contract) | ✅ KEEP |
| `LANE4_EXPERIMENTAL_CONSUMER_LABELING_SCOPE.md` | Lane 4 experimental consumer labeling | 📦 ARCHIVE → `docs/archive/governance/` (Owner Decision K resolved — Lane 4 was a temporary calibration activity to audit and label experimental/research surfaces; its purpose has been fulfilled by the constitutional authority model, authority maps, and Phase 1/2 governance; no longer an active work lane) |
| `transitional_artifacts.md` | Transitional artifacts registry | ✅ KEEP — active operational governance document; 10 of 11 transitional artifacts still ACTIVE (stub narrative generation, stub execution trace, stub certification logic, frontend confidence/divergence computation, etc.); sunset conditions not yet met; archive trigger: all 11 artifacts reach ELIMINATED status (Owner Decision L resolved) |

---

## 16. `docs/governance/` — Historical Decision Records (V-Series)

Type: Governance. Lifecycle: Historical. Authority: Superseded (decisions ratified and implemented). Recommended action: move to `docs/archive/governance/` as a batch.

| File | Decision | Action |
|------|---------|--------|
| `V001_API_ROUTING_DELTA.md` | V-001 API routing delta | 📦 ARCHIVE |
| `V001_EDGE_DECAY_ROUTING_DELTA.md` | V-001 edge-decay routing delta | 📦 ARCHIVE |
| `V001_RATIFICATION_DECISION.md` | V-001 ratification decision | 📦 ARCHIVE |
| `V001_REPLAY_COHORT_ADJUDICATION.md` | V-001 replay cohort adjudication | 📦 ARCHIVE |
| `V001_ROUTING_EQUIVALENCE_SCOPE.md` | V-001 routing equivalence scope | 📦 ARCHIVE |
| `V003_API_ERROR_CONSOLIDATION_SCOPE.md` | V-003 API error consolidation | 📦 ARCHIVE |
| `V006_CAPTURE_SCHEMA_SCOPE.md` | V-006 capture schema scope | 📦 ARCHIVE |
| `V006_CHRONOLOGY_BYTE_FIXTURES.md` | V-006 chronology byte fixtures | 📦 ARCHIVE |
| `V006_LIVE_CAPTURE_AUTHORITY_DECISION.md` | V-006 live capture authority | 📦 ARCHIVE |
| `V006_MANIFEST_DIALECT_POLICY.md` | V-006 manifest dialect policy | 📦 ARCHIVE |
| `V006_MANIFEST_MS_ASSUMPTION_PROBE_SCOPE.md` | V-006 manifest ms assumption probe | 📦 ARCHIVE |
| `V006_MANIFEST_MS_IMPACT_INVENTORY_SCOPE.md` | V-006 manifest ms impact inventory | 📦 ARCHIVE |
| `V006_PHASE_C_CLASSIFICATION.md` | V-006 Phase C classification | 📦 ARCHIVE |
| `V006_PRODUCER_RATIFICATION.md` | V-006 producer ratification | 📦 ARCHIVE |
| `V007_TYPE_AUTHORITY_DECISION.md` | V-007 type authority decision | 📦 ARCHIVE |
| `V007_TYPE_AUTHORITY_SCOPE.md` | V-007 type authority scope | 📦 ARCHIVE |
| `V008_TEST_ASSET_AUTHORITY_DECISION.md` | V-008 test asset authority decision | 📦 ARCHIVE |
| `V008_TEST_ASSET_AUTHORITY_SCOPE.md` | V-008 test asset authority scope | 📦 ARCHIVE |

---

## 17. `docs/governance/` — Soak Reports

Type: Evidence. Lifecycle: Historical. Authority: Informational.

| File | Date | Action |
|------|------|--------|
| `soak_report_202605290811.md` | 2026-05-29 08:12 | 📦 ARCHIVE |
| `soak_report_202605290813.md` | 2026-05-29 08:13 | 📦 ARCHIVE |
| `soak_report_202605290814.md` | 2026-05-29 08:14 | 📦 ARCHIVE |
| `soak_report_202605290817.md` | 2026-05-29 08:17 | 📦 ARCHIVE |
| `soak_report_202605290820.md` | 2026-05-29 08:20 | 📦 ARCHIVE |
| `soak_report_202605290825.md` | Unfilled template (literal `$(date ...)`) | 🗑️ DELETE |

---

## 18. `docs/migrations/`

Type: Governance. Lifecycle: Historical (migrations complete). Authority: Superseded.

| File | What it documented | Action |
|------|-------------------|--------|
| `phase5_aggregation_extraction.md` | Phase 5 aggregation extraction migration | 📦 ARCHIVE |
| `phase5_reporting_extraction.md` | Phase 5 reporting.rs extraction migration | 📦 ARCHIVE |

---

## 19. `docs/platform/`

Type: Specification. Product: ChronoSentiment. Lifecycle: Operational. Authority: Normative. Owner: Engineering.

| File | What it defines | Action |
|------|----------------|--------|
| `api.md` | API spec — /orders, /sessions, /ga, /analytics | ✅ KEEP |
| `db.sql` | Database schema | ✅ KEEP |
| `websocket.md` | WebSocket protocol | ✅ KEEP |

---

## 20. `docs/releases/`

Type: Evidence. Product: ChronoSentiment. Lifecycle: Operational. Authority: Informational. Owner: Engineering.

| File | What it records | Action |
|------|----------------|--------|
| `README.md` | Release manifest format and generation scripts | ✅ KEEP |
| `2026-05-28T164021Z_2ba395cf.json` | Release manifest 2026-05-28 | ✅ KEEP |
| `2026-05-29T011530Z_2ba395cf.json` | Release manifest 2026-05-29 01:15 | ✅ KEEP |
| `2026-05-29T012507Z_2ba395cf.json` | Release manifest 2026-05-29 01:25 | ✅ KEEP |
| `2026-05-29T012538Z_2ba395cf.json` | Release manifest 2026-05-29 01:25 (second) | ✅ KEEP |

---

## 21. `docs/risk/`

| File | Type | Product | Lifecycle | Action |
|------|------|---------|-----------|--------|
| `risk_register.md` | Governance | Shared | Operational | ✅ KEEP |

---

## 22. `docs/simulation/`

Type: Specification. Product: ChronoSentiment. Lifecycle: Operational. Authority: Normative. Owner: Engineering.

| File | What it defines | Action |
|------|----------------|--------|
| `ese.md` | Execution Simulation Engine specification | ✅ KEEP |
| `kernel.md` | Simulation Kernel Specification v1.0 (19 sections) | ✅ KEEP |
| `latency.md` | Latency model — regime-based latency | ✅ KEEP |

---

## 23. `docs/submission/`

Type: Evidence. Product: ChronoSentiment. Lifecycle: Operational. Authority: Informational. Owner: Programme.

| File | What it is | Action |
|------|-----------|--------|
| `architecture_pitch.md` | ChronoSentiment architecture pitch | ✅ KEEP |
| `qfth_executive_summary.md` | Deterministic Market Replay & Evaluation Infrastructure executive summary | ✅ KEEP |

---

## 24. `docs/testing/`

| File | Type | Product | Lifecycle | Action |
|------|------|---------|-----------|--------|
| `testing.md` | Specification | ChronoSentiment | Operational | ✅ KEEP |

---

## 25. `docs/ui/`

| File | Type | Product | Lifecycle | Action |
|------|------|---------|-----------|--------|
| `uiux.md` | Specification | ChronoSentiment | **Historical** | Superseded | Engineering | 📦 ARCHIVE → `docs/archive/engineering/` (Owner Decision M resolved — UI synchronization history for Phase 3 React prototype; observer-first and backend-authority principles remain canonical but are already represented in Event Flow Specification, Service Boundary Definition, SRS, and Architecture Evolution; component inventory and pass-by-pass history belong to exploratory prototype lineage superseded by current product direction) |

---

## 26. `docs/api/`

All four subdirectories are **empty** (confirmed July 2026). No files present in `cli/`, `grpc/`, `openapi/`, or `rust/`.

| Directory | Contents | Action |
|-----------|---------|--------|
| `docs/api/cli/` | Empty | 🗑️ DELETE empty directory or reserve for future CLI spec |
| `docs/api/grpc/` | Empty | 🗑️ DELETE empty directory or reserve for future gRPC spec |
| `docs/api/openapi/` | Empty | 🗑️ DELETE empty directory or reserve for future OpenAPI spec |
| `docs/api/rust/` | Empty | 🗑️ DELETE empty directory or reserve for future Rust API spec |

*(Owner Decision N resolved — see Section 39)*

---
## 27. Document Dependency Graph

Authority layers from highest to lowest. Documents at each layer are governed by the authority defined in the layers above. Historical derivation is described separately in the Knowledge Flow Graph (Section 31).

```
L1 — Constitutional (cannot be overridden)
├── docs/constitution/architecture.md
├── docs/constitution/AUTHORITY_GEOMETRY.md
├── docs/constitution/GENERIC_LAYER.md
├── docs/constitution/OBSERVATORY_AUTHORITY.md
├── docs/constitution/OPERATIONAL_SOVEREIGNTY.md
├── docs/constitution/OPTIMIZATION_BOUNDARY.md
├── docs/constitution/SEMANTIC_REALIZATION_GATES.md
├── docs/constitution/topology.md
└── docs/constitution/warning_policy.md

L2 — Programme Baseline (frozen; governs product definition)
├── docs/CHRONOSENTIMENT_PRD_V1.md
├── docs/CHRONOSENTIMENT_PRODUCT_CONCEPT.md
├── docs/CHRONOSENTIMENT_EVIDENCE_PROGRAMME.md
├── docs/ARCHITECTURE_EVOLUTION.md          ← grounded in L3
└── docs/CORALYS_EVIDENCE_GOVERNANCE_STANDARD.md

L3 — Engineering Baseline (frozen; constrains implementation)
├── docs/CODEBASE_ASSESSMENT.md             ← ground truth for L2
├── docs/Service_Boundary_Definition.md
├── docs/System_Guarantee.md
├── docs/governance/V006_SERIALIZATION_LAW_DECLARATION.md
└── docs/governance/V001_API_ADMISSIBILITY_CONTRACT.md

L4 — Active Contracts (operational; must not contradict L1–L3)
├── docs/contracts/UI_API_CONTRACT_v1.md
├── docs/contracts/REPLAY_ATTESTATION_CONTRACT_v1.md
├── docs/contracts/SCENARIO_DOMAIN_CONTRACT_v1.md
├── docs/contracts/TIMELINE_VIEW_CONTRACT.md
├── docs/contracts/TRADE_INSPECTOR_CONTRACT.md
└── docs/contracts/*.md (26 engineering contracts — to be migrated from docs/research/)

L5 — Specifications (normative; implement L4 contracts)
├── docs/simulation/kernel.md
├── docs/simulation/ese.md
├── docs/simulation/latency.md
├── docs/capabilities/*.md (5 files)
├── docs/platform/api.md, db.sql, websocket.md
└── docs/BENCHMARK-REFERENCE-SPECIFICATION-v1.0.md

L6 — Research Baseline (informational; feeds product decisions)
└── docs/research/CS-R-000 through CS-R-015 (16 documents)

L7 — Governance Records (operational; track decisions and state)
├── docs/governance/governance_index.md
├── docs/governance/governance_ledger_v011.md
├── docs/governance/semantic_registry.md
├── docs/governance/event_taxonomy.md
├── docs/governance/ontology.md
└── docs/risk/risk_register.md

L8 — Evidence and Certification (observational; cannot override L1–L7)
├── docs/certification/*.md (8 files + 10 JSON artifacts)
├── docs/releases/*.json (4 release manifests)
└── docs/governance/soak_*.md (observational soak results)

L9 — Historical / Superseded (archived; informational only)
├── docs/archive/*.md (9 files)
├── docs/governance/V001_* through V008_* (18 decision records)
├── docs/migrations/*.md (2 files)
└── docs/PRD_v3_3.md, PHASE1_*.md
```

---

## 28. Standards Classification

Documents that are reusable cross-product governance standards vs. product-specific content.

### Reusable Cross-Product Standards (Coralys Platform)

These documents define governance that applies to any product built on Coralys:

- `docs/CORALYS_EVIDENCE_GOVERNANCE_STANDARD.md` — evidence governance standard
- `docs/ARCHITECTURE_EVOLUTION.md` — architectural principles
- `docs/constitution/` (all 10 files) — constitutional authority
- `docs/governance/governance_index.md` — governance index
- `docs/governance/semantic_registry.md` — canonical semantic registry
- `docs/governance/event_taxonomy.md` — canonical event taxonomy
- `docs/governance/ontology.md` — system ontology
- `docs/PLATFORM_CRATE_RESPONSIBILITIES.md` — crate ownership
- `docs/BENCHMARK-GOVERNANCE.md` — benchmark governance
- `docs/risk/risk_register.md` — risk register

### Product-Specific: ChronoSentiment

- `docs/CHRONOSENTIMENT_PRD_V1.md`, `CHRONOSENTIMENT_PRODUCT_CONCEPT.md`, `CHRONOSENTIMENT_EVIDENCE_PROGRAMME.md`
- `docs/research/CS-R-000` through `CS-R-015`
- `docs/simulation/`, `docs/certification/`, `docs/contracts/`, `docs/submission/`
- `docs/System_Guarantee.md`, `docs/OPERATIONS.md`

### Product-Specific: UltraCrew / INRC

- `docs/INRC_DEMO_GUIDE.md`, `docs/INRC_PRODUCT_EVIDENCE_PROGRAMME.md`
- `docs/P001_MILESTONE.md`, `docs/P001_PILOT_RUNBOOK.md`
- `docs/BENCHMARK_RESULTS.md`, `docs/ULTRACREW_WORKFORCE_EVIDENCE.md`
- `docs/sunair_pilot_guide.md`, `docs/sunair_sales_playbook.md`


## 29. Logical vs Physical Architecture

**The dependency graph in this section is logical. The filesystem is physical. They are independent.**

The directory structure (`docs/constitution/`, `docs/research/`, `docs/governance/`) reflects how files were created and organised over time. It does not imply authority. A file in `docs/research/` is not necessarily lower authority than a file in `docs/constitution/`. Authority is determined by the classification in this survey, not by directory location.

Contributors must consult this survey — not the filesystem — to determine the authority of a document.

---

## 30. Authority Graph

Who governs whom. This graph defines override relationships: a document at a higher level cannot be contradicted by a document at a lower level. This is a normative constraint, not a historical derivation.

```
L1 — Constitutional (cannot be overridden by anything)
     docs/constitution/ (all 10 files)

          |
          v

L2 — Programme Baseline (frozen; governs product definition)
     docs/CHRONOSENTIMENT_PRD_V1.md
     docs/CHRONOSENTIMENT_PRODUCT_CONCEPT.md
     docs/CHRONOSENTIMENT_EVIDENCE_PROGRAMME.md
     docs/ARCHITECTURE_EVOLUTION.md
     docs/CORALYS_EVIDENCE_GOVERNANCE_STANDARD.md

          |
          v

L3 — Engineering Baseline (frozen; constrains implementation)
     docs/CODEBASE_ASSESSMENT.md
     docs/Service_Boundary_Definition.md
     docs/System_Guarantee.md
     docs/governance/V006_SERIALIZATION_LAW_DECLARATION.md
     docs/governance/V001_API_ADMISSIBILITY_CONTRACT.md

          |
          v

L4 — Active Contracts (operational; must not contradict L1-L3)
     docs/contracts/*.md (5 files)
     docs/contracts/*.md (26 engineering contracts — to be migrated from docs/research/; Owner Decision O resolved)

          |
          v

L5 — Specifications (normative; implement L4 contracts)
     docs/simulation/*.md
     docs/capabilities/*.md
     docs/platform/api.md, db.sql, websocket.md
     docs/BENCHMARK-REFERENCE-SPECIFICATION-v1.0.md

          |
          v

L6 — Governance Records (operational; track decisions and state)
     docs/governance/governance_index.md
     docs/governance/semantic_registry.md
     docs/governance/event_taxonomy.md
     docs/governance/ontology.md
     docs/risk/risk_register.md

          |
          v

L7 — Evidence and Certification (observational; cannot override L1-L6)
     docs/certification/*.md and *.json
     docs/releases/*.json
     docs/governance/soak_*.md

          |
          v

L8 — Research Baseline (informational; feeds product decisions but does not govern them)
     docs/research/CS-R-000 through CS-R-015

          |
          v

L9 — Historical / Superseded (archived; informational only)
     docs/archive/*.md
     docs/governance/V001_* through V008_* (decision records)
     docs/migrations/*.md
     docs/PRD_v3_3.md, PHASE1_*.md
```

**Note on ARCHITECTURE_EVOLUTION.md:** This document was written after `CODEBASE_ASSESSMENT.md` and is grounded in it (historical derivation). However, `ARCHITECTURE_EVOLUTION.md` is the normative authority for architectural decisions going forward. The assessment is the evidence base; the evolution document is the governing baseline. These are different relationships — see Section 31.

---

## 31. Knowledge Flow Graph

How ideas evolve from research to implementation. This graph defines temporal and epistemic dependencies — which documents informed which — not authority. A document earlier in this flow is not necessarily lower authority.

```
Secondary Research (CS-R series)
     CS-R-001 through CS-R-015
          |
          | informs
          v
Product Definition
     CHRONOSENTIMENT_PRD_V1.md
     CHRONOSENTIMENT_PRODUCT_CONCEPT.md
          |
          | grounds
          v
Codebase Assessment
     CODEBASE_ASSESSMENT.md
          |
          | informs
          v
Architecture Governance
     ARCHITECTURE_EVOLUTION.md
     Service_Boundary_Definition.md
          |
          | specifies
          v
Engineering Contracts
     docs/contracts/*.md
     docs/research/*_CONTRACT_v1.md
          |
          | implements
          v
Specifications
     docs/simulation/*.md
     docs/capabilities/*.md
     docs/platform/*.md
          |
          | validates
          v
Certification and Evidence
     docs/certification/*.md
     docs/releases/*.json
     docs/governance/soak_*.md
          |
          | informs (feedback loop)
          v
Future Research and Revision
     Phase 1B customer validation
     Next CS-R review cycle
```

---

## 32. Ownership Dimensions

Three ownership dimensions apply to documents in this repository. They are independent.

| Dimension | Values | Description |
|-----------|--------|-------------|
| **Platform** | Coralys | Documents that govern the platform regardless of which product is built on it. Reusable across all products. |
| **Product** | ChronoSentiment · UltraCrew/INRC | Documents that govern a specific product. Not reusable across products without adaptation. |
| **Programme** | P-001 Pilot · Phase 1A Research | Documents that govern a time-bounded programme of work. Become historical when the programme closes. |

### Platform Documents (Coralys)

These apply to any product built on Coralys. They are not product-specific.

- `docs/constitution/` (all 10 files)
- `docs/CORALYS_EVIDENCE_GOVERNANCE_STANDARD.md`
- `docs/ARCHITECTURE_EVOLUTION.md`
- `docs/ARCHITECTURE_GLOSSARY.md`
- `docs/PLATFORM_CRATE_RESPONSIBILITIES.md`
- `docs/BENCHMARK-GOVERNANCE.md`
- `docs/BENCHMARK-REFERENCE-SPECIFICATION-v1.0.md`
- `docs/governance/governance_index.md`, `semantic_registry.md`, `event_taxonomy.md`, `ontology.md`
- `docs/risk/risk_register.md`
- `docs/capabilities/*.md` (5 files)
- `docs/research/*_CONTRACT_v1.md` (26 engineering contracts — **logical home: `docs/contracts/`**; Owner Decision O resolved; migration pending; during migration perform light classification review to separate permanent engineering contracts from temporary migration/transition documents such as `ARCHIVE_CURATION_CONTRACT_v1.md`, `RUST_PORT_CONTRACT_v1.md`, `REPAIR_LABORATORY_ISOLATION_PROTOCOL_v1.md`)

### Product Documents: ChronoSentiment

- `docs/CHRONOSENTIMENT_PRD_V1.md`, `CHRONOSENTIMENT_PRODUCT_CONCEPT.md`, `CHRONOSENTIMENT_EVIDENCE_PROGRAMME.md`
- `docs/research/CS-R-000` through `CS-R-015`
- `docs/simulation/`, `docs/certification/`, `docs/contracts/`, `docs/submission/`
- `docs/System_Guarantee.md`, `docs/OPERATIONS.md`
- `docs/platform/api.md`, `db.sql`, `websocket.md`

### Product Documents: UltraCrew / INRC

- `docs/INRC_DEMO_GUIDE.md`, `docs/INRC_PRODUCT_EVIDENCE_PROGRAMME.md`
- `docs/BENCHMARK_RESULTS.md`, `docs/ULTRACREW_WORKFORCE_EVIDENCE.md`
- `docs/sunair_pilot_guide.md`, `docs/sunair_sales_playbook.md`

### Programme Documents: P-001 Pilot

- `docs/P001_MILESTONE.md`, `docs/P001_PILOT_RUNBOOK.md`

---

## 33. Lifecycle Transition Vocabulary

Standard states and permitted transitions. All documents must be in exactly one state at any time.

```
Draft --> Baseline --> Frozen --> Operational --> Historical --> Archived --> Retired
                          |
                          +--> (minor corrections only — see Freeze Policy below)
```

| State | Definition | Permitted changes |
|-------|-----------|-------------------|
| **Draft** | Being written; not yet authoritative | Any |
| **Baseline** | Approved as the current reference; may be revised | Substantive changes with owner approval |
| **Frozen** | Locked; governs downstream documents | Minor corrections only (see Freeze Policy) |
| **Operational** | Active reference for day-to-day engineering | Updates as the system evolves |
| **Historical** | Superseded but retained for lineage | No changes; read-only |
| **Archived** | Moved to `docs/archive/`; no longer referenced | No changes; read-only |
| **Retired** | Formally withdrawn; no longer valid | No changes; marked with retirement notice |

### Freeze Policy

A **Frozen** document may only be changed for the following reasons. All other changes require a constitutional amendment (see Section 37).

| Permitted change | Examples |
|-----------------|---------|
| Spelling corrections | Typos that do not change meaning |
| Broken link repairs | Update a URL or file path that has moved |
| Cross-reference updates | Update a reference to a document that has been renamed |
| Version metadata | Update the "Last reviewed" date after a scheduled review |

**Not permitted on a Frozen document without amendment:**
- Adding, removing, or changing normative requirements
- Changing authority relationships
- Changing ownership or lifecycle state
- Adding new sections

---

## 34. Standardised Review Trigger Vocabulary

Review triggers must use one of the following standard types. Document-specific triggers are expressed as instances of these types.

| Trigger Type | Definition | Example |
|-------------|-----------|---------|
| **Constitutional amendment** | A change to L1 constitutional documents | `docs/constitution/` files |
| **Programme milestone** | Completion of a defined programme phase | Phase 1B completion, P-001 pilot close |
| **Material external event** | A significant external change that affects the document's subject matter | New regulation, major competitor announcement, platform acquisition |
| **Scheduled review** | A calendar-based review regardless of external events | Annual review, semi-annual review |
| **Release** | A software release that changes the system the document describes | New Coralys release, new API version |
| **Owner change** | The document's owner changes | Personnel transition, team restructure |

---

## 35. Evidence Sub-Types

The "Evidence" document type is subdivided as follows to prevent conflation.

| Sub-type | Definition | Examples in this repository |
|----------|-----------|----------------------------|
| **Validation evidence** | Demonstrates that a system behaves as specified | `docs/certification/*.md`, `docs/certification/certified_artifacts/*.json` |
| **Operational evidence** | Records the operational state of a running system | `docs/governance/soak_*.md`, `docs/releases/*.json`, `docs/governance/OBSERVATIONAL_SOAK_RESULTS.md` |
| **Certification evidence** | Formal certification of a specific property | `docs/certification/PHASE6_EXPLAINABILITY_CERTIFICATION.json`, `docs/certification/replay_certification_log.md` |
| **Assessment evidence** | Structured evaluation of a system or codebase | `docs/CODEBASE_ASSESSMENT.md`, `docs/BENCHMARK_RESULTS.md` |

---

## 36. Archive Sub-Classification

The `docs/archive/` directory should be sub-classified as follows. Current contents are mixed; this is the target structure.

```
docs/archive/
    governance/       <- V-series decision records, soak reports, migration records
    products/         <- Superseded PRDs, product concepts, scope documents
    engineering/      <- Superseded architecture specs, SRS, SDS, PSD
    research/         <- Session handoffs, obsolete research notes
```

| Current location | Target location |
|-----------------|----------------|
| `docs/governance/V001_*` through `V008_*` (18 files) | `docs/archive/governance/` |
| `docs/governance/soak_report_202605290811/13/14/17/20.md` | `docs/archive/governance/` |
| `docs/migrations/phase5_*.md` | `docs/archive/governance/` |
| `docs/PRD_v3_3.md` | `docs/archive/products/` |
| `docs/PHASE1_DOMAIN_COMPARISON.md`, `PHASE1_GOVERNANCE_CHECKPOINT.md` | `docs/archive/products/` |
| `docs/archive/PSD_v1_1.md`, `SDS_v2_0.md` | `docs/archive/engineering/` |
| `docs/archive/Execution_Thinking_Book.md`, `kernel_truths.md` | `docs/archive/engineering/` |
| `docs/archive/Market_Regine_External_Influence_specification.md` | `docs/archive/engineering/` |
| `docs/archive/mocrostructure_dictionary.md` | `docs/archive/engineering/` (rename to `microstructure_dictionary.md`) |
| `docs/archive/short_side_*.md` | `docs/archive/research/` |
| `docs/research/HANDOFF.md` | `docs/archive/research/` |

---

## 37. Constitutional Amendment Process

L1 (Constitutional) and L2 (Programme Baseline) documents are Frozen. They may only be changed through the following process.

### L1 Amendment (Constitutional documents in `docs/constitution/`)

1. **Proposal** — Any contributor may propose an amendment by creating a document in `docs/governance/` named `CONSTITUTIONAL_AMENDMENT_PROPOSAL_<topic>.md`. The proposal must state: the specific change, the reason, and the documents affected.
2. **Review** — The proposal must be reviewed by the Programme owner. Review period: minimum 5 working days.
3. **Approval** — Approval requires explicit sign-off from the Programme owner. Approval is recorded in the proposal document.
4. **Ratification** — The approved change is applied to the constitutional document. The version number is incremented. The proposal document is moved to `docs/archive/governance/`.
5. **Cascade review** — All L2–L5 documents that reference the amended constitutional document must be reviewed for consistency within 30 days.

### L2 Amendment (Programme Baseline documents)

1. **Proposal** — Create a document in `docs/governance/` named `BASELINE_AMENDMENT_PROPOSAL_<topic>.md`. State the specific change, reason, and downstream impact.
2. **Review** — Review period: minimum 3 working days.
3. **Approval** — Approval requires explicit sign-off from the Programme owner.
4. **Ratification** — The approved change is applied. Version number incremented. Proposal archived.
5. **Cascade review** — Downstream documents reviewed within 14 days.

### What does not require an amendment

- Spelling corrections, broken link repairs, cross-reference updates, version metadata updates (see Freeze Policy, Section 29).
- Changes to L3–L9 documents (these follow their own lifecycle rules).

---

## 38. Survey Governance

This survey is itself a governed artefact.

| Field | Value |
|-------|-------|
| **Document Status** | Baseline v3.0 |
| **Lifecycle State** | Sections 27–38 (governance framework): **Frozen**. Sections 1–26 (file inventory) and Section 39 (rationalisation backlog): **Operational** (updated as the repository evolves). |
| **Authority** | Normative (governs cleanup and classification decisions) |
| **Owner** | Programme |
| **Product** | Shared (Coralys / ChronoSentiment / UltraCrew) |
| **Review Trigger** | Scheduled (semi-annual) · Material external event (major repository restructure) · Programme milestone (Phase 1B completion) · Recurring operational friction observed during cleanup backlog execution |
| **Freeze Policy** | Sections 27–38 are frozen as the Documentation Governance Baseline. Changes require a governance amendment (see Section 37). Sections 1–26 and Section 39 may be updated by any contributor without amendment. |
| **Amendment Process** | Governance framework (Sections 27–38): owner approval required, minimum 3 working days review, 14-day cascade. File inventory (Sections 1–26) and rationalisation backlog (Section 39): any contributor, no approval required. |

---

## 39. Repository Rationalisation Backlog

### Immediate — No Owner Decision Required

| # | Action | File(s) |
|---|--------|---------|
| 1 | DELETE | `docs/governance/soak_report_202605290825.md` — unfilled template |
| 2 | RENAME | `docs/archive/mocrostructure_dictionary.md` → `microstructure_dictionary.md` |
| 3 | ARCHIVE to `docs/archive/governance/` | All 18 V-series decision records (`V001_*`, `V003_*`, `V006_*` except `V006_SERIALIZATION_LAW_DECLARATION.md`, `V007_*`, `V008_*`) |
| 4 | ARCHIVE to `docs/archive/governance/` | Five soak reports (`soak_report_202605290811/13/14/17/20.md`) |
| 5 | ARCHIVE to `docs/archive/governance/` | `docs/migrations/phase5_*.md` |
| 6 | ARCHIVE to `docs/archive/research/` | `docs/research/HANDOFF.md` |
| 7 | ARCHIVE to `docs/archive/products/` | `docs/PRD_v3_3.md` |
| 8 | ARCHIVE to `docs/archive/products/` | `docs/PHASE1_DOMAIN_COMPARISON.md`, `docs/PHASE1_GOVERNANCE_CHECKPOINT.md` |
| 9 | MOVE to `docs/archive/engineering/` | `docs/archive/PSD_v1_1.md`, `SDS_v2_0.md`, `Execution_Thinking_Book.md`, `kernel_truths.md`, `Market_Regine_External_Influence_specification.md` |
| 10 | MOVE to `docs/archive/research/` | `docs/archive/short_side_*.md` |

### Owner Decision Required

| # | Question | File(s) |
|---|---------|---------|
| A ✅ | **RESOLVED** — `Backend_Architecture_Blueprint.md` is superseded by `ARCHITECTURE_EVOLUTION.md`. Reclassified as **Historical / Superseded**. Archive to `docs/archive/engineering/`. Retains value as ChronoSentiment conceptual design lineage but no longer governs architecture. | `docs/Backend_Architecture_Blueprint.md` |
| B ✅ | **RESOLVED** — `CODEBASE_ARCHITECTURE_ASSESSMENT.md` is a transitional document whose responsibilities have been split: codebase observations → `CODEBASE_ASSESSMENT.md`; architectural principles, decisions, invariants → `ARCHITECTURE_EVOLUTION.md`. No unique authoritative content remains. Archive to `docs/archive/engineering/`. | `docs/CODEBASE_ARCHITECTURE_ASSESSMENT.md` |
| C ✅ | **RESOLVED** — `Event_Flow_Specification.md` is **not** superseded by `Service_Boundary_Definition.md`. They are complementary: Event Flow defines runtime behaviour (how the system executes); Service Boundary defines architectural authority (who may do what). Both documents are retained as active specifications. | `docs/Event_Flow_Specification.md` |
| D ✅ | **RESOLVED** — `SRS_v1_6.md` is **not** superseded by `ARCHITECTURE_EVOLUTION.md`. They operate at different governance levels: SRS defines product requirements (*what ChronoSentiment must do*); Architecture Evolution defines platform constitutional governance (*how Coralys must be structured*). `ARCHITECTURE_EVOLUTION.md` constrains implementation but does not replace the product specification. Both retained. | `docs/SRS_v1_6.md` |
| E ✅ | **RESOLVED** — `MVP_Scope_Document_v2_1.md` is **not** superseded by `CHRONOSENTIMENT_PRD_V1.md`. PRD defines the overall product (market, customers, positioning, commercial scope); MVP Scope defines the engineering implementation boundaries for the execution-validation capability. They are complementary. ⚠️ **Action required:** MVP Scope should be reconciled with the PRD's broader Financial Decision Intelligence Platform framing — execution validation is one capability, not the entire product. | `docs/MVP_Scope_Document_v2_1.md` |
| F ✅ | **RESOLVED** — `PHASE2_DECISION_PROTOCOL.md` is an **active governance procedure**, not a completed record. It defines how Phase 2 authority reductions must be carried out (comparison against sealed baseline `881f4141`, classification, replay validation, lineage recording). Phase 2 targets from `AUTHORITY_MAP.md` are not yet fully processed. Document remains Operational until a Phase 2 Completion checkpoint is issued. | `docs/PHASE2_DECISION_PROTOCOL.md` |
| G ✅ | **RESOLVED** — `chrono_sentiment_full_system.md` is superseded. It was a monolithic early-stage design document mixing product positioning, architecture, algorithm, implementation, and operator guidance. Its authoritative content has been redistributed into: `CHRONOSENTIMENT_PRD_V1.md` (product), `SRS_v1_6.md` (requirements), `Event_Flow_Specification.md` (execution), `Service_Boundary_Definition.md` (service governance), `MVP_Scope_Document_v2_1.md` (MVP scope). Archive to `docs/archive/engineering/`. | `docs/research/chrono_sentiment_full_system.md` |
| H ✅ | **RESOLVED** — `chrono_sentiment_user_guide.md` belongs to an earlier product phase (ChronoSentiment as Institutional Live Inference Engine: signal generation, consensus voting, elite genomes, CLI operator dashboard). Current product is Financial Decision Intelligence Platform (research workspace, decision timeline, recommendation engine, explainability, governance). Archive to `docs/archive/engineering/` as historical product-phase documentation. | `docs/research/chrono_sentiment_user_guide.md` |
| I ✅ | **RESOLVED** — `DISCREPANCY_REPORT.md` is a historical transition document. Its principal conclusions (product identity, observer-first philosophy, service boundaries, replay authority, backend authority) have been absorbed into PRD, SRS, Event Flow Specification, Service Boundary Definition, and Architecture Evolution. Remaining items are prototype UI implementation details; the report itself recommended superseding the prototype rather than incrementally fixing it. Archive to `docs/archive/engineering/`. | `docs/research/DISCREPANCY_REPORT.md` |
| J ⚠️ | **DEFERRED** — Rust port completion cannot be confirmed from the contract alone. `RUST_PORT_CONTRACT_v1.md` defines migration requirements (Python frozen, Rust modules become canonical: `core/src/topology.rs`, `core/src/cognition.rs`, `core/src/morphology.rs`) but contains no implementation evidence, parity validation, or completion declaration. Remains active migration governance. Archive trigger: a subsequent milestone or governance document formally closing the migration. | `docs/research/RUST_PORT_CONTRACT_v1.md` |
| K ✅ | **RESOLVED** — Lane 4 is no longer active. It was a temporary calibration activity (observational audit of experimental/research surfaces: containment, labeling, classification only; no remediation). Its purpose has been superseded by the constitutional authority model, architecture evolution, and Phase 1/2 governance documents. Archive to `docs/archive/governance/` as completed governance scope. Retains value as lineage explaining research artifact classification decisions. | `docs/governance/LANE4_EXPERIMENTAL_CONSUMER_LABELING_SCOPE.md` |
| L ✅ | **RESOLVED** — Transition is **not complete**. `transitional_artifacts.md` remains an active operational tracking document. Registry shows 10 of 11 artifacts still ACTIVE; sunset conditions for ARTIFACT-005 through ARTIFACT-011 (NarrativeEngine, ReplayEngine, CertificationEngine, causal lineage hash chain, backend divergence/confidence emission) are not yet satisfied. Archive trigger: all artifacts reach ELIMINATED status. | `docs/governance/transitional_artifacts.md` |
| M ✅ | **RESOLVED** — `uiux.md` is not current as an active architecture document. Observer-first and backend-authority principles are canonical but already represented in Event Flow Specification, Service Boundary Definition, SRS, and Architecture Evolution. Component inventory (RunGA, CompareStrategies, GlobalRanking, StrategyInspector) and pass-by-pass synchronization history (Passes 1–6) belong to the exploratory React prototype lineage. Archive to `docs/archive/engineering/` as UI Synchronization History (Phase 3). | `docs/ui/uiux.md` |
| N ✅ | **RESOLVED** — All four `docs/api/` subdirectories (`cli/`, `grpc/`, `openapi/`, `rust/`) are **empty**. No specification files present. Action: delete empty directories or reserve for future API specifications. No documents to classify. | `docs/api/cli/`, `docs/api/grpc/`, `docs/api/openapi/`, `docs/api/rust/` |
| O ✅ | **RESOLVED** — Engineering contracts (26 files, actual count) should move to **`docs/contracts/`**, not `docs/governance/contracts/`. These are platform engineering invariants (replay semantics, topology, morphology, persistence, signal interfaces, state coherence, etc.) — not programme governance artefacts. `docs/governance/` is reserved for authority maps, governance frameworks, decision protocols, and lifecycle policies. During migration, perform a light classification review: temporary migration/transition documents (`ARCHIVE_CURATION_CONTRACT_v1.md`, `RUST_PORT_CONTRACT_v1.md`, `REPAIR_LABORATORY_ISOLATION_PROTOCOL_v1.md`) should be archived or treated separately rather than moved wholesale as permanent contracts. | `docs/research/*_CONTRACT_v1.md` |

---

*Repository Survey v2.2 | July 2026 | Repository Documentation Governance Catalogue*
*Governed artefact — see Section 38 for ownership and review policy*
*End of Repository Survey*