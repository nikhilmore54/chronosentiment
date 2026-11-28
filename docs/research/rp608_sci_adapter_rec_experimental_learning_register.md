# Experimental Learning Register (ELR)

**Status:** ACTIVE · Anuj 2026-11-28 Asia/Calcutta  
**Rules:** `.cursor/rules/chronosentiment-core.mdc` §3b/§3c · Coralys sealed · books never pooled · no Gate 5  
**Principle:** Every failure must teach something that changes the next experiment. Threshold rattling is not a process.

Operating loop: **Observe → Hypothesize → Experiment → Measure → Diagnose → Improve**  
Literature generates hypotheses; it does not pre-select architecture.

Successful experiment outcomes: **(A)** positive discovery · **(B)** negative discovery · **(C)** diagnostic discovery.

---

## Entry template

| Field | Purpose |
| --- | --- |
| Experiment ID | Unique identifier |
| Book | BTC or ETH (never pooled) |
| Hypothesis | What we expect to improve |
| Observed problem | Evidence motivating the attempt |
| Technique | What we are trying |
| Change boundary | Exactly what changes |
| Expected outcome | Measurable improvement |
| Failure criteria | What falsifies it |
| Actual results | W/L, payoff, net, coverage |
| Observed failure | What actually happened |
| Learning | What we discovered |
| Next hypothesis | What we should try next |
| Decision | Continue / revise / reject / hold |

---

## ELR-001 — ML-1D dual-dimension (BTC)

| Field | Content |
| --- | --- |
| Experiment ID | `ELR-001-ML1D-BTC` |
| Book | BTC |
| Hypothesis | Dual ACT/PASS + calibration/conformal improves sealed holdout expectancy |
| Observed problem | Need complementary adapter without editing Coralys |
| Technique | ML-1D dual-dimension classifier package |
| Change boundary | Offline sealed holdout only · no live arm |
| Expected outcome | ADVANCE_CANDIDATE on W/L + payoff + net |
| Failure criteria | Holdout fails W/L-first + expectancy |
| Actual results | **DO_NOT_ADVANCE** (H11 comparator beat E3) |
| Observed failure | Classifier did not produce durable after-cost edge |
| Learning | Directional re-ranking alone is insufficient; do not remine ML-1D |
| Next hypothesis | Locate where Coralys recommendations lose economic value (diagnose-first) |
| Decision | **reject** — permanent retirement |

## ELR-002 — ML-1D dual-dimension (ETH)

| Field | Content |
| --- | --- |
| Experiment ID | `ELR-002-ML1D-ETH` |
| Book | ETH |
| Hypothesis | Book-native ML-1D advances independently of BTC |
| Technique | ETH ML-1D + H11 wired |
| Actual results | **DO_NOT_ADVANCE** after H11 |
| Learning | Same class failure as BTC; cross-book rescue forbidden |
| Decision | **reject** — permanent retirement |

## ELR-003 — BTC SCI stacked twin soak (P3)

| Field | Content |
| --- | --- |
| Experiment ID | `ELR-003-BTC-SCI-STACK-SOAK` |
| Book | BTC |
| Hypothesis | Frozen SCI-v1 P1 cuts add observational selection value under Gate2E stack |
| Observed problem | Need live closed-book evidence for SCI |
| Technique | `CRYPTO_PLAN_SCI_ADAPTER_REC` · pack `btc_p1_side_models.json` |
| Change boundary | Observational twin · authority 0% · stacked filters inherited · exits frozen |
| Expected outcome | Closed W/L + payoff + net not hostile vs baselines (soak) |
| Failure criteria | Persistently hostile W/L / negative expectancy |
| Actual results | ~**1/37** · payoff ~0.066 · **net −112.31** · heavy upstream suppress |
| Observed failure | See event OBSERVE (`ELR-003-OBSERVE` below) |
| Learning | Stack obscures adapter ownership; friction + exit geometry dominate closed losses |
| Next hypothesis | Event-classified FIX candidates — **not** cut remine; **not** modify this armed soak |
| Decision | **hold** — `HOLD_SOAK` · investigate events · do not retune |

## ELR-003-OBSERVE — BTC SCI closed-event failure mix (2026-11-28 ~08:05 Asia/Calcutta)

Source: `trade_plan_overlay_sci_adapter_rec_v1` · n=38 closed · script `rp608_sci_adapter_rec_event_observe.py`

| Primary category | Count | Implication for next FIX |
| --- | --- | --- |
| EXECUTION_FRICTION | 24 | Prefer execution-economics experiment, not new directional classifier |
| DIRECTIONAL_FAILURE | 13 | Direction wrong on a minority; still real — separate from friction |
| WIN | 1 | — |

Co-tags (not exclusive): TIMING_EXIT_GEOMETRY 25 · OUTCOME_HORIZON_MISMATCH 10 · SELECTION_WEAK_ENTER 37 (soft — ENTER p̂ often &lt;0.40)  
Exits: GIVEBACK_PROTECT 12 · MAX_HORIZON 11 · TRAILING 8 · SL 7 · Side mix SHORT 28 / LONG 10  
Economics: Σgross −35.48 · Σfric 45.61 · Σnet −112.31 · 25 losses had peak MFE ≥ fric+$1 (path support hypothesis only)

**Learning that must change the next experiment:** the dominant demonstrated weakness on closed SCI plans is **after-cost / exit-geometry economics**, co-occurring with path MFE that often cleared friction+floor — not “need a better ENTER threshold.”

## ELR-004 — P1a SCI-clean twin (coverage ownership)

| Field | Content |
| --- | --- |
| Experiment ID | `ELR-004-BTC-SCI-CLEAN-P1A` |
| Book | BTC |
| Hypothesis | Bypassing DI_SKEW/regime reveals whether SCI has independent selection value |
| Observed problem | Ledger: DI_SKEW + PATH + OPEN_PLAN dominate S1; adapter signal obscured |
| Technique | `CRYPTO_PLAN_SCI_ADAPTER_REC_CLEAN` · `skip_gate2e_regime_align` · PATH kept |
| Change boundary | New sealed book · stacked SCI untouched · same pack/cuts · no Gate 5 |
| Expected outcome | Measurable coverage ownership + closed W/L/payoff/net vs stacked |
| Failure criteria | CLEAN still hostile after-cost with clear ownership |
| Actual results | **Inconclusive** (Anuj 09:02) — closed n=1 identical to stacked; overlap 2/2 |
| Observed failure | Treatment differentiation **not observed** |
| Learning | Findings A–C: no different population yet; selection restriction ≠ missed edge; friction ≠ directional |
| Next observation | **First valid DI / regime-alignment differential event** (not N more trades) |
| Decision | **hold** both DNA · no new sealed package until differential event + outcome classified · **do not retune** |

## ELR-005 — ETH SCI-v1 P1 (offline)

| Field | Content |
| --- | --- |
| Experiment ID | `ELR-005-ETH-SCI-V1-P1` |
| Book | ETH |
| Hypothesis | Book-native SCI-v1 is ADVANCE_CANDIDATE offline |
| Actual results | Frozen audit conditional GO · net still negative (disclose) |
| Decision | **hold** — twin arm only with Anuj authorization · independent book |

---

## ELR-004-OBSERVE — CLEAN vs stacked comparative (updated 2026-11-28 ~09:02 Asia/Calcutta)

RQ: Gate2E selection contamination removal → quality / coverage / after-cost on BTC?

| Field | Value |
| --- | --- |
| Result | **Inconclusive** |
| Treatment differentiation | **Not observed** |
| Actual results | Post-arm overlap 2/2 · closed n=1 identical net −4.27 · DI/regime idle in window |
| Measurement issues | PATH≠DI · Gate2E event-aligned only · Σnet vs (Σgross−Σfric) Δ **−32.42** · closed-only ≠ open |
| Findings A–C | No different population · restriction ≠ missed edge · friction ⊥ directional |
| Next observation | **First valid DI / regime-alignment differential event** |
| Decision | **hold** · milestone ≠ N trades · no new sealed package · no corrective |

Detail: `rp608_sci_adapter_rec_clean_vs_stacked_observe_2026_11_28.md` · refine `rp608_sci_adapter_rec_observe_refine_2026_11_28_0902.md`

---

## ELR-SCI-CLEAN-OBSERVE-002 — paired new closes (2026-11-28 ~10:34 Asia/Calcutta)

| Field | Value |
| --- | --- |
| Result | **Inconclusive** |
| Treatment differentiation | **Not observed** |
| Board | stacked 1/43 −126.71 · CLEAN 0/6 −14.40 |
| Paired Δnet −$5.62 | Sum of two new identical closes (−4.37 SL directional + −1.25 giveback friction) — **between-book Δ = 0** |
| New plan_ids | `…8551607`, `…0715969` — present and identical on both twins |
| DI/regime differential | **Still unobserved** (post-arm DI/PATH/regime = 0) |
| Next observation | First valid DI/regime-alignment differential event |
| Decision | **hold** · no force differential · no new sealed package |

Detail: `rp608_sci_adapter_rec_elr_sci_clean_observe_002_2026_11_28.md`

---

## ELR-SCI-CLEAN-OBSERVE-004 — broad adverse interval (2026-11-28 ~11:37 Asia/Calcutta)

| Field | Value |
| --- | --- |
| Result | **Inconclusive** |
| Treatment differentiation | **Not observed** |
| Paired Δnet −$8.83 / +5 closes | Five identical shared SCI plan_ids; between-book Δ=0 |
| Failure mix | 1× DIRECTIONAL SL · 4× EXECUTION_FRICTION giveback |
| B/AS overlap | No shared plan_id / decision idx; parallel SHORT givebacks nearby |
| DI/regime differential | **Still absent** |
| Next | First DI/regime differential · cross-control attribution only · no classifier |
| Decision | **hold** |

Detail: `rp608_sci_adapter_rec_elr_sci_clean_observe_004_2026_11_28.md`

---

## ELR-SCI-CLEAN-OBSERVE-005 — multi-book adverse (2026-11-28 ~12:06 Asia/Calcutta)

| Field | Value |
| --- | --- |
| Result | **Inconclusive** |
| Treatment differentiation | **Not observed** |
| BTC SCI +2 / −$5.42 | `…5588784` fric giveback · `…5808143` dir SL — identical on both twins |
| ETH B & AS +4 / −$8.15 | **4/4 plan_id + idx overlap** (shared pathway this window) |
| Process note | Aggregate Δ verified via event ledger (matched here; do not assume next time) |
| DI/regime differential | **Still absent** |
| Decision | **hold** |

Detail: `rp608_sci_adapter_rec_elr_sci_clean_observe_005_2026_11_28.md`

---

## ELR-SCI-CLEAN-OBSERVE-006 — quiet SCI pair (2026-11-28 ~12:33 Asia/Calcutta)

| Field | Value |
| --- | --- |
| Result | **Inconclusive** |
| Treatment differentiation | **Not observed** |
| BTC SCI +2 / −$1.63 | `…7431540` (−0.50) · `…7564229` (−1.13) — both GIVEBACK / EXECUTION_FRICTION · identical twins |
| DI/regime differential | **Still unmet** (PATH stamps on CLEAN ≠ DI) |
| Decision | **hold** |

Detail: `rp608_sci_adapter_rec_elr_sci_clean_observe_006_2026_11_28.md`

---

## ELR-SCI-CLEAN-OBSERVE-008 — no treatment exposure (2026-11-28 ~16:33 Asia/Calcutta)

| Field | Value |
| --- | --- |
| Result | **INCONCLUSIVE_NO_TREATMENT_EXPOSURE** |
| Treatment differentiation | **Not observed** |
| Overlap | **31/31** · clean_only 0 · DI 0/0 · PATH common |
| Matched net expectancy | ≈ **−1.8932258064516128** · Σnet **−58.69** · Δecon **0** |
| Failure mix | EXECUTION_FRICTION 24 · DIRECTIONAL_FAILURE 7 |
| Artefact | Retrospective 31-event ledger started (gross/fric/exit/MFE/hold_s/mae_dist/class; unrounded nets) |
| Next | First DI/regime differential · continue ledger |
| Decision | **hold** |

Detail: `rp608_sci_adapter_rec_elr_sci_clean_observe_008_2026_11_28.md` · JSON `rp608_sci_adapter_rec_elr008_ledger31.json`

### ELR-008 D1–D4 (2026-11-28 ~16:39 Asia/Calcutta)

| Field | Value |
| --- | --- |
| D1 | Residual **−16.8135** explained: Rust closes use `(Ne+Nx)×12bps` in net while Python stamps `(Ne+Nx)×6bps` → Other ≈ 1× stamped fee on 14 non-GIVEBACK |
| D2 | Class audit: 7 dir stable; 3 TRAIL misclass EXEC→WIN under single friction; friction still primary |
| D3 | Path booleans only; no first-adverse timestamp |
| D4 | GIVEBACK capture ~16% peak MFE; TRAIL ~65% gross / stamped net −1.20 |
| Decision | **hold** · measurement finding only · no mid-soak fix |

Detail: `rp608_sci_adapter_rec_elr_sci_clean_observe_008_d1d4_2026_11_28.md`

---

## Non-negotiables

1. Coralys remains sealed intelligence foundation.  
2. Every book retains independent experimental authority.  
3. One experiment → one identifiable hypothesis.  
4. No improvement without closed W/L, payoff, and after-cost evidence.  
5. Armed soaks are not modified; next TRY is a new sealed package.  
6. Independent validation samples are not re-checked until they “work.”

---

*End — ELR · 2026-11-28 Asia/Calcutta.*
