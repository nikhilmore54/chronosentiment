# RP-608 — Hit-rate / directional-accuracy literature register

**Date:** 2026-11-27 Asia/Calcutta  
**Author:** Anuj (extraction) · worker formalization  
**Rules:** `.cursor/rules/chronosentiment-core.mdc` §3b/§3c · Coralys no-touch · books never pooled  
**Status:** LITERATURE REGISTER ONLY — **not** Coralys targets · **not** SELECT · **not** implementation  
**Hard rules:** Directional accuracy ≠ trading W/L. Selective accuracy ≠ book win rate. Pooled crypto ≠ BTC-only. Paper % ≠ ACT cuts. Thresholds from frozen ChronoSentiment eval only.

**Companion:** quantitative evidence register (conformal / TC / BTC microstructure)  
**Store mirror:** Agent store `docs/sci-adapter-rec-hit-rate-literature-register.md`

---

## 0. Headline for Coralys (read first)

| Claim | Status |
| --- | --- |
| Peer-reviewed **15m BTC** directional accuracy in JFDS 2021 | **~53–54%** OOS (ensemble **53.786%**) |
| Published 70–100% “hit rates” elsewhere | Different horizon / selection / pooling / design — **do not** use as Coralys benchmarks without full protocol audit |
| Measurable 15m predictability | Yes (above 50% under paper’s binomial tests) |
| Economically viable 15m strategy in that paper | **No** after transaction costs (authors) |
| Replace Coralys with these models | **No** — measure reliability / selective use / economic relevance only |

**Literature reference (not target):** 53.786% ensemble 15m hit rate (Jaquart et al. 2021, Table 3).

---

## 1. Priority paper A — Jaquart, Dann & Weinhardt (2021)

**Short-term bitcoin market prediction via machine learning**  
*Journal of Finance and Data Science* 7:45–66 · https://doi.org/10.1016/j.jfds.2021.03.001

### Design

| Item | Reported |
| --- | --- |
| Horizons | 1, 5, 15, 60 minutes |
| Models | GRU, LSTM, FNN, LR, GBC, RF, Ensemble |
| Features | Technical, blockchain, sentiment/interest, asset |
| Sample | ~Mar–Dec 2019 (Bloomberg/Twitter/Blockchain.com); train ~5m / val ~1m / test ~3m |
| Target | Binary vs **train-median** m-minute return (balanced train classes) |

Best-by-horizon (authors): 1m GRU/LSTM competitive; 5m GBC; 15m Ensemble; 60m LSTM. Tech features most important for most methods. Predictability increases with horizon in their setup.

### Table 3 — OOS directional accuracy (15m focus)

| Model | 15m hit rate |
| --- | --- |
| GRU | 53.649% |
| LSTM | 53.197% |
| FNN | 52.082% |
| Logistic regression | 51.960% |
| GBC | 53.728% |
| Random Forest | 53.464% |
| **Ensemble** | **53.786%** |

Ensemble − GRU ≈ **0.137 pp** — **not** statistically significant (authors). All models significantly above 50% under their binomial tests.

### Economic note (same paper)

Quantile long-short: monthly returns up to ~39% **before** costs; **negative after** TC due to short holding / churn. Pre-cost trading return figure up to ~116% over three months does **not** survive frequent trading costs (authors).

**Coralys:** 15m is a meaningful horizon; ensemble ≠ free lunch; **hit rate without friction ≠ expectancy**. Do not replace Coralys.

---

## 2. Priority paper B — Jaquart, Köpke & Weinhardt (2022)

**Machine learning for cryptocurrency market prediction and trading**  
*Journal of Finance and Data Science* 8:331–352 · https://www.sciencedirect.com/science/article/pii/S2405918822000174

### Design

Daily binary relative moves; top **100** cryptos; LSTM/GRU/RF/GB/TCN/ensembles.

### Numbers

| Metric | Reported |
| --- | --- |
| Overall accuracy (all coins/models) | **52.9–54.1%** |
| Selective accuracy (top 10% confidence / class / day) | **57.5–59.5%** |
| LSTM/GRU ensemble ann. OOS Sharpe (w/ TC) | **3.23** |
| GRU ensemble Sharpe | **3.12** |
| Buy-and-hold market portfolio Sharpe | **1.33** |

**Coralys:** Motivation for **selective** reliability (accept high-evidence only). Qualifications: **daily**, **pooled 100 coins**, selective accuracy ≠ our single-book W/L.

---

## 3. Priority paper C — Arain & Snudden (2026)

**When Are Statistical Forecast Gains Economically Relevant? Evidence From Bitcoin Returns**  
*Journal of Forecasting* 45(3):1245–1260 (first published 2025-12-17) · https://onlinelibrary.wiley.com/doi/full/10.1002/for.70077

### Design

Real-time OOS BTC return forecasts; daily bivariate VARs; Oct 2021–Feb 2024; seven predictive indices; directional accuracy + economic profitability.

### Findings (qualitative + structural)

1. Seven indices → statistically significant directional-accuracy gains.  
2. MSFE largely **uninformative** about trading profitability.  
3. Mean directional accuracy alone **does not** explain profitability.  
4. Conditional directional accuracy (magnitude-aware) is more informative.  
5. Profits when accuracy stays stable in **large** moves.  
6. USD index and Shanghai SE index → significant profits under their excess-profitability test.

**Coralys:** Strong support for **separating** reliability (\(Q_t\)) from economic actionability (\(U_t\)) — average hit rate ≠ after-cost expectancy (ML-1D).

---

## 4. Aggressive hit-rate claims (do not benchmark Coralys)

| Study (as reported) | Claimed result | Qualification |
| --- | --- | --- |
| Hybrid BTC prediction | 100% directional | Monthly; 36m OOS — not 15m |
| On-chain BTC (2025) | 82.03% | Next-day — not 15m |
| Confidence-threshold multi-crypto (2025) | 82.68% executed-trade accuracy | 11 pairs; ~12% coverage — not BTC-only book |
| Jaquart et al. (2022) | 57.5–59.5% | Daily; top-10% confidence; pooled |

Audit leakage, selection, and coverage before any comparison. Prefer JFDS 2021 **53–54%** as the honest 15m literature reference.

---

## 5. Implications for ML-1D / RP-608

| Opportunity | Literature support | Product reading |
| --- | --- | --- |
| **A Reliability** | Calibration / conformal / drift (prior registers) | Estimate trustworthiness of Coralys outcomes |
| **B Selective hit-rate** | Jaquart 2022 52.9–54.1% → 57.5–59.5% | ACT only with sufficient evidence — still need BTC 15m single-book test |
| **C Hit rate vs expectancy** | Arain–Snudden 2026; Jaquart 2021 after-cost failure | Keep \(Q_t\) and \(U_t\) separate; W/L + payoff + net (Gate 2 v2) |

**Do not** replace Coralys with GRU/GBC/ensemble from these papers. Use them to understand measurement of predictive reliability and economic use of sealed recommendations.

---

## 6. Non-implications

No implementation · no remine · no Gate 5 · no Coralys edits · no conversion of 53.786% / 57.5% / Sharpe 3.23 into live cuts · no SELECT from this register alone.

---

*End — hit-rate literature register · 2026-11-27 Asia/Calcutta.*
