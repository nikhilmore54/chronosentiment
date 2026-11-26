#!/usr/bin/env bash
# RP-608 — run a research command on the Pi, then pull named artifacts.
# Does not copy credentials. Baseline frozen. Authority 0%.
set -euo pipefail

PI="${PI:-ultraspeed@192.168.0.9}"
REPO="${REPO:-/home/ultraspeed/ChronoSentiment_MEGA_FINAL}"
LOCAL="${LOCAL:-$(cd "$(dirname "$0")/../.." && pwd)}"
OUT_REL="${OUT_REL:-artifacts/rp608_econ_edge_diag}"

usage() {
  cat <<'EOF'
Usage:
  rp608_pi_research_run.sh push <local-rel-path> [...]
  rp608_pi_research_run.sh run  -- <remote command...>
  rp608_pi_research_run.sh pull <remote-rel-glob-or-file> [...]
  rp608_pi_research_run.sh kite-nse-a-join   # push script + exports, run join, pull artifacts

Env: PI, REPO, LOCAL, OUT_REL
EOF
}

cmd="${1:-}"
shift || true

case "$cmd" in
  push)
    [[ $# -ge 1 ]] || { usage; exit 2; }
    for f in "$@"; do
      rsync -az "$LOCAL/$f" "$PI:$REPO/$(dirname "$f")/"
    done
    ;;
  run)
    [[ "${1:-}" == "--" ]] && shift
    [[ $# -ge 1 ]] || { usage; exit 2; }
    ssh -o BatchMode=yes -o ConnectTimeout=20 "$PI" "cd $REPO && $*"
    ;;
  pull)
    [[ $# -ge 1 ]] || { usage; exit 2; }
    mkdir -p "$LOCAL/$OUT_REL"
    for f in "$@"; do
      rsync -az "$PI:$REPO/$f" "$LOCAL/$(dirname "$f")/"
    done
    ;;
  kite-nse-a-join)
    mkdir -p "$LOCAL/$OUT_REL"
    rsync -az "$LOCAL/scripts/research/rp608_wl_kite_nse_decision_time_bars.py" \
      "$PI:$REPO/scripts/research/"
    rsync -az \
      "$LOCAL/$OUT_REL"/session_export_2026-11-0{3,4,5}_nse_estimand_a_closed.jsonl \
      "$PI:$REPO/$OUT_REL/"
    ssh -o BatchMode=yes -o ConnectTimeout=20 "$PI" \
      "cd $REPO && python3 scripts/research/rp608_wl_kite_nse_decision_time_bars.py \
        --export-dir $OUT_REL --out-dir $OUT_REL"
    rsync -az \
      "$PI:$REPO/$OUT_REL/wl_kite_nse_decision_time_bars.md" \
      "$PI:$REPO/$OUT_REL/wl_kite_nse_decision_time_bars.json" \
      "$PI:$REPO/$OUT_REL/wl_nse_a_decision_time_features_kite_join.jsonl" \
      "$PI:$REPO/$OUT_REL/wl_kite_nse_15m_bars_cache.jsonl" \
      "$LOCAL/$OUT_REL/"
    echo "DONE artifacts in $LOCAL/$OUT_REL"
    ;;
  -h|--help|"")
    usage
    ;;
  *)
    echo "Unknown command: $cmd" >&2
    usage
    exit 2
    ;;
esac
