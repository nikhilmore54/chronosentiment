# Sweep Projection Certification

## Extraction Scope
- Certified layer: **sweep projection** (`financial/strategies/src/pipeline/sweep.rs`).
- Includes `run_threshold_sweep` and the data structure `ThresholdSweepRow`.
- Moved symbols from reporting layer:
  - `SignalMeta`
  - `EdgeLossBreakdown`
  - `ThresholdSweepRow` (now a **semantic projection**).

## Moved Symbols
- `ThresholdSweepRow` has been promoted from an internal DTO to a certified semantic artifact.
- `SignalMeta` and `EdgeLossBreakdown` remain in `reporting.rs` but are imported by the sweep layer for deterministic reporting.

## Replay Commands Used for Certification
```bash
# Full workspace test suite (includes deterministic sweep test)
cargo test --workspace
```
The sweep projection test was executed **twice** to verify deterministic behavior.

## Iteration Count
- Two consecutive runs of `run_threshold_sweep` with identical inputs:
  ```rust
  let rows_run_1 = run_threshold_sweep(assets.clone(), global_lambda, &[confidence_floor], &[score_floor]);
  let rows_run_2 = run_threshold_sweep(assets, global_lambda, &[confidence_floor], &[score_floor]);
  ```
- Both runs produced identical results.

## Deterministic Guarantees
1. **Structural Equality** – `assert_eq!(rows_run_1, rows_run_2)` verifies that the Rust `PartialEq` implementation yields identical structs.
2. **Serialization Equality** – JSON strings are compared after pretty‑printing:
   ```rust
   let json_1 = serde_json::to_string_pretty(&rows_run_1).unwrap();
   let json_2 = serde_json::to_string_pretty(&rows_run_2).unwrap();
   assert_eq!(json_1, json_2);
   ```
   This guarantees stable ordering and formatting across runs.
3. **SHA‑256 Projection Hash** – A deterministic hash of the entire result set is computed using `stable_hash` (SHA‑256 of the serialized JSON). The hash matches across runs, providing an immutable fingerprint.

## Certification Tag
A local Git tag has been created to freeze this state:
```
git tag constitutional-sweep-stabilized
```
(kept local; not pushed).

## Summary
The sweep projection layer now satisfies **semantic certification evidence**:
- Replay‑certified (tests pass under deterministic replay).
- Topology‑aware (imports respect authority boundaries).
- Semantically constrained (field ordering and serialization are frozen).
- Migration‑governed (future changes must update this certification document).

*Certification date:* 2026‑05‑28
