# ETH SCI-v1 P1 — frozen package audit (conditional twin gate)

**Date:** 2026-11-27 Asia/Calcutta  
**Protocol:** `RP-608-SCI-ADAPTER-REC-001`  
**Book:** `ETHUSDT_15M` only — **never pooled with BTC**  
**Rules:** `.cursor/rules/chronosentiment-core.mdc` §3b/§3c · Coralys sealed · no Gate 5  
**Parent:** diagnose-first charter · Anuj conditional GO for ETH twin

---

## Audit checklist

| Check | Result | Evidence |
| --- | --- | --- |
| Frozen pack identity | **PASS** | `docs/research/rp608_sci_adapter_rec_p1/eth_p1_side_models.json` · sha256 `082251d6eb0738c5feb7977a92c046997c199a0c7133b1115f8f5d9fa222e326` · 3736 bytes |
| Protocol / schema | **PASS** | `RP-608-SCI-ADAPTER-REC-001` · `price_volume_15m_v1` · schema_hash `f0cc1fe4…6b5f` |
| Ladder / cuts frozen | **PASS** | Ladder **A** · `p_min=0.38` · `p_lb=0.18` — no soak ratchet |
| Chronological firewall | **PASS** | Train→purge→val→sealed holdout Nov–Dec 2024 · purge 30 bars |
| Label construction | **PASS (disclosed)** | REC-001-H: TARGET_BEFORE_RISK=win · RISK_BEFORE_TARGET=loss · HORIZON censored · decision-time features only |
| Sample sufficiency | **PASS (thin coverage)** | Holdout enter_n=**139** · coverage **0.059** · baseline_n=2357 |
| Friction / net accounting | **DISCLOSE** | Holdout mean_net_proxy **−0.031** · payoff asym **1.87** · WR lift vs base (0.331 vs 0.293) with **still-negative net** |
| ML-1D independence | **PASS** | Package report has **no** ML-1D keys/deps · H11 role only as comparator for DNA ML-1D |
| Book pooling | **PASS** | Book field `ETHUSDT_15M` · never applied BTC pack |
| Coralys / Gate 5 | **PASS** | Untouched |

---

## Holdout scorecard (frozen)

| Fold | n | WR | base | cov | net proxy |
| --- | ---: | ---: | ---: | ---: | ---: |
| validation | 82 | 0.366 | 0.318 | 0.035 | +0.041 |
| **holdout** | **139** | **0.331** | 0.293 | 0.059 | **−0.031** |

Offline verdict remains **ADVANCE_CANDIDATE** (WR vs baseline). Economic honesty: **net still negative** — twin soak must not promote on WR alone (§3b / Gate 2 v2).

---

## Conditional twin GO

| Gate | Status |
| --- | --- |
| Integrity / independence audit | **Cleared** |
| Anuj explicit arm | **Still required** (this audit ≠ arm) |
| Book id (proposed) | `CRYPTO_PLAN_SCI_ADAPTER_REC_ETH` (separate dir; never pool with BTC SCI / CLEAN) |
| Pack path | `eth_p1_side_models.json` only |
| Stack | Prefer **P1a-style clean** path for ETH if armed (DI/regime ownership) — or document stacked inheritance explicitly |
| ML-1D | Not in path |

**Recommendation:** Audit supports **conditional GO** for an independent ETH SCI-v1 observational twin **after Anuj arm**. Do not claim live expectancy from offline WR alone.

---

*End — ETH P1 frozen audit · 2026-11-27 Asia/Calcutta.*
