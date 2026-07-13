# CVD-001 Evaluation Protocol
**Document:** CVD-001-EVALUATION-PROTOCOL-v1.0.md
**Date:** 2025-07-13
**Status:** FROZEN — defines success criteria before implementation begins
**Dataset:** Canadian Airline Dataset (anonymized, GERAD/Polytechnique Montréal)
**Classification:** Customer Validation Dataset (CVD), not a benchmark

---

## Purpose

This protocol defines, before any implementation work begins, the success criteria,
evaluation metrics, expected outputs, failure classifications, and evidence to collect
for the CVD-001 integration. It ensures CVD evaluations are as disciplined as UB
benchmark experiments.

CVD-001 answers a different question than UB-001/UB-002:

> **UB benchmarks:** What is the optimizer's behaviour under controlled conditions?
> **CVD-001:** Can UltraCrew successfully schedule realistic airline crew operations?

These are different questions requiring different evidence.

---

## Dataset Description

| Property | Value |
|---|---|
| Source | GERAD / Polytechnique Montréal |
| Domain | Major North American airline (anonymized) |
| Instances | instance1–instance7 + generators |
| Identifiers | Airport names, flight numbers anonymized |
| Constraint generators | credit_constraints, crew_availability, EmployeeLegPreferences, preferredVacations |
| Format | ZIP archives with instance files + generator source (C++) |

---

## Success Criteria

All criteria must be evaluated before CVD-001 is considered validated.

| Criterion | Target | Failure action |
|---|---|---|
| Import success | 100% of instance files parse without error | Classify gap (see below) |
| Constraint translation | All airline constraint types mapped to UltraCrew model | Document unmapped constraints |
| Schedule feasibility | HC = 0 on at least one instance | Investigate; do not modify optimizer |
| Runtime | Recorded for each instance | No target; baseline only |
| PAS | Measured on generated schedules | No target; baseline only |
| Planner edits | Measured and classified | No target; baseline only |
| Unsupported constructs | Classified by gap type | See gap taxonomy below |

---

## Evaluation Metrics

### Primary metrics (must be recorded)

| Metric | Description | Unit |
|---|---|---|
| Import success rate | Fraction of instance files successfully parsed | % |
| HC violations | Hard constraint violations in generated schedule | count |
| SC1 (fairness_penalty) | Fairness penalty from Coralys API | float |
| SC2 (fatigue_penalty) | Fatigue penalty from Coralys API | float |
| Fitness | Overall optimizer fitness score | float |
| Runtime | Wall-clock time for schedule generation | seconds |
| PAS | Planner Acceptance Score (using originalAssignmentCount denominator) | % |
| Manual edits | Number of planner edits after generation | count |

### Secondary metrics (record if available)

| Metric | Description |
|---|---|
| Crew utilization | Fraction of available crew assigned at least one duty |
| Duty coverage | Fraction of required duties covered |
| Constraint satisfaction rate | Fraction of soft constraints satisfied |
| Edit distribution | Breakdown of manual edits by type (shift_swap, coverage_fix, removal, weekend_change) |

---

## Gap Taxonomy

When an airline concept cannot be represented in UltraCrew, classify the gap:

| Gap type | Definition | Resolution path |
|---|---|---|
| UI limitation | Concept exists in Coralys but not exposed in UltraCrew UI | Stream B (product work) |
| Adapter limitation | Concept exists in Coralys API but not in the CVD-001 adapter | Stream B (adapter work) |
| Product limitation | Concept requires new UltraCrew workflow or data model | Stream B (design work) |
| Platform limitation | Concept requires new Coralys constraint or objective | Stream A (only if product-driven) |

Only Platform limitations may trigger future platform research (H9, UB-003, etc.).
UI, adapter, and product limitations are resolved in Stream B without touching Coralys.

---

## Airline Domain Mapping

The following mapping must be validated during CVD-001 integration:

| Airline concept | UltraCrew concept | Coralys API field | Notes |
|---|---|---|---|
| Crew member | Worker | `workers[].id` | |
| Qualification / rating | Skill | `workers[].skills` | May require multi-skill |
| Duty | Shift | `shifts[].id` | |
| Duty start time | Shift start_hour | `shifts[].start_hour` | Hours from week start |
| Duty duration | Shift duration | `shifts[].duration_hours` | |
| Required qualification | Required skill | `shifts[].required_skill` | |
| Historical flight hours | Historical workload | `historical_workloads` | SC2 input |
| Crew base | — | Not yet modelled | Gap: adapter limitation |
| Pairing | — | Not yet modelled | Gap: product limitation |
| Credit constraints | Soft constraint | Soft penalty | Partial mapping |
| Preferred vacations | — | Not yet modelled | Gap: product limitation |

---

## Evidence to Collect

For each CVD-001 instance evaluated, record:

1. Instance identifier (instance1–instance7)
2. Number of crew members
3. Number of duties
4. Number of constraint types
5. Import success / failure + error message if failed
6. Gap classification for each unmapped concept
7. Coralys API response: fitness, HC, SC1, SC2, runtime
8. PAS after planner review
9. Manual edit count and distribution
10. Any domain gaps discovered during evaluation

Store results in `benchmarks/customer_validation/CVD-001-v1.0.json` following the
same structure as UB result JSONs.

---

## Evaluation Procedure

1. **Schema analysis** — Read instance files, document field names and data types.
2. **Adapter implementation** — Write `scripts/cvd001_adapter.py` to parse instance
   files into Coralys API payload format.
3. **Dry run** — Run adapter on instance1 without calling Coralys. Verify payload structure.
4. **First API call** — Submit instance1 to Coralys (localhost:3001), record response.
5. **Gap classification** — For each unmapped concept, classify using gap taxonomy above.
6. **Full evaluation** — Run all available instances, record all metrics.
7. **Freeze** — Write `CVD-001-RESULTS-v1.0.md` with findings and freeze.

---

## Governance Constraints

Per GOV-001: CVD-001 is a product validation activity, not a research activity. It does
not justify platform changes unless a Platform limitation gap is identified and cannot be
resolved by adapter or product work.

Per GOV-002: If CVD-001 reveals a phenomenon that UB-001/UB-002 cannot explain, formulate
a specific hypothesis (H9 or later) with a defined protocol before running any experiments.
Do not run exploratory experiments against CVD-001 data.

Per GOV-003: CVD-001 follows the Customer Validation Dataset lifecycle, not the UB benchmark
lifecycle. It is not subject to the UB freeze/regression protocol. CVD-001 may be updated
as new airline instances are added without triggering a new version number.

---

## Entry Criteria for CVD-001 Evaluation

- [x] CVD taxonomy established (UB / CVD / SB / OB / Pilot Archive)
- [x] This evaluation protocol frozen and committed
- [ ] CVD-001 dataset located on disk and schema documented
- [ ] `scripts/cvd001_adapter.py` implemented
- [ ] First dry run completed (instance1, no API call)
- [ ] First API call completed (instance1, Coralys localhost:3001)
- [ ] Gap classification complete for instance1
- [ ] Full evaluation complete (all instances)
- [ ] `CVD-001-RESULTS-v1.0.md` frozen and committed