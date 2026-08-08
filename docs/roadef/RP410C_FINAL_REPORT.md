# RP-410C: Selection Pressure and Survival Funnel — Final Report

**Status:** FROZEN  
**Campaign:** `/tmp/rp410c_v2_validate` — 20 instances (setA-01 through setA-20), 46,195 candidate records  
**Analysis script:** `scripts/rp410c_selection_analysis.py --phase2`  
**Output data:** `docs/roadef/rp410c_data_v2/`  
**Date frozen:** 2026-08-05

---

## Aggregation Convention

Throughout this report, "entered population" means `decision_stage ∈ {Population, Elite, GlobalBest}`. Elite and GlobalBest records are a strict subset of population entrants — every candidate that reached the Elite or GlobalBest stage necessarily entered the population first. All zone-stratified tables use this canonical definition consistently.

---

## Executive Summary

RP-410C instruments the full five-stage DecisionEvent pipeline — Tournament, Promotion, Elite, GlobalBest, and Evaluation — and measures per-stage survival rates and zone-stratified Promotion Efficiency (PE) across 20 ROADEF instances. The campaign establishes three principal findings:

1. **The Elite stage exerts the greatest observed selection pressure** (90.65% loss rate), making it the dominant filter in the pipeline.
2. **Tail-zone candidates suffer a Promotion collapse** (39.6% PE vs 91–100% for all other zones). CapacityViolation is the dominant observed rejection reason associated with Tail candidates and is a plausible explanation for much of this loss, though the telemetry does not prove that every missing Tail promotion is caused by CapacityViolation.
3. **Transition zone achieves the highest end-to-end OSR** (1.965%), despite having the fewest generated candidates of any zone. This is the most actionable finding for RP-409 operator redesign.

These findings establish the quantitative baseline for RP-408 (comparator redesign) and RP-409 (operator redesign).

---

## 1. Campaign Parameters

| Parameter | Value |
|-----------|-------|
| Instances | 20 (setA-01 through setA-20) |
| Total candidate records | 46,195 |
| Telemetry schema | Phase 2 (full DecisionEvent) |
| Decision stages instrumented | Tournament, Population, Elite, GlobalBest, Evaluation |
| Zone classification | Peak (delta_rank1 > 0), Shoulder (delta_2_20 > 0), Transition (delta_21_100 > 0), Tail (else) |

---

## 2. Survival Funnel (aggregate, 20 instances)

| Stage | Count | Transition Rate |
|-------|-------|----------------|
| Generated | 46,195 | 100% |
| Tournament Winners | 16,650 | 36.04% of generated |
| Entered Population | 11,565 | 69.46% of winners |
| Entered Elite | 1,081 | 9.35% of population entrants |
| Became Global Best | 419 | 38.76% of elite entrants |
| **Overall OSR** | **419** | **0.907%** |

The funnel has two dominant loss points: Tournament (63.96% loss) and Elite (90.65% loss). The Promotion stage (Tournament → Population) loses 30.54% of winners in aggregate, but this figure conceals a severe zone-stratified split (see Section 5).

---

## 3. Stage Loss Rates

| Stage | Pool | Survivors | Lost | Loss Rate |
|-------|------|-----------|------|-----------|
| Tournament | 46,195 | 16,650 | 29,545 | 63.96% |
| Promotion | 16,650 | 11,565 | 5,085 | 30.54% |
| Elite | 11,565 | 1,081 | 10,484 | **90.65%** |
| GlobalBest | 1,081 | 419 | 662 | 61.24% |

The Elite stage rejects 9 out of every 10 candidates that enter the population. Whether this loss rate reflects intended elite pressure, unintended bias, or simply the fixed elite size relative to population size is not determinable from the current telemetry alone.

---

## 4. Operator Promotion Efficiency

| Operator | Tournament Wins | Entered Population | PE% |
|----------|----------------|-------------------|-----|
| crossover | 8,579 | 6,300 | 73.44% |
| crossover+mutation | 3,555 | 2,494 | 70.16% |
| mutation | 2,851 | 2,107 | 73.90% |
| elite | 1,665 | 664 | **39.88%** |

The `elite` operator has substantially lower Promotion PE (39.88%) compared to crossover and mutation operators (~73%). This is consistent with the zone analysis: elite-seeded candidates are more likely to be Tail-zone, and Tail-zone candidates suffer Promotion collapse (see Section 5.3).

---

## 5. Zone-Stratified Analysis (Phase 2)

### 5.1 End-to-End OSR by Zone

All counts use the canonical definition: "entered population" = `decision_stage ∈ {Population, Elite, GlobalBest}`.

| Zone | Generated | Tourn Win | Entered Pop | Entered Elite | GlobalBest | PE_Tourn | PE_Promo | PE_Elite | PE_GB | OSR |
|------|-----------|-----------|-------------|---------------|------------|----------|----------|----------|-------|-----|
| Peak | 6,811 | 2,288 | 2,288 | 103 | 44 | 33.59% | 100.0% | 4.50% | 42.72% | 0.646% |
| Shoulder | 14,804 | 4,972 | 4,972 | 266 | 134 | 33.59% | 100.0% | 5.35% | 50.38% | 0.905% |
| Transition | 4,785 | 1,856 | 1,856 | 168 | 94 | 38.79% | 100.0% | 9.05% | 55.95% | **1.965%** |
| Tail | 19,795 | 7,534 | 3,949 | 963 | 147 | 38.06% | **39.63%** | 24.39% | 15.27% | 0.743% |

**Key finding:** Transition zone achieves the highest OSR (1.965%) despite having the fewest generated candidates. Tail zone has the highest generation volume but the lowest GlobalBest PE (15.27%) and a severe Promotion collapse.

Note: `PE_Promo = Entered Pop / Tourn Win`. For Peak, Shoulder, and Transition, every tournament winner entered the population (100%). For Tail, only 39.63% did.

### 5.2 Tournament Win Rate by Zone

| Zone | Participants | Winners | Win Rate |
|------|-------------|---------|----------|
| Peak | 6,811 | 2,288 | 33.59% |
| Shoulder | 14,804 | 4,972 | 33.59% |
| Transition | 4,785 | 1,856 | 38.79% |
| Tail | 19,795 | 7,534 | 38.06% |

Tournament win rates are nearly uniform across zones (33–39%). The tournament subsystem does not discriminate by zone quality — it selects by fitness rank within the tournament pool, which is zone-agnostic. This finding **falsifies** the earlier working hypothesis that tournament selection was suppressing Peak-zone candidates. Tournament selection is not the source of Peak underrepresentation.

### 5.3 Promotion PE by Zone

| Zone | Tournament Winners | Entered Population | Promotion PE |
|------|-------------------|-------------------|--------------|
| Peak | 2,288 | 2,288 | **100.0%** |
| Shoulder | 4,972 | 4,972 | **100.0%** |
| Transition | 1,856 | 1,856 | **100.0%** |
| Tail | 7,534 | 3,949 | **39.63%** |

Peak, Shoulder, and Transition zones achieve 100% Promotion PE — every tournament winner from these zones enters the population. Tail zone collapses to 39.63%.

The 3,585 CapacityViolation records (all associated with Tail-zone candidates) are the dominant observed rejection reason for Tail candidates at the Evaluation stage. CapacityViolation is a plausible explanation for much of the Tail Promotion loss. However, the telemetry records each candidate's final decision stage, not a complete rejection audit trail, so it is not possible to attribute every missing Tail promotion to CapacityViolation from this data alone.

### 5.4 Elite PE by Zone

| Zone | Entered Population | Entered Elite | Elite PE |
|------|-------------------|--------------|----------|
| Peak | 2,288 | 103 | 4.50% |
| Shoulder | 4,972 | 266 | 5.35% |
| Transition | 1,856 | 168 | 9.05% |
| Tail | 3,949 | 963 | **24.39%** |

Elite PE is higher for Tail candidates (24.39%) than for Peak/Shoulder (4.50–5.35%). This is an observation from the telemetry. Whether slot position directly influences elite admission requires inspection of the replacement policy implementation, which is outside the scope of this report.

### 5.5 GlobalBest PE by Zone

| Zone | Entered Elite | Became GlobalBest | GlobalBest PE |
|------|--------------|------------------|---------------|
| Peak | 103 | 44 | 42.72% |
| Shoulder | 266 | 134 | 50.38% |
| Transition | 168 | 94 | **55.95%** |
| Tail | 963 | 147 | 15.27% |

GlobalBest PE follows zone quality in the expected order: Transition > Shoulder > Peak > Tail. Transition-zone candidates that reach the elite have a 55.95% chance of becoming the global best — the highest of any zone. Tail-zone elite candidates have only a 15.27% chance, consistent with their lower objective quality.

### 5.6 Population Slot Distribution by Zone

| Zone | Count | Min Slot | Max Slot | Mean Slot |
|------|-------|----------|----------|-----------|
| Peak | 2,288 | 0 | 49 | 26.90 |
| Shoulder | 4,972 | 0 | 49 | 28.33 |
| Transition | 1,856 | 0 | 49 | 22.60 |
| Tail | 3,949 | 0 | 46 | **14.93** |

Tail-zone candidates are assigned to lower-index population slots (mean 14.93) compared to Peak/Shoulder (mean 26.9–28.3). This is an observation from the telemetry. Whether slot position directly influences elite admission requires inspection of the replacement policy implementation.

---

## 6. Rejection Reason Analysis

| Stage | Reason | Zone | Count |
|-------|--------|------|-------|
| Tournament | LostTournament | Tail | 12,261 |
| Tournament | LostTournament | Shoulder | 9,832 |
| Population | EnteredPopulation | Shoulder | 4,972 |
| Tournament | LostTournament | Peak | 4,523 |
| Evaluation | CapacityViolation | Tail | **3,585** |
| Population | EnteredPopulation | Tail | 3,949 |
| Tournament | LostTournament | Transition | 2,929 |
| Population | EnteredPopulation | Peak | 2,288 |
| Population | EnteredPopulation | Transition | 1,856 |
| Elite | EnteredElite | Tail | 963 |
| GlobalBest | None | Tail | 147 |
| GlobalBest | None | Shoulder | 134 |
| Elite | EnteredElite | Shoulder | 266 |
| GlobalBest | None | Transition | 94 |
| Elite | EnteredElite | Transition | 168 |
| Elite | EnteredElite | Peak | 103 |
| GlobalBest | None | Peak | 44 |

All 3,585 CapacityViolation records are associated with Tail-zone candidates. No CapacityViolation records appear in Peak, Shoulder, or Transition zones. This is consistent with the Tail Promotion collapse but does not by itself prove that CapacityViolation is the sole cause.

---

## 7. Summary of Key Findings

**Finding 1 — Elite stage exerts the greatest observed selection pressure.** The Elite stage rejects 90.65% of candidates that enter the population. This is the largest single loss in the pipeline. Whether this reflects intended elite pressure or unintended bias is not determinable from the current telemetry.

**Finding 2 — Tail-zone Promotion collapse is associated with CapacityViolation.** Tail-zone tournament winners achieve only 39.63% Promotion PE, compared to 100% for all other zones. CapacityViolation is the dominant observed rejection reason for Tail candidates (3,585 records, all Tail zone) and is a plausible explanation for much of this loss.

**Finding 3 — Tournament selection is zone-agnostic.** Tournament win rates are 33–39% across all zones with no meaningful zone-based discrimination. This falsifies the earlier hypothesis that tournament selection was suppressing Peak-zone candidates.

**Finding 4 — Elite PE is higher for Tail-zone candidates.** Tail-zone candidates that survive Promotion enter the elite at 24.39%, while Peak/Shoulder candidates enter at 4.50–5.35%. The mechanism behind this inversion is not established by the current telemetry.

**Finding 5 — Transition zone achieves the highest OSR.** Despite having the fewest generated candidates (4,785), Transition-zone candidates achieve the highest OSR (1.965%) and the highest GlobalBest PE (55.95%). This is the most actionable finding for RP-409.

**Finding 6 — Tail zone dominates generation volume but underperforms.** Tail-zone candidates account for 42.8% of all generated candidates (19,795/46,195) but only 35.1% of GlobalBest events (147/419). The combination of Promotion collapse and low GlobalBest PE makes Tail-zone generation the least efficient use of evaluation budget.

---

## 8. Implications for RP-408 and RP-409

**RP-408 (Comparator redesign):** Phase 2 telemetry falsifies the hypothesis that tournament selection suppresses Peak candidates. The more productive experimental question for RP-408 is: does a lexicographic comparator change the distribution of candidates entering the Elite stage? Specifically, does it reduce the Elite PE inversion (Tail 24.39% vs Peak 4.50%) by altering which candidates survive Promotion? This is a testable hypothesis against the frozen Phase 2 baseline.

**RP-409 (Operator redesign):** Transition zone achieves the highest OSR (1.965%) with the fewest generated candidates. The primary RP-409 objective should be to increase Transition-zone candidate production while maintaining Peak improvements. The `elite` operator's low Promotion PE (39.88%) is consistent with its Tail-zone generation profile and is a candidate for redesign or replacement.

**RP-412 (Construction repair):** The 3,585 CapacityViolation records represent evaluation budget consumed by candidates that cannot enter the population. A construction repair mechanism that reduces the rate of capacity-violating candidates would directly improve Tail-zone Promotion PE and reduce wasted evaluations.

---

## 9. Data Provenance

| Artefact | Path |
|----------|------|
| Telemetry (moves) | `/tmp/rp410c_v2_validate/rp410_moves_setA-*.jsonl` |
| Telemetry (generations) | `/tmp/rp410c_v2_validate/rp410_generations_setA-*.jsonl` |
| Analysis script | `scripts/rp410c_selection_analysis.py` |
| Survival funnel CSV | `docs/roadef/rp410c_data_v2/funnel.csv` |
| Stage loss CSV | `docs/roadef/rp410c_data_v2/stage_loss.csv` |
| Operator PE CSV | `docs/roadef/rp410c_data_v2/operator_pe.csv` |
| Tournament PE by zone | `docs/roadef/rp410c_data_v2/tournament_pe_by_zone.csv` |
| Population PE by zone | `docs/roadef/rp410c_data_v2/population_pe_by_zone.csv` |
| Elite PE by zone | `docs/roadef/rp410c_data_v2/elite_pe_by_zone.csv` |
| GlobalBest PE by zone | `docs/roadef/rp410c_data_v2/globalbest_pe_by_zone.csv` |
| Reason frequency by zone | `docs/roadef/rp410c_data_v2/reason_freq_by_zone.csv` |
| Population slot distribution | `docs/roadef/rp410c_data_v2/population_slot_dist.csv` |
| OSR by zone | `docs/roadef/rp410c_data_v2/osr_by_zone.csv` |
| Auto-generated report | `docs/roadef/rp410c_data_v2/RP410C_PHASE2_ANALYSIS_REPORT.md` |

---

*This document is frozen. Do not modify. Subsequent experiments must reference this baseline.*