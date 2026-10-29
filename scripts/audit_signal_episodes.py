#!/usr/bin/env python3
"""
scripts/audit_signal_episodes.py

Performs deep event-level reconciliation of Gate C recommendations:
1. Clusters repeated raw emissions into distinct market episodes (gap > 60s).
2. Explicitly reconciles the outcome denominator: Matured + Stopped = Total Closed.
3. Evaluates episode-level (first-of-episode) performance vs raw overlapping emissions.
4. Breaks out realized stop loss vs stop trigger.
"""

import json
from pathlib import Path
import sys

WORKSPACE_DIR = Path(__file__).resolve().parent.parent
EVENT_DIR = WORKSPACE_DIR / "live_capture" / "shadow_validation" / "crypto" / "event_driven"

def main():
    ledger_path = EVENT_DIR / "event_recs_candidate_16b.jsonl"
    if not ledger_path.exists():
        print(f"Error: {ledger_path} not found")
        sys.exit(1)

    recs = [json.loads(l) for l in ledger_path.read_text().split("\n") if l.strip()]
    print(f"================================================================================")
    print(f"  GATE C: SIGNAL EPISODE CLUSTERING & DENOMINATOR RECONCILIATION AUDIT")
    print(f"  Total Raw Recommendations Analyzed: {len(recs)}")
    print(f"================================================================================")

    # Cluster into episodes
    episodes = []
    cur_ep = []
    for r in recs:
        t = r["timestamp_epoch_ms"]
        if not cur_ep:
            cur_ep.append(r)
        else:
            last_t = cur_ep[-1]["timestamp_epoch_ms"]
            if (t - last_t) > 60000:  # 60s gap threshold
                episodes.append(cur_ep)
                cur_ep = [r]
            else:
                cur_ep.append(r)
    if cur_ep:
        episodes.append(cur_ep)

    print(f"\n[*] Section 1: Signal Episode Decomposition (60-second quiescence boundary):")
    print(f"    - Total Raw Emissions:       {len(recs)}")
    print(f"    - Distinct Signal Episodes:  {len(episodes)}")
    print(f"    - Average Signals / Episode: {len(recs) / len(episodes):.1f}")
    print(f"    - Maximum Signals / Episode: {max(len(e) for e in episodes)}")

    print("\n[*] Section 2: Detailed Signal Episode Inventory:")
    for idx, ep in enumerate(episodes):
        t_start = ep[0]["timestamp_utc"][11:19]
        t_end = ep[-1]["timestamp_utc"][11:19]
        dur_s = (ep[-1]["timestamp_epoch_ms"] - ep[0]["timestamp_epoch_ms"]) / 1000.0
        p_start = ep[0]["quote"]["bid_price"]
        p_end = ep[-1]["quote"]["bid_price"]
        print(f"    Episode #{idx+1:02d}: {len(ep):2d} signals | {t_start} -> {t_end} UTC ({dur_s:5.1f}s) | Price: ${p_start:.2f} -> ${p_end:.2f}")

    # Build forward price timeline
    price_timeline = []
    for r in recs:
        t = r["timestamp_epoch_ms"]
        q = r.get("quote", {})
        bid = float(q.get("bid_price", 0.0))
        ask = float(q.get("ask_price", 0.0))
        price_timeline.append((t, bid, ask))
    price_timeline.sort(key=lambda x: x[0])

    print("\n[*] Section 3: Exact Denominator & Outcome Reconciliation:")
    for h_sec, h_label in [(900, "15m"), (1800, "30m"), (3600, "60m")]:
        h_ms = h_sec * 1000

        # Evaluate raw signals
        raw_matured = 0
        raw_stopped = 0
        raw_unresolved = 0
        raw_winners = 0
        raw_losers = 0
        raw_returns = []

        for r in recs:
            t_rec = r["timestamp_epoch_ms"]
            p_entry = float(r["recommendation"]["suggested_entry_price"])
            p_inval = float(r["recommendation"]["invalidation_price"])
            subsequent = [q for q in price_timeline if q[0] > t_rec]

            resolved = False
            exit_price = None
            exit_reason = None

            for q_time, q_bid, q_ask in subsequent:
                # SHORT: stop breached if ask >= p_inval
                if q_ask >= p_inval:
                    resolved = True
                    exit_price = q_ask
                    exit_reason = "STOP"
                    break
                if q_time >= t_rec + h_ms:
                    resolved = True
                    exit_price = q_ask  # exit short at ask
                    exit_reason = "HORIZON"
                    break

            if resolved and exit_price:
                gross_ret = ((p_entry - exit_price) / p_entry) * 10000.0
                net_ret = gross_ret - 12.0  # 12 bps friction
                raw_returns.append(net_ret)
                if exit_reason == "STOP":
                    raw_stopped += 1
                else:
                    raw_matured += 1
                if net_ret > 0:
                    raw_winners += 1
                else:
                    raw_losers += 1
            else:
                raw_unresolved += 1

        total_closed = raw_matured + raw_stopped
        win_rate = (raw_winners / total_closed * 100.0) if total_closed > 0 else 0.0
        net_exp = (sum(raw_returns) / total_closed) if total_closed > 0 else 0.0

        print(f"\n  --- Horizon: {h_label} ({h_sec}s) ---")
        print(f"    - Total Emitted:            {len(recs)}")
        print(f"    - Matured at Horizon:       {raw_matured}")
        print(f"    - Stopped Out:              {raw_stopped}")
        print(f"    - Still Unresolved (Open):  {raw_unresolved}")
        print(f"    - Reconciled Denominator:   {total_closed} (= {raw_matured} matured + {raw_stopped} stopped)")
        print(f"    - Winners (Net > 0):        {raw_winners}")
        print(f"    - Losers (Net <= 0):        {raw_losers}")
        print(f"    - Reconciled Win Rate:      {win_rate:.2f}% ({raw_winners} / {total_closed})")
        print(f"    - Net Expectancy:           {net_exp:.2f} bps (after 12.0 bps round-trip friction)")

        # Now evaluate on EPISODE LEVEL (first signal of each episode only)
        ep_winners = 0
        ep_losers = 0
        ep_stopped = 0
        ep_matured = 0
        ep_unresolved = 0
        ep_returns = []

        for ep in episodes:
            first_sig = ep[0]
            t_rec = first_sig["timestamp_epoch_ms"]
            p_entry = float(first_sig["recommendation"]["suggested_entry_price"])
            p_inval = float(first_sig["recommendation"]["invalidation_price"])
            subsequent = [q for q in price_timeline if q[0] > t_rec]

            resolved = False
            exit_price = None
            exit_reason = None

            for q_time, q_bid, q_ask in subsequent:
                if q_ask >= p_inval:
                    resolved = True
                    exit_price = q_ask
                    exit_reason = "STOP"
                    break
                if q_time >= t_rec + h_ms:
                    resolved = True
                    exit_price = q_ask
                    exit_reason = "HORIZON"
                    break

            if resolved and exit_price:
                gross_ret = ((p_entry - exit_price) / p_entry) * 10000.0
                net_ret = gross_ret - 12.0
                ep_returns.append(net_ret)
                if exit_reason == "STOP":
                    ep_stopped += 1
                else:
                    ep_matured += 1
                if net_ret > 0:
                    ep_winners += 1
                else:
                    ep_losers += 1
            else:
                ep_unresolved += 1

        ep_closed = ep_matured + ep_stopped
        ep_win_rate = (ep_winners / ep_closed * 100.0) if ep_closed > 0 else 0.0
        ep_net_exp = (sum(ep_returns) / ep_closed) if ep_closed > 0 else 0.0

        print(f"    >>> EPISODE-LEVEL (De-duplicated to first-of-episode):")
        print(f"        - Closed Episodes:      {ep_closed} ({ep_matured} matured, {ep_stopped} stopped, {ep_unresolved} unresolved)")
        print(f"        - Episode Winners:      {ep_winners} / {ep_closed} ({ep_win_rate:.2f}% Win Rate)")
        print(f"        - Episode Net Exp:      {ep_net_exp:.2f} bps")

    print(f"\n================================================================================\n")

if __name__ == "__main__":
    main()
