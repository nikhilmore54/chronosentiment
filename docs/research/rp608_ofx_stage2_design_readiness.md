# RP-608-OFX — Stage 2 Design & Readiness (Preparation Only)

**Protocol:** `RP-608-OFX`  
**Phase:** `STAGE_2_DESIGN_READINESS`  
**Disposition:** **AUTHORIZED FOR DESIGN ONLY · MECHANISM CLASSIFIER NOT AUTHORIZED**  
**Date locked:** 2026-11-04  
**Authority:** 0.0%  

**Decision (governing):** Stage 1 remains locked as `DOMINANTLY_FLAT_SHORT_HORIZON`.  
**Stage 2 mechanism classification (absorption / acceptance / failed auction) is not authorized.**  
This document prepares the protocol so a future run cannot silently invent labels or become trading rules.

**Accepted assessment (2026-11-04):** Clean research boundary. Next meaningful decision = whether eventual evidence satisfies the **preregistered readiness gates** — not reinterpretation of Stage 1. No Stage 2 execution or trading-rule change is implied by design/readiness authorization.

**Immutable priors:**  
- Stage 1 summary/events/quality audit — do not rewrite conclusions  
- Four-book baseline — frozen  
- Parent scope: [`rp608_ofx_order_flow_response_scoped.md`](rp608_ofx_order_flow_response_scoped.md)  
- Stage 1: [`rp608_ofx_stage1_aggression_response_observer.md`](rp608_ofx_stage1_aggression_response_observer.md)  
- Stage 1 audit: [`rp608_ofx_stage1_result_quality_audit.md`](rp608_ofx_stage1_result_quality_audit.md)

---

## 0. Why Stage 2 execution is held

| Evidence | Supports | Remaining limitation |
| --- | --- | --- |
| 24,714 unique events, zero duplicates | Sample event integrity | Not broader session coverage |
| Ambiguous excluded | Cleaner BUY/SELL population | Results only for classified inferred events |
| 5s median \|r\| 0.51 bps; 16.5% > ±2 bps | Generally small response | Most obs below observational boundary |
| Mean signed ~+0.19 bps vs sign-shuffle | Detectable directional component | Economically tiny |
| Symbols dispersed | Not few-symbol dominated | Short 12:xx IST window |
| Inferred aggression | Exploratory response analysis | Not confirmed executed aggressor |

**Key issue:** Stage 2 must distinguish real mechanisms — not re-bucket a narrow inferred-aggression sample with weak price response.

---

## 1. Stage 2 hypotheses (research, not rules)

Three **mutually exclusive** primary mechanism hypotheses (plus explicit non-classification):

| ID | Hypothesis | Plain-language claim under test |
| --- | --- | --- |
| **A** | Absorption → reclaim → continuation | Aggressive opposing flow fails to advance price through a defended location; reclaim confirms; continuation follows |
| **B** | Breakout → acceptance → retest | Aggression **with** movement through a defined level; price holds acceptance; retest (or defined continuation) holds |
| **C** | Failed auction → rejection → reversal | Probe beyond a level with aggression fails; price returns; subsequent retest fails |
| **U** | `UNCLASSIFIED` / `UNAVAILABLE` | Evidence insufficient, conflicting, or data-quality fail — **valid primary outcome**, not a residual to force into A/B/C |

Secondary observations (liquidity replenishment, persistence counts, impact ratios) may attach to a primary label but **must not** replace it.

---

## 2. Observable evidence required per label

All labels require a **predeclared location context** (Section 3). Aggression alone + flat mid response is **never** sufficient for A/B/C.

### 2.1 Shared prerequisites (all of A/B/C)

| Requirement | Spec |
| --- | --- |
| Location | Explicit level type + price + formation rule (e.g. prior-day high/low, OR high/low, VWAP, session structure) |
| Aggression | Preference for **executed** aggressor when available; if only inferred, record `aggression_class=INFERRED` and raise the evidence bar (Section 4) |
| Sequence | Ordered event timeline with timestamps; no candle-colour reconstruction of aggressor |
| Response | Price path relative to the level **and** relative to aggression direction |
| Confirmation | Structure confirmation after the aggression/response episode |
| Invalidation | Predeclared structure break that would falsify the setup |
| Quality | Quote age, missing depth, session gaps within limits (Section 5) |

If any shared prerequisite fails → primary = `UNAVAILABLE` or `UNCLASSIFIED` (see 2.5).

### 2.2 Absorption (A) — evidence checklist

**LONG absorption example (SHORT symmetric):**

1. Price at/near predeclared **support** location (distance band TBD in execution auth).  
2. Aggressive **selling** (executed preferred) into that location.  
3. **Failure to make meaningful lower progress** despite selling (response metric vs location, not vs ±2 bps Stage-1 flat band alone).  
4. Selling pressure fades **or** price **reclaims** the defended level.  
5. Price **holds** reclaim and confirms upward continuation (horizon predeclared).  
6. Invalidation: decisive break below defended structure.

**Forbidden shortcut:** “negative delta / sell inference ⇒ buy absorption.”

### 2.3 Acceptance / continuation (B) — evidence checklist

**LONG example (SHORT symmetric):**

1. Price approaches predeclared **resistance** (or breakout level).  
2. Aggressive **buying** as price breaks the level.  
3. Price **actually advances** through/holds beyond the level (movement required).  
4. **Acceptance:** holds above level for predeclared persistence.  
5. Retest holds **or** alternate predeclared continuation confirm.  
6. Invalidation: acceptance failure (return and close/hold back inside).

**Forbidden shortcut:** L1 imbalance cross or inferred buy without movement/acceptance.

### 2.4 Failed auction (C) — evidence checklist

**SHORT example (LONG symmetric):**

1. Price tests above predeclared resistance / prior high.  
2. Aggressive buying on the probe.  
3. Break **fails** to sustain.  
4. Price returns below the level.  
5. Subsequent retest fails.  
6. Invalidation: genuine acceptance above the level.

**Forbidden shortcut:** “opposite OF / opposite inference ⇒ failed auction.”

### 2.5 `UNCLASSIFIED` / `UNAVAILABLE` (mandatory)

| Outcome | When |
| --- | --- |
| `UNAVAILABLE` | Missing location, missing aggression quality, sync/gap failure, or required fields absent |
| `UNCLASSIFIED` | Location+data present but sequence does not meet A, B, or C evidence gates; or A/B/C conflict |
| Partial secondary only | Allowed; primary remains U |

**Hard rule:** No silent coercion of U into A/B/C to raise “classified %.”

---

## 3. Location context (required for Stage 2 execution)

Stage 2 execution (when separately authorized) must bind each candidate to one location record:

| Field | Purpose |
| --- | --- |
| `location_type` | e.g. `PRIOR_SESSION_HIGH`, `OR_HIGH`, `VWAP`, `STRUCTURE_SWING`, … |
| `location_price` | Numeric level |
| `location_formed_utc` | When level became valid |
| `location_rule_id` | Versioned formation rule |
| `distance_bps_at_event` | Event mid vs level |

Until location plumbing exists and is validated, Stage 2 **classifier run remains blocked** even if aggression data improves.

---

## 4. Actual vs inferred aggression (separation)

| Class | Definition | Stage 2 use |
| --- | --- | --- |
| `EXECUTED` | Trade print with aggressor from exchange semantics or high-confidence tape+quote method with coverage gate | Eligible for A/B/C if other gates pass |
| `INFERRED` | Kite LTQ/LTT vs quote inference (current 0D/1 path) | May enter **exploratory** tallies only; **default primary = UNCLASSIFIED** unless a future auth explicitly allows inferred-A/B/C with stricter thresholds |
| `AMBIGUOUS` | Inside spread / low confidence | Not A/B/C |
| `UNAVAILABLE` | Missing trade/quote sync | Not A/B/C |

**Preparation mandate:** Stage 2 code/schema must carry `aggression_class` as a first-class field. Mixing EXECUTED and INFERRED in one unlabeled pool is forbidden.

NSE path today is predominantly `INFERRED` until trade+quote quality exceeds Stage 0D/1 bars **and** a separate auth upgrades eligibility. Crypto `aggTrade` (when captured) is a separate book — never pooled with NSE for mechanism rates.

---

## 5. Coverage & minimum sample conditions (gates before classifier run)

All must be true before any Stage 2 **execution** authorization:

| Gate | Minimum condition (initial proposal — freeze at execution auth) |
| --- | --- |
| **G2.1 Session span** | ≥ **3** distinct NSE cash sessions with capture spanning ≥ **120 IST minutes** each (not a single 12:xx window) |
| **G2.2 Symbol breadth** | ≥ **50** symbols with ≥ N qualifying location-linked events (N set at execution auth; placeholder N≥20) |
| **G2.3 Aggression quality** | Either (a) EXECUTED coverage ≥ declared % with confidence field, **or** (b) explicit auth to run inferred-only exploratory mode with primary default UNCLASSIFIED |
| **G2.4 Location coverage** | ≥ declared fraction of candidate events have valid `location_*` |
| **G2.5 Baseline** | Predeclared response baseline (e.g. sign-shuffle and/or location-matched non-event controls) registered **before** labeling |
| **G2.6 Immortality of Stage 1** | Stage 1 artifacts unchanged; Stage 2 writes new paths only |
| **G2.7 Non-promotion** | Config flag `stage2_labels_may_affect_execution=false` enforced; no OF paper / Estimand A imports |

Fail any gate → remain in preparation; do not run classifier.

---

## 6. Predeclared baselines (required at execution auth)

Before labeling:

1. **Response baseline** — magnitude/sign nulls (as in Stage 1 audit) on the Stage 2 cohort.  
2. **Location baseline** — outcomes at the same location **without** the aggression sequence.  
3. **Friction note** — any economic commentary must use gross/friction/net separately; Stage 2 prep does not claim net edge.

---

## 7. Output contract (when later authorized)

| Output | Rule |
| --- | --- |
| Primary label | Exactly one of A / B / C / UNCLASSIFIED / UNAVAILABLE |
| Evidence block | Checklist fields + pass/fail per requirement |
| Aggression class | EXECUTED / INFERRED / AMBIGUOUS / UNAVAILABLE |
| Stage 1 linkage | Optional foreign key; must not mutate Stage 1 rows |
| Paths | `artifacts/rp608_ofx/stage2_*` only |

---

## 8. Anti-patterns (explicit)

- Relabeling Stage 1 WITH/AGAINST/FLAT as absorption/acceptance/failed auction  
- Using ±2 bps Stage 1 flat band as a Stage 2 mechanism threshold  
- Threshold-swapping OF±0.30 because Stage 2 “looks promising”  
- Pooling NSE inferred with crypto maker/taker into one mechanism rate  
- Silent conversion of research labels into live/paper exits  

---

## 9. What would reopen Stage 2 **execution** review

Reconsider classifier authorization only after the project establishes:

1. Broader session coverage beyond a short midday window  
2. Adequate evidence schemas for A/B/C (this doc) **implemented and dry-run on fixtures** with high UNCLASSIFIED rates acceptable  
3. Clear EXECUTED vs INFERRED treatment in outputs  
4. Predeclared baselines registered before labels  
5. Written confirmation that labels cannot feed execution/OF config  

Until then: **design/readiness only.**

---

## 10. Governance freeze

| Item | State |
| --- | --- |
| Stage 1 conclusion | Locked `DOMINANTLY_FLAT_SHORT_HORIZON` |
| Stage 2 mechanism observer run | **Not authorized** |
| Stage 2 design/readiness | **Authorized (this document)** |
| Four-book baseline | Frozen |
| Live order authority | 0% |
| Step 3+ / Gate R | Held / unreached |
