# RP-410C Selection Analysis Report

**Telemetry directory:** `/tmp/rp410c_v2_validate`
**Total candidate records:** 46,195
**Instances:** 20
**Phase 2 analysis:** enabled

## 1. Survival Funnel

| Stage | Count | Rate |
|-------|-------|------|
| Generated | 46,195 | 100% |
| Tournament Winners | 16,650 | 36.043% |
| Entered Population | 11,565 | 69.459% of winners |
| Entered Elite | 1,081 | 9.347% of pop |
| Became Global Best | 419 | 38.76% of elite |
| **Overall OSR** | 419 | **0.907%** |

## 2. Stage Loss Rates

| Stage | Pool | Survivors | Lost | Loss Rate |
|-------|------|-----------|------|-----------|
| Tournament | 46,195 | 16,650 | 29,545 | 63.957% |
| Promotion | 16,650 | 11,565 | 5,085 | 30.541% |
| Elite | 11,565 | 1,081 | 10,484 | 90.653% |
| GlobalBest | 1,081 | 419 | 662 | 61.24% |

## 3. Operator Promotion Efficiency

| Operator | Tournament Wins | Entered Population | PE% |
|----------|----------------|-------------------|-----|
| crossover | 8,579 | 6,300 | 73.435% |
| crossover+mutation | 3,555 | 2,494 | 70.155% |
| elite | 1,665 | 664 | 39.88% |
| mutation | 2,851 | 2,107 | 73.904% |

## 4. Decision Stage × Reason Frequency

| Stage | Reason | Count |
|-------|--------|-------|
| Tournament | LostTournament | 29,545 |
| Population | EnteredPopulation | 11,565 |
| Evaluation | CapacityViolation | 3,585 |
| Elite | EnteredElite | 1,081 |
| GlobalBest | None | 419 |

## 5. Objective Value Statistics (valid candidates)

- Count: 34,212
- Min: 13.848155
- Max: 5592470.556964
- Mean: 2533.338916

---

## Phase 2 — DecisionEvent Analysis

### 6. End-to-End OSR by Zone

| Zone | Generated | Tourn Win | Pop | Elite | GlobalBest | PE_Tourn | PE_Promo | PE_Elite | PE_GB | OSR |
|------|-----------|-----------|-----|-------|------------|----------|----------|----------|-------|-----|
| Peak | 6,811 | 2,288 | 2,288 | 103 | 44 | 33.5927% | 100.0% | 4.5017% | 42.7184% | 0.646% |
| Shoulder | 14,804 | 4,972 | 4,972 | 266 | 134 | 33.5855% | 100.0% | 5.35% | 50.3759% | 0.9052% |
| Transition | 4,785 | 1,856 | 1,856 | 168 | 94 | 38.7879% | 100.0% | 9.0517% | 55.9524% | 1.9645% |
| Tail | 19,795 | 7,534 | 3,949 | 963 | 147 | 38.0601% | 52.4157% | 24.3859% | 15.2648% | 0.7426% |

### 7. Tournament PE by Zone

| Zone | Participants | Winners | Win Rate |
|------|-------------|---------|----------|
| Peak | 6,811 | 2,288 | 33.593% |
| Shoulder | 14,804 | 4,972 | 33.586% |
| Transition | 4,785 | 1,856 | 38.788% |
| Tail | 19,795 | 7,534 | 38.06% |

### 8. Promotion PE by Zone

| Zone | Tournament Winners | Entered Population | Promotion PE |
|------|-------------------|-------------------|--------------|
| Peak | 2,288 | 2,185 | 95.498% |
| Shoulder | 4,972 | 4,706 | 94.65% |
| Transition | 1,856 | 1,688 | 90.948% |
| Tail | 7,534 | 2,986 | 39.634% |

### 9. Elite PE by Zone

| Zone | Entered Population | Entered Elite | Elite PE |
|------|-------------------|--------------|----------|
| Peak | 2,244 | 59 | 2.629% |
| Shoulder | 4,838 | 132 | 2.728% |
| Transition | 1,762 | 74 | 4.2% |
| Tail | 3,802 | 816 | 21.462% |

### 10. GlobalBest PE by Zone

| Zone | Entered Elite | Became GlobalBest | GlobalBest PE |
|------|--------------|------------------|---------------|
| Peak | 103 | 44 | 42.718% |
| Shoulder | 266 | 134 | 50.376% |
| Transition | 168 | 94 | 55.952% |
| Tail | 963 | 147 | 15.265% |

### 11. Rejection Reason Frequency by Zone (top 30)

| Stage | Reason | Zone | Count |
|-------|--------|------|-------|
| Tournament | LostTournament | Tail | 12,261 |
| Tournament | LostTournament | Shoulder | 9,832 |
| Population | EnteredPopulation | Shoulder | 4,706 |
| Tournament | LostTournament | Peak | 4,523 |
| Evaluation | CapacityViolation | Tail | 3,585 |
| Population | EnteredPopulation | Tail | 2,986 |
| Tournament | LostTournament | Transition | 2,929 |
| Population | EnteredPopulation | Peak | 2,185 |
| Population | EnteredPopulation | Transition | 1,688 |
| Elite | EnteredElite | Tail | 816 |
| GlobalBest | None | Tail | 147 |
| GlobalBest | None | Shoulder | 134 |
| Elite | EnteredElite | Shoulder | 132 |
| GlobalBest | None | Transition | 94 |
| Elite | EnteredElite | Transition | 74 |
| Elite | EnteredElite | Peak | 59 |
| GlobalBest | None | Peak | 44 |

### 12. Population Slot Distribution by Zone

| Zone | Count | Min Slot | Max Slot | Mean Slot |
|------|-------|----------|----------|-----------|
| Peak | 2,288 | 0 | 49 | 26.9 |
| Shoulder | 4,972 | 0 | 49 | 28.33 |
| Transition | 1,856 | 0 | 49 | 22.6 |
| Tail | 3,949 | 0 | 46 | 14.93 |
