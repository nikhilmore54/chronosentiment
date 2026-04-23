#!/usr/bin/env bash
set -euo pipefail

# Deterministic early-exit distribution for live_engine pipeline.
# Keeps bootstrap and edge debug enabled on both producer and consumer.
bash -lc 'GA_BOOTSTRAP=1 EDGE_DEBUG=1 python3 scripts/mock_streamer.py | GA_BOOTSTRAP=1 EDGE_DEBUG=1 cargo run --example live_engine' 2>&1 \
  | awk '
BEGIN { total=0 }
/\[EARLY_EXIT\]/ {
  total++;
  for (i=1; i<=NF; i++) {
    if ($i ~ /^reason=/) {
      r=$i;
      sub(/^reason=/,"",r);
      counts[r]++;
    }
  }
}
END {
  print "EARLY_EXIT_TOTAL=" total;
  for (k in counts) {
    printf "%d %s\n", counts[k], k;
  }
}'
