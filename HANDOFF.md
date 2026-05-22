# ChronoSentiment — Session Transfer Summary

## Project identity

**ChronoSentiment** = deterministic propagation-ecology instrumentation platform (not a trading bot).

| Layer | Role |
|-------|------|
| **Rust (`core`, `cs-ingest`)** | Causal/replay authority — observatory, PCA, persist, dedupe |
| **Python (`scripts/`)** | Orchestration — freeze, live sessions, certification harnesses |
| **Ecology scripts** | Observability on archives (signatures, atlas) — semantics unchanged during ingest migration |

**Invariants:** Same frozen input → same archive output. No randomness in strategy/replay. Price-time / barrier chronology must not be weakened.

---

## Major accomplishments (certified)

### 1. `cs-ingest` Rust crate (canonical ingest)

**Path:** `cs-ingest/` (workspace member in root `Cargo.toml`)

| Module | Role |
|--------|------|
| `frozen_loader.rs` | Load frozen gzip JSONL + manifest |
| `timeline.rs` | Timeline union + SHA256 fingerprint (16 hex, Python parity) |
| `dedupe.rs` | `dedupe_index.json` load/rebuild/save |
| `telemetry.rs` | Parse `[TELEMETRY]`, PCA/classify (rounded history in `hist` for Python parity) |
| `replay.rs` | `replay-step` orchestration |
| `observatory.rs` | Subprocess to `live_observatory` |
| `archive.rs` | GzipWriterPool (fixed `canonicalize()` bug on nonexistent paths) |
| `repair.rs` | Phase 4 gap recovery (`T_provider == T_barrier`) |

**Build:**
```bash
cargo build -p cs-ingest --release
cargo build -p chronosentiment_core --example live_observatory --release
```

**CLI:** `timeline`, `dedupe-verify`, `replay-step` (`--fresh`, `--start-interval`, `--max-intervals`, `--resume`, `--rebuild-dedupe`), `repair` subcommands

### 2. Ingest parity — PASSED (promotion done)

| Batch | Result |
|-------|--------|
| **900** (10 symbols, 5 barriers) | 50/50 exact field parity after fixes |
| **003** (500 symbols, 50 barriers) | **18,618/18,618** exact cross-runtime parity (legacy Python vs cs-ingest) |

**Critical fixes during certification:**
1. **Stale archive false parity:** Rust showed `persisted 0 | dedupe_skip 50` while compare still saw ticks — fixed with `replay-step --fresh` + parity harness passes `--fresh` on Rust leg.
2. **Curvature/history drift:** Rust stored unrounded values in `hist`; Python stores rounded — fixed in `telemetry.rs`.
3. **Harness self-compare bug:** After promotion, first leg defaulted to cs-ingest → compared Rust vs Rust — fixed with explicit `--python-ingest` on legacy leg (see regressions below).

**Promotion:** Production frozen ingest goes through `run_nse_cohort.py` → `run_frozen_via_cs_ingest()` → `cs-ingest replay-step`.

### 3. Certification harnesses (still in repo)

| Script | Purpose |
|--------|---------|
| `scripts/compare_ingest_parity.py` | Legacy Python vs cs-ingest; `--run` with `--fresh` |
| `scripts/certify_replay_chain.py` | `ingest-parity`, `full-replay`, `equiv-vs-live`, `all`, `post-soak-cert` |
| `scripts/compare_replay_equivalence.py` | Live archive vs replay archive per-ts |
| `scripts/verify_cohort_baseline.py` | Stage-1 baseline checks |
| `scripts/freeze_cohort_candles.py` | Full cohort freeze |

**`post-soak-cert` flow:** Read live `metadata/live_session_steps.jsonl` → map live barriers to frozen timeline indices → `full-replay` on that window → `equiv-vs-live`. Requires re-freeze after soak.

---

## Other-IDE cleanup — current architecture

### `scripts/run_nse_cohort.py` (~169 lines)

- **Only** calls `cs-ingest replay-step` (no `NSEIngestionEngine`, no `--python-ingest`)
- Flags: `--fresh`, `--resume`, `--rebuild-dedupe`, `--start-interval`, `--max-intervals`, `--run-label`
- `--from-frozen` is DEPRECATED no-op

### `scripts/run_live_session.py` (~236 lines) — REWRITTEN

**Old (certified soak era):** Warm `ObservatoryDaemon` → live yfinance per barrier → Python persist → `live_session_steps.jsonl` + participation/lag metrics.

**New (kernel runner):**
1. **Phase A:** `active_temporal_observatory()` — Binance kline + NSE India quote + yfinance visibility; or `wait_for_barrier_target`
2. **Phase B:** `incremental_update_cohort()` in `candle_substrate.py` (1d yfinance merge into frozen substrate)
3. **Phase C:** `run_frozen_via_cs_ingest(..., resume=True, fresh=False, max_intervals=None)` — full canonical replay each cycle; dedupe skips already-persisted `(symbol, ts)`

Flags: `--temporal-observatory`, `--max-barrier-wait-sec`, `--cycles`, `--batch-id`, `--run-label`

### `scripts/candle_substrate.py`

- `freeze_cohort()` — full freeze
- **`incremental_update_cohort()`** — merge 1d fetch into existing per-symbol gzip, refresh manifest

### Still present but not used by new live runner

- `scripts/observatory_daemon.py` — warm Rust observatory subprocess

### Ecology pipeline

- `scripts/ecology_signature_atlas.py` confirmed on disk
- Some `ecology_*.py` may have been moved to `archival_cold_storage/` during cleanup (verify locally)

---

## Known regressions after cleanup (must fix for full cert loop)

| Issue | Impact | Fix |
|-------|--------|-----|
| **`compare_ingest_parity.py` calls `--python-ingest`** | `run_nse_cohort.py` no longer has that flag → `--run` fails | Restore `scripts/legacy_python_ingest.py` OR switch to golden-archive regression |
| **No `live_session_steps.jsonl` in new live runner** | `post-soak-cert` / `certify all` equiv path breaks | Append minimal JSONL per cycle (`ts`, `barrier_committed`, optional latency) |
| **Lost participation / lag / `sync_breakdown`** | Harder to diagnose Yahoo fragmentation (batch 902 crypto) | Thin metadata logging on kernel runner |
| **Full replay every live cycle** | CPU cost; dedupe should limit writes | Confirm `persisted N \| dedupe_skip M` in logs each cycle |

---

## Cohort / batch status

| Batch | Notes |
|-------|-------|
| **003** | NSE 500; ingest parity **certified** 18,618/18,618 |
| **900** | LSE 10; live `lse_replay`; use `post-soak-cert` on live window, not `cert_full [0:50)` |
| **901** | **DISCARD** — invalid duplicate timestamps |
| **902** | Crypto 24/7; Yahoo participation fragmentation; Binance anchors in temporal observatory |

---

## Key paths

```
cs-ingest/
core/examples/live_observatory.rs
scripts/run_live_session.py
scripts/run_nse_cohort.py
scripts/candle_substrate.py
scripts/compare_ingest_parity.py
scripts/certify_replay_chain.py
scripts/compare_replay_equivalence.py
scripts/freeze_cohort_candles.py
scripts/ecology_signature_atlas.py
scripts/observatory_daemon.py

cohorts/batch_003.txt, batch_900.txt, batch_902.txt
state_archive/candles/batch_NNN/
state_archive/batches/batch_NNN/runs/
state_archive/metadata/ingest_parity_batch_*.json
```

---

## Commands cheat sheet

```bash
source .venv/bin/activate
cargo build -p cs-ingest --release
cargo build -p chronosentiment_core --example live_observatory --release

# Smoke replay
python3 scripts/run_nse_cohort.py --batch-id 900 --fresh --run-label smoke --max-intervals 5

# Parity (broken until legacy path restored)
python3 scripts/compare_ingest_parity.py --batch-id 900 --max-intervals 5 --run

# Full cert replay
python3 scripts/certify_replay_chain.py full-replay --batch-id 3 \
  --run-label cert_full --max-intervals 50 --fresh

# Post-soak (needs live_session_steps.jsonl + freeze)
python3 scripts/certify_replay_chain.py post-soak-cert \
  --batch-id 900 --live-label lse_replay --replay-label replay_equiv --freeze

# Live crypto
python3 scripts/freeze_cohort_candles.py --batch-id 902 --max-workers 2
python3 scripts/run_live_session.py --batch-id 902 --run-label crypto_24h \
  --cycles 12 --temporal-observatory --max-barrier-wait-sec 720
```

---

## Lessons / pitfalls

| Symptom | Likely cause |
|---------|----------------|
| Parity PASS but `persisted 0` | Stale archive/dedupe — use `--fresh` |
| zsh paste errors | User pasted markdown into shell |
| Crypto stall `aligned bar < target` | Feed sync starvation, not replay bug |
| equiv FAIL vs `cert_full` | Wrong time window — use live barrier window |
| Low participation | Yahoo fragmentation — check lags, not replay corruption |

---

## Certification state (snapshot)

| Layer | Status |
|-------|--------|
| cs-ingest `replay-step` | Certified exact parity vs legacy Python |
| Promotion to canonical ingest | Done in `run_nse_cohort.py` |
| Dedupe + fingerprint | Verified |
| Cross-runtime parity harness | **Broken** (missing `--python-ingest`) |
| Post-soak live equiv | **Broken** for new live runner (no steps JSONL) |
| Live participation diagnostics | **Removed** in kernel runner |
| Ecology semantics | Not modified during migration |
| `ga.rs` | Large file (~14k lines) — separate GA workstream, not part of ingest migration |

---

## Recommended next steps (priority order)

1. **Restore cert hooks:** legacy parity entrypoint OR golden archive; `live_session_steps.jsonl` in `run_live_session.py`
2. **Re-run smoke:** `run_nse_cohort --fresh --max-intervals 5` on batch 900
3. **Optional:** participation/lag logging on live kernel
4. **Live 902:** freeze → temporal observatory live soak → freeze → `post-soak-cert`
5. **Do not:** auto-forward stale bars, synthetic timestamps, or treat low participation as replay corruption without feed diagnostics

---

## Audit verdict (other-IDE cleanup)

**Mostly helpful:** single Rust ingest path, incremental substrate, temporal observatory, `cs-ingest repair`.

**Fix before declaring cert-ready:** parity harness + live steps metadata (+ optional live diagnostics).

---

## Empirical Governor Baseline

> **Empirical governor baseline (May 2026):** `batch_003` ≈ persistent fragmentation (sync <60%, dispersion >2.0); `batch_910` ≈ 100% atomic sync across all observed sessions; NSE open/close/post-close extend reconciliation for illiquid cohorts only. Use 910 as control, 003 as stress cohort for chronology constraints — not as replay corruption signal.
