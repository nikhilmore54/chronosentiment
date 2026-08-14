#!/usr/bin/env bash
# Install daily CS-P-003 tick at 16:00 local (after NSE close ~15:30 IST).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
DEST="$HOME/Library/LaunchAgents/com.chronosentiment.csp003-forward-tick.plist"
mkdir -p "$HOME/Library/LaunchAgents"
sed "s|REPO_ROOT|$ROOT|g" \
  "$ROOT/product_validation/forward_unfrozen_dev/com.chronosentiment.csp003-forward-tick.plist.template" \
  >"$DEST"
launchctl unload "$DEST" 2>/dev/null || true
launchctl load "$DEST"
echo "Loaded $DEST (daily 16:00 local). First tick: ./run_csp003_forward_tick.sh"
