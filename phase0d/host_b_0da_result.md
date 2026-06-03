# Phase 0D — Host B (0D-A) Result

Date: 2026-05-31

## Verdict: PASS ✓

Both SPY and GDAXI replay hashes match Host A baseline exactly.

```
macOS ARM64 (Apple Silicon T6000)
    ↓
Linux x86_64 (UTM emulated VM, Ubuntu 24.04 LTS)

Same commit · Same chronology · Same replay identity
```

**Phase 0D-A is CLOSED.**

---

## Host B Environment

| Item | Value |
|------|-------|
| Git commit | `1eebebef8cbbb53aacdd22237945195b41260de0` |
| Build toolchain | `rustc 1.91.1 (ed61e7d7e 2025-11-07)` / `cargo 1.91.1 (ea2d97820 2025-10-10)` |
| Toolchain override | `RUSTUP_TOOLCHAIN=1.91.1-x86_64-unknown-linux-gnu` (overrides repo `rust-toolchain.toml` which pins 1.77.2) |
| Architecture | `x86_64` |
| OS | Ubuntu 24.04 LTS (minimized) |
| Kernel | `Linux chronosentiment 7.0.0-14-generic #14-Ubuntu SMP PREEMPT_DYNAMIC Mon Apr 13 11:09:53 UTC 2026 x86_64` |
| Hypervisor | UTM (emulated x86_64 on Apple Silicon ARM64) |
| `Cargo.lock` sha256 | `1343212b7be6cae535b0ea55882e8fdffdfaf3dcfde23af97843e429a15cfba8` |
| Binary path | `target/release/financial_replay` |
| Binary sha256 | `0dcb681e067525ae7b72db018d6faeb6e4cf4f69ba1f092133d49975550d0b63` |
| Build time | 47m 16s |

Note: `rustc --version` in the default shell reports 1.77.2 (pinned by `rust-toolchain.toml` in the repo root). The actual build used 1.91.1 via `RUSTUP_TOOLCHAIN` env override, which is required for `edition = "2024"` in `financial/core/Cargo.toml`.

---

## Chronology File Verification

| File | sha256 | Status |
|------|--------|--------|
| `chronology/historical/spy_capture/spy_1779283800000.jsonl` | `d8566522996a1192b0580fadd2e5b6dca3d9a3eed21ab14340af9543d98dc415` | ✓ matches expected |
| `chronology/historical/dax_capture/gdaxi_1779346800000.jsonl` | `de8d68019ba2be5106bb10bde10c81187b748f7eb8fa7b470a9d28fc4441a6c8` | ✓ matches expected |

---

## Replay Results

### SPY

```
Substrate:  SPY
Topology:   baseline
Cognition:  rolling_50
Ticks:      2730
Output:     artifacts/SPY/baseline/rolling_50/

Actual hash:   871391e54c19226888a232f1b523306eddf378fb1d9e018a44a225a102d894b7
Expected hash: 871391e54c19226888a232f1b523306eddf378fb1d9e018a44a225a102d894b7

Result: PASS ✓
```

### GDAXI

```
Substrate:  GDAXI
Topology:   baseline
Cognition:  rolling_50
Ticks:      3562
Output:     artifacts/GDAXI/baseline/rolling_50/

Actual hash:   e360dc58be4454a57b0b1fb21caf7515b59aa62bb929073412ee4e9d87a36bb0
Expected hash: e360dc58be4454a57b0b1fb21caf7515b59aa62bb929073412ee4e9d87a36bb0

Result: PASS ✓
```

---

## Cross-Environment Comparison

| Item | Host A (macOS ARM64) | Host B (Linux x86_64) | Match |
|------|---------------------|----------------------|-------|
| Git commit | `1eebebef` | `1eebebef` | ✓ |
| Rust version | 1.91.1 (Homebrew) | 1.91.1 (rustup) | ✓ |
| `Cargo.lock` sha256 | `1343212b...` | `1343212b...` | ✓ |
| SPY chronology sha256 | `d8566522...` | `d8566522...` | ✓ |
| GDAXI chronology sha256 | `de8d6801...` | `de8d6801...` | ✓ |
| SPY replay hash | `871391e5...` | `871391e5...` | ✓ |
| GDAXI replay hash | `e360dc58...` | `e360dc58...` | ✓ |
| Binary sha256 | (macOS, not comparable) | `0dcb681e...` | N/A |

---

## Certification Statement

ChronoSentiment `financial_replay` at commit `1eebebef8cbbb53aacdd22237945195b41260de0` produces **bit-identical replay hashes** across:

- **macOS 26.4.1 / Darwin 25.4.0 / ARM64 (Apple Silicon T6000)**
- **Ubuntu 24.04 LTS / Linux 7.0.0 / x86_64 (UTM emulated)**

This demonstrates replay reproducibility across:
- OS boundary (macOS → Linux)
- Architecture boundary (ARM64 → x86_64)
- Toolchain boundary (Homebrew Rust → rustup Rust)

**Phase 0D-A: CLOSED**

---

## Next Step: Phase 0D-B (Raspberry Pi 5)

Run the same package on Raspberry Pi 5 (ARM64 Linux) to extend the certification chain:

```
macOS ARM64
    ↓ [0D-A CLOSED]
Linux x86_64 VM
    ↓ [0D-B pending]
Raspberry Pi 5 (ARM64 Linux)
```

Expected hashes remain the same:
```
SPY:   871391e54c19226888a232f1b523306eddf378fb1d9e018a44a225a102d894b7
GDAXI: e360dc58be4454a57b0b1fb21caf7515b59aa62bb929073412ee4e9d87a36bb0