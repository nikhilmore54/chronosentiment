# ChronoSentiment — Minimal Execution Path

**What runs in a live session. Nothing else.**

Aligned with `.cursor/rules/chronosentiment-core.mdc`: deterministic event-driven
ingest, price-time priority via frozen substrate replay, no randomness in the
live path.

---

## Rust authority (`cs-ingest`)

Causal ingest, repair, governor, and stage-1 verification live in
`financial/ese` (`cs-ingest`):

| Command | Role |
|---|---|
| `cs-ingest cohort` | Frozen-substrate replay (replaces `run_nse_cohort.py`) |
| `cs-ingest replay-step` | Low-level barrier → observatory → archive |
| `cs-ingest timeline` | Timeline fingerprint / frozen load check |
| `cs-ingest repair …` | Timestamp-locked gap recovery |
| `cs-ingest governor` | Telemetry → `governor_state.json` |
| `cs-ingest verify` | Archive integrity + baseline checks |
| `cs-ingest verify-chronology-fixtures` | V-006 chronology byte fixtures |

```bash
cargo build -p cs-ingest --release

./target/release/cs-ingest timeline --batch-id 3 --cohort cohorts/batch_003.txt

./target/release/cs-ingest cohort \
  --batch-id 3 --run-label replay_equiv --max-intervals 50 --fresh

./target/release/cs-ingest repair --archive-root state_archive --batch-id 3 detect

./target/release/cs-ingest governor --archive-dir state_archive/batches/batch_003/runs/replay_equiv --once

./target/release/cs-ingest verify --batch-id 3 --run-label replay_equiv
```

---

## Remaining Python (provider / orchestration only)

| Script | Role | Why still Python |
|---|---|---|
| [`freeze_cohort_candles.py`](../scripts/freeze_cohort_candles.py) | Create frozen substrate | yfinance / pandas freeze |
| [`candle_substrate.py`](../scripts/candle_substrate.py) | Freeze + incremental download helpers | yfinance / Binance / NSE |
| [`run_live_session.py`](../scripts/run_live_session.py) | Provider visibility wait + incremental merge + Rust resume | provider probes |
| [`certify_replay_chain.py`](../scripts/certify_replay_chain.py) | Multi-step cert orchestration | post-soak needs freeze helpers |
| [`compare_replay_equivalence.py`](../scripts/compare_replay_equivalence.py) | Live vs replay parity | archive analysis over live window |
| [`cs_bridge.py`](../scripts/cs_bridge.py) | Thin shell to `cs-ingest` | Python callers only |
| [`download_*.py`](../scripts/), [`fetch_candles.py`](../scripts/fetch_candles.py), [`yahoo_fetcher.py`](../scripts/yahoo_fetcher.py), [`build_nse_universe.py`](../scripts/build_nse_universe.py) | Data acquisition | market-data providers |
| [`real_data_streamer.py`](../scripts/real_data_streamer.py) | Pipe bars into `live_engine` | stdout bridge for shell harnesses |
| [`semantic_lint.py`](../scripts/semantic_lint.py) + lane2/3/4 scripts | CI policy tooling | GitHub Actions Python jobs |
| [`verify_strategy_identity_fixtures.py`](../scripts/verify_strategy_identity_fixtures.py) | V-001 strategy identity | differential multi-parser report |
| [`app.py`](../app.py) | Streamlit ops dashboard | UI |

Operational support still in Python (not on the hot path): `symbol_health.py`,
`storage_governor.py`, `cleanup_state_archive.py`, `schedule_live_run.py`,
`probe_provider_propagation.py`.

---

## The Live Session Sequence

```
1. freeze_cohort_candles.py          — create frozen substrate (once per cohort)
2. cs-ingest cohort                  — replay frozen substrate through observatory
3. cs-ingest governor                — start governor (reads archive, writes multiplier)
4. run_live_session.py               — warm buffers + incremental live barriers
5. cs-ingest verify                  — confirm archive integrity after session
```

Canonical health entry: [`validate_system.sh`](../scripts/validate_system.sh)
(shell + Rust only for core checks).

---

## What Is NOT in the Live Path

- `scripts/research/` — no execution authority
- Ecology atlases, policy competition, mock-streamer paths
- Historical one-off refactor scripts at repo root

---

## Anti-Drift Rules (Active Constraints)

1. No new script enters `scripts/` without a defined consumer in the live session sequence.
2. No new telemetry field without a defined consumer in the event filter or governor.
3. No research artifact gets execution authority without replay falsification.
4. No duplicate validation paths — `validate_system.sh` is the single entry point.
5. Every new component must answer: what execution decision changes because this exists?
