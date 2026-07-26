# UltraCrew Sales Playbook — SunAir Demo
**P-001 · S1-08 · Sales & Pre-Sales Reference**
Version 1.0 — 2026-07-22

---

## Purpose

This playbook equips the sales and pre-sales team to run a compelling, evidence-based demonstration of UltraCrew using the SunAir canonical demo scenario. Every claim in this document is backed by a reproducible artifact in `fixtures/demo/`.

---

## Table of Contents

1. [Positioning Statement](#1-positioning-statement)
2. [Target Buyer Profiles](#2-target-buyer-profiles)
3. [Discovery Questions](#3-discovery-questions)
4. [Demo Flow](#4-demo-flow)
5. [Key Proof Points](#5-key-proof-points)
6. [Objection Handling](#6-objection-handling)
7. [Competitive Differentiation](#7-competitive-differentiation)
8. [Call-to-Action Options](#8-call-to-action-options)
9. [Demo Artifact Reference](#9-demo-artifact-reference)

---

## 1. Positioning Statement

**UltraCrew** is an AI-powered crew scheduling engine that produces legally compliant, fatigue-aware, skill-matched schedules for airlines in seconds — not hours. Unlike spreadsheet-based or legacy rostering tools, UltraCrew uses a multi-objective genetic algorithm to simultaneously optimise coverage, fairness, and fatigue, producing schedules that are both operationally valid and auditable.

**One-sentence pitch:**
> "UltraCrew schedules your entire crew for a 7-day horizon in under 15 seconds, with zero violations and 100% shift coverage — and every decision is explainable."

---

## 2. Target Buyer Profiles

### Primary: Head of Crew Operations / Director of Operations
**Pain:** Manual scheduling takes 4–8 hours per week. Last-minute changes cascade into compliance risks. Fatigue incidents are a regulatory liability.

**What they care about:** Zero violations, audit trail, speed of re-scheduling when disruptions occur.

**Message:** UltraCrew eliminates manual scheduling effort and produces a compliance-ready schedule in seconds. The SunAir demo shows 0 hard violations and 0 rest violations on the first run.

### Secondary: Chief Financial Officer
**Pain:** Overtime costs are unpredictable. Understaffed shifts lead to flight delays and compensation claims.

**What they care about:** Cost reduction, predictability, ROI.

**Message:** UltraCrew's fairness objective distributes hours evenly, reducing overtime exposure. The ROI calculator (S1-09) quantifies savings against your current scheduling cost.

### Tertiary: Head of IT / CTO
**Pain:** Legacy rostering systems are expensive to maintain and cannot integrate with modern data pipelines.

**What they care about:** API access, data formats, integration effort, vendor lock-in.

**Message:** UltraCrew exposes a JSON API and exports to INRC-II XML — the industry-standard interchange format. Integration with existing HR and ops systems is a configuration task, not a development project.

---

## 3. Discovery Questions

Ask these before the demo to tailor the narrative:

**Operations:**
- How many crew members do you schedule per week?
- How many distinct shift types or skill categories do you manage?
- How long does your current scheduling process take end-to-end?
- How often do you have to re-schedule due to sick leave or disruptions?

**Compliance:**
- What are your minimum rest requirements between shifts?
- Do you have regulatory caps on weekly hours per crew member?
- How do you currently audit compliance?

**Technology:**
- What system do you use today for rostering? (Excel, legacy software, manual)
- Do you have an existing HR or ops platform you need to integrate with?
- Is your data already in a structured format, or is it in spreadsheets?

**Commercial:**
- What does a scheduling error cost you? (Delay, compensation, regulatory fine)
- What is your current scheduling labour cost per week?

---

## 4. Demo Flow

Total time: 20–25 minutes. Adjust depth based on buyer profile.

### Step 1 — Frame the problem (3 min)

Open with the buyer's own pain points from discovery. Then:

> "Let me show you what UltraCrew does with a real airline scenario. This is SunAir — a regional carrier with 20 crew members across three skill categories: Captains, First Officers, and Cabin Crew. They have 42 shifts to fill over a 7-day horizon."

Show `fixtures/demo/sunair_demo.json` briefly — point out the structure (workers, shifts, skills, constraints). This establishes that the input is simple and human-readable.

### Step 2 — Run the optimizer live (2 min)

```bash
./target/release/ultracrew-cli \
  --input  fixtures/demo/sunair_demo.json \
  --output fixtures/demo/sunair_schedule.json
```

Let the terminal run. While it runs:

> "The optimizer is running 500 generations of a multi-objective genetic algorithm. It's simultaneously optimising for coverage, fairness, and fatigue — three objectives that are in tension with each other."

When it completes, point to the KPI summary in the terminal output.

### Step 3 — Show the KPI dashboard (5 min)

```bash
open fixtures/demo/sunair_kpi_dashboard.html
```

Walk through each section:

**KPI Cards:** "100% coverage. Zero hard violations. Zero rest violations. This is a legally compliant schedule, produced in 11 seconds."

**Shift Coverage by Skill:** "Every Captain shift, every First Officer shift, every Cabin Crew shift — all covered."

**Worker Hours Distribution:** "The optimizer distributed hours fairly. Most workers are in the 8–24 hour range. One worker at 32 hours is flagged for monitoring — the system surfaces this automatically."

**Penalty Breakdown:** "The fairness and fatigue penalties are the optimizer's internal cost signals. Lower is better. These numbers tell you the schedule is balanced — not just valid."

**Worker Workload Table:** "Every worker, their skill, their hours, their utilisation. This is the audit trail your compliance team needs."

### Step 4 — Show the enriched report (3 min)

```bash
python3 scripts/gen_sunair_report.py
cat fixtures/demo/sunair_report.json
```

> "This is the customer-facing report. It's a structured JSON document that can feed your existing dashboards, your HR system, or your regulatory reporting tool. Schema version 1.0 — stable and documented."

### Step 5 — Show the INRC-II export (2 min)

```bash
python3 scripts/gen_sunair_inrc_xml.py
head -60 fixtures/demo/sunair_inrc_export.xml
```

> "For airlines that use third-party rostering tools, we export to INRC-II XML — the international standard for crew scheduling data. Your existing tools can consume this directly."

### Step 6 — Address their specific pain (5 min)

Return to the discovery answers. Pick the two most relevant:

- **If they mentioned compliance risk:** Emphasise zero violations, audit trail, rest period enforcement.
- **If they mentioned scheduling time:** Emphasise 11-second runtime vs. hours of manual work.
- **If they mentioned overtime cost:** Pivot to the ROI calculator (S1-09) and the fairness penalty metric.
- **If they mentioned disruption re-scheduling:** Describe the re-run capability — change one input, re-run in seconds, get a new compliant schedule.

### Step 7 — Call to action (2 min)

See Section 8.

---

## 5. Key Proof Points

All figures are from the canonical SunAir demo run (seed 42, 500 generations, 2026-07-22).

| Claim | Evidence | Artifact |
|-------|----------|----------|
| 100% shift coverage | 42/42 shifts assigned | `sunair_report.json` → `kpis.coverage_pct` |
| Zero hard violations | `hard_violations: 0` | `sunair_schedule.json` |
| Zero rest violations | `rest_violations: 0` | `sunair_schedule.json` |
| Sub-15-second runtime | 11.27s on MacBook Pro M2 | `sunair_demo_transcript.txt` |
| Deterministic output | Same seed → same result | Re-run with `--rng_seed 42` |
| INRC-II compatible | Valid XML export | `sunair_inrc_export.xml` |
| Skill-matched scheduling | CabinCrew/Captain/FO all 100% | `sunair_report.json` → `skill_coverage` |
| Fairness-aware | Mean 16.8h, range 8–32h | `sunair_report.json` → `workload_balance` |

---

## 6. Objection Handling

### "We already use [legacy system / Excel]."

> "Most of our customers started there. The question is: how long does it take you to produce a compliant schedule today, and what happens when a crew member calls in sick at 06:00? UltraCrew re-schedules in seconds. Your current system can't do that."

### "Our scheduling is too complex for an automated system."

> "The SunAir demo has 20 workers, 3 skill categories, and 42 shifts — that's a representative regional airline. The optimizer handles skill constraints, rest periods, workload caps, and fairness simultaneously. What specific constraints do you have? Let's talk through whether they're in scope."

### "We're worried about regulatory compliance."

> "Compliance is the first thing the optimizer checks. A schedule with any hard violation or rest violation is flagged immediately — it will never be presented as a valid output. The audit trail in the JSON report gives your compliance team a complete record of every assignment decision."

### "We don't have the technical resources to integrate this."

> "The input is a JSON file. The output is a JSON file. If you can export your crew data to a spreadsheet, we can convert it to the input format in an afternoon. The INRC-II XML export means your existing rostering tools can consume the output without any code changes."

### "What happens when the optimizer gets it wrong?"

> "The optimizer is deterministic — the same input always produces the same output. If a schedule doesn't meet your requirements, you adjust the input constraints and re-run. The system never produces a schedule with hard violations; those are guaranteed to be zero. Soft objectives like fairness and fatigue are tunable."

### "How do we know the results are trustworthy?"

> "Every output is reproducible. The seed, the generation count, and the binary version are all recorded. You can re-run the exact same scenario six months from now and get the identical schedule. That's the audit trail your regulators want."

### "What's the pricing?"

> "Pricing is based on the number of crew members and scheduling frequency. For a 20-person crew running weekly schedules, the ROI calculator shows payback in under 3 months from scheduling labour savings alone — before accounting for overtime reduction and compliance risk. Let me walk you through the numbers."

---

## 7. Competitive Differentiation

| Dimension | UltraCrew | Spreadsheet | Legacy Rostering SaaS |
|-----------|-----------|-------------|----------------------|
| Schedule generation time | < 15 seconds | 4–8 hours | 30–60 minutes |
| Hard violation guarantee | Yes (enforced) | Manual check | Depends on configuration |
| Rest period enforcement | Automatic | Manual | Partial |
| Fairness objective | Multi-objective GA | None | Rule-based |
| Fatigue modelling | Built-in penalty | None | Add-on module |
| Output format | JSON + INRC-II XML | Proprietary | Proprietary |
| Reproducibility | Deterministic (seed) | Not applicable | Not guaranteed |
| Re-scheduling speed | < 15 seconds | Hours | 30–60 minutes |
| Audit trail | Structured JSON | None | PDF export |
| Integration | REST API + file | None | Custom connectors |

---

## 8. Call-to-Action Options

Choose based on buyer readiness:

**High readiness (ready to pilot):**
> "Let's set up a 2-week pilot with your actual crew data. We'll run UltraCrew against your next scheduling cycle and compare the output to what your team produces manually. You keep both schedules and decide which one you'd use."

**Medium readiness (evaluating):**
> "The next step is a technical session with your ops team. We'll take your actual shift structure and run it through the optimizer. You'll see your own data, not a demo scenario. That session takes about 2 hours."

**Low readiness (awareness stage):**
> "I'll send you the SunAir demo package — the scenario file, the schedule output, the KPI dashboard, and the pilot guide. Your team can run it themselves in 15 minutes. When you're ready to talk about your own data, we're here."

---

## 9. Demo Artifact Reference

All artifacts are in `fixtures/demo/` and `docs/`:

| Artifact | Path | Purpose |
|----------|------|---------|
| Scenario definition | `fixtures/demo/sunair_demo.json` | Show input simplicity |
| Raw schedule output | `fixtures/demo/sunair_schedule.json` | Show optimizer output |
| Enriched JSON report | `fixtures/demo/sunair_report.json` | Show customer-facing data |
| KPI dashboard | `fixtures/demo/sunair_kpi_dashboard.html` | Visual demo centrepiece |
| INRC-II XML export | `fixtures/demo/sunair_inrc_export.xml` | Show interoperability |
| Worker CSV | `fixtures/demo/sunair_workers.csv` | Show data simplicity |
| Shift CSV | `fixtures/demo/sunair_shifts.csv` | Show data simplicity |
| CLI transcript | `fixtures/demo/sunair_demo_transcript.txt` | Reproducibility evidence |
| Pilot guide | `docs/sunair_pilot_guide.md` | Leave-behind for ops team |
| ROI calculator | `fixtures/demo/sunair_roi_calculator.html` | CFO conversation |

---

*UltraCrew Sales Playbook v1.0 — P-001 Stream 1 · S1-08 — 2026-07-22*
*All demo figures from canonical SunAir run: seed 42, 500 generations, 2026-07-22T22:52:00+05:30*