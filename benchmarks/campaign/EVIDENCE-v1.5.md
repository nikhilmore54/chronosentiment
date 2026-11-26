# Campaign v1.5 Evidence Report

**Document ID:** EVIDENCE-v1.5  
**Status:** Baseline Frozen  
**Date:** 2026-07-09  
**Campaign log:** `benchmarks/campaign/campaign_v1.5.log`  
**Binary:** `target/release/campaign` (built 2026-07-09 17:36)  
**Governance reference:** GOV-008 v1.2, GOV-009 v1.2, GOV-010 v1.1  

---

## 1. Scope

This report records the measured evidence produced by Campaign v1.5 across the full CVRPLIB ≤200-customer EUC_2D benchmark suite. It is the primary evidence document for Stage 2 qualification of FCF, FCS, and FUC-001 under GOV-010.

Measured facts and interpretations are separated throughout. Where interpretation status is pending external verification (P4, P5), this is stated explicitly.

---

## 2. Instance Coverage

| Category                          | Count   |
|-----------------------------------|--------:|
| Total in registry                 |     376 |
| Skipped (>200 customers)          |     232 |
| **Run**                           | **144** |
| FC-2 FAIL (optimization skipped)  |       4 |
| Optimized                         |     140 |
| BKS-known (gap comparable)        |     116 |
| X-family (no BKS)                 |      28 |

---

## 3. Solution Quality (116 BKS-known instances)

The 28 X-family instances have no published BKS and are excluded from gap analysis.

| Classification   | Count | % of 116 | Avg gap  |
|------------------|------:|----------:|---------:|
| ✅ Solved         |    48 |    41.4%  |  −4.17%  |
| 🟢 Near-optimal  |    47 |    40.5%  |  −1.68%  |
| 🟡 Competitive   |    17 |    14.7%  |  +1.67%  |
| 🟠 Weak          |     4 |     3.4%  | +11.92%  |

**Observed:** 95 of 116 BKS-known instances are within 1% of benchmark (Solved + Near-optimal). 112 of 116 are within 5%.

**Derived:** 81.9% within 1%; 96.6% within 5%.

**Gap distribution (116 instances):**

| Metric      | Value    |
|-------------|----------|
| Average gap | −1.75%   |
| Median gap  | 0.00%    |
| Best gap    | −32.65%  |
| Worst gap   | +9.99%   |

The average gap (−1.75%) is influenced by a relatively small number of negative-gap instances. The median gap (0.00%) is less sensitive to these outliers and therefore provides a more robust measure of central tendency. The causes of the negative-gap observations remain under investigation (P4, P5).

**Interpretation status — under investigation.** Contributing factors may include integer-vs-fractional objective representations, benchmark provenance differences, or benchmark fleet-semantics differences. Definitive attribution requires completion of P4 (benchmark provenance verification) and P5 (fleet semantics verification).

---

## 4. Performance

**Measured:**

| Metric                            | Value      |
|-----------------------------------|------------|
| FC-2 FAIL instances               | 4          |
| Average runtime (optimized inst.) | 27,638 ms  |
| Instances optimized               | 140        |

**Derived estimate:**

A simple lower-bound estimate of optimizer time avoided is 4 × average runtime ≈ **110 s**. Actual savings may differ because infeasible instances are not necessarily representative of average runtime.

Runtime measurements were collected using the Campaign v1.5 binary under the execution environment used for this campaign and should be interpreted as comparative campaign metrics rather than absolute performance benchmarks.

---

## 5. Fleet Utilization (FUC-001)

Measured across 131 instances (excludes FC-2 FAIL and instances where FUC did not emit).

**Observed:**

| Metric                    | Value  |
|---------------------------|--------|
| Average fleet utilization | 94.7%  |
| Average RCR               | 0.495  |
| Average CV                | 0.097  |

**Interpretation:** An average CV of 0.097 is consistent with highly uniform fleet loading across the evaluated instances. An average RCR of 0.495 is consistent with moderate residual-capacity concentration — one or two lightly-loaded routes per instance.

### Packing Classification Distribution

| Classification       | Count | % of 131 |
|----------------------|------:|----------:|
| HIGHLY_CONSOLIDATED  |    10 |     7.6%  |
| WELL_PACKED          |    49 |    37.4%  |
| BALANCED             |    69 |    52.7%  |
| UNEVEN               |     3 |     2.3%  |
| CAPACITY_LOOSE       |     0 |     0.0%  |
| EMPTY                |     0 |     0.0%  |

**Observed:** 128 of 131 instances were classified as BALANCED, WELL_PACKED, or HIGHLY_CONSOLIDATED.

**Derived:** 97.7% of evaluated instances fall into those three classifications.

No instance produced a CAPACITY_LOOSE or EMPTY classification. The three UNEVEN instances are candidates for further investigation. Their root cause has not yet been determined.

---

## 6. Feasibility Check Framework (FCF)

FCF ran on all 144 instances before optimization.

| Gate   | Function                              | Outcome              |
|--------|---------------------------------------|----------------------|
| FC-1   | Structural validity                   | 144/144 PASS         |
| FC-2.5 | Benchmark consistency                 | 144/144 PASS         |
| FC-2   | Capacity feasibility                  | 140 PASS, 4 FAIL     |
| FC-3   | Bin-pack FFD lower bound              | Non-blocking         |

### FCF Correctness

| Metric                                              | Value              |
|-----------------------------------------------------|--------------------|
| FC-2 FAIL instances                                 | 4                  |
| FC-2 FAIL instances later solved                    | 0                  |
| FC-2 false positives (feasible, wrongly failed)     | 0 (not observed)   |
| FC-2 false negatives (infeasible, wrongly passed)   | Not observed       |

**FC-2 precision (observed):** 4/4 = 100%, subject to campaign corpus. No incorrect FC-2 FAIL outcome was observed during Campaign v1.5. The 4 FC-2 FAIL instances had total demand exceeding fleet capacity and were correctly identified as mathematically infeasible.

---

## 7. Fleet Constraint Semantics (FCS)

FCS emitted for all 144 instances. Family-level constraint hypotheses applied:

| Family | Constraint hypothesis | Basis       |
|--------|-----------------------|-------------|
| A      | ATMOST(K)             | Hypothesis  |
| B      | ATMOST(K)             | Hypothesis  |
| E      | ATMOST(K)             | Hypothesis  |
| P      | ATMOST(K)             | Hypothesis  |
| M      | ATMOST(K)             | Hypothesis  |
| Tai    | ATMOST(K)             | Hypothesis  |
| CMT    | Unspecified           | Unknown     |
| X      | ATMOST(K)             | Hypothesis  |

All instances where routes_used ≤ K produced outcome=VALID. No FCS INVALID outcomes were recorded in v1.5.

FCS provides an explicit qualification of fleet-constraint semantics during campaign evaluation. When routes_used < K, an FCS VALID result indicates that the solution is considered permissible under the currently assigned ATMOST(K) semantics. These assignments remain hypotheses pending P5. Remaining negative-gap investigations relate to benchmark provenance and objective representation rather than fleet usage, and require completion of P4 and P5.

---

## 8. Qualification Capability Status (GOV-010 reference)

| Capability | Status                        | Notes                                        |
|------------|-------------------------------|----------------------------------------------|
| FCF        | Stage 2 Evidence Complete     | Full campaign evidence available (144 inst.) |
| FCS        | Stage 2 Evidence Complete     | Full campaign evidence available (144 inst.) |
| FUC-001    | Stage 2 Evidence Complete     | Full campaign evidence available (131 inst.) |
| EXEC-CERT  | Stage 1                       | Not yet implemented                          |

Stage 2 Evidence Complete means the data collection objective for Stage 2 validation has been satisfied. Promotion decisions remain subject to evidence review under GOV-010 and are not made by this report.

---

## 9. Evidence Status

| Capability        | Evidence Status                                           |
|-------------------|-----------------------------------------------------------|
| FCF               | Verified in Campaign v1.5                                 |
| FCS               | Verified in Campaign v1.5; benchmark semantics pending P5 |
| FUC-001           | Verified in Campaign v1.5                                 |
| Gap Qualification | Verified operationally; benchmark provenance pending P4   |

---

## 10. Open Items

| ID  | Item                                                              | Priority |
|-----|-------------------------------------------------------------------|----------|
| P4  | Benchmark provenance verification — CMT/Tai/M vs CVRPLIB.org     | High     |
| P5  | Fleet semantics verification — ATMOST vs EXACT from spec docs     | High     |
| P2  | Execution Certificate — per-instance PASS/FAIL artifact           | Medium   |

P4 and P5 are prerequisites for resolving the negative-gap interpretation and promoting FCS family assignments from Hypothesis to Verified.

---

## 11. Conclusion

Campaign v1.5 demonstrates that Coralys can now qualify its own results — explaining not only how good a solution is, but also whether it is comparable, feasible, and operationally sound. The qualification pipeline (FCF → FCS → FUC-001 → Gap) operated correctly across all 144 instances with no observed false positives, no capacity violations, and 97.7% of fleet packings classified BALANCED or better.

This evidence satisfies the data collection objective for Stage 2 validation under GOV-010. Promotion decisions remain subject to evidence review and completion of the outstanding provenance and fleet-semantics verification activities (P4, P5).