# UltraCrew — Objective Function Alignment with GERAD

**Status:** Planned
**Date:** —
**Relates to:** [UltraCrew vs GENCOL Pipeline Divergence Analysis](UltraCrew_GENCOL_Pipeline_Divergence_Analysis.md), Section 7 Step 3
**Prerequisite:** [Pairing-Topology Mutation Operator Evaluation](UltraCrew_Pairing_Topology_Mutation_Evaluation.md) (Step 1), [Layover Threshold Experiment](UltraCrew_Layover_Threshold_Experiment.md) (Step 2)
**Hypothesis under test:** Adding GENCOL-equivalent objective terms to [`ConstraintEngine.evaluate()`](../../adapters/ultracrew/src/constraint_engine.rs:39) improves benchmark alignment with the GERAD reference without degrading UltraCrew's primary objectives (coverage, fairness, fatigue).

---

## 1. Objective

Align UltraCrew's fitness function with the GERAD objective function by adding the four cost terms that GENCOL minimizes but UltraCrew currently does not model. Measure the effect on benchmark metrics.

---

## 2. Background

GENCOL minimizes:

```
pairing cost = deadhead cost + hotel cost + duty credit
             + time away from base (TAFB) + connection penalties
```

UltraCrew's current fitness function ([`constraint_engine.rs:39`](../../adapters/ultracrew/src/constraint_engine.rs:39)) does not include any of these terms. It optimizes coverage completeness, workload fairness, fatigue, and pairing completion reward.

Because the two systems minimize different objectives, they will converge to different solutions even on identical inputs. This experiment adds the four missing terms incrementally to measure their individual and combined effect.

---

## 3. Terms to Add

### 3.1 Time Away From Base (TAFB)

Penalize total layover hours per pairing. GENCOL minimizes this directly.

```
tafb_cost = Σ layover_hours(pairing) × tafb_weight
```

Suggested initial weight: tune against coverage score to avoid over-penalizing long pairings that are otherwise legal.

### 3.2 Hotel nights

Penalize number of overnight layovers per pairing.

```
hotel_cost = Σ overnight_layovers(pairing) × hotel_weight
```

### 3.3 Deadhead cost (proxy)

Penalize shifts where the assigned worker is not the most qualified available worker for that shift type. This is a proxy for deadhead cost (the cost of positioning a crew member who is not the primary assignment).

```
deadhead_cost = Σ qualification_gap(shift, worker) × deadhead_weight
```

### 3.4 Connection penalty

Penalize pairings where the inter-FDP rest gap is close to the minimum threshold (fragile connections that are at risk of becoming illegal under minor schedule perturbation).

```
connection_penalty = Σ max(0, min_rest_buffer - actual_rest_gap(fdp_i, fdp_{i+1})) × connection_weight
```

---

## 4. Experimental Design

Add terms one at a time, running the full GERAD benchmark after each addition:

- **Condition A:** Baseline (current fitness function, post Step 1 and Step 2)
- **Condition B:** + TAFB term
- **Condition C:** + TAFB + hotel nights
- **Condition D:** + TAFB + hotel nights + deadhead proxy
- **Condition E:** + TAFB + hotel nights + deadhead proxy + connection penalty (full alignment)

This incremental approach isolates the contribution of each term.

---

## 5. Metrics to Record

For each condition and each GERAD instance:

- Pairing count ratio (UltraCrew / GERAD reference)
- Compliance rate
- Multi-duty pairing ratio
- Mean pairing span
- Coverage score
- Fairness score (workload variance)
- Mean TAFB per pairing
- Mean hotel nights per pairing
- MOGA convergence curve

---

## 6. Results

*(To be filled in after experiment is run.)*

---

## 7. Weight Tuning

The four new terms introduce four new hyperparameters. Initial weights should be set so that the new terms are comparable in magnitude to the existing hard constraint penalties (order of magnitude: 100–500 per violation). Document the final weights used and the rationale for each.

---

## 8. Risks

Adding cost terms that conflict with coverage completeness may cause the MOGA to sacrifice coverage in favour of pairing cost minimization. Monitor coverage score carefully across all conditions. If coverage degrades, reduce the weight of the offending term or add a coverage floor constraint.

---

## 9. Conclusion

*(To be filled in after experiment is run.)*

---

## 10. Reference

See [UltraCrew vs GENCOL Pipeline Divergence Analysis](UltraCrew_GENCOL_Pipeline_Divergence_Analysis.md), Section 7 Step 3 for the architectural context of this investigation.

Kasirzadeh A., Saddoune M., Soumis F. (2017). Airline crew scheduling: models, algorithms, and data sets. *EURO Journal on Transportation and Logistics*, 6(2), 111–137. DOI: 10.1007/s13676-015-0080-x