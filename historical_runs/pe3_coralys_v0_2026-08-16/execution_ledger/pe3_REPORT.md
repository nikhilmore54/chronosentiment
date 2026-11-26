# P.E.3 Historical Replay Report — coralys-exec-v0

**Document type:** Product validation evidence  
**Parent:** CS-P-006-P.E.3.H  
**Execution model:** coralys-exec-v0 (ATR-anchored, TMV-scaled)  
**Does not:** modify C3-002, modify coralys-exec-v0 multipliers, fall back to +5%, touch P.E.2 ledger  

- coralys artifact hash: `3876ffa232f75068636aa058c6775671ac2f935ad2751c1253edd49e0770883f`
- path kind: `historical_pe3_replay`
- execution contract: `coralys_exec_v0_atr_tmv_20_sessions`
- certified T: 2026-07-15T03:45:00+00:00
- requested clock: 2026-07-15T03:45:00+00:00
- peeked_returns_at_seal: false
- statistical backtest: not done

- decisions: 7
- P.E.3 eligible (ATR available): 7
- excluded (ATR unavailable): 0
- NO_TRADE: 0
- TARGET: 5
- RISK (stop): 0
- HORIZON: 2
- AMBIGUOUS: 0

- determinism: true
- lookahead_clean: true
- poison_test_pass: true

- retrospective_characterization: true

## Lifecycle validation

P.E.3 historical replay: 7 decisions, 7 eligible, 0 excluded (no ATR), 0 NO_TRADE. Artifact: 3876ffa232f75068636aa058c6775671ac2f935ad2751c1253edd49e0770883f. determinism=true lookahead=true poison=true
