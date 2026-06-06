# Gen-158 Domination Report — SD-006 Proxy Geometry Attribution

**Sprint:** 3.7  
**Seed:** 1  
**Instance:** n050w4  
**Observer:** `inrc_official_total`  
**Event:** Archive eviction of best-ever champion at generation 158  

---

## 1. Victim — Tracker UID 3

| Field | Value |
|---|---|
| Genome hash | 3718436123731752534 |
| Tracker UID | 3 |
| OfficialTotal | 42120 |
| HC_Coverage | 13000 |
| HC_Skills | 3000 |
| HC_OneShiftPerDay | 0 |
| HC_ForbiddenSuccessions | 15000 |
| SoftTotal | 11120 |

## 2. Dominating Genome

| Field | Value |
|---|---|
| Genome hash | 15353041413419938336 |
| OfficialTotal | 44710 |
| HC_Coverage | 13000 |
| HC_Skills | 4000 |
| HC_OneShiftPerDay | 0 |
| HC_ForbiddenSuccessions | 16000 |
| SoftTotal | 11710 |

## 3. Proxy Delta Table (ΔO1–ΔO5)

Δ = Dominator − Victim. Negative = dominator improved on this objective.

| Objective | Victim | Dominator | Δ | Direction |
|---|---|---|---|---|
| O1 (HC_Coverage) | 0 | 0 | +0 | = equal |
| O2 (HC_Skills) | 0 | 0 | +0 | = equal |
| O3 (HC_Successions) | 27930 | 27690 | -240 | ↓ improved |
| O4 (SoftTotal) | 2 | 2 | -0 | = equal |
| O5 (HC_Violations) | 1 | 1 | -0 | = equal |

## 4. External Delta Table

Δ = Dominator − Victim. Positive = dominator is worse externally.

| Metric | Victim | Dominator | Δ |
|---|---|---|---|
| HC_Coverage | 13000 | 13000 | +0 |
| HC_Skills | 3000 | 4000 | +1000 |
| HC_OneShiftPerDay | 0 | 0 | +0 |
| HC_ForbiddenSucc | 15000 | 16000 | +1000 |
| SoftTotal | 11120 | 11710 | +590 |
| OfficialTotal | 42120 | 44710 | +2590 |

## 5. Dominance Proof

For Pareto domination: dominator ≤ victim on ALL objectives, strict < on at least one.

| Objective | Victim | Dominator | Relation | Holds? |
|---|---|---|---|---|
| O1 (HC_Coverage) | 0 | 0 | = | ✓ |
| O2 (HC_Skills) | 0 | 0 | = | ✓ |
| O3 (HC_Successions) | 27930 | 27690 | < | ✓ |
| O4 (SoftTotal) | 2 | 2 | = | ✓ |
| O5 (HC_Violations) | 1 | 1 | = | ✓ |

**Domination holds: YES**

## 6. Archive Rank Before Eviction (Gen 157)

All members of a Pareto archive are non-dominated by definition (Front 0).
Crowding distance measures isolation within the front.

| Field | Value |
|---|---|
| Pareto Front Rank | 0 |
| Crowding Distance | 0.0981 |

UID 3 had **low crowding distance** at gen 157 — it was in a dense region of proxy space, suggesting it may have been marginal before eviction.

---

## Attribution Summary

UID 3 was evicted because improving **O3 (HC_Successions)** was considered worth sacrificing **+2590 points** of official quality.

```
ΔO1 (HC_Coverage) = +0
ΔO2 (HC_Skills) = +0
ΔO3 (HC_Successions) = -240
ΔO4 (SoftTotal) = -0
ΔO5 (HC_Violations) = -0

which produced Pareto domination,

while causing

ΔOfficialTotal = +2590
```
