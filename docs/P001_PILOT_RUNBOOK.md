# UltraCrew Pilot Runbook

**Document**: S2-06  
**Programme**: P-001 — UltraCrew Adapter for Coralys Scheduling  
**Stream**: 2 — Production Hardening  
**Audience**: Pilot operator (airline ops team, IT administrator)  
**Version**: 1.0 — 2026-07-23  

---

## Overview

This runbook walks a pilot operator through the complete lifecycle of a first UltraCrew scheduling run: environment setup, configuration, execution, output verification, and sign-off. It uses the SunAir demonstration scenario as the canonical reference.

Estimated time to complete: **30–45 minutes** for a first run.

---

## Prerequisites

| Requirement | Minimum version | Check command |
|-------------|-----------------|---------------|
| Rust toolchain | 1.75 | `rustc --version` |
| Cargo | 1.75 | `cargo --version` |
| Git | 2.x | `git --version` |
| Disk space | 500 MB (build artefacts) | `df -h .` |
| RAM | 2 GB | — |

---

## Step 1 — Clone and build

```bash
# 1a. Clone the repository (skip if already cloned)
git clone <repository-url> coralys
cd coralys

# 1b. Build the ultracrew CLI in release mode
cargo build -p ultracrew --release

# 1c. Verify the binary exists
ls -lh target/release/ultracrew-cli
```

Expected output from step 1c:
```
-rwxr-xr-x  1 user  staff  ...  target/release/ultracrew-cli
```

If the build fails, check that all workspace dependencies are present:
```bash
cargo check -p ultracrew 2>&1 | head -20
```

---

## Step 2 — Verify the health check

Before running any schedule, confirm the adapter is healthy:

```bash
./target/release/ultracrew-cli --health
```

Expected output (JSON):
```json
{
  "status": "ok",
  "version": "0.1.0",
  "adapter": "ultracrew",
  "checks": {
    "config": "ok",
    "validator": "ok"
  }
}
```

**Sign-off gate**: `status` must be `"ok"` and both subsystem checks must be `"ok"` before proceeding.

If `status` is `"degraded"` or `"error"`, check the error message in the relevant subsystem field and consult the troubleshooting section at the end of this document.

---

## Step 3 — Prepare the input dataset

The SunAir demonstration dataset is included in the repository:

```
fixtures/demo/sunair_demo.json       # Schedule request (workers + shifts)
fixtures/demo/sunair_optimizer.toml  # Optimizer configuration
```

Inspect the dataset to confirm it is intact:

```bash
# Verify worker and shift counts
python3 -c "
import json
d = json.load(open('fixtures/demo/sunair_demo.json'))
print(f'Workers: {len(d[\"workers\"])}')
print(f'Shifts:  {len(d[\"shifts\"])}')
print(f'Seed:    {d[\"rng_seed\"]}')
print(f'Gens:    {d[\"generation_limit\"]}')
"
```

Expected output:
```
Workers: 20
Shifts:  42
Seed:    42
Gens:    500
```

For a custom dataset, replace `sunair_demo.json` with your own file following the same schema. See `docs/P001_MILESTONE.md` for the full schema reference.

---

## Step 4 — (Optional) Customise the optimizer configuration

The default configuration in `fixtures/demo/sunair_optimizer.toml` is:

```toml
[optimizer]
generation_limit = 500
population_size  = 50
rng_seed         = 42

[scenario]
planning_horizon_hours = 168.0
max_hours_per_worker   = 48.0
```

To override parameters without editing the file, create a local copy:

```bash
cp fixtures/demo/sunair_optimizer.toml my_config.toml
# Edit my_config.toml as needed
```

Supported parameters:

| Parameter | Default | Description |
|-----------|---------|-------------|
| `optimizer.generation_limit` | 200 | GA generations. Higher = better quality, slower. |
| `optimizer.population_size` | 50 | Candidate schedules per generation. |
| `optimizer.rng_seed` | (none) | Set for deterministic runs. Omit for random. |
| `scenario.planning_horizon_hours` | 168.0 | Planning window in hours (168 = 7 days). |
| `scenario.max_hours_per_worker` | 48.0 | Maximum hours per worker in the window. |

YAML format is also supported (`.yaml` or `.yml` extension).

---

## Step 5 — Run the first schedule

```bash
./target/release/ultracrew-cli \
  --input  fixtures/demo/sunair_demo.json \
  --output sunair_schedule_output.json \
  --profile fixtures/demo/sunair_optimizer.toml
```

The CLI prints a KPI summary to stderr during the run. A successful run looks like:

```
════════════════════════════════════════
  UltraCrew Schedule — KPI Summary
════════════════════════════════════════
  Coverage:        42/42 shifts (100.0%)
  Hard violations: 0
  Rest violations: 0
  Fitness score:   8649.6000
  Fairness penalty: 697.6
  Fatigue penalty:  652.8
  Workers used:    20/20
  Mean hours/worker: 16.8 h
  Min hours/worker:  8 h
  Max hours/worker:  24 h
  Runtime:         ~11 s
════════════════════════════════════════
```

The output file `sunair_schedule_output.json` contains the full solution.

---

## Step 6 — Validate the output

Run the built-in strict validator against the output to confirm it is well-formed:

```bash
python3 -c "
import json, sys

out = json.load(open('sunair_schedule_output.json'))
inp = json.load(open('fixtures/demo/sunair_demo.json'))

total_shifts   = len(inp['shifts'])
assigned       = len(out['assignments'])
coverage_pct   = assigned / total_shifts * 100

print(f'Coverage:        {assigned}/{total_shifts} ({coverage_pct:.1f}%)')
print(f'Hard violations: {out[\"hard_violations\"]}')
print(f'Rest violations: {out[\"rest_violations\"]}')
print(f'Fitness score:   {out[\"fitness\"]:.4f}')

# Sign-off criteria
ok = (
    coverage_pct == 100.0 and
    out['hard_violations'] == 0 and
    out['rest_violations'] == 0
)
print()
print('SIGN-OFF:', 'PASS' if ok else 'FAIL')
sys.exit(0 if ok else 1)
"
```

**Sign-off gate**: The script must print `SIGN-OFF: PASS`.

---

## Step 7 — Regression check (SunAir canonical baseline)

For the SunAir demo scenario with `rng_seed = 42` and `generation_limit = 500`, the canonical baseline is:

| KPI | Baseline |
|-----|----------|
| Coverage | 100.0% (42/42) |
| Hard violations | 0 |
| Rest violations | 0 |
| Fitness score | 8649.6 ± 1.0 |

If your output deviates from these values with the same seed and generation limit, the run is not deterministic. Check:

1. The `rng_seed` in your config matches `42`.
2. The `generation_limit` matches `500`.
3. The input dataset matches `fixtures/demo/sunair_demo.json` exactly.

---

## Step 8 — Log review

Logs are written to stderr. To capture them for review:

```bash
./target/release/ultracrew-cli \
  --input  fixtures/demo/sunair_demo.json \
  --output sunair_schedule_output.json \
  --profile fixtures/demo/sunair_optimizer.toml \
  2>ultracrew_run.log

# Review the log
cat ultracrew_run.log
```

Log level is controlled by the `ULTRACREW_LOG` environment variable:

```bash
ULTRACREW_LOG=debug ./target/release/ultracrew-cli --input ...
```

| Level | Use |
|-------|-----|
| `error` | Production monitoring |
| `warn` | Default for pilot |
| `info` | Normal operational output |
| `debug` | Per-generation detail |
| `trace` | Full genome trace (very verbose) |

---

## Step 9 — Sign-off checklist

Complete this checklist before declaring the pilot successful:

- [ ] Build succeeded (`cargo build -p ultracrew --release`)
- [ ] Health check returned `status: "ok"` with both subsystems green
- [ ] Input dataset validated (20 workers, 42 shifts, seed 42)
- [ ] Schedule run completed without errors
- [ ] Coverage = 100% (42/42 shifts assigned)
- [ ] Hard violations = 0
- [ ] Rest violations = 0
- [ ] Fitness score within ±1.0 of 8649.6 (deterministic run)
- [ ] Output JSON written and readable
- [ ] Log file reviewed — no unexpected errors

**Pilot sign-off**: Record the date, operator name, and output file hash:

```bash
echo "Date:     $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo "Operator: <your name>"
echo "Output:   $(shasum -a 256 sunair_schedule_output.json)"
```

---

## Troubleshooting

### Build fails: `error[E0433]: failed to resolve`

The workspace dependency graph is incomplete. Run from the repository root:
```bash
cargo build --workspace 2>&1 | head -30
```

### Health check returns `"degraded"`

Check the `checks` field for the failing subsystem. Common causes:

- `config: "config subsystem unavailable"` — the `toml` or `serde_yaml` crate is not linked. Rebuild with `cargo build -p ultracrew --release`.
- `validator: "validator subsystem unavailable"` — the strict validator module failed its self-test. File a bug with the full health response JSON.

### Coverage < 100%

The optimizer did not assign all shifts. Possible causes:

- `generation_limit` is too low (< 200). Increase to 500.
- Workers do not have the skills required by all shifts. Check the dataset.
- `max_hours_per_worker` is too low. Increase to at least `max_shift_duration`.

### Fitness score differs from baseline

If `rng_seed` is set and the score differs by more than ±1.0, the run is not deterministic. Verify:
- The exact same binary is used (no recompile between runs).
- The input JSON is byte-for-byte identical.
- No environment variable overrides the seed.

### Output file is empty or malformed

Check stderr for a serialisation error (`UC-EXP-001`). Ensure the output path is writable:
```bash
touch sunair_schedule_output.json && echo "writable"
```

---

## Reference

| Artifact | Path |
|----------|------|
| Input dataset | `fixtures/demo/sunair_demo.json` |
| Optimizer config (TOML) | `fixtures/demo/sunair_optimizer.toml` |
| Optimizer config (YAML) | `fixtures/demo/sunair_optimizer.yaml` |
| Canonical transcript | `fixtures/demo/sunair_demo_transcript.txt` |
| Canonical schedule output | `fixtures/demo/sunair_schedule.json` |
| Programme milestone doc | `docs/P001_MILESTONE.md` |
| Architecture governance | `docs/ARCHITECTURE_EVOLUTION.md` |

---

*End of Pilot Runbook — S2-06*