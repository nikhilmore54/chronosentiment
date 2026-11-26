# Phase C Deterministic Replay Fixture

This directory contains the immutable inputs used for proving determinism in ChronoSentiment Phase C.

**CRITICAL RULE: DO NOT AUTO-GENERATE OR OVERWRITE THESE FILES IN TESTS.**
These files are committed directly to Git as a permanently version-controlled deterministic input. The tests use these inputs to prove that Replay(T) and Outcome measurement are reproducible.

`scripts/phase_c_gate.sh` was removed (CLN-016 / CS-P-CLEAN-001). It printed PASS without executing tests and is not evidence. These fixtures remain.

### Fixture Manifest
- **Fixture version**: 1.0
- **Evaluation timestamp T**: 2026-08-01T12:00:00Z
- **Knowledge Lake version**: 1.0
- **Universe/instrument**: 22222222-2222-2222-2222-222222222222
- **Historical cases included**: 1 (Historical Analog 1)
- **Future observation range**: 2026-08-01T12:00:00Z to 2026-08-02T12:00:00Z
- **Expected horizon(s)**: 24h
- **Fixture schema version**: 1
- **SHA-256 of each fixture file**: (To be calculated and verified on test run)

### Files
- `observations.json`: The fixed slice of observations up to evaluation time T.
- `knowledge_state.json`: The state of the knowledge lake (prior scenarios, historical cases) at time T to exercise the Historical Reasoning Engine.
- `future_observations.json`: The fixed slice of future observations post-T used to test Outcome determinism (C2).
