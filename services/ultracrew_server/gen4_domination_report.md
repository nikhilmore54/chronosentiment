# Gen-4 Domination Report — SD-006 Proxy Geometry Attribution

**Sprint:** 3.7  
**Seed:** 61  
**Instance:** n050w4  
**Observer:** `inrc_official_total`  
**Event:** Archive eviction of best-ever champion at generation 4  

---

## 1. Victim — Tracker UID 1

| Field | Value |
|---|---|
| Genome hash | 12598260676544930757 |
| Tracker UID | 1 |
| OfficialTotal | 44400 |
| HC_Coverage | 11000 |
| HC_Skills | 0 |
| HC_OneShiftPerDay | 0 |
| HC_ForbiddenSuccessions | 21000 |
| SoftTotal | 12400 |

## 2. Dominating Genome

| Field | Value |
|---|---|
| Genome hash | 16917750229659732440 |
| OfficialTotal | 47170 |
| HC_Coverage | 16000 |
| HC_Skills | 0 |
| HC_OneShiftPerDay | 0 |
| HC_ForbiddenSuccessions | 19000 |
| SoftTotal | 12170 |

## 3. Proxy Delta Table (ΔO1–ΔO5)

Δ = Dominator − Victim. Negative = dominator improved on this objective.

| Objective | Victim | Dominator | Δ | Direction |
|---|---|---|---|---|
| O1 (HC_Coverage) | 0 | 0 | +0 | = equal |
| O2 (HC_Skills) | 0 | 0 | +0 | = equal |
| O3 (HC_Successions) | 31170 | 30780 | -390 | ↓ improved |
| O4 (SoftTotal) | 2 | 1 | -0 | = equal |
| O5 (HC_Violations) | 1 | 1 | -0 | = equal |

## 4. External Delta Table

Δ = Dominator − Victim. Positive = dominator is worse externally.

| Metric | Victim | Dominator | Δ |
|---|---|---|---|
| HC_Coverage | 11000 | 16000 | +5000 |
| HC_Skills | 0 | 0 | +0 |
| HC_OneShiftPerDay | 0 | 0 | +0 |
| HC_ForbiddenSucc | 21000 | 19000 | -2000 |
| SoftTotal | 12400 | 12170 | -230 |
| OfficialTotal | 44400 | 47170 | +2770 |

## 5. Dominance Proof

For Pareto domination: dominator ≤ victim on ALL objectives, strict < on at least one.

| Objective | Victim | Dominator | Relation | Holds? |
|---|---|---|---|---|
| O1 (HC_Coverage) | 0 | 0 | = | ✓ |
| O2 (HC_Skills) | 0 | 0 | = | ✓ |
| O3 (HC_Successions) | 31170 | 30780 | < | ✓ |
| O4 (SoftTotal) | 2 | 1 | = | ✓ |
| O5 (HC_Violations) | 1 | 1 | = | ✓ |

**Domination holds: YES**

## 6. Archive Rank Before Eviction (Gen 3)

All members of a Pareto archive are non-dominated by definition (Front 0).
Crowding distance measures isolation within the front.

| Field | Value |
|---|---|
| Pareto Front Rank | 0 |
| Crowding Distance | 1.8488 |

UID 1 had **high crowding distance** at gen 3 — it was well-isolated in proxy space, indicating it was NOT marginal before eviction.

---

## Attribution Summary

UID 1 was evicted because improving **O3 (HC_Successions)** was considered worth sacrificing **+2770 points** of official quality.

```
ΔO1 (HC_Coverage) = +0
ΔO2 (HC_Skills) = +0
ΔO3 (HC_Successions) = -390
ΔO4 (SoftTotal) = -0
ΔO5 (HC_Violations) = -0

which produced Pareto domination,

while causing

ΔOfficialTotal = +2770
```
