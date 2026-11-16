# RP-608 — Hypothesis Validation Programme

**Status:** **CONFIRMATION-ONLY HOLDING** · 2026-11-07  
**Authority:** 0.0% · Gate 2 **CLOSED** · Gate 3 unauthorized  
**Baseline / OF ±0.30 / stops / targets / horizon:** FROZEN  
**Holding charter:** `docs/research/rp608_confirmation_only_holding_state.md`  
**Related:** blocker plan `rp608_blocker_resolution_plan.md` · adaptive velocity `rp608_wr_adaptive_execution_velocity_001_scoped.md`  
**Latest result:** P4 H2 `SUPPORTED_SESSION_REGIME_DOMINANT` (descriptive) · Gate 2 memo `rp608_h2_crypto_plan_session_regime_p4_gate2.md`  
**Confirmation:** DQ monitor `NEED_MORE_SESSIONS` (2 powered sessions / 63 trades) — Nov 6–7 observations only

## Objective

Identify whether any **pre-registered** hypothesis provides **repeatable** predictive information about W/L, with economic context preserved.

Not: maximize WR via threshold scans, feature mining, or favorable-session selection.  
85% WR remains an **aspiration**, not a manipulable validation bar.

### Mandatory joint metrics (every test)

- W/L counts and WR  
- Payoff (avg\|win\| / avg\|loss\|)  
- Net after existing friction assumptions  
- Coverage / sample size  
- Session-to-session stability  
- Transfer to **untouched** chronological sessions  

WR-only “success” is forbidden (`.cursor/rules` §3b).

## Hypothesis register

| ID | Hypothesis | Status |
| --- | --- | --- |
| **H1** | Continuous entry quality — decision-time continuous magnitudes distinguish better entries within the existing signal population | **NOT_READY** (crypto native continuous absent) |
| **H2** | Session regime — observed WR is primarily session-level, not a stable entry-quality relationship | **P4 DONE** — `SUPPORTED_SESSION_REGIME_DOMINANT` (descriptive; Gate 2 not advanced) |
| **H3** | Order-flow alignment — decision-time OF alignment predicts W/L stably across sessions | Candidate — optional after H2; verify OF field presence (discovery secondary was all Missing) |
| **H4** | Adaptive execution velocity — urgency after signal approval affects realized execution quality | **`UNTESTABLE_UNDER_CURRENT_ENGINE_DESIGN`** — `rp608_h4_current_engine_boundary.md` |

### H1 — Continuous entry quality

**Question:** Do decision-time continuous momentum / trend-slope / price-location (or proven decision-time proxies) distinguish W/L, payoff, and net within the eligible signal population?  

**Motivation:** NSE Oct28/Oct30 same categorical pocket, different continuous magnitudes.  

**Exclusions:** No post-entry path; no threshold optimization for max WR; no book pooling.  

**Pass intuition:** Directionally consistent association across sessions + chronological validation + adequate coverage + non-deteriorating economics.

### H2 — Session regime

**Question:** Is WR variation explained mainly by session-level conditions rather than within-session entry-state relationships?  

**Exclusions:** Do not promote the strongest session as a policy.  

**Pass intuition:** Within-session entry-state signal survives after session separation; else session-regime dominates (descriptive, not an entry policy).

### H3 — Order-flow alignment

**Question:** Does decision-time OF-supported vs unsupported (per book definition) associate with W/L / payoff / net with session stability?  

**Exclusions:** Do not infer OF proves entry/exit defect; do not change OF ±0.30.  

**Pass intuition:** Same as H1, book-local, OF definition frozen before looking at validation.

### H4 — Adaptive execution velocity

**Requires:** submit → ack → first fill → complete + decision-time bid/ask/mid + actual fills.  

**Status:** Parked (NSE charter + crypto capture spec). Market-state analysis **must not** substitute for velocity evidence.

## Book tracks (never pool)

| Track | May investigate | Must not claim |
| --- | --- | --- |
| Crypto plan | Decision-time state where fields/joins are complete | Latency, observed slippage, velocity |
| Crypto OF paper | Touch-based entry & descriptive OF **if present** | Fill latency / execution improvement |
| NSE A | Continuous entry & session hypotheses (existing join) | Automatic transfer of discovery results |
| NSE OF | OF alignment & session relationships | That OF alignment proves entry/exit defect |

## Experiment sequence (Stages A–F)

| Stage | Work | Rule |
| --- | --- | --- |
| **A** | Freeze hypothesis definitions (question, outcomes, fields, exclusions, populations) | Before new results |
| **B** | Availability matrix per book | Missing stays missing; no leakage |
| **C** | Untouched evaluation window | Discovery / validation / confirmation; **002 scanned holdout is contaminated** for confirmation |
| **D** | Test one hypothesis at a time (H1→H2→H3 where supported) | No broad feature search / thr scan / best-session pick |
| **E** | Full result card | All eligible trades + economics + sessions + uncertainty |
| **F** | Governance review | Evidence package only — **no** automatic filter/exec/live change |

## Interpretation rules (locked)

| Result | Interpretation |
| --- | --- |
| Discovery disappears later | Not validated |
| WR↑ only via tiny subset | Narrow descriptive, not general improvement |
| WR↑ but payoff/net↓ | Not economic improvement |
| Direction flips across sessions | Unstable |
| Persists in chronological validation + coverage + economics | Candidate for Gate 2 **review** (not advance by default) |
| Lifecycle / actual fills absent | H4 remains untestable |

## Immediate work order

| Priority | Deliverable | Status |
| --- | --- | --- |
| P0 | Crypto prospective capture specification | **DONE** `rp608_crypto_prospective_capture_spec.md` |
| P1 | Crypto paper-engine capability audit | **DONE** `rp608_crypto_paper_engine_capability_audit.md` — H4 unavailable under current engines |
| P2 | Crypto hypothesis availability matrix | **DONE** `rp608_crypto_hypothesis_availability_matrix.md` |
| P3 | Freeze H1–H3 test protocols from P2 | **DONE** `rp608_hypothesis_p3_freeze.md` — H2-first |
| P4 | Run supported observational tests | **DONE** — H2 crypto plan · disc WR range 25.55 pp · val 39.24 pp · conf `NEED_MORE_SESSIONS` · `rp608_h2_crypto_plan_session_regime_p4.*` |
| P5 | Governance evidence package | **Memo filed** `rp608_h2_crypto_plan_session_regime_p4_gate2.md` — **no** Gate 2 advance |

### P4 H2 headline (2026-11-06 Pi)

| Window | n | WR% | Payoff | Net USDT |
| --- | ---: | ---: | ---: | ---: |
| Discovery ≤10-31 | 554 | 41.7 | 1.85 | +315 |
| Validation 11-01…05 | 441 | 33.1 | 0.53 | −747 |
| Confirmation ≥11-06 | 42 | 59.5 | 1.06 | +33 |

**Active while confirmation accumulates (2026-11-07):**  
1. Confirmation assessment protocol — `rp608_h2_prospective_confirmation_protocol.md`  
2. Accumulation charter — `rp608_h2_prospective_confirmation_accumulation.md`  
3. **Confirmation DQ monitor** — `rp608_h2_confirmation_dq_monitor.*` (states ELIGIBLE/INCOMPLETE/INTEGRITY_FAILURE; never auto-confirms H2)  
4. Existing economic-result audit — WR vs profitability on frozen evidence  
5. Reproducibility safeguards + sealed report registry  
6. Future hypothesis reopening criteria (H1/H3 gates; H4 closed under current engines)  

Leave H1/H3 unopened. No discovery rerun, thr scan, or latency inference from H2.

## Governance

Observational research is allowed. System changes (instrumentation implementation, filters, urgency, stops/targets, live authority) require **separate** authorization. Gate 2 stays closed until an explicit review.
