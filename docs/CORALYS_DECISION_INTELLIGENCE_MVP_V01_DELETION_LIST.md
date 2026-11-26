# Coralys Decision Intelligence MVP v0.1 — Deletion / Retirement List

**Document ID:** CORALYS_DECISION_INTELLIGENCE_MVP_V01_DELETION_LIST  
**Status:** DRAFT — requires owner review before any deletion is executed  
**Date:** 2026-08-17  
**Parent:** CORALYS_DECISION_INTELLIGENCE_MVP_V01.md  
**Rule:** Do not delete anything in this list without first running the dependency grep in Section 1.

---

## 0. Governing principle

The MVP spec (CORALYS_DECISION_INTELLIGENCE_MVP_V01.md) replaces the **product implementation** of the old Decision Observatory. It does **not** replace the research evidence, protocol documents, or immutable data archives.

The distinction is:

| Category | Action |
|---|---|
| Product implementation code superseded by MVP | **DELETE / RETIRE** |
| Duplicate lifecycle representations | **DELETE / CONSOLIDATE** |
| Historical evidence archives (ledger.json, REPORT.md, etc.) | **KEEP — immutable** |
| Research protocol documents (CS-P-006-P, CS-P-006-P.H, etc.) | **KEEP — historical record** |
| Frozen policy artifacts (C3-002, `5a43b9df…`) | **KEEP — immutable** |
| Shared infrastructure used by non-Observatory code | **KEEP / REFACTOR** |
| Unclear dependency | **DO NOT DELETE YET** |

---

## 1. Required pre-deletion grep

Before deleting any file, run:

```bash
# Find all references to Observatory implementation modules
grep -r \
  "observatory_slice\|observatory_historical\|observatory_prospective\|observatory_maturity\|observatory_execution\|observatory_registry\|observatory_live_execution\|observatory_historical_pe2\|observatory_historical_pe3\|ObservatoryLedger\|SealedDecisionRecord\|csp006_p_observe\|csp006_p_observatory\|csp006_p_prospective\|csp006_p_replay\|csp006_p_execute\|csp006_p_live_execute\|csp006_p_historical_pe2\|csp006_p_replay_pe3" \
  --include="*.rs" --include="*.toml" --include="*.ts" --include="*.tsx" \
  -l .
```

Classify every hit before proceeding. A file that appears in this list but is also referenced by non-Observatory code (e.g. portfolio replay, stop research) must be **refactored, not deleted**.

---

## 2. Rust source files — Observatory implementation

These files implement the old CS-P-006-P Observatory product layer. They are superseded by the new MVP Decision Ledger.

### 2.1 Decision support modules (adapters/chronosentiment/src/decision_support/)

| File | Observatory role | Dependency check required |
|---|---|---|
| [`observatory_slice.rs`](../adapters/chronosentiment/src/decision_support/observatory_slice.rs) | Core `ObservatoryLedger`, `SealedDecisionRecord`, HTML rendering, P.3–P.7 | **YES** — `SealedDecisionRecord` used by portfolio replay modules |
| [`observatory_historical.rs`](../adapters/chronosentiment/src/decision_support/observatory_historical.rs) | P.H historical replay engine | **YES** — `generate_historical_replay_decision` used by portfolio replay |
| [`observatory_prospective.rs`](../adapters/chronosentiment/src/decision_support/observatory_prospective.rs) | Prospective paper clock | **YES** — `generate_prospective_decision` used by live execution |
| [`observatory_maturity.rs`](../adapters/chronosentiment/src/decision_support/observatory_maturity.rs) | Maturity countdown, session horizon | **YES** — used by execution modules |
| [`observatory_execution.rs`](../adapters/chronosentiment/src/decision_support/observatory_execution.rs) | P.E targeted execution, Execution Contract v0 | **YES** — used by portfolio replay v0/v021 |
| [`observatory_registry.rs`](../adapters/chronosentiment/src/decision_support/observatory_registry.rs) | P.1 policy registry, C3-002 binding | **YES** — used by observatory_slice |
| [`observatory_live_execution.rs`](../adapters/chronosentiment/src/decision_support/observatory_live_execution.rs) | P.E.2 live execution path | **YES** — used by live execution PE3 |
| [`observatory_historical_pe2.rs`](../adapters/chronosentiment/src/decision_support/observatory_historical_pe2.rs) | P.E.2.H historical lifecycle validation | Low — research-only |
| [`observatory_historical_pe3.rs`](../adapters/chronosentiment/src/decision_support/observatory_historical_pe3.rs) | P.E.3 historical replay | Low — research-only |
| [`observatory_live_execution_pe3.rs`](../adapters/chronosentiment/src/decision_support/observatory_live_execution_pe3.rs) | P.E.3 live execution | Low — research-only |

**Important:** `observatory_slice.rs` and `observatory_historical.rs` contain `generate_historical_replay_decision` and `SealedDecisionRecord` which are used by `portfolio_replay_v0.rs` and `portfolio_replay_v021.rs`. These cannot be deleted until the portfolio replay modules are either retired or refactored to use the new MVP `DecisionRecord`.

### 2.2 Binary entry points (adapters/chronosentiment/src/bin/)

| Binary | Observatory role | Action |
|---|---|---|
| [`csp006_p_observatory.rs`](../adapters/chronosentiment/src/bin/csp006_p_observatory.rs) | P.3–P.7 sealed-then-measured path | **RETIRE** after MVP-002 |
| [`csp006_p_prospective.rs`](../adapters/chronosentiment/src/bin/csp006_p_prospective.rs) | Prospective paper clock | **RETIRE** after MVP-003 |
| [`csp006_p_observe.rs`](../adapters/chronosentiment/src/bin/csp006_p_observe.rs) | Maturity countdown / observation attachment | **RETIRE** after MVP-008 |
| [`csp006_p_replay.rs`](../adapters/chronosentiment/src/bin/csp006_p_replay.rs) | P.H historical replay | **RETIRE** — evidence already in `historical_replay_v1/` |
| [`csp006_p_execute.rs`](../adapters/chronosentiment/src/bin/csp006_p_execute.rs) | P.E.1 targeted execution | **RETIRE** — evidence in `targeted_execution_v0/` |
| [`csp006_p_live_execute.rs`](../adapters/chronosentiment/src/bin/csp006_p_live_execute.rs) | P.E.2 live execution | **RETIRE** — AWAITING_NEXT_SESSION; superseded by MVP user execution |
| [`csp006_p_historical_pe2.rs`](../adapters/chronosentiment/src/bin/csp006_p_historical_pe2.rs) | P.E.2.H historical validation | **RETIRE** — evidence in `historical_pe2_replay/` |
| [`csp006_p_replay_pe3.rs`](../adapters/chronosentiment/src/bin/csp006_p_replay_pe3.rs) | P.E.3 historical replay | **RETIRE** — P.E.3 not started; no evidence to preserve |

### 2.3 Test files (adapters/chronosentiment/tests/)

| Test file | What it tests | Action |
|---|---|---|
| [`csp006p_observatory_tests.rs`](../adapters/chronosentiment/tests/csp006p_observatory_tests.rs) | P.1 registry | **RETIRE** with registry |
| [`csp006p_observatory_slice_tests.rs`](../adapters/chronosentiment/tests/csp006p_observatory_slice_tests.rs) | P.3–P.7 slice | **RETIRE** with slice |
| [`csp006p_observatory_historical_tests.rs`](../adapters/chronosentiment/tests/csp006p_observatory_historical_tests.rs) | P.H replay | **RETIRE** with historical module |
| [`csp006p_observatory_maturity_tests.rs`](../adapters/chronosentiment/tests/csp006p_observatory_maturity_tests.rs) | Maturity countdown | **RETIRE** with maturity module |
| [`csp006p_observatory_prospective_tests.rs`](../adapters/chronosentiment/tests/csp006p_observatory_prospective_tests.rs) | Prospective clock | **RETIRE** with prospective module |
| [`csp006p_observatory_execution_tests.rs`](../adapters/chronosentiment/tests/csp006p_observatory_execution_tests.rs) | P.E execution | **RETIRE** with execution module |
| [`csp006p_observatory_live_execution_tests.rs`](../adapters/chronosentiment/tests/csp006p_observatory_live_execution_tests.rs) | P.E.2 live | **RETIRE** with live execution module |
| [`csp006p_observatory_historical_pe2_tests.rs`](../adapters/chronosentiment/tests/csp006p_observatory_historical_pe2_tests.rs) | P.E.2.H | **RETIRE** with PE2 module |

---

## 3. Infrastructure Observatory API

The `infrastructure/observatory/` directory contains an API server with Observatory-specific routes.

| File | Role | Action |
|---|---|---|
| [`infrastructure/observatory/api/src/handlers/strategy_handlers.rs`](../infrastructure/observatory/api/src/handlers/strategy_handlers.rs) | `observatory_handler()` route | **RETIRE** `observatory_handler` — replace with MVP Decision Feed API |
| [`infrastructure/observatory/api/src/routes/strategy_routes.rs`](../infrastructure/observatory/api/src/routes/strategy_routes.rs) | `/observatory` route registration | **RETIRE** `/observatory` route — replace with `/decisions` |

**Note:** Other routes in `strategy_handlers.rs` (evaluate, inspect, replay, GA, health) are **not** Observatory-specific and must be preserved.

---

## 4. Portfolio replay modules — dependency on Observatory types

These modules use `SealedDecisionRecord`, `generate_historical_replay_decision`, and `observatory_execution` types. They are **not** pure Observatory code — they implement the stop-loss research dataset construction. They must be **refactored** to use the new MVP `DecisionRecord` type, not deleted.

| File | Observatory dependency | Action |
|---|---|---|
| [`adapters/chronosentiment/src/decision_support/portfolio_replay_v0.rs`](../adapters/chronosentiment/src/decision_support/portfolio_replay_v0.rs) | `SealedDecisionRecord`, `observatory_execution`, `observatory_historical` | **REFACTOR** — decouple from Observatory types; keep stop research logic |
| [`adapters/chronosentiment/src/decision_support/portfolio_replay_v021.rs`](../adapters/chronosentiment/src/decision_support/portfolio_replay_v021.rs) | Same | **REFACTOR** — same |

These are the modules that produced `stop_research_dataset_v01`. Their logic is valuable; only the Observatory type dependency needs to be replaced.

---

## 5. chrono-ui / frontend Observatory components

Search for Observatory-specific UI components:

```bash
grep -r "Observatory\|observatory\|DecisionFeed\|decision_feed\|SealedDecision\|sealed_decision" \
  chrono-ui/src/ --include="*.ts" --include="*.tsx" -l
```

Any component that renders the old four-screen Observatory UI (Observatory / Decision Feed / Decision Detail / Policy Provenance) should be **retired** and replaced by the three MVP screens defined in CORALYS_DECISION_INTELLIGENCE_MVP_V01.md §14.

---

## 6. What must NOT be deleted

### 6.1 Immutable evidence archives

```text
product_validation/CS-P-006/observatory/ledger.json
product_validation/CS-P-006/observatory/historical_replay_v0/
product_validation/CS-P-006/observatory/historical_replay_v1/
product_validation/CS-P-006/observatory/targeted_execution_v0/
product_validation/CS-P-006/observatory/prospective/
product_validation/CS-P-006/observatory/prospective_execution_v0/
product_validation/CS-P-006/observatory/historical_pe2_replay/
```

These are the evidence records that established the 91/91 lifecycle PASS and P.E.2.H PASS. They are immutable research artifacts, not product code.

### 6.2 Research protocol documents

```text
docs/CS-P-006-P_DECISION_OBSERVATORY.md
docs/CS-P-006-P.H_HISTORICAL_REPLAY.md
docs/CS-P-006-P.H.1_DECISION_EVIDENCE_ENGINE.md
docs/CS-P-006-P.H.2_MARKET_SESSION_HORIZON.md
docs/CS-P-006-P.H.3_DECISION_EVIDENCE_DASHBOARD.md
docs/CS-P-006-P.E_TARGETED_DECISION_EXECUTION.md
docs/CS-P-006-P.E.1_EXECUTION_EVIDENCE_SURFACE.md
docs/CS-P-006-P.E.2_LIVE_EXECUTION_OBSERVATION.md
docs/CS-P-006-P.E.2.H_HISTORICAL_LIFECYCLE_VALIDATION.md
docs/CS-P-006-P.E.3_CORALYS_TARGET_DISCOVERY.md
docs/CS-P-006-P.E.3.A_CORALYS_TARGET_ARTIFACT.md
```

These are the research protocol lineage. They are historical records, not implementation targets.

### 6.3 Frozen policy artifacts

```text
product_validation/CS-P-006/discovery/  (all Search #1 and #2 artifacts)
genomes/  (all frozen genome files)
```

### 6.4 Stop research dataset

```text
datasets/stop_research_dataset_v01.json
datasets/stop_research_dataset_v01.csv
datasets/DATASET_REPORT.md
scripts/build_stop_research_dataset_v01.py
```

### 6.5 Coralys platform infrastructure

```text
coralys-core/
coralys-decision/
coralys-eval/
coralys-ecology/
coralys-moga/
coralys-matching/
coralys-planning/
coralys-policy/
coralys-recommendation/
coralys-simulation/
```

The `observatory` modules inside `coralys-moga/src/observatory.rs` and `coralys-ecology/src/state.rs` (`SearchStateObservatory`) are **MOGA search observability** — they are not the Decision Observatory product and must not be deleted.

### 6.6 historical_runs/

```text
historical_runs/portfolio_v04_1_capital_allocation_experiment/
```

Immutable experiment evidence.

---

## 7. Deletion sequence

Deletions must follow this order to avoid breaking the build:

1. **First:** Retire binary entry points (`csp006_p_*.rs` bins) — these have no dependents.
2. **Second:** Retire test files for Observatory modules.
3. **Third:** Refactor `portfolio_replay_v0.rs` and `portfolio_replay_v021.rs` to remove Observatory type dependencies (replace `SealedDecisionRecord` with new MVP `DecisionRecord`).
4. **Fourth:** Once portfolio replay no longer imports Observatory modules, retire `observatory_slice.rs`, `observatory_historical.rs`, `observatory_prospective.rs`, `observatory_maturity.rs`.
5. **Fifth:** Retire `observatory_execution.rs`, `observatory_registry.rs`, `observatory_live_execution.rs`, `observatory_historical_pe2.rs`, `observatory_historical_pe3.rs`, `observatory_live_execution_pe3.rs`.
6. **Sixth:** Remove `observatory_handler` and `/observatory` route from infrastructure API.
7. **Seventh:** Remove Observatory UI components from chrono-ui.
8. **Last:** Remove `pub mod observatory_*` entries from `adapters/chronosentiment/src/decision_support/mod.rs`.

---

## 8. Acceptance test for deletion

After each deletion step, the following must still pass:

```bash
cargo test -p chronosentiment-adapter
cargo test -p coralys-moga
cargo test -p coralys-ecology
```

The stop research dataset build script must still run:

```bash
python3 scripts/build_stop_research_dataset_v01.py
```

---

## 9. Status

**DRAFT — no deletions executed yet.**

This document is a pre-deletion analysis. No files have been removed. Owner review is required before any deletion is executed.

The implementation sequence in CORALYS_DECISION_INTELLIGENCE_MVP_V01.md §21 (MVP-001 through MVP-010) must reach MVP-002 (immutable Decision Ledger implemented) before any Observatory code is retired, so that the replacement exists before the original is removed.