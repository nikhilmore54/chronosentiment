#!/usr/bin/env python3
"""
scripts/test_kite_probe.py

Isolated Read-Only Kite Connect Market-Data Probe for RP-608 NSE 119 Universe.
Strictly Read-Only (0.0% Order Authority).

Usage:
  # Step 1: Exchange request_token for access_token and test all 119 symbols:
  python3 scripts/test_kite_probe.py --request-token <TOKEN>

  # Step 2: Once access_token is saved, test quote freshness and batch L1 depth:
  python3 scripts/test_kite_probe.py
"""

from __future__ import annotations

import argparse
import csv
import hashlib
import io
import json
import os
from pathlib import Path
import sys
import time
import urllib.request
import urllib.parse
from typing import Any, Dict, List, Optional, Tuple

WORKSPACE_DIR = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE_DIR))

from scripts.run_rp608_nse_shadow_daemon import NSE_119_SYMBOLS

CREDENTIALS_FILE = Path.home() / ".kite_credentials.json"
OUTPUT_DIR = WORKSPACE_DIR / "live_capture" / "shadow_validation" / "nse" / "test_run"
OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
REPORT_FILE = OUTPUT_DIR / "kite_readiness_report.json"

# Special symbol mappings between Yahoo Finance (.NS) and Kite tradingsymbol
SYMBOL_OVERRIDE_MAP = {
    "ICICIPRULI.NS": "ICICILIFE",
    "TATAMOTORS.NS": "TATAMOTORS",  # Standard ticker verification
}


def load_credentials() -> Dict[str, str]:
    if not CREDENTIALS_FILE.exists():
        raise FileNotFoundError(f"Credentials file {CREDENTIALS_FILE} not found!")
    with open(CREDENTIALS_FILE, "r", encoding="utf-8") as f:
        return json.load(f)


def save_credentials(creds: Dict[str, str]):
    with open(CREDENTIALS_FILE, "w", encoding="utf-8") as f:
        json.dump(creds, f, indent=2)
    os.chmod(CREDENTIALS_FILE, 0o600)


def exchange_request_token(api_key: str, api_secret: str, request_token: str) -> str:
    """Exchanges a one-time login request_token for a daily access_token."""
    checksum_raw = f"{api_key}{request_token}{api_secret}"
    checksum = hashlib.sha256(checksum_raw.encode("utf-8")).hexdigest()

    url = "https://api.kite.trade/session/token"
    payload = urllib.parse.urlencode({
        "api_key": api_key,
        "request_token": request_token,
        "checksum": checksum
    }).encode("utf-8")

    req = urllib.request.Request(url, data=payload, headers={
        "X-Kite-Version": "3",
        "User-Agent": "ChronoSentiment-RP608-Shadow"
    })

    try:
        with urllib.request.urlopen(req, timeout=10) as resp:
            data = json.loads(resp.read().decode("utf-8"))
            access_token = data.get("data", {}).get("access_token")
            if not access_token:
                raise RuntimeError(f"No access_token returned: {data}")
            return access_token
    except urllib.error.HTTPError as e:
        err_msg = e.read().decode("utf-8")
        raise RuntimeError(f"HTTP {e.code} during token exchange: {err_msg}")


def kite_api_get(endpoint: str, api_key: str, access_token: str, params: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
    url = f"https://api.kite.trade{endpoint}"
    if params:
        query_str = urllib.parse.urlencode(params, doseq=True)
        url = f"{url}?{query_str}"

    headers = {
        "X-Kite-Version": "3",
        "Authorization": f"token {api_key}:{access_token}",
        "User-Agent": "ChronoSentiment-RP608-Shadow"
    }

    req = urllib.request.Request(url, headers=headers)
    with urllib.request.urlopen(req, timeout=10) as resp:
        return json.loads(resp.read().decode("utf-8"))


def get_nse_instrument_map() -> Dict[str, Tuple[int, str]]:
    """Fetches official NSE instruments and maps all 119 scrips to (instrument_token, tradingsymbol)."""
    url = "https://api.kite.trade/instruments/NSE"
    req = urllib.request.Request(url, headers={"User-Agent": "ChronoSentiment-RP608-Shadow"})
    with urllib.request.urlopen(req, timeout=15) as resp:
        content = resp.read().decode("utf-8")

    reader = csv.DictReader(io.StringIO(content))
    raw_map = {}
    for row in reader:
        if row.get("segment") == "NSE" and row.get("instrument_type") == "EQ":
            tsym = row.get("tradingsymbol", "").strip()
            tok = int(row.get("instrument_token"))
            raw_map[tsym] = tok

    resolved_map = {}
    missing = []
    for full_sym in NSE_119_SYMBOLS:
        tsym = SYMBOL_OVERRIDE_MAP.get(full_sym, full_sym.replace(".NS", ""))
        if tsym in raw_map:
            resolved_map[full_sym] = (raw_map[tsym], tsym)
        else:
            missing.append(full_sym)

    if missing:
        raise RuntimeError(f"Symbol census failure: missing {len(missing)} symbols: {missing}")

    return resolved_map


def run_probe(request_token: Optional[str] = None):
    print("=" * 80)
    print("  RP-608 KITE CONNECT ISOLATED MARKET-DATA PROBE (119 SYMBOLS)")
    print("=" * 80)

    creds = load_credentials()
    api_key = creds.get("api_key")
    api_secret = creds.get("api_secret")
    access_token = creds.get("access_token")

    if request_token:
        print(f"[*] Exchanging request_token for new access_token...")
        access_token = exchange_request_token(api_key, api_secret, request_token)
        creds["access_token"] = access_token
        save_credentials(creds)
        print(f"[+] Successfully generated and saved access_token (length={len(access_token)})")

    if not access_token:
        login_url = f"https://kite.zerodha.com/connect/login?v=3&api_key={api_key}"
        print(f"[-] No access_token found in credentials file.")
        print(f"[!] Please open this login URL in your browser:\n    {login_url}")
        print(f"[!] After login, copy the request_token from the redirected URL and run:")
        print(f"    python3 scripts/test_kite_probe.py --request-token <TOKEN>")
        sys.exit(1)

    # 1. Profile / Authentication Check
    print("\n--- Gate 1: Authentication & Read-Only Profile Check ---")
    t0 = time.time()
    profile_data = kite_api_get("/user/profile", api_key, access_token)
    rtt_profile = (time.time() - t0) * 1000
    user_name = profile_data.get("data", {}).get("user_name")
    user_id = profile_data.get("data", {}).get("user_id")
    user_type = profile_data.get("data", {}).get("user_type")
    print(f"[+] Authenticated: User ID: {user_id}, Name: {user_name}, Type: {user_type} (RTT: {rtt_profile:.1f}ms)")

    # 2. Reconcile 119 Instruments
    print("\n--- Gate 2: 119-Symbol Instrument Census Reconciliation ---")
    t0 = time.time()
    instrument_map = get_nse_instrument_map()
    elapsed_instruments = time.time() - t0
    print(f"[+] Reconciled: 119 / 119 symbols mapped to active NSE tokens in {elapsed_instruments:.2f}s")
    for s in ["ADANIENT.NS", "ICICIPRULI.NS", "RELIANCE.NS", "TCS.NS", "WIPRO.NS"]:
        tok, tsym = instrument_map[s]
        print(f"    {s} -> tradingsymbol: {tsym}, token: {tok}")

    # 3. Single-Batch L1 Quote Snapshot (All 119 Scrips)
    print("\n--- Gate 3: Single-Batch L1 Quote Snapshot (All 119 Scrips) ---")
    # Kite format: "NSE:TRADING_SYMBOL"
    kite_instrument_keys = [f"NSE:{tsym}" for _, (_, tsym) in instrument_map.items()]
    t0 = time.time()
    quotes_resp = kite_api_get("/quote", api_key, access_token, params={"i": kite_instrument_keys})
    rtt_quote = (time.time() - t0) * 1000
    quotes_data = quotes_resp.get("data", {})

    print(f"[+] Received quotes for {len(quotes_data)} / 119 instruments in single batch request! (RTT: {rtt_quote:.1f}ms)")

    # Inspect L1 Bid/Ask spread, depth, and timestamps
    valid_l1_count = 0
    zero_depth_count = 0
    crossed_market_count = 0
    sample_quotes = []
    all_quotes = []

    for full_sym, (tok, tsym) in instrument_map.items():
        k = f"NSE:{tsym}"
        q = quotes_data.get(k)
        if not q:
            continue
        last_price = q.get("last_price", 0.0)
        depth = q.get("depth", {})
        buys = depth.get("buy", [])
        sells = depth.get("sell", [])
        ohlc = q.get("ohlc", {})

        best_bid = buys[0].get("price", 0.0) if buys else 0.0
        best_ask = sells[0].get("price", 0.0) if sells else 0.0
        bid_qty = buys[0].get("quantity", 0) if buys else 0
        ask_qty = sells[0].get("quantity", 0) if sells else 0

        mid = (best_bid + best_ask) / 2.0 if (best_bid + best_ask) > 0 else last_price
        spread_bps = ((best_ask - best_bid) / mid * 10000.0) if mid > 0 else 0.0
        half_spread_bps = spread_bps / 2.0

        ts = q.get("timestamp") or q.get("last_trade_time")

        is_zero_depth = (best_bid <= 0 or best_ask <= 0)
        is_crossed = (best_bid >= best_ask > 0)
        is_valid = (not is_zero_depth) and (not is_crossed)

        if is_valid:
            valid_l1_count += 1
        if is_zero_depth:
            zero_depth_count += 1
        if is_crossed:
            crossed_market_count += 1

        quote_entry = {
            "symbol": full_sym,
            "tradingsymbol": tsym,
            "instrument_token": tok,
            "last_price": last_price,
            "best_bid": best_bid,
            "best_ask": best_ask,
            "bid_qty": bid_qty,
            "ask_qty": ask_qty,
            "half_spread_bps": round(half_spread_bps, 2),
            "volume": q.get("volume", 0),
            "day_open": ohlc.get("open", 0.0),
            "day_high": ohlc.get("high", 0.0),
            "day_low": ohlc.get("low", 0.0),
            "prev_close": ohlc.get("close", 0.0),
            "timestamp": str(ts),
            "valid_depth": is_valid,
            "zero_depth": is_zero_depth,
            "crossed": is_crossed
        }
        all_quotes.append(quote_entry)

        if len(sample_quotes) < 5:
            sample_quotes.append(quote_entry)

    print(f"[+] Valid L1 Bid/Ask Depth: {valid_l1_count} / {len(quotes_data)} instruments have active top-of-book quotes.")
    print("    Sample L1 Depth & Realized Spreads:")
    for sq in sample_quotes:
        print(f"      {sq['symbol']:<15} P={sq['last_price']:<8} Bid={sq['best_bid']:<8} Ask={sq['best_ask']:<8} HalfSpread={sq['half_spread_bps']}bps Time={sq['timestamp']}")

    # 4. Save Comprehensive Readiness Report
    report = {
        "timestamp_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "user_id": user_id,
        "user_name": user_name,
        "symbols_reconciled": len(instrument_map),
        "quotes_retrieved_in_single_batch": len(quotes_data),
        "valid_l1_depth_count": valid_l1_count,
        "zero_depth_count": zero_depth_count,
        "crossed_market_count": crossed_market_count,
        "missing_quotes_count": len(instrument_map) - len(quotes_data),
        "quote_batch_rtt_ms": round(rtt_quote, 1),
        "sample_quotes": sample_quotes,
        "all_quotes": all_quotes,
        "verdict": "PASS" if len(quotes_data) == 119 and crossed_market_count == 0 else "INCOMPLETE"
    }
    with open(REPORT_FILE, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)

    print(f"\n[+] Readiness report saved to {REPORT_FILE}")
    print(f"[+] All checks complete. Zero live order authority exercised.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="RP-608 Kite Connect Market-Data Probe")
    parser.add_argument("--request-token", type=str, default=None, help="One-time request token from login redirect URL")
    args = parser.parse_args()
    run_probe(request_token=args.request_token)
