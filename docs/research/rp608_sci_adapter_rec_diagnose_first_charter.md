# RP-608-SCI-ADAPTER-REC — Diagnose-first charter (Anuj 2026-11-27)

**Date:** 2026-11-27 Asia/Calcutta  
**Status:** **ACTIVE** — supersedes dual-dimension / ML-1D as primary research architecture  
**Rules:** `.cursor/rules/chronosentiment-core.mdc` §3b/§3c · Coralys sealed · books never pooled · no Gate 5  
**Objective:** Establish a repeatable, **book-specific** mechanism through which Coralys recommendations retain economic value after the frozen trading plan and actual friction — **not** a higher hit-rate classifier.

---

## 0. Locked decisions

| Item | Decision |
| --- | --- |
| ML-1D | **Permanently DO_NOT_ADVANCE** under current hypothesis — no remine / rescue |
| BTC SCI soak | Continue **unchanged** → P4 closed W/L + payoff + net |
| ETH SCI-v1 offline P1 | Retain as **independent** candidate |
| ETH SCI-v1 twin | **Conditional GO** after frozen P1 audit |
| P1a SCI-clean | First **conditional** research priority (coverage ownership) |
| P1b longer live-plan labels | Conditional on demonstrated label mismatch |
| P3 conformal STANDASIDE | Defer until uncertainty is shown to be the problem |
| New dual ACT/PASS model | **No** |
| Coralys / Gate 5 / pooling | **Prohibited** |

**Principle:** Do not require every successful book to use the same corrective architecture. Common evaluation discipline; book-specific adapters.

---

## 1. Why the prior direction was insufficient

1. Selection improvement attempted before locating the loss.  
2. Failure mechanisms mixed (direction vs plan vs friction vs upstream exclusion vs regime).  
3. Evaluation population ownership unclear (DI_SKEW / PATH before adapter).  
4. BTC ≠ ETH — evidence against a universal learned corrective.

---

## 2. Three-layer architecture

| Layer | Job | Authority |
| --- | --- | --- |
| **L1 Recommendation integrity** | Valid, attributed Coralys decision; timestamp; upstream exclusion accounting; decision-time features | **0%** |
| **L2 Outcome diagnosis** | Where economic value is lost (direction / plan / regime / friction / selection) | **0%** |
| **L3 Book-specific corrective** | One book · one hypothesis · one sealed package — only after repeatable mechanism | Observational until Anuj |

Diagnosis ≠ intervention. L1/L2 never change strategy.

---

## 3. Event ledger (S0–S7) — required accounting

| Stage | Measurement |
| --- | --- |
| S0 | Total primary Coralys recommendation events |
| S1 | Upstream-eligible recommendation events |
| S2 | Events evaluated by SCI |
| S3 | Events accepted by SCI |
| S4 | Closed trading outcomes |
| S5 | Winning vs losing outcomes |
| S6 | Gross payoff + realized friction |
| S7 | Net economic result |

**Critical:** S1 vs S2 — DI_SKEW / PATH removals **before** adapter decision are **not** adapter attribution. Open plans ≠ closed W/L. Exclusion-reason ledger required (not one aggregate reject count).

### Economics (per book)

\[
GrossExpectancy = P(W)\cdot AvgWin - P(L)\cdot |AvgLoss|
\]
\[
NetExpectancy = GrossExpectancy - AvgFriction
\]
\[
NetReturnPerEligibleEvent = \frac{\sum R_i^{net}}{N_{\mathrm{eligible}}}
\]

Decompose deterioration: WR · avg win · avg loss · friction · rejection · distribution shift.

---

## 4. Hypothesis jobs (not three classifiers)

| ID | Question | When |
| --- | --- | --- |
| **P1a** | Independent SCI value without inherited DI_SKEW selection? | Coverage ownership unresolved |
| **P1b** | Label horizon vs live frozen-plan outcomes mismatch? | Label mismatch demonstrated |
| **P3** | Insufficient-evidence STANDASIDE (risk control, not alpha)? | Uncertainty shown as the problem |

Decision table: see Anuj message §7 — technique chosen because evidence identifies a solvable problem.

---

## 5. Gated sequence (now)

1. **P0** — complete BTC SCI soak (no intervention).  
2. **ETH P1 audit** — pack identity, sample, labels, friction, no ML-1D deps → conditional twin.  
3. **Diagnostic comparison** — S0–S7 + gross→net; no remine.  
4. **Select one** next hypothesis (P1a preferred if DI/coverage confirmed).  
5. **One sealed package** after Anuj SELECT.

---

## 6. Robustness requirements

1. Predictive validity · 2. Economic validity (W/L + payoff + friction) · 3. Selection ownership · 4. Temporal validity · 5. Book-specific validity.

---

## 7. Related artifacts

- ETH P1 audit: `docs/research/rp608_sci_adapter_rec_eth_p1_frozen_audit.md`  
- P1a scoped: `docs/research/rp608_sci_adapter_rec_p1a_sci_clean_scoped.md`  
- Diagnostic ledger: `scripts/research/rp608_sci_adapter_rec_diag_ledger.py`  
- ML-1D holdouts: permanently DNA (BTC + ETH)

---

*End — diagnose-first charter · 2026-11-27 Asia/Calcutta.*
