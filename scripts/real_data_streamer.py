import csv
import glob
import json
import os
import sys
import time
from datetime import datetime


DEFAULT_GLOB = "data/nse/5m/*.csv"
DEFAULT_MAX_SYMBOLS = 5
SLEEP_SECONDS = 0.001
DEFAULT_OFFSET = 0
DEFAULT_LIMIT = 0


def parse_ts(ts_str: str) -> int:
    try:
        return int(datetime.strptime(ts_str, "%Y-%m-%d %H:%M:%S").timestamp())
    except Exception:
        return 0


def load_readers(path_glob: str, max_symbols: int):
    files = sorted(glob.glob(path_glob))
    if not files:
        return []

    readers = []
    for path in files[:max_symbols]:
        handle = open(path, "r", newline="")
        readers.append(
            {
                "symbol": os.path.basename(path).replace(".csv", ""),
                "reader": csv.DictReader(handle),
                "handle": handle,
            }
        )
    return readers


def row_to_candle(symbol: str, row: dict):
    timestamp = row.get("timestamp", row.get("date", ""))
    return {
        "symbol": symbol,
        "timestamp": parse_ts(timestamp),
        "open": float(row.get("open", row.get("close", 0.0))),
        "high": float(row.get("high", row.get("close", 0.0))),
        "low": float(row.get("low", row.get("close", 0.0))),
        "close": float(row.get("close", 0.0)),
        "volume": float(row.get("volume", 0.0)),
    }


def main():
    path_glob = os.environ.get("REAL_STREAM_GLOB", DEFAULT_GLOB)
    max_symbols = int(os.environ.get("REAL_STREAM_SYMBOLS", str(DEFAULT_MAX_SYMBOLS)))
    offset = int(os.environ.get("REAL_STREAM_OFFSET", str(DEFAULT_OFFSET)))
    limit = int(os.environ.get("REAL_STREAM_LIMIT", str(DEFAULT_LIMIT)))
    readers = load_readers(path_glob, max_symbols)
    if not readers:
        print(f"[REAL_STREAMER] no files matched glob={path_glob}", file=sys.stderr)
        return

    print(
        f"[REAL_STREAMER] mode=historical_csv glob={path_glob} symbols={len(readers)} offset={offset} limit={limit}",
        file=sys.stderr,
    )

    emitted = 0
    try:
        while True:
            batch = []
            for item in readers:
                try:
                    row = next(item["reader"])
                except StopIteration:
                    continue
                batch.append(row_to_candle(item["symbol"], row))

            if not batch:
                break

            if offset > 0:
                offset -= 1
                continue

            if limit > 0 and emitted >= limit:
                break

            print(json.dumps(batch), flush=True)
            emitted += 1
            time.sleep(SLEEP_SECONDS)
    finally:
        for item in readers:
            item["handle"].close()


if __name__ == "__main__":
    main()
