# HDV-001-G Freeze Gate

**Frozen:** 2026-08-17
**Branch:** governance-hardening
**Commit at freeze:** f6a321976
**Frozen by:** HDV-001 programme

---

## Purpose

This document is the pre-analysis freeze gate for HDV-001 development-period
evidence. It must be completed and committed before any interpretation of
HDV-001-D/E results is used to modify C3-002, Coralys execution policy,
reference-risk boundaries, or stop-loss parameters.

Once this gate is signed, the development-period evidence is frozen.
Any subsequent changes to C3-002 or execution policy require a new HDV
programme (HDV-002 or later).

---

## Gate 1: Data Integrity

| Check | Expected | Actual | Status |
|-------|----------|--------|--------|
| Instruments in cache | 52 | 52 | PASS |
| Cache hash | 8e07ea77a474c1beb21835474f458c4b07bffb406641fa11f382303877e18112 | 8e07ea77a474c1beb21835474f458c4b07bffb406641fa11f382303877e18112 | PASS |
| Duplicate bars | 0 | 0 | PASS |
| NaN/null OHLCV rows | 0 | 0 | PASS |
| NSE calendar sessions | 23 | 23 | PASS |
| NSE holiday 2026-08-15 | excluded | excluded | PASS |
| Unexplained weekday gaps | 0 | 0 | PASS |
| Corporate action checks | 5/5 | 5/5 | PASS |
| Stale universe files | 0 | 0 (4 deleted) | PASS |

**Gate 1: PASS**

---

## Gate 2: Temporal Integrity

| Check | Rule | Verified | Status |
|-------|------|----------|--------|
| Decision-day bar exclusion | bar_date > decision_date_IST | Spot-checked: ADANIENT 2026-07-17 decision, first bar 2026-07-20 | PASS |
| Weekend skip | 2026-07-17 decision skips Sat/Sun | First bar 2026-07-20 (Monday) | PASS |
| Same-day decision | 2026-08-13 decision, first bar 2026-08-14 | Confirmed in extractor output | PASS |
| MATURING classification | sessions_available < 10 | 416 decisions correctly marked MATURING | PASS |
| COMPLETE classification | sessions_available >= 10 | 728 decisions correctly marked COMPLETE | PASS |
| No future data leakage | cache built 2026-08-17, no post-cache bars | Cache REQUIRED_END = 2026-08-13; fetch window ends 2026-08-17 | PASS |

**Gate 2: PASS**

---

## Gate 3: Metric Integrity

| Check | Rule | Verified | Status |
|-------|------|----------|--------|
| LONG MFE direction | positive = price rose above reference | mult=+1, MFE = max(close-ref)/ref | PASS |
| SHORT MFE direction | positive = price fell below reference | mult=-1, MFE = max(ref-close)/ref | PASS |
| MAE sign convention | negative = adverse excursion | min(returns) can be negative | PASS |
| Checkpoint coverage | sessions 1,2,3,5,10 | All 5 checkpoints present in output | PASS |
| time_to_target | first session close crosses target | LONG: close >= target; SHORT: close <= target | PASS |
| time_to_stop | first session close crosses stop adversely | LONG: close <= stop; SHORT: close >= stop | PASS |
| No execution mechanics | B/C allocation, lot size, slippage excluded | Metrics derived solely from price paths and decision metadata | PASS |

**Gate 3: PASS**

---

## Gate 4: Outcome Integrity

| Check | Rule | Verified | Status |
|-------|------|----------|--------|
| TARGET_BEFORE_RISK ordering | time_to_target <= time_to_stop | Code: if time_target <= time_stop -> TARGET_BEFORE_RISK | PASS |
| RISK_BEFORE_TARGET ordering | time_to_stop < time_to_target | Code: else -> RISK_BEFORE_TARGET | PASS |
| HORIZON classification | COMPLETE, neither boundary hit | Both time_to_target and time_to_stop are None | PASS |
| MATURING not classified as HORIZON | obs_status == MATURING -> MATURING | Checked in classifier before HORIZON branch | PASS |
| Same-session ambiguity documented | target takes precedence | Documented below as methodological assumption | PASS |
| No B/C execution leakage | outcome derived from price path only | Classifier reads only metrics file, not B/C fields | PASS |

### Methodological assumption: same-session target/risk ambiguity

When both target and reference-risk boundaries are crossed within the same
daily OHLC bar, the sequence within that session is unobservable from daily
data. HDV-001 resolves this as TARGET_BEFORE_RISK (target takes precedence).

This is a conservative assumption in favor of Coralys. It should be
documented as a limitation and revisited in HDV-002 if intraday data becomes
available.

**Gate 4: PASS**

---

## Gate 5: Reproducibility

| Artifact | Hash / Version | Status |
|----------|---------------|--------|
| Source dataset | datasets/stop_research_dataset_v01.json | Present |
| Price cache hash | 8e07ea77a474c1beb21835474f458c4b07bffb406641fa11f382303877e18112 | Recorded in cache_hash.txt |
| Price paths | datasets/hdv001/hdv001_price_paths_v1.json | Committed 588e1fe7f |
| Decision metrics | datasets/hdv001/hdv001_decision_metrics_v1.json | Committed f6a321976 |
| Outcome classification | datasets/hdv001/hdv001_outcomes_v1.json | Committed f6a321976 |
| Period definitions | docs/HDV_001_PERIODS.md | Committed 588e1fe7f |
| Methodology | docs/HDV_001_METHODOLOGY.md | Committed 5cd96db21 |
| Build scripts | scripts/hdv001_build_price_cache.py | Committed 588e1fe7f |
| Integrity verifier | scripts/hdv001_verify_cache_integrity.py | Committed 588e1fe7f |
| Path extractor | scripts/hdv001_extract_price_paths.py | Committed 588e1fe7f |
| Metrics calculator | scripts/hdv001_compute_metrics.py | Committed f6a321976 |
| Outcome classifier | scripts/hdv001_classify_outcomes.py | Committed f6a321976 |

**Gate 5: PASS**

---

## Gate 6: Baseline Readiness

The following baselines are specified here, before examining their results,
as required by HDV-001 methodology.

### Baseline A: Random direction

For every decision in the COMPLETE set (N=728):
- Preserve: instrument, decision_timestamp, reference_price, target_price,
  stop_price, price_path
- Replace: direction with random LONG/SHORT (seed=42 for reproducibility)
- Evaluate: same TARGET/RISK/HORIZON classification

### Baseline B: Inverse Coralys direction

For every decision in the COMPLETE set:
- Preserve: all fields
- Replace: direction with the opposite of Coralys direction
- Evaluate: same TARGET/RISK/HORIZON classification

### Baseline C: Simple momentum rule

For every decision:
- Use only information available before decision_date_IST
- Rule: if prior-session close > 20-session moving average -> LONG, else SHORT
- Evaluate: same TARGET/RISK/HORIZON classification

### Success criterion (pre-specified)

Coralys demonstrates a directional edge if:
- TARGET_BEFORE_RISK rate for Coralys > Baseline A by >= 5 percentage points
- TARGET_BEFORE_RISK rate for Coralys > Baseline B (inverse) by >= 5 pp
- The difference is consistent across at least 2 of the 4 Coralys state segments

This criterion is frozen here. It must not be changed after baselines are run.

**Gate 6: PASS — baselines specified before results examined**

---

## Development Evidence Summary (for reference only — do not optimize against)

Primary sample: 728 COMPLETE decisions (2026-07-14 to 2026-08-13)

### MAE/MFE (direction-normalized)

| Session | Median MFE | Median MAE | % MFE > 0 |
|---------|-----------|-----------|-----------|
| 1 | +0.138% | +0.138% | 55.2% |
| 2 | +0.658% | -0.189% | 70.5% |
| 3 | +1.037% | -0.434% | 77.2% |
| 5 | +1.723% | -0.774% | 82.5% |
| 10 | +2.803% | -1.270% | 88.5% |

### Outcome rates

| Outcome | Count | Rate |
|---------|-------|------|
| TARGET_BEFORE_RISK | 260 | 35.7% |
| RISK_BEFORE_TARGET | 302 | 41.5% |
| HORIZON | 166 | 22.8% |

### Segmentation by Coralys state

| State | N | TARGET | RISK | HORIZON |
|-------|---|--------|------|---------|
| Bullish_Positive | 299 | 33.1% | 26.8% | 40.1% |
| Bullish_Negative | 113 | 22.1% | 57.5% | 20.3% |
| Bearish_Positive | 95 | 44.2% | 46.3% | 9.5% |
| Bearish_Negative | 221 | 42.5% | 51.1% | 6.3% |

---

## Governance Constraints

The following constraints are in force from this point:

1. **C3-002 must not be modified** based on HDV-001-D/E findings.
2. **Reference-risk boundaries must not be changed** based on HDV-001-D/E findings.
3. **Stop-loss research must not resume** until HDV-001-F baselines are complete
   and the success criterion above is evaluated.
4. **The 416 MATURING decisions** must not be reclassified until their
   10-session observation windows complete.
5. **HDV-001 development-period evidence is now frozen.** Any re-run of the
   pipeline must produce identical results (verified by artifact hashes).
6. **Interpretation of results** must be performed against the pre-specified
   success criterion in Gate 6, not against post-hoc criteria.

---

## Next Milestone

**HDV-001-F: Independent Baseline and Economic Evidence**

Run Baselines A, B, C as specified in Gate 6.
Compute excursion statistics by outcome category and Coralys state.
Compare against the pre-specified success criterion.

Do not proceed to stop-loss research until HDV-001-F is complete.

---

## Freeze Declaration

All six gates pass. The HDV-001 development-period evidence is hereby frozen.

**HDV-001-G: FROZEN — 2026-08-17**