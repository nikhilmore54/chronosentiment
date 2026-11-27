# RP-608-SCI-ADAPTER-REC — Literature survey part 2  
## Why the current selection kernel is failing · alternatives from journals / proceedings

**Date:** 2026-11-27 Asia/Calcutta  
**Status:** SURVEY ONLY — **no further twin/cut/architecture design** until Anuj names a next package  
**Rules:** `.cursor/rules/chronosentiment-core.mdc` §3b/§3c · Coralys no-touch · books never pooled  
**Part 1:** `docs/research/rp608_sci_adapter_rec_literature_survey.md`  
**Live evidence (Pi ~15:06 IST):** SCI closed **0/1** net **−$2.81** · open 1 · DI_SKEW/PATH_INVALID dominate early; adapter_select **0/5** −$10.68; Gate2E B **0/21** −$40.69

---

## 0. Diagnosis (product, not mood)

| Observation | Interpretation vs peer-reviewed practice |
| --- | --- |
| SCI is an **S1=b side generator** (replaces Coralys side when ENTER) | López de Prado (**JPM** 2018) warns against learning **side and size/act** in one model; prefers primary side + **meta-label** act/pass |
| Early soak: mostly stack vetoes, then co-ride with Gate2E B; first close a loss | Selective classifier is **not owning** the RC curve — Gate2E DI_SKEW/path stack confounds coverage (Part 1 §1); first economic outcome hostile |
| adapter_select soft-score + Gate2E B also **0-win** soaks | Confirms heuristic **and** scientific-v1 paths are not yet expectancy-positive on live friction books |
| P1 holdout lift was modest (+3.7 pp WR) near base rate | Harvey–Liu–Zhu (**RFS** 2016) + later MHT work: small edges under multiple testing are often flukes; need higher evidentiary hurdles + after-cost metrics |

**Working conclusion:** “More of the same selective GLM generator” is the wrong next design. Peer-reviewed finance ML points to **meta-labeling on a primary side** (Coralys as white-box primary), **orthogonal features**, **after-cost objectives**, and **multiple-testing discipline** — not another side generator with tighter Wilson cuts.

---

## 1. Meta-labeling (primary side · secondary act/pass)

| Citation | Venue | Claim |
| --- | --- | --- |
| López de Prado (2018) | **Journal of Portfolio Management** 44(6):120–133 — *The 10 Reasons Most Machine Learning Funds Fail* | Pitfall: learn side+size together. **Solution: meta-labeling** — primary high-recall side; secondary predicts whether primary bet is true; secondary filters false positives / sizes; reduces black-box risk when primary is fundamental/white-box |
| Joubert (2022) | **Journal of Financial Data Science** | Framework: meta-layer improves Sharpe/drawdown; ties binary metrics to strategy metrics; meta features should add information **beyond** the primary signal |
| Meyer, Barziy & Joubert (2023) | **Journal of Financial Data Science** | Calibration matters for **fixed** sizing rules from meta-probabilities |

**ChronoSentiment mapping (hypothesis for a *new* sealed package — not live remine):**

| Role | Candidate |
| --- | --- |
| **Primary (side)** | Sealed Coralys / Estimand candidate (`policy_b_action`) — white box, high recall |
| **Secondary (act/pass)** | Adapter meta-model: P(primary bet wins \| features **orthogonal** to Coralys internals) |
| **Labels** | Triple-barrier / frozen live exit geometry outcomes (win/loss of the *proposed* side) |
| **Forbidden** | Secondary that only re-thresholds the same score the primary already used (no new information) |

This **inverts S1=b** (generator). It is therefore **Anuj-gated re-freeze**, not a silent P3 tweak.

---

## 2. Labeling geometry (why fixed-horizon win/loss misleads)

| Citation | Venue | Claim |
| --- | --- | --- |
| López de Prado (2018) | **JPM** / AFML Ch. 3 | Fixed-time horizon labels ignore path stops; **triple-barrier** (PT / SL / vertical) matches how capital is actually risked |

**Product reading:** P1 used REC-001-H TARGET/RISK/HORIZON — closer to barriers than naive \(h\)-bar return signs — **good**. Next package should still prefer **live crypto-plan barrier outcomes** (P1 caveat) so secondary learns the book it will trade.

---

## 3. Multiple testing / false discoveries (why “ADVANCE_CANDIDATE” is fragile)

| Citation | Venue | Claim |
| --- | --- | --- |
| Harvey, Liu & Zhu (2016) | **Review of Financial Studies** 29(1):5–68 | Factor zoo + data mining → usual \(t>2\) too low; recommend **\(t>3\)** class hurdles; most published findings likely false |
| Harvey & Liu (2020) | **Journal of Finance** | False/missed discoveries; calibrate Type I/II; double-bootstrap hurdles for FDR |
| Large-scale MHT studies (e.g. ~2M strategies literature) | Working / conference papers citing HLZ | Single-test rejection rates ≫ MHT-adjusted; majority of “discoveries” may be luck |

**Product reading:** A sealed holdout WR lift of ~3–4 pp near a ~30% base rate is **not** Gate 5 fuel. P4 must keep payoff + **live friction net** + coverage; treat offline ADVANCE as **hypothesis**, not proof. Do not stack more filters on hostile soaks (multiple testing on the same decision surface).

---

## 4. Cost-sensitive / switching-cost selection

| Citation | Venue | Claim |
| --- | --- | --- |
| Zhao et al. / related | **IEEE TKDE** cost-sensitive online classification line | Optimize weighted misclassification cost, not 0–1 accuracy |
| IEEE Access (2019) | Cost-sensitive stock direction + TI selection | Explicit misclassification cost / investment return coupling |
| Witkowski (2026) | **Computational Economics** | Supervised trading signals that ignore costs → excessive switching; conditioning on prior decision introduces **implicit switching-cost** regularization; fewer flips, better after-friction behavior |

**Product reading:** SCI/adapter_select/B soaks that churn or open into friction-hostile paths fail the economic objective even if offline WR inches up. Next package objective = **after-cost selective risk**, not raw classification accuracy. Path-dependence / min-hold already exist in Gate2X — do not invent a second soft-score; encode costs in the **meta** loss / threshold.

---

## 5. Regime-conditional act/pass (orthogonal feature family)

| Citation | Venue | Claim |
| --- | --- | --- |
| Chang, Choi & Park (2017) | **Journal of Econometrics** 196:127–143 | Endogenous regime switching; filtered regimes (not smoothed) for real-time use |
| Applied / JFEC regime-switching forecast papers | **Journal of Financial Econometrics** etc. | Regime models can improve OOS forecasts when regimes are economically meaningful |
| Meta-label practice (JFDS + controlled experiments) | JFDS 2022 | Meta lift requires features the primary **did not** already use (e.g. regime / vol) |

**Product reading:** A secondary that only sees the same 15m price+volume ranks the primary already embeds will **collapse to a stricter threshold** (no true meta). Candidate orthogonal families for a *new* package (adapter-owned, Coralys-sealed): realized vol regime, session clock, DI/stack **as covariates for meta only** (not as silent PRIMARY vetoes on the SCI-clean book), spread proxies — each predeclared, decision-time only.

---

## 6. What Part 1 still gets right (do not throw away)

From Part 1 journals/proceedings (still binding hygiene):

- Selective **coverage vs risk** language (JMLR)  
- Isotonic / Platt calibration (KDD’02 / MIT Press)  
- Wilson LB for sparse proportions (*Statistical Science*)  
- Purged chronological firewall (AFML + JCF PBO)  
- Skepticism of deep nets for this surface (ICAIF’21)

**Change of architecture ≠ abandonment of scientific method.**

---

## 7. Explicitly rejected “fixes” (literature + soak)

| Temptation | Why not |
| --- | --- |
| Tighten `p_min`/`p_lb` on live soak | Remine; HLZ multiplicity; confounds stack interaction |
| Strip DI_SKEW on the **same** SCI book mid-soak | Changes estimand without a sealed package |
| Another soft-score / trend-agree remine | Already failing observationally; Part 1 §1 defects |
| Replace Coralys with deeper generator | Meta-labeling / JPM 2018 argues the opposite |
| Promote on open MTM | §3b forbids |

---

## 8. Candidate next sealed packages (Anuj picks — no code yet)

| ID | Package | Literature driver | Touches S1? |
| --- | --- | --- | --- |
| **ML-1** | **Meta-label twin:** Coralys primary side · adapter secondary act/pass · orthogonal features · live barrier labels | JPM 2018 · JFDS 2022/23 | **Yes — re-freeze S1 toward (a) filter-on-primary** |
| **ML-2** | SCI-clean observational book (generator kept) **without** DI_SKEW stack — isolate RC curve | JMLR selective classification purity | No (ops isolation) |
| **ML-3** | Cost-sensitive threshold / switching-aware meta on ML-1 | TKDE / Computational Economics | Via ML-1 |
| **ML-4** | ETH join under **ML-1** (never pool) | S4 completeness | Via ML-1 |
| **HOLD** | Soak only; no new package | Current evidence thin (n=1 SCI close) | No |

**Recommendation from this survey (advisory):** Prefer **ML-1** as the scientific successor if Anuj accepts re-freezing S1. Keep v1 generator soaking as a **negative control**, not the promote path.

---

## 9. Decision requested

Reply with one of:

- **HOLD** — soak only  
- **ML-1** — authorize meta-label charter amendment (S1 re-freeze draft)  
- **ML-2** — SCI-clean twin only (no S1 change)  
- **DEEPER** — name a venue cluster for part 3 (e.g. only JFDS meta-labeling experiments, only RFS/JF multiple testing)

Default while waiting: **HOLD** + continue journals; **no implementation**.

---

*End part 2 — 2026-11-27 · current selection judged inadequate; meta-label literature is the leading peer-reviewed redirect.*
