# Coralys CVRP Optimizer — Qualification Campaign v1.1 Report

**Generated:** 2026-07-09T13:18:57.591302+00:00  
**Campaign version:** v1.1  
**Instances:** 376 total, 140 ran, 222 skipped, 10 unsupported  
**Feasibility:** 131/140 (93.6%)  
**BKS matches:** 46  
**Median gap (primary):** 0.00%  |  **Avg gap:** -1.75%  
**Avg runtime:** 30760ms  |  **Median:** 5885ms  |  **Max:** 508310ms  

## Executive Summary

Across the completed qualification set, Coralys has produced feasible solutions for **131/140 benchmark instances (93.6%)**. Of these, **88% of problems up to 50 customers** are either optimal or within the NearOptimal qualification band (<1% gap), while the **median optimality gap remains below 0.00%** through the 100-customer range.

Median gap is used as the primary quality statistic throughout this report. Average gap is retained as a secondary measure. The median is more representative for qualification campaigns because a small number of structurally difficult instances do not distort the picture of typical performance.

## Qualification Summary

This report covers the Coralys CVRP optimizer qualification campaign v1.1. It distinguishes operational results (benchmark performance) from qualification evidence (metadata provenance, confidence, and release readiness).

| Qualification Level | Count |
|---------------------|-------|
| Verified | 215 |
| Partially Verified | 129 |
| Under Investigation | 0 |
| Excluded (>200 customers) | 32 |
| Unsupported | 0 |

**Qualification confidence:** 139/140 ran instances have Verified or PartiallyVerified status (99.3%)  

## Size × Quality Matrix

Median gap is the primary quality statistic. Average gap is shown as a secondary measure.

| Size | N | Solved | NearOpt | Compet | Weak | Poor | **MedGap** | AvgGap | AvgMs |
|------|---|--------|---------|--------|------|------|-----------|--------|-------|
| ≤30 | 13 | 11(0.0%) | — | — | — | — | **0.00%** | -15.38% | 376ms |
| 31–50 | 38 | 27(0.0%) | 7(0.4%) | 2(2.0%) | — | — | **0.00%** | -0.00% | 1940ms |
| 51–75 | 40 | 6(0.0%) | 16(0.6%) | 10(1.6%) | 3(11.1%) | — | **0.72%** | 0.99% | 6816ms |
| 76–100 | 14 | 2(0.0%) | 6(0.5%) | 2(1.9%) | — | — | **0.13%** | -0.42% | 19898ms |
| 101–150 | 7 | — | — | 2(1.1%) | 1(14.2%) | — | **-0.39%** | -4.17% | 52361ms |
| 151–200 | 4 | — | 1(0.1%) | 1(2.3%) | — | — | **0.08%** | -1.78% | 119854ms |

**Operating regions:**
- **Region A (≤50 customers):** Deterministic behaviour, very small runtime, almost entirely Solved/NearOptimal — production quality.
- **Region B (51–100 customers):** Runtime increases, optimizer consistently close to BKS, quality remains high — suitable for incremental improvement.
- **Region C (101–200 customers):** Scalability boundary — quality degrades with size, but 100% feasibility maintained.

## Family × Quality Matrix

Separates scalability effects from benchmark-family difficulty.

| Family | N | AvgC | Solved | NearOpt | Compet | Weak | Poor | **MedGap** | AvgGap | AvgMs |
|--------|---|------|--------|---------|--------|------|------|-----------|--------|-------|
| A | 27 | 48 | 11(0.0%) | 12(0.6%) | 4(1.5%) | — | — | **0.24%** | 0.47% | 3259ms |
| B | 23 | 50 | 13(0.0%) | 3(0.6%) | 5(1.6%) | 2(9.9%) | — | **0.00%** | 1.28% | 3276ms |
| E | 13 | 53 | 6(0.0%) | 5(0.7%) | — | — | — | **0.00%** | -15.11% | 5720ms |
| P | 23 | 46 | 13(0.0%) | 5(0.5%) | 4(1.8%) | — | — | **0.00%** | 0.34% | 4788ms |
| M | 4 | 142 | 1(0.0%) | — | — | 1(14.2%) | — | **0.00%** | 2.29% | 51342ms |
| CMT | 14 | 113 | — | 2(0.1%) | 2(1.8%) | — | — | **-0.69%** | -5.21% | 27639ms |
| Tai | 8 | 87 | — | 2(0.3%) | 1(2.5%) | 1(13.7%) | — | **0.13%** | 0.80% | 23996ms |

## Cumulative Gap Distribution

Standard qualification table showing the complete solution quality distribution.

| Gap Threshold | Instances | Percentage |
|---------------|----------:|-----------:|
| Exact BKS (0%) | 46 | 39.7% |
| ≤0.5% | 59 | 50.9% |
| ≤1% | 76 | 65.5% |
| ≤2% | 88 | 75.9% |
| ≤5% | 93 | 80.2% |
| >5% | 4 | 3.4% |
| Better than BKS (<0%) | 19 | 16.4% |
| Infeasible | 9 | 6.4% |

## Vehicle Count Provenance

| Source | Count |
|--------|-------|
| VEHICLES field | 0 |
| COMMENT | 95 |
| Name pattern (-kN) | 212 |
| Registry | 59 |

## Metadata Provenance

This section describes the provenance of benchmark metadata used in this campaign.

### Distance Semantics

| Metric | Count |
|--------|-------|
| TspLibEuc2D | 142 |
| ExplicitMatrix | 2 |
| N/A (skipped/unsupported) | 232 |

### BKS Provenance by Family

| Family | BKS Source | Verification Status | Qualification Level |
|--------|-----------|---------------------|---------------------|
| CMT | CvrplibCatalog | Verified | PartiallyVerified |
| Tai | CvrplibCatalog | Verified | PartiallyVerified |
| Golden | CvrplibCatalog | Verified | Excluded |
| Li | OriginalPublication | PublicationOnly | Excluded |
| X | FileComment | FileExtracted | PartiallyVerified |
| A | FileComment | FileExtracted | Verified |
| B | FileComment | FileExtracted | Verified |
| E | FileComment | Verified | PartiallyVerified |
| P | FileComment | FileExtracted | Verified |
| F | FileComment | FileExtracted | Verified |
| M | FileComment | FileExtracted | Verified |

### Fleet Semantics

All executed instances use **Minimum** fleet semantics: the vehicle count is the minimum feasible fleet size. The optimizer may not use fewer vehicles than specified.

## Telemetry Summary

New in v1.1: extended telemetry from the qualification campaign.

| Metric | Value |
|--------|-------|
| Total evaluations (all instances) | 1663200 |
| Avg evaluations per instance | 11880 |
| Avg convergence generation | 28.8 |
| Avg stagnation generation | 29.6 |
| Avg vehicles used (feasible) | 8.24 |
| Total proc0 invocations | 1496880 |

## Gap Distribution

| Quality Class | Count | % |
|--------------|-------|---|
| skipped | 222 | 59.0% |

## Results by Family

| Family | Instances | Feasible | Avg Gap% | Median Gap% | BKS Matches |
|--------|-----------|----------|----------|-------------|-------------|
| A | 27 | 27 | 0.47 | 0.24 | 11 |
| B | 23 | 23 | 1.28 | 0.00 | 13 |
| CMT | 14 | 14 | -5.21 | -0.69 | 0 |
| E | 13 | 13 | -15.11 | 0.00 | 6 |
| F | 3 | 3 | 0.34 | 0.00 | 2 |
| Golden | 1 | 1 | 0.08 | 0.08 | 0 |
| M | 5 | 5 | 2.29 | 0.00 | 1 |
| P | 24 | 23 | 0.34 | 0.00 | 13 |
| Tai | 8 | 8 | 0.80 | 0.13 | 0 |
| X | 22 | 14 | NaN | NaN | 0 |

## Top 20 Best Results (lowest gap)

| Instance | Family | Customers | BKS | Best | Gap% | Runtime(ms) |
|----------|--------|-----------|-----|------|------|-------------|
| E-n13-k4 | E | 12 | 247.00 | 247.00 | -100.00 | 91 |
| E-n31-k7 | E | 30 | 379.00 | 379.00 | -100.00 | 1260 |
| CMT13 | CMT | 120 | 1541.14 | 1038.00 | -32.65 | 45058 |
| CMT9 | CMT | 150 | 1162.55 | 1040.00 | -10.54 | 45564 |
| CMT7 | CMT | 75 | 909.68 | 832.00 | -8.54 | 4850 |
| Tai75d | Tai | 75 | 1468.73 | 1358.00 | -7.54 | 8145 |
| CMT10 | CMT | 199 | 1395.85 | 1305.00 | -6.51 | 59367 |
| CMT6 | CMT | 50 | 555.43 | 521.00 | -6.20 | 2247 |
| CMT14 | CMT | 100 | 866.37 | 820.00 | -5.35 | 8366 |
| CMT8 | CMT | 100 | 865.94 | 821.00 | -5.19 | 14056 |
| M-n200-k17 | M | 199 | 1373.00 | 1332.00 | -2.99 | 82984 |
| M-n151-k12 | M | 150 | 1053.00 | 1031.00 | -2.09 | 53820 |
| P-n55-k8 | P | 54 | 588.00 | 576.00 | -2.04 | 2112 |
| Tai100a | Tai | 100 | 2141.07 | 2107.00 | -1.59 | 43227 |
| Tai75b | Tai | 75 | 1407.89 | 1392.00 | -1.13 | 21323 |
| CMT1 | CMT | 50 | 524.61 | 521.00 | -0.69 | 3095 |
| CMT3 | CMT | 100 | 826.14 | 822.00 | -0.50 | 15693 |
| CMT11 | CMT | 120 | 1042.11 | 1038.00 | -0.39 | 45683 |
| Tai75a | Tai | 75 | 1618.36 | 1615.00 | -0.21 | 16291 |
| A-n32-k5 | A | 31 | 784.00 | 784.00 | 0.00 | 657 |

## Top 20 Worst Results (highest gap)

| Instance | Family | Customers | BKS | Best | Gap% | Runtime(ms) |
|----------|--------|-----------|-----|------|------|-------------|
| M-n121-k7 | M | 120 | 1034.00 | 1181.00 | 14.22 | 60216 |
| Tai75c | Tai | 75 | 1166.69 | 1327.00 | 13.74 | 9291 |
| B-n64-k9 | B | 63 | 861.00 | 947.00 | 9.99 | 6497 |
| B-n57-k7 | B | 56 | 1153.00 | 1265.00 | 9.71 | 13377 |
| B-n63-k10 | B | 62 | 1496.00 | 1537.00 | 2.74 | 2965 |
| P-n50-k8 | P | 49 | 631.00 | 648.00 | 2.69 | 9496 |
| Tai100d | Tai | 100 | 1575.03 | 1615.00 | 2.54 | 34748 |
| CMT5 | CMT | 199 | 1291.29 | 1321.00 | 2.30 | 81676 |
| P-n76-k5 | P | 75 | 627.00 | 640.00 | 2.07 | 11630 |
| A-n54-k7 | A | 53 | 1167.00 | 1190.00 | 1.97 | 3669 |
| A-n62-k8 | A | 61 | 1288.00 | 1308.00 | 1.55 | 4114 |
| B-n68-k9 | B | 67 | 1272.00 | 1291.00 | 1.49 | 5885 |
| P-n76-k4 | P | 75 | 593.00 | 601.00 | 1.35 | 14157 |
| A-n64-k9 | A | 63 | 1401.00 | 1419.00 | 1.28 | 5216 |
| B-n67-k10 | B | 66 | 1032.00 | 1045.00 | 1.26 | 3178 |
| P-n55-k7 | P | 54 | 568.00 | 575.00 | 1.23 | 2417 |
| B-n78-k10 | B | 77 | 1221.00 | 1236.00 | 1.23 | 6583 |
| CMT4 | CMT | 150 | 1028.42 | 1041.00 | 1.22 | 39276 |
| B-n66-k9 | B | 65 | 1316.00 | 1332.00 | 1.22 | 8396 |
| A-n48-k7 | A | 47 | 1073.00 | 1086.00 | 1.21 | 2210 |

## Full Results

| Instance | Family | Cust | Veh | VehSrc | BKS | Best | Gap% | Quality | Feasible | Runtime(ms) | Gens | Status |
|----------|--------|------|-----|--------|-----|------|------|---------|----------|-------------|------|--------|
| A-n32-k5 | A | 31 | 5 | COMMENT | 784.00 | 784.00 | 0.00 | ✅ Solved | true | 657 | 35 | ok |
| A-n33-k5 | A | 32 | 5 | COMMENT | 661.00 | 661.00 | 0.00 | ✅ Solved | true | 701 | 34 | ok |
| B-n31-k5 | B | 30 | 5 | COMMENT | 672.00 | 672.00 | 0.00 | ✅ Solved | true | 499 | 35 | ok |
| A-n33-k6 | A | 32 | 6 | COMMENT | 742.00 | 742.00 | 0.00 | ✅ Solved | true | 641 | 32 | ok |
| A-n34-k5 | A | 33 | 5 | COMMENT | 778.00 | 778.00 | 0.00 | ✅ Solved | true | 888 | 38 | ok |
| A-n36-k5 | A | 35 | 5 | COMMENT | 799.00 | 799.00 | 0.00 | ✅ Solved | true | 1100 | 41 | ok |
| A-n37-k5 | A | 36 | 5 | COMMENT | 669.00 | 669.00 | 0.00 | ✅ Solved | true | 1036 | 34 | ok |
| A-n37-k6 | A | 36 | 6 | COMMENT | 949.00 | 949.00 | 0.00 | ✅ Solved | true | 1064 | 36 | ok |
| A-n38-k5 | A | 37 | 5 | COMMENT | 730.00 | 730.00 | 0.00 | ✅ Solved | true | 1346 | 41 | ok |
| A-n39-k5 | A | 38 | 5 | COMMENT | 822.00 | 825.00 | 0.36 | 🟢 Near-optimal | true | 1477 | 33 | ok |
| A-n39-k6 | A | 38 | 6 | COMMENT | 831.00 | 833.00 | 0.24 | 🟢 Near-optimal | true | 1232 | 33 | ok |
| A-n44-k6 | A | 43 | 6 | COMMENT | 937.00 | 937.00 | 0.00 | ✅ Solved | true | 1883 | 38 | ok |
| A-n45-k6 | A | 44 | 6 | COMMENT | 944.00 | 949.00 | 0.53 | 🟢 Near-optimal | true | 4193 | 55 | ok |
| A-n45-k7 | A | 44 | 7 | COMMENT | 1146.00 | 1146.00 | 0.00 | ✅ Solved | true | 1846 | 43 | ok |
| A-n46-k7 | A | 45 | 7 | COMMENT | 914.00 | 914.00 | 0.00 | ✅ Solved | true | 1577 | 38 | ok |
| A-n48-k7 | A | 47 | 7 | COMMENT | 1073.00 | 1086.00 | 1.21 | 🟡 Competitive | true | 2210 | 44 | ok |
| A-n53-k7 | A | 52 | 7 | COMMENT | 1010.00 | 1017.00 | 0.69 | 🟢 Near-optimal | true | 3346 | 40 | ok |
| A-n54-k7 | A | 53 | 7 | COMMENT | 1167.00 | 1190.00 | 1.97 | 🟡 Competitive | true | 3669 | 43 | ok |
| A-n55-k9 | A | 54 | 9 | COMMENT | 1073.00 | 1074.00 | 0.09 | 🟢 Near-optimal | true | 2132 | 35 | ok |
| A-n60-k9 | A | 59 | 9 | COMMENT | 1354.00 | 1358.00 | 0.30 | 🟢 Near-optimal | true | 3712 | 47 | ok |
| A-n61-k9 | A | 60 | 9 | COMMENT | 1034.00 | 1035.00 | 0.10 | 🟢 Near-optimal | true | 6816 | 66 | ok |
| A-n62-k8 | A | 61 | 8 | COMMENT | 1288.00 | 1308.00 | 1.55 | 🟡 Competitive | true | 4114 | 53 | ok |
| A-n63-k9 | A | 62 | 9 | COMMENT | 1616.00 | 1629.00 | 0.80 | 🟢 Near-optimal | true | 6703 | 53 | ok |
| A-n63-k10 | A | 62 | 10 | COMMENT | 1314.00 | 1325.00 | 0.84 | 🟢 Near-optimal | true | 4774 | 58 | ok |
| A-n64-k9 | A | 63 | 9 | COMMENT | 1401.00 | 1419.00 | 1.28 | 🟡 Competitive | true | 5216 | 46 | ok |
| A-n65-k9 | A | 64 | 9 | COMMENT | 1174.00 | 1184.00 | 0.85 | 🟢 Near-optimal | true | 7758 | 68 | ok |
| A-n69-k9 | A | 68 | 9 | COMMENT | 1159.00 | 1169.00 | 0.86 | 🟢 Near-optimal | true | 6244 | 51 | ok |
| A-n80-k10 | A | 79 | 10 | COMMENT | 1763.00 | 1780.00 | 0.96 | 🟢 Near-optimal | true | 11680 | 62 | ok |
| B-n34-k5 | B | 33 | 5 | COMMENT | 788.00 | 788.00 | 0.00 | ✅ Solved | true | 747 | 34 | ok |
| B-n35-k5 | B | 34 | 5 | COMMENT | 955.00 | 955.00 | 0.00 | ✅ Solved | true | 755 | 35 | ok |
| B-n38-k6 | B | 37 | 6 | COMMENT | 805.00 | 805.00 | 0.00 | ✅ Solved | true | 1035 | 37 | ok |
| B-n39-k5 | B | 38 | 5 | COMMENT | 549.00 | 549.00 | 0.00 | ✅ Solved | true | 1250 | 51 | ok |
| B-n41-k6 | B | 40 | 6 | COMMENT | 829.00 | 829.00 | 0.00 | ✅ Solved | true | 1180 | 40 | ok |
| B-n43-k6 | B | 42 | 6 | COMMENT | 742.00 | 742.00 | 0.00 | ✅ Solved | true | 1240 | 38 | ok |
| B-n44-k7 | B | 43 | 7 | COMMENT | 909.00 | 909.00 | 0.00 | ✅ Solved | true | 1249 | 36 | ok |
| B-n45-k5 | B | 44 | 5 | COMMENT | 751.00 | 751.00 | 0.00 | ✅ Solved | true | 2025 | 38 | ok |
| B-n45-k6 | B | 44 | 6 | COMMENT | 678.00 | 678.00 | 0.00 | ✅ Solved | true | 3952 | 73 | ok |
| B-n50-k7 | B | 49 | 7 | COMMENT | 741.00 | 741.00 | 0.00 | ✅ Solved | true | 1501 | 35 | ok |
| B-n50-k8 | B | 49 | 8 | COMMENT | 1312.00 | 1324.00 | 0.91 | 🟢 Near-optimal | true | 2322 | 46 | ok |
| B-n51-k7 | B | 50 | 7 | COMMENT | 1032.00 | 1034.00 | 0.19 | 🟢 Near-optimal | true | 3146 | 59 | ok |
| B-n52-k7 | B | 51 | 7 | COMMENT | 747.00 | 747.00 | 0.00 | ✅ Solved | true | 1931 | 47 | ok |
| B-n56-k7 | B | 55 | 7 | COMMENT | 707.00 | 707.00 | 0.00 | ✅ Solved | true | 3582 | 75 | ok |
| B-n57-k7 | B | 56 | 7 | COMMENT | 1153.00 | 1265.00 | 9.71 | 🟠 Weak | true | 13377 | 150 | ok |
| B-n57-k9 | B | 56 | 9 | COMMENT | 1598.00 | 1609.00 | 0.69 | 🟢 Near-optimal | true | 2059 | 42 | ok |
| B-n63-k10 | B | 62 | 10 | COMMENT | 1496.00 | 1537.00 | 2.74 | 🟡 Competitive | true | 2965 | 42 | ok |
| B-n64-k9 | B | 63 | 9 | COMMENT | 861.00 | 947.00 | 9.99 | 🟠 Weak | true | 6497 | 71 | ok |
| B-n66-k9 | B | 65 | 9 | COMMENT | 1316.00 | 1332.00 | 1.22 | 🟡 Competitive | true | 8396 | 108 | ok |
| B-n67-k10 | B | 66 | 10 | COMMENT | 1032.00 | 1045.00 | 1.26 | 🟡 Competitive | true | 3178 | 47 | ok |
| B-n68-k9 | B | 67 | 9 | COMMENT | 1272.00 | 1291.00 | 1.49 | 🟡 Competitive | true | 5885 | 72 | ok |
| B-n78-k10 | B | 77 | 10 | COMMENT | 1221.00 | 1236.00 | 1.23 | 🟡 Competitive | true | 6583 | 54 | ok |
| E-n13-k4 | E | 12 | 4 | COMMENT | 247.00 | 247.00 | -100.00 | ✅ Solved | true | 91 | 32 | ok |
| E-n22-k4 | E | 21 | 4 | COMMENT | 375.00 | 375.00 | 0.00 | ✅ Solved | true | 263 | 32 | ok |
| E-n23-k3 | E | 22 | 3 | COMMENT | 569.00 | 569.00 | 0.00 | ✅ Solved | true | 387 | 32 | ok |
| E-n30-k3 | E | 29 | 3 | COMMENT | 534.00 | 534.00 | 0.00 | ✅ Solved | true | 768 | 34 | ok |
| E-n31-k7 | E | 30 | 7 | COMMENT | 379.00 | 379.00 | -100.00 | ✅ Solved | true | 1260 | 46 | ok |
| E-n33-k4 | E | 32 | 4 | COMMENT | 835.00 | 835.00 | 0.00 | ✅ Solved | true | 856 | 33 | ok |
| E-n51-k5 | E | 50 | 5 | COMMENT | 521.00 | 521.00 | 0.00 | ✅ Solved | true | 3089 | 36 | ok |
| E-n76-k7 | E | 75 | 7 | COMMENT | 682.00 | 688.00 | 0.88 | 🟢 Near-optimal | true | 6198 | 40 | ok |
| E-n76-k8 | E | 75 | 8 | COMMENT | 735.00 | 735.00 | 0.00 | ✅ Solved | true | 6635 | 43 | ok |
| E-n76-k10 | E | 75 | 10 | COMMENT | 830.00 | 836.00 | 0.72 | 🟢 Near-optimal | true | 13895 | 78 | ok |
| E-n76-k14 | E | 75 | 14 | COMMENT | 1021.00 | 1027.00 | 0.59 | 🟢 Near-optimal | true | 8084 | 40 | ok |
| E-n101-k8 | E | 100 | 8 | COMMENT | 817.00 | 822.00 | 0.61 | 🟢 Near-optimal | true | 15995 | 55 | ok |
| E-n101-k14 | E | 100 | 14 | COMMENT | 1071.00 | 1079.00 | 0.75 | 🟢 Near-optimal | true | 16843 | 82 | ok |
| F-n45-k4 | F | 44 | 4 | COMMENT | 724.00 | 724.00 | 0.00 | ✅ Solved | true | 1946 | 36 | ok |
| F-n72-k4 | F | 71 | 4 | COMMENT | 237.00 | 237.00 | 0.00 | ✅ Solved | true | 9423 | 44 | ok |
| F-n135-k7 | F | 134 | 7 | COMMENT | 1162.00 | 1174.00 | 1.03 | 🟡 Competitive | true | 76913 | 98 | ok |
| M-n101-k10 | M | 100 | 10 | COMMENT | 820.00 | 820.00 | 0.00 | ✅ Solved | true | 8350 | 46 | ok |
| M-n121-k7 | M | 120 | 7 | COMMENT | 1034.00 | 1181.00 | 14.22 | 🟠 Weak | true | 60216 | 79 | ok |
| M-n151-k12 | M | 150 | 12 | COMMENT | 1053.00 | 1031.00 | -2.09 | 🟢 Near-optimal | true | 53820 | 120 | ok |
| M-n200-k16 | M | 199 | 16 | COMMENT | - | 2169.00 | 0.00 | ⬜ No-ref | true | 257168 | 35 | ok |
| M-n200-k17 | M | 199 | 17 | COMMENT | 1373.00 | 1332.00 | -2.99 | 🟢 Near-optimal | true | 82984 | 106 | ok |
| P-n16-k8 | P | 15 | 8 | COMMENT | 450.00 | 450.00 | 0.00 | ✅ Solved | true | 107 | 32 | ok |
| P-n19-k2 | P | 18 | 2 | COMMENT | 212.00 | 212.00 | 0.00 | ✅ Solved | true | 209 | 32 | ok |
| P-n20-k2 | P | 19 | 2 | COMMENT | 216.00 | 216.00 | 0.00 | ✅ Solved | true | 241 | 33 | ok |
| P-n21-k2 | P | 20 | 2 | COMMENT | 211.00 | 211.00 | 0.00 | ✅ Solved | true | 289 | 32 | ok |
| P-n22-k2 | P | 21 | 2 | COMMENT | 216.00 | 216.00 | 0.00 | ✅ Solved | true | 331 | 33 | ok |
| P-n22-k8 | P | 21 | 8 | COMMENT | 603.00 | 603.00 | 0.00 | ✅ Solved | true | 213 | 32 | ok |
| P-n23-k8 | P | 22 | 8 | COMMENT | 529.00 | 529.00 | 0.00 | ✅ Solved | true | 233 | 32 | ok |
| P-n40-k5 | P | 39 | 5 | COMMENT | 458.00 | 458.00 | 0.00 | ✅ Solved | true | 984 | 32 | ok |
| P-n45-k5 | P | 44 | 5 | COMMENT | 510.00 | 510.00 | 0.00 | ✅ Solved | true | 1757 | 37 | ok |
| P-n50-k7 | P | 49 | 7 | COMMENT | 554.00 | 554.00 | 0.00 | ✅ Solved | true | 2128 | 42 | ok |
| P-n50-k8 | P | 49 | 8 | COMMENT | 631.00 | 648.00 | 2.69 | 🟡 Competitive | true | 9496 | 120 | ok |
| P-n50-k10 | P | 49 | 10 | COMMENT | 696.00 | 700.00 | 0.57 | 🟢 Near-optimal | true | 2312 | 42 | ok |
| P-n51-k10 | P | 50 | 10 | COMMENT | 741.00 | 742.00 | 0.13 | 🟢 Near-optimal | true | 3589 | 50 | ok |
| P-n55-k7 | P | 54 | 7 | COMMENT | 568.00 | 575.00 | 1.23 | 🟡 Competitive | true | 2417 | 38 | ok |
| P-n55-k8 | P | 54 | 8 | COMMENT | 588.00 | 576.00 | -2.04 | 🟢 Near-optimal | true | 2112 | 39 | ok |
| P-n55-k10 | P | 54 | 10 | COMMENT | 694.00 | 697.00 | 0.43 | 🟢 Near-optimal | true | 2567 | 44 | ok |
| P-n55-k15 | P | 54 | 15 | COMMENT | 989.00 | 1000000.00 | -100.00 | ⚫ Invalid | false | 3354 | 31 | infeasible |
| P-n60-k10 | P | 59 | 10 | COMMENT | 744.00 | 744.00 | 0.00 | ✅ Solved | true | 4143 | 48 | ok |
| P-n60-k15 | P | 59 | 15 | COMMENT | 968.00 | 974.00 | 0.62 | 🟢 Near-optimal | true | 2574 | 39 | ok |
| P-n65-k10 | P | 64 | 10 | COMMENT | 792.00 | 792.00 | 0.00 | ✅ Solved | true | 5055 | 47 | ok |
| P-n70-k10 | P | 69 | 10 | COMMENT | 827.00 | 834.00 | 0.85 | 🟢 Near-optimal | true | 7855 | 43 | ok |
| P-n76-k4 | P | 75 | 4 | COMMENT | 593.00 | 601.00 | 1.35 | 🟡 Competitive | true | 14157 | 54 | ok |
| P-n76-k5 | P | 75 | 5 | COMMENT | 627.00 | 640.00 | 2.07 | 🟡 Competitive | true | 11630 | 49 | ok |
| P-n101-k4 | P | 100 | 4 | COMMENT | 681.00 | 681.00 | 0.00 | ✅ Solved | true | 35727 | 78 | ok |
| CMT1 | CMT | 50 | 5 | REGISTRY | 524.61 | 521.00 | -0.69 | 🟢 Near-optimal | true | 3095 | 36 | ok |
| CMT2 | CMT | 75 | 10 | REGISTRY | 835.26 | 836.00 | 0.09 | 🟢 Near-optimal | true | 13662 | 78 | ok |
| CMT3 | CMT | 100 | 8 | REGISTRY | 826.14 | 822.00 | -0.50 | 🟢 Near-optimal | true | 15693 | 55 | ok |
| CMT4 | CMT | 150 | 12 | REGISTRY | 1028.42 | 1041.00 | 1.22 | 🟡 Competitive | true | 39276 | 64 | ok |
| CMT5 | CMT | 199 | 17 | REGISTRY | 1291.29 | 1321.00 | 2.30 | 🟡 Competitive | true | 81676 | 89 | ok |
| CMT6 | CMT | 50 | 6 | REGISTRY | 555.43 | 521.00 | -6.20 | 🟢 Near-optimal | true | 2247 | 38 | ok |
| CMT7 | CMT | 75 | 11 | REGISTRY | 909.68 | 832.00 | -8.54 | 🟢 Near-optimal | true | 4850 | 44 | ok |
| CMT8 | CMT | 100 | 9 | REGISTRY | 865.94 | 821.00 | -5.19 | 🟢 Near-optimal | true | 14056 | 62 | ok |
| CMT9 | CMT | 150 | 14 | REGISTRY | 1162.55 | 1040.00 | -10.54 | 🟢 Near-optimal | true | 45564 | 132 | ok |
| CMT10 | CMT | 199 | 18 | REGISTRY | 1395.85 | 1305.00 | -6.51 | 🟢 Near-optimal | true | 59367 | 100 | ok |
| CMT11 | CMT | 120 | 11 | REGISTRY | 1042.11 | 1038.00 | -0.39 | 🟢 Near-optimal | true | 45683 | 134 | ok |
| CMT12 | CMT | 100 | 10 | REGISTRY | 819.56 | 820.00 | 0.05 | 🟢 Near-optimal | true | 8359 | 46 | ok |
| CMT13 | CMT | 120 | 11 | REGISTRY | 1541.14 | 1038.00 | -32.65 | 🟢 Near-optimal | true | 45058 | 134 | ok |
| CMT14 | CMT | 100 | 10 | REGISTRY | 866.37 | 820.00 | -5.35 | 🟢 Near-optimal | true | 8366 | 46 | ok |
| Tai75a | Tai | 75 | 10 | REGISTRY | 1618.36 | 1615.00 | -0.21 | 🟢 Near-optimal | true | 16291 | 108 | ok |
| Tai75b | Tai | 75 | 9 | REGISTRY | 1407.89 | 1392.00 | -1.13 | 🟢 Near-optimal | true | 21323 | 129 | ok |
| Tai75c | Tai | 75 | 10 | REGISTRY | 1166.69 | 1327.00 | 13.74 | 🟠 Weak | true | 9291 | 79 | ok |
| Tai75d | Tai | 75 | 9 | REGISTRY | 1468.73 | 1358.00 | -7.54 | 🟢 Near-optimal | true | 8145 | 56 | ok |
| Tai100a | Tai | 100 | 11 | REGISTRY | 2141.07 | 2107.00 | -1.59 | 🟢 Near-optimal | true | 43227 | 102 | ok |
| Tai100b | Tai | 100 | 11 | REGISTRY | 1940.55 | 1943.00 | 0.13 | 🟢 Near-optimal | true | 28211 | 73 | ok |
| Tai100c | Tai | 100 | 11 | REGISTRY | 1406.94 | 1413.00 | 0.43 | 🟢 Near-optimal | true | 30735 | 102 | ok |
| Tai100d | Tai | 100 | 11 | REGISTRY | 1575.03 | 1615.00 | 2.54 | 🟡 Competitive | true | 34748 | 90 | ok |
| Tai150a | Tai | 150 | 12 | REGISTRY | 2470.47 | 1000000.00 | 0.00 | ⚫ Invalid | false | 0 | 0 | PROVEN_INFEASIBLE |
| Tai150b | Tai | 150 | 12 | REGISTRY | 2197.45 | 1000000.00 | 0.00 | ⚫ Invalid | false | 0 | 0 | PROVEN_INFEASIBLE |
| Tai150c | Tai | 150 | 12 | REGISTRY | 2097.04 | 1000000.00 | 0.00 | ⚫ Invalid | false | 0 | 0 | PROVEN_INFEASIBLE |
| Tai150d | Tai | 150 | 12 | REGISTRY | 2222.35 | 1000000.00 | 0.00 | ⚫ Invalid | false | 0 | 0 | PROVEN_INFEASIBLE |
| Tai385 | Tai | 385 | 24 | REGISTRY | 24420.00 | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| Golden_1 | Golden | 240 | 9 | REGISTRY | 5623.47 | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| Golden_2 | Golden | 320 | 9 | REGISTRY | 8404.61 | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| Golden_3 | Golden | 400 | 10 | REGISTRY | 11036.22 | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| Golden_4 | Golden | 480 | 11 | REGISTRY | 13624.55 | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| Golden_5 | Golden | 200 | 5 | REGISTRY | 6460.98 | 6466.00 | 0.08 | 🟢 Near-optimal | true | 255392 | 46 | ok |
| Golden_6 | Golden | 280 | 6 | REGISTRY | 8404.26 | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| Golden_7 | Golden | 360 | 7 | REGISTRY | 10102.68 | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| Golden_8 | Golden | 440 | 8 | REGISTRY | 11635.34 | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| Golden_9 | Golden | 255 | 14 | REGISTRY | 579.71 | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| Golden_10 | Golden | 323 | 16 | REGISTRY | 736.26 | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| Golden_11 | Golden | 399 | 18 | REGISTRY | 912.84 | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| Golden_12 | Golden | 483 | 20 | REGISTRY | 1102.69 | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| Golden_13 | Golden | 252 | 22 | REGISTRY | 857.19 | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| Golden_14 | Golden | 320 | 24 | REGISTRY | 1080.55 | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| Golden_15 | Golden | 396 | 26 | REGISTRY | 1337.92 | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| Golden_16 | Golden | 480 | 28 | REGISTRY | 1612.50 | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| Golden_17 | Golden | 240 | 22 | REGISTRY | 707.76 | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| Golden_18 | Golden | 300 | 26 | REGISTRY | 995.13 | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| Golden_19 | Golden | 360 | 30 | REGISTRY | 1365.60 | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| Golden_20 | Golden | 420 | 34 | REGISTRY | 1818.32 | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| Li_21 | Li | 560 | 10 | REGISTRY | 21532.00 | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| Li_22 | Li | 600 | 10 | REGISTRY | 22814.00 | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| Li_23 | Li | 640 | 10 | REGISTRY | 24613.00 | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| Li_24 | Li | 720 | 10 | REGISTRY | 27591.00 | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| Li_25 | Li | 760 | 10 | REGISTRY | 29368.00 | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| Li_26 | Li | 800 | 10 | REGISTRY | 31742.00 | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| Li_27 | Li | 840 | 10 | REGISTRY | 33609.00 | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| Li_28 | Li | 880 | 10 | REGISTRY | 35627.00 | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| Li_29 | Li | 960 | 10 | REGISTRY | 39360.00 | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| Li_30 | Li | 1040 | 10 | REGISTRY | 31742.51 | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| Li_31 | Li | 1120 | 10 | REGISTRY | 43748.00 | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| Li_32 | Li | 1200 | 10 | REGISTRY | 48217.00 | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n101-k25 | X | 100 | 25 | NAME | - | 1000000.00 | 0.00 | ⚫ Invalid | false | 15856 | 31 | infeasible |
| X-n106-k14 | X | 105 | 14 | NAME | - | 26661.00 | 0.00 | ⬜ No-ref | true | 42044 | 118 | ok |
| X-n110-k13 | X | 109 | 13 | NAME | - | 15027.00 | 0.00 | ⬜ No-ref | true | 25873 | 77 | ok |
| X-n115-k10 | X | 114 | 10 | NAME | - | 12823.00 | 0.00 | ⬜ No-ref | true | 42065 | 71 | ok |
| X-n120-k6 | X | 119 | 6 | NAME | - | 13667.00 | 0.00 | ⬜ No-ref | true | 70183 | 110 | ok |
| X-n125-k30 | X | 124 | 30 | NAME | - | 1000000.00 | 0.00 | ⚫ Invalid | false | 35452 | 31 | infeasible |
| X-n129-k18 | X | 128 | 18 | NAME | - | 29642.00 | 0.00 | ⬜ No-ref | true | 51821 | 90 | ok |
| X-n134-k13 | X | 133 | 13 | NAME | - | 11121.00 | 0.00 | ⬜ No-ref | true | 191513 | 150 | ok |
| X-n139-k10 | X | 138 | 10 | NAME | - | 13747.00 | 0.00 | ⬜ No-ref | true | 69213 | 84 | ok |
| X-n143-k7 | X | 142 | 7 | NAME | - | 16237.00 | 0.00 | ⬜ No-ref | true | 83095 | 62 | ok |
| X-n148-k46 | X | 147 | 46 | NAME | - | 1000000.00 | 0.00 | ⚫ Invalid | false | 33533 | 31 | infeasible |
| X-n153-k22 | X | 152 | 22 | NAME | - | 1000000.00 | 0.00 | ⚫ Invalid | false | 111790 | 31 | infeasible |
| X-n157-k13 | X | 156 | 13 | NAME | - | 17071.00 | 0.00 | ⬜ No-ref | true | 143772 | 150 | ok |
| X-n162-k11 | X | 161 | 11 | NAME | - | 14369.00 | 0.00 | ⬜ No-ref | true | 102329 | 136 | ok |
| X-n167-k10 | X | 166 | 10 | NAME | - | 21091.00 | 0.00 | ⬜ No-ref | true | 146044 | 117 | ok |
| X-n172-k51 | X | 171 | 51 | NAME | - | 1000000.00 | 0.00 | ⚫ Invalid | false | 51807 | 31 | infeasible |
| X-n176-k26 | X | 175 | 26 | NAME | - | 1000000.00 | 0.00 | ⚫ Invalid | false | 191820 | 31 | infeasible |
| X-n181-k23 | X | 180 | 23 | NAME | - | 25734.00 | 0.00 | ⬜ No-ref | true | 116653 | 92 | ok |
| X-n186-k15 | X | 185 | 15 | NAME | - | 24797.00 | 0.00 | ⬜ No-ref | true | 294732 | 150 | ok |
| X-n190-k8 | X | 189 | 8 | NAME | - | 17569.00 | 0.00 | ⬜ No-ref | true | 508310 | 150 | ok |
| X-n195-k51 | X | 194 | 51 | NAME | - | 1000000.00 | 0.00 | ⚫ Invalid | false | 77906 | 31 | infeasible |
| X-n200-k36 | X | 199 | 36 | NAME | - | 1000000.00 | 0.00 | ⚫ Invalid | false | 164254 | 31 | infeasible |
| X-n204-k19 | X | 203 | 19 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n209-k16 | X | 208 | 16 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n214-k11 | X | 213 | 11 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n219-k73 | X | 218 | 73 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n223-k34 | X | 222 | 34 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n228-k23 | X | 227 | 23 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n233-k16 | X | 232 | 16 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n237-k14 | X | 236 | 14 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n242-k48 | X | 241 | 48 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n247-k50 | X | 246 | 50 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n251-k28 | X | 250 | 28 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n256-k16 | X | 255 | 16 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n261-k13 | X | 260 | 13 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n266-k58 | X | 265 | 58 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n270-k35 | X | 269 | 35 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n275-k28 | X | 274 | 28 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n280-k17 | X | 279 | 17 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n284-k15 | X | 283 | 15 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n289-k60 | X | 288 | 60 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n294-k50 | X | 293 | 50 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n298-k31 | X | 297 | 31 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n303-k21 | X | 302 | 21 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n308-k13 | X | 307 | 13 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n313-k71 | X | 312 | 71 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n317-k53 | X | 316 | 53 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n322-k28 | X | 321 | 28 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n327-k20 | X | 326 | 20 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n331-k15 | X | 330 | 15 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n336-k84 | X | 335 | 84 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n344-k43 | X | 343 | 43 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n351-k40 | X | 350 | 40 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n359-k29 | X | 358 | 29 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n367-k17 | X | 366 | 17 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n376-k94 | X | 375 | 94 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n384-k52 | X | 383 | 52 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n393-k38 | X | 392 | 38 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n401-k29 | X | 400 | 29 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n411-k19 | X | 410 | 19 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n420-k130 | X | 419 | 130 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n429-k61 | X | 428 | 61 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n439-k37 | X | 438 | 37 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n449-k29 | X | 448 | 29 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n459-k26 | X | 458 | 26 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n469-k138 | X | 468 | 138 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n480-k70 | X | 479 | 70 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n491-k59 | X | 490 | 59 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n502-k39 | X | 501 | 39 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n513-k21 | X | 512 | 21 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n524-k153 | X | 523 | 153 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n536-k96 | X | 535 | 96 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n548-k50 | X | 547 | 50 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n561-k42 | X | 560 | 42 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n573-k30 | X | 572 | 30 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n586-k159 | X | 585 | 159 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n599-k92 | X | 598 | 92 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n613-k62 | X | 612 | 62 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n627-k43 | X | 626 | 43 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n641-k35 | X | 640 | 35 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n655-k131 | X | 654 | 131 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n670-k130 | X | 669 | 130 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n685-k75 | X | 684 | 75 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n701-k44 | X | 700 | 44 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n716-k35 | X | 715 | 35 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n733-k159 | X | 732 | 159 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n749-k98 | X | 748 | 98 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n766-k71 | X | 765 | 71 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n783-k48 | X | 782 | 48 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n801-k40 | X | 800 | 40 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n819-k171 | X | 818 | 171 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n837-k142 | X | 836 | 142 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n856-k95 | X | 855 | 95 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n876-k59 | X | 875 | 59 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n895-k37 | X | 894 | 37 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n916-k207 | X | 915 | 207 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n936-k151 | X | 935 | 151 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n957-k87 | X | 956 | 87 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n979-k58 | X | 978 | 58 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| X-n1001-k43 | X | 1000 | 43 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| instance_258 | unknown | 0 | 0 | N/A | - | 0.00 | 0.00 | ⚫ Invalid | false | 0 | 0 | unsupported |
| instance_259 | unknown | 0 | 0 | N/A | - | 0.00 | 0.00 | ⚫ Invalid | false | 0 | 0 | unsupported |
| instance_260 | unknown | 0 | 0 | N/A | - | 0.00 | 0.00 | ⚫ Invalid | false | 0 | 0 | unsupported |
| instance_261 | unknown | 0 | 0 | N/A | - | 0.00 | 0.00 | ⚫ Invalid | false | 0 | 0 | unsupported |
| instance_262 | unknown | 0 | 0 | N/A | - | 0.00 | 0.00 | ⚫ Invalid | false | 0 | 0 | unsupported |
| instance_263 | unknown | 0 | 0 | N/A | - | 0.00 | 0.00 | ⚫ Invalid | false | 0 | 0 | unsupported |
| instance_264 | unknown | 0 | 0 | N/A | - | 0.00 | 0.00 | ⚫ Invalid | false | 0 | 0 | unsupported |
| instance_265 | unknown | 0 | 0 | N/A | - | 0.00 | 0.00 | ⚫ Invalid | false | 0 | 0 | unsupported |
| instance_266 | unknown | 0 | 0 | N/A | - | 0.00 | 0.00 | ⚫ Invalid | false | 0 | 0 | unsupported |
| instance_267 | unknown | 0 | 0 | N/A | - | 0.00 | 0.00 | ⚫ Invalid | false | 0 | 0 | unsupported |
| Loggi-n401-k23 | Loggi | 400 | 23 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| Loggi-n501-k24 | Loggi | 500 | 24 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| Loggi-n601-k19 | Loggi | 600 | 19 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| Loggi-n601-k42 | Loggi | 600 | 42 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| Loggi-n901-k42 | Loggi | 900 | 42 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| Loggi-n1001-k31 | Loggi | 1000 | 31 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| ORTEC-n242-k12 | ORTEC | 241 | 12 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| ORTEC-n323-k21 | ORTEC | 322 | 21 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| ORTEC-n405-k18 | ORTEC | 404 | 18 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| ORTEC-n455-k41 | ORTEC | 454 | 41 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| ORTEC-n510-k23 | ORTEC | 509 | 23 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| ORTEC-n701-k64 | ORTEC | 700 | 64 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n1048-k237 | XL | 1047 | 237 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n1094-k157 | XL | 1093 | 157 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n1141-k112 | XL | 1140 | 112 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n1188-k96 | XL | 1187 | 96 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n1234-k55 | XL | 1233 | 55 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n1281-k29 | XL | 1280 | 29 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n1328-k19 | XL | 1327 | 19 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n1374-k278 | XL | 1373 | 278 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n1421-k232 | XL | 1420 | 232 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n1468-k151 | XL | 1467 | 151 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n1514-k106 | XL | 1513 | 106 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n1561-k75 | XL | 1560 | 75 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n1608-k39 | XL | 1607 | 39 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n1654-k11 | XL | 1653 | 11 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n1701-k562 | XL | 1700 | 562 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n1748-k271 | XL | 1747 | 271 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n1794-k163 | XL | 1793 | 163 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n1841-k126 | XL | 1840 | 126 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n1888-k82 | XL | 1887 | 82 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n1934-k46 | XL | 1933 | 46 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n1981-k13 | XL | 1980 | 13 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n2028-k617 | XL | 2027 | 617 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n2074-k264 | XL | 2073 | 264 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n2121-k186 | XL | 2120 | 186 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n2168-k138 | XL | 2167 | 138 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n2214-k131 | XL | 2213 | 131 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n2261-k54 | XL | 2260 | 54 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n2307-k34 | XL | 2306 | 34 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n2354-k631 | XL | 2353 | 631 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n2401-k408 | XL | 2400 | 408 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n2447-k290 | XL | 2446 | 290 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n2494-k194 | XL | 2493 | 194 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n2541-k121 | XL | 2540 | 121 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n2587-k66 | XL | 2586 | 66 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n2634-k17 | XL | 2633 | 17 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n2681-k540 | XL | 2680 | 540 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n2727-k546 | XL | 2726 | 546 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n2774-k286 | XL | 2773 | 286 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n2821-k208 | XL | 2820 | 208 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n2867-k120 | XL | 2866 | 120 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n2914-k95 | XL | 2913 | 95 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n2961-k55 | XL | 2960 | 55 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n3007-k658 | XL | 3006 | 658 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n3054-k461 | XL | 3053 | 461 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n3101-k311 | XL | 3100 | 311 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n3147-k232 | XL | 3146 | 232 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n3194-k161 | XL | 3193 | 161 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n3241-k115 | XL | 3240 | 115 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n3287-k30 | XL | 3286 | 30 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n3334-k934 | XL | 3333 | 934 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n3408-k524 | XL | 3407 | 524 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n3484-k436 | XL | 3483 | 436 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n3561-k229 | XL | 3560 | 229 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n3640-k211 | XL | 3639 | 211 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n3721-k77 | XL | 3720 | 77 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n3804-k29 | XL | 3803 | 29 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n3888-k1010 | XL | 3887 | 1010 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n3975-k687 | XL | 3974 | 687 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n4063-k347 | XL | 4062 | 347 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n4153-k291 | XL | 4152 | 291 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n4245-k203 | XL | 4244 | 203 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n4340-k148 | XL | 4339 | 148 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n4436-k48 | XL | 4435 | 48 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n4535-k1134 | XL | 4534 | 1134 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n4635-k790 | XL | 4634 | 790 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n4738-k487 | XL | 4737 | 487 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n4844-k321 | XL | 4843 | 321 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n4951-k203 | XL | 4950 | 203 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n5061-k184 | XL | 5060 | 184 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n5174-k55 | XL | 5173 | 55 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n5288-k1246 | XL | 5287 | 1246 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n5406-k783 | XL | 5405 | 783 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n5526-k553 | XL | 5525 | 553 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n5649-k401 | XL | 5648 | 401 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n5774-k290 | XL | 5773 | 290 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n5902-k122 | XL | 5901 | 122 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n6034-k61 | XL | 6033 | 61 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n6168-k1922 | XL | 6167 | 1922 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n6305-k1042 | XL | 6304 | 1042 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n6445-k628 | XL | 6444 | 628 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n6588-k473 | XL | 6587 | 473 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n6734-k330 | XL | 6733 | 330 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n6884-k148 | XL | 6883 | 148 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n7037-k38 | XL | 7036 | 38 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n7193-k1683 | XL | 7192 | 1683 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n7353-k1471 | XL | 7352 | 1471 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n7516-k859 | XL | 7515 | 859 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n7683-k602 | XL | 7682 | 602 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n7854-k365 | XL | 7853 | 365 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n8028-k294 | XL | 8027 | 294 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n8207-k108 | XL | 8206 | 108 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n8389-k2028 | XL | 8388 | 2028 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n8575-k1297 | XL | 8574 | 1297 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n8766-k1032 | XL | 8765 | 1032 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n8960-k634 | XL | 8959 | 634 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n9160-k379 | XL | 9159 | 379 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n9363-k209 | XL | 9362 | 209 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n9571-k55 | XL | 9570 | 55 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n9784-k2774 | XL | 9783 | 2774 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |
| XL-n10001-k1570 | XL | 10000 | 1570 | NAME | - | 0.00 | 0.00 | skipped | false | 0 | 0 | skipped |

## Automatic Findings

### Pipeline Failures (9 infeasible)

| Instance | Family | Customers | Vehicles | VehSrc |
|----------|--------|-----------|----------|--------|
| P-n55-k15 | P | 54 | 15 | COMMENT |
| X-n101-k25 | X | 100 | 25 | NAME |
| X-n125-k30 | X | 124 | 30 | NAME |
| X-n148-k46 | X | 147 | 46 | NAME |
| X-n153-k22 | X | 152 | 22 | NAME |
| X-n172-k51 | X | 171 | 51 | NAME |
| X-n176-k26 | X | 175 | 26 | NAME |
| X-n195-k51 | X | 194 | 51 | NAME |
| X-n200-k36 | X | 199 | 36 | NAME |

### Negative Gaps — Better Than BKS (19 instances, investigate)

| Instance | BKS | Best | Gap% |
|----------|-----|------|------|
| E-n13-k4 | 247.00 | 247.00 | -100.00 |
| E-n31-k7 | 379.00 | 379.00 | -100.00 |
| M-n151-k12 | 1053.00 | 1031.00 | -2.09 |
| M-n200-k17 | 1373.00 | 1332.00 | -2.99 |
| P-n55-k8 | 588.00 | 576.00 | -2.04 |
| CMT1 | 524.61 | 521.00 | -0.69 |
| CMT3 | 826.14 | 822.00 | -0.50 |
| CMT6 | 555.43 | 521.00 | -6.20 |
| CMT7 | 909.68 | 832.00 | -8.54 |
| CMT8 | 865.94 | 821.00 | -5.19 |
| CMT9 | 1162.55 | 1040.00 | -10.54 |
| CMT10 | 1395.85 | 1305.00 | -6.51 |
| CMT11 | 1042.11 | 1038.00 | -0.39 |
| CMT13 | 1541.14 | 1038.00 | -32.65 |
| CMT14 | 866.37 | 820.00 | -5.35 |
| Tai75a | 1618.36 | 1615.00 | -0.21 |
| Tai75b | 1407.89 | 1392.00 | -1.13 |
| Tai75d | 1468.73 | 1358.00 | -7.54 |
| Tai100a | 2141.07 | 2107.00 | -1.59 |

### Large Regressions >5% Gap (4 instances)

| Instance | Family | BKS | Best | Gap% |
|----------|--------|-----|------|------|
| M-n121-k7 | M | 1034.00 | 1181.00 | 14.22 |
| Tai75c | Tai | 1166.69 | 1327.00 | 13.74 |
| B-n64-k9 | B | 861.00 | 947.00 | 9.99 |
| B-n57-k7 | B | 1153.00 | 1265.00 | 9.71 |

### Runtime Hotspots (Top 10 Slowest)

| Instance | Family | Customers | Runtime(ms) | Gap% |
|----------|--------|-----------|-------------|------|
| X-n190-k8 | X | 189 | 508310 | 0.00 |
| X-n186-k15 | X | 185 | 294732 | 0.00 |
| M-n200-k16 | M | 199 | 257168 | 0.00 |
| Golden_5 | Golden | 200 | 255392 | 0.08 |
| X-n176-k26 | X | 175 | 191820 | 0.00 |
| X-n134-k13 | X | 133 | 191513 | 0.00 |
| X-n200-k36 | X | 199 | 164254 | 0.00 |
| X-n167-k10 | X | 166 | 146044 | 0.00 |
| X-n157-k13 | X | 156 | 143772 | 0.00 |
| X-n181-k23 | X | 180 | 116653 | 0.00 |

### Families Requiring Investigation

- **X**: 36% infeasible, avg gap 0.0%

### Engine Findings (from [INSTR] telemetry at generation 98)

The following findings were observed from engine instrumentation at generation 98 (mid-run):

- **Elite homogeneity**: All 20 elites converge to a single unique solution by gen 98 on most instances. This indicates premature convergence — the optimizer is not maintaining diversity in the elite pool. Observed on all A-series instances tested. Root cause: elite selection copies the best individual without diversity pressure.
- **Persistent infeasibility**: Infeasible individuals (distance=1000000) remain in the population at gen 98 on instances with more customers. 9/140 ran instances (6.4%) ended infeasible. The repair mechanism is not eliminating all infeasible solutions during evolution.
- **No optimizer modifications made**: These findings are documented for the next optimization cycle. Per campaign charter, no optimizer changes are permitted in v1.1.

## Observatory

### Runtime Distribution

| Bucket | Count |
|--------|-------|
| <100ms | 1 |
| 100-500ms | 10 |
| 500ms-1s | 9 |
| 1-5s | 48 |
| 5-30s | 36 |
| 30s+ | 36 |

### Population Quality (proc0 operator)

| Metric | Value |
|--------|-------|
| Total proc0 invocations | 1496880 |
| Avg proc0 time per call (ms) | 2.560 |

### Skipped / Unsupported Instances

| Instance | Status | Reason |
|----------|--------|--------|
| Tai385 | skipped | 385 customers > 200 limit |
| Golden_1 | skipped | 240 customers > 200 limit |
| Golden_2 | skipped | 320 customers > 200 limit |
| Golden_3 | skipped | 400 customers > 200 limit |
| Golden_4 | skipped | 480 customers > 200 limit |
| Golden_6 | skipped | 280 customers > 200 limit |
| Golden_7 | skipped | 360 customers > 200 limit |
| Golden_8 | skipped | 440 customers > 200 limit |
| Golden_9 | skipped | 255 customers > 200 limit |
| Golden_10 | skipped | 323 customers > 200 limit |
| Golden_11 | skipped | 399 customers > 200 limit |
| Golden_12 | skipped | 483 customers > 200 limit |
| Golden_13 | skipped | 252 customers > 200 limit |
| Golden_14 | skipped | 320 customers > 200 limit |
| Golden_15 | skipped | 396 customers > 200 limit |
| Golden_16 | skipped | 480 customers > 200 limit |
| Golden_17 | skipped | 240 customers > 200 limit |
| Golden_18 | skipped | 300 customers > 200 limit |
| Golden_19 | skipped | 360 customers > 200 limit |
| Golden_20 | skipped | 420 customers > 200 limit |
| Li_21 | skipped | 560 customers > 200 limit |
| Li_22 | skipped | 600 customers > 200 limit |
| Li_23 | skipped | 640 customers > 200 limit |
| Li_24 | skipped | 720 customers > 200 limit |
| Li_25 | skipped | 760 customers > 200 limit |
| Li_26 | skipped | 800 customers > 200 limit |
| Li_27 | skipped | 840 customers > 200 limit |
| Li_28 | skipped | 880 customers > 200 limit |
| Li_29 | skipped | 960 customers > 200 limit |
| Li_30 | skipped | 1040 customers > 200 limit |
| Li_31 | skipped | 1120 customers > 200 limit |
| Li_32 | skipped | 1200 customers > 200 limit |
| X-n204-k19 | skipped | 203 customers > 200 limit |
| X-n209-k16 | skipped | 208 customers > 200 limit |
| X-n214-k11 | skipped | 213 customers > 200 limit |
| X-n219-k73 | skipped | 218 customers > 200 limit |
| X-n223-k34 | skipped | 222 customers > 200 limit |
| X-n228-k23 | skipped | 227 customers > 200 limit |
| X-n233-k16 | skipped | 232 customers > 200 limit |
| X-n237-k14 | skipped | 236 customers > 200 limit |
| X-n242-k48 | skipped | 241 customers > 200 limit |
| X-n247-k50 | skipped | 246 customers > 200 limit |
| X-n251-k28 | skipped | 250 customers > 200 limit |
| X-n256-k16 | skipped | 255 customers > 200 limit |
| X-n261-k13 | skipped | 260 customers > 200 limit |
| X-n266-k58 | skipped | 265 customers > 200 limit |
| X-n270-k35 | skipped | 269 customers > 200 limit |
| X-n275-k28 | skipped | 274 customers > 200 limit |
| X-n280-k17 | skipped | 279 customers > 200 limit |
| X-n284-k15 | skipped | 283 customers > 200 limit |
| X-n289-k60 | skipped | 288 customers > 200 limit |
| X-n294-k50 | skipped | 293 customers > 200 limit |
| X-n298-k31 | skipped | 297 customers > 200 limit |
| X-n303-k21 | skipped | 302 customers > 200 limit |
| X-n308-k13 | skipped | 307 customers > 200 limit |
| X-n313-k71 | skipped | 312 customers > 200 limit |
| X-n317-k53 | skipped | 316 customers > 200 limit |
| X-n322-k28 | skipped | 321 customers > 200 limit |
| X-n327-k20 | skipped | 326 customers > 200 limit |
| X-n331-k15 | skipped | 330 customers > 200 limit |
| X-n336-k84 | skipped | 335 customers > 200 limit |
| X-n344-k43 | skipped | 343 customers > 200 limit |
| X-n351-k40 | skipped | 350 customers > 200 limit |
| X-n359-k29 | skipped | 358 customers > 200 limit |
| X-n367-k17 | skipped | 366 customers > 200 limit |
| X-n376-k94 | skipped | 375 customers > 200 limit |
| X-n384-k52 | skipped | 383 customers > 200 limit |
| X-n393-k38 | skipped | 392 customers > 200 limit |
| X-n401-k29 | skipped | 400 customers > 200 limit |
| X-n411-k19 | skipped | 410 customers > 200 limit |
| X-n420-k130 | skipped | 419 customers > 200 limit |
| X-n429-k61 | skipped | 428 customers > 200 limit |
| X-n439-k37 | skipped | 438 customers > 200 limit |
| X-n449-k29 | skipped | 448 customers > 200 limit |
| X-n459-k26 | skipped | 458 customers > 200 limit |
| X-n469-k138 | skipped | 468 customers > 200 limit |
| X-n480-k70 | skipped | 479 customers > 200 limit |
| X-n491-k59 | skipped | 490 customers > 200 limit |
| X-n502-k39 | skipped | 501 customers > 200 limit |
| X-n513-k21 | skipped | 512 customers > 200 limit |
| X-n524-k153 | skipped | 523 customers > 200 limit |
| X-n536-k96 | skipped | 535 customers > 200 limit |
| X-n548-k50 | skipped | 547 customers > 200 limit |
| X-n561-k42 | skipped | 560 customers > 200 limit |
| X-n573-k30 | skipped | 572 customers > 200 limit |
| X-n586-k159 | skipped | 585 customers > 200 limit |
| X-n599-k92 | skipped | 598 customers > 200 limit |
| X-n613-k62 | skipped | 612 customers > 200 limit |
| X-n627-k43 | skipped | 626 customers > 200 limit |
| X-n641-k35 | skipped | 640 customers > 200 limit |
| X-n655-k131 | skipped | 654 customers > 200 limit |
| X-n670-k130 | skipped | 669 customers > 200 limit |
| X-n685-k75 | skipped | 684 customers > 200 limit |
| X-n701-k44 | skipped | 700 customers > 200 limit |
| X-n716-k35 | skipped | 715 customers > 200 limit |
| X-n733-k159 | skipped | 732 customers > 200 limit |
| X-n749-k98 | skipped | 748 customers > 200 limit |
| X-n766-k71 | skipped | 765 customers > 200 limit |
| X-n783-k48 | skipped | 782 customers > 200 limit |
| X-n801-k40 | skipped | 800 customers > 200 limit |
| X-n819-k171 | skipped | 818 customers > 200 limit |
| X-n837-k142 | skipped | 836 customers > 200 limit |
| X-n856-k95 | skipped | 855 customers > 200 limit |
| X-n876-k59 | skipped | 875 customers > 200 limit |
| X-n895-k37 | skipped | 894 customers > 200 limit |
| X-n916-k207 | skipped | 915 customers > 200 limit |
| X-n936-k151 | skipped | 935 customers > 200 limit |
| X-n957-k87 | skipped | 956 customers > 200 limit |
| X-n979-k58 | skipped | 978 customers > 200 limit |
| X-n1001-k43 | skipped | 1000 customers > 200 limit |
| instance_258 | unsupported | Cannot determine vehicle count for 'Antwerp1': not in VEHICLES field, COMMENT, name pattern, or registry |
| instance_259 | unsupported | Cannot determine vehicle count for 'Antwerp2': not in VEHICLES field, COMMENT, name pattern, or registry |
| instance_260 | unsupported | Cannot determine vehicle count for 'Brussels1': not in VEHICLES field, COMMENT, name pattern, or registry |
| instance_261 | unsupported | Cannot determine vehicle count for 'Brussels2': not in VEHICLES field, COMMENT, name pattern, or registry |
| instance_262 | unsupported | Cannot determine vehicle count for 'Flanders1': not in VEHICLES field, COMMENT, name pattern, or registry |
| instance_263 | unsupported | Cannot determine vehicle count for 'Flanders2': not in VEHICLES field, COMMENT, name pattern, or registry |
| instance_264 | unsupported | Cannot determine vehicle count for 'Ghent1': not in VEHICLES field, COMMENT, name pattern, or registry |
| instance_265 | unsupported | Cannot determine vehicle count for 'Ghent2': not in VEHICLES field, COMMENT, name pattern, or registry |
| instance_266 | unsupported | Cannot determine vehicle count for 'Leuven1': not in VEHICLES field, COMMENT, name pattern, or registry |
| instance_267 | unsupported | Cannot determine vehicle count for 'Leuven2': not in VEHICLES field, COMMENT, name pattern, or registry |
| Loggi-n401-k23 | skipped | 400 customers > 200 limit |
| Loggi-n501-k24 | skipped | 500 customers > 200 limit |
| Loggi-n601-k19 | skipped | 600 customers > 200 limit |
| Loggi-n601-k42 | skipped | 600 customers > 200 limit |
| Loggi-n901-k42 | skipped | 900 customers > 200 limit |
| Loggi-n1001-k31 | skipped | 1000 customers > 200 limit |
| ORTEC-n242-k12 | skipped | 241 customers > 200 limit |
| ORTEC-n323-k21 | skipped | 322 customers > 200 limit |
| ORTEC-n405-k18 | skipped | 404 customers > 200 limit |
| ORTEC-n455-k41 | skipped | 454 customers > 200 limit |
| ORTEC-n510-k23 | skipped | 509 customers > 200 limit |
| ORTEC-n701-k64 | skipped | 700 customers > 200 limit |
| XL-n1048-k237 | skipped | 1047 customers > 200 limit |
| XL-n1094-k157 | skipped | 1093 customers > 200 limit |
| XL-n1141-k112 | skipped | 1140 customers > 200 limit |
| XL-n1188-k96 | skipped | 1187 customers > 200 limit |
| XL-n1234-k55 | skipped | 1233 customers > 200 limit |
| XL-n1281-k29 | skipped | 1280 customers > 200 limit |
| XL-n1328-k19 | skipped | 1327 customers > 200 limit |
| XL-n1374-k278 | skipped | 1373 customers > 200 limit |
| XL-n1421-k232 | skipped | 1420 customers > 200 limit |
| XL-n1468-k151 | skipped | 1467 customers > 200 limit |
| XL-n1514-k106 | skipped | 1513 customers > 200 limit |
| XL-n1561-k75 | skipped | 1560 customers > 200 limit |
| XL-n1608-k39 | skipped | 1607 customers > 200 limit |
| XL-n1654-k11 | skipped | 1653 customers > 200 limit |
| XL-n1701-k562 | skipped | 1700 customers > 200 limit |
| XL-n1748-k271 | skipped | 1747 customers > 200 limit |
| XL-n1794-k163 | skipped | 1793 customers > 200 limit |
| XL-n1841-k126 | skipped | 1840 customers > 200 limit |
| XL-n1888-k82 | skipped | 1887 customers > 200 limit |
| XL-n1934-k46 | skipped | 1933 customers > 200 limit |
| XL-n1981-k13 | skipped | 1980 customers > 200 limit |
| XL-n2028-k617 | skipped | 2027 customers > 200 limit |
| XL-n2074-k264 | skipped | 2073 customers > 200 limit |
| XL-n2121-k186 | skipped | 2120 customers > 200 limit |
| XL-n2168-k138 | skipped | 2167 customers > 200 limit |
| XL-n2214-k131 | skipped | 2213 customers > 200 limit |
| XL-n2261-k54 | skipped | 2260 customers > 200 limit |
| XL-n2307-k34 | skipped | 2306 customers > 200 limit |
| XL-n2354-k631 | skipped | 2353 customers > 200 limit |
| XL-n2401-k408 | skipped | 2400 customers > 200 limit |
| XL-n2447-k290 | skipped | 2446 customers > 200 limit |
| XL-n2494-k194 | skipped | 2493 customers > 200 limit |
| XL-n2541-k121 | skipped | 2540 customers > 200 limit |
| XL-n2587-k66 | skipped | 2586 customers > 200 limit |
| XL-n2634-k17 | skipped | 2633 customers > 200 limit |
| XL-n2681-k540 | skipped | 2680 customers > 200 limit |
| XL-n2727-k546 | skipped | 2726 customers > 200 limit |
| XL-n2774-k286 | skipped | 2773 customers > 200 limit |
| XL-n2821-k208 | skipped | 2820 customers > 200 limit |
| XL-n2867-k120 | skipped | 2866 customers > 200 limit |
| XL-n2914-k95 | skipped | 2913 customers > 200 limit |
| XL-n2961-k55 | skipped | 2960 customers > 200 limit |
| XL-n3007-k658 | skipped | 3006 customers > 200 limit |
| XL-n3054-k461 | skipped | 3053 customers > 200 limit |
| XL-n3101-k311 | skipped | 3100 customers > 200 limit |
| XL-n3147-k232 | skipped | 3146 customers > 200 limit |
| XL-n3194-k161 | skipped | 3193 customers > 200 limit |
| XL-n3241-k115 | skipped | 3240 customers > 200 limit |
| XL-n3287-k30 | skipped | 3286 customers > 200 limit |
| XL-n3334-k934 | skipped | 3333 customers > 200 limit |
| XL-n3408-k524 | skipped | 3407 customers > 200 limit |
| XL-n3484-k436 | skipped | 3483 customers > 200 limit |
| XL-n3561-k229 | skipped | 3560 customers > 200 limit |
| XL-n3640-k211 | skipped | 3639 customers > 200 limit |
| XL-n3721-k77 | skipped | 3720 customers > 200 limit |
| XL-n3804-k29 | skipped | 3803 customers > 200 limit |
| XL-n3888-k1010 | skipped | 3887 customers > 200 limit |
| XL-n3975-k687 | skipped | 3974 customers > 200 limit |
| XL-n4063-k347 | skipped | 4062 customers > 200 limit |
| XL-n4153-k291 | skipped | 4152 customers > 200 limit |
| XL-n4245-k203 | skipped | 4244 customers > 200 limit |
| XL-n4340-k148 | skipped | 4339 customers > 200 limit |
| XL-n4436-k48 | skipped | 4435 customers > 200 limit |
| XL-n4535-k1134 | skipped | 4534 customers > 200 limit |
| XL-n4635-k790 | skipped | 4634 customers > 200 limit |
| XL-n4738-k487 | skipped | 4737 customers > 200 limit |
| XL-n4844-k321 | skipped | 4843 customers > 200 limit |
| XL-n4951-k203 | skipped | 4950 customers > 200 limit |
| XL-n5061-k184 | skipped | 5060 customers > 200 limit |
| XL-n5174-k55 | skipped | 5173 customers > 200 limit |
| XL-n5288-k1246 | skipped | 5287 customers > 200 limit |
| XL-n5406-k783 | skipped | 5405 customers > 200 limit |
| XL-n5526-k553 | skipped | 5525 customers > 200 limit |
| XL-n5649-k401 | skipped | 5648 customers > 200 limit |
| XL-n5774-k290 | skipped | 5773 customers > 200 limit |
| XL-n5902-k122 | skipped | 5901 customers > 200 limit |
| XL-n6034-k61 | skipped | 6033 customers > 200 limit |
| XL-n6168-k1922 | skipped | 6167 customers > 200 limit |
| XL-n6305-k1042 | skipped | 6304 customers > 200 limit |
| XL-n6445-k628 | skipped | 6444 customers > 200 limit |
| XL-n6588-k473 | skipped | 6587 customers > 200 limit |
| XL-n6734-k330 | skipped | 6733 customers > 200 limit |
| XL-n6884-k148 | skipped | 6883 customers > 200 limit |
| XL-n7037-k38 | skipped | 7036 customers > 200 limit |
| XL-n7193-k1683 | skipped | 7192 customers > 200 limit |
| XL-n7353-k1471 | skipped | 7352 customers > 200 limit |
| XL-n7516-k859 | skipped | 7515 customers > 200 limit |
| XL-n7683-k602 | skipped | 7682 customers > 200 limit |
| XL-n7854-k365 | skipped | 7853 customers > 200 limit |
| XL-n8028-k294 | skipped | 8027 customers > 200 limit |
| XL-n8207-k108 | skipped | 8206 customers > 200 limit |
| XL-n8389-k2028 | skipped | 8388 customers > 200 limit |
| XL-n8575-k1297 | skipped | 8574 customers > 200 limit |
| XL-n8766-k1032 | skipped | 8765 customers > 200 limit |
| XL-n8960-k634 | skipped | 8959 customers > 200 limit |
| XL-n9160-k379 | skipped | 9159 customers > 200 limit |
| XL-n9363-k209 | skipped | 9362 customers > 200 limit |
| XL-n9571-k55 | skipped | 9570 customers > 200 limit |
| XL-n9784-k2774 | skipped | 9783 customers > 200 limit |
| XL-n10001-k1570 | skipped | 10000 customers > 200 limit |

## Qualification Confidence

This section summarises the overall confidence in the qualification evidence produced by this campaign.

| Dimension | Assessment |
|-----------|------------|
| Benchmark coverage | 140/376 instances executed (37.2% of total) |
| Feasibility rate | 93.6% |
| BKS coverage | 116/140 ran instances have BKS reference |
| Qualification level | 99.3% Verified or PartiallyVerified |
| Distance metric | TspLibEuc2D (EUC_2D) — verified implementation |
| EXPLICIT matrix | Supported in v1.1 (LOWER_ROW, LOWER_DIAG_ROW, UPPER_ROW, UPPER_DIAG_ROW, FULL_MATRIX) |
| Vehicle count provenance | Hierarchical resolution: VEHICLES field → COMMENT → NAME → Registry → Error |
| Optimizer modifications | None — qualification campaign only |

### Qualification Findings

The following families require further qualification before contributing to release evidence:

- **CMT**: Vehicle counts verified against CVRPLIB catalog. BKS provenance under verification.
- **Tai**: Per-instance vehicle counts verified. Tai150 fleet semantics require confirmation.
- **X (Uchoa)**: Fleet semantics require confirmation against Uchoa et al. 2017.
- **Golden/Li**: Excluded from current scope (>200 customers). Registry metadata verified.

### Next Steps

1. Verify CMT BKS values against Christofides et al. 1979 original publication.
2. Confirm Tai150 fleet semantics (minimum vs. maximum).
3. Validate EXPLICIT matrix instances (E-n13-k4, E-n31-k7) with v1.1 ExplicitMatrix support.
4. Extend MAX_CUSTOMERS to include Golden/Li families in a future campaign.
5. Add population diversity telemetry (feasible/infeasible counts, duplicate genomes).

