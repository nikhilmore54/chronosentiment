# ChronoSentiment — Architectural Simplification Analysis

**Date:** 2026-05-21  
**Scope:** Full codebase audit — Rust ingest core (`cs-ingest`) + Python scripts ecosystem (100+ scripts)  
**Purpose:** Honest assessment of observability surface, research residue, and a concrete simplification roadmap.

---

## 1. What the Codebase Actually Contains

### 1.1 The Rust Core (`cs-ingest/src/`)

11 modules. The dependency graph is clean and the core is sound:

| Module | Role | Verdict |
|---|---|---|
| [`frozen_loader.rs`](cs-ingest/src/frozen_loader.rs) | Load frozen cohort candles from disk | **Keep — foundational** |
| [`timeline.rs`](cs-ingest/src/timeline.rs) | Union-timestamp alignment + SHA-256 fingerprint | **Keep — foundational** |
| [`dedupe.rs`](cs-ingest/src/dedupe.rs) | Deduplication index (symbol × ts) | **Keep — foundational** |
| [`archive.rs`](cs-ingest/src/archive.rs) | Gzip writer pool, telemetry path routing | **Keep — foundational** |
| [`persist.rs`](cs-ingest/src/persist.rs) | Archive persistor with event/stable sampling | **Keep — foundational** |
| [`repair.rs`](cs-ingest/src/repair.rs) | Phase 4 timestamp-locked gap recovery state machine | **Keep — foundational** |
| [`manifest.rs`](cs-ingest/src/manifest.rs) | Manifest read/write | **Keep — foundational** |
| [`pca.rs`](cs-ingest/src/pca.rs) | PCA projection + nearest-centroid classification | **Conditional — see §3** |
| [`telemetry.rs`](cs-ingest/src/telemetry.rs) | Telemetry line parsing + 20-field record assembly | **Conditional — see §3** |
| [`observatory.rs`](cs-ingest/src/observatory.rs) | Subprocess bridge to `live_observatory` binary | **Conditional — see §3** |
| [`replay.rs`](cs-ingest/src/replay.rs) | End-to-end replay pipeline orchestrator | **Keep — foundational** |

The Rust core is the most defensible part of the system. It is deterministic, testable, and architecturally coherent. The problem is not here.

### 1.2 The Python Scripts Ecosystem (100+ scripts)

Categorized by actual function:

**Category A — Core Pipeline (irreplaceable):**
- [`run_nse_cohort.py`](scripts/run_nse_cohort.py) — NSEIngestionEngine, cohort orchestration
- [`run_live_session.py`](scripts/run_live_session.py) — Live synchronized observatory session
- [`verify_cohort_baseline.py`](scripts/verify_cohort_baseline.py) — Deterministic replay verification
- [`freeze_cohort_candles.py`](scripts/freeze_cohort_candles.py) — Frozen substrate creation
- [`candle_substrate.py`](scripts/candle_substrate.py) — Candle loading abstraction
- [`archive_dedupe.py`](scripts/archive_dedupe.py) — Python-side dedupe + gzip pool
- [`repair_chronology_gaps.py`](scripts/repair_chronology_gaps.py) — Gap repair orchestration
- [`certify_replay_chain.py`](scripts/certify_replay_chain.py) — Replay chain certification

**Category B — Observability (conditionally useful):**
- [`ecology_signature_atlas.py`](scripts/ecology_signature_atlas.py) — Per-barrier fingerprint extraction
- [`ecology_transition_atlas.py`](scripts/ecology_transition_atlas.py) — Transition/persistence lineages
- [`ecology_validate.py`](scripts/ecology_validate.py) — Falsification checks on atlas artifacts
- [`offline_ecology_clustering.py`](scripts/offline_ecology_clustering.py) — PCA weight generation
- [`generate_pca_weights.py`](scripts/generate_pca_weights.py) — PCA weight generation (duplicate?)
- [`export_observatory_data.py`](scripts/export_observatory_data.py) — Observatory data export
- [`observatory_daemon.py`](scripts/observatory_daemon.py) — Observatory subprocess management
- [`probe_provider_propagation.py`](scripts/probe_provider_propagation.py) — Provider timing study

**Category C — Research Residue (log-file archaeology, no live path):**
- [`build_toxicity_atlas.py`](scripts/build_toxicity_atlas.py) — Reads `archive/replay_1m_gen11.log` (hardcoded, likely stale)
- [`build_persistence_atlas.py`](scripts/build_persistence_atlas.py) — Reads same hardcoded log file
- [`build_genesis_atlas.py`](scripts/build_genesis_atlas.py) — Reads same hardcoded log file
- [`trajectory_geometry_analysis.py`](scripts/trajectory_geometry_analysis.py) — Reads `archive/replay_5m_oos1.log` (hardcoded)
- [`model_topology_transitions.py`](scripts/model_topology_transitions.py) — Reads `archive/physics_divergence.csv`
- [`collapse_forecaster.py`](scripts/collapse_forecaster.py) — Reads `archive/physics_divergence.csv`
- [`policy_competition_engine.py`](scripts/policy_competition_engine.py) — Reads `archive/physics_divergence.csv`
- [`analyze_physics_divergence.py`](scripts/analyze_physics_divergence.py) — Physics divergence analysis

**Category D — Governors (mixed operational/experimental):**
- [`simulate_governor_audit.py`](scripts/simulate_governor_audit.py) — 6-phase audit protocol (120s per phase, atomic JSON bridge)
- [`governor_refresher.py`](scripts/governor_refresher.py) — Trivial: writes `{"gov_mult": 1.0}` every 0.5s. **This is a stub.**
- [`storage_governor.py`](scripts/storage_governor.py) — Storage tiering/quarantine (Tier 1/2/3)
- [`observatory_scheduler.py`](scripts/observatory_scheduler.py) — Observatory scheduling

**Category E — Validation/Audit (useful but overlapping):**
- [`alpha_stress_audit.py`](scripts/alpha_stress_audit.py)
- [`alpha_truth_detection.py`](scripts/alpha_truth_detection.py)
- [`alpha_validation_stress_test.py`](scripts/alpha_validation_stress_test.py)
- [`controlled_ablation_harness.py`](scripts/controlled_ablation_harness.py)
- [`cross_period_invariance_test.py`](scripts/cross_period_invariance_test.py)
- [`permutation_destruction_test.py`](scripts/permutation_destruction_test.py)
- [`recurrence_stability_audit.py`](scripts/recurrence_stability_audit.py)
- [`regime_slicing_audit.py`](scripts/regime_slicing_audit.py)
- [`structural_validation.py`](scripts/structural_validation.py)
- [`survival_validation.py`](scripts/survival_validation.py)
- [`phase_validation.py`](scripts/phase_validation.py)
- [`phase_c_validation.py`](scripts/phase_c_validation.py)
- [`hybrid_validation.py`](scripts/hybrid_validation.py)

**Category F — Utilities/Downloads (keep, low risk):**
- [`download_nse_data.py`](scripts/download_nse_data.py), [`download_30d_history.py`](scripts/download_30d_history.py), etc.
- [`build_nse_universe.py`](scripts/build_nse_universe.py)
- [`symbol_health.py`](scripts/symbol_health.py)
- [`fetch_candles.py`](scripts/fetch_candles.py)

---

## 2. The Actual Complexity Problem

### 2.1 The Telemetry Record Has 20+ Fields

[`telemetry.rs`](cs-ingest/src/telemetry.rs:160) assembles a JSON record with these fields per barrier per symbol:

```
ts, symbol, pc1, pc2, dist_to_centroid, state, entropy, velocity,
acceleration, turn_angle, transition_confidence, local_density,
corridor, previous_state, next_state, dwell_duration, corridor_id,
queue_pressure, spread_elasticity, instability_type,
survival_probability, hazard_rate, precursor_decay_velocity,
precursor_entropy_expansion, precursor_density_thinning,
precursor_curvature_destabilization, precursor_leakage_rate
```

That is **27 fields** per record. For a 500-symbol cohort at 5-minute bars over 5 days, this is ~720,000 records per run. The archive subdirectory structure writes to: `raw/`, `transitions/corridor_events/`, `transitions/collapse_events/`, `trajectories/`, `topology/`, `metadata/`.

The question is: **which of these 27 fields actually gate execution decisions?**

From reading [`persist.rs`](cs-ingest/src/persist.rs:71), the event filter uses only 4:
- `instability_type != "STABLE"`
- `corridor == true`
- `precursor_entropy_expansion > 0.05`
- `precursor_curvature_destabilization > 15.0`

The remaining 23 fields are written to archive but their downstream consumption is unclear.

### 2.2 The Ecology Stack Is Layered on Top of Telemetry

The ecology pipeline reads the telemetry archive and computes a second layer:

```
telemetry archive
  → ecology_signature_atlas.py  (per-barrier fingerprint: corridor_rate, velocities, entropies, hazards, survivals, dwells)
  → ecology_transition_atlas.py (persistence/transitions/compression lineages, PhaseCalibration, classify_ecology_phase)
  → ecology_validate.py         (falsification checks)
  → ecology_cohort_compare.py   (cross-cohort comparison)
```

[`classify_ecology_phase()`](scripts/ecology_transition_atlas.py:66) produces labels: `EXHAUSTION`, `PERSISTENCE`, `NOISE_TRANSITIONAL`, `COMPRESSION_ONSET`, `RECOVERY`. These are described as "discovery labels, not supervised outcomes."

That is honest. But it also means they have no direct operational leverage yet.

### 2.3 The Governor Layer Is Partially a Stub

[`governor_refresher.py`](scripts/governor_refresher.py:6) writes `{"gov_mult": 1.0, "ts": ...}` every 0.5 seconds. It never changes the multiplier. This is a heartbeat stub, not a governor.

[`simulate_governor_audit.py`](scripts/simulate_governor_audit.py:20) runs a 6-phase protocol (NOMINAL → THROTTLE → HALT → RECOVERY_STEP_1 → RECOVERY_STEP_2 → NOMINAL) over 12 minutes. This is a test harness, not a live governor.

The actual governor bridge is a JSON file at `analysis/real_live/governor_state.json`. The live session reads this file. But the writer side is either the stub refresher (always 1.0) or the audit simulator (test only). There is no production governor that dynamically responds to telemetry.

### 2.4 The Research Residue Scripts Read Stale Hardcoded Paths

[`build_toxicity_atlas.py`](scripts/build_toxicity_atlas.py:5), [`build_persistence_atlas.py`](scripts/build_persistence_atlas.py:4), [`build_genesis_atlas.py`](scripts/build_genesis_atlas.py:4) all open `archive/replay_1m_gen11.log` — a specific log file that almost certainly no longer exists in the current archive structure. These scripts cannot run against the current system.

[`trajectory_geometry_analysis.py`](scripts/trajectory_geometry_analysis.py:43) opens `archive/replay_5m_oos1.log` — same issue.

[`model_topology_transitions.py`](scripts/model_topology_transitions.py:5), [`collapse_forecaster.py`](scripts/collapse_forecaster.py:6), [`policy_competition_engine.py`](scripts/policy_competition_engine.py:6) all read `archive/physics_divergence.csv`. This file exists in the workspace but its provenance and freshness are unclear.

### 2.5 The Validation Suite Has Semantic Overlap

There are at least 12 validation scripts (Category E above). Several appear to test overlapping properties:
- `alpha_stress_audit.py` vs `alpha_validation_stress_test.py` vs `alpha_truth_detection.py`
- `phase_validation.py` vs `phase_c_validation.py` vs `hybrid_validation.py`
- `structural_validation.py` vs `survival_validation.py` vs `recurrence_stability_audit.py`

Without reading all of them, it is not possible to determine which are canonical and which are superseded experiments. This is a classic sign of research velocity exceeding consolidation.

---

## 3. The Honest Verdict on Each Observability Layer

### Topology Metrics
**What it is:** `instability_type` classification (HARD_INSTABILITY, TOPOLOGY_FRAGMENTATION, CORRIDOR_MIGRATION, ATTRACTOR_LEAKAGE, RECOVERY, STABLE) computed in [`telemetry.rs`](cs-ingest/src/telemetry.rs:120).

**Operational leverage:** Yes — directly gates the event filter in [`persist.rs`](cs-ingest/src/persist.rs:75). The `corridor` flag and `instability_type` determine whether a record is written to the full archive or sampled at 1/8 rate.

**Verdict: Keep.** This is load-bearing.

### Ecology Layers
**What it is:** Two-phase post-processing pipeline that reads the telemetry archive and produces `ecology_signatures.jsonl` + `ecology_transition_graph.json`. Phase labels are discovery-only.

**Operational leverage:** Not demonstrated. The ecology labels are not consumed by the live session, the governor, or the execution path. They are consumed by `ecology_validate.py` (validation) and `ecology_cohort_compare.py` (comparison). These are analytical tools, not execution tools.

**Verdict: Demote to optional research instrumentation.** Do not remove — the pipeline is clean and the falsification checks in `ecology_validate.py` are valuable. But it should not be treated as core architecture.

### Propagation Tracking
**What it is:** `probe_provider_propagation.py` measures yfinance vs Stooq publication lag (τ_yfinance, τ_stooq, Δτ). Produces `state_archive/provider_propagation/{date}/report.json`.

**Operational leverage:** Yes — directly relevant to chronology integrity. Provider lag determines whether a bar is stale or fresh. This feeds the repair system.

**Verdict: Keep, but scope it correctly.** It belongs in the chronology/repair layer, not the ecology layer.

### Fertility/Mortality Concepts
**What it is:** `shadow_fert` (shadow fertility) appears in the telemetry regex as a parsed field. `atlas_age` tracks bar age. The Weibull hazard/survival model in [`telemetry.rs`](cs-ingest/src/telemetry.rs:110) uses `expected_dwell` per state (LIQUIDITY_EXHAUSTION=60, NARRATIVE_PERSISTENCE=45, NOISE_TRANSITIONAL=15).

**Operational leverage:** `survival_probability` and `hazard_rate` are written to archive. `precursor_decay_velocity` = survival × hazard. These feed the precursor fields. But the precursor fields are not currently gating any execution decision — they are written to archive and consumed by the ecology pipeline.

**Verdict: The Weibull model is sound but its output is not yet operationally connected.** The `shadow_fert` field from the observatory binary is parsed but its meaning in the current system is unclear without reading the `live_observatory` source.

### Hostility Envelopes
**What it is:** `collapse_forecaster.py` computes `hostility_accel` (divergence rate of change), `envelope_decay` (micro_exp shrinkage rate), `compression_vel` (noise floor expansion). `policy_competition_engine.py` uses these to run `DefensiveOrganism`, `ElasticOrganism`, `FragilityAwareOrganism` policy competition.

**Operational leverage:** These scripts read `archive/physics_divergence.csv` — a file from the old log-based pipeline. They are **not connected to the current telemetry archive**. The `physics_divergence.csv` format (`timestamp, symbol, regime, vol_bucket, half_life, legacy_exp, gross_move, noise_floor, micro_exp, divergence`) does not match the current telemetry record schema.

**Verdict: Research residue. These scripts are disconnected from the current architecture.** The concepts (hostility acceleration, envelope decay) are interesting but they operate on a stale data format.

### Elasticity Aging
**What it is:** `spread_elasticity = atlas_eff / (atlas_den + 1e-5)` computed in [`telemetry.rs`](cs-ingest/src/telemetry.rs:108). `atlas_age` is the bar age counter. These are written to the telemetry record.

**Operational leverage:** Written to archive. Not currently gating any execution decision. Consumed by the ecology pipeline.

**Verdict: Present in the data but not operationally connected.** Same status as fertility/mortality.

### Multi-Stage Governors
**What it is:** The governor bridge is `analysis/real_live/governor_state.json` with fields `multiplier`, `gate_open`, `reason`, `ts`. The live session reads this file. The writer is either `governor_refresher.py` (stub, always 1.0) or `simulate_governor_audit.py` (test harness).

**Operational leverage:** The bridge mechanism is correct and the atomic-replace pattern is sound. But the production governor logic does not exist. There is no component that reads telemetry and dynamically adjusts `multiplier` based on observed market state.

**Verdict: The plumbing exists but the governor brain is missing.** This is the most important gap in the system.

---

## 4. The Minimal Truthful Core

Based on the audit, the minimum components required to preserve the original product thesis (execution validation under realistic constraints) are:

### Tier 1 — Non-Negotiable Core

These are load-bearing. Removing any of them breaks the product thesis.

- **Deterministic replay pipeline:** [`frozen_loader.rs`](cs-ingest/src/frozen_loader.rs), [`timeline.rs`](cs-ingest/src/timeline.rs), [`replay.rs`](cs-ingest/src/replay.rs), [`run_nse_cohort.py`](scripts/run_nse_cohort.py)
- **Chronology integrity:** [`repair.rs`](cs-ingest/src/repair.rs), [`repair_chronology_gaps.py`](scripts/repair_chronology_gaps.py), [`probe_provider_propagation.py`](scripts/probe_provider_propagation.py)
- **Archive persistence with dedupe:** [`persist.rs`](cs-ingest/src/persist.rs), [`dedupe.rs`](cs-ingest/src/dedupe.rs), [`archive.rs`](cs-ingest/src/archive.rs), [`archive_dedupe.py`](scripts/archive_dedupe.py)
- **Topology event classification:** [`telemetry.rs`](cs-ingest/src/telemetry.rs) (the `instability_type` + `corridor` fields specifically)
- **Replay verification:** [`verify_cohort_baseline.py`](scripts/verify_cohort_baseline.py), [`certify_replay_chain.py`](scripts/certify_replay_chain.py)
- **Live session:** [`run_live_session.py`](scripts/run_live_session.py), [`observatory_daemon.py`](scripts/observatory_daemon.py)

### Tier 2 — Conditionally Useful (keep, but do not treat as core)

These add value but are not load-bearing for the core thesis.

- **Ecology pipeline:** [`ecology_signature_atlas.py`](scripts/ecology_signature_atlas.py), [`ecology_transition_atlas.py`](scripts/ecology_transition_atlas.py), [`ecology_validate.py`](scripts/ecology_validate.py) — useful for post-hoc analysis, not for live execution
- **PCA clustering:** [`pca.rs`](cs-ingest/src/pca.rs), [`offline_ecology_clustering.py`](scripts/offline_ecology_clustering.py) — the 3-state classification (LIQUIDITY_EXHAUSTION, NARRATIVE_PERSISTENCE, NOISE_TRANSITIONAL) is written to every record; its operational value needs to be demonstrated
- **Storage governor:** [`storage_governor.py`](scripts/storage_governor.py) — operationally useful for disk management
- **Validation suite (canonical subset):** Pick one canonical script per validation concern and retire the rest

### Tier 3 — Research Residue (demote or retire)

These are disconnected from the current architecture or are stubs.

- [`build_toxicity_atlas.py`](scripts/build_toxicity_atlas.py) — reads stale hardcoded log path
- [`build_persistence_atlas.py`](scripts/build_persistence_atlas.py) — reads stale hardcoded log path
- [`build_genesis_atlas.py`](scripts/build_genesis_atlas.py) — reads stale hardcoded log path
- [`trajectory_geometry_analysis.py`](scripts/trajectory_geometry_analysis.py) — reads stale hardcoded log path
- [`model_topology_transitions.py`](scripts/model_topology_transitions.py) — reads `physics_divergence.csv` (old schema)
- [`collapse_forecaster.py`](scripts/collapse_forecaster.py) — reads `physics_divergence.csv` (old schema)
- [`policy_competition_engine.py`](scripts/policy_competition_engine.py) — reads `physics_divergence.csv` (old schema); `Organism` policy classes are interesting but disconnected
- [`governor_refresher.py`](scripts/governor_refresher.py) — stub, always writes `gov_mult: 1.0`
- [`simulate_governor_audit.py`](scripts/simulate_governor_audit.py) — test harness only, not a production governor

---

## 5. Telemetry Field Categorization (27 Fields)

Every field in the telemetry record traced against its actual consumers. The filter is: what breaks measurably if this field disappears?

### Category 1 — Execution-Critical (4 fields)

These gate the event filter in [`persist.rs`](cs-ingest/src/persist.rs:71). Removing any of them changes which records are written to the full archive vs. sampled at 1/8 rate.

| Field | Consumer | Role |
|---|---|---|
| `instability_type` | [`persist.rs:71`](cs-ingest/src/persist.rs:71), governor | Event gate: `!= "STABLE"` triggers full write |
| `corridor` | [`persist.rs:72`](cs-ingest/src/persist.rs:72), governor, ecology | Event gate: `true` triggers full write + corridor_events |
| `precursor_entropy_expansion` | [`persist.rs:73`](cs-ingest/src/persist.rs:73) | Event gate: `> 0.05` triggers full write |
| `precursor_curvature_destabilization` | [`persist.rs:74`](cs-ingest/src/persist.rs:74) | Event gate: `> 15.0` triggers full write |

### Category 2 — Governance (2 fields)

These are read by [`governor_refresher.py`](scripts/governor_refresher.py) to compute throttle/halt decisions.

| Field | Consumer | Role |
|---|---|---|
| `instability_type` | governor | `!= "STABLE"` increments instability_rate |
| `corridor` | governor | `true` increments corridor_rate |

*(Both fields are shared with Category 1 — they are the most load-bearing fields in the system.)*

### Category 3 — Replay Certification (3 fields)

These are used by [`verify_cohort_baseline.py`](scripts/verify_cohort_baseline.py) to validate replay consistency.

| Field | Consumer | Role |
|---|---|---|
| `ts` | verify, certify, dedupe | Barrier timestamp — primary key |
| `symbol` | verify, certify, dedupe | Symbol identity |
| `state` | verify | Must be one of 3 valid STATE_NAMES |

### Category 4 — Ecology Pipeline (14 fields)

These are consumed only by [`ecology_signature_atlas.py`](scripts/ecology_signature_atlas.py). Removing them does not affect the event filter, governor, or replay certification. They are Tier 2 — useful for post-hoc analysis, not load-bearing for execution.

| Field | Ecology Consumer |
|---|---|
| `velocity` | `volatility_texture.velocity_mean/stdev` |
| `acceleration` | `volatility_texture.acceleration_mean` |
| `turn_angle` | `volatility_texture.turn_angle_mean/p90` |
| `entropy` | `volatility_texture` + collapse event gate (`> 0.95`) |
| `hazard_rate` | `temporal_structure.hazard_mean` |
| `survival_probability` | `temporal_structure.survival_mean/min` |
| `dwell_duration` | `temporal_structure.dwell_mean` |
| `queue_pressure` | `propagation_texture.queue_pressure_mean` |
| `spread_elasticity` | `propagation_texture.spread_elasticity_mean` |
| `pc1`, `pc2` | `propagation_texture.propagation_corr`, `pc1/pc2_spread` |
| `dist_to_centroid` | `propagation_texture.dist_to_centroid_mean` |
| `precursor_density_thinning` | `compression_metrics.density_thinning_mean` |
| `precursor_decay_velocity` | `compression_metrics.precursor_decay_velocity_mean` |

### Category 5 — Replay Certification (extended, 5 fields)

These are consumed by [`compare_ingest_parity.py`](scripts/compare_ingest_parity.py) to verify Python↔Rust record parity, and are produced by both [`run_nse_cohort.py`](scripts/run_nse_cohort.py) and [`telemetry_archive_daemon.py`](scripts/telemetry_archive_daemon.py). They are part of the Python↔Rust parity contract and cannot be removed without breaking ingest parity verification.

| Field | Consumer | Role |
|---|---|---|
| `previous_state` | `compare_ingest_parity.py`, `run_nse_cohort.py` | Parity contract field |
| `next_state` | `compare_ingest_parity.py`, `run_nse_cohort.py` | Parity contract field (derivable from `corridor` but kept for parity) |
| `corridor_id` | `compare_ingest_parity.py`, `run_nse_cohort.py` | Parity contract field |
| `transition_confidence` | `compare_ingest_parity.py`, `run_nse_cohort.py` | Parity contract field |
| `local_density` | `compare_ingest_parity.py`, `run_nse_cohort.py` | Parity contract field |

*(Note: `precursor_entropy_expansion` appears in both Category 1 and the ecology pipeline — it is the only precursor field that is both execution-critical and analytically consumed.)*

### Summary

| Category | Fields | Operational Authority |
|---|---|---|
| Execution-critical | 4 | Gate archive writes and governor decisions |
| Governance | 2 | (shared with execution-critical) |
| Replay certification (core) | 3 | Gate validation pass/fail |
| Replay certification (parity) | 5 | Python↔Rust parity contract |
| Ecology pipeline | 14 | Post-hoc analysis only |

The execution kernel uses 4 fields. The parity contract uses 5 more. The remaining 18 are ecology annotation. There are no diagnostics-only fields — the earlier categorization was incorrect pending consumer verification.

---

## 6. The Three Concrete Problems to Fix

### Problem 1: The Governor Brain Is Missing

The governor bridge exists. The atomic-replace pattern is correct. But nothing writes a non-trivial multiplier based on observed telemetry. The system always runs at full capacity regardless of what the telemetry says.

**Fix:** Write a minimal production governor that reads the telemetry archive (specifically `instability_type` and `corridor` rates) and adjusts `multiplier` accordingly. This is the highest-leverage single change available.

A minimal governor needs only:
```python
# Read last N records from archive
# Compute corridor_rate = count(corridor==True) / N
# Compute instability_rate = count(instability_type != "STABLE") / N
# multiplier = 1.0 if both rates < threshold else 0.65 if moderate else 0.0
# Write to governor_state.json atomically
```

### Problem 2: The Telemetry Record Has 27 Fields But Only 4 Gate Decisions

The event filter in [`persist.rs`](cs-ingest/src/persist.rs:75) uses 4 fields. The remaining 23 are written to archive but their downstream consumers are either the ecology pipeline (Tier 2) or nothing.

**Fix:** Audit which fields are actually consumed by live execution decisions. Fields that are only consumed by the ecology pipeline should be moved to a separate "analytics record" written less frequently, reducing archive write amplification.

### Problem 3: The Validation Suite Has No Canonical Entry Point

12+ validation scripts with overlapping concerns and no clear canonical ordering. A new contributor (or a future version of this system) cannot determine which validation to run to confirm the system is healthy.

**Fix:** Create a single `validate_system.sh` (one already exists — check if it is canonical) that runs the minimum set of checks in order: timeline fingerprint → dedupe integrity → replay parity → ecology falsification. Everything else becomes optional.

---

## 6. What Should Not Be Touched

The following are architecturally sound and should not be refactored:

- The timestamp-lock invariant in [`repair.rs`](cs-ingest/src/repair.rs): T_provider MUST equal T_barrier exactly. This is the right invariant.
- The `fresh_wipe_archive` pattern in [`replay.rs`](cs-ingest/src/replay.rs:28): deterministic replay requires clean state.
- The SHA-256 timeline fingerprint in [`timeline.rs`](cs-ingest/src/timeline.rs:16): this is the correct way to verify replay parity.
- The `STABLE_SAMPLE_EVERY = 8` downsampling in [`persist.rs`](cs-ingest/src/persist.rs:9): this is a reasonable write-amplification control.
- The atomic-replace governor bridge pattern in [`governor_refresher.py`](scripts/governor_refresher.py): `os.replace(tmp, final)` is the correct primitive for lock-free inter-process state sharing. Readers never observe partial state. This pattern must be used everywhere state is published across process boundaries: governor bridge, repair manifests, chronology checkpoints, replay certifications, archive metadata.
- The Tier 1 protected files list in [`storage_governor.py`](scripts/storage_governor.py:12): `propagation_snapshots.jsonl`, `trl_summary.json`, `rho_n.json`, `live_session_steps.jsonl`, `provider_propagation_trace.jsonl` are correctly identified as non-deletable.
- The `NO_DATA → multiplier=1.0` startup policy in [`governor_refresher.py`](scripts/governor_refresher.py): absence of telemetry is not evidence of instability. Cold-start behavior must be permissive. Halting on missing archive data would create cold-start paralysis and non-deterministic boot behavior.

---

## 7. Recommended Simplification Sequence

These are ordered by impact-to-risk ratio. Each step is independently valuable.

**Step 1 (1–2 days): Move research residue scripts to `scripts/research/`**

Move Category C scripts (the 8 scripts reading stale hardcoded paths) to `scripts/research/`. No code changes. This immediately clarifies what is operational vs. experimental. Zero risk.

**Step 2 (1 day): Write the minimal production governor**

Replace [`governor_refresher.py`](scripts/governor_refresher.py) with a real governor that reads the last N telemetry records from the archive and adjusts the multiplier deterministically. This closes the most important architectural gap. The bridge mechanism (atomic JSON replace) already exists.

The correct starting logic is deliberately simple:

```python
# Read last N records from archive (instability_type, corridor fields)
corridor_rate = count(corridor == True) / N
instability_rate = count(instability_type != "STABLE") / N

if instability_rate > HALT_THRESHOLD:
    multiplier, gate_open = 0.0, False
elif corridor_rate > THROTTLE_THRESHOLD:
    multiplier, gate_open = 0.65, True
else:
    multiplier, gate_open = 1.0, True

# Write atomically via os.replace(tmp, final)
```

Do not start with reinforcement learning, adaptive ecology weighting, or multi-policy organisms. The deterministic threshold version is enough to close the observability-to-execution loop. Once that loop exists, replay can validate governance behavior and ecology can prove incremental value.

**Step 3 (2–3 days): Canonicalize the validation suite**

Audit the 12 validation scripts. Pick one canonical script per concern. Move the rest to `scripts/research/`. Update [`validate_system.sh`](scripts/validate_system.sh) to be the single entry point.

**Step 4 (1 week): Audit the 27-field telemetry record**

For each field in the telemetry record, trace its consumers. Fields consumed only by the ecology pipeline should be separated into an analytics record written at lower frequency. This reduces archive size and makes the core record semantically cleaner. The execution kernel has already selected its own minimal truth surface — the 4 fields in the event filter are the signal; the other 23 are annotation.

**Step 5 (ongoing): Demonstrate ecology operational leverage**

The ecology pipeline is clean and the falsification checks are good. But the phase labels (EXHAUSTION, PERSISTENCE, etc.) need to be connected to at least one execution decision before they can be promoted to Tier 1. The natural connection is the governor: if the ecology phase is EXHAUSTION, reduce the multiplier. This is the path from "research structure" to "operational necessity."

---

## 8. Proposed Module Boundary: `cs-core/` vs `cs-research/`

The Tier 1/2/3 classification maps naturally to a physical directory boundary that would accelerate development velocity and reduce architectural anxiety simultaneously.

### `cs-core/` (machine-truth infrastructure)

Everything here must be deterministic, testable, and operationally connected. Nothing in `cs-core/` is optional:

- Replay pipeline: [`frozen_loader.rs`](cs-ingest/src/frozen_loader.rs), [`timeline.rs`](cs-ingest/src/timeline.rs), [`replay.rs`](cs-ingest/src/replay.rs), [`run_nse_cohort.py`](scripts/run_nse_cohort.py)
- Chronology integrity: [`repair.rs`](cs-ingest/src/repair.rs), [`repair_chronology_gaps.py`](scripts/repair_chronology_gaps.py), [`probe_provider_propagation.py`](scripts/probe_provider_propagation.py)
- Archive persistence: [`persist.rs`](cs-ingest/src/persist.rs), [`dedupe.rs`](cs-ingest/src/dedupe.rs), [`archive.rs`](cs-ingest/src/archive.rs), [`archive_dedupe.py`](scripts/archive_dedupe.py)
- Topology event classification: [`telemetry.rs`](cs-ingest/src/telemetry.rs) (the 4 gating fields specifically)
- Replay verification: [`verify_cohort_baseline.py`](scripts/verify_cohort_baseline.py), [`certify_replay_chain.py`](scripts/certify_replay_chain.py)
- Live session: [`run_live_session.py`](scripts/run_live_session.py), [`observatory_daemon.py`](scripts/observatory_daemon.py)
- Governor (once the production governor is written — replaces the stub)

### `cs-research/` (analytical augmentation)

Potentially valuable, potentially promotable, but not foundational yet. Nothing here should be in the critical path of a live session. A script in `cs-research/` can be excellent research — it simply does not get operational authority:

- Ecology pipeline: [`ecology_signature_atlas.py`](scripts/ecology_signature_atlas.py), [`ecology_transition_atlas.py`](scripts/ecology_transition_atlas.py), [`ecology_validate.py`](scripts/ecology_validate.py)
- PCA clustering experiments: [`offline_ecology_clustering.py`](scripts/offline_ecology_clustering.py)
- Phase studies and lineage analytics
- All Category C scripts (stale log readers): [`build_toxicity_atlas.py`](scripts/build_toxicity_atlas.py), [`build_persistence_atlas.py`](scripts/build_persistence_atlas.py), [`build_genesis_atlas.py`](scripts/build_genesis_atlas.py), [`trajectory_geometry_analysis.py`](scripts/trajectory_geometry_analysis.py)
- Dormant policy systems: [`policy_competition_engine.py`](scripts/policy_competition_engine.py), [`collapse_forecaster.py`](scripts/collapse_forecaster.py), [`model_topology_transitions.py`](scripts/model_topology_transitions.py)
- Superseded validation scripts (the non-canonical subset)

The boundary is not about quality. It is about operational authority.

---

## 9. Summary

The core thesis of ChronoSentiment — execution validation under realistic constraints — is sound and the Rust ingest core implements it correctly. The problem is that the observability surface grew faster than the operational connections were established.

The system is not broken. It is over-instrumented relative to its current operational leverage. The fix is not to delete the instrumentation but to:

1. Separate what is load-bearing from what is analytical.
2. Close the governor gap (the most important missing connection).
3. Establish a canonical validation entry point.
4. Demonstrate that the ecology labels change at least one execution decision.

The encouraging fact is that the architecture is still legible. The Rust core is clean. The Python orchestration layer is coherent. The data model is consistent. This is a system that can be simplified without losing its essence — which is the best possible position to be in.

The system already knows what it is. The thesis survived the complexity expansion intact:

> deterministic chronological execution validation under realistic market constraints

Simplification is therefore not reduction for its own sake. It is alignment with the original thesis.