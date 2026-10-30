# UltraRoster Independent Optimization Assessment Protocol v1.0
**Framework for Retrospective Hospital Workforce Optimization Audits**  
**Classification:** Commercial & Operational Methodology  
**Target Engagement:** Paid Retrospective Diagnostic (Single Department, 30–80 Beds, 40–80 Nurses, 28 Days)

---

## 1. Executive Summary & Governance Scope

### 1.1 Objective
The UltraRoster Independent Optimization Assessment is a diagnostic engagement designed to evaluate whether mathematical combinatorial optimization can produce superior, lower-cost, and more equitable workforce allocations compared to a hospital's incumbent scheduling method.

The assessment is conducted **retrospectively on anonymized historical workforce data**. It requires **no live hospital information system (HIS) integration, no HRIS replacement, and zero operational disruption** to ward operations.

### 1.2 The Core Evaluation Principle
To ensure scientific validity and commercial integrity, the hospital's historical roster and the UltraRoster candidate schedules are evaluated against **the exact same institutional policies, encoded once and audited independently**. The optimizer is never judged against rules that the hospital baseline was not held to.

```
┌────────────────────────────────────────────────────────────────────────┐
│                   HOSPITAL ENGAGEMENT BOUNDARY                         │
│                                                                        │
│   [Historical Roster & Intake Data]      [Approved Hospital Policies]   │
│             (Anonymized CSV)               (Contract, Rest, Skills)    │
│                    │                                  │                │
│                    └─────────────────┬────────────────┘                │
│                                      ▼                                 │
│                     ┌──────────────────────────────────┐               │
│                     │  Independent Rule Validator      │               │
│                     └────────────────┬─────────────────┘               │
│                                      │                                 │
│             ┌────────────────────────┴────────────────────────┐        │
│             ▼                                                 ▼        │
│   [Baseline Roster Audit]                          [Coralys MOGA Engine]│
│   - Hard violations                                - 28-day horizon    │
│   - Scheduled overtime                             - Pareto frontier   │
│   - Weekend & night distribution                   - Up to 3 candidates│
│                                                               │        │
│                                                               ▼        │
│                                                    [Validator Re-Check]│
│                                                    - Independent audit │
│                                                    - Zero breaches     │
│             ┌─────────────────────────────────────────────────┘        │
│             ▼                                                          │
│   [Comparative Assessment Report]                                      │
│   - Side-by-side violation audit                                       │
│   - Scheduled overtime delta                                           │
│   - Fairness & preference metrics                                      │
└────────────────────────────────────────────────────────────────────────┘
```

### 1.3 Scope Boundaries & Exclusions
* **In Scope:**
  * 1 Clinical Department or Ward (e.g., Intensive Care, Oncology, Surgical Ward).
  * 30 to 80 active inpatient beds.
  * 40 to 80 nursing personnel (mixed seniority: In-Charge, Senior Staff Nurse, Staff Nurse).
  * One complete 28-day planning horizon (4 full calendar weeks).
  * Retrospective anonymized workforce data exchange via structured CSV.
* **Explicitly Out of Scope:**
  * Live clinical rostering or real-time shift dispatch.
  * Direct bidirectional API integration with HIS, HRMS, or biometric punch clocks.
  * Payroll processing or direct disbursement calculation.
  * Clinical patient acuity modeling or ventilator staffing ratio determination (these are provided as approved inputs).
  * Enterprise multi-tenancy or single sign-on (SSO).

---

## 2. Standard Input Data Contract (CSV Specifications)

Data is ingested using four normalized CSV files. All personal identifiable information (PII) must be stripped prior to transfer.

### 2.1 File 1: `staff.csv`
Defines the workforce pool, certified competencies, and contractual parameters.

| Column Name | Data Type | Requirement | Description / Permitted Values |
| :--- | :--- | :--- | :--- |
| `worker_id` | String | Mandatory | Anonymized identifier (e.g., `N_01`, `RN_104`). |
| `contract_type` | String | Mandatory | `FullTime`, `PartTime`, or `NightDedicated`. |
| `skills` | String | Mandatory | Semicolon-delimited skills (e.g., `HeadNurse;Nurse`, `ICU;Ventilator`). |
| `weekly_max_hours` | Decimal | Mandatory | Contractual maximum hours per week (e.g., `40.0`, `48.0`, `32.0`). |
| `max_consecutive_days` | Integer | Mandatory | Maximum continuous working days before off (default: `5` or `6`). |
| `min_rest_hours` | Integer | Mandatory | Minimum mandatory rest gap between shifts (default: `10` or `12`). |
| `max_weekends_per_cycle` | Integer | Mandatory | Maximum number of working weekends allowed in the 28-day cycle. |

```csv
worker_id,contract_type,skills,weekly_max_hours,max_consecutive_days,min_rest_hours,max_weekends_per_cycle
N_01,FullTime,HeadNurse;Nurse,40.0,5,10,2
N_02,FullTime,Nurse,40.0,5,10,2
N_03,PartTime,Nurse,24.0,4,12,2
N_04,NightDedicated,Nurse;ICU,36.0,4,14,2
```

### 2.2 File 2: `shifts.csv`
Defines the discrete shift demand slots required across the 28-day horizon (672 hours).

| Column Name | Data Type | Requirement | Description / Permitted Values |
| :--- | :--- | :--- | :--- |
| `shift_id` | Integer | Mandatory | Unique integer identifier across the entire 28-day horizon (e.g., `1` to `N`). |
| `date` | Date | Mandatory | Calendar date (`YYYY-MM-DD`). |
| `day_idx` | Integer | Mandatory | Relative day index (`0` to `27`). |
| `shift_type` | String | Mandatory | Standard shift label: `Early` (06:00–14:00), `Late` (14:00–22:00), `Night` (22:00–06:00). |
| `start_hour` | Integer | Mandatory | Horizon start hour (`day_idx * 24 + shift_start_offset`). |
| `duration_hours` | Integer | Mandatory | Duration of duty in hours (standard: `8`). |
| `required_skill` | String | Mandatory | Minimum certified skill required to staff the position. |
| `ward` | String | Mandatory | Ward or unit code (e.g., `ICU_Ward_A`). |

```csv
shift_id,date,day_idx,shift_type,start_hour,duration_hours,required_skill,ward
1,2026-07-01,0,Early,6,8,HeadNurse,ICU_Ward_A
2,2026-07-01,0,Early,6,8,Nurse,ICU_Ward_A
3,2026-07-01,0,Late,14,8,Nurse,ICU_Ward_A
4,2026-07-01,0,Night,22,8,Nurse,ICU_Ward_A
```

### 2.3 File 3: `history_boundary.csv`
Captures historical boundary conditions immediately preceding Day 0. Essential for multi-week continuity (prevents nurses from working 10 consecutive days across the cycle boundary).

| Column Name | Data Type | Requirement | Description |
| :--- | :--- | :--- | :--- |
| `worker_id` | String | Mandatory | Matches `staff.csv`. |
| `prior_shift_end_hour` | Integer | Mandatory | Relative hour when nurse completed their last shift prior to Day 0. |
| `consecutive_work_days_prior` | Integer | Mandatory | Continuous working streak ending on Day -1. |
| `consecutive_rest_days_prior` | Integer | Mandatory | Continuous rest streak ending on Day -1. |
| `working_weekends_month_to_date` | Integer | Mandatory | Count of weekends worked in the preceding 2 weeks. |

### 2.4 File 4: `unavailability.csv`
Pre-approved planned leave, medical absences, and mandatory training sessions.

| Column Name | Data Type | Requirement | Description |
| :--- | :--- | :--- | :--- |
| `worker_id` | String | Mandatory | Matches `staff.csv`. |
| `start_date` | Date | Mandatory | Start date (`YYYY-MM-DD`). |
| `end_date` | Date | Mandatory | End date (`YYYY-MM-DD`). |
| `reason` | String | Mandatory | `AnnualLeave`, `MedicalLeave`, `Training`, or `ApprovedOff`. |

---

## 3. Hospital Policy & Constraint Questionnaire

Before any optimization is run, the Chief Nursing Officer (CNO) and Nursing Superintendent must ratify this policy questionnaire. These answers establish the objective criteria for both the baseline audit and candidate optimization.

```markdown
### SECTION A: CALENDAR & CONTRACTUAL RULES
1. Weekly Boundary Definition:
   [ ] Monday 00:00 to Sunday 23:59
   [ ] Sunday 00:00 to Saturday 23:59
   [ ] Other (specify): _____________________

2. Contractual Working Hours:
   Full-Time Standard: ____ hours/week | Maximum Allowed Before Overtime: ____ hours/week
   Part-Time Standard: ____ hours/week | Maximum Allowed Before Overtime: ____ hours/week

3. Overtime Classification:
   Is overtime permissible as an operational fallback?
   [ ] Strictly prohibited (Hard Constraint)
   [ ] Permissible with soft penalty (Soft Objective)

### SECTION B: REST & ERGONOMIC CONSTRAINTS
4. Inter-Shift Rest Period:
   Minimum hours between shifts: [ ] 10 hours  [ ] 11 hours  [ ] 12 hours  [ ] 14 hours

5. Working Streaks:
   Maximum consecutive working days: [ ] 5 days  [ ] 6 days  [ ] 7 days
   Minimum consecutive rest days following a work streak: [ ] 1 day  [ ] 2 days

6. Night Duty Policies:
   Maximum consecutive night duties allowed: [ ] 2  [ ] 3  [ ] 4
   Minimum rest hours required following a night duty cycle: [ ] 24 hours  [ ] 48 hours
   Is a Night → Early quick return strictly forbidden? [ ] Yes (Hard)  [ ] No

### SECTION C: SKILL MIX & WEEKEND DISTRIBUTION
7. Leadership Coverage:
   Is at least one Senior Nurse / Head Nurse mandatory on EVERY shift?
   [ ] Yes, mandatory on all shifts (Hard Constraint)
   [ ] Yes, but night shifts exempt if experienced nurse present
   [ ] Desirable, not mandatory (Soft Objective)

8. Weekend Equity:
   Maximum working weekends per 28-day cycle: [ ] 2 weekends  [ ] 3 weekends
   Complete weekend rule: Must both Saturday & Sunday be assigned together?
   [ ] Yes (Complete weekends)  [ ] No (Individual days acceptable)
```

---

## 4. Baseline Roster Scoring Methodology

The hospital's actual 28-day historical schedule is ingested as `hospital_baseline.csv` (`worker_id`, `shift_id`). It is audited using the authoritative validator without modification.

### 4.1 Evaluation Dimensions
1. **Mandatory Hard Constraint Violations:**
   * **HC1 (Skill / Coverage Deficit):** Shift slots staffed by personnel lacking certified competencies, or senior coverage deficits.
   * **HC2 (Double Booking & Overlap):** A worker assigned to overlapping shifts or shifts separated by $< 0$ hours.
   * **HC3 (Weekly Contract Cap Breaches):** Total assigned hours within any 168-hour calendar week exceeding the worker's contract cap (`weekly_max_hours`).
   * **HC4 (Approved Leave Incursions):** Assignments falling within approved leave windows.
   * **Rest (Inter-Shift Deficit):** Assignments with rest gaps less than the hospital-approved minimum (e.g., $< 10$ hours).
2. **Soft Operational Trade-Offs:**
   * **Fairness Variance ($\sigma^2$):** Coefficient of variation in total hours allocated across full-time nursing cohorts.
   * **Night Duty Skew:** Variance in night shift burden across peer nurses.
   * **Weekend Distribution:** Number of nurses exceeding 2 working weekends in the cycle.
3. **Scheduled Overtime Exposure:**
   Sum of all scheduled hours allocated above weekly contractual caps:
   $$\text{Scheduled Overtime Hours} = \sum_{w \in \text{workers}} \sum_{k=1}^{4} \max\left(0, \text{Hours}_{w,k} - \text{ContractCap}_w\right)$$

---

## 5. Optimization Experiment & Pareto Candidate Generation

Using the ratified planning contract, UltraRoster executes the Coralys Multi-Objective Genetic Algorithm (MOGA) over the full 28-day horizon (672 hours).

### 5.1 Optimization Architecture
* **Horizon Representation:** Global 28-day assignment matrix ($W$ workers $\times$ $S$ discrete shifts).
* **Multi-Objective Search:** MOGA evaluates solutions on a 6-dimensional fitness vector:
  $$\vec{F} = \left[ f_{\text{hard}}, f_{\text{rest}}, f_{\text{fairness}}, f_{\text{fatigue}}, f_{\text{hc1}}, f_{\text{hc3}} \right]$$
* **Non-Dominated Archive:** Preserves the Pareto-optimal frontier where no candidate can improve one objective without degrading another.

### 5.2 Standard Candidate Deliverables
The optimizer extracts **up to three materially differentiated feasible candidates**:

| Candidate ID | Strategic Profile | Optimization Priority |
| :--- | :--- | :--- |
| **Candidate A** | **Cost & Overtime Minimized** | Eliminates scheduled overtime; balances contract thresholds to zero excess hours. |
| **Candidate B** | **Workforce Equity & Preferences** | Minimizes workload variance ($\sigma^2$); strictly equalizes weekend and night duties across peers. |
| **Candidate C** | **Clinical & Ergonomic Continuity** | Minimizes shift rotation turbulence; maximizes consecutive days off after night duties. |

---

## 6. Independent Feasibility & Audit Verification Procedure

The candidate schedules returned by the optimizer must pass an independent verification gate before presentation.

### 6.1 The Re-Computation Standard
* **Decoupled Verification:** The verification tool must **never read cached optimizer scores or penalty weights**.
* It takes only two artifacts:
  1. The raw assignment matrix: `HashMap<shift_id, worker_id>`
  2. The approved planning contract (`staff.csv`, `shifts.csv`, `unavailability.csv`, ratified policies).
* It reconstructs the entire timeline from raw assignments and re-computes every rule.

### 6.2 Acceptance Gate
A candidate schedule is approved for customer presentation **if and only if**:
$$\text{Hard Violations} = 0 \quad (\text{HC1} = 0, \, \text{HC2} = 0, \, \text{HC3} = 0, \, \text{HC4} = 0, \, \text{Rest Violations} = 0)$$

Any candidate containing even one mandatory violation is automatically disqualified.

---

## 7. Financial & Operational Measurement Rules

To ensure CFO-ready credibility, UltraRoster strictly enforces the separation between paper productivity and cash expenditure.

### 7.1 Cash Savings vs. Capacity Reinvestment

```
┌────────────────────────────────────────────────────────────────────────┐
│                        FINANCIAL IMPACT TAXONOMY                       │
│                                                                        │
│   CATEGORY 1: VERIFIED CASH BENEFIT (CFO Metrics)                      │
│   • Avoided Overtime: Direct reduction in payroll overtime payouts.    │
│   • Avoided Locum/Agency: Elimination of external nursing invoices.     │
│   • Formula:                                                           │
│     Cash Savings = Δ Overtime Paid + Δ Agency Billed - Software Cost   │
│                                                                        │
│   CATEGORY 2: OPERATIONAL CAPACITY REINVESTMENT (CNO Metrics)          │
│   • Supervisor Time Released: ~30-40 hours/month freed from scheduling. │
│   • Classification: Clinical care reinvestment / supervision hours.    │
│   • Rule: NEVER multiply supervisor hours by nurse wages and report as │
│           "cash-out-the-door savings" unless headcount is eliminated.  │
│                                                                        │
│   CATEGORY 3: RETENTION & RISK MITIGATION (COO Metrics)                │
│   • Regulatory Legal Compliance: Zero statutory rest breaches.         │
│   • Reduced Attrition Exposure: Equitable night and weekend balance.   │
└────────────────────────────────────────────────────────────────────────┘
```

### 7.2 Financial Attribution Protocol
1. **Historical Assessment Phase:**
   Calculate **Scheduled Overtime Exposure** (hours above contract cap in the roster). Do not claim historical exposure is realized cash savings until correlated with payroll logs.
2. **Controlled Shadow / Live Pilot Phase:**
   Compare the optimized period against an equivalent historical baseline (matching bed occupancy, patient acuity, and nursing headcount). Calculate actual payroll variance:
   $$\Delta \text{Overtime Cost} = \text{Baseline Paid Overtime} - \text{Pilot Paid Overtime}$$

---

## 8. Final Customer Assessment Deliverable Report (Template)

Upon completion of the audit, the hospital receives the following executive presentation and technical appendix.

```markdown
# UltraRoster Optimization Audit Report
**Hospital:** [Anonymized Hospital Name] | **Department:** [ICU / Surgical Ward]
**Dates Evaluated:** 28-Day Cycle ([Start Date] to [End Date])
**Workforce Evaluated:** [N] Nurses | [M] Shifts Scheduled

---

### Executive Scorecard: Baseline vs. Optimized Candidates

| Metric | Hospital Historical Baseline | Candidate A (Cost Minimized) | Candidate B (Equity Balanced) | Operational Significance |
| :--- | :---: | :---: | :---: | :--- |
| **Mandatory Hard Violations** | **14** | **0** | **0** | Regulatory & legal compliance |
| • Rest breaches (<10h gap) | 8 | 0 | 0 | Fatigue risk / quick returns |
| • Weekly hour cap breaches | 4 | 0 | 0 | Contract compliance |
| • Uncovered senior nurse shifts | 2 | 0 | 0 | Clinical leadership coverage |
| • Leave clashes | 0 | 0 | 0 | Approved leave protection |
| **Scheduled Overtime Exposure** | **112 hours** | **0 hours** | **16 hours** | Identifies avoidable payroll costs |
| **Workload Variance ($\sigma^2$)** | 18.4 | 4.2 | 1.8 | Cohort equity and burnout prevention |
| **Max Working Weekends** | 4 (3 nurses) | 2 (0 nurses) | 2 (0 nurses) | Weekend fairness across cohort |
| **Night Shift Load Variance** | ± 4 shifts | ± 1 shift | 0 shifts | Equal night duty burden |

---

### Key Operational Findings

1. **Rest Period Compliance:**
   The historical roster contained 8 instances where nurses had less than 10 hours between consecutive shifts (e.g., Late shift ending at 22:00 followed immediately by an Early shift at 06:00). UltraRoster's candidates eliminate 100% of these occurrences.

2. **Scheduled Overtime Exposure:**
   In the baseline schedule, 112 hours were scheduled beyond contractual 40h/week caps across 7 nurses, primarily due to localized coverage gaps. Candidate A meets 100% of shift requirements with zero scheduled overtime through optimized skill-matching.

3. **Weekend & Night Fairness:**
   In the historical schedule, 3 nurses were assigned to work all 4 weekends in the month, while others worked only 1. Candidate B caps weekend assignments at 2 per nurse across the entire team.

---

### Recommended Action Plan
1. Review candidate assignments with ward supervisors for clinical qualitative feedback.
2. Cross-reference the 112-hour scheduled overtime exposure against actual payroll payouts.
3. Conduct a 4-week prospective shadow observation to validate operational adoption.
```

---

## 9. Engagement Governance & Data Protection Agreement

1. **Data Minimization:** Only anonymized worker IDs, certification categories, and historical shift assignments are ingested. No patient information, employee names, national IDs, or home addresses are ever collected.
2. **Access Control:** Data is processed in an isolated single-tenant workspace. No customer data is shared or retained across client boundaries.
3. **Retention & Disposal:** All raw CSV files, intermediate genomes, and assessment databases are permanently sanitized and deleted within 30 days of deliverable delivery upon written request.
4. **Independent Intellectual Property:** The hospital retains complete ownership of its operational data and policies. Coralys retains all rights to its combinatorial solvers, MOGA algorithms, and optimization architectures.

