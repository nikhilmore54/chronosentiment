# Transition-Dynamics Step 3: Statistical Test Results

Testing H0: The transition displacement provides no incremental information over the static state.

## 1. Discovery Gate (Likelihood Ratio Test)
- **Static Log-Likelihood ($LL_S$)**: -7347.74
- **Transition Log-Likelihood ($LL_T$)**: -7311.63
- **Test Statistic ($D$)**: 72.21
- **p-value**: 2.083749e-16
- **Result**: PASS (Threshold p < 0.01)

## 2. Validation Gate (Out-Of-Sample Generalization)
- **Static $LL_S$**: -3053.34
- **Transition $LL_T$**: -3067.54
- **Static AUC**: 0.4755
- **Transition AUC**: 0.5210
- **Result**: FAIL (Requires $LL_T > LL_S$ AND $AUC_T > AUC_S$)

## Final Assessment
**Status: REJECTED**. The transition representation failed to demonstrate robust incremental information over the static snapshot. H0 is retained.

---

## Final Scientific Disposition

**Transition-Dynamics v0.1 — REJECTED**

- `W = 60m` — rejected for this representation.
- `ΔVA60` — rejected as part of the tested transition representation.
- `ΔP60` — rejected as part of the tested transition representation.
- Discovery data — consumed only by the predeclared discovery procedure.
- Validation data — quarantined.
- No threshold tuning.
- No alternate windows.
- No sequence mining.
- No MOGA follow-up.
- No production changes.
- No Coralys Core changes.

### Scientific Distinction
The experiment supports the conclusion that **this specific 60-minute transition representation did not survive the predefined validation standard.** It does not establish that every conceivable transition representation is useless. 

Crucially, the transition representation produced strong in-sample incremental explanatory power, but that advantage did not survive the out-of-sample likelihood criterion. This highlights the exact danger the validation gate was designed to catch: features that look deeply informative in discovery but fail to probabilistically generalize.

**The 60-minute transition hypothesis is now officially CLOSED.**