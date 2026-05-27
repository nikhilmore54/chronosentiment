# ChronoSentiment Operational Reference Manual

This manual contains the deterministic, stateless commands required to operate the ChronoSentiment platform. It does not contain architectural context, semantic philosophy, or deployment orchestration instructions.

## The Operator Facade

All operational tasks must be routed through the deterministic `./chrono` facade at the repository root.

```bash
./chrono help
```

## 1. Environment Bootstrap

Validates that the local environment (tools, toolchains, fixtures, corpus) is operationally ready. 

```bash
./chrono bootstrap
```

**Expected Outcome:** `[READY]`

## 2. Release Consistency Verification

Verifies that the current repository state is clean, reproducible, and ready for release packaging by running a double-build determinism check and verifying all fixtures.

```bash
./chrono release-verify
```

**Expected Outcome:** `[PASS]`

*Note: You can skip the double-build step to speed up local verification using `./chrono release-verify --skip-double-build`.*

## 3. Replay Smoke Verification

Validates the replay semantics against frozen byte and parser fixtures. This is a fast, post-deploy smoke gate.

```bash
./chrono smoke
```

**Expected Outcome:** `[PASS]`

## 4. Release Manifest Generation

Generates a formal JSON attestation of the repository state (git commit, hashes, determinism check status). This is run by CI before packaging.

```bash
./chrono manifest
```

**Expected Outcome:** `[PASS]`

## 5. Release Packaging

Creates a portable, self-contained `.tar.gz` release artifact. Automatically invokes consistency verification before packaging.

```bash
./chrono package
```

**Expected Outcome:** `[PASS]`

## 6. Continuous Integration (CI) Verification

The CI pipeline is fully automated and bounded. Pushing to `main` triggers `.github/workflows/deterministic-release-verification.yml`, which runs all steps above and validates the exact output artifact using the identical operator facade commands.
