#!/usr/bin/env bash
# Detect which (set, seed) pairs are still missing from benchmarks/e1/
# Expected pairs: setA-01..setA-20 with seeds 42,43,44

set -euo pipefail

BENCHMARK_DIR="benchmarks/e1"

# Generate the full list of expected identifiers
EXPECTED=()
for i in {01..20}; do
  for seed in 42 43 44; do
    EXPECTED+=("setA-${i}_${seed}")
  done
done

MISSING=()
for id in "${EXPECTED[@]}"; do
  json_file="$BENCHMARK_DIR/${id}.json"
  srpaths_file="$BENCHMARK_DIR/${id}-srpaths.json"
  if [[ ! -f "$json_file" ]] || [[ ! -f "$srpaths_file" ]]; then
    MISSING+=("$id")
  fi
done

# Output missing identifiers, one per line (caller can space‑separate)
for id in "${MISSING[@]}"; do
  echo "$id"
done
