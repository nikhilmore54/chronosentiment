# Gen-29 Domination Report — SD-006 Proxy Geometry Attribution

**Sprint:** 3.7  
**Seed:** 53  
**Instance:** n050w4  
**Observer:** `inrc_official_total`  
**Event:** Archive eviction of best-ever champion at generation 29  

---

## 1. Victim — Tracker UID 1

| Field | Value |
|---|---|
| Genome hash | 7126752934476581258 |
| Tracker UID | 1 |
| OfficialTotal | 43890 |
| HC_Coverage | 10000 |
| HC_Skills | 3000 |
| HC_OneShiftPerDay | 0 |
| HC_ForbiddenSuccessions | 20000 |
| SoftTotal | 10890 |

## 2. Dominating Genome

| Field | Value |
|---|---|
| Genome hash | 7205750220315119818 |
| OfficialTotal | 46875 |
| HC_Coverage | 9000 |
| HC_Skills | 4000 |
| HC_OneShiftPerDay | 0 |
| HC_ForbiddenSuccessions | 23000 |
| SoftTotal | 10875 |

## 3. Proxy Delta Table (ΔO1–ΔO5)

Δ = Dominator − Victim. Negative = dominator improved on this objective.

| Objective | Victim | Dominator | Δ | Direction |
|---|---|---|---|---|
| O1 (HC_Coverage) | 0 | 0 | +0 | = equal |
| O2 (HC_Skills) | 0 | 0 | +0 | = equal |
| O3 (HC_Successions) | 29160 | 27150 | -2010 | ↓ improved |
| O4 (SoftTotal) | 2 | 1 | -0 | = equal |
| O5 (HC_Violations) | 1 | 1 | +0 | = equal |

## 4. External Delta Table

Δ = Dominator − Victim. Positive = dominator is worse externally.

| Metric | Victim | Dominator | Δ |
|---|---|---|---|
| HC_Coverage | 10000 | 9000 | -1000 |
| HC_Skills | 3000 | 4000 | +1000 |
| HC_OneShiftPerDay | 0 | 0 | +0 |
| HC_ForbiddenSucc | 20000 | 23000 | +3000 |
| SoftTotal | 10890 | 10875 | -15 |
| OfficialTotal | 43890 | 46875 | +2985 |

## 5. Dominance Proof

For Pareto domination: dominator ≤ victim on ALL objectives, strict < on at least one.

| Objective | Victim | Dominator | Relation | Holds? |
|---|---|---|---|---|
| O1 (HC_Coverage) | 0 | 0 | = | ✓ |
| O2 (HC_Skills) | 0 | 0 | = | ✓ |
| O3 (HC_Successions) | 29160 | 27150 | < | ✓ |
| O4 (SoftTotal) | 2 | 1 | = | ✓ |
| O5 (HC_Violations) | 1 | 1 | = | ✓ |

**Domination holds: YES**

## 6. Archive Rank Before Eviction (Gen 28)

All members of a Pareto archive are non-dominated by definition (Front 0).
Crowding distance measures isolation within the front.

| Field | Value |
|---|---|
| Pareto Front Rank | 0 |
| Crowding Distance | 0.4637 |

UID 1 had **low crowding distance** at gen 28 — it was in a dense region of proxy space, suggesting it may have been marginal before eviction.

---

## Attribution Summary

UID 1 was evicted because improving **O3 (HC_Successions)** was considered worth sacrificing **+2985 points** of official quality.

```
ΔO1 (HC_Coverage) = +0
ΔO2 (HC_Skills) = +0
ΔO3 (HC_Successions) = -2010
ΔO4 (SoftTotal) = -0
ΔO5 (HC_Violations) = +0

which produced Pareto domination,

while causing

ΔOfficialTotal = +2985
```
