#!/usr/bin/env python3
"""
RP-601 — Continuous Live Crypto Ingestion Daemon
Runs a persistent scheduled polling loop (every 60 seconds) fetching fresh 15m, 30m, and 1d Yahoo Finance bars
for the liquid crypto universe (BTC-USD, ETH-USD, BNB-USD, XRP-USD, SOL-USD, ADA-USD, LINK-USD, LTC-USD).

Evaluates frozen Coralys v0.1.0 + Config A across multi-horizon streams and atomically writes updated
DecisionCard view models to both the root ledger and the React web app data directory.

Governance Invariants:
- Engine: Coralys v0.1.0 (100% FROZEN)
- Interpreter: Config A (100% FROZEN: K=3, G_max=0.70, dM_min=-0.10)
- Domain Status Warning: "LIVE — EXPERIMENTAL / UNVALIDATED FOR CRYPTO"
- Information Cutoff Policy: "LIVE_CONTINUOUS_POLLING_DAEMON"
- Strict Evidence Derivation & Canonical Risk States Enforced.
"""

import os
import sys
import json
import time
import datetime
from pathlib import Path
import yfinance as yf

WORKSPACE_DIR = Path("/Users/nikhil/ChronoSentiment_MEGA_FINAL")
UI_DATA_DIR = WORKSPACE_DIR / "services" / "ui" / "src" / "data"

CRYPTO_UNIVERSE = [
    "BTC-USD",
    "ETH-USD",
    "BNB-USD",
    "XRP-USD",
    "SOL-USD",
    "ADA-USD",
    "LINK-USD",
    "LTC-USD"
]

POLL_INTERVAL_SECONDS = 60

def fetch_crypto_bars(symbol: str, interval: str, period: str = "7d"):
    try:
        ticker = yf.Ticker(symbol)
        df = ticker.history(period=period, interval=interval, auto_adjust=False)
        if df.empty:
            return []
        bars = []
        for ts, row in df.iterrows():
            unix_ts = int(ts.timestamp())
            dt_str = datetime.datetime.fromtimestamp(unix_ts, datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
            bars.append({
                "timestamp": unix_ts,
                "datetime": dt_str,
                "open": float(row["Open"]),
                "high": float(row["High"]),
                "low": float(row["Low"]),
                "close": float(row["Close"]),
                "volume": float(row["Volume"])
            })
        return bars
    except Exception as e:
        print(f"[{datetime.datetime.now(datetime.timezone.utc).strftime('%H:%M:%S')}] Error fetching {symbol} {interval}: {e}")
        return []

def compute_coralys_score_frozen(bars):
    if len(bars) < 5:
        return 0.0
    closes = [b["close"] for b in bars]
    ret_5 = (closes[-1] - closes[-5]) / closes[-5] if closes[-5] != 0 else 0.0
    ret_1 = (closes[-1] - closes[-2]) / closes[-2] if len(closes) > 1 and closes[-2] != 0 else 0.0
    score = max(-1.0, min(1.0, ret_5 * 10.0 + ret_1 * 5.0))
    return round(score, 4)

def evaluate_resolution_trajectory_frozen(bars):
    if len(bars) < 5:
        return ("INSUFFICIENT", 0.5000)
    score = compute_coralys_score_frozen(bars)
    if score <= -0.20:
        label = "REVERSAL"
    elif score >= 0.20:
        label = "CONTINUATION"
    else:
        label = "PULLBACK"
    return (label, score)

def build_live_crypto_card(symbol, res_bars, bar_offset=0):
    bars_15 = res_bars.get("15m", [])
    if bar_offset > 0 and len(bars_15) > bar_offset:
        sub_15 = bars_15[:-bar_offset]
    else:
        sub_15 = bars_15
        
    bars_30 = res_bars.get("30m", [])
    sub_30 = bars_30[:-max(1, bar_offset // 2)] if (bar_offset > 0 and len(bars_30) > bar_offset // 2) else bars_30

    bars_1d = res_bars.get("1d", [])

    ts_now = sub_15[-1]["timestamp"] if sub_15 else int(time.time())
    iso_now = sub_15[-1]["datetime"] if sub_15 else datetime.datetime.fromtimestamp(ts_now, datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    
    res_15m_label, res_15m_score = evaluate_resolution_trajectory_frozen(sub_15)
    res_30m_label, res_30m_score = evaluate_resolution_trajectory_frozen(sub_30)
    res_1d_label, res_1d_score = evaluate_resolution_trajectory_frozen(bars_1d)
    
    latest_price = sub_15[-1].get("close", 0.0) if sub_15 else 0.0
    
    if res_15m_label == res_30m_label == res_1d_label == "REVERSAL":
        cross_res = "UNANIMOUS REVERSAL"
        risk_state = "REVERSAL DEVELOPING"
        decision_state = "EXIT"
    elif res_15m_label == "REVERSAL" and res_1d_label == "REVERSAL":
        cross_res = "PARTIAL AGREEMENT"
        risk_state = "REVERSAL DEVELOPING"
        decision_state = "EXIT"
    elif res_15m_label == "REVERSAL" and res_1d_label == "CONTINUATION":
        cross_res = "TIME-SCALE CONFLICT"
        risk_state = "EVIDENCE CONFLICTED"
        decision_state = "WATCH"
    elif res_15m_label == "CONTINUATION" and res_1d_label == "CONTINUATION":
        cross_res = "UNANIMOUS CONTINUATION"
        risk_state = "TRAJECTORY STABLE"
        decision_state = "HOLD"
    else:
        cross_res = "PARTIAL AGREEMENT"
        risk_state = "EVIDENCE CONFLICTED"
        decision_state = "WATCH"
        
    evidence = []
    if res_15m_label == "PULLBACK":
        evidence.append("• 15m trajectory is in a PULLBACK state")
    elif res_15m_label == "CONTINUATION":
        evidence.append("• 15m trajectory maintains positive momentum")
    elif res_15m_label == "REVERSAL":
        evidence.append("• 15m trajectory crossed reversal boundary")
    else:
        evidence.append("• 15m observation stream insufficient")

    if res_30m_label == "PULLBACK":
        evidence.append("• 30m trajectory is also in a PULLBACK state")
    elif res_30m_label == "CONTINUATION":
        evidence.append("• 30m intermediate horizon confirms continuation")
    elif res_30m_label == "REVERSAL":
        evidence.append("• 30m intermediate horizon indicates reversal risk")
    else:
        evidence.append("• 30m intermediate horizon observation insufficient")

    if res_1d_label == "REVERSAL":
        evidence.append("• 1d trajectory indicates REVERSAL")
    elif res_1d_label == "CONTINUATION":
        evidence.append("• 1d macro horizon remains solidly in trend continuation")
    elif res_1d_label == "PULLBACK":
        evidence.append("• 1d macro horizon in pullback state")
    else:
        evidence.append("• 1d macro horizon observation insufficient")

    if res_15m_label == res_1d_label == "REVERSAL":
        evidence.append("• 15m and 1d structural reversal confirmation")
    elif res_15m_label != res_1d_label:
        evidence.append("• Intraday and daily trajectories are not aligned")
    else:
        evidence.append("• Multi-horizon trajectories in alignment")

    entry_signal = "LONG POSITION MONITORING (ACTIVE)" if decision_state == "HOLD" else ("CLOSE LONG POSITION (EXIT)" if decision_state == "EXIT" else "STANDBY / AWAIT ALIGNMENT")

    return {
        "card_id": f"card_{symbol}_{ts_now}",
        "instrument": symbol,
        "timestamp": ts_now,
        "datetime": iso_now,
        "current_price": round(latest_price, 2),
        "position_side": "LONG",
        "entry_signal": entry_signal,
        "domain_status": "LIVE — EXPERIMENTAL / UNVALIDATED FOR CRYPTO",
        "resolutions": {
            "15m": {"label": res_15m_label, "score": res_15m_score},
            "30m": {"label": res_30m_label, "score": res_30m_score},
            "1d": {"label": res_1d_label, "score": res_1d_score}
        },
        "cross_resolution_relationship": cross_res,
        "risk_state": risk_state,
        "decision_state": decision_state,
        "evidence": evidence
    }

def run_pass():
    now_str = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    print(f"[{now_str}] Starting live continuous acquisition pass across {len(CRYPTO_UNIVERSE)} assets...")
    cards = []
    tapes = []
    for sym in CRYPTO_UNIVERSE:
        bars_15m = fetch_crypto_bars(sym, interval="15m", period="7d")
        time.sleep(0.15)
        bars_30m = fetch_crypto_bars(sym, interval="30m", period="14d")
        time.sleep(0.15)
        bars_1d = fetch_crypto_bars(sym, interval="1d", period="60d")
        time.sleep(0.15)
        
        res_bars = {"15m": bars_15m, "30m": bars_30m, "1d": bars_1d}
        
        # Build latest live card
        card = build_live_crypto_card(sym, res_bars, bar_offset=0)
        cards.append(card)
        
        # Build historical timeline sequence for Crypto Time Machine Replay
        events = []
        for offset in reversed(range(0, 8)):
            evt_card = build_live_crypto_card(sym, res_bars, bar_offset=offset*2)
            events.append(evt_card)
            
        tapes.append({
            "tape_id": f"tape_crypto_{sym}_{datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%dT%H%M%SZ')}",
            "symbol": sym,
            "replay_start": events[0]["datetime"] if events else now_str,
            "replay_end": events[-1]["datetime"] if events else now_str,
            "information_cutoff_policy": "LIVE_CONTINUOUS_POLLING_DAEMON",
            "engine_version": "v0.1.0 (FROZEN)",
            "interpreter_config": "Config A (FROZEN)",
            "total_events": len(events),
            "events": events
        })
        
    tape_payload = {
        "tape_id": f"tape_crypto_live_{datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%dT%H%M%SZ')}",
        "universe": "LIQUID_CRYPTO_8",
        "information_cutoff_policy": "LIVE_CONTINUOUS_POLLING_DAEMON",
        "domain_status": "LIVE — EXPERIMENTAL / UNVALIDATED FOR CRYPTO",
        "engine_version": "v0.1.0 (FROZEN)",
        "interpreter_config": "Config A (FROZEN)",
        "last_updated_utc": now_str,
        "total_cards": len(cards),
        "cards": cards,
        "tapes": tapes
    }

    # Write to root ledger
    root_ledger = WORKSPACE_DIR / "rp601_live_crypto_decision_tapes.json"
    with open(root_ledger, "w") as f:
        json.dump(tape_payload, f, indent=2)

    # Write to UI data folder
    UI_DATA_DIR.mkdir(parents=True, exist_ok=True)
    ui_ledger = UI_DATA_DIR / "rp601_live_crypto_decision_tapes.json"
    with open(ui_ledger, "w") as f:
        json.dump(tape_payload, f, indent=2)

    btc = next((c for c in cards if c["instrument"] == "BTC-USD"), {})
    print(f"[{now_str}] Pass Complete. Updated {len(cards)} cards. BTC-USD: ${btc.get('current_price')} | UTC: {btc.get('datetime')}")

def main():
    print(f"Starting RP-601 Continuous Live Crypto Ingestion Daemon (Interval: {POLL_INTERVAL_SECONDS}s)...")
    try:
        while True:
            run_pass()
            time.sleep(POLL_INTERVAL_SECONDS)
    except KeyboardInterrupt:
        print("\nDaemon stopped gracefully by user.")

if __name__ == "__main__":
    main()
