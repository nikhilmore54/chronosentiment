# REC-BASELINE-001 — Coralys Recommendation MVP v1 Baseline

**Document ID:** REC-BASELINE-001
**Version:** 1.0
**Status:** FROZEN
**Created:** 2026-08-18
**Snapshot date:** 2026-08-18 (IST ~13:30)

---

## Purpose

This is the frozen operational baseline for Coralys RecommendationEngine v1.
It records the exact recommendation state produced by the frozen algorithm
against the 101-ticker NSE universe on 2026-08-18.

**Rule:** This document must never be modified after creation. Any subsequent
algorithm change must produce a new baseline (REC-BASELINE-002, etc.) and
must not overwrite this one.

---

## Algorithm Version

| Component | Version / Commit |
|-----------|-----------------|
| RecommendationEngine | v1 |
| Git commit | `eeb466705790d9189a7496f407457631c11d1902` |
| Branch | `governance-hardening` |
| Evidence store | REC-001-H (101 tickers, 121,805 records) |
| Universe | `datasets/universes/coralys_102_v2.json` (101 valid tickers) |
| Decision pipeline | C3-002 |
| Endpoint | `GET /recommendations/v1/latest` |
| Deduplication | Latest per ticker (newest `decision_timestamp` wins) |

---

## Algorithm Semantics (frozen)

- **Analogue selection:** Exact match on (coralys_state, direction, volatility_regime, volume_regime); degrades to RelaxVol → RelaxBoth → StateOnly → NO_TRADE
- **Adaptive target:** 25th-percentile MFE of analogue population (first-exit semantics)
- **Adaptive risk:** Median MAE of analogue population
- **Adaptive horizon:** Median sessions_to_outcome of analogue population
- **BUY policy:** R:R ≥ 1.0 AND target_rate ≥ 0.30 AND sample_size ≥ 20
- **WATCH policy:** R:R ≥ 0.75 AND target_rate ≥ 0.25 AND sample_size ≥ 20 (but not BUY)
- **NO_TRADE:** All other cases (including StateOnly degradation)
- **Rank score:** `target_rate × (1 + adaptive_rr) / 2`

---

## Aggregate Counts

| Metric | Value |
|--------|-------|
| Universe | 101 tickers |
| Evaluated | 101 |
| Actionable | 60 |
| BUY | 14 |
| WATCH | 46 |
| NO_TRADE | 41 |

---

## Distributions (actionable only, n=60)

| Metric | min | p25 | median | p75 | max | mean |
|--------|-----|-----|--------|-----|-----|------|
| rank_score | 0.3141 | 0.4422 | 0.4725 | 0.4936 | 0.5638 | 0.4574 |
| adaptive_rr | 0.77 | 0.95 | 1.05 | 1.17 | 1.60 | 1.07 |
| horizon_sessions | 3.0 | 3.0 | 3.5 | 4.0 | 4.0 | 3.5 |
| sample_size | 23 | 98 | 137 | 176 | 322 | 146.8 |
| target_rate | 0.302 | 0.325 | 0.349 | 0.396 | 0.560 | 0.365 |

---

## Degradation Distribution (actionable only)

| Level | Count |
|-------|-------|
| Exact | 43 |
| RelaxBoth | 12 |
| StateOnly | 5 |

---

## C3-002 State Distribution (all 101 tickers)

C3-002 state = `trend/momentum` composite.

| State | Total | BUY | WATCH | NO_TRADE |
|-------|-------|-----|-------|----------|
| Bearish/Negative | 38 | 4 | 17 | 17 |
| Bearish/Positive | 14 | 6 | 5 | 3 |
| Bullish/Negative | 12 | 0 | 6 | 6 |
| Bullish/Positive | 37 | 4 | 18 | 15 |

**Note:** `Bullish/Negative` produces no BUY decisions in this snapshot — all 12 are WATCH or NO_TRADE.

---

## Volatility / Volume Distribution (all 101 tickers)

| vol_regime | Count |
|------------|-------|
| present | 71 |
| absent | 30 |

| volume_regime | Count |
|---------------|-------|
| Normal | 101 |

All 101 tickers have `volume_regime=Normal` in this snapshot.

---

## Direction Distribution (all 101 tickers)

| Direction | Count |
|-----------|-------|
| LONG | 79 |
| SHORT | 22 |

---

## BUY — 14 Decisions (rank_score desc)

| Ticker | Ref | Target | Risk | R:R | Hor | n | Degradation | Rate | Score | Dir |
|--------|-----|--------|------|-----|-----|---|-------------|------|-------|-----|
| POLYCAB.NS | 9,272.00 | 9,701.36 | 8,894.65 | 1.14 | 3.0 | 38 | Exact | 0.500 | 0.5638 | LONG |
| AUROPHARMA.NS | 1,636.60 | 1,715.27 | 1,581.43 | 1.43 | 4.0 | 171 | Exact | 0.421 | 0.5531 | LONG |
| SAIL.NS | 170.80 | 178.52 | 163.27 | 1.03 | 4.0 | 65 | Exact | 0.477 | 0.5410 | LONG |
| VEDL.NS | 267.50 | 282.30 | 258.24 | 1.60 | 3.5 | 40 | Exact | 0.325 | 0.5224 | LONG |
| HINDALCO.NS | 1,044.05 | 1,086.09 | 1,004.43 | 1.06 | 3.0 | 248 | Exact | 0.427 | 0.5198 | LONG |
| PNB.NS | 117.29 | 122.40 | 112.31 | 1.03 | 3.0 | 175 | Exact | 0.423 | 0.5141 | LONG |
| LT.NS | 4,081.40 | 4,217.50 | 3,939.27 | 0.96 | 4.0 | 58 | Exact | 0.414 | 0.5027 | LONG |
| DLF.NS | 676.90 | 702.67 | 647.19 | 0.87 | 4.0 | 169 | Exact | 0.426 | 0.4998 | LONG |
| HDFCBANK.NS | 724.80 | 740.49 | 706.62 | 0.86 | 4.0 | 137 | Exact | 0.423 | 0.4980 | LONG |
| JINDALSTEL.NS | 1,108.20 | 1,158.93 | 1,064.83 | 1.17 | 4.0 | 125 | RelaxBoth | 0.560 | 0.4970 | LONG |
| BANKBARODA.NS | 245.17 | 256.12 | 232.20 | 0.84 | 3.5 | 96 | Exact | 0.417 | 0.4928 | LONG |
| ULTRACEMCO.NS | 11,572.00 | 11,967.06 | 11,196.88 | 1.05 | 4.0 | 266 | RelaxBoth | 0.474 | 0.4422 | LONG |
| ICICIPRULI.NS | 502.60 | 524.16 | 480.44 | 0.97 | 3.0 | 170 | RelaxBoth | 0.459 | 0.4267 | LONG |
| MAHABANK.NS | 81.00 | 85.33 | 76.96 | 1.07 | 4.0 | 286 | RelaxBoth | 0.406 | 0.4100 | LONG |

---

## WATCH — 46 Decisions (rank_score desc)

| Ticker | Ref | Target | Risk | R:R | Hor | n | Degradation | Rate | Score | Dir |
|--------|-----|--------|------|-----|-----|---|-------------|------|-------|-----|
| PERSISTENT.NS | 5,490.00 | 5,770.11 | 5,274.81 | 1.30 | 3.0 | 169 | Exact | 0.373 | 0.5166 | LONG |
| TMCV.NS | 467.00 | 486.32 | 442.01 | 0.77 | 3.0 | 23 | Exact | 0.478 | 0.5165 | LONG |
| RECLTD.NS | 339.85 | 318.10 | 354.83 | 1.45 | 3.0 | 61 | Exact | 0.328 | 0.5091 | SHORT |
| ASIANPAINT.NS | 2,630.00 | 2,552.60 | 2,695.20 | 1.19 | 4.0 | 51 | Exact | 0.353 | 0.4952 | SHORT |
| GODREJPROP.NS | 2,025.00 | 1,907.50 | 2,109.10 | 1.40 | 4.0 | 65 | Exact | 0.308 | 0.4936 | SHORT |
| JUBLFOOD.NS | 504.65 | 528.65 | 485.47 | 1.25 | 3.0 | 98 | Exact | 0.337 | 0.4935 | LONG |
| AXISBANK.NS | 1,238.00 | 1,283.02 | 1,195.43 | 1.06 | 4.0 | 111 | Exact | 0.369 | 0.4905 | LONG |
| IDFCFIRSTB.NS | 85.15 | 88.24 | 82.25 | 1.06 | 4.0 | 161 | Exact | 0.366 | 0.4896 | LONG |
| UPL.NS | 564.75 | 583.90 | 543.24 | 0.89 | 4.0 | 117 | Exact | 0.393 | 0.4856 | LONG |
| KOTAKBANK.NS | 390.95 | 403.08 | 380.88 | 1.20 | 4.0 | 88 | Exact | 0.330 | 0.4852 | LONG |
| TRENT.NS | 2,974.10 | 3,101.34 | 2,823.56 | 0.85 | 3.5 | 48 | Exact | 0.396 | 0.4824 | LONG |
| HAVELLS.NS | 1,294.50 | 1,343.96 | 1,246.04 | 1.02 | 3.0 | 147 | Exact | 0.361 | 0.4823 | LONG |
| PIDILITIND.NS | 1,664.70 | 1,719.06 | 1,619.27 | 1.20 | 3.0 | 118 | Exact | 0.322 | 0.4807 | LONG |
| TVSMOTOR.NS | 4,364.90 | 4,540.27 | 4,209.15 | 1.13 | 3.0 | 224 | Exact | 0.335 | 0.4800 | LONG |
| TATASTEEL.NS | 184.74 | 190.92 | 177.22 | 0.82 | 4.0 | 134 | Exact | 0.396 | 0.4800 | LONG |
| IGL.NS | 150.66 | 156.29 | 144.62 | 0.93 | 4.0 | 113 | Exact | 0.372 | 0.4790 | LONG |
| HINDUNILVR.NS | 2,039.30 | 2,091.38 | 1,987.62 | 1.01 | 4.0 | 167 | Exact | 0.353 | 0.4774 | LONG |
| BHARTIARTL.NS | 1,946.60 | 2,002.82 | 1,889.25 | 0.98 | 4.0 | 257 | Exact | 0.354 | 0.4751 | LONG |
| NMDC.NS | 84.66 | 88.41 | 80.98 | 1.02 | 4.0 | 126 | Exact | 0.341 | 0.4725 | LONG |
| DRREDDY.NS | 1,186.30 | 1,225.76 | 1,144.72 | 0.95 | 3.0 | 124 | Exact | 0.355 | 0.4723 | LONG |
| IDEA.NS | 13.94 | 14.82 | 13.17 | 1.14 | 4.0 | 58 | Exact | 0.310 | 0.4695 | LONG |
| INFY.NS | 1,115.60 | 1,147.79 | 1,081.50 | 0.94 | 3.0 | 189 | Exact | 0.349 | 0.4690 | LONG |
| VBL.NS | 438.15 | 455.39 | 421.56 | 1.04 | 4.0 | 70 | Exact | 0.329 | 0.4682 | LONG |
| DABUR.NS | 402.50 | 414.21 | 391.29 | 1.04 | 4.0 | 143 | Exact | 0.322 | 0.4653 | LONG |
| SRF.NS | 2,630.00 | 2,748.83 | 2,520.57 | 1.09 | 3.0 | 129 | Exact | 0.310 | 0.4636 | LONG |
| WIPRO.NS | 179.30 | 184.98 | 173.17 | 0.93 | 3.0 | 156 | Exact | 0.340 | 0.4626 | LONG |
| ITC.NS | 270.80 | 276.30 | 263.94 | 0.80 | 3.0 | 144 | Exact | 0.361 | 0.4607 | LONG |
| MARUTI.NS | 13,789.00 | 14,241.69 | 13,375.76 | 1.10 | 3.0 | 189 | Exact | 0.302 | 0.4603 | LONG |
| INDHOTEL.NS | 716.85 | 682.36 | 748.81 | 1.08 | 3.0 | 69 | Exact | 0.304 | 0.4601 | SHORT |
| BAJAJFINSV.NS | 2,010.30 | 2,081.72 | 1,929.74 | 0.89 | 3.0 | 134 | Exact | 0.336 | 0.4566 | LONG |
| SUNPHARMA.NS | 1,874.60 | 1,819.23 | 1,929.82 | 1.00 | 4.0 | 75 | Exact | 0.307 | 0.4536 | SHORT |
| SBIN.NS | 1,056.60 | 1,087.52 | 1,019.10 | 0.82 | 3.0 | 235 | Exact | 0.332 | 0.4484 | LONG |
| HDFCLIFE.NS | 540.35 | 556.66 | 521.36 | 0.86 | 4.0 | 155 | Exact | 0.323 | 0.4472 | LONG |
| BERGEPAINT.NS | 544.40 | 568.43 | 527.38 | 1.41 | 4.0 | 124 | RelaxBoth | 0.331 | 0.4065 | LONG |
| INDUSINDBK.NS | 1,015.80 | 968.29 | 1,050.71 | 1.36 | 3.0 | 164 | RelaxBoth | 0.323 | 0.3977 | SHORT |
| BRITANNIA.NS | 5,538.50 | 5,750.21 | 5,372.13 | 1.27 | 3.0 | 143 | RelaxBoth | 0.322 | 0.3881 | LONG |
| SHREECEM.NS | 24,635.00 | 25,541.93 | 23,806.27 | 1.09 | 4.0 | 310 | RelaxBoth | 0.345 | 0.3820 | LONG |
| HCLTECH.NS | 1,296.60 | 1,240.16 | 1,337.53 | 1.38 | 3.0 | 130 | StateOnly | 0.377 | 0.3764 | SHORT |
| POWERGRID.NS | 267.85 | 276.56 | 259.97 | 1.11 | 3.0 | 227 | RelaxBoth | 0.330 | 0.3758 | LONG |
| SBILIFE.NS | 1,785.70 | 1,848.12 | 1,725.98 | 1.05 | 4.0 | 262 | RelaxBoth | 0.309 | 0.3591 | LONG |
| NAUKRI.NS | 1,364.20 | 1,279.13 | 1,422.29 | 1.46 | 3.0 | 127 | StateOnly | 0.323 | 0.3579 | SHORT |
| GODREJCP.NS | 928.50 | 960.50 | 897.43 | 1.03 | 4.0 | 320 | RelaxBoth | 0.309 | 0.3577 | LONG |
| MRF.NS | 133,555.00 | 137,145.52 | 129,073.28 | 0.80 | 3.0 | 322 | RelaxBoth | 0.354 | 0.3571 | LONG |
| HEROMOTOCO.NS | 5,724.50 | 5,470.00 | 5,943.61 | 1.16 | 3.0 | 129 | StateOnly | 0.326 | 0.3289 | SHORT |
| CHOLAFIN.NS | 1,881.00 | 1,788.83 | 1,972.12 | 1.01 | 3.0 | 181 | StateOnly | 0.337 | 0.3197 | SHORT |
| TITAN.NS | 5,057.60 | 4,878.61 | 5,232.88 | 1.02 | 3.0 | 176 | StateOnly | 0.324 | 0.3141 | SHORT |

---

## NO_TRADE — 41 Tickers

| Ticker | Ref | Score | Dir | Degradation |
|--------|-----|-------|-----|-------------|
| CANBK.NS | 129.59 | 0.4738 | LONG | Exact |
| GRASIM.NS | 3,279.70 | 0.4734 | LONG | Exact |
| TATACONSUM.NS | 1,067.70 | 0.4725 | LONG | Exact |
| LICI.NS | 407.50 | 0.4702 | LONG | Exact |
| COALINDIA.NS | 408.45 | 0.4658 | LONG | Exact |
| M&M.NS | 3,427.20 | 0.4635 | LONG | Exact |
| TCS.NS | 2,287.00 | 0.4615 | LONG | Exact |
| AMBUJACEM.NS | 413.30 | 0.4591 | SHORT | Exact |
| LUPIN.NS | 2,228.00 | 0.4587 | LONG | Exact |
| BAJFINANCE.NS | 1,098.30 | 0.4544 | LONG | Exact |
| CIPLA.NS | 1,434.30 | 0.4518 | LONG | Exact |
| ADANIGREEN.NS | 1,322.50 | 0.4469 | LONG | Exact |
| TECHM.NS | 1,582.80 | 0.4441 | LONG | Exact |
| ONGC.NS | 239.59 | 0.4364 | LONG | Exact |
| JSWSTEEL.NS | 1,276.00 | 0.4334 | LONG | Exact |
| APOLLOHOSP.NS | 8,827.00 | 0.4333 | SHORT | Exact |
| GAIL.NS | 172.51 | 0.4323 | SHORT | Exact |
| IRCTC.NS | 495.60 | 0.4286 | LONG | Exact |
| SIEMENS.NS | 3,953.70 | 0.4258 | LONG | Exact |
| ICICIBANK.NS | 1,407.50 | 0.4257 | SHORT | Exact |
| IOC.NS | 138.60 | 0.4248 | LONG | Exact |
| ADANIENT.NS | 3,026.80 | 0.4226 | LONG | Exact |
| NESTLEIND.NS | 1,467.00 | 0.4197 | LONG | Exact |
| MARICO.NS | 847.80 | 0.4140 | SHORT | Exact |
| EICHERMOT.NS | 8,087.50 | 0.4134 | LONG | Exact |
| MUTHOOTFIN.NS | 2,909.50 | 0.4079 | LONG | Exact |
| RELIANCE.NS | 1,324.40 | 0.3962 | LONG | Exact |
| NTPC.NS | 338.30 | 0.3932 | LONG | Exact |
| INDUSTOWER.NS | 372.85 | 0.3658 | LONG | RelaxBoth |
| COLPAL.NS | 1,915.50 | 0.3641 | LONG | RelaxBoth |
| BANDHANBNK.NS | 171.67 | 0.3579 | LONG | RelaxBoth |
| PAGEIND.NS | 36,785.00 | 0.3365 | LONG | RelaxBoth |
| BPCL.NS | 313.35 | 0.3333 | LONG | RelaxBoth |
| PETRONET.NS | 286.65 | 0.3329 | LONG | RelaxBoth |
| ADANIPORTS.NS | 1,681.00 | 0.3324 | LONG | RelaxBoth |
| MPHASIS.NS | 2,469.70 | 0.3294 | SHORT | StateOnly |
| OFSS.NS | 11,747.00 | 0.3158 | SHORT | StateOnly |
| TORNTPHARM.NS | 4,982.50 | 0.3139 | SHORT | RelaxBoth |
| BAJAJ-AUTO.NS | 11,650.00 | 0.3014 | SHORT | StateOnly |
| DIVISLAB.NS | 8,575.00 | 0.2838 | SHORT | StateOnly |
| BOSCHLTD.NS | 48,345.00 | 0.2822 | SHORT | StateOnly |

---

## Observation Schema (next milestone)

When prospective observation is implemented, each BUY/WATCH recommendation
from this baseline will be tracked with the following fields:

```
recommendation_id     (decision_id from sealed ledger)
ticker
decision_timestamp    (T0 — immutable)
reference_price       (T0 — immutable)
coralys_state         (T0 — immutable)
direction             (T0 — immutable)
action                (BUY / WATCH — immutable)
adaptive_target       (T0 — immutable)
adaptive_risk         (T0 — immutable)
adaptive_rr           (T0 — immutable)
adaptive_horizon_sessions (T0 — immutable)
analogue_count        (T0 — immutable)
degradation_level     (T0 — immutable)
target_rate           (T0 — immutable)
rank_score            (T0 — immutable)

observation_status    OPEN | CLOSED | INCOMPLETE

actual_mfe            (T+h — appended, never overwrites T0)
actual_mae            (T+h — appended, never overwrites T0)
target_reached        (T+h — appended)
risk_reached          (T+h — appended)
first_exit            TARGET | RISK | HORIZON | NONE
sessions_to_outcome   (T+h — appended)
outcome               TARGET_BEFORE_RISK | RISK_BEFORE_TARGET | HORIZON_EXPIRED | OPEN
```

**Critical rule:** The T0 snapshot fields are immutable. Outcome fields are
appended only. The T0 recommendation must never be retroactively changed.

---

## SHORT Population Analysis (2026-08-18)

**Total SHORT decisions: 22** (of 101 tickers = 21.8%)

### Evidence Class Distribution

| Evidence Class | Count | % of SHORTs | Action Assigned |
|----------------|-------|-------------|-----------------|
| Favourable     | 0     | 0%          | —               |
| Mixed          | 11    | 50%         | Watch           |
| Unfavourable   | 11    | 50%         | NoTrade         |

**Key finding: 0 Favourable SHORTs exist in this baseline.** Under a symmetric policy (`SHORT + Favourable → SELL`), zero SELL actions would have been emitted. The SHORT/SELL gap is not a missed opportunity at this snapshot — it is a structural consequence of the evidence quality of the SHORT population.

### R:R and Target Rate by Evidence Class

| Evidence Class | n  | RR min | RR max | RR avg | TargetRate avg |
|----------------|----|--------|--------|--------|----------------|
| Favourable     | 0  | —      | —      | —      | —              |
| Mixed          | 11 | 1.00   | 1.46   | 1.23   | 0.328          |
| Unfavourable   | 11 | 0.88   | 1.51   | 1.21   | 0.238          |

Mixed SHORTs have meaningfully higher target rates (0.328 vs 0.238) despite similar R:R profiles. The Unfavourable cohort has one outlier at RR=1.51 (MPHASIS_NS) but a target_rate of only 0.256 — insufficient for Favourable classification.

### Degradation Level Distribution (SHORTs)

| Degradation Level | Count |
|-------------------|-------|
| Exact             | 10    |
| StateOnly         | 10    |
| RelaxBoth         | 2     |

### C3-002 State of SHORT Population

All 22 SHORT tickers have `trend = Bullish`. This is structurally expected: SHORT decisions arise from bearish price action (negative momentum) within a broader bullish trend context — the system is identifying counter-trend exhaustion setups, not trend-following shorts.

| Trend   | Momentum | Count |
|---------|----------|-------|
| Bullish | Negative | 11    |
| Bullish | Positive | 11    |

### Per-Ticker SHORT Table

| Ticker         | EvidClass    | Action  |   RR | TargRate | DegLevel  | Trend   | Momentum |
|----------------|--------------|---------|-----:|----------|-----------|---------|----------|
| ASIANPAINT_NS  | Mixed        | Watch   | 1.19 | 0.353    | Exact     | Bullish | Negative |
| CHOLAFIN_NS    | Mixed        | Watch   | 1.01 | 0.337    | StateOnly | Bullish | Positive |
| GODREJPROP_NS  | Mixed        | Watch   | 1.40 | 0.308    | Exact     | Bullish | Negative |
| HCLTECH_NS     | Mixed        | Watch   | 1.38 | 0.377    | StateOnly | Bullish | Positive |
| HEROMOTOCO_NS  | Mixed        | Watch   | 1.16 | 0.326    | StateOnly | Bullish | Positive |
| INDHOTEL_NS    | Mixed        | Watch   | 1.08 | 0.304    | Exact     | Bullish | Negative |
| INDUSINDBK_NS  | Mixed        | Watch   | 1.36 | 0.323    | RelaxBoth | Bullish | Negative |
| NAUKRI_NS      | Mixed        | Watch   | 1.46 | 0.323    | StateOnly | Bullish | Positive |
| RECLTD_NS      | Mixed        | Watch   | 1.45 | 0.328    | Exact     | Bullish | Negative |
| SUNPHARMA_NS   | Mixed        | Watch   | 1.00 | 0.307    | Exact     | Bullish | Negative |
| TITAN_NS       | Mixed        | Watch   | 1.02 | 0.324    | StateOnly | Bullish | Positive |
| AMBUJACEM_NS   | Unfavourable | NoTrade | 1.24 | 0.271    | Exact     | Bullish | Negative |
| APOLLOHOSP_NS  | Unfavourable | NoTrade | 1.02 | 0.263    | Exact     | Bullish | Negative |
| BAJAJ-AUTO_NS  | Unfavourable | NoTrade | 1.31 | 0.241    | StateOnly | Bullish | Positive |
| BOSCHLTD_NS    | Unfavourable | NoTrade | 0.88 | 0.288    | StateOnly | Bullish | Positive |
| DIVISLAB_NS    | Unfavourable | NoTrade | 1.15 | 0.237    | StateOnly | Bullish | Positive |
| GAIL_NS        | Unfavourable | NoTrade | 1.45 | 0.174    | Exact     | Bullish | Negative |
| ICICIBANK_NS   | Unfavourable | NoTrade | 1.08 | 0.236    | Exact     | Bullish | Negative |
| MARICO_NS      | Unfavourable | NoTrade | 1.34 | 0.160    | Exact     | Bullish | Negative |
| MPHASIS_NS     | Unfavourable | NoTrade | 1.51 | 0.256    | StateOnly | Bullish | Positive |
| OFSS_NS        | Unfavourable | NoTrade | 1.19 | 0.294    | StateOnly | Bullish | Positive |
| TORNTPHARM_NS  | Unfavourable | NoTrade | 1.16 | 0.195    | RelaxBoth | Bullish | Negative |

### Policy Implication: SHORT → SELL Gap

Current `derive_action_v1()` mapping:
- `SHORT + Favourable` → **no branch exists** (would need to be added for SELL)
- `SHORT + Mixed` → Watch
- `SHORT + Unfavourable` → NoTrade

At this baseline snapshot, adding a `SHORT + Favourable → SELL` branch would produce **0 SELL actions** because no Favourable SHORTs exist. The gap is real but currently dormant. The decision to add the branch is a policy choice for future baselines, not a correction to this one.

---

## Archived Baselines

| Baseline | Date | Status | Notes |
|----------|------|--------|-------|
| REC-BASELINE-001 (this document) | 2026-08-18 | FROZEN | First operational baseline; Coralys v1; 101 tickers |

---

## Governance

- This document is canonical. Do not modify after creation.
- Next baseline: `REC-BASELINE-002` — to be created only after a deliberate algorithm change.
- Comparison methodology: B1 vs B2 requires same universe, same evidence boundary, same evaluation period.
- Source JSON: `/tmp/rec_latest.json` (ephemeral — not committed; regenerate from server if needed)
- Generator script: `scripts/gen_rec_baseline.py`