# E-GATE v3 Certification (B4)

**Result:** PASS

This gate extends E-GATE v2 with the temporal metadata invariant:

```
assessment.evaluation_timestamp <= decision.evaluation_timestamp
```

for every assessment→decision pair.

B3 is not modified. G-GATE v1.1 on B3 remains INCONCLUSIVE / leakage FAIL.

## Checks

See `invariant_checks.txt`.

Required:

1. Assessments = 195
2. Decisions = 195
3. Strategies = 110
4. Outcomes = 440
5. Assessment → Decision lineage (no orphan decisions)
6. Decision → Strategy lineage
7. Strategy → Outcome matches = 440
8. Exactly 4 outcomes per strategy
9. Horizons = 5D/10D/20D/60D
10. **assessment.evaluation_timestamp <= decision.evaluation_timestamp**
11. Outcome evaluation timestamps not before the parent decision
12. Outcome evaluation_timestamp < horizon_expiry_timestamp

## Dataset

- Database: `chrono_b4_test`
- Dump SHA-256: `f74e576e8e98b24058cc913b14a567d9ff4b3eabc75662ab9a96901b102f8cd6`

## Methodology

G-GATE, if run after this gate PASSes, must use frozen v1.1 unchanged
(`Y_h`, candidate, 55/27/28 split, bootstrap, seed `20260813`).
