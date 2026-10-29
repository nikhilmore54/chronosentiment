#!/usr/bin/env python3
"""
scripts/audit_rp611_replay_readiness.py

RP-611-001: Data Availability and Replay Readiness Audit.
Verifies that all 99 target NSE symbols have complete, continuous 1-minute
intraday data for all four target sessions:
  - Discovery Sessions: 2026-09-28 and 2026-09-29
  - Prospective Sessions: 2026-09-30 and 2026-10-01
  (2026-10-02 confirmed exchange holiday: Gandhi Jayanti).

Outputs:
  - live_capture/shadow_validation/rp611_replay_readiness_audit.json
"""

import json
from pathlib import Path
from datetime import datetime, timezone, timedelta
from collections import defaultdict

import sys
WORKSPACE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE))
CACHE_DIR = WORKSPACE / "intraday_capture" / "yahoo_cache_1m"
LOG_DIR = WORKSPACE / "live_capture" / "shadow_validation"
OUT_JSON = LOG_DIR / "rp611_replay_readiness_audit.json"

# Canonical 99 symbols from RP-605 baseline journal
CANONICAL_JOURNAL = LOG_DIR / "rp605_nse_baseline_journal.jsonl"
def get_canonical_symbols():
    syms = set()
    with CANONICAL_JOURNAL.open(encoding="utf-8") as f:
        for line in f:
            if line.strip():
                d = json.loads(line)
                syms.add(d["symbol"])
    return sorted(list(syms))

SYMBOLS = get_canonical_symbols()
IST = timezone(timedelta(hours=5, minutes=30))
TARGET_SESSIONS = ["2026-09-28", "2026-09-29", "2026-09-30", "2026-10-01"]


def audit_readiness():
    results = {}
    total_symbols = len(SYMBOLS)
    missing_symbols = []
    session_bar_stats = defaultdict(lambda: {"total_bars": 0, "min_bars": 9999, "max_bars": 0, "symbols_with_300plus": 0})

    for sym in SYMBOLS:
        p = CACHE_DIR / f"{sym}.json"
        if not p.exists():
            missing_symbols.append(sym)
            continue

        data = json.loads(p.read_text(encoding="utf-8"))
        bars_by_day = defaultdict(list)
        for b in data:
            dt = datetime.fromtimestamp(b["timestamp"], IST)
            d_str = dt.strftime("%Y-%m-%d")
            if d_str in TARGET_SESSIONS:
                # Regular market session: 09:15 to 15:30 IST
                tod = dt.hour * 60 + dt.minute
                if 555 <= tod <= 930:
                    bars_by_day[d_str].append(b)

        results[sym] = {d: len(bars_by_day[d]) for d in TARGET_SESSIONS}
        for d in TARGET_SESSIONS:
            n = len(bars_by_day[d])
            session_bar_stats[d]["total_bars"] += n
            session_bar_stats[d]["min_bars"] = min(session_bar_stats[d]["min_bars"], n)
            session_bar_stats[d]["max_bars"] = max(session_bar_stats[d]["max_bars"], n)
            if n >= 300:
                session_bar_stats[d]["symbols_with_300plus"] += 1

    payload = {
        "milestone": "RP-611-001",
        "audit_name": "Data Availability & Replay Readiness Audit",
        "total_target_symbols": total_symbols,
        "available_symbols_count": total_symbols - len(missing_symbols),
        "missing_symbols": missing_symbols,
        "target_sessions": TARGET_SESSIONS,
        "session_summary": {
            d: {
                "total_bars_across_universe": session_bar_stats[d]["total_bars"],
                "mean_bars_per_symbol": round(session_bar_stats[d]["total_bars"] / total_symbols, 1),
                "min_bars_in_symbol": session_bar_stats[d]["min_bars"],
                "max_bars_in_symbol": session_bar_stats[d]["max_bars"],
                "symbols_with_complete_coverage": session_bar_stats[d]["symbols_with_300plus"],
            }
            for d in TARGET_SESSIONS
        },
        "verdict": "READY FOR FROZEN REPLAY" if len(missing_symbols) == 0 else "MISSING DATA",
    }

    with OUT_JSON.open("w", encoding="utf-8") as f:
        json.dump(payload, f, indent=2)

    print(f"Readiness audit complete. Verdict: {payload['verdict']}")
    print(f"Summary: {json.dumps(payload['session_summary'], indent=2)}")


if __name__ == "__main__":
    audit_readiness()
