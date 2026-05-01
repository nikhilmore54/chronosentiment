#!/usr/bin/env bash
# Copy deterministic replay inputs into analysis/baselines/baseline_v1/ (read-only snapshot).
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

DEST="analysis/baselines/baseline_v1"
SRC_GRID="analysis/awr_grid"
REGISTRY_SRC="data/experiments.jsonl"

mkdir -p "${DEST}/diag_logs"

if [[ -f "${REGISTRY_SRC}" ]]; then
  cp "${REGISTRY_SRC}" "${DEST}/experiments.jsonl"
  echo "Copied ${REGISTRY_SRC} -> ${DEST}/experiments.jsonl"
else
  echo "Warning: ${REGISTRY_SRC} not found — baseline registry not updated." >&2
fi

shopt -s nullglob
files=( "${SRC_GRID}"/diag_final_distribution_limit400_offset*.log )
if [[ ${#files[@]} -eq 0 ]]; then
  echo "Warning: no ${SRC_GRID}/diag_final_distribution_limit400_offset*.log — diag_logs/ unchanged." >&2
else
  cp "${files[@]}" "${DEST}/diag_logs/"
  echo "Copied ${#files[@]} diagnostic log(s) -> ${DEST}/diag_logs/"
fi

echo "Done. Optionally edit ${DEST}/metadata.json summary to match this snapshot."
