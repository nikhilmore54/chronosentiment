# Portfolio Replay v0.4 — Allocation Model Comparison Matrix

**Experiment:** EqualWeight (control) vs MaxPerSymbol ₹20k (experiment)  
**Initial capital:** Rs.1000000 for all configs  

## P.E.2 Arm

| Config | Alloc | Universe | Lots | TARGET | STOP | HORIZON | Return | Velocity |
|--------|-------|----------|------|--------|------|---------|--------|----------|
| v04_A_25_equal | EqualWeight | 25 | 275 | 127 | 0 | 37 | +7.76% | 5.39x |
| v04_B_25_max | MaxPerSymbol Rs.20000 | 25 | 200 | 105 | 0 | 37 | +6.41% | 4.00x |
| v04_C_50_equal | EqualWeight | 50 | 539 | 259 | 0 | 74 | +4.36% | 5.15x |
| v04_D_50_max | MaxPerSymbol Rs.20000 | 50 | 294 | 149 | 0 | 74 | +5.70% | 5.88x |

## Coralys v0 Arm

| Config | Alloc | Universe | Lots | TARGET | STOP | HORIZON | Return | Velocity | Stop% | Premature% | Excursion% | Genuine% |
|--------|-------|----------|------|--------|------|---------|--------|----------|-------|------------|------------|----------|
| v04_A_25_equal | EqualWeight | 25 | 550 | 160 | 263 | 2 | +10.13% | 12.29x | 47.8% | 3.8% | 32.3% | 22.1% |
| v04_B_25_max | MaxPerSymbol Rs.20000 | 25 | 450 | 159 | 228 | 2 | +4.03% | 9.00x | 50.7% | 4.4% | 35.5% | 14.0% |
| v04_C_50_equal | EqualWeight | 50 | 1078 | 313 | 527 | 2 | +9.11% | 12.73x | 48.9% | 3.8% | 31.1% | 22.0% |
| v04_D_50_max | MaxPerSymbol Rs.20000 | 50 | 686 | 281 | 339 | 2 | +10.38% | 13.72x | 49.4% | 5.0% | 37.2% | 5.3% |

## Velocity Comparison: EqualWeight vs MaxPerSymbol

| Universe | EqualWeight Velocity (PE2/Coralys) | MaxPerSymbol Velocity (PE2/Coralys) | Delta |
|----------|-------------------------------------|--------------------------------------|-------|
| 25 | 5.39x / 12.29x | 4.00x / 9.00x | PE2: -1.39x / Coralys: -3.29x |
| 50 | 5.15x / 12.73x | 5.88x / 13.72x | PE2: +0.73x / Coralys: +0.99x |

## Interpretation

- **EqualWeight** deploys all available cash in every session with eligible signals.
  At high signal density (50 instruments), this exhausts capital in session 1.
- **MaxPerSymbol ₹20k** caps each lot at ₹20,000, leaving undeployed capital
  available for subsequent sessions. This should increase velocity at 50 instruments.
- If MaxPerSymbol velocity > EqualWeight velocity at 50 instruments, the hypothesis
  is confirmed: allocation policy (not capital amount) drives velocity collapse.

