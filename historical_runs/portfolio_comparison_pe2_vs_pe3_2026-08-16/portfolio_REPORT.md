# Portfolio Historical Replay v0.1

**Document type:** Product validation evidence  
**Experiment:** Portfolio Replay v0.1 — P.E.2 vs Coralys v0  
**Does not:** modify C3-002, modify coralys-exec-v0, touch P.E.2 or P.E.3-B archives  

## Setup

- Certified T: 2026-07-15T03:45:00+00:00
- Initial capital: Rs.5000.00
- C3-002 artifact: `5a43b9df97daa76d85edd7f7ef1c12c3a230ef292f7ecfa98ef9587647392121`
- Coralys artifact: `3876ffa232f75068636aa058c6775671ac2f935ad2751c1253edd49e0770883f`

## P.E.2 Arm (fixed +5%, no stop)

- Contract: `targeted_execution_v0_fixed_5pct_20_sessions`
- Positions opened: 7
- TARGET: 3 | STOP: 0 | HORIZON: 4 | AMBIGUOUS: 0
- Final portfolio value: Rs.5059.22
- Total return: +1.18%
- Realized P&L: Rs.+59.22
- Unrealized P&L: Rs.-0.00
- Max drawdown: 1.51% (Rs.75.73)
- Avg holding: 15.0 sessions

## Coralys v0 Arm (ATR/TMV target + enforced stop)

- Contract: `coralys_exec_v0_atr_tmv_stop_enforced_20_sessions`
- Positions opened: 7
- TARGET: 4 | STOP: 2 | HORIZON: 1 | AMBIGUOUS: 0
- Final portfolio value: Rs.5044.36
- Total return: +0.89%
- Realized P&L: Rs.+44.36
- Unrealized P&L: Rs.-0.00
- Max drawdown: 0.61% (Rs.30.73)
- Avg holding: 4.7 sessions

## Notes

Exploratory n=7. Not a statistical strategy backtest. Results are evidence, not proof. Both frozen artifacts unchanged.

P.E.2 arm: fixed +5% target, no stop. Coralys v0 arm: ATR/TMV target + enforced risk_boundary stop. Same C3-002 decisions, same market data, same period. Divergence is the product effect being measured.
