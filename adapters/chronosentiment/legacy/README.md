# Legacy — B3/B4 provenance (not the product path)

This tree is **feature-gated**. It is required to reproduce Knowledge Lake
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

**CLEAN-003-C:** obsolete observatory / RP-601 / week2 quarantine sources moved to
`archive/chronosentiment_legacy_quarantine/` (not compiled; provenance only).

Product decisions use `decision_support::TradingDecision` and `DecisionPolicy`.
Do not merge lake `reasoning::decision::Decision` into that contract in this
cleanup. That is a later deprecation, after nothing except reproduction depends
on the lake object.

B3 and B4 dumps remain immutable. Do not regenerate them from this tree.
