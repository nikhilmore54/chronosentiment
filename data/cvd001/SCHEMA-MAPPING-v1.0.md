# CVD-001 Schema Mapping
**Document:** SCHEMA-MAPPING-v1.0.md
**Date:** 2025-07-13
**Status:** FROZEN — Sprint 9 Milestone 1
**Preceding document:** DATASET-INVENTORY-v1.0.md

---

## Purpose

This document translates airline domain concepts from CVD-001 (instance1) into UltraCrew and Coralys API equivalents. It is structured in four sections:

1. **Direct mappings** — one-to-one concepts that translate without transformation
2. **Derived mappings** — information synthesized by the adapter from raw data
3. **Unsupported concepts** — no current UltraCrew equivalent
4. **Gap classification** — each unsupported concept classified by gap type

---

## Section 1 — Direct Mappings

These concepts map directly to existing Coralys API fields with minimal transformation.

| Airline concept | UltraCrew concept | Coralys API field | Notes |
|---|---|---|---|
| Crew member (EMP_ID) | Worker | `workers[].id` | 33 workers; IDs 1–33 |
| Qualification / aircraft type | Skill | `workers[].skills` | Defaulted to `["Crew"]` — see derived mappings |
| Active flight leg (LEG_DD_N) | Shift | `shifts[].id` | One shift per leg |
| Leg departure time | Shift start_hour | `shifts[].start_hour` | Hours from 2000-01-01 00:00 |
| Leg duration | Shift duration_hours | `shifts[].duration_hours` | arr_time − dep_time |
| Required crew type | Required skill | `shifts[].required_skill` | Defaulted to `"Crew"` |
| Prior credited hours | Historical workload | `historical_workloads[].hours` | From creditedHours file |
| Vacation / rest day | Off day | (empty schedule slot) | Optimizer assigns rest naturally |

---

## Section 2 — Derived Mappings

These concepts do not exist explicitly in the dataset but can be synthesized by the adapter from available information.

### 2.1 Worker Records

The dataset has no explicit employee master file. Workers are derived from:
- `listOfBases.csv`: total crew count per base (BASE1=7, BASE2=20, BASE3=6 → 33 total)
- `solution_0`: EMP_ID labels (EMP001–EMP033) and base assignments

**Derivation rule:**
```
EMP001–EMP007  → workers 1–7   (BASE1)
EMP008–EMP027  → workers 8–27  (BASE2)
EMP028–EMP033  → workers 28–33 (BASE3)
```

This positional mapping is inferred from the base crew counts. It must be validated against the reference solution.

### 2.2 Shift Start Hours

Leg departure times are given as `date_dep + hour_dep` (e.g., `2000-01-05 14:30`). The Coralys API requires `start_hour` as a float representing hours elapsed since the epoch start.

**Formula:**
```
start_hour = (date_dep - 2000-01-01).days * 24 + hour_dep_decimal
```

**Examples:**
- LEG_01_0: 2000-01-01 12:00 → start_hour = 0*24 + 12.0 = **12.0**
- LEG_05_0: 2000-01-05 08:30 → start_hour = 4*24 + 8.5 = **104.5**
- LEG_31_27: 2000-01-31 23:00 → start_hour = 30*24 + 23.0 = **743.0**

Cross-midnight legs: duration_hours may be fractional (e.g., 1.217h = 73 minutes).

### 2.3 Shift Durations

```
duration_hours = (arr_datetime - dep_datetime).total_seconds() / 3600
```

Cross-midnight legs (date_arr = date_dep + 1) are handled correctly by this formula.

### 2.4 Historical Workloads

The `creditedHours` file provides per-schedule credited hours. These map to `historical_workloads` as the SC2 fatigue input:

```
historical_workloads = [
  {"worker_id": schedule_number, "hours": credited_hours},
  ...
]
```

**Caveat:** Schedule numbering in `creditedHours` may not align with EMP numbering. The adapter must reconcile schedule N → EMP_ID → worker_id.

### 2.5 Contracts

The dataset has no contract types. All workers are assigned a uniform contract:
```
contract = "FullTime"  (default)
```

This is an adapter simplification. Actual contracts (part-time, reserve, etc.) are not available in instance1.

---

## Section 3 — Unsupported Concepts

These concepts appear in CVD-001 but have no current UltraCrew or Coralys equivalent.

### 3.1 Crew Base Assignment

Each crew member belongs to a home base (BASE1, BASE2, or BASE3). Scheduling rules require pairings to start and end at the home base. The Coralys API has no `base` field on workers.

**Impact:** The optimizer may assign crew to flights that are geographically unreachable from their home base.

### 3.2 Deadhead / Positioning Legs (PAL_LEG_DD_N)

A deadhead leg is a flight where the crew member travels as a passenger to reposition for the next active duty. Deadhead legs have cost but contribute zero credited hours.

**Impact:** If PAL_ legs are included as regular shifts, they will be incorrectly scored as credited hours.

### 3.3 Administrative Activities

Three activity types in the reference solution have no flight equivalent:
- `POST_COURRIEL`: administrative activity after a pairing (email/paperwork)
- `TDH_AGR_DD_N`: training or collective agreement activity
- `AST_VACATION_N`: assisted or partial vacation block

**Impact:** These activities block crew availability on specific days. Ignoring them may cause the optimizer to over-assign crew on those days.

### 3.4 Credit Hour Constraints

Each crew member must accumulate between 60h and 90h of credited (flying) hours over the planning period. Per-base credit targets are also defined (BASE1=326.9h, BASE2=1279.4h, BASE3=383.3h).

**Impact:** Without credit hour tracking, the optimizer may produce schedules where some crew are over- or under-utilized relative to the contractual bounds.

### 3.5 Daily Crew Availability

The number of crew available at each base varies by day (crew_avail_const.csv). On some days, BASE1 has 0 available crew (days 22–24).

**Impact:** Without availability constraints, the optimizer may assign crew on days they are contractually unavailable.

### 3.6 Pairings

A pairing is a sequence of connected flight legs starting and ending at the crew's home base, typically spanning 1–4 days. The reference solution uses `POST_PAIRING` markers to delimit pairings. Pairings are the fundamental scheduling unit in airline crew scheduling.

**Impact:** Without pairing structure, the optimizer treats each leg independently. This may produce schedules where crew are assigned to disconnected legs with no feasible travel path between them.

### 3.7 Aircraft Type Constraints

The params.txt file lists 7 aircraft types (NW_727, NW_757, NW_D94, NW_D95, NW_DC9, NW_319, NW_320). In a real airline, crew are qualified on specific aircraft types and can only operate flights using those types.

**Impact:** Without aircraft type constraints, any crew member may be assigned to any flight regardless of qualification. For initial CVD-001 validation, all crew are treated as uniformly qualified ("Crew" skill).

---

## Section 4 — Gap Classification

Each unsupported concept is classified using the CVD-001 Evaluation Protocol gap taxonomy.

| Gap | Concept | Classification | Justification | H9 / UB-003 trigger? |
|---|---|---|---|---|
| G1 | Crew base assignment | **Product capability gap** | Requires new base field in worker model and base-return constraint in adapter. No Coralys API change needed. | No |
| G2 | Deadhead legs (PAL_) | **Product capability gap** | Requires shift type classification in adapter (active vs. deadhead). No Coralys API change needed. | No |
| G3 | Administrative activities | **Adapter capability gap** | Can be approximated as crew unavailability (blocked days) in adapter pre-processing. No Coralys API change needed. | No |
| G4 | Credit hour constraints | **Product capability gap** | Requires post-processing validation layer. Credit bounds can be checked after schedule generation. No Coralys API change needed. | No |
| G5 | Daily crew availability | **Adapter capability gap** | Can be implemented as shift filtering in adapter pre-processing. No Coralys API change needed. | No |
| G6 | Pairings | **Product capability gap** (simplified) | For initial validation: legs treated as independent shifts. Full pairing support is a future product concern. No Coralys API change needed for simplified mode. | No |
| G7 | Aircraft type constraints | **Adapter capability gap** | Requires skill assignment from aircraft type data. Not available in instance1; defaulted to uniform "Crew" skill. | No |
| G8 | Geographic continuity | **Product capability gap** | Optimizer does not verify that a crew member's previous arrival airport matches the next departure airport. No Coralys API change needed. | No |

**Conclusion: No gap identified in CVD-001 instance1 requires Coralys platform changes.**

All gaps are resolvable at the adapter or product layer. H9 and UB-003 are not triggered by this analysis. Coralys can be used as-is for initial CVD-001 validation in simplified mode (legs as independent shifts, uniform crew qualification, no pairing structure).

---

## Architectural Decision: Leg vs. Duty as Shift Unit

The adapter must choose between two strategies:

**Strategy A — Flight Leg as Shift (chosen for Sprint 9)**
```
Flight Leg → Shift
```
- Simple, direct mapping
- Ignores pairing structure
- Produces valid but simplified schedules
- Sufficient for initial CVD-001 validation (credited hours ±2% target)

**Strategy B — Duty as Shift (future)**
```
Connected Legs → Duty → Shift
```
- Preserves pairing structure
- Requires duty generation algorithm
- Necessary for full airline crew scheduling compliance
- Deferred to Phase II product work

**Decision:** Strategy A for Sprint 9. Strategy B is a future product concern, not a platform concern.

---

## Next Step

Proceed to `GAP-ANALYSIS-v1.0.md` for resolution priority and implementation order, then `IMPORT-SPEC-v1.0.md` for the adapter specification.