# RP-608-SCI-ADAPTER-REC-001  
## Literature findings: techniques complementary to Coralys

**Research date:** 2026-11-27 Asia/Calcutta  
**Author:** Anuj (findings) · worker store formalization  
**Question:** What have the shortlisted peer-reviewed papers demonstrated, and which findings can improve Coralys **recommendation reliability** and **economic actionability** without changing Coralys?  
**Caveat:** Methodologies ≠ proven BTC 15m profitability. Dual-dimension design is a **hypothesis**.  
**Package:** ML-1D · `docs/research/rp608_sci_adapter_rec_ml1d_dual_dimension.md`  
**Store mirror:** Agent store `docs/sci-adapter-rec-literature-findings-coralys-complement.md`

---

## 1. Recommendation reliability

### Paper 1 — Gibbs & Candès (2024)
**Conformal Inference for Online Prediction with Arbitrary Distribution Shifts** · *JMLR* 25(162):1–36  
https://jmlr.org/papers/v25/22-1218.html

- Extends ACI with online tuning of the step-size (DtACI); adapts to magnitude/type of shift without knowing the rate; local regret bounds; tested on stock volatility + COVID.  
- **Coralys use:** reliability-monitoring / uncertainty layer around recommendation outcomes without retraining Coralys.  
- **Limit:** Coverage ≠ conditional correctness or profitable trading.

### Paper 2 — Barber, Candès, Ramdas & Tibshirani (2023)
**Conformal Prediction Beyond Exchangeability** · *Annals of Statistics* 51(2):816–845  
https://doi.org/10.1214/23-AOS2276

- Weighted quantiles + randomization for non-exchangeable / nonsymmetric algorithms; coverage loss bounded by deviation from exchangeability.  
- **Coralys use:** foundation for historical reliability under drift/dependence; weights must be train-declared and chronologically validated — not unrestricted recency hacks.  
- **Limit:** Not a universal finite-sample guarantee for arbitrary dependent finance sequences.

### Paper 3 — Bao et al. (2025)
**CAP: Online Selective Conformal Prediction with FCR Control** · *JMLR* 26(287)  
https://jmlr.org/papers/v26/24-0452.html

- Calibration after adaptive pick; selection-conditional coverage / FCR control.  
- **Coralys use:** adapter selects ACT subset → need selection-aware reliability statements.  
- **Limit:** Inference validity for selected statements — not an ACT/PASS trading rule.

### Paper 4 — Niculescu-Mizil & Caruana (2005)
**Predicting Good Probabilities with Supervised Learning** · *ICML* 2005  
- Platt / isotonic correct characteristic miscalibration; calibration data needs differ by model class.  
- **Coralys use:** estimate \(P(\text{frozen-plan win}\mid\text{Coralys rec, context})\) empirically; do not treat Coralys internal strength as calibrated probability.  
- **Limit:** Calibration alone ≠ recommendation quality.

---

## 2. Economic actionability

### Paper 5 — Gârleanu & Pedersen (2013)
**Dynamic Trading with Predictable Returns and Transaction Costs** · *Journal of Finance* 68(6):2309–2340  

- Optimal policy: aim in front of the target; trade partially; signal persistence vs cost.  
- **Coralys use:** borrow **economic reasoning** (act only when expected benefit clears cost) — not the portfolio-control architecture.

### Paper 6 — Murthy & Wald (2023)
**Optimal Trading with Transaction Costs and Short-Term Predictability** · *Quantitative Finance* 23(7–8):1115–1127  

- Exact rules under proportional costs + short-horizon predictability.  
- **Coralys use:** formulate \(U_t\) from reliability × gross payoff − friction.

### Paper 7 — Dyhrberg, Foley & Svec (2018)
**How Investible is Bitcoin?** · *Economics Letters* 171:140–143  

- Intraday liquidity/spread patterns; 2017–18 sample.  
- **Coralys use:** session-dependent friction features; do **not** treat sample as current venue truth.

### Paper 8 — Aleti & Mizrach (2021)
**Bitcoin Spot and Futures Market Microstructure** · *Journal of Futures Markets* 41(2):194–225  

- Achievable execution ≠ mid; trade-through losses; venue differences.  
- **Coralys use:** spread/slippage/venue assumptions at decision time.

### Paper 9 — Brauneis et al. (2022)
**Bitcoin Unchained: Determinants of Cryptocurrency Exchange Liquidity** · *Journal of Empirical Finance* 69:106–122  

- Same-exchange past liquidity, market-wide crypto liquidity, volatility, chain fees predict liquidity; traditional-market liquidity weak.  
- **Coralys use:** decision-time liquidity covariates for \(U_t\).

---

## 3. Cross-paper synthesis

| Capability | Supporting research | Contribution |
| --- | --- | --- |
| Probability calibration | Niculescu-Mizil & Caruana | Interpretable \(Q_t\) |
| Adaptive uncertainty | Gibbs & Candès | Shift-responsive coverage |
| Selection-aware inference | Bao et al. | Honest post-ACT reliability |
| Non-exchangeable robustness | Barber et al. | Drift/dependence-aware conformal |
| Execution economics | Gârleanu–Pedersen; Murthy–Wald; BTC microstructure | When benefit clears cost |

**Unresolved (Anuj):** conformal suitability for dependent BTC 15m outcomes; W/L vs payoff target; \(U_t\) sample stability; independent info in vol/session/liquidity; incremental value of reliability **after** economic selection.

---

## 4. Priority order (pre-implementation)

1. Reliability estimation + calibration  
2. Conditional economic utility  
3. BTC-specific liquidity / friction features  
4. Adaptive conformal (extension)  
5. Selection-aware conformal (extension)  
6. Combined dual policy (only after components)

---

*End — Coralys-complement findings · 2026-11-27 Asia/Calcutta.*
