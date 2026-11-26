#!/usr/bin/env bash
# Install LCUtest cron for armed Prospective V1 scheduled starts.
# Run on the Pi from the repo root AFTER arming.
set -euo pipefail
ROOT="${1:-/home/ultraspeed/ChronoSentiment_MEGA_FINAL}"
cd "$ROOT"
test -f scripts/run_rp608_prospective_v1_scheduled_start.py
CRON_LINE="*/5 * * * * cd ${ROOT} && /usr/bin/python3 scripts/run_rp608_prospective_v1_scheduled_start.py run >> artifacts/rp608_regime_breakout/prospective_v1/scheduled_start.cron.log 2>&1"
(crontab -l 2>/dev/null | grep -v 'run_rp608_prospective_v1_scheduled_start' || true; echo "$CRON_LINE") | crontab -
echo "Installed:"
crontab -l | grep scheduled_start || true
