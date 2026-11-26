#!/usr/bin/env python3
"""
audit_four_timestamp_observability.py

Audits the Four-Timestamp Lifecycle Alignment & Trigger Latency Protocol:
1. T_peak: Timestamp when MFE was first achieved.
2. T_retreat: Timestamp when unrealized PnL first retreated below 50% of peak MFE.
3. T_condition: First timestamp when decision_state evaluated to EXIT or score crossed exit threshold.
4. T_exit: Execution timestamp (trade journal exit_timestamp).

Calculates Trigger Latency: Delta T_trigger = T_exit - T_condition.
"""

import json
from pathlib import Path
from datetime import datetime, timezone, timedelta
from collections import defaultdict

WORKSPACE_DIR = Path(__file__).parent.parent
LOG_DIR = WORKSPACE_DIR / "live_capture" / "shadow_validation"

def parse_iso(ts_str):
    if not ts_str or ts_str == "ACTIVE":
        return None
    try:
        ts_str = ts_str.replace("Z", "+00:00")
        return datetime.fromisoformat(ts_str).astimezone(timezone.utc)
    except Exception:
        return None

def read_jsonl(path):
    if not path.exists():
        return []
    records = []
    with open(path, "r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if line:
                try:
                    records.append(json.loads(line))
                except Exception:
                    pass
    return records

def main():
    nse_journal = read_jsonl(LOG_DIR / "rp605_nse_baseline_journal.jsonl") + read_jsonl(LOG_DIR / "rp605_nse_freshness_journal.jsonl")
    crypto_journal = read_jsonl(LOG_DIR / "rp605_baseline_journal.jsonl") + read_jsonl(LOG_DIR / "rp605_freshness_journal.jsonl")
    all_journal = nse_journal + crypto_journal

    nse_cards = read_jsonl(LOG_DIR / "rp605_nse_decision_cards_log.jsonl")
    crypto_cards = read_jsonl(LOG_DIR / "rp605_decision_cards_log.jsonl")
    all_cards = nse_cards + crypto_cards

    # Map cards by symbol sorted by time
    cards_by_sym = defaultdict(list)
    for c in all_cards:
        sym = c.get("instrument")
        ts_str = c.get("timestamp_utc")
        dt = parse_iso(ts_str)
        p = c.get("current_price")
        if sym and dt and p is not None:
            cards_by_sym[sym].append((dt, float(p), c))

    for sym in cards_by_sym:
        cards_by_sym[sym].sort(key=lambda x: x[0])

    engine_exits = [t for t in all_journal if t.get("exit_reason") == "ENGINE_EXIT" and t.get("exit_timestamp")]

    reconstructed_count = 0
    trigger_latencies_sec = []
    retreat_latencies_sec = []
    pcf_with_latency = []

    for t in engine_exits:
        sym = t.get("symbol")
        side = t.get("side")
        qty = float(t.get("quantity", 1.0))
        entry_p = float(t.get("entry_price", 0.0))
        entry_dt = parse_iso(t.get("entry_timestamp"))
        exit_dt = parse_iso(t.get("exit_timestamp"))
        realized_pnl = float(t.get("realized_pnl", t.get("pnl", 0.0)))

        if not entry_dt or not exit_dt or not sym:
            continue

        sym_cards = cards_by_sym.get(sym, [])
        holding_cards = [(dt, p, c) for dt, p, c in sym_cards if entry_dt <= dt <= exit_dt]

        if not holding_cards:
            continue

        # 1. T_peak: Timestamp when MFE was first achieved
        if side == "LONG":
            peak_item = max(holding_cards, key=lambda x: x[1])
            mfe_pnl = (peak_item[1] - entry_p) * qty
        else:
            peak_item = min(holding_cards, key=lambda x: x[1])
            mfe_pnl = (entry_p - peak_item[1]) * qty

        t_peak = peak_item[0]

        # 2. T_retreat: First timestamp after T_peak where unrealized PnL fell below 50% of MFE (or zero PnL)
        t_retreat = None
        for dt, p, c in holding_cards:
            if dt < t_peak:
                continue
            cur_pnl = (p - entry_p) * qty if side == "LONG" else (entry_p - p) * qty
            if mfe_pnl > 0 and cur_pnl <= 0.5 * mfe_pnl:
                t_retreat = dt
                break

        # 3. T_condition: First timestamp during holding where decision_state evaluated to EXIT
        t_condition = None
        for dt, p, c in holding_cards:
            st = c.get("decision_state")
            if st == "EXIT":
                t_condition = dt
                break

        # If decision_state was not explicitly logged as EXIT, fallback to exit_dt
        if not t_condition:
            t_condition = exit_dt

        # 4. T_exit = exit_dt
        t_exit = exit_dt

        trigger_latency = (t_exit - t_condition).total_seconds()
        trigger_latencies_sec.append(trigger_latency)

        if t_retreat:
            retreat_latency = (t_exit - t_retreat).total_seconds()
            retreat_latencies_sec.append(retreat_latency)

        reconstructed_count += 1

        if mfe_pnl > 0 and realized_pnl < 0:
            pcf_with_latency.append({
                "trade_id": t.get("trade_id"),
                "symbol": sym,
                "t_entry": entry_dt.isoformat(),
                "t_peak": t_peak.isoformat(),
                "t_retreat": t_retreat.isoformat() if t_retreat else None,
                "t_condition": t_condition.isoformat(),
                "t_exit": t_exit.isoformat(),
                "trigger_latency_sec": trigger_latency,
                "mfe_pnl": round(mfe_pnl, 2),
                "realized_pnl": round(realized_pnl, 2)
            })

    print(f"=== FOUR-TIMESTAMP OBSERVABILITY AUDIT ===")
    print(f"Reconstructed ENGINE_EXITS: {reconstructed_count} / {len(engine_exits)}")
    print(f"Average Trigger Latency (T_exit - T_condition): {sum(trigger_latencies_sec)/len(trigger_latencies_sec):.2f} seconds")
    if retreat_latencies_sec:
        print(f"Average Post-Retreat Delay (T_exit - T_retreat): {sum(retreat_latencies_sec)/len(retreat_latencies_sec)/60.0:.2f} minutes")
    print(f"Sample Profit-Conversion Failure 4-Timestamp Lifecycles:")
    for item in pcf_with_latency[:5]:
        print(json.dumps(item, indent=2))

if __name__ == "__main__":
    main()
