# ChronoSentiment — Minimal Execution Path

**What runs in a live session. Nothing else.**

---

## The 26 Operational Scripts

After cleanup, `scripts/` contains the following operational scripts. Everything else is in `scripts/research/`.

### Core Pipeline

| Script | Role |
|---|---|
| [`run_nse_cohort.py`](../scripts/run_nse_cohort.py) | NSEIngestionEngine — cohort orchestration, frozen-substrate replay |
| [`run_live_session.py`](../scripts/run_live_session.py) | Live synchronized observatory session (warm + incremental) |
| [`candle_substrate.py`](../scripts/candle_substrate.py) | Frozen cohort candle loading |
| [`archive_dedupe.py`](../scripts/archive_dedupe.py) | Deduplication index + gzip writer pool |
| [`freeze_cohort_candles.py`](../scripts/freeze_cohort_candles.py) | Create frozen substrate from downloaded data |
| [`observatory_daemon.py`](../scripts/observatory_daemon.py) | Observatory subprocess management |
| [`repair_chronology_gaps.py`](../scripts/repair_chronology_gaps.py) | Timestamp-locked gap recovery |
| [`governor_refresher.py`](../scripts/governor_refresher.py) | Minimal production governor (telemetry → multiplier) |

### Verification

| Script | Role |
|---|---|
| [`verify_cohort_baseline.py`](../scripts/verify_cohort_baseline.py) | Archive integrity + replay consistency |
| [`certify_replay_chain.py`](../scripts/certify_replay_chain.py) | Replay chain certification |
| [`compare_replay_equivalence.py`](../scripts/compare_replay_equivalence.py) | Cross-run replay parity |
| [`compare_ingest_parity.py`](../scripts/compare_ingest_parity.py) | Ingest parity between runs |
| [`validate_system.sh`](../scripts/validate_system.sh) | Canonical system health entry point |

### Data Acquisition

| Script | Role |
|---|---|
| [`download_nse_data.py`](../scripts/download_nse_data.py) | Download NSE 5m bars |
| [`download_30d_history.py`](../scripts/download_30d_history.py) | 30-day history download |
| [`download_oos_data.py`](../scripts/download_oos_data.py) | Out-of-sample data download |
| [`download_cross_asset.py`](../scripts/download_cross_asset.py) | Cross-asset data download |
| [`build_nse_universe.py`](../scripts/build_nse_universe.py) | NSE universe construction |
| [`fetch_candles.py`](../scripts/fetch_candles.py) | Candle fetch utility |
| [`real_data_streamer.py`](../scripts/real_data_streamer.py) | Live data streaming |
| [`probe_provider_propagation.py`](../scripts/probe_provider_propagation.py) | Provider timing study (chronology) |

### Operational Support

| Script | Role |
|---|---|
| [`symbol_health.py`](../scripts/symbol_health.py) | Symbol health registry |
| [`storage_governor.py`](../scripts/storage_governor.py) | Archive storage tiering |
| [`cleanup_state_archive.py`](../scripts/cleanup_state_archive.py) | Archive cleanup |
| [`schedule_live_run.py`](../scripts/schedule_live_run.py) | Live run scheduling |
| [`run_incremental_cohort.py`](../scripts/run_incremental_cohort.py) | Incremental cohort ingestion |

---

## The Live Session Sequence

A complete live session runs in this order:

```
1. freeze_cohort_candles.py     — create frozen substrate (once per cohort)
2. run_nse_cohort.py            — replay frozen substrate through observatory
3. governor_refresher.py        — start governor (reads archive, writes multiplier)
4. run_live_session.py          — warm buffers + incremental live barriers
5. verify_cohort_baseline.py    — confirm archive integrity after session
```

The Rust binary (`cs-ingest`) runs inside steps 2 and 4 via subprocess.

---

## The Rust Binary Commands

```bash
# Timeline fingerprint (verify frozen substrate loads)
./target/release/cs-ingest timeline --batch-id 3 --cohort cohorts/batch_003.txt

# End-to-end replay step
./target/release/cs-ingest replay-step \
  --batch-id 3 \
  --cohort cohorts/batch_003.txt \
  --archive state_archive/batches/batch_003/runs/replay_equiv \
  --pca-weights observatory/provider_clustering_pca_weights.json \
  --observatory ./target/release/examples/live_observatory

# Chronology gap repair
./target/release/cs-ingest repair \
  --archive-root state_archive \
  --batch-id 3 \
  detect
```

---

## What Is NOT in the Live Path

The following are in `scripts/research/` and do not run during a live session:

- All mock-streamer / TRAP_POLICY scripts (old live_engine path)
- All `physics_divergence.csv` readers (old schema)
- All hardcoded log-file readers (`replay_1m_gen11.log`, `replay_5m_oos1.log`)
- Ecology pipeline (`ecology_signature_atlas.py`, `ecology_transition_atlas.py`) — post-hoc analysis only
- Policy competition engine, collapse forecaster, topology transition model

---

## Governor State

The governor writes to `analysis/real_live/governor_state.json`:

```json
{
  "multiplier": 1.0,
  "gate_open": true,
  "reason": "NOMINAL (instability=0.02, corridor=0.08)",
  "instability_rate": 0.02,
  "corridor_rate": 0.08,
  "window_size": 50,
  "ts": 1779379319
}
```

The live session reads this file. The governor runs as a separate process alongside the live session.

---

## Anti-Drift Rules (Active Constraints)

1. No new script enters `scripts/` without a defined consumer in the live session sequence.
2. No new telemetry field without a defined consumer in the event filter or governor.
3. No research artifact gets execution authority without replay falsification.
4. No duplicate validation paths — `validate_system.sh` is the single entry point.
5. Every new component must answer: what execution decision changes because this exists?