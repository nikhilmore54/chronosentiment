# HDV-001-F Baseline Execution Clarification

**Date:** 2026-08-17
**Status:** METHODOLOGICAL CLARIFICATION — supersedes first invalid run

---

## Invalid Run Record

The first execution of `scripts/hdv001_run_baselines.py` produced results that
are methodologically invalid and must not be interpreted as research findings.

The invalid run produced:

| Model | TARGET | RISK | HORIZON |
|-------|--------|------|---------|
| Coralys | 35.7% | 41.5% | 22.8% |
| Random_A | 65.2% | 21.8% | 12.9% |
| Inverse_B | 96.7% | 3.3% | 0.0% |
| Momentum_C | 62.9% | 21.7% | 15.4% |

The Inverse_B result of 96.7% is a diagnostic indicator of implementation
defect, not a research finding. The Random_A result of 65.2% is similarly
contaminated.

**No scientific conclusion about Coralys should be drawn from this run.**

---

## Defect 1: Directional geometry not transformed

The first implementation passed Coralys's absolute `target_price` and
`stop_price` to the baseline classifier without transforming them for the
baseline direction.

Example of the defect:

```
Coralys: LONG, reference=100, target=105, stop=95
Inverse: SHORT, target=105 (unchanged), stop=95 (unchanged)

Classifier asks: did SHORT close <= 105? YES, almost always.
```

This is not a meaningful inverse trade. The target/stop geometry must be
reconstructed around the reference price for each baseline direction.

### Corrected rule (Rule A)

For Baselines A and B, when the baseline direction differs from Coralys,
preserve the **absolute distances** from reference price and reconstruct
directional boundaries:

```python
target_distance = abs(target_price - reference_price)
stop_distance   = abs(stop_price   - reference_price)

if baseline_direction == "LONG":
    baseline_target = reference_price + target_distance
    baseline_stop   = reference_price - stop_distance
else:  # SHORT
    baseline_target = reference_price - target_distance
    baseline_stop   = reference_price + stop_distance
```

When the baseline direction equals the Coralys direction, the original
absolute prices are used unchanged.

---

## Defect 2: Momentum baseline used random fallback

The first implementation fell back to a random direction when the 20-session
moving average history was unavailable:

```python
if mom_dir is None:
    mom_dir = rng_primary.choice(["LONG", "SHORT"])  # WRONG
```

572 of 728 decisions (78.6%) used this fallback, making the Momentum_C
result effectively a second random baseline.

### Corrected rule (Rule B)

Baseline C requires a separate historical lookback cache covering at least
20 NSE sessions before the earliest decision date (2026-07-14).

The required lookback start is approximately 2026-06-13 (allowing for
weekends and holidays). A separate cache `hdv001_baseline_history_v1`
must be built covering 2026-06-01 to 2026-07-13.

No random fallback is permitted. If a decision still lacks 20 prior sessions
after the extended cache is built, it is excluded from the Momentum_C
comparison (not substituted with random).

---

## Governance note

The frozen success criterion from HDV-001-G Gate 6 is unchanged:

- Coralys TARGET_BEFORE_RISK rate > Baseline A by >= 5 pp
- Coralys TARGET_BEFORE_RISK rate > Baseline B by >= 5 pp
- Advantage in >= 2 of 4 Coralys state segments

The corrected run must evaluate against this criterion exactly as frozen.

---

## Next action

1. Build `hdv001_baseline_history_v1` cache (2026-06-01 to 2026-07-13)
2. Rewrite `scripts/hdv001_run_baselines.py` with Rules A and B
3. Produce corrected `hdv001_baseline_results_v1.json`
4. The invalid run artifacts are retained for audit but must not be cited