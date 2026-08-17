# HDV-001-E Outcome Classification Report

**Generated:** 2026-08-17
**Source:** `datasets/hdv001/hdv001_decision_metrics_v1.json`
**Output:** `datasets/hdv001/hdv001_outcomes_v1.json`

## Classification Rules

Applied in order:
1. TARGET_BEFORE_RISK -- target hit before or at same session as stop
2. RISK_BEFORE_TARGET -- stop hit before target
3. HORIZON            -- neither hit within 10 sessions (COMPLETE only)
4. MATURING           -- observation window not yet complete

## Outcome Counts

| Outcome | Count |
|---------|-------|
| TARGET_BEFORE_RISK | 260 |
| RISK_BEFORE_TARGET | 302 |
| HORIZON | 166 |
| MATURING | 416 |
| NO_SESSIONS | 0 |

## Outcome Rates (COMPLETE decisions, N=728)

| Outcome | Count | Rate |
|---------|-------|------|
| TARGET_BEFORE_RISK | 260 | 35.7% |
| RISK_BEFORE_TARGET | 302 | 41.5% |
| HORIZON | 166 | 22.8% |

## Segmentation by Direction

| Direction | N | TARGET | RISK | HORIZON |
|-----------|---|--------|------|---------|
| LONG | 615 | 38.2% | 38.5% | 23.2% |
| SHORT | 113 | 22.1% | 57.5% | 20.3% |

## Segmentation by Coralys Trend

| Trend | N | TARGET | RISK | HORIZON |
|-------|---|--------|------|---------|
| Bullish | 412 | 30.1% | 35.2% | 34.7% |
| Bearish | 316 | 43.0% | 49.7% | 7.3% |

## Segmentation by Trend + Momentum

| Trend + Momentum | N | TARGET | RISK | HORIZON |
|------------------|---|--------|------|---------|
| Bullish_Positive | 299 | 33.1% | 26.8% | 40.1% |
| Bullish_Negative | 113 | 22.1% | 57.5% | 20.3% |
| Bearish_Positive | 95 | 44.2% | 46.3% | 9.5% |
| Bearish_Negative | 221 | 42.5% | 51.1% | 6.3% |

## Governance Note

Do not modify C3-002 based on these findings.
HDV-001-G freeze gate must be passed before any implementation changes.
Primary analysis uses COMPLETE decisions only.
MATURING decisions will be reclassified when their observation windows complete.