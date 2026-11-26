# Gen-99 Domination Report — SD-006 Proxy Geometry Attribution

**Sprint:** 3.7  
**Seed:** 61  
**Instance:** n050w4  
**Observer:** `inrc_official_total`  
**Event:** Archive eviction of best-ever champion at generation 99  

---

## 1. Victim — Tracker UID 3

| Field | Value |
|---|---|
| Genome hash | 3412268138035746643 |
| Tracker UID | 3 |
| OfficialTotal | 41685 |
| HC_Coverage | 10000 |
| HC_Skills | 2000 |
| HC_OneShiftPerDay | 0 |
| HC_ForbiddenSuccessions | 19000 |
| SoftTotal | 10685 |

## 2. Dominating Genome

| Field | Value |
|---|---|
| Genome hash | 10811111517915509914 |
| OfficialTotal | 51865 |
| HC_Coverage | 13000 |
| HC_Skills | 1000 |
| HC_OneShiftPerDay | 0 |
| HC_ForbiddenSuccessions | 27000 |
| SoftTotal | 10865 |

## 3. Proxy Delta Table (ΔO1–ΔO5)

Δ = Dominator − Victim. Negative = dominator improved on this objective.

| Objective | Victim | Dominator | Δ | Direction |
|---|---|---|---|---|
| O1 (HC_Coverage) | 0 | 0 | +0 | = equal |
| O2 (HC_Skills) | 0 | 0 | +0 | = equal |
| O3 (HC_Successions) | 27660 | 26940 | -720 | ↓ improved |
| O4 (SoftTotal) | 3 | 3 | -0 | = equal |
| O5 (HC_Violations) | 1 | 1 | -0 | = equal |

## 4. External Delta Table

Δ = Dominator − Victim. Positive = dominator is worse externally.

| Metric | Victim | Dominator | Δ |
|---|---|---|---|
| HC_Coverage | 10000 | 13000 | +3000 |
| HC_Skills | 2000 | 1000 | -1000 |
| HC_OneShiftPerDay | 0 | 0 | +0 |
| HC_ForbiddenSucc | 19000 | 27000 | +8000 |
| SoftTotal | 10685 | 10865 | +180 |
| OfficialTotal | 41685 | 51865 | +10180 |

## 5. Dominance Proof

For Pareto domination: dominator ≤ victim on ALL objectives, strict < on at least one.

| Objective | Victim | Dominator | Relation | Holds? |
|---|---|---|---|---|
| O1 (HC_Coverage) | 0 | 0 | = | ✓ |
| O2 (HC_Skills) | 0 | 0 | = | ✓ |
| O3 (HC_Successions) | 27660 | 26940 | < | ✓ |
| O4 (SoftTotal) | 3 | 3 | = | ✓ |
| O5 (HC_Violations) | 1 | 1 | = | ✓ |

**Domination holds: YES**

## 6. Archive Rank Before Eviction (Gen 98)

All members of a Pareto archive are non-dominated by definition (Front 0).
Crowding distance measures isolation within the front.

| Field | Value |
|---|---|
| Pareto Front Rank | 0 |
| Crowding Distance | 0.0563 |

UID 3 had **low crowding distance** at gen 98 — it was in a dense region of proxy space, suggesting it may have been marginal before eviction.

---

## Attribution Summary

UID 3 was evicted because improving **O3 (HC_Successions)** was considered worth sacrificing **+10180 points** of official quality.

```
ΔO1 (HC_Coverage) = +0
ΔO2 (HC_Skills) = +0
ΔO3 (HC_Successions) = -720
ΔO4 (SoftTotal) = -0
ΔO5 (HC_Violations) = -0

which produced Pareto domination,

while causing

ΔOfficialTotal = +10180
```
