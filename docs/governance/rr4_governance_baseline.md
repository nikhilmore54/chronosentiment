# RR4 — Governance Baseline

**Programme:** Repository Rationalization  
**Phase:** RR4 — Governance Baseline  
**Status:** v1.0 — Initial Baseline  
**Produced:** 2026-08-02  
**Authority:** Repository Rationalization Programme  
**Inputs:**
- RR1: `docs/governance/REPOSITORY_CENSUS.md` (inventory baseline)
- RR2: `docs/governance/rr2a_cargo_analysis.csv`, `rr2b_files.csv`, `rr2b_module_edges.csv`, `rr2c_module_utilization.md` (structural baseline)
- RR3: `docs/governance/rr3_lineage.csv`, `rr3_lineage_graph.md`, `rr3_evolutionary_lineage.md` (historical baseline)

---

## 1. Purpose and Role

This document is the **Governance Baseline** for the ChronoSentiment repository.

A Governance Baseline differs from a Decision Register. A Decision Register records what was decided. A Governance Baseline records the repository's **approved lifecycle state** — the authoritative reference against which future audits, cleanups, and evolution decisions are measured.

Every future repository change can be evaluated against this baseline without re-running the full rationalization exercise. When the baseline becomes stale (new crates added, research streams completed, platform libraries promoted), it is amended with a versioned entry in the Amendment Log.

---

## 2. Lifecycle Dimensions

Each governed artefact carries four independent lifecycle dimensions:

| Dimension | Values | Meaning |
|-----------|--------|---------|
| **Historical State** | Superseded / Terminal / Active | Position in the evolutionary chain |
| **Research Outcome** | Successful / Negative / Inconclusive / N/A | Scientific result of the experiment |
| **Governance Decision** | Preserve / Archive / Delete / Promote / Pending | Approved lifecycle action |
| **Repository State** | Active / Archived / Removed | Current physical state in the repository |

These dimensions are independent. A superseded experiment may have a Successful outcome and be Preserved as a Canonical reference. A terminal experiment may be Archived if its research line is complete. Succession alone does not determine governance.

---

## 3. Heritage Classes

Research artefacts are additionally classified by heritage significance:

| Class | Meaning |
|-------|---------|
| **Canonical** | Essential milestone in Coralys platform evolution; must be preserved indefinitely |
| **Reference** | Needed to reproduce important results; preserved until results are independently verified |
| **Historical** | Preserved for provenance only; no active technical dependency |
| **Disposable** | No continuing historical or technical value; eligible for removal after archival window |

Heritage class is independent of Governance Decision. A Disposable artefact may still require an Archive step before deletion (per the deletion policy: Inventory → Utilization → Evolutionary Lineage → Archive → Delete).

---

## 4. Research Line Governance

Rather than governing every experiment individually, RR4 introduces **Research Line** governance. Each research line has an approved status that all member experiments inherit unless individually overridden.

| Research Line | Packages | Status | Rationale |
|---------------|----------|--------|-----------|
| CVRP Evolution Programme | `cvrp_server`, `cvrp` | **Active** | m30 pilot cluster is the most recent generation; stream not declared complete |
| UltraCrew Ecology | `ultracrew`, `ultracrew_server` | **Active** | m31 lineage ongoing; INRC ecology ablation series in progress |
| ROADEF Industrial Validation | `roadef` | **Complete** | m27_1 is the terminal generation; no successor planned |

**Inheritance rule:** All experiments in a Complete research line inherit Governance Decision = Archive unless individually overridden to Canonical or Reference. All experiments in an Active research line inherit Governance Decision = Preserve unless individually overridden.

---

## 5. Platform Library Baseline

Platform libraries are governed as Preserve by default. The following table records the approved baseline state for all 26 workspace members.

### 5.1 Coralys Platform Libraries

| Crate | Lifecycle | Governance Decision | Repository State | Notes |
|-------|-----------|--------------------|--------------------|-------|
| `coralys-core` | Platform | Preserve | Active | Core decision/evaluation models |
| `coralys-decision` | Platform | Preserve | Active | Decision engine |
| `coralys-ecology` | Platform | Preserve | Active | Ecology / fitness landscape |
| `coralys-eval` | Platform | Preserve | Active | Evaluation framework |
| `coralys-infrastructure` | Platform | Preserve | Active | Infrastructure utilities |
| `coralys-matching` | Platform | Preserve | Active | Matching algorithms (canonical replacement for `bipartite_matching.rs`) |
| `coralys-moga` | Platform | Preserve | Active | Multi-objective GA engine |
| `coralys-planning` | Platform | Preserve | Active | Planning layer |
| `coralys-policy` | Platform | Preserve | Active | Policy framework |
| `coralys-recommendation` | Platform | Preserve | Active | Recommendation engine |
| `coralys-simulation` | Platform | Preserve | Active | Simulation harness |
| `coralys-v2` | Platform | Preserve | Active | V2 platform layer |

### 5.2 Infrastructure and Services

| Crate | Lifecycle | Governance Decision | Repository State | Notes |
|-------|-----------|--------------------|--------------------|-------|
| `chronosentiment_core` | Platform | Preserve | Active | Core market adapter and capture daemon |
| `infrastructure/core` | Platform | Preserve | Active | Infrastructure core |
| `infrastructure/optimization` | Platform | Preserve | Active | Optimization infrastructure |
| `observatory` | Application | Preserve | Active | Observatory API service |
| `financial/core` | Platform | Preserve | Active | Financial domain core |
| `financial/ese` | Platform | Preserve | Active | ESE financial module |
| `financial/strategies` | Research | **Pending** | Active | Contains 5 dormant orphan modules (RR2-C); owner decision required |

### 5.3 Research Adapters

| Crate | Lifecycle | Governance Decision | Repository State | Notes |
|-------|-----------|--------------------|--------------------|-------|
| `adapters/cvrp` | Research | Preserve | Active | CVRP adapter; Active research line |
| `adapters/gerad` | Research | Preserve | Active | GERAD adapter |
| `adapters/cvd001` | Research | Preserve | Active | CVD001 adapter |
| `adapters/roadef` | Research | Archive | Active | ROADEF research line Complete; archive after m27 validation |
| `adapters/ultracrew` | Research | Preserve | Active | UltraCrew research line Active |
| `services/cvrp_server` | Research | Preserve | Active | CVRP experiment host; Active research line |
| `services/ultracrew_server` | Research | Preserve | Active | UltraCrew experiment host; Active research line |

---

## 6. Orphaned Module Baseline

The 9 orphaned library modules identified by RR2-C are governed individually. Succession is one input but not the sole determinant.

| File | Category | Evidence | Historical State | Research Outcome | Governance Decision | Heritage Class | Rationale |
|------|----------|----------|-----------------|-----------------|--------------------|--------------|-|
| `financial/strategies/src/paper.rs` | dormant | E3 | Terminal | Inconclusive | **Pending** | Reference | 1179 LOC paper-trading engine; too substantial to govern without owner input; may be Canonical if it is the primary paper-trading harness |
| `financial/strategies/src/edge_decay.rs` | dormant | E3 | Terminal | Inconclusive | **Pending** | Reference | Complete implementation; coupled to `edge_half_life_estimator.rs`; promote or archive together |
| `financial/strategies/src/edge_half_life_estimator.rs` | dormant | E3 | Terminal | Inconclusive | **Pending** | Reference | Complete; coupled to `edge_decay.rs` |
| `financial/strategies/src/signals.rs` | dormant | E3 | Terminal | Inconclusive | **Pending** | Reference | Signals vocabulary; overlap with `crate::domain` unresolved; promote or archive after domain audit |
| `financial/strategies/src/pipeline/certification/orchestration.rs` | incomplete | E2 | Terminal | Inconclusive | **Pending** | Historical | `pub(crate)` certification utilities; parent module chain broken; complete wiring or archive |
| `adapters/ultracrew/src/helpers.rs` | incomplete | E3 | Terminal | Inconclusive | **Pending** | Historical | Mixed stubs and real `run_optimization()`; owner must decide whether `ScheduleOptimizer` is live |
| `adapters/roadef/src/adapter.rs` | incomplete | E2 | Terminal | Inconclusive | Archive | Historical | ROADEF research line Complete; trait definition with no implementors; archive with the research line |
| `services/ultracrew_server/src/simulation_test.rs` | incomplete | E4 | Terminal | N/A | **Delete** | Disposable | 6-line scratch stub; no implementation value; no archival required |
| `adapters/ultracrew/src/inrc/bipartite_matching.rs` | obsolete | E5 | Superseded | N/A | **Delete** | Disposable | Self-declared deprecated tombstone; canonical replacement is `coralys-matching`; no archival required |

**Pending decisions** (5 files in `financial/strategies`) require owner input before governance can be finalized. The structural and historical evidence is complete; the missing dimension is research significance — whether these modules represent unintegrated capability that should be promoted, or completed research that should be archived.

---

## 7. Experiment Baseline — Research Line Inheritance

All 64 experiment binaries inherit their Governance Decision from their Research Line unless individually overridden below.

**CVRP Evolution Programme (Active):** All 32 experiments → Preserve  
**UltraCrew Ecology (Active):** All 18 experiments → Preserve  
**ROADEF Industrial Validation (Complete):** All 14 experiments → Archive

### 7.1 Individual Overrides

The following experiments are individually overridden from their Research Line default, based on heritage significance:

| Binary | Research Line | Default | Override | Heritage Class | Rationale |
|--------|--------------|---------|----------|---------------|-----------|
| `m8g_cvrp_validation` | CVRP | Preserve | **Canonical** | Canonical | Earliest CVRP validation; root of the entire CVRP lineage chain |
| `m11_reachability_atlas` | CVRP | Preserve | **Canonical** | Canonical | First reachability atlas; established the solution space mapping methodology |
| `m22_0_archive` | CVRP | Preserve | **Reference** | Reference | Archive snapshot of m22 baseline; needed to reproduce m22 results |
| `m22c_2_fidelity_audit` | CVRP | Preserve | **Reference** | Reference | Fidelity audit; validates the m22c variant series |
| `m25_benchmark` | ROADEF | Archive | **Reference** | Reference | ROADEF benchmark baseline; needed to reproduce m25 results even after line is archived |
| `m25_final` | ROADEF | Archive | **Canonical** | Canonical | Terminal ROADEF experiment; canonical result of the ROADEF validation programme |
| `m8g_cs_validation` | ULTRACREW | Preserve | **Canonical** | Canonical | Root of the UltraCrew lineage; earliest CS validation |
| `m8g_ultracrew_validation` | ULTRACREW | Preserve | **Canonical** | Canonical | Root of the UltraCrew lineage; earliest UltraCrew validation |
| `inrc_ecology_ablation_matrix` | ULTRACREW | Preserve | **Reference** | Reference | Ablation matrix; reference for the entire INRC ecology series |

---

## 8. Immediate Actions

Two artefacts have sufficient evidence for immediate action without waiting for owner input:

| Artefact | Evidence | Action | Prerequisite | Batch |
|----------|----------|--------|-------------|-------|
| `adapters/ultracrew/src/inrc/bipartite_matching.rs` | E5 | Delete | Confirm `coralys-matching` contains replacement | RR7 |
| `services/ultracrew_server/src/simulation_test.rs` | E4 | Delete | None | RR7 |

These deletions are batched into RR7 (Deletion Plan).

---

## 9. Pending Owner Decisions

The following governance decisions cannot be finalized without owner input:

| Artefact | Question | Options |
|----------|----------|---------|
| `financial/strategies/src/paper.rs` | Is this the primary paper-trading harness, or has it been superseded by another implementation? | Promote (add `pub mod paper;` to `lib.rs`) or Archive |
| `financial/strategies/src/edge_decay.rs` + `edge_half_life_estimator.rs` | Are edge-decay measurement and half-life estimation live requirements? | Promote together or Archive together |
| `financial/strategies/src/signals.rs` | Does `signals.rs` extend or duplicate `crate::domain`? | Promote after domain audit or Archive |
| `financial/strategies/src/pipeline/certification/orchestration.rs` | Is the `pipeline/certification` framework intended to be completed? | Complete wiring or Archive |
| `adapters/ultracrew/src/helpers.rs` | Is `crate::optimization::ScheduleOptimizer` a live, compilable type? | Wire `run_optimization()` or Archive |

---

## 10. Governance Baseline Summary

| Category | Count | Governance Decision |
|----------|-------|---------------------|
| Platform libraries | 12 | Preserve |
| Infrastructure / services | 7 | Preserve |
| Research adapters (active lines) | 5 | Preserve |
| Research adapters (complete lines) | 1 | Archive |
| `financial/strategies` crate | 1 | Pending |
| Orphan modules — immediate delete | 2 | Delete |
| Orphan modules — archive with line | 1 | Archive |
| Orphan modules — pending owner | 5 | Pending |
| Experiment binaries — CVRP (active) | 32 | Preserve (9 overrides: Canonical/Reference) |
| Experiment binaries — ULTRACREW (active) | 18 | Preserve (3 overrides: Canonical/Reference) |
| Experiment binaries — ROADEF (complete) | 14 | Archive (2 overrides: Canonical/Reference) |

**Total governed artefacts:** 98 (26 crates + 9 orphan modules + 64 experiment binaries - 1 crate counted in both)

---

## 11. Evidence Pipeline

This baseline is the product of a four-phase evidence pipeline:

```
RR1 — Inventory Evidence
  What exists: 26 workspace members, 468 source files, 111 executables
        │
        ▼
RR2 — Structural Evidence
  How it is connected: 263 module edges, 9 orphaned modules classified
        │
        ▼
RR3 — Historical Evidence
  How it evolved: 64 experiment binaries with lineage chains and DAG
        │
        ▼
RR4 — Governance Baseline (this document)
  What should happen next: lifecycle states for every governed artefact
        │
        ▼
  Future repository changes evaluated against this baseline
```

No governance decision in this document is based on a single evidence dimension. Every decision references at least two of: structural evidence (RR2), historical evidence (RR3), and research significance (owner knowledge). Where owner knowledge is missing, the decision is recorded as Pending rather than inferred.

---

## 12. Amendment Log

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-08-02 | governance-hardening | Initial Governance Baseline — RR4 v1.0 |