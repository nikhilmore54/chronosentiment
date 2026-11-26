# Gen-283 Domination Report — SD-006 Proxy Geometry Attribution

**Sprint:** 3.7  
**Seed:** 61  
**Instance:** n050w4  
**Observer:** `inrc_official_total`  
**Event:** Archive eviction of best-ever champion at generation 283  

---

## 1. Victim — Tracker UID 5

| Field | Value |
|---|---|
| Genome hash | 8180154467705640869 |
| Tracker UID | 5 |
| OfficialTotal | 40130 |
| HC_Coverage | 10000 |
| HC_Skills | 2000 |
| HC_OneShiftPerDay | 0 |
| HC_ForbiddenSuccessions | 17000 |
| SoftTotal | 11130 |

## 2. Dominating Genome

| Field | Value |
|---|---|
| Genome hash | 7085569832671284283 |
| OfficialTotal | 41885 |
| HC_Coverage | 11000 |
| HC_Skills | 1000 |
| HC_OneShiftPerDay | 0 |
| HC_ForbiddenSuccessions | 19000 |
| SoftTotal | 10885 |

## 3. Proxy Delta Table (ΔO1–ΔO5)

Δ = Dominator − Victim. Negative = dominator improved on this objective.

| Objective | Victim | Dominator | Δ | Direction |
|---|---|---|---|---|
| O1 (HC_Coverage) | 0 | 0 | +0 | = equal |
| O2 (HC_Skills) | 0 | 0 | +0 | = equal |
| O3 (HC_Successions) | 28350 | 27510 | -840 | ↓ improved |
| O4 (SoftTotal) | 2 | 2 | +0 | = equal |
| O5 (HC_Violations) | 1 | 1 | -0 | = equal |

## 4. External Delta Table

Δ = Dominator − Victim. Positive = dominator is worse externally.

| Metric | Victim | Dominator | Δ |
|---|---|---|---|
| HC_Coverage | 10000 | 11000 | +1000 |
| HC_Skills | 2000 | 1000 | -1000 |
| HC_OneShiftPerDay | 0 | 0 | +0 |
| HC_ForbiddenSucc | 17000 | 19000 | +2000 |
| SoftTotal | 11130 | 10885 | -245 |
| OfficialTotal | 40130 | 41885 | +1755 |

## 5. Dominance Proof

For Pareto domination: dominator ≤ victim on ALL objectives, strict < on at least one.

| Objective | Victim | Dominator | Relation | Holds? |
|---|---|---|---|---|
| O1 (HC_Coverage) | 0 | 0 | = | ✓ |
| O2 (HC_Skills) | 0 | 0 | = | ✓ |
| O3 (HC_Successions) | 28350 | 27510 | < | ✓ |
| O4 (SoftTotal) | 2 | 2 | = | ✓ |
| O5 (HC_Violations) | 1 | 1 | = | ✓ |

**Domination holds: YES**

## 6. Archive Rank Before Eviction (Gen 282)

All members of a Pareto archive are non-dominated by definition (Front 0).
Crowding distance measures isolation within the front.

| Field | Value |
|---|---|
| Pareto Front Rank | 0 |
| Crowding Distance | 0.0276 |

UID 5 had **low crowding distance** at gen 282 — it was in a dense region of proxy space, suggesting it may have been marginal before eviction.

---

## Attribution Summary

UID 5 was evicted because improving **O3 (HC_Successions)** was considered worth sacrificing **+1755 points** of official quality.

```
ΔO1 (HC_Coverage) = +0
ΔO2 (HC_Skills) = +0
ΔO3 (HC_Successions) = -840
ΔO4 (SoftTotal) = +0
ΔO5 (HC_Violations) = -0

which produced Pareto domination,

while causing

ΔOfficialTotal = +1755
```
