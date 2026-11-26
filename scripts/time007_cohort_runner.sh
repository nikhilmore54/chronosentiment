#!/usr/bin/env bash
# TIME-007 — Multi-Timestamp Cohort Runner
#
# Runs the frozen TIME-002→TIME-006 pipeline for each pre-specified T value
# in the historical grid. No algorithm changes. No outcome-based selection.
# Timestamps are derived mechanically from the RELIANCE.NS cache at 20-session
# intervals going back from T1 (2026-08-14).
#
# Governing rule:
#   TIME-007 may increase the quantity of evidence, but may not change
#   the experiment that generates the evidence.
#
# Pre-specified timestamp grid (derived before looking at outcomes):
#   T1  2026-08-14T10:15:00Z  bars_before=1239 bars_after=15  (existing — skip)
#   T2  2026-07-17T03:45:00Z  bars_before=1219 bars_after=35
#   T3  2026-06-19T03:45:00Z  bars_before=1199 bars_after=55
#   T4  2026-05-22T03:45:00Z  bars_before=1179 bars_after=75
#   T5  2026-04-24T03:45:00Z  bars_before=1159 bars_after=95
#   T6  2026-03-23T03:45:00Z  bars_before=1139 bars_after=115
#
# Usage:
#   bash scripts/time007_cohort_runner.sh
#   bash scripts/time007_cohort_runner.sh --dry-run
#   bash scripts/time007_cohort_runner.sh --cohorts T2,T3

set -euo pipefail

WORKSPACE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$WORKSPACE_ROOT"

CACHE_DIR="product_validation/CS-P-006/snapshot/20260814T183851Z_100instrument/yahoo_cache"
UNIVERSE="datasets/universes/coralys_102_v1.json"
EVIDENCE_STORE="datasets/recommendation/historical"
TIME_MACHINE_DIR="time_machine"

DRY_RUN=false
COHORT_FILTER=""

for arg in "$@"; do
    case "$arg" in
        --dry-run) DRY_RUN=true ;;
        --cohorts=*) COHORT_FILTER="${arg#--cohorts=}" ;;
    esac
done

# ── Pre-specified timestamp grid ──────────────────────────────────────────────
# Format: "LABEL|ISO_TIMESTAMP"
# T1 is the existing frozen cohort — included for completeness but skipped if
# evidence already exists.
declare -a COHORT_GRID=(
    "T1|2026-08-14T10:15:00Z"
    "T2|2026-07-17T03:45:00Z"
    "T3|2026-06-19T03:45:00Z"
    "T4|2026-05-22T03:45:00Z"
    "T5|2026-04-24T03:45:00Z"
    "T6|2026-03-23T03:45:00Z"
)

echo "[time007] TIME-007 — Multi-Timestamp Cohort Runner"
echo "[time007] =========================================="
echo "[time007] workspace: $WORKSPACE_ROOT"
echo "[time007] cache:     $CACHE_DIR"
echo "[time007] universe:  $UNIVERSE"
echo "[time007] dry_run:   $DRY_RUN"
echo "[time007] cohorts:   ${COHORT_FILTER:-ALL}"
echo ""

# ── Build all binaries once ───────────────────────────────────────────────────
echo "[time007] Building binaries..."
cargo build -p chronosentiment_adapter \
    --bin time002_reconstruct \
    --bin time003_replay \
    --bin time004_ledger \
    --bin time005_observe \
    --bin time006_evidence_dataset \
    2>&1 | tail -3
echo "[time007] Build complete."
echo ""

# ── Run pipeline for each cohort ──────────────────────────────────────────────
N_COHORTS=0
N_SKIPPED=0
N_FAILED=0

for entry in "${COHORT_GRID[@]}"; do
    LABEL="${entry%%|*}"
    AS_OF="${entry##*|}"

    # Apply cohort filter if specified
    if [[ -n "$COHORT_FILTER" ]]; then
        if [[ "$COHORT_FILTER" != *"$LABEL"* ]]; then
            continue
        fi
    fi

    # Cohort-namespaced output directories under time_machine/cohorts/<LABEL>/
    COHORT_DIR="$TIME_MACHINE_DIR/cohorts/$LABEL"
    RECON_DIR="$COHORT_DIR/reconstructions"
    DECISIONS_DIR="$COHORT_DIR/decisions"
    LEDGER_DIR="$COHORT_DIR/ledger"
    OBS_DIR="$COHORT_DIR/observations"
    EVIDENCE_DIR="$COHORT_DIR/evidence"

    echo "[time007] ── Cohort $LABEL: as_of=$AS_OF ──────────────────────────────"

    # Check if evidence already exists (idempotency).
    # T1 lives in time_machine/evidence/ (original location); T2-T6 in cohort dirs.
    CHECK_EVIDENCE_FILE="$EVIDENCE_DIR/latest_run.json"
    if [[ "$LABEL" == "T1" ]]; then
        CHECK_EVIDENCE_FILE="$TIME_MACHINE_DIR/evidence/latest_run.json"
    fi

    if [[ -f "$CHECK_EVIDENCE_FILE" ]]; then
        EXISTING_JOINED=$(python3 -c "
import json
try:
    d = json.load(open('$CHECK_EVIDENCE_FILE'))
    print(d.get('n_joined', 0))
except:
    print(0)
" 2>/dev/null || echo "0")
        if [[ "$EXISTING_JOINED" -gt 0 ]]; then
            echo "[time007] SKIP $LABEL — evidence already exists (n_joined=$EXISTING_JOINED)"
            N_SKIPPED=$((N_SKIPPED + 1))
            continue
        fi
    fi

    if [[ "$DRY_RUN" == "true" ]]; then
        echo "[time007] DRY-RUN: would run TIME-002→TIME-006 for $LABEL as_of=$AS_OF"
        continue
    fi

    # Create output directories
    mkdir -p "$RECON_DIR" "$DECISIONS_DIR" "$LEDGER_DIR/entries" "$LEDGER_DIR/audit" "$OBS_DIR" "$EVIDENCE_DIR"

    # ── Step 1: TIME-002 Reconstruct ──────────────────────────────────────────
    echo "[time007]   [1/5] TIME-002 reconstruct as_of=$AS_OF"
    cargo run -p chronosentiment_adapter --bin time002_reconstruct -- \
        --as-of     "$AS_OF" \
        --universe  "$UNIVERSE" \
        --cache-dir "$CACHE_DIR" \
        --output    "$RECON_DIR" \
        2>&1 | grep -E "^\[time002\].*(complete|error|total|result)" | head -5 || true
    # Check artifact exists (not exit code — grep may return 1 if no match)
    RECON_FILE=$(ls -t "$RECON_DIR"/TIME002-*.json 2>/dev/null | head -1 || true)
    if [[ -z "$RECON_FILE" ]]; then
        echo "[time007]   ERROR: no reconstruction artifact found in $RECON_DIR"
        N_FAILED=$((N_FAILED + 1))
        continue
    fi
    echo "[time007]   reconstruction: $RECON_FILE"

    # ── Step 2: TIME-003 Replay ───────────────────────────────────────────────
    echo "[time007]   [2/5] TIME-003 replay"
    cargo run -p chronosentiment_adapter --bin time003_replay -- \
        --reconstruction "$RECON_FILE" \
        --evidence       "$EVIDENCE_STORE" \
        --output         "$DECISIONS_DIR" \
        2>&1 | grep -E "^\[time003\].*(decided|error|total|result|accounting)" | head -5 || true
    # Check artifact exists
    DECISION_FILE=$(ls -t "$DECISIONS_DIR"/TIME003-*.json 2>/dev/null | head -1 || true)
    if [[ -z "$DECISION_FILE" ]]; then
        echo "[time007]   ERROR: no decision artifact found in $DECISIONS_DIR"
        N_FAILED=$((N_FAILED + 1))
        continue
    fi
    echo "[time007]   decision: $DECISION_FILE"

    # ── Step 3: TIME-004 Ledger ───────────────────────────────────────────────
    echo "[time007]   [3/5] TIME-004 ledger"
    cargo run -p chronosentiment_adapter --bin time004_ledger -- \
        --decisions "$DECISION_FILE" \
        --ledger    "$LEDGER_DIR/entries" \
        --audit     "$LEDGER_DIR/audit" \
        2>&1 | grep -E "^\[time004\].*(admitted|error|total|result|accounting)" | head -5 || true
    # Check ledger entries exist
    N_LEDGER_ENTRIES=$(ls "$LEDGER_DIR/entries"/*.json 2>/dev/null | wc -l | tr -d ' ' || echo "0")
    if [[ "$N_LEDGER_ENTRIES" -eq 0 ]]; then
        echo "[time007]   ERROR: no ledger entries found in $LEDGER_DIR/entries"
        N_FAILED=$((N_FAILED + 1))
        continue
    fi
    echo "[time007]   ledger entries: $N_LEDGER_ENTRIES"

    # ── Step 4: TIME-005 Observe ──────────────────────────────────────────────
    echo "[time007]   [4/5] TIME-005 observe"
    cargo run -p chronosentiment_adapter --bin time005_observe -- \
        --ledger "$LEDGER_DIR/entries" \
        --cache  "$CACHE_DIR" \
        --output "$OBS_DIR" \
        2>&1 | grep -E "^\[time005\].*(accounting|result|n_observed)" | head -5 || true
    # Check observations exist
    N_OBS=$(ls "$OBS_DIR"/TIME005-*.json 2>/dev/null | wc -l | tr -d ' ' || echo "0")
    if [[ "$N_OBS" -eq 0 ]]; then
        echo "[time007]   ERROR: no observations found in $OBS_DIR"
        N_FAILED=$((N_FAILED + 1))
        continue
    fi
    echo "[time007]   observations: $N_OBS"

    # ── Step 5: TIME-006 Evidence Dataset ────────────────────────────────────
    echo "[time007]   [5/5] TIME-006 evidence dataset"
    cargo run -p chronosentiment_adapter --bin time006_evidence_dataset -- \
        --ledger       "$LEDGER_DIR/entries" \
        --observations "$OBS_DIR" \
        --output       "$EVIDENCE_DIR" \
        2>&1 | grep -E "^\[time006\].*(accounting|result|n_joined)" | head -5 || true
    # Check evidence CSV exists
    if [[ ! -f "$EVIDENCE_DIR/evidence_dataset.csv" ]]; then
        echo "[time007]   ERROR: no evidence CSV found in $EVIDENCE_DIR"
        N_FAILED=$((N_FAILED + 1))
        continue
    fi

    N_COHORTS=$((N_COHORTS + 1))
    echo "[time007]   Cohort $LABEL COMPLETE"
    echo ""
done

# ── Aggregate all cohort evidence CSVs ───────────────────────────────────────
echo "[time007] ── Aggregating cohort evidence datasets ──────────────────────"
AGGREGATE_CSV="$TIME_MACHINE_DIR/cohorts/aggregate_evidence.csv"
mkdir -p "$TIME_MACHINE_DIR/cohorts"

HEADER_WRITTEN=false
> "$AGGREGATE_CSV"

for entry in "${COHORT_GRID[@]}"; do
    LABEL="${entry%%|*}"
    AS_OF="${entry##*|}"

    # T1 lives in the original location
    if [[ "$LABEL" == "T1" ]]; then
        COHORT_CSV="$TIME_MACHINE_DIR/evidence/evidence_dataset.csv"
    else
        COHORT_CSV="$TIME_MACHINE_DIR/cohorts/$LABEL/evidence/evidence_dataset.csv"
    fi

    if [[ ! -f "$COHORT_CSV" ]]; then
        echo "[time007] WARN: no CSV for $LABEL at $COHORT_CSV"
        continue
    fi

    if [[ "$HEADER_WRITTEN" == "false" ]]; then
        # Write header with cohort_label and as_of prepended
        echo -n "cohort_label,as_of_cohort," > "$AGGREGATE_CSV"
        head -1 "$COHORT_CSV" >> "$AGGREGATE_CSV"
        HEADER_WRITTEN=true
    fi

    # Append data rows with cohort label and as_of prepended
    tail -n +2 "$COHORT_CSV" | sed "s/^/$LABEL,$AS_OF,/" >> "$AGGREGATE_CSV"
    N_ROWS=$(tail -n +2 "$COHORT_CSV" | wc -l | tr -d ' ')
    echo "[time007] aggregated $LABEL ($AS_OF): $N_ROWS rows"
done

TOTAL_ROWS=$(tail -n +2 "$AGGREGATE_CSV" 2>/dev/null | wc -l | tr -d ' ' || echo "0")
echo "[time007] aggregate_evidence.csv: $TOTAL_ROWS data rows"
echo "[time007] written: $AGGREGATE_CSV"
echo ""

# ── Summary ───────────────────────────────────────────────────────────────────
echo "[time007] ── Summary ──────────────────────────────────────────────────"
echo "[time007] n_cohorts_run=$N_COHORTS"
echo "[time007] n_cohorts_skipped=$N_SKIPPED"
echo "[time007] n_cohorts_failed=$N_FAILED"
echo "[time007] aggregate_rows=$TOTAL_ROWS"
echo "[time007] result=OK"