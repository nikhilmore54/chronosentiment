# Gen-62 Domination Report — SD-006 Proxy Geometry Attribution

**Sprint:** 3.7  
**Seed:** 7  
**Instance:** n050w4  
**Observer:** `inrc_official_total`  
**Event:** Archive eviction of best-ever champion at generation 62  

---

## 1. Victim — Tracker UID 2

| Field | Value |
|---|---|
| Genome hash | 17655264524666821707 |
| Tracker UID | 2 |
| OfficialTotal | 47655 |
| HC_Coverage | 17000 |
| HC_Skills | 0 |
| HC_OneShiftPerDay | 0 |
| HC_ForbiddenSuccessions | 20000 |
| SoftTotal | 10655 |

## 2. Dominating Genome

| Field | Value |
|---|---|
| Genome hash | 13395713175965270099 |
| OfficialTotal | 49705 |
| HC_Coverage | 18000 |
| HC_Skills | 1000 |
| HC_OneShiftPerDay | 0 |
| HC_ForbiddenSuccessions | 21000 |
| SoftTotal | 9705 |

## 3. Proxy Delta Table (ΔO1–ΔO5)

Δ = Dominator − Victim. Negative = dominator improved on this objective.

| Objective | Victim | Dominator | Δ | Direction |
|---|---|---|---|---|
| O1 (HC_Coverage) | 0 | 0 | +0 | = equal |
| O2 (HC_Skills) | 0 | 0 | +0 | = equal |
| O3 (HC_Successions) | 29490 | 28050 | -1440 | ↓ improved |
| O4 (SoftTotal) | 2 | 2 | -0 | = equal |
| O5 (HC_Violations) | 2 | 2 | -0 | = equal |

## 4. External Delta Table

Δ = Dominator − Victim. Positive = dominator is worse externally.

| Metric | Victim | Dominator | Δ |
|---|---|---|---|
| HC_Coverage | 17000 | 18000 | +1000 |
| HC_Skills | 0 | 1000 | +1000 |
| HC_OneShiftPerDay | 0 | 0 | +0 |
| HC_ForbiddenSucc | 20000 | 21000 | +1000 |
| SoftTotal | 10655 | 9705 | -950 |
| OfficialTotal | 47655 | 49705 | +2050 |

## 5. Dominance Proof

For Pareto domination: dominator ≤ victim on ALL objectives, strict < on at least one.

| Objective | Victim | Dominator | Relation | Holds? |
|---|---|---|---|---|
| O1 (HC_Coverage) | 0 | 0 | = | ✓ |
| O2 (HC_Skills) | 0 | 0 | = | ✓ |
| O3 (HC_Successions) | 29490 | 28050 | < | ✓ |
| O4 (SoftTotal) | 2 | 2 | = | ✓ |
| O5 (HC_Violations) | 2 | 2 | = | ✓ |

**Domination holds: YES**

## 6. Archive Rank Before Eviction (Gen 61)

All members of a Pareto archive are non-dominated by definition (Front 0).
Crowding distance measures isolation within the front.

| Field | Value |
|---|---|
| Pareto Front Rank | 0 |
| Crowding Distance | 0.1900 |

UID 2 had **low crowding distance** at gen 61 — it was in a dense region of proxy space, suggesting it may have been marginal before eviction.

---

## Attribution Summary

UID 2 was evicted because improving **O3 (HC_Successions)** was considered worth sacrificing **+2050 points** of official quality.

```
ΔO1 (HC_Coverage) = +0
ΔO2 (HC_Skills) = +0
ΔO3 (HC_Successions) = -1440
ΔO4 (SoftTotal) = -0
ΔO5 (HC_Violations) = -0

which produced Pareto domination,

while causing

ΔOfficialTotal = +2050
```
