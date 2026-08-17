# Portfolio Replay v0.4 — Allocation Model Comparison Matrix

**Experiment:** EqualWeight (control) vs MaxPerSymbol ₹20k (experiment)  
**Initial capital:** Rs.1000000 for all configs  

## P.E.2 Arm

| Config | Alloc | Universe | Lots | TARGET | STOP | HORIZON | Return | Velocity |
|--------|-------|----------|------|--------|------|---------|--------|----------|
| v04_A_25_equal | EqualWeight | 27 | 297 | 138 | 0 | 40 | +8.10% | 5.39x |
| v04_B_25_max | MaxPerLot Rs.20000 | 27 | 216 | 113 | 0 | 40 | +7.16% | 4.32x |
| v04_C_50_equal | EqualWeight | 52 | 572 | 270 | 0 | 80 | +4.39% | 5.12x |
| v04_D_50_max | MaxPerLot Rs.20000 | 52 | 312 | 155 | 0 | 80 | +6.15% | 6.24x |

## Coralys v0 Arm

| Config | Alloc | Universe | Lots | TARGET | STOP | HORIZON | Return | Velocity | Stop% | Premature% | Excursion% | Genuine% |
|--------|-------|----------|------|--------|------|---------|--------|----------|-------|------------|------------|----------|
| v04_A_25_equal | EqualWeight | 27 | 594 | 171 | 295 | 3 | +8.67% | 12.58x | 49.7% | 4.7% | 32.5% | 20.3% |
| v04_B_25_max | MaxPerLot Rs.20000 | 27 | 486 | 165 | 257 | 3 | +3.51% | 9.72x | 52.9% | 5.1% | 35.8% | 12.5% |
| v04_C_50_equal | EqualWeight | 52 | 1144 | 329 | 574 | 3 | +8.09% | 12.93x | 50.2% | 4.2% | 32.4% | 20.7% |
| v04_D_50_max | MaxPerLot Rs.20000 | 52 | 728 | 288 | 372 | 3 | +9.79% | 14.56x | 51.1% | 4.6% | 37.9% | 4.8% |

## Velocity Comparison: EqualWeight vs MaxPerSymbol

| Universe | EqualWeight Velocity (PE2/Coralys) | MaxPerSymbol Velocity (PE2/Coralys) | Delta |
|----------|-------------------------------------|--------------------------------------|-------|
| 27 | 5.39x / 12.58x | 4.32x / 9.72x | PE2: -1.07x / Coralys: -2.86x |
| 52 | 5.12x / 12.93x | 6.24x / 14.56x | PE2: +1.12x / Coralys: +1.63x |

## Interpretation

- **EqualWeight** deploys all available cash in every session with eligible signals.
  At high signal density (50 instruments), this exhausts capital in session 1.
- **MaxPerSymbol ₹20k** caps each lot at ₹20,000, leaving undeployed capital
  available for subsequent sessions. This should increase velocity at 50 instruments.
- If MaxPerSymbol velocity > EqualWeight velocity at 50 instruments, the hypothesis
  is confirmed: allocation policy (not capital amount) drives velocity collapse.

