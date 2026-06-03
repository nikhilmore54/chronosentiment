# Phase 0D Certification Statement

## Identity

| Field | Value |
|-------|-------|
| Commit | `1eebebef8cbbb53aacdd22237945195b41260de0` |
| Toolchain | Rust `1.91.1` |
| Date certified | 2026-01-06 |

## Validated Hosts

| Host | OS | Architecture |
|------|----|--------------|
| Host A | macOS 26.4.1 | ARM64 |
| 0D-A | Linux Ubuntu 24.04 | x86_64 |
| 0D-B | Linux (Raspberry Pi 5) | ARM64 (aarch64) |

## Validated Chronologies

| Ticker | Substrate file |
|--------|---------------|
| SPY | `chronology/historical/spy_capture/spy_1779283800000.jsonl` |
| GDAXI | `chronology/historical/dax_capture/gdaxi_1779346800000.jsonl` |

## Observed Replay Identity Hashes

| Ticker | Replay identity hash |
|--------|---------------------|
| SPY | `871391e54c19226888a232f1b523306eddf378fb1d9e018a44a225a102d894b7` |
| GDAXI | `e360dc58be4454a57b0b1fb21caf7515b59aa62bb929073412ee4e9d87a36bb0` |

All three hosts produced identical values for both tickers.

## Scope of Claim

This certification establishes:

> For the validated chronologies, execution of `financial_replay` produced **identical replay identity hashes** across all tested hosts and architectures.

This is a claim about **replay identity reproducibility**, not about binary identity or trace artifact identity. Specifically:

- **Proven:** `chronology → replay_hash` is reproducible across macOS ARM64, Linux x86_64, and Linux ARM64 when using identical chronology inputs, source revision, and Rust toolchain version.
- **Not claimed:** bit-identical compiled binaries across architectures.
- **Not claimed:** bit-identical `trace_v1.json` or other intermediate artifacts across hosts (these were not compared in Phase 0D).

## Conclusion

Replay identity generation is reproducible across the tested operating systems and processor architectures when using identical chronology inputs, source revision (`1eebebef`), and Rust toolchain version (`1.91.1`).

This is the foundational technical property required for ChronoSentiment's certification chain. The major uncertainty — whether replay identity survives substrate changes across heterogeneous environments — is resolved affirmatively for the tested scope.

## Supporting Artifacts

| Artifact | Description |
|----------|-------------|
| [`phase0d/host_b_0da_result.md`](host_b_0da_result.md) | 0D-A Linux x86_64 VM full run record |
| [`phase0d/host_c_0db_result.md`](host_c_0db_result.md) | 0D-B Raspberry Pi 5 aarch64 full run record |

## Phase Status

| Phase | Status |
|-------|--------|
| 0A | ✅ Complete |
| 0B | ✅ Complete |
| 0C | ✅ Complete |
| 0D-A Replay Identity (Linux x86_64) | ✅ Closed |
| 0D-B Replay Identity (Linux aarch64) | ✅ Closed — replay hash verified, binary/trace artifact comparison out of scope for 0D |
| 0E Governance & Certification Policy | ⬜ Not started |