# Coralys Domain Packages

**Platform:** Coralys Workforce Solutions
**Version:** 1.0
**Status:** Active — Airline (UltraCrew) and Healthcare (UltraRoster) packages are production-ready

---

## Overview

A Domain Package is the complete set of assets required to run a Coralys WOA diagnostic and WDX demonstration for a specific workforce scheduling domain. Each package sits on top of the shared Coralys platform and Solution Engine — it does not change the software, only the language, data, constraints, and scenarios used during the engagement.

```
Coralys Platform
│
├── WOA Methodology (domain-neutral)
├── WDX Demonstration Framework (domain-neutral)
│
├── UltraCrew Solution Engine
│   └── Airline Domain Package  ← this document
│
├── UltraRoster Solution Engine
│   └── Healthcare Domain Package  ← this document
│
├── UltraShift Solution Engine  [future]
│   └── Manufacturing Domain Package
│
├── UltraRail Solution Engine  [future]
│   └── Rail Domain Package
│
└── UltraField Solution Engine  [future]
    └── Logistics / Field Services Domain Package
```

---

## What a Domain Package Contains

Every Domain Package defines six components:

| Component | Description |
|-----------|-------------|
| **Terminology map** | How generic Coralys terms translate into domain language |
| **WOA question bank** | Domain-specific diagnostic questions for each WOA dimension |
| **Demo dataset** | Realistic anonymised data for the WDX demonstration |
| **Constraint library** | The regulatory and operational rules relevant to this domain |
| **KPI set** | The metrics that matter to buyers in this domain |
| **Scenario scripts** | The three standard demo scenarios with expected outcomes |

---

## Package 1 — Airline (UltraCrew)

**Solution Engine:** UltraCrew
**Demo URL:** `http://localhost:3000`
**Regulatory context:** DGCA (India), EASA (Europe), FAA (USA), ICAO standards

### Terminology Map

| Coralys term | Airline term |
|-------------|-------------|
| Worker | Pilot / Crew member |
| Shift | Duty period |
| Schedule | Crew roster / Pairing |
| Constraint | Duty time limitation (DTL) |
| Optimisation run | Pairing optimisation |
| Divergence | Crew shortage / Coverage gap |
| Simulation | Disruption recovery scenario |

### WOA Question Bank — Airline

**Data Availability**
- How do you currently track pilot duty hours and rest periods?
- Is your crew data in a single system or spread across spreadsheets and OCC tools?
- How quickly can you produce a full crew availability report for tomorrow?
- Do you have historical data on disruption events and how they were resolved?

**Decision Process**
- When a pilot calls in sick 2 hours before departure, what is your current process?
- Who makes the final call on crew substitutions — OCC, crew planning, or the captain?
- How many people are involved in a typical disruption recovery decision?
- How long does it take from disruption event to crew reassignment confirmation?

**Tooling Maturity**
- What software do you currently use for crew scheduling?
- Do your scheduling tools enforce duty time limitations automatically, or is that a manual check?
- Can your current tools simulate the downstream impact of a single crew change?
- Do you have any optimisation capability today, or is scheduling primarily manual?

**Organisational Readiness**
- Is crew scheduling seen as a cost centre or a strategic capability in your organisation?
- Has your organisation attempted to implement a new scheduling system before? What happened?
- Who would be the internal champion for a crew optimisation project?
- What would need to be true for your organisation to commit to a 90-day pilot?

### Demo Dataset

**Scenario base:** 12-aircraft regional airline, 3 hubs, 48 crew members (24 pilots, 24 cabin crew)
**Roster period:** 28-day rolling roster
**Disruption events pre-loaded:** 3 (weather delay, sick call, aircraft swap)
**Regulatory rules active:** DGCA duty time limitations (default), switchable to EASA

### Constraint Library — Airline

| Constraint | Regulatory basis | Default |
|-----------|-----------------|---------|
| Maximum flight duty period (FDP) | DGCA CAR-FTL / EASA ORO.FTL | On |
| Minimum rest period between duties | DGCA CAR-FTL / EASA ORO.FTL | On |
| Maximum cumulative hours (28-day) | DGCA CAR-FTL | On |
| Minimum rest after long-haul | ICAO Annex 6 | On |
| Type rating requirement | Operator-specific | On |
| Recency requirement (90-day rule) | DGCA / EASA | On |
| Base assignment preference | Operator preference | Off |
| Seniority bidding | Union agreement | Off |

### KPI Set — Airline

| KPI | Typical baseline | UltraCrew target |
|-----|-----------------|-----------------|
| Crew utilisation rate | 68–74% | >82% |
| Disruption recovery time | 45–90 min | <15 min |
| Duty time violation rate | 1–3% | 0% |
| Deadhead cost per month | Varies | -20% |
| Roster publication lead time | 5–7 days | 14 days |
| Crew satisfaction score | Varies | Tracked |

### Scenario Scripts — Airline

**Scenario 1 — Baseline establishment**
*"Show me what your current roster looks like and where the gaps are."*
- Load the 28-day roster
- Run WOA baseline assessment
- Highlight duty time utilisation heatmap
- Identify 3 coverage gaps in the next 7 days

**Scenario 2 — Disruption recovery**
*"A pilot calls in sick 90 minutes before departure. What happens?"*
- Trigger sick-call event on Flight UC-204
- Show available crew within duty time limits
- Generate substitution proposal with compliance check
- Compare: manual process (45 min) vs UltraCrew (< 2 min)

**Scenario 3 — Optimisation impact**
*"If you could run the roster again from scratch, how much better could it be?"*
- Run optimisation on current 28-day period
- Show utilisation improvement (+8–12 percentage points)
- Show deadhead reduction
- Show duty time headroom gained

---

## Package 2 — Healthcare (UltraRoster)

**Solution Engine:** UltraRoster
**Demo URL:** `http://localhost:5173`
**Regulatory context:** Nursing council regulations, hospital accreditation standards, union agreements

### Terminology Map

| Coralys term | Healthcare term |
|-------------|----------------|
| Worker | Nurse / Care staff |
| Shift | Shift (day / evening / night) |
| Schedule | Roster / Rota |
| Constraint | Staffing ratio / Certification requirement |
| Optimisation run | Roster generation |
| Divergence | Understaffing / Coverage gap |
| Simulation | Sick leave / surge scenario |

### WOA Question Bank — Healthcare

**Data Availability**
- How do you currently track nurse certifications and specialisation (ICU, paediatrics, etc.)?
- Is your staffing data in a single system or managed across ward-level spreadsheets?
- How quickly can you identify which nurses are available for an unplanned shift tomorrow?
- Do you have historical data on sick leave patterns and how they were covered?

**Decision Process**
- When a nurse calls in sick for a night shift, what is your current escalation process?
- Who approves last-minute roster changes — the ward manager, nursing director, or HR?
- How many phone calls does a typical unplanned absence require to resolve?
- How long does it take from a sick call to confirmed cover?

**Tooling Maturity**
- What software do you currently use for nurse rostering?
- Does your current system enforce minimum staffing ratios automatically?
- Can your current tools show the downstream impact of one absence on the whole ward?
- Do you have any automated rostering capability, or is it primarily manual?

**Organisational Readiness**
- Is nurse rostering seen as an administrative function or a patient safety function?
- Has your organisation attempted to implement a rostering system before? What happened?
- Who would be the internal champion for a rostering optimisation project?
- What would need to be true for your organisation to commit to a 90-day pilot?

### Demo Dataset

**Scenario base:** 48-bed medical ward, 3 shifts/day, 32 nurses (8 ICU-certified, 6 paediatric-certified)
**Roster period:** 4-week rolling roster
**Disruption events pre-loaded:** 3 (sick call, surge admission, certification gap)
**Staffing ratios active:** 1:4 general ward, 1:2 ICU (default, configurable)

### Constraint Library — Healthcare

| Constraint | Basis | Default |
|-----------|-------|---------|
| Minimum nurse:patient ratio | Hospital accreditation / Nursing council | On |
| ICU certification requirement | Clinical governance | On |
| Maximum consecutive shifts | Nursing council / Union agreement | On |
| Minimum rest between shifts | Nursing council | On |
| Night shift rotation fairness | Union agreement | On |
| Paediatric certification requirement | Clinical governance | On |
| Senior nurse per shift minimum | Hospital policy | On |
| Preference-based scheduling | Staff wellbeing | Off |

### KPI Set — Healthcare

| KPI | Typical baseline | UltraRoster target |
|-----|-----------------|-------------------|
| Agency/bank staff usage | 15–25% of shifts | <8% |
| Sick leave coverage time | 2–4 hours | <30 min |
| Staffing ratio compliance | 92–96% | >99% |
| Roster publication lead time | 2–3 weeks | 4 weeks |
| Overtime hours per month | Varies | -25% |
| Staff satisfaction score | Varies | Tracked |

### Scenario Scripts — Healthcare

**Scenario 1 — Baseline establishment**
*"Show me what your current roster looks like and where the risks are."*
- Load the 4-week roster
- Run WOA baseline assessment
- Highlight staffing ratio compliance heatmap
- Identify shifts where ICU certification coverage is marginal

**Scenario 2 — Sick leave simulation**
*"A nurse becomes unavailable for tonight's night shift. What happens?"*
- Trigger sick-call event on Night Shift, Ward B
- Show available nurses with correct certifications within rest-period limits
- Generate cover proposal with compliance check
- Compare: manual process (2 hours of phone calls) vs UltraRoster (< 3 min)

**Scenario 3 — Surge scenario**
*"You have a sudden increase in ICU admissions. Can you staff it safely?"*
- Trigger +4 ICU admissions event
- Show current ICU-certified nurse availability
- Generate reallocation proposal maintaining ratio compliance on all wards
- Show which wards can absorb the reallocation without falling below minimum ratios

---

## Future Packages (Planned)

### Package 3 — Manufacturing (UltraShift)

**Status:** Planned — no Solution Engine yet
**Key constraints:** Shift patterns (continental, 4-on-4-off), skills matrix, machine certification, fatigue management
**Key KPIs:** OEE impact, overtime cost, skills coverage, absenteeism rate
**Regulatory context:** Working Time Directive, industry-specific safety certifications

### Package 4 — Rail (UltraRail)

**Status:** Planned — no Solution Engine yet
**Key constraints:** Train Operating Company (TOC) rules, RSSB fatigue risk management, route knowledge certification, rest facility availability
**Key KPIs:** Train punctuality impact, driver utilisation, fatigue score, disruption recovery time
**Regulatory context:** RSSB, ORR, Network Rail standards

### Package 5 — Logistics / Field Services (UltraField)

**Status:** Planned — no Solution Engine yet
**Key constraints:** Driver hours (EU/UK tachograph rules), vehicle certification, geographic coverage, SLA windows
**Key KPIs:** On-time delivery rate, driver utilisation, overtime cost, SLA compliance
**Regulatory context:** EU Regulation 561/2006, UK drivers' hours rules

---

## Using This Document

**For facilitators:** Before each engagement, confirm which Domain Package applies. Use the terminology map to translate Coralys language into the customer's language throughout the session. Use the WOA question bank to guide the diagnostic. Use the scenario scripts to structure the WDX demonstration.

**For product:** Evidence items in the Commercial Evidence Register tagged as `Solution` scope should reference the relevant Domain Package. Evidence tagged as `Platform` scope applies across all packages.

**For commercial:** The KPI set in each package is the basis for the ROI conversation. Do not invent KPIs — use the package values as the starting point and adjust based on the customer's actual baseline data.

---

*Document version: 1.0*
*Created: 2026-07-28*
*Maintained by: Coralys Commercial Team*