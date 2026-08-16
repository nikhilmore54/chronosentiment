# Historical P.E.2 Lifecycle Validation Report

**Document type:** Product validation evidence  
**Parent:** CS-P-006-P.E.2.H  
**Does not:** modify the P.E.2 specification, mutate the 14 August cohort, rewrite P.E.1 / Replay v0 / Replay v1 / live P.E.2, start P.E.3  

`.cursor/rules/chronosentiment-core.mdc`: the Decision and Execution Intent are sealed at T; future OHLC never chooses the target.

Historical P.E.2 lifecycle validation: **PASS**  
Statistical strategy backtest: **NOT PERFORMED**

This is a time-machine of the frozen P.E.2 control. Live P.E.2 remains `AWAITING_NEXT_SESSION` with 0 seals. The 14-August cohort stays decision-only.

## Clock

- requested T: `2026-07-15T03:45:00+00:00`
- certified T: `2026-07-15T03:45:00+00:00`
- path kind: `historical_pe2_replay`
- product label: Execution Contract v0
- execution contract: `targeted_execution_v0_fixed_5pct_20_sessions`
- target_pct: 5.0%
- max holding sessions: 20

## Integrity

- determinism: PASS
- no-lookahead: PASS
- poison test: PASS
- peeked_returns_at_seal: false
- prospective cohort mutated: false
- protected artifacts mutated: false

## Counts

- intents: 7
- execution intents: 7
- TARGET exits: 3
- HORIZON exits: 4
- GAP_THROUGH: 1
- HIGH_REACHED: 1
- LOW_REACHED: 1
- SESSION_CLOSE: 4

TARGET and HORIZON are both evidence. Trigger type records why the exit fired. Mean / median / total V, Sharpe, CAGR, and win rate are not product claims.

## Ticks

| Instrument | Certified T | Decision | Target | Exit | Trigger | Session | Execution price |
|---|---|---|---:|---|---|---:|---:|
| HDFCBANK.NS | 2026-07-15T03:45:00+00:00 | LONG | 856.22 | HORIZON | SESSION_CLOSE | 20 | 729.00 |
| ICICIBANK.NS | 2026-07-15T03:45:00+00:00 | LONG | 1474.58 | HORIZON | SESSION_CLOSE | 20 | 1431.70 |
| INFY.NS | 2026-07-15T03:45:00+00:00 | LONG | 1130.12 | TARGET | GAP_THROUGH | 10 | 1147.00 |
| RELIANCE.NS | 2026-07-15T03:45:00+00:00 | LONG | 1360.28 | HORIZON | SESSION_CLOSE | 20 | 1329.00 |
| TCS.NS | 2026-07-15T03:45:00+00:00 | LONG | 2298.66 | TARGET | HIGH_REACHED | 8 | 2298.66 |
| IDEA.NS | 2026-07-15T03:45:00+00:00 | SHORT | 12.99 | TARGET | LOW_REACHED | 7 | 12.99 |
| MAHABANK.NS | 2026-07-15T03:45:00+00:00 | SHORT | 76.09 | HORIZON | SESSION_CLOSE | 20 | 81.87 |

P.E.1, Replay v0/v1, the 14-August prospective ledger, and live `prospective_execution_v0` were not written. C.3-G is untouched. Search #3 is not authorized. P.E.3 is not started.
