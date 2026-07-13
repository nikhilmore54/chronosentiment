# CVD-001 Dataset Inventory
**Document:** DATASET-INVENTORY-v1.0.md
**Date:** 2025-07-13
**Status:** FROZEN — Sprint 9 Milestone 1
**Dataset:** G1422-DataSets.zip (GERAD / Polytechnique Montréal)
**Instance examined:** instance1

---

## Archive Contents

| File | Size | Purpose |
|---|---|---|
| `README.pdf` | 146 KB | Dataset documentation |
| `instance1.zip` – `instance7.zip` | 32–165 KB each | Airline scheduling instances |
| `instances.zip` | 659 KB | All instances bundled |
| `instances_and_generators.zip` | 820 KB | Instances + generator source |
| `generators.zip` | 161 KB | Generator source only |
| `credit_constraints.cpp` | 14 KB | Credit constraint generator |
| `crew_availability_constraints.cpp` | 20 KB | Crew availability generator |
| `EmployeeLegPreferences.cpp` | 9 KB | Employee leg preference generator |
| `preferredVacations.cpp` | 8 KB | Vacation preference generator |
| `params.txt` | 393 B | Generator parameters |
| `logo_gerad.gif` | 1 KB | Institutional logo |
| `page_web.html` | 1 KB | Web page |

**Total:** 18 files, 2.5 MB

---

## Instance1 File Inventory

| File | Description |
|---|---|
| `day_1.csv` – `day_31.csv` | Flight legs per calendar day (31 files) |
| `listOfBases.csv` | Airport list with base status and employee counts |
| `crew_avail_const.csv` | Per-base, per-day crew availability counts |
| `crew_avail_const_avg.csv` | Average crew availability (summary) |
| `credit_constrains.csv` | Per-base credit hour targets |
| `creditedHours` | Reference solution credited hours and costs |
| `creditedHours~` | Backup file (ignore) |
| `solution_0` | Reference solution (crew schedules) |
| `initialSolution.in` | Initial solution input |
| `.csv` | Empty file (artifact) |
| `enleveCarry.sh~` | Shell script backup (ignore) |
| `convertMaison.sh~` | Shell script backup (ignore) |

---

## Domain Entities

### 1. Airports and Bases

**File:** `listOfBases.csv`
**Columns:** `airport, status, nbEmployees`

| Entity | Count | Details |
|---|---|---|
| Active bases (status=1) | 3 | BASE1 (7 crew), BASE2 (20 crew), BASE3 (6 crew) |
| Non-base airports (status=0) | 20 | AIR1–AIR23 (excluding BASE positions) |
| Total airports | 23 | |

Crew are based at one of the three bases. Flights connect bases to non-base airports and back.

---

### 2. Flight Legs

**Files:** `day_1.csv` – `day_31.csv`
**Columns:** `leg_nb, airport_dep, date_dep, hour_dep, airport_arr, date_arr, hour_arr`

| Property | Value |
|---|---|
| Planning horizon | 31 days (2000-01-01 to 2000-01-31) |
| Leg ID format | `LEG_DD_N` (DD = day number, N = leg index) |
| Leg indices | Non-contiguous (gaps in numbering are normal) |
| Cross-midnight legs | Present (date_arr may be date_dep + 1) |
| Duplicate legs | Present (some legs appear with identical times) |

**Example (day_1.csv):**
```
LEG_01_0 , BASE1 , 2000-01-01 , 12:00 , AIR1  , 2000-01-01 , 13:13
LEG_01_1 , AIR1  , 2000-01-01 , 14:05 , BASE2 , 2000-01-01 , 15:19
LEG_01_5 , BASE2 , 2000-01-01 , 23:11 , AIR4  , 2000-01-02 , 00:22
```

---

### 3. Crew Availability

**File:** `crew_avail_const.csv`
**Structure:** Per-base, per-day crew count available (with 9.2% slack added)

| Base | Min available | Max available |
|---|---|---|
| BASE1 | 0 (day 22–24) | 5 (day 4) |
| BASE2 | 6 (day 1) | 11 (day 11) |
| BASE3 | 1 | 3 |

Total crew: BASE1=7, BASE2=20, BASE3=6 → **33 crew members total**

---

### 4. Credit Constraints

**File:** `credit_constrains.csv`
**Structure:** Per-base credit hour targets (3% slack added)

| Base | Credit target (hours) | Proportion |
|---|---|---|
| BASE1 | 326.9 | 16.4% |
| BASE2 | 1279.4 | 64.3% |
| BASE3 | 383.3 | 19.3% |

Credit hours represent the total flying time assigned to crew at each base over the planning period.

---

### 5. Reference Solution

**File:** `solution_0`
**Format:** One schedule per crew member

```
schedule N EMP_ID (BASE) : ACTIVITY→ACTIVITY→...→ACTIVITY;
```

**Activity types observed:**

| Activity | Description | Mapping status |
|---|---|---|
| `LEG_DD_N` | Active flight leg (crew flies) | Maps to Shift |
| `PAL_LEG_DD_N` | Deadhead/positioning leg (crew travels as passenger) | Partial — no Coralys equivalent |
| `VACATION` | Rest/vacation day | Maps to Off day |
| `POST_PAIRING` | Post-pairing rest period | Maps to rest constraint |
| `POST_COURRIEL` | Post-email/administrative activity | No Coralys equivalent |
| `TDH_AGR_DD_N` | Training/agreement activity | No Coralys equivalent |
| `AST_VACATION_N` | Assisted/partial vacation | No Coralys equivalent |

**Example schedule:**
```
schedule 5 EMP018 (BASE2) : LEG_01_6→PAL_LEG_01_8→LEG_01_9→LEG_02_32→...→POST_PAIRING;
```

---

### 6. Credited Hours

**File:** `creditedHours`
**Per-schedule metrics:**

| Metric | Description |
|---|---|
| Credited hours | Total flying hours assigned to this crew member |
| Schedule cost | Optimizer cost for this schedule |
| Number of vacations | Rest/vacation days in schedule |

**Example:**
```
Schedule 2 (BASE2): credited hours=82.2, cost=10143.7, vacations=11
Schedule 5 (BASE2): credited hours=61.2, cost=3408.0, vacations=23
```

---

### 7. Generator Parameters

**File:** `params.txt`

| Parameter | Value |
|---|---|
| Aircraft types | NW_727, NW_757, NW_D94, NW_D95, NW_DC9, NW_319, NW_320 |
| Preferred vacation employee % | 30% |
| Employee leg preference % | 10% |
| Credit constraint slack | 3% |
| Credit proportions | 16.4% / 64.3% / 19.3% (BASE1/2/3) |

---

## Summary Statistics (instance1)

| Metric | Value |
|---|---|
| Planning horizon | 31 days |
| Total crew | 33 (BASE1=7, BASE2=20, BASE3=6) |
| Active bases | 3 |
| Non-base airports | 20 |
| Total airports | 23 |
| Day files | 31 |
| Legs per day (day_1) | ~35 |
| Activity types in solution | 7 |
| Aircraft types (params) | 7 |

---

## Data Quality Notes

1. **Non-contiguous leg numbering:** Leg indices within a day are not sequential (e.g., day_1 has LEG_01_0, LEG_01_1, LEG_01_2, LEG_01_3, LEG_01_4, LEG_01_5, LEG_01_6, LEG_01_8 — no LEG_01_7). This is expected; gaps indicate legs that were filtered or not generated for this instance.

2. **Cross-midnight legs:** Some legs depart on day N and arrive on day N+1. The adapter must handle date arithmetic correctly.

3. **Duplicate legs:** Some legs appear with identical departure/arrival times (e.g., LEG_01_50 through LEG_01_54 in day_1.csv). These may represent multiple crew positions on the same flight.

4. **Backup files:** `creditedHours~`, `enleveCarry.sh~`, `convertMaison.sh~` are editor backup files. Ignore.

5. **Empty .csv file:** The `.csv` file in instance1 is empty (0 bytes). Ignore.

6. **Crew IDs in solution:** Employee IDs in solution_0 are `EMP001`–`EMP033` format, not matching the base-prefixed format in crew_avail_const.csv. The adapter must reconcile these.

---

## Next Step

Proceed to `SCHEMA-MAPPING-v1.0.md` — domain translation from airline concepts to UltraCrew/Coralys model.