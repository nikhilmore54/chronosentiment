# CS-P-001-F — Session / DecisionBrief temporal contract

**Document type:** Product-clock inventory  
**Status:** Investigation — no semantic choice yet  
**Date:** 2026-09-15  
**Parent:** CS-P-001-E (frozen), CS-P-001-D (frozen)  
**Does not mutate:** `deferred_live.rs` driver, TARGET / STOP / HORIZON, fill rebasing, INVERT, reassessment  
**Does not claim:** G-GATE predictive value  

`.cursor/rules/chronosentiment-core.mdc`: deterministic as-of events; no invented prices.

CS-P-001-E closed the stale-decision hypothesis. The remaining question is interface, not stop tuning:

> Why does Deferred Live auto-arm consume a 09:15 observation against a DecisionBrief whose as-of geometry is 15:30?

This record inventories the **implemented** contract and adjacent as-of contracts. It does **not** pick the intended product semantics.

---

## 1. Frozen predecessors

| Record | Statement |
| --- | --- |
| CS-P-001-D | Adverse fills can compress frozen T0 risk. Do not rebase geometry. |
| CS-P-001-E | Displacement is not post-snap staleness. 34/34 fill-before-snap, lag −6.25 h constant. |

---

## 2. Implemented Deferred Live contract

Three clocks, two join keys, **no snap gate**.

| Concern | Join | Clock used |
| --- | --- | --- |
| Which briefs to arm | `brief.date == session_date` | cohort calendar string |
| Eligibility | IC v1 `ACT` + finite target/risk | not a time join |
| When to `PAPER_ENTER` | first matching ticker observation | tape `obs.unix` |
| Fill price | `obs.price` | observation as-of (09:15 close) |
| Target / risk | `DecisionBrief.execution` | snap as-of (15:30 IST) |
| Horizon unix | `snap_unix + 300 minutes` | snap as-of (20:30 IST) |

Pinned in code:

- `select_session_briefs` — date string only (`deferred_live_session.rs`).
- `ingest` — first matching tick; no `obs.unix >= snap_unix` (`deferred_live_loop.rs`).
- `open_from_brief` — fill = `obs.price`; horizon T0 = `snap_unix.unwrap_or(obs.unix)` (`deferred_live.rs`).

Test `session_auto_arm_opens_on_observation_before_snap_unix` records this as current behaviour. It is not a proposal to add a gate.

CS-P-001 §4: *No field may be derived from information after the decision as-of time.* Session auto-arm currently fills **before** the brief’s recorded as-of.

---

## 3. Adjacent contract: TIME-009 / INTRA-001

Those paths already treat LIVE-005 `1000` as **NSE close**:

```text
09:15–15:30 IST  =  03:45–10:00 UTC
T0 close         =  15:30 IST = 10:00 UTC
```

TIME-009: outcome bars must be **strictly after** `source_snapshot_timestamp`. The first eligible bar is the next session, not same-day 09:15.

Deferred Live session tape is the **same cohort date from 09:15**, which TIME-009 would discard as pre-T0.

Same DecisionBrief lineage, two product clocks.

---

## 4. Candidate semantics (not selected)

Do not implement any of these from this record.

| Candidate | Fill clock | Consequence on the CS-P-001-D book |
| --- | --- | --- |
| A. Keep implemented contract | Same-day 09:15 vs close geometry | Current 34 fills / 2 STOPs. Clock inversion remains. |
| B. Fill only `obs.unix >= snap_unix` | Same day after 15:30 | Cached tape ends ~15:15. **Zero fills** on these sessions. |
| C. Align with TIME-009 (first bar strictly after snap) | Next IST session 09:15 | Would be a different book. Not measured. |
| D. Arm previous close onto next open | Prior-day brief, next-day 09:15 | Requires a date-shift join. Not measured. |

A is what the code does. B–D would be product-clock changes. Intended semantics are **not established**.

---

## 5. Verdict

1. Freeze CS-P-001-E. Stale-after-snap is rejected for this book.
2. The inversion is a **session/brief join**: calendar date + first tick, vs snap-as-of geometry and snap-based horizon.
3. TIME-009 already uses a post-snap firewall; Deferred Live session auto-arm does not.
4. Do not change auto-arm, do not add a snap gate, do not retune stops. CS-P-001-G is frozen: C≡D under as-of; cached session is A and is not declared an exception. Name that job before any join change.

Pinned test: `session_auto_arm_opens_on_observation_before_snap_unix`.
