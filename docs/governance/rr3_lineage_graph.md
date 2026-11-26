# RR3 — Experiment Lineage Graph

**Programme:** Repository Rationalization  
**Phase:** RR3 — Historical Evidence / Evolutionary Lineage Analysis  
**Produced:** 2026-08-02  

Each node shows: `binary_name  [generation]  first_commit_date  LOC`  
Edges represent: derived-from / superseded-by relationships inferred from
generation numbering and commit chronology.

> Note: Parent–successor edges are inferred from naming conventions.
> Where the naming convention is ambiguous, the edge represents the most
> likely evolutionary relationship. RR4 may revise individual edges.

## CVRP Research Stream  (32 experiments)

```
m8g_cvrp_validation  [m8g]  2026-07-06  286 LOC
  goal: Validation against reference
  └── m11_reachability_atlas
    m11_reachability_atlas  [m11]  2026-07-06  288 LOC
      goal: Reachability / solution space mapping
      └── m12_cvrp_repair_atlas
        m12_cvrp_repair_atlas  [m12]  2026-07-06  289 LOC
          goal: Reachability / solution space mapping
          └── m14_cvrp_recognizability_audit
            m14_cvrp_recognizability_audit  [m14]  2026-07-06  317 LOC
              goal: Structural or behavioural audit
              └── m15_cvrp_decoder_independence
                m15_cvrp_decoder_independence  [m15]  2026-07-06  222 LOC
                  goal: Decoder independence verification
                  └── m16_cvrp_reconstruction_guidance
                    m16_cvrp_reconstruction_guidance  [m16]  2026-07-06  271 LOC
                      goal: Reconstruction guidance
                      └── m17_cvrp_backbone_causality
                        m17_cvrp_backbone_causality  [m17]  2026-07-06  238 LOC
                          goal: Backbone causality analysis
                          └── m18_structural_invariants
                            m18_structural_invariants  [m18]  2026-07-06  224 LOC
                              goal: Structural invariant verification
                              └── m20_epistasis
                                m20_epistasis  [m20]  2026-07-06  132 LOC
                                  goal: Epistasis / gene interaction study
                                  └── m21_feasible_path
                                    m21_feasible_path  [m21]  2026-07-06  257 LOC
                                      goal: Feasibility path analysis
                                      ├── m21c_sa
                                        m21c_sa  [m21c]  2026-07-06  246 LOC
                                          goal: Simulated annealing
                                      ├── m21d_elite_sa
                                        m21d_elite_sa  [m21d]  2026-07-06  215 LOC
                                          goal: Simulated annealing
                                      ├── m22_0_archive
                                        m22_0_archive  [m22_0]  2026-07-06  356 LOC
                                          goal: General experiment
                                          ├── m22a_annealed_elite
                                            m22a_annealed_elite  [m22a]  2026-07-06  437 LOC
                                              goal: Simulated annealing variant
                                          ├── m22b_novelty_elitism
                                            m22b_novelty_elitism  [m22b]  2026-07-06  371 LOC
                                              goal: Novelty-elitism hybrid
                                          ├── m22c_0_route_sa
                                            m22c_0_route_sa  [m22c_0]  2026-07-06  230 LOC
                                              goal: Simulated annealing
                                          ├── m22c_1_memetic
                                            m22c_1_memetic  [m22c_1]  2026-07-06  584 LOC
                                              goal: Memetic algorithm variant
                                          ├── m22c_2_fidelity_audit
                                            m22c_2_fidelity_audit  [m22c_2]  2026-07-06  657 LOC
                                              goal: Structural or behavioural audit
                                          ├── m22f_cvrp_control
                                            m22f_cvrp_control  [m22f]  2026-07-06  150 LOC
                                              goal: Control / baseline experiment
                                          ├── m30_2_active_pilot
                                            m30_2_active_pilot  [m30_2]  2026-07-06  216 LOC
                                              goal: Active pilot / live deployment
                                          ├── m30_2a_1_ecology_audit
                                            m30_2a_1_ecology_audit  [m30_2]  2026-07-06  139 LOC
                                              goal: Structural or behavioural audit
                                          └── m30_2a_2_shadow_advisory
                                            m30_2a_2_shadow_advisory  [m30_2]  2026-07-06  169 LOC
                                              goal: Shadow advisory mode
                                      └── m22_scale_up
                                        m22_scale_up  [m22]  2026-07-06  670 LOC
                                          goal: Scale-up experiment

basin_characterization  [—]  2026-07-06  180 LOC
  goal: Basin of attraction characterisation

compare  [—]  2026-07-06  52 LOC
  goal: Comparative analysis

elite_manifold_probe  [—]  2026-07-06  95 LOC
  goal: Elite manifold probing

elite_partition_probe  [—]  2026-07-06  149 LOC
  goal: Partition probing

find_797  [—]  2026-07-06  38 LOC
  goal: Search / discovery experiment

frozen_partition_probe  [—]  2026-07-06  160 LOC
  goal: Partition probing

initial_basin_distribution  [—]  2026-07-06  71 LOC
  goal: Basin of attraction characterisation

search_config  [—]  2026-07-06  122 LOC
  goal: Search configuration study

seed_ecology_study  [—]  2026-07-06  276 LOC
  goal: Ecology / fitness landscape characterisation

```

## ULTRACREW / INRC Research Stream  (18 experiments)

```
m8g_cs_validation  [m8g]  2026-07-06  110 LOC
  goal: Validation against reference
  └── m9a_search_observatory
    m9a_search_observatory  [m9a]  2026-07-06  316 LOC
      goal: Search observatory
      └── m23a_synthetic
        m23a_synthetic  [m23a]  2026-07-07  203 LOC
          goal: Synthetic data experiment
          ├── m30_0b_passive_telemetry
            m30_0b_passive_telemetry  [m30_0]  2026-07-07  141 LOC
              goal: Passive observation / telemetry
              ├── m31_2a_engagement_audit
                m31_2a_engagement_audit  [m31_2]  2026-07-07  260 LOC
                  goal: Structural or behavioural audit
              └── m31_benchmarks
                m31_benchmarks  [m31]  2026-07-07  463 LOC
                  goal: Benchmark / performance measurement
          └── m30_0d_active_pilot
            m30_0d_active_pilot  [m30_0]  2026-07-06  221 LOC
              goal: Active pilot / live deployment

m8g_ultracrew_validation  [m8g]  2026-07-06  156 LOC
  goal: Validation against reference

config_sweep  [—]  2026-07-07  271 LOC
  goal: Configuration sweep

inrc_ecology_ablation_matrix  [—]  2026-07-07  353 LOC
  goal: Ablation study — component isolation

inrc_ecology_cost_curve  [—]  2026-07-07  348 LOC
  goal: Ecology / fitness landscape characterisation

inrc_ecology_memory_depth  [—]  2026-07-07  341 LOC
  goal: Ecology / fitness landscape characterisation

inrc_ecology_multi_week_ablation  [—]  2026-06-04  451 LOC
  goal: Ablation study — component isolation

inrc_ecology_response_curve  [—]  2026-07-07  347 LOC
  goal: Ecology / fitness landscape characterisation

inrc_m22_ancestry  [—]  2026-07-07  321 LOC
  goal: Ancestry / lineage tracing

inrc_natural_history_pilot  [—]  2026-07-07  347 LOC
  goal: Natural history pilot

policy_seed_runner  [—]  2026-07-06  347 LOC
  goal: Seed / initialisation study

story1  [—]  2026-07-07  97 LOC
  goal: Narrative / demonstration run

```

## ROADEF Research Stream  (14 experiments)

```
m25_8_bridge  [m25_8]  2026-07-06  338 LOC
  goal: Bridge / transition experiment
  ├── m26_1_observation_audit
    m26_1_observation_audit  [m26_1]  2026-07-06  154 LOC
      goal: Structural or behavioural audit
      └── m27_1_passive_operator_telemetry
        m27_1_passive_operator_telemetry  [m27_1]  2026-06-14  270 LOC
          goal: Passive observation / telemetry
  ├── m26_1c_discriminative_audit
    m26_1c_discriminative_audit  [m26_1]  2026-07-06  233 LOC
      goal: Structural or behavioural audit
  ├── m26_1d_failure_density
    m26_1d_failure_density  [m26_1]  2026-07-06  171 LOC
      goal: Failure density analysis
  ├── m26_1e_survival_curves
    m26_1e_survival_curves  [m26_1]  2026-07-06  150 LOC
      goal: Survival curve analysis
  ├── m26_3_passive_learner
    m26_3_passive_learner  [m26_3]  2026-06-14  240 LOC
      goal: Passive observation / telemetry
  ├── m26_4a_shadow_advisory
    m26_4a_shadow_advisory  [m26_4]  2026-06-14  303 LOC
      goal: Shadow advisory mode
  └── m26_4b_active_pilot
    m26_4b_active_pilot  [m26_4]  2026-06-14  307 LOC
      goal: Active pilot / live deployment

m25_8b_ecology  [m25_8]  2026-07-06  358 LOC
  goal: Ecology / fitness landscape characterisation

m25_benchmark  [m25]  2026-07-06  377 LOC
  goal: Benchmark / performance measurement

m25_final  [m25]  2026-07-06  284 LOC
  goal: General experiment

eval_profiler  [—]  2026-07-11  125 LOC
  goal: Evaluation profiler

tiny_solver  [—]  2026-07-06  259 LOC
  goal: Minimal / smoke-test solver

```
