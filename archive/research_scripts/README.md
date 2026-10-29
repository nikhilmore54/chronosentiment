# Research Scripts Archive

This directory contains historical, diagnostic, and exploratory Python scripts that were used during research phases (RP-401 through RP-614) and ad-hoc investigative workflows.

They have been archived to align the repository back with its **Rust-first production architecture** (`adapters/chronosentiment` and `coralys-*` crates).

---

## Directory Organization

1. **`diagnostic_and_phase_scripts/` (367 scripts):**
   - Historical discovery, diagnostic, and transition analysis scripts from past phases (e.g. `rp401_...` through `rp614_...`).
   - One-off diagnostic probes, schema patches, and scratch analyzers.
   - Historical replay and backfill harnesses that have been superseded by canonical Rust replay and governed shadow engines.

2. **`legacy_research/` (84 scripts):**
   - Exploratory parameter searches, ad-hoc regressions, and scratch notebooks previously situated in `scripts/research/`.

---

## Governed Active Scripts Policy (`scripts/`)

Only active operational runners and CI/governance harnesses remain in `scripts/`:
- **Active Operations:** `run_rp615_nse_preopen_check.py`, `emit_nse_session_recommendations.py`, `run_rp615_crypto_shadow_daemon.py`, `emit_rp615_crypto_live_metrics.py`.
- **Active Model Ingestion & Inference:** `run_rp615_predictive_system.py`, `train_rp615_predictive_model.py`, `build_rp615_predictive_dataset.py`, `rp615_feature_extractor.py`.
- **CI / Architectural Invariants:** `semantic_lint.py`, `check_arch_invariants.py`, `lane2-lane4` verification harnesses.
- **Active Research:** Only current active study scripts are maintained in `scripts/research/` (`study_historical_tick_levels.py`, `counterfactual_55_trades_revaluation.py`).

### Script Discipline Enforcement:
1. **No Standalone Scratch Scripts:** Do not create new ad-hoc `.py` files in `scripts/`.
2. **Extend Governed Harnesses:** Extend existing test suites in `tests/` or governed runners in `scripts/`.
3. **Rust Authority:** All stabilized execution logic, models, and telemetry must be ported to Rust crates/binaries rather than remaining in Python.
