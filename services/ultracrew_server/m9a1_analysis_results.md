# M9A.1 Analysis Results

## Phase 1 — Descriptive Statistics

- **Total Runs Analyzed**: 90
- **Last Improvement Gen**: Mean 2331.6, Median 1112.5, Max 9157
- **Total Improvements per Run**: Mean 1.3, Median 1.0
- **Best Fitness Achieved**: Mean 63870.3, Min 43730.0
- **Average History Novelty**: Mean 0.1337
- **Average Distance to Incumbent**: Mean 4094.8
- **Max Improvement Drought Length**: Mean 7464.7, Max 9994

## Phase 2 — Search Archetypes (Clustering)

### Cluster 2 (N=23)
- **Exemplar Trajectory**: Instance `n030w4`, Seed `1`
- **Average Best Fitness**: 62792.8
- **Outcome Insight**: Does this shape produce better outcomes? (Min: 45740.0, Max: 74970.0)

### Cluster 1 (N=29)
- **Exemplar Trajectory**: Instance `n030w4`, Seed `8`
- **Average Best Fitness**: 63591.7
- **Outcome Insight**: Does this shape produce better outcomes? (Min: 44025.0, Max: 87845.0)

### Cluster 3 (N=26)
- **Exemplar Trajectory**: Instance `n030w4`, Seed `9`
- **Average Best Fitness**: 67823.8
- **Outcome Insight**: Does this shape produce better outcomes? (Min: 43730.0, Max: 89805.0)

### Cluster 0 (N=12)
- **Exemplar Trajectory**: Instance `n030w4`, Seed `24`
- **Average Best Fitness**: 58042.5
- **Outcome Insight**: Does this shape produce better outcomes? (Min: 44520.0, Max: 75190.0)

## Phase 3 — Transition Analysis

**Breakthrough Threshold**: 1507.5 (Top 10% magnitude)

### Precursor Distribution Test
**Category: BREAKTHROUGH (N=10)**
- history_novelty: T-500 = 0.1178 -> T-50 = 0.1639
- distance_to_incumbent_best: T-500 = 3479.0000 -> T-50 = 4986.5000
- acceptance_rate: T-500 = 0.7500 -> T-50 = 0.3500

**Category: IMPROVEMENT (N=82)**
- history_novelty: T-500 = 0.1163 -> T-50 = 0.1374
- distance_to_incumbent_best: T-500 = 2450.3049 -> T-50 = 3655.7317
- acceptance_rate: T-500 = 0.6220 -> T-50 = 0.5244

**Category: RANDOM (N=900)**
- history_novelty: T-500 = 0.1323 -> T-50 = 0.1381
- distance_to_incumbent_best: T-500 = 3992.3111 -> T-50 = 4135.1444
- acceptance_rate: T-500 = 0.4200 -> T-50 = 0.4094

## Phase 4 — Regime Analysis & Opportunity Table

**Total Regime Transitions Identified**: 448784
- **Transitions yielding Improvement (within 200 gens)**: 10741 (2.4%)
- **Transitions yielding NO Improvement (Survivorship Bias check)**: 438043 (97.6%)

## Ledger Evaluation (H1-H7)

> **H1**: Improvements cluster near structural novelty spikes. **(TBD based on Phase 3)**
> **H2**: Improvements cluster near operator regime transitions. **(TBD based on Phase 4)**
> **H3**: Breakthroughs arise from different dynamics than ordinary improvements. **(TBD based on Phase 3)**
> **H4**: Acceptance-rate decay predicts breakthrough density. **(TBD based on Phase 3)**
> **H5**: Independent trajectories form recurring archetypal shapes. **(TBD based on Phase 2)**
> **H6**: Distance-to-incumbent exhibits characteristic behavior prior to breakthroughs. **(TBD based on Phase 3)**
> **H7**: Breakthrough probability depends more on search state than absolute generation. **(Supported if precursors are distinct from random baseline)**