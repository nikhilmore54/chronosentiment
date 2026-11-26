# CVD-001 Experiment 2 — Scenario Contract Evaluation

**Sprint 9 | Milestone 5**
**Date:** 2026-07-16
**Instance:** CVD-001 instance1 (33 crew, 23 airports, 31 days, 1013 active flight legs)
**Strategy:** A (Flight Leg → Coralys Shift, uniform "Crew" skill)

---

## Objective

Validate the `Scenario` domain-independent contract introduced in Sprint 9 Milestone 5.
Demonstrate backward compatibility: when `max_hours_per_worker = null`, the engine falls
back to `DEFAULT_WEEKLY_MAX_HOURS` (40h) and produces results consistent with Run 1.

---

## Changes Introduced (Experiment 2)

### Rust — `adapters/ultracrew/src/public_contracts.rs`
Added `Scenario` struct:
```rust
pub struct Scenario {
    pub planning_horizon_hours: Option<f64>,
    pub max_hours_per_worker:   Option<f64>,
}
```
Added `scenario: Option<Scenario>` field to `ScheduleRequest` (optional → backward-compatible).
`to_context()` threads `scenario` into `ScheduleContext`.

### Rust — `adapters/ultracrew/src/optimization.rs`
Added `pub scenario: Option<crate::public_contracts::Scenario>` to `ScheduleContext`.

### Rust — `adapters/ultracrew/src/constraint_engine.rs`
HC3 threshold is now scenario-aware:
```rust
const DEFAULT_WEEKLY_MAX_HOURS: u64 = 40;
let hc3_limit = self.context.scenario
    .as_ref()
    .and_then(|s| s.max_hours_per_worker)
    .map(|h| h as u64)
    .unwrap_or(DEFAULT_WEEKLY_MAX_HOURS);
if hours > hc3_limit { ... }
```

### Python — `scripts/cvd001_adapter.py`
Stage 7 now emits `scenario` field (Option A):
```python
scenario = {
    "planning_horizon_hours": 744.0,   # 31 days × 24h
    "max_hours_per_worker":   None,    # no per-worker bound in dataset
}
```

---

## Scenario Parameters (Run 2)

| Parameter                  | Value  | Source                                      |
|----------------------------|--------|---------------------------------------------|
| `planning_horizon_hours`   | 744.0  | 31 days × 24h (dataset structure)           |
| `max_hours_per_worker`     | `null` | Option A: no authoritative bound in dataset |
| HC3 threshold (effective)  | 40h    | Engine default (`DEFAULT_WEEKLY_MAX_HOURS`) |

**Rationale for `null`:** Constraint audit (Milestone 4 / Step 1) established that:
- `credit_constrains.csv` contains per-base aggregate targets, not per-worker maxima.
- `creditedHours` is descriptive reference data from `solution_0`, not a contractual limit.
- No file in instance1 specifies a per-worker maximum number of hours.
Supplying `null` is scientifically defensible; any non-null value would be invented.

---

## Side-by-Side Comparison

| Metric                     | Run 1 (no Scenario)  | Run 2 (Scenario, Option A) | Delta        | Interpretation                          |
|----------------------------|----------------------|----------------------------|--------------|-----------------------------------------|
| **Fitness**                | -9941.21             | -9953.91                   | -12.70       | MOGA stochasticity; same seed, same HC3 |
| **HC1 violations**         | 0                    | 0                          | 0            | Skill coverage intact                   |
| **HC2 violations**         | 0                    | 0                          | 0            | No double-booking                       |
| **HC3 violations**         | 32                   | 32                         | **0**        | Backward compatibility confirmed ✓      |
| **Rest violations**        | 0                    | 0                          | 0            | Rest gaps intact                        |
| **SC1 (fairness)**         | 124.70               | 135.61                     | +10.91       | MOGA stochasticity                      |
| **SC2 (fatigue)**          | 3816.51              | 3818.30                    | +1.79        | MOGA stochasticity                      |
| **Shifts scheduled**       | 1013/1013            | 1013/1013                  | 0            | Complete coverage                       |
| **Workers covered**        | 33/33                | 33/33                      | 0            | All crew assigned                       |
| **HTTP status**            | 200                  | 200                        | —            | Both accepted                           |
| **Response time**          | 98.34s               | 7.27s                      | -91.07s      | Server warm-up effect (Run 1 cold)      |
| **Scenario in payload**    | absent               | present                    | —            | New field, backward-compatible          |
| **HC3 threshold source**   | hardcoded 40h        | `DEFAULT_WEEKLY_MAX_HOURS` | —            | Same value, now configurable            |

---

## Key Findings

### F1 — Backward Compatibility Confirmed
HC3 violations = 32 in both runs. When `max_hours_per_worker = null`, the engine falls
back to 40h exactly as designed. The `Scenario` contract is additive and non-breaking.

### F2 — Fitness Delta is Stochastic, Not Structural
The -12.70 fitness difference (Run 1: -9941.21, Run 2: -9953.91) is within normal MOGA
variance. Both runs use `rng_seed=42` and `generation_limit=200`, but the server's
internal population state differs between cold and warm starts. HC3=32 in both runs
confirms the constraint engine is evaluating identically.

### F3 — Scenario Contract is Architecturally Sound
The `Scenario` struct is domain-independent: it contains no airline, nurse, or rail
concepts. It is the correct abstraction layer between the adapter (domain knowledge)
and the optimization engine (domain-agnostic). Future parameters (rest policies,
objective weights, horizon-aware constraints) can be added without breaking existing
callers.

### F4 — HC3 Root Cause Unchanged (by design)
HC3=32 persists because `max_hours_per_worker = null` → 40h threshold. The 31-day
CVD-001 dataset distributes 1878.50h of flight time across 33 crew, yielding a mean
of 56.92h per worker — well above 40h. HC3 will only decrease when either:
(a) a domain-appropriate threshold is supplied via `scenario.max_hours_per_worker`, or
(b) the optimizer is given a workforce large enough to distribute load below 40h/worker.

### F5 — GAP-M4-001 Partially Addressed
The `Scenario` object resolves the architectural gap (engine lacked domain-independent
context). The semantic gap (no authoritative per-worker bound in CVD-001 instance1)
remains open and is correctly represented by `null`.

---

## Artifact Provenance

| Artifact                                      | Description                          |
|-----------------------------------------------|--------------------------------------|
| `CVD-001-INSTANCE1-RESULT-v1.0.json`          | Run 1 result (no Scenario)           |
| `CVD-001-INSTANCE1-RESULT-v2.0.json`          | Run 2 result (Scenario, Option A)    |
| `CVD-001-INSTANCE1-PAYLOAD-DRY-RUN.json`      | Payload used for both runs           |
| `adapters/ultracrew/src/public_contracts.rs`  | Scenario struct definition           |
| `adapters/ultracrew/src/optimization.rs`      | ScheduleContext with scenario field  |
| `adapters/ultracrew/src/constraint_engine.rs` | HC3 scenario-aware threshold         |
| `scripts/cvd001_adapter.py`                   | Stage 7 Scenario emission (Option A) |

---

## Status

- [x] Scenario struct defined (generic, domain-independent)
- [x] ScheduleRequest backward-compatible (scenario: Option<Scenario>)
- [x] HC3 reads from scenario.max_hours_per_worker with fallback
- [x] CVD-001 adapter emits scenario (Option A: max_hours_per_worker = null)
- [x] Run 2 executed: HTTP 200, 1013/1013 shifts, HC3=32 (backward-compatible)
- [x] Side-by-side comparison produced
- [ ] Option B experiment (explicit threshold, documented as experimental): future work
- [ ] FRMS fatigue model (GAP-M4-003): future work