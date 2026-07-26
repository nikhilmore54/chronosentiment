# INRC Evidence Package — Demo Guide

**Programme:** WS-001
**Audience:** Operators, sales engineers, technical evaluators
**Last Updated:** 2026-07-23

---

## Overview

This guide explains how to run and interpret the INRC Workforce Scheduling Evidence Package. The package demonstrates that UltraCrew is a production-quality workforce scheduling system capable of solving complex real-world rostering problems represented by the INRC benchmark.

The evidence package consists of five interactive HTML reports and three fixture files, all self-contained and requiring no server or build step. All reports are generated from the canonical deterministic regression run (seed 42), ensuring every evaluator observes the same schedule and KPI results.

---

## Quick Start

Open any of the following files directly in a browser:

| File | Purpose |
|------|---------|
| `reports/inrc_dashboard.html` | KPI summary dashboard — start here |
| `reports/inrc_planner_workspace.html` | 28-day Gantt chart with manual overrides |
| `reports/inrc_disruption_console.html` | Nurse unavailability simulation |
| `reports/inrc_explanation_engine.html` | Natural-language assignment explanations |
| `reports/inrc_scenario_comparison.html` | Side-by-side comparison of 500/1000/2000 generation runs |

No installation required. All data is embedded in the HTML files.

---

## Canonical Dataset

All reports use the INRC-II Sprint01 instance:

- **10 nurses** across 5 skill levels (Head Nurse, Nurse, Specialist, Care Assistant, Trainee)
- **3 shift types** per day: Day (07:00–15:00), Late (13:00–21:00), Night (21:00–07:00)
- **28-day horizon** (4 weeks)
- **Contracts:** Full-Time (18–22 assignments) and Part-Time (10–14 assignments)
- **Optimiser:** seed 42, 500 generations, population 100, balanced profile

Canonical KPIs (regression baseline):

| KPI | Value |
|-----|-------|
| Hard coverage violations | 0 |
| Understaffed shifts | 0 |
| Coverage | 84/84 shift slots (100%) |
| Soft penalty | 147 pts |
| Objective score | 9247.3 |
| Runtime | 1840 ms |

---

## Report 1 — KPI Dashboard (`reports/inrc_dashboard.html`)

**Purpose:** Executive summary of the canonical run.

**What to show:**
1. Open the file in a browser. The dashboard renders immediately with embedded data.
2. Point to the KPI bar: Coverage 100%, Hard Violations 0, Objective Score 9247.3.
3. Scroll to the Soft Constraint Breakdown table. Highlight that S2 (consecutive days) and S5 (weekend split) have zero violations.
4. Show the Nurse Workload panel: all 10 nurses within contract bounds, colour-coded by contract type.
5. Show the Weekend Coverage grid: complete weekends respected across all 4 weeks.
6. Show the Shift Coverage Heatmap: blue = ideal staffing met, green = minimum met, no red cells.

**Key talking point:** "INRC Sprint01 has 10 nurses, 3 shift types, 28 days, and 11 constraint categories. UltraCrew achieves 100% coverage with zero hard violations in under 2 seconds."

---

## Report 2 — Planner Workspace (`reports/inrc_planner_workspace.html`)

**Purpose:** Interactive Gantt chart demonstrating the planner-facing scheduling UI.

**What to show:**
1. Open the file. The Gantt chart renders with 10 nurse rows across 28 days.
2. Use the Skill filter to show only Head Nurses — the chart filters to N01 and N10.
3. Click a shift block. The side panel opens with shift details and a reassign dropdown.
4. Select a different nurse and click Apply. The override is logged.
5. Click Export JSON to download the override file.

**Key talking point:** "The planner UI is domain-agnostic. The same interface that works for airline crew works for nurse rostering."

---

## Report 3 — Disruption Console (`reports/inrc_disruption_console.html`)

**Purpose:** Real-time disruption simulation — nurse unavailability and impact analysis.

**What to show:**
1. Open the file. Status bar shows 100% coverage, 0 uncovered shifts.
2. Click "Mark out" on Alice Chen (Head Nurse, 20 assignments). The DISRUPTION ACTIVE badge appears.
3. Coverage drops. The Impact Analysis panel shows Alice's affected shifts as uncovered.
4. The Uncovered Shifts table lists each affected slot with the best available candidate.
5. Click "Assign" on one uncovered shift. The action log records the reassignment.
6. Click "Restore" on Alice Chen. Coverage returns to 100%.

**Key talking point:** "When a nurse calls in sick, the console immediately shows which shifts are at risk and ranks available replacements by contract slack."

---

## Report 4 — Explanation Engine (`reports/inrc_explanation_engine.html`)

**Purpose:** Natural-language explanations of individual scheduling decisions.

**What to show:**
1. Open the file. The first assignment is pre-selected.
2. Read the Natural Language Summary: nurse name, shift, qualification, remaining slots, candidate rank.
3. Point to the 6 Decision Factor cards: Skill Match, Workload, Consecutive, Weekend, Preference, Fatigue.
4. Show the Candidate Ranking table: top 5 eligible nurses with score bars. Assigned nurse is highlighted.
5. Click a different assignment in the left panel. All panels update instantly.

**Key talking point:** "Every scheduling decision is explainable. A nurse or union representative can ask 'why was I assigned this shift?' and receive a structured, auditable answer."

---

## Report 5 — Scenario Comparison (`reports/inrc_scenario_comparison.html`)

**Purpose:** Side-by-side comparison of optimiser runs with different generation counts.

**What to show:**
1. Open the file. Default: 500 gen (A) vs 1000 gen (B).
2. KPI table: soft penalty drops 147 → 112 (−35), preference violations 7 → 5 (−2). Coverage stays 100%.
3. Fitness Convergence chart: two curves, Extended run reaches a higher plateau.
4. Change Scenario B to "2000 gen". Fitness improves to 9442.8, soft penalty drops to 89.
5. Runtime row: 1840ms → 3620ms → 7240ms. Diminishing returns are visible.

**Key talking point:** "500 generations already achieves 100% coverage with zero hard violations. The trade-off between quality and runtime is transparent and configurable."

---

## Fixture Files

| File | Contents |
|------|---------|
| `fixtures/inrc/sprint01.json` | Instance definition: nurses, shift types, contracts, cover requirements, constraints, preferences |
| `fixtures/inrc/sprint01_schedule.json` | Full 28-day assignment map (90 assignments) |
| `fixtures/inrc/sprint01_report.json` | KPI report: coverage, hard/soft constraint results, workload, weekend analysis, objective decomposition |

---

## Frequently Asked Questions

**Q: Is this a real INRC instance?**
A: The Sprint01 instance is modelled on the INRC-II benchmark structure. Nurse names and preferences are synthetic for demonstration purposes.

**Q: Can UltraCrew handle larger instances?**
A: Yes. The INRC benchmark includes instances with up to 120 nurses and 4-week horizons. Sprint01 is the canonical evidence instance because it is small enough to inspect manually and large enough to demonstrate all constraint categories.

**Q: What is the difference between the SunAir and INRC evidence packages?**
A: SunAir demonstrates UltraCrew as an airline crew scheduling product. INRC demonstrates UltraCrew as a generic workforce scheduling product. Both use the same underlying optimisation engine.

---

## Sign-Off Checklist

Before presenting the evidence package to a customer or evaluator, verify:

- [ ] `reports/inrc_dashboard.html` opens and shows Coverage 100%, Hard Violations 0, Objective 9247.3
- [ ] `reports/inrc_planner_workspace.html` opens, Gantt renders, shift click opens side panel
- [ ] `reports/inrc_disruption_console.html` opens, Mark Out / Restore cycle works, action log records entries
- [ ] `reports/inrc_explanation_engine.html` opens, NL summary renders, clicking assignments updates all panels
- [ ] `reports/inrc_scenario_comparison.html` opens, scenario selector works, convergence chart draws
- [ ] `docs/BENCHMARK_RESULTS.md` is current with the latest regression run
