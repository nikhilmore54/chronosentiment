# CS-P-006-B.1 — Chronological partition freeze

**Document type:** Frozen research-protocol partition  
**Status:** PASS — development / selection / evaluation frozen  
**Date:** 2026-08-14  
**Parent:** CS-P-006-B, CS-P-006-S1  
**Does not:** invent G-GATE 55/27/28, copy CS-P-004 year folds, mutate B3/B4, open B5, freeze a numeric fitness formula, freeze Decision Engine v1.0  

`.cursor/rules/chronosentiment-core.mdc`: deterministic partition of the certified grid; evaluate across all seven instruments together; no invented methodology.

---

## What this freezes

Exactly one object: the chronological **dataset partition** of the CS-P-006-S1 39 common timestamps.

```text
CS-P-006-B.1
├── split manifest
├── development timestamps     (protocol TRAIN)
├── selection timestamps       (protocol VALIDATION)
├── evaluation timestamps      (protocol TEST)
├── observation counts
├── boundary timestamps
├── partition hash
└── authorization = PASS
```

Domain names in code: `development`, `selection`, `evaluation`.  
Protocol roles in this document: TRAIN / VALIDATION / TEST.

Atomic unit:

```text
timestamp T
    ├── HDFCBANK.NS
    ├── ICICIBANK.NS
    ├── INFY.NS
    ├── RELIANCE.NS
    ├── TCS.NS
    ├── IDEA.NS
    └── MAHABANK.NS
```

All seven move together. An instrument at T cannot sit in development while a peer at the same T sits in selection.

---

## Algorithm

Certified coverage: 39 unique month-end 15:30 UTC timestamps, 2021-10-31 through 2024-12-31, every name TMV-complete (273 states).

Method: **contiguous equal thirds** of that timestamp grid.

* 39 is divisible by 3, so each partition has 13 timestamps and 91 observations.
* Tie-break: **none applicable**. If a future grid were not divisible by 3, remainder timestamps would be assigned to development so evaluation never gains extra history.

Not used: G-GATE 55/27/28, CS-P-004 2022/2023/2024 year folds, per-row assignment.

---

## Frozen boundaries

| Partition | Protocol role | n timestamps | n observations | First timestamp | Last timestamp |
|-----------|---------------|--------------|----------------|-----------------|----------------|
| development | TRAIN | 13 | 91 | 2021-10-31 15:30 UTC | 2022-10-31 15:30 UTC |
| selection | VALIDATION | 13 | 91 | 2022-11-30 15:30 UTC | 2023-11-30 15:30 UTC |
| evaluation | TEST | 13 | 91 | 2023-12-31 15:30 UTC | 2024-12-31 15:30 UTC |

Chronological order: development < selection < evaluation.

`PolicyArtifact` windows (`csp006a.policy_artifact.1` field names are frozen provenance):

* train: `[2021-10-31T15:30:00Z, 2022-11-30T15:30:00Z)`
* validation: `[2022-11-30T15:30:00Z, 2023-12-31T15:30:00Z)`
* test: `[2023-12-31T15:30:00Z, 2024-12-31T15:30:01Z)`

Partition hash:

`4354c81ef546003b1d11ec98cba83dd5f8c56b13c8b6055b8451614abdc4cfca`

Snapshot identity (S1):

`c21ec256133fb63656b35e68c5e1e72b72751ad2fb45f11c12f99ddb34a628c6`

Manifest: `product_validation/CS-P-006/partition/`

---

## What Coralys may and must not see

```text
Coralys may see:
    development outcomes          → evolution / fitness
    selection outcomes            → selection feedback only

Coralys must never see:
    evaluation outcomes
    evaluation performance
    evaluation-derived fitness
```

ChronoSentiment `decide_at(T)` never receives outcomes on any partition.

---

## What this does not freeze

* Numeric fitness formula. CS-P-006-B §5 already froze the **rules** (observation-path returns; NO_TRADE is standing aside; lake SHORT is not the objective; aggregate across seven instruments). Implementing that formula is CS-P-006-C, not a second date invention.
* A candidate policy.
* Decision Engine v1.0.

---

## Authorization

`coralys_search_is_authorized() == true` when S1 is certified and this partition is frozen.

CS-P-006-C has run against this partition. It must not read the evaluation partition for search.

---

## Code

| Piece | Location |
|-------|----------|
| Domain partition | `adapters/chronosentiment/src/decision_support/dataset_partition.rs` |
| Provenance mapping onto 006-A windows | `TrainingProvenance::from_chronological_partition` |
| Tests | `adapters/chronosentiment/tests/dataset_partition_tests.rs` |
| Naming inventory | `docs/CS-P-006-B.1_NAMING_DISCIPLINE_AUDIT.md` |

Engine version remains **`unfrozen-dev`**. No real capital.
