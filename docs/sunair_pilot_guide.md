# UltraCrew Pilot Guide — SunAir Demo
**P-001 · S1-07 · Operator-Facing Reference**
Version 1.0 — 2026-07-22

---

## Purpose

This guide is written for the operations team running the UltraCrew scheduling system during the SunAir pilot. It covers everything needed to go from a blank machine to a verified schedule in a single working session: environment setup, data preparation, running the optimizer, reading the outputs, and handling common problems.

No Rust or Python expertise is required to operate the system. The guide assumes you can open a terminal and run commands.

---

## Table of Contents

1. [System Requirements](#1-system-requirements)
2. [Installation](#2-installation)
3. [Data Preparation](#3-data-preparation)
4. [Running the Optimizer](#4-running-the-optimizer)
5. [Reading the Outputs](#5-reading-the-outputs)
6. [KPI Dashboard](#6-kpi-dashboard)
7. [Exporting Results](#7-exporting-results)
8. [Troubleshooting](#8-troubleshooting)
9. [Glossary](#9-glossary)
10. [Support Contacts](#10-support-contacts)

---

## 1. System Requirements

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| Operating System | macOS 12, Ubuntu 22.04, Windows 11 (WSL2) | macOS 14 / Ubuntu 24.04 |
| CPU | 4 cores | 8+ cores |
| RAM | 4 GB | 16 GB |
| Disk | 500 MB free | 2 GB free |
| Rust toolchain | 1.75 | 1.78+ (stable) |
| Python | 3.10 | 3.12 |
| Network | Required for initial install only | — |

---

## 2. Installation

### 2.1 Clone the repository

```bash
git clone https://github.com/your-org/ultracrew.git
cd ultracrew
```

### 2.2 Build the CLI binary

```bash
cargo build --release -p ultracrew-cli
```

The binary is placed at `target/release/ultracrew-cli`. Build time is approximately 2–4 minutes on first run; subsequent builds are incremental.

### 2.3 Verify the build

```bash
./target/release/ultracrew-cli --help
```

Expected output includes flags: `--input`, `--output`, `--profile`, `--generations`.

### 2.4 Python dependencies (for reporting scripts)

```bash
python3 -m pip install --upgrade pip
# No third-party packages required — scripts use stdlib only
```

---

## 3. Data Preparation

### 3.1 Input file format

The optimizer accepts a single JSON file. The canonical SunAir demo file is at `fixtures/demo/sunair_demo.json`. For a live scheduling run, prepare a file with the same structure:

```json
{
  "workers": [
    {
      "id": 1,
      "skills": ["Captain"],
      "historical_workloads": [40, 38, 42, 36]
    }
  ],
  "shifts": [
    {
      "id": 1,
      "start_hour": 6,
      "duration_hours": 8,
      "required_skill": "Captain"
    }
  ],
  "rng_seed": 42,
  "generation_limit": 500,
  "scenario": {
    "planning_horizon_hours": 168.0,
    "max_hours_per_worker": 48.0
  }
}
```

**Field reference:**

`workers[].id` — unique integer, must be stable across runs for reproducibility.

`workers[].skills` — list of skill strings. Valid values for SunAir: `"Captain"`, `"FirstOfficer"`, `"CabinCrew"`.

`workers[].historical_workloads` — list of integers (hours worked in each of the last N weeks). Used by the fairness objective. Provide at least 4 weeks.

`shifts[].start_hour` — hour offset from the start of the planning horizon (0 = midnight on day 1). A value of 6 means 06:00 on day 1.

`shifts[].duration_hours` — integer. All SunAir shifts are 8 hours.

`rng_seed` — integer. Fix this to the same value across runs to get deterministic results. The SunAir canonical seed is `42`.

`generation_limit` — number of genetic algorithm generations. Higher values improve solution quality at the cost of runtime. `500` is the SunAir standard; `200` is acceptable for quick iteration.

`scenario.max_hours_per_worker` — hard cap on total hours per worker in the planning horizon. SunAir contract limit is `48.0`.

### 3.2 Validating your input file

```bash
python3 -c "import json; json.load(open('your_scenario.json')); print('JSON valid')"
```

Common mistakes: trailing commas, missing closing braces, non-integer worker IDs.

### 3.3 CSV alternative

If your source data is in spreadsheet form, use the provided CSV templates:

- `fixtures/demo/sunair_workers.csv` — columns: `id`, `skills`, `historical_hours`
- `fixtures/demo/sunair_shifts.csv` — columns: `id`, `start_hour`, `duration_hours`, `required_skill`

A conversion script from CSV to JSON is planned for P-001 Stream 2.

---

## 4. Running the Optimizer

### 4.1 Standard run (SunAir demo)

```bash
./target/release/ultracrew-cli \
  --input  fixtures/demo/sunair_demo.json \
  --output fixtures/demo/sunair_schedule.json
```

Expected runtime: 10–15 seconds on a modern laptop. The optimizer prints a KPI summary to stderr on completion:

```
══════════════════════════════════════════
  UltraCrew Schedule — Optimization Complete
══════════════════════════════════════════
  Coverage:       42/42 shifts (100.0%)
  Hard violations: 0
  Rest violations: 0
  Fitness score:  8649.6000
  Fairness penalty: 697.6000
  Fatigue penalty:  652.8000
  Workload balance: mean 16.8h, min 8h, max 32h
  Runtime:        11.27s
══════════════════════════════════════════
```

### 4.2 Quick iteration run (fewer generations)

```bash
./target/release/ultracrew-cli \
  --input  fixtures/demo/sunair_demo.json \
  --output /tmp/quick_schedule.json \
  --generations 200
```

Use this during data preparation to verify your input file parses correctly. Solution quality will be lower but runtime drops to ~4 seconds.

### 4.3 Saving stderr output

```bash
./target/release/ultracrew-cli \
  --input  fixtures/demo/sunair_demo.json \
  --output fixtures/demo/sunair_schedule.json \
  2> run_log.txt
```

The `run_log.txt` file captures the KPI summary for audit purposes.

### 4.4 Reproducibility

The same `rng_seed` and `generation_limit` on the same binary version will always produce the same output. If results differ between runs, check that the binary version matches (`git log --oneline -1`) and that the input file is byte-identical.

---

## 5. Reading the Outputs

### 5.1 Raw schedule file (`sunair_schedule.json`)

The optimizer writes a JSON file with the following top-level fields:

| Field | Type | Meaning |
|-------|------|---------|
| `assignments` | object | Map of `shift_id → worker_id` for every assigned shift |
| `fitness` | float | Optimizer objective value (higher = better solution) |
| `hard_violations` | int | Number of skill mismatches or double-bookings. **Must be 0 for a valid schedule.** |
| `rest_violations` | int | Number of minimum-rest breaches. **Must be 0 for a compliant schedule.** |
| `fairness_penalty` | float | Workload imbalance cost (lower = more even distribution) |
| `fatigue_penalty` | float | Consecutive-shift fatigue cost (lower = better rest patterns) |
| `recommendations` | list | Optimizer-generated advisory notes |
| `telemetry` | object | Internal optimizer diagnostics (for support use) |

A schedule is **operationally valid** when `hard_violations == 0` and `rest_violations == 0`. The SunAir demo achieves both.

### 5.2 Enriched report (`sunair_report.json`)

Run the reporting script to produce a human-readable enriched report:

```bash
python3 scripts/gen_sunair_report.py
```

Output: `fixtures/demo/sunair_report.json`

The report adds per-worker workload breakdown, skill coverage percentages, and a full sorted assignment list. This is the file to share with airline operations management.

### 5.3 Interpreting assignments

Each entry in `assignments` maps a shift ID to the worker ID assigned to cover it:

```json
"assignments": {
  "1": 3,
  "2": 1,
  ...
}
```

Shift 1 is covered by Worker 3, Shift 2 by Worker 1, and so on. Cross-reference with `fixtures/demo/sunair_shifts.csv` to get the start time and skill for each shift.

---

## 6. KPI Dashboard

Open the dashboard in any modern browser — no server required:

```bash
open fixtures/demo/sunair_kpi_dashboard.html
# or on Linux:
xdg-open fixtures/demo/sunair_kpi_dashboard.html
```

The dashboard shows:

**KPI Cards** — at-a-glance status for coverage %, violations, fitness score, overtime workers, mean hours per worker, and penalty components.

**Shift Coverage by Skill** — donut chart showing how many shifts each skill category (Captain, FirstOfficer, CabinCrew) contributes.

**Worker Hours Distribution** — bar chart of hours worked per worker. Bars shown in amber indicate workers exceeding the overtime threshold (24 h). The contract maximum (48 h) is the y-axis ceiling.

**Penalty Breakdown** — donut chart splitting the total penalty between fairness (workload imbalance) and fatigue (consecutive-shift patterns).

**Worker Workload Summary** — full table with per-worker shift count, hours, utilisation bar, and overtime status.

**Understaffed Shifts** — table listing all 42 shifts with their coverage status. In the SunAir demo all shifts show "Covered".

---

## 7. Exporting Results

### 7.1 JSON report

```bash
python3 scripts/gen_sunair_report.py
# Output: fixtures/demo/sunair_report.json
```

### 7.2 INRC-II XML export

For interoperability with third-party rostering tools that consume the INRC-II standard:

```bash
python3 scripts/gen_sunair_inrc_xml.py
# Output: fixtures/demo/sunair_inrc_export.xml
```

The XML file contains the full scenario definition (skills, shift types, employees, contracts, cover requirements) and the optimizer solution (shift assignments and solution quality metadata).

### 7.3 CSV data

The worker and shift CSV files are pre-generated at:

- `fixtures/demo/sunair_workers.csv`
- `fixtures/demo/sunair_shifts.csv`

These can be opened directly in Excel or Google Sheets.

---

## 8. Troubleshooting

### Build fails: `error[E0433]: failed to resolve`

The Rust workspace is missing a dependency. Run:

```bash
cargo update
cargo build --release -p ultracrew-cli
```

### Optimizer exits with `hard_violations > 0`

This means the scenario has more shifts requiring a skill than workers possessing that skill. Check your worker skill assignments and shift `required_skill` values. Ensure the number of workers per skill is at least equal to the maximum number of concurrent shifts requiring that skill.

### Optimizer exits with `rest_violations > 0`

Two shifts assigned to the same worker overlap or are separated by fewer than the minimum rest period. Increase the gap between consecutive shifts in your scenario, or add more workers.

### `sunair_schedule.json` not found when running reporting scripts

The optimizer must be run first to produce the raw schedule file. Run step 4.1 before running any reporting scripts.

### Python script exits with `KeyError`

The `sunair_demo.json` and `sunair_schedule.json` files must be consistent — generated from the same scenario. If you have modified the scenario, re-run the optimizer before running the reporting scripts.

### Dashboard shows no charts

The dashboard requires an internet connection to load Chart.js from the CDN (`cdn.jsdelivr.net`). If running in an air-gapped environment, download Chart.js locally and update the `<script src>` tag in `sunair_kpi_dashboard.html` to point to the local file.

### Results differ between machines

Verify that the `rng_seed` in the input JSON is identical and that the binary was built from the same git commit (`git log --oneline -1`). Cross-platform floating-point differences are possible in rare cases; contact support if results diverge by more than 0.1% on fitness.

---

## 9. Glossary

**Coverage %** — percentage of shifts that have been assigned a qualified worker. Target: 100%.

**Hard violation** — a constraint breach that makes the schedule operationally invalid: a shift assigned to a worker without the required skill, or a worker assigned to two overlapping shifts. Must be zero.

**Rest violation** — a breach of the minimum rest period between consecutive shifts for a single worker. Must be zero for regulatory compliance.

**Fitness score** — the optimizer's internal objective value. Higher is better. Composed of coverage reward minus penalty terms.

**Fairness penalty** — cost term penalising unequal workload distribution across workers. Lower values indicate more even scheduling.

**Fatigue penalty** — cost term penalising consecutive shift patterns that accumulate fatigue. Lower values indicate better rest patterns.

**Generation limit** — the number of evolutionary algorithm iterations. More generations improve solution quality at the cost of runtime.

**RNG seed** — the random number generator seed. Fixing this value ensures deterministic, reproducible results.

**Planning horizon** — the total time window being scheduled, in hours. SunAir demo: 168 hours (7 days).

**INRC-II** — International Nurse Rostering Competition format, version 2. An XML-based standard for crew/nurse scheduling data exchange.

---

## 10. Support Contacts

For issues during the pilot, contact the UltraCrew engineering team:

| Issue type | Contact |
|------------|---------|
| Build or installation problems | Engineering — file a GitHub issue with `[pilot]` prefix |
| Incorrect schedule output | Engineering — attach `sunair_schedule.json` and `sunair_demo.json` |
| Dashboard not loading | Engineering — note browser version and OS |
| Data preparation questions | Operations lead |
| Contract or regulatory questions | Airline operations management |

Include the output of `git log --oneline -1` and your operating system version in all support requests.

---

*UltraCrew Pilot Guide v1.0 — P-001 Stream 1 · S1-07 — 2026-07-22*
*Coralys Scheduling Engine — SunAir Demo (seed 42, 500 generations)*
