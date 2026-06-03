# Phase 5B: Controlled Perturbation Experiments

We injected structural perturbations (latency, missed fill probability) into the replay engine across a 20-session stratified sample, and measured execution degradation.

## Fill Rate by Volatility and Perturbation
| Profile | Low Volatility | Medium Volatility | High Volatility |
|---------|----------------|-------------------|-----------------|
| Baseline | 100.00% | 100.00% | 100.00% |
| Low Latency | 100.00% | 100.00% | 100.00% |
| High Latency | 99.91% | 99.95% | 100.00% |
| Degraded Queue | 94.76% | 94.97% | 96.23% |

## Effective Slippage (bps) by Volatility and Perturbation
| Profile | Low Volatility | Medium Volatility | High Volatility |
|---------|----------------|-------------------|-----------------|
| Baseline | 0.50 | 0.50 | 0.50 |
| Low Latency | 7.24 | 8.42 | 11.59 |
| High Latency | 67.91 | 79.68 | 111.43 |
| Degraded Queue | 7.23 | 8.42 | 11.59 |

## Findings
> [!IMPORTANT]
> **Monotonic Degradation**: Execution metrics monotonically degrade as latency increases.
> **Environmental Amplification**: The degradation is non-linear with respect to environmental geometry. High volatility sessions suffer significantly higher slippage per millisecond of latency compared to low volatility sessions.
