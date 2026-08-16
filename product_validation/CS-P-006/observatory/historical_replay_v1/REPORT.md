# Observatory Historical Replay Report

**Document type:** Product validation evidence  
**Parent:** CS-P-006-P.H  
**Does not:** start C.3-G, run Search #3, retune C3-002, mutate the 14 August cohort, build a performance dashboard  

`.cursor/rules/chronosentiment-core.mdc`: same certified state at T + same sealed policy → same decision; outcomes never construct the decision.

This is the production Observatory running against a historical clock. Historical replay is a backtesting mechanism. This replay is not yet a statistical strategy backtest. Replay integrity is not strategy validation.

## Integrity

- historical replay integrity: PASS
- statistical strategy backtest: not done
- replay integrity ≠ strategy validation

## Contract

- replay contract: `historical_replay_v1_20_market_sessions`
Replay v0 (20 calendar days) is archived and is not reinterpreted here.

```text
horizon:
    duration = 20
    unit = MARKET_SESSIONS
    calendar_basis = TRADING_DAYS
    weekends = excluded
    market_holidays = excluded
```

- session rule: `latest_certified_session_at_or_before_requested_clock`
- trading-session horizon authorized: true
- path_kind: `historical_observatory_replay`
- policy: C3-002
- artifact: `5a43b9df97daa76d85edd7f7ef1c12c3a230ef292f7ecfa98ef9587647392121`
- decisions: 14
- completed evidence: 14
- peeked_returns: false
- determinism: PASS
- no-lookahead: PASS
- prospective cohort mutated: false

14 June 2026 is not a session. The certified market timestamp is the latest session ≤ the requested clock (12 Jun 2026, 03:45 UTC).

This decision was generated without access to information after T.

## Ticks

| Instrument | Requested clock | Decision time | Action | State hash | Closes | Status | Outcome | V | peeked | det | lookahead |
|---|---|---|---|---|---|---|---|---|---|---|---|
| HDFCBANK.NS | 2026-05-15T03:45:00+00:00 | 2026-05-15T03:45:00+00:00 | LONG | `a47e842d…` | Observation closes 12 Jun 2026, 03:45 UTC | OBSERVED | +0.64% | +0.64% | false | PASS | PASS |
| ICICIBANK.NS | 2026-05-15T03:45:00+00:00 | 2026-05-15T03:45:00+00:00 | SHORT | `34b67a2e…` | Observation closes 12 Jun 2026, 03:45 UTC | OBSERVED | +7.74% | -7.74% | false | PASS | PASS |
| INFY.NS | 2026-05-15T03:45:00+00:00 | 2026-05-15T03:45:00+00:00 | LONG | `a47e842d…` | Observation closes 12 Jun 2026, 03:45 UTC | OBSERVED | +1.93% | +1.93% | false | PASS | PASS |
| RELIANCE.NS | 2026-05-15T03:45:00+00:00 | 2026-05-15T03:45:00+00:00 | SHORT | `34b67a2e…` | Observation closes 12 Jun 2026, 03:45 UTC | OBSERVED | -2.80% | +2.80% | false | PASS | PASS |
| TCS.NS | 2026-05-15T03:45:00+00:00 | 2026-05-15T03:45:00+00:00 | LONG | `a47e842d…` | Observation closes 12 Jun 2026, 03:45 UTC | OBSERVED | -3.24% | -3.24% | false | PASS | PASS |
| IDEA.NS | 2026-05-15T03:45:00+00:00 | 2026-05-15T03:45:00+00:00 | LONG | `50a54bd9…` | Observation closes 12 Jun 2026, 03:45 UTC | OBSERVED | +15.06% | +15.06% | false | PASS | PASS |
| MAHABANK.NS | 2026-05-15T03:45:00+00:00 | 2026-05-15T03:45:00+00:00 | LONG | `50a54bd9…` | Observation closes 12 Jun 2026, 03:45 UTC | OBSERVED | +13.60% | +13.60% | false | PASS | PASS |
| HDFCBANK.NS | 2026-06-14T03:45:00+00:00 | 2026-06-12T03:45:00+00:00 | LONG | `5d0ced0f…` | Observation closes 10 Jul 2026, 03:45 UTC | OBSERVED | +8.56% | +8.56% | false | PASS | PASS |
| ICICIBANK.NS | 2026-06-14T03:45:00+00:00 | 2026-06-12T03:45:00+00:00 | LONG | `5d0ced0f…` | Observation closes 10 Jul 2026, 03:45 UTC | OBSERVED | +4.50% | +4.50% | false | PASS | PASS |
| INFY.NS | 2026-06-14T03:45:00+00:00 | 2026-06-12T03:45:00+00:00 | LONG | `5d0ced0f…` | Observation closes 10 Jul 2026, 03:45 UTC | OBSERVED | -4.34% | -4.34% | false | PASS | PASS |
| RELIANCE.NS | 2026-06-14T03:45:00+00:00 | 2026-06-12T03:45:00+00:00 | LONG | `a47e842d…` | Observation closes 10 Jul 2026, 03:45 UTC | OBSERVED | +1.14% | +1.14% | false | PASS | PASS |
| TCS.NS | 2026-06-14T03:45:00+00:00 | 2026-06-12T03:45:00+00:00 | LONG | `a47e842d…` | Observation closes 10 Jul 2026, 03:45 UTC | OBSERVED | -4.28% | -4.28% | false | PASS | PASS |
| IDEA.NS | 2026-06-14T03:45:00+00:00 | 2026-06-12T03:45:00+00:00 | LONG | `50a54bd9…` | Observation closes 10 Jul 2026, 03:45 UTC | OBSERVED | -4.77% | -4.77% | false | PASS | PASS |
| MAHABANK.NS | 2026-06-14T03:45:00+00:00 | 2026-06-12T03:45:00+00:00 | LONG | `50a54bd9…` | Observation closes 10 Jul 2026, 03:45 UTC | OBSERVED | -3.49% | -3.49% | false | PASS | PASS |

Outcome is an evidence field. It is not part of the sealed decision.
Winners and losers stay visible because their windows have closed. Fourteen observations are not a statistical performance study. Aggregates are not a homepage metric. Historical replay is a backtesting mechanism; replay integrity is not strategy validation.
