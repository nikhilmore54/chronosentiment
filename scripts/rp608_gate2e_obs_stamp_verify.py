#!/usr/bin/env python3
"""Verify Gate 2E observational pattern stamps (PATTERN_GATE must stay off)."""

from __future__ import annotations

import json
import os
from collections import deque
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CRYPTO = ROOT / "live_capture" / "shadow_validation" / "crypto"


def main() -> int:
    os.environ.setdefault("RP608_CRYPTO_PATTERN_GATE", "0")
    from scripts.rp608_crypto_trade_pattern_match import (
        entry_gate_enabled,
        load_library,
        match_entry,
    )

    lib_path = CRYPTO / "rp608_crypto_plan_trade_pattern_library.json"
    lib = load_library(lib_path)
    gate = entry_gate_enabled()
    print(
        "library",
        lib_path,
        "schema",
        lib.get("schema"),
        "adverse",
        (lib.get("counts") or {}).get("adverse_entry_patterns"),
        "mined",
        lib.get("mined_at_utc") or lib.get("generated_at_utc"),
    )
    print("entry_gate_enabled", gate, "(must be False for obs deploy)")
    if gate:
        print("FAIL: RP608_CRYPTO_PATTERN_GATE is on — live avoid not authorized")
        return 2

    sample = match_entry(
        library=lib,
        feats={
            "direction": "SHORT",
            "utc_session": "ASIA",
            "of_supported": True,
            "imb_bucket": "LE_M30",
            "atr_bucket": "MID",
            "stop_dist_bucket": "MID",
            "target_dist_bucket": "MID",
            "gross_rr_bucket": "MID",
            "vol_surge": "HIGH",
            "prior_stop_same_utc_day_dir": False,
        },
        gate_on=False,
    )
    print(
        "sample_match",
        sample.get("prediction"),
        "blocks_entry",
        sample.get("blocks_entry"),
        "cohort_n",
        sample.get("cohort_n"),
    )
    if sample.get("blocks_entry"):
        print("FAIL: blocks_entry True with gate_on=False")
        return 2

    state = json.loads(
        (CRYPTO / "trade_plan_overlay_v1" / "overlay_state.json").read_text()
    )
    ap = state.get("active_plan") or {}
    print("active_plan_pattern", ap.get("trade_pattern_match"))

    ev_path = CRYPTO / "trade_plan_overlay_v1" / "trade_plan_events.jsonl"
    recent: deque = deque(maxlen=40)
    entry_types = {
        "PLAN_CREATED",
        "SUPPRESSED_OPEN_PLAN",
        "SUPPRESSED_GATE2E_FAST_SL_COOLDOWN",
        "SUPPRESSED_GATE2E_PATH_INVALID",
        "SUPPRESSED_TRADE_PATTERN",
    }
    with ev_path.open() as f:
        for line in f:
            if not line.strip():
                continue
            try:
                row = json.loads(line)
            except json.JSONDecodeError:
                continue
            if row.get("event_type") in entry_types:
                recent.append(row)
    stamped = sum(1 for r in recent if r.get("trade_pattern_match"))
    print("recent_entry_events", len(recent), "with_trade_pattern_match", stamped)
    if recent:
        last = recent[-1]
        print(
            "last_event",
            last.get("event_type"),
            last.get("trade_pattern_match"),
        )
    print("OK_OBS_GATE2E")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
