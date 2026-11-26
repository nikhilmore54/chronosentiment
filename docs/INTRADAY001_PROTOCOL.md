# INTRADAY-001 Protocol — Intraday Prospective Observation Experiment

**Status:** DRAFT v4 — not yet frozen  
**Authorized:** 2026-08-21  
**Governance:** Independent of TIME-009 and TIME-010. Results must not be used to modify, interpret, condition, or otherwise influence TIME-009 or TIME-010.

---

## 1. Authorization and Governance Boundary

INTRADAY-001 was authorized to commence on **2026-08-21**, independently of the unresolved TIME-009/TIME-010 result.

**Explicit constraint:**

> INTRADAY-001 results must not be used to modify, interpret, condition, or otherwise influence TIME-009 or TIME-010. The two research tracks are completely independent.

The session-horizon research sequence remains:

```
TIME-008 (NEGATIVE) → TIME-009 (running) → TIME-010 (locked) → research decision
```

INTRADAY-001 runs in parallel on a separate track:

```
                 Coralys
                    │
        ┌───────────┴───────────┐
        │                       │
     TIME-009               INTRADAY-001
    session horizon         intraday horizon
        │                       │
        ▼                       ▼
    TIME-010                INTRADAY-001 analysis
    (locked)                (to be pre-registered)
```

---

## 2. Research Question

> **Does the frozen Coralys evidence classification (Favourable vs Mixed) discriminate short-horizon forward price outcomes when evaluated using intraday market-state observations on NSE equities?**

This is a prospective question. The classification is frozen before outcomes are observed. Unfavourable decisions are excluded from the primary comparison (same as TIME-009/TIME-010).

---

## 3. Observation Unit

Each observation is defined by:

```
instrument × evaluation_timestamp
```

One observation is generated per instrument per evaluation event. Multiple observations per instrument per session are permitted (one per 5-minute grid slot).

**Important limitation:** Observations from the same instrument within the same session are correlated. They are not independent trades. The statistical analysis must not treat a large N as equivalent to N independent market situations. See §11 (Dependence Limitation).

**Observation artifact fields (immutable at T0):**

| Field | Description |
|---|---|
| `decision_id` | Unique ID: `INTRA-001-{YYYYMMDD}-{HHMM}-{INSTRUMENT}` |
| `evaluation_timestamp` | ISO 8601 evaluation timestamp (IST), aligned to 5-min grid |
| `instrument` | NSE ticker |
| `reference_price` | LTP at evaluation time |
| `direction` | LONG / SHORT / NO_TRADE |
| `evidence_class` | Favourable / Mixed / Unfavourable (from frozen Coralys) |
| `target_price_h15` | Sealed at T0 for H15 horizon (see §6) |
| `risk_price_h15` | Sealed at T0 for H15 horizon |
| `target_price_h30` | Sealed at T0 for H30 horizon |
| `risk_price_h30` | Sealed at T0 for H30 horizon |
| `target_price_h60` | Sealed at T0 for H60 horizon |
| `risk_price_h60` | Sealed at T0 for H60 horizon |
| `atr_14_intraday` | Intraday ATR-14 at evaluation time (see §6 for precise definition) |
| `market_state` | Frozen TMV state at evaluation time |
| `vol_regime` | Volatility regime at evaluation time |
| `volume_regime` | Volume regime at evaluation time |
| `features_snapshot` | Full Coralys feature vector at T0 |
| `outcome_data_source` | `OHLCV_1MIN` (frozen for this experiment — see §7) |
| `producer` | `intraday001_observe.v1` |
| `policy_artifact_hash` | SHA-256 of frozen Coralys policy artifact |

---

## 4. Evaluation Schedule

Evaluations are triggered on a **fixed 5-minute grid only**. Event-triggered evaluations (volatility breakout, momentum regime change, etc.) are excluded from the primary experiment. They may be collected as a separate exploratory dataset but must not be mixed with the primary observation set.

**Primary evaluation grid:**

```
09:20, 09:25, 09:30, 09:35, …, 15:20, 15:25 IST
```

This gives **74 evaluation slots per instrument per session**:

```
(15:25 − 09:20) / 5 minutes + 1 = 365/5 + 1 = 74 slots
```

**Exclusion windows (no evaluations):**

- 09:15–09:19 IST — opening auction noise (first 5 minutes after open)
- 15:26–15:30 IST — closing auction noise

**Maximum evaluations per instrument per session:** 74

---

## 5. Observation Horizons

Three pre-specified horizons are evaluated for each observation:

| Horizon ID | Duration | Expiry rule |
|---|---|---|
| H15 | 15 minutes | evaluation_timestamp + 15 min |
| H30 | 30 minutes | evaluation_timestamp + 30 min |
| H60 | 60 minutes | evaluation_timestamp + 60 min |

Each T0 observation generates **three outcome records** (one per horizon). The three outcomes are correlated — they share the same T0 and reference price. The statistical analysis treats each horizon separately with the T0 observation as the experimental unit (see §9).

**Session-close truncation:** If a horizon expiry falls at or after 15:30 IST, the observation is truncated at the session close price and classified as `SESSION_CLOSE`. SESSION_CLOSE observations are included in Q1's denominator as non-target outcomes (see §7).

---

## 6. Target and Risk Methodology

### ATR-14 Definition (precise)

`atr_14_intraday` = Wilder ATR(14) calculated from **completed 5-minute OHLC bars** available strictly before the evaluation timestamp. The current (incomplete) bar is excluded. Minimum warm-up: 14 completed 5-minute bars (70 minutes of trading). If fewer than 14 completed bars are available, the observation is excluded (not imputed).

### Target and Risk Percentages

Target and risk percentages are computed from the intraday ATR-14 and the same Coralys TMV multiplier logic as the session-horizon model:

```
target_pct = clamp(atr_14_intraday / reference_price × target_multiplier, 0.005, 0.05)
risk_pct   = clamp(atr_14_intraday / reference_price × risk_multiplier,   0.003, 0.03)
```

TMV multipliers (frozen — same as session-horizon model):

| TMV State | Target multiplier | Risk multiplier |
|---|---|---|
| Bullish / Positive | 2.0 | 1.0 |
| Bullish / Negative | 1.5 | 0.75 |
| Bearish / Positive | 1.5 | 0.75 |
| Bearish / Negative | 1.0 | 0.5 |

### Target and Risk Price Formulas (explicit)

```
LONG direction:
  target_price = reference_price × (1 + target_pct)
  risk_price   = reference_price × (1 − risk_pct)

SHORT direction:
  target_price = reference_price × (1 − target_pct)
  risk_price   = reference_price × (1 + risk_pct)
```

The same target_pct and risk_pct values are used for all three horizons (H15, H30, H60). The horizon affects only the observation window, not the price levels.

---

## 7. Outcome Classification

### Primary outcome data source (frozen)

**Primary outcome source: 1-minute OHLCV.** Tick data may be captured concurrently for audit and reconstruction purposes but shall not be substituted into the primary outcome dataset after observations begin. The data source is recorded in each observation artifact as `outcome_data_source = OHLCV_1MIN` and must not change during the experiment.

### Outcome categories

For each (observation, horizon) pair, the outcome is classified as:

| Outcome | Condition |
|---|---|
| `TARGET` | Price reached or exceeded target_price within the horizon window |
| `RISK` | Price reached or breached risk_price within the horizon window |
| `SESSION_CLOSE` | Horizon expiry falls at or after 15:30 IST; exit at session close price |
| `HORIZON` | Neither target nor risk reached; horizon elapsed within session |
| `AMBIGUOUS` | Both HIGH >= target_price and LOW <= risk_price (LONG), or both LOW <= target_price and HIGH >= risk_price (SHORT), occur within the same 1-minute bar |

### First-touch rule (OHLCV mode)

Because the primary data source is 1-minute OHLCV, intrabar tick ordering is not available. When both target and risk levels are breached within the same 1-minute bar, the outcome is `AMBIGUOUS`. AMBIGUOUS observations are excluded from Q1.

For non-ambiguous bars: TARGET is recorded when the relevant extreme (HIGH for LONG, LOW for SHORT) reaches the target level. RISK is recorded when the relevant extreme (LOW for LONG, HIGH for SHORT) reaches the risk level. The first bar in the horizon window where a boundary is reached determines the outcome.

### SESSION_CLOSE denominator treatment

SESSION_CLOSE observations are included in Q1's denominator as non-target outcomes. They are not excluded. This prevents survivorship/availability selection bias.

---

## 8. Eligibility for Primary Comparison

An observation is eligible for the primary comparison (Q1) if all of the following hold:

- `direction` is LONG or SHORT (not NO_TRADE)
- `evidence_class` is Favourable or Mixed (Unfavourable excluded)
- `atr_14_intraday` is not null (minimum 14 completed 5-min bars available)
- Outcome is not AMBIGUOUS
- Evaluation timestamp is within the primary grid (09:20–15:25 IST)

---

## 9. Primary Endpoints and Statistical Analysis

### Primary tests

Three pre-specified primary-horizon tests:

| Test | Null hypothesis | Alternative |
|---|---|---|
| Q1-H15 | target_rate(Favourable, H15) <= target_rate(Mixed, H15) | target_rate(Favourable, H15) > target_rate(Mixed, H15) |
| Q1-H30 | target_rate(Favourable, H30) <= target_rate(Mixed, H30) | target_rate(Favourable, H30) > target_rate(Mixed, H30) |
| Q1-H60 | target_rate(Favourable, H60) <= target_rate(Mixed, H60) | target_rate(Favourable, H60) > target_rate(Mixed, H60) |

All tests are one-sided. Family-wise alpha is controlled at 0.05 using Bonferroni correction:

```
alpha_per_horizon = 0.05 / 3 = 0.016667
```

### Statistical test

**Pre-specified test:** one-sided two-proportion z-test comparing the Favourable vs Mixed target-attainment proportions for each horizon independently.

**Limitation acknowledgment (frozen):** The two-proportion z-test assumes independent Bernoulli observations. As documented in §11, INTRADAY-001 observations are not independent (within-instrument, within-session, overlapping horizons). The nominal p-values produced by the z-test are therefore observation-level and do not fully account for clustering. This limitation is acknowledged and frozen. The test is retained for its simplicity, pre-specifiability, and comparability with TIME-010. Results must be interpreted with this limitation in mind. The secondary analysis includes a session-date clustered sensitivity check (see §9 secondary metrics). This provides a partial correction for within-session dependence but does not fully account for within-instrument dependence across sessions.

### Experimental unit

The experimental unit is the **T0 observation** (instrument × evaluation_timestamp). The three horizon outcomes (H15, H30, H60) are correlated outcomes from the same T0 and are analyzed separately — they are not pooled.

### Secondary metrics (exploratory, not pre-specified for significance)

- Median MAE % (maximum adverse excursion before target or horizon)
- Median MFE % (maximum favourable excursion)
- Time-to-target distribution (minutes to target, for TARGET outcomes)
- Realized return at horizon expiry (for HORIZON and SESSION_CLOSE outcomes)
- Signal decay: target rate by evaluation time slot (09:20, 09:25, …)
- Spread/slippage sensitivity: simulated at 0.05%, 0.10%, 0.20%
- Session-date clustered sensitivity: target rate comparison using session-date as cluster unit (partial correction for within-session dependence; does not fully account for within-instrument dependence across sessions)

---

## 10. Conclusion Classification

The frozen conclusion classification for INTRADAY-001 uses the following deterministic algorithm:

**Step 1 — Per-horizon estimability (evaluated first, before any statistical test):**

```
For each horizon H in {H15, H30, H60}:
    if N_Favourable_eligible(H) < 100 OR N_Mixed_eligible(H) < 100:
        horizon H = NOT_ESTIMABLE  (excluded from conclusion)
    else:
        perform Q1 one-sided two-proportion z-test for horizon H
        classify as: significant_positive, significant_adverse, or not_significant
```

A "significant adverse" result means target_rate(Favourable) < target_rate(Mixed) at alpha=0.016667 one-sided (i.e. the z-test for the reversed hypothesis is significant).

**Step 2 — Global conclusion (applied to estimable horizons only, in precedence order):**

```
If no horizon is estimable:
    conclusion = NOT_ESTIMABLE

Else if any estimable horizon is significant_positive
     AND any estimable horizon is significant_adverse:
    conclusion = INCONCLUSIVE

Else if any estimable horizon is significant_positive
     AND cohort consistency criterion is met for that horizon:
    conclusion = POSITIVE

Else if any estimable horizon is significant_positive:
    conclusion = PARTIAL

Else:
    conclusion = NEGATIVE
```

**Cohort consistency criterion:** For a given horizon, the Favourable target rate exceeds the Mixed target rate in at least ceil(0.67 × N_session_dates) session dates with eligible observations in both classes.

**N >= 100 is not a stopping criterion.** The experiment runs to the pre-specified stopping condition (§12) regardless of whether N reaches 100. Estimability is evaluated at the stopping condition, not during the observation window.

---

## 11. Dependence Limitation (Explicit)

INTRADAY-001 generates multiple observations per instrument per session (up to 74). These observations are **not independent**:

- **Within-instrument dependence:** consecutive 5-minute evaluations of the same instrument share market state, price level, and trend.
- **Overlapping horizons:** H15, H30, and H60 outcomes from the same T0 are correlated.
- **Within-session dependence:** observations from the same session share the same market regime.

The primary analysis (Q1) uses a two-proportion z-test that assumes independent observations. The nominal p-values do not fully account for clustering. A large N should not be interpreted as equivalent to N independent trades. The secondary session-date clustered sensitivity check (§9) provides a partial correction for within-session dependence but does not fully account for within-instrument dependence across sessions.

---

## 12. Stopping Condition

```
min(60 session-days of observations, 12 calendar weeks from first observation date)
```

The stopping condition is evaluated on observation count and elapsed calendar time only — never on outcomes or target rates. N >= 100 per evidence class is not a stopping criterion.

---

## 13. Paper / Research Mode

**All INTRADAY-001 observations are PAPER / RESEARCH only.**

No real-money execution is implied or authorized. Recording an observation does not place an order.

---

## 14. Data Pipeline (to be implemented)

```
Market feed (1-minute OHLCV — primary outcome source)
      ↓
Intraday state builder
  (rolling Wilder ATR-14 from completed 5-min bars,
   TMV state, vol/volume regime)
      ↓
5-minute grid trigger (09:20, 09:25, …, 15:25 IST)
      ↓
Frozen Coralys evaluation
      ↓
INTRADAY-001 decision artifact (T0 sealed, immutable)
  outcome_data_source = OHLCV_1MIN (frozen)
      ↓
Outcome observer (intraday001_observe)
  per horizon: H15, H30, H60
  using 1-minute OHLCV bars only
      ↓
Outcome artifacts
      ↓
prospective_intraday_evidence.csv
      ↓
intraday001_analysis.py (frozen, pre-registered before stopping condition)
```

Tick data may be captured concurrently for audit purposes but must not be used as the primary outcome source.

The analysis script (`scripts/intraday001_analysis.py`) must be written and committed **before** the INTRADAY-001 stopping condition is reached.

---

## 15. Protocol Freeze

This document must be reviewed and committed before the first INTRADAY-001 observation artifact is generated. Once the first observation is written, the following are frozen and may not be changed:

- Research question (§2)
- Observation unit definition (§3)
- Evaluation schedule and grid (§4)
- Horizons (§5)
- ATR-14 definition (§6)
- Target/risk formulas (§6)
- Primary outcome data source: OHLCV_1MIN (§7)
- Outcome classification rules (§7)
- SESSION_CLOSE denominator treatment (§7)
- First-touch rule (§7)
- Eligibility criteria (§8)
- Primary tests Q1-H15/H30/H60 (§9)
- Statistical test: one-sided two-proportion z-test (§9)
- Bonferroni alpha = 0.016667 (§9)
- Conclusion classification and precedence rules (§10)
- Stopping condition (§12)

---

*INTRADAY-001 | Draft v4 | 2026-08-21*