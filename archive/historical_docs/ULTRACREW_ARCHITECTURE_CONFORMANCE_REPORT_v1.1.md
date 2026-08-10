# UltraCrew Architecture Conformance Report v1.1

> **Status**: Conformance Report v1.1 — frozen
> **Date**: 2026-07-20
> **Assessor**: Architecture Conformance Assessment against Architecture Baseline v1.0
> **Scope**: `adapters/ultracrew`, `services/ultracrew_server`, platform crate dependencies
> **Method**: Direct code inspection of all source files added or modified during Phase A engineering. v1.0 findings carried forward unchanged where no new evidence exists.
> **Supersedes**: `ULTRACREW_ARCHITECTURE_CONFORMANCE_REPORT.md` (v1.0, 2026-07-20)
> **Next checkpoint**: v1.2 after first customer pilot

---

## What Changed Since v1.0

Phase A engineering is complete. The three P1 pilot blockers identified in v1.0 have been resolved:

| P1 Item (v1.0) | v1.0 Status | v1.1 Status | Evidence |
|---|---|---|---|
| No generic data import adapter | Missing | **Implemented** | `adapters/ultracrew/src/generic_import.rs` — `GenericImporter` with JSON + CSV, full validation, template export |
| No generic REST endpoint | Partial (INRC only) | **Implemented** | `POST /api/schedule`, `POST /api/reschedule`, `POST /api/validate` confirmed live in `services/ultracrew_server/src/main.rs` |
| No generic schedule export | Partial (INRC only) | **Implemented** | `adapters/ultracrew/src/generic_export.rs` — `GenericExporter` with JSON + CSV, 11/11 unit tests pass; `GET /api/export/formats` and `POST /api/export/:format` wired |

Build verification: `cargo build -p ultracrew -p ultracrew_server` completed with exit code 0. All warnings are pre-existing cosmetic issues unrelated to Phase A deliverables.

No architectural changes were made. The architecture score is unchanged. Score improvements reflect product completeness and pilot readiness only.

---

## Executive Summary

| Dimension | v1.0 Score | v1.1 Score | Basis for change |
|---|---|---|---|
| Architecture Conformance | **8.5/10** | **8.5/10** | No architectural changes made. P-4 violation (INRC coupling in server) remains. |
| Product Completeness | **6/10** | **7.5/10** | Generic Import, Generic REST API, and Generic Export implemented. Three previously missing capabilities now present. |
| Pilot Readiness | **7/10** | **8.5/10** | All three P1 blockers resolved. The end-to-end pilot workflow now exists. P2 items (parallel validation, structured logging, error hardening) remain. |

| Area | v1.0 Maturity | v1.1 Maturity |
|---|---|---|
| Architecture | Mature | Mature (unchanged) |
| Coralys Optimisation Platform | Mature | Mature (unchanged) |
| Product Engineering | Emerging | **Functional** |
| Workforce Operations Platform | In Progress | In Progress (unchanged) |

The end-to-end scheduling workflow now exists independent of the INRC benchmark format:

```
Customer Data
      ↓
Generic Import  (generic_import.rs — JSON or CSV)
      ↓
ScheduleRequest  (public_contracts.rs — domain-independent)
      ↓
Coralys Optimisation  (pipeline.rs → coralys-moga)
      ↓
ScheduleSolution  (schedule_solution.rs)
      ↓
Generic Export  (generic_export.rs — JSON or CSV)
      ↓
Customer
```

This is the minimum technical capability the v1.0 report identified as required before beginning a pilot. That threshold has been crossed.

**Top strengths (updated):**
- End-to-end optimization pipeline is implemented and INRC-II validated
- `coralys-moga` consumed correctly through trait interfaces (`FitnessEvaluator`, `MutationOperator`, `CrossoverOperator`, `GenomeFactory`)
- `coralys-ecology` integrated for fatigue/historical workload tracking
- `public_contracts.rs` provides a domain-independent `Scenario` / `ScheduleRequest` / `RescheduleRequest` API
- Constraint engine produces structured `ConstraintReport` with per-constraint scores, violated/satisfied lists, and warnings
- Recommendation engine generates actionable per-constraint recommendations
- Observatory telemetry captures per-generation fitness, diversity, and timing
- Disruption recovery (`RescheduleRequest` with `locked_assignments`) is implemented
- **NEW**: Generic Import accepts customer data as JSON or two-file CSV without INRC dependency
- **NEW**: Generic REST API exposes `ScheduleRequest` / `RescheduleRequest` / `ValidateRequest` contracts
- **NEW**: Generic Export serialises `ScheduleSolution` to JSON or CSV with configurable options

**Remaining risks (updated):**
- `ultracrew_server` still imports `InrcScenario` directly — P-4 violation, not a pilot blocker
- No parallel validation support — P2, needed during pilot
- No structured logging — P2, needed during pilot
- `unwrap()` calls remain in hot paths — P2, needed during pilot
- `decision_intelligence.rs` remains a 38-line stub
- No authentication, multi-tenancy, or deployment configuration

---

## Architecture Conformance Matrix

Changes from v1.0 are marked **[UPDATED]**. All other rows are carried forward unchanged.

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
| REST API (generic contracts) | Endpoints for `ScheduleRequest`, `RescheduleRequest` | **[UPDATED]** `POST /api/schedule`, `POST /api/reschedule`, `POST /api/validate` confirmed live; accept `ScheduleRequest` / `RescheduleRequest` / `ValidateRequest` directly | **Implemented** | `services/ultracrew_server/src/main.rs` lines 833, 875, 927 |
| Schedule export | Customer-usable schedule output | **[UPDATED]** `generic_export.rs` implements JSON and CSV export from `ScheduleSolution`; `GET /api/export/formats` and `POST /api/export/:format` wired | **Implemented** | `adapters/ultracrew/src/generic_export.rs`; `main.rs` export routes |
| Generic data import | Customer data → `ScheduleRequest` | **[UPDATED]** `generic_import.rs` implements JSON (direct) and CSV (two-file: workers + shifts) import with full validation and template export | **Implemented** | `adapters/ultracrew/src/generic_import.rs` |
| Authentication / multi-tenancy | Not specified in baseline | Not implemented | Not Implemented | No evidence in source |

---

## GTM Capability Matrix

Changes from v1.0 are marked **[UPDATED]**.

| GTM Capability | v1.0 Status | v1.1 Status | Evidence |
|---|---|---|---|
| **Workforce Planning and Scheduling** | | | |
| Automated schedule generation | Implemented | Implemented | `pipeline.rs` — end-to-end GA optimization |
| Constraint management (skill, overlap, rest, hours) | Implemented | Implemented | `constraint_engine.rs` — HC1, HC2, HC3, Rest, SC1, SC2 |
| Fairness optimisation | Implemented | Implemented | SC1 variance penalty + workload-balanced swap mutation |
| Multi-site / multi-team scheduling | Not Implemented | Not Implemented | No site/team concept in domain model |
| Demand-driven capacity planning | Not Implemented | Not Implemented | No demand/coverage concept in generic layer |
| Contract rule enforcement | Partial | Partial | HC3 (max hours) implemented; consecutive days, weekend rules absent |
| **Operational Decision Support** | | | |
| Constraint audit trail | Implemented | Implemented | `ConstraintReport.violated_constraints`, `constraint_scores` |
| Actionable recommendations | Implemented | Implemented | `RecommendationEngine` — per-constraint severity and action |
| Trade-off analysis | Not Implemented | Not Implemented | `decision_intelligence.rs` is a stub |
| What-if scenario analysis | Not Implemented | Not Implemented | No scenario comparison capability |
| Cost comparison | Not Implemented | Not Implemented | No cost model |
| **Day-of-Operations** | | | |
| Disruption recovery | Implemented | Implemented | `RescheduleRequest` with `locked_assignments` |
| Schedule publication | Not Implemented | Not Implemented | No publication or distribution mechanism |
| Shift swap / leave handling | Not Implemented | Not Implemented | No leave or availability model |
| Notifications | Not Implemented | Not Implemented | No notification capability |
| **Workforce Visibility** | | | |
| Optimization telemetry | Implemented | Implemented | `Observatory` — per-generation fitness, diversity, timing |
| Labour cost / utilisation | Not Implemented | Not Implemented | No cost model; utilisation derivable from assignments |
| Compliance reporting | Partial | Partial | Constraint violation counts available; no formatted report |
| Dashboards | Not Implemented | Not Implemented | No UI integration for workforce visibility |
| **Enterprise Connectivity** | | | |
| INRC2 data import | Implemented | Implemented | `inrc/parser.rs` |
| Generic data import (HRMS, CSV) | Not Implemented | **[UPDATED] Implemented** | `generic_import.rs` — JSON and two-file CSV with validation |
| Payroll / attendance export | Not Implemented | **[UPDATED] Implemented** | `generic_export.rs` — JSON and CSV from `ScheduleSolution` |

---

## Pilot Readiness Matrix

Changes from v1.0 are marked **[UPDATED]**.

| Capability | v1.0 Status | v1.1 Status | Notes |
|---|---|---|---|
| Schedule generation | Ready | Ready | End-to-end pipeline validated against INRC-II benchmark |
| Hard constraint enforcement | Ready | Ready | HC1 (skill), HC2 (overlap), HC3 (max hours), Rest (8h gap) |
| Soft constraint optimisation | Ready | Ready | SC1 (fairness), SC2 (fatigue) |
| Disruption recovery | Ready | Ready | `RescheduleRequest` with locked assignments implemented |
| Constraint audit trail | Ready | Ready | Per-constraint scores, violated/satisfied lists, warnings |
| Actionable recommendations | Ready | Ready | Per-constraint severity and recommended action |
| Optimization telemetry | Ready | Ready | Per-generation fitness, diversity, elapsed time |
| Generic data input (non-INRC) | Needs Work | **[UPDATED] Ready** | `generic_import.rs` — JSON and CSV import, full validation, template export for customer onboarding |
| Schedule export (customer format) | Needs Work | **[UPDATED] Ready** | `generic_export.rs` — JSON and CSV export; `POST /api/export/:format` endpoint live |
| REST API (generic contracts) | Needs Work | **[UPDATED] Ready** | `POST /api/schedule`, `/api/reschedule`, `/api/validate` accept domain-independent contracts |
| Parallel validation support | Needs Work | Needs Work | No mechanism to run UltraCrew alongside existing process and compare outputs — Phase B |
| Observability / logging | Needs Work | Needs Work | Observatory captures telemetry but no structured logging for pilot monitoring — Phase B |
| Deployment configuration | Missing | Missing | No Docker, environment config, or deployment documentation — Phase C |
| Authentication | Missing | Missing | Post-pilot unless customer specifically requires it |
| Error handling (production-grade) | Needs Work | Needs Work | `unwrap()` calls remain in hot paths — Phase B |
| Multi-tenancy | Missing | Missing | Post-pilot |

---

## Technical Debt

Changes from v1.0 are marked **[RESOLVED]**.

**P1 — Blocks pilot delivery**

1. ~~**No generic data import adapter.**~~ **[RESOLVED]** `generic_import.rs` implements `GenericImporter` with JSON and two-file CSV support, full validation (duplicate IDs, empty lists, zero-duration shifts, uncovered skills), and template export functions for customer onboarding. Registered in `lib.rs`.

2. ~~**No generic REST endpoint.**~~ **[RESOLVED]** `POST /api/schedule`, `POST /api/reschedule`, and `POST /api/validate` are confirmed live in `services/ultracrew_server/src/main.rs`. All three accept domain-independent `ScheduleRequest` / `RescheduleRequest` / `ValidateRequest` contracts and return `ScheduleResponse` with assignments, metrics, constraint report, recommendations, and telemetry.

3. ~~**No generic schedule export.**~~ **[RESOLVED]** `generic_export.rs` implements `GenericExporter` with JSON and CSV serialisation from `ScheduleSolution`. `GET /api/export/formats` and `POST /api/export/:format` are wired into the router. 11/11 unit tests pass.

**P2 — Degrades pilot quality**

4. **No parallel validation support.** Unchanged. The Pilot Methodology requires running UltraCrew alongside the customer's existing process and comparing outputs. No mechanism exists for this. Phase B.

5. **`unwrap()` in hot paths.** Unchanged. `optimization.rs` line 82 (`expect("Invalid EvolutionConfig")`), `constraint_engine.rs` line 53 (`unwrap()` on assignment lookup). Phase B.

6. **No structured logging.** Unchanged. The `Observatory` telemetry is good for optimization analysis but a pilot needs operational logs (request received, optimization started, solution generated, errors). Phase B.

**P3 — Deployment (can be hosted by UltraCrew team for early pilot)**

7. **No deployment configuration.** Unchanged. No Docker, environment variable support, or deployment documentation. Phase C.

8. **Authentication and multi-tenancy.** Unchanged. Post-pilot.

**P4 — Future extensibility**

9. **`ultracrew_server` imports INRC types.** Unchanged. Known violation recorded as P-4 in the Architecture Baseline. Does not block a pilot.

10. **Parallel recommendation implementations.** Unchanged. `coralys-recommendation` declared in server `Cargo.toml` but `adapters/ultracrew` has its own `recommendation.rs`.

11. **Domain model missing Coverage, Contract, Roster.** Unchanged. Generic `models.rs` has `Worker`, `Shift`, `Skill`. Coverage requirements, contract rules, and Roster as a first-class concept are absent from the generic layer.

12. **`decision_intelligence.rs` is a 38-line stub.** Unchanged. Deferred pending pilot evidence.

---

## Recommended Next Steps

**Phase A — COMPLETE.** All three deliverables implemented and build-verified.

**Phase B — Pilot support (do these during the pilot)**

1. **Parallel validation support** — run UltraCrew on historical schedule data and compare output against the customer's existing schedule. This is the core of the Parallel Validation pilot stage.

2. **Structured logging** — operational logs (request received, optimization started, solution generated, errors) for pilot monitoring.

3. **Replace `unwrap()` in hot paths** — `constraint_engine.rs` line 53 and `optimization.rs` line 82. A malformed customer input should return a structured error, not a server panic.

**Phase C — Deployment (defer unless customer requires self-hosting)**

4. **Docker deployment configuration** with environment variable support for port, log level, and seed.

5. **Authentication** — post-pilot unless the first customer specifically requires it.

**Post-pilot (driven by evidence from pilot delivery)**

6. **Decouple `ultracrew_server` from INRC types** (P-4 from Architecture Baseline). Address before adding a second customer domain.

7. **Strengthen `decision_intelligence.rs`** — scenario comparison, structured explanation of optimizer decisions. Deferred pending pilot evidence. The pilot will determine which of trade-offs, recommendations, risk, staffing cost, fatigue, overtime, or explainability matters most.

---

## Pilot Exit Criteria

Unchanged from v1.0. These criteria are outcome-based, not feature-based.

- Customer workforce data can be imported without manual intervention
- Schedule generation completes successfully on customer data
- Constraint violations are explainable to the customer planner
- Recommended schedule can be exported in a format the customer can use
- Parallel validation demonstrates acceptable operational quality against the customer's existing schedule
- Customer planners can complete the end-to-end workflow without UltraCrew team assistance

---

## Architectural Principle Violations

Unchanged from v1.0. Phase A did not introduce or resolve any architectural principle violations.

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

## Evidence Gate Status

| Evidence Gate | v1.0 Status | v1.1 Status |
|---|---|---|
| Algorithm Validation | Complete | Complete |
| Platform Validation | Complete | Complete |
| Commercial Readiness | Complete | Complete |
| Productisation (Phase A) | In Progress | **Complete** |
| Pilot Validation | Next | Next |

---

*This report answers: How close is the current UltraCrew implementation to the frozen Architecture Baseline v1.0, and what is the minimum remaining work required to support a successful customer pilot?*

*Answer as of v1.1: The three P1 pilot blockers have been resolved. The end-to-end scheduling workflow exists independent of the INRC benchmark format. The remaining work before a pilot is Phase B (parallel validation, structured logging, error hardening). The next major source of learning should come from a real customer, not another internal assessment.*

---

*This report is a recurring engineering checkpoint. v1.0: baseline. v1.1: after Phase A (this document). v1.2: after first pilot. v2.0: after second customer domain.*

---

## Note for v1.2 Authors

At v1.2 (after the first customer pilot), add a **Customer Evidence** section as the first section after the Executive Summary. This will be the first report section driven primarily by external evidence rather than internal implementation. Suggested structure:

| Observation | Evidence source | Outcome |
|---|---|---|
| Import usability | Customer onboarding session | TBD |
| Schedule quality | Parallel validation results | TBD |
| Planner acceptance | Pilot interviews / feedback | TBD |
| Runtime reliability | Production logs | TBD |
| Recommendation usefulness | User feedback | TBD |

All other sections should be updated only where customer or operational evidence changes the finding. Architecture score should remain unchanged unless architectural issues are resolved or new violations are discovered. The principle of updating only what evidence supports applies equally to v1.2.