# Operational Workflow Guide

The standard operating procedure for using ChronoSentiment Core consists of three mechanical steps.

## 1. Replay
Execute a chronological substrate through the deterministic engine.

```bash
./chrono replay <topology> <cognition> --substrate-file <file>
```
**Expected Output:** A `trace_v1.json` and `metadata.json` artifact generated in the `artifacts/` directory.
**Failure Semantics:** If the substrate is malformed JSONL, the engine will halt immediately and emit a parsing failure.

## 2. Certify
Passively attest that the generated artifact matches canonical boundaries.

```bash
./chrono certify <artifact_directory>
```
**Expected Output:** `[OK] Manifest Certification Passed.`
**Failure Semantics:** Any deviation in geometry, state logic, tick count, or duration invariants results in `[FAIL] ATTESTATION_FAILED`.

## 3. Inspect
If certification fails, mechanically locate the exact tick of divergence.

```bash
./chrono inspect <baseline_substrate> <suspect_artifact>
```
**Expected Output:** The exact zero-indexed tick where `Expected` != `Actual`.
**Failure Semantics:** If the files do not exist or the artifact is corrupted, inspection will safely abort and explicitly warn the user.
