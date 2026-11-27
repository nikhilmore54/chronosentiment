# RP-608-SCI-ADAPTER-REC — P1b live-label + ETH G0 audit

**Date:** 2026-11-27 Asia/Calcutta  
**Trigger:** Anuj “continue” after ML-1D offline ladder complete (DO_NOT_ADVANCE)  
**Rules:** `.cursor/rules/chronosentiment-core.mdc` §3b/§3c · Coralys no-touch · books never pooled · no Gate 5  
**Status:** **G0 FAIL** for both next packages — no sealed offline design authorized

---

## Context

ML-1D offline package finished: R1 → Ec1/Ec2 → H9 → R3 → R2. Primary **DO_NOT_ADVANCE**. Practical steer next names:

| Package | Intent |
| --- | --- |
| **P1b** | Label join = **live plan outcomes** (not only REC-001-H geometry) |
| **P2/ETH** | ETH 15m + label join · own sealed book (never pool BTC) |

---

## P1b — live plan outcomes

### Inventory

| Source | n | Span (UTC) | Notes |
| --- | ---: | --- | --- |
| `artifacts/rp608_econ_edge_diag/.../trade_plan_outcomes.jsonl` | 1014 | 2026-10-26 → 2026-11-06 (~11d) | Protocol `RP-608-TRADE-PLAN-OVERLAY-001` · authority 0% simulated · has `net_pnl_usdt` / friction |
| Session exports `session_export_*crypto_recommendation_plan_closed.jsonl` | 294 | 2026-11-02 → 2026-11-05 | BTCUSDT only |
| REC-001-H `BTCUSDT.jsonl.gz` (ML-1D labels) | ~17.5k lines | 2024 calendar | Geometry outcomes, not live plans |

Exit mix (1014): STOP_LOSS_HIT 531 · PROFIT_TARGET_HIT 359 · MAX_HORIZON_60M_EXPIRED 119 · ambiguous 5. Empirical win rate on `net_pnl_usdt>0` ≈ **38.4%**; mean net ≈ **−0.39 USDT**.

### Feature join feasibility

| Need | Available? |
| --- | --- |
| Decision-time 15m price+volume history at plan entry | **No** — `datasets/crypto_15m/binance_btcusdt_2024_15m.json` ends **2024-12-31**; live plans are **2026-10/11** |
| Chronological train / purge / val / sealed holdout | **No** — ~11 days cannot support ML-1D-scale firewall |
| Symbol field on primary 1014 journal | **Absent** (session exports are BTC-tagged) |

### G0 verdict — P1b

**PARTIAL UNBLOCK (2026-11-27):** BTC 15m 2026-09→11 ingested (`datasets/crypto_15m/binance_btcusdt_2026q4_15m.json`). Feature join works.

**Still FAIL for sealed ADVANCE package:** ~11d / 890 TP/SL joins — no train/val/holdout firewall.

**Observational transfer run:** `docs/research/rp608_sci_adapter_rec_ml1d_p1b_transfer.md` — frozen 2024 packs → live plans; E1 lifts WR+net vs E0; verdict `DO_NOT_ADVANCE_TRANSFER_OBSERVATIONAL`.

**Full sealed P1b still needs:** months more live-plan closures + symbol tags · not cut remine.

---

## ETH sealed package

| Need | Available? |
| --- | --- |
| `datasets/crypto_15m/*eth*` 15m bars | **Missing** |
| `datasets/recommendation/historical_crypto/ETHUSDT.jsonl*` | **Missing** (only `BTCUSDT.jsonl.gz`) |
| E9 ETH scenario JSON | Present but **not** a REC-001-H / Coralys→frozen-plan join |

### G0 verdict — ETH

**PASS (2026-11-27):** Ingested `binance_ethusdt_2024_15m.json` + REC-001-H `ETHUSDT.jsonl.gz`.

**Sealed ML-1D ETH package run:** `docs/research/rp608_sci_adapter_rec_ml1d_eth_holdout.md` — **ADVANCE_CANDIDATE** offline (never pooled; H11 skipped; no automatic twin).

---

## What this does *not* authorize

- Pi twin / Gate 5 / Coralys edits / soak cut ratchet  
- Treating overlay 11-day net as ML-1D ADVANCE evidence  
- Building ETH package on BTC holdout  

---

## Recommended Anuj choices

| Option | Meaning |
| --- | --- |
| **A. Close CDR-10** | ML-1D offline research complete · DO_NOT_ADVANCE stands |
| **B. Data ops for P1b** | Ingest 2025–2026 BTC 15m + journal hygiene, then re-run G0 |
| **C. Data ops for ETH** | Ingest ETHUSDT 15m + labels, then sealed ETH-only package |

---

*End — P1b/ETH G0 audit · 2026-11-27 Asia/Calcutta.*
