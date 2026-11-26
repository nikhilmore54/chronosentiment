# Observational Soak Results

**Label:** `Post-Governance Observational Replay Soak`

---

## Provider Identity Capture

| Field            | Value (fill after run) |
|------------------|------------------------|
| Provider         |                        |
| Symbol namespace |                        |
| Timestamp unit   |                        |
| Cohort           |                        |
| Quote semantics  |                        |

*Record the exact provider (e.g., Coinbase, Kraken), the symbol namespace used, the timestamp unit (ms vs s), the cohort identifier, and any quote‑currency semantics.*

---

## Timestamp‑Unit Continuity Check

Before starting the soak, run the helper script `scripts/check_timestamp_units.py` on a sample of the fetched data to ensure the provider emits **milliseconds** timestamps, matching the existing lineage.

```bash
python3 scripts/check_timestamp_units.py <path-to-sample-jsonl>
```

The script will:
1. Parse the first few entries of the JSON‑Lines file.
2. Detect the numeric field named `timestamp` (or similar).
3. Verify that the values are greater than `10^12` (typical ms epoch) and not in the `10^9` range (seconds).
4. Exit with status 0 on success, or print a warning and exit with status 1 if a seconds‑based timestamp is detected.

If the script reports a mismatch, abort the soak until the provider configuration is adjusted.

---

## Observation Log (to be filled post‑run)

- **Lint warning count:** 
- **Glossary change attempts:** 
- **PR discussion metrics:** 
- **Replay test result:** 
- **Any anomalies:** 

---

*All entries must be completed before closing the observation window.*
