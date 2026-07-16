# Sprint 10 — Milestone 1: Benchmark Evidence Acquisition & Provenance

**Document:** SPRINT10-M1-EVIDENCE-ACQUISITION.md  
**Date:** 2026-07-16  
**Status:** IN PROGRESS  
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

| Artifact | Status | Evidence Level | Location / Notes |
|---|---|---|---|
| Benchmark evaluator source code | ⚠ Not found in local artifacts | E1 | Not in instance1/; not in benchmark source files (b8b2a9c2); external search pending |
| README.pdf | ⚠ Not found in local artifacts | E2 | Not on local filesystem; not in instance1/; external search pending |
| GERAD technical reports (G-2010-xx) | 🔍 Pending | E2 | https://www.gerad.ca/en/papers |
| ROADEF 2010 challenge documentation | 🔍 Pending | E2 | http://www.roadef.org/challenge/2010/ |
| Authors' public repositories | 🔍 Pending | E2 | Quesnel, Rousseau, Desaulniers — Polytechnique Montréal / GERAD |
| Author correspondence | ⏸ Not attempted | E2 | Pending Searches S1–S4 |
| credit_constraints.cpp | ✅ Found | E3 | data/cvd001/credit_constraints.cpp (b8b2a9c2) |
| crew_availability_constraints.cpp | ✅ Found | E3 | data/cvd001/crew_availability_constraints.cpp (b8b2a9c2) |
| EmployeeLegPreferences.cpp | ✅ Found | E3 | data/cvd001/EmployeeLegPreferences.cpp (b8b2a9c2) |
| preferredVacations.cpp | ✅ Found | E3 | data/cvd001/preferredVacations.cpp (b8b2a9c2) |
| params.txt | ✅ Found | E3 | data/cvd001/params.txt (b8b2a9c2) |
| credit_constrains.csv | ✅ Found | E4 | data/cvd001/instance1/credit_constrains.csv — per-base caps: BASE1=326.9h, BASE2=1279.4h, BASE3=383.3h |
| crew_avail_const.csv | ✅ Found | E4 | data/cvd001/instance1/crew_avail_const.csv |
| creditedHours (binary) | ✅ Found | E4 | data/cvd001/instance1/creditedHours — binary executable; content not yet parsed |
| day_1.csv … day_31.csv | ✅ Found | E4 | data/cvd001/instance1/day_*.csv — 31 daily flight leg files |
| listOfBases.csv | ✅ Found | E4 | data/cvd001/instance1/listOfBases.csv |
| solution_0 (reference solution) | ✅ Found | E5 | data/cvd001/instance1/solution_0 — reference solution from benchmark authors |
| initialSolution.in | ✅ Found | E5 | data/cvd001/instance1/initialSolution.in |

---

## Established Facts

These facts have passed the evidence threshold and are distinguished from hypotheses.

| Fact | Source | Evidence Level | Confidence |
|---|---|---|---|
| CVD-001 has 33 crew members | instance1/ CSV files | E4 | High |
| CVD-001 has 1013 active flight legs over 31 days | instance1/ CSV files | E4 | High |
| credit_constrains.csv contains per-base aggregate caps | credit_constrains.csv | E4 | High |
| Caps are BASE1=326.9h, BASE2=1279.4h, BASE3=383.3h | credit_constrains.csv | E4 | High |
| credit_constraints.cpp generates credit_constrains.csv from a reference solution with 3% slack | credit_constraints.cpp | E3 | High |
| 33/33 workers exceed 40h from historical workloads alone | scripts/hc3_audit.py | E4 | High |
| Historical workload range: 23.7h–84.9h, mean 68.75h | scripts/hc3_audit.py | E4 | High |
| Total assigned flight hours (Run 1): 1878.50h | scripts/hc3_audit.py | E4 | High |

---

## Open Questions

| ID | Question | Highest Current Evidence | Confidence |
|---|---|---|---|
| Q1 | Is HC3 a hard feasibility constraint? | E3–E4 | Low |
| Q2 | Are per-base credit caps enforced during optimization? | E3–E4 | Low |
| Q3 | How are credited hours accumulated across a bid period? | E4 | Medium |
| Q4 | Does a benchmark evaluator implementation exist publicly? | None | — |
| Q5 | What is the intended planning horizon for HC3 (weekly / monthly / bid-period)? | E6 (hypothesis) | Low |

---

## Search Plan

### S1 — GERAD Archive

Target: https://www.gerad.ca/en/papers  
Query: "ROADEF 2010" OR "crew scheduling" OR "G-2010" OR "Quesnel"  
Looking for: Technical report with evaluator description or problem statement  
Status: 🔍 Pending

### S2 — ROADEF 2010 Challenge Page

Target: http://www.roadef.org/challenge/2010/  
Looking for: Problem statement PDF, evaluator download, supplementary materials  
Status: 🔍 Pending

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

## Search Execution Log

*(Populated as searches are executed)*

---

## Stopping Rule Status

The stopping rule triggers when all of the following are complete:
- [ ] S1: GERAD archive searched
- [ ] S2: ROADEF 2010 materials searched
- [ ] S3: Authors' public repositories searched
- [ ] S4: General web search completed
- [ ] S5: Direct author contact attempted (if practical)

**Current status: 0/5 complete.**

---

## Next Action

Execute S1 (GERAD archive) and S2 (ROADEF 2010 challenge page). Record results in the Search Execution Log above. Do not modify Coralys code until Milestone 1 is complete or the stopping rule is triggered.