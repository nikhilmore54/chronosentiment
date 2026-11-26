#!/usr/bin/env python3
"""
live_fetch_to_inbox.py

Polls Yahoo Finance 1-minute bars for all tickers in a LIVE-005 dataset
and appends new observations to a JSONL inbox file consumable by the Rust
deferred_live_decision_loop --inbox mode.

ObservationLine format (one JSON object per line):
  {"ticker": "AXISBANK_NS", "unix": 1789703100, "price": 1247.6, "high": 1248.0, "low": 1246.5}

Usage:
  python3 scripts/live_fetch_to_inbox.py \\
    --dataset datasets/live005_20260922.json \\
    --out /tmp/live_obs.jsonl \\
    [--interval 30]   # poll interval in seconds (default 30)
    [--once]          # single fetch then exit (for smoke-testing)
    [--date 2026-09-22]  # IST date filter (default: today)
"""
import argparse
import datetime
import json
import os
import sys
import time

try:
    import yfinance as yf
except ImportError:
    print("yfinance not installed. Run: pip install yfinance", file=sys.stderr)
    sys.exit(1)

# NSE IST market hours
MARKET_OPEN_IST  = datetime.time(9, 15)
MARKET_CLOSE_IST = datetime.time(15, 30)
IST = datetime.timezone(datetime.timedelta(hours=5, minutes=30))


def brief_ticker_to_yahoo(ticker: str) -> str:
    """AXISBANK_NS → AXISBANK.NS"""
    return ticker.replace("_NS", ".NS").replace("_BO", ".BO")


def yahoo_to_brief_ticker(symbol: str) -> str:
    """AXISBANK.NS → AXISBANK_NS"""
    return symbol.replace(".NS", "_NS").replace(".BO", "_BO")


def load_tickers(dataset_path: str) -> list[str]:
    """Extract unique brief tickers from a live005 dataset JSON."""
    with open(dataset_path) as f:
        records = json.load(f)
    if isinstance(records, list):
        tickers = sorted(set(r["ticker"] for r in records if r.get("ticker")))
    elif isinstance(records, dict):
        entries = records.get("positions", records.get("decisions", records.get("briefs", [])))
        tickers = sorted(set(r["ticker"] for r in entries if r.get("ticker")))
    else:
        tickers = []
    return tickers


def ist_date_str(dt: datetime.datetime) -> str:
    return dt.astimezone(IST).strftime("%Y-%m-%d")


def is_market_open() -> bool:
    now_ist = datetime.datetime.now(IST)
    t = now_ist.time()
    return MARKET_OPEN_IST <= t <= MARKET_CLOSE_IST


def fetch_new_bars(
    tickers: list[str],
    last_unix: dict[str, int],
    date_str: str,
    delay_between: float = 0.3,
) -> list[dict]:
    """
    For each ticker, fetch latest 1m bars via yfinance.
    Returns list of ObservationLine dicts for bars newer than last_unix[ticker]
    that fall on date_str (IST).
    """
    new_obs = []
    for i, brief_ticker in enumerate(tickers):
        if i > 0:
            time.sleep(delay_between)
        yahoo_sym = brief_ticker_to_yahoo(brief_ticker)
        try:
            tk = yf.Ticker(yahoo_sym)
            df = tk.history(period="1d", interval="1m", auto_adjust=False)
            if df is None or df.empty:
                continue
            after_unix = last_unix.get(brief_ticker, 0)
            for ts_idx, row in df.iterrows():
                # Ensure timezone-aware → unix
                if hasattr(ts_idx, "timestamp"):
                    ts_unix = int(ts_idx.timestamp())
                else:
                    ts_unix = int(ts_idx.to_pydatetime().timestamp())
                # Filter: must be on the target IST date and newer than last seen
                bar_dt = datetime.datetime.fromtimestamp(ts_unix, tz=IST)
                if bar_dt.strftime("%Y-%m-%d") != date_str:
                    continue
                if ts_unix <= after_unix:
                    continue
                # Valid new bar
                obs = {
                    "ticker": brief_ticker,
                    "unix": ts_unix,
                    "price": float(row.get("Close", 0.0)),
                    "high": float(row.get("High", 0.0)),
                    "low": float(row.get("Low", 0.0)),
                }
                if obs["price"] <= 0.0:
                    continue
                new_obs.append(obs)
                if ts_unix > last_unix.get(brief_ticker, 0):
                    last_unix[brief_ticker] = ts_unix
        except Exception as e:
            print(f"  [{brief_ticker}] fetch error: {e}", file=sys.stderr)
    # Sort by (unix, ticker) — same order as cached_session_tape
    new_obs.sort(key=lambda o: (o["unix"], o["ticker"]))
    return new_obs


def append_to_inbox(obs_list: list[dict], out_path: str) -> int:
    """Append observations as JSONL to the inbox file. Returns count written."""
    if not obs_list:
        return 0
    with open(out_path, "a") as f:
        for obs in obs_list:
            f.write(json.dumps(obs) + "\n")
    return len(obs_list)


def main():
    ap = argparse.ArgumentParser(description="Yahoo 1m live fetcher → JSONL inbox")
    ap.add_argument("--dataset", required=True, help="Path to live005 dataset JSON")
    ap.add_argument("--out", required=True, help="Output JSONL inbox path")
    ap.add_argument("--interval", type=int, default=30, help="Poll interval in seconds")
    ap.add_argument("--date", default=None, help="IST session date YYYY-MM-DD (default: today)")
    ap.add_argument("--once", action="store_true", help="Fetch once and exit")
    ap.add_argument("--delay", type=float, default=0.3, help="Delay between per-ticker fetches (s)")
    args = ap.parse_args()

    tickers = load_tickers(args.dataset)
    if not tickers:
        print("ERROR: no tickers found in dataset", file=sys.stderr)
        sys.exit(1)

    date_str = args.date or datetime.datetime.now(IST).strftime("%Y-%m-%d")
    print(f"[sidecar] Session date: {date_str}")
    print(f"[sidecar] Tickers: {len(tickers)}")
    print(f"[sidecar] Output: {args.out}")
    print(f"[sidecar] Poll interval: {args.interval}s")

    # Ensure output directory exists
    os.makedirs(os.path.dirname(os.path.abspath(args.out)), exist_ok=True)

    last_unix: dict[str, int] = {}
    round_num = 0

    while True:
        round_num += 1
        now_ist = datetime.datetime.now(IST)
        print(f"\n[sidecar] Round {round_num} at {now_ist.strftime('%H:%M:%S IST')} ...", flush=True)

        new_obs = fetch_new_bars(tickers, last_unix, date_str, delay_between=args.delay)
        written = append_to_inbox(new_obs, args.out)

        if new_obs:
            first_ts = datetime.datetime.fromtimestamp(new_obs[0]["unix"], tz=IST).strftime("%H:%M")
            last_ts  = datetime.datetime.fromtimestamp(new_obs[-1]["unix"], tz=IST).strftime("%H:%M")
            print(f"[sidecar] +{written} observations  bars {first_ts}→{last_ts}", flush=True)
        else:
            print("[sidecar] No new bars this round.", flush=True)

        if args.once:
            print(f"[sidecar] --once: exiting after round 1. Total written: {written}")
            break

        # Check if market is closed — stop polling after close
        now_ist = datetime.datetime.now(IST)
        if now_ist.time() > MARKET_CLOSE_IST:
            print("[sidecar] Market closed (past 15:30 IST). Exiting.")
            break

        time.sleep(args.interval)


if __name__ == "__main__":
    main()
