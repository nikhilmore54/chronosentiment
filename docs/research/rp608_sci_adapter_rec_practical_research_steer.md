# RP-608-SCI-ADAPTER-REC — Practical research steer

**Date:** 2026-11-27 Asia/Calcutta  
**Audience:** product direction for the Coralys-wrapped adapter (not a Gate 5 packet)  
**Rules:** `.cursor/rules/chronosentiment-core.mdc` §3b/§3c · Coralys no-touch · S1–S10 freeze · books never pooled  
**Live context:** P3 BTC twin armed · first opens co-ride Gate2E B LONG · closed n still small · `HOLD_SOAK`  
**Peer-reviewed survey:** `docs/research/rp608_sci_adapter_rec_literature_survey.md`  
**Part 2 (current selection failing → meta-label redirect):** `docs/research/rp608_sci_adapter_rec_literature_survey_part2.md`  
**Research mission (revised dual-dimension):** `docs/research/rp608_sci_adapter_rec_research_mission.md`  
**ML-1D package (primary hypothesis, await SELECT):** `docs/research/rp608_sci_adapter_rec_ml1d_dual_dimension.md`  
**Anuj literature findings:** `docs/research/rp608_sci_adapter_rec_literature_findings_coralys_complement.md`  
**Part 3 strengtheners:** `docs/research/rp608_sci_adapter_rec_literature_survey_part3.md`

---

## 1. What the literature says (practical takeaways)

| Theme | Sources (indicative) | Product implication |
| --- | --- | --- |
| **Abstain > always trade** | Chow reject-option; *Trading via Selective Classification* (arXiv:2110.14914); reject-option surveys | STANDASIDE is the product feature. Coverage/accuracy trade-off must be **predeclared and scored**, not “open whenever Coralys flickers.” |
| **Calibrate, then cut** | Zadrozny & Elkan isotonic; sklearn calibration notes (~isotonic needs enough val n; Platt safer when small) | Keep **GLM → isotonic on validation → joint \(\hat p\) + Wilson LB** (S2/S3). Do **not** soak-ratchet cuts. Merge isotonic bins when val n is thin (already done in P1). |
| **Time leakage kills lift** | López de Prado purged/embargo / CPCV | Chronological train→purge→val→**burned holdout** stays mandatory. CPCV is for **inside-train** model selection only — not a substitute for the sealed holdout. |
| **Wilson LB for sparse evidence** | Wilson 1927; Brown/Cai/DasGupta; Agresti–Coull | Ranking / ENTER by **lower bound**, not raw \(\hat p\), is correct for small effective n. Freeze `p_min`/`p_lb` for the soak epoch. |
| **Price+volume is necessary, not sufficient** | Crypto prediction surveys; minute microstructure studies (weak alpha after fees) | S10 price+volume schema stays. Expect **modest** W/L lift (P1 ~+3.7 pp). Judge with **payoff + live friction net**, never WR-only. |
| **Deep nets / GBDT overfit HF crypto** | Microstructure + walk-forward BTC papers under costs | Stay on **ladder A (GLM)**; Stage B elastic-net only if val gates fail. Ban soak-time deep models. |
| **Conformal / selective calibration (later)** | Angelopoulos & Bates; calibrated selective classification; FinAbstain-style abstain stacks | **v1.1 research track only** — inductive conformal singleton-accept as an *optional* STANDASIDE layer on a **new sealed package**, not a silent retune of P1 cuts. |

---

## 2. What live soak already taught (product constraints)

1. **Generator works** — SCI ENTER stamps fire (`rust_ok`, \(\hat p\sim0.34\)); S1=b is not decorative.  
2. **Stack interaction dominates early coverage** — DI_SKEW / PATH_INVALID suppressed most candidates before first `PLAN_CREATED`. That is **not** evidence the calibrator is wrong; it is evidence the twin still sits under Gate2E PRIMARY-ish guards.  
3. **Open marks ≠ science** — SCI and Gate2E B co-opened the same BTC LONG; uPNL is noise until closed W/L (§3b).  
4. **Baselines stay negative controls** — Gate2E B / `adapter_select` remain S6 comparators; do not “fix” them into SCI or pool books.

---

## 3. Correct product shape (v1 locked vs next)

### Keep shipping as v1 (no Anuj re-freeze needed)

| Layer | Spec |
| --- | --- |
| **Role** | Adapter-only **side generator** around sealed Coralys (S1=b) |
| **Features** | Completed 15m **price+volume** schema `price_volume_15m_v1` |
| **Model** | Per-side L2 logistic + isotonic (ladder A) |
| **Policy** | ENTER iff \(\hat p_\star \ge 0.30\) and Wilson LB \(\ge 0.18\) |
| **Eval** | W/L-first per book + payoff + net + coverage (Gate 2 v2 / P4 draft) |
| **Ops** | BTC twin env on; ETH/NSE deferred; authority 0%; Coralys / OF ±0.30 / traj15 untouched |

### Steer next (Anuj-gated packages — do not soak-hack)

| Priority | Package | Why |
| --- | --- | --- |
| **P0 ops** | Finish chronological soak → score P4 draft (`HOLD_SOAK` / `ADVANCE_CANDIDATE_SOAK` / `DO_NOT_ADVANCE`) | Product truth is closed W/L under frozen exits |
| **P1a** | **SCI-clean twin** option: SCI generator + frozen exits, **without** DI_SKEW/regime stack (separate book id / dir) | Isolates calibrator expectancy from Gate2E stack; literature: selective classifier should own abstention, not inherit unrelated vetoes |
| **P1b** | Label join = **live plan outcomes** (not only REC-001-H geometry) for next offline package | Closes P1 caveat #3; walk-forward under true friction |
| **P2** | ETH 15m+label join → own sealed package (never pool with BTC) | S4 completeness |
| **P3 research** | Optional **conformal singleton-accept** STANDASIDE on val fold of a *new* package | Distribution-free abstain layer; report coverage by regime (exchangeability caveat) |
| **Out** | Soft-score retunes, Gate2E B lookback remine, Coralys edits, GBDT/deep HF models, cut ratchet on hostile days | Explicitly anti-scientific per charter §1 |

---

## 4. One-sentence product north star

Ship a **calibrated selective classifier** (GLM + isotonic + Wilson abstain) that generates side with disclosed coverage, evaluated W/L-first under frozen exits — Coralys remains a sealed oracle, not the scoring kernel.

---

## 5. Immediate do / don't

**Do:** keep soaking; journal DI_SKEW vs SCI-STANDASIDE separately; score closed books only; wire ETH package when join ready.  
**Don't:** strip DI_SKEW on the live SCI book without a named twin; promote on open MTM; remine P1 holdout; touch `coralys-*` or Gate 5 floors.

---

*End — practical steer for `RP-608-SCI-ADAPTER-REC-001` · 2026-11-27.*
