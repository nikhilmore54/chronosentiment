# Scientific Coralys-Wrapped Recommendation Engine (Adapter Charter)

**Protocol:** `RP-608-SCI-ADAPTER-REC-001`  
**Date:** 2026-11-27 (Asia/Calcutta)  
**Status:** P2 TWIN CODE COMPLETE (2026-11-27 Asia/Calcutta) — BTC observational twin env default **off** (Anuj-gated arm); P1 BTC `ADVANCE_CANDIDATE`; dual soak untouched  
**Authority:** 0.0% until Gate 5 after sealed offline package + observational twin promote  
**Working base:** `governance-hardening`  
**Aligned with:** Project `preferences.md` · `.cursor/rules/chronosentiment-core.mdc` §3b/§3c · Gate 2 v2 (`docs/research/rp608_gate2_amendment_v2.md`) · Coralys no-touch · Rust-first / Python fail-closed I/O · 15m OHLCV first · books never pooled  

**Supersedes (as the scientific path):** heuristic Gate 2E A/B knob stacks and clean-slate `adapter_select` soft-score `|S|≥0.35` as *promotion-grade* selection science. Those twins continue **observational** on Pi and are **baseline comparators** (S6); they are not the scientific promote path.

**Repo mirror (PR):** `docs/research/rp608_scientific_coralys_adapter_recommendation_engine.md`

---

## 0. One-line intent

Build a **complete, scientifically based trade recommendation engine as an adapter around Coralys**: Coralys/`coralys-*` remain a **sealed black-box oracle**; the ChronoSentiment adapter owns feature construction, labeling, statistical modeling/calibration, ENTER/STANDASIDE with uncertainty, and W/L-first evaluation — **zero edits to any `coralys-*` crate**.

---

## 1. Problem statement — why current heuristic gates are not scientific

### 1.1 What is running (observational context)

| Track | Mechanism | Dual-soak ~12:30 IST 2026-11-27 (Asia/Calcutta) |
| --- | --- | --- |
| Gate 2E A | Live PRIMARY stacked filters (pattern allow_only, B-persist, require-OF, regime align, …) | Live stack; not a sealed scientific package |
| Gate 2E B | Binary 15m lookback **agreement** (≥2 agree / 0 oppose) | BTC 20L / ETH 38L → **DO_NOT_ADVANCE** |
| `adapter_select` V1 | Soft score from hand weights + **\|S_side\| ≥ 0.35** enter band | BTC 5L −$10.68 / ETH 1L −$7.20; suppress rising |

Preferences lock: leave both soaks observational; prefer a **scientifically based** Coralys-wrapped adapter over further soft-score / gate stacking.

### 1.2 Why Gate 2E A/B fail the scientific bar

| Defect | Gate 2E A (stack) | Gate 2E B (trend-agree) |
| --- | --- | --- |
| **No predeclared estimand → holdout seal** | Knobs added as successive band-aids on live PRIMARY without a single sealed offline package that freezes H₀ before seeing soak W/L | Agreement rule frozen from design intuition; soak then used as discovery, not confirmation |
| **Post-candidate veto only** | Filters an already-emitted side; cannot re-rank expectancy or abstain with calibrated uncertainty | Same architecture: allow/block after Coralys/Estimand chose side |
| **Agreement ≠ expectancy** | — | Sign-vote consistency is not P(win), not E[net], not calibrated coverage |
| **Knob stacking / multiple testing** | Each new filter reuses contaminated decision surface; family-wise error uncontrolled | Retuning L3/L8/L20 thresholds on hostile soak would be remine |
| **No uncertainty** | Binary open/suppress | Binary STANDASIDE; no confidence interval or calibration curve |

### 1.3 Why `adapter_select` soft-score `|S|≥0.35` is not scientific

Clean-slate slice 1 (`RP-608-ADAPTER-SELECT-15M-V1`) correctly moved selection into the adapter and left Coralys untouched — but the **scoring kernel** is still heuristic:

1. **Hand-chosen functional form** — weights `0.45/0.35/0.20`, blends `0.35` mag-rank / `0.65` sat, `0.85` soft / `0.15` path are design constants, not coefficients fit under a chronological firewall with a locked test fold.
2. **Enter band `|S|≥0.35` (D2)** — a unitless soft score threshold without mapping to P(win\|features) or E[net\|features]; no isotonic/platt calibration, no predeclared power/coverage for that cut.
3. **No labeled offline package before freeze** — D1–D5 were product freezes for an observational twin, not a Gate 2E sealed package with train / purge / holdout and “never remine holdout.”
4. **Score is not a probability or expectancy** — `S ∈ [−1,+1]` has no likelihood interpretation; ENTER/STANDASIDE therefore cannot report uncertainty scientifically.
5. **Same post-candidate filter shape** — still `candidate_side × score` gating, not a full recommendation engine that can abstain, rank, or emit calibrated ENTER with disclosed coverage.

**Verdict:** A/B and `adapter_select` remain useful **observational baselines / negative controls**. They are **not** the scientific recommendation engine Anuj requested.

---

## 2. Scientific method (binding)

### 2.1 Predeclared hypotheses (before any fit on evaluation windows)

Each package must state, in writing, before code touches labeled outcomes on the evaluation fold:

| Element | Requirement |
| --- | --- |
| **H₀ / H₁** | Falsifiable: e.g. “decision-time feature vector X improves W/L vs ungated BASELINE on book B at horizon H” |
| **Estimand** | Primary = closed-trade **W/L** per book; supporting = payoff asymmetry, friction, net, coverage |
| **Universe** | Symbols, sessions, decision-clock (15m first), entry source (Coralys/Estimand candidate filter vs generator — Anuj decision) |
| **Label** | Win/loss under **frozen** live exit geometry (no Gate 5 retune inside the package) |
| **Feature set** | Closed list; decision-time only; versioned schema |
| **Model family** | Named a priori (see §4.3); complexity / regularization / early-stop rules fixed |
| **Split plan** | Chronological train → purge gap → validation → **sealed holdout**; or purged / embargoed CV equivalent |
| **Advance rule** | Gate 2 v2 §2.1 verbatim + book-specific floors declared before results |
| **Failure modes** | Coverage floor breach, holdout non-replication, WR-only net-hostile, leakage, remine |

### 2.2 Decision-time features only

**Allowed at entry decision \(t\):** completed 15m OHLCV available at \(t\), Coralys/Estimand **sealed oracle fields already emitted on the decision card / bridge** at \(t\), adapter-constructed transforms of those (ranks, lags ending at \(t\)).

**Forbidden as entry predictors:** post-entry MFE/MAE/path, later marks, exit reason, future OF, forming/partial bar, any field requiring lookahead reconstruction without an audited as-of join.

Candle OHLCV volume is **not** a tick trade-authenticity filter (`.cursor/rules/rp608-tick-volume-trade-filter.mdc`); tick trade-like gating stays separate hygiene.

### 2.3 Chronological firewall

```
[ older train  |  purge/embargo  |  validation  |  SEALED HOLDOUT (never remine)  |  live soak ]
```

- Order by decision timestamp; no random row shuffle across time.
- **Purge / embargo** ≥ max feature lookback + label horizon (and any label construction lag) so train rows cannot leak into holdout via overlapping paths.
- Purged k-fold / combinatorial purged CV (Lopez de Prado–style) allowed **inside** the train+validation block only; the outermost holdout remains untouched until a single predeclared score.

### 2.4 Holdout never remine

Once a holdout window is scored for ADVANCE / DO_NOT_ADVANCE, it is **burned**. New hypotheses require **new** windows and/or new feature schemas with a fresh sealed package. Remining 3–5 Nov or other contaminated windows remains forbidden (`rp608_gate2_amendment_v2.md`).

---

## 3. Coralys as sealed oracle — adapter-consumed inputs only

### 3.1 Hard boundary

- **No edits** to `coralys-*` crates, `coralys_decision_server`, or Coralys recommendation policy versions.
- Adapter may **call** existing frozen APIs / consume already-persisted decision artifacts.
- Do **not** invent Coralys endpoints, fields, or reinterpret baseline OF as aggression (OFX boundary locked).

### 3.2 Inspected adapter boundaries (repo facts — consume only what exists)

| Surface | Location | What the adapter may treat as sealed oracle |
| --- | --- | --- |
| Decision cards / overlay | `scripts/run_rp608_shadow_daemon.py`, `scripts/rp608_trade_plan_overlay.py` | Emitted card fields at decision time, including `policy_b_action` (candidate side), `reference_price`, `atr_14`, OF support stamps (`order_flow_supported` / imbalance / label) **as labeled constructs**, pattern/regime stamps already on the card, timestamps / bar indices |
| Position-state bridge | `adapters/chronosentiment/src/product/coralys_state_bridge.rs` | Frozen `RecommendationEngineV1` / `evaluate_coralys_direction_aligned_state` outputs (`RecommendationRecordV1`, `CoralysPositionAction`) — interpret only; do not change Coralys internals |
| Gate 2X decide wrapper | `adapters/chronosentiment/src/product/gate2x_decide.rs` | Calls bridge; adapter research must not alter Coralys invocation contract |
| Clean-slate / Gate 2E B loops | `rp608_position_decide --adapter-select-15m-loop` / `--trend-agree-loop` | Pattern for Rust-owned CLI + Python fail-closed I/O; **new** scientific engine gets a **new** protocol id / CLI, not a silent retune of V1 constants |
| Portfolio product engine | `adapters/.../recommendation_engine.rs` | Separate portfolio allocation product surface — not the RP-608 shadow entry oracle; do not conflate |

### 3.3 Explicit non-invention rule

If a desired Coralys signal is **not** already present on the decision card or bridge record at time \(t\), the scientific adapter **must not** fabricate it by reaching into Coralys crates. Options: (a) drop the feature; (b) construct an **adapter-owned** feature from market data the adapter already has (e.g. completed 15m closes); (c) request a **future** Coralys export change as a separate Anuj-gated programme (out of scope here).

### 3.4 Construct discipline for OF stamps

Baseline `OF_SUP` / ±0.30 on cards = **L1 resting imbalance** (locked). Scientific features may use the stamp as a categorical/numeric covariate **without** relabeling it as aggressor flow. Aggression/volume-surge research stays OFX-only and must not pool into this engine’s promote path without a separate protocol.

---

## 4. Adapter-owned recommendation engine architecture

### 4.1 Pipeline (conceptual)

```
Coralys sealed oracle @ t (optional covariates)     completed 15m OHLCV + volume @ t
        │                                                      │
        ▼                                                      ▼
┌──────────────────────┐                            ┌──────────────────────┐
│ Oracle feature pack  │                            │ Adapter price+volume │
│ (black-box; optional)│                            │ features (Rust)      │
└──────────┬───────────┘                            └──────────┬───────────┘
           └─────────────────────┬────────────────────────────────┘
                                 ▼
                    ┌────────────────────────────┐
                    │ Label join (offline only)  │  frozen exits → W/L, payoff, net
                    │ Chronological / purged CV  │
                    └──────────────┬─────────────┘
                                   ▼
                    ┌────────────────────────────┐
                    │ Adaptive ladder A→B +      │  fit train; calibrate val
                    │ isotonic → p̂_L, p̂_S + CI   │  (S1=b generator)
                    └──────────────┬─────────────┘
                                   ▼
                    ┌────────────────────────────┐
                    │ Joint policy (S3)          │  ENTER / STANDASIDE / INSUFFICIENT
                    │ + journal uncertainty (S9) │
                    └──────────────┬─────────────┘
                                   ▼
                 Hygiene (existing) → new observational twin → scorecard vs baselines (S6)
```

### 4.2 Feature construction (15m first)

**Phase-0 feature families (adapter-owned; completed bars only; S10 = price + volume):**

| Family | Sketch | Notes |
| --- | --- | --- |
| **Price — signed returns** | \(r_k=(c_{-1}-c_{-k})/\|c_{-k}\|\) for predeclared \(k\) (start `{3,8,20}`) | Magnitude retained (unlike B sign-only) |
| **Price — rank / path** | Rolling rank or robust z of \|r_k\|; fraction of L20 steps aligned with \(r_{20}\) | Window length sealed before fit |
| **Volume — relative / surge** | Completed-bar volume vs rolling mean (e.g. 20); \(\Delta\log V\); rank of volume | Candle volume is a **bar feature**, not a tick trade-authenticity filter |
| **Side scores (S1=b)** | Evaluate LONG and SHORT feature packs independently → \(\hat p_L, \hat p_S\) | Generator mode: pick argmax if joint cut clears; else STANDASIDE |
| Oracle pack (optional covariates) | Coralys/Estimand sealed fields **if present** at \(t\) (OF stamp as L1 construct, ATR/ref, …) | Pass-through; no Coralys edits; may be absent under pure generator path |
| Insufficient | Missing price/volume lookback → `INSUFFICIENT` | Fail-closed |

**Later intervals** (5m, 1h, …) only after 15m package seals a scientific result or a deliberate multi-horizon extension decision — preferences: 15m first; score **per horizon book**, never pool.

### 4.3 Labeling

| Label | Definition |
| --- | --- |
| **Primary** | Closed trade win (1) / loss (0) under frozen production exit geometry for that book |
| **Supporting** | Signed net after friction; avg\|win\|/avg\|loss\|; gross; hold time (reporting only) |
| **Horizon books** | Separate label streams per book (BTC plan, ETH plan, NSE A, …) — never pooled for fit or score |
| **Censoring** | Still-open trades excluded from closed W/L until resolved; report open count |

### 4.4 Model / rule family (statistical justification)

Pick **one** family per sealed package (Anuj decision §8). Allowed v1 candidates:

| Family | Justification | Constraints |
| --- | --- | --- |
| **A. Calibrated logistic / GLM** | Interpretable linear log-odds; Wald/LRT for coefficient significance on train; isotonic or Platt on validation only | L2 / early feature freeze; no stepwise remine on holdout |
| **B. Regularized GLM / elastic net** | Handles correlated lookbacks; selection via purged CV inside train block | Hyperparams locked before holdout |
| **C. Monotone scorecard + isotonic** | Maps a predeclared monotone transform of features to calibrated P(win) | Cut on calibrated p̂, not raw soft score |
| **D. Predeclared sparse rule list** | Human-auditable; only if each clause has a predeclared hypothesis and coverage floor | No clause added after seeing holdout |

**Frozen choice (S2 + S8 adaptive):** start at **A**, always attach **isotonic calibration on validation** (C’s calibration layer), escalate to **B** only under the predeclared adaptive ladder in §8 — never jump to unconstrained ML.

**Not allowed for v1 promote path:** unconstrained gradient boosting / deep nets without a separate complexity protocol; random forests tuned on holdout; stacking Gate 2E A knobs as “features” without sealing; online remine of coefficients from Pi soak.

**Determinism:** frozen weights + frozen feature schema → same inputs → same recommendation (ChronoSentiment invariant). Offline CV may use fixed seeds only for non-decision tie-breaks; live path has **no** randomness.

### 4.5 Calibration and ENTER / STANDASIDE with uncertainty

1. Fit predictive score on train → raw \(s\).
2. Calibrate on validation → \(\hat p = P(\mathrm{win}\mid s)\) (isotonic or Platt; method sealed).
3. Optional expectancy map \(\hat u = \hat p\cdot\mathbb{E}[|win|] - (1-\hat p)\cdot\mathbb{E}[|loss|]\) using **train** payoff moments only (disclose; do not retune exits).
4. **Policy (frozen S1=b · S3=joint):**
   - Build \(\hat p_L, \hat p_S\) (and CIs) from price+volume features at \(t\)  
   - `INSUFFICIENT` if feature pack incomplete  
   - Candidate side = \(\arg\max(\hat p_L, \hat p_S)\); require \(\hat p_{\star} \ge p_{\min}\) **and** CI lower bound \(\ge p_{\mathrm{lb}}\) (joint)  
   - Optional expectancy disclosure \(\hat u\) (train payoff moments) — **not** a substitute cut unless a later package freezes it  
   - else `STANDASIDE`
5. Journal (frozen S9): \(\hat p_L/\hat p_S\), chosen side, CI / reliability bin, feature schema hash, model ladder stage (A|B), band, block reason. UI surfacing deferred.

### 4.6 Rust / Python ownership

| Layer | Owner |
| --- | --- |
| Feature math, model apply, calibration lookup, band policy | **Rust** in `adapters/chronosentiment` (new module + CLI; new protocol id) |
| Offline package fit / purged CV / reports | Rust preferred; Python thin I/O / plotting OK |
| Daemon fan-out / journal | Python fail-closed if Rust bin missing |
| Coralys crates | **No-touch** |

---

## 5. Evaluation protocol (Gate 2 v2 aligned)

### 5.1 Scorecard (per book — never pooled)

| Priority | Metric | Role |
| --- | --- | --- |
| **1** | Closed W/L (wins / closed; loss rate) | Primary advance metric |
| **2** | Payoff asymmetry avg\|win\|/avg\|loss\| | Required disclosure |
| **3** | Friction, gross, net | Required disclosure; WR-only net-hostile = fail |
| **4** | Coverage | Floor: preferred cohort ≥ **min(40% of matched BASELINE opens, or absolute \(n\ge 80\))** on holdout unless tighter floor predeclared |
| **5** | MFE/path grades | Hypothesis support only; not entry causes; not loss “explanations” alone |

Morning MTM / open marks are **not** substitutes for closed-trade W/L.

### 5.2 Controls

- **BASELINE** ungated (or production-matched) twin for the same book  
- Optional: Gate 2E A, Gate 2E B, `adapter_select` V1 as **observational** comparators — not parents of this promote path  
- Matched decision-time cohort where possible (same candidate events)

### 5.3 Twin observational promote path

1. Offline package → `ADVANCE_CANDIDATE` or `DO_NOT_ADVANCE` (memo).  
2. If ADVANCE_CANDIDATE: arm **new** observational twin env (new flag; default off) — **do not** flip Gate 2E B or `adapter_select` knobs.  
3. Chronological Pi soak; same W/L-first scorecard.  
4. Promote criteria met → Gate 5 packet (still Anuj-gated; authority 0% until approved).  
5. Failure → hold or abandon; **no** silent threshold ratchet on the soak window.

---

## 6. Explicit non-goals

1. **No `coralys-*` edits** (crates, server policy, recommendation v0 internals).  
2. **No Gate 5 floor / OF ±0.30 / traj15 / stop-target / hold retunes** without Anuj approval and a Gate 5 packet.  
3. **No book pooling** (BTC+ETH+NSE metrics never combined for fit, score, or advance).  
4. **No restart / retune** of live Gate 2E B or `adapter_select` soaks for this charter; leave dual soak observational.  
5. **No OFX authority** and no reinterpretation of baseline OF as aggressor flow.  
6. **No WR-only success**; no symbol/hour ban lists as the sole rule; no remine of burned holdouts.  
7. **No silent replacement** of PRIMARY with the scientific twin without Gate 5.  
8. **P2/P3** still require Anuj gate before Pi arm; dual soak baselines remain untouched.

---

## 7. Phased plan

| Phase | Deliverable | Exit criteria |
| --- | --- | --- |
| **P0 — Charter** (this doc) | Scientific method + boundaries + **S1–S10 freeze** | **DONE** 2026-11-27 |
| **P1 — Offline package** | Price+volume feature schema v1, labels under frozen exits, purged CV / chronological splits, adaptive A→B fit, joint calibration cuts, sealed holdout scorecard, leakage audit | **BTC `ADVANCE_CANDIDATE`** · ETH/NSE not run · review `docs/research/rp608_sci_adapter_rec_p1_btc_review.md` |
| **P2 — Sealed twin** | Rust apply path + new protocol id + Python fail-closed client + twin book dirs; env default off | **CODE LANDED** · scoped `docs/research/rp608_sci_adapter_rec_p2_twin_scoped.md` |
| **P3 — Pi soak** | Anuj-gated arm on agreed books (15m BTC first; ETH when join ready) | **ARMED 2026-11-27 IST** BTC only · memo `docs/research/rp608_sci_adapter_rec_p3_pi_arm.md` · Gate2E B + adapter_select baselines left running; ETH no SCI |
| **P4 — Promote criteria** | Soak W/L direction matches offline; coverage floor; payoff/net disclosed; price+volume schema locked | **DRAFT** `docs/research/rp608_sci_adapter_rec_p4_promote_criteria_draft.md` · soak `HOLD_SOAK` until closed n; Gate 5 only after Anuj |

---

## 8. Frozen decisions (Anuj 2026-11-27 Asia/Calcutta)

| # | Anuj input | Freeze |
| --- | --- | --- |
| **S1** | **b** | Adapter is a **side generator**: emit LONG/SHORT from \(\hat p_L/\hat p_S\) independently of Coralys candidate side. Coralys remains sealed oracle for optional covariates / position bridge only — not the entry-side authority on this twin. |
| **S2** | Best suited | **Calibrated logistic / GLM (A)** as v1 default, with **isotonic calibration** on validation. Best suited because: (1) small-n per book, (2) interpretable coefficients for Gate 2E review, (3) pairs cleanly with joint CI cuts (S3), (4) deterministic apply path in Rust. Sparse rules (D) too brittle for price+volume interactions; unconstrained ML banned. |
| **S3** | Joint | ENTER only if \(\hat p_{\star} \ge p_{\min}\) **and** CI lower bound \(\ge p_{\mathrm{lb}}\). Exact \(p_{\min}/p_{\mathrm{lb}}\) locked in P1 before holdout score (predeclared on validation only). |
| **S4** | Sources connected | **15m first.** Book order: **BTC** offline+twin first (canonical 15m + REC-001-H history present); **ETH** as soon as OHLCV+label join is wired (sources connected — do not block design); **NSE** after crypto 15m package seals or parallel only if Anuj arms. Never pool books. Other intervals deferred. |
| **S5** | Historical OHLCV/ticks available | **P1 chronological default (BTC 15m / 2024 store, Asia/Calcutta dates):** train `2024-01-01 → 2024-08-31`; purge/embargo ≥ max(lookback bars, label horizon) — default **10 completed 15m bars + label horizon**; validation `2024-09-01 → 2024-10-31`; **sealed holdout** `2024-11-01 → 2024-12-31`. Live RP-608 research windows already burned for *live-journal* remine (incl. 2026-11-03…05 and contaminated Gate 2E thr scans) stay burned. Tick history may support hygiene/replay audits; **entry features remain completed 15m OHLCV (+ bar volume)** unless a later package freezes tick features under the same firewall. |
| **S6** | Leave running; use as baseline | Gate 2E B + `adapter_select` V1 stay armed observational; scientific twin = **new** protocol/env only. Scorecards compare scientific twin vs BASELINE **and** vs A/B/`adapter_select` as baselines — never share mutable knobs. |
| **S7** | History available | Canonical P1 loci: `datasets/crypto_15m/binance_btcusdt_2024_15m.json` · `datasets/recommendation/historical_crypto/` (BTCUSDT.jsonl.gz) · Pi/shadow decision-card + trade-plan journals when labeling under frozen live exits · NSE `datasets/recommendation/historical/` only for NSE book (never pooled). ETH 15m join path to be recorded in P1 memo when wired. |
| **S8** | Prefer adaptive | **Adaptive complexity ladder (sealed, not online):** Stage 0 = A (GLM) + isotonic; if validation fails predeclared calibration/coverage/power gates → Stage 1 = B (elastic net) with hyperparams chosen only inside train purged-CV, then re-calibrate on validation; **one** ladder choice frozen before holdout. Max features v1: ≤ 24 (price+volume core + optional oracle covariates). Max purged-CV rounds inside train: ≤ 5. Ban: GBDT/deep nets/RF; ban soak-time coefficient updates. |
| **S9** | No idea → recommended default | **Journal-first:** every ENTER/STANDASIDE stamp must carry \(\hat p\), CI (or reliability bin), ladder stage, schema hash. Scorecard must show calibration ECE/reliability. **UI deferred** until after P2 twin exists. |
| **S10** | Consider price + volume | Feature schema **must** include price **and** completed-bar volume families (S10). Promote bar: soak **W/L direction** matches offline ADVANCE_CANDIDATE **and** coverage floor met **and** payoff/net disclosed; magnitude band match optional disclosure, not a hard fail unless P1 freezes tighter bands. Price+volume ablation (price-only vs price+volume) predeclared inside train/val only — holdout sees the frozen winner once. |

### 8.1 Locked at P1 BTC package (2026-11-27)

| Item | Lock |
| --- | --- |
| \(p_{\min}\), \(p_{\mathrm{lb}}\) | **0.30 / 0.18** (validation-only grid; holdout sealed after) |
| Feature schema | `price_volume_15m_v1` · 13 cols · hash `f0cc1fe4…6b5f` |
| Ladder | **A** (GLM + isotonic) |
| Labels | REC-001-H TARGET_BEFORE_RISK / RISK_BEFORE_TARGET; HORIZON censored |
| BTC verdict | **ADVANCE_CANDIDATE** (holdout WR 33.4% vs base 29.7%; cov 40.0%; asym ≈1.98) |
| ETH 15m path | Still to wire before ETH package |
| Pi journals | Confirm on P2/P3 twin under live frozen exits |

---

## 9. Relation to prior docs

| Doc | Relation |
| --- | --- |
| `docs/clean-slate-15m-adapter-selection.md` (Project store) / `docs/research/rp608_clean_slate_15m_adapter_selection.md` | Parallel clean-slate track; scientifically superseded as promote-grade method; twin may keep soaking |
| `docs/research/rp608_adapter_select_15m_v1_scoped.md` | Frozen D1–D5 observational baseline — not the scientific kernel |
| `docs/research/rp608_gate2e_15m_ohlcv_trend_agree_scoped.md` | Gate 2E B observational — DO_NOT_ADVANCE path; leave running |
| `docs/research/rp608_gate2_amendment_v2.md` | Binding advance criteria for this engine’s offline + twin evaluation |
| `docs/research/rp608_wl_improvement_plan.md` | Parent WL-IMPROVE programme; this charter is the scientific Gate 2E-shaped adapter path |
| `docs/research/rp608_sci_adapter_rec_practical_research_steer.md` | Short literature→product steer; does not unfreeze S1–S10 |
| `docs/research/rp608_sci_adapter_rec_literature_survey.md` | Peer-reviewed journals/transactions/proceedings survey (part 1) |
| `docs/research/rp608_sci_adapter_rec_literature_survey_part2.md` | Part 2: live selection failure · meta-labeling (JPM/JFDS) · HLZ multiplicity · **design freeze** / Anuj package pick |

---

## 10. Immediate ops constraints (this charter)

- Do **not** change `RP608_ADAPTER_SELECT_15M_*` or Gate 2E B env/knobs (S6 baselines).  
- Do **not** modify live twin soft-score weights or `|S|≥0.35`.  
- S1–S10 are **frozen**; P1 BTC offline **ADVANCE_CANDIDATE**; P2 twin code landed; **P3 BTC Pi arm live** (`RP608_SCI_ADAPTER_REC_TWIN=1` on BTC observer only).  
- Next: **soak → P4 promote criteria** only after Anuj; still no Coralys edits, still authority 0%; ETH SCI deferred until join; no Gate 5 from arm alone.

---

*End of charter — `RP-608-SCI-ADAPTER-REC-001` · S1–S10 frozen · P1 BTC ADVANCE_CANDIDATE · P2 twin ready · P3 BTC Pi armed 2026-11-27.*
