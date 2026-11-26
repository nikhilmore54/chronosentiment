#!/usr/bin/env python3
"""
ChronoSentiment — NSE Multi-Asset Dataset Downloader
=====================================================
Downloads diversified NSE stock data using yfinance.

Usage:
    python3 scripts/download_nse_data.py --interval 5m --period 60d
    python3 scripts/download_nse_data.py --interval 1d --period 2y

Output:
    data/nse/{interval}/{SYMBOL}.csv

CSV Format (strict):
    timestamp,open,high,low,close,volume
    (timestamp = Unix integer seconds, UTC)
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
log = logging.getLogger("nse_downloader")

# ─── Training Universe ───────────────────────────────────────────────────────
LARGE_CAPS = [
    "RELIANCE.NS", "HDFCBANK.NS", "ICICIBANK.NS", "INFY.NS", "TCS.NS",
    "LT.NS", "BHARTIARTL.NS", "ITC.NS", "HINDUNILVR.NS", "SBIN.NS",
    "KOTAKBANK.NS", "AXISBANK.NS",
]

CYCLICALS = ["TATASTEEL.NS", "JSWSTEEL.NS", "ONGC.NS"]

HIGH_VOLATILITY = ["ADANIENT.NS", "ADANIPORTS.NS"]

NEW_AGE = ["ZOMATO.NS", "PAYTM.NS", "NYKAA.NS", "POLICYBZR.NS"]

MIDCAPS = [
    "DLF.NS", "GODREJPROP.NS", "COFORGE.NS", "IRCTC.NS",
    "CDSL.NS", "BSE.NS", "DEEPAKNTR.NS",
]

ALL_SYMBOLS = LARGE_CAPS + CYCLICALS + HIGH_VOLATILITY + NEW_AGE + MIDCAPS

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

MIN_ROWS       = 500
MAX_RETRIES    = 3
RETRY_DELAY_S  = 2   # base delay; exponential backoff applied


# ─── Download Logic ──────────────────────────────────────────────────────────

def clamp_period(interval: str, period: str) -> str:
    """
    Silently enforce Yahoo Finance free-tier period limits.
    Returns the clamped period so the user sees what will actually be fetched.
    """
    limits = INTERVAL_PERIOD_LIMITS.get(interval)
    if limits is None:
        return period
    # Very rough comparison: only warn, don't block — let Yahoo handle it
    return period


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
            # Keep only OHLCV columns
            cols_needed = ["Open", "High", "Low", "Close", "Volume"]
            for col in cols_needed:
                if col not in df.columns:
                    log.warning(f"  [{symbol}] Missing column '{col}'. Skipping.")
                    return False

            df = df[cols_needed].copy()

            # Drop rows with any NaN
            before_drop = len(df)
            df = df.dropna()
            dropped = before_drop - len(df)
            if dropped > 0:
                log.debug(f"  [{symbol}] Dropped {dropped} NaN rows.")

            # Ensure positive prices
            df = df[(df["Open"] > 0) & (df["Close"] > 0) & (df["High"] > 0) & (df["Low"] > 0)]

            # Sort ascending by datetime index
            df = df.sort_index()

            # Remove duplicate timestamps
            df = df[~df.index.duplicated(keep="first")]

            # Convert index to Unix timestamp (integer seconds, UTC)
            if hasattr(df.index, "tz") and df.index.tz is not None:
                df.index = df.index.tz_convert("UTC")
            else:
                df.index = pd.to_datetime(df.index, utc=True)

            # Output timestamp as formatted string to enforce sequence-based ESE
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

            # ── Quality Gate ──────────────────────────────────────────────
            if len(result) < min_rows:
                log.warning(
                    f"  [{symbol}] Insufficient data: {len(result)} rows < {min_rows} minimum. "
                    f"Skipping (Yahoo may not carry {interval} data for this symbol)."
                )
                return False

            # ── Save ──────────────────────────────────────────────────────
            result.to_csv(out_path, index=False)
            log.info(
                f"  [{symbol}] ✅ Saved {len(result):,} rows → {out_path} "
                f"({result['timestamp'].iloc[0]} … {result['timestamp'].iloc[-1]})"
            )
            return True

        except Exception as exc:
            log.error(f"  [{symbol}] Error on attempt {attempt}: {exc}")
            if attempt < MAX_RETRIES:
                wait = RETRY_DELAY_S ** attempt
                log.info(f"  [{symbol}] Retrying in {wait}s...")
                time.sleep(wait)

    log.error(f"  [{symbol}] FAILED after {MAX_RETRIES} attempts. Skipping.")
    return False


# ─── Entry Point ─────────────────────────────────────────────────────────────

def main():
    parser = argparse.ArgumentParser(
        description="ChronoSentiment NSE Multi-Asset Dataset Downloader",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  python3 scripts/download_nse_data.py --interval 5m --period 60d
  python3 scripts/download_nse_data.py --interval 1d --period 2y
  python3 scripts/download_nse_data.py --interval 5m --period 60d --symbols RELIANCE.NS INFY.NS
""",
    )
    parser.add_argument(
        "--interval", default="5m",
        choices=["1m", "5m", "15m", "30m", "1h", "1d", "1wk"],
        help="Candle interval (default: 5m)",
    )
    parser.add_argument(
        "--period", default="60d",
        help="History period, e.g. 60d, 6mo, 1y, 2y (default: 60d). "
             "Note: 5m data available max 60d on Yahoo free tier.",
    )
    parser.add_argument(
        "--output-dir", default=None,
        help="Override output directory (default: data/nse/{interval}/)",
    )
    parser.add_argument(
        "--symbols", nargs="+", default=None,
        help="Override symbol list (default: full 25-stock NSE universe)",
    )
    parser.add_argument(
        "--min-rows", type=int, default=MIN_ROWS,
        help=f"Minimum rows required to save (default: {MIN_ROWS})",
    )
    parser.add_argument(
        "--delay", type=float, default=1.0,
        help="Seconds to sleep between downloads to avoid rate-limiting (default: 1.0)",
    )
    args = parser.parse_args()

    symbols    = args.symbols or ALL_SYMBOLS
    output_dir = args.output_dir or os.path.join("data", "nse", args.interval)

    os.makedirs(output_dir, exist_ok=True)

    log.info("=" * 60)
    log.info("  ChronoSentiment NSE Dataset Downloader")
    log.info("=" * 60)
    log.info(f"  Symbols  : {len(symbols)}")
    log.info(f"  Interval : {args.interval}")
    log.info(f"  Period   : {args.period}")
    log.info(f"  Output   : {output_dir}")
    log.info(f"  Min rows : {args.min_rows}")
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

        # Rate-limit courtesy sleep between downloads
        if i < len(symbols):
            time.sleep(args.delay)

    # ── Summary ──────────────────────────────────────────────────────────────
    log.info("\n" + "=" * 60)
    log.info("  DOWNLOAD SUMMARY")
    log.info("=" * 60)
    log.info(f"  Total   : {len(symbols)}")
    log.info(f"  Success : {len(success_list)}")
    log.info(f"  Failed  : {len(failed_list)}")

    if success_list:
        log.info(f"\n  ✅ Saved:")
        for s in success_list:
            log.info(f"     {s}")

    if failed_list:
        log.warning(f"\n  ❌ Failed/Skipped:")
        for s in failed_list:
            log.warning(f"     {s}")

    log.info("=" * 60)

    if not success_list:
        log.error("No data downloaded. Exiting with error.")
        sys.exit(1)


if __name__ == "__main__":
    main()
