# Quarantine (not compiled) — CLEAN-003-C archive

**Moved from** `adapters/chronosentiment/legacy/quarantine/` on 2026-11-24.

Preserved sources that must not appear on the product or `legacy-lake` compile
graph.

| Asset | Why kept | Why not compiled |
|-------|----------|------------------|
| `tests/week2_tests.rs` | Shows the stale `AssessmentValue` API | Does not compile against current `AssessmentEngine` |
| `reasoning/*.rs` | Unused engines previously commented out of `reasoning/mod.rs` | Wall-clock / `Uuid::new_v4` / hidden thresholds |
| `*.bak` | Accidental backups | Not source of record |

`scripts/phase_c_gate.sh` was **removed** (CLN-016): it printed PASS without
running tests. Phase C fixtures remain at `tests/fixtures/phase_c_replay/`.
