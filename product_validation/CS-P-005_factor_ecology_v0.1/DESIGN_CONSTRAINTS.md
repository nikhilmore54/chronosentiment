# Candidate-policy design constraints (not a policy)

- Specify the candidate policy before evaluating it. Do not search thresholds on these outcomes.
- TrendMappingPolicy remains the live default until a candidate is frozen as a new version.
- NO_TRADE must be an explicit confluence miss, not an accident of missing Trend.
- Volatility may be used only as a magnitude available at T (atr_14). Do not invent High/Low.
- atr_14 is in price units and is not comparable across instruments; do not use a global ATR cutoff.
- Outcomes in this report are measurement, not decision inputs.
- Do not freeze Decision Engine v1.0 from this analysis.
- Observed distinct Trend×Momentum×Vol states: 4.
- Current TrendMappingPolicy NO_TRADE count on this snapshot: 0/195 (descriptive).

Do not search thresholds against the attached 60D outcomes.
