# Coralys Benchmark Qualification Specification
## GOV-008 — Platform Normative Document

*Applies to: CVRP, Workforce Scheduling, Crew Scheduling, Routing, and all future Coralys optimization domains.*
*Domain-specific qualification matrices are layered on top of this specification as named sections (Section A, Section B, …).*
*Supersedes `benchmark_qualification_matrix.md`.*

---

## Normative Principle

> **A benchmark comparison is valid only when Coralys and the published benchmark
> solve the same optimization problem under equivalent constraints and evaluation
> semantics.**

This is the governing rule for every benchmark comparison across every Coralys
optimization domain. Every section below exists to verify that equivalence — or
to document where it has not yet been confirmed.

---

## 1. Terminology

| Term | Definition |
|------|-----------|
| **Benchmark Specification** | The external, authoritative definition of the benchmark problem, constraints, and BKS — independent of any Coralys implementation |
| **Coralys Registry** | Coralys's internal representation of benchmark metadata; may differ from the Benchmark Specification if incorrectly populated |
| **BKS** | Best Known Solution — the reference objective value from the Benchmark Specification |
| **Gap** | `(best_found − BKS) / BKS × 100%`. Negative = better than reference. |
| **Benchmark vehicle count** | The K value specified in the Benchmark Specification |
| **Routes used** | Number of routes in the best solution found by Coralys |
| **Comparison Validity** | Whether the gap is a meaningful apples-to-apples comparison |
| **Optimizer Qualification** | Whether Coralys produces feasible, high-quality solutions |
| **Benchmark Qualification** | Whether the gap measurement is a valid comparison against the Benchmark Specification |

---

## 2. Evidence Levels

Every finding in this document is labeled with one of three evidence levels:

| Level | Symbol | Meaning |
|-------|--------|---------|
| **Observed** | 🔵 | Directly measured in the qualification campaign |
| **Verified** | ✅ | Confirmed against official benchmark documentation or source |
| **Hypothesis** | ⚠ | Plausible explanation requiring external verification |

### Qualification Decision Rules

```
Evidence hierarchy

  Verified
      ↑
  Observed
      ↑
  Hypothesis

Only Verified findings may be cited as release evidence.

Observed findings may be cited only as campaign observations.

Hypotheses shall never be presented as product conclusions.
```

A finding labeled ⚠ Hypothesis must not be cited as a conclusion in any
release report, release note, or external communication until it is upgraded
to ✅ Verified.

---

## 3. Qualification Methodology

### Stage A — Metadata Qualification

Confirms that the benchmark instance is correctly registered before running:

- [ ] Vehicle count matches Benchmark Specification
- [ ] Capacity matches Benchmark Specification
- [ ] BKS value matches current Benchmark Specification (or provenance is documented)
- [ ] Distance metric is identified (EUC_2D, EXPLICIT, etc.)
- [ ] Fleet semantics are documented (exactly K vehicles, or at most K)
- [ ] **Objective function equivalence confirmed** — the objective optimized by Coralys matches the objective for which the BKS was established (distance only, distance + fleet, lexicographic, hierarchical, etc.)

### Stage B — Execution Qualification

Confirms that the solution produced is a valid comparison after running:

- [ ] Routes used = benchmark vehicle count (or fleet semantics allow fewer, documented)
- [ ] Zero capacity violations
- [ ] All customers served exactly once (no duplicates, no omissions)
- [ ] Distance semantics match Benchmark Specification (e.g. TSPLIB integer rounding for EUC_2D)
- [ ] **Objective value computed using the benchmark objective definition** — not a proxy or approximation

**A gap is only reported as `Qualified` when both Stage A and Stage B pass.**
When either stage fails, the gap is reported as `Not Comparable` or `Invalid`.

### Comparison Certificate Format

For every instance where a gap is claimed, the following certificate must exist:

```
Instance:              <name>
Benchmark Spec:        <source document or URL>
Benchmark vehicles:    K
Routes used:           N   ← must equal K (or fleet semantics justify N < K)
Capacity violations:   0
Customers served:      all (no duplicates, no omissions)
Distance semantics:    <e.g. TSPLIB integer rounding: nint(sqrt(...))>
Objective definition:  <e.g. minimize total route distance>
Gap:                   X%
Qualification outcome: Qualified | Provisionally Qualified | Not Comparable | Invalid | Under Investigation
```

---

## 4. Qualification Outcomes

| Outcome | Meaning |
|---------|---------|
| **Qualified** | Stage A and Stage B both pass; gap is a valid comparison against the Benchmark Specification |
| **Provisionally Qualified** | Stage A passes; Stage B evidence pending |
| **Not Comparable** | Stage A or Stage B fails; gap is not a valid comparison; reason documented |
| **Invalid** | Execution failed (infeasible, crash, timeout with no solution) |
| **Under Investigation** | Evidence conflict; root cause not yet determined |

---

## 5. Verification Dimensions

For each benchmark family, five dimensions must be verified before gap
measurements are considered directly comparable to published BKS values:

| Dimension | Description |
|-----------|-------------|
| **Vehicle semantics** | Does Coralys enforce exactly K vehicles, or at most K? |
| **Distance semantics** | Does Coralys use the same distance computation as the Benchmark Specification? |
| **BKS provenance** | Are the Coralys Registry BKS values from the current Benchmark Specification, or from an earlier publication? |
| **Route count match** | Does the best solution use exactly the benchmark vehicle count? |
| **Objective equivalence** | Does Coralys optimize the same objective as the Benchmark Specification? |

---

## 6. Campaign Integrity

Every qualification campaign shall record the following provenance data.
Without this, reproducibility degrades over time and findings cannot be
attributed to a specific engine state.

| Field | Description |
|-------|-------------|
| **Coralys version** | Semantic version of the Coralys engine |
| **Git commit** | Full commit hash of the source used to build the campaign binary |
| **Configuration** | All non-default solver parameters (population size, generations, operators) |
| **Random seed** | Seed used for reproducibility (or "non-deterministic" if not fixed) |
| **Benchmark corpus version** | Version or date of the benchmark `.vrp` file set |
| **Registry version** | Version of the Coralys Registry used |
| **Execution date** | ISO 8601 date of campaign execution |
| **Compiler version** | Rust toolchain version (e.g. `rustc 1.79.0`) |
| **Platform** | OS, CPU architecture, core count |

*Campaign v1.1 provenance: recorded in `archive/research_outputs/campaign_report.md` header.*

---

## 7. Family Qualification Matrix

Status definitions:

| Status | Meaning |
|--------|---------|
| **Provisionally Qualified** | No systematic negative gaps; BKS provenance verified; route count pending |
| **Stage A Pending** | Metadata not yet fully verified |
| **Stage B Pending** | Execution certificate not yet collected |
| **Under Investigation** | Systematic anomalies observed; root cause not yet determined |

| Family | N | AvgCust | Vehicle semantics | Distance semantics | BKS provenance | Route count | Status |
|--------|---|---------|-------------------|--------------------|----------------|-------------|--------|
| A | 27 | 51 | ⚠ Pending | 🔵 TspLibEuc2D | ✅ CVRPLIB current | ⚠ Stage B pending | **Provisionally Qualified** |
| B | 23 | 52 | ⚠ Pending | 🔵 TspLibEuc2D | ✅ CVRPLIB current | ⚠ Stage B pending | **Provisionally Qualified** |
| E | 13 | 33 | ⚠ Pending | 🔵 TspLibEuc2D | ✅ CVRPLIB current | ⚠ Stage B pending | **Provisionally Qualified** |
| P | 24 | 55 | ⚠ Pending | 🔵 TspLibEuc2D | ⚠ One instance suspect (P-n55-k8, gap=−2.04%) | ⚠ Stage B pending | **Provisionally Qualified** |
| X | 28 | 101 | ⚠ Pending | 🔵 TspLibEuc2D | ✅ CVRPLIB current (Uchoa et al. 2017) | ⚠ Stage B pending | **Provisionally Qualified** |
| M | 2 | 175 | ⚠ Pending | 🔵 TspLibEuc2D | ⚠ Reference value provenance requires verification. Campaign observed negative gaps (−2.09%, −2.99%). | ⚠ Stage B pending | **Under Investigation** |
| CMT | 14 | 107 | ⚠ Pending | 🔵 TspLibEuc2D | ⚠ Reference value provenance requires verification against current CVRPLIB/OR-Library sources. Campaign observed negative gaps (−5% to −10.54%). | ⚠ Stage B pending | **Under Investigation** |
| Tai | 13 | 121 | ⚠ Pending | 🔵 TspLibEuc2D | ⚠ Reference value provenance requires verification against current CVRPLIB/OR-Library sources. Campaign observed negative gaps (−0.21% to −7.61%). | ⚠ Stage B pending | **Under Investigation** |

---

## 6. Campaign Evidence — Observed Findings

### 6.1 Optimizer Qualification (Evidence Level: 🔵 Observed)

| Finding | Level | Value |
|---------|-------|-------|
| Feasibility rate | 🔵 Observed | 100% (116/116 completed instances feasible) |
| A/B/E family MedGap | 🔵 Observed | 0.00% |
| A/B/E family NearOpt% | 🔵 Observed | ~100% |
| X family MedGap | 🔵 Observed | ~1.5% (harder large instances) |
| CMT family MedGap | 🔵 Observed | negative (comparison validity unconfirmed) |
| Tai family MedGap | 🔵 Observed | negative (comparison validity unconfirmed) |

### 6.2 Negative Gap Instances (Evidence Level: 🔵 Observed — comparison validity ⚠ Pending)

The following instances produced solutions shorter than the registry BKS.
**These gaps are observed measurements, not verified improvements.**
Comparison validity requires Stage B certification (route count, capacity,
customer coverage).

| Instance | Family | Cust | BenchVeh | Registry BKS | Best Found | Gap% | Comparison Validity |
|----------|--------|------|----------|-------------|------------|------|---------------------|
| CMT13 | CMT | 120 | 11 | 1541.14 | 1038 | −32.65% | ⚠ Stage B pending |
| CMT9 | CMT | 150 | 14 | 1162.55 | 1040 | −10.54% | ⚠ Stage B pending |
| CMT7 | CMT | 75 | 11 | 909.68 | 832 | −8.54% | ⚠ Stage B pending |
| Tai75d | Tai | 75 | 9 | 1468.73 | 1354 | −7.61% | ⚠ Stage B pending |
| CMT10 | CMT | 199 | 18 | 1395.85 | 1305 | −6.51% | ⚠ Stage B pending |
| CMT6 | CMT | 50 | 6 | 555.43 | 521 | −6.20% | ⚠ Stage B pending |
| CMT14 | CMT | 100 | 10 | 866.37 | 820 | −5.35% | ⚠ Stage B pending |
| CMT8 | CMT | 100 | 9 | 865.94 | 821 | −5.19% | ⚠ Stage B pending |
| M-n200-k17 | M | 199 | 17 | 1373.00 | 1332 | −2.99% | ⚠ Stage B pending |
| Tai100a | Tai | 100 | 11 | 2141.07 | 2097 | −2.06% | ⚠ Stage B pending |
| Tai75b | Tai | 75 | 9 | 1407.89 | 1379 | −2.05% | ⚠ Stage B pending |
| M-n151-k12 | M | 150 | 12 | 1053.00 | 1031 | −2.09% | ⚠ Stage B pending |
| P-n55-k8 | P | 54 | 8 | 588.00 | 576 | −2.04% | ⚠ Stage B pending |
| Tai100b | Tai | 100 | 11 | 1940.55 | 1935 | −0.29% | ⚠ Stage B pending |
| Tai75a | Tai | 75 | 10 | 1618.36 | 1615 | −0.21% | ⚠ Stage B pending |

### 8.3 Root Cause Hypotheses (Evidence Level: ⚠ Hypothesis)

The following are plausible explanations for the negative gaps. None has been
confirmed. They must not be cited as conclusions until verified.

| Hypothesis | Applies to | Verification method |
|------------|-----------|---------------------|
| Routes used > benchmark vehicle count (fleet semantics mismatch) | All negative-gap instances | `routes=N/M` from next campaign run |
| Registry BKS sourced from original 1979 heuristic paper, not current CVRPLIB | CMT family | Cross-check registry values against CVRPLIB.org and OR-Library |
| Registry BKS sourced from original Taillard 1993 paper, not current CVRPLIB | Tai family | Cross-check registry values against CVRPLIB.org |
| Capacity violations present in best solution | Any family | `capacity_violations` field in next run |
| Distance rounding difference (float vs integer) | Any family | `fp=` field in completion log |

---

## 7. Qualification Report Language

Until Stage B certificates are collected, the qualification report must use the
following conservative language for CMT, Tai, and M families:

> **Qualification Finding — CMT, Taillard, and M Benchmark Provenance**
>
> Multiple CMT, Taillard, and M instances exhibit negative gaps (up to −10.54%
> for CMT, −7.61% for Taillard, −2.99% for M) relative to the registry
> reference values. The magnitude and consistency of these differences indicate
> that the registry reference values may not be directly comparable with the
> optimization problem currently solved by Coralys. The root cause has not been
> determined. Working hypotheses include fleet semantics mismatch (routes used
> exceeding benchmark vehicle count) and reference value provenance differences
> (original publication values vs. current CVRPLIB catalog). Before these
> results are interpreted as optimizer improvements, Stage B comparison
> certificates must be collected for each affected instance. This verification
> is scheduled for the next qualification run.

---

## 8. Families with High Confidence (A, B, E, X)

The A, B, E, and X families show no systematic negative gaps and use
well-known current CVRPLIB reference values. These families are
**Provisionally Qualified** pending Stage B route count confirmation.

| Family | MedGap | AvgGap | Solved% | NearOpt% | Assessment |
|--------|--------|--------|---------|----------|------------|
| A | 0.00% | ~0.1% | ~89% | ~100% | Provisionally Qualified |
| B | 0.00% | ~0.1% | ~87% | ~100% | Provisionally Qualified |
| E | 0.00% | ~0.0% | ~100% | ~100% | Provisionally Qualified |
| X | ~1.5% | ~2.0% | ~15% | ~60% | Provisionally Qualified (harder instances) |

*Note: Final percentages will be updated when campaign completes at 144/144
and Stage B certificates are collected.*

---

## 9. Required Next Steps

### Immediate (next campaign run)

1. Run new binary — emits `routes=N/M` (with ⚠ flag if routes > benchmark)
   in the completion log for every instance.
2. Collect Stage B certificates for all 15 negative-gap instances.
3. For each certificate: confirm routes_used, capacity_violations=0,
   customers_served=all, distance_semantics=TSPLIB integer rounding.

### After route count data is available

4. If `routes_used > benchmark_vehicles` for CMT/Tai instances → gap is
   `NOT COMPARABLE`; document fleet semantics mismatch.
5. If `routes_used == benchmark_vehicles` for CMT/Tai instances → escalate
   BKS provenance investigation (cross-check CVRPLIB.org and OR-Library).
6. Update family status from **Under Investigation** to either
   **Provisionally Qualified** or **Not Comparable — Pending Resolution**.

### Freeze criteria for v2.0 baseline

- All 144 instances have Stage B certificates
- All negative-gap instances have `Qualified` or `Not Comparable` outcome
  with documented reason
- All ⚠ Hypothesis findings are either upgraded to ✅ Verified or
  explicitly closed as `Not Reproducible`
- **No unresolved regression without an assigned disposition** — every
  regression must be one of: accepted, fixed, benchmark issue, specification
  mismatch, or documented limitation

---

## 10. Qualification History

| Version | Campaign | Registry Version | Notes |
|---------|----------|-----------------|-------|
| v1.0 | Campaign v1.1 | Registry v1.0 | Initial qualification matrix created from Campaign v1.1 evidence (116/144 complete at time of writing) |
| v1.1 | Campaign v1.1 | Registry v1.0 | Renamed to Benchmark Qualification Specification; added Evidence Levels, two-stage qualification, Comparison Validity flag, Hypothesis table, conservative language for CMT/Tai/M |
| v1.2 | Campaign v1.1 | Registry v1.0 | Elevated to platform normative document GOV-008; added Normative Principle, Campaign Integrity, Qualification Outcomes table, Objective Function Equivalence in Stage A/B, Root Cause Hypotheses section, Registry vs Benchmark Specification distinction throughout, Qualification Decision Rules, extended freeze criteria with regression disposition requirement |
