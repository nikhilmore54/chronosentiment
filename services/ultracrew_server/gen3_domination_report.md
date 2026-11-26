# Gen-3 Domination Report — SD-006 Proxy Geometry Attribution

**Sprint:** 3.7  
**Seed:** 61  
**Instance:** n050w4  
**Observer:** `inrc_official_total`  
**Event:** Archive eviction of best-ever champion at generation 3  

---

## 1. Victim — Tracker UID 1

| Field | Value |
|---|---|
| Genome hash | 5809301253750319800 |
| Tracker UID | 1 |
| OfficialTotal | 46050 |
| HC_Coverage | 14000 |
| HC_Skills | 1000 |
| HC_OneShiftPerDay | 0 |
| HC_ForbiddenSuccessions | 19000 |
| SoftTotal | 12050 |

## 2. Dominating Genome

| Field | Value |
|---|---|
| Genome hash | 2161470144460023483 |
| OfficialTotal | 50245 |
| HC_Coverage | 17000 |
| HC_Skills | 1000 |
| HC_OneShiftPerDay | 0 |
| HC_ForbiddenSuccessions | 21000 |
| SoftTotal | 11245 |

## 3. Proxy Delta Table (ΔO1–ΔO5)

Δ = Dominator − Victim. Negative = dominator improved on this objective.

| Objective | Victim | Dominator | Δ | Direction |
|---|---|---|---|---|
| O1 (HC_Coverage) | 0 | 0 | +0 | = equal |
| O2 (HC_Skills) | 0 | 0 | +0 | = equal |
| O3 (HC_Successions) | 31440 | 30750 | -690 | ↓ improved |
| O4 (SoftTotal) | 1 | 1 | -0 | = equal |
| O5 (HC_Violations) | 1 | 1 | -0 | = equal |

## 4. External Delta Table

Δ = Dominator − Victim. Positive = dominator is worse externally.

| Metric | Victim | Dominator | Δ |
|---|---|---|---|
| HC_Coverage | 14000 | 17000 | +3000 |
| HC_Skills | 1000 | 1000 | +0 |
| HC_OneShiftPerDay | 0 | 0 | +0 |
| HC_ForbiddenSucc | 19000 | 21000 | +2000 |
| SoftTotal | 12050 | 11245 | -805 |
| OfficialTotal | 46050 | 50245 | +4195 |

## 5. Dominance Proof

For Pareto domination: dominator ≤ victim on ALL objectives, strict < on at least one.

| Objective | Victim | Dominator | Relation | Holds? |
|---|---|---|---|---|
| O1 (HC_Coverage) | 0 | 0 | = | ✓ |
| O2 (HC_Skills) | 0 | 0 | = | ✓ |
| O3 (HC_Successions) | 31440 | 30750 | < | ✓ |
| O4 (SoftTotal) | 1 | 1 | = | ✓ |
| O5 (HC_Violations) | 1 | 1 | = | ✓ |

**Domination holds: YES**

## 6. Archive Rank Before Eviction (Gen 2)

All members of a Pareto archive are non-dominated by definition (Front 0).
Crowding distance measures isolation within the front.

| Field | Value |
|---|---|
| Pareto Front Rank | 0 |
| Crowding Distance | ∞ (boundary solution) |

UID 1 was a **boundary solution** at gen 2 — it occupied an extreme position in at least one proxy objective dimension. This indicates it was NOT marginal before eviction; it was a structurally important archive member.

---

## Attribution Summary

UID 1 was evicted because improving **O3 (HC_Successions)** was considered worth sacrificing **+4195 points** of official quality.

```
ΔO1 (HC_Coverage) = +0
ΔO2 (HC_Skills) = +0
ΔO3 (HC_Successions) = -690
ΔO4 (SoftTotal) = -0
ΔO5 (HC_Violations) = -0

which produced Pareto domination,

while causing

ΔOfficialTotal = +4195
```
