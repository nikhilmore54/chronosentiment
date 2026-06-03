# Phase 0D-B Certification Result — Raspberry Pi 5 (aarch64)

## Environment

| Field | Value |
|-------|-------|
| Host | Raspberry Pi 5 (LCUtest) |
| User | `ultraspeed@192.168.0.3` |
| Architecture | `aarch64` |
| OS | Linux (aarch64-unknown-linux-gnu) |
| Rust toolchain | `1.91.1-aarch64-unknown-linux-gnu` |
| Repo commit | `1eebebef8cbbb53aacdd22237945195b41260de0` |
| Build completed | 2026-01-06T07:16 (IST) — `Finished release profile in 2m 31s` |
| Binary size | 1.3M |
| Binary path | `/home/ultraspeed/ChronoSentiment_MEGA_FINAL/target/release/financial_replay` |

## Replay Results

### SPY Chronology

| Field | Value |
|-------|-------|
| Ticks processed | 2730 |
| Topology | `baseline` |
| Cognition | `rolling_50` |
| Output | `artifacts/SPY/baseline/rolling_50` |
| SHA-256 replay hash | `871391e54c19226888a232f1b523306eddf378fb1d9e018a44a225a102d894b7` |
| Status | ✅ PASS |

### GDAXI Chronology

| Field | Value |
|-------|-------|
| Ticks processed | 3562 |
| Topology | `baseline` |
| Cognition | `rolling_50` |
| Output | `artifacts/GDAXI/baseline/rolling_50` |
| SHA-256 replay hash | `e360dc58be4454a57b0b1fb21caf7515b59aa62bb929073412ee4e9d87a36bb0` |
| Status | ✅ PASS |

## Cross-Host Hash Comparison

| Ticker | Host A (macOS ARM64) | 0D-A (Linux x86_64 VM) | 0D-B (Raspberry Pi 5 aarch64) | Match |
|--------|---------------------|------------------------|-------------------------------|-------|
| SPY    | `871391e54c19226888a232f1b523306eddf378fb1d9e018a44a225a102d894b7` | `871391e54c19226888a232f1b523306eddf378fb1d9e018a44a225a102d894b7` | `871391e54c19226888a232f1b523306eddf378fb1d9e018a44a225a102d894b7` | ✅ IDENTICAL |
| GDAXI  | `e360dc58be4454a57b0b1fb21caf7515b59aa62bb929073412ee4e9d87a36bb0` | `e360dc58be4454a57b0b1fb21caf7515b59aa62bb929073412ee4e9d87a36bb0` | `e360dc58be4454a57b0b1fb21caf7515b59aa62bb929073412ee4e9d87a36bb0` | ✅ IDENTICAL |

## Conclusion

All three hosts — macOS ARM64, Linux x86_64 VM, and Raspberry Pi 5 aarch64 — produce **bit-identical replay hashes** for both SPY and GDAXI chronologies when built from the same pinned Rust toolchain (`1.91.1`) and repo commit (`1eebebef`).

The `financial_replay` binary is **fully deterministic and reproducible across architectures and operating systems**.

**Phase 0D-B: COMPLETE ✅**
**Phase 0D: FULLY CERTIFIED ✅**