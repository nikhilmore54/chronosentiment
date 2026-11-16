# RP-608-WR-ADAPTIVE-EXECUTION-VELOCITY-001 — Observational Specification (Frozen)

**Protocol:** `RP-608-WR-ADAPTIVE-EXECUTION-VELOCITY-001`  
**Date:** 2026-11-06  
**Authority:** 0.0% · baseline frozen · Gate 2 **CLOSED** · Gate 3 unauthorized  
**Book:** NSE Estimand A only (books never pooled)  
**Host:** Pi (observational diagnostics)  
**Predecessor evidence:** `RP-608-WR-RANKING-001` continuous-field audit  

## Status

**REGISTERED — OBSERVATIONAL SPEC ONLY.**  
**Unit boundary (2026-11-06):** inventory closed · charter registered · **awaiting prospective capture** · no further ranking run · no threshold scan · no execution change.  

Central principle: market state may explain entry opportunity; execution instrumentation must establish whether execution behavior contributes **independently**. The charter enables future E2; it does **not** authorize urgency or execution-policy changes.

No execution-velocity policy is authorized. No live or paper execution change. No OF/stop/target/horizon change.

## Research question (locked)

> Does a decision-time adaptive execution-velocity response to continuous market state improve entry outcomes relative to the frozen execution baseline, across unseen sessions and at measurable coverage?

**Not:**

- Does `momentum_4bar_bps < 55` produce ~90% WR?  
- Can we retune categorical `p_hat` thresholds on the contaminated holdout?  
- Is Oct 28’s 31/6 a validated policy?

## Why this unit

RANKING-001 showed Oct 28 and Oct 30 share the same categorical entry pocket (`MOM_UP ∩ LOC_MID`, LONG, hours 11–12, `p_hat ≥ 0.50`) but differ in continuous magnitudes:

| Session | Selected WR | `momentum_4bar_bps` mean | `trend_slope_bps` mean |
| --- | ---: | ---: | ---: |
| 2026-10-28 | 83.8% (31/6) | 46.5 | 19.4 |
| 2026-10-30 | 59.3% (16/11) | 70.0 | 39.8 |

Categorical score treats these as equivalent. Continuous state may characterize **entry urgency** — but 1-bar/4-bar win-ranking **flips across sessions**, so no fixed “higher mom → slower entry” rule is assumed.

Oct 28 also had a higher **full-session** baseline WR (58.5% vs 37.9%). Any adaptive-execution claim must be separated from favorable session conditions.

## Critical distinction (locked)

| Mechanism | Definition | Changes trade population? |
| --- | --- | --- |
| **A. Execution adaptation** | Signal already approved; only how quickly the system attempts to execute (urgency / aggression / delay-to-touch) | No (same intended trades; possibly different fill price / fillability) |
| **B. Entry-timing adaptation** | Validity / release of the entry depends on continuous mom/trend/location evolving | **Yes** (different trades, WR, coverage) |

These are **not interchangeable**. Delaying until momentum “cools” is Mechanism B unless the delayed attempt is still the same approved signal under a pure execution-latency policy with a hard cancel/abandon rule that is predeclared and measured separately.

This unit’s **primary estimand** is Mechanism **A**. Mechanism B, if studied, must be labeled as **entry-selection / timing**, not “execution velocity,” and cannot borrow Gate language from A.

## Research hypothesis

> Entry execution velocity should adapt to the continuous decision-time market state, rather than remain uniform across all entries within the same categorical signal.

This is a **research hypothesis**, not a demonstrated improvement.

## Frozen observational protocol (first deliverable)

### 0. Contamination firewall

- Oct 28’s 31/6 and continuous contrasts are **discovery / motivation only**.  
- **Forbidden:** construct an adaptive policy on Oct 28 (or any holdout day used to inspire the cut) and present that same day as validation.  
- Discovery fit window for any state→urgency mapping: **discovery days only** (`2026-11-03…05`), unless a later protocol explicitly redefines windows.  
- Prospective confirmation requires **later untouched sessions** beyond the contaminated holdout join window.

### 1. Market-state characterization (decision-time only)

Allowed continuous fields (already in Kite join; look-ahead-safe):

- `momentum_1bar_bps`, `momentum_4bar_bps`  
- `trend_slope_bps`  
- `price_location_pct`  
- Joint state with categorical pocket labels (`trend`, `momentum`, `price_location`, `side`, `hour_ist`)

Forbidden as predictors: post-entry MFE/MAE, exit reason, holding path, later marks.

**No assumed monotonic map** from momentum to urgency. First report stability of state→outcome associations across sessions; only then propose candidate urgency schedules.

### 2. Execution-velocity characterization (observational)

Before any counterfactual policy, measure **what the frozen baseline already does**:

- Time from signal/decision timestamp → entry fill (if recoverable from journals)  
- Entry vs contemporaneous touch / mid (slippage proxy)  
- Whether urgency proxies differ across continuous-state strata **under today’s uniform policy**

If journals lack latency fields, label `UNAVAILABLE` — do not impute. State characterization may still proceed; execution-velocity effects then remain unidentified until instrumentation exists.

### 3. Entry-price effects

Attribute differences to:

| Bucket | Evidence |
| --- | --- |
| Fill / slippage | Entry vs touch/mid at decision and at fill |
| Timing within bar / delay | Decision→fill latency distribution by state |
| Underlying signal / session | Same-state WR stratified by session baseline |

Do not credit “adaptive velocity” for session-quality differences.

### 4. Outcome effects (required jointly)

Any candidate comparison reports together:

- W/L counts and WR  
- Coverage (trades retained / eligible)  
- Payoff (avg\|win\| / avg\|loss\|)  
- Friction and **net**  
- Session stability / concentration (no single-session ≥60% of claimed lift mass)

WR-only success is forbidden (`.cursor/rules` §3b).

### 5. Session transfer

Predeclared contrast set (descriptive): Oct 28, Oct 30, Nov 2, discovery aggregate, Nov 6 forward if present.  
**Validation set:** later untouched IST sessions not used in policy construction.  
Pass requires transfer on validation — not replication of Oct 28.

## Estimands (predeclared)

| ID | Estimand | Mechanism |
| --- | --- | --- |
| E1 | Association: continuous state strata vs baseline W/L / payoff / net (descriptive) | State only |
| E2 | Association: state strata vs observed decision→fill latency / slippage (if available) | A observational |
| E3 | Counterfactual Mechanism A (sim only, later tranche): urgency schedule changes fill model only; **same trade IDs** | A |
| E4 | Counterfactual Mechanism B (sim only, later tranche, separately labeled): state-gated release changes trade set | B — not “velocity” |

E3/E4 require a **separate authorized simulation tranche** after this observational spec and instrumentation check. They are **not** authorized by registering this document.

## Criteria for “genuine adaptive-velocity improvement” (informational)

All required before any Gate 2 reopen discussion for Mechanism A:

1. Instrumentation sufficient for E2 or a validated fill model for E3.  
2. Discovery-constructed urgency map; **untouched** forward sessions show lift vs frozen baseline on W/L **and** net, at coverage ≥ predeclared floor (default ≥25% of eligible in the studied pocket, or explicit pocket-coverage disclosure).  
3. Lift not dominated by one session (≥60% mass).  
4. Payoff ≥ 1 and net > 0 on the evaluated set (supporting).  
5. Mechanism labeled A vs B correctly; no OF/stop/target/horizon change.  
6. Oct 28 not used as confirmation.

Failing these → improvement **not** demonstrated; Gate 2 stays closed.

## Forbidden

- Live or paper execution-velocity changes  
- Using Oct 28 (or contaminated holdout thr lore) to build and validate the same policy  
- Treating Mechanism B as Mechanism A  
- Fixed mom→speed rules without cross-session stability evidence  
- Threshold re-scans on contaminated holdout for categorical `p_hat`  
- Feature sprawl beyond declared continuous state fields without a new protocol  
- Pooling books; changing OF ±0.30 / stops / targets / horizon  
- Claiming Gate 2 advance from this registration alone  

## Governance

| Item | Status |
| --- | --- |
| Baseline execution | **FROZEN** |
| OF ±0.30 | **FROZEN** |
| Stops / targets / horizon | **UNCHANGED** |
| Live authority | **0.0%** |
| Gate 2 | **CLOSED** |
| Gate 3+ | **NOT AUTHORIZED** |
| OFX | Independent; not pooled (`.cursor/rules` rp608-ofx) |

## Artifacts (this tranche)

- Scope (this file): `docs/research/rp608_wr_adaptive_execution_velocity_001_scoped.md`  
- Gate 2 memo: `docs/research/rp608_wr_adaptive_execution_velocity_001_gate2.md`  
- Later (not this deliverable): observational report + optional sim tranche under new authorization  

## Research sequence (locked 2026-11-06)

1. Continuous market-state audit — **completed** (`RP-608-WR-RANKING-001`)  
2. NSE execution journal inventory — **completed** (E2 blocked; hypothesis not disproven)  
3. **NSE instrumentation charter** — **registered dependency** (`rp608_wr_adaptive_execution_velocity_001_instrumentation_charter.md`)  
4. Prospective event capture + integrity validation (NSE)  
5. E2 observational diagnostics — **only if** completeness floors met  
6. Separate governance before any execution experiment (E3) or Mechanism B (E4)  

### Parallel track — crypto (independent; not a substitute for NSE)

7. Crypto journal inventory — **completed**  
8. Crypto prospective capture spec (P0) — **registered** (`rp608_crypto_prospective_capture_spec.md`)  
9. P1 capability audit — H4 untestable under current engines  
10. Hypothesis Validation P4 H2 — **completed** (descriptive; Gate 2 not advanced) — does **not** reopen H4  
11. H4 current-engine boundary — **closed** as `UNTESTABLE_UNDER_CURRENT_ENGINE_DESIGN` (`rp608_h4_current_engine_boundary.md`)  

H4 is not an open research loop under current crypto engines. Market-state / session-regime results must not be labeled velocity results.

Do **not** rerun E1 under this unit’s label. Do **not** derive urgency rules from Oct 28/30 or from crypto sessions.
