#!/bin/bash
# Double-click or: open scripts/research/rp608_pull_pi_journals_2026-11-05.command
# Pulls four-book journals from LCUtest and runs W/L-first session export for 2026-11-05.
set -euo pipefail
PI="${PI:-ultraspeed@192.168.0.9}"
REPO="/home/ultraspeed/ChronoSentiment_MEGA_FINAL"
LOCAL="/Users/nikhil/ChronoSentiment_MEGA_FINAL"
DAY="2026-11-05"
OUT="$LOCAL/artifacts/rp608_econ_edge_diag/source_journals_$DAY"

cd "$LOCAL"
mkdir -p \
  "$LOCAL/live_capture/shadow_validation/nse/of_paper_ledger_v1" \
  "$LOCAL/live_capture/shadow_validation/crypto/trade_plan_overlay_v1" \
  "$OUT/nse/of_paper_ledger_v1" "$OUT/crypto"

echo "=== SSH $PI ==="
ssh -o BatchMode=yes -o ConnectTimeout=15 "$PI" 'echo PI_OK; TZ=Asia/Kolkata date; hostname'

echo "=== rsync NSE A + pending ==="
rsync -az --progress \
  "$PI:$REPO/live_capture/shadow_validation/nse/rp608_nse_trade_journal.jsonl" \
  "$PI:$REPO/live_capture/shadow_validation/nse/rp608_nse_pending_evaluations.json" \
  "$PI:$REPO/live_capture/shadow_validation/nse/rp608_nse_latest_evaluations.json" \
  "$LOCAL/live_capture/shadow_validation/nse/"

echo "=== rsync NSE OF ==="
rsync -az --progress \
  "$PI:$REPO/live_capture/shadow_validation/nse/of_paper_ledger_v1/of_trade_outcomes.jsonl" \
  "$PI:$REPO/live_capture/shadow_validation/nse/of_paper_ledger_v1/of_paper_trades_open.json" \
  "$PI:$REPO/live_capture/shadow_validation/nse/of_paper_ledger_v1/of_paper_state.json" \
  "$LOCAL/live_capture/shadow_validation/nse/of_paper_ledger_v1/"

echo "=== rsync crypto plan + OF ==="
rsync -az --progress \
  "$PI:$REPO/live_capture/shadow_validation/crypto/trade_plan_overlay_v1/trade_plan_outcomes.jsonl" \
  "$PI:$REPO/live_capture/shadow_validation/crypto/trade_plan_overlay_v1/of_trade_outcomes.jsonl" \
  "$PI:$REPO/live_capture/shadow_validation/crypto/trade_plan_overlay_v1/of_paper_trades_open.json" \
  "$LOCAL/live_capture/shadow_validation/crypto/trade_plan_overlay_v1/"

echo "=== mirror dated artifact ==="
cp -f "$LOCAL/live_capture/shadow_validation/nse/rp608_nse_trade_journal.jsonl" "$OUT/nse/"
cp -f "$LOCAL/live_capture/shadow_validation/nse/of_paper_ledger_v1/"* "$OUT/nse/of_paper_ledger_v1/" 2>/dev/null || true
cp -f "$LOCAL/live_capture/shadow_validation/crypto/trade_plan_overlay_v1/trade_plan_outcomes.jsonl" "$OUT/crypto/"
cp -f "$LOCAL/live_capture/shadow_validation/crypto/trade_plan_overlay_v1/of_trade_outcomes.jsonl" "$OUT/crypto/"

echo "=== line counts ==="
wc -l \
  "$LOCAL/live_capture/shadow_validation/nse/rp608_nse_trade_journal.jsonl" \
  "$LOCAL/live_capture/shadow_validation/nse/of_paper_ledger_v1/of_trade_outcomes.jsonl" \
  "$LOCAL/live_capture/shadow_validation/crypto/trade_plan_overlay_v1/trade_plan_outcomes.jsonl" \
  "$LOCAL/live_capture/shadow_validation/crypto/trade_plan_overlay_v1/of_trade_outcomes.jsonl"

echo "=== session export $DAY ==="
python3 scripts/research/rp608_econ_edge_session_export.py --day "$DAY" \
  --out-dir artifacts/rp608_econ_edge_diag

echo "=== W/L-first attribution ==="
python3 scripts/research/rp608_econ_edge_wl_first_attribution.py --day "$DAY" \
  --export-dir artifacts/rp608_econ_edge_diag

echo "DONE — see artifacts/rp608_econ_edge_diag/session_export_${DAY}.md"
echo "Press Enter to close."
read -r _
