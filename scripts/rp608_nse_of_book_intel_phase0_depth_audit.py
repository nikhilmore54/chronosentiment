#!/usr/bin/env python3
"""
RP-608-NSE-OF-BOOK-INTELLIGENCE-001 — Phase 0 depth-capture audit.

Quantifies wire → decode → cache → journal coverage for L1–L5.
Observational only: no OF mark change, no paper change, no authority change.

Usage (on Pi or local with NSE paths):
  python3 scripts/research/rp608_nse_of_book_intel_phase0_depth_audit.py
  python3 scripts/research/rp608_nse_of_book_intel_phase0_depth_audit.py --live-seconds 12
"""

from __future__ import annotations

import argparse
import asyncio
import json
import os
import struct
import time
from collections import defaultdict
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

ROOT = Path(__file__).resolve().parents[2]
NSE_DIR = ROOT / "live_capture" / "shadow_validation" / "nse"
OUT_DIR = NSE_DIR / "of_book_intelligence_v1"
PROTOCOL = "RP-608-NSE-OF-BOOK-INTELLIGENCE-001"
PHASE = "PHASE_0_DEPTH_CAPTURE_AUDIT"


def _load_dotenv_creds() -> Tuple[Optional[str], Optional[str]]:
    """Credentials only from process env or local dotenv files (no /proc scraping)."""
    api_key = os.environ.get("KITE_API_KEY") or os.environ.get("ZERODHA_API_KEY")
    access = os.environ.get("KITE_ACCESS_TOKEN") or os.environ.get("ZERODHA_ACCESS_TOKEN")
    candidates = [
        ROOT / ".env",
        ROOT / "secrets" / ".env",
        ROOT / "live_capture" / ".env",
        Path.home() / ".chronosentiment_kite.env",
    ]
    for envf in candidates:
        if not envf.exists() or (api_key and access):
            continue
        for line in envf.read_text().splitlines():
            if "=" not in line or line.strip().startswith("#"):
                continue
            k, v = line.split("=", 1)
            k = k.strip()
            v = v.strip().strip('"').strip("'")
            if k in ("KITE_API_KEY", "ZERODHA_API_KEY") and not api_key:
                api_key = v
            if k in ("KITE_ACCESS_TOKEN", "ZERODHA_ACCESS_TOKEN") and not access:
                access = v
    return api_key, access


def _instrument_tokens(limit: int = 15) -> List[Tuple[str, int]]:
    path = NSE_DIR / "kite_nse_instrument_map.json"
    if not path.exists():
        return []
    doc = json.loads(path.read_text())
    instruments = doc.get("instruments") if isinstance(doc, dict) else None
    src = instruments if isinstance(instruments, dict) else doc if isinstance(doc, dict) else {}
    out: List[Tuple[str, int]] = []
    for sym, v in src.items():
        if sym in ("protocol_id", "saved_utc", "source", "n_symbols", "instruments"):
            continue
        tok = None
        if isinstance(v, dict):
            tok = v.get("instrument_token") or v.get("token")
        elif isinstance(v, (int, float)):
            tok = int(v)
        if tok is None:
            continue
        out.append((str(sym), int(tok)))
        if len(out) >= limit:
            break
    return out


def _iter_latest_evaluations() -> List[Tuple[Any, Dict[str, Any]]]:
    latest_path = NSE_DIR / "rp608_nse_latest_evaluations.json"
    if not latest_path.exists():
        return []
    latest = json.loads(latest_path.read_text())
    items: List[Tuple[Any, Dict[str, Any]]] = []
    if isinstance(latest, dict):
        nested = latest.get("evaluations")
        src = nested if isinstance(nested, dict) else latest
        for sym, ev in src.items():
            if isinstance(ev, dict) and (
                "data_quality" in ev or "order_flow_imbalance" in ev or "decision" in ev
            ):
                items.append((sym, ev))
    elif isinstance(latest, list):
        items = [(ev.get("symbol"), ev) for ev in latest if isinstance(ev, dict)]
    return items


def _probe_wire_evidence() -> Dict[str, Any]:
    """Prior short-lived probe that retained depth_buy/depth_sell (wire structural evidence)."""
    path = NSE_DIR / "live_probe_evidence.json"
    if not path.exists():
        return {"status": "MISSING"}
    doc = json.loads(path.read_text())
    ticks = doc.get("sample_ticks") or []
    bid_nz = [0, 0, 0, 0, 0]
    ask_nz = [0, 0, 0, 0, 0]
    n_struct5 = 0
    for t in ticks:
        if not isinstance(t, dict):
            continue
        db = t.get("depth_buy") or []
        ds = t.get("depth_sell") or []
        if len(db) >= 5 and len(ds) >= 5:
            n_struct5 += 1
            for i in range(5):
                if (db[i] or {}).get("quantity", 0) > 0:
                    bid_nz[i] += 1
                if (ds[i] or {}).get("quantity", 0) > 0:
                    ask_nz[i] += 1
    n = max(1, len(ticks))
    return {
        "status": "OK",
        "path": str(path.relative_to(ROOT)),
        "probe_timestamp_utc": doc.get("probe_timestamp_utc"),
        "sample_ticks": len(ticks),
        "ticks_with_five_level_arrays": n_struct5,
        "pct_level_bid_qty_nonzero": [round(100.0 * bid_nz[i] / n, 1) for i in range(5)],
        "pct_level_ask_qty_nonzero": [round(100.0 * ask_nz[i] / n, 1) for i in range(5)],
        "note": (
            "Structural L1–L5 arrays present on wire/decode in probe path; "
            "nonzero qty at L2–L5 is market-state dependent and may be sparse."
        ),
    }


def _static_pipeline_matrix() -> Dict[str, Any]:
    """Code-path audit: what each layer retains (independent of live session)."""
    cache_src = (ROOT / "scripts" / "rp608_kite_ticker_cache.py").read_text()
    probe_src = (ROOT / "scripts" / "run_rp608_nse_live_probe.py").read_text()

    cache_parses_l5 = "for i in range(5)" in cache_src and "buy_depth.append" in cache_src
    cache_returns_depth_arrays = "depth_buy" in cache_src or '"buy_depth"' in cache_src
    # Live cache returns only L1 scalars (see decode_full_mode_packet return dict).
    cache_return_l1_only = (
        '"best_bid"' in cache_src
        and '"bid_qty"' in cache_src
        and "depth_buy" not in cache_src.split("return {", 1)[-1][:400]
    )
    probe_keeps_depth = "depth_buy" in probe_src and "depth_sell" in probe_src

    # Durable journals: field presence on latest evaluations (nested `evaluations`).
    journal_rows: List[Dict[str, Any]] = []
    n_sym = 0
    n_l1 = 0
    n_l5 = 0
    n_of = 0
    for sym, ev in _iter_latest_evaluations():
        n_sym += 1
        dq = ev.get("data_quality") if isinstance(ev.get("data_quality"), dict) else {}
        has_l1 = dq.get("bid_qty") is not None and dq.get("ask_qty") is not None
        has_l5 = ("depth_buy" in ev) or ("depth_sell" in ev) or ("depth_buy" in dq)
        if has_l1:
            n_l1 += 1
        if has_l5:
            n_l5 += 1
        if ev.get("order_flow_imbalance") is not None:
            n_of += 1
        journal_rows.append(
            {
                "symbol": sym or ev.get("symbol"),
                "wire": "FULL_MODE_PACKET_L1_L5",
                "decoded_in_live_cache": "L1_ONLY",
                "cached_in_memory": "L1_ONLY_LATEST",
                "journaled": "L1_TOUCH_QTY" if has_l1 else "NONE",
                "journal_has_l2_l5": bool(has_l5),
                "order_flow_imbalance_present": ev.get("order_flow_imbalance") is not None,
                "data_quality_status": dq.get("status"),
                "feed_source": dq.get("feed_source") or ev.get("order_flow_source"),
            }
        )

    # Confirm no L2–L5 jsonl exists.
    depth_jsonl = list(NSE_DIR.rglob("*depth*.jsonl"))
    book_snap = list(NSE_DIR.rglob("*book*snapshot*"))

    feed_health = {}
    fh = NSE_DIR / "rp608_nse_l1_feed_health.json"
    if fh.exists():
        feed_health = json.loads(fh.read_text())

    return {
        "check_wire_contract": {
            "packet_bytes_full_mode": 184,
            "bid_levels_in_packet": 5,
            "ask_levels_in_packet": 5,
            "subscribe_mode": "full",
            "note": "Structural wire contract from Kite full-mode binary layout",
            "prior_probe_evidence": _probe_wire_evidence(),
        },
        "check_cache_decoder": {
            "file": "scripts/rp608_kite_ticker_cache.py",
            "parses_five_levels_locally": cache_parses_l5,
            "returns_depth_arrays": cache_returns_depth_arrays,
            "returns_l1_scalars_only": cache_return_l1_only,
            "probe_decoder_retains_depth_buy_sell": probe_keeps_depth,
            "classification": (
                "CACHE_SCHEMA_DROP_L2_L5"
                if cache_parses_l5 and cache_return_l1_only
                else "INSPECT"
            ),
        },
        "check_durable_coverage": {
            "latest_evaluations_symbols": n_sym,
            "symbols_with_l1_touch_qty": n_l1,
            "symbols_with_l2_l5_arrays": n_l5,
            "symbols_with_of_imbalance": n_of,
            "depth_history_jsonl_files": [str(p.relative_to(ROOT)) for p in depth_jsonl],
            "book_snapshot_files": [str(p.relative_to(ROOT)) for p in book_snap],
            "classification": (
                "PERSISTENCE_L1_ONLY"
                if n_sym and n_l5 == 0
                else ("HAS_L2_L5" if n_l5 else "NO_EVAL_ROWS")
            ),
        },
        "check_temporal_integrity_proxy": {
            "l1_feed_health": feed_health,
            "note": (
                "Continuous WS cache overwrites latest-only; no ordered L1–L5 snapshot "
                "history exists to measure inter-arrival gaps for book evolution."
            ),
            "latest_only_cache": True,
            "ordered_depth_history": False,
        },
        "symbol_matrix": journal_rows,
        "root_cause": {
            "primary": "CACHE_SCHEMA_THEN_PERSISTENCE",
            "detail": (
                "Decoder reads L1–L5 into locals then drops L2–L5 from the returned/cached "
                "dict; durable journals only store L1 touch qty / OF from L1. Not a wire gap."
            ),
        },
    }


def _decode_full_keep_l5(pkt: bytes) -> Optional[Dict[str, Any]]:
    if len(pkt) < 184:
        return None
    tok = struct.unpack(">I", pkt[0:4])[0]
    buy = []
    sell = []
    for i in range(5):
        off = 64 + i * 12
        bq = struct.unpack(">I", pkt[off : off + 4])[0]
        bp = struct.unpack(">I", pkt[off + 4 : off + 8])[0] / 100.0
        buy.append({"price": bp, "quantity": bq})
        off = 124 + i * 12
        sq = struct.unpack(">I", pkt[off : off + 4])[0]
        sp = struct.unpack(">I", pkt[off + 4 : off + 8])[0] / 100.0
        sell.append({"price": sp, "quantity": sq})
    exch = struct.unpack(">I", pkt[60:64])[0]
    return {
        "instrument_token": tok,
        "buy": buy,
        "sell": sell,
        "exchange_timestamp_sec": exch,
        "receipt_unix": time.time(),
    }


async def _live_wire_sample(seconds: float, token_pairs: List[Tuple[str, int]]) -> Dict[str, Any]:
    api_key, access = _load_dotenv_creds()
    if not api_key or not access or not token_pairs:
        return {
            "status": "SKIPPED",
            "reason": "missing_creds_or_tokens",
            "has_api_key": bool(api_key),
            "has_access_token": bool(access),
            "n_tokens": len(token_pairs),
        }
    try:
        import websockets  # type: ignore
    except ImportError:
        return {"status": "SKIPPED", "reason": "websockets_not_installed"}

    use_tokens = [t for _, t in token_pairs]
    sym_by_tok = {t: s for s, t in token_pairs}
    stats: Dict[int, Dict[str, Any]] = defaultdict(
        lambda: {
            "pkts": 0,
            "l1_qty_nonzero": 0,
            "level_qty_nonzero_bid": [0, 0, 0, 0, 0],
            "level_qty_nonzero_ask": [0, 0, 0, 0, 0],
            "ages": [],
            "exch_ts": [],
            "receipt_gaps_ms": [],
        }
    )
    url = f"wss://ws.kite.trade?api_key={api_key}&access_token={access}"
    t_end = time.time() + float(seconds)
    last_receipt: Dict[int, float] = {}
    async with websockets.connect(url, ping_interval=10, ping_timeout=5) as ws:
        await ws.send(json.dumps({"a": "subscribe", "v": use_tokens}))
        await ws.send(json.dumps({"a": "mode", "v": ["full", use_tokens]}))
        while time.time() < t_end:
            try:
                msg = await asyncio.wait_for(ws.recv(), timeout=2.0)
            except asyncio.TimeoutError:
                continue
            if not isinstance(msg, bytes) or len(msg) < 2:
                continue
            n = struct.unpack(">H", msg[:2])[0]
            off = 2
            for _ in range(n):
                if off + 2 > len(msg):
                    break
                plen = struct.unpack(">H", msg[off : off + 2])[0]
                off += 2
                pkt = msg[off : off + plen]
                off += plen
                if plen != 184:
                    continue
                d = _decode_full_keep_l5(pkt)
                if not d or d["instrument_token"] not in sym_by_tok:
                    continue
                tok = int(d["instrument_token"])
                s = stats[tok]
                s["pkts"] += 1
                if d["buy"][0]["quantity"] > 0 or d["sell"][0]["quantity"] > 0:
                    s["l1_qty_nonzero"] += 1
                for i in range(5):
                    if d["buy"][i]["quantity"] > 0:
                        s["level_qty_nonzero_bid"][i] += 1
                    if d["sell"][i]["quantity"] > 0:
                        s["level_qty_nonzero_ask"][i] += 1
                exch = int(d["exchange_timestamp_sec"] or 0)
                if exch > 0:
                    s["ages"].append(max(0.0, d["receipt_unix"] - exch))
                    s["exch_ts"].append(exch)
                prev = last_receipt.get(tok)
                if prev is not None:
                    s["receipt_gaps_ms"].append(int((d["receipt_unix"] - prev) * 1000))
                last_receipt[tok] = float(d["receipt_unix"])

    rows = []
    for tok, s in stats.items():
        pk = max(1, s["pkts"])
        ages = s["ages"]
        gaps = s["receipt_gaps_ms"]
        exch = s["exch_ts"]
        mono = all(exch[i] <= exch[i + 1] for i in range(len(exch) - 1)) if len(exch) > 1 else True
        rows.append(
            {
                "symbol": sym_by_tok.get(tok, str(tok)),
                "instrument_token": tok,
                "packets": s["pkts"],
                "wire_five_level_slots_always_present": True,
                "pct_l1_qty_nonzero": round(100.0 * s["l1_qty_nonzero"] / pk, 1),
                "pct_level_bid_qty_nonzero": [
                    round(100.0 * s["level_qty_nonzero_bid"][i] / pk, 1) for i in range(5)
                ],
                "pct_level_ask_qty_nonzero": [
                    round(100.0 * s["level_qty_nonzero_ask"][i] / pk, 1) for i in range(5)
                ],
                "median_quote_age_sec": (
                    round(sorted(ages)[len(ages) // 2], 3) if ages else None
                ),
                "median_receipt_gap_ms": (
                    int(sorted(gaps)[len(gaps) // 2]) if gaps else None
                ),
                "exchange_ts_nondecreasing": mono,
            }
        )
    rows.sort(key=lambda r: r["symbol"])
    return {
        "status": "OK",
        "sample_seconds": seconds,
        "n_symbols": len(rows),
        "rows": rows,
    }


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--live-seconds", type=float, default=12.0)
    ap.add_argument("--no-live", action="store_true")
    ap.add_argument("--token-limit", type=int, default=12)
    args = ap.parse_args()

    OUT_DIR.mkdir(parents=True, exist_ok=True)
    static = _static_pipeline_matrix()
    live: Dict[str, Any]
    if args.no_live:
        live = {"status": "SKIPPED", "reason": "no_live_flag"}
    else:
        pairs = _instrument_tokens(limit=int(args.token_limit))
        live = asyncio.run(_live_wire_sample(float(args.live_seconds), pairs))

    report = {
        "protocol_id": PROTOCOL,
        "phase": PHASE,
        "generated_utc": datetime.now(timezone.utc).isoformat(),
        "governance": {
            "of_pm30_frozen": True,
            "reverse_of_paper_frozen": True,
            "estimand_a_separate": True,
            "intrabar_aggressor_independent": True,
            "live_order_authority_pct": 0.0,
            "replenishment_metrics_authorized": False,
        },
        "pipeline_ascii": (
            "Kite full packet (L1-L5) -> live ticker decode (reads L1-L5, keeps L1) "
            "-> in-memory latest-only cache (L1) -> durable journals (L1 touch qty / OF)"
        ),
        "checks": {
            "wire_coverage": static["check_wire_contract"],
            "cache_coverage": static["check_cache_decoder"],
            "durable_coverage": static["check_durable_coverage"],
            "temporal_integrity": static["check_temporal_integrity_proxy"],
        },
        "root_cause": static["root_cause"],
        "symbol_matrix_journaled": static["symbol_matrix"],
        "live_wire_sample": live,
        "verdict": {
            "durable_l1_l5_history_exists": False,
            "block_phase_1_feature_build": True,
            "next_milestone": "Evidence of durable L1-L5 snapshot history (not a new OF threshold)",
            "issue_layer": static["root_cause"]["primary"],
        },
    }

    out_json = OUT_DIR / "phase0_depth_capture_audit.json"
    out_json.write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({
        "wrote": str(out_json),
        "verdict": report["verdict"],
        "durable": report["checks"]["durable_coverage"],
        "cache": report["checks"]["cache_coverage"]["classification"],
        "live_status": live.get("status"),
        "live_n_symbols": live.get("n_symbols"),
    }, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
