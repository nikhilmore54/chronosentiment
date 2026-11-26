# Section 3 — Characterizing the Coralys Optimizer

**Status:** Stub — Phase 3 research plan. No experiments have been run yet.

**Depends on:** [`UltraCrew_Pairing_Topology_Mutation_Evaluation.md`](UltraCrew_Pairing_Topology_Mutation_Evaluation.md) Section 2 (frozen 2026-08-01)

**Freeze policy for Section 2:** Do not modify any subsection of Section 2 except for typographical or numerical corrections explicitly approved by the reviewer. Every conclusion in Section 2 is a fixed premise for this section.

---

## Research Framing

Section 2 answered the question: *Can we optimize the existing Greedy + Local Search pipeline?*

Section 3 addresses a stronger question: *Should constructive scheduling itself be treated as an optimization problem?*

This reframing reflects the evidence accumulated in Section 2. The central finding — that 73–95% of Greedy's candidate evaluations are immediately discarded, with the waste rate growing with instance size — motivates not just a faster scheduler but a different class of scheduler: one that searches the assignment space directly rather than constructing solutions exhaustively.

Section 3 is therefore **Phase II: Characterizing the Coralys Optimizer**, not a continuation of the Greedy comparison. Once sufficient evidence has accumulated that Coralys is a viable constructive scheduler, Greedy becomes a baseline rather than the subject of research. The primary questions become:

- Why does Coralys converge?
- When does it converge?
- What limits convergence?
- Which operators matter?
- How does diversity evolve?
- Why does one initialization outperform another?

**Research rule:** Every conclusion must be supported by measured experimental evidence. No architectural recommendations may be added without benchmark data.

---

## Evidence Gates

The experiments are organized around three progressively stronger claims. Each gate must be passed before the next is entered.

### Gate 1 — Can Coralys construct schedules?

A feasibility study. Greedy is still the comparison target at this stage.

Questions:
- Can Coralys produce feasible schedules without Greedy?
- Can it equal Greedy on objective value?
- Can it exceed Greedy?
- Does initialization strategy matter?
- Is the repair operator sufficient for large instances?

Experiments: Landscape Analysis (foundational), Coralys-only Construction, Initialization Study, Operator Study.

Deliverable: *Coralys is (or is not) a viable constructive scheduler.*

---

### Gate 2 — Can Coralys become the default scheduler?

Once feasibility is established, characterize engineering behaviour. Greedy is now a baseline, not the subject of research.

Questions:
- How does runtime scale with population and generation count?
- How robust is Coralys across multiple random seeds?
- What are the search dynamics (diversity, entropy, convergence rate, stagnation)?
- What is the minimum useful population size?
- Where is the convergence point?

Experiments: Multi-seed Validation, Population Scaling, Generation Scaling, Search Dynamics.

Deliverable: *Coralys is predictable, scalable, and operationally stable.*

---

### Gate 3 — Can Coralys become the scheduling engine?

Only after Gates 1 and 2 should the research expand the optimization problem itself. Greedy is no longer referenced.

Questions:
- Can Coralys optimize legality constraints alongside workload balance?
- Can it navigate a multi-objective Pareto front?
- Does it scale to industrial problem sizes?

Experiments: Legality-aware Evolution, Multi-objective Coralys, Scalability.

Deliverable: *Coralys optimizes the complete scheduling problem rather than only workload balance.*

---

## Planned Experiments

### Experiment 0 — Characterization of the Coralys Search Space (Foundational)

**This experiment is foundational and should be run before all others.**

Section 2.15 measured the Greedy candidate landscape (legal rate, improvement rate, tie rate, waste rate). This experiment measures the optimization landscape itself — the structure of the search space that Coralys must navigate — organized into three analytical layers.

---

#### Layer 1 — Search Space Geometry

Measures the static shape of the fitness landscape before any search dynamics are considered. Sample 100,000 random genomes and compute:

- **Fitness distribution** — histogram of `WorkloadBalanceObjective` values; reveals whether the landscape is unimodal, multimodal, or flat
- **Feasible fraction** — proportion of random genomes that satisfy all hard constraints without repair
- **Entropy** — Shannon entropy of the discretized fitness histogram; high entropy indicates a diffuse, hard-to-exploit landscape
- **Basin sizes** — estimated by random-restart hill climbing; large basins indicate exploitable gradient structure
- **Neutrality** — fraction of single-gene neighbours with equal fitness; high neutrality implies drift-dominated search

---

#### Layer 2 — Search Dynamics

Measures how fitness changes as the search moves through the space. Requires random walks of length ≥ 1,000 steps:

- **Fitness-distance correlation (FDC)** — correlation between fitness and Hamming distance to the best-known solution; FDC > 0.15 indicates a problem amenable to gradient-following
- **Fitness autocorrelation** — autocorrelation of fitness along random walks at lag 1, 2, 5, 10; rapid decay indicates a rugged landscape
- **Random-walk ruggedness** — variance of fitness differences along random walks; high variance means small steps produce large fitness changes
- **Local optima density** — number of local optima found per 1,000 hill-climbing restarts; high density indicates a multi-funnel landscape

---

#### Layer 3 — Representation Quality

Measures how well the genome representation supports the search operators. Requires controlled perturbation experiments:

- **Gene influence distribution** — for each genome position, the mean |Δf| from a single-gene mutation; reveals whether influence is uniform or concentrated in a few positions
- **Epistasis matrix** — pairwise interaction strength between genome positions; high off-diagonal values indicate that genes cannot be optimized independently
- **Mutation locality** — distribution of |Δf| for single-gene mutations; a tight distribution near zero indicates local, exploitable mutations
- **Crossover disruption** — mean fitness drop when two fit parents are crossed; high disruption indicates the representation does not support recombination well
- **Repair frequency** — fraction of offspring requiring constraint repair after crossover and mutation; high repair frequency indicates the feasible region is sparse

---

#### Decision Elasticity (cross-layer metric)

Decision Elasticity characterizes how the search space responds to random mutations. For each of 10,000 random (genome, mutation) pairs, compute Δf = f(x′) − f(x) and report:

- **P(improvement)** — probability that a random mutation improves fitness
- **P(neutrality)** — probability that a random mutation leaves fitness unchanged
- **P(catastrophic degradation)** — probability that a random mutation increases fitness by more than 10% of the current value
- **E[Δf | improvement]** — expected improvement magnitude given that improvement occurs

Decision Elasticity is the primary indicator of whether Coralys's mutation operator is well-matched to this problem. A high P(improvement) with a large E[Δf | improvement] indicates that random search is already effective and that the EA's selection pressure adds little value. A low P(improvement) with a high P(catastrophic degradation) indicates that the mutation operator is too destructive and should be replaced or tuned before further experiments.

---

#### Implementation Note — Shared Sampling Pipeline

All metrics in Layers 1–3 and Decision Elasticity must consume the **same sampled population** rather than each generating independent samples. The implementation pipeline is:

```
Genome Sampler  (deterministic, seeded, n = 100,000)
      ↓
Fitness Evaluator  (WorkloadBalanceObjective + feasibility check)
      ↓
Metric Collector  (one pass per layer; no re-sampling)
      ↓
Result Aggregator  (per-instance statistics)
      ↓
CSV / JSON / Markdown  (via harness persistence + report modules)
```

This ensures identical data across all metrics, lower computational cost, and simpler reproducibility. Future metrics added to Experiment 0 must consume the same sampled population; they must not introduce a new sampling pass.

---

#### Intermediate Artifact — `landscape_sample`

In addition to the derived statistics, persist the sampled population itself as a reusable dataset:

```
results/experiment_0/instance_<N>/landscape_sample.csv
```

Each row represents one sampled genome and contains:

| Column | Description |
|--------|-------------|
| `genome_id` | Sequential integer (0-indexed) |
| `genome_hash` | FNV-1a 64-bit hash of the genome encoding |
| `objective_value` | `WorkloadBalanceObjective` score |
| `is_feasible` | Boolean — satisfies all hard constraints without repair |
| `n_constraint_violations` | Count of violated hard constraints |
| `repair_steps` | Number of repair iterations required (0 if feasible) |
| `mutation_neighbor_delta` | Mean \|Δf\| over 10 random single-gene mutations from this genome |

This dataset becomes the reusable foundation for future metrics. Any metric that can be computed from (genome, fitness, feasibility, neighborhood) does not require a new sampling run. It also enables independent replication: a reviewer can re-derive all reported statistics from `landscape_sample.csv` without re-running the experiment.

---

#### Freeze Protocol

Treat this specification the same way Section 2 was treated before implementation.

Before writing any implementation code:

1. **Review the metric definitions** — confirm that each metric in Layers 1–3 and Decision Elasticity is precisely defined (formula, sample size, statistical estimator, units)
2. **Freeze the sampling protocol** — record the exact seed, sample size (n = 100,000 for Layer 1–3, n = 10,000 for Decision Elasticity), and genome generation procedure
3. **Freeze the statistical procedures** — record the exact estimators (e.g. Shannon entropy with base-2 logarithm and bin width = 0.01 × fitness range; FDC using Pearson r; autocorrelation at lags 1, 2, 5, 10)
4. **Record the freeze date** — append a `<!-- Frozen: YYYY-MM-DD -->` comment to this section header once the above three steps are complete

Do not change metric definitions after seeing results. Post-hoc metric redefinition weakens the evidential strength of the study and must be treated as a new experiment with a new experiment number.

---

This experiment explains *why* Coralys works (or does not work) on this problem class. Most optimization papers report that an algorithm performs well; very few explain the structure of the search landscape that makes it perform well. Because Coralys is intended as a general optimization platform, this experiment has value beyond UltraCrew — it begins to build a scientific understanding of the optimization problems Coralys is designed to solve.

**Gate:** 1

---

### Experiment 1 — Coralys-only Construction

Remove Greedy from the pipeline. Run Coralys EA directly on the pairing sets produced by the pairing builder.

Pipeline:
```
Flights → Pairing Builder → Coralys EA
```

Measure:
- Objective value (vs Greedy baseline from Section 2)
- Runtime
- Convergence curve (objective vs generation)
- Feasibility (% of population that is feasible at termination)
- Repair count (how many times the repair operator fires per run)

**Gate:** 1

---

### Experiment 2 — Initialization Study

Round-robin initialization showed promise in Section 2.17. Compare:
- Round-robin
- Random
- Greedy-seeded
- Heuristic-seeded
- Mixed population

Measure: convergence speed, final objective, diversity, robustness across seeds.

**Gate:** 1

---

### Experiment 3 — Operator Study

Current Coralys uses mutation + crossover. Evaluate each independently.

Conditions:
- Mutation only
- Crossover only
- Mutation + crossover (current)
- Varying mutation probabilities
- Varying crossover probabilities

Determine: operator contribution to solution quality and convergence speed.

**Gate:** 1

---

### Experiment 4 — Multi-seed Validation

Section 2 used `seed = 42` (single-seed limitation acknowledged in Section 2.17). Evaluate 10–30 independent seeds.

Measure:
- Mean, median, standard deviation, best, worst objective value
- 95% confidence intervals

Goal: Determine robustness rather than single-run performance.

**Gate:** 2

---

### Experiment 5 — Population Scaling

Evaluate population sizes: 20, 50, 100, 200, 400.

Measure: convergence speed, runtime, final objective value, population diversity.

Determine: minimum useful population size.

**Gate:** 2

---

### Experiment 6 — Generation Scaling

Evaluate generation counts: 25, 50, 100, 200, 400, 800.

Measure: objective value vs generations.

Identify: convergence point and diminishing returns threshold.

**Gate:** 2

---

### Experiment 7 — Search Dynamics (Observability Layer)

Instrument Coralys to collect per-generation metrics:
- Population diversity
- Entropy
- Mutation acceptance rate
- Crossover acceptance rate
- Fitness variance
- Convergence rate
- Stagnation detection
- Population similarity

This becomes the Coralys observability layer for all future experiments. It answers: why does Coralys converge, when does it converge, and what limits convergence?

**Gate:** 2

---

### Experiment 8 — Legality-aware Evolution

Current experiments optimize `WorkloadBalanceObjective` without full legality constraints. Introduce legality into fitness.

Measure:
- Repair frequency
- Feasible population percentage
- Convergence
- Runtime

**Gate:** 3

---

### Experiment 9 — Multi-objective Coralys

Replace single objective with multiple objectives. Candidate objectives:
- Workload balance
- Duty balance
- Robustness
- Reserve utilisation
- Deadhead
- Hotel nights
- TAFB

Evaluate Pareto front quality.

**Gate:** 3

---

### Experiment 10 — Scalability

Generate increasing problem sizes: 500, 1000, 2000, 5000, 10000 pairings.

Measure: runtime, memory, convergence, solution quality.

Identify complexity experimentally (not analytically).

**Gate:** 3

---

## Section Structure

Each experiment subsection follows the discipline established in Section 2:

1. Experimental Objective
2. Instrumentation
3. Experimental Configuration
4. Raw Measurements
5. Derived Metrics
6. Statistical Analysis
7. Findings
8. Threats to Validity
9. Conclusions

No conclusions before presenting the supporting measurements.

---

## Deliverables

At the completion of Section 3, the research should answer, with evidence:

1. What is the structure of the airline scheduling optimization landscape?
2. Can Coralys replace Greedy construction?
3. Under what problem sizes does it outperform deterministic construction?
4. Which initialization strategy is best?
5. Which evolutionary operators contribute most?
6. How robust is Coralys across multiple random seeds?
7. What are the search dynamics (diversity, entropy, convergence, stagnation)?
8. Does legality-aware evolution remain competitive?
9. How does Coralys scale with problem size?
10. What evidence supports deploying Coralys as the default constructive scheduler?

---

## Open Questions from Section 2

The following open research questions from Section 2.18 are directly addressed by this section:

- **Open RQ1:** Where is the crossover point between Greedy-faster and Coralys-faster regimes? (between Instance 3 P×R=6,627 and Instance 4 P×R=82,795) → Experiment 10 (Scalability)
- **Open RQ2:** Why does Coralys outperform Greedy on Instances 3 and 5? (structural properties of the objective landscape) → Experiment 0 (Landscape Analysis), Experiments 3, 7
- **Open RQ3:** Can Coralys optimize earlier decisions (duty boundaries, pairing topology, crew assignment)? → Experiment 9 (Multi-objective), future Section 4

---

## Research Narrative

Section 2 = Phase I: Understanding the Existing Scheduler.
Section 3 = Phase II: Characterizing the Coralys Optimizer.

The narrative arc across both sections:

1. Identify the Local Search bottleneck (Section 2.10–2.11)
2. Remove it; observe the new bottleneck (Section 2.12–2.13)
3. Characterize the Greedy search landscape (Section 2.15)
4. Justify an architectural change (Section 2.16)
5. Validate the change experimentally (Section 2.17)
6. Characterize the optimization landscape itself (Section 3, Experiment 0)
7. Establish Coralys as a viable constructive scheduler (Section 3, Gate 1)
8. Characterize its engineering behaviour (Section 3, Gate 2)
---

## Phase II Execution Roadmap

The following milestones translate the experiment plan into an ordered implementation sequence. Each milestone is a prerequisite for the next.

### Milestone 1 — Experimental Infrastructure (prerequisite for all experiments)

Freeze the current [`adapters/airline/tests/gerad_coralys.rs`](../../adapters/airline/tests/gerad_coralys.rs) as the baseline. Build a reusable experiment harness that:
- Emits machine-readable CSV/JSON output for every run
- Standardizes logging format across all experiments
- Generates automatic per-experiment reports
- Requires no manual extraction of metrics

Every subsequent experiment emits identical metrics automatically.

### Milestone 2 — Coralys Observability Layer (prerequisite for Experiments 0–10)

Before changing algorithms, make Coralys observable. Instrument every generation with:
- Best fitness
- Average fitness
- Population diversity
- Entropy
- Mutation acceptance rate
- Crossover acceptance rate
- Feasible population %
- Repair count
- Stagnation indicator
- Elapsed time

This observability layer is the instrumentation used throughout Section 3.

### Milestone 3 — Experiment 0: Landscape Analysis

First scientific experiment. Generate ~100,000 random genomes and measure fitness distribution, neutrality, ruggedness, epistasis, FDC, basin sizes, local optima density, and entropy. Establishes the properties of the search space before evaluating algorithms.

### Milestone 4 — Experiment 1: Coralys-only Scheduler

Remove Greedy. Keep the same constraints and objectives. Optimize assignment directly. Answers Gate 1 question: *Can Coralys construct schedules without Greedy?*

### Milestone 5 — Experiments 2–3: Initialization and Operator Study

Only after Coralys can build schedules. Compare initialization strategies (empty, random, round-robin, Greedy-seeded, heuristic, previous roster) and operator contributions (mutation only, crossover only, combined, varying probabilities).

### Milestone 6 — Experiments 4–7: Engineering Characterization (Gate 2)

Only after feasibility is demonstrated. Run population scaling, generation scaling, multi-seed validation (10–30 seeds, 95% confidence intervals), and convergence analysis. Completes Gate 2.

### Milestone 7 — Experiments 8–9: Multi-objective Coralys (Gate 3)

Only after Coralys is established as a constructive scheduler. Expand from workload balance to legality, TAFB, hotel nights, deadheads, reserve coverage, and robustness. Begins Gate 3.

### Milestone 8 — Experiment 10: Scalability

Generate increasing problem sizes (500, 1000, 2000, 5000, 10000 pairings). Identify complexity experimentally. Completes Gate 3.

---

## Publication Strategy

The work separates into a sequence of publishable research artifacts, each building directly on the evidence established by the previous one:

| Paper | Title | Source |
|-------|-------|--------|
| Paper I | Bottleneck Analysis of Deterministic Crew Scheduling | Section 2 (complete) |
| Paper II | Characterizing the Search Landscape of Airline Crew Scheduling | Experiment 0 |
| Paper III | Coralys as a Constructive Evolutionary Scheduler | Gate 1 |
| Paper IV | Engineering Characteristics of a Population-Based Scheduler | Gate 2 |
| Paper V | Multi-objective Evolutionary Workforce Scheduling | Gate 3 |

**Highest-priority next task:** Milestone 1 (experimental infrastructure). Once the harness exists, every subsequent experiment can be reproduced automatically, logged consistently, and incorporated into Section 3 with minimal manual effort.
9. Expand to the complete scheduling problem (Section 3, Gate 3)