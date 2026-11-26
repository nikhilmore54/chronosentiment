#!/usr/bin/env python3
"""
RP-608 — Kite historical candle backfill for NSE 119 (Pi credentials).

Authority 0%. Uses ~/.kite_credentials.json via KiteDataFeedAdapter.

IMPORTANT: Kite Connect historical API does NOT provide tick-by-tick history.
Only OHLCV candles (minute … day). This script backfills 15minute (default)
and optionally minute candles for CLOSE_GRID / Gate 2X path research.

Writes research artifact JSON (does not overwrite live daemon bar_history
unless --install-live-bar-history is set).
"""

from __future__ import annotations

import argparse
import json
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from collections import defaultdict
from datetime import date, datetime, timedelta, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional, Sequence, Set, Tuple

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT))

from scripts.run_rp608_nse_shadow_daemon import (  # noqa: E402
    KiteAuthError,
    KiteDataFeedAdapter,
    KiteRateLimitError,
    NSE_119_SYMBOLS,
)

# Historical endpoint soft-limit ≈3 req/s; stay under to avoid 429 storms.
HIST_THROTTLE_SEC = float(
    __import__("os").environ.get("RP608_KITE_HIST_THROTTLE_SEC", "0.45")
)

IST = timezone(timedelta(hours=5, minutes=30))
# Kite Connect historical request windows (Zerodha forum / community docs):
# minute≈60d, 3/5/10m≈100d, 15/30m≈200d, 60m≈400d, day≈2000d per request.
MAX_SPAN_DAYS = {
    "minute": 60,
    "5minute": 100,
    "15minute": 200,
    "30minute": 200,
    "60minute": 400,
    "day": 2000,
}


def _parse_day(s: str) -> date:
    return date.fromisoformat(s)


def _iter_chunks(d0: date, d1: date, max_days: int) -> List[Tuple[date, date]]:
    out: List[Tuple[date, date]] = []
    cur = d0
    while cur <= d1:
        end = min(cur + timedelta(days=max_days - 1), d1)
        out.append((cur, end))
        cur = end + timedelta(days=1)
    return out


def _session_window_utc(day: date) -> Tuple[datetime, datetime]:
    start = datetime(day.year, day.month, day.day, 9, 15, tzinfo=IST).astimezone(timezone.utc)
    end = datetime(day.year, day.month, day.day, 15, 30, tzinfo=IST).astimezone(timezone.utc)
    return start, end


def _bar_to_dict(b: Any) -> Dict[str, Any]:
    return {
        "timestamp_utc_ms": int(b.timestamp_utc_ms),
        "open": float(b.open),
        "high": float(b.high),
        "low": float(b.low),
        "close": float(b.close),
        "volume": float(b.volume),
        "is_completed": True,
        "source": "kite_historical_backfill",
    }


def _fetch_interval(
    adapter: KiteDataFeedAdapter,
    *,
    symbol: str,
    token: int,
    interval: str,
    from_utc: datetime,
    to_utc: datetime,
) -> List[Dict[str, Any]]:
    """
    Generic historical candle fetch (extends adapter's 15m helper for other intervals).
    """
    adapter.reload_credentials_if_stale()
    if not adapter.access_token or not adapter.api_key:
        raise KiteAuthError("MISSING_KITE_ACCESS_TOKEN")

    from_ist = from_utc.astimezone(IST)
    to_ist = to_utc.astimezone(IST)
    query_str = urllib.parse.urlencode(
        {
            "from": from_ist.strftime("%Y-%m-%d %H:%M:%S"),
            "to": to_ist.strftime("%Y-%m-%d %H:%M:%S"),
        }
    )
    url = f"https://api.kite.trade/instruments/historical/{token}/{interval}?{query_str}"

    def _once() -> List[Any]:
        # Local throttle (do not rely solely on adapter; max-history runs are long).
        elapsed = time.monotonic() - float(getattr(adapter, "_last_historical_fetch_mono", 0.0) or 0.0)
        wait_s = HIST_THROTTLE_SEC - elapsed
        if wait_s > 0:
            time.sleep(wait_s)
        adapter._last_historical_fetch_mono = time.monotonic()
        headers = {
            "X-Kite-Version": "3",
            "Authorization": f"token {adapter.api_key}:{adapter.access_token}",
            "User-Agent": "ChronoSentiment-RP608-HistBackfill",
        }
        req = urllib.request.Request(url, headers=headers)
        with urllib.request.urlopen(req, timeout=adapter.timeout_sec) as resp:
            if resp.status != 200:
                raise urllib.error.HTTPError(
                    url, int(resp.status), f"HTTP_{resp.status}", resp.headers, None
                )
            body = json.loads(resp.read().decode("utf-8"))
            return list((body.get("data") or {}).get("candles") or [])

    try:
        candles = _once()
    except urllib.error.HTTPError as e:
        if e.code in (401, 403):
            adapter.reload_credentials_if_stale(force=True)
            candles = _once()
        elif e.code == 429:
            raise KiteRateLimitError("KITE_HTTP_429") from e
        else:
            raise

    out: List[Dict[str, Any]] = []
    for c in candles:
        if not isinstance(c, list) or len(c) < 6:
            continue
        try:
            c_dt = datetime.fromisoformat(str(c[0]))
            if c_dt.tzinfo is None:
                c_dt = c_dt.replace(tzinfo=IST)
            ts_ms = int(c_dt.astimezone(timezone.utc).timestamp() * 1000)
            out.append(
                {
                    "timestamp_utc_ms": ts_ms,
                    "open": float(c[1]),
                    "high": float(c[2]),
                    "low": float(c[3]),
                    "close": float(c[4]),
                    "volume": float(c[5]),
                    "is_completed": True,
                    "source": "kite_historical_backfill",
                }
            )
        except Exception:
            continue
    return out


def _merge_bars(existing: List[Dict[str, Any]], new: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
    by_ts: Dict[int, Dict[str, Any]] = {}
    for b in existing + new:
        try:
            ts = int(b["timestamp_utc_ms"])
        except Exception:
            continue
        # Prefer completed historical over incomplete snapshots.
        prev = by_ts.get(ts)
        if prev is None:
            by_ts[ts] = b
            continue
        prev_incomplete = prev.get("is_completed") is False or str(prev.get("source", "")).endswith(
            "snapshot"
        )
        new_incomplete = b.get("is_completed") is False or str(b.get("source", "")).endswith("snapshot")
        if prev_incomplete and not new_incomplete:
            by_ts[ts] = b
        elif (not prev_incomplete) and new_incomplete:
            pass
        else:
            by_ts[ts] = b
    return [by_ts[k] for k in sorted(by_ts)]


def _symbols_from_journal(journal: Path) -> Set[str]:
    syms: Set[str] = set()
    if not journal.exists():
        return syms
    with journal.open() as f:
        for line in f:
            if not line.strip():
                continue
            o = json.loads(line)
            s = o.get("symbol")
            if s:
                syms.add(str(s))
    return syms


def main() -> None:
    ap = argparse.ArgumentParser(description="Kite historical candle backfill (not ticks)")
    ap.add_argument("--from-day", type=str, required=True, help="IST YYYY-MM-DD inclusive")
    ap.add_argument("--to-day", type=str, required=True, help="IST YYYY-MM-DD inclusive")
    ap.add_argument(
        "--interval",
        type=str,
        default="15minute",
        choices=sorted(MAX_SPAN_DAYS.keys()),
        help="Kite candle interval (historical ticks unavailable)",
    )
    ap.add_argument(
        "--symbols",
        type=str,
        default="journal",
        help="journal | universe | comma list",
    )
    ap.add_argument(
        "--journal",
        type=Path,
        default=Path("live_capture/shadow_validation/nse/rp608_nse_trade_journal.jsonl"),
    )
    ap.add_argument(
        "--out",
        type=Path,
        default=Path("artifacts/rp608_nse_kite_hist_bars/rp608_nse_bar_history_backfill.json"),
    )
    ap.add_argument(
        "--install-live-bar-history",
        action="store_true",
        help="Also write merged bars into live_capture/.../rp608_nse_bar_history.json",
    )
    ap.add_argument(
        "--live-bar-history",
        type=Path,
        default=Path("live_capture/shadow_validation/nse/rp608_nse_bar_history.json"),
    )
    ap.add_argument("--max-symbols", type=int, default=0, help="0 = all selected")
    args = ap.parse_args()

    d0 = _parse_day(args.from_day)
    d1 = _parse_day(args.to_day)
    if d1 < d0:
        raise SystemExit("to-day < from-day")

    adapter = KiteDataFeedAdapter(NSE_119_SYMBOLS)
    adapter._initialize_instrument_map()
    if not adapter.api_key or not adapter.access_token:
        raise SystemExit(
            f"Missing Kite credentials at {adapter.credentials_file} "
            "(api_key/access_token). Re-auth on Pi dashboard first."
        )
    print(
        f"[+] Kite creds loaded from {adapter.credentials_file} "
        f"(api_key_len={len(adapter.api_key)} token_len={len(adapter.access_token)})",
        flush=True,
    )
    print(
        "[!] Kite historical = OHLCV candles only (no tick-by-tick history).",
        flush=True,
    )

    if args.symbols == "universe":
        symbols = list(NSE_119_SYMBOLS)
    elif args.symbols == "journal":
        symbols = sorted(_symbols_from_journal(args.journal) or set(NSE_119_SYMBOLS))
    else:
        symbols = [s.strip() for s in args.symbols.split(",") if s.strip()]
    if args.max_symbols and args.max_symbols > 0:
        symbols = symbols[: args.max_symbols]

    # Seed from live file if installing / merging.
    hist: Dict[str, List[Dict[str, Any]]] = {s: [] for s in symbols}
    if args.live_bar_history.exists():
        try:
            live = json.loads(args.live_bar_history.read_text())
            if isinstance(live, dict):
                for s, bars in live.items():
                    if s in hist and isinstance(bars, list):
                        hist[s] = [b for b in bars if isinstance(b, dict)]
        except Exception as exc:
            print(f"[!] live bar_history seed skipped: {exc}", flush=True)

    max_span = int(MAX_SPAN_DAYS[args.interval])
    chunks = _iter_chunks(d0, d1, max_span)
    print(
        f"[+] Backfill interval={args.interval} days={d0}..{d1} "
        f"chunks={len(chunks)} symbols={len(symbols)} "
        f"throttle>={HIST_THROTTLE_SEC}s",
        flush=True,
    )

    errors: Dict[str, int] = defaultdict(int)
    fetched_bars = 0
    t0 = time.monotonic()

    for si, sym in enumerate(symbols, start=1):
        mapping = adapter.instrument_map.get(sym)
        if not mapping:
            errors["UNMAPPED"] += 1
            print(f"[{si}/{len(symbols)}] {sym} UNMAPPED", flush=True)
            continue
        token = int(mapping[0])
        sym_new: List[Dict[str, Any]] = []
        for c0, c1 in chunks:
            from_utc, _ = _session_window_utc(c0)
            _, to_utc = _session_window_utc(c1)
            try:
                bars = _fetch_interval(
                    adapter,
                    symbol=sym,
                    token=token,
                    interval=args.interval,
                    from_utc=from_utc,
                    to_utc=to_utc,
                )
                sym_new.extend(bars)
            except KiteAuthError as exc:
                errors["AUTH"] += 1
                print(f"[!] AUTH fail {sym}: {exc}", flush=True)
                raise SystemExit(2) from exc
            except KiteRateLimitError:
                errors["RATE_LIMIT"] += 1
                sleep_s = 5.0
                print(f"[!] 429 on {sym}; sleeping {sleep_s:.0f}s", flush=True)
                time.sleep(sleep_s)
                ok = False
                for attempt in range(3):
                    try:
                        bars = _fetch_interval(
                            adapter,
                            symbol=sym,
                            token=token,
                            interval=args.interval,
                            from_utc=from_utc,
                            to_utc=to_utc,
                        )
                        sym_new.extend(bars)
                        ok = True
                        break
                    except KiteRateLimitError:
                        time.sleep(5.0 * (attempt + 2))
                    except Exception as exc2:
                        errors["FETCH_FAIL"] += 1
                        print(f"[!] retry fail {sym} {c0}..{c1}: {exc2}", flush=True)
                        break
                if not ok:
                    errors["FETCH_FAIL"] += 1
                    print(f"[!] give up chunk {sym} {c0}..{c1}", flush=True)
            except Exception as exc:
                errors["FETCH_FAIL"] += 1
                print(f"[!] fail {sym} {c0}..{c1}: {exc}", flush=True)

        hist[sym] = _merge_bars(hist.get(sym) or [], sym_new)
        fetched_bars += len(sym_new)
        if si % 10 == 0 or si == len(symbols):
            print(
                f"[{si}/{len(symbols)}] {sym} +{len(sym_new)} bars "
                f"(sym_total={len(hist[sym])}) elapsed={time.monotonic()-t0:.0f}s",
                flush=True,
            )

    meta = {
        "protocol_id": "RP-608-NSE-KITE-HIST-BAR-BACKFILL",
        "authority_pct": 0.0,
        "note": "Kite historical candles only; tick-by-tick history is not available from Kite API.",
        "interval": args.interval,
        "from_day_ist": args.from_day,
        "to_day_ist": args.to_day,
        "n_symbols": len(symbols),
        "n_bars_fetched_this_run": fetched_bars,
        "errors": dict(errors),
        "credentials_file": str(adapter.credentials_file),
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
    }

    # Persist as {symbol: [bars]} compatible with daemon bar_history shape,
    # plus sidecar meta.
    args.out.parent.mkdir(parents=True, exist_ok=True)
    payload = {s: hist[s] for s in sorted(hist)}
    args.out.write_text(json.dumps(payload))
    meta_path = args.out.with_suffix(".meta.json")
    meta_path.write_text(json.dumps(meta, indent=2) + "\n")
    print(f"[+] wrote {args.out} ({args.out.stat().st_size} bytes)", flush=True)
    print(f"[+] wrote {meta_path}", flush=True)
    print(json.dumps(meta, indent=2))

    if args.install_live_bar_history:
        # Merge into full universe file.
        live: Dict[str, List[Dict[str, Any]]] = {}
        if args.live_bar_history.exists():
            try:
                live = json.loads(args.live_bar_history.read_text())
            except Exception:
                live = {}
        if not isinstance(live, dict):
            live = {}
        for s, bars in hist.items():
            live[s] = _merge_bars(list(live.get(s) or []), bars)
        tmp = args.live_bar_history.with_suffix(".json.tmp_backfill")
        tmp.write_text(json.dumps(live))
        tmp.replace(args.live_bar_history)
        print(f"[+] installed live bar_history → {args.live_bar_history}", flush=True)


if __name__ == "__main__":
    main()
