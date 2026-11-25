# RP-608 — Order Flow Side/Quantity Semantics Audit

**Protocol:** `RP-608-OF-SIDE-QTY-SEMANTICS-AUDIT-001`  
**Mode:** READ-ONLY INVESTIGATION  
**Date:** 2026-11-07  
**Authority:** 0.0% · Gate 2 v1 **CLOSED** (v2 split 2E/2X active 2026-11-15; this audit still forbids ±0.30 change) · baseline / OF ±0.30 **FROZEN**  
**Governance:** `.cursor/rules/chronosentiment-core.mdc` §3b/§3c · `.cursor/rules/rp608-ofx.mdc`  
**Authorization:** Explicit one-off semantic audit (not a discovery workstream; no formula/threshold change).

---

## 1. Executive finding

**Conclusion:** `NO_MISMATCH_FOUND_IN_AUDITED_PATH`

Within every audited production and research OF classification path, **bid-side quantity is paired with bid fields and ask-side quantity with ask fields**. The signed imbalance formula and LONG/SHORT thresholds are internally consistent with that pairing. **No bid/ask reversal, no cross-record side↔qty swap, and no sign flip of an already-signed value were confirmed in code.**

What **is** confirmed by direct implementation evidence:

1. The construct labeled “Order Flow” / `OF_SUP` / opposite-side OF exit is **L1 resting top-of-book (TOB) displayed quantity imbalance**, not executed aggressor trade flow  
   \(\Delta Q = Q_{\text{buy-aggressor}} - Q_{\text{sell-aggressor}}\).
2. Source fields on the OF path are resting book quantities (`bidQty`/`askQty` or Kite depth buy[0]/sell[0]).
3. Executed aggressor side and quantity are **not inputs** to baseline OF classification (and are **not available** on durable crypto/NSE OF journals — already locked in OFX Stage 0).

Therefore:

| Claim | Status |
| --- | --- |
| Buy qty assigned to sell label (or reverse) inside L1 formula | **Not found** |
| Bid/ask feed fields swapped vs exchange contract | **Not found** (Binance `B`→bid qty, `A`→ask qty; Kite buy depth→bid, sell depth→ask) |
| Resting book qty interpreted as executed buy/sell pressure under the name “order flow” | **Confirmed naming/construct misalignment** (not a pairing bug) |
| Historical OF_SUP / reverse-OF as validated *trade-flow* classifications | **Not accepted** — they classify **displayed L1 liquidity dominance** |
| Corrected aggressor OF will improve WR/net | **Not inferred** (out of scope) |

**Disposition for consumers:** treat baseline OF labels as **L1 book-imbalance support**, not aggressor-flow confirmation, until a separately authorized programme defines and journals executed-flow semantics.

---

## 1b. Corrected interpretation (governance lock — 2026-11-07)

**Status:** `LOCKED` · terminology / interpretation only · **no** code, threshold, Gate 2, or authority change.

We initially framed risk as a possible **side/quantity pairing** defect. The audit establishes something more fundamental: **baseline OF and intended volume/aggression-led Order Flow are different signals.** Baseline OF is not necessarily incorrectly implemented; it is a **different construct**.

| Construct | What it measures | What it can establish |
| --- | --- | --- |
| Baseline OF (`OF_SUP`, ±0.30, reverse-OF exit) | L1 resting bid/ask quantity imbalance | Relative **displayed** liquidity at the touch |
| Candle direction | OHLC price movement | Whether the bar closed up or down |
| Executed aggressor flow | Buy-aggressor vs sell-aggressor quantity | Direction of inferred or observed aggressive execution |
| Aggression + response | Executed aggression vs subsequent price response | Potential absorption / divergence candidates (research only) |

**Canonical example (not measurable by baseline OF):** candle closes red; executed buying aggression dominates; price still fails to advance (or sellers continue to push price down). That is disagreement between **aggression and price response**, not a candle contradiction — and **not** what L1 `OF_SUP` detects.

**Institutional label:** calling aggressive flow “institutional” remains a **hypothesis**. Aggression alone does not identify participant type.

### Binding rules (do not violate)

1. **Do not** modify baseline OF (±0.30, reverse-OF, holds, persistence, paper books).  
2. **Do not** reinterpret `OF_SUP` as executed buying/selling pressure.  
3. **Do not** use resting bid/ask imbalance as proof of absorption.  
4. **Do not** retrospectively relabel historical OF results.  
5. **Keep** RP-608 confirmation-only holding unchanged.  
6. **Keep** RP-608-OFX as the separately governed research home for aggression + response.  
7. **Do not** treat the two constructs as equivalent in future research, reports, or Gate discussions.  
8. **No** change to Gate 2, baseline config, or execution authority follows from this clarification alone.

**Next step from this clarification:** preserve the distinction in governance records (this section + decision ledger + holding state + INDEX). Not a formula change.

**Governance decision recorded:** `ACCEPT_CONSTRUCT_DISTINCTION` (option 1 clarified: keep L1 implementation frozen; correct terminology/interpretation only). Options 2–3 remain future-gated and are **not** authorized here.

---

## 2. Source schema and field semantics

### 2.1 Crypto (Binance)

| Source | Field | Documented meaning | Used as |
| --- | --- | --- | --- |
| WS `btcusdt@bookTicker` | `b` | Best bid price | `best_bid_price` |
| WS `btcusdt@bookTicker` | `B` | Best bid quantity | `best_bid_qty` / `bid_qty` |
| WS `btcusdt@bookTicker` | `a` | Best ask price | `best_ask_price` |
| WS `btcusdt@bookTicker` | `A` | Best ask quantity | `best_ask_qty` / `ask_qty` |
| REST `/api/v3/ticker/bookTicker` | `bidPrice` / `bidQty` / `askPrice` / `askQty` | Same L1 resting book | Same |

**Side semantics:** resting quote side (displayed liquidity), **not** aggressor.  
**Quantity semantics:** displayed TOB size at best bid/ask.  
**Executed trade tape / `aggTrade.isBuyerMaker`:** not on the OF mark path (OFX Stage 0: crypto aggressor **NOT FEASIBLE TODAY** from durable journals).

Evidence: `scripts/run_rp608_shadow_daemon.py` bookTicker qty map ~L1291–1305 (`B`→`best_bid_qty`/`bid_qty`, `A`→`best_ask_qty`/`ask_qty`); `scripts/rp608_trade_plan_overlay.py` REST book ~L645–649 + `fetch_live_btcusdt_book` ~L659 (alias). Line numbers refreshed 2026-11-25; semantics unchanged.

### 2.2 NSE (Kite depth packet)

| Source | Field | Meaning | Used as |
| --- | --- | --- | --- |
| Full-mode depth bytes 64–124 | Buy depth[0] qty/price | Resting buy (bid) | `bid_qty` / `best_bid` |
| Full-mode depth bytes 124–184 | Sell depth[0] qty/price | Resting sell (ask) | `ask_qty` / `best_ask` |

Evidence: `scripts/run_rp608_nse_live_probe.py` L72–93; Rust decode `adapters/chronosentiment/src/nse_market_events/decode.rs` L49–87.

**Aggressor / LTQ:** not decoded into production OF mark inputs (OFX Stage 0).

### 2.3 Tick-feature fallback

When card L1 qty path fails support, daemon may reclassify from `features_tick.imbalance` / `micro_vs_mid_bps` (`l1_from_tick_features`). That imbalance is computed from the **same** `TickQuote.bid_qty`/`ask_qty` bookTicker quantities (`scripts/rp608_crypto_tick_policy.py` L57–61). Not an independent trade-flow source.

---

## 3. Side-to-quantity mapping table

| Source side | Source quantity | Interpretation | Signed contribution in current OF |
| --- | --- | --- | --- |
| Resting bid (buy quote) | `B` / `bidQty` / depth_buy[0].qty | Pending buy-side **displayed** liquidity | \(+q_{\text{bid}}\) in numerator |
| Resting ask (sell quote) | `A` / `askQty` / depth_sell[0].qty | Pending sell-side **displayed** liquidity | \(-q_{\text{ask}}\) in numerator |
| Buy aggressor | *(absent on OF path)* | Aggressive buying | **Not used** |
| Sell aggressor | *(absent on OF path)* | Aggressive selling | **Not used** |

**Implemented signed measure:**

\[
\text{imbalance} = \frac{q_{\text{bid}} - q_{\text{ask}}}{q_{\text{bid}} + q_{\text{ask}}} \in [-1,+1]
\]

(None if denom ≤ 0.)

**Support rules** (`ORDER_FLOW_IMBALANCE_THRESHOLD = 0.30`):

| Position side | Qty condition | Microprice condition (if prices present) |
| --- | --- | --- |
| LONG | imbalance ≥ +0.30 | `micro_vs_mid_bps ≥ 0` (or skip if None) |
| SHORT | imbalance ≤ −0.30 | `micro_vs_mid_bps ≤ 0` (or skip if None) |

Microprice: \((P_{\text{ask}} q_{\text{bid}} + P_{\text{bid}} q_{\text{ask}}) / (q_{\text{bid}}+q_{\text{ask}})\); sign vs mid is consistent with standard TOB microprice (more bid qty → micro toward ask → positive vs mid).

**Exit:** opposite-side support (LONG exits when SHORT support fires; vice versa) — still L1 resting imbalance, not aggressor reverse flow.

---

## 4. Exact implementation call path

```
Binance WS bookTicker {b,B,a,A}
        │
        ▼
run_rp608_shadow_daemon._book_ticker_loop
  latest_book_ticker[best_bid_qty=B, best_ask_qty=A]
  TickQuote(bid_qty=B, ask_qty=A) → tick policy imbalance
        │
        ▼
evaluate_on_tick / decision_card emit
  classify_order_flow_support(side, l1_from_book_ticker(quote))
    └─ fallback: l1_from_tick_features(features_tick)
  stamp order_flow_supported / label / imbalance / source
        │
        ▼
TradePlanOverlayManager.process_decision_card
  plan OF stamps + _activate_order_flow_recommendation (crypto OF paper)
        │
        ▼
evaluate_order_flow_tick / evaluate_market_tick
  is_order_flow_exit_signal(active_side, L1)
    → classify_order_flow_support(opposite_side, L1)
  exit_reason OPPOSITE_SIDE_ORDER_FLOW → of_trade_outcomes.jsonl
```

**Shared mark (Python):** `scripts/rp608_order_flow_mark.py`  
- `l1_imbalance` L61–68  
- `classify_order_flow_support` L98–180  
- `is_order_flow_exit_signal` L27–58  
- `l1_from_book_ticker` L183–193  
- `l1_from_nse_quote` L196–224  
- `l1_from_tick_features` L276–285  

**Rust parity (NSE):** `adapters/chronosentiment/src/nse_market_events/of_mark.rs` L83–161 (same formula/threshold).

**Crypto OF paper sizing:** `of_entry_signal_qty` — LONG→bid qty, SHORT→ask qty (`rp608_trade_plan_overlay.py` L128–136) — resting same-side touch size, not aggressor size.

**Docs already describing L1 (not aggressor):** `docs/research/rp608_crypto_order_flow_recommendation.md` L8–13.

---

## 5. Current calculation and sign convention

| Item | Value |
| --- | --- |
| Formula | `(bid_qty - ask_qty) / (bid_qty + ask_qty)` |
| Sign | Positive ⇒ bid-heavy resting book; negative ⇒ ask-heavy |
| LONG support | Positive imbalance ≥ +0.30 (+ micro ≥ 0 when priced) |
| SHORT support | Negative imbalance ≤ −0.30 (+ micro ≤ 0 when priced) |
| Unsupported | `order_flow_supported=False`, `order_flow_label=None` (no negative badge) |
| Threshold | Frozen `0.30` (do not change in this audit) |
| Construct class | **Displayed L1 liquidity imbalance** |

Minimal worked example (existing test/source convention):

| Inputs | Imbalance | LONG support | SHORT support |
| --- | --- | --- | --- |
| bid_qty=90, ask_qty=10 | +0.80 | True (≥0.30) | False |
| bid_qty=10, ask_qty=90 | −0.80 | False | True |
| bid_qty=50, ask_qty=50 | 0.00 | False | False |

Interpretation under **executed-flow** language would be incorrect: these rows do **not** assert 90 units of buy-aggressor volume vs 10 sell-aggressor. They assert **displayed** bid size vs ask size at TOB.

---

## 6. Confirmed mismatches (pairing)

**None confirmed** for:

- Bid qty labeled as ask (or reverse) in ingest → classify  
- Buy-depth mapped to ask_qty in NSE decode  
- Downstream reinterpretation flipping the signed imbalance  
- Side taken from one record and quantity from another on the primary L1 path  

**Confirmed construct fact (not a pairing defect):** OF criteria compare resting bid/ask quantities and brand the result as order-flow support/exit.

---

## 7. Unverified semantics and missing source evidence

| Item | Status |
| --- | --- |
| Aggressor side on OF mark path | **ABSENT** — not an input |
| Executed buy/sell quantity on OF mark path | **ABSENT** |
| Whether any offline research script recomputes OF from trades | **Not found** in audited OF mark / overlay / NSE of_mark paths |
| Live exchange field letters vs Binance public contract | Mapping matches published `bookTicker` schema; **not** re-validated against a live packet capture in this audit |
| Whether human-facing “order flow” language caused analytical overclaim | Likely yes in OF_SUP / reverse-OF *interpretation*; code comments already say “L1” / “TOB” |

Mark any claim that baseline OF equals \(\Delta Q\) aggressor flow as **UNVERIFIED / FALSE relative to implementation**.

---

## 8. Existing test coverage and gaps

**Covered** (`tests/test_rp608_order_flow_mark.py`, `tests/test_rp608_nse_of_mark_event_ref.py`, Rust parity tests):

- LONG support when bid-heavy; SHORT when ask-heavy  
- Exit only on opposite-side confirm; neutral does not exit  
- NSE object → `l1_from_nse_quote` field pass-through  
- Threshold boundaries (±0.30)  
- Paper lifecycle / sizing using entry-side L1 qty  

**Gaps (tests can pass despite conceptual issues):**

- No test that distinguishes aggressor \(\Delta Q\) from resting imbalance  
- No fixture asserting Binance `B`/`A` letters against an exchange sample dump  
- No test forbidding resting qty from being described as trade flow  
- Reversed convention in **both** production and tests would still be green (self-consistent but wrong vs exchange) — mitigated by matching Binance/Kite published field names, not by independent oracle  

**This audit did not add or modify tests.**

---

## 9. Downstream artifact impact inventory

| Consumer | Uses OF how? | Affected by L1≠aggressor? |
| --- | --- | --- |
| Crypto OF paper (`of_trade_outcomes.jsonl`, reverse-OF exits) | Direct classify + exit | **Yes** — labels are L1 dominance, not trade-flow |
| Crypto plan stamps (`order_flow_supported` on plan/outcomes) | Direct card stamp | **Yes** for OF_SUP strata; plan TP/SL/60m geometry independent |
| NSE OF paper / mark | Direct L1 classify | **Yes** |
| NSE Estimand A | Entry/exit not OF-gated by this mark | **No** for A execution path; OF fields incidental if present |
| OF_SUP / imbalance bucket research (Phase 2 crypto/NSE OF entry quality, discovery v2 lineage) | Consumes stamped fields | **Yes** — interpret as L1 support slices |
| Opposite-side OF exit observations | Direct | **Yes** — reverse **book** confirm |
| H2 session-regime P4 **primary** (session WR dispersion on crypto plan) | Outcomes W/L by session | **No** for primary disposition — does not require OF fields |
| H2 P4 **secondary** OF_SUP split | Optional descriptive | Discovery window showed OF **Missing** (n=554); any later OF strata = L1 stamps only |
| RP-608-OFX Stage 0–1 | Separate aggression research | **Not the same calculation**; already treats aggressor as distinct / blocked on journals |
| Adaptive velocity / RANKING / WR discovery | Ranking on plan outcomes | **Indirect** only where OF features entered lineage (blocked rediscovery); not a side-swap bug |

Books remain separate; no pooling. Baseline execution thresholds untouched by this audit.

---

## 10. Historical classifications — potentially affected?

| Classification | Survive as stated? | Survive under L1-book reading? |
| --- | --- | --- |
| `ORDER_FLOW_SUPPORTED` / OF_SUP=True | Not as aggressor confirmation | Yes as “L1 imbalance supported side at mark time” |
| Imbalance buckets | Not as trade-flow intensity | Yes as \(|q_b-q_a|/(q_b+q_a)\) strata |
| Opposite-side OF exit | Not as aggressor reverse flow | Yes as opposite L1 support fire |
| Crypto/NSE OF paper PnL series | Accounting intact | Mechanism attribution must say **L1 reverse-imbalance exit** |
| H2 session-regime dominance | Independent of OF construct | Unaffected as primary finding |
| Claims that OF “validated” trade selection | Do not survive without redefinition | — |

**Do not** treat prior OF economic reports as validated aggressor-flow evidence.  
**Do not** silently rewrite historical labels in this audit.

Reconstruction: where `execution_quote` / depth journals retain bid/ask qty at decision time, L1 imbalance **can** be recomputed. Aggressor-based reclassification **cannot** be reconstructed from current OF journals (tape absent).

---

## 11. Evidence references (paths / symbols / lines)

| Evidence | Location |
| --- | --- |
| Imbalance formula | `scripts/rp608_order_flow_mark.py` `l1_imbalance` L61–68 |
| Support / sign rules | `classify_order_flow_support` L160–180 |
| Exit = opposite support | `is_order_flow_exit_signal` L27–58 |
| bookTicker normalize | `l1_from_book_ticker` L183–193 |
| Daemon B→bid qty, A→ask qty | `scripts/run_rp608_shadow_daemon.py` ~L1291–1305 (bookTicker); OF path uses same L1 qtys via `l1_from_book_ticker` |
| REST bidQty/askQty | `scripts/rp608_trade_plan_overlay.py` ~L645–649 · alias `fetch_live_btcusdt_book` ~L659 |
| Tick imbalance same formula | `scripts/rp608_crypto_tick_policy.py` L57–61 |
| NSE buy→bid, sell→ask | `scripts/run_rp608_nse_live_probe.py` L90–93; `decode.rs` L64–67 |
| Rust parity | `adapters/chronosentiment/src/nse_market_events/of_mark.rs` L83–161 |
| Product doc (L1 formula) | `docs/research/rp608_crypto_order_flow_recommendation.md` L8–13 |
| Aggressor unavailable | `docs/research/rp608_ofx_stage0_data_feasibility.md` L33–43, L78+ |
| H2 OF secondary Missing | `artifacts/rp608_econ_edge_diag/rp608_h2_crypto_plan_session_regime_p4.md` L43–49 |
| Tests (convention) | `tests/test_rp608_order_flow_mark.py` L31–45, L643–670 |

---

## 12. Governance decision (recorded)

| Decision | State |
| --- | --- |
| `ACCEPT_CONSTRUCT_DISTINCTION` | **RECORDED 2026-11-07** — §1b |
| Baseline OF implementation | **Unchanged** (not treated as a pairing bug requiring fix) |
| Terminology | Baseline OF = **resting-liquidity / L1 TOB imbalance**; not executed-volume directional OF |
| Formula / ±0.30 / reverse-OF / historical labels | **Frozen — no change** |
| Gate 2 / execution authority | **Unchanged** (0%) |
| Confirmation-only holding | **Unchanged** |
| Aggression + response research | **RP-608-OFX only** (separate governance) |

Still **not** authorized: ±0.30 changes, reverse-OF mutation, paper/live execution changes, Gate 2 reopen, H1/H3/H4 reopen, historical reclassification, or claims that a future aggressor construct improves WR/net.

Future options (not opened): separately authorize an executed-flow programme under OFX boundaries; impact quantification only after a definition exists.

---

## Strict boundary compliance

- No production/research code modified for this clarification  
- No OF threshold/formula/classification changes  
- No corrected OF implementation  
- No trading, backtests, discovery, threshold scans, or hypothesis evaluation  
- No frozen artifact regeneration  
- No baseline/OF config change  
- H1–H4 not reopened  
- Gate 2 / authority unchanged  
- No inference that corrected OF improves WR or profitability  
- Construct distinction locked in §1b so future work does not equate the two signals
