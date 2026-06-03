# Soak Design for BTCUSD / ETHUSD / SOLUSD

## Overview

**Baseline comparison commit:** `cbf0f859148b9b0f3497a38aa44cd2d441166d23`
**Current HEAD commit:** `7a028394`

The purpose of this soak is **not** merely to confirm that a live feed for a symbol works, but to verify that the symbol can act as a **stable replay identity** across the entire chronology lifecycle – from historical data ingestion through live capture and back‑to‑back replay verification.

The purpose of this soak is **not** merely to confirm that a live feed for a symbol works, but to verify that the symbol can act as a **stable replay identity** across the entire chronology lifecycle – from historical data ingestion through live capture and back‑to‑back replay verification.

The design introduces three complementary layers:

| Layer | Scope | Primary purpose |
|------|-------|-----------------|
| **Historical Cohort** | Bounded historical window (e.g. `2026-04-01 → 2026-05-01` or last 30 days) | Validate chronology generation, manifest determinism, timestamp semantics, and replay certification using the same provider pipelines that produced the original certified data. |
| **Live Cohort** | Real‑time capture for a short window (15–30 min) | Verify monotonic timestamps, absence of duplicate folds, manifest stability, and that the live ingestion path does not break any governance tooling. |
| **Historical → Live Boundary** | Cross‑provider continuity check (Yahoo Finance ↔ Binance) | Ensure that the same symbol identity can be reproduced across providers without hidden translation or namespace mismatches. |

---

## Provider Identity Declaration

For each provider we explicitly record the observable attributes of the symbol feed. This prevents later implicit assumptions and treats each provider's feed as a **distinct identity** until continuity is proven.

| Provider | Symbol | Quote Asset | Timestamp Unit (observed) | Resolution |
|----------|--------|------------|---------------------------|------------|
| Yahoo    | BTCUSD | USD | ms (epoch milliseconds) | 1 m / 5 m |
| Binance  | BTCUSD | USD | ms (epoch milliseconds) | 1 m / 5 m |
| Yahoo    | ETHUSD | USD | ms | 1 m / 5 m |
| Binance  | ETHUSD | USD | ms | 1 m / 5 m |
| Yahoo    | SOLUSD | USD | ms | 1 m / 5 m |
| Binance  | SOLUSD | USD | ms | 1 m / 5 m |

> **Note:** The table captures the *observed* timestamp unit; any deviation will be flagged during the soak.

---

## Detailed Test Steps

### 1. Historical Cohort
1. **Select symbols**: `BTCUSD`, `ETHUSD`, `SOLUSD`.
2. **Define window**: e.g. `2026-04-01` to `2026-05-01` (or last 30 days).
3. Run the existing ingestion scripts (e.g. `scripts/ingest_historical.py`) for each symbol **and each provider**.
4. For each generated chronology record:
   - **Chronology hash** – deterministic SHA‑256 of the entire chronology directory.
   - **Identity translation count** – number of symbol‑to‑symbol translations performed by the ingestion pipeline (should be `0`).
   - **Manifest integrity** – `manifest.json` contains a single, ordered list of files with correct `size`, `sha256`, and `timestamp` fields.
   - **Timestamp units** – verify they match the provider‑declared unit and are strictly monotonic.
   - **Replay certification** – execute `scripts/ci_fast.sh` (or the verifier CLI) against the chronology; it must exit with status 0.
   - **Catalog generation** – ensure `catalog.json` is reproducible on a second run.
5. Store the recorded hashes and translation counts in `historical_hashes.json` under `fixtures/strategy_identity/`.

### 2. Live Cohort
1. Start the live ingestion daemon for the three symbols on each provider (e.g. `scripts/live_ingest.py`).
2. Run for a **15‑30 min** window while the observation window is active.
3. Capture for each provider:
   - **Chronology hash** for the live‑generated chronology.
   - **Identity translation count** (again should be `0`).
   - **Monotonic timestamps** and absence of duplicate folds.
   - **Manifest stability** – check incremental growth.
4. After the window, stop the daemon and run replay certification as in the historical cohort.
5. Store live hashes and translation counts in `live_hashes.json`.

### 3. Historical → Live Boundary Review
1. Compare the provider sources used in the historical window (Yahoo) with the live source (Binance).
2. Generate a **continuity matrix**:
   ```
   Provider | Namespace | Replay Compatibility | Normalization Required | Translation Count
   -------------------------------------------------------------------------------------------
   Yahoo    | BTCUSD    | PASS                  | NONE                   | 0
   Binance  | BTCUSD    | PASS                  | NONE                   | 0
   ```
3. Record any required normalization steps (e.g., price scaling, ticker suffix removal) as explicit evidence.

---

## Failure Classification

When a soak run fails, classify the root cause using the table below. This aids post‑soak analysis and governance reporting.

| Failure                        | Classification            |
|-------------------------------|---------------------------|
| Symbol unavailable on provider | Provider limitation        |
| Timestamp‑unit mismatch       | Chronology incompatibility |
| Symbol normalization required | Identity drift             |
| Replay mismatch               | Certification failure      |
| Catalog divergence            | Authority drift            |
| Unexpected exception in script| Implementation issue       |
| Identity translation detected | Identity drift             |

---

## Success Criteria
- **All historical chronologies** produce deterministic hashes, have `identity_translation_count = 0`, and pass replay certification.
- **Live ingestion** yields deterministic hashes, `identity_translation_count = 0`, monotonic timestamps, and also passes replay certification.
- **Cross‑provider continuity matrix** reports **PASS** with **NONE** normalization and **0** translation count for every symbol.
- **Provider identity pass** – the continuity matrix reports `provider_identity_pass = true` for each provider/symbol.
- **Replay determinism pass** – replay certification succeeds with deterministic replay hash matching the chronology hash.
- **No governance‑tool warnings** (lint, verifier) are triggered during any phase.
- **Failure classification** remains empty (i.e., no failures recorded).
- **All historical chronologies** produce deterministic hashes, have `identity_translation_count = 0`, and pass replay certification.
- **Live ingestion** yields deterministic hashes, `identity_translation_count = 0`, monotonic timestamps, and also passes replay certification.
- **Cross‑provider continuity matrix** reports **PASS** with **NONE** normalization and **0** translation count for every symbol.
- **No governance‑tool warnings** (lint, verifier) are triggered during any phase.
- **Failure classification** remains empty (i.e., no failures recorded).

If any criterion fails, the soak is considered **inconclusive** and the recorded failure class drives the next investigative step.

---

## Evidence Collection & Reporting
- All artifacts (chronologies, manifests, catalogs, verification logs, hash JSON files, translation‑count JSON files) are stored under `fixtures/strategy_identity/` following the existing layout.
- A **soak report** (`docs/governance/soak_report_<date>.md`) is automatically generated summarizing:
  - Run parameters (window, symbols, providers)
  - Chronology hashes (historical vs. live)
  - Identity translation counts
  - Pass/fail outcomes per layer
  - Continuity matrix
  - Failure classifications, if any, and recommended actions
- The report is linked from the governance index for future auditability.

---

## Instrumentation & Automation
- Add a wrapper script `scripts/run_soak.sh` that:
  1. Executes the historical cohort and captures logs, hashes, and translation counts.
  2. Starts the live daemon for the configured interval.
  3. Performs the continuity review, writes the failure classification table, and verifies translation counts.
  4. Generates the final markdown report.
- The script can be invoked manually or scheduled via CI when the observation window is active.

---

## Timeline (suggested)
| Phase | Duration | Owner |
|------|----------|-------|
| Historical Cohort Execution | 1 day (batch job) | Data‑engineer |
| Live Cohort Execution | 30 min (manual trigger) | Ops / CI runner |
| Continuity Review & Reporting | 2 h | Governance lead |
| Decision & Follow‑up | 1 day | Team lead |

---

**Note:** This design is purely *documentation* and does **not** modify any authority surface. It adds a reproducible evidence‑gathering workflow that aligns with the existing governance posture.

---

*Links to related artifacts:* 
- [Governance ledger V‑011](file:///Users/nikhil/ChronoSentiment_MEGA_FINAL/docs/governance/governance_ledger_v011.md)
- [Authority map](file:///Users/nikhil/ChronoSentiment_MEGA_FINAL/AUTHORITY_MAP.md)
- [Governance index](file:///Users/nikhil/ChronoSentiment_MEGA_FINAL/docs/governance/governance_index.md)
