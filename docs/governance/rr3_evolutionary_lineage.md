# RR3 — Evolutionary Lineage Analysis

**Programme:** Repository Rationalization  
**Phase:** RR3 — Historical Evidence / Evolutionary Lineage Analysis  
**Status:** Complete  
**Produced:** 2026-08-02  
**Input:** `docs/governance/rr2a5_executable_inventory.csv` (Experiment rows)  
**Outputs:**
  - `docs/governance/rr3_lineage.csv` — tabular lineage record (one row per binary)
  - `docs/governance/rr3_lineage_graph.md` — Experiment Lineage Graph (DAG)
  - This document — narrative analysis

---

## 1. Scope

RR3 reconstructs the evolutionary history of the 64 experiment binaries
identified by RR2-A.5. These binaries represent the primary research footprint
of the repository: 16,769 LOC of experimental code across five packages.

The analysis answers four questions for each binary:
1. When was it introduced and when was it last modified?
2. What hypothesis or goal did it evaluate?
3. Which binary preceded it (parent) and which followed it (successors)?
4. What was its outcome — did it produce a successor, or is it terminal?

---

## 2. Methodology

**Generation numbering.** Experiment binaries follow a consistent naming
convention: `m{N}[letter][_variant]`. The numeric prefix `N` encodes the
generation. Sub-generations use letter suffixes (m22a, m22b, m22c) and
variant indices (m22c_0, m22c_1, m22c_2). Non-generation binaries (those
without an `m` prefix) are treated as infrastructure or utility experiments.

**Parent inference.** Within each cluster, the parent of generation N is
the binary with the largest generation number less than N. For sub-generations,
the parent is the base generation (m22a → m22). For variants, the parent is
the previous variant index (m22c_1 → m22c_0).

**Outcome inference.** A binary is classified as Superseded if it has at
least one successor in the same cluster. It is Terminal if no successor exists.
Terminal status does not imply failure — the most recent generation in a
research stream is always terminal by definition.

**Evidence level.** All experiment binaries are classified E3 (no external
caller imports their symbols). They are reachable from compilation roots
(each has its own `main()`) but are not called by any library code.

---

## 3. Cluster Summaries

### 3.1 CVRP Research Stream

| Field | Value |
|-------|-------|
| Packages | cvrp, cvrp_server |
| Experiment binaries | 32 |
| Total LOC | 8,107 |
| Generation range | m8–m30 |
| Active period | 2026-07-06 → 2026-07-07 |
| Superseded | 11 |
| Terminal | 21 |

**Terminal binaries (most recent in their lineage):**

- `basin_characterization` — Basin of attraction characterisation (180 LOC, last active 2026-07-06)
- `compare` — Comparative analysis (52 LOC, last active 2026-07-06)
- `elite_manifold_probe` — Elite manifold probing (95 LOC, last active 2026-07-06)
- `elite_partition_probe` — Partition probing (149 LOC, last active 2026-07-06)
- `find_797` — Search / discovery experiment (38 LOC, last active 2026-07-06)
- `frozen_partition_probe` — Partition probing (160 LOC, last active 2026-07-06)
- `initial_basin_distribution` — Basin of attraction characterisation (71 LOC, last active 2026-07-06)
- `m21c_sa` — Simulated annealing (246 LOC, last active 2026-07-06)
- `m21d_elite_sa` — Simulated annealing (215 LOC, last active 2026-07-06)
- `m22_scale_up` — Scale-up experiment (670 LOC, last active 2026-07-06)
- `m22a_annealed_elite` — Simulated annealing variant (437 LOC, last active 2026-07-06)
- `m22b_novelty_elitism` — Novelty-elitism hybrid (371 LOC, last active 2026-07-06)
- `m22c_0_route_sa` — Simulated annealing (230 LOC, last active 2026-07-06)
- `m22c_1_memetic` — Memetic algorithm variant (584 LOC, last active 2026-07-06)
- `m22c_2_fidelity_audit` — Structural or behavioural audit (657 LOC, last active 2026-07-06)
- `m22f_cvrp_control` — Control / baseline experiment (150 LOC, last active 2026-07-06)
- `m30_2_active_pilot` — Active pilot / live deployment (216 LOC, last active 2026-07-06)
- `m30_2a_1_ecology_audit` — Structural or behavioural audit (139 LOC, last active 2026-07-06)
- `m30_2a_2_shadow_advisory` — Shadow advisory mode (169 LOC, last active 2026-07-06)
- `search_config` — Search configuration study (122 LOC, last active 2026-07-07)
- `seed_ecology_study` — Ecology / fitness landscape characterisation (276 LOC, last active 2026-07-06)

**Infrastructure / utility experiments (no generation prefix):**

- `basin_characterization` — Basin of attraction characterisation (180 LOC)
- `compare` — Comparative analysis (52 LOC)
- `elite_manifold_probe` — Elite manifold probing (95 LOC)
- `elite_partition_probe` — Partition probing (149 LOC)
- `find_797` — Search / discovery experiment (38 LOC)
- `frozen_partition_probe` — Partition probing (160 LOC)
- `initial_basin_distribution` — Basin of attraction characterisation (71 LOC)
- `search_config` — Search configuration study (122 LOC)
- `seed_ecology_study` — Ecology / fitness landscape characterisation (276 LOC)

---

### 3.2 ULTRACREW / INRC Research Stream

| Field | Value |
|-------|-------|
| Packages | ultracrew, ultracrew_server |
| Experiment binaries | 18 |
| Total LOC | 5,093 |
| Generation range | m8–m31 |
| Active period | 2026-06-04 → 2026-07-07 |
| Superseded | 4 |
| Terminal | 14 |

**Terminal binaries (most recent in their lineage):**

- `config_sweep` — Configuration sweep (271 LOC, last active 2026-07-07)
- `inrc_ecology_ablation_matrix` — Ablation study — component isolation (353 LOC, last active 2026-07-07)
- `inrc_ecology_cost_curve` — Ecology / fitness landscape characterisation (348 LOC, last active 2026-07-07)
- `inrc_ecology_memory_depth` — Ecology / fitness landscape characterisation (341 LOC, last active 2026-07-07)
- `inrc_ecology_multi_week_ablation` — Ablation study — component isolation (451 LOC, last active 2026-07-07)
- `inrc_ecology_response_curve` — Ecology / fitness landscape characterisation (347 LOC, last active 2026-07-07)
- `inrc_m22_ancestry` — Ancestry / lineage tracing (321 LOC, last active 2026-07-07)
- `inrc_natural_history_pilot` — Natural history pilot (347 LOC, last active 2026-07-07)
- `m30_0d_active_pilot` — Active pilot / live deployment (221 LOC, last active 2026-07-07)
- `m31_2a_engagement_audit` — Structural or behavioural audit (260 LOC, last active 2026-07-07)
- `m31_benchmarks` — Benchmark / performance measurement (463 LOC, last active 2026-07-07)
- `m8g_ultracrew_validation` — Validation against reference (156 LOC, last active 2026-07-06)
- `policy_seed_runner` — Seed / initialisation study (347 LOC, last active 2026-07-06)
- `story1` — Narrative / demonstration run (97 LOC, last active 2026-07-07)

**Infrastructure / utility experiments (no generation prefix):**

- `config_sweep` — Configuration sweep (271 LOC)
- `inrc_ecology_ablation_matrix` — Ablation study — component isolation (353 LOC)
- `inrc_ecology_cost_curve` — Ecology / fitness landscape characterisation (348 LOC)
- `inrc_ecology_memory_depth` — Ecology / fitness landscape characterisation (341 LOC)
- `inrc_ecology_multi_week_ablation` — Ablation study — component isolation (451 LOC)
- `inrc_ecology_response_curve` — Ecology / fitness landscape characterisation (347 LOC)
- `inrc_m22_ancestry` — Ancestry / lineage tracing (321 LOC)
- `inrc_natural_history_pilot` — Natural history pilot (347 LOC)
- `policy_seed_runner` — Seed / initialisation study (347 LOC)
- `story1` — Narrative / demonstration run (97 LOC)

---

### 3.3 ROADEF Research Stream

| Field | Value |
|-------|-------|
| Packages | roadef |
| Experiment binaries | 14 |
| Total LOC | 3,569 |
| Generation range | m25–m27 |
| Active period | 2026-06-14 → 2026-07-11 |
| Superseded | 2 |
| Terminal | 12 |

**Terminal binaries (most recent in their lineage):**

- `eval_profiler` — Evaluation profiler (125 LOC, last active 2026-07-11)
- `m25_8b_ecology` — Ecology / fitness landscape characterisation (358 LOC, last active 2026-07-06)
- `m25_benchmark` — Benchmark / performance measurement (377 LOC, last active 2026-07-06)
- `m25_final` — General experiment (284 LOC, last active 2026-07-06)
- `m26_1c_discriminative_audit` — Structural or behavioural audit (233 LOC, last active 2026-07-06)
- `m26_1d_failure_density` — Failure density analysis (171 LOC, last active 2026-07-06)
- `m26_1e_survival_curves` — Survival curve analysis (150 LOC, last active 2026-07-06)
- `m26_3_passive_learner` — Passive observation / telemetry (240 LOC, last active 2026-07-06)
- `m26_4a_shadow_advisory` — Shadow advisory mode (303 LOC, last active 2026-07-06)
- `m26_4b_active_pilot` — Active pilot / live deployment (307 LOC, last active 2026-07-06)
- `m27_1_passive_operator_telemetry` — Passive observation / telemetry (270 LOC, last active 2026-06-14)
- `tiny_solver` — Minimal / smoke-test solver (259 LOC, last active 2026-07-06)

**Infrastructure / utility experiments (no generation prefix):**

- `eval_profiler` — Evaluation profiler (125 LOC)
- `tiny_solver` — Minimal / smoke-test solver (259 LOC)

---

## 4. Full Lineage Table

See `docs/governance/rr3_lineage.csv` for the machine-readable record.
The table below shows the key fields for each binary.

| Binary | Cluster | Gen | LOC | First Commit | Last Active | Parent | Outcome |
|--------|---------|-----|-----|-------------|-------------|--------|---------|
| `m8g_cvrp_validation` | CVRP | m8g | 286 | 2026-07-06 | 2026-07-06 | `—` | Superseded |
| `m11_reachability_atlas` | CVRP | m11 | 288 | 2026-07-06 | 2026-07-06 | `m8g_cvrp_validation` | Superseded |
| `m12_cvrp_repair_atlas` | CVRP | m12 | 289 | 2026-07-06 | 2026-07-06 | `m11_reachability_atlas` | Superseded |
| `m14_cvrp_recognizability_audit` | CVRP | m14 | 317 | 2026-07-06 | 2026-07-06 | `m12_cvrp_repair_atlas` | Superseded |
| `m15_cvrp_decoder_independence` | CVRP | m15 | 222 | 2026-07-06 | 2026-07-06 | `m14_cvrp_recognizability_audit` | Superseded |
| `m16_cvrp_reconstruction_guidance` | CVRP | m16 | 271 | 2026-07-06 | 2026-07-06 | `m15_cvrp_decoder_independence` | Superseded |
| `m17_cvrp_backbone_causality` | CVRP | m17 | 238 | 2026-07-06 | 2026-07-06 | `m16_cvrp_reconstruction_guidance` | Superseded |
| `m18_structural_invariants` | CVRP | m18 | 224 | 2026-07-06 | 2026-07-06 | `m17_cvrp_backbone_causality` | Superseded |
| `m20_epistasis` | CVRP | m20 | 132 | 2026-07-06 | 2026-07-06 | `m18_structural_invariants` | Superseded |
| `m21_feasible_path` | CVRP | m21 | 257 | 2026-07-06 | 2026-07-06 | `m20_epistasis` | Superseded |
| `m21c_sa` | CVRP | m21c | 246 | 2026-07-06 | 2026-07-06 | `m21_feasible_path` | Terminal (no known successor) |
| `m21d_elite_sa` | CVRP | m21d | 215 | 2026-07-06 | 2026-07-06 | `m21_feasible_path` | Terminal (no known successor) |
| `m22_0_archive` | CVRP | m22_0 | 356 | 2026-07-06 | 2026-07-06 | `m21_feasible_path` | Superseded |
| `m22_scale_up` | CVRP | m22 | 670 | 2026-07-06 | 2026-07-06 | `m21_feasible_path` | Terminal (no known successor) |
| `m22a_annealed_elite` | CVRP | m22a | 437 | 2026-07-06 | 2026-07-06 | `m22_0_archive` | Terminal (no known successor) |
| `m22b_novelty_elitism` | CVRP | m22b | 371 | 2026-07-06 | 2026-07-06 | `m22_0_archive` | Terminal (no known successor) |
| `m22c_0_route_sa` | CVRP | m22c_0 | 230 | 2026-07-06 | 2026-07-06 | `m22_0_archive` | Terminal (no known successor) |
| `m22c_1_memetic` | CVRP | m22c_1 | 584 | 2026-07-06 | 2026-07-06 | `m22_0_archive` | Terminal (no known successor) |
| `m22c_2_fidelity_audit` | CVRP | m22c_2 | 657 | 2026-07-06 | 2026-07-06 | `m22_0_archive` | Terminal (no known successor) |
| `m22f_cvrp_control` | CVRP | m22f | 150 | 2026-07-06 | 2026-07-06 | `m22_0_archive` | Terminal (no known successor) |
| `m30_2_active_pilot` | CVRP | m30_2 | 216 | 2026-07-06 | 2026-07-06 | `m22_0_archive` | Terminal (no known successor) |
| `m30_2a_1_ecology_audit` | CVRP | m30_2 | 139 | 2026-07-06 | 2026-07-06 | `m22_0_archive` | Terminal (no known successor) |
| `m30_2a_2_shadow_advisory` | CVRP | m30_2 | 169 | 2026-07-06 | 2026-07-06 | `m22_0_archive` | Terminal (no known successor) |
| `basin_characterization` | CVRP | — | 180 | 2026-07-06 | 2026-07-06 | `—` | Terminal (no known successor) |
| `compare` | CVRP | — | 52 | 2026-07-06 | 2026-07-06 | `—` | Terminal (no known successor) |
| `elite_manifold_probe` | CVRP | — | 95 | 2026-07-06 | 2026-07-06 | `—` | Terminal (no known successor) |
| `elite_partition_probe` | CVRP | — | 149 | 2026-07-06 | 2026-07-06 | `—` | Terminal (no known successor) |
| `find_797` | CVRP | — | 38 | 2026-07-06 | 2026-07-06 | `—` | Terminal (no known successor) |
| `frozen_partition_probe` | CVRP | — | 160 | 2026-07-06 | 2026-07-06 | `—` | Terminal (no known successor) |
| `initial_basin_distribution` | CVRP | — | 71 | 2026-07-06 | 2026-07-06 | `—` | Terminal (no known successor) |
| `search_config` | CVRP | — | 122 | 2026-07-06 | 2026-07-07 | `—` | Terminal (no known successor) |
| `seed_ecology_study` | CVRP | — | 276 | 2026-07-06 | 2026-07-06 | `—` | Terminal (no known successor) |
| `m25_8_bridge` | ROADEF | m25_8 | 338 | 2026-07-06 | 2026-07-06 | `—` | Superseded |
| `m25_8b_ecology` | ROADEF | m25_8 | 358 | 2026-07-06 | 2026-07-06 | `—` | Terminal (no known successor) |
| `m25_benchmark` | ROADEF | m25 | 377 | 2026-07-06 | 2026-07-06 | `—` | Terminal (no known successor) |
| `m25_final` | ROADEF | m25 | 284 | 2026-07-06 | 2026-07-06 | `—` | Terminal (no known successor) |
| `m26_1_observation_audit` | ROADEF | m26_1 | 154 | 2026-07-06 | 2026-07-06 | `m25_8_bridge` | Superseded |
| `m26_1c_discriminative_audit` | ROADEF | m26_1 | 233 | 2026-07-06 | 2026-07-06 | `m25_8_bridge` | Terminal (no known successor) |
| `m26_1d_failure_density` | ROADEF | m26_1 | 171 | 2026-07-06 | 2026-07-06 | `m25_8_bridge` | Terminal (no known successor) |
| `m26_1e_survival_curves` | ROADEF | m26_1 | 150 | 2026-07-06 | 2026-07-06 | `m25_8_bridge` | Terminal (no known successor) |
| `m26_3_passive_learner` | ROADEF | m26_3 | 240 | 2026-06-14 | 2026-07-06 | `m25_8_bridge` | Terminal (no known successor) |
| `m26_4a_shadow_advisory` | ROADEF | m26_4 | 303 | 2026-06-14 | 2026-07-06 | `m25_8_bridge` | Terminal (no known successor) |
| `m26_4b_active_pilot` | ROADEF | m26_4 | 307 | 2026-06-14 | 2026-07-06 | `m25_8_bridge` | Terminal (no known successor) |
| `m27_1_passive_operator_telemetry` | ROADEF | m27_1 | 270 | 2026-06-14 | 2026-06-14 | `m26_1_observation_audit` | Terminal (no known successor) |
| `eval_profiler` | ROADEF | — | 125 | 2026-07-11 | 2026-07-11 | `—` | Terminal (no known successor) |
| `tiny_solver` | ROADEF | — | 259 | 2026-07-06 | 2026-07-06 | `—` | Terminal (no known successor) |
| `m8g_cs_validation` | ULTRACREW | m8g | 110 | 2026-07-06 | 2026-07-06 | `—` | Superseded |
| `m8g_ultracrew_validation` | ULTRACREW | m8g | 156 | 2026-07-06 | 2026-07-06 | `—` | Terminal (no known successor) |
| `m9a_search_observatory` | ULTRACREW | m9a | 316 | 2026-07-06 | 2026-07-06 | `m8g_cs_validation` | Superseded |
| `m23a_synthetic` | ULTRACREW | m23a | 203 | 2026-07-07 | 2026-07-07 | `m9a_search_observatory` | Superseded |
| `m30_0b_passive_telemetry` | ULTRACREW | m30_0 | 141 | 2026-07-07 | 2026-07-07 | `m23a_synthetic` | Superseded |
| `m30_0d_active_pilot` | ULTRACREW | m30_0 | 221 | 2026-07-06 | 2026-07-07 | `m23a_synthetic` | Terminal (no known successor) |
| `m31_2a_engagement_audit` | ULTRACREW | m31_2 | 260 | 2026-07-07 | 2026-07-07 | `m30_0b_passive_telemetry` | Terminal (no known successor) |
| `m31_benchmarks` | ULTRACREW | m31 | 463 | 2026-07-07 | 2026-07-07 | `m30_0b_passive_telemetry` | Terminal (no known successor) |
| `config_sweep` | ULTRACREW | — | 271 | 2026-07-07 | 2026-07-07 | `—` | Terminal (no known successor) |
| `inrc_ecology_ablation_matrix` | ULTRACREW | — | 353 | 2026-07-07 | 2026-07-07 | `—` | Terminal (no known successor) |
| `inrc_ecology_cost_curve` | ULTRACREW | — | 348 | 2026-07-07 | 2026-07-07 | `—` | Terminal (no known successor) |
| `inrc_ecology_memory_depth` | ULTRACREW | — | 341 | 2026-07-07 | 2026-07-07 | `—` | Terminal (no known successor) |
| `inrc_ecology_multi_week_ablation` | ULTRACREW | — | 451 | 2026-06-04 | 2026-07-07 | `—` | Terminal (no known successor) |
| `inrc_ecology_response_curve` | ULTRACREW | — | 347 | 2026-07-07 | 2026-07-07 | `—` | Terminal (no known successor) |
| `inrc_m22_ancestry` | ULTRACREW | — | 321 | 2026-07-07 | 2026-07-07 | `—` | Terminal (no known successor) |
| `inrc_natural_history_pilot` | ULTRACREW | — | 347 | 2026-07-07 | 2026-07-07 | `—` | Terminal (no known successor) |
| `policy_seed_runner` | ULTRACREW | — | 347 | 2026-07-06 | 2026-07-06 | `—` | Terminal (no known successor) |
| `story1` | ULTRACREW | — | 97 | 2026-07-07 | 2026-07-07 | `—` | Terminal (no known successor) |

---

## 5. Evidence Gaps

The following questions cannot be answered from git history alone and
require owner input or code inspection in RR4:

1. **Experiment goals**: The goal field is inferred from binary names.
   Names like `m22f_cvrp_control` or `story1` require human interpretation
   to confirm the actual hypothesis being tested.

2. **Outcome quality**: Terminal status means no successor was created,
   not that the experiment succeeded or failed. The actual result (positive,
   negative, inconclusive) is not recoverable from structural analysis.

3. **Cross-cluster dependencies**: Some experiments in one cluster may
   have informed experiments in another (e.g., CVRP findings feeding
   ULTRACREW design). These cross-cluster relationships are not captured
   by the intra-cluster parent/successor model.

4. **Canonical replacements**: Where a terminal experiment's findings
   were incorporated into a platform library (e.g., coralys-moga,
   coralys-matching), the canonical replacement should be recorded.
   This requires owner knowledge and is deferred to RR4.

---

## 6. RR4 Inputs

RR3 provides the following inputs to RR4 (Governance Decisions):

1. **Archive candidates**: All superseded binaries (those with successors)
   are candidates for archival. They represent completed research steps
   whose findings have been incorporated into successor experiments.

2. **Preserve candidates**: Terminal binaries in active research streams
   should be preserved until the stream is declared complete.

3. **Delete candidates (E5 only)**: Only `adapters/ultracrew/src/inrc/bipartite_matching.rs`
   (identified in RR2-C) has E5 evidence. No experiment binary reaches E5
   on structural evidence alone.

4. **Lineage graph**: The DAG in `rr3_lineage_graph.md` provides the
   visual foundation for RR4 archival sequencing — superseded binaries
   should be archived in reverse chronological order (oldest first).

---

## 7. Amendment Log

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-08-02 | governance-hardening | Initial RR3 lineage analysis for all 64 experiment binaries |