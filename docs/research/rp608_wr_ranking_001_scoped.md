# RP-608-WR-RANKING-001 — Ranking and Coverage Improvement (Scoped)

**Protocol:** `RP-608-WR-RANKING-001`  
**Date:** 2026-11-06  
**Authority:** 0.0% · baseline frozen · Gate 2 **CLOSED** · Gate 3 unauthorized  
**Book:** NSE Estimand A only  
**Host:** Pi  
**Supersedes (for research focus):** threshold discovery on contaminated holdout (001/002)

## Research question (locked)

> Can existing decision-time information rank winners above losers consistently across sessions, while retaining useful coverage?

**Not:** which threshold maximizes holdout WR.

## Why this unit

002 established: 70% @ 8.6% coverage is narrow/session-skewed; thr 0.50 post-selected on holdout; holdout AUC ≈ 0.51.  
Oct 28 session: **31/37 = 83.8%** (−1.2 pp vs 85%) is a **promising session signal**, not proof of a general 85% policy.

## Frozen evaluation protocol

### A. Ranking construction (discovery-only)

- Score `p_hat_entry_v1` unchanged: unweighted mean of empirical \(P(\mathrm{win}\mid\mathrm{level})\)  
- Fit **only** on discovery days `2026-11-03…05`  
- Features: `trend`, `momentum`, `price_location`, `side`, `hour_ist`  
- Labels: existing net W/L (unchanged)  
- **No** threshold re-optimization on holdout  

### B. Ranking diagnostics (predeclared)

- AUC / mean score win vs loss (overall + per session)  
- Score distribution by session  
- Within-session ranking where n≥20  

### C. WR–coverage without best-threshold picking

Predeclared **coverage bands** (quantiles of score), not scanned for max WR:

| Band | Definition |
| --- | --- |
| ALL | 100% eligible |
| TOP50 | top 50% by `p_hat` |
| TOP25 | top 25% by `p_hat` |
| TOP10 | top 10% by `p_hat` |
| SLICE_MOM_LOC | `MOM_UP_>10bps` ∩ `LOC_MID_THIRD` (composition slice, not a thr) |

Report for each band: WR, W/L counts, coverage, payoff, net — on discovery and on holdout **descriptively**. Do **not** promote the band with highest holdout WR as a Gate 2 candidate from this unit alone.

### D. Session stability & concentration

- Per-session WR, n, feature mix  
- Share of selected/high-score mass by session  
- Explicit Oct 28 vs Oct 30 vs Nov 2 contrast  

### E. Chronology note (mandatory)

Discovery fit uses Nov 3–5. Holdout-pre days (incl. Oct 28) **precede** the fit window. Applying discovery-fit scores to Oct 28 is **not** prospective prediction.  
Oct 28 analysis prioritizes **composition** and **within-session** win/loss contrasts. Prospective confirmation requires **later untouched sessions after the contaminated holdout window**.

### F. Criteria for “genuine ranking improvement”

All required:

1. Discovery AUC materially &gt; 0.55 and holdout/forward AUC does not collapse to ~0.50  
2. At a **predeclared** coverage band (≥25% coverage), WR lifts vs ALL on discovery **and** on an untouched forward window  
3. Lift not dominated by a single session (≥60% of band mass)  
4. Payoff ≥ 1 and net &gt; 0 on that band (supporting)  
5. No label/outcome redefinition; no OF/stop/target change  

Failing these → ranking improvement **not** demonstrated; Gate 2 stays closed.

## Oct 28 investigation (this deliverable)

Using decision-time fields only:

1. What differed in Oct 28’s entry feature mix vs Oct 30 / Nov 2 / discovery?  
2. Was success concentrated in `MOM_UP ∩ LOC_MID`?  
3. Within Oct 28, do winners differ from losers on entry features?  
4. Does discovery-fit `p_hat` rank within Oct 28 (transfer diagnostic only)?  
5. Status of later untouched sessions for prospective check  

## Forbidden

- Threshold scans on contaminated holdout  
- Feature mining expansion this unit  
- Hour/symbol bans from holdout  
- Stop/target/OF/horizon changes  
- Treating 70% @ 8.6% or Oct 28 alone as Gate 2 advance  

## Artifacts

- Script: `scripts/research/rp608_wr_ranking_001.py`  
- Report: `artifacts/rp608_econ_edge_diag/rp608_wr_ranking_001.{md,json}`  
- Gate 2: `docs/research/rp608_wr_ranking_001_gate2.md`
