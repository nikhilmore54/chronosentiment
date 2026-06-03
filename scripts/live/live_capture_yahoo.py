#!/usr/bin/env python3
"""
Live 1‑minute capture from Yahoo Finance (NIFTY & BANKNIFTY)
================================================================
Purpose:
  - Continuously capture the current trading day 1‑minute OHLCV candles for the
    NIFTY and BANKNIFTY indices while the market is open.
  - Preserve the raw Yahoo JSON response for provenance and hash it.
  - Store a canonical 1‑minute CSV (unchanged) and an internally derived
    5‑minute CSV, each with its own SHA‑256 hash.
  - Emit a `capture_manifest.json` containing the full reconstruction chain:
    raw JSON hash → canonical 1‑minute hash → derived 5‑minute hash, plus
    session metadata.

The script is *not* part of the Phase 1B‑2.75 replay‑corpus; it simply
starts building live material that can later be incorporated into the replay
corpus after the normal candidate‑registry / availability‑audit pipeline.

Usage:
  python3 scripts/live/live_capture_yahoo.py [--symbols NIFTY BANKNIFTY] \\
      [--duration MINUTES] [--output-dir live_capture]

Options:
  --duration   Number of minutes to run the capture loop. Omit for "run until
                market close" (the script will stop when Yahoo no longer returns
                new data).
  --output-dir Directory where the session folder will be created.

Dependencies:
  - yfinance
  - pandas
  - hashlib
"""

import argparse
import datetime
import hashlib
import json
import time
import uuid
from pathlib import Path

import pandas as pd
import yfinance as yf

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------
YF_SYMBOLS = {
    "NIFTY": "^NSEI",
    "BANKNIFTY": "^NSEBANK",
}
SCRIPT_VERSION = "1.1.0"

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(8192), b""):
            h.update(chunk)
    return h.hexdigest()


def derive_5m(df: pd.DataFrame) -> pd.DataFrame:
    """Resample a 1‑minute DataFrame to 5‑minute OHLCV.
    Expected columns: timestamp, open, high, low, close, volume.
    """
    df = df.set_index("timestamp")
    df.index = pd.to_datetime(df.index, utc=True)
    # Use "5min" instead of "5T" for compatibility with the pandas version.
    resampled = df.resample("5min").agg({
        "open": "first",
        "high": "max",
        "low": "min",
        "close": "last",
        "volume": "sum",
    })
    resampled = resampled.dropna().reset_index()
    resampled["timestamp"] = resampled["timestamp"].dt.strftime("%Y-%m-%d %H:%M:%S")
    return resampled


def fetch_symbol(sym: str) -> tuple[dict, pd.DataFrame, str]:
    """Download the latest 1‑minute data for *sym*.
    Returns raw JSON payload, a DataFrame with UTC timestamps, and the SHA‑256 of the raw JSON.
    """
    yf_sym = YF_SYMBOLS.get(sym, sym)
    ticker = yf.Ticker(yf_sym)
    try:
        df = ticker.history(period="1d", interval="1m", auto_adjust=True)
    except Exception as e:
        print(f"[{sym}] Yahoo fetch error: {e}")
        df = pd.DataFrame()
    if df.empty:
        empty_json = {"symbol": yf_sym, "records": []}
        return empty_json, pd.DataFrame(), sha256_bytes(json.dumps(empty_json).encode())
    # Reset index to get the datetime column as a regular column.
    df_reset = df.reset_index()
    # Identify the timestamp column (usually "Datetime").
    timestamp_col = next(
        (c for c in df_reset.columns if str(c).lower() in {"datetime", "date", "timestamp"}),
        None,
    )
    if timestamp_col is None:
        raise ValueError(f"Timestamp column not found for {sym}")
    # Convert timestamps to ISO strings for JSON serialisation.
    df_reset[timestamp_col] = df_reset[timestamp_col].apply(lambda x: x.isoformat())
    raw_records = df_reset.to_dict(orient="records")
    raw_json = {"symbol": yf_sym, "records": raw_records}
    raw_bytes = json.dumps(raw_json, separators=(",", ":"), ensure_ascii=False).encode("utf-8")
    raw_hash = sha256_bytes(raw_bytes)
    # Build DataFrame with proper UTC timestamps for further processing.
    df = df_reset.copy()
    df["timestamp"] = pd.to_datetime(df_reset[timestamp_col], utc=True)
    df = df[["timestamp", "Open", "High", "Low", "Close", "Volume"]].copy()
    df.columns = ["timestamp", "open", "high", "low", "close", "volume"]
    return raw_json, df, raw_hash

# ---------------------------------------------------------------------------
# Main capture routine
# ---------------------------------------------------------------------------

def capture(symbols, output_root: Path, duration_minutes: int | None):
    today_str = datetime.datetime.now().strftime("%Y-%m-%d")
    session_id = f"NSE_{datetime.datetime.utcnow().strftime('%Y_%m_%d')}"
    session_dir = output_root / today_str
    raw_dir = session_dir / "raw"
    canonical_dir = session_dir / "canonical"
    derived_dir = session_dir / "derived"
    for d in (raw_dir, canonical_dir, derived_dir):
        d.mkdir(parents=True, exist_ok=True)

    manifest = {
        "session_id": session_id,
        "capture_date": today_str,
        "capture_start_utc": datetime.datetime.utcnow().isoformat() + "Z",
        "capture_end_utc": None,
        "script_version": SCRIPT_VERSION,
        "provider": "Yahoo Finance",
        "symbols": {},
    }

    # Track last timestamp per symbol for incremental fetching.
    last_timestamp: dict[str, pd.Timestamp | None] = {sym: None for sym in symbols}

    start_time = time.time()
    while True:
        for sym in symbols:
            raw_json, df, raw_hash = fetch_symbol(sym)
            if df.empty:
                continue
            # Keep only new rows.
            if last_timestamp[sym] is not None:
                df = df[df["timestamp"] > last_timestamp[sym]]
                if df.empty:
                    continue
            last_timestamp[sym] = df["timestamp"].max()

            # Write raw JSON.
            raw_path = raw_dir / f"{sym}.json"
            with raw_path.open("w", encoding="utf-8") as f:
                json.dump(raw_json, f, indent=2)

            # Append to canonical CSV.
            canonical_path = canonical_dir / f"{sym}_1m.csv"
            write_header = not canonical_path.exists()
            df_to_write = df[["timestamp", "open", "high", "low", "close", "volume"]]
            df_to_write.to_csv(canonical_path, mode="a", header=write_header, index=False)
            canonical_hash = sha256_file(canonical_path)

            # Derive 5‑minute CSV from full canonical data.
            full_canonical = pd.read_csv(canonical_path)
            derived_df = derive_5m(full_canonical)
            derived_path = derived_dir / f"{sym}_5m.csv"
            derived_df.to_csv(derived_path, index=False)
            derived_hash = sha256_file(derived_path)

            # Update manifest entry for this symbol.
            manifest["symbols"][sym] = {
                "provider_symbol": YF_SYMBOLS.get(sym, sym),
                "raw_json_path": str(raw_path),
                "raw_json_sha256": raw_hash,
                "canonical_1m_path": str(canonical_path),
                "canonical_1m_sha256": canonical_hash,
                "derived_5m_path": str(derived_path),
                "derived_5m_sha256": derived_hash,
                "retrieval_method": "yfinance",
                "capture_timestamp_utc": datetime.datetime.utcnow().isoformat() + "Z",
                "script_version": SCRIPT_VERSION,
            }
            print(f"[{sym}] captured {len(df)} new rows – session {session_id}")

        # Loop termination conditions
        if duration_minutes is not None:
            elapsed = (time.time() - start_time) / 60
            if elapsed >= duration_minutes:
                break
        else:
            now_utc = datetime.datetime.now(datetime.timezone.utc)
            latest_ts = max(
                pd.to_datetime(pd.read_csv(canonical_dir / f"{s}_1m.csv")["timestamp"], utc=True).max()
                for s in symbols
                if (canonical_dir / f"{s}_1m.csv").exists()
            )
            if (now_utc - latest_ts).total_seconds() > 120:
                print("No new data for >2 min – assuming market closed. Exiting loop.")
                break
        # Sleep until next minute boundary
        time_to_next_minute = 60 - datetime.datetime.utcnow().second
        time.sleep(time_to_next_minute)

    manifest["capture_end_utc"] = datetime.datetime.utcnow().isoformat() + "Z"
    manifest_path = session_dir / "capture_manifest.json"
    with manifest_path.open("w", encoding="utf-8") as f:
        json.dump(manifest, f, indent=2)
    print(f"Capture manifest written to {manifest_path}")


def main():
    parser = argparse.ArgumentParser(description="Live 1‑minute capture from Yahoo Finance")
    parser.add_argument("--symbols", nargs="+", default=["NIFTY", "BANKNIFTY"],
                        help="Symbols to capture (default: NIFTY BANKNIFTY)")
    parser.add_argument("--duration", type=int, default=None,
                        help="Number of minutes to run the capture loop (optional)")
    parser.add_argument("--output-dir", default="live_capture",
                        help="Root directory for session data")
    args = parser.parse_args()
    output_root = Path(args.output_dir)
    capture(args.symbols, output_root, args.duration)

if __name__ == "__main__":
    main()
