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

**Status: PENDING**

**Question:** Were arcs 658/606/303/968 already overloaded in the initial constructed genomes before any evolutionary operator executed, or did the gen-0 evolutionary transition create or propagate the overload?

**Method:** Instrument the initial population **before any evolutionary operator executes**. For each initial genome member, record for arcs 968/658/606/303:

```
[c1c] stage=initial member=N arc=ARC sat=S overloaded=true/false
```

Then track parent→child relationships through gen-0 crossover.

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

### C1-D — Alternative-Path Availability

**Status: LOCKED** (pending C1-C)

**Question:** Did viable alternative routing paths exist for the demands that load the bottleneck arcs?

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