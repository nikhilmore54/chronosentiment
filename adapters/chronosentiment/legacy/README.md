# Legacy — B3/B4 provenance (not the product path)

This tree is **quarantined**. It is required to reproduce Knowledge Lake
population for B3/B4. It is not the ChronoSentiment product decision system.

Build generators:

```text
cargo build -p chronosentiment_adapter --features legacy-lake --bin m4_populate_knowledge_lake
```

| Path | Role |
|------|------|
| `bin/m4_populate_knowledge_lake.rs` | Historical lake generator. Preserve exact behaviour, including `Uuid::new_v4` identity and StrategyEngine SHORT omission. Do not “fix” it to improve B4 coverage. |
| `bin/m4_validation_gate.rs` | Heritage validation gate using the same generators. |
| `bin/m1_*`, `m2_*`, `m3_*`, `m4_time_machine_demo.rs` | Demos. Not product. |
| `quarantine/rp601_crypto_daemon.rs` | Obsolete Yahoo-polling crypto daemon. Deprecated; not an RP-615 component. Active path is `scripts/run_rp615_crypto_shadow_daemon.py`. |
| `quarantine/csp006_p_observatory.rs` | Obsolete HTML observatory screen generator. Superseded by MVP Decision Ledger (`live005_ledger`). |
| `quarantine/csp006_p_observe.rs` | Obsolete maturity tick countdown. Superseded by `time009_observe`. |
| `quarantine/csp006_p_replay.rs` | Obsolete P.H historical replay. Superseded by `time003_replay` / `time004_ledger`. Evidence in `historical_replay_v1/`. |
| `quarantine/csp006_p_execute.rs` | Obsolete P.E targeted execution replay. Superseded by `deferred_live_decision_loop`. Evidence in `targeted_execution_v0/`. |
| `quarantine/csp006_p_live_execute.rs` | Obsolete P.E.2 August 14 live execution. Superseded by `deferred_live_decision_loop` / `live005_ledger`. |
| `quarantine/csp006_p_historical_pe2.rs` | Obsolete P.E.2.H historical control replay. Superseded by portfolio replay v0.3/v0.41. Evidence in `historical_pe2_replay/`. |
| `quarantine/csp006_p_replay_pe3.rs` | Obsolete P.E.3 prototype replay. Abandoned before operational deployment. |
| `quarantine/` | Dead sources kept for provenance (`week2_tests`, unused engines). **Not compiled.** |

Product decisions use `decision_support::TradingDecision` and `DecisionPolicy`.
Do not merge lake `reasoning::decision::Decision` into that contract in this
cleanup. That is a later deprecation, after nothing except reproduction depends
on the lake object.

B3 and B4 dumps remain immutable. Do not regenerate them from this tree.
