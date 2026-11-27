# RP-608-SCI-ADAPTER-REC-ML1D-001 — ETHUSDT sealed holdout

**Date:** 2026-11-27 Asia/Calcutta  
**Book:** `ETHUSDT_15M` only — **never pooled with BTC**  
**Rules:** `.cursor/rules/chronosentiment-core.mdc` §3b/§3c · Coralys sealed · no Gate 5  
**Data:** `binance_ethusdt_2024_15m.json` + REC-001-H `ETHUSDT.jsonl.gz`  
**Artifacts:** `docs/research/rp608_sci_adapter_rec_ml1d/eth_ml1d_*.json`  
**H11 pack:** `docs/research/rp608_sci_adapter_rec_p1/eth_p1_side_models.json` (ETH-native SCI-v1)

---

## Verdict

**DO_NOT_ADVANCE** (permanent under diagnose-first charter) · **no Pi twin** · no remine

Earlier ADVANCE_CANDIDATE (H11 skipped) is **superseded**. Completing H11 shows ETH SCI-v1 side generator beats ML-1D E3 on holdout WR and net — H11 falsifier fires. Architecture retired — see `rp608_sci_adapter_rec_diagnose_first_charter.md`.

| Gate | Result |
| --- | --- |
| E3 coverage | Pass (n=646) |
| E3 WR vs E0 | Pass (0.316 vs 0.293) |
| E3 net vs E0 | Pass (less negative) |
| E3 net ≥ max(E1,E2) | Pass (marginal) |
| H11 SCI-v1 (ETH-native) | **Fail** — H11 WR 0.331 / net −0.031 ≥ E3 0.316 / −0.043 |

---

## Holdout scorecard

| Exp | n | WR | cov | mean net proxy |
| --- | ---: | ---: | ---: | ---: |
| E0 | 2357 | 0.293 | 1.00 | −0.079 |
| E1 | 966 | 0.311 | 0.41 | −0.045 |
| E2 | 672 | 0.313 | 0.29 | −0.044 |
| **E3** | 646 | 0.316 | 0.27 | −0.043 |
| **H11** | **139** | **0.331** | 0.059 | **−0.031** |

R1 val Brier: 0.318 → 0.219. Extensions do not rescue. R2 earlier flag but E1+R2 thins/harms.

---

## ETH SCI-v1 P1 (H11 source · separate protocol)

Sealed ETH SCI-v1 offline (`RP-608-SCI-ADAPTER-REC-001`): holdout n=139 WR 0.331 vs base 0.293 · net −0.031 · **ADVANCE_CANDIDATE** as side generator. That does **not** authorize ML-1D twin; any ETH SCI twin remains Anuj-gated and book-isolated.

---

## vs BTC ML-1D (informational only — not pooled)

BTC ML-1D was already **DO_NOT_ADVANCE** (H2/H11). ETH dual-filter looked ADVANCE without H11; native H11 aligns both books on **DO_NOT_ADVANCE** for ML-1D.

---

## Non-goals

No Gate 5 · no Coralys edits · no pooling · no automatic Pi twin · no BTC pack on ETH.

---

*End — ETH ML-1D sealed holdout (H11 complete) · 2026-11-27 Asia/Calcutta.*
