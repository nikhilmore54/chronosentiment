# RP-608 — W/L Improvement Plan

**Protocol:** `RP-608-WL-IMPROVE-001`  
**Status:** APPROVED RESEARCH PROGRAM · 2026-11-06 · **Gate 2 amended 2026-11-15**  
**Live order authority:** **0.0%**  
**Execution baseline:** FROZEN until Gate 5 (OF ±0.30, Estimand A thresholds/stops/holds, crypto plan exits)  

## Disposition

Prior work (friction, exits, actionable MFE, OF mechanics) remains available as diagnostics and as inputs to **Gate 2X**.  
Research focus is **durable expectancy per book** via two parallel paths: entry selection (**Gate 2E**) and exit/cost geometry (**Gate 2X**). Live execution stays frozen until Gate 5.

Primary session metric remains W/L (`.cursor/rules/chronosentiment-core.mdc` §3b). Payoff and net remain **required disclosures**; under Gate 2 v2 they are not a single veto that forces entry-only research to fix hostile exit geometry.

**Gate 2 v2 (binding):** `docs/research/rp608_gate2_amendment_v2.md` — split **2E** (entry) / **2X** (exit). Monolithic v1 “entry must clear W/L+payoff+net with no thinning” is **superseded** for advance criteria.

## Locked sequence (2026-11-06 Gate 1 · updated 2026-11-15)

1. **Gate 1 review** — diagnostic definitions, sample integrity, descriptive-only interpretation  
2. **Phase 2 Crypto plan** — decision-time entry-quality associations only  
3. **Cross-session replication** — beyond 2026-11-03…05  
4. **Hypothesis specification** — falsifiable hypotheses (entry and/or exit)  
5. **Gate 2 review (v2)** — **2E** and/or **2X** may ADVANCE independently → Gate 3E / 3X offline  

Gate 3E/3X remain unauthorized until the matching Gate 2 advances. NSE A may pursue 2E/2X on its own book (no longer strictly deferred behind crypto plan replication).

## Gate 1 outcome

**PASS** — `docs/research/rp608_wl_improve_gate1_diagnostic_completeness.md`

| Book | Interpretation |
| --- | --- |
| NSE A | Weak hours / mid bars → later investigation candidates |
| NSE OF | Weakness across imbalance → OF alone does not explain |
| Crypto plan | Highest-priority Phase 2 |
| Crypto OF | Small-n / noisy |

Leads only — **not** filters. No hour bans, H0 changes, or OF threshold changes.

## Phase 2 Crypto plan — decision-time rule

Narrow question: *Using information available at the entry decision, which observable conditions distinguish subsequent wins from losses?*

| Allowed predictors | Forbidden as predictors |
| --- | --- |
| Entry timestamp, H0/horizon, OF state, direction, ATR, RR/geometry, DQ, latch | Post-entry MFE/MAE, exit, later marks, holding path |

Unavailable without look-ahead-safe reconstruction: trend, momentum, price location, multi-bar signal persistence — labeled, not imputed.

## Construct distinction (LOCKED 2026-11-07)

Baseline OF stamps / ±0.30 paper = **L1 resting liquidity imbalance**, not volume/aggression-led Order Flow. Do not treat `OF_SUP` as executed buying pressure or absorption evidence. Aggression + response → **RP-608-OFX** only. Audit: `docs/research/rp608_orderflow_side_quantity_semantics_audit.md` §1b. Confirmation-only holding / Gate 2 / authority unchanged by this lock.

## Forbidden

- Changing OF ±0.30 / live stops/targets/horizons **without Gate 5**  
- Equating baseline OF with executed aggressor flow / absorption  
- Universal trend filter / symbol bans from single-session loss lists  
- Stop/target changes solely to lift historical WR without 2X evidence  
- Coverage below the **predeclared Gate 2E floor** as “success”  
- Gate 3E/3X without matching Gate 2E/2X ADVANCE  
- Remining consumed/contaminated windows as a new Gate 2E pass  

## Progress

| Item | Status |
| --- | --- |
| Phase 1 multi-session diagnostic | **DONE** |
| Gate 1 review | **PASS** |
| Phase 2 Crypto plan (decision-time only) | **DONE (3–5 Nov window)** |
| Cross-session replication beyond 3–5 Nov | **DONE** · REPLICATES=0 on holdout ex-1029 |
| Hypothesis specification | **NONE under v1** — reopen under Gate 2E/2X packages |
| Gate 2 (v1 disposition) | **DO NOT ADVANCE** · historical record (immutable) |
| **Gate 2 amendment v2** | **ACTIVE 2026-11-15** · split 2E/2X · `rp608_gate2_amendment_v2.md` |
| **Gate 2X-001** | **DONE** · crypto G/H60/H120 **ADVANCE_CANDIDATE** · NSE F scratch **DO_NOT_ADVANCE** · `rp608_gate2x_001_review.md` |
| **Gate 2X-002** | **DONE** · NSE G_ohlc **DO_NOT_ADVANCE** · `rp608_gate2x_002_nse_a_ohlc_ambiguous_scoped.md` |
| **Gate 2X-003** | **DONE** · CLOSE_GRID C/F_path **ADVANCE_CANDIDATE** (relative; fidelity caveat) · `rp608_gate2x_003_nse_a_close_grid_fpath_review.md` |
| **Gate 3 offline proceed** | **DONE** · 3X G **PASS** · 3E BAR_ge_12 **CONFOUND_DOMINANT** · `rp608_gate3_offline_proceed_review.md` |
| **Gate 5 Crypto G** | **DEPLOYED 2026-11-15** · Pi crypto shadow · `ambiguous_tie_break=TP` · authority 0% · `rp608_gate5_packet_crypto_g_draft.md` |
| **Gate 3X H* path-stress** | **PASS_BOTH 2026-11-16** · H60/H120_PATH holdout Δ+255/+381 · `rp608_gate3x_crypto_h_path_review.md` |
| **Gate 5 Crypto H\*** | **DEPLOYED 2026-11-16** · H120_PATH live · `EARLY_SL_PATH_SCRATCH` · authority 0% · `rp608_gate5_packet_crypto_h_draft.md` |
| **Gate 5 Crypto Adaptive TP** | **DEPLOY 2026-11-21** · post-1.5×ATR extend +0.5×ATR / SL lock · Gate 2X v1.4 continue-vs-exit · OF ±0.30 still frozen · `rp608_gate5_packet_crypto_adaptive_tp_draft.md` |
| **Gate 5 NSE Adaptive TP** | **DEPLOY 2026-11-21** · Estimand A LONG parity · +0.5×ATR (or +0.5% entry) extend / SL lock · Gate 2X v1.4 · `RP-608-GATE5-NSE-ADAPTIVE-TP-001` |
| Discovery v2 (algorithm upgrade) | **DONE** · Rust `rp608_wl_crypto_plan_discovery_v2` · Gate 2 criteria unchanged · still closed |
| NSE A Kite feature join (ext. holdout) | **DONE** · Pi · 1625 feature-ready rows |
| NSE A Kite discovery v2 | **DONE** · WR REPLICATES on mom/location · payoff/net bar fail · Gate 2 **DO NOT ADVANCE** |
| WR Improvement v2 / ENTRY-DISCOVERY-001 | **DONE** · Stages 1–5 · tier PARTIAL · best holdout WR 70% @ 8.6% cov · Gate 2 **DO NOT ADVANCE** · gap to 85% = −15 pp |
| WR ENTRY-DISCOVERY-002 (frontier audit) | **DONE** · holdout contaminated for thr pick · no powered 85% · Gate 2 **DO NOT ADVANCE** · ranking AUC documented |
| WR RANKING-001 (ranking + Oct 28 study) | **DONE** · frozen eval protocol · Oct 28 full 58.6% vs 002-slice 83.8% (31/6) · AUC transfer fail · §F genuine improvement **false** · Gate 2 **DO NOT ADVANCE** · no thr re-scan |
| WR ADAPTIVE-EXECUTION-VELOCITY-001 | NSE/crypto inventories **CLOSED** · H4 parked · NSE charter + crypto capture spec **REGISTERED** · Gate 2 **DO NOT ADVANCE** |
| Hypothesis Validation Programme | H2 confirmation accumulation **continues** · holding **amended** 2026-11-15 to allow 2E prep + 2X offline · see `rp608_confirmation_only_holding_state.md` |
| Phase 2 NSE OF / Crypto OF | **DEFERRED** (OF books); NSE A / crypto **plan** on 2E/2X track |
| Gate 3E / 3X | Unauthorized until matching Gate 2 ADVANCE |
| Gate 5 (production) | Unauthorized — live baseline frozen |

## Artifacts

* Gate 1: `docs/research/rp608_wl_improve_gate1_diagnostic_completeness.md`  
* Phase 1: `artifacts/rp608_econ_edge_diag/wl_trade_selection_diag_2026-11-03_to_05.*`  
* Phase 2 Crypto plan: `artifacts/rp608_econ_edge_diag/wl_phase2_crypto_plan_entry_quality_2026-11-03_to_05.*`  
* Replication: `artifacts/rp608_econ_edge_diag/wl_phase2_crypto_plan_replication_beyond_2026-11-03_to_05.*`  
* **Gate 2 amendment v2:** `docs/research/rp608_gate2_amendment_v2.md`  
* Gate 2X-001 scope/review/results: `docs/research/rp608_gate2x_001_scoped.md` · `docs/research/rp608_gate2x_001_review.md` · `artifacts/rp608_gate2x_001/`  

* Hypothesis + Gate 2 disposition (v1): `docs/research/rp608_wl_improve_crypto_plan_hypothesis_and_gate2.md`  
* Discovery v2 scope: `docs/research/rp608_wl_crypto_plan_discovery_v2_scoped.md`  
* Discovery v2 report: `artifacts/rp608_econ_edge_diag/wl_crypto_plan_discovery_v2.md`  
* NSE A Kite discovery scope: `docs/research/rp608_wl_nse_a_kite_discovery_v2_scoped.md`  
* NSE A Kite discovery report: `artifacts/rp608_econ_edge_diag/wl_nse_a_kite_discovery_v2.md`  
* NSE A Kite Gate 2 disposition: `docs/research/rp608_wl_improve_nse_a_kite_gate2.md`  
* WR Entry Discovery 001 scope: `docs/research/rp608_wr_entry_discovery_001_scoped.md`  
* WR Entry Discovery 001 report: `artifacts/rp608_econ_edge_diag/rp608_wr_entry_discovery_001.md`  
* WR Entry Discovery 002 scope: `docs/research/rp608_wr_entry_discovery_002_scoped.md`  
* WR Entry Discovery 002 report: `artifacts/rp608_econ_edge_diag/rp608_wr_entry_discovery_002.md`  
* WR Entry Discovery 002 Gate 2: `docs/research/rp608_wr_entry_discovery_002_gate2.md`  
* WR Ranking 001 scope: `docs/research/rp608_wr_ranking_001_scoped.md`  
* WR Ranking 001 report: `artifacts/rp608_econ_edge_diag/rp608_wr_ranking_001.md`  
* WR Ranking 001 Gate 2: `docs/research/rp608_wr_ranking_001_gate2.md`  
* WR Adaptive Execution Velocity 001 scope: `docs/research/rp608_wr_adaptive_execution_velocity_001_scoped.md`  
* WR Adaptive Execution Velocity 001 Gate 2: `docs/research/rp608_wr_adaptive_execution_velocity_001_gate2.md`  
* WR Adaptive Execution Velocity 001 journal inventory: `artifacts/rp608_econ_edge_diag/rp608_wr_adaptive_execution_velocity_001_journal_inventory.md`  
* WR Adaptive Execution Velocity 001 instrumentation charter: `docs/research/rp608_wr_adaptive_execution_velocity_001_instrumentation_charter.md`  
* WR Adaptive Execution Velocity 001 crypto journal inventory: `artifacts/rp608_econ_edge_diag/rp608_wr_adaptive_execution_velocity_001_crypto_journal_inventory.md`  
* Hypothesis Validation Programme: `docs/research/rp608_hypothesis_validation_programme.md`  
* Blocker Resolution Plan: `docs/research/rp608_blocker_resolution_plan.md`  
* Crypto Prospective Capture Spec (P0): `docs/research/rp608_crypto_prospective_capture_spec.md`  
* Crypto Hypothesis Availability Matrix (P2): `artifacts/rp608_econ_edge_diag/rp608_crypto_hypothesis_availability_matrix.md`  
* Crypto Paper-Engine Capability Audit (P1): `docs/research/rp608_crypto_paper_engine_capability_audit.md`  
* Hypothesis P3 Freeze: `docs/research/rp608_hypothesis_p3_freeze.md`  
* P4 H2 Crypto Plan Session Regime: `artifacts/rp608_econ_edge_diag/rp608_h2_crypto_plan_session_regime_p4.md`  
* P4 H2 Gate 2 / results memo: `docs/research/rp608_h2_crypto_plan_session_regime_p4_gate2.md`  
* H2 Prospective Confirmation Accumulation: `docs/research/rp608_h2_prospective_confirmation_accumulation.md`  
* H2 Confirmation Assessment Protocol: `docs/research/rp608_h2_prospective_confirmation_protocol.md`  
* P4 H2 Reproducibility Audit: `artifacts/rp608_econ_edge_diag/rp608_h2_p4_reproducibility_audit.md`  
* Research Decision Ledger: `docs/research/rp608_research_decision_ledger.md`  
* H4 Current-Engine Boundary: `docs/research/rp608_h4_current_engine_boundary.md`  
* H2 Confirmation DQ Monitor: `scripts/research/rp608_h2_confirmation_dq_monitor.py` · `artifacts/rp608_econ_edge_diag/rp608_h2_confirmation_dq_monitor.md`  
* Existing Economic-Result Audit: `artifacts/rp608_econ_edge_diag/rp608_existing_economic_result_audit.md`  
* Research Reproducibility Safeguards: `docs/research/rp608_research_reproducibility_safeguards.md`  
* Future Hypothesis Reopening Criteria: `docs/research/rp608_future_hypothesis_reopening_criteria.md`  
* Confirmation-Only Holding State: `docs/research/rp608_confirmation_only_holding_state.md`  
* Premature/deferred note: `artifacts/rp608_econ_edge_diag/PREMATURE_PHASE2_DEFER_NOTE.md`  
