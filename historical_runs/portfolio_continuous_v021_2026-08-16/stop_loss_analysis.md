# Stop-Loss Behaviour Analysis — Portfolio Replay v0.2.1
**Source:** `historical_runs/portfolio_continuous_v021_2026-08-16/continuous_ledger.json` (IMMUTABLE)  **Purpose:** Understand the 70 Coralys STOP exits — not to tune the stop-loss  **Constraint:** Coralys v0 parameters are frozen. This is an observation exercise.
## Summary
Total STOP exits: **70**
| Category | Count | % | Avg realized return | Avg opportunity cost |
|----------|-------|---|---------------------|----------------------|
| PREMATURE_STOP | 33 | 47% | -1.44% | -58.64% |
| TEMPORARY_EXCURSION | 24 | 34% | -2.15% | +16.11% |
| STOP_TOO_TIGHT | 8 | 11% | -1.67% | -5.35% |
| GENUINE_ADVERSE | 3 | 4% | -2.32% | -2.31% |
| DIRECTION_FAILURE | 2 | 3% | -2.70% | -4.29% |

**Total realized P&L from stopped lots:** Rs.-554.65  
**Total opportunity cost (hold-to-horizon counterfactual):** Rs.-3626.94  
**Net stop impact vs hold-to-horizon:** Rs.+3072.29  

## Premature Stops (33) — Target was reachable after stop
These are the most actionable: the stop prevented capturing a move that subsequently occurred.

| Trade ID | Instrument | Entry | Stop | Target | Exit | Hold | Realized | Opp Cost |
|----------|------------|-------|------|--------|------|------|----------|----------|
| coralys-TCS-seq3 | TCS.NS | 2269.00 | 2213.83 (2.4%) | 2379.34 (4.9%) | 2213.83 | 3s | Rs.-17.86 | Rs.+29.81 |
| coralys-TCS-seq4 | TCS.NS | 2251.10 | 2198.46 (2.3%) | 2356.39 (4.7%) | 2198.46 | 3s | Rs.-17.00 | Rs.+35.52 |
| coralys-INFY-seq3 | INFY.NS | 1096.50 | 1071.43 (2.3%) | 1146.64 (4.6%) | 1071.43 | 2s | Rs.-16.79 | Rs.+49.44 |
| coralys-INFY-seq4 | INFY.NS | 1087.10 | 1063.11 (2.2%) | 1135.09 (4.4%) | 1063.11 | 2s | Rs.-16.05 | Rs.+55.64 |
| coralys-INFY-seq5 | INFY.NS | 1073.50 | 1049.74 (2.2%) | 1121.03 (4.4%) | 1049.74 | 1s | Rs.-16.02 | Rs.+65.27 |
| coralys-IDEA-seq1 | IDEA.NS | 13.67 | 13.96 (2.1%) | 13.10 (4.2%) | 13.96 | 1s | Rs.-14.95 | Rs.+8.88 |
| coralys-INFY-seq7 | INFY.NS | 1047.40 | 1025.44 (2.1%) | 1091.32 (4.2%) | 1025.44 | 1s | Rs.-13.16 | Rs.+73.66 |
| coralys-INFY-seq6 | INFY.NS | 1052.10 | 1037.41 (1.4%) | 1081.47 (2.8%) | 1037.41 | 1s | Rs.-10.20 | Rs.+82.12 |
| coralys-IDEA-seq6 | IDEA.NS | 13.54 | 13.36 (1.3%) | 13.90 (2.7%) | 13.36 | 1s | Rs.-9.79 | Rs.-730.95 |
| coralys-IDEA-seq5 | IDEA.NS | 13.49 | 13.31 (1.4%) | 13.85 (2.7%) | 13.31 | 1s | Rs.-9.77 | Rs.-723.87 |
| coralys-IDEA-seq7 | IDEA.NS | 13.20 | 13.02 (1.4%) | 13.56 (2.7%) | 13.02 | 1s | Rs.-8.59 | Rs.-627.80 |
| coralys-RELIANCE-seq6 | RELIANCE.NS | 1288.60 | 1275.71 (1.0%) | 1314.37 (2.0%) | 1275.71 | 1s | Rs.-7.31 | Rs.+11.40 |
| coralys-RELIANCE-seq5 | RELIANCE.NS | 1303.70 | 1290.66 (1.0%) | 1329.77 (2.0%) | 1290.66 | 1s | Rs.-7.24 | Rs.+2.78 |
| coralys-RELIANCE-seq7 | RELIANCE.NS | 1272.20 | 1259.48 (1.0%) | 1297.64 (2.0%) | 1259.48 | 1s | Rs.-6.28 | Rs.+18.01 |
| coralys-IDEA-seq8 | IDEA.NS | 13.13 | 12.95 (1.4%) | 13.49 (2.7%) | 12.95 | 2s | Rs.-6.07 | Rs.-445.73 |
| coralys-MAHABANK-seq10 | MAHABANK.NS | 80.20 | 79.02 (1.5%) | 82.56 (2.9%) | 79.02 | 3s | Rs.-5.67 | Rs.-385.78 |
| coralys-IDEA-seq9 | IDEA.NS | 13.06 | 12.89 (1.3%) | 13.41 (2.7%) | 12.89 | 3s | Rs.-5.11 | Rs.-384.34 |
| coralys-IDEA-seq10 | IDEA.NS | 12.95 | 12.78 (1.3%) | 13.28 (2.6%) | 12.78 | 3s | Rs.-4.95 | Rs.-385.78 |
| coralys-MAHABANK-seq11 | MAHABANK.NS | 80.73 | 79.57 (1.4%) | 83.05 (2.9%) | 79.57 | 1s | Rs.-4.76 | Rs.-330.67 |
| coralys-IDEA-seq11 | IDEA.NS | 13.05 | 12.88 (1.3%) | 13.38 (2.6%) | 12.88 | 1s | Rs.-4.23 | Rs.-330.67 |
| coralys-RELIANCE-seq9 | RELIANCE.NS | 1280.00 | 1267.20 (1.0%) | 1305.60 (2.0%) | 1267.20 | 1s | Rs.-3.84 | Rs.+8.62 |
| coralys-IDEA-seq12 | IDEA.NS | 12.87 | 12.70 (1.3%) | 13.20 (2.6%) | 12.70 | 5s | Rs.-3.67 | Rs.-282.10 |
| coralys-MAHABANK-seq12 | MAHABANK.NS | 79.26 | 78.25 (1.3%) | 81.28 (2.6%) | 78.25 | 5s | Rs.-3.60 | Rs.-282.10 |
| coralys-IDEA-seq13 | IDEA.NS | 13.01 | 12.84 (1.3%) | 13.35 (2.6%) | 12.84 | 1s | Rs.-3.14 | Rs.-240.76 |
| coralys-MAHABANK-seq13 | MAHABANK.NS | 79.33 | 78.37 (1.2%) | 81.24 (2.4%) | 78.37 | 2s | Rs.-2.90 | Rs.-240.76 |
| coralys-IDEA-seq14 | IDEA.NS | 12.86 | 12.69 (1.3%) | 13.19 (2.6%) | 12.69 | 3s | Rs.-2.65 | Rs.-206.62 |
| coralys-IDEA-seq15 | IDEA.NS | 12.90 | 12.74 (1.2%) | 13.21 (2.4%) | 12.74 | 2s | Rs.-2.49 | Rs.-204.40 |
| coralys-MAHABANK-seq14 | MAHABANK.NS | 79.57 | 78.66 (1.1%) | 81.38 (2.3%) | 78.66 | 1s | Rs.-2.35 | Rs.-206.62 |
| coralys-MAHABANK-seq15 | MAHABANK.NS | 78.42 | 77.54 (1.1%) | 80.19 (2.3%) | 77.54 | 3s | Rs.-2.30 | Rs.-204.40 |
| coralys-RELIANCE-seq15 | RELIANCE.NS | 1290.90 | 1277.99 (1.0%) | 1316.72 (2.0%) | 1277.99 | 1s | Rs.-2.04 | Rs.+2.82 |
| coralys-IDEA-seq16 | IDEA.NS | 12.80 | 12.65 (1.2%) | 13.11 (2.4%) | 12.65 | 1s | Rs.-1.72 | Rs.-143.66 |
| coralys-MAHABANK-seq16 | MAHABANK.NS | 79.20 | 78.30 (1.1%) | 81.00 (2.3%) | 78.30 | 1s | Rs.-1.63 | Rs.-143.66 |
| coralys-MAHABANK-seq17 | MAHABANK.NS | 78.17 | 77.26 (1.2%) | 79.99 (2.3%) | 77.26 | 1s | Rs.-1.19 | Rs.-101.71 |

## Temporary Excursion Stops (24) — Price recovered after stop
| Trade ID | Instrument | Entry | Stop | Sessions to recovery | Realized | Opp Cost |
|----------|------------|-------|------|----------------------|----------|----------|
| coralys-MAHABANK-seq2 | MAHABANK.NS | 79.49 | 82.02 | 1 | Rs.-22.98 | Rs.-19.58 |
| coralys-MAHABANK-seq3 | MAHABANK.NS | 79.90 | 82.16 | 1 | Rs.-20.81 | Rs.+734.53 |
| coralys-MAHABANK-seq6 | MAHABANK.NS | 79.17 | 81.34 | 7 | Rs.-20.06 | Rs.+730.95 |
| coralys-MAHABANK-seq4 | MAHABANK.NS | 80.52 | 82.67 | 1 | Rs.-19.43 | Rs.+726.95 |
| coralys-MAHABANK-seq7 | MAHABANK.NS | 78.48 | 80.59 | 7 | Rs.-16.84 | Rs.+627.80 |
| coralys-ICICIBANK-seq5 | ICICIBANK.NS | 1450.87 | 1418.03 | 7 | Rs.-16.38 | Rs.-18.24 |
| coralys-ICICIBANK-seq4 | ICICIBANK.NS | 1447.99 | 1415.47 | 7 | Rs.-16.33 | Rs.-16.91 |
| coralys-MAHABANK-seq8 | MAHABANK.NS | 81.03 | 83.08 | 1 | Rs.-11.30 | Rs.+445.73 |
| coralys-TCS-seq11 | TCS.NS | 2446.60 | 2378.26 | 1 | Rs.-9.24 | Rs.-11.56 |
| coralys-RELIANCE-seq3 | RELIANCE.NS | 1327.20 | 1310.94 | 13 | Rs.-9.00 | Rs.-10.24 |
| coralys-TCS-seq10 | TCS.NS | 2398.00 | 2348.27 | 1 | Rs.-8.00 | Rs.-5.94 |
| coralys-TCS-seq12 | TCS.NS | 2431.80 | 2365.10 | 1 | Rs.-7.74 | Rs.-8.20 |
| coralys-RELIANCE-seq4 | RELIANCE.NS | 1323.10 | 1309.87 | 12 | Rs.-7.27 | Rs.-7.91 |
| coralys-INFY-seq11 | INFY.NS | 1155.60 | 1130.34 | 1 | Rs.-7.23 | Rs.+4.21 |
| coralys-HDFCBANK-seq8 | HDFCBANK.NS | 742.80 | 754.81 | 3 | Rs.-7.20 | Rs.+8.37 |
| coralys-TCS-seq13 | TCS.NS | 2365.60 | 2301.86 | 1 | Rs.-6.49 | Rs.-0.46 |
| coralys-MAHABANK-seq9 | MAHABANK.NS | 80.63 | 79.32 | 9 | Rs.-6.27 | Rs.-384.34 |
| coralys-INFY-seq12 | INFY.NS | 1155.10 | 1130.06 | 1 | Rs.-6.12 | Rs.+3.71 |
| coralys-HDFCBANK-seq9 | HDFCBANK.NS | 739.55 | 751.18 | 5 | Rs.-6.04 | Rs.+5.56 |
| coralys-HDFCBANK-seq10 | HDFCBANK.NS | 735.40 | 746.00 | 5 | Rs.-5.56 | Rs.+3.44 |
| coralys-INFY-seq14 | INFY.NS | 1180.00 | 1154.08 | 4 | Rs.-4.54 | Rs.-1.70 |
| coralys-RELIANCE-seq13 | RELIANCE.NS | 1307.80 | 1291.62 | 2 | Rs.-2.98 | Rs.+0.17 |
| coralys-TCS-seq17 | TCS.NS | 2373.00 | 2306.97 | 1 | Rs.-2.83 | Rs.-0.51 |
| coralys-RELIANCE-seq14 | RELIANCE.NS | 1319.00 | 1305.81 | 2 | Rs.-2.07 | Rs.-1.61 |

## Direction Failure Stops (2) — Stop correctly identified adverse move
| Trade ID | Instrument | Entry | Stop | Post-stop adverse | Realized |
|----------|------------|-------|------|-------------------|----------|
| coralys-TCS-seq14 | TCS.NS | 2473.70 | 2406.41 | -5.01% | Rs.-5.62 |
| coralys-TCS-seq15 | TCS.NS | 2460.00 | 2394.06 | -4.48% | Rs.-5.48 |

## Genuine Adverse Stops (3)
| Trade ID | Instrument | Stop dist | MAE | Post-stop adverse | Realized |
|----------|------------|-----------|-----|-------------------|----------|
| coralys-ICICIBANK-seq9 | ICICIBANK.NS | 2.17% | 1.87% | 0.00% | Rs.-8.36 |
| coralys-TCS-seq16 | TCS.NS | 2.76% | 2.62% | -2.15% | Rs.-3.97 |
| coralys-ICICIBANK-seq16 | ICICIBANK.NS | 2.02% | 2.01% | -2.99% | Rs.-2.90 |

## Stop Too Tight (8) — Stop distance < 2% from entry
| Trade ID | Instrument | Stop dist | MAE | Realized |
|----------|------------|-----------|-----|----------|
| coralys-HDFCBANK-seq17 | HDFCBANK.NS | 1.00% | 0.72% | Rs.-1.02 |
| coralys-RELIANCE-seq17 | RELIANCE.NS | 1.34% | 0.60% | Rs.-1.36 |
| coralys-HDFCBANK-seq1 | HDFCBANK.NS | 1.67% | 4.64% | Rs.-11.91 |
| coralys-HDFCBANK-seq3 | HDFCBANK.NS | 1.78% | 5.12% | Rs.-13.10 |
| coralys-HDFCBANK-seq2 | HDFCBANK.NS | 1.78% | 3.80% | Rs.-12.85 |
| coralys-ICICIBANK-seq17 | ICICIBANK.NS | 1.82% | 2.50% | Rs.-1.86 |
| coralys-ICICIBANK-seq15 | ICICIBANK.NS | 1.99% | 2.31% | Rs.-4.07 |
| coralys-ICICIBANK-seq14 | ICICIBANK.NS | 2.00% | 0.37% | Rs.-4.13 |

## Per-Instrument Stop Breakdown
| Instrument | Stops | Premature | Temp excursion | Gap-through | Direction fail | Genuine | Tight | Total P&L |
|------------|-------|-----------|----------------|-------------|----------------|---------|-------|-----------|
| HDFCBANK.NS | 7 | 0 | 3 | 0 | 0 | 0 | 4 | Rs.-57.69 |
| ICICIBANK.NS | 7 | 0 | 2 | 0 | 0 | 2 | 3 | Rs.-54.01 |
| IDEA.NS | 13 | 13 | 0 | 0 | 0 | 0 | 0 | Rs.-77.14 |
| INFY.NS | 8 | 5 | 3 | 0 | 0 | 0 | 0 | Rs.-90.11 |
| MAHABANK.NS | 15 | 8 | 7 | 0 | 0 | 0 | 0 | Rs.-142.09 |
| RELIANCE.NS | 10 | 5 | 4 | 0 | 0 | 0 | 1 | Rs.-49.39 |
| TCS.NS | 10 | 2 | 5 | 0 | 2 | 1 | 0 | Rs.-84.21 |

## Interpretation

The 70 stops are not a single phenomenon. The classification above separates:

- **Premature stops**: the stop prevented capturing a move that subsequently occurred. These are the most actionable — they suggest the stop boundary may be too close to entry relative to the instrument's normal volatility.
- **Temporary excursion stops**: the price recovered after the stop. These suggest the stop was triggered by noise rather than a genuine adverse move.
- **Gap-through stops**: the exit price was significantly beyond the stop boundary. These are execution-quality events, not strategy failures.
- **Direction failure stops**: the stop correctly identified a losing trade. These are the stops working as intended.
- **Genuine adverse stops**: the stop was protective and the price did not recover.
- **Stop too tight**: the stop distance was very narrow, suggesting ATR-based sizing may be producing stops that are too close to entry for some instruments.

**Do not tune Coralys v0 based on this analysis.** The next step is to expand the universe (v0.3) and observe whether the same stop behaviour pattern persists across a larger instrument set. Only then should coralys-exec-v1 be designed.
