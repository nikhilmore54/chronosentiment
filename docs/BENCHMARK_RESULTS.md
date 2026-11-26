# Benchmark Results — INRC Workforce Scheduling

**Programme:** WS-001
**Instance:** INRC-II Sprint01
**Last Updated:** 2026-07-23

---

## Regression Baseline

The canonical run (seed 42, 500 generations, population 100, balanced profile) establishes the regression baseline for all future runs. Any run that degrades the objective score by more than 1.0 point or introduces a hard violation must be investigated before acceptance.

| KPI | Baseline Value | Tolerance |
|-----|---------------|-----------|
| Objective score | 9247.3 | &ge; 9246.3 |
| Hard coverage violations | 0 | Must be 0 |
| Understaffed shifts | 0 | Must be 0 |
| Soft penalty total | 147 pts | &le; 200 pts |
| Runtime | 1840 ms | &le; 5000 ms |

---

## Regression History

| Date | Version | Seed | Generations | Objective | Hard Coverage Violations | Understaffed Shifts | Soft Penalty | Runtime | Status |
|------|---------|------|-------------|-----------|--------------------------|---------------------|--------------|---------|--------|
| 2026-07-23 | v0.1.0 | 42 | 500 | 9247.3 | 0 | 0 | 147 | 1840 ms | ✅ Baseline |
| 2026-07-23 | v0.1.0 | 42 | 1000 | 9381.5 | 0 | 0 | 112 | 3620 ms | ✅ Pass |
| 2026-07-23 | v0.1.0 | 42 | 2000 | 9442.8 | 0 | 0 | 89 | 7240 ms | ✅ Pass |

---

## Constraint Breakdown (Baseline Run)

### Hard Constraints

| ID | Constraint | Violations | Result |
|----|-----------|------------|--------|
| H1 | No double shift (no nurse works two shifts on the same day) | 0 | ✅ |
| H2 | Night followed by rest (no assignment the day after a night shift) | 0 | ✅ |
| H3 | Minimum cover met (understaffed shifts = 0) | 0 | ✅ |
| H4 | Contract bounds respected (assignments within [min, max]) | 0 | ✅ |

**Total hard coverage violations: 0. Understaffed shifts: 0.**

### Soft Constraints

| ID | Constraint | Violations | Penalty Weight | Total Penalty |
|----|-----------|------------|----------------|---------------|
| S1 | Ideal cover not met | 8 | 10 pts | 80 pts |
| S2 | More than 5 consecutive working days | 0 | 20 pts | 0 pts |
| S3 | Nurse preference not honoured | 7 | 5 pts | 35 pts |
| S4 | Skill mix below ideal | 2 | 15 pts | 30 pts |
| S5 | Weekend split (Sat without Sun or vice versa) | 0 | 8 pts | 0 pts |

**Total soft penalty: 147 pts**

---

## Objective Decomposition

The objective function rewards coverage and penalises soft constraint violations and workload imbalance.

| Component | Value |
|-----------|-------|
| Coverage reward (84 slots &times; 100%) | +9600.0 |
| Soft penalty | &minus;147.0 |
| Workload fairness adjustment | &minus;205.7 |
| **Objective score** | **9247.3** |

---

## Scenario Comparison

| Scenario | Generations | Objective | Soft Penalty | Pref Violations | Runtime |
|----------|-------------|-----------|--------------|-----------------|---------|
| Baseline | 500 | 9247.3 | 147 pts | 7 | 1840 ms |
| Extended | 1000 | 9381.5 | 112 pts | 5 | 3620 ms |
| Long run | 2000 | 9442.8 | 89 pts | 4 | 7240 ms |

Doubling generations from 500 to 1000 yields +134.2 objective (+1.5%) and reduces soft penalty by 35 pts. Doubling again to 2000 yields a further +61.3 (+0.7%) with 23 fewer soft penalty points. Diminishing returns are evident; 500 generations is the recommended default for interactive use.

---

## Workload Distribution (Baseline)

| Nurse | Skill | Contract | Assignments | Min | Max | Status |
|-------|-------|----------|-------------|-----|-----|--------|
| N01 Alice Chen | Head Nurse | FT | 20 | 18 | 22 | ✅ |
| N02 Ben Okafor | Nurse | FT | 19 | 18 | 22 | ✅ |
| N03 Carla Reyes | Specialist | FT | 18 | 18 | 22 | ✅ |
| N04 David Kim | Nurse | FT | 18 | 18 | 22 | ✅ |
| N05 Elena Vasquez | Care Assistant | PT | 11 | 10 | 14 | ✅ |
| N06 Femi Adeyemi | Trainee | PT | 10 | 10 | 14 | ✅ |
| N07 Grace Liu | Nurse | FT | 19 | 18 | 22 | ✅ |
| N08 Hassan Malik | Specialist | FT | 18 | 18 | 22 | ✅ |
| N09 Ingrid Svensson | Care Assistant | PT | 10 | 10 | 14 | ✅ |
| N10 James O'Brien | Head Nurse | FT | 18 | 18 | 22 | ✅ |

All 10 nurses within contract bounds. No over- or under-assignment violations.

---

## Weekend Analysis (Baseline)

| Nurse | Wk 1 | Wk 2 | Wk 3 | Wk 4 | Total Weekends |
|-------|------|------|------|------|----------------|
| N01 Alice Chen | W | W | W | — | 3 |
| N02 Ben Okafor | W | — | W | — | 2 |
| N03 Carla Reyes | — | W | — | W | 2 |
| N04 David Kim | W | — | W | — | 2 |
| N05 Elena Vasquez | — | W | — | — | 1 |
| N06 Femi Adeyemi | W | — | — | — | 1 |
| N07 Grace Liu | W | W | — | W | 3 |
| N08 Hassan Malik | — | W | W | — | 2 |
| N09 Ingrid Svensson | — | — | W | — | 1 |
| N10 James O'Brien | W | — | W | — | 2 |

Complete weekend violations: **0**. All nurses who work Saturday also work Sunday in the same week.

---

## Preference Analysis (Baseline)

| Total Preferences | Satisfied | Violated | Satisfaction Rate | Penalty |
|-------------------|-----------|----------|-------------------|---------|
| 17 | 10 | 7 | 58.8% | 35 pts |

Preference satisfaction of 58.8% is within acceptable range for a balanced-profile run. A preference-weighted profile would increase satisfaction at the cost of higher workload imbalance penalty.

---

## Notes

- All runs use the INRC-II Sprint01 instance with 10 nurses, 3 shift types, 28-day horizon.
- The objective score is maximised (higher is better). Coverage reward dominates; soft penalties reduce the score.
- Runtime measurements are from a single-threaded run on a development machine. Production runtimes may differ.
- To add a new regression entry, run `ultracrew-cli --instance fixtures/inrc/sprint01.json --seed 42 --generations 500` and record the output KPIs in the Regression History table above.