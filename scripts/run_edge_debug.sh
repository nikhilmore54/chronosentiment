#!/usr/bin/env zsh
# ─────────────────────────────────────────────────────────────────
# ChronoSentiment — Edge Degeneracy Diagnostic
# Runs ONE generation with GA_DEBUG=1 and prints only the signals
# needed to confirm the 5 surgical fixes are working.
# Run from ChronoSentiment_MEGA_FINAL root:
#   chmod +x run_edge_debug.sh && ./run_edge_debug.sh
# ─────────────────────────────────────────────────────────────────

source ~/.zshrc 2>/dev/null || true
source ~/.cargo/env 2>/dev/null || true

LOG=/tmp/ga_edge_debug.log

echo "🔬 Running edge diagnostic (1 generation, pop=3)..."
echo "   Full log → $LOG"
echo ""

GA_DEBUG=1 \
DATA_FOLDER=/Users/nikhil/ChronoSentiment_MEGA_FINAL/test_assets \
GA_POPULATION_SIZE=3 \
GA_GENERATIONS=1 \
GA_MAX_HOLD_BARS=15 \
MIN_CANDLES=50 \
cargo run --example train_nse 2>&1 | tee "$LOG"

echo ""
echo "════════════════════════════════════════════════════════════"
echo "📊 EDGE_DEBUG output (first 20):"
echo "────────────────────────────────"
grep "EDGE_DEBUG" "$LOG" | head -20

echo ""
echo "📊 EDGE COLLAPSE detection:"
echo "────────────────────────────"
grep "EDGE COLLAPSE" "$LOG" | head -10

echo ""
echo "📊 REGIME_BANDS:"
echo "────────────────────────────"
grep "REGIME_BANDS" "$LOG" | head -5

echo ""
echo "📊 EFF_CHECK_NEW (norm_pnl diversity check):"
echo "────────────────────────────"
grep "EFF_CHECK_NEW" "$LOG" | head -10

echo ""
echo "📊 EXEC_FLOW (edge values flowing to executor):"
echo "────────────────────────────"
grep "EXEC_FLOW" "$LOG" | head -5
