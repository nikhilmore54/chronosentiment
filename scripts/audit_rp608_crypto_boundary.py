#!/usr/bin/env python3
"""
scripts/audit_rp608_crypto_boundary.py

Read-Only Audit for RP-608-SHADOW-001B (Binance BTC/USDT).
Reconciles observations against the registered prospective boundary:
  Window Start: 2026-10-26T00:00:00.000Z (05:30:00 IST / 1792972800000 ms).

Strictly Read-Only: Zero modifications to running systemd daemon, checkpoint, or ledgers.
"""

from __future__ import annotations

import hashlib
import json
from datetime import datetime, timezone
from pathlib import Path
import sys
from typing import Any, Dict, List

WORKSPACE_DIR = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE_DIR))

REGISTERED_START_UTC_MS = 1792972800000  # 2026-10-26T00:00:00.000Z
FIFTEEN_MINUTES_MS = 15 * 60 * 1000

CRYPTO_DIR = WORKSPACE_DIR / "live_capture" / "shadow_validation" / "crypto"
CHECKPOINT_PATH = CRYPTO_DIR / "rp608_checkpoint.json"
TELEMETRY_PATH = CRYPTO_DIR / "rp608_shadow_telemetry.jsonl"
DECISION_CARDS_PATH = CRYPTO_DIR / "rp608_decision_cards.jsonl"
TRADE_JOURNAL_PATH = CRYPTO_DIR / "rp608_trade_journal.jsonl"
OPPORTUNITY_PATH = CRYPTO_DIR / "rp608_opportunity_registry.jsonl"


def compute_payload_checksum(data: Dict[str, Any]) -> str:
    d = {k: v for k, v in data.items() if k != "checksum"}
    canonical_json = json.dumps(d, sort_keys=True, separators=(",", ":"))
    return hashlib.sha256(canonical_json.encode("utf-8")).hexdigest()


def audit_crypto_boundary():
    print("=" * 85)
    print("  RP-608-SHADOW-001B: PROSPECTIVE BOUNDARY RECONCILIATION AUDIT")
    print(f"  Registered Start Boundary: 2026-10-26T00:00:00.000Z (05:30:00 IST / {REGISTERED_START_UTC_MS} ms)")
    print("=" * 85)

    if not CHECKPOINT_PATH.exists():
        print(f"[-] Checkpoint file not found: {CHECKPOINT_PATH}")
        sys.exit(1)

    with open(CHECKPOINT_PATH, "r", encoding="utf-8") as f:
        checkpoint = json.load(f)

    # 1. Checksum Validation
    rec_sum = checkpoint.get("checksum", "")
    calc_sum = compute_payload_checksum(checkpoint)
    checksum_valid = (rec_sum == calc_sum)
    print(f"[*] Checkpoint Integrity: {'VALID' if checksum_valid else 'TAMPERED / MISMATCH'}")
    print(f"    Recorded:   {rec_sum}")
    print(f"    Calculated: {calc_sum}")

    last_hb_ms = checkpoint.get("last_heartbeat_utc_ms", 0)
    last_hb_iso = checkpoint.get("last_heartbeat_utc", "")
    last_durable_bar = checkpoint.get("last_durable_bar_idx", 0)
    last_accounted_bar = checkpoint.get("last_accounted_bar_idx", 0)
    last_close_ms = checkpoint.get("last_completed_candle_close_ms", 0)

    print(f"[*] Last Heartbeat: {last_hb_iso} ({last_hb_ms} ms)")
    print(f"[*] Cursors: last_durable_bar_idx={last_durable_bar}, last_accounted_bar_idx={last_accounted_bar}")
    print(f"[*] Last Candle Close: {last_close_ms} ms")

    # 2. Segregate Decision Cards (Pre-Window vs Prospective)
    cards: List[Dict[str, Any]] = []
    if DECISION_CARDS_PATH.exists():
        with open(DECISION_CARDS_PATH, "r", encoding="utf-8") as f:
            cards = [json.loads(l) for l in f if l.strip()]

    pre_window_cards = []
    prospective_cards = []
    for c in cards:
        close_ms = c.get("close_time_ms", 0)
        if close_ms < REGISTERED_START_UTC_MS:
            pre_window_cards.append(c)
        else:
            prospective_cards.append(c)

    print("\n--- Decision Card Accounting ---")
    print(f"  Total Cards Recorded:         {len(cards)}")
    print(f"  Pre-Window Startup Cards:     {len(pre_window_cards)}  (EXCLUDED from prospective sample)")
    print(f"  Prospective Observation Cards: {len(prospective_cards)} (Formal evaluation sample)")

    # 3. Telemetry Gates Audit
    telem_gates = checkpoint.get("telemetry_gates", {})
    print("\n--- Operational Telemetry Gates ---")
    print(f"  Total Completed Bars (Daemon): {telem_gates.get('total_completed_bars', 0)}")
    print(f"  Quote Coverage:               {telem_gates.get('quote_coverage_pct', 0.0)}%")
    print(f"  Stale Quote Count:            {telem_gates.get('stale_quote_count', 0)}")
    print(f"  Max Clock Drift:              {telem_gates.get('max_drift_ms', 0)} ms")
    print(f"  Consecutive Missing Bars:     {telem_gates.get('consecutive_missing_bars', 0)}")
    print(f"  Total Missing Bars:           {telem_gates.get('total_missing_bars', 0)}")
    print(f"  24h Reconnect Count:          {telem_gates.get('reconnection_count_24h', 0)}")
    print(f"  Window Tainted by Breach:     {telem_gates.get('window_tainted_by_breach', False)}")

    # 4. Prospective Window Status
    now_ms = int(datetime.now(timezone.utc).timestamp() * 1000)
    print("\n--- Boundary Status ---")
    if now_ms < REGISTERED_START_UTC_MS:
        remaining_sec = (REGISTERED_START_UTC_MS - now_ms) / 1000.0
        print(f"  STATUS: PRE_WINDOW_BUFFER (Window opens in {remaining_sec/60.0:.1f} minutes at 05:30:00 IST)")
    else:
        elapsed_sec = (now_ms - REGISTERED_START_UTC_MS) / 1000.0
        expected_bars = int(elapsed_sec // (15 * 60))
        print(f"  STATUS: WINDOW_ACTIVE (Active for {elapsed_sec/3600.0:.2f} hours)")
        print(f"  Expected Prospective Bars:    {expected_bars}")
        print(f"  Actual Prospective Bars:      {len(prospective_cards)}")


if __name__ == "__main__":
    audit_crypto_boundary()
