# Management Behaviour Census v0.3 — Action Transition Analysis

> **Note:** SHORT positions have been excluded from this analysis due to invalid initial candidate_stop_price calculations.

## 1. High-Level Categories

### SAME ACTION (n=4317)
- Forward Return: 5m: -0.01% | 15m: -0.04% | 30m: -0.07% | 60m: -0.13%
- Post-Update Excursion: MFE: 0.55% | MAE: -0.98%
- Probabilities: Hit Target: 6.9% | Hit Stop: 71.4% | Pos 60m Ret: 34.8%
- Eventual H300: -0.50%

### ACTION_CHANGE (ALL) (n=981)
- Forward Return: 5m: -0.01% | 15m: -0.04% | 30m: -0.11% | 60m: -0.18%
- Post-Update Excursion: MFE: 0.61% | MAE: -1.04%
- Probabilities: Hit Target: 10.1% | Hit Stop: 65.3% | Pos 60m Ret: 29.5%
- Eventual H300: -0.57%

## 2. Action Reversal Breakdown

### ACTION_CHANGE (To AVOID) (n=0)
- No data.

### ACTION_CHANGE (Other) (n=981)
- Forward Return: 5m: -0.01% | 15m: -0.04% | 30m: -0.11% | 60m: -0.18%
- Post-Update Excursion: MFE: 0.61% | MAE: -1.04%
- Probabilities: Hit Target: 10.1% | Hit Stop: 65.3% | Pos 60m Ret: 29.5%
- Eventual H300: -0.57%

## 3. Specific Transitions

###  -> ACT (n=86)
- Forward Return: 5m: -0.06% | 15m: -0.09% | 30m: -0.44% | 60m: -0.34%
- Post-Update Excursion: MFE: 0.48% | MAE: -1.58%
- Probabilities: Hit Target: 8.1% | Hit Stop: 67.4% | Pos 60m Ret: 22.1%
- Eventual H300: -1.04%

###  -> MONITOR (n=1)
- Forward Return: 5m: 0.15% | 15m: -0.35% | 30m: -0.45% | 60m: -0.43%
- Post-Update Excursion: MFE: 0.41% | MAE: -1.09%
- Probabilities: Hit Target: 0.0% | Hit Stop: 100.0% | Pos 60m Ret: 0.0%
- Eventual H300: 0.08%

### ACT -> ACT (n=2271)
- Forward Return: 5m: -0.03% | 15m: -0.07% | 30m: -0.11% | 60m: -0.17%
- Post-Update Excursion: MFE: 0.51% | MAE: -1.16%
- Probabilities: Hit Target: 5.2% | Hit Stop: 74.5% | Pos 60m Ret: 31.6%
- Eventual H300: -0.60%

### ACT -> AVOID (n=25)
- Forward Return: 5m: 0.03% | 15m: 0.09% | 30m: 0.20% | 60m: 0.17%
- Post-Update Excursion: MFE: 0.74% | MAE: -0.99%
- Probabilities: Hit Target: 8.0% | Hit Stop: 48.0% | Pos 60m Ret: 72.0%
- Eventual H300: -0.50%

### ACT -> MONITOR (n=210)
- Forward Return: 5m: 0.01% | 15m: -0.02% | 30m: -0.06% | 60m: -0.14%
- Post-Update Excursion: MFE: 0.63% | MAE: -0.84%
- Probabilities: Hit Target: 11.0% | Hit Stop: 70.0% | Pos 60m Ret: 34.3%
- Eventual H300: -0.49%

### AVOID -> ACT (n=3)
- Forward Return: 5m: -0.01% | 15m: -0.13% | 30m: -0.07% | 60m: -0.09%
- Post-Update Excursion: MFE: 0.35% | MAE: -1.27%
- Probabilities: Hit Target: 0.0% | Hit Stop: 66.7% | Pos 60m Ret: 33.3%
- Eventual H300: -1.00%

### AVOID -> AVOID (n=1511)
- Forward Return: 5m: 0.01% | 15m: 0.01% | 30m: -0.01% | 60m: -0.06%
- Post-Update Excursion: MFE: 0.57% | MAE: -0.71%
- Probabilities: Hit Target: 8.0% | Hit Stop: 67.4% | Pos 60m Ret: 39.9%
- Eventual H300: -0.36%

### AVOID -> MONITOR (n=224)
- Forward Return: 5m: -0.01% | 15m: -0.05% | 30m: -0.09% | 60m: -0.19%
- Post-Update Excursion: MFE: 0.61% | MAE: -1.11%
- Probabilities: Hit Target: 9.8% | Hit Stop: 61.6% | Pos 60m Ret: 25.4%
- Eventual H300: -0.57%

### MONITOR -> ACT (n=163)
- Forward Return: 5m: -0.01% | 15m: -0.06% | 30m: -0.12% | 60m: -0.23%
- Post-Update Excursion: MFE: 0.64% | MAE: -0.97%
- Probabilities: Hit Target: 11.7% | Hit Stop: 68.1% | Pos 60m Ret: 22.7%
- Eventual H300: -0.56%

### MONITOR -> AVOID (n=269)
- Forward Return: 5m: -0.00% | 15m: -0.02% | 30m: -0.07% | 60m: -0.14%
- Post-Update Excursion: MFE: 0.59% | MAE: -1.00%
- Probabilities: Hit Target: 9.7% | Hit Stop: 63.9% | Pos 60m Ret: 31.6%
- Eventual H300: -0.50%

### MONITOR -> MONITOR (n=535)
- Forward Return: 5m: -0.00% | 15m: -0.02% | 30m: -0.06% | 60m: -0.14%
- Post-Update Excursion: MFE: 0.63% | MAE: -0.94%
- Probabilities: Hit Target: 11.0% | Hit Stop: 69.3% | Pos 60m Ret: 33.6%
- Eventual H300: -0.48%

