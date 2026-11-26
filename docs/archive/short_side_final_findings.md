# Short-Side Signal Diagnostic — Final Findings

Aligned with `.cursor/rules/chronosentiment-core.mdc`:
- deterministic replay only
- event-driven diagnostics
- read-only instrumentation (no strategy/gate/GA behavior changes)

## Objective

Determine whether the absence of `SELL` recommendations is caused by:

- data/regime bias
- signal-generation asymmetry
- gate suppression
- ranking (ML/elite) bias

## Methodology

Three read-only diagnostic layers were used:

### 1. Raw Signal Layer

- `[RAW_TENDENCY]`
  - `bullish_events`
  - `bearish_events`
  - `wait_events`

### 2. Candidate/Pipeline Layer

- `[SIDE_DISTRIBUTION]`
  - `candidates_buy`, `candidates_sell`
  - `pass_buy`, `pass_sell`
  - `final_buy`, `final_sell`

### 3. Component-Level Layer

- `[COMPONENT_DIAGNOSTIC]`
  - `momentum_neg`
  - `composite_neg`
  - `score_neg`
  - `near_bearish`

All counters are observational only and deterministic.

## Experiment Setup

- Buckets tested:
  - `largecap`
  - `midsmall`
  - `cyclic`
- Offsets:
  - 10 deterministic slices (`0..3600`, step `400`)
- Config:
  - current elite + alternate elite checks
  - fixed `LIMIT=400`
  - no parameter/threshold changes

## Results

### Raw Signal Layer

- `bearish_events = 0` across all tested buckets and offsets
- `bullish_events > 0` appears in multiple offsets

### Candidate Layer

- `SELL_candidates = 0` across all runs
- `final_sell = 0` across all runs

### Component Layer

- `momentum_neg = 0`
- `composite_neg = 0`
- `score_neg = 0`
- `near_bearish > 0` only in `1/10` offsets (largecap matrix)

### Distribution Summary (final largecap matrix)

- `offsets_with_bearish_events>0 = 0/10`
- `offsets_with_near_bearish>0 = 1/10`
- `offsets_with_final_sell>0 = 0/10`

## Classification

This matches **Pattern C (weak/sparse near-bearish presence)**:

- bearish pressure is rare and weak
- direction never crosses bearish boundary
- no downstream suppression evidence

## Root Cause

**Pre-candidate signal asymmetry** in current setup:

- bearish directional state is not emitted at source
- negative conviction does not materialize
- downstream layers do not get SELL input to process

## Ruled Out Causes

- gate asymmetry
- ranking/elite suppression
- execution path asymmetry
- confirmation timing as primary cause

## Decision

**Do not enable short-side logic for MVP** under current evidence.

Rationale:

- no stable bearish structure observed
- no SELL candidate formation across controlled sweeps
- unlocking shorts now would add noise without validated signal support

## Revisit Criteria

Reopen short-side design only if one or more occur:

- `near_bearish > 0` in at least `4/10` slices
- any slice with `bearish_events > 0`
- any slice with `SELL_candidates > 0`
- longer horizon sweeps (`LIMIT >= 4000`) produce stable bearish structure

## Conclusion

The absence of `SELL` is not a defect; it is a measured property of current signal behavior and sampled regimes.

Current system posture:

**Selective high-conviction long-side decision system**.
