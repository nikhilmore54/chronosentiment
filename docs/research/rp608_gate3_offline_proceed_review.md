# RP-608 Gate 3 Offline Proceed — Review

**Date:** 2026-11-15  
**Runner:** `scripts/research/rp608_gate3_offline_proceed.py`  
**Results:** `artifacts/rp608_gate3_offline/gate3_offline_proceed.md`  
**Authority:** 0%  

## Dispositions

| Track | Result | Action |
| --- | --- | --- |
| **Gate 3X Crypto G** | `PASS_GATE3X_DRAFT_GATE5_PACKET` | Draft Gate 5 packet written — **await explicit accept before code change** |
| **Gate 3E NSE BAR_ge_12** | `CONFOUND_DOMINANT` | **Do not promote** — late bars inflate square-off share (+0.44); within-day pairs only 79/219 favor late |

## NSE confound (why BAR_ge_12 dies)

Late entries are not cleaner selection; they hit a different exit stack (much more mandatory square-off, less max-hold path). Timing filter ≠ entry quality.

## Crypto G (why it lives)

Ambiguous TP∧SL → TP is a pure tie-break on known marks; holdout Δnet +249.8 USDT with session robustness and mid-haircut still non-negative for promote path.

## Next

1. **You accept/reject** Gate 5 draft for G (`docs/research/rp608_gate5_packet_crypto_g_draft.md`).  
2. NSE: drop BAR_ge_12; resume bar-history restore for 2X path-continue.  
3. H* min-hold: only after path stress (not in this packet).  
