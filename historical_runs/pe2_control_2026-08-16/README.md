# P.E.2 Control Archive — Fixed +5% Execution

**Status: IMMUTABLE. Do not modify.**

Archived: 2026-08-16

---

## What this is

This directory is the frozen archive of the P.E.2 historical execution run.

P.E.2 used a **fixed +5% target** with **no risk boundary** and a **20-session maximum hold**.
It is the control baseline against which P.E.3 (coralys-exec-v0) is compared.

---

## Execution contract

| Field | Value |
|---|---|
| `execution_contract` | `targeted_execution_v0_fixed_5pct_20_sessions` |
| `execution_path_kind` | `prospective_execution_v0` |
| Target rule | Fixed +5% from entry price |
| Risk rule | None (no stop boundary) |
| Maximum hold | 20 sessions |
| Decision policy | C3-002 (RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH) |

---

## Files

```
execution_ledger/
  prospective_execution_v0_ledger.json   — live execution ledger (P.E.2 path)
  historical_pe2_replay_ledger.json      — full historical replay under P.E.2 contract

decisions/
  execution_intents.json                 — sealed execution intents (fixed +5% target)

reports/
  execution_report.json                  — execution replay report

metadata.json                            — machine-readable archive metadata
README.md                                — this file
```

---

## Comparison with P.E.3

| Dimension | P.E.2 (this archive) | P.E.3 (coralys-exec-v0) |
|---|---|---|
| Market data | Same | Same |
| Decision policy | C3-002 | C3-002 |
| Direction | Same | Same |
| Entry | Same | Same |
| Target | Fixed +5% | ATR(14)/TMV (coralys-exec-v0) |
| Risk boundary | None | ATR(14)/TMV (coralys-exec-v0) |
| Maximum hold | 20 sessions | 20 sessions |
| Execution path | `prospective_execution_v0` | `prospective_execution_pe3_v0` |
| Artifact | N/A (deterministic policy) | `3876ffa232f75068636aa058c6775671ac2f935ad2751c1253edd49e0770883f` |

---

## Immutability guarantee

This archive was frozen on 2026-08-16. No records in this directory should be modified.

The P.E.3 historical replay will be written to a **separate** directory:
`historical_runs/pe3_coralys_v0_2026-08-16/`

The two archives are independent datasets. Neither contaminates the other.

---

## Research questions this archive enables

Once P.E.3 is complete, the paired comparison will answer:

- Which trades changed from TARGET → HORIZON under P.E.3?
- Which changed from TARGET → RISK?
- How did holding duration change?
- How did realized return change?
- Which TMV states benefited from Coralys execution?
- Which instruments benefited?
- Where did Coralys risk boundaries hurt versus help?
- **Did Coralys change the outcome of the same underlying C3-002 decisions?**

This last question isolates **execution intelligence from directional intelligence** — the core purpose of the P.E.2 vs P.E.3 comparison.