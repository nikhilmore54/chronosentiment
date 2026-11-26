#!/usr/bin/env python3
"""
RP-608-NSE-OF-BOOK-INTELLIGENCE-001 — Phase 2 observational features.

From durable L1–L5 snapshots, compute:
  I5, bid persistence during price decline, replenishment / apparent withdrawal,
  and forward 1/2/4 completed-bar returns net of a half-spread + statutory friction proxy.

OF ±0.30 / reverse-OF paper / Estimand A are untouched.
Sample-size caveats are mandatory in the report — short tapes are telemetry only.

Usage:
  python3 scripts/research/rp608_nse_of_book_intel_phase2_features.py
"""

from __future__ import annotations

import argparse
import json
import sys
from collections import defaultdict
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT))

from scripts.run_rp608_nse_shadow_daemon import calc_indian_equity_friction  # noqa: E402

PROTOCOL = "RP-608-NSE-OF-BOOK-INTELLIGENCE-001"
PHASE = "PHASE_2_FEATURES_AND_PRELIM_EVENTS"
NSE_DIR = ROOT / "live_capture" / "shadow_validation" / "nse"
BOOK_DIR = NSE_DIR / "of_book_intelligence_v1"
BAR_MS = 15 * 60 * 1000

# Observational feature params — NOT OF entry thresholds.
I5_SUPPORT = 0.20
PERSIST_SNAPS = 3
BID_SUM_TOL = 0.02  # allow 2% bid-sum erosion and still call "persistent"
REPLENISH_BOUNCE = 0.10  # 10% recovery after a prior drop counts as replenishment
WITHDRAW_DROP = 0.15  # 15% bid-sum drop without price improvement = apparent withdrawal
MIN_SNAPS_FOR_FEATURES = 8
MIN_EVENTS_FOR_CLAIM = 30  # below this: telemetry only


def _qty_sum(levels: Any) -> float:
    if not isinstance(levels, list):
        return 0.0
    s = 0.0
    for lv in levels[:5]:
        if isinstance(lv, dict):
            try:
                s += float(lv.get("quantity") or 0.0)
            except (TypeError, ValueError):
                pass
    return s


def i5_imbalance(depth_buy: Any, depth_sell: Any) -> Optional[float]:
    b = _qty_sum(depth_buy)
    a = _qty_sum(depth_sell)
    d = b + a
    if d <= 0:
        return None
    return (b - a) / d


def mid_price(row: Dict[str, Any]) -> Optional[float]:
    try:
        bb = float(row.get("best_bid") or 0.0)
        ba = float(row.get("best_ask") or 0.0)
    except (TypeError, ValueError):
        return None
    if bb > 0 and ba > 0 and ba >= bb:
        return 0.5 * (bb + ba)
    try:
        lp = float(row.get("last_price") or 0.0)
    except (TypeError, ValueError):
        return None
    return lp if lp > 0 else None


def half_spread_bps(row: Dict[str, Any]) -> float:
    try:
        bb = float(row.get("best_bid") or 0.0)
        ba = float(row.get("best_ask") or 0.0)
    except (TypeError, ValueError):
        return 5.0
    m = mid_price(row)
    if not m or m <= 0 or ba <= bb:
        return 5.0
    return max(0.0, ((ba - bb) / m) * 10000.0 / 2.0)


def load_snapshots(path: Path) -> Dict[str, List[Dict[str, Any]]]:
    by: Dict[str, List[Dict[str, Any]]] = defaultdict(list)
    if not path.exists():
        return {}
    with path.open(encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                o = json.loads(line)
            except json.JSONDecodeError:
                continue
            sym = o.get("symbol")
            if not sym:
                continue
            o["_i5"] = i5_imbalance(o.get("depth_buy"), o.get("depth_sell"))
            o["_bid_sum"] = _qty_sum(o.get("depth_buy"))
            o["_ask_sum"] = _qty_sum(o.get("depth_sell"))
            o["_mid"] = mid_price(o)
            by[str(sym)].append(o)
    for sym in by:
        by[sym].sort(key=lambda r: int(r.get("sample_utc_ms") or 0))
    return dict(by)


def load_bars(path: Path) -> Dict[str, List[Dict[str, Any]]]:
    if not path.exists():
        return {}
    doc = json.loads(path.read_text(encoding="utf-8"))
    out: Dict[str, List[Dict[str, Any]]] = {}
    if not isinstance(doc, dict):
        return out
    for sym, bars in doc.items():
        if not isinstance(bars, list):
            continue
        cleaned = []
        for b in bars:
            if not isinstance(b, dict):
                continue
            try:
                ts = int(b.get("timestamp_utc_ms") or 0)
                o = float(b["open"])
                c = float(b["close"])
            except (KeyError, TypeError, ValueError):
                continue
            cleaned.append(
                {
                    "timestamp_utc_ms": ts,
                    "open": o,
                    "high": float(b.get("high") or max(o, c)),
                    "low": float(b.get("low") or min(o, c)),
                    "close": c,
                    "volume": float(b.get("volume") or 0.0),
                    "is_completed": bool(b.get("is_completed", True)),
                }
            )
        cleaned.sort(key=lambda x: x["timestamp_utc_ms"])
        out[str(sym)] = cleaned
    return out


def bar_index_at(bars: List[Dict[str, Any]], ts_ms: int) -> Optional[int]:
    """Index of bar whose [open, open+15m) contains ts_ms; else last bar that opened <= ts."""
    if not bars:
        return None
    best = None
    for i, b in enumerate(bars):
        start = int(b["timestamp_utc_ms"])
        if start <= ts_ms < start + BAR_MS:
            return i
        if start <= ts_ms:
            best = i
    return best


def annotate_features(rows: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
    """Per-snapshot dynamics vs prior snap."""
    out = []
    for i, r in enumerate(rows):
        prev = rows[i - 1] if i > 0 else None
        bid = float(r["_bid_sum"] or 0.0)
        mid = r["_mid"]
        feat: Dict[str, Any] = {
            "symbol": r.get("symbol"),
            "sample_utc_ms": int(r.get("sample_utc_ms") or 0),
            "sample_iso": r.get("sample_iso"),
            "i5": r.get("_i5"),
            "bid_sum": bid,
            "ask_sum": float(r["_ask_sum"] or 0.0),
            "mid": mid,
            "last_price": r.get("last_price"),
            "half_spread_bps": half_spread_bps(r),
            "bid_delta": None,
            "mid_delta_bps": None,
            "apparent_replenish": False,
            "apparent_withdraw": False,
        }
        if prev is not None:
            pb = float(prev["_bid_sum"] or 0.0)
            pm = prev["_mid"]
            if pb > 0:
                feat["bid_delta"] = (bid - pb) / pb
                if feat["bid_delta"] <= -WITHDRAW_DROP:
                    feat["apparent_withdraw"] = True
            if pm and mid and pm > 0:
                feat["mid_delta_bps"] = ((mid - pm) / pm) * 10000.0
            # replenishment: prior snap withdrew, this snap recovers
            if prev.get("_prior_withdraw") and pb > 0 and bid >= pb * (1.0 + REPLENISH_BOUNCE):
                feat["apparent_replenish"] = True
            # mark for next
        r["_prior_withdraw"] = bool(feat["apparent_withdraw"])
        out.append(feat)
    # second pass: replenish using withdraw flag on previous feature row
    for i in range(1, len(out)):
        if out[i - 1]["apparent_withdraw"] and out[i]["bid_delta"] is not None:
            if out[i]["bid_delta"] >= REPLENISH_BOUNCE:
                out[i]["apparent_replenish"] = True
    return out


def detect_bid_support_while_falling(
    feats: List[Dict[str, Any]],
    *,
    persist: int = PERSIST_SNAPS,
    i5_min: float = I5_SUPPORT,
) -> List[Dict[str, Any]]:
    """Windows where I5 stays supportive and mid/last falls across `persist` snaps."""
    events: List[Dict[str, Any]] = []
    n = len(feats)
    if n < persist:
        return events
    i = persist - 1
    while i < n:
        window = feats[i - persist + 1 : i + 1]
        i5s = [w["i5"] for w in window]
        mids = [w["mid"] for w in window]
        if any(v is None for v in i5s) or any(v is None for v in mids):
            i += 1
            continue
        if not all(float(v) >= i5_min for v in i5s):
            i += 1
            continue
        bid0 = float(window[0]["bid_sum"] or 0.0)
        bid1 = float(window[-1]["bid_sum"] or 0.0)
        if bid0 <= 0:
            i += 1
            continue
        bid_persist = bid1 >= bid0 * (1.0 - BID_SUM_TOL)
        mid0 = float(mids[0])
        mid1 = float(mids[-1])
        if mid0 <= 0:
            i += 1
            continue
        price_bps = ((mid1 - mid0) / mid0) * 10000.0
        falling = price_bps < 0.0
        if bid_persist and falling:
            replenish_any = any(bool(w["apparent_replenish"]) for w in window)
            withdraw_any = any(bool(w["apparent_withdraw"]) for w in window)
            end = window[-1]
            events.append(
                {
                    "symbol": end["symbol"],
                    "event_utc_ms": int(end["sample_utc_ms"]),
                    "event_iso": end["sample_iso"],
                    "persist_snaps": persist,
                    "i5_end": float(end["i5"]),
                    "i5_min_window": float(min(i5s)),
                    "bid_sum_start": bid0,
                    "bid_sum_end": bid1,
                    "price_move_bps": round(price_bps, 3),
                    "replenish_in_window": replenish_any,
                    "withdraw_in_window": withdraw_any,
                    "half_spread_bps": float(end["half_spread_bps"] or 0.0),
                    "entry_mid": mid1,
                    "classification": (
                        "potential_bullish_absorption"
                        if replenish_any or bid_persist
                        else "bid_support_price_down"
                    ),
                }
            )
            # skip ahead to reduce overlapping duplicates
            i += persist
            continue
        i += 1
    return events


def attach_candle_and_forward(
    events: List[Dict[str, Any]],
    bars_by_sym: Dict[str, List[Dict[str, Any]]],
) -> List[Dict[str, Any]]:
    enriched = []
    for ev in events:
        sym = str(ev["symbol"])
        bars = bars_by_sym.get(sym) or []
        idx = bar_index_at(bars, int(ev["event_utc_ms"]))
        row = dict(ev)
        row["bar_idx"] = idx
        row["candle_red"] = None
        row["forming_vs_open_bps"] = None
        if idx is not None and 0 <= idx < len(bars):
            b = bars[idx]
            o = float(b["open"])
            c = float(b["close"])
            entry = float(ev["entry_mid"])
            row["bar_open"] = o
            row["bar_close"] = c
            row["candle_red"] = c < o if b.get("is_completed") else (entry < o)
            if o > 0:
                row["forming_vs_open_bps"] = round(((entry - o) / o) * 10000.0, 3)
            # forward completed-bar closes
            for h in (1, 2, 4):
                j = idx + h
                key_g = f"fwd_{h}_bar_gross_bps"
                key_n = f"fwd_{h}_bar_net_bps"
                if j < len(bars) and bars[j].get("is_completed", True):
                    px = float(bars[j]["close"])
                    if entry > 0:
                        gross = ((px - entry) / entry) * 10000.0
                        hs = float(ev.get("half_spread_bps") or 0.0)
                        # long observational: buy ~ mid+half_spread, sell ~ future mid-half_spread
                        # plus statutory friction on 1 share notional proxy
                        buy_px = entry * (1.0 + hs / 10000.0)
                        sell_px = px * (1.0 - hs / 10000.0)
                        fr = calc_indian_equity_friction(
                            buy_price=buy_px,
                            sell_price=sell_px,
                            quantity=1.0,
                            half_spread_bps=0.0,  # already in prices
                        )
                        fee_bps = 0.0
                        notional = buy_px
                        try:
                            # half_spread already in buy/sell prices; avoid double-counting spread/slippage
                            # by using statutory components only via effective friction with hs=0 path.
                            total_fr = float(fr.get("total_friction") or 0.0)
                            if notional > 0:
                                fee_bps = (total_fr / notional) * 10000.0
                        except (TypeError, ValueError, AttributeError):
                            fee_bps = 0.0
                        net = ((sell_px - buy_px) / buy_px) * 10000.0 - fee_bps
                        row[key_g] = round(gross, 3)
                        row[key_n] = round(net, 3)
                        row[f"fwd_{h}_available"] = True
                    else:
                        row[f"fwd_{h}_available"] = False
                else:
                    row[f"fwd_{h}_available"] = False
                    row[key_g] = None
                    row[key_n] = None
        enriched.append(row)
    return enriched


def _session_id(ev: Dict[str, Any]) -> str:
    """NSE cash session date in IST (independent-session counter)."""
    ts = int(ev.get("event_utc_ms") or 0)
    if ts <= 0:
        return "unknown"
    # IST = UTC+5:30
    ist_ms = ts + int(5.5 * 3600 * 1000)
    return datetime.fromtimestamp(ist_ms / 1000.0, tz=timezone.utc).strftime("%Y-%m-%d")


def summarize(events: List[Dict[str, Any]], snaps_per_sym: Dict[str, int]) -> Dict[str, Any]:
    n = len(events)
    red = [e for e in events if e.get("candle_red") is True]
    weak = [
        e
        for e in events
        if e.get("candle_red") is True
        or (e.get("forming_vs_open_bps") is not None and float(e["forming_vs_open_bps"]) < 0)
    ]
    with_repl = [e for e in weak if e.get("replenish_in_window")]
    no_repl = [e for e in weak if not e.get("replenish_in_window")]
    with_wd = [e for e in weak if e.get("withdraw_in_window")]
    no_wd = [e for e in weak if not e.get("withdraw_in_window")]

    symbols = {str(e.get("symbol")) for e in events if e.get("symbol")}
    sessions = {_session_id(e) for e in events}
    sym_session = {
        f"{e.get('symbol')}|{_session_id(e)}" for e in events if e.get("symbol")
    }

    def _horizon_stats(subset: List[Dict[str, Any]], h: int) -> Dict[str, Any]:
        rows = [
            e
            for e in subset
            if e.get(f"fwd_{h}_available") and e.get(f"fwd_{h}_bar_net_bps") is not None
        ]
        nets = [float(e[f"fwd_{h}_bar_net_bps"]) for e in rows]
        if not nets:
            return {
                "n_events": 0,
                "n_unique_symbols": 0,
                "n_sessions": 0,
                "mean_net_bps": None,
                "hit_rate": None,
            }
        hits = sum(1 for x in nets if x > 0)
        return {
            "n_events": len(nets),
            "n_unique_symbols": len({str(e.get("symbol")) for e in rows}),
            "n_sessions": len({_session_id(e) for e in rows}),
            "mean_net_bps": round(sum(nets) / len(nets), 3),
            "hit_rate": round(hits / len(nets), 3),
        }

    min_snaps = min(snaps_per_sym.values()) if snaps_per_sym else 0
    # Independence-aware gate: events alone are not enough.
    claim_ok = (
        n >= MIN_EVENTS_FOR_CLAIM
        and len(symbols) >= 20
        and len(sessions) >= 3
        and min_snaps >= 60
    )
    return {
        "independence": {
            "n_events": n,
            "n_unique_symbols": len(symbols),
            "n_independent_sessions": len(sessions),
            "n_symbol_session_pairs": len(sym_session),
            "sessions": sorted(sessions),
            "note": (
                "Event count ≠ independent observations. Report events, unique symbols, "
                "and sessions separately; do not treat clustered intra-symbol windows as IID."
            ),
        },
        "n_events_bid_support_price_down": n,
        "n_events_candle_red": len(red),
        "n_events_price_below_open": len(weak),
        "n_events_replenish_in_weak": len(with_repl),
        "n_events_withdraw_in_weak": len(with_wd),
        "fwd_1_all": _horizon_stats(events, 1),
        "fwd_2_all": _horizon_stats(events, 2),
        "fwd_4_all": _horizon_stats(events, 4),
        "fwd_1_red_or_below_open": _horizon_stats(weak, 1),
        "fwd_2_red_or_below_open": _horizon_stats(weak, 2),
        "fwd_4_red_or_below_open": _horizon_stats(weak, 4),
        # Incremental information checks (still observational).
        "fwd_1_weak_with_replenish": _horizon_stats(with_repl, 1),
        "fwd_1_weak_without_replenish": _horizon_stats(no_repl, 1),
        "fwd_1_weak_with_withdraw": _horizon_stats(with_wd, 1),
        "fwd_1_weak_without_withdraw": _horizon_stats(no_wd, 1),
        "fwd_2_weak_with_replenish": _horizon_stats(with_repl, 2),
        "fwd_4_weak_with_replenish": _horizon_stats(with_repl, 4),
        "tape": {
            "symbols": len(snaps_per_sym),
            "min_snaps_per_symbol": min_snaps,
            "max_snaps_per_symbol": max(snaps_per_sym.values()) if snaps_per_sym else 0,
            "median_snaps_per_symbol": (
                sorted(snaps_per_sym.values())[len(snaps_per_sym) // 2] if snaps_per_sym else 0
            ),
        },
        "next_measurement_checklist": [
            "bid support persistence across consecutive snapshots while mid falls",
            "replenishment incremental value vs static I5",
            "apparent withdrawal as invalidation of bullish-looking support",
            "fwd-1 / fwd-2 / fwd-4 net after Indian friction",
            "consistency across symbols and sessions (not a few names)",
        ],
        "verdict": {
            "actionable_claim_authorized": False,
            "reason": (
                "PASS_SAMPLE"
                if claim_ok
                else "TELEMETRY_ONLY_INSUFFICIENT_HISTORY_OR_EVENTS"
            ),
            "min_events_for_claim": MIN_EVENTS_FOR_CLAIM,
            "min_unique_symbols_for_claim": 20,
            "min_sessions_for_claim": 3,
            "note": (
                "Accumulate ordered depth history until fwd-1/2/4 are populated and "
                "independence counts clear the gate. OF ±0.30 remains frozen."
            ),
        },
    }


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--snapshots", type=Path, default=BOOK_DIR / "depth_snapshots.jsonl")
    ap.add_argument("--bars", type=Path, default=NSE_DIR / "rp608_nse_bar_history.json")
    ap.add_argument("--out-dir", type=Path, default=BOOK_DIR)
    args = ap.parse_args()
    args.out_dir.mkdir(parents=True, exist_ok=True)

    by = load_snapshots(args.snapshots)
    bars = load_bars(args.bars)
    snaps_per = {s: len(v) for s, v in by.items()}

    feature_path = args.out_dir / "phase2_depth_features.jsonl"
    event_path = args.out_dir / "phase2_bid_support_while_falling_events.jsonl"
    summary_path = args.out_dir / "phase2_feature_summary.json"

    all_events: List[Dict[str, Any]] = []
    with feature_path.open("w", encoding="utf-8") as ff:
        for sym, rows in sorted(by.items()):
            if len(rows) < MIN_SNAPS_FOR_FEATURES:
                continue
            feats = annotate_features(rows)
            for feat in feats:
                ff.write(json.dumps(feat, separators=(",", ":")) + "\n")
            evs = detect_bid_support_while_falling(feats)
            all_events.extend(evs)

    all_events = attach_candle_and_forward(all_events, bars)
    with event_path.open("w", encoding="utf-8") as ef:
        for ev in all_events:
            ef.write(json.dumps(ev, separators=(",", ":")) + "\n")

    summary = {
        "protocol_id": PROTOCOL,
        "phase": PHASE,
        "generated_utc": datetime.now(timezone.utc).isoformat(),
        "params": {
            "i5_support": I5_SUPPORT,
            "persist_snaps": PERSIST_SNAPS,
            "bid_sum_tol": BID_SUM_TOL,
            "replenish_bounce": REPLENISH_BOUNCE,
            "withdraw_drop": WITHDRAW_DROP,
            "note": "Observational params only — not OF entry thresholds",
        },
        "governance": {
            "of_pm30_frozen": True,
            "reverse_of_paper_frozen": True,
            "live_order_authority_pct": 0.0,
        },
        "summary": summarize(all_events, snaps_per),
        "paths": {
            "features": str(feature_path),
            "events": str(event_path),
        },
    }
    summary_path.write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({
        "wrote_summary": str(summary_path),
        "independence": summary["summary"]["independence"],
        "tape": summary["summary"]["tape"],
        "verdict": summary["summary"]["verdict"]["reason"],
        "fwd_1_red": summary["summary"]["fwd_1_red_or_below_open"],
        "fwd_2_red": summary["summary"]["fwd_2_red_or_below_open"],
        "fwd_4_red": summary["summary"]["fwd_4_red_or_below_open"],
        "fwd_1_replenish": summary["summary"]["fwd_1_weak_with_replenish"],
        "fwd_1_no_replenish": summary["summary"]["fwd_1_weak_without_replenish"],
        "fwd_1_withdraw": summary["summary"]["fwd_1_weak_with_withdraw"],
        "fwd_1_no_withdraw": summary["summary"]["fwd_1_weak_without_withdraw"],
    }, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
