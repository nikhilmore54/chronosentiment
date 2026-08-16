# Observatory Historical Replay Report

**Document type:** Product validation evidence  
**Parent:** CS-P-006-P.H  
**Does not:** start C.3-G, run Search #3, retune C3-002, mutate the 14 August cohort, build a performance dashboard  

`.cursor/rules/chronosentiment-core.mdc`: same certified state at T + same sealed policy → same decision; outcomes never construct the decision.

This is the production Observatory running against a historical clock. It is not a look-ahead backtest and not a statistical strategy backtest.

## Integrity

- historical replay integrity: PASS
- statistical strategy backtest: not done

## Contract

```text
horizon:
    duration = 20 days
    calendar_basis = CALENDAR_DAYS
```

- session rule: `latest_certified_session_at_or_before_requested_clock`
- trading-session horizon authorized: false
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
| HDFCBANK.NS | 2026-05-15T03:45:00+00:00 | 2026-05-15T03:45:00+00:00 | LONG | `a47e842d…` | Observation closes 4 Jun 2026, 03:45 UTC | OBSERVED | -1.73% | -1.73% | false | PASS | PASS |
| ICICIBANK.NS | 2026-05-15T03:45:00+00:00 | 2026-05-15T03:45:00+00:00 | SHORT | `34b67a2e…` | Observation closes 4 Jun 2026, 03:45 UTC | OBSERVED | +0.58% | -0.58% | false | PASS | PASS |
| INFY.NS | 2026-05-15T03:45:00+00:00 | 2026-05-15T03:45:00+00:00 | LONG | `a47e842d…` | Observation closes 4 Jun 2026, 03:45 UTC | OBSERVED | +7.35% | +7.35% | false | PASS | PASS |
| RELIANCE.NS | 2026-05-15T03:45:00+00:00 | 2026-05-15T03:45:00+00:00 | SHORT | `34b67a2e…` | Observation closes 4 Jun 2026, 03:45 UTC | OBSERVED | -2.45% | +2.45% | false | PASS | PASS |
| TCS.NS | 2026-05-15T03:45:00+00:00 | 2026-05-15T03:45:00+00:00 | LONG | `a47e842d…` | Observation closes 4 Jun 2026, 03:45 UTC | OBSERVED | +0.33% | +0.33% | false | PASS | PASS |
| IDEA.NS | 2026-05-15T03:45:00+00:00 | 2026-05-15T03:45:00+00:00 | LONG | `50a54bd9…` | Observation closes 4 Jun 2026, 03:45 UTC | OBSERVED | +15.29% | +15.29% | false | PASS | PASS |
| MAHABANK.NS | 2026-05-15T03:45:00+00:00 | 2026-05-15T03:45:00+00:00 | LONG | `50a54bd9…` | Observation closes 4 Jun 2026, 03:45 UTC | OBSERVED | +2.41% | +2.41% | false | PASS | PASS |
| HDFCBANK.NS | 2026-06-14T03:45:00+00:00 | 2026-06-12T03:45:00+00:00 | LONG | `5d0ced0f…` | Observation closes 2 Jul 2026, 03:45 UTC | OBSERVED | +4.74% | +4.74% | false | PASS | PASS |
| ICICIBANK.NS | 2026-06-14T03:45:00+00:00 | 2026-06-12T03:45:00+00:00 | LONG | `5d0ced0f…` | Observation closes 2 Jul 2026, 03:45 UTC | OBSERVED | +4.42% | +4.42% | false | PASS | PASS |
| INFY.NS | 2026-06-14T03:45:00+00:00 | 2026-06-12T03:45:00+00:00 | LONG | `5d0ced0f…` | Observation closes 2 Jul 2026, 03:45 UTC | OBSERVED | -6.76% | -6.76% | false | PASS | PASS |
| RELIANCE.NS | 2026-06-14T03:45:00+00:00 | 2026-06-12T03:45:00+00:00 | LONG | `a47e842d…` | Observation closes 2 Jul 2026, 03:45 UTC | OBSERVED | +0.81% | +0.81% | false | PASS | PASS |
| TCS.NS | 2026-06-14T03:45:00+00:00 | 2026-06-12T03:45:00+00:00 | LONG | `a47e842d…` | Observation closes 2 Jul 2026, 03:45 UTC | OBSERVED | -4.32% | -4.32% | false | PASS | PASS |
| IDEA.NS | 2026-06-14T03:45:00+00:00 | 2026-06-12T03:45:00+00:00 | LONG | `50a54bd9…` | Observation closes 2 Jul 2026, 03:45 UTC | OBSERVED | -2.82% | -2.82% | false | PASS | PASS |
| MAHABANK.NS | 2026-06-14T03:45:00+00:00 | 2026-06-12T03:45:00+00:00 | LONG | `50a54bd9…` | Observation closes 2 Jul 2026, 03:45 UTC | OBSERVED | +4.28% | +4.28% | false | PASS | PASS |

Outcome is an evidence field. It is not part of the sealed decision.
Winners and losers stay visible because their windows have closed. Fourteen observations are not a statistical performance study. Aggregates are not a homepage metric.
