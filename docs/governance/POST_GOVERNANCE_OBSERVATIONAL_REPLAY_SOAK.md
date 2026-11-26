# Post-Governance Observational Replay Soak

**Label:** `Post-Governance Observational Replay Soak`

## Scope & Intent
- Validate that **governance exposure** (semantic lint, PR template, CODEOWNERS, posture doc) co‑exists with live market data ingestion without compromising deterministic replay.
- Run a **controlled live fetch** of the canonical **USD‑pair** cohort:
  - `BTCUSD`
  - `ETHUSD`
  - `SOLUSD`
- Observe the system for **15–20 minutes** under the existing **warning‑only** CI configuration.

## Success Criterion
- **Chronology integrity** remains intact (monotonic timestamps, no duplicate folds, no unit regression).
- **Replay identity** passes before‑and‑after checks:
  ```bash
  cargo test replay --release -- --test-threads=1
  ```
- **Symbol authority stability**: no implicit aliasing (e.g., `BTCUSD` ↔ `BTCUSDT`) and no silent normalisation.
- No **hard failures** in CI; only warning messages appear.

## Operational Steps
1. **Create cohort file** `cohorts/batch_usd_observational.txt` with one symbol per line (USD format):
   ```text
   BTCUSD
   ETHUSD
   SOLUSD
   ```
    # Verify timestamps before running the soak (lightweight check)
    head -5 <sample_capture_file>.jsonl
    python3 scripts/check_timestamp_units.py <sample_capture_file>.jsonl

    # Ensure timestamps are 13‑digit milliseconds, monotonic, and provider‑explicit.

2. **Run the live session** (or the appropriate wrapper) for a single cycle batch:
   ```bash
   python3 scripts/run_live_session.py \
       --batch-file cohorts/batch_usd_observational.txt \
       --cycles 1 \
       --bar-sec 300 \
       --temporal-observatory \
       --run-label soak_observational
   ```
   - Adjust `--cycles` or `--duration` if you prefer a timed run (e.g., `sleep 900` after start).
3. **Monitor CI**:
   - Ensure the GitHub Actions workflow `semantic_lint.yml` runs and reports *warnings* only.
4. **After completion** run the replay test to verify identity:
   ```bash
   cargo test replay --release -- --test-threads=1
   ```
5. **Collect metrics**:
   - Lint warning count.
   - Glossary change attempts.
   - PR discussion length / reviewer comments.
   - Any CI retries or cancellations.
6. **Document observations** in `docs/governance/OBSERVATIONAL_SOAK_RESULTS.md`.

## What to Avoid During the Soak
- Adding new asset symbols or exchanges.
- Switching back to USDT quote pairs (`BTCUSDT`, `ETHUSDT`, `SOLUSDT`).
- Modifying chronology schema or introducing new authority surfaces.
- Hard‑fail CI enforcement.
  - Orchestration or runtime code changes.

**Provider Boundary Clause**

| Field                 | Value                                                   |
| --------------------- | ------------------------------------------------------- |
| Quote asset policy    | Explicit USD-pair operational cohort                    |
| Historical divergence | Prior chronology fixtures may contain USDT semantics    |
| Equivalence posture   | USD and USDT identities are NOT interchangeable         |
| Governance posture    | observational only — no normalization migration implied |

USD-pair identities are intentionally selected because replay‑compatible historical data exists across both Binance and Yahoo Finance providers, improving cross‑provider deterministic recoverability without implying USD/USDT equivalence.


## Follow‑Up
- After the 3‑7 day observation window, review the collected metrics.
- Decide whether to:
  - Keep the current warning‑only posture.
  - Tighten lint rules or move to hard‑fail enforcement.
  - Open a new tranche for additional governance hardening.

---
*This plan encodes the operational label, intent, admissibility boundary, and escalation posture, preventing future reinterpretation as a research or expansion activity.*
