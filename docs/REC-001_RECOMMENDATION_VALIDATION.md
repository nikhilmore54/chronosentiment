# REC-001 — Recommendation Engine Prospective Validation & Policy

**Document ID:** REC-001  
**Version:** 1.0  
**Status:** Active — v0 prospective observation  
**Created:** 2026-08-18  
**Frozen:** No — policy section is frozen; evidence section accumulates  

---

## Purpose

REC-001 defines:

1. The frozen **Recommendation Engine v0 policy** — rules, scoring formula, and invariants.
2. The **prospective validation protocol** — what REC-001 is trying to establish, how outcomes are recorded, and what constitutes validation, failure, or inconclusive.

REC-001 does **not** tune the engine. Post-hoc tuning is prohibited.

---

## Governance Boundaries

### Independence from HDV-002

REC-001 and HDV-002 are **independent research programmes**.

- REC-001 cannot modify C3-002 risk boundaries.
- HDV-002 cannot use REC-001 recommendation outcomes as an optimization feedback loop unless its methodology explicitly authorizes it.
- Adaptive R:R is a RecommendationEngine v1 concern, not a C3-002 or HDV-002 concern.

### Independence from C3-002

- C3-002 is frozen. REC-001 operates on top of C3-002 decisions.
- REC-001 cannot alter the direction, trend, momentum, or volatility produced by C3-002.
- REC-001 can only rank and filter decisions already produced by C3-002.

### Volume governance

- Volume and relative_volume_20 may be captured in the historical reconstruction dataset (REC-001-H).
- Volume is **not** a Recommendation Engine v0 decision variable.
- Volume-conditioned recommendations require separate validation before becoming operational.

---

## Recommendation Engine v0 — Frozen Policy

### Evidence hierarchy

Level 1 — Narrow analogue match: same direction + same Coralys state (trend + momentum), minimum 15 samples.  
Level 2 — Broad analogue match: same Coralys state only, minimum 15 samples.  
Insufficient — fewer than 15 samples at any level.

Evidence source: HDV-001 frozen dataset (`datasets/hdv001/hdv001_outcomes_v1.json`), 728 COMPLETE decisions.

### Evidence classification

| Class | Criterion |
|-------|-----------|
| Favourable | target_before_risk_rate ≥ 0.55 |
| Mixed | 0.40 ≤ target_before_risk_rate < 0.55 |
| Unfavourable | target_before_risk_rate < 0.40 |
| Insufficient | sample_size < 15 at all levels |

### Action rules

| Action | Condition |
|--------|-----------|
| BUY | geometry present AND (evidence Favourable OR (Mixed AND R:R ≥ 1.5)) |
| WATCH | geometry present AND evidence Mixed AND R:R < 1.5 |
| NO_TRADE | geometry absent OR evidence Unfavourable OR Insufficient OR direction == NO_TRADE |

**v0 invariant:** BUY requires valid geometry (reference_price, indicative_target, indicative_risk, R:R all non-null). A recommendation without a known entry price cannot be acted upon.

### Geometry (v0)

Geometry is derived from C3-002 ATR-based TMV multipliers. R:R is always approximately 2.0 in v0 because `t_mul` is always 2× `r_mul` in the TMV multipliers. This is a known v0 limitation — adaptive geometry is a v1 concern.

### Rank score formula

```
rank_score =
    (target_before_risk_rate × 0.50)
  + (min(rr / 3.0, 1.0) × 0.30)
  + (freshness × 0.20)
```

Where:
- `target_before_risk_rate` — from historical evidence
- `rr` — from geometry (capped at 3.0 for scoring purposes)
- `freshness` — 1.0 if effective_session is today or tomorrow, else 0.5

### Policy version

`RECOMMENDATION_POLICY_VERSION = "v0"` — frozen with HDV-001.

### Implementation verification

- 54 tests in `coralys-decision` crate
- 32 tests in `coralys_decision_server`
- 86 total, 0 failed (verified 2026-08-18)

**Implementation complete ≠ research validated.** The 86 tests verify correctness of the implementation against the frozen policy. They do not validate whether v0 improves decision selection.

---

## Prospective Validation Protocol

### Research question

> Does Recommendation Engine v0 improve decision selection and prioritisation among C3-002 decisions?

Specifically:
- Do BUY recommendations achieve target_before_risk at a higher rate than the HDV-001 baseline?
- Does rank score correlate with outcome quality?
- Is the BUY/WATCH/NO_TRADE classification informative?

### Prospective population

- Universe: UNIV-001 (`datasets/universes/coralys_102_v1.json`) — 102 NSE stocks, frozen 2026-08-18.
- Pipeline: C3-002 (`csp006_p_prospective`) with `--universe` flag.
- Decisions are certified at T (latest session ≤ now) and sealed in the prospective ledger.
- Outcomes are recorded at observation boundary (not before).

### What constitutes a recommendation

A recommendation is a `RecommendationRecord` produced by `RecommendationEngine::evaluate()` for a certified C3-002 decision with `CertificationStatus::Certified`.

### How outcomes are recorded

Outcomes are recorded via `POST /decisions/{id}/outcome` on the Decision Server. The outcome must be one of:
- `TARGET_BEFORE_RISK` — indicative target reached before indicative risk boundary
- `RISK_BEFORE_TARGET` — indicative risk boundary reached before target
- `HORIZON` — neither target nor risk reached within the observation horizon

Outcomes are recorded at the observation boundary. No early peek. No post-hoc adjustment.

### Primary metrics

1. **BUY target rate** — fraction of BUY recommendations that achieve TARGET_BEFORE_RISK
2. **NO_TRADE avoidance rate** — fraction of NO_TRADE decisions that would have been RISK_BEFORE_TARGET
3. **Rank score correlation** — Spearman correlation between rank_score and outcome quality

### Secondary metrics

- WATCH target rate
- Evidence class breakdown by outcome
- Freshness effect on outcome

### Evaluation horizon

Minimum 200 COMPLETE decisions (same threshold as HDV-002-A) before any evaluation.

### Success / failure / inconclusive criteria

| Result | Criterion |
|--------|-----------|
| Validated | BUY target rate ≥ HDV-001 baseline + 5pp AND n ≥ 200 |
| Inconclusive | BUY target rate within ±5pp of baseline OR n < 200 |
| Failed | BUY target rate < HDV-001 baseline − 5pp AND n ≥ 200 |

### No post-hoc tuning

Once prospective observation begins, the v0 policy is frozen. No parameter changes, no threshold adjustments, no evidence re-weighting. Changes go into a separately versioned v1 candidate.

---

## Known v0 Limitations (Architectural Findings)

The 102-stock expansion (REC-001-B, 2026-08-18) confirmed:

1. **State-level clustering** — stocks sharing the same Coralys state inherit identical evidence, R:R, and rank scores. KOTAKBANK, LT, CANBK, IDEA, NMDC all scored 0.6211 despite different market characteristics.
2. **Fixed R:R** — R:R is always ~2.0 because C3-002 geometry uses fixed `t_mul = 2 × r_mul`.
3. **No ticker-specific evidence** — the engine cannot distinguish TCS from IDEA within the same state bucket.

These are expected v0 limitations, not bugs. The correct response is:

```
REC-001 v0 (current)
    ↓ prospective observation
REC-001-H (historical reconstruction — ticker-aware, volume, MFE/MAE)
    ↓ evidence enrichment
RecommendationEngine v1 (candidate)
    ↓ prospective comparison against v0
```

---

## Calculation Ownership

| Calculation | Owner |
|-------------|-------|
| Coralys direction | C3-002 (frozen) |
| ATR-14 | `csp006_p_prospective` pipeline |
| Target/risk geometry | C3-002 TMV multipliers |
| Historical evidence | `EvidenceStore` (`coralys-decision/src/recommendation/evidence.rs`) |
| Recommendation action | `RecommendationEngine` (`coralys-decision/src/recommendation/engine.rs`) |
| Rank score | `RecommendationEngine` |
| Rendering | `chrono-ui` (display only — no re-ranking, no re-scoring) |
| Execution fill | User / execution layer |
| Outcome | Decision Server (`POST /decisions/{id}/outcome`) |
| Research validation | REC-001 (this document) |

---

## Related Artifacts

| ID | Path | Relationship |
|----|------|-------------|
| HDV-001-F | `datasets/hdv001/HDV_001_F_DETERMINATION.md` | Evidence source for v0 — FROZEN |
| HDV-002-A | `docs/HDV_002_METHODOLOGY.md` | Independent risk-boundary research — not a feedback loop |
| UNIV-001 | `datasets/universes/coralys_102_v1.json` | Prospective universe — frozen 2026-08-18 |
| REC-001-H | (not started) | Historical reconstruction — prerequisite for v1 |
| SVC-001 | `services/coralys_decision_server/` | Serves `GET /recommendations/latest` |
| SVC-002 | `chrono-ui/` | Displays recommendations — no policy logic |