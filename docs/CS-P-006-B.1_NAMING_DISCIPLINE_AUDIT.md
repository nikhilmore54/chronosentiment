# CS-P-006-B.1 — Naming-discipline audit

**Document type:** Inventory (no rename authorized)  
**Status:** Complete — classification only  
**Date:** 2026-08-14  
**Parent:** CS-P-006-B.1  
**Does not:** rename types, rewrite modules, break the B3/B4 evidence chain  

`.cursor/rules/chronosentiment-core.mdc`: keep the decide path a domain state machine; research chronology belongs in provenance.

Classes:

* **DOMAIN** — legitimate product/research-engine concept; should remain.
* **PROVENANCE** — identity of an artifact, dump, schema, or protocol; keep as metadata, not as the object's type name going forward.
* **RESEARCH WORKFLOW** — internal programme phase; should not be a core domain type name.
* **LEGACY** — candidate for later cleanup; do not rename in this freeze.

No broad rename is authorized by this document.

---

## New CS-P-006-B.1 code (this freeze)

| Identifier | Location | Class | Note |
|------------|----------|-------|------|
| `PartitionKind::{Development,Selection,Evaluation}` | `dataset_partition.rs` | DOMAIN | Experimental design roles, not programme IDs |
| `ChronologicalPartition`, `TimePartition` | `dataset_partition.rs` | DOMAIN | |
| `SearchOutcomeAccess` | `dataset_partition.rs` | DOMAIN | evolution / selection_feedback / forbidden |
| `CHRONOLOGICAL_PARTITION_HASH` | `csp006_protocol.rs` | PROVENANCE | Frozen identity of this experiment's partition |
| `TrainingProvenance.{train,validation,test}` | `policy_artifact.rs` | PROVENANCE | Frozen `csp006a.policy_artifact.1` field names; mapped from domain partitions |
| `docs/CS-P-006-B.1_*.md` | `/docs` | RESEARCH WORKFLOW | Protocol documents may use TRAIN / VALIDATION / TEST |
| `product_validation/CS-P-006/partition/` | evidence | PROVENANCE | Manifest + timestamp lists |

---

## Existing production / domain types

| Identifier | Location | Class | Note |
|------------|----------|-------|------|
| `TradingDecision`, `DecisionAction`, `DecisionPolicy` | `decision_support/` | DOMAIN | |
| `AssessmentProfile`, `FactorAvailability` | `reasoning/assessment.rs` | DOMAIN | |
| `PolicyArtifact`, `ArtifactDecisionPolicy` | `policy_artifact.rs` | DOMAIN | Consumption contract |
| `SplitWindow` | `policy_artifact.rs` | DOMAIN | Generic time window |
| `OBJECT_SCHEMA_VERSION = csp004.decision.1` | `decision_support/mod.rs` | PROVENANCE | Schema id, not a phase object |
| `REPLAY_PRODUCER = csp004.adapter.v0.1` | `replay.rs` | PROVENANCE | |
| `SCHEMA_VERSION = csp004.lab.0` | `laboratory.rs` | PROVENANCE | |
| `WalkForwardFold.{train_end,test_start,test_n}` | `laboratory.rs` | RESEARCH WORKFLOW | CS-P-004 lab fold language; later cleanup |
| `POLICY_ARTIFACT_SCHEMA_VERSION = csp006a.policy_artifact.1` | `policy_artifact.rs` | PROVENANCE | |

---

## B-series / gate / database names

| Identifier | Location | Class | Note |
|------------|----------|-------|------|
| `chrono_b3_test`, `chrono_b4_test` | binaries, sqlx tests | PROVENANCE | Certified DB names; renaming would break the evidence chain |
| `B4_DUMP_SHA256` / `b4_dump_sha256` | `csp002_b4_historical_run.rs`, protocol | PROVENANCE | |
| `audit_b4_coverage` | `csp006_protocol.rs` | RESEARCH WORKFLOW | Explicitly “B4 is not the 7-name universe” |
| `CERTIFIED_FIVE_INSTRUMENT_SNAPSHOT` | `csp006_protocol.rs` | PROVENANCE | |
| `csp002_b4_historical_run.rs` | `src/bin` | RESEARCH WORKFLOW | Product bins must start with `csp` (existing invariant) |
| `G_GATE_*`, `m5_*`, `m6_*` | `research/` feature-gated | RESEARCH WORKFLOW | Already quarantined |
| `phase_c_gate.sh` | removed; invariant test | LEGACY | Must stay absent |
| Comments “Not B5” | snapshot/enrichment | PROVENANCE | Guardrail against incrementing B-series |

---

## CS-P-006 module filenames

| Identifier | Location | Class | Note |
|------------|----------|-------|------|
| `csp006_protocol.rs` | `decision_support/` | RESEARCH WORKFLOW | Protocol constants; acceptable as a programme module, not a domain type |
| `csp006_snapshot.rs` | `decision_support/` | RESEARCH WORKFLOW | Snapshot builder |
| `csp006_research_snapshot` | `src/bin` | RESEARCH WORKFLOW | Forced `csp*` bin prefix |
| `dataset_partition.rs` | `decision_support/` | DOMAIN | B.1 uses this name on purpose |
| `policy_genome.rs` | `decision_support/` | DOMAIN | Rule-list genome Coralys evolves; not a phase type |
| `observation_value.rs` | `decision_support/` | DOMAIN | Observation-path fitness; rejects evaluation |
| `policy_discovery.rs` | `decision_support/` | RESEARCH WORKFLOW | Orchestrates evolution/selection; no `CoralysPhase` |
| `policy_handoff.rs` | `decision_support/` | RESEARCH WORKFLOW | ChronoSentiment holdout after seal |
| `csp006_policy_discovery` | `src/bin` | RESEARCH WORKFLOW | Forced `csp*` bin prefix |
| `search_observability.rs` | `decision_support/` | RESEARCH WORKFLOW | C.2 / C.2-O archive contract; not a phase type |
| `population_ecology.rs` | `decision_support/` | RESEARCH WORKFLOW | C.2-P analysis of Search #1 archive; not a phase type |
| `csp006_population_ecology` | `src/bin` | RESEARCH WORKFLOW | Forced `csp*` bin prefix; identity-gated replay |
| `recommendation_outcome.rs` | `decision_support/` | RESEARCH WORKFLOW | C.2-R sealed-artifact scorecard; not a phase type |
| `csp006_recommendation_outcome` | `src/bin` | RESEARCH WORKFLOW | Forced `csp*` bin prefix; does not evolve |
| `selection_decision_value.rs` | `decision_support/` | RESEARCH WORKFLOW | C.2-S selection/objective review; not a phase type |
| `csp006_selection_review` | `src/bin` | RESEARCH WORKFLOW | Forced `csp*` bin prefix; does not evolve |
| `decision_value_landscape.rs` | `decision_support/` | RESEARCH WORKFLOW | C.2-D measurement contract; not a phase type |
| `csp006_decision_value` | `src/bin` | RESEARCH WORKFLOW | Forced `csp*` bin prefix; does not evolve |
| `used_as_coralys_fitness` | `decision_value_landscape.rs` | RESEARCH WORKFLOW | Explicit false; C.2-D advantage is not fitness |
| `CS-P-006-M.1` | `docs/` | RESEARCH WORKFLOW | Decision-value specification; not a phase type |
| `CS-P-006-N` | `docs/` | RESEARCH WORKFLOW | Harness; not a search; C.3 not authorized |
| `csp006_decision_value_harness` | `src/bin` | RESEARCH WORKFLOW | Forced `csp*` bin prefix; does not evolve |
| `ProtocolValue` | `decision_value_harness.rs` | DOMAIN | M.1 scalar; not constructible from regret |
| `CS-P-006-C.3` | `docs/` | RESEARCH WORKFLOW | Protocol authorization; not a phase type; Search #2 not started |
| `DevelopmentValue` | `decision_value_fitness.rs` | DOMAIN | M.1 development-slice evaluator; not Search #1 |
| `score_decision_value` | `decision_value_fitness.rs` | DOMAIN | M.1 protocol V; rejects evaluation |
| `living_selection_pool` | `c3_implementation.rs` | DOMAIN | Unique living-slot identities; not offspring |
| `select_on_selection_value` | `c3_implementation.rs` | DOMAIN | Selection-slice M.1 score of living candidates |
| `evolve_on_development_value` | `c3_implementation.rs` | RESEARCH WORKFLOW | Hard-gated; Search #2 not run |
| `SEARCH_TWO_RUN_AUTHORIZED` | `c3_implementation.rs` | RESEARCH WORKFLOW | Explicit false |
| `csp006_c3_implementation` | `src/bin` | RESEARCH WORKFLOW | Forced `csp*` bin prefix; verification only |
| `CS-P-006-C.3-I` | `docs/` | RESEARCH WORKFLOW | Implementation gate; not a phase type; Search #2 not run |
| `c3_run.rs` | `decision_support/` | RESEARCH WORKFLOW | One Search #2 experiment; not a phase type |
| `csp006_c3_search` | `src/bin` | RESEARCH WORKFLOW | Forced `csp*` bin prefix; one complete run |
| `DecisionValueSearchEvidence` | `c3_run.rs` | DOMAIN | Development-value search record; not a phase type |
| `CS-P-006-C.3-R` | `docs/` | RESEARCH WORKFLOW | Run authorization; not permission to iterate |
| `c3_comparison.rs` | `decision_support/` | RESEARCH WORKFLOW | Sealed-artifact review; not a phase type |
| `csp006_c3_comparison` | `src/bin` | RESEARCH WORKFLOW | Forced `csp*` bin prefix; does not evolve |
| `CS-P-006-C.3-C` | `docs/` | RESEARCH WORKFLOW | Comparative review; Search #3 not authorized |
| `c3_rule_ecology.rs` | `decision_support/` | RESEARCH WORKFLOW | Live-rule diagnostic; not a phase type |
| `csp006_c3_rule_ecology` | `src/bin` | RESEARCH WORKFLOW | Forced `csp*` bin prefix; does not evolve |
| `candidate_research_artifact` | `c3_rule_ecology.rs` | PROVENANCE | Search #2 promotion status; not a strategy type |
| `CS-P-006-C.3-D` | `docs/` | RESEARCH WORKFLOW | Rule ecology; Search #3 not authorized |
| `c3_rule_persistence.rs` | `decision_support/` | RESEARCH WORKFLOW | Persistence diagnostic; not a phase type |
| `csp006_c3_rule_persistence` | `src/bin` | RESEARCH WORKFLOW | Forced `csp*` bin prefix; does not evolve |
| `CS-P-006-C.3-E` | `docs/` | RESEARCH WORKFLOW | Rule persistence; no pass threshold; Search #3 not authorized |
| `c3_state_landscape.rs` | `decision_support/` | RESEARCH WORKFLOW | State × action diagnostic; not a phase type |
| `csp006_c3_state_landscape` | `src/bin` | RESEARCH WORKFLOW | Forced `csp*` bin prefix; does not evolve |
| `CS-P-006-C.3-F` | `docs/` | RESEARCH WORKFLOW | State landscape; no product claim; Search #3 not authorized |
| `REGIME_PERSISTENCE_EXPERIMENT_AUTHORIZED` | `csp006_protocol.rs` | RESEARCH WORKFLOW | Explicit false; not a detector |
| `CS-P-006-C.3-G` | `docs/` | RESEARCH WORKFLOW | Question stated; experiment not authorized; Search #3 not authorized |
| `C3-002` | `observatory_registry.rs` | PROVENANCE | Paper-only product label for Search #2; not a strategy type |
| `Candidate C3-002` | `observatory_registry.rs` | PROVENANCE | Customer-facing research policy name; not Strategy v2 |
| `observatory_registry.rs` | `decision_support/` | RESEARCH WORKFLOW | P.1 registry; does not decide |
| `CS-P-006-P` | `docs/` | RESEARCH WORKFLOW | Decision Observatory protocol; evidence dashboard; no early peek |
| `CS-P-006-P.H` | `docs/` | RESEARCH WORKFLOW | Historical Observatory Replay; not C.3-G; no lookahead |
| `observatory_historical.rs` | `decision_support/` | RESEARCH WORKFLOW | Historical clock; same C3-002 decide path |
| `csp006_p_replay` | `src/bin` | RESEARCH WORKFLOW | Forced `csp*` bin prefix; does not mutate prospective |
| `HISTORICAL_REPLAY_PATH_KIND` | `observatory_historical.rs` | PROVENANCE | Separate ledger from prospective and the 91 |
| `CS-P-006-P.H.1` | `docs/` | RESEARCH WORKFLOW | Decision Evidence Engine; replay integrity ≠ strategy validation |
| `HORIZON_CALENDAR_BASIS` | `observatory_maturity.rs` | DOMAIN | TRADING_DAYS; Observatory 20D is 20 market sessions |
| `HORIZON_UNIT` | `observatory_maturity.rs` | DOMAIN | MARKET_SESSIONS |
| `SESSION_RESOLUTION_RULE` | `observatory_maturity.rs` | DOMAIN | Latest certified session ≤ requested clock |
| `TRADING_SESSION_HORIZON_AUTHORIZED` | `observatory_maturity.rs` | RESEARCH WORKFLOW | Observatory product contract; C3-002 unchanged |
| `CS-P-006-P.H.2` | `docs/` | RESEARCH WORKFLOW | Replay v1 market-session horizon; v0 archived |
| `CS-P-006-P.H.3` | `docs/` | RESEARCH WORKFLOW | Decision Evidence Dashboard; not a performance scoreboard |
| `CS-P-006-P.E` | `docs/` | RESEARCH WORKFLOW | Targeted execution; not P.7; target sealed at T |
| `CS-P-006-P.E.1` | `docs/` | RESEARCH WORKFLOW | Frozen evidence surface; Execution Contract v0 owns target_pct |
| `CS-P-006-P.E.2` | `docs/` | RESEARCH WORKFLOW | Frozen prospective lifecycle; not a 5% quality test |
| `CS-P-006-P.E.B` | `docs/` | RESEARCH WORKFLOW | Pointer to P.E.3 |
| `CS-P-006-P.E.3` | `docs/` | RESEARCH WORKFLOW | Coralys Target Discovery; specified not started; P.E.2 is the control |
| `SealedExecutionIntent` | `observatory_execution.rs` | DOMAIN | Execution intent at T; not a C3-002 field |
| `DecisionIntent` | `decision_intent.rs` | DOMAIN | Pairing helper; does not merge target into C3-002 |
| `TARGET_SOURCE_CORALYS` | `decision_intent.rs` | DOMAIN | coralys_state_at_t; search not authorized |
| `TriggerType` | `observatory_execution.rs` | DOMAIN | HIGH_REACHED / LOW_REACHED / GAP_THROUGH / SESSION_CLOSE / AMBIGUOUS |
| `observatory_live_execution.rs` | `decision_support/` | RESEARCH WORKFLOW | Next cohort with contract from T; AWAITING_NEXT_SESSION is valid |
| `csp006_p_live_execute` | `src/bin` | RESEARCH WORKFLOW | Forced `csp*` bin prefix; does not overwrite P.E.1 or 14-Aug |
| `EXECUTION_CONTRACT_LABEL` | `observatory_execution.rs` | DOMAIN | Product label “Execution Contract v0”; not a C3-002 field |
| `observatory_execution.rs` | `decision_support/` | RESEARCH WORKFLOW | OHLC first-exit; does not retune C3-002 |
| `csp006_p_execute` | `src/bin` | RESEARCH WORKFLOW | Forced `csp*` bin prefix; does not overwrite v0/v1/prospective |
| `EXECUTION_TARGET_PCT` | `observatory_execution.rs` | DOMAIN | Sealed +5% parameter of execution contract v0; not path-optimized |
| `ExitReason` | `observatory_execution.rs` | DOMAIN | TARGET / STOP / HORIZON / AMBIGUOUS / NO_TRADE / OBSERVING |
| `observatory_slice.rs` | `decision_support/` | RESEARCH WORKFLOW | P.3–P.7 sealed-then-measured path and product screens; not a phase type |
| `csp006_p_observatory` | `src/bin` | RESEARCH WORKFLOW | Forced `csp*` bin prefix; does not evolve |
| `SealedDecisionRecord` | `observatory_slice.rs` | DOMAIN | Immutable paper decision; outcomes forbidden |
| `ui_decision_status` | `observatory_slice.rs` | DOMAIN | Customer SEALED / OBSERVING / OBSERVED; not a rewrite |
| `observation_status` | `observatory_slice.rs` | DOMAIN | Append-only COMPLETED on the observation |
| `OBSERVATORY_P7_STARTED` | `observatory_slice.rs` | RESEARCH WORKFLOW | Product screens; not a live clock |
| `OBSERVATORY_PROSPECTIVE_STARTED` | `observatory_slice.rs` | RESEARCH WORKFLOW | Prospective paper clock; not CS-P-003 validation |
| `observatory_prospective.rs` | `decision_support/` | RESEARCH WORKFLOW | Live-clock C3-002 seal; outcomes forbidden at T |
| `csp006_p_prospective` | `src/bin` | RESEARCH WORKFLOW | Forced `csp*` bin prefix; does not observe |
| `PROSPECTIVE_PATH_KIND` | `observatory_prospective.rs` | PROVENANCE | Separate ledger from the historical 91 |
| `observatory_maturity.rs` | `decision_support/` | RESEARCH WORKFLOW | Countdown / OUTCOME DUE; not a detector |
| `csp006_p_observe` | `src/bin` | RESEARCH WORKFLOW | Refreshes countdown; does not peek while OBSERVING |
| `UI_STATUS_OUTCOME_DUE` | `observatory_maturity.rs` | DOMAIN | Window closed; observation not yet appended |
| `INTERMEDIATE_INTERPRETATION_AUTHORIZED` | `observatory_maturity.rs` | RESEARCH WORKFLOW | Explicit false |
| `UNIVERSE_EXPANSION_AUTHORIZED` | `observatory_maturity.rs` | RESEARCH WORKFLOW | Explicit false |

---

## Environment / configuration

| Identifier | Class | Note |
|------------|-------|------|
| `DATABASE_URL` matching `chrono_b3_test` / `chrono_b4_test` | PROVENANCE | Refuse-to-write guard |
| `B4_DUMP_SHA256` | PROVENANCE | CS-P-002 runner |
| `CHRONO_YAHOO_CACHE_DIR` | DOMAIN | Cache location |
| `G_GATE_OUT_DIR`, `G_GATE_DATASET` | RESEARCH WORKFLOW | Feature-gated research binaries |

---

## Later cleanup (not this freeze)

1. `TrainingProvenance` field names `train` / `validation` / `test` — schema amendment to `development` / `selection` / `evaluation` if a future artifact version is opened.
2. CS-P-004 `WalkForwardFold` train/test field names.
3. `csp*` binary filenames vs domain verbs (`historical_run`, `forward_session`).
4. `audit_b4_coverage` rename to `certified_five_name_dump_coverage` once callers are ready.

Do not perform those renames until a dedicated cleanup authorizes them. Certified hashes and dumps stay immutable.
