# RP-608-WL-IMPROVE-001 — NSE A Kite Discovery → Gate 2 Disposition

**Date:** 2026-11-06  
**Authority:** 0.0% · baseline frozen  
**Book:** NSE Estimand A only  
**Artifact:** `artifacts/rp608_econ_edge_diag/wl_nse_a_kite_discovery_v2.md`  
**Scope:** `docs/research/rp608_wl_nse_a_kite_discovery_v2_scoped.md`

## Formal disposition

**Gate 2: DO NOT ADVANCE** (no Gate 3 offline experiment; no live / OF / threshold change).

Crypto plan Gate 2 disposition remains **DO NOT ADVANCE** (separate book).

## What replicated (W/L)

| Candidate | Discovery WR gap | Holdout verdict | Notes |
| --- | ---: | --- | --- |
| `MOM_UP_>10bps` vs `MOM_DOWN_<-10bps` | +12.2 pp | REPLICATES (pooled) | Forward day (Nov 6) **sign-flips** |
| `MOM_UP_>10bps` vs `MOM_FLAT` | +15.4 pp | REPLICATES (pooled) | Holdout gap thin (+3.0 pp); fwd flips |
| `LOC_MID_THIRD` vs `LOC_UPPER_THIRD` | +10.3 pp | REPLICATES | Most stable across pre + fwd |
| Hour 12 vs 14 | +23.4 pp | REPLICATES | Diagnostic only — blocked |

Trend contrasts do **not** replicate. Book is effectively LONG-only in window (no side contrast).

## Why Gate 2 still does not advance (§3b)

Preferred (“high WR”) cohorts fail supporting economic bars:

| Preferred side | Discovery payoff | Discovery net | Holdout net |
| --- | ---: | ---: | ---: |
| `MOM_UP_>10bps` | 0.92 | −₹11.1k | −₹30.7k |
| `LOC_MID_THIRD` | 0.88 | +₹1.9k | −₹5.6k |

W/L-first does **not** mean WR-only success. Payoff &lt; 1 and holdout nets ≤ 0 on the preferred slice mean a selection filter is not yet a defensible improvement proposal.

## Allowed continuations (not Gate 3)

1. Keep monitoring Nov+ sessions under frozen baseline (Pi host).  
2. Payoff-aware re-score / joint WR–payoff–net thresholds — research-only.  
3. Crypto still needs non-Kite bars for the same feature family.  
4. Do **not** promote hour filters; do **not** change OF ±0.30.

## Non-claims

Does not authorize Gate 3, live authority, stop/target edits, or book pooling.
