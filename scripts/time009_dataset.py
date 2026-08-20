#!/usr/bin/env python3
"""
TIME-009 Dataset Aggregator — time009_dataset.py

Reads all TIME009-OBS-*.json artifacts from the observations directory and
produces a flat prospective_evidence.csv containing only COMPLETE observations.

PENDING observations are excluded from the CSV (they have no outcome yet).
The CSV is the input to the TIME-010 analysis script.

Governance invariants:
  - Only COMPLETE observations are included (observation_status == "COMPLETE").
  - No outcome fields are synthesised or imputed.
  - The script is read-only with respect to the observation artifacts.
  - Columns are fixed and pre-specified (no dynamic column selection).

Usage:
    python3 scripts/time009_dataset.py \
        --observations time_machine/analysis/TIME009/observations \
        --output       time_machine/analysis/TIME009/prospective_evidence.csv

Output columns (pre-specified, TIME-010 contract):
    observation_id, decision_id, cohort_date, ticker, direction, action,
    evidence_class, certification_status, degradation_level,
    reference_price, adaptive_target, adaptive_risk, adaptive_horizon_sessions,
    vol_regime, volume_regime, sample_size, target_rate, rank_score,
    horizon_sessions, n_bars_after_t0, n_bars_in_horizon,
    exit_reason, sessions_to_outcome,
    target_reached, risk_reached, horizon_reached, ambiguous,
    actual_mfe, actual_mae, realized_return,
    eligible_for_primary_comparison,
    observed_at, producer
"""

import argparse
import csv
import json
import os
import sys
from pathlib import Path

# ── Pre-specified output columns (TIME-010 contract) ─────────────────────────

OUTPUT_COLUMNS = [
    "observation_id",
    "decision_id",
    "cohort_date",
    "ticker",
    "direction",
    "action",
    "evidence_class",
    "certification_status",
    "degradation_level",
    "reference_price",
    "adaptive_target",
    "adaptive_risk",
    "adaptive_horizon_sessions",
    "vol_regime",
    "volume_regime",
    "sample_size",
    "target_rate",
    "rank_score",
    "horizon_sessions",
    "n_bars_after_t0",
    "n_bars_in_horizon",
    "exit_reason",
    "sessions_to_outcome",
    "target_reached",
    "risk_reached",
    "horizon_reached",
    "ambiguous",
    "actual_mfe",
    "actual_mae",
    "realized_return",
    "eligible_for_primary_comparison",
    "observed_at",
    "producer",
]


def parse_args():
    p = argparse.ArgumentParser(description="TIME-009 dataset aggregator")
    p.add_argument(
        "--observations",
        default="time_machine/analysis/TIME009/observations",
        help="Directory containing TIME009-OBS-*.json artifacts",
    )
    p.add_argument(
        "--output",
        default="time_machine/analysis/TIME009/prospective_evidence.csv",
        help="Output CSV path",
    )
    return p.parse_args()


def load_observations(obs_dir: Path):
    """Load all TIME009-OBS-*.json files. Returns (complete, pending, errors)."""
    complete = []
    pending = []
    errors = []

    if not obs_dir.exists():
        print(f"[time009_dataset] observations dir not found: {obs_dir}", file=sys.stderr)
        return complete, pending, errors

    for fname in sorted(obs_dir.iterdir()):
        if fname.suffix != ".json" or fname.name == "latest_run.json":
            continue
        try:
            with open(fname) as f:
                obs = json.load(f)
        except Exception as e:
            errors.append((fname.name, str(e)))
            continue

        status = obs.get("observation_status", "")
        if status == "COMPLETE":
            complete.append(obs)
        elif status == "PENDING":
            pending.append(obs)
        else:
            errors.append((fname.name, f"unknown observation_status={status!r}"))

    return complete, pending, errors


def extract_row(obs: dict) -> dict:
    """Extract the pre-specified columns from a COMPLETE observation."""
    row = {}
    for col in OUTPUT_COLUMNS:
        val = obs.get(col)
        # Normalise None → empty string for CSV clarity.
        row[col] = "" if val is None else val
    return row


def main():
    args = parse_args()
    obs_dir = Path(args.observations)
    output_path = Path(args.output)

    print("[time009_dataset] TIME-009 Dataset Aggregator")
    print("[time009_dataset] ==============================")
    print(f"[time009_dataset] observations: {obs_dir}")
    print(f"[time009_dataset] output:       {output_path}")

    complete, pending, errors = load_observations(obs_dir)

    print(f"[time009_dataset] n_complete={len(complete)}")
    print(f"[time009_dataset] n_pending={len(pending)}")
    print(f"[time009_dataset] n_errors={len(errors)}")

    for fname, err in errors:
        print(f"[time009_dataset] ERROR {fname}: {err}", file=sys.stderr)

    if not complete:
        print("[time009_dataset] result=SKIP reason=no_complete_observations")
        print("[time009_dataset] (PENDING observations will become COMPLETE as horizons elapse)")
        return 0

    # Sort by cohort_date, then ticker for deterministic output.
    complete.sort(key=lambda o: (o.get("cohort_date", ""), o.get("ticker", "")))

    # Write CSV.
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with open(output_path, "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=OUTPUT_COLUMNS)
        writer.writeheader()
        for obs in complete:
            writer.writerow(extract_row(obs))

    print(f"[time009_dataset] result=OK")
    print(f"[time009_dataset] n_rows_written={len(complete)}")
    print(f"[time009_dataset] output={output_path}")

    # Summary by evidence_class × certification_status.
    from collections import Counter
    counts = Counter(
        (o.get("evidence_class", "?"), o.get("certification_status", "?"))
        for o in complete
    )
    print("[time009_dataset] breakdown (evidence_class × certification_status):")
    for (ec, cs), n in sorted(counts.items()):
        print(f"[time009_dataset]   {ec} × {cs}: {n}")

    eligible = sum(1 for o in complete if o.get("eligible_for_primary_comparison"))
    print(f"[time009_dataset] n_eligible_for_primary_comparison={eligible}")

    return 0


if __name__ == "__main__":
    sys.exit(main())