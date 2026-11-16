#!/bin/bash
set -euo pipefail
LOCAL="/Users/nikhil/ChronoSentiment_MEGA_FINAL"
DAY="2026-11-05"
J="$LOCAL/live_capture/shadow_validation/nse/rp608_nse_trade_journal.jsonl"
LOG="$LOCAL/artifacts/rp608_econ_edge_diag/watch_export_${DAY}.log"
mkdir -p "$LOCAL/artifacts/rp608_econ_edge_diag"
cd "$LOCAL"
{
  echo "=== $(date) watching for fresh journals ==="
  for i in $(seq 1 360); do
    lines=0
    [[ -f "$J" ]] && lines=$(wc -l < "$J" | tr -d ' ')
    echo "poll $i lines=$lines"
    if [[ "$lines" -gt 500 ]]; then
      echo FRESH
      python3 scripts/research/rp608_econ_edge_session_export.py --day "$DAY" --out-dir artifacts/rp608_econ_edge_diag
      python3 scripts/research/rp608_econ_edge_wl_first_attribution.py --day "$DAY" --export-dir artifacts/rp608_econ_edge_diag
      echo WATCH_EXPORT_DONE
      exit 0
    fi
    sleep 5
  done
  echo TIMEOUT_NO_JOURNALS
  exit 1
} >>"$LOG" 2>&1
