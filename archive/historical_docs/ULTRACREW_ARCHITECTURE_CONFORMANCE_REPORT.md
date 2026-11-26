# UltraCrew Architecture Conformance Report v1.0

> **Status**: Conformance Report v1.0 — frozen
> **Date**: 2026-07-20
> **Assessor**: Architecture Conformance Assessment against Architecture Baseline v1.0
> **Scope**: `adapters/ultracrew`, `services/ultracrew_server`, platform crate dependencies
> **Method**: Direct code inspection of all source files in `adapters/ultracrew/src/`, `services/ultracrew_server/Cargo.toml`, and platform crate manifests.

---

## Executive Summary

| Dimension | Score |
|---|---|
| Architecture Conformance | **8.5/10** |
| Product Completeness | **6/10** |
| Pilot Readiness | **7/10** |

| Area | Maturity |
|---|---|
| Architecture | Mature |
| Coralys Optimisation Platform | Mature |
| Product Engineering | Emerging |
| Workforce Operations Platform | In Progress |

UltraCrew has successfully validated the core scheduling architecture through implementation and benchmark execution. The remaining work is primarily product engineering — customer data integration, operational workflows, deployment, and pilot enablement — rather than architectural or optimisation research. Coralys is no longer the bottleneck. The engineering focus has shifted from proving the platform to productising UltraCrew.

The most important architectural elements are in place: platform separation, trait-based optimisation, domain adapters, public contracts, observability, and evolution pipeline. Most of the missing work is product completeness, not architectural non-conformance. These are not the same thing.

**UltraCrew's remaining engineering work is largely domain-independent.** Import, export, REST, logging, deployment, and workflow improvements benefit healthcare, manufacturing, retail, logistics, and hospitality equally. This is exactly what Coralys was designed to achieve: a platform whose product engineering investments compound across domains.

**Top strengths:**
- End-to-end optimization pipeline is implemented and INRC-II validated
- `coralys-moga` consumed correctly through trait interfaces (`FitnessEvaluator`, `MutationOperator`, `CrossoverOperator`, `GenomeFactory`, `Evaluated`, `Genome`)
- `coralys-ecology` integrated for fatigue/historical workload tracking
- `public_contracts.rs` provides a domain-independent `Scenario` / `ScheduleRequest` / `RescheduleRequest` API — the right abstraction boundary
- Constraint engine produces structured `ConstraintReport` with per-constraint scores, violated/satisfied lists, and warnings
- Recommendation engine generates actionable per-constraint recommendations
- Observatory telemetry captures per-generation fitness, diversity, and timing
- Disruption recovery (`RescheduleRequest` with `locked_assignments`) is implemented

**Top risks:**
- `ultracrew_server` imports `InrcScenario` directly — violates Principle 2 (products depend on interfaces, not implementations)
- Generic workforce layer (`src/workforce/`) is a 3-file stub — UltraCrew cannot yet accept non-INRC input without the INRC parser
- No REST API for the generic `ScheduleRequest` / `RescheduleRequest` contracts — the server exposes INRC-specific endpoints only
- `decision_intelligence.rs` is a 38-line stub — not a Decision Intelligence capability
- No schedule export, import, or publication capability — pilot requires schedule output in customer-usable format
- No authentication, multi-tenancy, or deployment configuration

---

## Architecture Conformance Matrix

| Area | Planned (Architecture Baseline v1.0) | Current State | Status | Evidence |
|---|---|---|---|---|
| Repository structure | `adapters/`, `services/`, platform crates | Present and correctly named | Conformant | Directory listing |
| Product layer separation | UltraCrew is a product, not the platform | `adapters/ultracrew` is correctly a product adapter | Conformant | `Cargo.toml` path structure |
| Platform dependency via interfaces | Products depend on platform interfaces, not algorithms | `coralys-moga` consumed via `FitnessEvaluator`, `MutationOperator`, `CrossoverOperator`, `GenomeFactory` traits | Conformant | `optimization.rs` lines 5, 256, 351, 378, 400 |
| coralys-moga usage | Optimization engine | Used correctly via trait interfaces | Implemented | `optimization.rs`, `lib.rs` |
| coralys-ecology usage | Workforce fatigue / historical workload | `WorkforceEcology` integrated in `ScheduleContext`, fatigue penalty in constraint engine | Implemented | `constraint_engine.rs` lines 110–114, `optimization.rs` line 142 |
| coralys-core usage | Decision lineage, evaluation results | Declared as dependency in `Cargo.toml` | Partial — dependency declared, integration depth unclear | `adapters/ultracrew/Cargo.toml` |
| coralys-matching usage | Bipartite matching | Declared as dependency; `inrc/bipartite_matching.rs` exists | Partial — used in INRC path only | `adapters/ultracrew/Cargo.toml`, `inrc/bipartite_matching.rs` |
| coralys-recommendation usage | Platform recommendation capability | `coralys-recommendation` declared in server `Cargo.toml` but UltraCrew has its own `recommendation.rs` | Divergent — parallel recommendation implementations | `services/ultracrew_server/Cargo.toml`, `adapters/ultracrew/src/recommendation.rs` |
| coralys-policy usage | Policy enforcement | Declared in server `Cargo.toml` | Not Used in product layer | `services/ultracrew_server/Cargo.toml` |
| coralys-decision usage | Decision models | Not declared in `adapters/ultracrew/Cargo.toml` | Not Used | `adapters/ultracrew/Cargo.toml` |
| coralys-simulation usage | Simulation capability | Not declared in `adapters/ultracrew/Cargo.toml` | Not Used | `adapters/ultracrew/Cargo.toml` |
| coralys-infrastructure usage | Infrastructure capability | Not declared in `adapters/ultracrew/Cargo.toml` | Not Used | `adapters/ultracrew/Cargo.toml` |
| Server decoupled from INRC types | `ultracrew_server` should not import `InrcScenario` | Server imports `ultracrew` (which contains INRC types); INRC-specific endpoints exist | Violation — known, recorded as P-4 in Architecture Baseline | `services/ultracrew_server/Cargo.toml`, `CODEBASE_ASSESSMENT.md` line 81 |
| Generic workforce layer | `src/workforce/` as functional generic layer | 3-file stub (`mod.rs`, `ecology_adapter.rs`, `workforce_metrics.rs`) | Partial — stub only | `adapters/ultracrew/src/workforce/` |
| Domain model faithfulness | Worker, Shift, Skill, Coverage, Contract, Roster | Worker, Shift, Skill implemented; Coverage, Contract, Roster absent from generic layer | Partial | `adapters/ultracrew/src/models.rs` |
| Execution contract | `public_contracts.rs` as domain-independent API | `Scenario`, `ScheduleRequest`, `RescheduleRequest`, `ValidateRequest` implemented | Implemented | `adapters/ultracrew/src/public_contracts.rs` |
| Disruption recovery | `RescheduleRequest` with locked assignments | Implemented — `locked_assignments` in `ScheduleContext`, `RescheduleRequest.to_context()` | Implemented | `public_contracts.rs` lines 59–98, `optimization.rs` lines 269–278 |
| Observability | Per-generation telemetry | `Observatory` captures generation telemetry, diversity probe, timing | Implemented | `optimization.rs` lines 60–136 |
| Recommendation engine | Actionable per-constraint recommendations | `RecommendationEngine` generates per-constraint recommendations with severity and action | Implemented | `recommendation.rs` |
| Decision intelligence | Scenario comparison, trade-off analysis | 38-line stub returning fitness metrics only | Stub | `decision_intelligence.rs` |
| REST API (generic contracts) | Endpoints for `ScheduleRequest`, `RescheduleRequest` | Server exposes INRC-specific endpoints; generic contract endpoints unclear | Partial | `services/ultracrew_server/src/` file listing |
| Schedule export | Customer-usable schedule output | `inrc/exporter.rs` exists for INRC format; no generic export | Partial — INRC only | `adapters/ultracrew/src/inrc/exporter.rs` |
| Authentication / multi-tenancy | Not specified in baseline | Not implemented | Not Implemented | No evidence in source |

---

## GTM Capability Matrix

| GTM Capability | Status | Evidence |
|---|---|---|
| **Workforce Planning and Scheduling** | | |
| Automated schedule generation | Implemented | `pipeline.rs` — end-to-end GA optimization |
| Constraint management (skill, overlap, rest, hours) | Implemented | `constraint_engine.rs` — HC1, HC2, HC3, Rest, SC1, SC2 |
| Fairness optimisation | Implemented | SC1 variance penalty + workload-balanced swap mutation |
| Multi-site / multi-team scheduling | Not Implemented | No site/team concept in domain model |
| Demand-driven capacity planning | Not Implemented | No demand/coverage concept in generic layer |
| Contract rule enforcement | Partial | HC3 (max hours) implemented; consecutive days, weekend rules absent |
| **Operational Decision Support** | | |
| Constraint audit trail | Implemented | `ConstraintReport.violated_constraints`, `constraint_scores` |
| Actionable recommendations | Implemented | `RecommendationEngine` — per-constraint severity and action |
| Trade-off analysis | Not Implemented | `decision_intelligence.rs` is a stub |
| What-if scenario analysis | Not Implemented | No scenario comparison capability |
| Cost comparison | Not Implemented | No cost model |
| **Day-of-Operations** | | |
| Disruption recovery | Implemented | `RescheduleRequest` with `locked_assignments` |
| Schedule publication | Not Implemented | No publication or distribution mechanism |
| Shift swap / leave handling | Not Implemented | No leave or availability model |
| Notifications | Not Implemented | No notification capability |
| **Workforce Visibility** | | |
| Optimization telemetry | Implemented | `Observatory` — per-generation fitness, diversity, timing |
| Labour cost / utilisation | Not Implemented | No cost model; utilisation derivable from assignments |
| Compliance reporting | Partial | Constraint violation counts available; no formatted report |
| Dashboards | Not Implemented | No UI integration for workforce visibility |
| **Enterprise Connectivity** | | |
| INRC2 data import | Implemented | `inrc/parser.rs` |
| Generic data import (HRMS, CSV) | Not Implemented | No generic import adapter |
| Payroll / attendance export | Not Implemented | No export beyond INRC format |

---

## Pilot Readiness Matrix

| Capability | Status | Notes |
|---|---|---|
| Schedule generation | Ready | End-to-end pipeline validated against INRC-II benchmark |
| Hard constraint enforcement | Ready | HC1 (skill), HC2 (overlap), HC3 (max hours), Rest (8h gap) |
| Soft constraint optimisation | Ready | SC1 (fairness), SC2 (fatigue) |
| Disruption recovery | Ready | `RescheduleRequest` with locked assignments implemented |
| Constraint audit trail | Ready | Per-constraint scores, violated/satisfied lists, warnings |
| Actionable recommendations | Ready | Per-constraint severity and recommended action |
| Optimization telemetry | Ready | Per-generation fitness, diversity, elapsed time |
| Generic data input (non-INRC) | Needs Work | `public_contracts.rs` API exists but no generic import adapter; customer data must be manually mapped to `ScheduleRequest` |
| Schedule export (customer format) | Needs Work | INRC exporter exists; generic CSV/JSON export needed for pilot |
| REST API (generic contracts) | Needs Work | Server exposes INRC endpoints; generic `ScheduleRequest` endpoint needed |
| Parallel validation support | Needs Work | No mechanism to run UltraCrew alongside existing process and compare outputs |
| Observability / logging | Needs Work | Observatory captures telemetry but no structured logging for pilot monitoring |
| Deployment configuration | Missing | No Docker, environment config, or deployment documentation |
| Authentication | Missing | No authentication mechanism |
| Error handling (production-grade) | Needs Work | `unwrap()` calls in hot paths (e.g. `optimization.rs` line 82, `constraint_engine.rs` line 53) |
| Multi-tenancy | Missing | No tenant isolation |

---

## Technical Debt

Prioritised by impact on pilot delivery.

**P1 — Blocks pilot delivery**

1. **No generic data import adapter.** The `public_contracts.rs` `ScheduleRequest` API is the right abstraction, but there is no mechanism to load customer data (CSV, JSON, HRMS export) into it. A pilot customer cannot use UltraCrew without manually constructing `ScheduleRequest` JSON. Evidence: `adapters/ultracrew/src/workforce/` is a 3-file stub; `inrc/parser.rs` is INRC-specific.

2. **No generic REST endpoint.** The server exposes INRC-specific endpoints. A pilot integration requires a REST endpoint accepting `ScheduleRequest` and returning `ScheduleSolution`. Evidence: `services/ultracrew_server/Cargo.toml` imports `ultracrew` (INRC types accessible); no generic endpoint confirmed in source listing.

3. **No generic schedule export.** `inrc/exporter.rs` produces INRC-format output. A pilot customer needs schedule output in a usable format (CSV, JSON, or integration with their existing system). Evidence: `adapters/ultracrew/src/inrc/exporter.rs` exists; no generic equivalent.

**P2 — Degrades pilot quality**

4. **No parallel validation support.** The Pilot Methodology requires running UltraCrew alongside the customer's existing process and comparing outputs. No mechanism exists for this. Evidence: no parallel run or comparison capability in source.

5. **`unwrap()` in hot paths.** `optimization.rs` line 82 (`expect("Invalid EvolutionConfig")`), `constraint_engine.rs` line 53 (`unwrap()` on assignment lookup). These will panic on malformed input rather than returning structured errors. Evidence: direct code inspection.

6. **No structured logging.** The `Observatory` telemetry is good for optimization analysis but a pilot needs operational logs (request received, optimization started, solution generated, errors). Evidence: no logging framework in source.

**P3 — Deployment (can be hosted by UltraCrew team for early pilot)**

7. **No deployment configuration.** No Docker, environment variable support, or deployment documentation. For an early pilot where UltraCrew hosts the application, this can be deferred. Evidence: no Dockerfile or deployment config in source.

8. **Authentication and multi-tenancy.** Not implemented. Classified as **post-pilot** unless the first customer specifically requires them. For a hosted pilot in a controlled environment, lightweight or no authentication is acceptable. Evidence: no authentication mechanism in source.

**P4 — Future extensibility**

9. **`ultracrew_server` imports INRC types.** Known violation recorded as P-4 in the Architecture Baseline. Does not block a pilot. Evidence: `CODEBASE_ASSESSMENT.md` line 81; `services/ultracrew_server/Cargo.toml`.

10. **Parallel recommendation implementations.** `coralys-recommendation` is declared in the server `Cargo.toml` but `adapters/ultracrew` has its own `recommendation.rs`. These may diverge. Evidence: `services/ultracrew_server/Cargo.toml`, `adapters/ultracrew/src/recommendation.rs`.

11. **Domain model missing Coverage, Contract, Roster.** The generic `models.rs` has `Worker`, `Shift`, `Skill`. Coverage requirements, contract rules (consecutive days, weekends), and Roster as a first-class concept are absent from the generic layer. Evidence: `adapters/ultracrew/src/models.rs`.

12. **`decision_intelligence.rs` is a 38-line stub.** The GTM positions Decision Intelligence as a core differentiator. The current implementation returns five fitness metrics. No trade-off analysis, scenario comparison, or explainability beyond constraint audit trail. Evidence: `decision_intelligence.rs` lines 1–38.

---

## Recommended Next Steps

Organised into three phases. Stop after Phase A and run a pilot. The next major source of learning should come from a real customer, not another internal assessment.

**Phase A — End-to-end usable system (do these before the pilot)**

1. **Generic CSV/JSON import adapter** — maps customer workforce data to `ScheduleRequest`. The `public_contracts.rs` API is already the right target. Estimated scope: one new file in `adapters/ultracrew/src/` or a new `adapters/customer_import/` crate.

2. **Generic REST endpoint** — `ultracrew_server` endpoint accepting `ScheduleRequest` and returning `ScheduleSolution`. This is the integration point for any pilot customer system.

3. **Generic schedule export** — CSV or JSON from `ScheduleSolution`. The INRC exporter is a reference implementation. A pilot customer needs to see the schedule in a format they can use.

**Phase B — Pilot support (do these during the pilot)**

4. **Parallel validation support** — run UltraCrew on historical schedule data and compare output against the customer's existing schedule. This is the core of the Parallel Validation pilot stage.

5. **Structured logging** — operational logs (request received, optimization started, solution generated, errors) for pilot monitoring. The `Observatory` telemetry is good for optimization analysis but not for operational monitoring.

6. **Replace `unwrap()` in hot paths** — `constraint_engine.rs` line 53 and `optimization.rs` line 82. A malformed customer input should return a structured error, not a server panic.

**Phase C — Deployment (defer unless customer requires self-hosting)**

7. **Docker deployment configuration** with environment variable support for port, log level, and seed. For an early pilot where UltraCrew hosts the application, this can be deferred.

8. **Authentication** — post-pilot unless the first customer specifically requires it.

**Post-pilot (driven by evidence from pilot delivery)**

9. **Decouple `ultracrew_server` from INRC types** (P-4 from Architecture Baseline). Not a pilot blocker. Address before adding a second customer domain.

10. **Strengthen `decision_intelligence.rs`** — scenario comparison, structured explanation of optimizer decisions. Foundation of the GTM's "explainable recommendations" claim. **Deferred pending pilot evidence.** Customers may want trade-offs, recommendations, risk, staffing cost, fatigue, overtime, or explainability. The pilot will determine which of these matters most. Do not expand `decision_intelligence.rs` until that evidence exists.

---

## Pilot Exit Criteria

A pilot is considered technically successful when all of the following are true:

- Customer workforce data can be imported without manual intervention
- Schedule generation completes successfully on customer data
- Constraint violations are explainable to the customer planner
- Recommended schedule can be exported in a format the customer can use
- Parallel validation demonstrates acceptable operational quality against the customer's existing schedule
- Customer planners can complete the end-to-end workflow without UltraCrew team assistance

These criteria are outcome-based, not feature-based. They are agreed jointly with the customer during Discovery and recorded in the Pilot Readiness Checklist.

---

## Architectural Principle Violations

| Principle | Status | Evidence |
|---|---|---|
| P1 — Generalize after two implementations | Not violated | Generic layer exists; INRC is first implementation |
| P2 — Products depend on interfaces, not algorithms | **Violated** — `ultracrew_server` imports INRC types via `ultracrew` crate | `CODEBASE_ASSESSMENT.md` line 81; recorded as P-4 |
| P3 — `coralys-scheduling` scope resolved by Phase 1 | Not applicable — `coralys-scheduling` not used by UltraCrew | Directory listing |
| P4 — All changes follow Observe → Propose → Decide → Record | Not assessed — governance process | N/A |
| P5 — Open questions resolved by evidence | Not violated | INRC-II benchmark provides evidence |
| P8 — Preserve separation between Optimization, Planning, Decision Intelligence | **Partially violated** — `decision_intelligence.rs` is a stub; Decision Intelligence not meaningfully separated from constraint reporting | `decision_intelligence.rs` lines 1–38 |
| P10 — Domains are modeled faithfully | **Partially violated** — generic domain model (`Worker`, `Shift`, `Skill`) is incomplete; Coverage, Contract, Roster absent | `adapters/ultracrew/src/models.rs` |

---

*This report answers: How close is the current UltraCrew implementation to the frozen Architecture Baseline v1.0, and what is the minimum remaining work required to support a successful customer pilot?*

*Answer: The optimization engine is pilot-ready. The integration layer (import, export, REST API, deployment) is not. The minimum work for a pilot is Phase A (Generic Import, Generic REST API, Generic Export). After Phase A, stop and run a pilot. The next major source of learning should come from a real customer, not another internal assessment.*

---

*This report is a recurring engineering checkpoint. Rerun at major milestones: v1.1 after Phase A, v1.2 after first pilot, v2.0 after second customer domain.*