# RP-608-WL-IMPROVE — NSE Estimand A Kite Discovery v2 (Scoped)

**Date:** 2026-11-06  
**Protocol:** `RP-608-WL-IMPROVE-001-NSE-A-KITE-DISCOVERY-V2`  
**Authority:** 0.0% · baseline frozen · Gate 2 **criteria unchanged**  
**Book:** NSE Estimand A only (never pooled with OF / crypto)

## Why this is authorized

Crypto plan Gate 2 disposition is **DO NOT ADVANCE** on recorded decision-time fields.  
Allowed continuation (Gate 2 doc): reconstruct look-ahead-safe trend / momentum / price-location and run a **new** Phase 2 — new evidence family, not re-mining crypto 3–5 Nov.

Kite historical join unlocks those features for **NSE A only**. This does **not** auto-reopen crypto Gate 2.

## Windows (predeclared)

| Window | Days (IST) |
| --- | --- |
| Discovery | 2026-11-03, 2026-11-04, 2026-11-05 |
| Holdout pre | 2026-10-27, 2026-10-28, 2026-10-30, 2026-11-02 |
| Holdout forward | 2026-11-06 |

Oct 29 excluded from holdout (same anomaly hygiene as crypto lineage; not re-cut for convenience).

## Feature families (predeclared)

| Family | Feature | Source |
| --- | --- | --- |
| market_state | `trend` | Kite 15m, bar close &lt; entry |
| market_state | `momentum` | Kite 15m, look-ahead-safe |
| market_state | `price_location` | Kite 15m prior range |
| directional_state | `side` | Journal at entry |
| temporal_descriptors | `hour_ist` | Entry timestamp (diagnostic only) |

Unavailable keys are **not imputed**. Post-entry MFE/MAE/exit are **forbidden predictors**.

## Gate 2 criteria (external, unchanged)

Same as crypto discovery v2 / plan:

1. Measurable entry-quality hypothesis  
2. Decision-time only  
3. Discovery support (n≥15 each side, positive WR gap)  
4. Holdout replication (same-sign, powered)  
5. Falsifiable on holdout  
6. Coverage-reducing WR alone ≠ success  

## Execution host

**Pi** (`docs/research/rp608_pi_research_execution.md`) for Kite join; discovery script may run on Pi after join.

## Outputs

- `artifacts/rp608_econ_edge_diag/wl_nse_a_kite_discovery_v2.md`
- `artifacts/rp608_econ_edge_diag/wl_nse_a_kite_discovery_v2.json`
- Extended join: `wl_nse_a_decision_time_features_kite_join.jsonl`

## Non-claims

Does not change OF ±0.30, stops/targets, Estimand A entry thresholds.  
Does not pool books. Does not authorize Gate 3. Crypto still needs non-Kite bars.
