#!/usr/bin/env python3
"""
Deterministic Yahoo polling loop that emits engine-compatible JSON batches.

Usage example:
  python3 scripts/fetch_stream_loop.py --symbols "BTC-USD,ETH-USD" --interval 1m --cadence-seconds 2
"""

from __future__ import annotations

import argparse
import json
import sys
import time
from datetime import datetime
from pathlib import Path

_ROOT = Path(__file__).resolve().parents[1]
if str(_ROOT) not in sys.path:
    sys.path.insert(0, str(_ROOT))

from scripts.fetch_candles import fetch_latest


def parse_args() -> argparse.Namespace:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--symbols", required=True, help="Comma-separated Yahoo symbols")
    ap.add_argument("--interval", default="1m", help="Yahoo interval (default: 1m)")
    ap.add_argument("--n-candles", type=int, default=3, help="Candles per fetch call")
    ap.add_argument("--cadence-seconds", type=float, default=2.0, help="Polling cadence")
    ap.add_argument(
        "--max-steps",
        type=int,
        default=0,
        help="Optional step limit for deterministic finite runs (0 = unbounded)",
    )
    return ap.parse_args()


def main() -> int:
    args = parse_args()
    symbols = [s.strip() for s in args.symbols.split(",") if s.strip()]
    if not symbols:
        print("[STREAM] no symbols provided", file=sys.stderr)
        return 1

    last_ts_by_symbol: dict[str, int] = {s: 0 for s in symbols}
    steps = 0

    print(
        f"[STREAM] start symbols={symbols} interval={args.interval} cadence={args.cadence_seconds}s",
        file=sys.stderr,
    )

    first_run = True
    while True:
        if args.max_steps > 0 and steps >= args.max_steps:
            print(f"[STREAM] stop max_steps={args.max_steps}", file=sys.stderr)
            return 0

        if first_run:
            # Warmup: Emit all fetched candles to fill engine history
            for symbol in symbols:
                try:
                    candles = fetch_latest(symbol, args.interval, args.n_candles)
                    candles = sorted(candles, key=lambda c: int(c.get("timestamp", 0)))
                    for c in candles:
                        ts = int(c["timestamp"])
                        if ts <= last_ts_by_symbol[symbol]:
                            continue
                        last_ts_by_symbol[symbol] = ts
                        batch_item = {
                            "symbol": symbol,
                            "timestamp": ts,
                            "open": float(c["open"]),
                            "high": float(c["high"]),
                            "low": float(c["low"]),
                            "close": float(c["close"]),
                            "volume": float(c["volume"]),
                        }
                        # Emit individual candle for warmup processing
                        print(json.dumps([batch_item]), flush=True)
                except Exception as e:
                    print(f"[STREAM] warmup_error symbol={symbol} err={e}", file=sys.stderr)
            first_run = False
            print(f"[STREAM] warmup complete", file=sys.stderr, flush=True)
            continue

        batch: list[dict] = []
        for symbol in symbols:
            try:
                candles = fetch_latest(symbol, args.interval, 3) # Regular polling
            except Exception as e:
                print(f"[STREAM] fetch_error symbol={symbol} err={e}", file=sys.stderr)
                continue

            candles = sorted(candles, key=lambda c: int(c.get("timestamp", 0)))
            if not candles:
                continue
            latest = candles[-1]
            try:
                latest_ts = int(latest["timestamp"])
            except (TypeError, ValueError, KeyError):
                continue
            prev_ts = int(last_ts_by_symbol.get(symbol, 0))
            if latest_ts > prev_ts:
                status = "new_candle"
                last_ts_by_symbol[symbol] = latest_ts
                batch.append(
                    {
                        "symbol": symbol,
                        "timestamp": latest_ts,
                        "open": float(latest["open"]),
                        "high": float(latest["high"]),
                        "low": float(latest["low"]),
                        "close": float(latest["close"]),
                        "volume": float(latest["volume"]),
                    }
                )
            else:
                status = "same_candle_snapshot"
            
            # (Optional) Log status
            # print(f"[STREAM] symbol={symbol} ts={latest_ts} status={status}", file=sys.stderr, flush=True)

        now = datetime.now().strftime("%H:%M:%S")
        if batch:
            # One line == one synchronized timestep input for live_engine.
            print(json.dumps(batch), flush=True)
            print(
                f"[STREAM] emitted symbols={len(batch)} at={now}",
                file=sys.stderr,
                flush=True,
            )
        else:
            print(f"[STREAM] no_data_at_source at={now}", file=sys.stderr, flush=True)

        steps += 1
        time.sleep(args.cadence_seconds)


if __name__ == "__main__":
    raise SystemExit(main())
