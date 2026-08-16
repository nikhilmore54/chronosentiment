# Targeted Decision Execution Report

**Document type:** Product validation evidence  
**Parent:** CS-P-006-P.E  
**Does not:** start C.3-G, run Search #3, retune C3-002, path-optimize the target, mutate the 14 August cohort  

`.cursor/rules/chronosentiment-core.mdc`: the target is sealed at T; future OHLC never chooses the target.

C3-002 chooses direction only. Execution Contract v0 owns `target_pct = 5.0%` and the 20-market-session maximum hold. Historical replay is a backtesting mechanism. This replay is not yet a statistical strategy backtest. Replay integrity is not strategy validation.

## Layers

| Layer | Question |
|---|---|
| Decision | Was LONG / SHORT / NO_TRADE selected from the certified state? |
| Execution | Did the predefined target get reached before the maximum holding period? |
| Evidence | What was the realized value after that exit? |

TARGET and HORIZON exits are both evidence. Neither is hidden.

## Integrity

- product label: Execution Contract v0
- execution contract: `targeted_execution_v0_fixed_5pct_20_sessions`
- target source: `deterministic_policy_parameter`
- target_pct: 5.0%
- max holding sessions: 20
- stop authorized: false
- target path-optimization authorized: false
- peeked_returns_at_seal: false
- prospective cohort mutated: false
- statistical strategy backtest: not done

Replay v0/v1 close-to-close observations are not reinterpreted here.

## Counts

- decisions: 14
- exits: 14
- TARGET: 8
- HORIZON: 6
- NO_TRADE: 0

## Ticks

| Instrument | Requested clock | Decision time | Direction | Entry | Target | Hit | Hit session | Exit | Reason | Hold | V |
|---|---|---|---|---:|---:|---|---:|---:|---|---:|---:|
| HDFCBANK.NS | 2026-05-15T03:45:00+00:00 | 2026-05-15T03:45:00+00:00 | LONG | 755.01 | 792.76 | false | — | 759.88 | HORIZON | 20 | +0.64% |
| ICICIBANK.NS | 2026-05-15T03:45:00+00:00 | 2026-05-15T03:45:00+00:00 | SHORT | 1234.10 | 1172.39 | false | — | 1329.59 | HORIZON | 20 | −7.74% |
| INFY.NS | 2026-05-15T03:45:00+00:00 | 2026-05-15T03:45:00+00:00 | LONG | 1095.30 | 1150.06 | true | 2 | 1150.06 | TARGET | 2 | +5.00% |
| RELIANCE.NS | 2026-05-15T03:45:00+00:00 | 2026-05-15T03:45:00+00:00 | SHORT | 1330.25 | 1263.74 | true | 16 | 1263.74 | TARGET | 16 | +5.00% |
| TCS.NS | 2026-05-15T03:45:00+00:00 | 2026-05-15T03:45:00+00:00 | LONG | 2221.53 | 2332.61 | true | 2 | 2332.61 | TARGET | 2 | +5.00% |
| IDEA.NS | 2026-05-15T03:45:00+00:00 | 2026-05-15T03:45:00+00:00 | LONG | 12.95 | 13.60 | true | 2 | 13.60 | TARGET | 2 | +5.00% |
| MAHABANK.NS | 2026-05-15T03:45:00+00:00 | 2026-05-15T03:45:00+00:00 | LONG | 76.85 | 80.69 | true | 6 | 80.69 | TARGET | 6 | +5.00% |
| HDFCBANK.NS | 2026-06-14T03:45:00+00:00 | 2026-06-12T03:45:00+00:00 | LONG | 759.88 | 797.88 | true | 9 | 798.50 | TARGET | 9 | +5.08% |
| ICICIBANK.NS | 2026-06-14T03:45:00+00:00 | 2026-06-12T03:45:00+00:00 | LONG | 1329.59 | 1396.07 | true | 15 | 1402.18 | TARGET | 15 | +5.46% |
| INFY.NS | 2026-06-14T03:45:00+00:00 | 2026-06-12T03:45:00+00:00 | LONG | 1116.40 | 1172.22 | false | — | 1068.00 | HORIZON | 20 | −4.34% |
| RELIANCE.NS | 2026-06-14T03:45:00+00:00 | 2026-06-12T03:45:00+00:00 | LONG | 1293.00 | 1357.65 | false | — | 1307.80 | HORIZON | 20 | +1.14% |
| TCS.NS | 2026-06-14T03:45:00+00:00 | 2026-06-12T03:45:00+00:00 | LONG | 2149.61 | 2257.09 | false | — | 2057.72 | HORIZON | 20 | −4.28% |
| IDEA.NS | 2026-06-14T03:45:00+00:00 | 2026-06-12T03:45:00+00:00 | LONG | 14.90 | 15.64 | false | — | 14.19 | HORIZON | 20 | −4.77% |
| MAHABANK.NS | 2026-06-14T03:45:00+00:00 | 2026-06-12T03:45:00+00:00 | LONG | 87.30 | 91.67 | true | 11 | 91.67 | TARGET | 11 | +5.00% |

Exit reason TARGET means the high (LONG) or low (SHORT) reached the sealed target. HORIZON means the 20th market session closed without a hit. Both are evidence. Aggregates are not a homepage metric. C3-002 does not have a 5% target.
