# RP-608-REGIME-BREAKOUT-001 — Prospective Observation Protocol (Corrected Arm C)

**Protocol ID:** `RP-608-REGIME-BREAKOUT-001-PROSPECTIVE-v1`  
**Parent:** `RP-608-REGIME-BREAKOUT-001`  
**Prerequisite disposition:** `artifacts/rp608_regime_breakout/arm_c_measurement_disposition.md`  
**Live order authority:** **0.0%**  
**Holdout Sessions 41–68:** **SEALED**  
**Arm A:** **FROZEN**  
**Dynamic exits / threshold tuning:** **FORBIDDEN** during this window  
**Start-rule version:** **1.0** (see §10)  

Aligned with `.cursor/rules/chronosentiment-core.mdc`: deterministic event-driven decisions; multi-scenario robustness; no promotion from incomplete evidence.

---

## 0. Status (binding)

| Field | Value |
|-------|--------|
| Observation counting | **NOT STARTED** (Session 1 not begun) |
| Corrected prospective A/B/C evaluation | **AUTHORIZED** on LCUtest (`authorized: true`) — clock awaits §10 start rule + explicit operator start |
| Version / provenance artifact | `artifacts/rp608_regime_breakout/prospective_v1_authorization.json` |
| Start-rule amendment | **1.0** — governance-approved in conversation; does not rewrite prior authorization provenance |
| Existing shadow services | May continue under current safeguards; **do not** count output until a market-specific start event is persisted |

**Hard rule:** Keeping shadow infrastructure running ≠ collecting evidence for corrected Arm C. Historical Gate 4 / diagnostic packing audits are **prior evidence under different provenance** and must not be re-labeled as this prospective window.

---

## 1. Purpose

Collect a **predefined**, newly versioned, clean prospective A/B/C economic evaluation under **corrected Donchian packing** and locked breakout thresholds, then decide whether Arm C improves net outcomes over Arm B with sufficient coverage.

Central question (unchanged):

> Does Arm C improve net outcomes over Arm B while retaining sufficient coverage?

This protocol answers that question with **forward observation**, not by reinterpreting Gate 4.

---

## 2. Authorization & versioning gate (must precede counting)

Before **any** session counts toward Sessions 1–20, all of the following must be recorded in a dedicated provenance artifact (suggested path: `artifacts/rp608_regime_breakout/prospective_v1_authorization.json`):

1. **Explicit human authorization** of `RP-608-REGIME-BREAKOUT-001-PROSPECTIVE-v1` (name, UTC timestamp, statement of intent).  
2. **Code / config provenance:** git rev, hashes for `breakout_lifecycle.py`, historical/prospective harness entrypoints, and confirmation that `pack_bars_for_breakout_eval` is used for NSE and crypto.  
3. **Fixture attestation:** lifecycle fixture suite green (boundary inclusion, long/short, quote–signal alignment, dedupe/terminal stickiness, packer consistency).  
4. **Controls attestation:** Arm A frozen; holdout sealed; live authority 0.0%; locked δ / spread caps / freshness / Net R:R unchanged; no dynamic-exit branch enabled.  
5. **Output namespace:** a **new** artifact directory (e.g. `artifacts/rp608_regime_breakout/prospective_v1/`) that will **not** overwrite Gate 4 (`historical_abc_research/`) or legacy Arm C audits.  
6. **Clock start:** first session/time-block index that will be labeled `prospective_shadow` for this version.

Until that artifact exists with `authorized: true`, interim service logs remain operational hygiene only.

---

## 3. Markets (never pooled)

| Market | Observation unit | Notes |
|--------|------------------|-------|
| NSE cash equities | Trading **session** (exchange calendar day with completed intraday bars) | Evaluate on its own promotion gate |
| Crypto (BTC/USDT) | Preregistered **time-block** (per parent protocol), evaluated **separately** | Do not map 1:1 to NSE sessions to manufacture N |

**Forbidden:** pooling NSE + crypto opportunities, trades, or p-values to pass a gate.

**Floor distinction:** Parent protocol in-sample floors (≥40 NSE sessions / ≥20 crypto time-blocks) are **eligibility floors for the development (Gate 4) analysis**. They do **not** automatically prove that a 20-session prospective window is statistically sufficient. Twenty sessions is a **practical initial evaluation window**, not a power guarantee.

---

## 4. Predefined observation schedule (NSE session framing)

| Window | Sessions | Purpose | Allowed actions |
|--------|----------|---------|-----------------|
| Integrity | **1–5** | Completed-bar processing, regime classifications, breakout lifecycle transitions, quote freshness, deduplication, persistence, Arm A invariance | Integrity report only; **no** threshold or logic changes |
| Collection | **6–10** | Opportunity coverage; whether Arm C produces valid qualifications under **real** quote conditions | Descriptive metrics only; interim PnL **not** a tuning signal |
| Completion | **11–20** | Full paired A/B/C outcomes including rejects + counterfactual paths | Continue collection; no mid-window redesign |
| Decision | **Session 20 close** | Formal classification: promotion-gate **pass** / **fail** / **INCONCLUSIVE** | Extend only under §6 predefined rule |

Crypto follows the same **phase intent** on its own time-block clock after separate authorization of crypto counting, with evaluation against crypto preregistered floors—not the NSE session count.

### 4.1 Sessions 1–5 — Integrity checklist

Verify (and record) without performance-driven changes:

- Completed-bar-only regime features (no forming-bar leakage)  
- Regime label distribution sanity (`UNAVAILABLE` / `SIDEWAYS` / `TRANSITIONING` / `DIRECTIONAL`)  
- Breakout lifecycle: IDLE → WATCH / CROSS → `TRADE_QUALIFIED` | `INVALIDATED` | `EXPIRED`  
- Quote freshness (age ≤ 500 ms for active path); spread-cap enforcement  
- Dedupe keys stable; terminal states sticky  
- Persistence / journal completeness for taken and rejected opportunities  
- Arm A invariance vs frozen baseline fixtures  

**Fail integrity → stop counting; remediate infrastructure; restart Session 1 under same version only if the version charter allows a declared restart (no silent trim).**

### 4.2 Sessions 6–10 — Coverage & qualification (descriptive)

- Arm A opportunity rate; Arm B coverage; Arm C qualification rate under live/real quotes  
- Treat expectancy / PF as **descriptive**; do not change strategy, buffers, or R:R  

### 4.3 Sessions 11–20 — Complete paired books

Retain every opportunity with dispositions (`taken` / `regime_reject` / `breakout_reject`) and Arm A counterfactual path for rejects (economic book: reject → 0 for paired Δ).

At Session 20, compute **per market**:

- Net expectancy (opportunity and taken)  
- Profit factor  
- Coverage  
- Win rate and payoff (when taken-trade floors met)  
- Drawdown (session path)  
- Session-aware / clustered uncertainty (bootstrap seed `20261028` unless a new seed is preregistered in the authorization artifact)

### 4.4 Session 20 — Formal decision

Apply parent §4.5 conjoint promotion gate **per market**, with prospective label `prospective_shadow`:

- **Pass** — floors met and conjoint criteria satisfied  
- **Fail** — floors met and criteria not satisfied  
- **INCONCLUSIVE** — floors or statistical requirements unmet  

Do **not** relax criteria to avoid `INCONCLUSIVE`.

---

## 5. Arms & locked measurement semantics

| Arm | Logic |
|-----|--------|
| A | Frozen baseline |
| B | A ∩ `DIRECTIONAL` |
| C | `DIRECTIONAL` ∩ `TRADE_QUALIFIED` |

**Packing (corrected, mandatory):** `pack_bars_for_breakout_eval` — Donchian from completed bars `t-20..t-1`; signal bar is evaluation/forming slot; locked δ / spread / freshness / Net R:R unchanged.

Gate 4 packing defect is **closed** for this version; Gate 4 results remain historical provenance only.

---

## 6. Predefined extension rule (no criterion relaxation)

If at Session 20 the preregistered minimum sample or statistical requirements are unmet:

1. Classify outcome **`INCONCLUSIVE`**.  
2. Optionally authorize **`PROSPECTIVE-v1-EXT-N`** for a fixed additional block of **+10 sessions** (NSE) or the crypto equivalent time-blocks stated in that extension charter.  
3. Extension must inherit the same locked thresholds, packing, and output namespace rules; it must not reopen holdout or raise live authority.  
4. Maximum extensions under this charter: **two** (+20 sessions total) unless a new parent protocol version is written.

No mid-window “just a few more days” without a versioned extension artifact.

---

## 7. Explicit non-goals

- Watching the market indefinitely without a decision clock  
- Counting pre-authorization shadow logs toward prospective N  
- Overwriting or reinterpreting Gate 4 / legacy Arm C audits as prospective economics  
- Pooling markets, relaxing floors, or enabling dynamic exits to manufacture a pass  
- Any live order authority > 0.0%  

---

## 8. Immediate operating posture (now)

1. Keep existing shadow infrastructure operating under current safeguards.  
2. Run deployment preflight: `python3 scripts/run_rp608_prospective_v1_preflight.py` — must pass on the **deployed** revision (LCUtest) before any authorize decision. A green **local** preflight is necessary but not sufficient. Preflight **never** sets `authorized: true`.  
3. Pi gate report: `artifacts/rp608_regime_breakout/prospective_v1_pi_authorization_gate.md`.  
4. **Do not** start Session 1 counting until §2 authorization is explicitly flipped with human provenance **after** Pi `preflight_pass`.  
4. After authorization: Session 1 only when session-start criteria are met; integrity review at Session 5 (**no tuning**); formal decision at Session 20 (or `INCONCLUSIVE` + optional predefined extension).  
5. Until then: Arm A frozen; Sessions 41–68 sealed; live order authority **0.0%**.

Implementation helpers: `scripts/rp608_prospective_v1.py` (clock, DQ gates, namespace write guard, extension rule); `tests/test_rp608_prospective_v1.py`.

---

## 9. Artifact index

| Artifact | Role |
|----------|------|
| This document | Preregistered prospective schedule & gates + §10 start rule |
| `artifacts/rp608_regime_breakout/prospective_v1_authorization.json` | Authorization provenance (do not rewrite on start) |
| `artifacts/rp608_regime_breakout/prospective_v1_preflight.json` | Latest preflight attestation |
| `artifacts/rp608_regime_breakout/prospective_v1/` | Prospective outputs (isolated namespace) |
| `artifacts/rp608_regime_breakout/prospective_v1/start_events/` | Market-specific start events (`NSE.json`, `CRYPTO.json`) |
| `artifacts/rp608_regime_breakout/prospective_v1_clock_state.json` | NSE/crypto unit clocks (independent; no pooling) |
| `artifacts/rp608_regime_breakout/historical_abc_research/` | Gate 4 — immutable for this protocol |
| `artifacts/rp608_regime_breakout/arm_c_measurement_disposition.md` | Packing defect disposition |
| `tests/test_rp608_breakout_lifecycle_fixtures.py` | Measurement/fixture attestation |
| `tests/test_rp608_prospective_v1.py` | Auth/clock/DQ/namespace/start-rule tests |
| `scripts/run_rp608_prospective_v1_start.py` | Prescribed market start CLI |

---

## 10. Session Commencement and Clock-Start Rule (v1.0)

**Amendment status:** Approved as governance intent in conversation; implemented as start-rule **1.0**. This section does **not** alter Arm A, thresholds, exits, Gate 4 results, or the authorization receipt’s historical fields.

**Protocol clock states (authoritative):** `NOT_STARTED` | `STARTED` | `BLOCKED`.  
**Implementation-only:** pre-start gate `PASS`/`FAIL` may be recorded as readiness evidence; it is **not** a silent extra protocol state and does **not** start the clock.

### 10.A NSE start

1. Count only the **first complete regular NSE trading session** whose **session open** (09:15 IST on a trading day) occurs **strictly after** the recorded start-decision timestamp.  
2. All mandatory pre-start checks (§10.E) must **PASS** before that session begins.  
3. A session **already in progress** when the decision is recorded **cannot** qualify.  
4. If any required check fails, is unknown, or cannot be verified → session does **not** count; persist a reason-coded `BLOCKED` audit record.  
5. **Late invocation:** if the operator invokes start **after** the eligible session has opened, **reject** that session (do not start mid-session). Recompute the next eligible boundary.

**Observation window timestamp:** the persisted `eligible_session_boundary` (NSE trading date / open) defines which complete session is Session 1. `start_event_utc` is audit provenance for when the start command was accepted; it does **not** redefine the boundary.

### 10.B Crypto start

1. Count only the **first complete UTC calendar-day** whose `00:00:00 UTC` boundary occurs **strictly after** the recorded start-decision timestamp.  
2. Data-quality, deployment, and integrity gates apply **independently** to crypto.  
3. Do **not** infer crypto start from NSE start or vice versa.  
4. Late invocation after the eligible UTC day has begun → **reject** that day; recompute the next midnight UTC.

### 10.C Explicit invocation

Readiness, authorization, daemon uptime, and green preflight **never** start the clock alone.

The designated operator must invoke the prescribed start operation (`scripts/run_rp608_prospective_v1_start.py`). The controller must persist a valid start event identifying: market, eligible session boundary, `start_event_utc` (and IST for NSE), protocol id, start-rule version, deployed revision, operator, and pre-start gate evidence.

### 10.D Restrictions preserved

- Sessions 1–5 integrity-only (no tuning)  
- Arm A frozen; thresholds unchanged  
- Holdout 41–68 sealed; Gate 4 artifacts protected  
- NSE and crypto accounting separate (no pooling)  
- Live order authority **0.0%**  
- Extension rule (§6) and parent promotion gates unchanged  
- **No retroactive counting** of observations before the market-specific start event  

### 10.E Mandatory pre-start gates

Before accepting a start: authorized revision match; required controls (Arm A frozen, holdout sealed, live authority 0.0%); namespace isolation / Gate 4 write refusal; completed-bar / packing integrity attestation; Arm A invariance attestation from current preflight; market-specific eligibility as above. Unknown or failed → `BLOCKED` with reason code.

### 10.F Idempotency and recovery

Repeating the same start command must not create a second start, move the boundary, or reset counters. Daemon restart must reload authoritative start state and must not invent or reset a start event.
