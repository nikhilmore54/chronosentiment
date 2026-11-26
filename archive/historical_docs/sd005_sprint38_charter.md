# Sprint 3.8 Charter — SD-005: Feasibility Lineage Investigation

**Sprint:** 3.8  
**Status:** OPEN  
**Depends on:** Sprint 3.7 (SD-006 CLOSED)  
**Target artifact:** `feasible_lineage_report.md`

---

## Research State Entering Sprint 3.8

```
SD-003: CLOSED — Proxy/External Misalignment (Pareto domination geometry)
SD-006: CLOSED — O3 (HC_Successions proxy) is the sole driving objective

Known:
    O3 pressure causes champion eviction.
    The dominator improves O3 while worsening HC_Coverage and HC_ForbiddenSuccessions externally.
    This is confirmed across 4 independent seeds (gen2, gen29, gen158, gen283).

Unknown:
    Does O3-driven domination also remove feasibility from the archive?
```

SD-005 is no longer an independent feasibility census question.

It is a **causal dependency investigation**: does the same O3 mechanism that evicts
externally-good champions also evict feasible solutions, producing the observed 0% feasibility?

---

## SD-005 Classification Table (frozen before any run)

| Observation | Classification |
|---|---|
| No feasible solution ever produced by the evaluator | Discovery Failure |
| Feasible solution produced but never admitted to archive | Admission Failure |
| Feasible solution admitted then evicted (exit_reason = Dominated) | Retention Failure |
| Feasible solution survives in archive but archive remains ~0% feasible | Representation Failure |
| Feasible solutions persist in archive at gen 5000 | SD-005 Falsified |

This table is immutable. Classification is applied post-run against the lineage evidence.
No post-hoc reinterpretation is permitted.

---

## Instrumentation Design

### Core Principle

Sprint 3.6 proved that lifecycle tracking (not aggregate statistics) was the decisive instrument
for SD-003. Sprint 3.7 confirmed the same pattern for SD-006: a single `DominationEvent` record
with chain-of-custody fields produced the attribution.

Sprint 3.8 applies the same discipline to feasibility.

### FeasibleLifecycle Struct

```rust
struct FeasibleLifecycle {
    genome_hash: u64,
    discovered_at: u32,       // generation when evaluator first returned feasible=true
    admitted_at: Option<u32>, // generation when admitted to archive (None = never admitted)
    evicted_at: Option<u32>,  // generation when evicted from archive (None = still present)
    exit_reason: Option<ExitReason>, // Dominated | Crowding | ArchiveLimit | Unknown
    hc_total: usize,          // sum of all HC violation counts at discovery
    official_total: f64,      // official_total at discovery
    proxy: Vec<f64>,          // proxy objective vector at discovery
}
```

This mirrors `ChampionTracker` discipline: one record per feasible genome, with full
chain-of-custody from discovery through admission through eviction.

### Three-Phase Lineage Questions

**Phase 1 — First feasible genome:**
```
generation
official_total
archive admitted?
archive lifetime (gens)
exit reason
```

**Phase 2 — Best feasible genome (lowest official_total among all feasible ever seen):**
```
generation
official_total
archive lifetime (gens)
exit reason
```

**Phase 3 — All feasible genomes:**
```
total feasible genomes discovered
total admitted to archive
total evicted (with exit_reason breakdown)
still in archive at gen 5000
```

### Feasibility Census Timeline

Sampled every 100 generations (not every generation — O(N) per-gen scoring is expensive):

```
Gen 0, 100, 200, ..., 5000:
    feasible_count        (hc_total == 0)
    near_feasible_5       (hc_total <= 5)
    near_feasible_10      (hc_total <= 10)
    infeasible_count
```

This is Section 3 of the report. It answers whether feasibility ever appeared transiently
even if it did not persist.

---

## Causal Chain Hypothesis

If Sprint 3.8 discovers:

```
Feasible genome
    ↓ admitted to archive
    ↓ evicted (exit_reason = Dominated)
    ↓ dominator improves O3
```

then SD-005 and SD-006 collapse into a single causal chain:

```
O3 pressure
    ↓
Proxy domination
    ↓
Champion eviction (SD-006, CLOSED)
    ↓
Feasible genome eviction (SD-005, would be CLOSED as Retention Failure)
    ↓
0% feasible archive
```

This is the highest-value result. It provides a unified explanation for both anomalies
supported by chain-of-custody evidence rather than aggregate statistics.

If instead no feasible genome is ever produced, the classification is Discovery Failure —
the O3 mechanism is not implicated, and the root cause is upstream of the archive entirely
(evaluator landscape, mutation operators, or constraint structure).

---

## Target Artifact

### `feasible_lineage_report.md`

#### Section 1 — First Feasible Genome

| Field | Value |
|---|---|
| Genome hash | |
| Discovered at generation | |
| HC_Total at discovery | |
| OfficialTotal at discovery | |
| Archive admitted? | |
| Admitted at generation | |
| Archive lifetime (gens) | |
| Evicted at generation | |
| Exit reason | |

#### Section 2 — Best Feasible Genome

| Field | Value |
|---|---|
| Genome hash | |
| Discovered at generation | |
| OfficialTotal at discovery | |
| Archive lifetime (gens) | |
| Exit reason | |

If first == best, Section 2 states: "Same as Section 1."

#### Section 3 — Feasibility Census Timeline

| Generation | feasible_count | near_feasible_5 | near_feasible_10 | infeasible_count |
|---|---|---|---|---|
| 0 | | | | |
| 100 | | | | |
| ... | | | | |
| 5000 | | | | |

#### Section 4 — SD-005 Classification

Applies the frozen classification table to the lineage evidence.
States the classification and the evidence record that supports it.

If the causal chain hypothesis is confirmed (Retention Failure + O3 dominator):
states the unified SD-005/SD-006 causal chain explicitly.

---

## Implementation Plan

### Step 1 — Add FeasibleLifecycle tracking to `inrc_archive_forensics.rs`

New state variables alongside existing Sprint 3.7 instrumentation:

```rust
let mut feasible_lifecycles: Vec<FeasibleLifecycle> = Vec::new();
let mut feasible_archive_members: HashMap<u64, usize> = HashMap::new(); // genome_hash → index in feasible_lifecycles
let mut census_timeline: Vec<(u64, usize, usize, usize, usize)> = Vec::new(); // (gen, feasible, near5, near10, infeasible)
```

In the generation loop, after `score_inrc_official`:

```rust
if child_score.feasible {
    // Record discovery
    let lifecycle = FeasibleLifecycle {
        genome_hash: child_uid,
        discovered_at: g as u32,
        admitted_at: if was_inserted { Some(g as u32) } else { None },
        evicted_at: None,
        exit_reason: None,
        hc_total: child_score.hc_coverage + child_score.hc_skills
                + child_score.hc_one_shift_per_day + child_score.hc_forbidden_successions,
        official_total: child_score.official_total,
        proxy: child_fitness.clone(),
    };
    let idx = feasible_lifecycles.len();
    feasible_lifecycles.push(lifecycle);
    if was_inserted {
        feasible_archive_members.insert(child_uid, idx);
    }
}
```

In the eviction loop, after existing eviction handling:

```rust
if let Some(&lc_idx) = feasible_archive_members.get(old_uid) {
    feasible_lifecycles[lc_idx].evicted_at = Some(g as u32);
    feasible_lifecycles[lc_idx].exit_reason = Some(reason);
    feasible_archive_members.remove(old_uid);
}
```

Census sampling every 100 generations:

```rust
if g % 100 == 0 {
    let feasible = engine.archive.solutions.iter()
        .filter(|s| {
            let sc = score_inrc_official(&s.genome, &scenario, &inrc_optimizer);
            sc.feasible
        }).count();
    // near_feasible_5, near_feasible_10, infeasible computed similarly
    census_timeline.push((g, feasible, near5, near10, infeasible));
}
```

**Note:** Per-generation full-archive scoring is expensive (O(archive_size) evaluations per gen).
Census sampling at 100-gen intervals keeps the 5000-gen run tractable.

### Step 2 — Write `feasible_lineage_report.md`

After the generation loop, using the same report-writing pattern as the Sprint 3.7
domination report writer (lines 524–676 of the current binary).

Four sections as specified above. Classification applied in Section 4.

### Step 3 — Smoke run (--gens 100)

Verify the binary compiles and the report is produced without panics.
Check that `FeasibleLifecycle` records are populated correctly.

### Step 4 — Full run (seed=61, --gens 5000)

Use the canonical seed from Sprint 3.7 for continuity.
Produce `feasible_lineage_report.md`.

### Step 5 — Classify SD-005

Apply the frozen classification table to the lineage evidence.
Update `sd003_resolution.md` (or a new `sd005_resolution.md`) with the classification.

---

## Exit Criterion

Sprint 3.8 is complete when:

1. `feasible_lineage_report.md` is produced with all four sections populated
2. SD-005 is classified against the frozen table with supporting evidence
3. If Retention Failure: the dominator's proxy vector is recorded and O3 delta is stated
4. If Discovery Failure: the classification is stated with the total feasible count (0)
5. `sd003_resolution.md` (or `sd005_resolution.md`) is updated and committed

---

## What Would NOT Be Acceptable

- Classifying SD-005 from the census alone (Section 3) without lineage evidence (Sections 1–2)
- Post-hoc reinterpretation of the classification table
- Skipping the per-genome chain-of-custody record in favour of aggregate counts
- Treating "0 feasible at gen 5000" as sufficient evidence for Discovery Failure
  (it is consistent with Discovery Failure AND with Retention Failure where all feasible
  genomes were evicted before gen 5000)

The lineage record is what distinguishes these two cases. It is the mandatory instrument.