# RC-001A2: Constructor Feasibility — Multi-Slot Disabled Links

**Status:** Fix implemented in source (v1.5), pending campaign validation  
**Date:** 2026-08-06  
**Instances affected:** setA-02, setA-05 (IFR=0.000 with greedy constructor)

---

## Symptom

```
B: IFR=0.000  g0best=inf  valid=false  g0dup=50
```

The greedy constructor produces 50 identical infeasible genomes. Every genome
is rejected by the evaluator before evolution begins.

---

## Root Cause

The constructor routes demands using only the disabled links from **time slot 0**:

```rust
// BEFORE (bug):
let disabled_links: HashSet<u64> = gd.evaluator.scenario.interventions
    .iter()
    .find(|iv| iv.t == 0)          // ← only t=0
    .map(|iv| iv.links.iter().copied().collect())
    .unwrap_or_default();
```

The genome stores one waypoint sequence per demand, shared across all time slots.
The evaluator calls `expand_sr_path` for **every** time slot using those same waypoints.

If a time slot `t ≥ 1` has an intervention that disables a link traversed by the
`t=0` route, `expand_sr_path` fails at that slot → `compute_loads` returns `None`
→ evaluator marks the genome invalid.

On setA-02 and setA-05, the `t=1` (or later) interventions disable links that the
greedy constructor's `t=0` routes traverse. Every genome is therefore infeasible
before evolution starts.

---

## Evidence

| Instance | Slots | IFR (Arm B) | g0dup | Diagnosis |
|----------|-------|-------------|-------|-----------|
| setA-01  | 2     | 1.000       | 50    | No cross-slot conflict |
| setA-02  | ?     | 0.000       | 50    | Cross-slot disabled link conflict |
| setA-03  | 2     | 0.440       | 50    | Partial cross-slot conflict |
| setA-05  | ?     | 0.000       | 50    | Cross-slot disabled link conflict |

The `g0dup=50` on infeasible instances confirms all 50 genomes are identical
(deterministic constructor) and all infeasible (same cross-slot conflict).

---

## Fix Applied (v1.5)

Use the **union of disabled links across all time slots**:

```rust
// AFTER (fix):
let disabled_links: HashSet<u64> = gd.evaluator.scenario.interventions
    .iter()
    .flat_map(|iv| iv.links.iter().copied())  // ← all time slots
    .collect();
let n_disabled_links = disabled_links.len();
```

This is conservative: avoids any link disabled at any time slot. Routes are
valid for all slots. Suboptimal (may avoid links only disabled at one slot)
but guarantees cross-slot feasibility.

The `n_disabled_links` count is now emitted in the `[greedy]` telemetry line
to confirm how many links are being avoided per instance.

---

## Expected Outcome After Fix

- setA-02: IFR should rise from 0.000 to > 0
- setA-05: IFR should rise from 0.000 to > 0
- Instances with no cross-slot conflicts (setA-01, setA-03): IFR unchanged

---

## Remaining Risk

If IFR remains 0 after this fix, the failure is in the waypoint conversion
(`path_to_waypoints_rc001`) or the evaluator's `expand_sr_path` logic, not
in the disabled-link routing. That would require a separate investigation.

---

## Classification

- **RC-001A1** (arc/link ID mismatch): ✅ FIXED in v1.4
- **RC-001A2** (multi-slot disabled links): ✅ FIXED in v1.5 — pending validation
- **RC-001B** (population diversity, g0dup=50): ⚠ OPEN — requires ε-greedy or GRASP
- **RC-002** (evolution operator corruption): ⚠ OPEN — confirmed on setA-04