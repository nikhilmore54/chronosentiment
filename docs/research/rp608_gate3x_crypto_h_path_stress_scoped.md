# RP-608 — Gate 3X Crypto H* Path-Stress

**Protocol:** `RP-608-GATE3X-CRYPTO-H-PATH-001`  
**Status:** **EXECUTED · PASS** → draft Gate 5 H packet  
**Date:** 2026-11-15 (scoped) · **Run:** 2026-11-16  
**Depends on:** Gate 5 G deployed  

## Why

H60/H120 journal scratch@entry was ADVANCE_CANDIDATE but **optimistic**. Path stress required before any Gate 5 for min-hold.

## Checks (offline) — completed

1. For historical `STOP_LOSS_HIT` with hold &lt; 60s/120s: Binance **1s** mid path vs entry±friction.  
2. CF: scratch only if mid leaves band adverse then **revisits** inside `[entry, entry+T]`; else keep SL.  
3. Holdout Δnet + session robustness under PATH fill.

## Result

**`PASS_GATE3X_DRAFT_GATE5_H_PACKET_BOTH`**  
Holdout: H60_PATH Δnet **+255.36** · H120_PATH Δnet **+381.17** · paired share **1.0** both.  

- Review: `docs/research/rp608_gate3x_crypto_h_path_review.md`  
- Artifacts: `artifacts/rp608_gate3x_crypto_h_path/`  
- Gate 5 draft: `docs/research/rp608_gate5_packet_crypto_h_draft.md` (**not deployed**)

## Non-goals

Deploy H* without Gate 5 accept; change ATR stops; OF ±0.30; OPT-only live scratch.
