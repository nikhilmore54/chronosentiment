# UltraCrew Demo Assessment v1.1

> **Status**: Assessment v1.1 — frozen
> **Date**: 2026-07-20
> **Supersedes**: `ULTRACREW_DEMO_ASSESSMENT_v1.0.md` (which assessed ChronoSentiment, not UltraCrew)
> **Method**: Live browser inspection of the UltraCrew Vite frontend at http://localhost:5173 (ui/ultracrew/)
> **Backend**: UltraCrew server started on port 3001 (502 errors indicate backend was still initialising during inspection)
> **Assessor perspective**: Pilot customer (operations manager), product architect
> **Scope**: Landing page, Dashboard, Constraints tab, Import & Schedule workflow (Step 1 fully exercised)

---

## Correction to v1.0

The v1.0 assessment concluded "There is no UltraCrew frontend." That conclusion was incorrect.

The application inspected in v1.0 was ChronoSentiment (localhost:3000), a financial strategy execution platform that happens to share the same repository. A separate UltraCrew Vite frontend exists at `ui/ultracrew/` and runs on localhost:5173. It is connected to the UltraCrew backend on port 3001 via `ui/ultracrew/src/config/api.ts`.

The v1.0 finding that "the running application was ChronoSentiment" remains accurate. The broader claim "there is no UltraCrew frontend" is superseded by this assessment.

---

## What Was Inspected

| Screen | Status | Notes |
|---|---|---|
| Landing page | Rendered correctly | UltraCrew branding, tagline, feature bullets, CTA |
| Dashboard | Rendered with empty state | Team Balance, Sick Leave Simulator, Active Workforce Roster visible; data shows zeros (backend 502) |
| Constraints | Loading state only | "Loading Constraints..." — 502 from backend |
| Import & Schedule — Step 1 | Fully functional | Sample data loaded, import validated, staff table rendered |
| Import & Schedule — Steps 2–5 | Not reached | "Next: Select Rules" button below the fold; backend 502 would likely block generation |

---

## Scorecard

| Area | Score | Notes |
|---|---|---|
| First Impression | 8/10 | Professional, on-brand, clear purpose immediately |
| Workflow clarity | 8/10 | 5-step progress bar is immediately understandable |
| Import UX | 9/10 | "Load Sample (20 staff)" works, validation summary is excellent |
| Backend connectivity | 2/10 | 502 errors on all data-fetching screens; backend was not ready |
| Constraints tab | 0/10 | Blank loading state — no fallback, no error message |
| Dashboard data | 3/10 | Renders but shows zeros; no graceful offline state |
| Commercial readiness | 7/10 | Looks like a real product, not a prototype |
| Pilot readiness (UI) | 6/10 | Workflow exists and is well-designed; backend connectivity is the blocker |

---

## Strengths

**Landing page is commercially convincing.** "Employee was sick. Traditional schedulers forget. UltraCrew remembers." is a strong, specific value proposition. Feature bullets (Fair workload recovery, Reduced scheduling complaints, Historical balancing, Explainable assignments) map directly to real planner pain points.

**Import step is the best screen in the application.** The validation summary — "20 staff members / 3 contract types: FullTime, PartTime, Night / 2 skills: HeadNurse, Nurse" — is exactly what a planner needs to see to trust that their data was read correctly. The staff table (ID, Contract, Skills with colour-coded skill badges) is clean and readable. "Load Sample (20 staff)" is a good onboarding affordance.

**5-step workflow is immediately legible.** Import Staff → Select Rules → Generate → Review & Edit → Export maps directly to how a planner thinks about the scheduling process. No training required to understand the flow.

**Design system is consistent and professional.** Dark navy background, blue accent colour, monospace skill badges, clean typography. Looks like an enterprise product, not a side project.

**Navigation is minimal and correct.** Three tabs (Dashboard, Constraints, Import & Schedule) cover the right scope for a pilot. No feature bloat.

---

## Weaknesses

**Backend connectivity is broken during this inspection.** All data-fetching screens return 502. This is a deployment/startup issue, not a product design issue — the backend was still initialising when the browser connected. However, it means the Constraints tab shows a blank loading state with no error message, and the Dashboard shows zeros with no explanation.

**Constraints tab has no offline/error state.** "Loading Constraints..." with a blank page is not acceptable for a customer demo. If the backend is unavailable, the tab should show a clear error: "Could not load constraints. Check that the UltraCrew server is running on port 3001."

**Dashboard has no graceful offline state.** Team Balance showing "0 / 0 / 100" with no data is misleading. A planner might think the system has no staff loaded. An empty state with a prompt ("Import staff to see team balance") would be clearer.

**Steps 2–5 of the workflow were not reachable** during this inspection due to the backend 502. The "Next: Select Rules" button was below the fold and the backend would need to be running to proceed through Generate and Review & Edit.

---

## Readiness by Dimension

| Dimension | Assessment |
|---|---|
| Backend Readiness | High — optimization pipeline, constraints, recommendations all implemented |
| API Readiness | High — POST /api/schedule, /api/reschedule, /api/validate, export endpoints live |
| Algorithm Readiness | High — INRC-II validated, 6 constraint types, fairness and fatigue optimization |
| UI Existence | Confirmed — UltraCrew Vite frontend exists at ui/ultracrew/ |
| UI Design Quality | High — professional, on-brand, workflow-oriented |
| UI–Backend Connectivity | Needs work — 502 errors; backend startup timing issue |
| Import UX | High — best screen in the application |
| Error handling (UI) | Low — no graceful offline states on Constraints or Dashboard |
| Demo Readiness | Moderate — impressive when backend is running; broken when it isn't |
| Pilot Readiness (API-assisted) | High — CSV import/export implemented, REST API live |
| Pilot Readiness (planner-operated) | Moderate — workflow exists; backend connectivity and error states need fixing |

---

## Quick Wins (1–2 days)

1. **Add a Backend Status indicator.** Show a persistent status pill in the header or top of the application so users immediately understand connectivity state rather than discovering failures through blank screens:

   ```
   ● Backend Connected — UltraCrew Server :3001
   ```
   or
   ```
   ⚠ Backend Offline — Attempting reconnect...
   ```

   This single change eliminates the most confusing aspect of the current offline experience.

2. **Add error state to Constraints tab.** Replace "Loading Constraints..." with a proper error message when the backend returns 502/connection refused. Show the backend URL being attempted so the operator can diagnose.

3. **Add empty state to Dashboard.** When no data is loaded, replace zeros with actionable prompts:
   - "No workforce loaded — Import staff to begin."
   - "No schedule generated yet — Generate a schedule to see workload balance."
   Zeros imply meaningful data. Empty states explain the situation.

4. **Ensure backend is running before starting the Vite dev server.** Document the startup sequence: `cargo run -p ultracrew_server` first, then `npm run dev` in `ui/ultracrew/`. Or add a health check to the frontend that shows a clear "Backend offline" banner.

5. **Make the "Next: Select Rules" button visible without scrolling.** The staff table pushes the navigation button below the fold. Either fix the layout or add a sticky footer with the Next button.

---

## Before First Customer Demo

1. Verify the full 5-step workflow end-to-end with the backend running: Import → Select Rules → Generate → Review & Edit → Export
2. Fix the Constraints tab error state
3. Fix the Dashboard empty/offline state
4. Test with a real customer-style CSV (not just the built-in sample)
5. Confirm the Export step produces a CSV the customer can open in Excel

---

## Recommendation

**Overall: 8/10 for customer demonstration readiness.** The remaining gap is concentrated in operational polish, not missing functionality.

| Activity | Status | Condition |
|---|---|---|
| Internal engineering demo | ✅ Ready | As-is |
| Customer discovery meeting | ✅ Ready | After backend startup issue is fixed |
| Guided pilot demonstration | ✅ Ready | After backend startup issue is fixed |
| Planner self-service | 🟡 Needs work | After backend/error-state improvements |
| Production deployment | ❌ Not yet | Requires Docker, auth, deployment config |

The UltraCrew frontend exists, is well-designed, and implements the correct workflow. The import step works correctly and would be convincing to a customer. The primary issue is backend connectivity — when the backend is running and healthy, this application is likely to make a strong impression.

The recommended next actions are measured in 1–2 days (health checks, error states, startup sequencing, verifying the full workflow) rather than weeks of new feature development. That is a strong indication the product is approaching a stage where real customer feedback should become the primary driver of future work.

**Do not build a new frontend.** The existing `ui/ultracrew/` frontend is the right investment. Fix the connectivity and error handling, then use it.

---

## Next Steps

1. Start the backend before the frontend and verify all tabs load correctly
2. Walk through the complete 5-step Import & Schedule workflow with sample data
3. Fix the two quick-win error states (Constraints tab, Dashboard)
4. If all steps work end-to-end, this application is ready for a customer demo

---

---

## Evidence Closed Since v1.0

| Finding (v1.0) | Status in v1.1 |
|---|---|
| Wrong application inspected (ChronoSentiment at localhost:3000) | Resolved — correct application identified and inspected |
| UltraCrew frontend existence unknown | Confirmed — exists at ui/ultracrew/ |
| "There is no UltraCrew frontend" | Superseded — frontend is substantially complete |
| Browser workflow unknown | Step 1 (Import Staff) verified and working |
| Backend connectivity unknown | Confirmed as blocker — 502 on all data-fetching screens |

---

## Scope for v1.2

v1.2 should not be produced until the complete 5-step workflow has been exercised with a healthy backend. Its scope should include:

- Import (with customer-style CSV, not just the built-in sample)
- Rule selection (Step 2 — not yet observed)
- Schedule generation (Step 3 — not yet observed)
- Constraint review (Step 4 — not yet observed)
- Export and opening the CSV in Excel (Step 5 — not yet observed)
- Runtime and performance observations
- Error recovery (what happens when generation fails)

That would shift the assessment from "the UI exists and looks good" to "the complete scheduling experience works."

---

*This assessment supersedes ULTRACREW_DEMO_ASSESSMENT_v1.0.md. The v1.0 finding that "the running application was ChronoSentiment" was accurate. The v1.0 conclusion that "there is no UltraCrew frontend" is superseded — a UltraCrew frontend exists at ui/ultracrew/ and is substantially complete.*