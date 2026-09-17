# CS-P-001-H — Cached session harness: product-contract decision

**Document type:** Product-contract decision  
**Status:** Unsigned — neither option selected; do not implement from this record  
**Date:** 2026-09-15  
**Parent:** CS-P-001-G (frozen)  
**Does not mutate:** `deferred_live.rs`, auto-arm, snap gate, date-shift join, delayed fill, TARGET / STOP / HORIZON, reassessment, INVERT  
**Does not claim:** G-GATE predictive value  
**Does not amend:** CS-P-001-D, CS-P-001-E, CS-P-001-G  

`.cursor/rules/chronosentiment-core.mdc`: deterministic as-of events; no invented prices.

CS-P-001-G closed the ambiguity. This record is the **decision boundary**, not a patch.

---

## Decision (unsigned)

Select **exactly one**. Until selected and written here, implementation is not authorized.

| Option | Job the cached `--session DATE` harness represents | Semantics |
| --- | --- | --- |
| **1** | Live as-of simulator | Cached Deferred Live must implement **C ≡ D** — first observation after the 15:30 IST close snap (next-session open) |
| **2** | Same-day clock exception | Cached `--session DATE` remains **A** — same-calendar-day 09:15 tape — and that exception is **explicitly documented** |

**Selected:** _none_

---

## Frozen predecessors (do not reopen)

| Record | Statement |
| --- | --- |
| CS-P-001-D | Adverse fills can compress frozen T0 risk. Do not rebase geometry. |
| CS-P-001-E | Displacement is not post-snap staleness. 34/34 fill-before-snap. |
| CS-P-001-G | Live as-of is C ≡ D. Cached harness is A. A is not a documented exception. Nothing to patch until the job is named. |

---

## Pin until this record is signed

```text
session_auto_arm_opens_on_observation_before_snap_unix
```

No snap gate, no date shift, no delayed fill, no stop rebasing, no reassessment, no INVERT.

Selecting option 1 later is a join change. Selecting option 2 later is a documentation change. Neither is this record.
