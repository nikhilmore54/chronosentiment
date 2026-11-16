# RP-608-NSE-OF-BOOK-INTELLIGENCE-001 — Displayed Liquidity Pressure, Replenishment & Absorption

**Protocol ID:** `RP-608-NSE-OF-BOOK-INTELLIGENCE-001`  
**Disposition:** **SCOPED · IN DELIVERY** — single objective; no feature influence until history + outcome validation reviewed  
**Date locked:** 2026-11-02  
**Live order authority:** **0.0%**  
**Feed basis:** Kite Ticker **`mode=full`** five-level depth (already subscribed on the NSE daemon path)  
**Parent OF stack:** L1 ±0.30 inspection + `RP-608-NSE-OF-PAPER-TRADE-001` (**frozen**)

### Single objective (tight)

Develop an NSE Order Book Intelligence signal that detects **persistent buying support while price is still falling**, before the candle turns green.

Immediate delivery chain (no new protocols / side investigations / OF thresholds):

1. ~~Audit whether L1–L5 can be captured and retained as timestamped history~~ → Phase 0 done; was not retained.  
2. **Retain** L1–L5 as durable timestamped history (Phase 1 — in progress).  
3. Measure bid persistence, replenishment, price response on that history.  
4. Test whether those observations predict subsequent upside (after friction).

**Complementary stream (do not merge):**  
[`rp608_nse_of_intrabar_absorption_scoped.md`](rp608_nse_of_intrabar_absorption_scoped.md) — trade-level aggressor / execution pressure (`RP-608-NSE-OF-INTRABAR-ABSORPTION-001`). That stream needs aggressor classification coverage. **This** stream does **not** wait on that coverage.

**Parallel research (do not merge / do not feed paper):**  
[`rp608_ofx_order_flow_response_scoped.md`](rp608_ofx_order_flow_response_scoped.md) — `RP-608-OFX` may consume depth history as an input for liquidity-response observation; it must **not** change OF±0.30 paper rules or this protocol’s feature-influence freeze.

**Also exclude:** Estimand A / `RP-608-NSE-EXIT-PATH-001` · OF-REC-PERF cohorts until amended · any claim of institutional identity.

---

## 0. Governance state (preserve)

| Item | State |
| --- | --- |
| This protocol scope | **LOCKED** |
| NSE OF ±0.30 mark / reverse-OF / anti-churn | **FROZEN** |
| NSE OF paper ledger | **FROZEN** — book-intel must **not** open/close/suppress paper trades |
| Intrabar aggressor absorption protocol | **Independent** — compare later; never prerequisite |
| Live order authority | **0.0%** |
| Phase 0 depth-capture audit | **EXECUTED** 2026-11-02 — see [`rp608_nse_of_book_intel_phase0_depth_audit.md`](rp608_nse_of_book_intel_phase0_depth_audit.md) |
| Phase 0 verdict | Durable L1–L5 history **was** absent; root cause **CACHE_SCHEMA_THEN_PERSISTENCE** |
| Phase 1 durable L1–L5 history | **IN PROGRESS** — cache retains L1–L5; journal `of_book_intelligence_v1/depth_snapshots.jsonl` |
| Feature metrics (persistence / replenishment) | **Harness running** — results telemetry until history gate clears |
| Recommendation / paper influence | **FORBIDDEN** until Detect → Persist → Evaluate reviewed |

**Claim boundary:** we measure **displayed** liquidity, liquidity **changes**, and **price response**. We do **not** claim to know actual institutional buying or selling, hidden liquidity, confirmed cancellations, or executed aggressor volume from snapshots alone.

**Primary research question:** Does **persistent displayed bid support** predict subsequent upward price movement **even while the current candle remains red** (and the symmetric ask/distribution case)?

That is measurable — and a stronger foundation than adding another imbalance threshold.

---

## 1. Why a separate stream

Current OF uses top-of-book only:

\[
L1=\frac{BidQty_1-AskQty_1}{BidQty_1+AskQty_1}
\]

Aggressor-volume absorption is valuable but **feed-gated**. Five-level full-mode depth is **already on the wire**. Order Book Intelligence extracts pressure / replenishment / withdrawal / price-response from that book **without** requiring trade-side classification.

Eventually compare:

1. Book-intel alone  
2. Aggressor-absorption alone (if coverage allows)  
3. Joint agreement  

Keep results stratified; do not pool into Estimand A or OF paper outcomes.

---

## 2. What five-level depth can support

| Measurement | What it tells us |
| --- | --- |
| Bid / ask quantity (L1–L5) | Displayed buying / selling interest |
| Bid/ask imbalance (\(I_5\)) | Relative displayed liquidity across five levels |
| Spread | Cost / liquidity conditions |
| Depth concentration | Liquidity near best vs deeper levels |
| Bid / ask replenishment | Displayed depth recovers after depletion |
| Depth withdrawal | Apparent disappearance **without** proving cancellation |
| Mid-price movement | How price responds to displayed-liquidity changes |

**Especially interesting:** replenishment, apparent withdrawal, and mid-price response under persistent pressure.

**Limitation (authoritative):** snapshots are not an order-level event feed. Classify disappearance as **apparent withdrawal**, not confirmed cancel. No institutional labels.

---

## 3. Four independent signals

### A — Directional book pressure

\[
I_5=\frac{\sum BidQty_5-\sum AskQty_5}{\sum BidQty_5+\sum AskQty_5}
\]

Also retain L1 imbalance for comparison to the frozen ±0.30 mark.

### B — Liquidity replenishment

Detect whether bid depth repeatedly recovers after declining, or ask depth recovers after being consumed — across **event-time / frequent snapshots**, not one observation per 15m candle.

### C — Liquidity withdrawal

Detect significant disappearance of displayed depth between snapshots. Label **apparent withdrawal** only.

### D — Price response

Whether mid (or last) advances / retreats **despite** persistent displayed pressure — the absorption locus.

---

## 4. Observational classifications (book × price)

| Order book behaviour | Price behaviour | Classification |
| --- | --- | --- |
| Bid dominance + replenishment | Stable / limited downside despite selling pressure proxy | Potential **bullish absorption** |
| Bid dominance | Price advances | **Bullish pressure** |
| Ask dominance + replenishment | Stable / limited upside despite buying pressure proxy | Potential **bearish absorption** |
| Ask dominance | Price declines | **Bearish pressure** |

Maps the target scenario: **red / weak forming candle** + persistent bid replenishment + limited downward progress → potential bullish absorption **from the book alone**.

Illustrative sequences (synthetic charts in discussion) are **not** measured NSE data — live work must use real timestamped snapshots.

---

## 5. Dynamics requirement

A single imbalance reading is insufficient. Capture **evolution**:

- Bid depth strengthening while ask thins (or reverse)
- Mid/last still soft (or firm)
- Persistence across successive polls before class sticks

Cadence: event-time on WS full packets **or** fixed high-frequency snapshot journal (Phase 1 design). **Not** once-per-completed-15m-bar only.

---

## 6. Phased execution

| Phase | Work | Output |
| --- | --- | --- |
| **0** | Audit existing Kite depth capture vs what is durable today | **DONE** — coverage matrix + [`rp608_nse_of_book_intel_phase0_depth_audit.md`](rp608_nse_of_book_intel_phase0_depth_audit.md) |
| **1** | Record timestamped five-level snapshots (receipt + exchange time, stale flags) | Durable order-book history |
| **2** | Features: \(I_5\), L1, concentration, replenishment, withdrawal, spread dynamics | **Harness live** — [`rp608_nse_of_book_intel_phase2_features.md`](rp608_nse_of_book_intel_phase2_features.md); accumulate tape before claims |
| **3** | Classify pressure / potential absorption with persistence | Intrabar class stream |
| **4** | Subsequent price movement **after friction**; LONG/SHORT separate | OOS evidence |
| **5** | Compare incremental value vs frozen OF ±0.30 | Decision input only — no auto-promote |

### Controls (mandatory)

- Preserve OF ±0.30 marking and reverse-OF paper exits  
- Separate from Estimand A and from intrabar aggressor-volume experiment  
- Record missing snapshots and stale depth  
- Do **not** interpret displayed depth as executed volume  
- Sensitivity to spoofing / fleeting liquidity (short-lived depth spikes)  
- Evaluate LONG and SHORT separately  

---

## 7. Phase 0 — known feed facts (pre-audit baseline)

As of lock date, before the formal Phase 0 note:

| Component | Reality |
| --- | --- |
| Kite WS subscribe | `mode=full` on ≤119 NSE tokens (`rp608_kite_ticker_cache.py`) |
| Wire payload | 184-byte full packets include **5 bid + 5 ask** levels |
| Live ticker cache decode | Parses L1–L5 internally but **persists only L1** (`best_bid/ask`, `bid_qty/ask_qty`, LTP, cumulative volume) — **L2–L5 discarded** |
| REST `/quote` path | Daemon can read `depth.buy/sell` arrays at batch cadence; not a durable high-freq book tape |
| Live probe decoder | Retains fuller depth structs for short probes — not the continuous journal |

**Phase 0 must therefore answer:** can we extend durable capture to five-level snapshots with acceptable volume/staleness, without changing OF marks? If only L1 remains durable, book-intel collapses toward replenishment-of-L1 only and must say so explicitly.

---

## 8. Isolation matrix

| Surface | Interaction |
| --- | --- |
| OF ±0.30 stamp | Unchanged; book-intel runs **alongside** |
| NSE OF paper | Unchanged; no open/close/suppress from book classes |
| `RP-608-NSE-OF-INTRABAR-ABSORPTION-001` | Sibling research; **not** a prerequisite |
| OF-REC-PERF cohorts 1–4 | Do not pool until amended |
| EXIT-PATH / Estimand A | Excluded |
| Authority | Remains **0%** |

---

## 9. Success / shelve criteria (for later review)

**Useful if:** persistent bid (ask) support / replenishment classes show separable forward net outcomes after friction — especially **bullish absorption while price is still weak**.

**Shelve or narrow if:** fleeting-liquidity noise dominates; L2–L5 add nothing over L1; or classes fail persistence tests.

---

## 10. Disposition

| Item | Status |
| --- | --- |
| Scope lock | **LOCKED** 2026-11-02 |
| Two-stream architecture | Book-intel **and** aggressor-absorption — complementary |
| OF / paper / Arm A | **No changes** |
| Sole next milestone | **Continue capture → accumulate independent sessions → rerun existing Phase 2 harness** |
| Until milestone clears | **TELEMETRY ONLY · INSUFFICIENT EVIDENCE** — no claims, no OF/paper/authority change |
| Forbidden until review | Additional experiments · threshold changes · OF promotion · paper-exit changes |

**Next review will focus on:** complete fwd-1/2/4 · multi-session/symbol independence · replenish vs static \(I_5\) · withdrawal vs persistent bid support · net after friction.

No new imbalance threshold and no paper-rule change are authorized by this scope.
