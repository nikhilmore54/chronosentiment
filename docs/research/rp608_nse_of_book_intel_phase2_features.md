# RP-608-NSE-OF-BOOK-INTELLIGENCE-001 — Phase 2 Features (Observational)

**Objective:** detect persistent bid support while price is falling; test subsequent upside after friction.  
**OF ±0.30 / reverse-OF paper:** **frozen**  
**Authority:** 0.0%

## Inputs

- `of_book_intelligence_v1/depth_snapshots.jsonl` (Phase 1 tape)
- `rp608_nse_bar_history.json` (1/2/4 completed-bar outcomes + candle colour)

## Outputs

| File | Content |
| --- | --- |
| `phase2_depth_features.jsonl` | Per-snapshot \(I_5\), bid/ask sums, mid Δ, apparent replenish/withdraw |
| `phase2_bid_support_while_falling_events.jsonl` | Candidate absorption windows |
| `phase2_feature_summary.json` | Aggregate + independence counts + sample-size verdict |

## Event definition (observational — not an OF threshold)

Over `persist_snaps=3` successive snapshots:

- \(I_5 \ge 0.20\) throughout
- Bid-sum does not erode more than 2%
- Mid price declines (bps &lt; 0)

Then join candle (red / below open) and forward net returns at 1, 2, 4 completed bars (half-spread in prices + statutory friction proxy).

## Reporting discipline (mandatory)

Keep these **separate** — 75 events are not 75 independent observations:

| Counter | Meaning |
| --- | --- |
| `n_events` | Detected windows (may cluster intra-symbol) |
| `n_unique_symbols` | Distinct names contributing events |
| `n_independent_sessions` | Distinct NSE cash session dates (IST) |
| `n_symbol_session_pairs` | Symbol×session units |

Summary also stratifies weak-candle outcomes by **replenish vs not** and **withdraw vs not** (incremental information vs static \(I_5\)).

## Claim gate

Short tapes are **telemetry only**. Negative fwd-1 is **not** hypothesis rejection while horizons are incomplete and independence counts are thin.

Gate (all required): enough events **and** ≥20 unique symbols **and** ≥3 sessions **and** deeper per-symbol snap history — plus populated fwd-1/2/4.

## Next measurement (preserve harness; accumulate tape)

1. Bid support persistence across consecutive snapshots while mid falls  
2. Whether replenishment adds information beyond static \(I_5\)  
3. Whether apparent withdrawal invalidates otherwise bullish-looking support  
4. Whether fwd-1 / fwd-2 / fwd-4 improve after Indian friction  
5. Consistency across symbols and sessions — not a few names  

**Sole next milestone (locked):** continue capture → accumulate independent sessions → rerun this harness.  

**Until then:** `TELEMETRY ONLY · INSUFFICIENT EVIDENCE`.  

**Forbidden:** additional experiments, threshold changes, OF promotion, paper-exit changes, authority change.  

**Next review lens:** complete fwd-1/2/4 · multi-session/symbol independence · replenish vs static \(I_5\) · withdrawal vs persistent support · net after friction.

## First runs (2026-11-02) — telemetry only

Initial pass (~24 snaps/symbol): 75 events; fwd-1 red/below-open ≈ −18 bps / 0% hit; fwd-2/4 pending. Re-run as history grows; do not promote.
