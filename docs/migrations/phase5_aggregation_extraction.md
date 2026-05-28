# Migration Manifest: Phase 5 Aggregation Extraction

## Intent
Extracting semantic aggregation and data grouping logic from `pipeline/old.rs`. This zone is responsible for collecting, reshaping, and slicing chronology windows (e.g., `scenarios_from_candles`) and exposing semantic truth via endpoints (`generate_latest_signals`). It performs zero causal decision-making, evaluation, scoring, or routing.

## Extracted Symbols
- [ ] `...` (To be populated)

## Known Unchanged Invariants
- `pipeline/old.rs` continues to handle orchestration, sweeps, and routing.
- Replay equivalence hashes must remain perfectly intact.
- Aggregation boundaries (candle slices, grouping) remain identical.

## Certification Hashes
- **Pre-Migration Replay Hash**: PENDING
- **Post-Migration Replay Hash**: PENDING
- **Equivalence Status**: PENDING
