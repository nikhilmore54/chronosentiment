# CS-P-001-C — Deferred Live reassessment sidecar (2026-09-07)

**Document type:** Experimental record  
**Status:** Closed for this branch  
**Date:** 2026-09-14  
**Tape:** CACHED_1M session 2026-09-07 (instant)  
**Parent:** CS-P-001 Stage B/C  
**Does not mutate:** `deferred_live.rs`, Paper Trader v0.2, Stage C `score_deferred_live`, `/paper-trades`  
**Does not claim:** G-GATE predictive value  
**Does not open:** confirmation-of-deterioration experiment (parked)

`.cursor/rules/chronosentiment-core.mdc`: deterministic as-of events; no invented prices.

This is a **sidecar measurement** over the frozen Deferred Live book. It is not a trading mechanism and not a production reassessment rule.

---

## 1. Frozen book (Stage C baseline)

9 ACT decisions entered on 2026-09-07. 8 remained OPEN. 1 STOP.

| Stage C | Value |
|---------|--------|
| Mean open mark | **+0.77%** |
| Realized STOP (PIDILITIND) | **−0.12%** |
| Mean fill vs DecisionBrief | **−1.10%** |

OPEN = mark-to-tape. TARGET / STOP / HORIZON = realized. Performance is descriptive.

---

## 2. Sidecar hierarchy (same observation stream)

```text
Frozen Deferred Live
        │
        └── Stage C marks
                │
                └── Reassessment sidecar
                       ├── v0.2 path-shape       −5.89 pp Δ
                       └── adverse-mark gate     −3.15 pp Δ
```

INVERT disabled. Fills remain observed `paper_entry_price`. Evaluator unchanged.

| Rule | Exits | Helped | Hurt | Suppressed | Mean Δ | Sum Δ |
|------|------:|-------:|-----:|-----------:|-------:|------:|
| Path-shape alone (v0.2) | 8 | 1 | 7 | — | −0.74% | **−5.89 pp** |
| Path-shape + as-of mark < 0 | 5 | 1 | 4 | 3 | −0.63% | **−3.15 pp** |
| No reassessment (frozen) | 0 | — | — | — | — | **0** |

PIDILITIND: frozen STOP at bar 1; neither sidecar rule fires (min-hold 6).

---

## 3. Verdict

1. **Path-shape alone → reject.** Exits too readily while positions are still profitable.
2. **Path-shape + adverse mark → better, not acceptable.** Preserves the three obvious green-position cases and keeps IDEA's help. Still cuts four temporary underwater positions.
3. **No reassessment → remains the frozen baseline.**

The remaining failure mode is not “don't exit winners.” It is:

> An adverse mark is not sufficient evidence that the original thesis has actually deteriorated.

The sidecar may see a legitimate adverse mark **as of the decision point**. Subsequent recovery cannot be used in that as-of decision.

---

## 4. Per-name record (gated vs frozen)

| Name | Frozen | v0.2 Δ | Gated | Note |
|------|--------|--------|-------|------|
| IDEA | OPEN −2.83% | **+1.58%** | **+1.58%** | Help retained |
| JSWSTEEL | OPEN +1.95% | −1.30% | held | Green at structure print |
| RELIANCE | OPEN +0.82% | −0.68% | held | Green at structure print |
| SAIL | OPEN +3.51% | −0.77% | held | Green at structure print |
| COALINDIA | OPEN −0.06% | −0.56% | −0.56% | Temporary dip / still hurt |
| BANDHANBNK | OPEN +0.27% | −1.10% | −1.10% | As-of adverse, then recovered |
| BAJAJFINSV | OPEN +1.44% | −1.51% | −1.51% | As-of adverse, then recovered |
| JUBLFOOD | OPEN +1.06% | −1.55% | −1.55% | As-of adverse, then recovered |
| PIDILITIND | STOP −0.12% | — | — | Frozen STOP |

IDEA: `−2.83% → −1.25%` at the as-of exit.

---

## 5. Parked next question

When resumed, investigate **confirmation of deterioration**, not a more elaborate adverse threshold.

Do not graft onto Stage C. Do not enable on 8080 by default.

**Conclusion:** the adverse-mark gate is a useful diagnostic improvement, **not a production reassessment rule**.
