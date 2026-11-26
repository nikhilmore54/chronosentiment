#!/usr/bin/env python3
"""Minimal timestamp unit verification for ChronoSentiment live captures.

Usage:
    python3 scripts/check_timestamp_units.py <path-to-sample>.jsonl

The script reads the first 5 lines of the provided JSONL file and checks:
* Each record contains a numeric ``timestamp`` (or ``ts``) field.
* The timestamp is expressed in **milliseconds** (13‑digit epoch value).
* The timestamps are monotonically increasing.

If any check fails, a concise warning is printed and the script exits with status 1.
Otherwise it prints a short success summary and exits with 0.
"""
import sys
import json

def extract_timestamp(obj):
    # The capture files may use ``timestamp`` or ``ts`` as the key.
    for key in ("timestamp", "ts"):
        if key in obj:
            return obj[key]
    return None

def main():
    if len(sys.argv) != 2:
        print("Usage: check_timestamp_units.py <path-to-sample>.jsonl")
        sys.exit(2)
    path = sys.argv[1]
    try:
        with open(path, "r", encoding="utf-8") as f:
            lines = [next(f) for _ in range(5)]
    except StopIteration:
        print("File has fewer than 5 lines – cannot verify monotonicity.")
        sys.exit(1)
    except Exception as e:
        print(f"Error reading file: {e}")
        sys.exit(1)

    timestamps = []
    for i, line in enumerate(lines, start=1):
        try:
            obj = json.loads(line)
        except json.JSONDecodeError as e:
            print(f"Line {i} is not valid JSON: {e}")
            sys.exit(1)
        ts = extract_timestamp(obj)
        if ts is None:
            print(f"Line {i} missing timestamp field.")
            sys.exit(1)
        if not isinstance(ts, (int, float)):
            print(f"Line {i} timestamp is not numeric: {ts}")
            sys.exit(1)
        # Millisecond epoch should be at least 13 digits (> 10^12)
        if ts < 1_000_000_000_000:
            print(f"Line {i} timestamp appears to be seconds (value {ts}). Expected milliseconds.")
            sys.exit(1)
        timestamps.append(int(ts))
        # Show a preview of the record for manual inspection
        print(f"Line {i}: timestamp={ts}, symbol={obj.get('symbol', obj.get('pair', 'N/A'))}")

    # Verify monotonic increasing order
    for earlier, later in zip(timestamps, timestamps[1:]):
        if later <= earlier:
            print("Timestamps are not strictly increasing.")
            sys.exit(1)
    print("✅ Timestamp unit verification passed (millisecond, monotonic, 5 lines inspected).")
    sys.exit(0)

if __name__ == "__main__":
    main()
