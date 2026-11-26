#!/usr/bin/env python3
"""Test Zerodha Kite Connect historical candle endpoint.
Prerequisite: an access token must be present in `archive/transient_texts/kite_access_token.txt`
(you obtain it by running `scripts/kite_test_auth.py`).
"""
import os
import sys
from datetime import datetime

# Load access token
TOKEN_FILE = "archive/transient_texts/kite_access_token.txt"
if not os.path.exists(TOKEN_FILE):
    print(f"[!] Access token file '{TOKEN_FILE}' not found. Run kite_test_auth.py first.")
    sys.exit(1)
with open(TOKEN_FILE) as f:
    access_token = f.read().strip()

# Load API key (required by KiteConnect)
API_KEY = os.getenv("KITE_API_KEY") or "YOUR_API_KEY"
if API_KEY == "YOUR_API_KEY":
    print("[!] Set your API key via KITE_API_KEY environment variable.")
    sys.exit(1)

from kiteconnect import KiteConnect

kite = KiteConnect(api_key=API_KEY)
kite.set_access_token(access_token)

# NIFTY 50 instrument token (as per Zerodha docs)
INSTRUMENT_TOKEN = 256265

# Example date range
FROM = "2025-01-01 09:15:00"
TO   = "2025-01-01 15:30:00"

try:
    data = kite.historical_data(
        instrument_token=INSTRUMENT_TOKEN,
        from_date=FROM,
        to_date=TO,
        interval="minute",
    )
    print(f"Historical candle test:\nSUCCESS\nRecords returned: {len(data)}")
    if data:
        # Show first three rows (dicts)
        for row in data[:3]:
            print(row)
except Exception as e:
    print("Historical candle test:\nFAILED")
    print(e)
