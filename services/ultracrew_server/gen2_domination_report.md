# Gen-2 Domination Report — SD-006 Proxy Geometry Attribution

**Sprint:** 3.7  
**Seed:** 71  
**Instance:** n050w4  
**Observer:** `inrc_official_total`  
**Event:** Archive eviction of best-ever champion at generation 2  

---

## 1. Victim — Tracker UID 1

| Field | Value |
|---|---|
| Genome hash | 13285646020972734058 |
| Tracker UID | 1 |
| OfficialTotal | 56950 |
| HC_Coverage | 14000 |
| HC_Skills | 2000 |
| HC_OneShiftPerDay | 0 |
| HC_ForbiddenSuccessions | 29000 |
| SoftTotal | 11950 |

## 2. Dominating Genome

| Field | Value |
|---|---|
| Genome hash | 16903677187281215029 |
| OfficialTotal | 57390 |
| HC_Coverage | 14000 |
| HC_Skills | 2000 |
| HC_OneShiftPerDay | 0 |
| HC_ForbiddenSuccessions | 30000 |
| SoftTotal | 11390 |

## 3. Proxy Delta Table (ΔO1–ΔO5)

Δ = Dominator − Victim. Negative = dominator improved on this objective.

| Objective | Victim | Dominator | Δ | Direction |
|---|---|---|---|---|
| O1 (HC_Coverage) | 0 | 0 | +0 | = equal |
| O2 (HC_Skills) | 0 | 0 | +0 | = equal |
| O3 (HC_Successions) | 29010 | 28560 | -450 | ↓ improved |
| O4 (SoftTotal) | 2 | 2 | -0 | = equal |
| O5 (HC_Violations) | 2 | 1 | -0 | = equal |

## 4. External Delta Table

Δ = Dominator − Victim. Positive = dominator is worse externally.

| Metric | Victim | Dominator | Δ |
|---|---|---|---|
| HC_Coverage | 14000 | 14000 | +0 |
| HC_Skills | 2000 | 2000 | +0 |
| HC_OneShiftPerDay | 0 | 0 | +0 |
| HC_ForbiddenSucc | 29000 | 30000 | +1000 |
| SoftTotal | 11950 | 11390 | -560 |
| OfficialTotal | 56950 | 57390 | +440 |

## 5. Dominance Proof

For Pareto domination: dominator ≤ victim on ALL objectives, strict < on at least one.

| Objective | Victim | Dominator | Relation | Holds? |
|---|---|---|---|---|
| O1 (HC_Coverage) | 0 | 0 | = | ✓ |
| O2 (HC_Skills) | 0 | 0 | = | ✓ |
| O3 (HC_Successions) | 29010 | 28560 | < | ✓ |
| O4 (SoftTotal) | 2 | 2 | = | ✓ |
| O5 (HC_Violations) | 2 | 1 | = | ✓ |

**Domination holds: YES**

## 6. Archive Rank Before Eviction (Gen 1)

All members of a Pareto archive are non-dominated by definition (Front 0).
Crowding distance measures isolation within the front.

| Field | Value |
|---|---|
| Pareto Front Rank | 0 |
| Crowding Distance | ∞ (boundary solution) |

UID 1 was a **boundary solution** at gen 1 — it occupied an extreme position in at least one proxy objective dimension. This indicates it was NOT marginal before eviction; it was a structurally important archive member.

---

## Attribution Summary

UID 1 was evicted because improving **O3 (HC_Successions)** was considered worth sacrificing **+440 points** of official quality.

```
ΔO1 (HC_Coverage) = +0
ΔO2 (HC_Skills) = +0
ΔO3 (HC_Successions) = -450
ΔO4 (SoftTotal) = -0
ΔO5 (HC_Violations) = -0

which produced Pareto domination,

while causing

ΔOfficialTotal = +440
```
