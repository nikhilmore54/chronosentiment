# Divergence Isolation Walkthrough

This 15-minute exercise demonstrates the core utility of ChronoSentiment: locating exactly where deterministic equivalence breaks.

## Step 1: Baseline Generation
Run the deterministic replay on a canonical substrate.
```bash
./chrono replay osc_50_1.0 rolling_50 --substrate-file core/chronology/historical/semi_vacuum_5m.jsonl
```
**Expected Output:**
```text
[OK] Replay engine executed successfully.
```

## Step 2: Inject a Defect
We will force a microscopic float accumulation drift (`+0.000001`) into the core engine to simulate a cross-language type coercion bug.
```bash
CHRONO_INJECT_FLOAT_COERCION=1 ./chrono replay osc_50_1.0 rolling_50 --substrate-file core/chronology/historical/semi_vacuum_5m.jsonl
```
**Expected Output:**
```text
[WARN] Float Coercion Drift Defect Active!
[OK] Replay engine executed successfully.
```

## Step 3: Observe Certification Failure
Attempt to certify the defective replay against the canonical manifest.
```bash
./chrono certify artifacts/semi_vacuum_5m/osc_50_1.0/rolling_50
```
**Expected Output:**
```text
[FAIL] ATTESTATION_FAILED
[FAIL] GEOMETRY_MISMATCH
```
**Failure Semantics:** The manifest verifier correctly rejects the artifact because the state footprint no longer matches the canonical baseline.

## Step 4: Mechanical Divergence Isolation
We know it failed. Now we find *where*. Run the inspect tool to mechanically isolate the drift.
```bash
./chrono inspect core/chronology/historical/semi_vacuum_5m.jsonl artifacts/semi_vacuum_5m/osc_50_1.0/rolling_50
```
**Expected Output:**
```text
[FAIL] Replay divergence detected.

First divergence:
- Event Index: 10
- Expected Price: 218.8500061035156
- Actual Price:   218.8500071035156

[OK] Divergence isolation complete.
```

## Step 5: Intentional Determinism Break (Artifact Corruption)
What happens if someone tampers with the artifact itself, rather than the engine?
1. Open the output manifest `artifacts/semi_vacuum_5m/osc_50_1.0/rolling_50/metadata.json`.
2. Find `"total_events"` and change its value by 1.
3. Run the certification again:
```bash
./chrono certify artifacts/semi_vacuum_5m/osc_50_1.0/rolling_50
```
**Expected Output:**
```text
[FAIL] ATTESTATION_FAILED
[FAIL] EVENT_COUNT_MISMATCH
```
**Failure Semantics:** The verifier structurally rejects tampered, corrupted, or incomplete artifacts immediately, refusing to certify invalid environments.
