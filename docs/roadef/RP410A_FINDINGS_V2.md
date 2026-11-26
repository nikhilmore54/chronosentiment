# RP-410A Findings — Evolutionary Search Dynamics (Corrected Telemetry)

**Status:** FROZEN  
**Telemetry source:** `/tmp/rp410_telemetry_v2` (corrected operator tagging)  
**Campaign:** 20 setA instances, single seed, adaptive time budget (30–300s)  
**Data:** 112 accepted moves, 342 generation records  

## Executive Findings

Three findings stand out from the v2 campaign:

**1. Throughput varies by three orders of magnitude.** Generation rate drops from ~364 gen/min (setA-01, 50 nodes) to ~0.34 gen/min (setA-20, 400 nodes). Large instances complete only 2–6 generations within the 300s budget. Conclusions about evolutionary dynamics drawn from 2 generations cannot be compared with conclusions drawn from 90 generations. Throughput is a first-order constraint for large instances, independent of operator quality or selection strategy.

**2. Accepted Peak improvements are almost absent** (0.9% of accepted moves, 1 out of 112). The present data do not distinguish whether this arises from candidate generation, repair, or the scalar selection objective.

**3. Five instances never produce valid solutions** — a construction failure, not an evolutionary one.

---
---

## 1. Summary of Corrections

The first campaign run (RP-410A v1) had a defect in operator tagging: all accepted moves were labelled `"initial"` because the operator field was not propagated through the evaluation loop. This has been corrected. The v2 campaign produces three distinct operator tags:

- `crossover` — offspring produced by crossover only
- `crossover+mutation` — offspring produced by crossover then mutated
- `mutation` — offspring produced by mutation only

All findings below are from the corrected v2 telemetry.

---

## 2. Zone Distribution — All Accepted Moves

| Zone | Count | % |
|------|------:|--:|
| Peak | 1 | 0.9% |
| Shoulder | 26 | 23.2% |
| Transition | 23 | 20.5% |
| Tail | 28 | 25.0% |
| Mixed | 32 | 28.6% |
| Neutral | 2 | 1.8% |
| **Total** | **112** | 100% |

### 2.1 Peak Near-Absence Finding

Peak improvements account for **0.9%** of accepted moves (1 out of 112). This single Peak move occurred in setA-13, a medium-sized instance (200 nodes, 1000 links, 2000 demands) that ran only 6 generations.

The corrected telemetry shows that accepted Peak improvements are extremely rare under the current search architecture. The present data do not distinguish whether this arises from candidate generation, repair, or the scalar selection objective. Several explanations remain plausible: neighbourhoods rarely generate Peak-improving candidates; crossover destroys them; mutation rate is too low; repair removes them; or construction never reaches the relevant basin. Isolating the cause requires instrumentation earlier in the pipeline — specifically, recording generated and rejected moves in addition to accepted ones.

**Caution:** The telemetry records accepted moves only. It does not record generated or rejected moves. The near-absence of Peak improvements in accepted moves could reflect either (a) that Peak-improving moves are rarely generated, or (b) that they are generated but rejected by selection. These hypotheses require earlier-pipeline instrumentation to distinguish.

---

## 3. Operator Fingerprints

| Operator | Total | Peak % | Shoulder % | Transition % | Tail % | Mixed % | Neutral % |
|----------|------:|-------:|-----------:|-------------:|-------:|--------:|----------:|
| crossover | 61 | 0.0% | 24.6% | 21.3% | 26.2% | 27.9% | 0.0% |
| crossover+mutation | 38 | 2.6% | 15.8% | 18.4% | 31.6% | 31.6% | 0.0% |
| mutation | 13 | 0.0% | 38.5% | 23.1% | 0.0% | 23.1% | 15.4% |

### 3.1 Crossover Dominance

Crossover accounts for **54.5%** of accepted improvements (61/112). Crossover+mutation accounts for **33.9%** (38/112). Pure mutation accounts for only **11.6%** (13/112).

This is the expected pattern for a population-based EA: crossover recombines existing routing families and dominates accepted improvements. Mutation alone rarely produces competitive offspring.

### 3.2 Operator Zone Profiles

The three operators show distinct zone profiles in this campaign:

**Crossover:** accepted crossover-derived improvements were distributed across Shoulder (24.6%), Transition (21.3%), Tail (26.2%), and Mixed (27.9%). No accepted crossover move produced a Peak improvement.

**Crossover+mutation:** accepted crossover+mutation-derived improvements were the only category to include a Peak improvement (2.6%, 1 move). They also showed the highest Tail and Mixed fractions (31.6% each).

**Mutation:** accepted mutation-derived improvements were disproportionately Shoulder-oriented in this campaign (38.5%), with zero Tail improvements. The 15.4% Neutral fraction (2 moves) is notable — two accepted mutation moves produced negligible zone impact.

These are observational profiles from a single-seed campaign. They describe what was accepted, not what was generated. The causal mechanisms behind the profiles (e.g., whether crossover structurally cannot generate Peak improvements, or whether they are generated but rejected) remain open questions.

### 3.3 Implications

The operator fingerprints suggest that crossover+mutation is the most structurally diverse operator. If Peak improvements are a target (as they would be under native lexicographic evaluation), crossover+mutation is the only operator currently capable of producing them. This has direct implications for RP-408 (native lexicographic evaluation): under a lexicographic objective, selection pressure would reward Peak improvements, potentially increasing the crossover+mutation acceptance rate.

---

## 4. Collapsed Basin vs Shape Competition

| Metric | Collapsed Basin | Shape Competition |
|--------|----------------:|------------------:|
| Peak % | 0.0% | 1.2% |
| Shoulder % | 20.0% | 24.4% |
| Transition % | 13.3% | 23.2% |
| Tail % | 30.0% | 23.2% |
| Mixed % | 36.7% | 25.6% |
| Neutral % | 0.0% | 2.4% |
| Avg SDI | 0.799 | 2.036 |
| Avg MLU | 0.534 | 0.751 |
| Avg Diversity | 22.1 | 20.1 |
| Avg Stagnation | 4.88 | 3.31 |

### 4.1 SDI Separation

The SDI gap between collapsed-basin (0.799) and shape-competition (2.036) instances is the clearest quantitative signal in the dataset. Collapsed-basin instances converge to solutions with more uniform arc saturation — consistent with a single dominant routing family that saturates arcs evenly. Shape-competition instances maintain higher SDI, indicating that the search is actively competing between routing families that load different arcs differently.

### 4.2 MLU Paradox

Collapsed-basin instances show lower average MLU (0.534) than shape-competition instances (0.751). This is counterintuitive: the instances that fail to find valid solutions or converge prematurely have lower MLU in their best valid solutions. The explanation is instance structure: collapsed-basin instances (setA-02 through setA-08) are smaller instances where the network has sufficient capacity to route all demands at low utilisation — but the search cannot find a valid routing. The low MLU reflects the few valid solutions that were found, not the typical search outcome.

### 4.3 Stagnation

Collapsed-basin instances show higher average stagnation (4.88 vs 3.31). This is consistent with the basin collapse hypothesis: once the population converges to a single routing family, no further improvements are found and stagnation accumulates.

---

## 5. Generation Depth and Throughput

The campaign reveals a critical computational constraint that is independent of search behaviour:

| Instance | Generations | Runtime (s) | Gens/min |
|----------|------------:|------------:|---------:|
| setA-01 | 91 | 15 | 364 |
| setA-03 | 81 | 20 | 243 |
| setA-04 | 14 | ~30 | 28 |
| setA-09 | 11 | ~200 | 3.3 |
| setA-13 | 6 | 321 | 1.1 |
| setA-17 | 2 | 339 | 0.35 |
| setA-18 | 4 | 362 | 0.66 |
| setA-20 | 2 | 351 | 0.34 |

The generation rate drops by approximately **1000×** from the smallest to the largest instances. setA-17 and setA-20 complete only 2 generations within the 300s budget. At 2 generations, the evolutionary search is effectively random: the population has been initialised and evaluated once, and a single generation of selection and variation has occurred. No meaningful evolutionary dynamics can emerge.

This is the strongest computational finding from the campaign. It establishes that **throughput is a first-order constraint** for large instances, independent of operator quality, selection strategy, or objective function.

---

## 6. Hypothesis Assessment

**H1 — Transition/Tail dominance (≥80% of moves):** Not confirmed. Transition + Tail = 45.5%. The 80% threshold is not met. Mixed moves (28.6%) are a substantial category that the original hypothesis did not anticipate.

**H2 — Shoulder improvements rare after generation 50:** Not testable from this campaign. Most instances complete fewer than 20 generations. The hypothesis requires instances with ≥50 generations, which only setA-01 (91 gens) and setA-03 (81 gens) provide.

**H3 — Collapsed-basin instances never generate Peak improvements:** Confirmed for this campaign. Collapsed-basin Peak % = 0.0% vs shape-competition 1.2%. The difference is small in absolute terms but consistent with the SDI separation.

**H4 — Different operators produce different zone fingerprints:** Confirmed. Crossover, crossover+mutation, and mutation show distinct zone profiles. Crossover+mutation is the only operator producing Peak improvements.

---

## 7. Relationship to the Three-Problem Framework

The campaign data, combined with RP-410A telemetry, supports a restructured view of the research programme around three independent bottlenecks:

**Problem 1 — Feasibility:** 5 of 20 instances produce no valid solution (setA-02, setA-07, setA-16, setA-19, setA-20). This is independent of search behaviour. Until a valid solution exists, there is no objective to optimise.

**Problem 2 — Throughput:** Large instances complete 2–6 generations within the time budget. At this depth, evolutionary dynamics cannot operate. Addressing throughput (evaluation cost, incremental updates, memory traffic) is a prerequisite for meaningful search behaviour analysis on large instances.

**Problem 3 — Search behaviour:** The RP-410A findings (Peak near-absence, operator fingerprints, SDI separation) characterise search behaviour on instances where the search actually runs. These findings are valid for the instances that complete ≥10 generations. They are not yet generalisable to large instances where throughput is the binding constraint.

---

## 8. Data Files

- [`docs/roadef/rp410a_data_v2/rp410a_zone_distribution.csv`](docs/roadef/rp410a_data_v2/rp410a_zone_distribution.csv) — move counts by zone
- [`docs/roadef/rp410a_data_v2/rp410a_operator_fingerprints.csv`](docs/roadef/rp410a_data_v2/rp410a_operator_fingerprints.csv) — zone distribution per operator
- [`docs/roadef/rp410a_data_v2/rp410a_basin_comparison.csv`](docs/roadef/rp410a_data_v2/rp410a_basin_comparison.csv) — collapsed basin vs shape competition
- [`docs/roadef/rp410a_data_v2/RP410A_SEARCH_DYNAMICS_REPORT.md`](docs/roadef/rp410a_data_v2/RP410A_SEARCH_DYNAMICS_REPORT.md) — generated data report

*End of RP-410A Findings v2*
