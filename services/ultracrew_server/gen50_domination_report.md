# Gen-50 Domination Report — SD-006 Proxy Geometry Attribution

**Sprint:** 3.7  
**Seed:** 97  
**Instance:** n050w4  
**Observer:** `inrc_official_total`  
**Event:** Archive eviction of best-ever champion at generation 50  

---

## 1. Victim — Tracker UID 2

| Field | Value |
|---|---|
| Genome hash | 9196064798837302133 |
| Tracker UID | 2 |
| OfficialTotal | 41565 |
| HC_Coverage | 11000 |
| HC_Skills | 1000 |
| HC_OneShiftPerDay | 0 |
| HC_ForbiddenSuccessions | 19000 |
| SoftTotal | 10565 |

## 2. Dominating Genome

| Field | Value |
|---|---|
| Genome hash | 3902977196847180850 |
| OfficialTotal | 45530 |
| HC_Coverage | 12000 |
| HC_Skills | 0 |
| HC_OneShiftPerDay | 0 |
| HC_ForbiddenSuccessions | 23000 |
| SoftTotal | 10530 |

## 3. Proxy Delta Table (ΔO1–ΔO5)

Δ = Dominator − Victim. Negative = dominator improved on this objective.

| Objective | Victim | Dominator | Δ | Direction |
|---|---|---|---|---|
| O1 (HC_Coverage) | 0 | 0 | +0 | = equal |
| O2 (HC_Skills) | 0 | 0 | +0 | = equal |
| O3 (HC_Successions) | 24750 | 23970 | -780 | ↓ improved |
| O4 (SoftTotal) | 3 | 3 | -0 | = equal |
| O5 (HC_Violations) | 2 | 2 | -0 | = equal |

## 4. External Delta Table

Δ = Dominator − Victim. Positive = dominator is worse externally.

| Metric | Victim | Dominator | Δ |
|---|---|---|---|
| HC_Coverage | 11000 | 12000 | +1000 |
| HC_Skills | 1000 | 0 | -1000 |
| HC_OneShiftPerDay | 0 | 0 | +0 |
| HC_ForbiddenSucc | 19000 | 23000 | +4000 |
| SoftTotal | 10565 | 10530 | -35 |
| OfficialTotal | 41565 | 45530 | +3965 |

## 5. Dominance Proof

For Pareto domination: dominator ≤ victim on ALL objectives, strict < on at least one.

| Objective | Victim | Dominator | Relation | Holds? |
|---|---|---|---|---|
| O1 (HC_Coverage) | 0 | 0 | = | ✓ |
| O2 (HC_Skills) | 0 | 0 | = | ✓ |
| O3 (HC_Successions) | 24750 | 23970 | < | ✓ |
| O4 (SoftTotal) | 3 | 3 | = | ✓ |
| O5 (HC_Violations) | 2 | 2 | = | ✓ |

**Domination holds: YES**

## 6. Archive Rank Before Eviction (Gen 49)

All members of a Pareto archive are non-dominated by definition (Front 0).
Crowding distance measures isolation within the front.

| Field | Value |
|---|---|
| Pareto Front Rank | 0 |
| Crowding Distance | ∞ (boundary solution) |

UID 2 was a **boundary solution** at gen 49 — it occupied an extreme position in at least one proxy objective dimension. This indicates it was NOT marginal before eviction; it was a structurally important archive member.

---

## Attribution Summary

UID 2 was evicted because improving **O3 (HC_Successions)** was considered worth sacrificing **+3965 points** of official quality.

```
ΔO1 (HC_Coverage) = +0
ΔO2 (HC_Skills) = +0
ΔO3 (HC_Successions) = -780
ΔO4 (SoftTotal) = -0
ΔO5 (HC_Violations) = -0

which produced Pareto domination,

while causing

ΔOfficialTotal = +3965
```
