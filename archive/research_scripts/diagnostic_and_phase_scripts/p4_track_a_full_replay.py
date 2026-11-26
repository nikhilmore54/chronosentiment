#!/usr/bin/env python3
"""
P4 Track A — Full Historical Replay Validation
================================================
Applies the frozen Sprint 4 Track A operating baseline to the entire
available historical decision universe (chronological order).

Frozen policy (immutable — do NOT modify):
  direction    = SHORT
  action       = Watch
  target_rate  ∈ [0.314607, 0.327778)
  rank_score   ≥ 0.35

This script is VALIDATION ONLY. Results must not be used to tune
the policy. The Sprint 4 N=15 walk-forward artifact remains immutable.

Output: datasets/p4_track_a_replay.csv and .json
        logs/track_a_validation.log (via shell redirect)
"""

import csv
import json
import statistics
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
LEDGER_DIR = REPO_ROOT / "live_capture" / "ledger" / "entries"
OBS_DIR = REPO_ROOT / "time_machine" / "analysis" / "TIME009" / "observations"
OUT_CSV = REPO_ROOT / "datasets" / "p4_track_a_replay.csv"
OUT_JSON = REPO_ROOT / "datasets" / "p4_track_a_replay.json"

# ── Frozen policy ──────────────────────────────────────────────────────────────
DIRECTION = "SHORT"
ACTION = "Watch"
TARGET_RATE_LO = 0.314607
TARGET_RATE_HI = 0.327778
RANK_SCORE_MIN = 0.35

def load_json(path):
    with open(path) as f:
        return json.load(f)

def run():
    print("=== P4 Track A — Full Historical Replay Validation ===")
    print(f"Frozen policy: {DIRECTION} {ACTION} "
          f"target_rate∈[{TARGET_RATE_LO},{TARGET_RATE_HI}) "
          f"rank_score≥{RANK_SCORE_MIN}")
    print()

    # Load all COMPLETE observations
    print("Loading COMPLETE observations...")
    obs_by_id = {}
    for p in OBS_DIR.glob("TIME009-OBS-*.json"):
        try:
            o = load_json(p)
            if o.get("observation_status") == "COMPLETE" and o.get("realized_return") is not None:
                obs_by_id[o["decision_id"]] = o
        except Exception:
            pass
    print(f"  COMPLETE observations: {len(obs_by_id)}")

    # Load all ledger entries and apply frozen policy
    print("Loading ledger entries and applying frozen policy...")
    all_decisions = []
    qualifying = []

    for p in LEDGER_DIR.glob("*.json"):
        try:
            e = load_json(p)
            did = e.get("decision_id")
            if did not in obs_by_id:
                continue
            obs = obs_by_id[did]

            # Extract cohort date for chronological ordering
            # decision_id format: LIVE-005-YYYYMMDD-HHMM-TICKER_NS
            parts = did.split("-")
            cohort_date = parts[2] if len(parts) > 2 else "00000000"

            rec = {
                "decision_id": did,
                "cohort_date": cohort_date,
                "ticker": e.get("ticker"),
                "direction": e.get("direction"),
                "action": e.get("action"),
                "target_rate": e.get("target_rate") or 0.0,
                "rank_score": e.get("rank_score") or 0.0,
                "sample_size": e.get("sample_size") or 0,
                "evidence_class": e.get("evidence_class"),
                "certification_status": e.get("certification_status"),
                "reference_price": e.get("reference_price"),
                "adaptive_target": e.get("adaptive_target"),
                "adaptive_risk": e.get("adaptive_risk"),
                "realized_return": obs.get("realized_return"),
                "exit_reason": obs.get("exit_reason"),
                "target_reached": obs.get("target_reached"),
                "gap_through": obs.get("exit_reason", "").endswith("GAP_THROUGH"),
                "win": obs.get("realized_return", 0) > 0,
            }
            all_decisions.append(rec)

            # Apply frozen policy
            if (rec["direction"] == DIRECTION
                    and rec["action"] == ACTION
                    and TARGET_RATE_LO <= rec["target_rate"] < TARGET_RATE_HI
                    and rec["rank_score"] >= RANK_SCORE_MIN):
                qualifying.append(rec)

        except Exception as ex:
            print(f"  [warn] {p.name}: {ex}", file=sys.stderr)

    # Sort chronologically
    all_decisions.sort(key=lambda r: r["cohort_date"])
    qualifying.sort(key=lambda r: r["cohort_date"])

    print(f"  Total COMPLETE decisions: {len(all_decisions)}")
    print(f"  Qualifying under frozen policy: {len(qualifying)}")
    print()

    # ── Statistics ─────────────────────────────────────────────────────────────
    def stats(label, recs):
        if not recs:
            print(f"{label}: N=0")
            return
        rets = [r["realized_return"] for r in recs]
        wins = [r for r in recs if r["win"]]
        gaps = [r for r in recs if r["gap_through"]]
        risk_exits = [r for r in recs if r["exit_reason"] == "RISK"]
        exits = {}
        for r in recs:
            exits[r["exit_reason"]] = exits.get(r["exit_reason"], 0) + 1
        tickers = {}
        for r in recs:
            tickers[r["ticker"]] = tickers.get(r["ticker"], 0) + 1

        print(f"=== {label} ===")
        print(f"  N:          {len(recs)}")
        print(f"  Mean:       {statistics.mean(rets)*100:+.3f}%")
        print(f"  Median:     {statistics.median(rets)*100:+.3f}%")
        if len(rets) >= 2:
            print(f"  Stdev:      {statistics.stdev(rets)*100:.3f}%")
        print(f"  Min:        {min(rets)*100:+.3f}%")
        print(f"  Max:        {max(rets)*100:+.3f}%")
        print(f"  Win rate:   {len(wins)/len(recs):.1%} ({len(wins)}/{len(recs)})")
        print(f"  RISK exits: {len(risk_exits)}")
        print(f"  Gap-through:{len(gaps)}")
        print(f"  Exits:      {exits}")
        print(f"  Tickers:    {tickers}")
        print()

    # Cohort-date breakdown
    def cohort_breakdown(label, recs):
        by_cohort = {}
        for r in recs:
            by_cohort.setdefault(r["cohort_date"], []).append(r)
        print(f"=== {label} — by cohort date ===")
        for cohort in sorted(by_cohort):
            cohort_recs = by_cohort[cohort]
            rets = [r["realized_return"] for r in cohort_recs]
            wins = [r for r in cohort_recs if r["win"]]
            exits = [r["exit_reason"] for r in cohort_recs]
            print(f"  {cohort}: N={len(cohort_recs)}, wins={len(wins)}, "
                  f"mean={statistics.mean(rets)*100:+.3f}%, exits={exits}")
        print()

    # Sprint 4 walk-forward reference (immutable)
    print("=== SPRINT 4 WALK-FORWARD REFERENCE (immutable) ===")
    print("  N=15, Mean=+0.696%, Median=+0.657%, Win=73.3%, RISK=1, Gap=0")
    print()

    stats("Track A Full Replay — qualifying decisions", qualifying)
    cohort_breakdown("Track A Full Replay", qualifying)

    # Sentinel value check
    sentinel_count = sum(1 for r in qualifying if abs(r["realized_return"] - 1.0) < 0.0001)
    if sentinel_count > 0:
        print(f"[WARN] {sentinel_count} decisions have realized_return=+1.0000 (sentinel/target-reached). "
              f"These inflate mean. Median is the correct comparison point.")
        non_sentinel = [r for r in qualifying if abs(r["realized_return"] - 1.0) >= 0.0001]
        if non_sentinel:
            ns_rets = [r["realized_return"] for r in non_sentinel]
            ns_wins = [r for r in non_sentinel if r["win"]]
            print(f"  Non-sentinel N={len(non_sentinel)}, "
                  f"Mean={statistics.mean(ns_rets)*100:+.3f}%, "
                  f"Median={statistics.median(ns_rets)*100:+.3f}%, "
                  f"Win={len(ns_wins)/len(non_sentinel):.1%}")
        print()

    # Write outputs
    OUT_CSV.parent.mkdir(parents=True, exist_ok=True)
    if qualifying:
        fieldnames = list(qualifying[0].keys())
        with open(OUT_CSV, "w", newline="") as f:
            writer = csv.DictWriter(f, fieldnames=fieldnames)
            writer.writeheader()
            writer.writerows(qualifying)
        print(f"CSV written: {OUT_CSV}")

    with open(OUT_JSON, "w") as f:
        json.dump(qualifying, f, indent=2, default=str)
    print(f"JSON written: {OUT_JSON}")

if __name__ == "__main__":
    run()