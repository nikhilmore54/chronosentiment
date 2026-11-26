#!/usr/bin/env python3
"""
Probe Gate2E-suppressed (or flat PRIMARY) prospective B/C signals → counterfactual
overlay geometry on Binance 1m path, with Gate 2E candle volume stamps.

Each CF row reports price path (TP/SL/net/MFE) **and** entry/exit
`vol_burst_20` + `vol_surge` (15m volume / SMA20 of prior 20 completed bars).
That is the plan-pattern candle axis — **not** OFX 3× tick aggression.

Example UI row: bar #2019 · LONG/LONG · ref · ret · atr — maps to decision card
via bar_idx + reference_price + utc time, not decision_event_idx alone.

Books never pooled. Authority 0%. Does not open trades.
Does not change allow_only / OF ±0.30 / Gate 5.
Do not loosen allow_only on price path alone.

Usage:
  python3 scripts/research/rp608_probe_suppressed_prospective.py --evt 259091
  python3 scripts/research/rp608_probe_suppressed_prospective.py --bar 2019 --ref 59471.42
  python3 scripts/research/rp608_probe_suppressed_prospective.py --since-hours 6 --long-long-only
"""

from __future__ import annotations

import argparse
import json
import sys
import urllib.request
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

WORKSPACE = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(WORKSPACE))
from scripts.rp608_crypto_trade_pattern_library import vol_surge_bucket  # noqa: E402

CRYPTO = WORKSPACE / "live_capture" / "shadow_validation" / "crypto"
CARDS = CRYPTO / "rp608_decision_cards.jsonl"
PRIMARY_EVENTS = CRYPTO / "trade_plan_overlay_v1" / "trade_plan_events.jsonl"
BASELINE_OUTCOMES = (
    CRYPTO / "trade_plan_overlay_baseline_parallel_v1" / "trade_plan_outcomes.jsonl"
)

SL_ATR = 1.0
TP_ATR = 1.5
NOTIONAL = 1000.0
FRICTION_BPS_SIDE = 6.0
HORIZON_MIN = 60
INTERVAL_15M_MS = 15 * 60 * 1000


@dataclass
class CFResult:
    result: str
    hit_bar_min: int
    hit_px: float
    gross: float
    friction: float
    net: float
    mfe: float
    mae: float
    path_high: float
    path_low: float


def _fetch_klines(
    symbol: str,
    start_ms: int,
    end_ms: Optional[int] = None,
    *,
    interval: str = "1m",
    limit: int = 1000,
) -> List[list]:
    end_ms = end_ms or int(datetime.now(timezone.utc).timestamp() * 1000)
    url = (
        f"https://api.binance.com/api/v3/klines?symbol={symbol}"
        f"&interval={interval}&startTime={int(start_ms)}&endTime={int(end_ms)}"
        f"&limit={int(limit)}"
    )
    req = urllib.request.Request(url, headers={"User-Agent": "rp608-probe/1.0"})
    with urllib.request.urlopen(req, timeout=12) as resp:
        return json.loads(resp.read().decode())


def _vol_burst_from_15m(
    klines_15m: List[list], *, bar_open_ms: int
) -> Optional[float]:
    """
    Gate 2E candle vol_burst_20 at the 15m bar whose open == bar_open_ms.
    = bar.volume / mean(volume of prior 20 completed bars). Not OFX.
    """
    if not klines_15m:
        return None
    idx = None
    for i, row in enumerate(klines_15m):
        if int(row[0]) == int(bar_open_ms):
            idx = i
            break
    if idx is None:
        # nearest completed bar at or before target open
        for i in range(len(klines_15m) - 1, -1, -1):
            if int(klines_15m[i][0]) <= int(bar_open_ms):
                idx = i
                break
    if idx is None or idx < 20:
        return None
    prior = [float(klines_15m[i][5]) for i in range(idx - 20, idx)]
    sma = sum(prior) / 20.0
    if sma <= 0:
        return None
    return float(klines_15m[idx][5]) / sma


def resolve_entry_volume(
    card: Dict[str, Any], symbol: str, signal_ms: int
) -> Dict[str, Any]:
    """Prefer card.vol_burst_20; else recompute from Binance 15m OHLCV."""
    burst: Optional[float] = None
    source = "missing"
    raw = card.get("vol_burst_20")
    if raw is not None:
        try:
            burst = float(raw)
            source = "decision_card"
        except (TypeError, ValueError):
            burst = None
    surge_raw = card.get("vol_surge")
    if burst is None:
        # Need prior 20 + signal bar: fetch ~6h before close
        start = int(signal_ms) - (25 * INTERVAL_15M_MS)
        end = int(signal_ms) + INTERVAL_15M_MS
        try:
            kl15 = _fetch_klines(symbol, start, end, interval="15m", limit=50)
        except Exception:
            kl15 = []
        bar_open = (int(signal_ms) // INTERVAL_15M_MS) * INTERVAL_15M_MS
        # close_time_ms is usually bar close; open is close - 15m + 1ms-ish
        # Binance openTime aligns to 15m grid; prefer open of containing bar.
        burst = _vol_burst_from_15m(kl15, bar_open_ms=bar_open)
        if burst is None and kl15:
            # try open = close_time floored minus one interval
            alt = bar_open - INTERVAL_15M_MS
            burst = _vol_burst_from_15m(kl15, bar_open_ms=alt)
        source = "binance_15m_recompute" if burst is not None else "unavailable"
    if isinstance(surge_raw, str) and surge_raw in {"LOW", "MED", "HIGH", "UNK"}:
        surge = surge_raw
    else:
        surge = vol_surge_bucket(burst)
    return {
        "vol_burst_20": round(burst, 6) if burst is not None else None,
        "vol_surge": surge,
        "vol_source": source,
        "vol_axis": f"V{surge}",
    }


def resolve_exit_volume(
    symbol: str, signal_ms: int, hit_bar_min: int
) -> Dict[str, Any]:
    """Candle vol at the 15m bar containing the CF exit (Gate 2X observational)."""
    exit_ms = int(signal_ms) + int(hit_bar_min) * 60_000
    bar_open = (exit_ms // INTERVAL_15M_MS) * INTERVAL_15M_MS
    start = bar_open - (25 * INTERVAL_15M_MS)
    end = bar_open + INTERVAL_15M_MS
    try:
        kl15 = _fetch_klines(symbol, start, end, interval="15m", limit=50)
    except Exception:
        kl15 = []
    burst = _vol_burst_from_15m(kl15, bar_open_ms=bar_open)
    surge = vol_surge_bucket(burst)
    return {
        "exit_vol_burst_20": round(burst, 6) if burst is not None else None,
        "exit_vol_surge": surge,
        "exit_vol_axis": f"V{surge}",
        "exit_bar_open_ms": bar_open,
    }


def simulate_long(entry: float, atr: float, klines: List[list]) -> CFResult:
    stop = round(entry - SL_ATR * atr, 2)
    target = round(entry + TP_ATR * atr, 2)
    qty = NOTIONAL / entry
    mfe = 0.0
    mae = 0.0
    hit = None
    hit_bar = 0
    hit_px = entry
    for i, row in enumerate(klines[: HORIZON_MIN + 1]):
        _o, h, l, c = map(float, row[1:5])
        mfe = max(mfe, h - entry)
        mae = min(mae, l - entry)
        hit_tp = h >= target
        hit_sl = l <= stop
        if hit_tp and hit_sl:
            hit, hit_px, hit_bar = "AMBIGUOUS_SL", stop, i
            break
        if hit_sl:
            hit, hit_px, hit_bar = "STOP_LOSS_HIT", stop, i
            break
        if hit_tp:
            hit, hit_px, hit_bar = "PROFIT_TARGET_HIT", target, i
            break
    if hit is None:
        if len(klines) >= HORIZON_MIN:
            hit_px = float(klines[HORIZON_MIN - 1][4])
            hit, hit_bar = "MAX_HORIZON_60M", HORIZON_MIN - 1
        else:
            hit_px = float(klines[-1][4]) if klines else entry
            hit, hit_bar = "OPEN_MTM", max(0, len(klines) - 1)
    gross = (hit_px - entry) * qty
    fric = NOTIONAL * (FRICTION_BPS_SIDE / 10000.0) + (hit_px * qty) * (
        FRICTION_BPS_SIDE / 10000.0
    )
    hi = max(float(r[2]) for r in klines) if klines else entry
    lo = min(float(r[3]) for r in klines) if klines else entry
    return CFResult(
        result=hit,
        hit_bar_min=hit_bar,
        hit_px=hit_px,
        gross=round(gross, 4),
        friction=round(fric, 4),
        net=round(gross - fric, 4),
        mfe=round(mfe, 2),
        mae=round(mae, 2),
        path_high=hi,
        path_low=lo,
    )


def find_card(
    *,
    evt: Optional[int] = None,
    bar: Optional[int] = None,
    ref: Optional[float] = None,
) -> Optional[Dict[str, Any]]:
    if not CARDS.exists():
        return None
    best = None
    best_d = None
    with open(CARDS, "rb") as f:
        f.seek(0, 2)
        f.seek(max(0, f.tell() - 20_000_000))
        for line in f:
            if not line.strip():
                continue
            try:
                r = json.loads(line)
            except json.JSONDecodeError:
                continue
            if evt is not None and int(r.get("decision_event_idx") or -1) != evt:
                continue
            if bar is not None and int(r.get("bar_idx") or -1) != bar:
                if evt is None:
                    # allow bar-only with ref filter
                    if ref is None:
                        continue
                    if int(r.get("bar_idx") or -1) != bar:
                        continue
            if ref is not None:
                try:
                    d = abs(float(r.get("reference_price") or 0) - ref)
                except (TypeError, ValueError):
                    continue
                if d > 5.0:
                    continue
                if best is None or d < best_d:
                    best, best_d = r, d
                continue
            return r
    return best


def primary_suppressed(evt: int, signal_ms: int) -> bool:
    if not PRIMARY_EVENTS.exists():
        return False
    with open(PRIMARY_EVENTS, "rb") as f:
        f.seek(0, 2)
        f.seek(max(0, f.tell() - 4_000_000))
        raw = f.read().decode("utf-8", "ignore")
    for line in raw.splitlines():
        if "SUPPRESSED_TRADE_PATTERN" not in line:
            continue
        if str(signal_ms) in line or f"PLAN-" in line:
            try:
                r = json.loads(line)
            except json.JSONDecodeError:
                continue
            plan = r.get("plan") or {}
            if plan.get("source_decision_event_idx") == evt:
                return True
            if abs(int(plan.get("entry_time_ms") or 0) - signal_ms) < 2000:
                return True
            # event may lack plan body — timestamp match
            if str(signal_ms) in str(r.get("plan_id") or "") or str(signal_ms) in line:
                return True
    return True  # conservative: UI prospective often means no PRIMARY open


def probe_card(card: Dict[str, Any], symbol: str = "BTCUSDT") -> Dict[str, Any]:
    entry = float(card.get("reference_price") or 0)
    atr = float(card.get("atr_14") or 0)
    ms = int(card.get("close_time_ms") or 0)
    evt = int(card.get("decision_event_idx") or 0)
    if entry <= 0 or atr <= 0 or ms <= 0:
        return {"error": "bad_card"}
    entry_vol = resolve_entry_volume(card, symbol, ms)
    kl = _fetch_klines(symbol, ms)
    cf = simulate_long(entry, atr, kl)
    exit_vol = resolve_exit_volume(symbol, ms, cf.hit_bar_min)
    return {
        "decision_event_idx": evt,
        "bar_idx": card.get("bar_idx"),
        "signal_utc": datetime.fromtimestamp(ms / 1000, tz=timezone.utc).isoformat(),
        "policy_b": card.get("policy_b_action"),
        "policy_c": card.get("policy_c_action"),
        "entry": entry,
        "atr": atr,
        "stop": round(entry - SL_ATR * atr, 2),
        "target": round(entry + TP_ATR * atr, 2),
        "order_flow_supported": card.get("order_flow_supported"),
        "order_flow_imbalance": card.get("order_flow_imbalance"),
        # Gate 2E candle volume (≠ OFX tick aggression)
        "vol_burst_20": entry_vol["vol_burst_20"],
        "vol_surge": entry_vol["vol_surge"],
        "vol_axis": entry_vol["vol_axis"],
        "vol_source": entry_vol["vol_source"],
        "exit_vol_burst_20": exit_vol["exit_vol_burst_20"],
        "exit_vol_surge": exit_vol["exit_vol_surge"],
        "exit_vol_axis": exit_vol["exit_vol_axis"],
        "counterfactual": {
            "result": cf.result,
            "hit_bar_min": cf.hit_bar_min,
            "hit_px": cf.hit_px,
            "gross": cf.gross,
            "friction": cf.friction,
            "net": cf.net,
            "mfe": cf.mfe,
            "mae": cf.mae,
            "path_high": cf.path_high,
            "path_low": cf.path_low,
            "vol_burst_20": entry_vol["vol_burst_20"],
            "vol_surge": entry_vol["vol_surge"],
            "vol_axis": entry_vol["vol_axis"],
            "exit_vol_burst_20": exit_vol["exit_vol_burst_20"],
            "exit_vol_surge": exit_vol["exit_vol_surge"],
            "exit_vol_axis": exit_vol["exit_vol_axis"],
        },
        "note": (
            "Overlay 1.0×ATR SL / 1.5×ATR TP / 60m; $1000; 6bps/side; "
            "vol_surge=15m OHLCV/SMA20 (Gate2E) — not OFX; "
            "do not loosen allow_only on price alone"
        ),
    }


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--evt", type=int, default=None)
    ap.add_argument("--bar", type=int, default=None)
    ap.add_argument("--ref", type=float, default=None)
    ap.add_argument("--symbol", default="BTCUSDT")
    ap.add_argument("--since-hours", type=float, default=None)
    ap.add_argument("--long-long-only", action="store_true")
    ap.add_argument("--json", action="store_true")
    ap.add_argument("--limit", type=int, default=15)
    args = ap.parse_args()

    if args.evt or args.bar or args.ref:
        card = find_card(evt=args.evt, bar=args.bar, ref=args.ref)
        if not card:
            print("card not found", file=sys.stderr)
            return 1
        # disambiguate bar+ref: prefer LONG/LONG matching ref
        if args.bar and args.ref and card.get("policy_b_action") != "LONG":
            # rescan for LONG near ref on that bar
            card = find_card(bar=args.bar, ref=args.ref) or card
        report = probe_card(card, symbol=args.symbol)
        if args.json:
            json.dump(report, sys.stdout, indent=2)
            print()
        else:
            cf = report["counterfactual"]
            print(
                f"evt={report['decision_event_idx']} bar={report['bar_idx']} "
                f"{report['policy_b']}/{report['policy_c']} entry={report['entry']:.2f}"
            )
            print(
                f"  signal={report['signal_utc']} OF={report['order_flow_supported']} "
                f"vol={report['vol_axis']} burst={report['vol_burst_20']} "
                f"({report['vol_source']}) exit_vol={report['exit_vol_axis']} "
                f"exit_burst={report['exit_vol_burst_20']}"
            )
            print(
                f"  stop={report['stop']} target={report['target']} "
                f"→ {cf['result']} @{cf['hit_bar_min']}m net={cf['net']} "
                f"(gross={cf['gross']} fric={cf['friction']}) "
                f"MFE={cf['mfe']} MAE={cf['mae']}"
            )
            print(
                "  (Gate2E candle vol ≠ OFX; W/L+payoff+net required; "
                "allow_only unchanged)"
            )
        return 0

    # Batch: recent LONG/LONG cards
    if not CARDS.exists():
        print("no cards", file=sys.stderr)
        return 1
    since = None
    if args.since_hours is not None:
        since = int(
            (datetime.now(timezone.utc).timestamp() - float(args.since_hours) * 3600)
            * 1000
        )
    rows: List[Dict[str, Any]] = []
    with open(CARDS, "rb") as f:
        f.seek(0, 2)
        f.seek(max(0, f.tell() - 12_000_000))
        for line in f:
            if not line.strip():
                continue
            try:
                r = json.loads(line)
            except json.JSONDecodeError:
                continue
            if args.long_long_only:
                if r.get("policy_b_action") != "LONG" or r.get("policy_c_action") != "LONG":
                    continue
            elif r.get("policy_b_action") not in ("LONG", "SHORT"):
                continue
            ms = int(r.get("close_time_ms") or 0)
            if since and ms < since:
                continue
            rows.append(r)
    # One CF per 15m bar_idx (prefer bar-close / last card on that bar).
    # Midbar spam shares the same candle vol_burst — not useful for Gate2E reads.
    by_bar: Dict[Tuple[Any, Any], Dict[str, Any]] = {}
    for r in rows:
        key = (r.get("bar_idx"), r.get("policy_b_action"))
        prev = by_bar.get(key)
        if prev is None:
            by_bar[key] = r
            continue
        # Prefer completed-bar emit; else latest close_time_ms
        def _rank(card: Dict[str, Any]) -> Tuple[int, int]:
            reason = str(card.get("emit_reason") or "")
            completed = 1 if ("close" in reason.lower() or reason in {"BAR_CLOSE", "bar_close"}) else 0
            return (completed, int(card.get("close_time_ms") or 0))

        if _rank(r) >= _rank(prev):
            by_bar[key] = r
    uniq = sorted(
        by_bar.values(),
        key=lambda c: int(c.get("close_time_ms") or 0),
    )[-args.limit :]
    reports = []
    for r in uniq:
        try:
            reports.append(probe_card(r, symbol=args.symbol))
        except Exception as exc:
            reports.append({"error": str(exc), "evt": r.get("decision_event_idx")})
    # Closed-path only for W/L (OPEN_MTM is not a closed trade)
    closed = [
        p
        for p in reports
        if isinstance(p.get("counterfactual"), dict)
        and str(p["counterfactual"].get("result") or "") != "OPEN_MTM"
    ]
    if args.json:
        json.dump(
            {
                "n": len(reports),
                "n_closed_path": len(closed),
                "probes": reports,
                "note": "W/L uses closed-path only; OPEN_MTM excluded",
            },
            sys.stdout,
            indent=2,
        )
        print()
        return 0
    wins = sum(1 for p in closed if p["counterfactual"].get("net", 0) > 0)
    losses = sum(1 for p in closed if p["counterfactual"].get("net", 0) <= 0)
    net_sum = sum(float(p["counterfactual"]["net"]) for p in closed)
    win_abs = [
        abs(float(p["counterfactual"]["net"]))
        for p in closed
        if p["counterfactual"].get("net", 0) > 0
    ]
    loss_abs = [
        abs(float(p["counterfactual"]["net"]))
        for p in closed
        if p["counterfactual"].get("net", 0) <= 0
    ]
    avg_w = (sum(win_abs) / len(win_abs)) if win_abs else None
    avg_l = (sum(loss_abs) / len(loss_abs)) if loss_abs else None
    payoff = (avg_w / avg_l) if avg_w is not None and avg_l and avg_l > 0 else None
    by_vol: Dict[str, Dict[str, Any]] = {}
    for p in closed:
        v = str(p.get("vol_surge") or "UNK")
        b = by_vol.setdefault(v, {"wins": 0, "losses": 0, "n": 0, "net": 0.0})
        b["n"] += 1
        net = float(p["counterfactual"]["net"])
        b["net"] += net
        if net > 0:
            b["wins"] += 1
        else:
            b["losses"] += 1
    print(
        f"Probed {len(reports)} bars ({len(closed)} closed-path CF + Gate2E vol; never pooled) "
        f"W/L={wins}/{wins+losses} payoff={payoff} net_sum={round(net_sum, 2)}"
    )
    if by_vol:
        print("  by entry vol_surge (candle ≠ OFX; closed-path only):")
        for k, v in sorted(by_vol.items()):
            wr = (v["wins"] / v["n"]) if v["n"] else None
            print(
                f"    V{k}: W/L={v['wins']}/{v['n']} wr={wr} net={round(v['net'], 2)}"
            )
    for p in reports:
        if "error" in p:
            print(" ", p)
            continue
        cf = p["counterfactual"]
        print(
            f"  evt={p['decision_event_idx']} bar={p['bar_idx']} "
            f"{p['policy_b']}/{p['policy_c']} @{p['entry']:.2f} "
            f"vol={p.get('vol_axis')} burst={p.get('vol_burst_20')} "
            f"exit={p.get('exit_vol_axis')} → "
            f"{cf['result']} net={cf['net']}"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
