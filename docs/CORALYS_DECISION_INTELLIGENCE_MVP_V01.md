# CORALYS DECISION INTELLIGENCE — MVP v0.1

**Document ID:** CORALYS_DECISION_INTELLIGENCE_MVP_V01  
**Document type:** Product / MVP specification  
**Status:** Draft — implementation specification  
**Date:** 2026-08-17  
**Supersedes for MVP implementation:** the product-layer portions of CS-P-006-P — Decision Observatory  
**Parent research protocol:** CS-P-006  
**Related product protocol:** CS-P-006-P — Decision Observatory  
**Primary product:** Coralys Decision Intelligence  
**Execution principle:** User-controlled execution; Coralys does not allocate capital or determine quantity.

---

## 1. Purpose

This document defines the first actionable product built around the certified Coralys decision framework.

The product is intentionally narrower than an autonomous trading system.

Its purpose is:

> **Turn a certified Coralys decision into an immutable, inspectable decision record that a user can evaluate, optionally execute, and later reconcile with the observed outcome.**

The product preserves the temporal boundary established by CS-P-006-P:

```text
Information available at T
        ↓
Certified state
        ↓
Coralys
        ↓
Decision
        ↓
Certification / Seal
        ↓
USER DECIDES WHETHER / HOW TO EXECUTE
        ↓
Observation
        ↓
Outcome
```

Coralys does **not** infer user behaviour, allocate portfolio capital, rank opportunities for capital deployment, or autonomously execute trades.

---

## 2. Relationship to CS-P-006-P

The existing [`CS-P-006-P — Decision Observatory`](CS-P-006-P_DECISION_OBSERVATORY.md) established the product-layer principle around the frozen CS-P-006 research loop.

It demonstrated the historical lifecycle:

```text
C3-002 → Decision → Seal → Observe → Outcome → Measure
```

and explicitly preserved the temporal firewall: the sealed decision contains no future return, outcome, regret, evaluation score, or future-derived confidence. Observation is appended later. The historical path achieved **91/91 lifecycle PASS**, while prospective operation remained OBSERVING.

The new MVP does **not** invalidate those research findings.

It narrows and operationalizes the product boundary:

```text
CS-P-006
   │
   └── certified research / policy layer
              ↓
       Coralys Decision
              ↓
   Coralys Decision Intelligence
              │
              ├── Decision Ledger
              ├── Decision Feed
              ├── Decision Detail
              ├── User Execution Record
              └── Outcome / Evidence
```

The old Observatory concepts that are not required for this MVP are not part of the new implementation target.

---

## 3. Product Proposition

Customer-facing proposition:

> **Coralys identifies decision opportunities from information available at the time, records the decision and its provenance, and preserves the evidence that emerges afterward without hindsight contamination.**

This is **not** positioned as: "Our AI predicts stocks."

The product exposes decision evidence rather than making unsupported profitability or confidence claims.

---

## 4. Product Boundary

### 4.1 Coralys is responsible for

- interpreting certified decision-time state
- producing the certified decision
- producing the reference risk boundary currently defined by the frozen Coralys execution artifact
- preserving policy/artifact provenance
- producing a deterministic decision record

### 4.2 The user is responsible for

- deciding whether to act
- deciding quantity
- deciding portfolio allocation
- deciding whether to use cash, margin, or another execution method
- recording or connecting the actual execution
- closing or modifying the position in real life

### 4.3 ChronoSentiment / product layer is responsible for

- presenting certified decisions
- maintaining the immutable decision ledger
- recording user execution events
- recording subsequent observations/outcomes
- exposing historical evidence
- maintaining temporal and provenance integrity

### 4.4 Explicitly out of scope

The MVP does **not**:

- allocate capital
- rank decisions by capital priority
- recommend quantity
- simulate user behaviour
- automatically execute trades
- integrate with a broker
- optimize stop-loss parameters
- claim an optimal stop
- generate a confidence percentage
- infer profitability from backtest return
- replace the Coralys research protocol
- modify C3-002 or other frozen research artifacts

---

## 5. Core Product Object — Decision Record

The fundamental product object is an immutable `DecisionRecord`.

Conceptual structure:

```text
DecisionRecord
│
├── Identity
├── Certification
├── Decision
├── Reference Risk
├── Execution
├── Outcome
└── Evidence
```

The original decision is **immutable** after certification.

Later events do not modify the historical decision; they append lifecycle information.

---

## 6. Identity

Minimum fields:

```json
{
  "decision_id": "coralys-ADANIENT-20260817T101500Z-001",
  "instrument": "ADANIENT.NS",
  "decision_timestamp": "2026-08-17T10:15:00Z"
}
```

**Requirements:**

`decision_id` must be:
- unique
- deterministic or reproducibly derivable
- immutable
- suitable for joining all subsequent lifecycle events

The decision timestamp is the authoritative temporal boundary.

---

## 7. Certification

Example:

```json
{
  "certification": {
    "status": "CERTIFIED",
    "policy_artifact_hash": "3876ffa2...",
    "decision_pipeline": "C3-002",
    "certified_timestamp": "2026-08-17T10:15:00Z",
    "data_snapshot": "..."
  }
}
```

The certification record must allow an independent reviewer to answer:

> **Exactly which Coralys artifact, decision pipeline and data state produced this decision?**

**Certification invariants:**

A certified decision must have:
- decision timestamp
- policy artifact identity/hash
- source data/snapshot identity
- decision pipeline identity
- certification status

A decision that cannot establish its provenance must **not** be marked `CERTIFIED`.

---

## 8. Decision

The ledger stores the output of the canonical Coralys decision pipeline.

Example:

```json
{
  "decision": {
    "direction": "LONG",
    "trend": "Bullish",
    "momentum": "Positive",
    "volatility": "present",
    "target_price": 1234.50
  }
}
```

The MVP must **not** manufacture fields that Coralys does not currently produce.

In particular, the MVP must not invent:
- confidence
- probability of success
- expected return
- ranking score
- quality score

The product records the certified decision; research determines whether additional decision attributes are justified later.

---

## 9. Reference Risk Boundary

The current Coralys execution boundary is exposed as a **Reference Risk Boundary**.

Example:

```json
{
  "reference_risk": {
    "boundary_price": 1180.25,
    "boundary_type": "CORALYS_V0_ATR_TMV",
    "status": "REFERENCE"
  }
}
```

This naming is deliberate.

The MVP must **not** describe the current boundary as:
- optimal stop
- statistically optimal stop
- best stop
- guaranteed protection

The stop-loss research programme remains a separate evidence track.

Future research may replace or refine the reference boundary without changing the core `DecisionRecord` schema.

---

## 10. User Execution

Execution is external to Coralys decision generation.

Initial state:

```json
{
  "execution": {
    "status": "NOT_RECORDED"
  }
}
```

Supported lifecycle states:

```text
NOT_RECORDED
USER_IGNORED
USER_EXECUTED
USER_CANCELLED
```

If the user records an execution:

```json
{
  "execution": {
    "status": "USER_EXECUTED",
    "execution_timestamp": "2026-08-17T10:20:00Z",
    "quantity": null,
    "execution_price": null,
    "execution_source": "USER"
  }
}
```

Quantity and execution price are populated only when actual user/external execution data exists.

**Critical invariant:**

Coralys must **never** infer quantity or allocation from:
- universe size
- recommendation rank
- conviction
- available capital
- historical return
- signal density

---

## 11. Outcome

Outcome is temporally subsequent to the decision.

Example:

```json
{
  "outcome": {
    "status": "OPEN",
    "exit_reason": null,
    "exit_timestamp": null,
    "exit_price": null,
    "realized_pnl": null
  }
}
```

Possible product-level states:

```text
OPEN
TARGET
REFERENCE_RISK
HORIZON
USER_CLOSED
```

The exact outcome semantics must remain those of the certified evaluation layer.

**Temporal invariant:**

An outcome must never be present on the original certified decision object before its observation boundary has passed.

---

## 12. Evidence

Evidence is an enrichment layer and must **not** contaminate the original decision.

Conceptual structure:

```json
{
  "evidence": {
    "similar_decisions_count": null,
    "median_mae_pct": null,
    "p90_mae_pct": null,
    "median_mfe_pct": null,
    "median_time_to_target_sessions": null
  }
}
```

Fields remain unavailable until supported by validated research datasets.

The current stop research dataset is an evidence/research asset, not a source of new decision-time rules.

---

## 13. Event / Lifecycle Model

The preferred internal model is **append-only lifecycle events**.

Conceptually:

```text
DecisionCreated
      ↓
DecisionCertified
      ↓
UserExecutionRecorded
      ↓
ReferenceRiskReached / TargetReached / HorizonReached
      ↓
DecisionClosed
```

The current materialized `DecisionRecord` may be derived from these events.

**Principle:**

> The decision is immutable; the lifecycle is append-only.

This preserves the distinction between:
- what Coralys knew at T
- what the user subsequently did
- what the market subsequently did

---

## 14. MVP Screens

The MVP requires only three primary user surfaces.

### 14.1 Decision Feed

Purpose: show current certified decisions.

Minimum information:
- Timestamp
- Instrument
- Direction
- Decision status
- Reference risk
- Target
- Certification status

Example:

```text
CORALYS DECISIONS

10:15  ADANIENT   LONG
       Bullish / Positive
       Target ₹XXXX
       Reference Risk ₹XXXX
       CERTIFIED

10:15  BPCL       LONG
       Bullish / Positive
       Target ₹XXX
       Reference Risk ₹XXX
       CERTIFIED
```

No capital ranking is shown.

### 14.2 Decision Detail

Minimum sections:
- Instrument
- Direction
- Decision timestamp
- Certification
- Coralys state
- Target
- Reference Risk
- Historical Evidence
- Lifecycle
- User Action

The screen must make temporal provenance visible.

### 14.3 Decision History

Minimum fields:
- Date
- Instrument
- Direction
- Decision status
- User action
- Outcome

Example:

```text
17 Aug  ADANIENT  LONG   Certified  Executed  OPEN
17 Aug  BPCL      LONG   Certified  Ignored   —
16 Aug  INFY      SHORT  Certified  Executed  TARGET
```

This becomes the user's decision memory.

---

## 15. Product API

The first API surface should be intentionally small.

**Decisions:**

```text
GET  /decisions
GET  /decisions/{decision_id}
```

**Execution:**

```text
POST /decisions/{decision_id}/execution
```

**Outcome:**

```text
GET  /decisions/{decision_id}/outcome
POST /decisions/{decision_id}/outcome
```

Outcome creation must be restricted to authorized observation processes.

**Evidence:**

```text
GET  /decisions/{decision_id}/evidence
```

No allocation endpoint is required.  
No ranking endpoint is required.  
No autonomous order endpoint is required.

---

## 16. Temporal Integrity Requirements

Temporal integrity is a **hard product invariant**.

For decision timestamp T:

```text
Decision information <= T
```

Anything generated after T belongs to observation/evidence.

**Forbidden inside the certified decision:**
- future price
- future return
- future target hit
- future stop hit
- future regret
- future evaluation score
- future-derived confidence
- future-derived ranking

This preserves the core CS-P-006-P rule that **outcomes never construct the decision**.

---

## 17. Reproducibility

A certified decision must be reproducible from its provenance.

Minimum reproducibility tuple:

```text
policy artifact hash
+
source data snapshot
+
decision timestamp
+
instrument
+
decision pipeline version
```

A reproducibility failure is a **product integrity failure**, not merely a diagnostic warning.

---

## 18. Research Separation

The product and research layers remain separate.

```text
PRODUCT LAYER
──────────────────────────────
Decision Ledger
Decision UI
User Execution
Outcome Recording
Evidence Presentation

RESEARCH LAYER
──────────────────────────────
stop_research_dataset_v01
MAE/MFE research
Regime research
Counterfactual stop policies
Future Coralys execution research
```

The research layer may produce evidence that later becomes product capability.

The product layer must **not** silently turn research observations into strategy rules.

---

## 19. Stop-Loss Research Boundary

The current stop behaviour is an observed property of Coralys v0.

The existing evidence establishes that stop behaviour varies across realized populations and that MaxPerLot changes the population of realized trades rather than demonstrably improving the stop mechanism.

Therefore:

```text
Current Coralys v0 risk boundary
        ↓
REFERENCE RISK BOUNDARY
        ↓
Stop Research
        ↓
Evidence
        ↓
Future validated risk boundary
```

No stop optimisation is part of MVP v0.1.

---

## 20. MVP Acceptance Criteria

### AC-01 — Certified decision

Every product decision has:
- instrument
- timestamp
- direction/action
- policy artifact
- provenance
- certification status

### AC-02 — Temporal firewall

No information after the decision timestamp can enter the certified decision.

### AC-03 — Immutable decision

Once certified, the decision cannot be overwritten.

### AC-04 — User-controlled execution

The user can record whether they acted without Coralys determining quantity or allocation.

### AC-05 — Outcome separation

Outcome information is appended only after the relevant observation boundary.

### AC-06 — Reproducibility

A certified decision can be reconstructed from its provenance tuple.

### AC-07 — No invented confidence

The product does not display unsupported confidence percentages.

### AC-08 — No allocation

The MVP contains no capital-allocation or portfolio-ranking mechanism.

### AC-09 — Evidence separation

Research-derived evidence cannot modify the original decision.

### AC-10 — Lifecycle auditability

A reviewer can reconstruct:

```text
what Coralys knew
→ what Coralys decided
→ what the user did
→ what subsequently happened
```

---

## 21. Implementation Sequence

```text
MVP-001  Freeze DecisionRecord schema
MVP-002  Implement immutable Decision Ledger
MVP-003  Connect canonical Coralys decision pipeline
MVP-004  Implement certification + provenance
MVP-005  Decision Feed API
MVP-006  Decision Detail API
MVP-007  User execution recording
MVP-008  Outcome recording
MVP-009  Minimal Decision UI
MVP-010  Temporal and reproducibility tests
```

Only after MVP-010 should research enrichment be connected.

---

## 22. Explicitly Deferred

The following are future milestones, not MVP requirements:

- Stop-policy discovery
- ATR-normalised stop research
- Regime-conditioned risk boundary
- Broker integration
- Autonomous execution
- Portfolio allocation
- Capital optimisation
- Decision ranking
- Personalised user modelling
- Predictive probability/confidence
- Advanced recommendation explanations

---

## 23. Product Success Criterion

The MVP is successful if a real user can take a certified Coralys decision and answer four questions:

1. What did Coralys decide?
2. What did Coralys know when it decided?
3. What did I choose to do?
4. What happened afterward?

The product does **not** need to prove profitability to satisfy this milestone.

It needs to prove **decision integrity, provenance, temporal correctness and usable decision memory**.

---

## 24. Architectural Principle

The governing principle is:

> **Coralys discovers. The product certifies and records. The user decides whether and how to execute. The observation layer records what happens afterward.**

This preserves the distinction between:
- Decision
- Execution
- Outcome

and prevents portfolio allocation or user behaviour from being mistaken for Coralys decision intelligence.

---

## 25. Status

**MVP v0.1 — SPECIFICATION COMPLETE / IMPLEMENTATION NOT YET FROZEN**

This document is the implementation target for the first actionable Coralys Decision Intelligence product.

The existing [`CS-P-006-P — Decision Observatory`](CS-P-006-P_DECISION_OBSERVATORY.md) remains historical research/product evidence. Its validated temporal lifecycle findings are retained, while obsolete or superseded implementation paths should be removed only after repository-level dependency verification.