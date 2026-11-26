# CB-000 Baseline Freeze

**Status:** FROZEN — immutable reference baseline for all RC campaigns  
**Date:** 2026-08-06  
**Lifecycle version:** RC Lifecycle v1.0

---

## Definition

CB-000 is the immutable competition baseline against which all RC milestones are measured.

CB-000 is defined as the Coralys MOGA solver with:

- `ConstructionMode::Random` — 70% ECMP default path, 30% random waypoint
- Scalar comparator (`ComparatorMode::Scalar`)
- No repair heuristics
- No local search
- No large neighbourhood search
- Standard uniform crossover (`RoadefCrossover`)
- Standard mutation (`RoadefMutator`)

CB-000 is never modified. It is preserved as a selectable mode (`ConstructionMode::Random`) in all future campaign binaries.

---

## Provenance

| Field | Value |
|-------|-------|
| Git commit | `d288dd1d8a636f9e035ada3a3546cf85d7f28cc3` |
| Compiler | rustc 1.91.1 (ed61e7d7e 2025-11-07) (Homebrew) |
| Cargo | 1.91.1 (Homebrew) |
| Seed (RC campaigns) | 42 |
| Population size | 50 |
| Elite count | 5 |
| Generation limit | 500 |
| Mutation rate | 0.3 |
| Crossover rate | 0.7 |
| No-improvement limit | 20 |
| Time budget | adaptive: clamp(0.5ms × demands × links, 30s, 300s) |

---

## Frozen EEB Metrics (CB-000 baseline)

From RP-407 / RP-411 / RP-412 frozen evidence:

| Metric | CB-000 Value | Source |
|--------|-------------|--------|
| IFR (mean across setA) | 10.6% | RP-407 |
| IFR = 0% instances | 6/20 | RP-407 |
| Evaluation dominance | ~95% of runtime | RP-411 |

These values are the reference for all RC milestone comparisons.

---

## Usage in RC Campaigns

Every RC campaign binary must:

1. Run Arm A with `ConstructionMode::Random` (CB-000)
2. Run Arm B with the RC candidate mode
3. Use the same fixed seed (42) for both arms
4. Report IFR, official ROADEF objective, n_eval, and runtime for both arms
5. Apply the acceptance criterion from RC Lifecycle v1.0

---

## What CB-000 is NOT

CB-000 is not the best possible random baseline. It is the baseline that existed at the start of the Competition Engineering phase. Its purpose is to provide a stable, reproducible reference point, not to represent the theoretical ceiling of random construction.

---

## Integrity

This file must not be modified after the first RC campaign runs.

If the baseline definition ever needs to change (e.g. a bug fix that affects all arms equally), a new baseline CB-001 must be defined and all prior RC reports must note the baseline change.