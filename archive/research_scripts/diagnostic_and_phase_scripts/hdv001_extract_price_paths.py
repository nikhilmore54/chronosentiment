#!/usr/bin/env python3
"""
HDV-001-C Price Path Extractor
================================
For every one of the 1,144 certified Coralys decisions, extract the
subsequent N NSE sessions from the frozen price cache, respecting the
decision-time information boundary.

Temporal rule (strict):
  The first eligible bar satisfies:  bar_date > decision_date_in_IST
  No bar from the decision date itself may enter the path.

  decision_time is stored as UTC (e.g. "2026-07-16T18:30:00+00:00").
  NSE closes at 15:30 IST = 10:00 UTC.  Coralys decisions are issued
  at 18:30 UTC = 00:00 IST next day, i.e. after the close of the
  decision date.  The decision_date_in_IST is therefore:
      decision_time_utc + 5h30m  -> date component

  First eligible bar: bar_date > decision_date_in_IST

Output:
  datasets/hdv001/hdv001_price_paths_v1.json
  datasets/hdv001/HDV_001_C_EXTRACTOR_REPORT.md

Schema per record:
  {
    "decision_id":      str,
    "instrument":       str,
    "direction":        "LONG" | "SHORT",
    "decision_time":    str (ISO-8601 UTC),
    "decision_date_ist": str (YYYY-MM-DD),
    "reference_price":  float,   # entry_price from dataset
    "target_price":     float,
    "stop_price":       float,
    "sessions_available": int,   # how many bars we found (may be < N_SESSIONS)
    "observation_status": "COMPLETE" | "MATURING",
    "sessions": [
      {
        "session":  int,   # 1-indexed
        "date":     str,
        "open":     float,
        "high":     float,
        "low":      float,
        "close":    float,
        "volume":   float
      },
      ...
    ]
  }
"""

import json
import sys
from datetime import date, datetime, timezone, timedelta
from pathlib import Path

# ── paths ─────────────────────────────────────────────────────────────────────
WORKSPACE    = Path(__file__).resolve().parent.parent
DATASET_PATH = WORKSPACE / "datasets" / "stop_research_dataset_v01.json"
CACHE_DIR    = WORKSPACE / "datasets" / "hdv001" / "hdv001_price_cache_v1"
OUTPUT_PATH  = WORKSPACE / "datasets" / "hdv001" / "hdv001_price_paths_v1.json"
REPORT_PATH  = WORKSPACE / "datasets" / "hdv001" / "HDV_001_C_EXTRACTOR_REPORT.md"

# ── constants ─────────────────────────────────────────────────────────────────
N_SESSIONS   = 10          # primary observation horizon
IST_OFFSET   = timedelta(hours=5, minutes=30)
REQUIRED_END = date(2026, 8, 13)   # last date in frozen cache required window

# ── helpers ───────────────────────────────────────────────────────────────────

def symbol_to_filename(symbol: str) -> str:
    """TCS.NS -> TCS_NS.json,  M&M.NS -> MANDM_NS.json"""
    return symbol.replace("&M", "ANDM").replace(".", "_") + ".json"

def load_cache(symbol: str) -> list[dict] | None:
    path = CACHE_DIR / symbol_to_filename(symbol)
    if not path.exists():
        return None
    with open(path) as f:
        data = json.load(f)
    return sorted(data["bars"], key=lambda b: b["date"])

def decision_date_ist(decision_time_str: str) -> date:
    """
    Parse ISO-8601 UTC timestamp and convert to IST date.
    e.g. "2026-07-16T18:30:00+00:00" -> date(2026, 7, 17)
    """
    dt_utc = datetime.fromisoformat(decision_time_str)
    if dt_utc.tzinfo is None:
        dt_utc = dt_utc.replace(tzinfo=timezone.utc)
    dt_ist = dt_utc + IST_OFFSET
    return dt_ist.date()

def extract_path(bars: list[dict], decision_date: date, n: int) -> list[dict]:
    """
    Return up to n bars where bar_date > decision_date, in chronological order.
    """
    eligible = [
        b for b in bars
        if date.fromisoformat(b["date"]) > decision_date
    ]
    return eligible[:n]

# ── main ──────────────────────────────────────────────────────────────────────

def main():
    print("=" * 70)
    print("HDV-001-C PRICE PATH EXTRACTOR")
    print("=" * 70)

    # load decisions
    with open(DATASET_PATH) as f:
        dataset = json.load(f)
    decisions = dataset["records"]
    print(f"Loaded {len(decisions)} decisions from dataset")

    # load all caches once
    cache_map: dict[str, list[dict]] = {}
    for rec in decisions:
        sym = rec["instrument"]
        if sym not in cache_map:
            bars = load_cache(sym)
            if bars is None:
                print(f"  WARN: no cache file for {sym}")
            cache_map[sym] = bars or []

    # extract paths
    results = []
    stats = {
        "total": 0,
        "complete": 0,
        "maturing": 0,
        "no_cache": 0,
        "zero_sessions": 0,
    }

    for rec in decisions:
        stats["total"] += 1
        sym          = rec["instrument"]
        decision_id  = rec["decision_id"]
        direction    = rec["direction"]
        decision_ts  = rec["decision_time"]
        ref_price    = rec["entry_price"]
        target_price = rec["target_price"]
        stop_price   = rec["stop_price"]

        bars = cache_map.get(sym)
        if not bars:
            stats["no_cache"] += 1
            results.append({
                "decision_id":       decision_id,
                "instrument":        sym,
                "direction":         direction,
                "decision_time":     decision_ts,
                "decision_date_ist": None,
                "reference_price":   ref_price,
                "target_price":      target_price,
                "stop_price":        stop_price,
                "sessions_available": 0,
                "observation_status": "NO_CACHE",
                "sessions":          [],
            })
            continue

        dec_date = decision_date_ist(decision_ts)
        path     = extract_path(bars, dec_date, N_SESSIONS)

        sessions_available = len(path)
        if sessions_available == 0:
            stats["zero_sessions"] += 1

        # COMPLETE if we have N_SESSIONS bars, or if the last required date
        # is in the cache and we have all available sessions up to REQUIRED_END
        last_required_in_cache = any(
            date.fromisoformat(b["date"]) == REQUIRED_END
            for b in bars
        )
        if sessions_available >= N_SESSIONS:
            obs_status = "COMPLETE"
            stats["complete"] += 1
        elif last_required_in_cache and dec_date >= REQUIRED_END:
            # decision is on or after the last cached date; no future bars yet
            obs_status = "MATURING"
            stats["maturing"] += 1
        elif sessions_available < N_SESSIONS:
            obs_status = "MATURING"
            stats["maturing"] += 1
        else:
            obs_status = "COMPLETE"
            stats["complete"] += 1

        session_records = []
        for i, bar in enumerate(path, start=1):
            session_records.append({
                "session": i,
                "date":    bar["date"],
                "open":    bar["open"],
                "high":    bar["high"],
                "low":     bar["low"],
                "close":   bar["close"],
                "volume":  bar["volume"],
            })

        results.append({
            "decision_id":        decision_id,
            "instrument":         sym,
            "direction":          direction,
            "decision_time":      decision_ts,
            "decision_date_ist":  dec_date.isoformat(),
            "reference_price":    ref_price,
            "target_price":       target_price,
            "stop_price":         stop_price,
            "sessions_available": sessions_available,
            "observation_status": obs_status,
            "sessions":           session_records,
        })

    # ── write output ──────────────────────────────────────────────────────────
    output = {
        "version":          "hdv001_price_paths_v1",
        "built_at":         datetime.now(timezone.utc).isoformat(),
        "n_sessions_target": N_SESSIONS,
        "n_decisions":      len(results),
        "stats":            stats,
        "paths":            results,
    }
    with open(OUTPUT_PATH, "w") as f:
        json.dump(output, f, indent=2)
    print(f"\nWrote {len(results)} price paths to {OUTPUT_PATH.relative_to(WORKSPACE)}")

    # ── print summary ─────────────────────────────────────────────────────────
    print(f"\nSummary:")
    print(f"  Total decisions   : {stats['total']}")
    print(f"  COMPLETE (>= {N_SESSIONS} sessions): {stats['complete']}")
    print(f"  MATURING (< {N_SESSIONS} sessions) : {stats['maturing']}")
    print(f"  NO_CACHE          : {stats['no_cache']}")
    print(f"  Zero sessions     : {stats['zero_sessions']}")

    # ── spot-check a few paths ────────────────────────────────────────────────
    print(f"\nSpot-check (first 3 COMPLETE paths):")
    shown = 0
    for r in results:
        if r["observation_status"] == "COMPLETE" and shown < 3:
            print(f"  {r['decision_id'][:16]}... {r['instrument']} {r['direction']}")
            print(f"    decision_date_ist: {r['decision_date_ist']}")
            print(f"    first bar: {r['sessions'][0]['date']}  close={r['sessions'][0]['close']:.2f}")
            print(f"    last bar:  {r['sessions'][-1]['date']}  close={r['sessions'][-1]['close']:.2f}")
            print(f"    sessions:  {r['sessions_available']}")
            shown += 1

    print(f"\nSpot-check (first 3 MATURING paths):")
    shown = 0
    for r in results:
        if r["observation_status"] == "MATURING" and shown < 3:
            print(f"  {r['decision_id'][:16]}... {r['instrument']} {r['direction']}")
            print(f"    decision_date_ist: {r['decision_date_ist']}")
            if r["sessions"]:
                print(f"    first bar: {r['sessions'][0]['date']}  close={r['sessions'][0]['close']:.2f}")
                print(f"    last bar:  {r['sessions'][-1]['date']}  close={r['sessions'][-1]['close']:.2f}")
            else:
                print(f"    no bars yet")
            print(f"    sessions:  {r['sessions_available']}")
            shown += 1

    # ── write report ──────────────────────────────────────────────────────────
    report_lines = [
        "# HDV-001-C Price Path Extractor Report",
        "",
        f"**Generated:** 2026-08-17",
        f"**Source dataset:** `datasets/stop_research_dataset_v01.json`",
        f"**Price cache:** `datasets/hdv001/hdv001_price_cache_v1/`",
        f"**Output:** `datasets/hdv001/hdv001_price_paths_v1.json`",
        f"**Observation horizon:** {N_SESSIONS} NSE sessions",
        "",
        "## Temporal Rule",
        "",
        "First eligible bar satisfies: `bar_date > decision_date_in_IST`",
        "",
        "No bar from the decision date itself enters the path.",
        "decision_time (UTC) is converted to IST (+05:30) to determine the decision date.",
        "",
        "## Statistics",
        "",
        f"| Metric | Value |",
        f"|--------|-------|",
        f"| Total decisions | {stats['total']} |",
        f"| COMPLETE (>= {N_SESSIONS} sessions) | {stats['complete']} |",
        f"| MATURING (< {N_SESSIONS} sessions) | {stats['maturing']} |",
        f"| NO_CACHE | {stats['no_cache']} |",
        f"| Zero sessions | {stats['zero_sessions']} |",
        "",
        "## Notes",
        "",
        "MATURING decisions are those whose 10-session observation window has not",
        "yet completed as of the cache build date (2026-08-17). These will become",
        "COMPLETE as future sessions are added to the cache.",
        "",
        "All 1,144 decisions receive a path record regardless of whether Config B",
        "or C actually realized the trade. HDV-001 evaluates the Coralys decision,",
        "not the historical portfolio execution mechanics.",
    ]
    REPORT_PATH.write_text("\n".join(report_lines))
    print(f"\nReport written to: {REPORT_PATH.relative_to(WORKSPACE)}")

    if stats["no_cache"] > 0:
        print(f"\nWARN: {stats['no_cache']} decisions have no cache file.")
        sys.exit(1)
    else:
        print("\nHDV-001-C: Price path extraction COMPLETE.")
        sys.exit(0)

if __name__ == "__main__":
    main()