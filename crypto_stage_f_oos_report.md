# Stage F: Out-Of-Sample Chronological Test

Testing Hypothesis F1 on a fresh, strictly unseen chronological dataset.

## 1. Data Integrity & Provenance
- **Source File**: `high_vol_BTCUSDT.json`
- **Time Range**: 2026-08-19 08:00:00 to 2026-08-22 08:00:00
- **Raw Observation Count**: 4321
- **Eligible T0 Count** (after 1440m warmup): 2882
- **Complete 60-bar Count**: 2822
- **Complete 300-bar Count (Census N)**: 2582
- **Excluded/Incomplete T0s**: 300

### Directional Breakdown
- UP Trend Count: 2582
- DOWN Trend Count: 0
- ZERO Trend Count: 0

## 2. Phenotype Recurrence (Persistent Build)
- **Classified Persistent Builds**: 563
- **Frequency**: 21.80% of OOS population

### Persistent Build Median Path (OOS)
| 15m | 30m | 60m | 120m | 300m |
|---|---|---|---|---|
| 0.0002 | 0.0024 | 0.0057 | 0.0124 | 0.0247 |

## 3. Structural Overlap
**Measurement:** Overlap between `Persistent Build` and `Adverse Excursion & Recovery`

- **Intersection (PB ∩ AER)**: 514 occurrences
- **P(Adverse Recovery | Persistent Build)**: 91.30%
- **Comparison with Frozen Census**: 91.30% (OOS) vs 92.00% (Historical)

## 4. State Association Recurrence
**Measurement:** Overrepresentation of `HIGH VA + LOW persistence` within `Persistent Build`

- **P(HIGH/LOW) [Population Share]**: 12.24% (316 / 2582)
- **P(HIGH/LOW | Persistent Build)**: 16.70% (94 / 563)
- **Overrepresentation Ratio**: 1.36x

## 5. Failure Modes Assessment
Evaluation of whether Hypothesis F1 survived contact with the OOS block:

**Result**: HYPOTHESIS SURVIVED. The Persistent Build path structure recurred with similar prevalence, retained its extreme overlap with Adverse Recovery, and the HIGH/LOW state cohort remained strictly overrepresented.
