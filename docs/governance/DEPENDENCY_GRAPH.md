# Repository Dependency Graph

**Document ID:** GOV-DEP-001
**Version:** 1.0
**Status:** Active
**Created:** 2026-08-01
**Scope:** All five knowledge systems — typed dependency edges between research, code, evidence, and governance nodes

---

## Purpose

This graph extends `docs/code_map.html` (Rust-only) to cover the full repository. It answers:

- *"If I change X, what else must be updated?"*
- *"What does Y depend on?"*
- *"What evidence validates claim Z?"*

---

## Edge Type Taxonomy

| Symbol | Meaning |
|--------|---------|
| `→impl` | Code implements a contract or specification |
| `→val` | Evidence validates a claim in a research document |
| `→frz` | A freeze event locks a section of a document |
| `→prod` | A harness/test produces an evidence artifact |
| `→dep` | A component depends on another at runtime or compile time |
| `→sup` | A newer asset supersedes an older one |
| `→ref` | A document references another for context |
| `→gov` | A governance artifact governs another asset |
| `→dup` | Two assets have overlapping scope (consolidation needed) |
| `→ext` | An asset extends another (adds to, does not replace) |

---

## Layer 0 — Governance → Everything

```
GOV-KS-001 (Knowledge Survey)
  →gov  ALL canonical assets

GOV-IDX-001 (Canonical Index)
  →gov  ALL canonical assets
  →ref  GOV-KS-001

GOV-DEP-001 (Dependency Graph — this file)
  →gov  ALL canonical assets
  →ref  GOV-KS-001
  →ref  GOV-IDX-001
  →ext  docs/code_map.html (Rust-only code map)

GOV-CLN-001 (Cleanup Register)
  →gov  DUP-001 through DUP-006 (duplicate pairs in Knowledge Survey)
  →ref  GOV-KS-001
```

---

## Layer 1 — Research → Evidence

### UC-R-001: Pairing Topology Mutation Evaluation

```
UC-R-001 §2 (FROZEN 2026-08-01)
  →val  BENCH-001 (gerad_coralys.rs — GERAD Coralys v1.0 baseline)
  →val  EV-001 (results/ — CSV metrics from harness runs)
  →val  EV-005 (reports/ — markdown reports from harness)
  →ref  STANDARD-002 (Benchmark Governance)
  →ref  STANDARD-003 (Benchmark Reference Specification)
  →frz  BENCH-001 (baseline frozen on same date)

UC-R-001 §3 (Active — Experiment 0 spec complete)
  →dep  HARNESS-001 through HARNESS-006 (experiment harness)
  →dep  BENCH-001 (GERAD Coralys baseline — read-only reference)
  →ref  CONTRACT-007 (Metrology Layer)
  →ref  CONTRACT-011 (Persistence Semantics)
  →ref  UC-R-002 (Section 3 document)
```

### UC-R-002: Coralys Native Scheduler — Section 3

```
UC-R-002
  →dep  ADAPTER-001 (airline adapter)
  →dep  HARNESS-001 through HARNESS-006
  →ref  UC-R-001 §2 (frozen baseline results)
  →ref  CONTRACT-017 (Rust Port)
  →prod EV-001 (results/ — Experiment 0 will produce landscape_sample.csv)
```

### CS-R Series (ChronoSentiment Research)

```
CS-R-015 (Investment Thesis)
  →ref  CS-R-001 (Market Landscape)
  →ref  CS-R-002 (Competitive Landscape)
  →ref  CS-R-003 (Customer Problem Evidence)
  →ref  CS-R-005 (Pricing Analysis)

CS-R-015A (Executive Investment Summary)
  →ref  CS-R-015 (Investment Thesis)

CS-R-008 (Point-in-Time Architecture Review)
  →ref  ARCH-001 (Coralys Platform Architecture)
```

---

## Layer 2 — Code → Research

### Experiment Harness → Research

```
HARNESS-001 (schema.rs)
  →impl UC-R-002 §Experiment 0 (ExperimentConfig, GenerationRecord, RunSummary)
  →impl STANDARD-002 (Benchmark Governance — structured result format)

HARNESS-002 (logging.rs)
  →impl CONTRACT-009 (Observability Semantics)

HARNESS-003 (persistence.rs)
  →impl CONTRACT-011 (Persistence Semantics)
  →prod EV-001 (results/ — CSV, JSON)
  →prod EV-005 (reports/ — markdown)

HARNESS-004 (reproducibility.rs)
  →impl CONTRACT-019 (State Coherence)
  →impl CONTRACT-020 (Surface Hash — FNV-1a checksum)

HARNESS-005 (report.rs)
  →impl UC-R-002 §Experiment 0 (markdown output format)
  →prod EV-005 (reports/)

HARNESS-006 (mod.rs)
  →dep  HARNESS-001 through HARNESS-005 (re-exports all)
```

### Benchmarks → Research

```
BENCH-001 (gerad_coralys.rs — FROZEN)
  →impl UC-R-001 §2.17 (baseline parameters: pop=50, gen=200, xover=0.80, k=3, seed=42)
  →dep  ADAPTER-001 (airline adapter)
  →dep  CRATE-007 (coralys-moga)
  →prod EV-001 (results/)
  →val  UC-R-001 §2 (all Section 2 claims validated by this benchmark)

BENCH-002 (gerad_e2e.rs)
  →dep  ADAPTER-001
  →dep  CRATE-007

BENCH-003 (benchmark.rs)
  →dep  ADAPTER-001
```

### Adapters → Crates

```
ADAPTER-001 (airline/)
  →dep  CRATE-001 (coralys-core)
  →dep  CRATE-007 (coralys-moga)
  →dep  CRATE-004 (coralys-eval)
  →dep  CRATE-005 (coralys-infrastructure)
  →impl CONTRACT-017 (Rust Port — airline domain in Rust)

ADAPTER-002 (chronosentiment/)
  →dep  CRATE-001 (coralys-core)
  →dep  CRATE-004 (coralys-eval)
  →impl CONTRACT-001 (Crypto Substrate)
  →impl CONTRACT-018 (Signal Interface)

ADAPTER-003 (roadef/)
  →dep  CRATE-001 (coralys-core)
  →dep  CRATE-007 (coralys-moga)
```

### Services → Adapters

```
SERVICE-001 (ultracrew_server/)
  →dep  ADAPTER-001 (airline adapter)
  →dep  CRATE-001 (coralys-core)

APP-001 (ultracrew-pilot-portal/)
  →dep  SERVICE-001 (HTTP API)
```

---

## Layer 3 — Evidence → Research (validation edges)

```
EV-001 (results/)
  →val  UC-R-001 §2 (Section 2 claims — GERAD benchmark results)
  →val  UC-R-002 (Section 3 — future Experiment 0 results)

EV-002 (logs/)
  →val  UC-R-001 §2.15 (raw counts validated against runtime logs)

EV-003 (pilot_sessions/)
  →val  EV-GOV-003 (UltraCrew Workforce Evidence)

CERT-001 through CERT-008
  →val  EV-GOV-001 (ChronoSentiment Evidence Programme)
  →val  EV-GOV-002 (INRC Product Evidence Programme)
```

---

## Layer 4 — Supersession Edges

```
GOV-KS-001 →sup GOV-OLD-001 (docs/REPOSITORY_SURVEY.md)
```

---

## Layer 5 — Duplicate Edges (consolidation needed)

```
docs/REPOSITORY_SURVEY.md →dup docs/governance/KNOWLEDGE_SURVEY.md
docs/RESEARCH_LINEAGE.md →dup docs/research/RESEARCH_LINEAGE.md
docs/ChronoSentiment_Personal_Blueprint_v1.md →dup docs/ChronoSentiment_Product_Blueprint_v1.md
docs/CODEBASE_ASSESSMENT.md →dup docs/CODEBASE_ARCHITECTURE_ASSESSMENT.md
docs/EP-001_MILESTONE.md →dup docs/P001_MILESTONE.md
```

All duplicate edges are tracked in GOV-CLN-001 (Cleanup Register).

---

## Impact Analysis — "If I change X"

| Asset Changed | Must Also Update |
|---------------|-----------------|
| `BENCH-001` (frozen — do not change) | UC-R-001 §2, GOV-KS-001, GOV-IDX-001 |
| `HARNESS-001` (schema.rs) | UC-R-002 (Experiment 0 spec), any test that uses harness types |
| `HARNESS-003` (persistence.rs) | EV-001 directory structure, UC-R-002 landscape_sample spec |
| `ADAPTER-001` (airline/) | BENCH-001, BENCH-002, BENCH-003, SERVICE-001, all harness tests |
| `CRATE-007` (coralys-moga) | ADAPTER-001, ADAPTER-003, BENCH-001, BENCH-002 |
| `UC-R-001 §2` (frozen) | Requires reviewer approval; update GOV-KS-001 freeze date |
| `UC-R-002` (Section 3) | GOV-KS-001, GOV-IDX-001 |
| `CONTRACT-011` (Persistence Semantics) | HARNESS-003, ADAPTER-002 |
| `GOV-KS-001` (Knowledge Survey) | GOV-IDX-001, GOV-DEP-001 |

---

## Maintenance Protocol

1. When adding a new asset: add a node and all outbound edges to this graph.
2. When freezing a section: add a `→frz` edge from the freeze event to the frozen asset.
3. When archiving an asset: remove its edges and add a note in GOV-CLN-001.
4. When a duplicate is resolved: remove the `→dup` edge and log the resolution in GOV-CLN-001.
5. When the Rust code map (`docs/code_map.html`) is regenerated: verify that all `→dep` edges between crates in this graph are consistent with it.

---

*Last updated: 2026-08-01 | Maintained by: Repository Governance*