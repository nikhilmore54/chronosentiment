#!/usr/bin/env python3
"""
fetch_v03_bar_cache.py — Acquire OHLCV bar data for Portfolio Replay v0.3 universe.

Fetches daily bars for all 100 NSE instruments in the v0.3 universe using yfinance,
covering the same historical period as the v0.2.1 7-instrument cache.

Output format matches the existing cache exactly:
  [{timestamp: int (Unix epoch, market open ~09:15 IST), open, high, low, close, adj_close, volume}]

Output directory:
  product_validation/CS-P-006/snapshot/20260814T183851Z_100instrument/yahoo_cache/

Usage:
  python3 scripts/fetch_v03_bar_cache.py [--output-dir <dir>] [--dry-run]

Requirements:
  pip install yfinance

Notes:
  - Instruments already present in the output dir are skipped (idempotent).
  - The 7 v0.2.1 instruments are included; if their files already exist in the
    existing 7-instrument cache, they are copied rather than re-fetched.
  - yfinance may rate-limit; the script retries with exponential backoff.
  - Instruments that fail after retries are logged to fetch_errors.json.
  - The script does NOT modify the immutable 7-instrument cache directory.
"""

import json
import os
import shutil
import sys
import time
import argparse
from pathlib import Path
from datetime import datetime, timezone

# ── Universe ──────────────────────────────────────────────────────────────────

# All 100 instruments for v0.3-C (superset of v0.3-A and v0.3-B)
V03_C_100 = [
    # v0.2.1 baseline (7)
    "HDFCBANK.NS", "RELIANCE.NS", "TCS.NS", "INFY.NS",
    "ICICIBANK.NS", "HINDUNILVR.NS", "ITC.NS",
    # NSE large-cap expansion (18) → total 25 for v0.3-A
    "KOTAKBANK.NS", "AXISBANK.NS", "SBIN.NS", "BAJFINANCE.NS",
    "BHARTIARTL.NS", "ASIANPAINT.NS", "MARUTI.NS", "TITAN.NS",
    "SUNPHARMA.NS", "WIPRO.NS", "HCLTECH.NS", "ULTRACEMCO.NS",
    "NESTLEIND.NS", "POWERGRID.NS", "NTPC.NS", "ONGC.NS",
    "TMPV.NS", "TATASTEEL.NS",
    # Mid-cap expansion (25) → total 50 for v0.3-B
    "ADANIENT.NS", "ADANIPORTS.NS", "BAJAJFINSV.NS", "BPCL.NS",
    "BRITANNIA.NS", "CIPLA.NS", "COALINDIA.NS", "DIVISLAB.NS",
    "DRREDDY.NS", "EICHERMOT.NS", "GRASIM.NS", "HEROMOTOCO.NS",
    "HINDALCO.NS", "INDUSINDBK.NS", "JSWSTEEL.NS", "LT.NS",
    "M&M.NS", "PIDILITIND.NS", "SBILIFE.NS", "SHREECEM.NS",
    "SIEMENS.NS", "TECHM.NS", "TRENT.NS", "UPL.NS",
    "VEDL.NS",
    # Broad market expansion (50) → total 100 for v0.3-C
    "ABCAPITAL.NS", "ABFRL.NS", "ACC.NS", "AMBUJACEM.NS",
    "APOLLOHOSP.NS", "APOLLOTYRE.NS", "AUROPHARMA.NS", "BALKRISIND.NS",
    "BANDHANBNK.NS", "BANKBARODA.NS", "BERGEPAINT.NS", "BIOCON.NS",
    "BOSCHLTD.NS", "CANBK.NS", "CHOLAFIN.NS", "COLPAL.NS",
    "CONCOR.NS", "CUMMINSIND.NS", "DABUR.NS", "DLF.NS",
    "ESCORTS.NS", "EXIDEIND.NS", "FEDERALBNK.NS", "GAIL.NS",
    "GODREJCP.NS", "GODREJPROP.NS", "HAVELLS.NS", "HDFCAMC.NS",
    "HDFCLIFE.NS", "ICICIPRULI.NS", "IDFCFIRSTB.NS", "IGL.NS",
    "INDUSTOWER.NS", "IRCTC.NS", "JUBLFOOD.NS", "LICHSGFIN.NS",
    "LUPIN.NS", "MARICO.NS", "MCDOWELL-N.NS", "MFSL.NS",
    "MPHASIS.NS", "MRF.NS", "MUTHOOTFIN.NS", "NAUKRI.NS",
    "NMDC.NS", "PAGEIND.NS", "PEL.NS", "PERSISTENT.NS",
    "PFC.NS", "PNB.NS",
]

# ── Historical period (same as v0.2.1 cache) ─────────────────────────────────
# First bar: 1629085500 → 2021-08-16 09:15 IST
# Last bar:  1786679100 → 2026-08-10 09:15 IST
# yfinance uses date strings for daily bars
PERIOD_START = "2021-08-01"   # slightly before first bar to ensure coverage
PERIOD_END   = "2026-08-16"   # day after last bar

# ── Existing 7-instrument cache (copy from here if available) ─────────────────
EXISTING_CACHE_DIR = Path(
    "product_validation/CS-P-006/snapshot/20260814T183851Z_7instrument/yahoo_cache"
)

# ── Default output directory ──────────────────────────────────────────────────
DEFAULT_OUTPUT_DIR = Path(
    "product_validation/CS-P-006/snapshot/20260814T183851Z_100instrument/yahoo_cache"
)

# ── Retry config ──────────────────────────────────────────────────────────────
MAX_RETRIES = 3
RETRY_BACKOFF_BASE = 5  # seconds


def fetch_bars_yfinance(symbol: str, start: str, end: str) -> list[dict]:
    """Fetch daily OHLCV bars from Yahoo Finance via yfinance.

    Returns a list of bar dicts in the cache format:
      {timestamp, open, high, low, close, adj_close, volume}

    timestamp is Unix epoch seconds (market open ~09:15 IST = 03:45 UTC for NSE).
    """
    import yfinance as yf

    ticker = yf.Ticker(symbol)
    df = ticker.history(start=start, end=end, interval="1d", auto_adjust=False)

    if df.empty:
        raise ValueError(f"no data returned for {symbol}")

    bars = []
    for ts, row in df.iterrows():
        # ts is a pandas Timestamp (timezone-aware or naive)
        try:
            epoch = int(ts.timestamp())
        except Exception:
            epoch = int(ts.to_pydatetime().replace(tzinfo=timezone.utc).timestamp())

        bars.append({
            "timestamp": epoch,
            "open":      float(row["Open"]),
            "high":      float(row["High"]),
            "low":       float(row["Low"]),
            "close":     float(row["Close"]),
            "adj_close": float(row["Adj Close"]),
            "volume":    float(row["Volume"]),
        })

    return bars


def fetch_with_retry(symbol: str, start: str, end: str, max_retries: int) -> list[dict] | None:
    """Fetch bars with exponential backoff. Returns None on final failure."""
    for attempt in range(1, max_retries + 1):
        try:
            bars = fetch_bars_yfinance(symbol, start, end)
            return bars
        except Exception as e:
            wait = RETRY_BACKOFF_BASE * (2 ** (attempt - 1))
            print(f"  [attempt {attempt}/{max_retries}] {symbol} failed: {e}")
            if attempt < max_retries:
                print(f"  retrying in {wait}s...")
                time.sleep(wait)
    return None


def main():
    parser = argparse.ArgumentParser(description="Fetch v0.3 bar cache for 100 NSE instruments")
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=DEFAULT_OUTPUT_DIR,
        help=f"Output directory for .json bar files (default: {DEFAULT_OUTPUT_DIR})",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print what would be fetched without actually fetching",
    )
    parser.add_argument(
        "--symbols",
        nargs="+",
        default=None,
        help="Fetch only these symbols (default: all 100)",
    )
    args = parser.parse_args()

    output_dir: Path = args.output_dir
    output_dir.mkdir(parents=True, exist_ok=True)

    symbols = args.symbols if args.symbols else V03_C_100

    print(f"v0.3 bar cache acquisition")
    print(f"  output_dir: {output_dir}")
    print(f"  period:     {PERIOD_START} → {PERIOD_END}")
    print(f"  symbols:    {len(symbols)}")
    print(f"  dry_run:    {args.dry_run}")
    print()

    if args.dry_run:
        for sym in symbols:
            out_path = output_dir / f"{sym}.json"
            existing = EXISTING_CACHE_DIR / f"{sym}.json"
            if out_path.exists():
                print(f"  SKIP (exists):  {sym}")
            elif existing.exists():
                print(f"  COPY (from 7i): {sym}")
            else:
                print(f"  FETCH:          {sym}")
        return

    # Check yfinance is available
    try:
        import yfinance  # noqa: F401
    except ImportError:
        print("ERROR: yfinance not installed. Run: pip install yfinance")
        sys.exit(1)

    errors: dict[str, str] = {}
    fetched = 0
    copied = 0
    skipped = 0

    for i, sym in enumerate(symbols, 1):
        out_path = output_dir / f"{sym}.json"

        # Already present — skip
        if out_path.exists():
            print(f"[{i:3d}/{len(symbols)}] SKIP    {sym}")
            skipped += 1
            continue

        # Available in existing 7-instrument cache — copy
        existing = EXISTING_CACHE_DIR / f"{sym}.json"
        if existing.exists():
            shutil.copy2(existing, out_path)
            print(f"[{i:3d}/{len(symbols)}] COPY    {sym}  (from 7-instrument cache)")
            copied += 1
            continue

        # Fetch from Yahoo Finance
        print(f"[{i:3d}/{len(symbols)}] FETCH   {sym} ...", end="", flush=True)
        bars = fetch_with_retry(sym, PERIOD_START, PERIOD_END, MAX_RETRIES)

        if bars is None:
            print(f"  FAILED")
            errors[sym] = "max retries exceeded"
            continue

        out_path.write_text(json.dumps(bars, indent=2))
        print(f"  OK  ({len(bars)} bars)")
        fetched += 1

        # Brief pause to avoid rate-limiting
        time.sleep(0.5)

    # ── Summary ───────────────────────────────────────────────────────────────
    print()
    print(f"═══ Acquisition complete ═══")
    print(f"  fetched:  {fetched}")
    print(f"  copied:   {copied}")
    print(f"  skipped:  {skipped}")
    print(f"  errors:   {len(errors)}")

    if errors:
        error_path = output_dir / "fetch_errors.json"
        error_path.write_text(json.dumps(errors, indent=2))
        print(f"  error log: {error_path}")
        print()
        print("Failed symbols:")
        for sym, reason in errors.items():
            print(f"  {sym}: {reason}")
        sys.exit(1)
    else:
        print()
        print(f"All {len(symbols)} symbols acquired successfully.")
        print(f"Cache ready at: {output_dir}")
        print()
        print("Next step — run the v0.3 experiment (strict mode):")
        print(f"  cargo run --bin csp010_portfolio_v03 -- \\")
        print(f"    --search-two product_validation/CS-P-006/discovery/20260815T051900Z_c3 \\")
        print(f"    --cache-dir  {output_dir} \\")
        print(f"    --output-base historical_runs/portfolio_v03_universe_robustness \\")
        print(f"    --strict")


if __name__ == "__main__":
    main()
