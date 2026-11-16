# Watchlist + Pre-open + Warmup Governance Lock

**Date:** 2026-11-02  
**Authority:** 0.0%  
**Live G3:** experimental `cross_gap_neutral_v1` enabled 2026-11-03 (revert `RP608_G3_WARMUP_POLICY=same_session`) · **G1–G2 / G4–G5 / OF ±0.30:** unchanged · **Authority:** 0%

## Disposition

| Item | Status |
| --- | --- |
| Existing 20-name next-session watchlist | **Preserved** as prior-session candidate cohort |
| Policy A | Live baseline |
| Policy B | Diagnostic only |
| Policy C | Leading experimental candidate; multi-session **economics queued independently** |
| Policy D | Investigation authorized; blocked on usable pre-open evidence |
| Pre-open capture remediation | Authorized (daily REST + WS continuity) |
| Nov 3 2026 | **Data-integrity milestone**, not strategy-performance |

## Three-stage observation (no rule change)

1. **Previous close** — watchlist rankings, traj15_eod, EOD OF marks (not entry auth).
2. **Pre-open** — opening-context only; never independently mints G4 PASS.
3. **Continuous market** — live traj15 + L1 revalidation; existing G1–G5.

## Watchlist LONG-only finding

`scripts/rp608_nse_next_session_recs.py` is **structurally long-only**:

- Hardcodes `side = "LONG"`.
- Keeps symbols with `traj15 ≥ +0.05` **or** session return ≥ **+25 bps**.
- Does not emit SHORT candidates.

So absence of SHORTs on the board is a **generation-policy property**, not proof that the market had no short-side morphology that day. Cross-session warmup economics must either (a) evaluate LONG watches only and label the bias, or (b) add a separate observational SHORT watch track later — without changing live entry rules yet.

## Provenance rule (Nov 3+)

Every captured field must retain its source label. Missing stays missing (not coerced to 0).  
Do not treat `FALLBACK_LAST_PRICE` as indicative clearing.

## Parallel tracks

- **Track 1:** Nov-3 capture-quality (coverage, continuity, provenance, REST snap consistency, 09:15 transition).
- **Track 2:** Multi-session Policy C economics (does not wait on Track 1).
- **Track 3:** Policy D vs C only after Track 1 yields reliable pre-open observations.

## First Policy C econ snapshot (diagnostic · n small)

Artifact: `artifacts/rp608_cross_session_warmup/policy_c_econ_2026-10-29_30.json`  
Sessions: 2026-10-29 + 2026-10-30 · cohort: 20 watchlist LONGs · friction proxy 12 bps.

| Policy | n | Mean net bps | Win rate | Profit factor | Notes |
| --- | ---: | ---: | ---: | ---: | --- |
| A same-session | 36 | **+13.74** | 36.1% | **1.68** | Baseline |
| C gap-neutral | 38 | +1.25 | 31.6% | 1.04 | 25 timing advances · 2 incremental-only |

### Paired timing-advance incremental value (identical A exit)

Artifact: `artifacts/rp608_cross_session_warmup/policy_c_paired_timing_value_2026-10-29_30.json`  
n=25 pairs · exit locked to baseline A exit · equal 12 bps round-trip cancels in delta.

| Result | Value |
| --- | ---: |
| Mean incremental Δ (C early − A) | **+50.5 bps** |
| Early better / worse | **22 / 3** (88%) |
| Mean entry edge | +49.7 bps |
| Survive +10 bps early-extra cost | still ~mean +40 bps · 84% better |
| Survive +35 bps early-extra cost | mean +15.5 bps · 68% better |

**Reconciliation:** Portfolio A>C can coexist with paired early-entry benefit because portfolio C uses **its own exits** and mixes non-paired trades. Paired identical-exit isolates entry-timing value only. **Still not a deploy decision** — expand sessions/regimes; LONG-only bias remains.
