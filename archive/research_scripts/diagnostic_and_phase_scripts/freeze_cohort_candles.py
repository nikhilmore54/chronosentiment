#!/usr/bin/env python3
"""
Freeze yfinance OHLC for a cohort batch into an immutable local substrate.

Usage:
  python3 scripts/freeze_cohort_candles.py --batch-id 3
  python3 scripts/run_nse_cohort.py --batch-id 3 --from-frozen
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from candle_substrate import freeze_cohort


def main():
    parser = argparse.ArgumentParser(description="Freeze cohort OHLC candles for deterministic replay")
    parser.add_argument("--batch-id", type=int, required=True)
    parser.add_argument("--interval", default="5m")
    parser.add_argument("--period", default="5d")
    parser.add_argument(
        "--max-workers",
        type=int,
        default=15,
        help="Parallel download workers (use 1-2 for crypto to avoid yfinance rate limits)",
    )
    args = parser.parse_args()

    cohort = Path(f"cohorts/batch_{args.batch_id:03d}.txt")
    if not cohort.exists():
        print(f"❌ {cohort} not found", file=sys.stderr)
        sys.exit(1)

    freeze_cohort(
        cohort, args.batch_id, args.interval, args.period, max_workers=args.max_workers
    )


if __name__ == "__main__":
    main()
