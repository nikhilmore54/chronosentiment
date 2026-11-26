# Benchmark Qualification Matrix — Coralys v1.1

Generated from Qualification Campaign v1.1 evidence.
This document separates **optimizer qualification** from **benchmark qualification**.

A gap is only meaningful when the optimization problem solved by Coralys is
identical to the problem for which the BKS was established. This matrix tracks
that verification per family.

---

## Verification Dimensions

For each family, four dimensions must be verified before gap measurements are
considered directly comparable to published BKS values:

| Dimension | Description |
|-----------|-------------|
| **Vehicle semantics** | Does Coralys enforce exactly K vehicles, or at most K? |
| **Distance semantics** | Does Coralys use TSPLIB integer rounding (floor of Euclidean + 0.5)? |
| **BKS provenance** | Are the registry BKS values from the current CVRPLIB catalog, or from the original publication? |
| **Route count match** | Does the best solution use exactly the benchmark vehicle count? |

---

## Family Qualification Status

| Family | N | AvgCust | Vehicle semantics | Distance semantics | BKS provenance | Route count match | Status |
|--------|---|---------|-------------------|--------------------|----------------|-------------------|--------|
| A | 27 | 51 | ⚠ Pending | ✅ TspLibEuc2D | ✅ CVRPLIB current | ⚠ Pending (new run) | **Partial** |
| B | 23 | 52 | ⚠ Pending | ✅ TspLibEuc2D | ✅ CVRPLIB current | ⚠ Pending (new run) | **Partial** |
| E | 13 | 33 | ⚠ Pending | ✅ TspLibEuc2D | ✅ CVRPLIB current | ⚠ Pending (new run) | **Partial** |
| P | 24 | 55 | ⚠ Pending | ✅ TspLibEuc2D | ⚠ One instance suspect (P-n55-k8, gap=-2.04%) | ⚠ Pending (new run) | **Partial** |
| M | 2 | 175 | ⚠ Pending | ✅ TspLibEuc2D | ⚠ Suspect (M-n151-k12 gap=-2.09%, M-n200-k17 gap=-2.99%) | ⚠ Pending (new run) | **Pending** |
| CMT | 14 | 107 | ⚠ Pending | ✅ TspLibEuc2D | ❌ Registry uses original Christofides 1979 heuristic values — multiple instances show gaps of −5% to −10.5% | ⚠ Pending (new run) | **Pending** |
| Tai | 13 | 121 | ⚠ Pending | ✅ TspLibEuc2D | ⚠ Registry likely uses original Taillard 1993 values — Tai75d gap=-7.61%, Tai100a gap=-2.06% | ⚠ Pending (new run) | **Pending** |
| X | 28 | 101 | ⚠ Pending | ✅ TspLibEuc2D | ✅ CVRPLIB current (Uchoa et al. 2017) | ⚠ Pending (new run) | **Partial** |

---

## Campaign Evidence — Negative Gap Instances

The following instances produced solutions shorter than the registry BKS.
These require route count verification before any conclusion can be drawn.

| Instance | Family | Cust | BenchVeh | Registry BKS | Best Found | Gap% | Route count verified |
|----------|--------|------|----------|-------------|------------|------|----------------------|
| CMT9 | CMT | 150 | 14 | 1162.55 | 1040 | -10.54% | ⚠ Pending |
| CMT7 | CMT | 75 | 11 | 909.68 | 832 | -8.54% | ⚠ Pending |
| CMT10 | CMT | 199 | 18 | 1395.85 | 1305 | -6.51% | ⚠ Pending |
| CMT6 | CMT | 50 | 6 | 555.43 | 521 | -6.20% | ⚠ Pending |
| CMT14 | CMT | 100 | 10 | 866.37 | 820 | -5.35% | ⚠ Pending |
| CMT8 | CMT | 100 | 9 | 865.94 | 821 | -5.19% | ⚠ Pending |
| CMT13 | CMT | 120 | 11 | 1541.14 | 1038 | -32.65% | ⚠ Pending |
| Tai75d | Tai | 75 | 9 | 1468.73 | 1354 | -7.61% | ⚠ Pending |
| Tai100a | Tai | 100 | 11 | 2141.07 | 2097 | -2.06% | ⚠ Pending |
| Tai75b | Tai | 75 | 9 | 1407.89 | 1379 | -2.05% | ⚠ Pending |
| M-n200-k17 | M | 199 | 17 | 1373.00 | 1332 | -2.99% | ⚠ Pending |
| M-n151-k12 | M | 150 | 12 | 1053.00 | 1031 | -2.09% | ⚠ Pending |
| P-n55-k8 | P | 54 | 8 | 588.00 | 576 | -2.04% | ⚠ Pending |
| Tai100b | Tai | 100 | 11 | 1940.55 | 1935 | -0.29% | ⚠ Pending |
| Tai75a | Tai | 75 | 10 | 1618.36 | 1615 | -0.21% | ⚠ Pending |

---

## Required Next Step — Comparison Certificate Run

The next campaign run (with new binary) will emit `routes=N/M` in the completion
log for every instance, where N = routes used by best solution and M = benchmark
vehicle count.

For every negative-gap instance, the comparison certificate requires:

```
benchmark_vehicles = N
routes_used        = N   (must match exactly)
capacity_violations = 0
customers_served   = all (no duplicates, no omissions)
distance_semantics = TSPLIB integer rounding
gap                = X%  (only meaningful if all above pass)
```

If `routes_used > benchmark_vehicles`, the gap is not comparable and must be
excluded from the qualification report.

---

## Qualification Report Language

Until route count verification is complete, the qualification report should use
the following conservative language for CMT and Tai families:

> **Qualification Finding — CMT and Taillard Benchmark Provenance**
>
> Multiple CMT and Taillard instances exhibit substantial negative gaps (up to
> −10.54% for CMT, −7.61% for Taillard) relative to the registry reference
> values. The magnitude and consistency of these differences indicate that the
> registry reference values may not be directly comparable with the optimization
> problem currently solved by Coralys. Before these results are interpreted as
> optimizer improvements, the following must be verified for each affected
> instance: (1) route count matches the benchmark vehicle specification,
> (2) no capacity violations are present, (3) all customers are served exactly
> once, and (4) BKS provenance is confirmed against the current CVRPLIB catalog.
> This verification is scheduled for the next qualification run.

---

## Families with High Confidence (A, B, E, X)

The A, B, E, and X families show no systematic negative gaps and use well-known
CVRPLIB reference values. These families are considered **provisionally qualified**
pending route count confirmation from the next run.

| Family | MedGap | AvgGap | Solved% | NearOpt% | Assessment |
|--------|--------|--------|---------|----------|------------|
| A | 0.00% | ~0.1% | ~89% | ~100% | Provisionally qualified |
| B | 0.00% | ~0.1% | ~87% | ~100% | Provisionally qualified |
| E | 0.00% | ~0.0% | ~100% | ~100% | Provisionally qualified |
| X | ~1.5% | ~2.0% | ~15% | ~60% | Provisionally qualified (harder instances) |

*Note: Final percentages will be updated when campaign completes at 144/144.*