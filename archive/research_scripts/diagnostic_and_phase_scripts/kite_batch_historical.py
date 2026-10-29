# scripts/kite_batch_historical.py
"""Batch acquisition of Kite historical 1‑minute data.
Uses the existing `kite_historical_minimal.py` driver for each trading day.
"""
import argparse
import sys
from pathlib import Path

# Local imports
import importlib.util

# Load the minimal capture module dynamically
spec = importlib.util.spec_from_file_location(
    "kite_historical_minimal",
    Path(__file__).parent / "kite_historical_minimal.py",
)
if spec is None:
    sys.exit("[!] Could not locate kite_historical_minimal.py")
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)  # type: ignore

# NSE calendar helper
from nse_calendar import trading_days_between


def main(symbols, start_date, end_date, output_dir):
    dates = trading_days_between(start_date, end_date)
    print(f"[INFO] Found {len(dates)} trading days between {start_date} and {end_date}")
    for date_str in dates:
        print(f"[INFO] Capturing {date_str} …")
        # The minimal script creates the correct directory layout under output_dir/date
        mod.main(symbols, date_str, output_dir)

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Batch historical capture via Kite")
    parser.add_argument("--symbols", nargs="+", required=True, help="Symbols to capture (e.g., NIFTY BANKNIFTY)")
    parser.add_argument("--start-date", required=True, help="Start date YYYY-MM-DD")
    parser.add_argument("--end-date", required=True, help="End date YYYY-MM-DD")
    parser.add_argument("--output-dir", default="historical_capture/batch", help="Root output directory")
    args = parser.parse_args()
    main(args.symbols, args.start_date, args.end_date, args.output_dir)
