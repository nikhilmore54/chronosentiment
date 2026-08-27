# Coralys Optimization Platform: ROADEF Research Log

> **Primary Purpose:** Establish Coralys's general optimization capability against recognized hard combinatorial optimization problems.
> **Question:** *What is Coralys learning about optimization?*
> **Role:** Research proving ground for the Coralys engine. Helps discover and validate general Coralys optimization mechanisms.
> **Track separation:** This log covers ROADEF/Coralys research only. Airlines/UltraCrew findings are maintained separately.

---

## Track Context

ROADEF is currently an **architecture-validation campaign**: a matched comparison of the Legacy MOGA control vs. the Coralys `EvolutionaryPipeline` candidate, using the same domain model, construction process, and seeds. `coralys-core` is frozen during this evaluation. Algorithmic improvements are deferred until conformance evidence is complete.

The evidence chain produced here is **Coralys optimization evidence**, not airline evidence. IP positioning: Coralys = general optimization platform; UltraCrew = airline application product.

---

## P10-C1: Lineage and Bottleneck Investigation (ROADEF)

**Goal:** Understand when, where, and through which lineage path arc capacity overload bottlenecks first appear in the evolutionary pipeline.

**Governance:** Observational only. No optimizer, constructor, crossover, routing, or Coralys changes authorized until C1-F.

---

### C1-A — Bottleneck Census

**Status: COMPLETE** (`bc70bc30b`)

**Finding:** Four arcs dominate capacity violations across the 7-instance ladder:

| Arc | Instance | Frequency |
|-----|----------|-----------|
| 658 | setA-13  | 96.7% of [diag] events |
| 606 | setA-16  | 74.0% of [diag] events |
| 303 | setA-16  | 18.4% of [diag] events |
| 968 | setA-19  | 91.1% of [diag] events |

These four arcs are the C1-B/C1-C/C1-D/C1-E/C1-F target arcs.

---

### C1-B — First Appearance

**Status: COMPLETE** (`0f1896fa4` + `10ef251e3` + `2652c0bb5` + `b61392a73`)

**Method:** Added `[c1b]` instrumentation to all three `Ok(false)` arms in `pipeline_impl.rs`. Created `phase10c1_lineage` binary using `io::stdout()` as log sink (no SIGPIPE risk). Ran setA-13/16/19, seed=42, 5 gens.

**Final first-appearance table:**

| Instance | Arc | First gen | First origin | first_sat | Total events | Gen distribution |
|----------|-----|-----------|--------------|-----------|--------------|-----------------|
| setA-13  | 658 | **0** | crossover_ca | 1.209401 | 195 | 80/71/33/9/2 |
| setA-16  | 606 | **0** | crossover_ca | 1.135245 | 489 | 139/127/131/115/122 |
| setA-16  | 303 | **0** | crossover_ca | 1.000000 | 145 | 139/127/131/115/122 |
| setA-19  | 968 | **0** | **mutation** | 1.042154 | 139 | 71/41/23/4 |

**Key observations:**

- All four target arcs first appear at **gen=0**.
- Arcs 658/606/303 share `crossover_ca` as first origin.
- Arc 968 is materially different: first origin is **`mutation`**.
- Arc 303 first_sat=1.000000 — exactly at the capacity boundary.
- setA-13 additional context: wall_ms=1,226,198 (~20.4 min), init rejection sampling 494 retries/1 success/0% repair rate, valid=1/50 at gen=0, Gen 0→1 objective: 987,099.76 → 81.46.

**Defensible C1-B statement:**
> All four bottleneck arcs (658, 606, 303, 968) are already observable at the gen-0 transition. Arcs 658/606/303 show crossover_ca as first origin; arc=968 shows mutation as first origin. None of this establishes whether overload was inherited from initial constructed genomes or created by operators.

**Causal status: unresolved.** C1-B establishes *when/where* overload first appears in operator-generated offspring. C1-C must determine *whether* the overload was inherited from the initial constructed genomes or created by the gen-0 operator pass.

---

### C1-C — Parent Comparison

**Status: IN PROGRESS** (`64f8c03c7` — instrumentation + binary; sweep running)

**Question:** Were arcs 658/606/303/968 already overloaded in the initial constructed genomes before any evolutionary operator executed, or did the gen-0 evolutionary transition create or propagate the overload?

**Method:** Instrument the initial population **before any evolutionary operator executes**. For each initial genome member, emit exactly one record per `(member, arc)`:

```
[c1c] stage=initial member=N arc=ARC max_flow=F cap=C max_sat=S overloaded=true/false
```

`max_sat` = maximum saturation across all time slots for that arc.
`overloaded` = `max_sat > 1.0` (strict: flow strictly exceeds capacity).

Then cross-reference with gen-0 `[c1b]` lines for parent→child correlation.

**Causal taxonomy:**

| Initial parents | Gen-0 child | Classification |
|----------------|-------------|----------------|
| overloaded + overloaded | overloaded | **inherited** |
| feasible + feasible | overloaded | **crossover-created** |
| overloaded + feasible (either order) | overloaded | **inherited/propagated** |
| any + any | feasible | **no causal event** |

The child being feasible means this evolutionary transition did not create the target overload, regardless of parent states.

**Governance:** Pure instrumentation. No behavioral changes.

---

#### C1-C Partial Finding — setA-13 (seed=42, 1 gen)

**Evidence source:** `evidence/phase10_p10c1c_initial_scan_raw.txt` + `evidence/phase10_p10c1c_initial_scan_stderr.txt`

**Key observations:**

| Metric | Value | Source |
|--------|-------|--------|
| Population size | 50 | config |
| Valid individuals at gen-0 | 1/50 (2%) | stderr |
| Initial major violations | **46** | `[rc002] gen=0 initial: maj=46` |
| Crossover-created violations | **0** | `[rc002] crossover: maj=0` |
| Mutation-created violations | **0** | `[rc002] mutation: maj=0` |
| Constructor rejection sampling | 494 retries, 1 success | stderr |
| Wall time (1 gen, 50 individuals) | ~815 seconds (~13.6 min) | stderr |
| Best solution found at gen | 0 | stderr |

**Arc 658 initial overload (selected members):**

| Member | max_sat (approx) |
|--------|-----------------|
| 1 | 1.077 |
| 3 | 1.081 |
| 9 | 1.222 |
| 16 | 1.201 |
| 26 | 1.261 |
| 49 | 1.081 |

Arcs 968, 606, 303 reported `overloaded=false` across all displayed initial members.

**Note — instrumentation issues (not reasons to discard the result):**
- Old instrumentation (pre-fix) emitted one line per time-slot violation per arc, causing duplicate lines per member. Fixed in `64f8c03c7` to emit one line per `(member, arc)` using max saturation.
- `sat=0.999999 overloaded=true` boundary case: `evaluate_violations()` uses `sat >= 1.0 - 1e-6`; the corrected `[c1c]` instrumentation uses strict `> 1.0`.
- Arc 659 appears in some `[diag]` lines alongside arc 658 — relationship between 658/659 not yet established.
- `max_sat` in `[diag]` header sometimes differs from the displayed `arc_overloaded` sat value — diagnostic display issue, not a causal finding.

**C1-C finding — setA-13 (defensible statement):**
> The dominant infeasibility is introduced during initial population construction following failed repair, not demonstrated to be created by crossover or mutation. The initial population contains 46 major violation events, predominantly on arc 658, with only 1 of 50 individuals valid. Gen-0 crossover and mutation produce zero classified causal violation events. The result therefore shifts the investigation from evolutionary operators toward constructor/repair feasibility and performance.

**Causal status:** For arc 658 / setA-13, the evidence strongly supports **inherited** classification: the bottleneck is present in the initial constructed genomes before any evolutionary operator executes. Crossover and mutation are propagating/recombining an already-infeasible population.

**setA-19 sweep complete** (`63dac87ad`). All three instances now have authoritative C1-C data.

---

#### C1-C Finding — setA-13 (corrected instrumentation, authoritative)

**Evidence source:** `evidence/phase10_p10c1c_initial_scan_corrected_raw.txt` (`6e2be7f17`)

One record per `(member, arc)`, `overloaded = max_sat > 1.0` (strict).

| Arc | Overloaded=true | Overloaded=false | % overloaded |
|-----|----------------|-----------------|--------------|
| 658 | **47** | 3 | **94%** |
| 606 | 0 | 50 | 0% |
| 303 | 0 | 50 | 0% |
| 968 | 0 | 50 | 0% |

**Causal classification — arc 658 / setA-13: INHERITED**
> 47 of 50 initial constructed genomes already carry arc 658 overload before any evolutionary operator executes.

---

#### C1-C Finding — setA-16 (preliminary sweep, authoritative for overloaded=false)

**Evidence source:** `evidence/phase10_p10c1c_initial_scan_raw.txt` (setA-16 section, `957ff1242`)

Note: old instrumentation emits per-time-slot lines, but since all results are `overloaded=false`, there are no duplicate lines — old and corrected instrumentation produce identical output when no violations exist. This data is therefore authoritative.

| Arc | Overloaded=true | Overloaded=false | % overloaded |
|-----|----------------|-----------------|--------------|
| 606 | **0** | 50 | **0%** |
| 303 | **0** | 50 | **0%** |

**Causal classification — arcs 606/303 / setA-16: CROSSOVER-CREATED**
> None of the 50 initial constructed genomes carry arc 606 or arc 303 overload. The gen-0 crossover events observed in C1-B are creating the overload, not propagating an inherited bottleneck.

---

#### C1-C Finding — setA-19 (corrected instrumentation, authoritative)

**Evidence source:** `evidence/phase10_p10c1c_seta19_raw.txt` (`63dac87ad`)

| Arc | Overloaded=true | Overloaded=false | % overloaded |
|-----|----------------|-----------------|--------------|
| 968 | **45** | 5 | **90%** |
| 658 | 0 | 50 | 0% |
| 606 | 0 | 50 | 0% |

**Causal classification — arc 968 / setA-19: INHERITED**
> 45 of 50 initial constructed genomes already carry arc 968 overload before any evolutionary operator executes. The C1-B `mutation` first-origin label was observational, not causal — the overload pre-exists mutation.

---

#### C1-C Complete Cross-Instance Summary

| Instance | Arc | Initial overloaded | % | C1-B first origin | Causal classification |
|----------|-----|-------------------|---|-------------------|-----------------------|
| setA-13 | 658 | 47/50 | 94% | crossover_ca | **INHERITED** |
| setA-16 | 606 | 0/50 | 0% | crossover_ca | **CROSSOVER-CREATED** |
| setA-16 | 303 | 0/50 | 0% | crossover_cb | **CROSSOVER-CREATED** |
| setA-19 | 968 | 45/50 | 90% | mutation | **INHERITED** |

**Key finding:** The C1-B `first_origin` label is not a reliable indicator of causal origin. Arc 658 (setA-13) and arc 606 (setA-16) both show `crossover_ca` as first origin in C1-B, but have opposite causal classifications in C1-C. The `first_origin` label records where the overload was first *observed* in the evolutionary pipeline, not where it *originated*.

**C1-C status: COMPLETE** (`63dac87ad`)

---

### C1-D — Constructor/Repair Isolation

**Status: LOCKED** (pending C1-C complete)

**Revised question (from C1-C setA-13 finding):** The C1-C evidence shifts the investigation from evolutionary operators toward constructor/repair feasibility. C1-D therefore asks: Is the 815-second initialization behavior and 92% initial infeasibility rate primarily a construction problem, a repair problem, or an interaction between construction/repair and the subsequent pipeline?

**Method:** Run 3 controlled cases for setA-13, seed=42, 50 individuals:

| Case | Constructor | Repair | MOGA | Purpose |
|------|-------------|--------|------|---------|
| A | ON | ON | OFF | Can construction+repair establish feasibility by itself? |
| B | ON | OFF | OFF | How much infeasibility is attributable to repair? |
| C | ON | ON | ON | What does the complete pipeline actually cost? (current baseline) |

**Metrics per case:** population size, successful constructions, repair attempts, repair failures, major violations, minor violations, epsilon violations, maximum saturation, arc distribution, wall time.

**Governance:** Observational. No algorithmic changes. Wall-clock and termination behavior are first-class metrics.

**Note:** Original C1-D question ("Did viable alternative routing paths exist?") is deferred — it presupposes evolutionary operators as the causal source, which C1-C has not confirmed.

---

### C1-E — Representation Test

**Status: LOCKED** (pending C1-C)

**Question:** Can the genome/topology representation express those alternatives, or does the representation itself constrain the available choices?

---

### C1-F — Causal Classification

**Status: LOCKED** (pending C1-C through C1-E)

**Classification targets:**

1. **Inherited** — parent already contains the bottleneck.
2. **Constructed** — both parents individually don't explain it, but the offspring's inherited combination does.
3. **Variation-introduced** — crossover/mutation creates the commitment.
4. **Representation/topology constrained** — the operator had no meaningful alternative representation available.
5. **Inevitable** — alternatives existed in representation, but every viable lineage converged on the bottleneck.

---

## Evidence Files

| File | Description |
|------|-------------|
| `evidence/phase10_p10c1_lineage_raw.txt` | Raw [c1b] sweep output (1,083 lines) |
| `evidence/phase10_p10c1_lineage_stderr.txt` | Sweep stderr log |

## Commit Chain

| Commit | Description |
|--------|-------------|
| `bc70bc30b` | C1-A: bottleneck census |
| `0f1896fa4` | C1-B: [c1b] instrumentation in pipeline_impl.rs |
| `10ef251e3` | C1-B: phase10c1_lineage binary |
| `2652c0bb5` | C1-B: partial sweep evidence + working tree checkpoint |
| `b61392a73` | C1-B: final sweep evidence — all 4 arcs captured |