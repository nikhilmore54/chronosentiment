# Crypto Feature Census v0.1
## Stage A: Orthogonality Audit (Train Split)

### 1. Descriptive Statistics of New Dimensions
| Feature | Mean | Std Dev | Min | Max | Missing Rate |
|---|---|---|---|---|---|
| `rvol_60m_24h` | 1.0356 | 0.6859 | 0.1476 | 7.1549 | 0.0% |
| `volume_acceleration_60m` | 1.2145 | 0.8744 | 0.0587 | 14.5932 | 0.0% |
| `price_volume_alignment_60m` | 0.0272 | 0.2554 | -0.9214 | 0.9011 | 0.0% |
| `avg_range_pct_60m` | 0.0004 | 0.0002 | 0.0000 | 0.0016 | 0.0% |
| `wick_asymmetry_60m` | 0.5043 | 0.1430 | 0.0074 | 0.9947 | 0.0% |
| `close_location_60m` | 0.4973 | 0.3012 | 0.0000 | 1.0000 | 0.0% |
| `trend_strength_60m` | 5.9915 | 4.7908 | 0.0000 | 46.6731 | 0.0% |

### 2. Cross-Correlation Among New Features (Pearson)
| Feature | `rvol_60m_24h` | `volume_acceleration_60m` | `price_volume_alignment_60m` | `avg_range_pct_60m` | `wick_asymmetry_60m` | `close_location_60m` | `trend_strength_60m` |
|---|---|---|---|---|---|---|---|
| `rvol_60m_24h` | 1.00 | 0.12 | -0.06 | 0.61 | 0.03 | 0.02 | 0.01 |
| `volume_acceleration_60m` | 0.12 | 1.00 | 0.00 | 0.00 | 0.01 | 0.02 | 0.15 |
| `price_volume_alignment_60m` | -0.06 | 0.00 | 1.00 | -0.00 | 0.29 | 0.22 | -0.02 |
| `avg_range_pct_60m` | 0.61 | 0.00 | -0.00 | 1.00 | -0.03 | -0.00 | -0.12 |
| `wick_asymmetry_60m` | 0.03 | 0.01 | 0.29 | -0.03 | 1.00 | 0.19 | -0.02 |
| `close_location_60m` | 0.02 | 0.02 | 0.22 | -0.00 | 0.19 | 1.00 | 0.02 |
| `trend_strength_60m` | 0.01 | 0.15 | -0.02 | -0.12 | -0.02 | 0.02 | 1.00 |

### 3. Pearson Correlation vs Existing 13-field State
| New Feature | `volatility_1m_std_24h` | `upside_excursion_24h` | `downside_excursion_24h` | `trend_dir` | `trend_return_15m` | `trend_return_60m` | `trend_return_240m` | `volatility_std_60m` | `volatility_std_240m` | `volatility_ratio_60m_24h` | `volatility_ratio_240m_24h` | `persistence_60m` | `persistence_240m` |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `rvol_60m_24h` | 0.01 | 0.06 | 0.05 | -0.00 | -0.02 | -0.02 | -0.10 | 0.59 | 0.35 | 0.63 | 0.42 | 0.12 | 0.32 |
| `volume_acceleration_60m` | -0.04 | -0.02 | 0.01 | 0.01 | 0.02 | 0.02 | -0.01 | 0.06 | -0.06 | 0.10 | -0.03 | 0.18 | 0.11 |
| `price_volume_alignment_60m` | -0.01 | 0.18 | 0.13 | 0.28 | 0.19 | 0.48 | 0.35 | 0.02 | 0.03 | 0.07 | 0.08 | -0.03 | -0.03 |
| `avg_range_pct_60m` | 0.46 | 0.18 | -0.09 | -0.04 | -0.02 | -0.06 | -0.13 | 0.96 | 0.80 | 0.72 | 0.55 | 0.03 | 0.14 |
| `wick_asymmetry_60m` | -0.11 | 0.01 | 0.10 | 0.16 | 0.12 | 0.32 | 0.21 | -0.03 | -0.03 | 0.05 | 0.07 | -0.02 | 0.08 |
| `close_location_60m` | 0.01 | -0.04 | -0.07 | 0.03 | 0.60 | 0.67 | 0.28 | -0.00 | 0.03 | 0.01 | 0.04 | 0.01 | -0.02 |
| `trend_strength_60m` | -0.15 | -0.08 | 0.03 | -0.00 | 0.01 | -0.01 | -0.01 | -0.10 | -0.20 | -0.03 | -0.14 | 0.97 | 0.18 |

### 4. Spearman (Rank) Correlation vs Existing 13-field State
| New Feature | `volatility_1m_std_24h` | `upside_excursion_24h` | `downside_excursion_24h` | `trend_dir` | `trend_return_15m` | `trend_return_60m` | `trend_return_240m` | `volatility_std_60m` | `volatility_std_240m` | `volatility_ratio_60m_24h` | `volatility_ratio_240m_24h` | `persistence_60m` | `persistence_240m` |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `rvol_60m_24h` | -0.01 | 0.13 | 0.14 | 0.09 | -0.01 | 0.01 | -0.04 | 0.57 | 0.40 | 0.67 | 0.50 | 0.08 | 0.28 |
| `volume_acceleration_60m` | -0.01 | -0.00 | 0.01 | 0.01 | 0.04 | 0.03 | 0.01 | 0.01 | -0.05 | 0.01 | -0.06 | 0.18 | 0.05 |
| `price_volume_alignment_60m` | 0.03 | 0.19 | 0.15 | 0.28 | 0.12 | 0.45 | 0.35 | 0.06 | 0.04 | 0.09 | 0.08 | -0.01 | -0.01 |
| `avg_range_pct_60m` | 0.53 | 0.22 | -0.11 | -0.01 | -0.01 | -0.01 | -0.04 | 0.97 | 0.85 | 0.69 | 0.55 | 0.01 | 0.09 |
| `wick_asymmetry_60m` | -0.07 | 0.02 | 0.07 | 0.16 | 0.11 | 0.38 | 0.26 | -0.03 | -0.04 | 0.04 | 0.06 | -0.02 | 0.07 |
| `close_location_60m` | 0.01 | -0.06 | -0.07 | 0.03 | 0.67 | 0.76 | 0.29 | 0.01 | 0.02 | 0.01 | 0.03 | 0.02 | -0.02 |
| `trend_strength_60m` | -0.11 | -0.05 | 0.02 | -0.00 | -0.01 | 0.03 | -0.00 | -0.09 | -0.15 | -0.02 | -0.09 | 0.99 | 0.14 |
