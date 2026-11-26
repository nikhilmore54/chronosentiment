# Coralys Research Programme — ROADEF 2012 Benchmark Series

**Document status:** Living programme document — updated 2026-08-06; strategic review applied 2026-08-06; TRL maturity model added 2026-08-06; RP-409C frozen 2026-08-06; three-layer framing adopted 2026-08-06; RC-008 and EEB governance added 2026-08-06; architecture frozen 2026-08-06; primary objective and extended EEB equation added 2026-08-06
**Primary Programme Objective:** Develop and submit a state-of-the-art solver for the ROADEF 2012 Challenge while extracting scientifically validated optimisation methodologies and reusable Coralys platform capabilities. Competition performance is the primary near-term objective. Science explains the competition results. Platform capabilities are extracted from validated work, not built speculatively.
**Phase 1 (Scientific Foundation):** COMPLETE — RP-406C through RP-409C all frozen
**Phase 2 (Competition Optimisation):** Active — two-track structure (Research Track + Competition Track)

**Programme architecture (frozen 2026-08-06):**

```
Scientific Research (RP-series)
        │  establishes causal understanding; produces frozen evidence reports
        ▼
Competition Engineering (RC-series + RS-series)
    ├── RC-001 … RC-008  independent A/B campaigns against CB-000
    └── RS-001 … RS-003  release engineering: integration, tuning, submission freeze
        │  RS is the release phase of Competition Engineering, not a separate layer
        ▼
Platform Development (Coralys)
        extracts reusable optimisation infrastructure for all Coralys domains
```

**Three programme layers:**
- **Scientific Research** — explains *why* the solver behaves as it does. Produces frozen evidence reports (RP-series). Phase 1 complete; RP-409D and RP-409E remain as optional refinement studies.
- **Competition Engineering** — improves the solver against the ROADEF benchmark. RC-series: independent A/B campaigns against CB-000. RS-series: release engineering (integration, tuning, submission freeze). RS is the release phase of Competition Engineering, not a separate engineering discipline.
- **Platform Development** — consolidates validated optimisation capabilities, telemetry, and methodology into reusable Coralys platform components. Primary output: Coralys Evolution Observatory (candidate genealogy, tournament analytics, promotion funnel, rejection tracing, EEB instrumentation, subsystem metrics). Reusable across UltraCrew, CVRP, workforce scheduling, and other Coralys domains.

**Three-level objective distinction (governing all comparator decisions):**
- **Competition evaluation objective:** ROADEF lexicographic ranking (fixed by the benchmark — cannot be changed)
- **Internal evolutionary comparator:** Implementation choice; current reference is `ComparatorMode::Scalar`, retained because RP-408 showed it outperforms the first lexicographic comparator design
- **Research objective:** Develop a comparator that more effectively aligns evolutionary search with the ROADEF lexicographic evaluation without sacrificing solution quality — an open research direction

---

## 1. Programme Context

RP-406C established a stable scientific baseline for Coralys behaviour against the ROADEF 2012
sprint-reference solutions across all 20 Set A instances. The report is permanently frozen and
serves as the benchmark baseline for all subsequent milestones.

Key findings carried forward from RP-406C:

- **Two regimes identified:** Collapsed Basin (6 instances) vs. normal Shape Competition (14 instances).
- **Collapsed Basin instances:** setA-06, setA-08, setA-10, setA-13, setA-16, setA-19.
- **Four durable wins (regression baseline):** setA-12, setA-15, setA-17, setA-18.
- **Four-zone solution signature:** Peak / Shoulder / Transition / Tail.
- **Shoulder region (Ranks 2–20)** is where competitions are won and lost.
- **Shoulder Dominance Index (SDI)** defined as the primary optimisation metric for RP-409.
- **ROADEF competition evaluation** uses lexicographic ranking of sorted link load vectors — this is fixed by the benchmark and cannot be changed. The internal evolutionary comparator is a separate implementation choice; the current reference is the scalar comparator (see header).

RP-407 and RP-410A have since extended this baseline with feasibility analysis and search dynamics
evidence. All three are frozen. RP-411, RP-412, RP-410B, and RP-409C are now complete with findings reports.

---

## 2. Five-Subsystem Framework

The solver is viewed as five interacting subsystems. RP-410B revealed that Promotion (tournament
selection, elite replacement, global-best update) is mechanistically distinct from Variation
(crossover, mutation) and warrants its own subsystem. Every research programme targets exactly one
subsystem. No new optimisation feature may be merged unless its intended subsystem is identified
and its success is evaluated against the frozen baseline metrics for that subsystem.

| Subsystem        | Mechanisms                                          | Question                                                        | Status                                      |
| ---------------- | --------------------------------------------------- | --------------------------------------------------------------- | ------------------------------------------- |
| **Construction** | Constructor, repair                                 | Can we produce a feasible population?                           | Instrumented and characterised (RP-407, RP-412) |
| **Execution**    | Evaluation loop, generation budget                  | Can enough generations be executed for evolution to matter?     | Characterised (RP-411); phase timing gap remains |
| **Variation**    | Crossover, mutation                                 | What kinds of candidates do operators produce? (COR)            | Characterised (RP-410A, RP-410B) |
| **Promotion**    | Tournament, elite replacement, global-best update   | Which candidates survive selection? (PE)                        | Fully characterised (RP-410C, RP-409C) |
| **Objective**    | Scalar MLU vs. lexicographic comparator             | Does the objective reward the behaviour required by the competition? | Hypothesis established; testable after RP-410C (RP-408) |

**Note on RP-410C and RP-409C placement:** Both study the Promotion subsystem, not the Variation
subsystem. RP-410C records tournament outcomes, elite replacement decisions, and population entry.
RP-409C wires full candidate genealogy (parent IDs, tournament ID) through the evaluation boundary
so that the promotion funnel is reconstructable from raw telemetry. Together they provide
end-to-end observability of the Promotion pipeline.

---

## 3. Frozen Baseline

The following reports are permanently frozen and serve as the scientific baseline for all
subsequent milestones. They must not be modified. Future RPs reference them rather than revising them.

| Report                                                                          | Subsystem    | Primary Outcome                                                                 |
| ------------------------------------------------------------------------------- | ------------ | ------------------------------------------------------------------------------- |
| [`RP406C_BENCHMARK_REPORT.md`](RP406C_BENCHMARK_REPORT.md)                     | All          | Benchmark characterisation and lexicographic behaviour                          |
| [`RP407_FINDINGS.md`](RP407_FINDINGS.md)                                        | Construction | Feasibility analysis; construction failure vs. evolutionary collapse            |
| [`RP410A_FINDINGS_V2.md`](RP410A_FINDINGS_V2.md)                               | Variation    | Search dynamics, operator fingerprints, zone behaviour                          |
| [`RP410C_FINAL_REPORT.md`](RP410C_FINAL_REPORT.md)                             | Promotion    | Full survival funnel; Tournament/Population/Elite PE decomposition              |
| [`RP411_412_BASELINE_REPORT.md`](RP411_412_BASELINE_REPORT.md)                 | Execution / Construction | Eval=99.99% of runtime; throughput 1,555× spread; mean IFR=10.6%; 6/20 IFR=0% |
| [`RP408_REPORT.md`](RP408_REPORT.md)                                           | Objective    | First lexicographic comparator design not adopted as default; scalar comparator retained as reference; improved comparator designs remain open research direction |
| [`RP409A_REPORT.md`](RP409A_REPORT.md)                                         | Variation    | Crossover dominates accepted improvements; mutation Peak ACR = 0; ACR/APS/AOR metrics |
| [`RP409B_REPORT.md`](RP409B_REPORT.md)                                         | Variation    | PeakTargeted mutation hypothesis falsified; indirect population perturbation mechanism |
| [`RP409C_REPORT.md`](RP409C_REPORT.md)                                         | Promotion    | Full genealogy wired: parent IDs, tournament ID, population slot, elite slot, rejection reasons all propagated through evaluation boundary; promotion funnel reconstructable from telemetry |

---

## 4. Effective Evaluation Budget Framework

### 4.1 The Unifying Hypothesis

The diagnostic phase (RP-406C through RP-412) has established a single unifying hypothesis
that governs all subsequent optimisation work:

> **The diagnostic phase identifies Effective Evaluation Budget (EEB) as the dominant
> limiting factor observed in the current implementation.**
>
> The principal resource consumed by the ROADEF solver is evaluation budget. Construction
> quality determines how much of that budget is spent on feasible candidates. Variation
> determines the kinds of candidates generated. Promotion determines which candidates survive.
> Execution throughput determines how much total search can be performed. All subsequent
> optimisation work should therefore be evaluated in terms of its impact on effective
> evaluation budget — not just whether the final objective improves.

**Note on scope:** The evidence strongly supports EEB as a major limiting factor in the
current implementation. It does not yet prove EEB is the fundamental limiting factor in all
cases — some instances may also be limited by search landscape characteristics or
representation quality even if evaluation became effectively free. The hypothesis is
falsifiable: if RP-408 and RP-409 deliver the expected gains on fast-tier instances but
slow-tier instances remain unimproved, that is consistent with the EEB hypothesis. If
fast-tier instances also fail to improve, the hypothesis requires revision.

**Operational definition:**

For this research programme, Effective Evaluation Budget is operationally defined as:

```
EEB = N_eval × IFR × COR × PE
    = N_eval × IFR × OSR
```

where every term is a directly measured telemetry quantity:
- `N_eval` = total evaluations executed within the time budget (RP-411; baseline: median 8.5 generations × 50 pop = ~425 evals/instance)
- `IFR` = Initial Feasibility Rate — fraction of evaluations starting from a feasible candidate (RP-412; baseline: mean 10.6%)
- `COR` = Candidate Opportunity Rate — fraction of generated candidates that improve a target zone (RP-410A/B; baseline: Peak 0.91%, Shoulder 7.01%)
- `PE` = Promotion Efficiency — fraction of zone-improving candidates that survive selection (RP-410C; baseline: Peak 3.01%, Shoulder 6.35%)
- `OSR` = Overall Success Rate = COR × PE — end-to-end probability a generated candidate becomes an accepted improvement (baseline: Peak 0.027%, Shoulder 0.445%)

**Note on the model:** EEB is an engineering effectiveness index rather than a strict probabilistic identity. `IFR` characterises the quality of the initial search state (measured once at construction), whereas `COR` and `PE` characterise the effectiveness of subsequent evolutionary search (measured over the full run). The product serves as a practical index for comparing solver configurations rather than an exact probability.

This gives an intuitive decomposition:

```
EEB  =  Search Budget  ×  Search Quality
     =  N_eval         ×  (IFR × OSR)
```

**Extended EEB equation (execution engineering link):** Because `N_eval` is itself determined by the time budget and the cost of each evaluation, the full equation is:

```
         Time Budget
N_eval = ─────────────────
         Evaluation Cost

         Time Budget
EEB    = ───────────────── × IFR × COR × PE
         Evaluation Cost
```

This links execution engineering directly to search effectiveness. The algorithmic RC milestones improve the quality side (IFR, COR, PE). Execution engineering — incremental evaluation, parallel evaluation, memory optimisation — reduces Evaluation Cost and therefore increases N_eval within the same time budget. Both attack EEB through different mechanisms and are complementary rather than competing. Execution engineering is not part of the current RC sequence but is identified as a future engineering stream after RC-008.

**Figure 1. Effective Evaluation Budget (EEB) Framework**

```
                    Search Budget
                          │
                       N_eval
                          │
                          ▼
           Effective Evaluation Budget
                          ▲
                          │
                   Search Quality
             IFR  ×  COR  ×  PE
           = IFR  ×      OSR
```

Subsequent intervention RPs (RP-408 onward) use this framework as their primary experimental
model. Each intervention changes one term while the others are held constant by the
frozen-baseline experimental design.

Each RP targets one primary measurable factor while treating the remaining factors as controlled baseline variables, held constant by the frozen-baseline experimental design:

| RP | Factor changed | Primary metric | Mechanism |
|----|---------------|----------------|-----------|
| RP-411 / Exec. Opt. | `N_eval` ↑ | evals/s, generation count | Faster evaluation → more evaluations per unit time |
| RP-412 D4 | `IFR` ↑ | IFR (mean, per-instance) | Repair-based construction → more feasible initial candidates |
| RP-408 | `PE` ↑ | Peak PE, Shoulder PE | Better comparator → better survival of useful candidates through Promotion |
| RP-409 | `COR` ↑ | Peak COR, Transition COR | Better operators → more candidates in productive zones |

The decomposition allows the contribution of each intervention to be measured independently
while keeping the remaining factors fixed. Every future RP modifies exactly one measurable
factor; changes in overall search effectiveness can be attributed quantitatively to the
subsystem under investigation.

Every frozen report contributes evidence toward this hypothesis:

| Report | Subsystem | Evidence |
|--------|-----------|---------|
| RP-406C | All | Where improvements occur; four-zone solution signature |
| RP-410A/B | Variation | What operators generate; zone-specific COR |
| RP-410C | Promotion | What survives; full survival funnel PE decomposition |
| RP-411 | Execution | How many evaluations are affordable; 99.99% eval fraction; 1,555× throughput spread |
| RP-412 | Construction | How many of those evaluations start from feasible candidates; mean IFR 10.6% |

### 4.2 Causal Chain

The diagnostic data supports a specific causal chain explaining why the most difficult
instances receive almost no useful evolutionary search:

```
Poor constructor (mean IFR 10.6%; 6/20 instances IFR=0%)
        ↓
90% infeasible initial population
        ↓
Very expensive evaluator (99.99% of runtime; 1,555× throughput spread)
        ↓
Very few generations (median 8.5; 2 generations for setA-17, setA-20)
        ↓
Little evolutionary search (16/20 instances time-budget-starved)
        ↓
Few elite candidates
        ↓
Few global improvements
```

The problem is not primarily that the evolutionary algorithm is weak. The problem is that
the current implementation operates under an extremely constrained effective evaluation
budget. For the most difficult instances, the search is not converging to a poor solution —
it often never has sufficient opportunity to search at all.

### 4.3 Common Denominator

Each intervention in the programme increases the expected information gained per evaluation:

| RP | Manipulated subsystem | EEB factor | Expected response | Primary metric |
|----|-----------------------|-----------|-------------------|----------------|
| RP-408 | Objective | `PE` ↑ | Promotion improves: better candidates survive selection | Peak PE, Shoulder PE |
| RP-409 | Variation | `COR` ↑ | Better candidate production: more candidates in productive zones | Peak COR, Transition COR |
| RP-412 D4 | Construction | `IFR` ↑ | Better initial state: more feasible candidates at generation 0 | IFR (mean, per-instance) |
| Exec. Opt. | Execution | `N_eval` ↑ | Larger search budget: more evaluations within the time limit | evals/s, generation count |

None of these replaces the others. They are complementary interventions targeting different
stages of the same pipeline.

### 4.4 RP-408 and RP-409 Are Necessary but Not Sufficient

This is the most important implication of the diagnostic phase.

Suppose RP-408 improves comparator quality by 15%. If an instance only executes 2 generations,
the impact will be modest because there are very few selection events. Likewise RP-409 may
generate better offspring, but if only a few hundred candidates are ever evaluated, its effect
is bounded by the available search budget.

The expected impact of RP-408 and RP-409 will therefore be greatest on the faster instances
(setA-01 through setA-09, which run 10–105 generations) and smaller on the most
evaluation-limited instances (setA-17, setA-20 at 2 generations each). This does not make
them unimportant — it defines where they can realistically deliver gains and sets correct
expectations for the A/B experiments.

### 4.5 Post-RP-409 Priorities

After RP-409, the programme will have optimised construction diagnostics, variation, promotion,
operator behaviour, comparator behaviour, and execution diagnostics. At that point, the largest
remaining technical challenge is no longer evolutionary logic. It is the cost and quality of
evaluation and feasibility generation:

1. **Evaluation cost** — 99.99% of runtime is consumed by the evaluator. Incremental evaluation
   is the highest-leverage single intervention available.
2. **Construction quality** — mean IFR 10.6% means 89% of initial candidates are wasted
   evaluations. Repair-based construction is the second highest-leverage intervention.
3. **Constraint handling** — the RP-412 D4 failure taxonomy (per-constraint breakdown) is a
   prerequisite for targeted repair design.

These become the primary focus after RP-409 if the objective is to continue improving beyond
the current intervention scope.

### 4.6 Reusability

The EEB framework is independent of ROADEF. The abstraction

```
Construction → Variation → Promotion → Execution → Objective
```

combined with

```
EEB = Search Budget × Search Quality = N_eval × (IFR × COR × PE)
```

applies to any evolutionary optimiser for which analogous definitions of IFR, COR, and PE
can be derived from telemetry. Future Coralys domains (airline crew scheduling, nurse
rostering, CVRP, workforce optimisation) could be analysed using the same methodology by
redefining the domain-specific metrics while keeping the experimental framework unchanged.
This elevates the programme from a ROADEF benchmark study to a reusable research methodology
for evolutionary optimisation.

---

## 5. Programme Table

### Phase 1 — Scientific Foundation (COMPLETE, 2026-08-06)

All Phase 1 milestones are frozen. The scalar comparator is retained as the reference comparator. The lexicographic comparator module remains as validated experimental infrastructure but is not the default.

| Milestone   | Subsystem    | Primary Outcome                                                                 | Status    |
| ----------- | ------------ | ------------------------------------------------------------------------------- | --------- |
| **RP-406C** | All          | Benchmark characterisation; four-zone signature; two regimes                    | ✅ Frozen |
| **RP-407**  | Construction | Feasibility analysis; Type I/II failure framework                               | ✅ Frozen |
| **RP-410A** | Variation    | Search dynamics; operator fingerprints; zone behaviour                          | ✅ Frozen |
| **RP-410C** | Promotion    | Full survival funnel; Tournament/Population/Elite PE decomposition              | ✅ Frozen |
| **RP-411**  | Execution    | Eval=99.99% of runtime; throughput 1,555× spread; median 8.5 gens              | ✅ Frozen |
| **RP-412**  | Construction | Mean IFR=10.6%; 6/20 IFR=0%; 894 capacity violations                           | ✅ Frozen |
| **RP-408**  | Objective    | First lexicographic comparator design not adopted as default; scalar comparator retained as reference; improved comparator designs remain open research direction | ✅ Frozen |
| **RP-409A** | Variation    | Crossover dominates accepted improvements; mutation Peak ACR = 0               | ✅ Frozen |
| **RP-409B** | Variation    | PeakTargeted mutation hypothesis falsified; indirect population perturbation mechanism identified | ✅ Frozen |
| **RP-409C** | Promotion    | Full genealogy wired: parent IDs, tournament ID, population slot, elite slot, rejection reasons propagated through evaluation boundary; promotion funnel reconstructable from telemetry | ✅ Frozen |

### Phase 2 — Competition Optimisation (Active, from 2026-08-06)

Phase 2 operates on two parallel tracks. The sole success criterion for the Competition Track is: **does it lower the ROADEF objective under the official benchmark?** The Research Track deepens understanding of Coralys MOGA behaviour and makes future operator design evidence-driven.

#### Research Track

| Milestone    | Purpose                                    | Primary Question                                                                 | Status  |
| ------------ | ------------------------------------------ | -------------------------------------------------------------------------------- | ------- |
| **RP-409C**  | Promotion Pipeline Analysis                | Did mutation generate Peak offspring? Were they rejected at tournament, elite replacement, or objective comparison? | ✅ Frozen |
| **RP-409D**  | Selection Dynamics                         | How does tournament selection filter the population over generations?             | Planned |
| **RP-409E**  | Diversity Dynamics                         | How does population diversity evolve, and does it correlate with stagnation?     | Planned |

#### Competition Track

Each Competition Track milestone answers one question: does this lower the ROADEF objective? Reports follow the same A/B discipline as RP-409B.

##### Technology Readiness Levels (TRL) for Competition Milestones

Each RC milestone is tracked through six maturity stages. A milestone advances to the next stage only when the exit criterion for the current stage is met.

| TRL Stage      | Meaning                                                                 | Exit criterion                                      |
| -------------- | ----------------------------------------------------------------------- | --------------------------------------------------- |
| **Concept**    | Idea documented; hypothesis stated; target subsystem identified         | Design note written and reviewed                    |
| **Prototype**  | Implementation complete; compiles and runs against at least one instance | Passes smoke test on setA-01                       |
| **Benchmark**  | Full A/B campaign complete against CB-000 across all 20 Set A instances | Campaign report produced                            |
| **Accepted**   | A/B result shows statistically significant improvement over CB-000      | Report frozen; improvement confirmed                |
| **Integrated** | Component merged into the RS-001 integration branch                     | Integration build passes regression suite           |
| **Frozen**     | Locked for submission; no further changes permitted                     | RS-003 freeze confirmed                             |

##### Competition Track Milestone Table

| Milestone   | Purpose                          | Primary Question                                              | Depends on       | TRL Stage  |
| ----------- | -------------------------------- | ------------------------------------------------------------- | ---------------- | ---------- |
| **RC-001**  | Constructor improvement          | Does a better constructor lower the objective?                | CB-000           | Concept    |
| **RC-002**  | Repair heuristics                | Does targeted repair lower the objective?                     | CB-000           | Concept    |
| **RC-003**  | Large-neighbourhood destroy/repair | Does LNS lower the objective?                               | CB-000           | Concept    |
| **RC-004**  | Problem-specific crossover       | Does a ROADEF-aware crossover lower the objective?            | CB-000           | Concept    |
| **RC-005**  | Local search                     | Does a local search post-processor lower the objective?       | CB-000           | Concept    |
| **RC-006**  | Competition-aligned comparator redesign | Does a promotion-pipeline-informed comparator lower the objective? | RP-409C, RP-410C | Concept |
| **RC-007**  | Automated solver configuration          | Does systematic algorithm configuration lower the objective?  | CB-000           | Concept    |
| **RC-008**  | Component interaction study             | Do accepted RC components remain additive when combined?       | RC-001 through RC-007 | Concept |

**Note on RP-409C:** RP-409C is Research Track infrastructure, not a Competition Track milestone. Its purpose is to make Coralys easier to improve by exposing the promotion pipeline. It is not expected to directly lower the ROADEF objective.

**Note on parallel execution:** RP-409C and RC-001/RC-002 are designed to run in parallel. RC-001 through RC-005 and RC-007 are already justified by frozen Phase 1 evidence and do not depend on RP-409C. Only RC-006 (comparator redesign) must wait for RP-409C findings, because its design depends on promotion-pipeline evidence identifying where competition-aligned candidates are lost.

**RC-007 scope:** RC-007 is automatic algorithm configuration, not manual parameter tuning. It covers systematic search over all tunable solver parameters — population size, mutation rate, crossover rate, tournament size, elite size, stagnation limits, and any operator-specific parameters introduced by RC-001 through RC-006 — using established automatic configuration methods (e.g. irace, SMAC, Bayesian optimisation). It is distinct from RS-002 (which is a final submission tuning pass) because RC-007 is an A/B engineering milestone with quantitative comparison against CB-000, not a manual tuning exercise.

**RC-008 scope:** RC-008 answers the component interaction question: do individually validated RC improvements remain beneficial when combined? RC-001 may improve 2%, RC-002 may improve 3%, but their combined effect may be less than 5% (negative interaction), more than 5% (positive interaction), or exactly 5% (additive). RC-008 runs a systematic combination study across all accepted RC components before RS-001 integration, so that RS-001 can include only components that are beneficial in combination. This makes RS-001 largely mechanical rather than exploratory.

#### Competition Baseline (CB-000)

Every Competition Track milestone compares against CB-000, not just against the immediately previous RC. This ensures cumulative progress is quantifiable and no regression goes undetected.

| Component | Setting |
|---|---|
| Comparator | Scalar (ComparatorMode::Scalar) |
| Constructor | Original (single random initialisation) |
| Crossover | Original uniform crossover |
| Mutation | Original uniform mutation |
| Repair | None |
| LNS | None |
| Local Search | None |

CB-000 is permanently frozen. Every RC report must include a CB-000 vs. Current comparison in addition to any incremental comparison.

#### Submission Track

ROADEF evaluates a complete solver, not individual ideas. The Submission Track integrates validated RC components into a contest-ready artifact.

| Milestone | Purpose |
|---|---|
| **RS-001** | Assemble first competition solver from all validated RC components |
| **RS-002** | Final parameter tuning pass and ablation studies |
| **RS-003** | Final submission freeze and reproducibility validation |

**Submission assembly chain:**

```
CB-000
   │
   ├── RC-001  Constructor improvement          ✓ (when complete)
   ├── RC-002  Repair heuristics                ✓ (when complete)
   ├── RC-003  Large-neighbourhood search       ✓ (when complete)
   ├── RC-004  ROADEF-aware crossover           ✓ (when complete)
   ├── RC-005  Local search                     ✓ (when complete)
   ├── RC-006  Competition-aligned comparator   ✓ (when complete; requires RP-409C)
   ├── RC-007  Automated solver configuration   ✓ (when complete)
   └── RC-008  Component interaction study      ✓ (when complete; requires RC-001–RC-007)
        │
        ▼
RS-001  Integration — assemble components validated as beneficial in combination (RC-008)
        │
        ▼
RS-002  Parameter tuning — final tuning pass
        │
        ▼
RS-003  Freeze — final submission freeze and reproducibility validation
```

Every RC contribution is traceable. Ablation studies in RS-002 can remove any single RC component and measure the objective impact, making the contribution of each engineering milestone independently verifiable.

---

## 5b. Formal Pipeline Metrics

Three formal metrics separate generation behaviour from promotion behaviour and end-to-end
efficiency. These metrics apply to all subsequent pipeline experiments.

### Candidate Opportunity Rate (COR)

For zone Z:

```
COR(Z) = Generated candidates improving Z / Total generated candidates
```

Baseline values from RP-410B (v3 campaign):
- Peak COR = 133 / 14,600 = **0.91%**
- Shoulder COR = 1,023 / 14,600 = **7.01%**

COR measures what the variation operators produce. It is independent of selection and objective.
A change in COR indicates a change in operator behaviour (RP-409 target).

### Promotion Efficiency (PE)

For zone Z:

```
PE(Z) = Accepted candidates improving Z / Valid candidates improving Z
```

Baseline values from RP-410B (v3 campaign):
- Peak PE = 4 / 133 = **3.01%**
- Shoulder PE = 65 / 1,023 = **6.35%**

PE measures what survives the optimisation process. It is sensitive to the objective function
and the promotion mechanism. The 2.1× gap between Shoulder PE (6.35%) and Peak PE (3.01%)
is the primary quantitative target for RP-408.

**Note:** "Accepted" in the current telemetry means "became new global best." RP-410C will
extend this to the full decision path, enabling PE to be decomposed into:
- **Tournament PE:** `tournament_survived / valid`
- **Population PE:** `entered_population / tournament_survived`
- **Elite PE:** `elite_replacement / entered_population`
- **Global-best PE:** `became_global_best / entered_population`

### Overall Success Rate (OSR)

For zone Z:

```
OSR(Z) = Accepted candidates improving Z / Total generated candidates
       = COR(Z) × PE(Z)
```

Baseline values from RP-410B (v3 campaign):
- Peak OSR = 4 / 14,600 = **0.027%**
- Shoulder OSR = 65 / 14,600 = **0.445%**

OSR is the end-to-end success probability: given one generated candidate, what is the
probability it becomes an accepted improvement in zone Z? Because it is a ratio of raw
counts (accepted / generated), it has a direct probabilistic interpretation and is
comparable across experiments without unit ambiguity.

The 16.5× gap between Shoulder OSR (0.445%) and Peak OSR (0.027%) reflects the combined
effect of both the generation bottleneck (low Peak COR = 0.91%) and the promotion
bottleneck (low Peak PE = 3.01%). OSR is the single number that captures the full pipeline
cost of producing one Peak improvement.

---

## 6. Phase 2 Roadmap

Phase 1 is complete. The programme now operates on two parallel tracks. The Research Track deepens understanding of Coralys MOGA behaviour; the Competition Track improves the ROADEF objective directly. Both tracks use the same A/B experimental discipline established in RP-408 through RP-409B.

```
Phase 1 COMPLETE (2026-08-06)
RP-406C through RP-409C — all frozen
        │
        ▼
Phase 2 — Two Parallel Tracks (run simultaneously)

Research Track                    Competition Track
──────────────                    ─────────────────
RP-409C  ✅ FROZEN                RC-001 Constructor  ◄── START HERE (parallel)
Promotion Pipeline Analysis       RC-002 Repair        ◄── START HERE (parallel)
        │                         RC-007 Automated Solver Config (after RC-001/RC-002)
        ▼                         RC-003 LNS
RP-409D  ◄── START HERE           RC-004 Crossover
Selection Dynamics                RC-005 Local Search
        │                         RC-006 Competition-Aligned Comparator
        ▼                                │  (requires RP-409C ✅)
RP-409E                           RC-008 Component Interaction Study
Diversity Dynamics                       │  (requires RC-001–RC-007)
                                         ▼
                                  Submission Track
                                  RS-001 Integration
                                  RS-002 Tuning
                                  RS-003 Final Submission
```

**Research Track purpose:** Understand why Coralys MOGA behaves the way it does. Findings inform Competition Track design. Research milestones are not expected to directly improve the ROADEF objective, although the resulting insights may enable future Competition Track improvements.

**Competition Track purpose:** Lower the ROADEF objective under the official benchmark. Every RC milestone answers one question: does this lower the objective? Reports follow the same A/B discipline as RP-409B, comparing against CB-000.

**RC-006 note:** RC-006 (Competition-Aligned Comparator Redesign) is a new engineering milestone, not a continuation of RP-408. It builds on evidence from RP-408, RP-409C, and RP-410C. Its objective is to design an internal evolutionary comparator that better aligns search behaviour with the fixed ROADEF lexicographic evaluation. The design will be guided by promotion-pipeline evidence identifying where competition-aligned candidates are lost.

**Coralys Evolution Observatory:** The outputs of RP-409C are platform-level capabilities, not ROADEF-specific instrumentation. Candidate IDs, genealogy tracking, tournament outcome recording, elite replacement logging, population admission events, and rejection reasons are generic evolutionary telemetry that applies to any Coralys domain. RP-409C is now frozen; the promotion pipeline instrumentation is a reusable component of the Coralys Evolution Observatory — the same infrastructure that supports UltraCrew, CVRP, workforce scheduling, and other optimisation domains. This elevates RP-409C from a ROADEF experiment to a foundational platform milestone.

**Execution priority (updated 2026-08-06):** The recommended execution sequence for the Competition Track is:

1. **RC-001** — Constructor improvement. Mean IFR = 10.6% is the dominant EEB bottleneck; improving construction raises the effective search budget before any variation changes.
2. **RC-002** — Repair heuristics. Complements RC-001 by recovering candidates that would otherwise be discarded; addresses the same feasibility bottleneck from a different angle.
3. **RC-007** — Automated solver configuration. Systematic algorithm configuration is more valuable on a stronger baseline (post RC-001/RC-002) than on the current solver.
4. **RC-003 / RC-004 / RC-005** — LNS, ROADEF-aware crossover, local search. Evaluated independently against the strengthened baseline; can proceed in parallel.
5. **RC-006** — Competition-aligned comparator redesign. Now fully informed by RP-408, RP-410C, and RP-409C evidence; can target observed promotion bottlenecks rather than relying on a priori assumptions.
6. **RC-008** — Component interaction study. Validates that individually accepted RC components remain beneficial in combination before RS-001 integration. Prevents RS-001 from becoming exploratory.

RP-409D and RP-409E can proceed alongside this work without blocking any engineering milestone.

---

## 7. Research Governance

> **No new optimisation feature may be merged unless its intended subsystem is identified (Construction, Execution, Variation, Promotion, or Objective) and its success is evaluated against the frozen baseline metrics for that subsystem.**

This prevents future changes from becoming "black-box improvements." Every modification has a clear hypothesis, a target subsystem, and quantitative evidence showing whether it achieved its intended effect.

**Competition Track governance:** Every RC milestone must include a CB-000 vs. Current comparison. Incremental comparisons (RC-N vs. RC-N-1) are permitted as supplementary evidence but do not replace the CB-000 comparison.

**EEB subsystem mapping (mandatory for every RC report):** Every RC report must state which subsystem the intervention primarily targets and identify the EEB term(s) expected to change. This preserves the same causal discipline established for the RP-series.

| Milestone | Subsystem             | Expected EEB effect                                                              |
| --------- | --------------------- | -------------------------------------------------------------------------------- |
| RC-001    | Construction          | IFR ↑                                                                            |
| RC-002    | Construction          | IFR ↑                                                                            |
| RC-003    | Variation             | COR ↑                                                                            |
| RC-004    | Variation             | COR ↑                                                                            |
| RC-005    | Variation / Promotion | COR ↑ and/or PE ↑                                                                |
| RC-006    | Objective / Promotion | PE ↑                                                                             |
| RC-007    | Cross-cutting         | Amplifies existing improvements; no single EEB term targeted                     |
| RC-008    | Integration           | Validation only; no direct EEB target                                            |

**Comparator governance:** The scalar comparator (`ComparatorMode::Scalar`) is the reference implementation. Any future comparator design must be tested as an A/B experiment against the scalar reference. The ROADEF competition evaluation objective (lexicographic ranking) is fixed and cannot be changed; the internal evolutionary comparator is an implementation choice that may be improved. Comparator changes are judged by their ability to improve the official ROADEF objective, not by increasing agreement with lexicographic ordering in isolation.

---

## 8. Publication Structure

The research programme clusters into publishable themes:

| Paper | Theme | Milestones |
|-------|-------|------------|
| 1 | Benchmark Characterisation | RP-406C |
| 2 | Feasibility Analysis | RP-407, RP-412 |
| 3 | Search Dynamics | RP-410A, RP-410C |
| 4 | Execution Scaling | RP-411 |
| 5 | Comparator Design for Lexicographic Benchmarks | RP-408 |
| 6 | Operator Attribution and Redesign | RP-409A, RP-409B |
| 7 | Promotion Pipeline Analysis | RP-409C, RP-409D, RP-409E |
| 8 | Competition Engineering: Constructor, Repair, and Local Search | RC-001, RC-002, RC-005 |
| 9 | Competition Engineering: Neighbourhood Search and Crossover | RC-003, RC-004 |
| 10 | Automatic Algorithm Configuration for Evolutionary Solvers | RC-007 |
| 11 | Competition-Aligned Comparator Design | RC-006 |
| 12 | Coralys ROADEF Solver Architecture | RC-008, RS-001 |
| 13 | Component Ablation Study | RS-002, RS-003 |

Papers 1–4 are descriptive/diagnostic; Papers 5–6 are interventional; Paper 7 is mechanistic; Papers 8–11 are engineering papers; Papers 12–13 are competition papers. The diagnostic papers establish the causal baseline that makes the interventional papers falsifiable. Paper 12 describes the solver architecture and the component interaction evidence (RC-008) that justifies the integration choices. Paper 13 is a standalone ablation study demonstrating the independent contribution of each RC component — these are typically distinct contributions in the solver engineering literature.

---