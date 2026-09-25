# Stage F-MOGA v0.1 Results

Evolutionary search for a compact state-conditioned description of the `Persistent_Build` phenotype.

## Discovered Candidate Rule
```text
((volume_acceleration_60m >= 0.7304) & (volume_acceleration_60m <= 0.8435)) & (persistence_240m > 0.1581)
```

## Performance Matrix
| Metric | Discovery | Validation |
|---|---|---|
| Candidate support N | 164 | 66 |
| Candidate coverage % | 0.81% | 0.79% |
| PB prevalence (Baseline) | 10.42% | 10.50% |
| P(PB \| C) | 27.44% | 18.18% |
| Enrichment | 2.63x | 1.73x |
| Block enrichment median | 2.79x | N/A |
| Block sign/stability | PASS | N/A |
| Tree depth | 2 | 2 |

## Validation Gate Assessment
- **Requirement**: Validation N >= 50
- **Actual**: 66
- **Requirement**: Validation Enrichment >= 0.80 × Discovery Enrichment
- **Actual**: 1.73x vs target 2.11x

**Status**: FAIL. The rule failed to survive unseen validation data.
## Final Scientific Disposition

**Status: REJECTED**

**Stage F-MOGA v0.1 identified a compact state-conditioned candidate that substantially enriched Persistent Build in the Discovery sample, but the candidate failed the predeclared chronological validation-retention gate. It is therefore rejected as a validated state-conditioned representation of Persistent Build. No production, IC, admission, or Coralys-Core changes follow from this result.**

### Scientific Conclusion
A recurring behavioural phenotype (Persistent Build → early adverse excursion/recovery structure) does not automatically imply that the currently admitted state coordinates (`volume_acceleration_60m`, `persistence_240m`) contain a stable compact selector for that phenotype. 

The Coralys evolutionary-search machinery successfully executed the intended discovery → validation → rejection workflow. It has **not** demonstrated a surviving crypto-specific state→phenotype rule.

*This experiment is now closed. The validation dataset is quarantined and must not be used for subsequent parameter tuning.*
