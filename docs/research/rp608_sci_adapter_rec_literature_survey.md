# RP-608-SCI-ADAPTER-REC — Literature survey (journals / transactions / proceedings)

**Date:** 2026-11-27 Asia/Calcutta  
**Purpose:** Ground the Coralys-wrapped adapter in **peer-reviewed** venues **before any further design**.  
**Status:** **SURVEY ONLY** — no new twin, no cut change, no Coralys edit, no Gate 5.  
**Rules:** `.cursor/rules/chronosentiment-core.mdc` §3b/§3c · S1–S10 freeze · books never pooled  
**Companion:** `docs/research/rp608_sci_adapter_rec_practical_research_steer.md` (product implications)  
**Part 2 (selection failure + meta-label alternatives):** `docs/research/rp608_sci_adapter_rec_literature_survey_part2.md`

---

## 0. Scope and reading rule

| Include | Exclude (or mark clearly) |
| --- | --- |
| Journals, transactions, ACM/IEEE/NeurIPS/KDD/ICAIF **proceedings**, *Foundations and Trends*, *Statistical Science*, *Annals of Statistics* | Blog posts, GitHub READMEs as primary authority |
| Canonical **books** only when the method originated there (cite as book, not journal) | Treating SSRN/arXiv-only drafts as settled design law |

**Reading rule for ChronoSentiment:** v1 may continue soaking under frozen S1–S10. **No further architecture / twin / cut design** until Anuj reviews this survey and names the next package.

---

## 1. Selective classification / reject option (core ML)

| Citation | Venue | Claim relevant to us |
| --- | --- | --- |
| Chow (1957, 1970) | Classical pattern-recognition literature (reject option origin) | Optimal abstain when posterior max is below a cost-derived threshold |
| Bartlett & Wegkamp (2008) | **JMLR** 9:1823–1840 | Convex hinge-style surrogate for classification **with reject**; risk consistency |
| El-Yaniv & Wiener (2010) | **JMLR** 11:1605–1641 | Formal **risk–coverage (RC)** trade-off; selective classifier = \((f,g)\) |
| Yuan & Wegkamp (2010) | **JMLR** 11:111–130 | Convex risk minimization for reject option; consistency conditions |
| Franc, Prusa & Voracek (2023) | **JMLR** 24(11):1–49 | Cost-based / bounded-improvement / bounded-abstention models → same Bayes+selection strategy; **proper uncertainty scores** for black-box classifiers |

**Product reading:** STANDASIDE is not a hack — it is the scientific object. Score **selective risk** and **coverage** jointly (P4 draft already points this way). Do not judge SCI by open-rate alone.

---

## 2. Selective classification **applied to trading** (proceedings)

| Citation | Venue | Claim relevant to us |
| --- | --- | --- |
| Chalkidis & Savani (2021/2022) | **ACM ICAIF ’21** proceedings (doi:10.1145/3490354.3494379) | Selective (and ternary) classifiers → strategies that **flat when abstaining**; walk-forward; **logistic regression** selective variants among the more robust under slippage; LSTMs never profitable in their commodity-futures setup |

**Product reading:** Closest peer-reviewed analogue to SCI. Supports: (a) logistic selective baseline, (b) walk-forward / chronological eval, (c) skepticism toward deep sequence models for this product surface. Does **not** authorize remine of P1 holdout or Coralys edits.

---

## 3. Probability calibration (proceedings / book chapter)

| Citation | Venue | Claim relevant to us |
| --- | --- | --- |
| Platt (1999/2000) | Chapter in *Advances in Large-Margin Classifiers* (**MIT Press**) | Sigmoid map from scores → probabilities (Platt scaling); better on **small** calibration sets than flexible nonparametrics |
| Zadrozny & Elkan (2002) | **ACM KDD ’02** proceedings, pp. 694–699 | **Isotonic / PAV** calibration for ranking scores; reliability diagrams; multiclass via calibrated binaries |
| Leathart et al. (2017) | **ACML** / PMLR 77:145–160 | Calibration trees; reaffirms isotonic vs Platt trade-offs |

**Product reading:** S2 (GLM + isotonic on validation) is **proceedings-backed**. Keep bin-merging when val n is thin (P1 already did). Platt remains the ladder fallback if isotonic overfits — already allowed by adaptive A→B spirit, without soak-time improvisation.

---

## 4. Wilson / binomial CI (journals)

| Citation | Venue | Claim relevant to us |
| --- | --- | --- |
| Wilson (1927) | *Journal of the American Statistical Association* (classical) | Score interval for a binomial proportion |
| Brown, Cai & DasGupta (2001) | **Statistical Science** 16(2):101–133 | Wald intervals erratic; recommend **Wilson** (or Jeffreys) for small \(n\); Agresti–Coull for larger \(n\) |
| Brown, Cai & DasGupta (2002) | **Annals of Statistics** 30:160–201 | Asymptotic expansions / further CI theory |

**Product reading:** Joint \(\hat p\) + **Wilson LB** cuts (S3) are journal-standard, not a soft-score invention. Freeze cuts for the soak epoch; do not Wald-substitute under small enter counts.

---

## 5. Time-series CV / backtest overfitting (book + journals)

| Citation | Venue | Claim relevant to us |
| --- | --- | --- |
| López de Prado (2018) | **Book:** *Advances in Financial Machine Learning* (Wiley), Ch. 7 & 12 | **Purged/embargoed** K-fold; **CPCV** for combinatorial OOS paths — **not** a journal article |
| Bailey, Borwein, López de Prado & Zhu (2017) | **Journal of Computational Finance** 20(4) | *Probability of Backtest Overfitting* (PBO) — journal peer of the overfitting program |
| Arian, Norouzi Mobarekeh & Seco (2024) | **Knowledge-Based Systems** 305:112477 | Synthetic controlled comparison; **CPCV** superior to walk-forward on PBO/DSR in their setup |

**Product reading:** Chronological firewall + purge stays mandatory. CPCV is appropriate **inside train/val model selection** on a **new** sealed package — not a reason to remine the burned 2024-11…12 holdout. Prefer journal/PBO language in Gate 5 packets over “backtest looked good.”

---

## 6. Crypto price–volume predictability (finance journals / letters)

| Citation | Venue | Claim relevant to us |
| --- | --- | --- |
| Balcilar, Bouri, Gupta & Roubaud (2017) | **Economic Modelling** 64:74–81 | Volume → Bitcoin returns **in quantiles**, weak/none in extreme bull/bear; volume **not** helpful for volatility |
| Bouri, Lau, Lucey & Roubaud (2019) | **Finance Research Letters** 29:340–346 | Volume Granger-causes **extreme** ± returns across major coins; volatility link limited |
| Fousekis & Grigoriadis (2021) | **Studies in Economics and Finance** | Cross-quantilogram: extremes lead volume; high volume precedes extreme positive returns |
| Sahoo & Sethi (2022+) | **International Journal of Finance & Economics** | Price–volume tests often **fail to reject efficiency** for many top coins (post-break) |
| Xue & Zhou (2026) | **Journal of Banking & Finance** 190 | Look-ahead-clean vision transformers on OHLCV-like images; cross-sectional crypto returns — **different task** (portfolio sort), not our single-book 15m W/L twin |

**Product reading:** S10 price+volume is **justified but contested**. Expect **modest, regime-dependent** lift (matches P1 ~+3.7 pp WR). Never pool books; never promote on WR alone; friction net mandatory. Do **not** leap to ViT/deep generative designs for the RP-608 twin without a new sealed package and Anuj gate — ICAIF’21 already favored simpler selective LR under costs.

---

## 7. Conformal / distribution-free abstention (journal / FnT / NeurIPS)

| Citation | Venue | Claim relevant to us |
| --- | --- | --- |
| Angelopoulos & Bates (2023) | **Foundations and Trends® in Machine Learning** 16(4):494–591 | Canonical gentle introduction; sets with finite-sample coverage; discusses abstention |
| Einbinder et al. (2024) | **JMLR** 25 | Label-noise robustness of conformal prediction |
| Ding, Angelopoulos, Bates, Jordan & Tibshirani (2023) | **NeurIPS** proceedings | Class-conditional conformal with many classes |
| Angelopoulos, Candès & Tibshirani (2023) | **NeurIPS** proceedings | Conformal PID for **time-series** prediction (exchangeability caveats matter for us) |

**Product reading:** Strong **v1.1 research track** for a *new* sealed package (singleton-accept → STANDASIDE). **Not** a silent overlay on live P1 cuts. Financial time series break vanilla exchangeability — any ChronoSentiment conformal design must cite time-series conformal work and report coverage **by regime**, not claim IID guarantees.

---

## 8. Synthesis → design freeze

### Already aligned with peer-reviewed core (keep v1)

1. Selective ENTER/STANDASIDE with disclosed coverage (JMLR + ICAIF’21)  
2. GLM + isotonic calibration (KDD’02; Platt as small-n alternative)  
3. Wilson LB joint cut (Statistical Science / Annals)  
4. Chronological purge + burned holdout (AFML + JCF PBO program)  
5. Price+volume features with modest expectations (EM / FRL / IJFE tension)

### Explicitly **not** next without a new Anuj-gated package

| Temptation | Why the literature says wait |
| --- | --- |
| Strip DI_SKEW on live SCI book mid-soak | Confounds calibrator RC curve with unrelated Gate2E stack (survey §1: selective classifier should own abstention) |
| Deep nets / ViT for 15m single-book twin | ICAIF’21 + crypto fee/overfit literature; JBF’26 is a different estimand |
| Soak-time cut ratchet | Remines burned science; violates holdout discipline (§5) |
| Conformal “quick add” on P1 pack | Needs calibration fold + regime reporting; new package id |

### Allowed ops while design is frozen

- Continue P3 soak; W/L-first scorecards; P4 draft scoring vocabulary  
- Journal stack-suppress vs SCI-STANDASIDE separately  
- Leave Gate2E B / `adapter_select` as S6 baselines  

---

## 9. Recommended reading order (short)

1. El-Yaniv & Wiener (2010) JMLR — RC trade-off  
2. Chalkidis & Savani (ICAIF’21) — trading selective classification  
3. Zadrozny & Elkan (KDD’02) — isotonic  
4. Brown, Cai & DasGupta (2001) Statistical Science — Wilson  
5. Bailey et al. (2017) JCF — PBO  
6. Angelopoulos & Bates (2023) FnTML — conformal (for *later* package only)

---

## 10. Decision requested from Anuj

**Hold further design** until one of:

- **A.** Continue soak only (no new twin/package)  
- **B.** Authorize survey-driven **next sealed package** (name: e.g. SCI-clean twin / live-label offline / conformal-v1.1 / ETH join)  
- **C.** Request deeper dive on one venue cluster (e.g. only conformal time-series, or only crypto volume journals)

No code/architecture work proceeds past this survey under option default **A**.

---

*End of literature survey — `RP-608-SCI-ADAPTER-REC-001` · 2026-11-27.*
