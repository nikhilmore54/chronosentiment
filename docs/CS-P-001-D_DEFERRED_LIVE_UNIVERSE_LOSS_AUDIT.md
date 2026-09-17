# CS-P-001-D — Deferred Live universe loss audit (frozen observation)

**Document type:** Product observation  
**Status:** Frozen — do not retune TARGET / STOP / HORIZON from this record  
**Date:** 2026-09-15  
**Tape:** CACHED_1M instant, sessions 2026-09-03 / 2026-09-04 / 2026-09-07  
**Parent:** CS-P-001 Stage B/C  
**Does not mutate:** `deferred_live.rs`, Paper Trader v0.2, Stage C `score_deferred_live`, INVERT, reassessment  
**Does not claim:** G-GATE predictive value  

`.cursor/rules/chronosentiment-core.mdc`: deterministic as-of events; no invented prices; parameters stay bounded.

This freezes the full-universe Deferred Live paper book and the fill-displacement measurement on that same ledger. It is not a trading-rule change.

---

## 1. Frozen book

| Question | Finding |
| --- | --- |
| How many ACT paper entries? | **34** |
| Realized losses? | **2** |
| Realized-loss rate of entire book | **5.9%** (2/34) |
| Closed-loss rate | **100% (2/2)** |
| TARGET failures? | **No — 0 TARGET** |
| HORIZON losses? | **No — 0 HORIZON** |
| Losses by direction | **2 SHORT, 0 LONG** |
| Low-OQS losses? | **No — OQS 80 and 81** |
| Common mechanism | **Adverse fill → compressed T0 stop distance → STOP** |
| Is this enough to retune? | **No — n=2** |

32 OPEN positions are tape-end marks, not losses. The unix horizon is `snap_unix + 300 minutes`; P4/TIME009 snap is 15:30 IST and the 1m tape ends 15:15, so HORIZON cannot fire on this same-day cache. 14 negative OPEN marks stay unrealized.

### The two realized losses

**TCS** 2026-09-04 SHORT OQS 80  
`2296.00 decision → 2346.00 fill → 2363.10 STOP`  
Fill **−2.18%** vs the SHORT decision price. Remaining frozen risk **+0.73%**. Realized **−0.73%** in 3 bars.

**PIDILITIND** 2026-09-07 SHORT OQS 81  
`1589.70 decision → 1625.50 fill → 1627.51 STOP`  
Fill **−2.25%**. Remaining frozen risk **+0.12%**. Realized **−0.12%** in 1 bar.

The closed losses are not a mysterious signal failure inside those two names. They occur because the paper fill and the frozen T0 risk geometry are materially separated.

---

## 2. Fill displacement (same 34 fills, no re-ingest)

Question: how much of the book's eventual outcome is distorted by the adverse fill itself?

For each ACT fill, frozen geometry is viewed at:

```text
T0 DecisionBrief.entry_price
        vs
Deferred Live paper_entry_price
        vs
frozen adaptive_risk / adaptive_target
```

Stop-span consumed = `(fill − decision) / (risk − decision)`. Risk is not rebased onto the fill.

### Fill vs decision

| Bucket | n | STOP | Mean fill | Mean stop consumed | Mean remaining stop | Mean outcome |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Favorable (≥ 0) | 12 | 0 | +0.88% | −25.6% | +4.91% | −0.91% |
| Mild adverse [−1%, 0) | 10 | 0 | −0.57% | +14.8% | +3.36% | +0.51% |
| Adverse [−2%, −1%) | 7 | 0 | −1.38% | +33.6% | +2.84% | +1.03% |
| Severe adverse (< −2%) | 5 | **2** | −2.58% | +69.5% | +1.58% | +1.07% |

Both STOPs sit in **severe adverse fill**. That bucket has five names, not two. Adverse fill is therefore a **regime**, not an isolated pair:

| Name | Date | Fill vs dec | T0 stop room | Remaining stop | Span consumed | Exit | Outcome |
| --- | --- | --- | --- | --- | --- | --- | --- |
| HCLTECH | 4 Sep | −3.35% | 4.09% | 0.71% | 82.0% | OPEN | +2.37% |
| SAIL | 4 Sep | −2.75% | 7.72% | 4.84% | 35.6% | OPEN | +0.34% |
| SAIL | 7 Sep | −2.36% | 3.90% | 1.51% | 60.5% | OPEN | +3.51% |
| **PIDILITIND** | 7 Sep | **−2.25%** | 2.38% | **0.12%** | **94.7%** | **STOP** | **−0.12%** |
| **TCS** | 4 Sep | **−2.18%** | 2.92% | **0.73%** | **74.5%** | **STOP** | **−0.73%** |

### T0 stop span consumed by fill

| Bucket | n | STOP | Mean remaining stop |
| --- | ---: | ---: | ---: |
| Fill away from stop (< 0) | 12 | 0 | +4.91% |
| Consumed 0–50% | 18 | 0 | +3.24% |
| Consumed 50–90% | 3 | **1 (TCS)** | +0.98% |
| Consumed ≥ 90% | 1 | **1 (PIDILITIND)** | +0.12% |

Remaining stop room falls monotonically as fill becomes more adverse. That is systematic envelope compression across the universe, not a two-name accident.

Tape-end marks do **not** follow the same slope: favorable fills have the worst mean mark (−0.91%), severe-adverse fills the best (+1.07%). Fill displacement distorts the **stop envelope**. It does not, on this tape, determine OPEN marks.

Severe adverse fill is not sufficient for STOP. HCLTECH filled worse than both losers (remaining stop 0.71%, comparable to TCS) and stayed OPEN +2.37% because the path never tagged the compressed boundary.

---

## 3. Verdict

1. Freeze the universe loss audit. Do not convert 32 OPEN names into losses.
2. Both realized losses share one mechanism: adverse SHORT fill consumed most of the T0 decision→risk span.
3. That mechanism is visible as a **bucket**, not only as two rows: 5/34 fills are < −2%; remaining stop shrinks as displacement worsens.
4. Do **not** rebase `adaptive_risk` onto `paper_entry_price`. Closed n=2. Changing geometry would turn an observation into a premature policy.

Artifacts: `datasets/deferred_live_universe_loss_audit.json`, `datasets/deferred_live_fill_displacement.json`.
