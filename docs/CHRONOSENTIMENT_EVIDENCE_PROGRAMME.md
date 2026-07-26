# ChronoSentiment — Evidence Programme Structure

**Document type:** Programme Planning
**Status:** Draft — Not yet started
**Date:** 2026-07-23
**Predecessor methodology:** UltraCrew Evidence Portfolio (M-series, P-001, WS-001)

---

## Purpose

This document defines the evidence programme structure for ChronoSentiment, applying the same evidence-driven product development methodology established for UltraCrew.

The methodology separates three things that are often conflated:

- **Product demonstration:** "Here is what a user experiences."
- **Technical evidence:** "Here is objective evidence that the engine performs as claimed."
- **Governance:** "Here is how we know those claims are reproducible."

ChronoSentiment is a financial decision intelligence platform. Its evidence portfolio must demonstrate not trading profitability, but **deterministic analysis, reproducible recommendations, explainable decision support, and consistent decision reconstruction**.

---

## Evidence Hierarchy

```
Architecture Evidence (M-series)
        │
        ▼
Production Readiness (P-001 / Atlas Capital)
        │
        ▼
Decision Evidence (DS-001 / Canonical Dataset)
        │
        ▼
Future Domain Evidence
        (Macro, Execution, Portfolio, Risk)
```

Each level answers a distinct question. No two programmes duplicate each other's evidence.

---

## Programme Inventory

### M-Series — Architecture Evidence

**Status:** Not started
**Analogous to:** UltraCrew M-series (M6.5–M6.7)

**Questions answered:**
- Is the architecture coherent and governed?
- Is the observatory model sound?
- Are the core APIs stable and versioned?
- Is explainability designed in from the start, not retrofitted?
- Is the decision lineage model auditable?

**Governance:** Milestone validation reports, one per architectural claim. Each frozen independently.

**Exit criteria:** All architectural claims have a corresponding validation report. No open architectural questions.

---

### P-001 — Production Readiness

**Status:** Not started
**Analogous to:** UltraCrew P-001 (SunAir)

**Canonical demonstration entity:** Atlas Capital (or BlueRiver Investments)

A realistic investment firm with a defined universe of securities, a fixed historical period, and a representative set of research and decision workflows.

**Deliverables:**
- Portfolio dashboard — holdings, P&L, exposure, risk metrics
- Research workspace — sentiment timeline, event overlay, signal history
- Decision timeline — chronological record of recommendations and outcomes
- Execution replay — deterministic reconstruction of a past decision
- Recommendation engine — current signals with confidence and rationale
- Explainability — natural-language explanation of each recommendation
- Scenario comparison — side-by-side comparison of two analysis runs

**Exit criteria:** All seven deliverables browser-verified. Canonical dataset deterministic (fixed seed / fixed data snapshot). Hard explainability violations = 0.

---

### DS-001 — Decision Evidence

**Status:** Not started
**Analogous to:** UltraCrew WS-001 (INRC)

**Key difference from UltraCrew:** No universally accepted public benchmark exists for financial decision intelligence. The canonical evidence dataset must therefore be defined explicitly rather than inherited from a competition.

**Canonical evidence dataset definition:**

| Dimension | Specification |
|-----------|--------------|
| Securities universe | Fixed set of 20–30 liquid equities (e.g. S&P 500 constituents) |
| Historical period | Fixed 12-month window (e.g. 2024-01-01 to 2024-12-31) |
| Data sources | Public OHLCV, earnings calendars, SEC filings, macro releases |
| Evaluation protocol | Fixed: deterministic replay, recommendation consistency, explanation coverage |
| Seed / snapshot | Versioned data snapshot, reproducible by any evaluator |

**The goal is not to claim "best trading system."** The goal is to show that, given the same information, ChronoSentiment produces the same analysis and exposes its reasoning consistently.

**Deliverables:**
- DS-E1: Canonical dataset (JSON/CSV, versioned)
- DS-E2: Deterministic replay report — same input → same output, every run
- DS-E3: Recommendation consistency report — signal stability across runs
- DS-E4: Explanation coverage report — % of recommendations with full NL explanation
- DS-E5: Decision Evidence dashboard (HTML, self-contained)
- DS-E6: Executive evidence document (`CHRONOSENTIMENT_DECISION_EVIDENCE.md`)
- DS-E7: Technical evidence document (`DECISION_BENCHMARK_RESULTS.md`)
- DS-E8: Demo guide (`CHRONOSENTIMENT_DEMO_GUIDE.md`)

**Exit criteria:** Deterministic replay confirmed (same output across 3 independent runs). Explanation coverage ≥ 95%. No hard consistency violations.

---

### Future Domain Evidence

**Status:** Not started
**Analogous to:** UltraCrew RD-001, HC-001, FS-001

| Programme | Domain | Primary question |
|-----------|--------|-----------------|
| ME-001 | Macro event evidence | Does ChronoSentiment correctly identify and respond to macro events? |
| EX-001 | Execution evidence | Does ChronoSentiment support execution-quality decision timing? |
| PF-001 | Portfolio evidence | Does ChronoSentiment support portfolio-level decision coherence? |
| RK-001 | Risk evidence | Does ChronoSentiment correctly identify and flag risk events? |

Each future programme inherits the governance pattern from DS-001 without redesign.

---

## Governance Principles (inherited from UltraCrew)

| Rule | Description |
|------|-------------|
| Evidence first | Deliverables accepted only after working demonstrations. |
| Canonical dataset | All demonstrations use the versioned canonical dataset unless otherwise noted. |
| Determinism | All runs must be reproducible from the same input. Seed or data snapshot must be fixed. |
| Regression | DS-E2 canonical run establishes the baseline; all subsequent runs must meet or exceed it. |
| Domain separation | Decision evidence must not reference scheduling terminology. Financial-domain language throughout. |
| Completion | Exit criteria govern completion, not percentage complete. |

---

## One Important Distinction

UltraCrew's evidence was grounded in an externally defined benchmark (INRC). ChronoSentiment's evidence must be grounded in a **self-defined but publicly verifiable canonical dataset**.

This means the evidence programme must be more explicit about:

1. **What data was used** — exact securities, exact date range, exact data sources.
2. **What was measured** — recommendation consistency, explanation coverage, replay accuracy.
3. **What was not claimed** — trading profitability, alpha generation, market outperformance.

The evidence demonstrates that ChronoSentiment is a **reliable, explainable, reproducible decision intelligence system** — not that it predicts markets.

---

## Relationship to UltraCrew

```
UltraCrew Evidence Portfolio
        │
        ├── M-series  Architecture
        ├── P-001     Production Readiness (SunAir)
        └── WS-001    Workforce Benchmark (INRC)  ✅ frozen

ChronoSentiment Evidence Portfolio
        │
        ├── M-series  Architecture
        ├── P-001     Production Readiness (Atlas Capital)
        └── DS-001    Decision Evidence (Canonical Dataset)
```

The governance model is shared. The evidence is independent. The two portfolios together demonstrate that the underlying evidence-driven methodology is itself domain-agnostic.

---

## Next Steps

1. **Define the canonical dataset** — select securities universe, historical period, data sources, and evaluation protocol. Version and freeze.
2. **Start M-series** — architecture validation reports for observatory model, API stability, decision lineage, explainability design.
3. **Start P-001** — Atlas Capital demonstration entity, seven deliverables, browser-verified.
4. **Start DS-001** — canonical dataset evidence, eight deliverables, determinism and explanation coverage verified.

Do not start DS-001 before P-001 is frozen. The production demonstration must precede the independent evidence programme, as it did for UltraCrew.