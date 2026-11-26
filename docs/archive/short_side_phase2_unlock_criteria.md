# Phase 2: Short-Side Unlock Criteria

Aligned with `.cursor/rules/chronosentiment-core.mdc`:
- deterministic replay only
- read-only diagnostics before logic changes
- explicit, testable promotion gates

## Purpose

Define pre-agreed, evidence-based conditions for when short-side logic can be safely revisited.

This avoids ad-hoc threshold changes and preserves auditability.

## Entry Preconditions (must all pass)

1. **Signal Presence**
   - At least one controlled sweep shows `bearish_events > 0`.

2. **Candidate Feasibility**
   - At least one controlled sweep shows `SELL_candidates > 0`.

3. **Stability Across Slices**
   - `near_bearish > 0` in at least `4/10` deterministic offsets.

4. **Cross-Context Evidence**
   - Evidence appears in at least one non-largecap context (`midsmall` or `cyclic`) OR in longer-horizon sweeps (`LIMIT >= 4000`).

If any precondition fails, short-side remains deferred.

## Unlock Stages

### Stage A: Shadow-Only Unlock

Allowed action:
- Add shadow-only bearish mapping (no execution impact).

Required checks:
- Shadow `SELL` appears in multiple offsets.
- No regression in existing long-side diagnostics.

Promotion gate to Stage B:
- Shadow sell quality is non-degenerate and reproducible across reruns.

### Stage B: Controlled Candidate Unlock

Allowed action:
- Allow bearish candidate creation in controlled mode.

Required checks:
- Candidate and pass-through counters remain stable run-to-run.
- No deterministic instability introduced.

Promotion gate to Stage C:
- SELL pipeline reaches final stage in at least some slices without degrading baseline reliability.

### Stage C: Limited Live Unlock

Allowed action:
- Enable short-side logic in limited rollout with strict monitoring.

Required checks:
- No material degradation in overall decision quality.
- Drawdown and retention remain within accepted policy bands.

## Hard Stops

Revert to prior stage immediately if any occur:

- deterministic replay mismatch for same input
- sudden spike in low-quality SELL emissions
- sustained degradation in core reliability metrics

## Required Reporting Format Per Sweep

Use one table per sweep:

- `offset`
- `bullish_events`
- `bearish_events`
- `near_bearish`
- `BUY_candidates`
- `SELL_candidates`
- `final_buy`
- `final_sell`

Include one-line classification:

- `signal_absent`
- `candidate_blocked`
- `gate_blocked`
- `ranking_blocked`
- `short_supported`

## Decision Rule Summary

Short-side design work begins only when evidence transitions from:

- "bearish absent at source"

to:

- "bearish present and reproducible pre-candidate"

This keeps Phase 2 disciplined, reversible, and deterministic.
