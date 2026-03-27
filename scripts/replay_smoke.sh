#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${BASE_URL:-http://localhost:8000/signals/replay-suggestions}"

if ! command -v jq >/dev/null 2>&1; then
  echo "jq is required for this smoke script."
  exit 1
fi

echo "Running SUMMARY mode..."
curl -sS "${BASE_URL}?mode=summary" | jq '{
  participation_pct: .metrics.participation_pct,
  avg_suggestions_per_event: .metrics.avg_suggestions_per_event,
  strategy_flips: .metrics.strategy_flips,
  flip_rate: .metrics.flip_rate,
  effective_signals: .metrics.effective_signals,
  effective_signal_rate: .metrics.effective_signal_rate,
  rejected_low_edge: .metrics.final_debug.rejected_low_edge,
  rejected_low_exec: .metrics.final_debug.rejected_low_exec,
  rejected_hold: .metrics.final_debug.rejected_hold,
  suppressed_stability: .metrics.final_debug.suppressed_stability
}'

echo ""
echo "Running FLIP diagnostics..."
curl -sS "${BASE_URL}?mode=summary" | jq '{
  flip_rate: .metrics.flip_rate,
  flips: .metrics.strategy_flips,
  effective_signal_rate: .metrics.effective_signal_rate,
  persistence: .metrics.top_strategy_persistence_ticks
}'

echo ""
echo "Running Tradability verdict..."
SUMMARY_JSON="$(curl -sS "${BASE_URL}?mode=summary")"
echo "${SUMMARY_JSON}" | jq '{
  tradability_band: .metrics.tradability_band,
  effective_signal_rate: .metrics.effective_signal_rate,
  flip_rate: .metrics.flip_rate
}'

VERDICT="$(echo "${SUMMARY_JSON}" | jq -r '.metrics.tradability_band')"
RATE="$(echo "${SUMMARY_JSON}" | jq -r '.metrics.effective_signal_rate')"
FLIP="$(echo "${SUMMARY_JSON}" | jq -r '.metrics.flip_rate')"

case "${VERDICT}" in
  "Unusable")
    ICON="❌"
    ;;
  "Sparse")
    ICON="⚠️"
    ;;
  "Tradable")
    ICON="✅"
    ;;
  "Strong")
    ICON="🔥"
    ;;
  "Overactive")
    ICON="⚠️"
    ;;
  *)
    ICON="❓"
    ;;
esac

echo ""
echo "Tradability: ${ICON} ${VERDICT} (rate=${RATE}, flip=${FLIP})"

echo ""
echo "Running SAMPLED mode..."
curl -sS "${BASE_URL}?mode=sampled&sample_rate=20&limit=500" | jq '{
  sample_points: (.timeline | length),
  first_point: .timeline[0],
  last_point: .timeline[-1]
}'

echo ""
echo "Running FULL mode (limited)..."
curl -sS "${BASE_URL}?mode=full&limit=200" | jq '{
  total_points: (.timeline | length)
}'

echo ""
echo "Running PnL diagnostics..."
curl -sS "${BASE_URL}?mode=summary" | jq '{
  trades: .pnl.total_trades,
  win_rate: .pnl.win_rate,
  avg_pnl: .pnl.avg_pnl,
  total_pnl: .pnl.total_pnl,
  edge_retention: .pnl.edge_retention
}'
