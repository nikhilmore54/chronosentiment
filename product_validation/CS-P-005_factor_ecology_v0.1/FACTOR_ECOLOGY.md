# CS-P-005 Factor Ecology Analysis v0.1

Information-state discovery. **Not a trading recommendation.** Not B5. Not G-GATE. Not Decision Engine v1.0.

Rows: 195

## Availability

- Momentum=AVAILABLE: 195
- Trend=AVAILABLE: 195
- Volatility=AVAILABLE: 195

## Current TrendMappingPolicy actions (descriptive, not a candidate)

- LONG: 110
- SHORT: 85

## Trend × Momentum

| State | n |
|---|---:|
| Bearish×Negative | 57 |
| Bearish×Positive | 28 |
| Bullish×Negative | 32 |
| Bullish×Positive | 78 |

## Trend × Momentum × Volatility

| State | n |
|---|---:|
| Trend=Bearish | Momentum=Negative | Vol=AVAILABLE | 57 |
| Trend=Bearish | Momentum=Positive | Vol=AVAILABLE | 28 |
| Trend=Bullish | Momentum=Negative | Vol=AVAILABLE | 32 |
| Trend=Bullish | Momentum=Positive | Vol=AVAILABLE | 78 |

## roc_20 distribution (bars ≤ T; not a threshold)

Quantiles {
    n: 195,
    min: Some(
        -17.66852573292488,
    ),
    p25: Some(
        -3.6620051892754977,
    ),
    median: Some(
        0.6748130078134779,
    ),
    p75: Some(
        4.829860497614232,
    ),
    max: Some(
        16.294911758501886,
    ),
}

## atr_14 distribution (bars ≤ T; magnitude only; not High/Low)

Quantiles {
    n: 195,
    min: Some(
        21.152326311383927,
    ),
    p25: Some(
        35.41795131138393,
    ),
    median: Some(
        44.52409144810268,
    ),
    p75: Some(
        183.09666224888392,
    ),
    max: Some(
        533.8490687779018,
    ),
}

## 60D outcomes attached as measurement only

available=110 unavailable=85

These numbers must not be used to pick X/Y cutoffs.

## Design constraints for a later candidate

- Specify the candidate policy before evaluating it. Do not search thresholds on these outcomes.
- TrendMappingPolicy remains the live default until a candidate is frozen as a new version.
- NO_TRADE must be an explicit confluence miss, not an accident of missing Trend.
- Volatility may be used only as a magnitude available at T (atr_14). Do not invent High/Low.
- atr_14 is in price units and is not comparable across instruments; do not use a global ATR cutoff.
- Outcomes in this report are measurement, not decision inputs.
- Do not freeze Decision Engine v1.0 from this analysis.
- Observed distinct Trend×Momentum×Vol states: 4.
- Current TrendMappingPolicy NO_TRADE count on this snapshot: 0/195 (descriptive).
