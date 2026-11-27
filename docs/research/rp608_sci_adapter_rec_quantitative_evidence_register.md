# RP-608 — Quantitative evidence register  
## Nine shortlisted papers (SCI-ADAPTER-REC / ML-1D)

**Date:** 2026-11-27 Asia/Calcutta  
**Author:** Anuj (extraction) · worker formalization  
**Rules:** `.cursor/rules/chronosentiment-core.mdc` §3b/§3c · Coralys no-touch · no Gate 5  
**Status:** EVIDENCE REGISTER ONLY — **not** a threshold packet · **not** SELECT · **not** implementation  
**Hard rule:** A number in a paper is **not** an effect size for Coralys. Coverage targets ≠ trading accuracy. Historical BTC spreads ≠ current book friction assumptions. Thresholds come from the frozen ChronoSentiment eval contract only.

**Related:** ML-1D charter · findings complement · survey part 3  
**Store mirror:** Agent store `docs/sci-adapter-rec-quantitative-evidence-register.md`

---

## 0. How to read this register

| Kind of number | Use for Coralys |
| --- | --- |
| Coverage / FCR targets (e.g. 90%, 10%) | Method config / validity — **not** ACT cutoffs |
| Simulation n, windows, γ grids | Reproducibility / design hygiene |
| Theoretical bounds | Reliability of weighted CP under drift |
| Commodity Sharpe (gross vs net) | Motivation for separate economic layer |
| BTC spread / depth / RTC (2017–2019) | Feature *candidates* + order-of-magnitude history — **not** live cost tables |
| Missing full-text tables | Marked incomplete — do not invent |

---

## 1. Recommendation reliability

### 1.1 Gibbs & Candès (2024) — DtACI  
*JMLR* 25:1–36 · [PDF](https://www.jmlr.org/papers/volume25/22-1218/22-1218.pdf)

| Parameter | Reported |
| --- | --- |
| Target coverage | 90% |
| γ candidates (8) | \{0.001, 0.002, 0.004, 0.008, 0.016, 0.032, 0.064, 0.128\} |
| MC trials (sim) | 100 |
| Local coverage window | 500 days |
| Equity names | NVIDIA, AMD, BlackBerry, Fannie Mae |
| Equity estimation window | 1,250 days |
| Extra application | 4 US counties COVID case counts |

**Shift environments:** stationary; smooth AR drift (innovation var 0.006); jump (phases with means ±0.075 / ±1.5). Comparators: DtACI, AgACI, MVP.

**Findings (qualitative + principal numeric):** Stationary → MVP/AgACI slight edge; smooth → DtACI≈AgACI; abrupt → DtACI adapts better; ~**90% local coverage** on equity-vol sets; local errors near ideal Bernoulli. **No** universal accuracy/trading return % lift.

**Coralys:** Adaptive uncertainty yes; coverage ≠ win probability.

### 1.2 Barber et al. (2023) — Beyond exchangeability  
*Annals of Statistics* 51(2):816–845 · [PDF](https://projecteuclid.org/journals/annals-of-statistics/volume-51/issue-2/Conformal-prediction-beyond-exchangeability/10.1214/23-AOS2276.pdf)

| Parameter | Reported |
| --- | --- |
| Sim n | 2,000 |
| Dim | 4 |
| Target coverage | 90% |
| Environments | IID; 2 abrupt β changes @ 500 & 1,500; gradual linear drift |
| Weight decay (exp) | 0.99 |
| Real domains | Electricity; election forecasting |

\(\beta_1=(2,1,0,0)\), \(\beta_2=(0,-2,-1,0)\), \(\beta_3=(0,0,2,1)\).

**Theory (stated conditions):** gradual drift \(\mathrm{CoverageGap}\le 2\epsilon/(1-\rho)\); abrupt change \(k\) steps ago \(\mathrm{CoverageGap}\le \rho^k\).

**Findings:** Weighting improves coverage under shift; nonsymmetric weighted fit can shorten intervals; IID recovers ordinary CP. **No** BTC expectancy %.

**Coralys:** Math for non-exchangeable history; train-declared weights only.

### 1.3 Bao et al. (2025) — CAP  
*JMLR* 26(287):1–74 · [PDF](https://jmlr.org/papers/volume26/24-0452/24-0452.pdf)

| Parameter | Reported |
| --- | --- |
| Target FCR | 10% |
| Online horizon (shift sims) | 2,000 |
| Initial labelled | 50 |
| γ candidates (6) | 0.008 … 0.256 |
| Initial \(I\) | 200 |
| Shift scenarios | 4 (IID, slow, abrupt, TS dependence) |
| One reported comparison replications | 1,000 |

**Findings:** Ordinary DtACI can inflate FCR after selection; CAP controls FCR in IID; CAP alone weak under shift; **CAP-DtACI** more reliable FCR under studied shifts. Target FCR ≠ trading loss rate.

**DAVIS drug discovery:** n=25,772; train 15,000; initial labelled 50; online test 2,000; 3 hidden layers; target FCR 10%. At illustrated index ~1,300 FCP: CAP **10.01%**; ordinary online CP **2.32%** (over-conservative); LORD-CI **0.26%**.

**Coralys:** Selection-aware calibration after ACT — R3 hygiene.

### 1.4 Niculescu-Mizil & Caruana (2005) — Calibration  
*ICML* 2005:625–632 · [PDF](https://icml.cc/Conferences/2005/proceedings/papers/079_GoodProbabilities_NiculescuMizilCaruana.pdf)

| Parameter | Reported |
| --- | --- |
| Problems | 8 |
| Algorithms | 10 |
| Train / calib per problem | 4,000 / 1,000 |
| Reliability bins | 10 |
| Total test obs | 133,594 (ADULT 35,222 … HS 4,366; positive rates 3–53%) |

**Findings:** Boosted-tree sigmoid distortion in **7/8**; Platt for sigmoid; isotonic for general monotone (overfits sooner if calib scarce); after calib, boosted/RF/SVM strongest in their experiments. **No** universal ECE reduction % for other domains.

**Coralys:** Independent OOS calibration of frozen-plan outcome probs — not “GLM always wins.”

---

## 2. Economic actionability

### 2.1 Gârleanu & Pedersen (2013)  
*Journal of Finance* 68(6):2309–2340 · [PDF](https://pages.stern.nyu.edu/~lpederse/papers/DynamicTrading.pdf)

| Parameter | Reported |
| --- | --- |
| Futures | 15 commodities |
| Sample | 1996-01-01 → 2009-01-23 |
| Signals | 5d / 12m / 5y |
| Comparators | Dynamic · Markowitz · static TC opts (10 speeds) |

**Annualized Sharpe (Table II excerpt — low / high cost):**

| Strategy | Low gross | Low net | High gross | High net |
| --- | --- | --- | --- | --- |
| Markowitz | 0.83 | **−9.84** | 0.83 | **−10.11** |
| Dynamic | 0.62 | **0.58** | 0.58 | **0.53** |
| Best static (reported) | — | 0.46 | — | 0.39 |

Dynamic beats best static by ~**0.12 / 0.14** Sharpe (low/high). Gross-best ≠ net-best; persistence-aware dynamic beats speed-only static.

**Coralys:** Separate reliability from act cost. **Do not** import sizing / partial adjustment into adapter.

### 2.2 Murthy & Wald (2023)  
*Quantitative Finance* 23(7–8):1115–1127

MA(1) returns · proportional TC · exact multi-asset rules · simulations. Costs, vol, predictability shape optimal trade; gains when TC vary across assets.

**Audit status:** Detailed simulation tables **not** extracted from accessible full text — **no invented Sharpe/%**.

**Coralys:** Conditional friction matters; multi-asset allocation ≠ single-book ACT/PASS.

### 2.3 Dyhrberg, Foley & Svec (2018)  
*Economics Letters* 171:140–143

| Parameter | Reported |
| --- | --- |
| Exchanges | 3 |
| Sample | 2017-09-07 → 2018-05-10 |
| Trades/exchange/day | 13,729–35,849 |
| Avg trade value | $3,912–$6,230 |
| Quoted spread | 0.54–7.80 bps |
| Effective spread | **5.60–22.51 bps** |
| BBO depth | $29,903–$168,466 |

Effective ≫ quoted; US-hours peak activity/vol and tighter spreads. **Not** current venue costs.

**Coralys:** Use execution-cost estimates, not quote-only.

### 2.4 Aleti & Mizrach (2021)  
*Journal of Futures Markets* 41(2):194–225

| Measure | Reported |
| --- | --- |
| Avg daily BTC/USD (venues) | $146m |
| Spot / CME median trade | <$1,300 / >$18,000 |
| Avg bid–ask | **0.0298% (2.98 bps)** |
| Impact trades >$1m | <1% |
| Coinbase ≤50 ms trades / cancels | 2.5% / 15.5% |
| Spreads >0.8% duration | 226 s |
| Est. trade-through losses (aggregate hist.) | **$36m** |

CME leads discovery in sample; many executions trade through better quotes.

**Coralys:** Execution quality ≠ recommendation quality. $36m is aggregate history, not per-strategy cost.

### 2.5 Brauneis et al. (2022)  
*Journal of Empirical Finance* 69:106–122 · [PDF](https://madoc.bib.uni-mannheim.de/64079/1/Bitcoin-unchained--Determinants-of-cryptocurrency-_2022_Journal-of-Empirical.pdf)

| Parameter | Reported |
| --- | --- |
| Exchanges | Bitfinex, Bitstamp, Coinbase Pro |
| Sample | 2017-12-15 → 2019-12-15 |
| Transactions / OB snaps / 1m intervals | 90.99m / 22.42m / 702,788 |

**Means (bps except depth $000):**

| Measure | Bitfinex | Bitstamp | Coinbase Pro |
| --- | --- | --- | --- |
| Quoted / effective spread | 0.82 / 1.29 | 7.11 / 7.76 | 0.52 / 1.33 |
| Price impact | 0.61 | 0.87 | 0.49 |
| RTC $500 / $2k / $40k / $100k | 1.01 / 1.29 / 4.86 / 9.49 | 7.80 / 8.50 / 15.39 / 21.97 | 0.81 / 1.07 / 4.07 / 7.93 |
| Avg BBO depth ($000) | 43.46 | 13.92 | 67.57 |

Liquidity determinants (daily TS regs): same-exchange past liquidity, crypto-wide liquidity, crypto vol, chain fees; **not** traditional-market liquidity (reported). Full coeff tables **not** all transcribed.

**Coralys:** Exchange- and size-dependent friction features; use **our** venue/size, not 2017–19 averages as live cuts.

---

## 3. Consolidated headline numbers

| Paper | Headline quantitative finding | Bucket |
| --- | --- | --- |
| Gibbs–Candès | ~90% local coverage under shifting equity-vol | Reliability |
| Barber et al. | Gap ≤ \(2\epsilon/(1-\rho)\) (gradual; stated cond.) | Reliability |
| Bao et al. | CAP FCP 10.01% vs 10% target (DAVIS example) | Post-selection |
| Niculescu-Mizil | Sigmoid distortion 7/8 boosted problems | Calibration |
| Gârleanu–Pedersen | Net Sharpe 0.58 vs best static 0.46; Markowitz net −9.84 | Economic |
| Murthy–Wald | Exact MA(1)+TC rules (tables pending) | Economic |
| Dyhrberg | Effective spreads 5.60–22.51 bps | BTC exec |
| Aleti–Mizrach | Spread 2.98 bps; $36m trade-through losses (agg.) | BTC exec |
| Brauneis | $2k RTC 1.07–8.50 bps (3 venues, hist.) | BTC exec |

---

## 4. Five conclusions (Anuj)

1. Reliability measurable **independent** of Coralys internals.  
2. Selection changes validity of uncertainty (CAP).  
3. TC can invert gross-attractive strategies (Markowitz gross 0.83 → net −9.84).  
4. BTC costs are **exchange- and size-dependent** (Brauneis).  
5. Keep reliability and economic outputs **separate** — combination is hypothesis, not proven.

---

## 5. Audit completeness

| Paper | Status |
| --- | --- |
| Gibbs–Candès | Config + main coverage; not all figure values |
| Barber | Sims + bounds; not all empirical figure coverages |
| Bao | Config + selected FCP; not every sim table |
| Niculescu-Mizil | Sizes + comparative; not all calib error numbers |
| Gârleanu–Pedersen | Sharpe table extracted |
| Murthy–Wald | **Tables not available** in accessible full text |
| Dyhrberg | Core spread/depth extracted |
| Aleti–Mizrach | Principal microstructure measures extracted |
| Brauneis | Core liquidity/RTC; not all regression coeffs |

Register is **not** fully numerically audited until outstanding tables are extracted. BTC liquidity samples are **2017–2019** — do not use as current execution assumptions.

---

## 6. Next research metrics (not thresholds)

| Priority | Question | Evidence to collect on **our** book |
| --- | --- | --- |
| 1 | Can reliability be calibrated? | Brier, log loss, calib error |
| 2 | Drift deterioration? | Rolling coverage, calib drift, detection delay |
| 3 | Selection preserves uncertainty? | Selection-conditional coverage / FCR |
| 4 | Act worthwhile? | Gross, fees, spread, slip, net expectancy |
| 5 | Liquidity context adds? | Conditional net by liquidity |
| 6 | Combination incremental? | E3 vs E1/E2 after-cost (ML-1D) |

**Do not** convert 90% coverage → 90% reliability cut, or 1–8 bps history → live friction cuts.

---

## 7. Non-implications

No implementation · no remine · no Gate 5 · no SELECT from this register alone · no Coralys edits.

---

*End — quantitative evidence register · 2026-11-27 Asia/Calcutta.*
