#!/usr/bin/env python3
"""
ChronoSentiment — LSE Multi-Asset Dataset Downloader
=====================================================
Downloads diversified LSE stock data using yfinance.

Usage:
    python3 scripts/download_lse_data.py --interval 5m --period 60d
    python3 scripts/download_lse_data.py --interval 1d --period 2y

Output:
    data/lse/{interval}/{SYMBOL}.csv

CSV Format (strict):
    timestamp,open,high,low,close,volume
    (timestamp = %Y-%m-%d %H:%M:%S)
"""

import argparse
import os
import sys
import time
import logging
import pandas as pd
import yfinance as yf

# ─── Logging ────────────────────────────────────────────────────────────────
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] %(message)s",
    datefmt="%H:%M:%S",
)
log = logging.getLogger("lse_downloader")

# ─── LSE Universe ───────────────────────────────────────────────────────────
LSE_SYMBOLS = [
    "VOD.L", "BP.L", "HSBA.L", "GSK.L", "AZN.L", 
    "RIO.L", "LLOY.L", "BARC.L", "TSCO.L", "SHEL.L",
    "RR.L", "AAL.L", "GLEN.L", "BA.L", "NWG.L"
]

# Yahoo Finance free tier limits for intraday data
INTERVAL_PERIOD_LIMITS = {
    "1m":  "7d",    # max 7 days for 1m
    "5m":  "60d",   # max 60 days for 5m
    "15m": "60d",
    "30m": "60d",
    "1h":  "730d",
    "1d":  "5y",
    "1wk": "10y",
}

MIN_ROWS       = 100 # Lowered slightly for first test
MAX_RETRIES    = 3
RETRY_DELAY_S  = 2   # base delay; exponential backoff applied

def download_symbol(symbol: str, interval: str, period: str, output_dir: str, min_rows: int = MIN_ROWS) -> bool:
    """
    Download one symbol with retry logic.
    Returns True on success, False on failure/skip.
    """
    out_path = os.path.join(output_dir, f"{symbol}.csv")

    for attempt in range(1, MAX_RETRIES + 1):
        try:
            log.info(f"  [{symbol}] Attempt {attempt}/{MAX_RETRIES} — interval={interval} period={period}")

            ticker = yf.Ticker(symbol)
            df = ticker.history(period=period, interval=interval, auto_adjust=True)

            if df is None or df.empty:
                log.warning(f"  [{symbol}] No data returned (empty DataFrame). Retrying...")
                time.sleep(RETRY_DELAY_S ** attempt)
                continue

            # ── Data Cleaning ─────────────────────────────────────────────
            cols_needed = ["Open", "High", "Low", "Close", "Volume"]
            for col in cols_needed:
                if col not in df.columns:
                    log.warning(f"  [{symbol}] Missing column '{col}'. Skipping.")
                    return False

            df = df[cols_needed].copy()
            df = df.dropna()

            # Ensure positive prices
            df = df[(df["Open"] > 0) & (df["Close"] > 0) & (df["High"] > 0) & (df["Low"] > 0)]
            df = df.sort_index()
            df = df[~df.index.duplicated(keep="first")]

            # Time conversion
            if hasattr(df.index, "tz") and df.index.tz is not None:
                df.index = df.index.tz_convert("UTC")
            else:
                df.index = pd.to_datetime(df.index, utc=True)

            # Output timestamp as formatted string
            df["timestamp"] = df.index.strftime('%Y-%m-%d %H:%M:%S')

            # Rename to lowercase
            df = df.rename(columns={
                "Open":   "open",
                "High":   "high",
                "Low":    "low",
                "Close":  "close",
                "Volume": "volume",
            })

            # Ensure numeric types
            for col in ["open", "high", "low", "close", "volume"]:
                df[col] = pd.to_numeric(df[col], errors="coerce")
            df = df.dropna()

            # Final column order
            result = df[["timestamp", "open", "high", "low", "close", "volume"]].copy()

            if len(result) < min_rows:
                log.warning(f"  [{symbol}] Insufficient data: {len(result)} rows < {min_rows}. Skipping.")
                return False

            # ── Save ──────────────────────────────────────────────────────
            result.to_csv(out_path, index=False)
            log.info(f"  [{symbol}] ✅ Saved {len(result):,} rows → {out_path}")
            return True

        except Exception as exc:
            log.error(f"  [{symbol}] Error on attempt {attempt}: {exc}")
            if attempt < MAX_RETRIES:
                wait = RETRY_DELAY_S ** attempt
                log.info(f"  [{symbol}] Retrying in {wait}s...")
                time.sleep(wait)

    log.error(f"  [{symbol}] FAILED after {MAX_RETRIES} attempts. Skipping.")
    return False

def main():
    parser = argparse.ArgumentParser(description="ChronoSentiment LSE Multi-Asset Dataset Downloader")
    parser.add_argument("--interval", default="5m", choices=["1m", "5m", "15m", "30m", "1h", "1d", "1wk"])
    parser.add_argument("--period", default="60d")
    parser.add_argument("--output-dir", default=None)
    parser.add_argument("--symbols", nargs="+", default=None)
    parser.add_argument("--min-rows", type=int, default=MIN_ROWS)
    parser.add_argument("--delay", type=float, default=1.0)
    args = parser.parse_args()

    symbols    = args.symbols or LSE_SYMBOLS
    output_dir = args.output_dir or os.path.join("data", "lse", args.interval)

    os.makedirs(output_dir, exist_ok=True)

    log.info("=" * 60)
    log.info("  ChronoSentiment LSE Dataset Downloader")
    log.info("=" * 60)
    log.info(f"  Symbols  : {len(symbols)}")
    log.info(f"  Interval : {args.interval}")
    log.info(f"  Period   : {args.period}")
    log.info(f"  Output   : {output_dir}")
    log.info("=" * 60)

    success_list = []
    failed_list  = []

    for i, symbol in enumerate(symbols, 1):
        log.info(f"\n[{i:02d}/{len(symbols):02d}] {symbol}")
        ok = download_symbol(symbol, args.interval, args.period, output_dir, args.min_rows)
        if ok:
            success_list.append(symbol)
        else:
            failed_list.append(symbol)

        if i < len(symbols):
            time.sleep(args.delay)

    log.info("\n" + "=" * 60)
    log.info("  DOWNLOAD SUMMARY")
    log.info("=" * 60)
    log.info(f"  Total   : {len(symbols)}")
    log.info(f"  Success : {len(success_list)}")
    log.info(f"  Failed  : {len(failed_list)}")
    log.info("=" * 60)

    if not success_list:
        log.error("No data downloaded. Exiting with error.")
        sys.exit(1)

if __name__ == "__main__":
    main()
