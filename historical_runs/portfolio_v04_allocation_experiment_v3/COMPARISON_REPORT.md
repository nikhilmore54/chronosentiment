# Portfolio Replay v0.4 — Allocation Model Comparison Matrix

**Experiment:** EqualWeight (control) vs MaxPerSymbol ₹20k (experiment)  
**Initial capital:** Rs.1000000 for all configs  

## P.E.2 Arm

| Config | Alloc | Universe | Lots | TARGET | STOP | HORIZON | Return | Velocity |
|--------|-------|----------|------|--------|------|---------|--------|----------|
| v04_G_7_equal | EqualWeight | 7 | 84 | 35 | 0 | 12 | +8.61% | 5.41x |
| v04_H_7_max | MaxPerLot Rs.20000 | 7 | 105 | 40 | 0 | 12 | +3.58% | 2.10x |
| v04_A_25_equal | EqualWeight | 27 | 297 | 138 | 0 | 40 | +8.10% | 5.39x |
| v04_B_25_max | MaxPerLot Rs.20000 | 27 | 216 | 113 | 0 | 40 | +7.16% | 4.32x |
| v04_C_50_equal | EqualWeight | 52 | 572 | 270 | 0 | 80 | +4.39% | 5.12x |
| v04_D_50_max | MaxPerLot Rs.20000 | 52 | 312 | 155 | 0 | 80 | +6.15% | 6.24x |
| v04_E_100_equal | EqualWeight | 103 | 1030 | 506 | 0 | 158 | +2.92% | 5.03x |
| v04_F_100_max | MaxPerLot Rs.20000 | 103 | 412 | 197 | 0 | 158 | +3.41% | 8.24x |

## Coralys v0 Arm

| Config | Alloc | Universe | Lots | TARGET | STOP | HORIZON | Return | Velocity | Stop% | Premature% | Excursion% | Genuine% |
|--------|-------|----------|------|--------|------|---------|--------|----------|-------|------------|------------|----------|
| v04_G_7_equal | EqualWeight | 7 | 161 | 40 | 87 | 2 | +1.26% | 11.17x | 54.0% | 8.0% | 32.2% | 21.8% |
| v04_H_7_max | MaxPerLot Rs.20000 | 7 | 161 | 40 | 87 | 2 | -0.52% | 3.22x | 54.0% | 8.0% | 32.2% | 21.8% |
| v04_A_25_equal | EqualWeight | 27 | 594 | 171 | 295 | 3 | +8.67% | 12.58x | 49.7% | 4.7% | 32.5% | 20.3% |
| v04_B_25_max | MaxPerLot Rs.20000 | 27 | 486 | 165 | 257 | 3 | +3.51% | 9.72x | 52.9% | 5.1% | 35.8% | 12.5% |
| v04_C_50_equal | EqualWeight | 52 | 1144 | 329 | 574 | 3 | +8.09% | 12.93x | 50.2% | 4.2% | 32.4% | 20.7% |
| v04_D_50_max | MaxPerLot Rs.20000 | 52 | 728 | 288 | 372 | 3 | +9.79% | 14.56x | 51.1% | 4.6% | 37.9% | 4.8% |
| v04_E_100_equal | EqualWeight | 103 | 2266 | 617 | 1180 | 7 | +4.06% | 12.77x | 52.1% | 4.7% | 37.5% | 20.3% |
| v04_F_100_max | MaxPerLot Rs.20000 | 103 | 1236 | 452 | 702 | 7 | +10.66% | 24.72x | 56.8% | 6.7% | 44.4% | 5.6% |

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

