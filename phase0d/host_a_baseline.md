# Phase 0D — Host A Baseline Package

Captured: 2026-05-30

---

## Git Commit

```
1eebebef8cbbb53aacdd22237945195b41260de0
```

---

## Toolchain

```
rustc 1.91.1 (ed61e7d7e 2025-11-07) (Homebrew)
cargo 1.91.1 (Homebrew)
```

Note: Rust installed via Homebrew (no rustup). Toolchain is `stable` channel pinned to 1.91.1.

---

## OS / Hardware

```
uname -a:
Darwin Nikhils-MacBook-Pro.local 25.4.0 Darwin Kernel Version 25.4.0: Thu Mar 19 19:30:44 PDT 2026; root:xnu-12377.101.15~1/RELEASE_ARM64_T6000 arm64

uname -m:
arm64

ProductName:    macOS
ProductVersion: 26.4.1
BuildVersion:   25E253
```

Architecture: **ARM64 (Apple Silicon)**

---

## Cargo.lock Hash

```
sha256: 1343212b7be6cae535b0ea55882e8fdffdfaf3dcfde23af97843e429a15cfba8
```

---

## Certified Chronology Files

| File | sha256 |
|------|--------|
| `chronology/historical/spy_capture/spy_1779283800000.jsonl` | `d8566522996a1192b0580fadd2e5b6dca3d9a3eed21ab14340af9543d98dc415` |
| `chronology/historical/dax_capture/gdaxi_1779346800000.jsonl` | `de8d68019ba2be5106bb10bde10c81187b748f7eb8fa7b470a9d28fc4441a6c8` |

Both match expected values. ✓

---

## Certified Replay Hashes (Host A)

| Substrate | Topology | Cognition | Expected Hash |
|-----------|----------|-----------|---------------|
| SPY | baseline | rolling_50 | `871391e54c19226888a232f1b523306eddf378fb1d9e018a44a225a102d894b7` |
| GDAXI | baseline | rolling_50 | `e360dc58be4454a57b0b1fb21caf7515b59aa62bb929073412ee4e9d87a36bb0` |

Confirmed from `artifacts/SPY/baseline/rolling_50/replay_hash.txt` and `artifacts/GDAXI/baseline/rolling_50/replay_hash.txt`.

---

## Phase 0D Certification Targets

These are the hashes Host B (Linux x86_64 VM) must reproduce exactly to close 0D-A:

```
SPY:   871391e54c19226888a232f1b523306eddf378fb1d9e018a44a225a102d894b7
GDAXI: e360dc58be4454a57b0b1fb21caf7515b59aa62bb929073412ee4e9d87a36bb0
```

---

## Host B Setup Checklist (0D-A: Linux x86_64 VM)

### VM Spec
- Hypervisor: UTM (emulate, not virtualize)
- Guest OS: Ubuntu Server 24.04 LTS AMD64
- CPU: 4 cores
- RAM: 8 GB
- Disk: 40 GB

### Build Dependencies (inside VM)
```bash
sudo apt update
sudo apt install -y build-essential git curl pkg-config python3 python3-pip clang
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
rustup default 1.91.1
```

### Clone at Certified Commit
```bash
git clone <repo-url>
cd ChronoSentiment_MEGA_FINAL
git checkout 1eebebef8cbbb53aacdd22237945195b41260de0
```

### Transfer Chronology Files
Copy only:
- `chronology/historical/spy_capture/spy_1779283800000.jsonl`
- `chronology/historical/dax_capture/gdaxi_1779346800000.jsonl`

### Verify Chronology Hashes (must match before proceeding)
```bash
sha256sum chronology/historical/spy_capture/spy_1779283800000.jsonl
# expected: d8566522996a1192b0580fadd2e5b6dca3d9a3eed21ab14340af9543d98dc415

sha256sum chronology/historical/dax_capture/gdaxi_1779346800000.jsonl
# expected: de8d68019ba2be5106bb10bde10c81187b748f7eb8fa7b470a9d28fc4441a6c8
```

### Capture Host B Environment
```bash
git rev-parse HEAD
sha256sum Cargo.lock
rustc --version
cargo --version
rustup show active-toolchain
uname -a
uname -m
```

### Build
```bash
cargo build --release --manifest-path financial/strategies/Cargo.toml
find . -name financial_replay -type f
sha256sum <binary-path>
```

### Run SPY Replay
```bash
cargo run --release \
  --manifest-path financial/strategies/Cargo.toml \
  --bin financial_replay -- \
  --substrate SPY \
  --substrate-file chronology/historical/spy_capture/spy_1779283800000.jsonl \
  --topology baseline \
  --cognition rolling_50

cat artifacts/SPY/baseline/rolling_50/replay_hash.txt
# must equal: 871391e54c19226888a232f1b523306eddf378fb1d9e018a44a225a102d894b7
```

### Run GDAXI Replay
```bash
cargo run --release \
  --manifest-path financial/strategies/Cargo.toml \
  --bin financial_replay -- \
  --substrate GDAXI \
  --substrate-file chronology/historical/dax_capture/gdaxi_1779346800000.jsonl \
  --topology baseline \
  --cognition rolling_50

cat artifacts/GDAXI/baseline/rolling_50/replay_hash.txt
# must equal: e360dc58be4454a57b0b1fb21caf7515b59aa62bb929073412ee4e9d87a36bb0
```

---

## Closure Criterion for 0D-A

Both hashes match → **0D-A CLOSED**.

Record Host B environment output alongside this file as `phase0d/host_b_0da_result.md`.