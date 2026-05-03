#!/usr/bin/env bash
# Final calibration smoke: same deterministic emitter + grid search, longer tape (default 1000 steps).
# Aligned with chronosentiment-core.mdc — reproducible from VALIDATION_BATCH_STEPS + grid script.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
export VALIDATION_BATCH_STEPS="${VALIDATION_BATCH_STEPS:-1000}"
python3 scripts/grid_search_momentum_bootstrap.py --steps "${VALIDATION_BATCH_STEPS}" "$@"
