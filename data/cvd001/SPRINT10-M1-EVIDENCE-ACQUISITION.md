# Sprint 10 — Milestone 1: Benchmark Evidence Acquisition & Provenance

**Document:** SPRINT10-M1-EVIDENCE-ACQUISITION.md
**Date:** 2026-07-16
**Status:** IN PROGRESS — Local evidence acquisition complete (S0); external evidence acquisition (S1–S5) pending
**Sprint:** 10 — Benchmark Reproduction & Semantic Validation  
**Milestone:** M1 — Benchmark Evidence Acquisition & Provenance

---

## Objective

Recover every authoritative artifact for the CVD-001 benchmark and establish its provenance and trustworthiness. Reconstruction precedes classification: no implementation decisions are made until this catalogue is complete or the stopping rule is triggered.

**HC3 remains unchanged during Milestone 1.**

---

## Evidence Hierarchy (reference)

| Level | Source | Authority |
|---|---|---|
| E1 | Benchmark evaluator source code | Highest |
| E2 | Official benchmark documentation (README, technical reports) | High |
| E3 | Dataset generation code | Medium |
| E4 | Dataset artifacts (credit_constrains.csv, creditedHours, etc.) | Medium |
| E5 | Observed benchmark outputs | Low |
| E6 | Research hypotheses | Lowest |

---

## Search Outcome Classification

| Symbol | Meaning |
|---|---|
| ✅ Found | Artifact obtained |
| ❌ Confirmed unavailable | Searched authoritative location; artifact absent |
| ⚠ Not found in local artifacts | Not in currently available local files; external search pending |
| 🔍 Pending | Not yet searched |
| ⚠ Partial | Related artifact found, but not sufficient |

---

## Milestone 1 Expected Outcomes

Milestone 1 has three possible results. Not finding the evaluator is still a successful completion of Milestone 1.

**Outcome A — Evaluator found (E1 evidence obtained)**
Proceed to Milestone 2 with high-authority evidence. Semantic reconstruction is grounded in source code.

**Outcome B — Documentation found but evaluator unavailable (E2 evidence obtained)**
Proceed to Milestone 2 with official documentation. Semantic reconstruction is grounded in problem statement.

**Outcome C — Neither evaluator nor documentation found**
Stopping rule triggered. Proceed under Option C (documented working hypothesis). All conclusions labeled E6 with explicit confidence.

---

## Evidence Catalogue

| Artifact | Status | Evidence Level | Provenance | Notes |
|---|---|---|---|---|
| Benchmark evaluator source code | ⚠ Not found in local artifacts | E1 | Unknown | Not in instance1/; not in b8b2a9c2; external search pending |
| README.pdf | ⚠ Not found in local artifacts | E2 | Unknown | Not on local filesystem; external search pending |
| GERAD technical reports (G-2010-xx) | 🔍 Pending | E2 | https://www.gerad.ca/en/papers | Search S1 pending |
| ROADEF 2010 challenge documentation | 🔍 Pending | E2 | http://www.roadef.org/challenge/2010/ | Search S2 pending |
| Authors' public repositories | 🔍 Pending | E2 | Polytechnique Montréal / GERAD | Search S3 pending |
| Author correspondence | ⏸ Not attempted | E2 | — | Pending S1–S4 |
| credit_constraints.cpp | ✅ Found — analyzed | E3 | Committed b8b2a9c2 | Generator analyzed; parser and credit-generation semantics documented (F9, F12, F13) |
| crew_availability_constraints.cpp | ✅ Found — analyzed | E3 | Committed b8b2a9c2 | Generator analyzed; duty-count semantics documented (F10, F11) |
| EmployeeLegPreferences.cpp | ✅ Found — analyzed | E3 | Committed b8b2a9c2 | Generator analyzed; preference-data generation documented (F14) |
| preferredVacations.cpp | ✅ Found — analyzed | E3 | Committed b8b2a9c2 | Generator analyzed; vacation-data generation documented |
| params.txt | ✅ Found | E3 | Committed b8b2a9c2 | data/cvd001/params.txt |
| credit_constrains.csv | ✅ Found | E4 | Committed b8b2a9c2 | Per-base caps: BASE1=326.9h, BASE2=1279.4h, BASE3=383.3h |
| crew_avail_const.csv | ✅ Found | E4 | Committed b8b2a9c2 | data/cvd001/instance1/crew_avail_const.csv |
| creditedHours | ✅ Found | E4 | Committed b8b2a9c2 | Workload values extracted; record structure reconstructed from `credit_constraints.cpp` parser (E3, F12). Runtime semantics remain unverified pending E1/E2 evidence. |
| day_1.csv … day_31.csv | ✅ Found | E4 | Committed b8b2a9c2 | 31 daily flight leg files |
| listOfBases.csv | ✅ Found | E4 | Committed b8b2a9c2 | data/cvd001/instance1/listOfBases.csv |
| solution_0 (reference solution) | ✅ Found | E5 | Committed b8b2a9c2 | Reference solution from benchmark authors |
| initialSolution.in | ✅ Found | E5 | Committed b8b2a9c2 | data/cvd001/instance1/initialSolution.in |

---

## Established Facts

These facts have passed the evidence threshold and are distinguished from hypotheses. Later documents may cite by ID (e.g. F3) rather than repeating the full statement.

| ID | Fact | Source | Evidence Level | Confidence |
|---|---|---|---|---|
| F1 | CVD-001 has 33 crew members | instance1/ CSV files | E4 | High |
| F2 | CVD-001 has 1013 active flight legs over 31 days | instance1/ CSV files | E4 | High |
| F3 | credit_constrains.csv contains per-base aggregate caps | credit_constrains.csv | E4 | High |
| F4 | Caps are BASE1=326.9h, BASE2=1279.4h, BASE3=383.3h | credit_constrains.csv | E4 | High |
| F5 | credit_constraints.cpp generates credit_constrains.csv from a reference solution with 3% slack | credit_constraints.cpp | E3 | High |
| F6 | 33/33 workers exceed 40h from historical workloads alone | scripts/hc3_audit.py | E4 | High |
| F7 | Historical workload range: 23.7h–84.9h, mean 68.75h | scripts/hc3_audit.py | E4 | High |
| F8 | Total assigned flight hours (Run 1): 1878.50h | scripts/hc3_audit.py | E4 | High |
| F9 | solution_0 format is: `schedule N EMP_ID (BASE) : TASK-->TASK-->...TASK;` | crew_availability_constraints.cpp L150–158 | E3 | High |
| F10 | Deadhead tasks (TDH prefix) are excluded from duty counting and from leg preferences | crew_availability_constraints.cpp L225; EmployeeLegPreferences.cpp L196 | E3 | High |
| F11 | A duty boundary is defined by a new calendar day in the flight number (LEG_DD_N, chars 4–5 = day) | crew_availability_constraints.cpp L231–238 | E3 | High |
| F12 | creditedHours file format per record: `<N> <M(BASE)> : credited_hours = <value>` followed by cost and vacation lines | credit_constraints.cpp L131–156 | E3 | High |
| F13 | During cap generation, `credit_constraints.cpp` increments a briefing-credit accumulator by one unit per detected duty and subtracts the accumulated value from the reference solution total when computing base credit caps. Code comments describe this as briefing/debriefing credit. The evaluator's runtime treatment of this adjustment is unknown. | credit_constraints.cpp L361–466 | E3 | High (observation); Low (interpretation) |
| F14 | `EmployeeLegPreferences.cpp` generates the employee-leg preference dataset (`PreferredAirLegs.csv`) by randomly assigning a percentage of base legs to each employee while excluding deadheads. This documents dataset generation; evaluator enforcement of SC4 remains unknown. | EmployeeLegPreferences.cpp | E3 | High |
| F15 | All locally available benchmark C++ sources are dataset generation utilities; no evaluator implementation is present locally. | S0 analysis | E3 | High |
| F16 | The dataset-generation utilities (`credit_constraints.cpp`, `crew_availability_constraints.cpp`, `EmployeeLegPreferences.cpp`, `preferredVacations.cpp`) originate from the official `G1422-DataSets.zip` supplementary package, authored by Frédéric Quesnel (with Atoosa Kasirzadeh for `preferredVacations.cpp`). This establishes their provenance as official dataset-generation artifacts rather than third-party reconstructions. | DATASET-INVENTORY-v1.0.md; file headers; confirmed by G-2014-22 Additional Material | E2 | High |
| F17 | GERAD Technical Report G-2014-22 (*Airline crew scheduling: Models, algorithms, and data sets*, Kasirzadeh, Saddoune, Soumis) officially distributes `G1422-DataSets.zip` as supplementary material. This establishes the authoritative publication source and provenance of the dataset package analyzed in Sprint 10. | https://www.gerad.ca/en/papers/G-2014-22 | E2 | High |

---

## Open Questions

| ID | Question | Highest Current Evidence | Confidence |
|---|---|---|---|
| Q1 | Is HC3 a hard feasibility constraint or a soft penalty? | E3–E4 | Low — generator code does not contain evaluator logic |
| Q2 | Are per-base credit caps (credit_constrains.csv) enforced as hard constraints during optimization? | E3 (F5) | Low — generator produces caps but enforcement semantics unknown |
| Q3 | How are credited hours accumulated across a bid period — is the formula `max(paid_minutes, 480) + deadhead_minutes × 2`? | E4 (creditedHours values) | Medium — formula consistent with observed values but not confirmed from evaluator |
| Q4 | Does a benchmark evaluator implementation exist publicly? | E2 search evidence (S1) | — no evaluator located in published supplementary material of G-2014-22 |
| Q5 | What is the intended planning horizon for HC3 (weekly / monthly / bid-period)? | E6 (hypothesis) | Low |
| Q6 | Does the briefing/debriefing credit subtraction in credit_constraints.cpp (F13) mean the evaluator adds 1h per duty to credited hours, or is it only used for cap generation? | E3 (F13) | Low — code subtracts from cap generation only; evaluator behavior unknown |

---

## Threats to Validity

| Threat | Impact |
|---|---|
| The original evaluator source may no longer be publicly available | Limits evidence to E3–E4; forces Option C. **Mitigation:** Official dataset provenance has now been established through GERAD Technical Report G-2014-22 (F17), reducing uncertainty about the authenticity of the dataset-generation artifacts even though evaluator semantics remain unavailable. |
| Dataset generators describe instance construction, not necessarily evaluator semantics | Generator behavior should not be treated as proof of runtime evaluation rules |
| Reference solution (solution_0) may encode assumptions not documented elsewhere | E5 evidence may be misleading without E1–E2 context |
| Public archives may have changed since the original benchmark release (2010) | Search results may be incomplete |
| The benchmark does not publish a formal specification of the `creditedHours` file; its record structure has been reconstructed from dataset-generation code (E3) but evaluator interpretation remains unverified | Runtime semantics may still differ from the reconstructed interpretation |

---

## Search Plan

### S1 — GERAD Archive

Target: https://www.gerad.ca/en/papers
Query: "ROADEF 2010" OR "crew scheduling" OR "G-2010" OR "Quesnel"
Looking for: Technical report with evaluator description or problem statement
Status: ✅ Complete — see Search Execution Log (S1)

### S2 — ROADEF 2010 Challenge Page

Target: http://www.roadef.org/challenge/2010/
Looking for: Problem statement PDF, evaluator download, supplementary materials
Status: ✅ Complete — see Search Execution Log (S2)

### S3 — Authors' Public Repositories

Targets:
- Frédéric Quesnel (Polytechnique Montréal)
- Louis-Martin Rousseau (Polytechnique Montréal / CIRRELT)
- Guy Desaulniers (Polytechnique Montréal / GERAD)

Platforms: GitHub, institutional pages, ResearchGate, Google Scholar  
Looking for: Evaluator source, problem statement, supplementary code  
Status: 🔍 Pending

### S4 — General Web Search

Queries:
- "ROADEF 2010 crew scheduling evaluator"
- "CVD-001 benchmark evaluator source"
- "challenge-roadef-2010 crew scheduling"
- "Quesnel Rousseau Desaulniers 2010 crew scheduling"

Status: 🔍 Pending

### S5 — Author Correspondence

Action: Draft email to benchmark authors requesting evaluator source or problem statement  
Trigger: After S1–S4 complete without finding E1/E2 evidence  
Status: ⏸ Pending

---

## Knowledge Progress

| Item | Sprint 9 | After S0 |
|---|---|---|
| Evaluator source (E1) | Unknown | Still not found locally |
| `creditedHours` format | Partially inferred | Formally reconstructed from parser (F12) |
| Duty counting semantics | Inferred | Confirmed from generator (F10, F11) |
| Deadhead handling | Inferred | Confirmed from generator (F10) |
| SC4 preference data generation | Unknown | Documented from generator (F14) |
| SC5 vacation data generation | Unknown | Documented from generator |
| HC3 enforcement | Unknown | Still unknown — evaluator absent |
| Briefing/debriefing credit | Unknown | Generator behavior documented; evaluator behavior unknown (F13) |

---

## Search Execution Log

*(Populated as searches are executed. Each entry follows the template below.)*

```
Search ID:
Date:
Target:
Queries used:
Artifacts examined:
Results:
Evidence obtained:
Evidence level:
Conclusion:
Follow-up required:
```

---

### S0 — Local Artifact Analysis (pre-search)

```
Search ID: S0
Date: 2026-07-16
Target: data/cvd001/ — all locally committed .cpp files and dataset files
Queries used: N/A (direct file read)
Artifacts examined:
  - credit_constraints.cpp (471 lines)
  - crew_availability_constraints.cpp (602 lines)
  - EmployeeLegPreferences.cpp (284 lines)
  - preferredVacations.cpp (241 lines)
  - DATASET-INVENTORY-v1.0.md
Results:
  All four .cpp files are instance generation tools authored by Frédéric Quesnel (and
  Atoosa Kasirzadeh for preferredVacations.cpp). None is the benchmark evaluator.
  The files originate from G1422-DataSets.zip (GERAD / Polytechnique Montréal).

  Key findings:
  - creditedHours file format formally documented from parser (credit_constraints.cpp L131–156):
    each record is `<N> <M(BASE)> : credited_hours = <value>` followed by cost and vacation lines.
  - Duty boundary definition confirmed: new calendar day in flight number chars 4–5 (LEG_DD_N).
  - Deadheads (TDH prefix) excluded from duty counting and from leg preferences in all files.
  - PAL_ prefix stripped before day extraction in both crew_availability_constraints.cpp and
    credit_constraints.cpp.
  - Briefing/debriefing credit: credit_constraints.cpp subtracts 1 unit per duty from the
    reference solution total before computing base caps. The comment says "1h briefing + 1h
    debriefing" but the code increments by 1 per duty (not 2). This is used only for cap
    generation, not confirmed as evaluator behavior.
  - SC4 (leg preferences): generated by EmployeeLegPreferences.cpp from initialSolution.in;
    deadheads excluded; output is PreferredAirLegs.csv.
  - SC5 (vacation preferences): generated by preferredVacations.cpp; vacation window
    2000-01-01 to 2000-01-31; duration 2–15 days; output is personalizedEmployees.csv.
    (Note: file comment says 2002/10 but actual code uses 2000-01-.)

Result:
  Generator source fully analyzed. Benchmark evaluator absent from local artifacts.

Evidence obtained: F9, F10, F11, F12, F13, F14, F15 (see Established Facts table)
Evidence level: E3
Conclusion: Local E3 evidence is exhausted. No evaluator source (E1) found locally.
  creditedHours format is now formally documented. Remaining open questions (Q1, Q2, Q3,
  Q5, Q6) require E1 or E2 evidence from external searches S1–S4.
Follow-up required: Proceed with S1 (GERAD archive) and S2 (ROADEF 2010 challenge page).
```

---

### S1 — GERAD Archive Search

```
Search ID: S1
Date: 2026-07-16
Target: https://www.gerad.ca/en/papers (Cahiers du GERAD)
Queries used:
  - "Quesnel crew scheduling" → No results
  - Note: GERAD Cahiers search is JavaScript-driven; URL parameters do not filter results.
    Confirmed working by observing result count change from 3247 to 0 on first successful query.
  - Direct URL attempt: https://www.gerad.ca/en/papers/G-1422 → 404 Not Found
  - Provenance identified via user-supplied reference: GERAD G-2014-22
    https://www.gerad.ca/en/papers/G-2014-22
Artifacts examined:
  - GERAD Cahiers du GERAD search interface (3247 total papers indexed)
  - GERAD Technical Report G-2014-22: "Airline crew scheduling: Models, algorithms, and data sets"
    (Kasirzadeh, Saddoune, Soumis) — Additional Material: G1422-DataSets.zip
Result:
  Located GERAD Technical Report G-2014-22, "Airline crew scheduling: Models, algorithms,
  and data sets" (Kasirzadeh, Saddoune, Soumis). The report distributes G1422-DataSets.zip
  as official supplementary material, confirming the provenance of the locally analyzed
  generator source files (F16 upgraded to E2). No evaluator source code, checker, scoring
  executable, or README describing evaluation semantics was found in the published
  supplementary material accompanying G-2014-22. The report was later published in EURO
  Journal on Transportation and Logistics (2017).

  Observation (not established fact): The published journal article acknowledges "Frédéric
  Quesnel for his help in preparing the data sets and generators" and references the GENCOL
  software library as having been provided to the research team. GENCOL is the optimization
  framework used in the experiments; it is not the benchmark evaluator. This observation
  suggests some experimental tooling was not publicly released, but does not establish that
  the evaluator is absent or that it is contained within GENCOL.

Evidence obtained: F17 (E2) — dataset provenance confirmed via G-2014-22; F16 upgraded to E2
Evidence level: E2
Conclusion: ✅ Dataset provenance established (E2). No evaluator source code or scoring
  software found in the published supplementary material of G-2014-22. Evaluator has not
  yet been located; absence from supplementary material does not establish universal
  unavailability.
Follow-up required: Proceed with S2 (ROADEF challenge pages) and S3 (authors' repositories).
```

---

### S2 — ROADEF Challenge Page Search

```
Search ID: S2
Date: 2026-07-16
Target: http://www.roadef.org/challenge/ (all years)
Queries used: Browsed challenge years 2007, 2009, 2010 via roadef.org challenge navigation
Artifacts examined:
  - ROADEF/EURO Challenge 2010: "A Large-Scale Energy Management Problem" (EDF)
  - ROADEF Challenge 2009: "Disruption Management for Commercial Aviation" (Amadeus)
  - ROADEF Challenge 2007: "Technicians and Interventions Scheduling for Telecommunications"
    (France Télécom R&D)
Result:
  No evidence was found that CVD-001 formed part of the public ROADEF challenge benchmark
  series. The ROADEF challenges examined (2007, 2009, 2010) cover energy management,
  disruption management, and telecom scheduling — none match the airline crew scheduling
  structure of CVD-001 (33 crew, 3 bases, 31-day horizon, instance1–instance7).
  The currently established provenance (F17) identifies CVD-001 as a dataset distributed
  through GERAD Technical Report G-2014-22, not a ROADEF challenge benchmark.
  Note: The original S2 search plan assumed a ROADEF 2010 connection; this assumption
  was not supported by the evidence. The search was nonetheless completed as planned.

Evidence obtained: None (E1 or E2 not found); negative finding consistent with F17 provenance
Evidence level: N/A
Conclusion: ❌ No ROADEF challenge materials matching CVD-001 found. Consistent with
  G-2014-22 provenance (F17). E1/E2 evidence for the evaluator must be sought through
  authors' institutional pages and general web search (S3, S4).
Follow-up required: Proceed with S3 (authors' institutional repositories) and S4 (web search).
```

---

## Stopping Rule Status

The stopping rule triggers when all of the following are complete:
- [x] S1: GERAD archive searched — ✅ E2 provenance recovered (G-2014-22); no E1 evaluator located
- [x] S2: ROADEF challenge pages searched — ❌ No ROADEF evidence matching CVD-001
- [ ] S3: Authors' public repositories searched
- [ ] S4: General web search completed
- [ ] S5: Direct author contact attempted (if practical)

**Current status: 2/5 complete. Key finding:** Current evidence identifies CVD-001 as a dataset distributed through GERAD Technical Report G-2014-22. No evidence was found that it formed part of the public ROADEF challenge benchmark series.

---

## Next Action

Execute S3 (authors' institutional repositories: Frédéric Quesnel, Louis-Martin Rousseau, Guy Desaulniers at Polytechnique Montréal / GERAD / CIRRELT) and S4 (general web search for the dataset package and any associated evaluator). Do not modify Coralys code until Milestone 1 is complete or the stopping rule is triggered.