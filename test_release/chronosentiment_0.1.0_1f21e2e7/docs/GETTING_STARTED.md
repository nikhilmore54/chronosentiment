# Getting Started

This guide will get you running the deterministic replay engine in under 10 minutes.

## 1. Prerequisites
- Python 3.10+
- Rust (Cargo)

## 2. Install
```bash
./chrono install
```
**Expected Output:**
```text
Building trace_replay engine in release mode...
[OK] trace_replay engine compiled successfully.
```
**Failure Semantics:** If Cargo is missing, the installation will abort. If the build fails, it will print standard cargo error output and exit.

## 3. Verify Deterministic Replay (The Observer)
We will run a canonical substrate through the engine and passively certify the resulting trace artifact.

Tier 1 scope and primary scenario (`2026_multi_stage_cascade_transition_1m`): see [`docs/governance/DEMO_SCOPE.md`](docs/governance/DEMO_SCOPE.md).

```bash
./chrono demo
```

**Expected Output:**
```text
[OK] Replay engine executed successfully.
[OK] Manifest Certification Passed.
[OK] Deterministic boundaries preserved.
```
**Failure Semantics:**
If your environment violates structural determinism (e.g., non-IEEE 754 floats, broken JSON serialization), certification will immediately output `[FAIL] ATTESTATION_FAILED`.

## 4. Next Steps
Move on to the [Divergence Walkthrough](DIVERGENCE_WALKTHROUGH.md) to intentionally break determinism and isolate the failure mechanically.
