# RP-412 Construction Funnel Analysis

**Telemetry source:** `/tmp/rp410_telemetry_v3`
**Runs analysed:** 20

---

## Executive Summary

Of 20 runs across 20 instances, **12 (60%) produced at least one valid individual** at construction time. 8 runs produced zero valid individuals — the search never began for those instances.

Mean Initial Feasibility Rate (IFR): **11.2%** (range 0.0%–80.0%).

---

## 1. Construction Feasibility by Instance

| Instance | Valid | Invalid | IFR | Any Feasible | Gens Run |
|----------|------:|--------:|----:|:------------:|---------:|
| setA-01 | 6 | 44 | 12.0% | ✓ | 76 |
| setA-02 | 0 | 50 | 0.0% | ✗ | 21 |
| setA-03 | 3 | 47 | 6.0% | ✓ | 39 |
| setA-04 | 8 | 42 | 16.0% | ✓ | 14 |
| setA-05 | 40 | 10 | 80.0% | ✓ | 10 |
| setA-06 | 5 | 45 | 10.0% | ✓ | 13 |
| setA-07 | 0 | 50 | 0.0% | ✗ | 15 |
| setA-08 | 1 | 49 | 2.0% | ✓ | 9 |
| setA-09 | 7 | 43 | 14.0% | ✓ | 10 |
| setA-10 | 0 | 50 | 0.0% | ✗ | 13 |
| setA-11 | 18 | 32 | 36.0% | ✓ | 10 |
| setA-12 | 1 | 49 | 2.0% | ✓ | 10 |
| setA-13 | 0 | 50 | 0.0% | ✗ | 15 |
| setA-14 | 3 | 47 | 6.0% | ✓ | 9 |
| setA-15 | 7 | 43 | 14.0% | ✓ | 8 |
| setA-16 | 0 | 50 | 0.0% | ✗ | 5 |
| setA-17 | 13 | 37 | 26.0% | ✓ | 2 |
| setA-18 | 0 | 50 | 0.0% | ✗ | 8 |
| setA-19 | 0 | 50 | 0.0% | ✗ | 3 |
| setA-20 | 0 | 50 | 0.0% | ✗ | 2 |

---

## 2. Type I Failure Instances (zero valid at gen 0)

The following instances produced **no valid individuals** during construction. The evolutionary search never began for these instances. This is a Type I (construction) failure, not an evolutionary failure.

- `setA-02`
- `setA-07`
- `setA-10`
- `setA-13`
- `setA-16`
- `setA-18`
- `setA-19`
- `setA-20`

---

## 3. Feasible Instances

12 instances produced at least one valid individual:

- `setA-01`: IFR = 12.0% (6/50 valid)
- `setA-03`: IFR = 6.0% (3/50 valid)
- `setA-04`: IFR = 16.0% (8/50 valid)
- `setA-05`: IFR = 80.0% (40/50 valid)
- `setA-06`: IFR = 10.0% (5/50 valid)
- `setA-08`: IFR = 2.0% (1/50 valid)
- `setA-09`: IFR = 14.0% (7/50 valid)
- `setA-11`: IFR = 36.0% (18/50 valid)
- `setA-12`: IFR = 2.0% (1/50 valid)
- `setA-14`: IFR = 6.0% (3/50 valid)
- `setA-15`: IFR = 14.0% (7/50 valid)
- `setA-17`: IFR = 26.0% (13/50 valid)

---

## 4. Gen-0 Cross-Check

The `generation0_valid_count` field in `GenerationRecord` (gen 0) should match `valid_count` in `ConstructionRecord`. Mismatches indicate a telemetry wiring bug.

All cross-checks passed — no mismatches.

---

## 5. Reserved Fields (RP-412 Phase 2)

The following fields are reserved for deeper evaluator instrumentation and are currently zero for all runs:

- `capacity_violation_count` — requires per-individual violation breakdown
- `budget_violation_count` — requires per-individual violation breakdown
- `repair_attempts` — repair is not yet a separate phase in this harness
- `repair_successes` — repair is not yet a separate phase in this harness

When the evaluator exposes per-constraint violation counts, these fields will distinguish capacity violations from segment-budget violations, enabling targeted constructor improvements.

---

## 6. Implications for Research Programme

**If `valid_count = 0` for an instance:** the bottleneck is entirely in the Construction subsystem. Changing selection (RP-408) or variation operators (RP-409) cannot help. The constructor must be fixed first.

**If `valid_count > 0` but IFR is low:** the search begins but with a sparse feasible seed. Evolutionary pressure may be insufficient to maintain feasibility. This is the boundary between Type I and Type II failure.

**If IFR is high:** construction is not the bottleneck. Proceed to RP-411 (throughput) and RP-410B (candidate pipeline) to identify where search efficiency is lost.
