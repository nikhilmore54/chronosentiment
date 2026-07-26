# UltraCrew Demo Assessment v1.0

> **Status**: Assessment v1.0 — frozen
> **Date**: 2026-07-20
> **Method**: Live browser inspection of the running application at http://localhost:3000
> **Assessor perspective**: Pilot customer (operations manager), product architect
> **Scope**: All four tabs of the running UI — Run GA, Inspect Strategy, Compare Strategies, Global Ranking

---

## Critical Finding — Wrong Product

**The running UI is not UltraCrew. It is ChronoSentiment.**

The application at localhost:3000 is the ChronoSentiment financial strategy execution platform. Every element of the UI confirms this:

| Element | What it shows |
|---|---|
| Header | "ChronoSentiment · Execution Intelligence Platform" |
| Footer | "ChronoSentiment · NSE Execution Intelligence · v2026" |
| Tab 1 | "Run GA" — Population, Generations, Mutation, Seed, Signal Filters, Top-K per asset, Include weak signals |
| Tab 2 | "Inspect Strategy" — Strategy ID, Seed, Causal reconstruction, Reconstruct trace |
| Tab 3 | "Compare Strategies" — Strategy IDs (comma-separated), Execute comparison |
| Tab 4 | "Global Ranking" — Avg PnL, Std Dev, Exec Fitness, GA Fitness, Classification |
| Status bar | Phase, Throttle, Cohort, Queue, Fill latency, Sync ratio, EPS |

None of these concepts — PnL, execution fitness, signal filters, causal reconstruction, cohort, throttle state — have any relationship to workforce scheduling.

**There is no UltraCrew frontend.** The UltraCrew product exists only as a Rust backend API. The frontend that exists is a separate product (ChronoSentiment) that happens to live in the same repository.

---

## Scorecard

| Area | Score | Notes |
|---|---|---|
| First Impression (as UltraCrew) | 0/10 | Shows wrong product entirely |
| Workflow (Import → Validate → Generate → Review → Export) | 0/10 | Workflow does not exist in this UI |
| Usability | N/A | Cannot assess workforce scheduling usability |
| Decision Support | N/A | No scheduling decision support present |
| Pilot Readiness | 0/10 | Cannot be shown to a workforce scheduling customer |
| Commercial Readiness (as UltraCrew) | 0/10 | Wrong product |
| ChronoSentiment UI quality (on its own terms) | 7/10 | Clean, professional, well-structured for its actual purpose |

---

## ChronoSentiment UI — Observed Quality (on its own terms)

This is not a criticism of the ChronoSentiment UI. Assessed as a financial strategy execution tool, it is well-built:

**Strengths:**
- Clean dark-neutral design system with consistent typography and spacing
- Persistent left-rail navigation with clear workspace labels
- Operational awareness strip (Phase, Throttle, Cohort, Queue, Fill latency, Sync, EPS) — appropriate for a trading system
- Live IST clock in the header
- Graceful offline states — "Observatory unavailable", "No trace loaded", "No ranking data", "Failed to fetch" — all with clear explanatory text
- Consistent empty-state patterns across all four tabs
- Breadcrumb in the top-right showing active workspace

**Weaknesses (minor, for ChronoSentiment):**
- All four tabs show empty/offline states because the ChronoSentiment backend is not running — expected
- "Store unavailable" error on Run GA tab is shown in amber text without further guidance

---

## What This Means for UltraCrew

### The UltraCrew product has no frontend.

The backend API is complete (Phase A). The frontend does not exist. A pilot customer cannot interact with UltraCrew through a browser.

### Options

**Option 1 — Build a UltraCrew frontend (recommended for pilot)**

A minimal workforce scheduling UI needs:
- Import screen: upload workers CSV + shifts CSV, or paste JSON
- Validate screen: show constraint violations before optimising
- Generate screen: trigger `POST /api/schedule`, show progress
- Results screen: roster grid (workers × shifts), constraint report, recommendations
- Export screen: download JSON or CSV

This is a 2–3 week engineering effort for a functional pilot UI. It does not need to be polished — it needs to be usable by a planner.

**Option 2 — API-only pilot**

Run the pilot entirely through the REST API with a technical contact at the customer. The customer provides data as CSV/JSON, UltraCrew returns results as JSON/CSV. No UI required. This is viable for a first pilot with a technically capable customer.

**Option 3 — Repurpose ChronoSentiment UI**

The ChronoSentiment UI has a good design system and component structure. It could be adapted for workforce scheduling, but the domain concepts are completely different (PnL vs. roster, strategies vs. workers, signals vs. shifts). Repurposing would require replacing essentially all components. Starting fresh is likely faster.

---

## Readiness by Dimension

The single "0/10" pilot readiness score above applies only to browser-based customer interaction. The backend is substantially more mature. A more precise breakdown:

| Dimension | Assessment |
|---|---|
| Backend Readiness | High — optimization pipeline, constraints, recommendations, telemetry all implemented |
| API Readiness | High — POST /api/schedule, /api/reschedule, /api/validate, export endpoints all live |
| Algorithm Readiness | High — INRC-II validated, 6 constraint types, fairness and fatigue optimization |
| Demo Readiness (browser) | Low — no UltraCrew UI exists |
| Planner Self-Service Readiness | Low — no UI for independent planner operation |
| Pilot Readiness (API-assisted) | High — CSV import/export implemented, REST API live |
| Pilot Readiness (planner-operated) | Moderate — requires minimal UI before planners can operate independently |

## Recommendation

**Verdict: Not ready for a browser demo. Ready for an API-assisted pilot.**

The distinction matters. Customer meetings and planner-operated pilots have different requirements.

**Customer discovery / initial pilot (API-assisted):** No UI required. The workflow is:

```
Customer CSV
      ↓
GenericImporter (generic_import.rs)
      ↓
POST /api/schedule
      ↓
CSV Output (generic_export.rs)
      ↓
Excel review together
```

This is how many enterprise B2B scheduling pilots begin. The value proposition is demonstrated through the quality of the output, not the polish of the interface. Do not delay customer engagement to build a UI.

**Planner-operated pilot:** Once planners are expected to operate the system independently, a minimal UI becomes necessary. Build it after observing how planners actually work in the API-assisted phase — not before. The UI should remove friction that is observed, not friction that is assumed.

**Do not spend 2–3 weeks building a frontend before talking to customers.** The sequence should be: find a pilot customer → run scheduling exercises via API → observe planner workflow → build the smallest UI that removes the friction you observe. This keeps the UI grounded in real workflows rather than assumptions, and is consistent with the evidence-driven governance model applied throughout this project.

---

## Quick Wins (1–2 days, if building a UI)

1. Single-page import form: workers CSV upload + shifts CSV upload + "Generate Schedule" button
2. Results table: shift_id | worker_id | worker_skills | constraint_status
3. Summary panel: fitness score, hard violations count, fairness penalty
4. Download button: export as CSV

That is the minimum viable UltraCrew UI for a pilot demo.

---

## Before First Customer Demo (if UI is required)

1. Build the minimal import → generate → results → export flow described above
2. Replace all ChronoSentiment branding with UltraCrew branding
3. Connect to the live `POST /api/schedule` endpoint
4. Test with the INRC sample data to verify end-to-end flow in the browser

---

*This assessment answers: "If a customer sat in front of this application tomorrow, what would they think?" Answer: They would see a financial trading platform, not a workforce scheduling tool. The UltraCrew backend is pilot-ready. The UltraCrew frontend does not yet exist.*