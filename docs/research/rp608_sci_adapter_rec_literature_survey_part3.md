# RP-608-SCI-ADAPTER-REC — Literature survey part 3  
## Strengthening pass before ML-1D SELECT

**Date:** 2026-11-27 Asia/Calcutta  
**Trigger:** Anuj — before selecting ML-1D, investigate (1) conformal for dependent financial time series, (2) conditional predictive inference, (3) distributionally robust decision-making under transaction costs.  
**Status:** SURVEY ONLY · no implementation · no remine  
**Rules:** `.cursor/rules/chronosentiment-core.mdc` §3b/§3c · Coralys no-touch  
**Depends on:** Anuj findings · ML-1D charter  
**Store mirror:** Agent store `docs/sci-adapter-rec-literature-survey-part3.md`

---

## 0. Verdict (read this first)

| Area | Peer-reviewed backbone | Product implication for ML-1D |
| --- | --- | --- |
| **Dependent-series conformal** | Barber et al. *AoS* 2023 · Gibbs–Candès *JMLR* 2024 | Prefer **weighted / adaptive** conformal for reliability **uncertainty**; do **not** claim ordinary split-CP guarantees on BTC bars. Marginal coverage can look OK under mild dependence; **conditional** coverage (esp. high-vol regimes) is where it breaks — exactly when ACT hurts most. |
| **Conditional / selection-aware inference** | Bao et al. *JMLR* 2025 · Jin et al. *JRSS-B* 2025 · Gazin et al. *JRSS-B* 2025 | After ACT selection, report reliability **conditional on selection** (or control FCR). R3 is mandatory hygiene for E3 reporting, not an optional polish. |
| **Cost-aware robust decisions** | Gârleanu–Pedersen *JF* 2013 · Murthy–Wald *QF* 2023 · DRO+TC line (Wasserstein cost-sensitive portfolio; *QF* DRLSTARR) | Economic layer should optimize **after-cost utility under friction**, optionally with **ambiguity** around payoff/friction laws — still as ACT/PASS thresholds, **not** portfolio rebalancing engines. |

**Gate G1 (literature strengthen):** **SATISFIED for hypothesis design.** Remaining before SELECT: **G0** data/feature availability; then explicit Anuj **SELECT**.

---

## 1. Conformal prediction for dependent financial time series

### 1.1 Peer-reviewed anchors (already in Anuj stack)

| Paper | Venue | Strengthener |
| --- | --- | --- |
| Barber, Candès, Ramdas, Tibshirani (2023) | *Annals of Statistics* | Coverage degradation quantified via weighted TV / non-exchangeability; weighted quantiles + randomization allow nonsymmetric / drifting pipelines |
| Gibbs & Candès (2024) | *JMLR* 25(162) | DtACI: online adaptation of ACI step-size; local regret under arbitrary shifts; equity-vol experiment |

### 1.2 What recent controlled / applied work adds (read carefully)

| Source | Status | Claim useful to us |
| --- | --- | --- |
| Split conformal under β-mixing / “switch coefficient” analyses (e.g. 2025 time-series predictive inference preprints) | Mostly **preprint** | Explains why split-CP often *looks* fine under short-range dependence; bounds coverage loss — use as theory hygiene, not as a published BTC license |
| Rolling / temporal CP for financial returns (TCP-style) | Mostly **preprint** | Empirical BTC coverage can hit nominal **marginally**; interval *scores* often lose to volatility models; crisis / lower-tail calibration can fail |
| Controlled DGP studies (iid / AR / GARCH / breaks) | Working / non-journal | **Marginal** coverage survives stationary dependence; **conditional** coverage fails in high-vol terciles; ACI repairs post-break coverage at width cost; score normalization by vol helps conditional honesty |

### 1.3 ML-1D reliability design rules (from this area)

1. Treat conformal as an **uncertainty / evidence-sufficiency** instrument around Coralys outcome scores — **not** as \(Q_t\) itself.  
2. Prefer **DtACI / weighted** schedules over vanilla split-CP for live soak monitoring.  
3. Predeclare: evaluate **conditional** coverage by vol / session / regime strata (Gate 2 v2: W/L-first still rules economic success).  
4. Connect conformal target to **frozen-plan W/L** (or residual) explicitly — conformal sets ≠ win probability.  
5. Reject “coverage ≈ edge” narratives (HLZ multiplicity still applies).

---

## 2. Conditional predictive inference (post-selection honesty)

### 2.1 Peer-reviewed anchors

| Paper | Venue | Strengthener |
| --- | --- | --- |
| Bao et al. (2025) CAP | *JMLR* | Online selective CP with FCR; calibration after adaptive pick |
| Jin, Ren & Candès (2025) — *Confidence on the focal* | *JRSS Series B* 87(4):1239–1259 | Finite-sample **selection-conditional** coverage for focal units after arbitrary (permutation-invariant) selection rules; generalizes Mondrian-style ideas |
| Gazin, Heller, Marandon & Solari (2025) — informative sets + FCR | *JRSS Series B* 87(4):909–929 | InfoSP / InfoSCOP: informative (e.g. small) prediction sets after selection with FCR control |

### 2.2 Adjacent (use as product, not as conformal theorem)

| Paper | Venue | Note |
| --- | --- | --- |
| *Trading via Selective Classification* | arXiv:2110.14914 (ICAIF-adjacent selective trading) | Abstention improves accuracy / Sharpe under slippage in studied futures — supports **ACT/PASS product shape**, not BTC proof |

### 2.3 ML-1D reliability design rules

1. **R3 is not optional for E3:** any claim about “accepted ACT reliability” must use selection-conditional or FCR-controlled inference.  
2. Selection rule for theory must be **predeclared and train-symmetric** where required by the chosen method (or use CAP/focal frameworks that state their invariance conditions).  
3. Separate journals: (a) **decision** ACT/PASS from \(U_t\)+evidence policy; (b) **reporting** intervals on the ACT subset.  
4. Do not shrink conformal sets mid-soak to “look selective.”

---

## 3. Distributionally robust decisions under transaction costs

### 3.1 Decision-theory / trading anchors (peer-reviewed)

| Paper | Venue | Strengthener |
| --- | --- | --- |
| Gârleanu & Pedersen (2013) | *Journal of Finance* | Predictable returns + TC → partial adjustment; persistence-weighted aim |
| Murthy & Wald (2023) | *Quantitative Finance* | Short-term predictability + proportional TC → cost-dependent optimal trade |

**Adapter reading:** encode as **PASS when expected net utility ≤ 0** (or ≤ predeclared hurdle), with friction estimated at decision time — **not** continuous portfolio targeting (no sizing authority).

### 3.2 DRO + costs (use cautiously)

| Paper | Venue / status | Strengthener |
| --- | --- | --- |
| Liu et al. / related *Quantitative Finance* DRO portfolio (e.g. DRLSTARR Wasserstein) | *Quantitative Finance* 2021+ | Worst-case performance under ambiguity + trading constraints — methodology for **robust \(U_t\)**, not copy-paste MIQP into adapter |
| Cost-sensitive Wasserstein log-optimal portfolios (Hsieh–Yu line; IEEE L-CSS follow-ons) | Emerging / some 2024–26 | Explicit convex TC + ambiguity radius → flight-to-safety as costs/ambiguity rise — maps to **more PASS** under thin evidence / wide friction ambiguity |

### 3.3 ML-1D economic design rules

1. Primary object: \(\hat U_t = \widehat{\mathbb{E}}[R^{\mathrm{net}}\mid X_t]\) with \(R^{\mathrm{net}}\) = frozen-plan payoff − fees − spread/slippage proxy.  
2. Optional robustification: replace point \(\hat U_t\) with **worst-case** \(\inf_{P\in\mathcal{U}}\mathbb{E}_P[R^{\mathrm{net}}]\) over a **train-declared** ambiguity set (Wasserstein / moment) — still yields ACT/PASS.  
3. BTC microstructure papers (Dyhrberg; Aleti–Mizrach; Brauneis) supply **feature candidates**, not 2017–18 cost levels.  
4. DRO must not reintroduce sizing or continuous rebalancing (authority 0%).  
5. Ambiguity radius selection is a **multiple-testing** hazard — lock on val, burn holdout once.

---

## 4. How this changes ML-1D (without authorizing code)

| Component | Part 3 update |
| --- | --- |
| R1 Calibration | Remains first — \(Q_t\) from supervised calib on frozen-plan W/L |
| R2 Adaptive conformal | Prefer Gibbs–Candès DtACI / Barber weights as **uncertainty monitors**; stratum-conditional coverage required |
| R3 Selection-aware | Elevate to **required** for E3 scientific reporting (JRSS-B / JMLR CAP) |
| Ec1 / Ec2 Utility | Keep Gârleanu–Pedersen / Murthy–Wald logic; add optional DRO wrapper as research extension **after** point \(U_t\) works |
| Combine | Still test E1/E2/E3; anti-double-count \(Q_t\) inside \(U_t\) |

---

## 5. Explicit non-claims

- No paper proves dual-dimension BTC 15m expectancy.  
- Conformal coverage ≠ W/L improvement.  
- DRO portfolio papers ≠ permission to change sizing/exits.  
- Preprint TCP / controlled-DGP studies inform risk of **conditional** coverage failure — they are not Gate 5 evidence.

---

## 6. Next ops (still Anuj-gated)

1. **G0:** inventory decision-time features (spread, depth, session, vol) + historical Coralys→frozen-plan join sufficiency.  
2. **SELECT ML-1D** explicitly if G0 green.  
3. Only then: sealed offline package design (E0–E3) — still no Pi arm until offline gates.

---

*End — literature survey part 3 · 2026-11-27 Asia/Calcutta.*
