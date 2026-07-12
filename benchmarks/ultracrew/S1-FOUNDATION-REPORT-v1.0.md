# UltraCrew Sprint 1 — Workflow Foundation Report v1.0

**Status:** FROZEN  
**Date:** 2026-07-12  
**Commit:** 3f6f2bd6 (governance-hardening)  
**Build:** ✓ 33 modules, 277KB, 0 TypeScript errors

---

## Objective

Deliver the first end-to-end demonstrable planner workflow.

The goal was not to build a complete scheduling system. It was to establish the workflow skeleton that a planner would actually use — so that every subsequent sprint improves a real workflow rather than adding isolated features.

---

## Definition of Done (Sprint 1)

| Criterion | Status |
|---|---|
| A planner imports a staff CSV | ✓ |
| A planner selects a rule set | ✓ |
| A planner generates a schedule | ✓ |
| A planner edits assignments | ✓ |
| A planner exports the roster to Excel | ✓ |

**Milestone name:** Workflow Foundation Complete  
**Not claimed:** Publishable Schedule Generation (that is Sprint 2)

---

## Architecture

```
ui/ultracrew/src/workflow/
├── WorkflowTypes.ts        — domain types, rule presets, shift constants
├── WorkflowUtils.ts        — CSV parser, schedule builders, Excel exporter
├── WorkflowComponents.tsx  — shared styles, Stepper, SummaryRow, SkillBadge
├── PlannerWorkflow.tsx     — orchestrator (wires all 5 screens)
├── ImportStaff.tsx         — Step 1: CSV upload + validation summary
├── SelectRules.tsx         — Step 2: preset picker + custom JSON
├── GenerateSchedule.tsx    — Step 3: /api/schedule + synthetic fallback
├── ReviewSchedule.tsx      — Step 4: 28-day grid + shift picker + explain modal
└── ExportRoster.tsx        — Step 5: stats + Excel download + print
```

### Naming convention established

Files describe responsibility, not sequence. No `Step1`, `Step2` prefixes. The `PlannerWorkflow` orchestrator determines order; filenames describe what each screen does. This convention applies to all future UltraCrew screens.

### Module split rationale

| File | Responsibility |
|---|---|
| `WorkflowTypes.ts` | Types only — no React, no logic |
| `WorkflowUtils.ts` | Pure functions — parsers, builders, exporters |
| `WorkflowComponents.tsx` | Shared UI primitives — no business logic |
| Step files | Single responsibility — one screen each |
| `PlannerWorkflow.tsx` | State only — no rendering logic |

---

## What was built

### ImportStaff

- CSV file upload or paste
- Parses `id,contract,skills` columns
- Duplicate and missing-field detection
- Live validation summary: staff count, contract types, skills, warnings
- "Load Sample (8 staff)" for instant demo
- Scrollable staff table preview

### SelectRules

- Three presets: Hospital Standard, INRC Demo, Light Rules
- Custom JSON option with parse-time validation
- Radio-button selection with visual highlight

### GenerateSchedule

- Calls `/api/schedule` with imported staff + selected rule payload
- Animated 5-stage progress indicator (MOGA stages)
- **Synthetic fallback:** if backend unavailable, generates a deterministic schedule so the workflow continues unblocked for demos and sales presentations

### ReviewSchedule

- 28-day scrollable grid with colour-coded shift chips
  - Early = blue (#38bdf8)
  - Late = amber (#f59e0b)
  - Night = indigo (#818cf8)
  - Off = muted
- Weekend columns highlighted
- Click any cell → shift picker popover (Early / Late / Night / Off)
- Edit counter tracks manual changes
- `?` button on each cell → explain modal
- Constraint health bar (hard violations, soft violations, warnings)

### ExportRoster

- Summary stats: staff, days, total shifts, coverage %, hard/soft violations
- Shift distribution breakdown (Early/Late/Night counts)
- "Download Excel (.xls)" — TSV file that opens directly in Excel/Google Sheets
- "Print / Save PDF" — triggers browser print dialog
- Recommendations panel from the constraint engine
- "Start New Schedule" resets the full workflow

---

## Known Limitations

### Generate step (critical)

The synthetic fallback produces a schedule that is structurally valid (no hard violations in the synthetic path) but is not operationally meaningful. It does not:

- Respect actual coverage requirements
- Optimise for fairness or fatigue
- Produce a roster a planner could publish without significant manual editing

This is intentional for Sprint 1. The fallback exists to keep demos working without a backend. It is a temporary bridge, not a product feature.

### Explain step (partial)

The explain modal provides constraint-level reasoning (contract type, skill match, weekend allocation, hard violation count). It does not yet provide:

- Comparison with alternative assignments
- Counterfactual reasoning ("if we changed this, then...")
- Constraint weight explanations

### Export step (partial)

- Excel export is TSV-based (opens in Excel, not a native .xlsx)
- No PDF generation (browser print only)
- No roster versioning or locking

### No Demo Mode / Planner Mode distinction

The synthetic fallback is currently invisible to the planner. Sprint 2 should make this explicit:

```
○ Demo Mode    — synthetic schedule, instant startup, no backend required
● Planner Mode — Coralys optimization, real constraints, publishable output
```

---

## Sprint 2 Definition

**Milestone name:** Publishable Schedule Generation

**Objective:** Make the Generate step produce a schedule a planner can confidently publish with only minor manual adjustments.

**Definition of Done:**

| KPI | Target |
|---|---|
| Hard constraint violations | 0 |
| Coverage | 100% of required shifts covered |
| Manual edits required | < 5% of assignments |
| Time to first publishable roster | < 5 minutes from import |
| Planner acceptance | Planner can publish without rebuilding |

**Key work:**

1. Wire `GenerateSchedule` to the real Coralys MOGA pipeline with customer-imported staff and selected rules
2. Introduce Demo Mode / Planner Mode toggle (synthetic vs. real)
3. Improve constraint satisfaction: zero hard violations on generated output
4. Re-optimize after manual edits (partial re-solve on changed cells)
5. Add "Publish" step between Review and Export (lock, version, distribute)

**What Sprint 2 does not include:**

- Drag-and-drop editing (click-to-change is sufficient)
- PDF generation (browser print is sufficient)
- Rule editor UI (JSON import is sufficient)
- GERAD airline dataset integration (Sprint 3+)

---

## Product Milestone Distinction

| Milestone | Status |
|---|---|
| Workflow Foundation | ✓ Sprint 1 — FROZEN |
| Publishable Schedule Generation | Sprint 2 |
| Planner Trust (< 5% edits) | Sprint 3 |
| Pilot Customer Deployment | Sprint 4+ |

---

## Weekly Success Criterion

> Did UltraCrew become more usable?

**Yes.** A planner can now arrive with their own data and leave with a roster in five steps, without any backend dependency blocking the demonstration. The product story shifted from "look at our optimizer" to "here is how a planner uses UltraCrew."

That is the first commercially meaningful UltraCrew milestone.

---

*Report frozen at commit 3f6f2bd6. Do not modify. Sprint 2 begins from this baseline.*