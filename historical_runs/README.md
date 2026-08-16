# Historical Runs

This directory contains immutable archives of historical execution runs.

Each run is a complete, self-contained snapshot of a specific execution experiment.
Runs are **never mixed or overwritten**. Each has its own directory, metadata, and ledger.

---

## Structure

```
historical_runs/
├── pe2_control_2026-08-16/        IMMUTABLE — P.E.2 control (fixed +5%)
│   ├── execution_ledger/
│   ├── decisions/
│   ├── reports/
│   ├── metadata.json
│   └── README.md
│
└── pe3_coralys_v0_2026-08-16/     PENDING_REPLAY — P.E.3 (coralys-exec-v0)
    ├── execution_ledger/
    ├── decisions/
    ├── reports/
    ├── metadata.json
    └── README.md
```

---

## Experiments

| Directory | Experiment | Contract | Status | Purpose |
|---|---|---|---|---|
| `pe2_control_2026-08-16/` | P.E.2 | Fixed +5%, no risk | IMMUTABLE | Historical control baseline |
| `pe3_coralys_v0_2026-08-16/` | P.E.3 | coralys-exec-v0 ATR/TMV | IMMUTABLE | Current execution history |

---

## Rules

1. **Never modify an IMMUTABLE archive.** Once `status = IMMUTABLE`, the directory is frozen.
2. **Never mix P.E.2 and P.E.3 records.** Each experiment has its own ledger.
3. **P.E.3 uses the frozen coralys artifact hash:** `3876ffa232f75068636aa058c6775671ac2f935ad2751c1253edd49e0770883f`
4. **Historical replay outcomes are not learning feedback.** They populate the evidence ledger but are tagged `retrospective_characterization`, not `live_feedback`.
5. **The portal shows P.E.3 as current.** P.E.2 is shown as the historical control reference.

---

## Comparison

The P.E.2 and P.E.3 archives use the same:
- Historical market data
- C3-002 decision policy
- Universe
- Entry prices
- Maximum hold (20 sessions)

They differ only in the **execution contract** (target rule + risk rule).
This isolates **execution intelligence from directional intelligence** — the core purpose of the comparison.