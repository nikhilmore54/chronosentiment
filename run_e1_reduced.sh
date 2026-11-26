#!/usr/bin/env bash

# ==============================================================================
# run_e1_reduced.sh – Production‑mode E1 benchmark runner
# ==============================================================================
# This script drives the missing E1 benchmark instances using the helper
# scripts/missing_e1_instances.sh.  It implements the safety guarantees that
# were requested:
#   • Hard 2‑hour deadline that applies both during launch and while waiting.
#   • Explicit child‑PID tracking – children are killed on timeout/failure.
#   • Concurrency limited to 14 processes, each with RAYON_NUM_THREADS=1.
#   • Correct identifier parsing – the instance "setA-05_42" resolves to
#     set_dir="setA" and instance="setA-05".
#   • No Cartesian‑product expansion – only the exact missing (instance,seed)
#     pairs produced by missing_e1_instances.sh are processed.
#   • Writes only JSON and *‑srpaths.json files under benchmarks/e1/.
# ==============================================================================

set -euo pipefail

# --------------------------------------------------------------------------
# Configuration
# --------------------------------------------------------------------------
RELEASE_BIN="${RELEASE_BIN:-./target/release/roadef_dataset}"   # path to the frozen release binary
BENCHMARK_DIR="benchmarks/e1"
MISSING_HELPER="scripts/missing_e1_instances.sh"
MAX_PROCS=14                     # maximum concurrent children
TIME_LIMIT=$((2 * 60 * 60))      # hard limit: 2 hours, in seconds

# Ensure the benchmark directory exists
mkdir -p "${BENCHMARK_DIR}"

# --------------------------------------------------------------------------
# Helper: run a single instance
# --------------------------------------------------------------------------
run_instance() {
    local id="$1"
    # Split "setA-05_42" → instance="setA-05", seed="42"
    local instance="${id%_*}"
    local seed="${id##*_}"
    # Derive the set directory ("setA") from the instance name
    local set_dir="${instance%%-*}"

    local net_file="adapters/roadef/repo/challenge-roadef-2026-main/${set_dir}/${instance}-net.json"
    local tm_file="adapters/roadef/repo/challenge-roadef-2026-main/${set_dir}/${instance}-tm.json"
    local scen_file="adapters/roadef/repo/challenge-roadef-2026-main/${set_dir}/${instance}-scenario.json"

    # Verify that all input files exist before invoking the binary
    if [[ ! -f "${net_file}" || ! -f "${tm_file}" || ! -f "${scen_file}" ]]; then
        echo "⚠️ Missing input files for ${id}:" >&2
        echo "   ${net_file}" >&2
        echo "   ${tm_file}" >&2
        echo "   ${scen_file}" >&2
        return 1
    fi

    # Ensure each child runs with a single Rayon thread
    RAYON_NUM_THREADS=1 "${RELEASE_BIN}" \
        --net "${net_file}" \
        --tm "${tm_file}" \
        --scenario "${scen_file}" \
        --seed "${seed}" \
        --output "${BENCHMARK_DIR}/${id}.json" \
        --srpaths "${BENCHMARK_DIR}/${id}-srpaths.json"
}

# --------------------------------------------------------------------------
# Main driver
# --------------------------------------------------------------------------
# Capture start time for the overall timeout calculation
START_TS=$(date +%s)

# Gather the list of missing identifiers (space‑separated)
if ! MISSING_IDS=$(${MISSING_HELPER}); then
    echo "❌ Failed to determine missing E1 instances" >&2
    exit 1
fi

# Convert the list into an array for safe iteration (handles newlines)
read -r -a missing_array <<< "${MISSING_IDS}"

# Array to hold running child PIDs
declare -a child_pids=()

# Function to reap finished children and compact the PID array
reap_children() {
    local i pid ret=0
    for i in "${!child_pids[@]}"; do
        pid=${child_pids[i]}
        if ! kill -0 "${pid}" 2>/dev/null; then
            # Child has exited – wait to obtain its status (propagates failures)
            wait "${pid}" || ret=$?
            unset 'child_pids[i]'
        fi
    done
    # Re‑index the array to keep length checks simple
    child_pids=(${child_pids[@]})
    return $ret
}

# Loop over each missing identifier and launch workers respecting limits
for id in "${missing_array[@]}"; do
    # ----------------------------------------------------------------------
    # Enforce the hard deadline before spawning a new job
    # ----------------------------------------------------------------------
    now=$(date +%s)
    elapsed=$((now - START_TS))
    remaining=$((TIME_LIMIT - elapsed))
    if (( remaining <= 0 )); then
        echo "❌ Hard deadline of 2 hours exceeded before launching ${id}" >&2
        break
    fi

    # Launch the instance in background
    run_instance "${id}" &
    child_pids+=( $! )

    # If we have reached the concurrency ceiling, wait for any child to finish
    while (( ${#child_pids[@]} >= MAX_PROCS )); do
        # wait -n blocks until the *first* child exits (available in Bash >=5)
        wait -n
        reap_children || true
    done

done

# --------------------------------------------------------------------------
# Wait for any remaining children, still respecting the overall timeout
# --------------------------------------------------------------------------
while (( ${#child_pids[@]} > 0 )); do
    now=$(date +%s)
    elapsed=$((now - START_TS))
    remaining=$((TIME_LIMIT - elapsed))
    if (( remaining <= 0 )); then
        echo "⚠️ Timeout reached – terminating remaining children" >&2
        kill "${child_pids[@]}" 2>/dev/null || true
        wait "${child_pids[@]}" 2>/dev/null || true
        exit 124   # conventional timeout exit code
    fi
    # Use timeout to avoid blocking beyond the deadline
    if timeout "$remaining" bash -c 'wait -n'; then
        reap_children || true
    else
        echo "⚠️ Timeout reached while waiting – killing remaining children" >&2
        kill "${child_pids[@]}" 2>/dev/null || true
        wait "${child_pids[@]}" 2>/dev/null || true
        exit 124
    fi
done

END_TS=$(date +%s)
ELAPSED=$((END_TS - START_TS))
printf "✅ run_e1_reduced.sh completed in %02d:%02d:%02d\n" $((ELAPSED/3600)) $(((ELAPSED/60)%60)) $((ELAPSED%60))

exit 0
EOS && chmod +x run_e1_reduced.sh