# RP-608 OF paper entry quality (2026-11-21)

**Protocol:** `RP-608-OF-PAPER-ENTRY-QUALITY-001`  
**Book:** crypto OF paper only (not NSE this round; never pooled with CRYPTO_PLAN)  
**Authority:** 0% · paper booking filter only  
**Aligned with:** `.cursor/rules/chronosentiment-core.mdc` §3b–3d · `.cursor/rules/rp608-ofx.mdc`

## Locked non-goals

- No change to `ORDER_FLOW_IMBALANCE_THRESHOLD` (±0.30 L1 resting imbalance)
- No reverse-OF exit definition change (`OPPOSITE_SIDE_ORDER_FLOW` + min hold + opposite persist)
- No OFX aggression as OF entry
- No Gate 2X / Adaptive TP edits in this package

## Quality bar (observational)

Reverse-OF holds that print ~15–60 bps (or hold ≥10m with net > 0) vs remainder WEAK cohort.

Diag: `scripts/research/rp608_of_paper_quality_diag.py` on `of_trade_outcomes.jsonl`.

**2026-11-21 same-day result:** depth_quality fields (same_side notional, combined touch, |imb|) did **not** cleanly separate QUALITY vs WEAK (WEAK often denser). Coverage for a same_side floor raise from QUALITY p25 was not justified → **do not raise** `of_paper_min_same_side_notional_usdt`.

## Logic shipped

1. **Entry OF persistence** (`of_paper_entry_persist_ms` = 30_000): same-side OF must hold continuously before paper open (mirrors exit persist). Flicker opens suppressed.
2. **WS continuation:** `TradePlanOverlayManager.continue_of_paper_entry_persist` + shadow daemon bookTicker path completes arm→open when L1 stays supportive.
3. **Depth floors:** existing thin-opposite / min qty / min same_side notional retained; combined-touch floor stays **0**.
4. **Gate:** `RP608_OF_PAPER_QUALITY_GATE` (`1` suppress · `0` stamp-only). Stamp `of_entry_quality` on open and on `OF_PAPER_OPEN_SUPPRESSED` (`would_have_opened`).

## Success read

Fewer flicker entries; remaining reverse-OF exits should skew toward the bps/hold bar. Disclose W/L + payoff + net per cohort — not WR-only.
