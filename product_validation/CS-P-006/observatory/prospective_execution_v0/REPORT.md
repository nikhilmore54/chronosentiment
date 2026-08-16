# Live Execution Observation Report

**Document type:** Product validation evidence  
**Parent:** CS-P-006-P.E.2  
**Does not:** mutate the 14 August cohort, rewrite P.E.1, retune C3-002, start C.3-G, run Search #3  

`.cursor/rules/chronosentiment-core.mdc`: the target is sealed at T; future OHLC never chooses the target.

C3-002 chooses direction only. Execution Contract v0 owns `target_pct = 5.0%`. The 14-August cohort was sealed without an execution intent and remains untouched. P.E.2 will attach Execution Contract v0 only to the next eligible cohort at T.

```text
14-Aug cohort
Decision only
7 OBSERVING
No execution intent
```

```text
Next eligible cohort
Decision + Execution Intent
P.E.2 control
```

- product label: Execution Contract v0
- path kind: `prospective_execution_v0`
- seal status: `AWAITING_NEXT_SESSION`
- certified T: 2026-08-14T03:45:00+00:00
- target_pct: 5.0%
- 14 August cohort mutated: false
- P.E.1 sidecar mutated: false
- peeked_returns_at_seal: false
- statistical strategy backtest: not done

- decisions: 0
- OBSERVING: 0
- TARGET: 0
- HORIZON: 0

The 14-August cohort was sealed without an execution intent and remains untouched. P.E.2 will attach Execution Contract v0 only to the next eligible cohort at T. AWAITING_NEXT_SESSION until a session strictly after 2026-08-14T03:45:00Z exists. IDEA and MAHABANK remain in the universe.
