# P4 Track B — Intraday Discovery Results

**Date:** 2026-09-11
**Population:** 956 COMPLETE decisions
**Intraday dataset:** 5m, 102 tickers, Jun 22 – Sep 11 2026
**Matched to intraday:** 956 / 956

**Horizons:** H15 (3 bars), H30 (6), H60 (12), H120 (24), H180 (36), H300 (60)
**Extreme move threshold:** |return| > 5% at horizon

**Three objectives:**
1. Genuine return prediction (ordinary movement)
2. Path prediction (MFE / MAE)
3. Extreme-move prediction

---

## 1. Baseline — all decisions by direction and horizon

| Cohort | Horizon | N | Mean | Median | Win% | Loss% | Mean MFE | Mean MAE | Extreme% |
|---|---|---|---|---|---|---|---|---|---|
| All COMPLETE | H15 | 956 | -0.01% | -0.01% | 47.5% | 50.4% | +0.14% | -0.15% | 0.0% |
| All COMPLETE | H30 | 956 | +0.02% | +0.01% | 50.4% | 48.0% | +0.25% | -0.23% | 0.0% |
| All COMPLETE | H60 | 956 | +0.01% | +0.02% | 51.5% | 47.9% | +0.35% | -0.33% | 0.0% |
| All COMPLETE | H120 | 956 | +0.00% | -0.01% | 49.3% | 50.1% | +0.48% | -0.45% | 0.0% |
| All COMPLETE | H180 | 956 | -0.02% | -0.03% | 47.7% | 51.8% | +0.56% | -0.55% | 0.1% |
| All COMPLETE | H300 | 956 | -0.02% | -0.03% | 48.5% | 51.3% | +0.69% | -0.67% | 0.1% |

| Watch (all) | H15 | 417 | -0.02% | -0.02% | 44.8% | 52.5% | +0.15% | -0.17% | 0.0% |
| Watch (all) | H30 | 417 | +0.01% | +0.00% | 49.2% | 49.2% | +0.25% | -0.26% | 0.0% |
| Watch (all) | H60 | 417 | +0.02% | +0.03% | 52.0% | 46.8% | +0.36% | -0.34% | 0.0% |
| Watch (all) | H120 | 417 | +0.02% | -0.02% | 49.4% | 50.4% | +0.49% | -0.46% | 0.0% |
| Watch (all) | H180 | 417 | -0.01% | -0.02% | 49.2% | 50.6% | +0.58% | -0.56% | 0.2% |
| Watch (all) | H300 | 417 | -0.03% | -0.03% | 48.9% | 50.8% | +0.69% | -0.69% | 0.2% |

| LONG Watch | H15 | 230 | -0.10% | -0.10% | 30.9% | 67.4% | +0.10% | -0.19% | 0.0% |
| LONG Watch | H30 | 230 | -0.06% | -0.10% | 35.7% | 62.2% | +0.20% | -0.29% | 0.0% |
| LONG Watch | H60 | 230 | -0.07% | -0.07% | 41.7% | 56.1% | +0.31% | -0.39% | 0.0% |
| LONG Watch | H120 | 230 | -0.10% | -0.17% | 38.3% | 61.3% | +0.45% | -0.52% | 0.0% |
| LONG Watch | H180 | 230 | -0.12% | -0.13% | 42.6% | 57.4% | +0.52% | -0.62% | 0.4% |
| LONG Watch | H300 | 230 | -0.24% | -0.29% | 37.0% | 62.6% | +0.59% | -0.77% | 0.4% |

| SHORT Watch | H15 | 187 | +0.08% | +0.09% | 62.0% | 34.2% | +0.22% | -0.14% | 0.0% |
| SHORT Watch | H30 | 187 | +0.11% | +0.08% | 65.8% | 33.2% | +0.31% | -0.21% | 0.0% |
| SHORT Watch | H60 | 187 | +0.13% | +0.12% | 64.7% | 35.3% | +0.41% | -0.29% | 0.0% |
| SHORT Watch | H120 | 187 | +0.17% | +0.17% | 63.1% | 36.9% | +0.55% | -0.38% | 0.0% |
| SHORT Watch | H180 | 187 | +0.14% | +0.14% | 57.2% | 42.2% | +0.65% | -0.49% | 0.0% |
| SHORT Watch | H300 | 187 | +0.22% | +0.27% | 63.6% | 36.4% | +0.82% | -0.60% | 0.0% |

---

## 2. SHORT Watch — intraday decomposition by sample_size band

| Cohort | Horizon | N | Mean | Median | Win% | Loss% | Mean MFE | Mean MAE | Extreme% |
|---|---|---|---|---|---|---|---|---|---|
| SHORT Watch sample 1–50 | H15 | 20 | +0.03% | +0.07% | 60.0% | 40.0% | +0.23% | -0.18% | 0.0% |
| SHORT Watch sample 1–50 | H30 | 20 | +0.04% | -0.01% | 50.0% | 50.0% | +0.32% | -0.28% | 0.0% |
| SHORT Watch sample 1–50 | H60 | 20 | +0.01% | +0.04% | 55.0% | 45.0% | +0.41% | -0.36% | 0.0% |
| SHORT Watch sample 1–50 | H120 | 20 | +0.13% | -0.06% | 45.0% | 55.0% | +0.61% | -0.55% | 0.0% |
| SHORT Watch sample 1–50 | H180 | 20 | +0.11% | +0.10% | 60.0% | 35.0% | +0.76% | -0.65% | 0.0% |
| SHORT Watch sample 1–50 | H300 | 20 | +0.25% | +0.16% | 60.0% | 40.0% | +0.97% | -0.70% | 0.0% |

| SHORT Watch sample 51–100 | H15 | 68 | +0.10% | +0.11% | 66.2% | 29.4% | +0.23% | -0.12% | 0.0% |
| SHORT Watch sample 51–100 | H30 | 68 | +0.13% | +0.09% | 72.1% | 27.9% | +0.31% | -0.20% | 0.0% |
| SHORT Watch sample 51–100 | H60 | 68 | +0.12% | +0.15% | 67.6% | 32.4% | +0.40% | -0.29% | 0.0% |
| SHORT Watch sample 51–100 | H120 | 68 | +0.20% | +0.32% | 70.6% | 29.4% | +0.52% | -0.35% | 0.0% |
| SHORT Watch sample 51–100 | H180 | 68 | +0.14% | +0.39% | 60.3% | 39.7% | +0.63% | -0.47% | 0.0% |
| SHORT Watch sample 51–100 | H300 | 68 | +0.21% | +0.45% | 64.7% | 35.3% | +0.71% | -0.55% | 0.0% |

| SHORT Watch sample 101–150 | H15 | 24 | -0.07% | +0.06% | 58.3% | 33.3% | +0.15% | -0.26% | 0.0% |
| SHORT Watch sample 101–150 | H30 | 24 | +0.00% | +0.08% | 58.3% | 41.7% | +0.31% | -0.37% | 0.0% |
| SHORT Watch sample 101–150 | H60 | 24 | +0.05% | +0.12% | 54.2% | 45.8% | +0.43% | -0.46% | 0.0% |
| SHORT Watch sample 101–150 | H120 | 24 | +0.09% | +0.26% | 62.5% | 37.5% | +0.59% | -0.62% | 0.0% |
| SHORT Watch sample 101–150 | H180 | 24 | +0.07% | +0.18% | 54.2% | 45.8% | +0.66% | -0.66% | 0.0% |
| SHORT Watch sample 101–150 | H300 | 24 | +0.12% | +0.18% | 54.2% | 45.8% | +0.89% | -0.74% | 0.0% |

| SHORT Watch sample 151+ | H15 | 75 | +0.12% | +0.09% | 60.0% | 37.3% | +0.23% | -0.10% | 0.0% |
| SHORT Watch sample 151+ | H30 | 75 | +0.13% | +0.08% | 66.7% | 30.7% | +0.31% | -0.16% | 0.0% |
| SHORT Watch sample 151+ | H60 | 75 | +0.19% | +0.08% | 68.0% | 32.0% | +0.42% | -0.22% | 0.0% |
| SHORT Watch sample 151+ | H120 | 75 | +0.19% | +0.11% | 61.3% | 38.7% | +0.55% | -0.30% | 0.0% |
| SHORT Watch sample 151+ | H180 | 75 | +0.16% | +0.05% | 54.7% | 45.3% | +0.65% | -0.41% | 0.0% |
| SHORT Watch sample 151+ | H300 | 75 | +0.26% | +0.27% | 66.7% | 33.3% | +0.86% | -0.57% | 0.0% |


---

## 3. SHORT Watch — intraday decomposition by rank_score quartile

| Cohort | Horizon | N | Mean | Median | Win% | Loss% | Mean MFE | Mean MAE | Extreme% |
|---|---|---|---|---|---|---|---|---|---|
| SHORT Watch rank_score Q1 | H15 | 46 | +0.04% | +0.06% | 56.5% | 39.1% | +0.21% | -0.18% | 0.0% |
| SHORT Watch rank_score Q1 | H30 | 46 | +0.06% | +0.08% | 65.2% | 32.6% | +0.30% | -0.25% | 0.0% |
| SHORT Watch rank_score Q1 | H60 | 46 | +0.10% | +0.06% | 63.0% | 37.0% | +0.41% | -0.33% | 0.0% |
| SHORT Watch rank_score Q1 | H120 | 46 | +0.05% | +0.08% | 54.3% | 45.7% | +0.53% | -0.43% | 0.0% |
| SHORT Watch rank_score Q1 | H180 | 46 | +0.11% | +0.06% | 56.5% | 43.5% | +0.64% | -0.54% | 0.0% |
| SHORT Watch rank_score Q1 | H300 | 46 | +0.23% | +0.24% | 67.4% | 32.6% | +0.90% | -0.68% | 0.0% |

| SHORT Watch rank_score Q2 | H15 | 47 | +0.13% | +0.09% | 63.8% | 36.2% | +0.25% | -0.13% | 0.0% |
| SHORT Watch rank_score Q2 | H30 | 47 | +0.21% | +0.22% | 68.1% | 31.9% | +0.40% | -0.19% | 0.0% |
| SHORT Watch rank_score Q2 | H60 | 47 | +0.23% | +0.20% | 70.2% | 29.8% | +0.49% | -0.25% | 0.0% |
| SHORT Watch rank_score Q2 | H120 | 47 | +0.25% | +0.10% | 61.7% | 38.3% | +0.66% | -0.38% | 0.0% |
| SHORT Watch rank_score Q2 | H180 | 47 | +0.13% | +0.06% | 53.2% | 46.8% | +0.72% | -0.47% | 0.0% |
| SHORT Watch rank_score Q2 | H300 | 47 | +0.15% | +0.13% | 55.3% | 44.7% | +0.87% | -0.62% | 0.0% |

| SHORT Watch rank_score Q3 | H15 | 47 | +0.07% | +0.11% | 63.8% | 27.7% | +0.19% | -0.10% | 0.0% |
| SHORT Watch rank_score Q3 | H30 | 47 | +0.06% | +0.07% | 68.1% | 29.8% | +0.25% | -0.19% | 0.0% |
| SHORT Watch rank_score Q3 | H60 | 47 | +0.06% | +0.14% | 59.6% | 40.4% | +0.35% | -0.30% | 0.0% |
| SHORT Watch rank_score Q3 | H120 | 47 | +0.17% | +0.32% | 72.3% | 27.7% | +0.49% | -0.37% | 0.0% |
| SHORT Watch rank_score Q3 | H180 | 47 | +0.15% | +0.39% | 63.8% | 36.2% | +0.65% | -0.49% | 0.0% |
| SHORT Watch rank_score Q3 | H300 | 47 | +0.21% | +0.49% | 68.1% | 31.9% | +0.75% | -0.57% | 0.0% |

| SHORT Watch rank_score Q4 | H15 | 47 | +0.09% | +0.10% | 63.8% | 34.0% | +0.22% | -0.15% | 0.0% |
| SHORT Watch rank_score Q4 | H30 | 47 | +0.10% | +0.08% | 61.7% | 38.3% | +0.30% | -0.22% | 0.0% |
| SHORT Watch rank_score Q4 | H60 | 47 | +0.13% | +0.14% | 66.0% | 34.0% | +0.39% | -0.28% | 0.0% |
| SHORT Watch rank_score Q4 | H120 | 47 | +0.21% | +0.22% | 63.8% | 36.2% | +0.52% | -0.36% | 0.0% |
| SHORT Watch rank_score Q4 | H180 | 47 | +0.15% | +0.07% | 55.3% | 42.6% | +0.61% | -0.46% | 0.0% |
| SHORT Watch rank_score Q4 | H300 | 47 | +0.30% | +0.24% | 63.8% | 36.2% | +0.76% | -0.52% | 0.0% |


---

## 4. SHORT Watch — intraday decomposition by target_rate quartile

| Cohort | Horizon | N | Mean | Median | Win% | Loss% | Mean MFE | Mean MAE | Extreme% |
|---|---|---|---|---|---|---|---|---|---|
| SHORT Watch target_rate Q1 | H15 | 46 | +0.07% | +0.05% | 56.5% | 41.3% | +0.17% | -0.09% | 0.0% |
| SHORT Watch target_rate Q1 | H30 | 46 | +0.05% | +0.03% | 58.7% | 39.1% | +0.21% | -0.17% | 0.0% |
| SHORT Watch target_rate Q1 | H60 | 46 | +0.07% | +0.06% | 58.7% | 41.3% | +0.28% | -0.24% | 0.0% |
| SHORT Watch target_rate Q1 | H120 | 46 | +0.15% | +0.14% | 65.2% | 34.8% | +0.42% | -0.31% | 0.0% |
| SHORT Watch target_rate Q1 | H180 | 46 | +0.11% | +0.06% | 54.3% | 45.7% | +0.53% | -0.41% | 0.0% |
| SHORT Watch target_rate Q1 | H300 | 46 | +0.16% | +0.27% | 71.7% | 28.3% | +0.63% | -0.53% | 0.0% |

| SHORT Watch target_rate Q2 | H15 | 47 | +0.07% | +0.09% | 66.0% | 29.8% | +0.18% | -0.11% | 0.0% |
| SHORT Watch target_rate Q2 | H30 | 47 | +0.10% | +0.10% | 72.3% | 27.7% | +0.30% | -0.19% | 0.0% |
| SHORT Watch target_rate Q2 | H60 | 47 | +0.08% | +0.12% | 66.0% | 34.0% | +0.39% | -0.29% | 0.0% |
| SHORT Watch target_rate Q2 | H120 | 47 | +0.12% | +0.21% | 68.1% | 31.9% | +0.51% | -0.39% | 0.0% |
| SHORT Watch target_rate Q2 | H180 | 47 | +0.05% | +0.20% | 59.6% | 40.4% | +0.58% | -0.50% | 0.0% |
| SHORT Watch target_rate Q2 | H300 | 47 | +0.12% | +0.32% | 61.7% | 38.3% | +0.68% | -0.57% | 0.0% |

| SHORT Watch target_rate Q3 | H15 | 47 | +0.08% | +0.10% | 66.0% | 29.8% | +0.26% | -0.19% | 0.0% |
| SHORT Watch target_rate Q3 | H30 | 47 | +0.09% | +0.10% | 63.8% | 36.2% | +0.36% | -0.29% | 0.0% |
| SHORT Watch target_rate Q3 | H60 | 47 | +0.15% | +0.19% | 66.0% | 34.0% | +0.48% | -0.36% | 0.0% |
| SHORT Watch target_rate Q3 | H120 | 47 | +0.20% | +0.23% | 63.8% | 36.2% | +0.64% | -0.47% | 0.0% |
| SHORT Watch target_rate Q3 | H180 | 47 | +0.19% | +0.17% | 57.4% | 42.6% | +0.70% | -0.51% | 0.0% |
| SHORT Watch target_rate Q3 | H300 | 47 | +0.35% | +0.19% | 66.0% | 34.0% | +0.92% | -0.59% | 0.0% |

| SHORT Watch target_rate Q4 | H15 | 47 | +0.10% | +0.11% | 59.6% | 36.2% | +0.27% | -0.16% | 0.0% |
| SHORT Watch target_rate Q4 | H30 | 47 | +0.18% | +0.09% | 68.1% | 29.8% | +0.37% | -0.21% | 0.0% |
| SHORT Watch target_rate Q4 | H60 | 47 | +0.21% | +0.17% | 68.1% | 31.9% | +0.49% | -0.27% | 0.0% |
| SHORT Watch target_rate Q4 | H120 | 47 | +0.23% | +0.08% | 55.3% | 44.7% | +0.63% | -0.37% | 0.0% |
| SHORT Watch target_rate Q4 | H180 | 47 | +0.18% | +0.14% | 57.4% | 40.4% | +0.80% | -0.54% | 0.0% |
| SHORT Watch target_rate Q4 | H300 | 47 | +0.25% | +0.28% | 55.3% | 44.7% | +1.04% | -0.69% | 0.0% |


---

## 5. LONG Watch — intraday decomposition by sample_size band

| Cohort | Horizon | N | Mean | Median | Win% | Loss% | Mean MFE | Mean MAE | Extreme% |
|---|---|---|---|---|---|---|---|---|---|
| LONG Watch sample 1–50 | H15 | 18 | -0.02% | -0.14% | 33.3% | 66.7% | +0.23% | -0.26% | 0.0% |
| LONG Watch sample 1–50 | H30 | 18 | -0.02% | -0.13% | 38.9% | 61.1% | +0.33% | -0.40% | 0.0% |
| LONG Watch sample 1–50 | H60 | 18 | -0.03% | -0.28% | 44.4% | 55.6% | +0.47% | -0.48% | 0.0% |
| LONG Watch sample 1–50 | H120 | 18 | -0.17% | -0.31% | 38.9% | 61.1% | +0.50% | -0.67% | 0.0% |
| LONG Watch sample 1–50 | H180 | 18 | -0.21% | -0.25% | 44.4% | 55.6% | +0.58% | -0.78% | 0.0% |
| LONG Watch sample 1–50 | H300 | 18 | -0.45% | -0.28% | 33.3% | 61.1% | +0.60% | -0.97% | 0.0% |

| LONG Watch sample 51–100 | H15 | 54 | -0.11% | -0.07% | 31.5% | 68.5% | +0.10% | -0.20% | 0.0% |
| LONG Watch sample 51–100 | H30 | 54 | -0.06% | -0.16% | 29.6% | 68.5% | +0.24% | -0.30% | 0.0% |
| LONG Watch sample 51–100 | H60 | 54 | -0.11% | -0.17% | 31.5% | 66.7% | +0.30% | -0.42% | 0.0% |
| LONG Watch sample 51–100 | H120 | 54 | -0.12% | -0.14% | 33.3% | 66.7% | +0.40% | -0.54% | 0.0% |
| LONG Watch sample 51–100 | H180 | 54 | -0.08% | -0.21% | 40.7% | 59.3% | +0.48% | -0.64% | 1.9% |
| LONG Watch sample 51–100 | H300 | 54 | -0.22% | -0.28% | 33.3% | 66.7% | +0.54% | -0.81% | 1.9% |

| LONG Watch sample 101–150 | H15 | 137 | -0.11% | -0.10% | 29.2% | 68.6% | +0.09% | -0.18% | 0.0% |
| LONG Watch sample 101–150 | H30 | 137 | -0.07% | -0.09% | 36.5% | 60.6% | +0.17% | -0.28% | 0.0% |
| LONG Watch sample 101–150 | H60 | 137 | -0.06% | -0.05% | 43.8% | 53.3% | +0.30% | -0.38% | 0.0% |
| LONG Watch sample 101–150 | H120 | 137 | -0.11% | -0.17% | 38.7% | 60.6% | +0.45% | -0.51% | 0.0% |
| LONG Watch sample 101–150 | H180 | 137 | -0.16% | -0.11% | 43.1% | 56.9% | +0.52% | -0.62% | 0.0% |
| LONG Watch sample 101–150 | H300 | 137 | -0.26% | -0.29% | 38.0% | 62.0% | +0.59% | -0.77% | 0.0% |

| LONG Watch sample 151+ | H15 | 21 | -0.05% | -0.06% | 38.1% | 57.1% | +0.09% | -0.13% | 0.0% |
| LONG Watch sample 151+ | H30 | 21 | -0.05% | -0.08% | 42.9% | 57.1% | +0.14% | -0.21% | 0.0% |
| LONG Watch sample 151+ | H60 | 21 | -0.10% | +0.01% | 52.4% | 47.6% | +0.24% | -0.31% | 0.0% |
| LONG Watch sample 151+ | H120 | 21 | +0.09% | -0.16% | 47.6% | 52.4% | +0.49% | -0.40% | 0.0% |
| LONG Watch sample 151+ | H180 | 21 | +0.15% | -0.08% | 42.9% | 57.1% | +0.57% | -0.44% | 0.0% |
| LONG Watch sample 151+ | H300 | 21 | +0.09% | -0.25% | 42.9% | 57.1% | +0.66% | -0.53% | 0.0% |


---

## 6. LONG Watch — intraday decomposition by rank_score quartile

| Cohort | Horizon | N | Mean | Median | Win% | Loss% | Mean MFE | Mean MAE | Extreme% |
|---|---|---|---|---|---|---|---|---|---|
| LONG Watch rank_score Q1 | H15 | 57 | -0.07% | -0.08% | 33.3% | 63.2% | +0.10% | -0.15% | 0.0% |
| LONG Watch rank_score Q1 | H30 | 57 | -0.04% | -0.04% | 38.6% | 59.6% | +0.18% | -0.24% | 0.0% |
| LONG Watch rank_score Q1 | H60 | 57 | -0.04% | -0.02% | 49.1% | 50.9% | +0.31% | -0.35% | 0.0% |
| LONG Watch rank_score Q1 | H120 | 57 | -0.05% | -0.15% | 45.6% | 54.4% | +0.46% | -0.49% | 0.0% |
| LONG Watch rank_score Q1 | H180 | 57 | -0.12% | -0.28% | 40.4% | 59.6% | +0.53% | -0.59% | 0.0% |
| LONG Watch rank_score Q1 | H300 | 57 | -0.23% | -0.41% | 36.8% | 63.2% | +0.62% | -0.74% | 0.0% |

| LONG Watch rank_score Q2 | H15 | 58 | -0.11% | -0.10% | 25.9% | 72.4% | +0.09% | -0.19% | 0.0% |
| LONG Watch rank_score Q2 | H30 | 58 | -0.13% | -0.16% | 29.3% | 69.0% | +0.19% | -0.32% | 0.0% |
| LONG Watch rank_score Q2 | H60 | 58 | -0.10% | -0.17% | 31.0% | 67.2% | +0.30% | -0.43% | 0.0% |
| LONG Watch rank_score Q2 | H120 | 58 | -0.19% | -0.21% | 29.3% | 69.0% | +0.41% | -0.55% | 0.0% |
| LONG Watch rank_score Q2 | H180 | 58 | -0.18% | -0.13% | 41.4% | 58.6% | +0.45% | -0.62% | 0.0% |
| LONG Watch rank_score Q2 | H300 | 58 | -0.34% | -0.31% | 31.0% | 69.0% | +0.48% | -0.76% | 0.0% |

| LONG Watch rank_score Q3 | H15 | 57 | -0.14% | -0.14% | 33.3% | 64.9% | +0.10% | -0.23% | 0.0% |
| LONG Watch rank_score Q3 | H30 | 57 | -0.12% | -0.08% | 42.1% | 56.1% | +0.16% | -0.34% | 0.0% |
| LONG Watch rank_score Q3 | H60 | 57 | -0.14% | -0.06% | 45.6% | 52.6% | +0.27% | -0.42% | 0.0% |
| LONG Watch rank_score Q3 | H120 | 57 | -0.10% | -0.17% | 35.1% | 64.9% | +0.47% | -0.55% | 0.0% |
| LONG Watch rank_score Q3 | H180 | 57 | -0.17% | -0.32% | 40.4% | 59.6% | +0.54% | -0.68% | 0.0% |
| LONG Watch rank_score Q3 | H300 | 57 | -0.24% | -0.38% | 38.6% | 61.4% | +0.64% | -0.85% | 0.0% |

| LONG Watch rank_score Q4 | H15 | 58 | -0.06% | -0.10% | 31.0% | 69.0% | +0.12% | -0.18% | 0.0% |
| LONG Watch rank_score Q4 | H30 | 58 | +0.03% | -0.10% | 32.8% | 63.8% | +0.26% | -0.26% | 0.0% |
| LONG Watch rank_score Q4 | H60 | 58 | -0.01% | -0.03% | 41.4% | 53.4% | +0.35% | -0.35% | 0.0% |
| LONG Watch rank_score Q4 | H120 | 58 | -0.07% | -0.13% | 43.1% | 56.9% | +0.45% | -0.50% | 0.0% |
| LONG Watch rank_score Q4 | H180 | 58 | -0.01% | -0.01% | 48.3% | 51.7% | +0.55% | -0.59% | 1.7% |
| LONG Watch rank_score Q4 | H300 | 58 | -0.14% | -0.16% | 41.4% | 56.9% | +0.61% | -0.75% | 1.7% |


---

## 7. Extreme-move analysis — which decision characteristics predict large intraday moves?

Extreme move = |intraday return| > 5% at horizon.

| Cohort | Horizon | N | Extreme% | Mean return (extreme) | Mean return (non-extreme) |
|---|---|---|---|---|---|
| All Watch | H15 | 417 | 0.0% | — | -0.02% |
| All Watch | H30 | 417 | 0.0% | — | +0.01% |
| All Watch | H60 | 417 | 0.0% | — | +0.02% |
| All Watch | H120 | 417 | 0.0% | — | +0.02% |
| All Watch | H180 | 417 | 0.2% | +6.65% | -0.02% |
| All Watch | H300 | 417 | 0.2% | +7.14% | -0.05% |
| LONG Watch | H15 | 230 | 0.0% | — | -0.10% |
| LONG Watch | H30 | 230 | 0.0% | — | -0.06% |
| LONG Watch | H60 | 230 | 0.0% | — | -0.07% |
| LONG Watch | H120 | 230 | 0.0% | — | -0.10% |
| LONG Watch | H180 | 230 | 0.4% | +6.65% | -0.15% |
| LONG Watch | H300 | 230 | 0.4% | +7.14% | -0.27% |
| SHORT Watch | H15 | 187 | 0.0% | — | +0.08% |
| SHORT Watch | H30 | 187 | 0.0% | — | +0.11% |
| SHORT Watch | H60 | 187 | 0.0% | — | +0.13% |
| SHORT Watch | H120 | 187 | 0.0% | — | +0.17% |
| SHORT Watch | H180 | 187 | 0.0% | — | +0.14% |
| SHORT Watch | H300 | 187 | 0.0% | — | +0.22% |

---

## 8. Best intraday combinations — candidate policy matrix

Searching combinations of direction × quality variable × horizon.
Ranked by median return (gap-through excluded: |return| <= 5%).

| Combination | Horizon | N | Mean | Median | Win% | Loss% | Extreme% |
|---|---|---|---|---|---|---|---|
| SHORT Watch Exact | H15 | 83 | +0.08% | +0.10% | 63.9% | 31.3% | 0.0% |
| SHORT Watch Exact | H60 | 83 | +0.09% | +0.14% | 63.9% | 36.1% | 0.0% |
| SHORT Watch Exact | H300 | 83 | +0.23% | +0.40% | 65.1% | 34.9% | 0.0% |
| SHORT Watch sample 51–150 | H15 | 92 | +0.06% | +0.10% | 64.1% | 30.4% | 0.0% |
| SHORT Watch sample 51–150 | H60 | 92 | +0.10% | +0.15% | 64.1% | 35.9% | 0.0% |
| SHORT Watch sample 51–150 | H300 | 92 | +0.19% | +0.29% | 62.0% | 38.0% | 0.0% |
| SHORT Watch sample 151+ | H15 | 75 | +0.12% | +0.09% | 60.0% | 37.3% | 0.0% |
| SHORT Watch sample 151+ | H60 | 75 | +0.19% | +0.08% | 68.0% | 32.0% | 0.0% |
| SHORT Watch sample 151+ | H300 | 75 | +0.26% | +0.27% | 66.7% | 33.3% | 0.0% |
| SHORT Watch target_rate Q2 | H15 | 47 | +0.07% | +0.09% | 66.0% | 29.8% | 0.0% |
| SHORT Watch target_rate Q2 | H60 | 47 | +0.08% | +0.12% | 66.0% | 34.0% | 0.0% |
| SHORT Watch target_rate Q2 | H300 | 47 | +0.12% | +0.32% | 61.7% | 38.3% | 0.0% |
| SHORT Watch target_rate Q3 | H15 | 47 | +0.08% | +0.10% | 66.0% | 29.8% | 0.0% |
| SHORT Watch target_rate Q3 | H60 | 47 | +0.15% | +0.19% | 66.0% | 34.0% | 0.0% |
| SHORT Watch target_rate Q3 | H300 | 47 | +0.35% | +0.19% | 66.0% | 34.0% | 0.0% |
| LONG Watch sample 151+ | H15 | 21 | -0.05% | -0.06% | 38.1% | 57.1% | 0.0% |
| LONG Watch sample 151+ | H60 | 21 | -0.10% | +0.01% | 52.4% | 47.6% | 0.0% |
| LONG Watch sample 151+ | H300 | 21 | +0.09% | -0.25% | 42.9% | 57.1% | 0.0% |
| LONG Watch rank_score Q3 | H15 | 57 | -0.14% | -0.14% | 33.3% | 64.9% | 0.0% |
| LONG Watch rank_score Q3 | H60 | 57 | -0.14% | -0.06% | 45.6% | 52.6% | 0.0% |
| LONG Watch rank_score Q3 | H300 | 57 | -0.24% | -0.38% | 38.6% | 61.4% | 0.0% |
| All Watch sample 51–150 | H15 | 283 | -0.05% | -0.04% | 41.0% | 56.2% | 0.0% |
| All Watch sample 51–150 | H60 | 283 | -0.02% | -0.01% | 48.1% | 50.2% | 0.0% |
| All Watch sample 51–150 | H300 | 283 | -0.11% | -0.10% | 44.9% | 55.1% | 0.4% |

---

## 9. Intraday vs daily return comparison

Does the intraday path at H300 match the daily realized_return?

| Cohort | N | Daily mean | H300 mean | Daily median | H300 median |
|---|---|---|---|---|---|
| All Watch | 417 | +2.24% | -0.03% | +0.07% | -0.03% |
| LONG Watch | 230 | -12.29% | -0.24% | -0.42% | -0.29% |
| SHORT Watch | 187 | +20.11% | +0.22% | +0.95% | +0.27% |

---

## Interpretation notes

- All results are in-sample on the 300 COMPLETE historical development set.
- Intraday returns are direction-adjusted: LONG profits from price rise, SHORT from price fall.
- Extreme move threshold: |return| > 5% at horizon.
- MFE = maximum favourable excursion over horizon bars.
- MAE = maximum adverse excursion over horizon bars.
- Policy must be frozen before seeing each observation's future intraday path.
- TIME-009 observations used as outcome data only — not a tuning target.

**Next step:** Identify strongest intraday candidates, lock policy, walk-forward on Aug 24+ cohorts.