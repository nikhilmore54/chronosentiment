# RP-608-NSE-OF-INTRABAR-ABSORPTION-001 — Directional Volume & Intrabar Absorption Diagnostics

**Protocol ID:** `RP-608-NSE-OF-INTRABAR-ABSORPTION-001`  
**Disposition:** **SCOPED · QUEUED** — documentation lock; no telemetry build until Phase 0 data gate  
**Date locked:** 2026-11-02  
**Live order authority:** **0.0%**  
**Parent OF stack:** L1 ±0.30 inspection + `RP-608-NSE-OF-PAPER-TRADE-001` (**frozen**)  
**Related (do not conflate):**
- `RP-608-NSE-OF-PAPER-TRADE-001` — paper lifecycle on L1 side-agree (unchanged)
- `RP-608-NSE-OF-BOOK-INTELLIGENCE-001` — displayed depth pressure/replenishment ([`rp608_nse_of_book_intelligence_scoped.md`](rp608_nse_of_book_intelligence_scoped.md)); **complementary, not a prerequisite or substitute**
- `RP-608-OF-REC-PERF-001` — OF vs non-OF recommendation outcomes (queued; separate)
- `RP-608-NSE-EXIT-PATH-001` — Estimand A exit-path falsification (**OF excluded**)
- `RP-608-OF-DEPTH-QUALITY-001` — depth telemetry only
- `RP-608-OFX` — parallel Order Flow Response Research ([`rp608_ofx_order_flow_response_scoped.md`](rp608_ofx_order_flow_response_scoped.md)); may reuse Phase 0 coverage findings; **must not** alter this protocol’s freeze or OF paper rules

---

## 0. Governance state (preserve)

| Item | State |
| --- | --- |
| This protocol scope | **LOCKED** |
| NSE OF ±0.30 mark / reverse-OF exit / anti-churn | **FROZEN** — must not be altered by this work |
| NSE OF paper ledger rules | **FROZEN** — absorption is **not** an entry/exit filter |
| Estimand A / Arm A / EXIT-PATH | **OUT OF SCOPE** |
| Live order authority | **0.0%** |
| Phase 0 data-gate | **NOT EXECUTED** |
| Phase 1 telemetry capture | **NOT AUTHORIZED** until Phase 0 passes or records coverage fail |
| Recommendation / paper influence | **FORBIDDEN** until Detect → Persist → Evaluate outcomes reviewed |

**Design principle:** Detect first. Confirm persistence second. Evaluate outcomes third. Only then consider whether absorption deserves to influence a recommendation.

No daemon OF rule change, no paper-ledger change, and no authority change are authorized by this scope.

---

## 1. Why this protocol exists

Current NSE OF marking uses **displayed L1 liquidity positioning**:

\[
L1=\frac{BidQty-AskQty}{BidQty+AskQty}
\]

A value of +0.51 means bid quantity exceeds ask quantity on the book. It does **not** establish that buyers are aggressively consuming the ask.

Three observations must stay separate:

| Signal | What it tells us |
| --- | --- |
| L1 imbalance | Which side has more **displayed** quantity |
| Candle volume | How much traded, **without** aggressor identity |
| Directional volume delta | Whether **buyer-** or **seller-initiated** trades dominate |

A stock can show \(L1=+0.60\) while directional volume delta \(VD=-0.45\): strong displayed bids with sellers initiating more of the actual trading. Treating that as unambiguously bullish is misleading.

**Central question:** Is current OF measuring liquidity positioning when confirmation of directional **execution pressure** (and, intrabar, absorption of opposing pressure) is what we actually need?

**Primary research intent (answered):** detect potential **accumulation / distribution before price confirms** — e.g. buying absorbed while the forming candle is still red — rather than waiting for the candle to close green. That capability is named **Intrabar Absorption Detection**.

This is **not** proof of institutional participation. It is an observable footprint. Never label a participant institutional without participant-level evidence.

---

## 2. Phenomena and four observational classes

### 2.1 Absorption vs initiative

| Class | Intrabar interpretation |
| --- | --- |
| **A — Bullish absorption** | Sellers initiate / price soft; downside progress limited; buying absorbs selling |
| **B — Bullish initiative** | Buyers initiate aggressively and price responds upward |
| **C — Bearish absorption** | Buyers initiate / price firm; upside progress limited; selling absorbs buying |
| **D — Bearish initiative** | Sellers initiate aggressively and price responds downward |

Example footprint (bullish absorption): candle trading below open (red forming), elevated relative volume, aggressive sell prints met with bid replenishment, limited downward progress → possible accumulation. Reverse for distribution into a green forming candle.

### 2.2 Independent dimensions (do not collapse)

| Dimension | Measures |
| --- | --- |
| Price response | Intrabar direction vs open, range, close location (when available) |
| Aggressor flow | Buyer- vs seller-initiated volume (+ unclassified) |
| Absorption | Signed volume vs price progress / impact; liquidity replenishment where observable |

Useful composite (investigational only — not a trading rule):

\[
\text{Price Impact}=\frac{\text{Signed Price Change (bps)}}{\text{Signed Volume / Relative Volume}}
\]

Evaluate **alongside** volume delta; never as a standalone gate.

---

## 3. What must be recorded (when data exists)

At every OF evaluation **and** on an intrabar poll cadence (Phase 1+), capture observational fields:

| Field | Purpose |
| --- | --- |
| `signal_direction` | LONG / SHORT / none (existing recommendation context) |
| `l1_imbalance` | Existing book imbalance |
| `candle_volume` / `relative_volume_20` | Total / vs prior 20 completed candles |
| `buy_initiated_volume` | Aggressive buying (classified) |
| `sell_initiated_volume` | Aggressive selling (classified) |
| `volume_delta` / `volume_delta_ratio` | \(V_{buy}-V_{sell}\) and normalized \(VD\) |
| `volume_direction_support` | Supports / opposes / neutral vs `signal_direction` |
| `classification_coverage` | Classified ÷ (classified + unclassified) |
| `price_impact_bps_per_rel_vol` | Signed impact metric (investigational) |
| `close_location` | Near high / mid / low when bar-complete |
| `bid_ask_replenish_flags` | Depth replenish / deplete observations |
| `absorption_class` | A/B/C/D or `unclassified` |
| `onset_utc_ms` / `persist_ms` | Signal onset and persistence |
| `event_timestamp` | Align with OF stamp / poll |
| `outcome_1_2_4_bar_net_bps` | Subsequent friction-aware outcomes (offline join) |

**Forbidden reconstruction:** do **not** manufacture buyer/seller volume from OHLCV candle colour or close direction.

---

## 4. Contingency table (outcome analysis — offline only)

Once coverage is adequate, classify each OF-aligned observation:

| L1 support | Directional volume | Interpretation |
| --- | --- | --- |
| Supports | Supports | Book and executed flow agree |
| Supports | Opposes | Potential false-positive OF mark |
| Opposes | Supports | Potential hidden directional demand |
| Opposes | Opposes | Both disagree with recommendation |

Compare subsequent outcomes **separately for LONG and SHORT**, using **realized net after friction**. Do not pool with Estimand A or EXIT-PATH cohorts.

---

## 5. Data constraint (authoritative)

Aggressor side is an **inference** from timestamp-aligned **trade prints and contemporaneous quotes**, with an explicit **unknown / ambiguous** bucket.

### 5.1 Current ChronoSentiment NSE feed (as of lock date)

| Source | What we have | Gap for this protocol |
| --- | --- | --- |
| Completed 15m OHLCV | Unsigned candle volume | Cannot recover aggressor |
| Kite REST `/quote` depth | L1–L5 displayed qty | Liquidity positioning only |
| Kite WS `mode=full` (daemon ticker cache) | LTP, cumulative `volume`, bid/ask L1 (cache currently drops `last_traded_quantity` / LTT) | Condensed quote snapshots, **not** a durable trade tape |
| NSE OF paper / OF stamps | L1 ±0.30 side-agree | Intentionally unchanged |

**Implication:** Phase 0 must prove whether a durable, timestamp-aligned trade+quote stream (or a rigorously bounded proxy with reported coverage) can be built from Kite full-mode deltas (`Δvolume`, `last_traded_quantity`, LTP vs bid/ask) without inventing classifications. If coverage is insufficient, **record the limitation** and stop — do not fake \(VD\) from candles.

### 5.2 Labels that are never allowed

- “Institutional buy/sell” without participant-level evidence
- Treating unclassified volume as opposing or supporting
- Using absorption class as an OF paper entry/exit reason

---

## 6. Phased execution plan

| Phase | Work | Gate |
| --- | --- | --- |
| **0 — Data gate** | Inventory printable fields; measure classification coverage on a live cash window; document unknown rate | **PASS** (≥ declared coverage threshold, TBD in Phase 0 note) **or FAIL+stop** with written limitation |
| **1 — Observational capture** | Journal fields in §3 alongside existing OF stamps; no UI promotion required | Phase 0 pass; OF ±0.30 untouched |
| **2 — Persistence** | Require multi-poll persistence before class sticks; report onset→confirm latency | Phase 1 rows exist |
| **3 — Outcome evaluation** | 1/2/4-bar (and/or session) net-after-friction by class × side; L1×VD contingency | Offline only; separate from OF-REC-PERF until reviewed |
| **4 — Influence decision** | Whether absorption may later inform marks/filters | **Explicit human gate** — default **no** |

---

## 7. Isolation from frozen OF / paper / Estimand A

| Surface | Interaction |
| --- | --- |
| OF ±0.30 inspection stamp | Unchanged; absorption runs **alongside** |
| `RP-608-NSE-OF-PAPER-TRADE-001` | Unchanged; absorption **must not** open/close paper trades |
| OF-REC-PERF four cohorts | Absorption telemetry is a **fifth observational stream** — do not pool into cohort 1–4 until a later amendment |
| Estimand A / EXIT-PATH-001 | **Excluded** |
| Live authority | Remains **0%** |

---

## 8. Success / falsification criteria (for later review)

**Useful if:** absorption / initiative classes (with adequate coverage) show separable forward net outcomes after friction, especially for **bullish absorption while price is still weak** (early accumulation hypothesis).

**Falsified / shelved if:** coverage chronically too low; classes unstable without persistence; or net outcomes after friction are indistinguishable from L1-only OF marks.

---

## 9. Disposition

| Item | Status |
| --- | --- |
| Scope lock | **LOCKED** 2026-11-02 |
| Primary intent | Intrabar accumulation/distribution **before** candle colour confirms |
| OF / paper / Arm A | **No changes** |
| Next authorized step | **Phase 0 data-gate note only** (coverage measurement) — not threshold or entry work |

**Reference (conceptual, external):** absorption as opposing aggression vs limited price progress is a known order-flow footprint framing (e.g. Bookmap absorption indicator literature). ChronoSentiment implements only what our feeds can classify with explicit coverage.
