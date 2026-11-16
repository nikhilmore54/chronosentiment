#!/bin/bash
set -euo pipefail
PI="${PI:-ultraspeed@192.168.0.9}"
REPO="/home/ultraspeed/ChronoSentiment_MEGA_FINAL"
LOCAL="/Users/nikhil/ChronoSentiment_MEGA_FINAL"
DAY="2026-11-05"
OUT="$LOCAL/artifacts/rp608_econ_edge_diag/source_journals_$DAY"
LOG="$LOCAL/artifacts/rp608_econ_edge_diag/pi_pull_${DAY}.log"
exec > >(tee -a "$LOG") 2>&1
cd "$LOCAL"
mkdir -p \
  "$LOCAL/live_capture/shadow_validation/nse/of_paper_ledger_v1" \
  "$LOCAL/live_capture/shadow_validation/crypto/trade_plan_overlay_v1" \
  "$OUT/nse/of_paper_ledger_v1" "$OUT/crypto"
echo "=== $(date) SSH $PI ==="
ssh -o BatchMode=yes -o ConnectTimeout=20 "$PI" 'echo PI_OK; TZ=Asia/Kolkata date; hostname'
rsync -az \
  "$PI:$REPO/live_capture/shadow_validation/nse/rp608_nse_trade_journal.jsonl" \
  "$PI:$REPO/live_capture/shadow_validation/nse/rp608_nse_pending_evaluations.json" \
  "$PI:$REPO/live_capture/shadow_validation/nse/rp608_nse_latest_evaluations.json" \
  "$LOCAL/live_capture/shadow_validation/nse/"
rsync -az \
  "$PI:$REPO/live_capture/shadow_validation/nse/of_paper_ledger_v1/of_trade_outcomes.jsonl" \
  "$PI:$REPO/live_capture/shadow_validation/nse/of_paper_ledger_v1/of_paper_trades_open.json" \
  "$PI:$REPO/live_capture/shadow_validation/nse/of_paper_ledger_v1/of_paper_state.json" \
  "$LOCAL/live_capture/shadow_validation/nse/of_paper_ledger_v1/"
rsync -az \
  "$PI:$REPO/live_capture/shadow_validation/crypto/trade_plan_overlay_v1/trade_plan_outcomes.jsonl" \
  "$PI:$REPO/live_capture/shadow_validation/crypto/trade_plan_overlay_v1/of_trade_outcomes.jsonl" \
  "$PI:$REPO/live_capture/shadow_validation/crypto/trade_plan_overlay_v1/of_paper_trades_open.json" \
  "$LOCAL/live_capture/shadow_validation/crypto/trade_plan_overlay_v1/"
cp -f "$LOCAL/live_capture/shadow_validation/nse/rp608_nse_trade_journal.jsonl" "$OUT/nse/"
cp -f "$LOCAL/live_capture/shadow_validation/nse/of_paper_ledger_v1/"* "$OUT/nse/of_paper_ledger_v1/" 2>/dev/null || true
cp -f "$LOCAL/live_capture/shadow_validation/crypto/trade_plan_overlay_v1/trade_plan_outcomes.jsonl" "$OUT/crypto/"
cp -f "$LOCAL/live_capture/shadow_validation/crypto/trade_plan_overlay_v1/of_trade_outcomes.jsonl" "$OUT/crypto/"
wc -l \
  "$LOCAL/live_capture/shadow_validation/nse/rp608_nse_trade_journal.jsonl" \
  "$LOCAL/live_capture/shadow_validation/nse/of_paper_ledger_v1/of_trade_outcomes.jsonl" \
  "$LOCAL/live_capture/shadow_validation/crypto/trade_plan_overlay_v1/trade_plan_outcomes.jsonl" \
  "$LOCAL/live_capture/shadow_validation/crypto/trade_plan_overlay_v1/of_trade_outcomes.jsonl"
python3 scripts/research/rp608_econ_edge_session_export.py --day "$DAY" --root . --out-dir artifacts/rp608_econ_edge_diag
python3 scripts/research/rp608_econ_edge_wl_first_attribution.py --day "$DAY" --export-dir artifacts/rp608_econ_edge_diag
echo SYNC_EXPORT_DONE
