# Short-Side Diagnostic Checklist

Aligned with `.cursor/rules/chronosentiment-core.mdc`:
- deterministic runs only
- event-driven pipeline visibility
- read-only diagnostics (no strategy mutation)

## Goal

Identify exactly where `SELL` disappears in the pipeline and classify the cause unambiguously.

## Required Counters (Minimal Set)

Collect these per run:

- `BUY_candidates`
- `SELL_candidates`
- `BUY_pass`
- `SELL_pass`
- `BUY_final`
- `SELL_final`

These are sufficient to classify most short-side issues in one pass.

## Layered Plan

### Layer 1: Data / Regime / Universe

Hypotheses covered: regime bias, universe bias, sample coverage issues.

Run:
- multiple offsets (target 10-15)
- longer windows (e.g. `LIMIT=4000`)
- optional alternate symbol bucket

Capture:
- `BUY_final`, `SELL_final` per run

Interpretation:
- If `SELL_final > 0` anywhere: system can emit shorts; current slices were biased.
- If `SELL_final = 0` everywhere: move to deeper pipeline layers.

### Layer 2: Candidate Generation

Hypotheses covered: signal/candidate asymmetry.

Capture:
- `BUY_candidates`, `SELL_candidates`

Interpretation:
- If `SELL_candidates = 0`: candidate generation is long-biased.
- If `SELL_candidates > 0`: move to gate layer.

### Layer 3: Gate Stack

Hypotheses covered: gate asymmetry (volatility, momentum, confirm/aging).

Capture:
- `BUY_pass`, `SELL_pass`

Interpretation:
- If `SELL_candidates > 0` and `SELL_pass = 0`: gate asymmetry confirmed.
- If `SELL_pass > 0`: move to ranking layer.

Optional deepening:
- Add per-filter drop counters:
  - `sell_dropped_by_vol`
  - `sell_dropped_by_momentum`
  - `sell_dropped_by_confirm`

### Layer 4: Ranking / Elite

Hypotheses covered: elite/ranking bias.

Capture:
- `BUY_final`, `SELL_final`

Interpretation:
- If `SELL_pass > 0` and `SELL_final = 0`: likely ranking/elite bias.
- If `SELL_final > 0`: ranking supports shorts.

### Layer 5: Execution / Trigger (Only if Needed)

Use only if prior layers are inconclusive.

Capture:
- intents created by side
- intents triggered by side

Interpretation:
- If `SELL_created > 0` and `SELL_triggered = 0`: execution/trigger asymmetry.

## Fast Decision Tree

1. `SELL_candidates = 0`
   - Candidate generation / signal asymmetry

2. `SELL_candidates > 0` and `SELL_pass = 0`
   - Gate asymmetry

3. `SELL_pass > 0` and `SELL_final = 0`
   - Ranking / elite bias

4. `SELL_final > 0`
   - Shorts are supported; current slices were regime-biased

## One-Pass Execution Sequence

1. Add minimal counters (read-only instrumentation).
2. Run 10-offset sweep with larger limit.
3. Print per-run side summary.
4. Classify via decision tree above.

## Notes For Demo and Stakeholders

If asked "why no shorts?":

"The system emits only validated conviction. In these slices, bearish setups did not pass validation."

This is consistent with deterministic validation, not a missing feature.
