#!/usr/bin/env python3
"""
RP-608-WL-IMPROVE — Kite historical 15m fetch + look-ahead-safe decision-time
feature join for NSE Estimand A closed trades.

Uses production `KiteDataFeedAdapter.fetch_historical_15m_bars` (Stage 0B path).
Authority 0%. Baseline / OF ±0.30 frozen. Does not open Gate 2.

Causal rule: only bars with close time < entry_time are used for features.
Crypto plan is out of scope (Kite has no BTCUSDT).
"""

from __future__ import annotations

import argparse
import json
import math
import sys
import time
from collections import defaultdict
from dataclasses import asdict, dataclass
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional, Sequence, Tuple

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT))

from scripts.run_rp608_nse_shadow_daemon import (  # noqa: E402
    KiteAuthError,
    KiteDataFeedAdapter,
    KiteRateLimitError,
)

IST = timezone(timedelta(hours=5, minutes=30))
DAYS_DEFAULT = ["2026-11-03", "2026-11-04", "2026-11-05"]


@dataclass
class Bar:
    close_ms: int
    open: float
    high: float
    low: float
    close: float
    volume: float


def pdt_ms(v: Any) -> Optional[int]:
    if v is None:
        return None
    if isinstance(v, (int, float)):
        ms = int(v)
        return ms if ms > 10_000_000_000 else ms * 1000
    try:
        dt = datetime.fromisoformat(str(v).replace("Z", "+00:00"))
        if dt.tzinfo is None:
            dt = dt.replace(tzinfo=timezone.utc)
        return int(dt.timestamp() * 1000)
    except Exception:
        return None


def load_closed(export_dir: Path, days: Sequence[str]) -> List[Dict[str, Any]]:
    rows: List[Dict[str, Any]] = []
    for day in days:
        path = export_dir / f"session_export_{day}_nse_estimand_a_closed.jsonl"
        if not path.exists():
            continue
        for line in path.read_text(encoding="utf-8", errors="ignore").splitlines():
            if not line.strip():
                continue
            r = json.loads(line)
            r["_session_day"] = day
            rows.append(r)
    return rows


def session_utc_window(day: str, lookback_days: int = 5) -> Tuple[datetime, datetime]:
    # IST cash session ± lookback for momentum/trend warmup
    y, m, d = map(int, day.split("-"))
    start_ist = datetime(y, m, d, 9, 15, tzinfo=IST) - timedelta(days=lookback_days)
    end_ist = datetime(y, m, d, 15, 30, tzinfo=IST)
    return start_ist.astimezone(timezone.utc), end_ist.astimezone(timezone.utc)


def bars_before(bars: Sequence[Bar], entry_ms: int) -> List[Bar]:
    return [b for b in bars if b.close_ms < entry_ms]


def feat_from_prior(prior: Sequence[Bar], entry_price: float) -> Dict[str, Any]:
    if len(prior) < 2:
        return {
            "trend": "TREND_UNAVAILABLE",
            "momentum": "MOMENTUM_UNAVAILABLE",
            "price_location": "PRICE_LOCATION_UNAVAILABLE",
            "n_prior_bars": len(prior),
        }
    closes = [b.close for b in prior]
    # Momentum: last completed bar return vs prior close
    mom_1 = (closes[-1] / closes[-2] - 1.0) * 1e4 if closes[-2] else None
    mom_4 = None
    if len(closes) >= 5 and closes[-5]:
        mom_4 = (closes[-1] / closes[-5] - 1.0) * 1e4
    # Trend: sign of SMA(4) slope over last 8 closes if available
    trend = "TREND_UNAVAILABLE"
    if len(closes) >= 8:
        sma_recent = sum(closes[-4:]) / 4.0
        sma_older = sum(closes[-8:-4]) / 4.0
        if sma_older:
            slope_bps = (sma_recent / sma_older - 1.0) * 1e4
            if slope_bps > 5:
                trend = "TREND_UP"
            elif slope_bps < -5:
                trend = "TREND_DOWN"
            else:
                trend = "TREND_FLAT"
            trend_slope_bps = round(slope_bps, 3)
        else:
            trend_slope_bps = None
    else:
        trend_slope_bps = None

    # Price location in prior session range (prior bars only)
    hi = max(b.high for b in prior[-16:]) if prior else None
    lo = min(b.low for b in prior[-16:]) if prior else None
    loc = "PRICE_LOCATION_UNAVAILABLE"
    loc_pct = None
    if hi is not None and lo is not None and hi > lo and entry_price > 0:
        loc_pct = (entry_price - lo) / (hi - lo)
        if loc_pct < 0.33:
            loc = "LOC_LOWER_THIRD"
        elif loc_pct < 0.66:
            loc = "LOC_MID_THIRD"
        else:
            loc = "LOC_UPPER_THIRD"

    def mom_bucket(x: Optional[float]) -> str:
        if x is None:
            return "MOMENTUM_UNAVAILABLE"
        if x > 10:
            return "MOM_UP_>10bps"
        if x < -10:
            return "MOM_DOWN_<-10bps"
        return "MOM_FLAT"

    return {
        "trend": trend,
        "trend_slope_bps": trend_slope_bps,
        "momentum": mom_bucket(mom_4 if mom_4 is not None else mom_1),
        "momentum_1bar_bps": None if mom_1 is None else round(mom_1, 3),
        "momentum_4bar_bps": None if mom_4 is None else round(mom_4, 3),
        "price_location": loc,
        "price_location_pct": None if loc_pct is None else round(loc_pct, 4),
        "n_prior_bars": len(prior),
        "prior_last_close": closes[-1],
    }


def fetch_symbol_days(
    adapter: KiteDataFeedAdapter,
    symbol: str,
    days: Sequence[str],
    cache: Dict[str, List[Bar]],
) -> Tuple[str, Optional[str]]:
    key = symbol
    if key in cache:
        return "CACHE_HIT", None
    # One window covering all days + lookback from earliest
    earliest = min(days)
    latest = max(days)
    from_utc, _ = session_utc_window(earliest, lookback_days=7)
    _, to_utc = session_utc_window(latest, lookback_days=0)
    try:
        raw = adapter.fetch_historical_15m_bars(symbol, from_utc, to_utc)
    except KiteAuthError as e:
        return "AUTH_FAIL", str(e)
    except KiteRateLimitError as e:
        return "RATE_LIMIT", str(e)
    except Exception as e:
        return "FETCH_ERROR", f"{type(e).__name__}:{e}"
    bars: List[Bar] = []
    for b in raw:
        bars.append(
            Bar(
                close_ms=int(b.timestamp_utc_ms),
                open=float(b.open),
                high=float(b.high),
                low=float(b.low),
                close=float(b.close),
                volume=float(b.volume),
            )
        )
    bars.sort(key=lambda x: x.close_ms)
    cache[key] = bars
    return "OK", None


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--export-dir", default="artifacts/rp608_econ_edge_diag")
    ap.add_argument("--out-dir", default="artifacts/rp608_econ_edge_diag")
    ap.add_argument("--days", nargs="+", default=DAYS_DEFAULT)
    ap.add_argument("--symbol-limit", type=int, default=0, help="0 = all symbols in closed set")
    ap.add_argument("--sleep-sec", type=float, default=0.35)
    args = ap.parse_args()

    export_dir = Path(args.export_dir)
    out_dir = Path(args.out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)

    closed = load_closed(export_dir, args.days)
    symbols = sorted({str(r.get("symbol")) for r in closed if r.get("symbol")})
    if args.symbol_limit and args.symbol_limit > 0:
        symbols = symbols[: args.symbol_limit]

    report: Dict[str, Any] = {
        "protocol": "RP-608-WL-IMPROVE-001",
        "phase": "kite_nse_decision_time_bars",
        "authority": "0.0%",
        "baseline_frozen": True,
        "gate2_criteria_changed": False,
        "book": "nse_estimand_a",
        "crypto_note": "Kite historical does not cover crypto plan (BTCUSDT); crypto feature gap unchanged.",
        "days": list(args.days),
        "n_closed_trades": len(closed),
        "n_symbols": len(symbols),
        "fetch_status": {},
        "join_summary": {},
    }

    adapter = KiteDataFeedAdapter(tuple(symbols) if symbols else ("RELIANCE.NS",))
    if not adapter.api_key or not adapter.access_token:
        report["fetch_status"] = {"overall": "MISSING_CREDENTIALS"}
        report["research_conclusion"] = (
            "Kite credentials missing/empty. Re-auth via dashboard, then re-run this script."
        )
        (out_dir / "wl_kite_nse_decision_time_bars.md").write_text(
            "# Kite NSE decision-time bars\n\n**MISSING_CREDENTIALS**\n", encoding="utf-8"
        )
        (out_dir / "wl_kite_nse_decision_time_bars.json").write_text(
            json.dumps(report, indent=2), encoding="utf-8"
        )
        print(json.dumps({"status": "MISSING_CREDENTIALS"}, indent=2))
        return 2

    cache: Dict[str, List[Bar]] = {}
    auth_fail = False
    ok_n = 0
    for i, sym in enumerate(symbols):
        status, err = fetch_symbol_days(adapter, sym, args.days, cache)
        report["fetch_status"][sym] = {"status": status, "error": err, "n_bars": len(cache.get(sym, []))}
        if status == "AUTH_FAIL":
            auth_fail = True
            report["fetch_status"]["overall"] = "AUTH_FAIL"
            report["fetch_status"]["auth_error"] = err
            break
        if status == "OK":
            ok_n += 1
        if args.sleep_sec > 0 and i + 1 < len(symbols):
            time.sleep(args.sleep_sec)

    if auth_fail:
        report["research_conclusion"] = (
            "Kite historical returned auth failure (likely expired access_token). "
            "Re-authenticate Kite on the dashboard / Pi credential hot-reload path "
            "(same as Stage 0B), then re-run. No features imputed."
        )
        md = [
            "# RP-608 — Kite NSE Decision-Time Bars",
            "",
            "**Status:** AUTH_FAIL",
            "",
            report["research_conclusion"],
            "",
            "Crypto plan: still non-Kite. Gate 2 criteria unchanged.",
            "",
        ]
        (out_dir / "wl_kite_nse_decision_time_bars.md").write_text("\n".join(md), encoding="utf-8")
        (out_dir / "wl_kite_nse_decision_time_bars.json").write_text(
            json.dumps(report, indent=2), encoding="utf-8"
        )
        print(json.dumps({"status": "AUTH_FAIL", "error": report["fetch_status"].get("auth_error")}, indent=2))
        return 3

    # Persist bar cache (research artifact)
    bars_path = out_dir / "wl_kite_nse_15m_bars_cache.jsonl"
    with bars_path.open("w", encoding="utf-8") as fh:
        for sym, bars in cache.items():
            for b in bars:
                fh.write(json.dumps({"symbol": sym, **asdict(b)}) + "\n")

    # Join features
    join_path = out_dir / "wl_nse_a_decision_time_features_kite_join.jsonl"
    n_ok = n_unavail = 0
    with join_path.open("w", encoding="utf-8") as fh:
        for r in closed:
            sym = str(r.get("symbol"))
            entry_ms = pdt_ms(r.get("entry_utc") or r.get("entry_time_ms"))
            entry_px = r.get("entry_price")
            net = r.get("net_pnl_inr")
            if entry_ms is None or entry_px is None or net is None:
                n_unavail += 1
                continue
            prior = bars_before(cache.get(sym, []), int(entry_ms))
            feats = feat_from_prior(prior, float(entry_px))
            if feats["trend"] == "TREND_UNAVAILABLE":
                n_unavail += 1
            else:
                n_ok += 1
            out = {
                "session_day": r.get("_session_day"),
                "symbol": sym,
                "trade_id": r.get("trade_id"),
                "entry_utc_ms": entry_ms,
                "entry_price": entry_px,
                "net_pnl_inr": net,
                "win": float(net) > 0,
                "side": r.get("side"),
                "entry_bar_idx": r.get("entry_bar_idx"),
                "source": "kite_historical_15m",
                "look_ahead_safe": True,
                **feats,
            }
            fh.write(json.dumps(out) + "\n")

    report["join_summary"] = {
        "joined_rows": n_ok + n_unavail,
        "feature_ready_rows": n_ok,
        "feature_unavailable_rows": n_unavail,
        "symbols_fetched_ok": ok_n,
        "bars_cache": str(bars_path),
        "join_path": str(join_path),
    }
    report["fetch_status"]["overall"] = "OK" if ok_n else "NO_BARS"
    report["research_conclusion"] = (
        f"Fetched Kite 15m for {ok_n}/{len(symbols)} symbols; "
        f"look-ahead-safe features ready on {n_ok} NSE A rows, "
        f"unavailable on {n_unavail}. This improves NSE A feature observability only; "
        "does not reopen Gate 2; crypto plan still needs a non-Kite bar source."
    )

    # W/L by new features (descriptive only)
    joined = []
    for line in join_path.read_text().splitlines():
        if line.strip():
            joined.append(json.loads(line))

    def wl_by(field: str) -> List[Dict[str, Any]]:
        buckets: Dict[str, List[float]] = defaultdict(list)
        for row in joined:
            if row.get(field) and "UNAVAILABLE" not in str(row.get(field)):
                buckets[str(row[field])].append(float(row["net_pnl_inr"]))
        out = []
        for k, nets in buckets.items():
            w = sum(1 for x in nets if x > 0)
            n = len(nets)
            out.append({"key": k, "n": n, "wins": w, "wr_pct": round(100 * w / n, 2) if n else None})
        out.sort(key=lambda r: (r["wr_pct"] is None, r["wr_pct"] or 999, -r["n"]))
        return out

    report["descriptive_wl"] = {
        "by_trend": wl_by("trend"),
        "by_momentum": wl_by("momentum"),
        "by_price_location": wl_by("price_location"),
    }

    md = [
        "# RP-608 — Kite NSE Decision-Time Feature Join",
        "",
        f"**Overall fetch:** {report['fetch_status'].get('overall')}",
        f"**Symbols OK:** {ok_n}/{len(symbols)} · **Feature-ready rows:** {n_ok} · **Unavailable:** {n_unavail}",
        "",
        report["research_conclusion"],
        "",
        "## Descriptive W/L (not Gate 2)",
        "",
        f"- Trend: `{report['descriptive_wl']['by_trend']}`",
        f"- Momentum: `{report['descriptive_wl']['by_momentum']}`",
        f"- Price location: `{report['descriptive_wl']['by_price_location']}`",
        "",
        "## Governance",
        "",
        "Baseline frozen · OF ±0.30 frozen · Gate 2 criteria unchanged · Gate 3 unauthorized.",
        "Crypto plan: Kite cannot supply BTC bars.",
        "",
    ]
    (out_dir / "wl_kite_nse_decision_time_bars.md").write_text("\n".join(md), encoding="utf-8")
    (out_dir / "wl_kite_nse_decision_time_bars.json").write_text(
        json.dumps(report, indent=2, default=str), encoding="utf-8"
    )
    print(json.dumps({"status": report["fetch_status"]["overall"], "join": report["join_summary"]}, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
