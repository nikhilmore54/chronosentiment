#!/bin/bash
set -e
cd core
echo "Fetching 2026_recent_crossfeed_1h (Yahoo 1m)..."
python3 ../scripts/yahoo_fetcher.py --symbol BTC-USD --name 2026_recent_crossfeed_1h_yahoo_1m
echo "Fetching 2026_recent_discontinuity_1h (Yahoo 1m)..."
python3 ../scripts/yahoo_fetcher.py --symbol BTC-USD --name 2026_recent_discontinuity_1h_yahoo_1m
