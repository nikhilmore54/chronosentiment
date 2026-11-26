# RP-608-WL-IMPROVE-001 — Crypto Plan Decision-Time Feature Sufficiency & Hypothesis Discovery **v2**

**Protocol version:** `RP-608-WL-IMPROVE-001-DISCOVERY-V2`  
**Date locked:** 2026-11-06  
**Authority:** 0.0% · baseline frozen · OF ±0.30 frozen  
**Implementation:** Rust in `chronosentiment_adapter` research laboratory  

```bash
cargo run -p chronosentiment_adapter --features research --release \
  --bin rp608_wl_crypto_plan_discovery_v2 -- \
  --out-dir artifacts/rp608_econ_edge_diag
```

Sources: `adapters/chronosentiment/research/src/wl_crypto_plan_discovery_v2/` ·  
`adapters/chronosentiment/research/bin/rp608_wl_crypto_plan_discovery_v2.rs`  

## Boundary (immutable)

| Item | Status |
| --- | --- |
| Phase 1 / Crypto Phase 2 / 3–5 Nov discovery | Complete · descriptive only · **not overwritten** |
| Holdout replication v1 | Failed · **immutable evidence** |
| Gate 2 (v1 disposition) | Not advanced · retained |
| Gate 2 **criteria** | **Unchanged** |
| Gate 3 | Unauthorized |
| Other books | Deferred |
| Execution / stops / targets / H0 | Unchanged |

v2 is a **new research version**. It must not rewrite v1 artifacts or reinterpret failed replication as success.

## Objective

Improve the *discovery algorithm* so that Gate 2 becomes *reachable through better evidence*, without guaranteeing it opens.

**Success:** either a falsifiable, evidence-supported Gate 2 candidate, **or** a reliable finding that available decision-time evidence is insufficient.

## Pipeline (Rust)

1. Data integrity + feature-availability audit  
2. Decision-time feature inventory  
3. Broad, controlled association discovery (predeclared families)  
4. Candidate stability + redundancy analysis  
5. Falsifiable hypothesis compiler  
6. Existing Gate 2 criteria applied unchanged → review or research-only  

## Feature families (predeclared)

1. Directional state  
2. OF support state  
3. Direction × OF interaction  
4. Market-state descriptors (imbalance, ATR, RR/geometry where available)  
5. Temporal descriptors (hour)  

Forbidden predictors: post-entry MFE/MAE, exit, later marks.  
Do not invent trend/momentum/price-location/persistence from future prices.

## v1 lineage (do not rediscover as independent evidence)

LONG vs SHORT · OF_SUP · LONG×OF · imbalance · ATR · hour — recorded in lineage registry with prior discovery/holdout outcomes.

## Gate 2 criteria (external, unchanged)

As locked in `docs/research/rp608_wl_improvement_plan.md` and applied in  
`docs/research/rp608_wl_improve_crypto_plan_hypothesis_and_gate2.md`:

- Specific measurable entry-quality hypothesis with descriptive support  
- Decision-time observables only  
- Holdout replication required before Gate 2 advance  
- Coverage-reducing WR alone is not success  
- No baseline / OF / execution change from Gate 2 alone  

v2 diagnostics (Simpson flags, session domination, redundancy) **inform** promotion; they do **not** replace these criteria.
