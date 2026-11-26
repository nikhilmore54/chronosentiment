# Gen-1014 Domination Report — SD-006 Proxy Geometry Attribution

**Sprint:** 3.7  
**Seed:** 61  
**Instance:** n050w4  
**Observer:** `inrc_official_total`  
**Event:** Archive eviction of best-ever champion at generation 1014  

---

## 1. Victim — Tracker UID 7

| Field | Value |
|---|---|
| Genome hash | 7372273663129860317 |
| Tracker UID | 7 |
| OfficialTotal | 44910 |
| HC_Coverage | 16000 |
| HC_Skills | 0 |
| HC_OneShiftPerDay | 0 |
| HC_ForbiddenSuccessions | 19000 |
| SoftTotal | 9910 |

## 2. Dominating Genome

| Field | Value |
|---|---|
| Genome hash | 1216680875723693833 |
| OfficialTotal | 60755 |
| HC_Coverage | 20000 |
| HC_Skills | 1000 |
| HC_OneShiftPerDay | 0 |
| HC_ForbiddenSuccessions | 30000 |
| SoftTotal | 9755 |

## 3. Proxy Delta Table (ΔO1–ΔO5)

Δ = Dominator − Victim. Negative = dominator improved on this objective.

| Objective | Victim | Dominator | Δ | Direction |
|---|---|---|---|---|
| O1 (HC_Coverage) | 0 | 0 | +0 | = equal |
| O2 (HC_Skills) | 0 | 0 | +0 | = equal |
| O3 (HC_Successions) | 20910 | 20580 | -330 | ↓ improved |
| O4 (SoftTotal) | 4 | 4 | -0 | = equal |
| O5 (HC_Violations) | 1 | 1 | +0 | = equal |

## 4. External Delta Table

Δ = Dominator − Victim. Positive = dominator is worse externally.

| Metric | Victim | Dominator | Δ |
|---|---|---|---|
| HC_Coverage | 16000 | 20000 | +4000 |
| HC_Skills | 0 | 1000 | +1000 |
| HC_OneShiftPerDay | 0 | 0 | +0 |
| HC_ForbiddenSucc | 19000 | 30000 | +11000 |
| SoftTotal | 9910 | 9755 | -155 |
| OfficialTotal | 44910 | 60755 | +15845 |

## 5. Dominance Proof

For Pareto domination: dominator ≤ victim on ALL objectives, strict < on at least one.

| Objective | Victim | Dominator | Relation | Holds? |
|---|---|---|---|---|
| O1 (HC_Coverage) | 0 | 0 | = | ✓ |
| O2 (HC_Skills) | 0 | 0 | = | ✓ |
| O3 (HC_Successions) | 20910 | 20580 | < | ✓ |
| O4 (SoftTotal) | 4 | 4 | = | ✓ |
| O5 (HC_Violations) | 1 | 1 | = | ✓ |

**Domination holds: YES**

## 6. Archive Rank Before Eviction (Gen 1013)

All members of a Pareto archive are non-dominated by definition (Front 0).
Crowding distance measures isolation within the front.

| Field | Value |
|---|---|
| Pareto Front Rank | 0 |
| Crowding Distance | 0.0197 |

UID 7 had **low crowding distance** at gen 1013 — it was in a dense region of proxy space, suggesting it may have been marginal before eviction.

---

## Attribution Summary

UID 7 was evicted because improving **O3 (HC_Successions)** was considered worth sacrificing **+15845 points** of official quality.

```
ΔO1 (HC_Coverage) = +0
ΔO2 (HC_Skills) = +0
ΔO3 (HC_Successions) = -330
ΔO4 (SoftTotal) = -0
ΔO5 (HC_Violations) = +0

which produced Pareto domination,

while causing

ΔOfficialTotal = +15845
```
