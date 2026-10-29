#!/usr/bin/env python3
"""Kite Connect authentication helper.
This script guides you through the login flow to obtain an access token.
You must provide YOUR_API_KEY and YOUR_API_SECRET via environment variables
or by editing the placeholders before running.
Do NOT share your API credentials publicly.
"""
import os
import sys
from urllib.parse import urlparse, parse_qs

# Step 1: obtain API key (set via env var or edit placeholder)
API_KEY = os.getenv("KITE_API_KEY") or "YOUR_API_KEY"
if API_KEY == "YOUR_API_KEY":
    print("[!] Please set your Kite API key via the KITE_API_KEY environment variable or edit the script.")
    sys.exit(1)

from kiteconnect import KiteConnect

kite = KiteConnect(api_key=API_KEY)
print("Login URL (open in a browser):")
print(kite.login_url())
print("\nAfter logging in, you will be redirected to a URL like:\nhttp://127.0.0.1:8000/callback?request_token=XXXXX\nCopy the request_token value and paste it below.")
request_token = input("Enter request_token: ").strip()
if not request_token:
    print("[!] No request token provided.")
    sys.exit(1)

# Step 2: obtain API secret (set via env var or edit placeholder)
API_SECRET = os.getenv("KITE_API_SECRET") or "YOUR_API_SECRET"
if API_SECRET == "YOUR_API_SECRET":
    print("[!] Please set your Kite API secret via the KITE_API_SECRET environment variable or edit the script.")
    sys.exit(1)

try:
    data = kite.generate_session(request_token, api_secret=API_SECRET)
    access_token = data["access_token"]
    print("\nSuccess! Your access token is:\n" + access_token)
    # Optionally, store it for later use
    with open("archive/transient_texts/kite_access_token.txt", "w") as f:
        f.write(access_token)
    print("Access token saved to archive/transient_texts/kite_access_token.txt")
except Exception as e:
    print(f"[ERROR] Failed to generate session: {e}")
    sys.exit(1)
