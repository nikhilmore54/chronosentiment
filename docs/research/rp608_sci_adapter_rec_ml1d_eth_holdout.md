# RP-608-SCI-ADAPTER-REC-ML1D-001 — ETHUSDT sealed holdout

**Date:** 2026-11-27 Asia/Calcutta  
**Book:** `ETHUSDT_15M` only — **never pooled with BTC**  
**Rules:** `.cursor/rules/chronosentiment-core.mdc` §3b/§3c · Coralys sealed · no Gate 5  
**Data:** `binance_ethusdt_2024_15m.json` + REC-001-H `ETHUSDT.jsonl.gz`  
**Artifacts:** `docs/research/rp608_sci_adapter_rec_ml1d/eth_ml1d_*.json`

---

## Verdict

**ADVANCE_CANDIDATE** (offline gates on this book) · **no Pi twin until Anuj arm** · net still negative (disclose).

| Gate | Result |
| --- | --- |
| E3 coverage | Pass (n=646) |
| E3 WR vs E0 | Pass (0.316 vs 0.293) |
| E3 net vs E0 | Pass (less negative) |
| E3 net ≥ max(E1,E2) | Pass (marginal) |
| H11 SCI-v1 | **Skipped** (BTC pack must not score ETH) |

---

## Holdout scorecard

| Exp | n | WR | cov | mean net proxy |
| --- | ---: | ---: | ---: | ---: |
| E0 | 2357 | 0.293 | 1.00 | −0.079 |
| E1 | 966 | 0.311 | 0.41 | −0.045 |
| E2 | 672 | 0.313 | 0.29 | −0.044 |
| **E3** | 646 | **0.316** | 0.27 | **−0.043** |

R1 val Brier: 0.318 → 0.219. Extensions do not rescue further. R2 ACI earlier flag but E1+R2 thins/harms (same pattern as BTC).

---

## vs BTC ML-1D (informational only — not pooled)

BTC sealed holdout was **DO_NOT_ADVANCE** (H2/H11). ETH clears the same offline gate table without H11. Books remain separate; no cross-book promote.

---

## Non-goals

No Gate 5 · no Coralys edits · no pooling · no automatic Pi twin from ADVANCE_CANDIDATE alone.

---

*End — ETH ML-1D sealed holdout · 2026-11-27 Asia/Calcutta.*
