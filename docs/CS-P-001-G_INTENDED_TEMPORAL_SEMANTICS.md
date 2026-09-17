# CS-P-001-G — Intended session/brief temporal semantics (reading)

**Document type:** Product-contract reading  
**Status:** Frozen — ambiguity closed; implementation not authorized; do not amend  
**Date:** 2026-09-15  
**Parent:** CS-P-001-F (investigation inventory), CS-P-001-E (frozen), CS-P-001-D (frozen)  
**Does not mutate:** `deferred_live.rs`, auto-arm, snap gate, date-shift join, delayed fill, TARGET / STOP / HORIZON  
**Does not claim:** G-GATE predictive value  
**Does not select** A/B/C/D for implementation  

`.cursor/rules/chronosentiment-core.mdc`: deterministic as-of events; no invented prices.

CS-P-001-F listed four join candidates. This record asks what the **existing product contract and session workflow already imply**, without treating that implication as a patch.

---

## Frozen conclusion

This paragraph is the freeze. Do not soften, strengthen, or implement from it.

CS-P-001-G closes the ambiguity without making the implementation decision.

Live Stage B:

```text
DecisionBrief
    ↓
ARM
    ↓
WAITING FILL
    ↓
later MarketObservation
    ↓
PAPER_ENTER
```

Given LIVE-005 `1000` as **15:30 IST cash close**:

```text
T0 = 15:30 IST
same-day 09:15 = pre-T0       ✗
same-day after 15:30 = none   ✗
next 09:15 = first post-T0    ✓
```

Therefore **C ≡ D** under the existing as-of semantics.

The conflict is specifically:

| Contract/job | Semantics |
| --- | --- |
| Live Stage B / CS-P-002 / TIME-009 | **C ≡ D — next-session open** |
| Current cached `--session DATE` harness | **A — same-day first bar** |

The documentation does **not** currently declare A as an intentional exception.

**There is nothing to patch yet. The product must first name which job the cached session harness represents.**

Until that semantic decision is explicitly made: no snap gate, no date shift, no delayed fill, and no stop rebasing.

Pinned implementation remains `session_auto_arm_opens_on_observation_before_snap_unix`.

---

## 1. Two workflows on one runtime

| Path | How a fill happens | Clock relationship |
| --- | --- | --- |
| Live / Cockpit Stage B | DecisionBrief exists → arm (auto or `POST /arm`) → **WAITING FILL** → later `MarketObservation` → `PAPER_ENTER` | Observation is subsequent to the brief |
| Cached `--session DATE` | `begin_session(date)` then `cached_session_tape(date)` from IST 09:15 | Observation can precede `snap_unix` |

Cockpit empty-state copy (manual path): *Arm a DecisionBrief and supply live MarketObservation events.* `WAITING FILL` is ACT + armed + no paper row. That workflow does not say “replay the cohort date from the open.”

---

## 2. What the as-of contract already says

CS-P-001 §4: no field derived from information **after** the decision as-of.

CS-P-002 §2 temporal firewall, for decision timestamp `T`:

- Decision inputs are as-of `≤ T`.
- Simulated / paper execution may use market evolution **after `T` only as outcome**, never as input.

TIME-009 / INTRA-001 on the same LIVE-005 lineage: bars **strictly after** `source_snapshot_timestamp`. LIVE-005 `1000` is **10:00 UTC = 15:30 IST cash close**.

If `T` is that close snap, then:

- Same-day 09:15 is **before `T`**. Using a 15:30 DecisionBrief to fill at 09:15 treats a decision that did not exist at fill time as if it did.
- Same-day bars after 15:30: the 1m tape ends ~15:15. Candidate **B is empty** on NSE regular hours.
- First bar strictly after close `T` is the **next IST 09:15**. Candidate **C and D are the same join** whenever T0 is cash close.

Paper Trader v0.2 (`V02_START_BAR = 12`) is a different clock (intraday bar index on a historical walk). It is not the Deferred Live unix horizon and does not resolve this join.

---

## 3. What this does *not* settle

The **live/paper as-of contract** points at **C ≡ D**: fill on the first observation after close `T` (next session open).

The **cached session harness** (`DEFERRED_LIVE_DATE` + `cached_session_tape`) was written as a **same-calendar-day controlled clock**. That is candidate **A** by construction. No product sentence says “CACHED_1M session mode is exempt from CS-P-002 §2.” None says it must obey TIME-009 either.

So intended semantics are still **two conflicting jobs**, not one winner:

| Job | Implied join | If chosen |
| --- | --- | --- |
| Honor Stage B / CS-P-002 / TIME-009 as-of | C ≡ D (next open after close snap) | Cached same-date 09:15 fills are out of contract |
| Honor `--session DATE` as a same-day tape | A (calendar date + first bar) | Clock inversion vs close snap stays an explicit exception |

Choosing either is a product-clock decision. It is not implied strongly enough to patch the driver from this reading alone.
