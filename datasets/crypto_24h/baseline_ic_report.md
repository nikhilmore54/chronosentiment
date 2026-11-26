Loading dataset from /Users/nikhil/ChronoSentiment_MEGA_FINAL/datasets/crypto_24h/ic_discovery_BTCUSDT.csv...
Split: Train 24876 | Validate 8292 | Holdout 8293
Analyzing volatility_1m_std_24h...
Analyzing upside_excursion_24h...
Analyzing downside_excursion_24h...
Analyzing trend_dir...

# Train Set Baseline IC Research

| Feature | Horizon | Spearman | Pearson | HitRate | Mono | Q1 Mean Ret | Q5 Mean Ret | Pos Blocks % |
|---|---|---|---|---|---|---|---|---|
| volatility_1m_std_24h | 15m | 0.0039 | -0.0014 | 50.9% | Mixed | 0.01% | 0.00% | 45.7% (35 blocks) |
| volatility_1m_std_24h | 30m | 0.0027 | -0.0013 | 50.5% | Mixed | 0.02% | 0.01% | 40.0% (35 blocks) |
| volatility_1m_std_24h | 60m | 0.0089 | 0.0001 | 50.1% | Mixed | 0.04% | 0.02% | 40.0% (35 blocks) |
| volatility_1m_std_24h | 120m | 0.0357 | 0.0036 | 51.2% | Mixed | 0.07% | 0.06% | 34.3% (35 blocks) |
| volatility_1m_std_24h | 300m | 0.0718 | 0.0026 | 53.5% | Mixed | 0.17% | 0.15% | 51.4% (35 blocks) |
| upside_excursion_24h | 15m | -0.0379 | -0.0188 | 48.2% | Mixed | 0.01% | -0.01% | 40.0% (35 blocks) |
| upside_excursion_24h | 30m | -0.0534 | -0.0290 | 48.1% | Mixed | 0.02% | -0.01% | 42.9% (35 blocks) |
| upside_excursion_24h | 60m | -0.0650 | -0.0478 | 46.9% | Mixed | 0.03% | -0.04% | 37.1% (35 blocks) |
| upside_excursion_24h | 120m | -0.0953 | -0.0657 | 45.8% | Mixed | 0.06% | -0.08% | 40.0% (35 blocks) |
| upside_excursion_24h | 300m | -0.1212 | -0.0832 | 42.9% | Mixed | 0.04% | -0.15% | 40.0% (35 blocks) |
| downside_excursion_24h | 15m | -0.0491 | -0.0368 | 47.6% | Mixed | 0.01% | -0.01% | 34.3% (35 blocks) |
| downside_excursion_24h | 30m | -0.0653 | -0.0551 | 46.5% | Mixed | 0.01% | -0.01% | 40.0% (35 blocks) |
| downside_excursion_24h | 60m | -0.0851 | -0.0858 | 46.5% | Mixed | 0.04% | -0.04% | 40.0% (35 blocks) |
| downside_excursion_24h | 120m | -0.1373 | -0.1262 | 45.3% | Mixed | 0.08% | -0.10% | 34.3% (35 blocks) |
| downside_excursion_24h | 300m | -0.1867 | -0.1790 | 44.3% | Mixed | 0.20% | -0.11% | 42.9% (35 blocks) |
| trend_dir | 15m | -0.0313 | -0.0116 | 21.5% | Mixed | 0.00% | -0.00% | 11.1% (18 blocks) |
| trend_dir | 30m | -0.0359 | -0.0149 | 21.4% | Mixed | 0.00% | -0.01% | 16.7% (18 blocks) |
| trend_dir | 60m | -0.0354 | -0.0296 | 21.0% | Mixed | 0.01% | -0.02% | 16.7% (18 blocks) |
| trend_dir | 120m | -0.0912 | -0.0756 | 19.0% | Down | 0.03% | -0.03% | 0.0% (18 blocks) |
| trend_dir | 300m | -0.0862 | -0.0760 | 19.4% | Mixed | 0.04% | -0.04% | 5.6% (18 blocks) |
