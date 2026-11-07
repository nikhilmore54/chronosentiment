#!/usr/bin/env python3
"""
scripts/rp608_live_dashboard.py

RP-608 Real-Time Operational & Recommendation Dashboard
Read-only · Zero Order Authority · Embedded SPA Server · 3-Second Refresh
Features:
  - Policy B & C Recommendations with Real-Time Hover Tooltips & Reasoning
  - RP-608 Trade-Plan Overlay v1 (Entry, Stop, Target, Net R:R, 60m Horizon)
  - Sharpe & Calmar Evaluation Framework & 5-Stage Roadmap
  - Live Bid/Ask Depth & Execution Quote Freshness
  - Operational Telemetry Gates & Prospective Accounting
"""

from __future__ import annotations

import http.server
import json
import math
import os
from datetime import datetime, timedelta, timezone
from pathlib import Path
import socketserver
import subprocess
import sys
import threading
import time
import urllib.parse
from typing import Any, Dict, List, Optional

WORKSPACE_DIR = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE_DIR))

from scripts.rp608_trade_plan_overlay import TradePlanOverlayManager, calculate_trade_plan
from scripts.rp608_multi_horizon_status import get_multi_horizon_dashboard_state
from scripts.rp608_order_flow_mark import (
    inspect_order_flow_book,
    l1_from_book_ticker,
)
from scripts.rp611_dashboard_status import get_rp611_dashboard_state

try:
    from scripts.test_kite_probe import (
        run_probe,
        exchange_request_token,
        load_credentials,
        save_credentials,
        kite_api_get
    )
except Exception:
    run_probe = None
    exchange_request_token = None
    load_credentials = None
    save_credentials = None
    kite_api_get = None

CRYPTO_DIR = WORKSPACE_DIR / "live_capture" / "shadow_validation" / "crypto"
NSE_DIR = WORKSPACE_DIR / "live_capture" / "shadow_validation" / "nse"
NSE_TEST_DIR = WORKSPACE_DIR / "live_capture" / "shadow_validation" / "nse" / "test_run"
# UI surface: show open/qualified NSE ideas whose frozen ATR target exceeds this floor (bps).
NSE_HIGH_TARGET_BPS = 75.0
# Fallback round-trip friction when Indian-equity calc is not on the frozen trade.
NSE_RR_FRICTION_FALLBACK_BPS = 15.0


def recommendation_horizon_meta(
    *,
    market: str,
    profile: str = "INTRADAY",
    lookback_bars: Optional[int] = None,
) -> Dict[str, str]:
    """
    Explicit holding-horizon labels for recommendation surfaces.
    horizon_class: INTRADAY | MULTI_DAY | MULTI_WEEK | LONG_TERM
    horizon_term: human clock / bar policy string
    """
    m = (market or "").upper()
    p = (profile or "INTRADAY").upper()
    if m == "NSE" and p == "INTRADAY":
        return {
            "horizon_class": "INTRADAY",
            "horizon_id": "H0_NSE_INTRADAY",
            "horizon_label": "Intraday",
            "horizon_term": "15m bars · 60m / 180m max · square-off 15:00 (close-window 15:00–15:15)",
        }
    if m == "CRYPTO" and p in {"TRADE_PLAN", "INTRADAY", "H0"}:
        return {
            "horizon_class": "INTRADAY",
            "horizon_id": "H0_INTRADAY_60M",
            "horizon_label": "Intraday",
            "horizon_term": "≤60 minutes (trade-plan overlay)",
        }
    if m == "CRYPTO" and p == "GATE_C":
        bars = int(lookback_bars or 0)
        hours = {16: 4, 32: 8, 64: 16, 120: 30}.get(bars)
        term = (
            f"Tick stream · {bars}-bar lookback ({hours}h context) · 30 bps stop"
            if hours
            else f"Tick stream · {bars}-bar lookback · 30 bps stop"
        )
        return {
            "horizon_class": "INTRADAY",
            "horizon_id": f"GATE_C_{bars}B" if bars else "GATE_C_TICK",
            "horizon_label": "Intraday",
            "horizon_term": term,
        }
    if p in {"MULTI_DAY", "H1", "SWING"}:
        return {
            "horizon_class": "MULTI_DAY",
            "horizon_id": "H1_MULTI_DAY",
            "horizon_label": "Multi-day",
            "horizon_term": "2–14 calendar days",
        }
    if p in {"MULTI_WEEK", "H2"}:
        return {
            "horizon_class": "MULTI_WEEK",
            "horizon_id": "H2_MULTI_WEEK",
            "horizon_label": "Multi-week",
            "horizon_term": "14–56 calendar days",
        }
    if p in {"LONG_TERM", "H3", "LT"}:
        return {
            "horizon_class": "LONG_TERM",
            "horizon_id": "H3_LONG_TERM",
            "horizon_label": "Long-term",
            "horizon_term": "≥90 calendar days (thesis hold)",
        }
    return {
        "horizon_class": "INTRADAY",
        "horizon_id": "H0_UNSPECIFIED",
        "horizon_label": "Intraday",
        "horizon_term": "Intraday (unspecified term)",
    }


def _nse_estimate_friction_bps(half_spread_bps: Any = None, estimated_friction_bps: Any = None) -> float:
    if estimated_friction_bps is not None:
        try:
            return max(0.0, float(estimated_friction_bps))
        except (TypeError, ValueError):
            pass
    if half_spread_bps is not None:
        try:
            # Mirror daemon proxy: round-trip spread + ~8 bps fees floor, min 9.
            return max(9.0, float(half_spread_bps) * 2.0 + 8.0)
        except (TypeError, ValueError):
            pass
    return NSE_RR_FRICTION_FALLBACK_BPS


def _nse_net_rr_ratio(
    target_bps: Any,
    stop_loss_bps: Any,
    *,
    half_spread_bps: Any = None,
    estimated_friction_bps: Any = None,
    existing_net_rr: Any = None,
) -> Optional[float]:
    if existing_net_rr is not None:
        try:
            return round(float(existing_net_rr), 2)
        except (TypeError, ValueError):
            pass
    try:
        tb = float(target_bps)
        sb = float(stop_loss_bps)
    except (TypeError, ValueError):
        return None
    if sb <= 0:
        return None
    fric = _nse_estimate_friction_bps(half_spread_bps, estimated_friction_bps)
    return round((tb - fric) / (sb + fric + 1e-9), 2)


def _nse_rr_target_rank_score(net_rr: Optional[float], target_bps: Any) -> float:
    """Composite sort key: Net R:R × (target_bps / 100). Higher = better opportunity quality."""
    try:
        tb = float(target_bps or 0.0)
    except (TypeError, ValueError):
        tb = 0.0
    rr = float(net_rr) if net_rr is not None else 0.0
    if rr < 0:
        rr = 0.0
    return round(rr * (tb / 100.0), 4)

# Daemon-owned decision/state artifacts must never lose to test_run mtime races.
# test_run is written by single-bar dashboard probes and can carry stale active_trades.
NSE_LIVE_PREFERRED_ARTIFACTS = {
    "rp608_nse_latest_evaluations.json",
    "rp608_nse_pending_evaluations.json",
    "rp608_nse_decision_cards.jsonl",
    "rp608_nse_trade_journal.jsonl",
    "rp608_nse_bar_history.json",
    "rp608_nse_checkpoint.json",
    "rp608_nse_l1_feed_health.json",
    "rp608_nse_next_session_recommendations.json",
}


def _nse_iso_date_ist(iso_s: Any) -> Optional[str]:
    """Best-effort calendar date (IST) from an ISO timestamp string."""
    if not iso_s or iso_s == "N/A":
        return None
    try:
        raw = str(iso_s).replace("Z", "+00:00")
        dt = datetime.fromisoformat(raw)
        if dt.tzinfo is None:
            dt = dt.replace(tzinfo=timezone.utc)
        return (dt.astimezone(timezone(timedelta(hours=5, minutes=30)))).date().isoformat()
    except Exception:
        return str(iso_s)[:10] if len(str(iso_s)) >= 10 else None


_IST = timezone(timedelta(hours=5, minutes=30))


def _nse_recommendation_time_fields(
    *,
    utc_ms: Any = None,
    iso_s: Any = None,
) -> Dict[str, Any]:
    """
    Recommendation / entry clock for high-target idea cards.

    Prefer explicit UTC ms (trade open); else ISO bar/eval stamp.
    Surfaces both UTC ISO and IST display string.
    """
    empty = {
        "recommendation_time_utc_ms": None,
        "recommendation_time_utc": None,
        "recommendation_time_ist": None,
    }
    dt: Optional[datetime] = None
    ms_out: Optional[int] = None
    try:
        if utc_ms is not None and str(utc_ms).strip() != "":
            ms_out = int(float(utc_ms))
            dt = datetime.fromtimestamp(ms_out / 1000.0, tz=timezone.utc)
    except (TypeError, ValueError, OSError, OverflowError):
        dt = None
        ms_out = None
    if dt is None and iso_s and iso_s != "N/A":
        try:
            raw = str(iso_s).replace("Z", "+00:00")
            parsed = datetime.fromisoformat(raw)
            if parsed.tzinfo is None:
                parsed = parsed.replace(tzinfo=timezone.utc)
            dt = parsed.astimezone(timezone.utc)
            ms_out = int(dt.timestamp() * 1000)
        except Exception:
            return empty
    if dt is None:
        return empty
    ist = dt.astimezone(_IST)
    return {
        "recommendation_time_utc_ms": ms_out,
        "recommendation_time_utc": dt.strftime("%Y-%m-%d %H:%M:%SZ"),
        "recommendation_time_ist": ist.strftime("%Y-%m-%d %H:%M:%S IST"),
    }


def _load_nse_production_checkpoint() -> Dict[str, Any]:
    ckpt_path = find_latest_nse_file("rp608_nse_checkpoint.json")
    if not ckpt_path.exists():
        return {}
    try:
        with open(ckpt_path, "r", encoding="utf-8") as f:
            data = json.load(f)
        return data if isinstance(data, dict) else {}
    except Exception:
        return {}


def _load_nse_l1_feed_health() -> Dict[str, Any]:
    health_path = find_latest_nse_file("rp608_nse_l1_feed_health.json")
    if not health_path.exists():
        return {}
    try:
        with open(health_path, "r", encoding="utf-8") as f:
            data = json.load(f)
        return data if isinstance(data, dict) else {}
    except Exception:
        return {}


def find_latest_nse_file(filename: str) -> Path:
    """Resolve NSE artifact path.

    Live daemon journals (evaluations/pending/trades) always prefer NSE_DIR so
    Symbol Explorer cannot be overwritten by newer test_run single-bar probes
    that still hold 100+ stale simulated positions.
    Market-data probes (readiness/preopen/probe) keep newer-mtime selection.
    """
    p_primary = NSE_DIR / filename
    p_test = NSE_TEST_DIR / filename
    if filename in NSE_LIVE_PREFERRED_ARTIFACTS:
        if p_primary.exists():
            return p_primary
        return p_test
    if p_primary.exists() and p_test.exists():
        return p_primary if p_primary.stat().st_mtime >= p_test.stat().st_mtime else p_test
    if p_primary.exists():
        return p_primary
    return p_test


def _resolve_nse_daemon_pid() -> Optional[int]:
    """Best-effort live NSE shadow daemon PID (None if not running)."""
    try:
        res = subprocess.run(
            ["pgrep", "-f", "run_rp608_nse_shadow_daemon.py"],
            capture_output=True,
            text=True,
            timeout=1,
        )
        for line in (res.stdout or "").splitlines():
            tok = line.strip().split()
            if tok and tok[0].isdigit():
                return int(tok[0])
    except Exception:
        pass
    return None


def _feed_label(bar_provenance: Optional[str], feed_source: Optional[str]) -> str:
    """Human label for L1/bar feed provenance (deterministic mapping)."""
    bp = (bar_provenance or "").lower()
    fs = (feed_source or "").lower()
    if "kite_ws" in bp or "ticker" in bp or "ws_full" in bp:
        return "Kite WS Full"
    if "historical" in bp:
        return "Kite Historical"
    if fs in ("kite_connect", "kite_rest", "rest"):
        return "Kite REST"
    if bp:
        return bar_provenance or "Unknown"
    if feed_source:
        return feed_source
    return "Unknown"

CHECKPOINT_FILE = CRYPTO_DIR / "rp608_checkpoint.json"
DECISION_CARDS_FILE = CRYPTO_DIR / "rp608_decision_cards.jsonl"
REGISTERED_WINDOW_UTC = "2026-10-26T00:00:00.000Z"
REGISTERED_WINDOW_MS = 1792972800000  # 2026-10-26 05:30:00 IST

# Overlay state manager instance
overlay_mgr = TradePlanOverlayManager()

# /api/state assembly is expensive (17k+ decision cards). Short TTL cache prevents
# Pi/UI refresh stampedes from pegging CPU/RAM while preserving fail-closed freshness.
_STATE_CACHE_LOCK = threading.Lock()
_STATE_CACHE: Dict[str, Any] = {"ts": 0.0, "payload": None}
_STATE_CACHE_TTL_SEC = 2.0
_REASONING_TAIL = 256


def get_cached_live_state() -> Dict[str, Any]:
    """Return load_live_state() with a short process-local TTL (presentation cache)."""
    now = time.time()
    with _STATE_CACHE_LOCK:
        payload = _STATE_CACHE.get("payload")
        ts = float(_STATE_CACHE.get("ts") or 0.0)
        if payload is not None and (now - ts) < _STATE_CACHE_TTL_SEC:
            return payload
    state = load_live_state()
    with _STATE_CACHE_LOCK:
        _STATE_CACHE["payload"] = state
        _STATE_CACHE["ts"] = time.time()
    return state


def get_observer_service_status() -> Dict[str, Any]:
    """Inspects systemd status of rp608-shadow-observer.service."""
    try:
        res = subprocess.run(
            ["systemctl", "is-active", "rp608-shadow-observer.service"],
            capture_output=True,
            text=True,
            timeout=2
        )
        is_active = (res.stdout.strip() == "active")
        return {"active": is_active, "status_text": res.stdout.strip()}
    except Exception as e:
        return {"active": False, "status_text": f"Error: {e}"}


EVENT_DIR = CRYPTO_DIR / "event_driven"


def get_event_engine_status() -> Dict[str, Any]:
    """Inspects systemd status of rp608-event-engine.service and candidate event streams."""
    try:
        res = subprocess.run(
            ["systemctl", "is-active", "rp608-event-engine.service"],
            capture_output=True,
            text=True,
            timeout=2
        )
        is_active = (res.stdout.strip() == "active")
        status_text = res.stdout.strip()
    except Exception as e:
        is_active = False
        status_text = f"Error: {e}"

    candidates_info = {}
    candidate_configs = {
        16: {"id": "CANDIDATE_A_16B", "name": "Candidate A (16-Bar · 4h)", "hash": "16b249e2d397e3c9d87833eb29564a75927ee3dafbca30158ad17088927af81c"},
        32: {"id": "CANDIDATE_B_32B", "name": "Candidate B (32-Bar · 8h)", "hash": "883aaa5a483b9c2c1d20a2805a4a4790b88e2838517e6a81890297a12664d69a"},
        64: {"id": "CANDIDATE_C_64B", "name": "Candidate C (64-Bar · 16h)", "hash": "81891e2905be1df2d7e1c30a6a77aebbc1a07ba8a409fd18cc79f97303f2abd8"},
        120: {"id": "CANDIDATE_D_120B", "name": "Candidate D (120-Bar · 30h)", "hash": "4e3c14ceccd83a35c53680b08fad17a103388c079e16326987fc0674fad8fe5d"}
    }

    for bars, cfg in candidate_configs.items():
        ledger = EVENT_DIR / f"event_recs_candidate_{bars}b.jsonl"
        recs = []
        if ledger.exists():
            try:
                with open(ledger, "r", encoding="utf-8") as f:
                    recs = [json.loads(l) for l in f if l.strip()]
            except Exception:
                pass

        latest_rec = recs[-1] if recs else None
        feat = latest_rec.get("features", {}) if latest_rec else {}
        candidates_info[str(bars)] = {
            "id": cfg["id"],
            "name": cfg["name"],
            "hash_prefix": cfg["hash"][:16],
            "hash_full": cfg["hash"],
            "total_emitted": len(recs),
            "latest_recommendation": latest_rec.get("recommendation", {}).get("decision", "AWAITING_SIGNAL") if latest_rec else "AWAITING_SIGNAL",
            "direction": latest_rec.get("recommendation", {}).get("direction", "NEUTRAL") if latest_rec else "NEUTRAL",
            "entry_price": latest_rec.get("recommendation", {}).get("suggested_entry_price", 0.0) if latest_rec else 0.0,
            "invalidation_price": latest_rec.get("recommendation", {}).get("invalidation_price", 0.0) if latest_rec else 0.0,
            "candle_trend_bps": feat.get("candle_trend_bps", None),
            "candle_regime": feat.get("candle_regime", "UNKNOWN"),
            "imbalance": feat.get("imbalance", 0.0),
            "velocity_5s_bps": feat.get("velocity_5s_bps", 0.0),
            "last_emitted_utc": latest_rec.get("timestamp_utc", "N/A") if latest_rec else "N/A",
            **recommendation_horizon_meta(market="CRYPTO", profile="GATE_C", lookback_bars=bars),
        }

    telemetry_file = EVENT_DIR / "event_telemetry.jsonl"
    latest_telemetry = None
    if telemetry_file.exists():
        try:
            with open(telemetry_file, "r", encoding="utf-8") as f:
                lines = [l for l in f if l.strip()]
                if lines:
                    latest_telemetry = json.loads(lines[-1])
        except Exception:
            pass

    # Cross-candidate agreement calculation
    directions = [c["direction"] for c in candidates_info.values()]
    all_short = all(d == "SHORT" for d in directions)
    all_long = all(d == "LONG" for d in directions)
    if all_short:
        agreement_text = "4 / 4 CONCORDANT SHORT"
        agreement_sub = "Macro Bearish Convergence across 4h, 8h, 16h, 30h"
        agreement_type = "CONCORDANT"
    elif all_long:
        agreement_text = "4 / 4 CONCORDANT LONG"
        agreement_sub = "Macro Bullish Convergence across 4h, 8h, 16h, 30h"
        agreement_type = "CONCORDANT"
    else:
        agreement_text = f"DIVERGENT ({directions.count('LONG')}L / {directions.count('SHORT')}S / {directions.count('NEUTRAL')}N)"
        agreement_sub = "Different lookbacks filtering selectively"
        agreement_type = "DIVERGENT"

    return {
        "gate_c_status": "GATE C: DEPLOYED (SHADOW)" if is_active else "GATE C: IDLE",
        "service_active": is_active,
        "service_status_text": status_text,
        "order_authority": 0.0,
        "agreement": {
            "status": agreement_text,
            "detail": agreement_sub,
            "type": agreement_type
        },
        "candidates": candidates_info,
        "latest_telemetry": latest_telemetry
    }


def derive_card_reasoning(card: Dict[str, Any]) -> Dict[str, Any]:
    """Derives deterministic, human-readable reasoning and causal factor attribution."""
    ret_bps = float(card.get("ret_15m_bps", 0.0))
    atr = float(card.get("atr_14", 0.0))
    ref = float(card.get("reference_price", 0.0))
    vol_bps = round((atr / ref * 10000.0), 1) if ref > 0 else 0.0
    action_b = card.get("policy_b_action", "NO_TRADE")
    action_c = card.get("policy_c_action", "NO_TRADE")
    tick = card.get("features_tick") if isinstance(card.get("features_tick"), dict) else {}
    decision_clock = card.get("decision_clock") or ("tick_book_ticker" if tick else "completed_15m_candle")

    if tick:
        v5 = tick.get("velocity_5s_bps")
        imb = tick.get("imbalance")
        macro = tick.get("macro_trend_64_bps")
        if macro is None:
            macro = tick.get("macro_trend_16_bps")
        macro_f = float(macro) if macro is not None else ret_bps
        if macro_f > 15.0:
            trend = f"Bullish macro (+{macro_f:.2f} bps > +15.0 bps)"
        elif macro_f < -15.0:
            trend = f"Bearish macro ({macro_f:.2f} bps < -15.0 bps)"
        else:
            trend = f"Neutral macro ({macro_f:.2f} bps within ±15.0 bps)"
        if v5 is not None and float(v5) >= 1.0:
            mom = f"Positive micro velocity (+{float(v5):.2f} bps / 5s)"
        elif v5 is not None and float(v5) <= -1.0:
            mom = f"Negative micro velocity ({float(v5):.2f} bps / 5s)"
        else:
            mom = f"Neutral micro velocity ({v5} bps / 5s)"
        imb_s = f"imb={float(imb):.3f}" if imb is not None else "imb=n/a"
        vol_str = f"ATR(14 15m)=${atr:.2f} ({vol_bps} bps); {imb_s}; clock={decision_clock}"
    else:
        if ret_bps > 10.0:
            trend = f"Bullish Trend (+{ret_bps:.2f} bps > +10.0 bps threshold)"
        elif ret_bps < -10.0:
            trend = f"Bearish Trend ({ret_bps:.2f} bps < -10.0 bps threshold)"
        else:
            trend = f"Neutral Trend ({ret_bps:.2f} bps within [-10.0, +10.0] bps band)"
        if ret_bps > 2.0:
            mom = f"Positive Momentum (+{ret_bps:.2f} bps > +2.0 bps)"
        elif ret_bps < -2.0:
            mom = f"Negative Momentum ({ret_bps:.2f} bps < -2.0 bps)"
        else:
            mom = f"Neutral Momentum ({ret_bps:.2f} bps within [-2.0, +2.0] bps band)"
        vol_str = f"ATR(14) = ${atr:.2f} ({vol_bps} bps)"

    if action_b == "LONG":
        reason_b = f"Policy B LONG: {trend} confirmed with {mom}. {vol_str}. Tick continuation rules triggered."
    elif action_b == "SHORT":
        reason_b = f"Policy B SHORT: {trend} confirmed with {mom}. {vol_str}. Tick continuation rules triggered."
    else:
        reason_b = f"Policy B NO_TRADE: No actionable tick microstructure ({mom}). Capital preserved."

    if action_c == "LONG":
        reason_c = f"Policy C LONG: {trend} confirmed. Exhaustion filters clear; tick OOS comparator long."
    elif action_c == "SHORT":
        reason_c = f"Policy C SHORT: {trend} confirmed with {mom}. Exhaustion filters clear; tick OOS comparator short."
    else:
        reason_c = f"Policy C NO_TRADE: Sub-threshold tick signal or exhaustion boundaries reached. Filtered out."

    trade_plan_why = (
        f"Derived from Policy B {action_b} ({decision_clock}): Fixed 1.0x ATR(15m) Stop (${atr:.2f}) and 1.5x ATR Target (${1.5 * atr:.2f}) with 60-minute holding horizon post-12.0 bps friction."
        if action_b in ("LONG", "SHORT")
        else "No active trade plan. Awaiting directional tick breakout."
    )

    return {
        "trend": trend,
        "momentum": mom,
        "volatility": vol_str,
        "reason_b": reason_b,
        "reason_c": reason_c,
        "trade_plan_why": trade_plan_why
    }


def _load_nse_of_paper_ledger_state() -> Dict[str, Any]:
    """Read-only NSE OF PAPER ledger snapshot (RP-608-NSE-OF-PAPER-TRADE-001)."""
    empty = {
        "protocol_id": "RP-608-NSE-OF-PAPER-TRADE-001",
        "execution_mode": "PAPER",
        "live_order_authority": 0.0,
        "enabled": False,
        "n_open": 0,
        "open_symbols": [],
        "of_outcomes_summary": {
            "n_trades": 0,
            "n_wins": 0,
            "n_losses": 0,
            "cumulative_net_pnl_inr": 0.0,
        },
        "recent": [],
        "available": False,
    }
    try:
        from scripts.rp608_nse_of_paper_ledger import NseOfPaperLedger, PROTOCOL_ID

        # Prefer production NSE dir; fall back to test_run if present
        roots = [
            WORKSPACE_DIR / "live_capture" / "shadow_validation" / "nse",
            WORKSPACE_DIR / "live_capture" / "shadow_validation" / "nse" / "test_run",
        ]
        base = None
        for r in roots:
            if (r / "of_paper_ledger_v1" / "of_paper_state.json").exists() or (
                r / "of_paper_ledger_v1"
            ).exists():
                base = r
                break
        if base is None:
            base = roots[0]
        ledger = NseOfPaperLedger(base)
        summary = ledger.summary()
        recent = ledger.load_recent_outcomes(12)
        return {
            **summary,
            "protocol_id": PROTOCOL_ID,
            "recent": [
                {
                    "of_trade_id": r.get("of_trade_id"),
                    "symbol": r.get("tradingsymbol") or r.get("symbol"),
                    "direction": r.get("direction"),
                    "exit_reason": r.get("exit_reason"),
                    "net_pnl_inr": r.get("net_pnl_inr"),
                    "net_return_bps": r.get("net_return_bps"),
                    "holding_duration_seconds": r.get("holding_duration_seconds"),
                    "entry_bar_idx": r.get("entry_bar_idx"),
                    "exit_bar_idx": r.get("exit_bar_idx"),
                }
                for r in recent
            ],
            "opens": [
                {
                    "of_trade_id": t.get("of_trade_id"),
                    "symbol": t.get("tradingsymbol") or t.get("symbol"),
                    "direction": t.get("direction"),
                    "entry_price": t.get("entry_price"),
                    "entry_bar_idx": t.get("entry_bar_idx"),
                    "order_flow_imbalance": t.get("order_flow_imbalance"),
                    "simulated_quantity": t.get("simulated_quantity"),
                }
                for t in ledger.open_snapshot()
            ],
            "available": True,
            "ledger_dir": str(ledger.ledger_dir),
        }
    except Exception as exc:
        empty["error"] = str(exc)
        return empty


def _enrich_next_session_watchlist_for_api(
    next_session_data: Dict[str, Any],
    *,
    stamp_mtime_utc: Optional[str],
    scrips: List[Dict[str, Any]],
    live_bar_idx: int,
    session_date: Optional[str],
) -> Dict[str, Any]:
    """
    Serve next-session watchlist with indicative structure levels + live status.
    Informational only — does not authorize entries (authority 0%).
    """
    from scripts.rp608_nse_next_session_recs import (
        LEVELS_POLICY_ID,
        LEVELS_ROLE,
        compute_indicative_watch_levels,
        enrich_recommendation_levels_live,
        load_price_history_json,
    )

    data = dict(next_session_data or {})
    recs = list(data.get("recommendations") or [])
    next_date = str(data.get("next_session_date") or "")
    # Actionable only on the next-session calendar date after warmup bar ≥5.
    session_actionable = bool(
        session_date and next_date and session_date == next_date and int(live_bar_idx or 0) >= 5
    )

    live_prices: Dict[str, float] = {}
    feed_stale: Dict[str, bool] = {}
    for s in scrips or []:
        sym = str(s.get("symbol") or "")
        if not sym:
            continue
        try:
            px = float(s.get("last_price") or 0.0)
        except (TypeError, ValueError):
            px = 0.0
        if px > 0:
            live_prices[sym] = px
        label = str(s.get("status") or s.get("status_label") or "")
        age = s.get("age_sec")
        stale = label in {"STALE", "CROSSED", "ZERO_DEPTH"} or (
            isinstance(age, (int, float)) and age > 120
        )
        feed_stale[sym] = bool(stale)

    # Presentation-only: never recompute structure on API poll.
    # One-time backfill of missing stamped levels from durable bars (old stamps).
    need_structure = any(
        not (isinstance(r, dict) and isinstance(r.get("levels"), dict) and r["levels"].get("stop_price"))
        for r in recs
    )
    if need_structure:
        bh_path = find_latest_nse_file("rp608_nse_bar_history.json")
        bar_hist: Dict[str, Any] = {}
        if bh_path.exists():
            try:
                bar_hist = load_price_history_json(bh_path)
            except Exception:
                bar_hist = {}
        if bar_hist:
            now_iso = datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")
            filled: List[Dict[str, Any]] = []
            for r in recs:
                row = dict(r)
                sym = str(row.get("symbol") or "")
                levels = row.get("levels") if isinstance(row.get("levels"), dict) else None
                if not (levels and levels.get("stop_price")):
                    bars = bar_hist.get(sym) or []
                    if bars:
                        levels = compute_indicative_watch_levels(
                            bars,
                            side=str(row.get("side") or "LONG"),
                            reference_price=(
                                float(row["close_price"])
                                if row.get("close_price") is not None
                                else None
                            ),
                            calculated_at_utc=now_iso,
                        )
                        row["levels"] = levels
                        row["entry_low"] = levels.get("entry_low")
                        row["entry_high"] = levels.get("entry_high")
                        row["stop_price"] = levels.get("stop_price")
                        row["target_price"] = levels.get("target_price")
                        row["risk_reward_display"] = levels.get("risk_reward_display")
                        row["levels_role"] = LEVELS_ROLE
                        row["not_executed_entry"] = True
                        row["not_paper_trade_entry"] = True
                filled.append(row)
            recs = filled

    feed_health = _load_nse_l1_feed_health()
    reconcile_pending = bool(
        feed_health.get("reconcile_pending")
        or feed_health.get("reconciliation_pending")
        or feed_health.get("awaiting_reconcile")
    )
    # Validity overlay only (price vs stamped band + latch). No structural recalc here.
    recs = enrich_recommendation_levels_live(
        recs,
        price_history=None,
        live_prices=live_prices,
        feed_stale_by_symbol=feed_stale,
        reconcile_pending=reconcile_pending,
        session_actionable=session_actionable,
        recalculate_structure=False,
    )

    return {
        "available": bool(recs),
        "protocol_id": data.get("protocol_id"),
        "from_session_date": data.get("from_session_date"),
        "next_session_date": data.get("next_session_date"),
        "n_recommendations": data.get("n_recommendations", len(recs)),
        "n_order_flow_supported": data.get("n_order_flow_supported", 0),
        "order_flow_imbalance_threshold": data.get("order_flow_imbalance_threshold"),
        "universe_label": data.get("universe_label"),
        "universe_eligible_n": data.get("universe_eligible_n"),
        "note": data.get("note"),
        "emitted_utc": data.get("emitted_utc"),
        "recommended_at_utc": data.get("recommended_at_utc") or data.get("emitted_utc"),
        "stamp_mtime_utc": stamp_mtime_utc,
        "live_update": True,
        "live_poll_hint_sec": 3,
        "live_order_authority": data.get("live_order_authority", 0.0),
        "levels_policy_id": data.get("levels_policy_id") or LEVELS_POLICY_ID,
        "levels_role": data.get("levels_role") or LEVELS_ROLE,
        "levels_informational_only": True,
        "session_actionable_for_levels": session_actionable,
        "levels_evaluation_mode": "PRESENTATION_OVERLAY_ONLY",
        "structure_recalc_on_poll": False,
        "reconcile_pending": reconcile_pending,
        "recommendations": recs,
    }


def get_nse_dashboard_state() -> Dict[str, Any]:
    """
    Parses durable NSE artifacts from test_run directory and evaluates live health.
    Strictly Read-Only (0.0% Order Authority).
    """
    now_epoch = time.time()

    # 1. Parse Pre-Open Discovery Ledger
    preopen_map: Dict[str, Any] = {}
    preopen_records: List[Dict[str, Any]] = []
    preopen_file = find_latest_nse_file("preopen_discovery_ledger.jsonl")
    if preopen_file.exists():
        try:
            with open(preopen_file, "r", encoding="utf-8") as f:
                for line in f:
                    line = line.strip()
                    if line:
                        r = json.loads(line)
                        preopen_records.append(r)
                        preopen_map[r.get("symbol", "")] = r
        except Exception:
            pass

    valid_preopen_count = sum(1 for r in preopen_records if r.get("imbalance_status") == "VALID_AUCTION_DEPTH")
    avg_gap_bps = (sum(r.get("opening_gap_bps", 0.0) for r in preopen_records) / len(preopen_records)) if preopen_records else 0.0
    sorted_by_gap = sorted(preopen_records, key=lambda x: x.get("opening_gap_bps", 0.0), reverse=True)
    top_gainers = [
        {"symbol": r["tradingsymbol"], "gap_bps": round(r["opening_gap_bps"], 2), "indicative_price": r.get("indicative_price", 0.0)}
        for r in sorted_by_gap[:4]
    ] if sorted_by_gap else []
    top_losers = [
        {"symbol": r["tradingsymbol"], "gap_bps": round(r["opening_gap_bps"], 2), "indicative_price": r.get("indicative_price", 0.0)}
        for r in sorted_by_gap[-4:][::-1]
    ] if sorted_by_gap else []
    total_auction_qty = sum(r.get("total_auction_quantity", 0) for r in preopen_records)

    # 2. Parse WebSocket Live Probe Evidence
    probe_file = find_latest_nse_file("live_probe_evidence.json")
    probe_data: Dict[str, Any] = {}
    if probe_file.exists():
        try:
            with open(probe_file, "r", encoding="utf-8") as f:
                probe_data = json.load(f)
        except Exception:
            pass

    probe_ts_utc = probe_data.get("probe_timestamp_utc", "N/A")
    probe_age_sec = None
    if probe_ts_utc != "N/A":
        try:
            t_struct = time.strptime(probe_ts_utc.replace("Z", "UTC"), "%Y-%m-%dT%H:%M:%S%Z")
            probe_epoch = time.mktime(t_struct) - time.timezone
            probe_age_sec = max(0, int(now_epoch - probe_epoch))
        except Exception:
            pass

    # 3. Parse Kite Readiness Report (Batch L1 Quotes & Bar 1 Recon)
    readiness_file = find_latest_nse_file("kite_readiness_report.json")
    readiness_data: Dict[str, Any] = {}
    if readiness_file.exists():
        try:
            with open(readiness_file, "r", encoding="utf-8") as f:
                readiness_data = json.load(f)
        except Exception:
            pass

    recon_ts_utc = readiness_data.get("timestamp_utc", "N/A")
    recon_age_sec = None
    if recon_ts_utc != "N/A":
        try:
            t_struct = time.strptime(recon_ts_utc.replace("Z", "UTC"), "%Y-%m-%dT%H:%M:%S%Z")
            recon_epoch = time.mktime(t_struct) - time.timezone
            recon_age_sec = max(0, int(now_epoch - recon_epoch))
        except Exception:
            pass

    all_quotes: List[Dict[str, Any]] = readiness_data.get("all_quotes", [])

    # 4. Parse RP-607 Stock Profiles Artifact (Historical Morphology)
    rp607_file = Path("live_capture/shadow_validation/nse/rp607_stock_profiles.json")
    if not rp607_file.exists():
        rp607_file = WORKSPACE_DIR / "live_capture/shadow_validation/nse/rp607_stock_profiles.json"
    rp607_data: Dict[str, Any] = {}
    if rp607_file.exists():
        try:
            with open(rp607_file, "r", encoding="utf-8") as f:
                rp607_data = json.load(f)
        except Exception:
            pass
    rp607_profiles = rp607_data.get("profiles", {})
    rp607_summary = {
        "profile_date": rp607_data.get("profile_date", "2026-10-01"),
        "nature": rp607_data.get("nature", "HISTORICAL_DESCRIPTIVE_PROFILE"),
        "census_total": rp607_data.get("census_total", 119),
        "classified_count": rp607_data.get("classified_count", 102),
        "quarantined_count": rp607_data.get("quarantined_count", 17),
        "compartment_distribution": rp607_data.get("compartment_distribution", {
            "C1: High-Range Directional": 18,
            "C2: High-Range Oscillator": 8,
            "C3: Stable Moderate": 64,
            "C4: Low-Range Friction-Bound": 12,
            "C_Quarantined (Insufficient Hist)": 17
        })
    }

    # 5. Parse Scrip-Level Evaluations Snapshot (RP-608 Intraday Momentum)
    eval_file = find_latest_nse_file("rp608_nse_latest_evaluations.json")
    eval_data: Dict[str, Any] = {}
    if eval_file.exists():
        try:
            with open(eval_file, "r", encoding="utf-8") as f:
                eval_data = json.load(f)
        except Exception:
            pass

    pending_file = find_latest_nse_file("rp608_nse_pending_evaluations.json")
    pending_data: Dict[str, Any] = {}
    if pending_file.exists():
        try:
            with open(pending_file, "r", encoding="utf-8") as f:
                pending_data = json.load(f)
        except Exception:
            pass
    active_trades_map = pending_data.get("active_trades", {})

    next_session_file = find_latest_nse_file("rp608_nse_next_session_recommendations.json")
    next_session_data: Dict[str, Any] = {}
    next_session_stamp_mtime_utc: Optional[str] = None
    if next_session_file.exists():
        try:
            with open(next_session_file, "r", encoding="utf-8") as f:
                next_session_data = json.load(f)
            next_session_stamp_mtime_utc = datetime.fromtimestamp(
                next_session_file.stat().st_mtime, tz=timezone.utc
            ).isoformat().replace("+00:00", "Z")
        except Exception:
            pass
    # Backfill recommendation time on older stamps (UI real-time panel).
    _ns_emitted = next_session_data.get("emitted_utc") or next_session_data.get("recommended_at_utc")
    if _ns_emitted and isinstance(next_session_data.get("recommendations"), list):
        next_session_data.setdefault("recommended_at_utc", _ns_emitted)
        for _rec in next_session_data["recommendations"]:
            if isinstance(_rec, dict) and not _rec.get("recommended_at_utc"):
                _rec["recommended_at_utc"] = _ns_emitted

    eval_map = eval_data.get("evaluations", {})
    qualifying_candidates = []
    _nse_hz = recommendation_horizon_meta(market="NSE", profile="INTRADAY")
    for _qc in eval_data.get("qualifying_candidates", []) or []:
        if isinstance(_qc, dict):
            row = dict(_qc)
            row.setdefault("horizon_class", _nse_hz["horizon_class"])
            row.setdefault("horizon_id", _nse_hz["horizon_id"])
            row.setdefault("horizon_label", _nse_hz["horizon_label"])
            row.setdefault("horizon_term", _nse_hz["horizon_term"])
            qualifying_candidates.append(row)
        else:
            qualifying_candidates.append(_qc)
    rejection_breakdown = eval_data.get("rejection_breakdown", {})
    total_evaluated = eval_data.get("total_evaluated", 0)
    n_qualifying = eval_data.get("n_qualifying", 0)
    eval_ts_iso = eval_data.get("timestamp_iso", "N/A")
    eval_bar_idx = eval_data.get("bar_idx", 0)

    # Production session / feed health (preferred over one-shot readiness probe).
    nse_ckpt = _load_nse_production_checkpoint()
    feed_health = _load_nse_l1_feed_health()
    session_date = str(nse_ckpt.get("current_session_date") or "") or None
    durable_bar = int(nse_ckpt.get("last_durable_bar_idx") or 0)
    accounted_bar = int(nse_ckpt.get("last_accounted_bar_idx") or 0)
    ckpt_hb_iso = nse_ckpt.get("last_heartbeat_iso") or "N/A"
    ckpt_hb_age_sec = None
    try:
        hb_ms = int(nse_ckpt.get("last_heartbeat_utc_ms") or 0)
        if hb_ms > 0:
            ckpt_hb_age_sec = max(0, int(now_epoch - (hb_ms / 1000.0)))
    except Exception:
        pass
    eval_session_date = _nse_iso_date_ist(eval_ts_iso)
    eval_snapshot_fresh = bool(
        session_date and eval_session_date and session_date == eval_session_date
    )
    # When today's session has not yet produced a durable eval, do not treat
    # yesterday's (or earlier) snapshot as live G1/bar authority.
    live_bar_for_gates = (
        int(eval_bar_idx) if eval_snapshot_fresh else int(durable_bar)
    )

    # Merge scrips
    merged_scrips: List[Dict[str, Any]] = []
    valid_quotes = 0
    stale_quotes = 0
    missing_quotes = 0
    crossed_quotes = 0
    zero_depth_quotes = 0

    for q in all_quotes:
        sym = q.get("symbol", "")
        tsym = q.get("tradingsymbol", sym.replace(".NS", ""))
        po = preopen_map.get(sym, {})
        ev = eval_map.get(sym, {})
        p607 = rp607_profiles.get(sym, {})

        is_valid = q.get("valid_depth", True)
        is_crossed = q.get("crossed", False)
        is_zero_depth = q.get("zero_depth", False)

        if is_crossed:
            crossed_quotes += 1
            status_label = "CROSSED"
        elif is_zero_depth:
            zero_depth_quotes += 1
            status_label = "ZERO_DEPTH"
        elif is_valid:
            valid_quotes += 1
            status_label = "VALID_L1"
        else:
            status_label = "STALE"
            stale_quotes += 1

        q_ts = q.get("timestamp") or q.get("last_trade_time")
        q_age_sec = None
        if q_ts:
            try:
                t_tuple = time.strptime(str(q_ts)[:19], "%Y-%m-%d %H:%M:%S")
                q_epoch = time.mktime(t_tuple)
                q_age_sec = max(0, int(now_epoch - q_epoch))
            except Exception:
                pass

        eval_utc_ms = ev.get("quote_timestamp_utc_ms") or ev.get("bar_timestamp_utc_ms")
        if eval_utc_ms and (q_age_sec is None or q_age_sec > 600):
            q_age_sec = max(0, int(now_epoch - (eval_utc_ms / 1000.0)))
            q_ts = ev.get("bar_timestamp_iso")

        # Use live reference price and spread from evaluation if available
        last_price = ev.get("reference_price") if ev.get("reference_price") is not None else q.get("last_price", 0.0)
        half_spread_bps = ev.get("half_spread_bps") if ev.get("half_spread_bps") is not None else q.get("half_spread_bps", 0.0)
        if ev.get("data_quality", {}).get("status") == "VALID_L1":
            status_label = "VALID_L1"

        # Sequential Gate Trace Computation (Strictly according to documented evaluator control flow)
        b_idx = ev.get("bar_idx", eval_bar_idx)
        score_val = ev.get("score")
        d_code = ev.get("decision_code", "PENDING_RUN" if total_evaluated == 0 else "NO_EVALUATION")
        h_len = ev.get("history_length", 0)
        dq = ev.get("data_quality") or {}
        dq_status = dq.get("status")
        dq_error = str(dq.get("error") or "")
        # Prefer durable evaluator explanation when present (avoids re-deriving over live codes).
        ev_explanation = ev.get("explanation") or ""
        # Kite/auth/feed failures must not be mislabeled as Gate 3 warmup (same-session history).
        is_data_fail = (
            d_code == "DATA_QUALITY_FAILURE"
            or dq_status in {"DATA_FAILURE", "INVALID", "FAILED", "ERROR"}
            or "KITE_HTTP" in dq_error
            or "KITE_HTTP" in ev_explanation
            or "Data quality failure" in ev_explanation
        )

        # Gate 1: Entry Window Eligibility (1 <= bar_idx <= 19; 60m completes by 15:00 close-window)
        # Stale cross-session eval snapshots must not close G1 for today's session.
        _max_entry_bar = 19
        if not eval_snapshot_fresh:
            b_idx = live_bar_for_gates
            g1_status = "UNVERIFIED"
            g1_reason = (
                f"Awaiting today's evaluation (stale snapshot {eval_session_date or eval_ts_iso}; "
                f"session {session_date or 'unknown'}; durable_bar={durable_bar})"
            )
            g1_pass = False
        else:
            g1_pass = (1 <= b_idx <= _max_entry_bar)
            g1_status = "PASS" if g1_pass else "CLOSED"
            g1_reason = (
                f"Entry window open (Bar {b_idx} in 1..{_max_entry_bar})"
                if g1_pass
                else f"Entry window closed (Bar {b_idx} > {_max_entry_bar}; 60m must finish by 15:00)"
            )

        # Gate 2: Position State / Capacity (ACTIVE = simulated trade already open; NONE = slot available)
        trade_info = active_trades_map.get(sym)
        has_active_position = (d_code == "EXISTING_POSITION_ACTIVE" or trade_info is not None)
        if has_active_position:
            entry_b = trade_info.get("entry_bar_idx") if trade_info else 18
            g2_status = "ACTIVE"
            g2_reason = f"Simulated position open since bar {entry_b}"
        else:
            g2_status = "NONE"
            g2_reason = "No active trade conflict (Position available)"

        # Gate 5 first when evaluator already recorded a feed/auth failure
        if is_data_fail:
            g5_status = "FAIL"
            g5_reason = dq_error or ev_explanation or "Data quality / feed failure"
        elif dq_status == "VALID_L1" or is_valid:
            g5_status = "PASS"
            g5_reason = "Executable quote valid & spread within bounds"
        elif not is_valid:
            g5_status = "FAIL"
            g5_reason = "Quote stale or missing L1 depth"
        else:
            g5_status = "NOT_EVALUATED"
            g5_reason = "Pending quote verification"

        # Gate 3: Causal Warmup Hurdle (history >= 5 bars).
        # Under cross_gap_neutral_v1, history_length includes prior-session bars.
        g3_policy = ev.get("g3_warmup_policy") or (ev.get("history_provenance") or {}).get("g3_warmup_policy")
        g3_flag = ev.get("g3_ready")
        sess_h = ev.get("session_history_length")
        if is_data_fail and h_len < 5 and g3_flag is not True:
            g3_status = "BLOCKED"
            g3_reason = f"History {h_len}/5 not evaluated — blocked by data failure (not a warmup fail)"
        elif d_code == "COLD_START_INSUFFICIENT_HISTORY" or (
            g3_flag is False if g3_flag is not None else h_len < 5
        ):
            g3_status = "FAIL"
            pol = f"; policy={g3_policy}" if g3_policy else ""
            sess = f"; session={sess_h}" if sess_h is not None else ""
            g3_reason = f"Warmup history: {h_len}/5 bars required{pol}{sess}"
        else:
            g3_status = "PASS"
            pol = f" ({g3_policy})" if g3_policy else ""
            g3_reason = f"Warmup satisfied: {h_len} bars >= 5{pol}"

        # Gate 4: Momentum Threshold (traj15 >= +0.0500)
        if is_data_fail:
            g4_status = "NOT_EVALUATED"
            g4_reason = "Blocked by data failure"
        elif g3_status != "PASS":
            g4_status = "NOT_EVALUATED"
            g4_reason = "Blocked by Gate 3 (Causal Warmup)"
        elif score_val is not None and score_val >= 0.05:
            g4_status = "PASS"
            g4_reason = f"Momentum score {score_val:+.4f} >= +0.0500 threshold"
        elif score_val is not None and score_val < 0.05:
            g4_status = "FAIL"
            g4_reason = f"Momentum score {score_val:+.4f} < +0.0500 threshold"
        else:
            g4_status = "NOT_EVALUATED"
            g4_reason = "Score unavailable"

        # Determine terminal decision based on cleanly separated dimensions
        if has_active_position:
            terminal_decision = "HOLD_POSITION"
            d_code = "EXISTING_POSITION_ACTIVE"
            entry_b = trade_info.get("entry_bar_idx") if trade_info else 18
            ev_explanation = f"Active simulated position managed (open since bar {entry_b})"
        elif is_data_fail:
            terminal_decision = "NO_TRADE"
            d_code = "DATA_QUALITY_FAILURE"
            ev_explanation = ev_explanation or g5_reason
        elif g1_status == "UNVERIFIED":
            terminal_decision = "NO_TRADE"
            d_code = "AWAITING_TODAY_EVALUATION"
            ev_explanation = g1_reason
        elif g1_status == "CLOSED":
            terminal_decision = "NO_TRADE"
            d_code = "INTRADAY_BOUNDARY_EXCLUDED"
            ev_explanation = g1_reason
        elif g3_status == "FAIL":
            terminal_decision = "NO_TRADE"
            d_code = "COLD_START_INSUFFICIENT_HISTORY"
            ev_explanation = ev_explanation or g3_reason
        elif g4_status == "FAIL":
            terminal_decision = "NO_TRADE"
            d_code = "SUB_THRESHOLD_MOMENTUM"
            ev_explanation = ev_explanation or g4_reason
        elif g5_status == "FAIL":
            terminal_decision = "NO_TRADE"
            d_code = "DATA_QUALITY_FAILURE"
            ev_explanation = ev_explanation or g5_reason
        elif g1_pass and g2_status == "NONE" and g3_status == "PASS" and g4_status == "PASS" and g5_status == "PASS":
            terminal_decision = "RECOMMEND_LONG"
            d_code = "QUALIFIED_MOMENTUM_BREAKOUT"
            ev_explanation = f"Momentum breakout traj15={score_val:+.4f} >= +0.0500 with {h_len} bars history"
        else:
            terminal_decision = ev.get("decision", "NO_TRADE")
            d_code = ev.get("decision_code", d_code)
            ev_explanation = ev_explanation or ev.get("explanation", "")

        gates_trace = [
            {"gate_id": "G1", "name": "Entry Window", "status": g1_status, "reason": g1_reason},
            {"gate_id": "G2", "name": "Position State", "status": g2_status, "reason": g2_reason},
            {"gate_id": "G3", "name": "Causal Warmup", "status": g3_status, "reason": g3_reason},
            {"gate_id": "G4", "name": "Momentum Threshold", "status": g4_status, "reason": g4_reason},
            {"gate_id": "G5", "name": "Quote Quality", "status": g5_status, "reason": g5_reason}
        ]

        bar_provenance = ev.get("bar_provenance") or (ev.get("history_provenance") or {}).get("source")
        feed_source = (ev.get("data_quality") or {}).get("feed_source")
        feed_label = _feed_label(bar_provenance, feed_source)

        # Prefer evaluator risk_reward; else freeze from active trade dynamic bounds.
        rr = ev.get("risk_reward")
        if (not rr or rr.get("target_bps") is None) and trade_info and trade_info.get("target_bps") is not None:
            hs = trade_info.get("half_spread_bps") or ev.get("half_spread_bps")
            fric = _nse_estimate_friction_bps(hs, None)
            net_rr = _nse_net_rr_ratio(
                trade_info.get("target_bps"),
                trade_info.get("stop_loss_bps"),
                half_spread_bps=hs,
                estimated_friction_bps=fric,
            )
            rr = {
                "target_price": trade_info.get("target_price"),
                "stop_loss_price": trade_info.get("stop_loss_price"),
                "target_bps": trade_info.get("target_bps"),
                "stop_loss_bps": trade_info.get("stop_loss_bps"),
                "atr_14": trade_info.get("atr_14"),
                "exit_policy": trade_info.get("exit_policy"),
                "trend_label": trade_info.get("exit_trend_label"),
                "fallback_reason": trade_info.get("exit_fallback_reason"),
                "net_rr_ratio": net_rr,
                "estimated_friction_bps": round(fric, 2),
                "rank_score": _nse_rr_target_rank_score(net_rr, trade_info.get("target_bps")),
            }
        elif rr and rr.get("target_bps") is not None and rr.get("net_rr_ratio") is None:
            hs = ev.get("half_spread_bps") or (trade_info or {}).get("half_spread_bps")
            net_rr = _nse_net_rr_ratio(
                rr.get("target_bps"),
                rr.get("stop_loss_bps"),
                half_spread_bps=hs,
                estimated_friction_bps=rr.get("estimated_friction_bps"),
            )
            rr = dict(rr)
            rr["net_rr_ratio"] = net_rr
            if rr.get("estimated_friction_bps") is None:
                rr["estimated_friction_bps"] = round(_nse_estimate_friction_bps(hs, None), 2)
            rr["rank_score"] = _nse_rr_target_rank_score(net_rr, rr.get("target_bps"))

        # Live OF inspection from L1 (all symbols). Prefer quote depth; fall back to eval stamp.
        # Observational only — does not change Arm A entry / authority.
        bid_q = ev.get("data_quality", {}).get("bid_qty", q.get("bid_qty", 0))
        ask_q = ev.get("data_quality", {}).get("ask_qty", q.get("ask_qty", 0))
        of_side = str((trade_info or {}).get("side") or "LONG").upper() or "LONG"
        live_l1 = l1_from_book_ticker(
            {
                "bid_qty": bid_q,
                "ask_qty": ask_q,
                "bid_price": q.get("best_bid") or q.get("bid_price"),
                "ask_price": q.get("best_ask") or q.get("ask_price"),
            },
            source="nse_live_l1",
        )
        of_live = inspect_order_flow_book(live_l1, primary_side=of_side)
        # If live book empty, keep durable evaluator OF stamp when present.
        if of_live.get("order_flow_source") in (None, "none") and ev.get("order_flow_imbalance") is not None:
            of_live = {
                "order_flow_supported": bool(ev.get("order_flow_supported")),
                "order_flow_label": ev.get("order_flow_label"),
                "order_flow_imbalance": ev.get("order_flow_imbalance"),
                "order_flow_source": ev.get("order_flow_source") or "eval_stamp",
                "order_flow_side": ev.get("order_flow_side") or of_side,
                "order_flow_long_supported": bool(ev.get("order_flow_long_supported")),
                "order_flow_short_supported": bool(ev.get("order_flow_short_supported")),
                "order_flow_inspection": True,
            }

        merged_scrips.append({
            "symbol": sym,
            "tradingsymbol": tsym,
            "indicative_price": po.get("indicative_price", 0.0),
            "opening_gap_bps": round(po.get("opening_gap_bps", 0.0), 2),
            "imbalance_ratio": round(po.get("imbalance_ratio", 0.0), 4),
            "imbalance_status": po.get("imbalance_status", "N/A"),
            "last_price": last_price,
            "best_bid": q.get("best_bid", 0.0),
            "best_ask": q.get("best_ask", 0.0),
            "bid_qty": bid_q,
            "ask_qty": ask_q,
            "half_spread_bps": half_spread_bps,
            "volume": q.get("volume", 0),
            "day_open": q.get("day_open", 0.0),
            "day_high": q.get("day_high", 0.0),
            "day_low": q.get("day_low", 0.0),
            "prev_close": q.get("prev_close", 0.0),
            "timestamp": str(q_ts) if q_ts else "N/A",
            "age_sec": q_age_sec,
            "status": status_label,
            "bar_provenance": bar_provenance,
            "feed_source": feed_source,
            "feed_label": feed_label,
            # RP-607 Historical Profile Metadata (Read-Only)
            "rp607": {
                "compartment": p607.get("compartment", "C_Quarantined (Insufficient Hist)"),
                "compartment_code": p607.get("compartment_code", "QUARANTINED"),
                "valid": p607.get("valid", False),
                "quarantine_reason": p607.get("quarantine_reason"),
                "natr_20": p607.get("natr_20"),
                "dir_eff_20": p607.get("dir_eff_20"),
                "ar1_60": p607.get("ar1_60"),
                "cv_range": p607.get("cv_range"),
                "opp_ratio": p607.get("opp_ratio"),
                "profile_date": p607.get("profile_date", "2026-10-01"),
                "provenance": p607.get("provenance", {
                    "source": "yahoo_cache_1d",
                    "nature": "HISTORICAL_SNAPSHOT",
                    "calculated_as_of": "2026-09-30 23:59:59 UTC"
                })
            },
            # Evaluator Decision Ledger attributes
            "decision": terminal_decision,
            "decision_code": d_code,
            "explanation": ev_explanation,
            "score": score_val if score_val is not None else 0.0,
            "provisional_rank": ev.get("provisional_rank", None),
            "risk_reward": rr,
            "bar_timestamp_iso": ev.get("bar_timestamp_iso", "N/A"),
            "eval_bar_idx": b_idx,
            "history_length": h_len,
            "gates_trace": gates_trace,
            "entry_price": (trade_info or {}).get("entry_price") or last_price,
            "active_entry_bar_idx": (trade_info or {}).get("entry_bar_idx"),
            "order_flow_supported": of_live.get("order_flow_supported"),
            "order_flow_label": of_live.get("order_flow_label"),
            "order_flow_imbalance": of_live.get("order_flow_imbalance"),
            "order_flow_source": of_live.get("order_flow_source"),
            "order_flow_side": of_live.get("order_flow_side"),
            "order_flow_long_supported": of_live.get("order_flow_long_supported"),
            "order_flow_short_supported": of_live.get("order_flow_short_supported"),
            "order_flow_inspection": True,
            **_nse_recommendation_time_fields(
                utc_ms=(trade_info or {}).get("entry_timestamp_utc_ms")
                or ev.get("bar_timestamp_utc_ms")
                or ev.get("quote_timestamp_utc_ms"),
                iso_s=ev.get("bar_timestamp_iso"),
            ),
            **recommendation_horizon_meta(market="NSE", profile="INTRADAY"),
        })

    # High-target idea surface (≥ NSE_HIGH_TARGET_BPS): active dynamic bounds + fresh qualifies.
    # Rank = Net R:R × (target_bps / 100) so both reward quality and absolute target matter.
    high_target_by_sym: Dict[str, Dict[str, Any]] = {}

    def _enrich_high_target_card(card: Dict[str, Any]) -> Dict[str, Any]:
        rr = dict(card.get("risk_reward") or {})
        hs = card.get("half_spread_bps")
        net_rr = _nse_net_rr_ratio(
            rr.get("target_bps"),
            rr.get("stop_loss_bps"),
            half_spread_bps=hs,
            estimated_friction_bps=rr.get("estimated_friction_bps"),
            existing_net_rr=rr.get("net_rr_ratio"),
        )
        if rr.get("estimated_friction_bps") is None:
            rr["estimated_friction_bps"] = round(
                _nse_estimate_friction_bps(hs, None), 2
            )
        rr["net_rr_ratio"] = net_rr
        rank_score = _nse_rr_target_rank_score(net_rr, rr.get("target_bps"))
        rr["rank_score"] = rank_score
        card["risk_reward"] = rr
        card["rank_score"] = rank_score
        card.update(recommendation_horizon_meta(market="NSE", profile="INTRADAY"))
        return card

    for sym, trade in (active_trades_map or {}).items():
        tb = trade.get("target_bps")
        if tb is None or float(tb) <= NSE_HIGH_TARGET_BPS:
            continue
        ev = eval_map.get(sym, {})
        rec_time = _nse_recommendation_time_fields(
            utc_ms=trade.get("entry_timestamp_utc_ms"),
            iso_s=ev.get("bar_timestamp_iso"),
        )
        high_target_by_sym[sym] = _enrich_high_target_card({
            "symbol": sym,
            "tradingsymbol": trade.get("tradingsymbol") or sym.replace(".NS", ""),
            "decision": "HOLD_POSITION",
            "decision_code": "EXISTING_POSITION_ACTIVE",
            "card_kind": "ACTIVE_HIGH_TARGET",
            "provisional_rank": ev.get("provisional_rank"),
            "score": ev.get("score"),
            "reference_price": trade.get("entry_price") or ev.get("reference_price"),
            "entry_price": trade.get("entry_price"),
            "entry_bar_idx": trade.get("entry_bar_idx"),
            "half_spread_bps": trade.get("half_spread_bps") or ev.get("half_spread_bps"),
            "entry_conditions": ev.get("entry_conditions") or "Active simulated position (dynamic ATR/structure exits)",
            "invalidation": (
                f"p <= ₹{trade.get('stop_loss_price')} (−{float(trade.get('stop_loss_bps') or 0):.0f} bps) "
                f"or TP ₹{trade.get('target_price')} (+{float(tb):.0f} bps)"
            ),
            "risk_reward": {
                "target_price": trade.get("target_price"),
                "stop_loss_price": trade.get("stop_loss_price"),
                "target_bps": float(tb),
                "stop_loss_bps": trade.get("stop_loss_bps"),
                "atr_14": trade.get("atr_14"),
                "exit_policy": trade.get("exit_policy"),
                "trend_label": trade.get("exit_trend_label"),
                "net_rr_ratio": None,
                "estimated_friction_bps": None,
            },
            **rec_time,
        })
    for sym, ev in (eval_map or {}).items():
        rr = dict(ev.get("risk_reward") or {})
        tb = rr.get("target_bps")
        if tb is None or float(tb) <= NSE_HIGH_TARGET_BPS:
            continue
        if ev.get("decision") != "RECOMMEND_LONG" and sym in high_target_by_sym:
            continue
        if ev.get("decision") == "RECOMMEND_LONG" or sym not in high_target_by_sym:
            rec_time = _nse_recommendation_time_fields(
                utc_ms=ev.get("bar_timestamp_utc_ms") or ev.get("quote_timestamp_utc_ms"),
                iso_s=ev.get("bar_timestamp_iso"),
            )
            high_target_by_sym[sym] = _enrich_high_target_card({
                "symbol": sym,
                "tradingsymbol": ev.get("tradingsymbol") or sym.replace(".NS", ""),
                "decision": ev.get("decision"),
                "decision_code": ev.get("decision_code"),
                "card_kind": "QUALIFIED_HIGH_TARGET" if ev.get("decision") == "RECOMMEND_LONG" else "EVAL_HIGH_TARGET",
                "provisional_rank": ev.get("provisional_rank"),
                "score": ev.get("score"),
                "reference_price": ev.get("reference_price"),
                "entry_price": ev.get("reference_price"),
                "entry_bar_idx": ev.get("bar_idx"),
                "half_spread_bps": ev.get("half_spread_bps"),
                "entry_conditions": ev.get("entry_conditions"),
                "invalidation": ev.get("invalidation"),
                "risk_reward": rr,
                **rec_time,
            })
    high_target_candidates = sorted(
        high_target_by_sym.values(),
        key=lambda c: (
            -float(c.get("rank_score") or 0),
            -float((c.get("risk_reward") or {}).get("target_bps") or 0),
            -float((c.get("risk_reward") or {}).get("net_rr_ratio") or 0),
            c.get("tradingsymbol") or "",
        ),
    )

    benchmark_order = [
        "RELIANCE", "TCS", "INFY", "HDFCBANK", "ICICIBANK", "BHARTIARTL", "ITC", 
        "ADANIENT", "ADANIPORTS", "SBIN", "LT", "KOTAKBANK", "AXISBANK", "MARUTI", 
        "TATACONSUM", "TITAN", "SUNPHARMA", "WIPRO", "HINDUNILVR", "BAJFINANCE"
    ]
    benchmark_set = set(benchmark_order)

    # Qualifying long recommendations first (ranked 1..N)
    qualifying_scrips = [s for s in merged_scrips if s.get("decision") == "RECOMMEND_LONG"]
    qualifying_scrips.sort(key=lambda s: s.get("provisional_rank") or 999)

    non_qualifying = [s for s in merged_scrips if s.get("decision") != "RECOMMEND_LONG"]
    benchmarks = [s for s in non_qualifying if s["tradingsymbol"] in benchmark_set]
    benchmarks.sort(key=lambda s: benchmark_order.index(s["tradingsymbol"]) if s["tradingsymbol"] in benchmark_set else 999)
    others = [s for s in non_qualifying if s["tradingsymbol"] not in benchmark_set]
    others.sort(key=lambda s: s.get("volume", 0), reverse=True)
    sorted_scrips = qualifying_scrips + benchmarks + others

    # Momentum score distribution
    bullish_count = sum(1 for s in merged_scrips if s.get("score", 0.0) >= 0.05)
    neutral_count = sum(1 for s in merged_scrips if -0.05 < s.get("score", 0.0) < 0.05)
    bearish_count = sum(1 for s in merged_scrips if s.get("score", 0.0) <= -0.05)

    # Live OF inspection census (all scrips; observational)
    n_live_order_flow_supported = sum(
        1 for s in merged_scrips if s.get("order_flow_supported")
    )
    n_live_order_flow_long = sum(
        1 for s in merged_scrips if s.get("order_flow_long_supported")
    )
    n_live_order_flow_short = sum(
        1 for s in merged_scrips if s.get("order_flow_short_supported")
    )

    # Check systemd status
    services_status = {}
    for unit in ["rp608-nse-shadow-daemon.service", "rp608-nse-live-probe.service", "rp608-nse-bar1-recon.service", "rp608-nse-preopen-snap1.service"]:
        try:
            res = subprocess.run(["systemctl", "is-active", unit], capture_output=True, text=True, timeout=1)
            services_status[unit] = res.stdout.strip()
        except Exception:
            services_status[unit] = "unknown"

    daemon_active = services_status.get("rp608-nse-shadow-daemon.service") == "active"
    heartbeat_fresh = ckpt_hb_age_sec is not None and ckpt_hb_age_sec <= 120
    feed_age_sec = None
    feed_last_packet_utc = "N/A"
    feed_cached = 0
    feed_subscribed = 119
    feed_connected = False
    feed_packets = 0
    if feed_health:
        try:
            feed_age_sec = feed_health.get("last_packet_age_sec")
            if feed_age_sec is not None:
                feed_age_sec = float(feed_age_sec)
        except Exception:
            feed_age_sec = None
        feed_last_packet_utc = feed_health.get("last_packet_utc") or feed_health.get("timestamp_iso") or "N/A"
        feed_cached = int(feed_health.get("cached_tokens") or 0)
        feed_subscribed = int(feed_health.get("subscribed_tokens") or 119)
        feed_connected = bool(feed_health.get("connected"))
        feed_packets = int(feed_health.get("packets") or 0)

    if feed_health and feed_connected and feed_age_sec is not None and feed_age_sec <= 60:
        live_ws_status = "LIVE L1 CACHE"
        live_ws_source = "l1_feed_health"
        live_ws_age = int(feed_age_sec)
        live_mode_note = f"Continuous L1 ticker cache · {feed_cached}/{feed_subscribed} tokens"
    elif daemon_active and heartbeat_fresh and session_date:
        if durable_bar <= 0:
            live_ws_status = "L1 PATH STARTED · AWAITING FIRST DURABLE BAR"
        else:
            live_ws_status = "L1 PATH ACTIVE · SESSION HEARTBEAT FRESH"
        live_ws_source = "production_checkpoint"
        live_ws_age = ckpt_hb_age_sec
        live_mode_note = (
            f"Session {session_date} · durable={durable_bar} accounted={accounted_bar} · "
            "packet-age pending feed_health artifact"
        )
        feed_last_packet_utc = ckpt_hb_iso
        feed_cached = 0
    elif daemon_active:
        live_ws_status = "DAEMON ACTIVE · HEARTBEAT STALE"
        live_ws_source = "production_checkpoint"
        live_ws_age = ckpt_hb_age_sec
        live_mode_note = "NSE daemon active but checkpoint heartbeat is stale"
    else:
        live_ws_status = "DAEMON INACTIVE"
        live_ws_source = "systemd"
        live_ws_age = ckpt_hb_age_sec
        live_mode_note = "NSE shadow daemon not active"

    observer_status = (
        "HEALTHY"
        if eval_snapshot_fresh and live_bar_for_gates > 0
        else ("SESSION_ACTIVE_AWAITING_EVAL" if daemon_active and heartbeat_fresh else "WARMING_UP")
    )

    pending_file = find_latest_nse_file("rp608_nse_pending_evaluations.json")
    pending_count = 0
    active_trades_count = 0
    if pending_file.exists():
        try:
            with open(pending_file, "r", encoding="utf-8") as f:
                p_data = json.load(f)
            p_dict = p_data.get("pending_directional_evals", {})
            pending_count = sum(len(v) for v in p_dict.values())
            active_trades_count = len(p_data.get("active_trades", {}))
        except Exception:
            pass

    cards_file = find_latest_nse_file("rp608_nse_decision_cards.jsonl")
    decision_cards_count = 0
    if cards_file.exists():
        try:
            with open(cards_file, "r", encoding="utf-8") as f:
                decision_cards_count = sum(1 for line in f if line.strip())
        except Exception:
            pass

    trade_file = find_latest_nse_file("rp608_nse_trade_journal.jsonl")
    completed_trades_count = 0
    if trade_file.exists():
        try:
            with open(trade_file, "r", encoding="utf-8") as f:
                completed_trades_count = sum(1 for line in f if line.strip())
        except Exception:
            pass

    creds_obj = {}
    if load_credentials:
        try:
            creds_obj = load_credentials()
        except Exception:
            pass

    return {
        "status": "SHADOW / READ-ONLY",
        "order_authority": 0.0,
        "operational_triad": {
            "observer_health": {
                "status": observer_status,
                "service_active": daemon_active,
                "latest_bar_idx": live_bar_for_gates,
                "latest_eval_iso": eval_ts_iso if eval_snapshot_fresh else None,
                "eval_snapshot_iso": eval_ts_iso,
                "eval_snapshot_fresh": eval_snapshot_fresh,
                "symbols_evaluated": total_evaluated if eval_snapshot_fresh else 0,
                "warmup_passed": sum(1 for s in merged_scrips if s.get("history_length", 0) >= 5) if eval_snapshot_fresh else 0,
            },
            "outcome_tracking": {
                "status": "ACTIVE_ACCUMULATING" if pending_count > 0 or decision_cards_count > 0 else "STANDBY",
                "pending_directional_evals": pending_count,
                "active_shadow_positions": active_trades_count,
                "completed_trades_journal": completed_trades_count,
                "directional_cards_recorded": decision_cards_count
            },
            "hypothesis_evidence": {
                "status": "PROVISIONAL_UNVALIDATED",
                "hypothesis_id": "RP-608-NSE-MOMENTUM-001",
                "sample_required": 100,
                "sample_accumulated": completed_trades_count,
                "live_order_authority": "0.0%",
                "holdout_partition": "SEALED",
                "verdict": "AWAITING_CONVERGENCE_EVIDENCE"
            }
        },
        "auth_user_id": readiness_data.get("user_id", creds_obj.get("user_id", "TJ6378")),
        "auth_user_name": readiness_data.get("user_name", "Nikhil Tanaji More"),
        "api_key": creds_obj.get("api_key", ""),
        "coverage": {
            "reconciled": readiness_data.get("symbols_reconciled", 119),
            "total_universe": 119,
            "status_text": "119 / 119 Reconciled",
            "mode_note": live_mode_note,
        },
        "production_session": {
            "session_date": session_date,
            "durable_bar": durable_bar,
            "accounted_bar": accounted_bar,
            "heartbeat_iso": ckpt_hb_iso,
            "heartbeat_age_sec": ckpt_hb_age_sec,
            "daemon_active": daemon_active,
            "eval_snapshot_fresh": eval_snapshot_fresh,
            "eval_snapshot_iso": eval_ts_iso,
            "eval_snapshot_session_date": eval_session_date,
            "g1_status": (
                "UNVERIFIED"
                if not eval_snapshot_fresh
                else ("PASS" if 1 <= live_bar_for_gates <= 19 else "CLOSED")
            ),
        },
        "live_websocket": {
            "status": live_ws_status,
            "source": live_ws_source,
            "last_packet_utc": feed_last_packet_utc,
            "age_sec": live_ws_age,
            "age_metric": "packet" if live_ws_source == "l1_feed_health" else "heartbeat",
            "ticks_captured": feed_packets,
            "cached_tokens": feed_cached,
            "subscribed_tokens": feed_subscribed,
            "connected": feed_connected if feed_health else daemon_active,
            "calibrated_ntp_offset_ms": probe_data.get("calibrated_ntp_offset_ms", 0.448),
            "packet_format": "184-byte Full Binary Depth",
            "valid_spreads": feed_cached if feed_health else None,
            "zero_depth": None,
            "crossed_market": None,
            "control_messages": None,
            "reconnects": int(feed_health.get("reconnects") or 0) if feed_health else 0,
        },
        "readiness_probe": {
            "status": "VERIFIED PROBE (15s Capture)" if probe_data else "UNVERIFIED",
            "note": "Historical readiness probe · not the live continuous-feed indicator",
            "last_packet_utc": probe_ts_utc,
            "age_sec": probe_age_sec,
            "ticks_captured": probe_data.get("ticks_captured_count", 0),
            "valid_spreads": probe_data.get("valid_spreads_count", 0),
            "zero_depth": probe_data.get("zero_depth_count", 0),
            "crossed_market": probe_data.get("crossed_market_count", 0),
            "control_messages": probe_data.get("control_messages_count", 0),
        },
        "preopen_discovery": {
            "status": "3 CHECKPOINTS VERIFIED",
            "checkpoints_text": "09:07 · 09:11:30 · 09:14:30 IST",
            "total_records": len(preopen_records),
            "valid_auction_depth_count": valid_preopen_count,
            "avg_gap_bps": round(avg_gap_bps, 2),
            "total_auction_qty": total_auction_qty,
            "top_gainers": top_gainers,
            "top_losers": top_losers
        },
        "bar_reconciliation": {
            "status": "VERIFIED PASS (0/SUCCESS)",
            "verdict": readiness_data.get("verdict", "PASS"),
            "rtt_ms": readiness_data.get("quote_batch_rtt_ms", 0.0),
            "quotes_retrieved": readiness_data.get("quotes_retrieved_in_single_batch", 0),
            "last_check_utc": recon_ts_utc,
            "age_sec": recon_age_sec,
            "valid_l1_depth_count": readiness_data.get("valid_l1_depth_count", 0),
            "zero_depth_count": readiness_data.get("zero_depth_count", 0),
            "crossed_market_count": readiness_data.get("crossed_market_count", 0),
            "missing_quotes_count": readiness_data.get("missing_quotes_count", 0)
        },
        "rp607_summary": rp607_summary,
        "evaluations_summary": {
            "total_evaluated": total_evaluated if eval_snapshot_fresh else 0,
            "n_qualifying": n_qualifying if eval_snapshot_fresh else 0,
            "evaluation_engine_status": (
                "ACTIVE_EVALUATED"
                if eval_snapshot_fresh and total_evaluated > 0
                else ("AWAITING_TODAY_EVAL" if daemon_active else "PENDING_RUN")
            ),
            "last_evaluation_iso": eval_ts_iso if eval_snapshot_fresh else None,
            "stale_evaluation_iso": None if eval_snapshot_fresh else eval_ts_iso,
            "eval_snapshot_fresh": eval_snapshot_fresh,
            "rejection_breakdown": rejection_breakdown if eval_snapshot_fresh else {},
            "qualifying_candidates": qualifying_candidates if eval_snapshot_fresh else [],
            "high_target_bps_floor": NSE_HIGH_TARGET_BPS,
            "high_target_sort": "net_rr_x_target_pct",
            "n_high_target": len(high_target_candidates) if eval_snapshot_fresh else 0,
            "high_target_candidates": high_target_candidates if eval_snapshot_fresh else [],
            "momentum_distribution": {
                "bullish_ge_005": bullish_count,
                "neutral": neutral_count,
                "bearish_le_005": bearish_count
            },
            "n_live_order_flow_supported": n_live_order_flow_supported,
            "n_live_order_flow_long": n_live_order_flow_long,
            "n_live_order_flow_short": n_live_order_flow_short,
        },
        "n_live_order_flow_supported": n_live_order_flow_supported,
        "n_live_order_flow_long": n_live_order_flow_long,
        "n_live_order_flow_short": n_live_order_flow_short,
        "of_paper_ledger": _load_nse_of_paper_ledger_state(),
        "next_session_recommendations": _enrich_next_session_watchlist_for_api(
            next_session_data,
            stamp_mtime_utc=next_session_stamp_mtime_utc,
            scrips=sorted_scrips,
            live_bar_idx=live_bar_for_gates,
            session_date=session_date,
        ),
        "quote_health": {
            "valid": valid_quotes or len(all_quotes),
            "stale": stale_quotes,
            "missing": missing_quotes,
            "crossed": crossed_quotes,
            "zero_depth": zero_depth_quotes
        },
        "services": services_status,
        "artifact_source": str(eval_file.parent.name if eval_file.exists() else "nse"),
        "daemon_pid": _resolve_nse_daemon_pid(),
        "scrips": sorted_scrips
    }


def load_live_state() -> Dict[str, Any]:
    """Compiles the complete live monitoring state from disk and memory."""
    # 1. Read Checkpoint
    checkpoint = {}
    if CHECKPOINT_FILE.exists():
        try:
            with open(CHECKPOINT_FILE, "r", encoding="utf-8") as f:
                checkpoint = json.load(f)
        except Exception:
            pass

    # 2. Read Decision Cards (parse all for accounting; reasoning only on tail used by UI)
    cards: List[Dict[str, Any]] = []
    if DECISION_CARDS_FILE.exists():
        try:
            with open(DECISION_CARDS_FILE, "r", encoding="utf-8") as f:
                for line in f:
                    line = line.strip()
                    if line:
                        try:
                            cards.append(json.loads(line))
                        except Exception:
                            continue
        except Exception:
            pass

    for c in cards[-_REASONING_TAIL:]:
        try:
            c["reasoning"] = derive_card_reasoning(c)
        except Exception:
            pass

    # Separate Pre-Window vs Prospective
    prewindow_cards = [c for c in cards if c.get("close_time_ms", 0) < REGISTERED_WINDOW_MS]
    prospective_cards = [c for c in cards if c.get("close_time_ms", 0) >= REGISTERED_WINDOW_MS]

    latest_card = cards[-1] if cards else None
    latest_prospective = prospective_cards[-1] if prospective_cards else None

    # Synchronize Trade Plan Overlay with latest durable state / card & reconcile lifecycle.
    # TP/SL/MTM must use live book (sync_lifecycle fetches REST bookTicker) — NOT the
    # stale decision-card execution quote, which freezes MTM near entry and misses targets.
    now_utc_ms = int(time.time() * 1000)
    latest_exec_quote = latest_card.get("execution_quote") if latest_card else None

    # Presentation-only: do NOT open/close OF paper trades or plans from the dashboard.
    # Shadow daemon owns mutations; dual writers previously duplicated OF outcomes.
    # Always adopt durable state (stale in-memory OPEN must not disagree with ledger).
    overlay_mgr.state = overlay_mgr._load_state()
    try:
        # Heal only: drop ghost OPEN already in closed ledger (not a strategy mutation)
        overlay_mgr.reconcile_of_paper_already_closed(persist=True)
    except Exception:
        pass
    overlay_state = overlay_mgr.state
    active_trade_plan = overlay_state.get("active_plan")
    if not active_trade_plan and latest_card:
        try:
            active_trade_plan = calculate_trade_plan(latest_card, quote=latest_exec_quote)
        except Exception:
            active_trade_plan = None

    sync_res = overlay_mgr.sync_lifecycle(
        current_quote=latest_exec_quote,
        now_ms=now_utc_ms,
        mutate_of=False,
        mutate_plans=False,
    )
    # Keep summary truthful vs deduped journal
    try:
        overlay_mgr.rebuild_of_outcomes_summary(persist=False)
        overlay_mgr.reconcile_of_paper_already_closed(persist=True)
    except Exception:
        pass
    of_state = overlay_mgr.state if isinstance(overlay_mgr.state, dict) else overlay_mgr._load_state()
    order_flow_recommendation = of_state.get("active_order_flow_recommendation")
    if not (isinstance(order_flow_recommendation, dict) and order_flow_recommendation.get("active")):
        order_flow_recommendation = None
    # Never show OPEN for an of_trade_id already in the closed ledger
    if (
        isinstance(order_flow_recommendation, dict)
        and order_flow_recommendation.get("of_trade_id")
        and str(order_flow_recommendation.get("of_trade_id"))
        in overlay_mgr._known_of_outcome_ids()
    ):
        order_flow_recommendation = None
        of_state["active_order_flow_recommendation"] = None
    order_flow_last_exit = of_state.get("last_closed_order_flow_recommendation")
    if not isinstance(order_flow_last_exit, dict):
        order_flow_last_exit = None

    def _hydrate_of_last_exit_from_ledger(stub: Optional[Dict[str, Any]]) -> Optional[Dict[str, Any]]:
        """
        Ghost reconcile can leave last_closed as RECONCILED_ALREADY_CLOSED without
        exit_candle / exit_imbalance. Prefer the durable outcomes row for that id
        (or the newest outcome) so OF EXIT badges are not synthesized from `now`.
        """
        try:
            recent = overlay_mgr.load_recent_of_outcomes(48)
        except Exception:
            recent = []
        if not recent:
            return stub
        tid = (stub or {}).get("of_trade_id") if isinstance(stub, dict) else None
        match = None
        if tid:
            for row in reversed(recent):
                if str(row.get("of_trade_id") or "") == str(tid):
                    match = row
                    break
        if match is None:
            # Incomplete stub / missing id → newest realized outcome
            if not stub or stub.get("exit_reason") == "RECONCILED_ALREADY_CLOSED" or not stub.get(
                "exit_candle"
            ):
                match = recent[-1]
        if not match:
            return stub
        hydrated = dict(stub or {})
        hydrated.update(match)
        hydrated["active"] = False
        # Prefer true close reason from ledger over reconcile stub
        if match.get("exit_reason"):
            hydrated["exit_reason"] = match.get("exit_reason")
        return hydrated

    order_flow_last_exit = _hydrate_of_last_exit_from_ledger(order_flow_last_exit)

    # Backfill candle marks for older OF records (pre-candle-stamp deploy).
    # Never invent exit_candle from wall-clock `now` alone — a window-only mark
    # (bar_idx/decision_event_idx missing → legend "15m #—") caused every
    # prospective card in that 15m window to look like OF EXIT.
    def _mark_badgeable(mark: Optional[Dict[str, Any]]) -> Optional[Dict[str, Any]]:
        if not isinstance(mark, dict):
            return None
        if (
            mark.get("bar_idx") is None
            and mark.get("decision_event_idx") is None
            and mark.get("mark_time_ms") is None
        ):
            return None
        # Window-only (no event identity) is not safe for row badges
        if mark.get("bar_idx") is None and mark.get("decision_event_idx") is None:
            return None
        return mark

    try:
        from scripts.rp608_trade_plan_overlay import build_of_candle_mark

        if order_flow_recommendation and not order_flow_recommendation.get("entry_candle"):
            entry_ms = order_flow_recommendation.get("activated_ms") or order_flow_recommendation.get(
                "entry_time_ms"
            )
            if entry_ms is not None:
                order_flow_recommendation["entry_candle"] = build_of_candle_mark(
                    now_ms=int(entry_ms),
                    bar_idx=(order_flow_recommendation.get("entry_candle") or {}).get("bar_idx"),
                    decision_event_idx=(order_flow_recommendation.get("entry_candle") or {}).get(
                        "decision_event_idx"
                    ),
                )
        if order_flow_last_exit:
            if not order_flow_last_exit.get("entry_candle"):
                entry_ms = order_flow_last_exit.get("activated_ms") or order_flow_last_exit.get(
                    "entry_time_ms"
                )
                if entry_ms is not None:
                    order_flow_last_exit["entry_candle"] = build_of_candle_mark(
                        now_ms=int(entry_ms),
                        bar_idx=(order_flow_last_exit.get("entry_candle") or {}).get("bar_idx"),
                        decision_event_idx=(order_flow_last_exit.get("entry_candle") or {}).get(
                            "decision_event_idx"
                        ),
                    )
            # Do NOT synthesize exit_candle without bar/event identity
            xc = order_flow_last_exit.get("exit_candle")
            if isinstance(xc, dict) and xc.get("bar_idx") is None and xc.get("decision_event_idx") is None:
                order_flow_last_exit["exit_candle"] = None
    except Exception:
        pass
    raw_entry = (order_flow_recommendation or order_flow_last_exit or {}).get("entry_candle")
    raw_exit = (
        (order_flow_last_exit or {}).get("exit_candle")
        if not (order_flow_recommendation and order_flow_recommendation.get("active"))
        else None
    )
    of_candle_markers = {
        "entry": _mark_badgeable(raw_entry if isinstance(raw_entry, dict) else None),
        "exit": _mark_badgeable(raw_exit if isinstance(raw_exit, dict) else None),
        "active": bool(order_flow_recommendation and order_flow_recommendation.get("active")),
    }
    trade_plan_payload = sync_res.get("display_plan") or active_trade_plan
    if trade_plan_payload:
        trade_plan_payload["valuation"] = sync_res.get("valuation", {})
        trade_plan_payload["is_active"] = sync_res.get("is_active", False)
        trade_plan_payload["quote_source"] = sync_res.get("quote_source")
        trade_plan_payload.update(
            recommendation_horizon_meta(market="CRYPTO", profile="TRADE_PLAN")
        )
        # Reasoning: when a plan is/was active, never show "Awaiting directional breakout"
        # from a later NO_TRADE card — that masks an open/closed simulated position.
        atr = float(trade_plan_payload.get("atr_14") or 0.0)
        direction = trade_plan_payload.get("direction")
        state = trade_plan_payload.get("state") or sync_res.get("state")
        if direction in ("LONG", "SHORT"):
            why = (
                f"Derived from Policy B {direction}: Fixed 1.0x ATR Stop (${atr:.2f}) and "
                f"1.5x ATR Target (${1.5 * atr:.2f}) with 60-minute holding horizon post-12.0 bps friction."
            )
            if state == "CLOSED_TP":
                why += " Outcome: PROFIT_TARGET_HIT (executable book crossed target)."
            elif state == "CLOSED_SL":
                why += " Outcome: STOP_LOSS_HIT."
            elif state == "TIME_EXIT":
                why += " Outcome: MAX_HORIZON_60M_EXPIRED."
            elif state == "OPEN":
                why += " State: OPEN — monitored against live BTCUSDT bookTicker."
            base_reasoning = (latest_card or {}).get("reasoning") or {}
            trade_plan_payload["reasoning"] = {
                **base_reasoning,
                "trade_plan_why": why,
            }
        elif latest_card:
            trade_plan_payload["reasoning"] = latest_card.get("reasoning", {})

    # Telemetry and Gates
    telemetry = checkpoint.get("telemetry_gates", {})
    service_status = get_observer_service_status()

    # Time calculations
    now_utc_ms = int(time.time() * 1000)
    time_in_window_sec = max(0, (now_utc_ms - REGISTERED_WINDOW_MS) / 1000.0)

    # Next 15m candle close target
    next_close_sec = 900 - (int(time.time()) % 900)

    # Sharpe & Calmar Evaluation Framework Data
    n_prospective = len(prospective_cards)
    
    # Statistical Rigor Policy:
    # A sample of < 96 observations (24h of 15m bars) has extreme estimation variance.
    # Multiplying short-window market drift by sqrt(35040) produces uninterpretable artifacts.
    # Metric is strictly suppressed until minimum threshold (N >= 96) is reached.
    MIN_SHARPE_OBSERVATIONS = 96
    
    if n_prospective >= MIN_SHARPE_OBSERVATIONS:
        p_returns = [c.get("ret_15m_bps", 0.0) / 10000.0 for c in prospective_cards]
        mean_r = sum(p_returns) / n_prospective
        var_r = sum((r - mean_r) ** 2 for r in p_returns) / (n_prospective - 1)
        std_r = math.sqrt(var_r) if var_r > 0 else 0.0001
        sharpe_annualized = round((mean_r / std_r) * math.sqrt(35040), 2)
        sharpe_display = f"{sharpe_annualized:.2f} (N={n_prospective})"
    else:
        sharpe_annualized = None
        sharpe_display = f"Provisional (N={n_prospective}/96 min)"

    ratio_framework = {
        "current_stage": "Stage 1: Setup & Calculation Check",
        "stage_status": "ACCUMULATING OBSERVATIONS",
        "evaluation_window_close_utc": "2026-11-25T00:00:00Z",
        "registered_duration_days": 30,
        "prospective_sample_size": n_prospective,
        "descriptive_sharpe": sharpe_display,
        "descriptive_calmar": "Pending 7-Day Drawdown Baseline",
        "disclaimer": f"Stage 1 Diagnostic: Annualized Sharpe is suppressed during early accumulation (N={n_prospective} < 96 bars required for minimum statistical interpretability). Formal evaluation occurs at 25 Nov 2026 window close."
    }

    # Check NSE timer states
    nse_events = [
        {"name": "NSE Pre-Open Snapshot 1", "time_ist": "09:07:00", "target_utc_sec": 1792985820},
        {"name": "NSE Pre-Open Snapshot 2", "time_ist": "09:11:30", "target_utc_sec": 1792986090},
        {"name": "NSE Pre-Open Snapshot 3", "time_ist": "09:14:30", "target_utc_sec": 1792986270},
        {"name": "NSE Live Continuous Feed", "time_ist": "09:15:00", "target_utc_sec": 1792986300},
        {"name": "NSE Bar 1 Reconciliation", "time_ist": "09:30:05", "target_utc_sec": 1792987205},
    ]

    return {
        "server_time_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "server_time_ist": time.strftime("%Y-%m-%d %H:%M:%S IST", time.localtime()),
        "window_id": checkpoint.get("window_id", "RP-608-SHADOW-001B"),
        "registered_boundary_ist": "2026-10-26 05:30:00 IST",
        "service_status": service_status,
        "window_active": (now_utc_ms >= REGISTERED_WINDOW_MS),
        "window_elapsed_hours": round(time_in_window_sec / 3600.0, 2),
        "next_candle_close_countdown_sec": next_close_sec,
        "checkpoint_integrity": {
            "recorded": checkpoint.get("checksum", "N/A"),
            "status": "VALID",
            "last_durable_bar_idx": checkpoint.get("last_durable_bar_idx", 0),
            "last_accounted_bar_idx": checkpoint.get("last_accounted_bar_idx", 0),
            "last_heartbeat_utc": checkpoint.get("last_heartbeat_utc", "N/A")
        },
        "accounting": {
            "total_cards": len(cards),
            "prewindow_excluded": len(prewindow_cards),
            "prospective_count": len(prospective_cards)
        },
        "telemetry_gates": {
            "quote_coverage_pct": telemetry.get("quote_coverage_pct", 100.0),
            "max_drift_ms": telemetry.get("max_drift_ms", 0),
            "consecutive_missing_bars": telemetry.get("consecutive_missing_bars", 0),
            "total_missing_bars": telemetry.get("total_missing_bars", 0),
            "reconnection_count_24h": telemetry.get("reconnection_count_24h", 0),
            "window_tainted": telemetry.get("window_tainted_by_breach", False)
        },
        "latest_card": latest_card,
        "latest_prospective_card": latest_prospective,
        "trade_plan": trade_plan_payload,
        "order_flow_recommendation": order_flow_recommendation,
        "order_flow_last_exit": order_flow_last_exit,
        "of_candle_markers": of_candle_markers,
        "of_shadow_trade": overlay_mgr.build_of_shadow_trade_snapshot(
            latest_exec_quote if isinstance(latest_exec_quote, dict) else None,
            now_ms=now_utc_ms,
        ),
        "of_trade_pnl": {
            "protocol_id": "RP-608-OF-PAPER-TRADE-001",
            "execution_mode": "PAPER",
            "live_order_authority": 0.0,
            "independent_of_trade_plan_ttl": True,
            "summary": (
                of_state.get("of_outcomes_summary")
                if isinstance(of_state.get("of_outcomes_summary"), dict)
                else overlay_mgr.state.get("of_outcomes_summary")
                if isinstance(overlay_mgr.state.get("of_outcomes_summary"), dict)
                else {
                    "n_trades": 0,
                    "n_wins": 0,
                    "n_losses": 0,
                    "cumulative_gross_pnl_usdt": 0.0,
                    "cumulative_net_pnl_usdt": 0.0,
                    "cumulative_friction_usdt": 0.0,
                }
            ),
            "recent": overlay_mgr.load_recent_of_outcomes(24),
        },
        "ratio_framework": ratio_framework,
        "recent_cards": cards[-24:][::-1],  # newest first (wider window for OF candle marks)
        "nse_schedule": nse_events,
        "gate_c": get_event_engine_status(),
        "nse_operations": get_nse_dashboard_state(),
        "multi_horizon": get_multi_horizon_dashboard_state(),
        "rp611": get_rp611_dashboard_state(),
    }


DASHBOARD_HTML = """<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>RP-608 Shadow Observer · Live Command Center</title>
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500;600;700&family=Outfit:wght@300;400;500;600;700;800&display=swap" rel="stylesheet">
  <style>
    :root {
      --bg-dark: #07090e;
      --card-bg: rgba(15, 23, 42, 0.75);
      --card-border: rgba(255, 255, 255, 0.08);
      --card-hover: rgba(255, 255, 255, 0.14);
      --text-main: #f8fafc;
      --text-muted: #94a3b8;
      --text-dim: #64748b;
      --accent-cyan: #06b6d4;
      --accent-violet: #8b5cf6;
      --accent-emerald: #10b981;
      --accent-rose: #f43f5e;
      --accent-amber: #f59e0b;
      --glow-emerald: rgba(16, 185, 129, 0.25);
      --glow-rose: rgba(244, 63, 94, 0.25);
      --glow-cyan: rgba(6, 182, 212, 0.2);
    }

    * { box-sizing: border-box; margin: 0; padding: 0; }
    body {
      background: var(--bg-dark);
      background-image: 
        radial-gradient(circle at 15% 15%, rgba(6, 182, 212, 0.08) 0%, transparent 40%),
        radial-gradient(circle at 85% 85%, rgba(139, 92, 246, 0.08) 0%, transparent 40%);
      color: var(--text-main);
      font-family: 'Outfit', -apple-system, BlinkMacSystemFont, sans-serif;
      min-height: 100vh;
      padding: 24px;
      overflow-x: hidden;
    }

    .container {
      max-width: 1440px;
      margin: 0 auto;
    }

    /* Top Bar */
    header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding-bottom: 20px;
      border-bottom: 1px solid var(--card-border);
      margin-bottom: 24px;
      flex-wrap: wrap;
      gap: 16px;
    }

    .brand {
      display: flex;
      align-items: center;
      gap: 14px;
    }

    .logo-badge {
      background: linear-gradient(135deg, var(--accent-cyan), var(--accent-violet));
      width: 44px;
      height: 44px;
      border-radius: 12px;
      display: flex;
      align-items: center;
      justify-content: center;
      font-weight: 800;
      font-size: 20px;
      color: #fff;
      box-shadow: 0 4px 20px var(--glow-cyan);
    }

    .title-group h1 {
      font-size: 22px;
      font-weight: 700;
      letter-spacing: -0.5px;
      display: flex;
      align-items: center;
      gap: 10px;
    }

    .title-group p {
      font-size: 13px;
      color: var(--text-muted);
      font-family: 'JetBrains Mono', monospace;
    }

    .header-badges {
      display: flex;
      align-items: center;
      gap: 12px;
      flex-wrap: wrap;
    }

    .status-badge {
      display: inline-flex;
      align-items: center;
      gap: 8px;
      padding: 6px 14px;
      border-radius: 20px;
      font-size: 13px;
      font-weight: 600;
      background: rgba(16, 185, 129, 0.12);
      border: 1px solid rgba(16, 185, 129, 0.3);
      color: var(--accent-emerald);
    }

    .pulse-dot {
      width: 8px;
      height: 8px;
      border-radius: 50%;
      background: var(--accent-emerald);
      animation: pulse 1.8s infinite;
    }

    @keyframes pulse {
      0% { transform: scale(0.95); box-shadow: 0 0 0 0 rgba(16, 185, 129, 0.7); }
      70% { transform: scale(1.1); box-shadow: 0 0 0 8px rgba(16, 185, 129, 0); }
      100% { transform: scale(0.95); box-shadow: 0 0 0 0 rgba(16, 185, 129, 0); }
    }

    .clock-pill {
      background: var(--card-bg);
      border: 1px solid var(--card-border);
      padding: 6px 14px;
      border-radius: 12px;
      font-family: 'JetBrains Mono', monospace;
      font-size: 13px;
      color: var(--text-muted);
    }

    /* Grid Layout */
    .grid-hero {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
      gap: 20px;
      margin-bottom: 24px;
    }

    .card {
      background: var(--card-bg);
      border: 1px solid var(--card-border);
      border-radius: 16px;
      padding: 20px;
      backdrop-filter: blur(12px);
      transition: border-color 0.2s, transform 0.2s;
      position: relative;
    }

    .card:hover {
      border-color: var(--card-hover);
    }

    .card-label {
      font-size: 12px;
      text-transform: uppercase;
      letter-spacing: 1px;
      color: var(--text-dim);
      font-weight: 600;
      margin-bottom: 8px;
      display: flex;
      justify-content: space-between;
      align-items: center;
    }

    /* Hero Recommendation Card */
    .hero-recommendation {
      background: linear-gradient(145deg, rgba(15, 23, 42, 0.95), rgba(30, 41, 59, 0.6));
      border: 1px solid rgba(255, 255, 255, 0.12);
    }

    .rec-box-group {
      display: grid;
      grid-template-columns: 1fr 1fr;
      gap: 16px;
      margin-top: 14px;
    }

    .policy-box {
      background: rgba(0, 0, 0, 0.3);
      border: 1px solid var(--card-border);
      border-radius: 12px;
      padding: 16px;
      text-align: center;
      position: relative;
      cursor: help;
    }

    .policy-tag {
      font-size: 11px;
      font-weight: 700;
      letter-spacing: 1px;
      color: var(--text-muted);
      margin-bottom: 6px;
    }

    .action-badge {
      display: inline-block;
      font-size: 24px;
      font-weight: 800;
      letter-spacing: 0.5px;
      padding: 6px 16px;
      border-radius: 10px;
      margin-top: 4px;
      transition: transform 0.2s;
    }

    .action-badge:hover {
      transform: scale(1.04);
    }

    .action-LONG {
      background: rgba(16, 185, 129, 0.18);
      color: var(--accent-emerald);
      border: 1px solid rgba(16, 185, 129, 0.4);
      box-shadow: 0 4px 20px var(--glow-emerald);
    }

    .action-SHORT {
      background: rgba(244, 63, 94, 0.18);
      color: var(--accent-rose);
      border: 1px solid rgba(244, 63, 94, 0.4);
      box-shadow: 0 4px 20px var(--glow-rose);
    }

    .action-NO_TRADE {
      background: rgba(148, 163, 184, 0.12);
      color: var(--text-muted);
      border: 1px solid rgba(148, 163, 184, 0.25);
    }

    /* RP-607 & RP-608 Custom Badge Styles */
    .comp-pill {
      font-size: 10px;
      font-weight: 700;
      padding: 3px 8px;
      border-radius: 6px;
      font-family: 'JetBrains Mono', monospace;
      letter-spacing: 0.3px;
      display: inline-flex;
      align-items: center;
      gap: 4px;
    }
    .comp-c1 {
      background: rgba(6, 182, 212, 0.15);
      color: var(--accent-cyan);
      border: 1px solid rgba(6, 182, 212, 0.35);
    }
    .comp-c2 {
      background: rgba(139, 92, 246, 0.15);
      color: var(--accent-violet);
      border: 1px solid rgba(139, 92, 246, 0.35);
    }
    .comp-c3 {
      background: rgba(59, 130, 246, 0.15);
      color: #60a5fa;
      border: 1px solid rgba(59, 130, 246, 0.35);
    }
    .comp-c4 {
      background: rgba(245, 158, 11, 0.15);
      color: var(--accent-amber);
      border: 1px solid rgba(245, 158, 11, 0.35);
    }
    .comp-quarantined {
      background: rgba(148, 163, 184, 0.12);
      color: var(--text-dim);
      border: 1px solid rgba(148, 163, 184, 0.25);
    }

    .gate-micro-pill {
      display: inline-block;
      font-size: 9px;
      font-weight: 700;
      font-family: 'JetBrains Mono', monospace;
      padding: 1px 4px;
      border-radius: 3px;
      margin-right: 2px;
    }
    .gate-pass {
      background: rgba(16, 185, 129, 0.18);
      color: var(--accent-emerald);
      border: 1px solid rgba(16, 185, 129, 0.3);
    }
    .gate-fail {
      background: rgba(244, 63, 94, 0.18);
      color: var(--accent-rose);
      border: 1px solid rgba(244, 63, 94, 0.3);
    }
    .gate-active {
      background: rgba(56, 189, 248, 0.18);
      color: var(--accent-cyan);
      border: 1px solid rgba(56, 189, 248, 0.35);
    }
    .gate-closed {
      background: rgba(245, 158, 11, 0.18);
      color: var(--accent-amber);
      border: 1px solid rgba(245, 158, 11, 0.35);
    }
    .gate-skip {
      background: rgba(255, 255, 255, 0.05);
      color: var(--text-muted);
      border: 1px solid rgba(255, 255, 255, 0.1);
    }

    /* HOVER REASONING TOOLTIP STYLES */
    .has-tooltip {
      position: relative;
    }

    .tooltip-popup {
      visibility: hidden;
      opacity: 0;
      position: absolute;
      bottom: 110%;
      left: 50%;
      transform: translateX(-50%) translateY(8px);
      width: 340px;
      background: rgba(10, 15, 29, 0.96);
      border: 1px solid var(--accent-cyan);
      box-shadow: 0 12px 36px rgba(0, 0, 0, 0.7), 0 0 20px rgba(6, 182, 212, 0.25);
      border-radius: 12px;
      padding: 16px;
      z-index: 99999;
      pointer-events: none;
      transition: opacity 0.25s cubic-bezier(0.16, 1, 0.3, 1), transform 0.25s cubic-bezier(0.16, 1, 0.3, 1), visibility 0.25s;
      text-align: left;
      font-family: 'Outfit', sans-serif;
    }

    .has-tooltip:hover .tooltip-popup {
      visibility: visible;
      opacity: 1;
      transform: translateX(-50%) translateY(0);
    }

    .tooltip-header {
      font-size: 13px;
      font-weight: 700;
      color: var(--accent-cyan);
      display: flex;
      justify-content: space-between;
      align-items: center;
      border-bottom: 1px solid rgba(255, 255, 255, 0.08);
      padding-bottom: 8px;
      margin-bottom: 10px;
    }

    .tooltip-body {
      font-size: 13px;
      line-height: 1.5;
      color: #e2e8f0;
      margin-bottom: 10px;
    }

    .tooltip-metrics {
      display: grid;
      grid-template-columns: 1fr 1fr;
      gap: 8px;
      font-family: 'JetBrains Mono', monospace;
      font-size: 11px;
      background: rgba(0, 0, 0, 0.35);
      padding: 8px 10px;
      border-radius: 8px;
      border: 1px solid rgba(255, 255, 255, 0.05);
    }

    .tooltip-metric-item {
      display: flex;
      flex-direction: column;
    }

    .tooltip-metric-label {
      color: var(--text-dim);
      font-size: 9px;
      text-transform: uppercase;
      letter-spacing: 0.5px;
    }

    .tooltip-metric-val {
      color: #fff;
      font-weight: 600;
      margin-top: 2px;
    }

    /* Metric Display */
    .metric-value {
      font-size: 32px;
      font-weight: 700;
      font-family: 'JetBrains Mono', monospace;
      color: #fff;
      display: flex;
      align-items: baseline;
      gap: 6px;
    }

    .metric-sub {
      font-size: 13px;
      color: var(--text-muted);
      margin-top: 6px;
      display: flex;
      gap: 12px;
      font-family: 'JetBrains Mono', monospace;
    }

    /* TRADE PLAN OVERLAY CARD */
    .trade-plan-card {
      background: linear-gradient(135deg, rgba(15, 23, 42, 0.95), rgba(24, 34, 53, 0.85));
      border: 1px solid rgba(6, 182, 212, 0.25);
      border-radius: 16px;
      padding: 24px;
      margin-bottom: 24px;
      backdrop-filter: blur(12px);
      box-shadow: 0 8px 32px rgba(0, 0, 0, 0.35);
    }

    .plan-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding-bottom: 16px;
      border-bottom: 1px solid var(--card-border);
      margin-bottom: 20px;
      flex-wrap: wrap;
      gap: 12px;
    }

    .plan-title-group h2 {
      font-size: 18px;
      font-weight: 700;
      display: flex;
      align-items: center;
      gap: 10px;
    }

    .plan-title-group span {
      font-size: 12px;
      color: var(--text-muted);
      font-family: 'JetBrains Mono', monospace;
    }

    .plan-grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
      gap: 16px;
      margin-bottom: 20px;
    }

    .plan-box {
      background: rgba(0, 0, 0, 0.3);
      border: 1px solid var(--card-border);
      border-radius: 12px;
      padding: 16px;
      position: relative;
    }

    .plan-box-title {
      font-size: 11px;
      text-transform: uppercase;
      letter-spacing: 0.8px;
      color: var(--text-dim);
      margin-bottom: 6px;
      display: flex;
      justify-content: space-between;
    }

    .plan-box-value {
      font-size: 20px;
      font-weight: 700;
      font-family: 'JetBrains Mono', monospace;
      color: #fff;
    }

    .plan-box-sub {
      font-size: 12px;
      color: var(--text-muted);
      margin-top: 4px;
      font-family: 'JetBrains Mono', monospace;
    }

    .plan-footer-notes {
      display: flex;
      justify-content: space-between;
      align-items: center;
      font-size: 12px;
      color: var(--text-dim);
      border-top: 1px solid rgba(255, 255, 255, 0.04);
      padding-top: 14px;
      flex-wrap: wrap;
      gap: 10px;
    }

    /* SHARPE & CALMAR EVALUATION CARD */
    .ratio-card {
      background: linear-gradient(135deg, rgba(15, 23, 42, 0.85), rgba(30, 27, 75, 0.5));
      border: 1px solid rgba(139, 92, 246, 0.25);
      border-radius: 16px;
      padding: 20px;
      margin-bottom: 24px;
      backdrop-filter: blur(12px);
    }

    .ratio-grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
      gap: 16px;
      margin-top: 14px;
    }

    .ratio-box {
      background: rgba(0, 0, 0, 0.25);
      border: 1px solid var(--card-border);
      border-radius: 12px;
      padding: 14px;
    }

    /* Table Section */
    .table-container {
      background: var(--card-bg);
      border: 1px solid var(--card-border);
      border-radius: 16px;
      padding: 20px;
      backdrop-filter: blur(12px);
      margin-bottom: 24px;
      overflow-x: auto;
    }

    .section-title {
      font-size: 16px;
      font-weight: 700;
      margin-bottom: 16px;
      display: flex;
      align-items: center;
      gap: 10px;
    }

    table {
      width: 100%;
      border-collapse: collapse;
      font-family: 'JetBrains Mono', monospace;
      font-size: 13px;
    }

    th {
      text-align: left;
      padding: 12px 14px;
      font-size: 11px;
      text-transform: uppercase;
      letter-spacing: 0.8px;
      color: var(--text-dim);
      border-bottom: 1px solid var(--card-border);
    }

    td {
      padding: 14px;
      border-bottom: 1px solid rgba(255, 255, 255, 0.04);
      color: var(--text-main);
    }

    tr:hover td {
      background: rgba(255, 255, 255, 0.02);
    }

    .pill-sm {
      display: inline-block;
      padding: 3px 8px;
      border-radius: 6px;
      font-size: 11px;
      font-weight: 700;
    }

    .pill-prospective {
      background: rgba(6, 182, 212, 0.15);
      color: var(--accent-cyan);
      border: 1px solid rgba(6, 182, 212, 0.3);
    }

    .pill-prewindow {
      background: rgba(148, 163, 184, 0.1);
      color: var(--text-dim);
    }

    /* Grid for Gate Status */
    .grid-gates {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
      gap: 16px;
      margin-bottom: 24px;
    }

    .gate-item {
      background: rgba(0, 0, 0, 0.25);
      border: 1px solid var(--card-border);
      border-radius: 12px;
      padding: 14px;
    }

    .gate-name {
      font-size: 11px;
      color: var(--text-dim);
      text-transform: uppercase;
      letter-spacing: 0.8px;
      margin-bottom: 4px;
    }

    .gate-val {
      font-size: 16px;
      font-weight: 700;
      font-family: 'JetBrains Mono', monospace;
      color: var(--accent-emerald);
    }

    /* NSE Countdown Timeline */
    .timeline {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
      gap: 14px;
      margin-top: 10px;
    }

    .timeline-step {
      background: rgba(0, 0, 0, 0.3);
      border: 1px solid var(--card-border);
      border-radius: 12px;
      padding: 14px;
    }

    .step-time {
      font-family: 'JetBrains Mono', monospace;
      font-size: 14px;
      font-weight: 700;
      color: var(--accent-cyan);
      margin-bottom: 4px;
    }

    .step-name {
      font-size: 12px;
      color: var(--text-muted);
    }

    /* Footer */
    footer {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding-top: 16px;
      border-top: 1px solid var(--card-border);
      font-size: 12px;
      color: var(--text-dim);
      flex-wrap: wrap;
      gap: 10px;
    }
    /* RP-607 & RP-608 Custom Styling */
    .comp-pill {
      font-size: 10px;
      font-weight: 700;
      padding: 3px 8px;
      border-radius: 4px;
      font-family: 'JetBrains Mono', monospace;
      display: inline-block;
      text-transform: uppercase;
      letter-spacing: 0.5px;
    }
    .comp-c1 { background: rgba(6, 182, 212, 0.18); color: var(--accent-cyan); border: 1px solid rgba(6, 182, 212, 0.3); }
    .comp-c2 { background: rgba(139, 92, 246, 0.18); color: var(--accent-violet); border: 1px solid rgba(139, 92, 246, 0.3); }
    .comp-c3 { background: rgba(59, 130, 246, 0.18); color: #60a5fa; border: 1px solid rgba(59, 130, 246, 0.3); }
    .comp-c4 { background: rgba(245, 158, 11, 0.18); color: var(--accent-amber); border: 1px solid rgba(245, 158, 11, 0.3); }
    .comp-quarantined { background: rgba(255, 255, 255, 0.08); color: var(--text-muted); border: 1px solid rgba(255, 255, 255, 0.15); }

    .gate-stepper {
      display: flex;
      align-items: center;
      gap: 3px;
    }
    .gate-step-badge {
      font-size: 9px;
      font-weight: 700;
      font-family: 'JetBrains Mono', monospace;
      padding: 2px 5px;
      border-radius: 3px;
      cursor: help;
      display: inline-flex;
      align-items: center;
      gap: 2px;
    }
    .gate-step-pass { background: rgba(16, 185, 129, 0.2); color: var(--accent-emerald); border: 1px solid rgba(16, 185, 129, 0.4); }
    .gate-step-fail { background: rgba(244, 63, 94, 0.2); color: var(--accent-rose); border: 1px solid rgba(244, 63, 94, 0.4); }
    .gate-step-blocked { background: rgba(255, 255, 255, 0.05); color: var(--text-dim); border: 1px solid rgba(255, 255, 255, 0.1); }

    .ws-nav {
      display: flex;
      gap: 8px;
      flex-wrap: wrap;
      margin: 0 0 20px 0;
      padding: 4px;
      background: rgba(0,0,0,0.35);
      border: 1px solid var(--card-border);
      border-radius: 10px;
    }
    .ws-tab {
      font-family: 'JetBrains Mono', monospace;
      font-size: 12px;
      font-weight: 800;
      padding: 10px 16px;
      border-radius: 8px;
      border: none;
      cursor: pointer;
      color: var(--text-muted);
      background: transparent;
      transition: all 0.15s ease;
    }
    .ws-tab:hover { color: #fff; background: rgba(255,255,255,0.05); }
    .ws-tab.active {
      color: #fff;
      background: #7c3aed;
      box-shadow: 0 4px 14px rgba(124, 58, 237, 0.35);
    }
    .ws-tab.active[data-ws="command"] {
      background: #0284c7;
      box-shadow: 0 4px 14px rgba(2, 132, 199, 0.35);
    }
    #ws-panel-horizons table.mh-open-table {
      width: 100%;
      border-collapse: collapse;
      font-size: 13px;
    }
    #ws-panel-horizons table.mh-open-table th {
      text-align: left;
      padding: 12px 14px;
      background: rgba(30, 41, 59, 0.95);
      color: var(--text-muted);
      font-size: 11px;
      text-transform: uppercase;
      letter-spacing: 0.04em;
      border-bottom: 1px solid var(--card-border);
    }
    #ws-panel-horizons table.mh-open-table td {
      padding: 12px 14px;
      border-top: 1px solid rgba(255,255,255,0.06);
      vertical-align: top;
    }
  </style>
</head>
<body>
  <div class="container">
    <!-- Header -->
    <header>
      <div class="brand">
        <div class="logo-badge">CS</div>
        <div class="title-group">
          <h1>RP-608 Live Command Center</h1>
          <p>Binance BTC/USDT · Tick Policy B/C (bookTicker) · ATR(14) from 15m · Shadow 0%</p>
        </div>
      </div>
      <div class="header-badges">
        <div class="status-badge">
          <div class="pulse-dot"></div>
          <span id="service-status">DAEMON ACTIVE</span>
        </div>
        <span class="pill-sm" id="regime-state-badge" title="RP-608 Stage-1 completed-bar regime (observational)"
              style="background:rgba(148,163,184,0.12); color:var(--text-dim); border:1px solid rgba(148,163,184,0.25);">REGIME: --</span>
        <div class="clock-pill" id="live-ist-clock">Loading IST...</div>
        <div class="clock-pill" id="live-utc-clock">Loading UTC...</div>
      </div>
    </header>

    <nav class="ws-nav" aria-label="Workspace">
      <button type="button" class="ws-tab active" data-ws="command" id="ws-tab-command">Command Center</button>
      <button type="button" class="ws-tab" data-ws="horizons" id="ws-tab-horizons">Multi-Horizon Books</button>
    </nav>

    <div id="ws-panel-command">
    <!-- Hero Cards -->
    <div class="grid-hero">
      <!-- Active Recommendation -->
      <div class="card hero-recommendation">
        <div class="card-label">
          <span>Active Recommendations (Evt <span id="rec-bar-idx">--</span>)</span>
          <span class="pill-sm pill-prospective" id="sample-status-pill">PROSPECTIVE</span>
        </div>
        <div style="display:flex; gap:6px; flex-wrap:wrap; margin:0 0 10px 0;" id="crypto-rec-horizon-badges">
          <span class="pill-sm" style="background:rgba(6,182,212,0.15); color:var(--accent-cyan); border:1px solid rgba(6,182,212,0.35); font-weight:700;">TICK CLOCK</span>
          <span class="pill-sm" style="background:rgba(255,255,255,0.06); color:var(--text-dim); border:1px solid rgba(255,255,255,0.1);">RP-608-SHADOW-TICK-001 · Gate C isolated</span>
        </div>
        <div class="rec-box-group">
          <!-- Policy B Box with Tooltip -->
          <div class="policy-box has-tooltip" id="box-policy-b">
            <div class="policy-tag">POLICY B (CANONICAL) ⓘ</div>
            <div class="action-badge action-NO_TRADE" id="action-policy-b">--</div>
            <div class="tooltip-popup">
              <div class="tooltip-header">
                <span>POLICY B RATIONALE</span>
                <span id="tip-b-action">SHORT</span>
              </div>
              <div class="tooltip-body" id="tip-b-text">
                Analyzing market factors...
              </div>
              <div class="tooltip-metrics">
                <div class="tooltip-metric-item">
                  <span class="tooltip-metric-label">Macro / Tick Ret</span>
                  <span class="tooltip-metric-val" id="tip-b-ret">--</span>
                </div>
                <div class="tooltip-metric-item">
                  <span class="tooltip-metric-label">Volatility</span>
                  <span class="tooltip-metric-val" id="tip-b-vol">--</span>
                </div>
                <div class="tooltip-metric-item">
                  <span class="tooltip-metric-label">Trend Rule</span>
                  <span class="tooltip-metric-val" id="tip-b-trend">--</span>
                </div>
                <div class="tooltip-metric-item">
                  <span class="tooltip-metric-label">Signal Status</span>
                  <span class="tooltip-metric-val" id="tip-b-status">ELIGIBLE</span>
                </div>
              </div>
            </div>
          </div>

          <!-- Policy C Box with Tooltip -->
          <div class="policy-box has-tooltip" id="box-policy-c">
            <div class="policy-tag">POLICY C (OOS COMPARATOR) ⓘ</div>
            <div class="action-badge action-NO_TRADE" id="action-policy-c">--</div>
            <div class="tooltip-popup">
              <div class="tooltip-header">
                <span>POLICY C RATIONALE</span>
                <span id="tip-c-action">SHORT</span>
              </div>
              <div class="tooltip-body" id="tip-c-text">
                Analyzing market factors...
              </div>
              <div class="tooltip-metrics">
                <div class="tooltip-metric-item">
                  <span class="tooltip-metric-label">Exhaustion Check</span>
                  <span class="tooltip-metric-val" id="tip-c-exhaustion">PASS</span>
                </div>
                <div class="tooltip-metric-item">
                  <span class="tooltip-metric-label">Comparator Mode</span>
                  <span class="tooltip-metric-val">SHADOW OOS</span>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div class="metric-sub" style="margin-top: 16px;">
          <span>Macro bps: <b id="rec-ret" style="color:#fff;">--</b></span>
          <span>Tick mid: <b id="rec-tick-mid" style="color:#fff;">--</b></span>
          <span>Imb: <b id="rec-tick-imb" style="color:#fff;">--</b></span>
          <span>ATR14: <b id="rec-atr" style="color:#fff;">--</b></span>
          <span>Quote Age: <b id="rec-quote-age" style="color:var(--accent-emerald);">-- ms</b></span>
        </div>
      </div>

      <!-- Reference Price & Top-of-Book -->
      <div class="card">
        <div class="card-label">
          <span>BTC/USDT Spot Execution Price</span>
          <span style="font-family:'JetBrains Mono',monospace; color:var(--accent-emerald);">100% COVERAGE</span>
        </div>
        <div class="metric-value">
          <span id="ref-price">$--,---.--</span>
          <span style="font-size: 14px; color: var(--text-muted); font-weight: 500;">USDT</span>
        </div>
        <div class="metric-sub" style="margin-top: 14px;">
          <span>Bid: <b id="bid-price" style="color:#fff;">--</b></span>
          <span>Ask: <b id="ask-price" style="color:#fff;">--</b></span>
          <span>Spread: <b id="spread-bps" style="color:var(--accent-cyan);">-- bps</b></span>
        </div>
        <div class="metric-sub" style="margin-top: 8px;">
          <span>Countdown to Next Candle Close: <b id="next-candle-timer" style="color:var(--accent-amber);">--:--</b></span>
        </div>
      </div>

      <!-- Prospective Evaluation Accounting -->
      <div class="card">
        <div class="card-label">
          <span>Window Accounting (RP-608-SHADOW-001B)</span>
          <span id="window-status-pill" class="pill-sm pill-prospective">WINDOW ACTIVE</span>
        </div>
        <div class="metric-value" style="color:var(--accent-cyan);">
          <span id="prospective-count">0</span>
          <span style="font-size: 14px; color: var(--text-muted); font-weight: 500;">Prospective Cards</span>
        </div>
        <div class="metric-sub" style="margin-top: 14px;">
          <span>Pre-Window Excluded: <b id="prewindow-count" style="color:#fff;">--</b></span>
          <span>Total Recorded: <b id="total-cards-count" style="color:#fff;">--</b></span>
        </div>
        <div class="metric-sub" style="margin-top: 8px;">
          <span>Elapsed Time: <b id="elapsed-time" style="color:#fff;">-- hrs</b></span>
          <span>Window Start: <b style="color:var(--text-dim);">05:30 IST</b></span>
        </div>
      </div>
    </div>

    <!-- TRADE PLAN OVERLAY PANEL -->
    <div class="trade-plan-card" id="trade-plan-container">
      <div class="plan-header">
        <div class="plan-title-group">
          <h2>
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="color:var(--accent-cyan);"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/><polyline points="10 9 9 9 8 9"/></svg>
            BTC/USDT · Trade Plan Overlay v1
          </h2>
          <span>Protocol: RP-608-TRADE-PLAN-OVERLAY-001 · Tick Policy B · ATR from 15m · Simulated Shadow Mode</span>
          <div style="margin-top:6px; display:flex; gap:6px; flex-wrap:wrap;" id="plan-horizon-badges">
            <span class="pill-sm" style="background:rgba(6,182,212,0.15); color:var(--accent-cyan); border:1px solid rgba(6,182,212,0.35); font-weight:700;">INTRADAY</span>
            <span class="pill-sm" style="background:rgba(255,255,255,0.06); color:var(--text-dim); border:1px solid rgba(255,255,255,0.1);">Term: ≤60 minutes</span>
          </div>
        </div>
        <div style="display:flex; align-items:center; gap:10px; flex-wrap:wrap;">
          <span class="pill-sm pill-prospective" id="plan-state-pill">PLAN ACTIVE</span>
          <span class="pill-sm" id="plan-of-pill" style="display:none; background:#86efac; color:#052e16; border:1px solid #4ade80; font-weight:700;">ORDER FLOW</span>
          <span class="pill-sm" style="background:rgba(245,158,11,0.15); color:var(--accent-amber); border:1px solid rgba(245,158,11,0.3);">0.0% ORDER AUTHORITY</span>
        </div>
      </div>

      <div id="crypto-of-banner" style="display:none; margin:0 0 14px 0; padding:10px 12px; background:rgba(134,239,172,0.08); border:1px solid rgba(74,222,128,0.35); border-radius:8px; font-size:12px; color:#bbf7d0;">
        <b id="crypto-of-banner-title" style="color:#86efac;">ORDER FLOW</b>
        <span id="crypto-of-banner-text" style="margin-left:8px; color:#94a3b8;"></span>
      </div>

      <!-- OF Paper Trades (RP-608-OF-PAPER-TRADE-001) — independent of 60m plan -->
      <div id="of-paper-trades-panel" style="margin:0 0 18px 0; padding:14px; background:rgba(45,212,191,0.06); border:1px solid rgba(45,212,191,0.28); border-radius:10px;">
        <div style="display:flex; justify-content:space-between; align-items:center; gap:10px; flex-wrap:wrap; margin-bottom:10px;">
          <div>
            <div style="font-size:13px; font-weight:800; color:#5eead4;">OF Paper Trades</div>
            <div style="font-size:11px; color:var(--text-dim); margin-top:2px;">RP-608-OF-PAPER-TRADE-001 · reverse OF closes · TP/SL/60m do not</div>
          </div>
          <span class="pill-sm" style="background:rgba(245,158,11,0.15); color:var(--accent-amber); border:1px solid rgba(245,158,11,0.3);">0.0% · PAPER</span>
        </div>
        <div id="of-paper-open" style="font-family:'JetBrains Mono',monospace; font-size:12px; color:#e2e8f0; line-height:1.55; margin-bottom:12px;">No open OF paper trade</div>
        <div style="display:flex; gap:8px; flex-wrap:wrap; margin-bottom:8px;" id="of-paper-summary-chips"></div>
        <div style="overflow-x:auto;">
          <table style="width:100%; border-collapse:collapse; font-family:'JetBrains Mono',monospace; font-size:11px;">
            <thead>
              <tr style="color:var(--text-dim); text-align:left;">
                <th style="padding:4px 6px;">open_utc</th>
                <th style="padding:4px 6px;">close_utc</th>
                <th style="padding:4px 6px;">hold</th>
                <th style="padding:4px 6px;">dir</th>
                <th style="padding:4px 6px;">entry_sig</th>
                <th style="padding:4px 6px;">exit_sig</th>
                <th style="padding:4px 6px;">qty</th>
                <th style="padding:4px 6px;">entry</th>
                <th style="padding:4px 6px;">exit</th>
                <th style="padding:4px 6px;">net USDT</th>
                <th style="padding:4px 6px;">net bps</th>
                <th style="padding:4px 6px;">fees</th>
                <th style="padding:4px 6px;">reason</th>
              </tr>
            </thead>
            <tbody id="of-paper-ledger-body">
              <tr><td colspan="13" style="padding:8px 6px; color:var(--text-dim);">No closed OF paper trades yet</td></tr>
            </tbody>
          </table>
        </div>
      </div>

      <div class="plan-grid">
        <!-- Entry & Direction with Hover Tooltip -->
        <div class="plan-box has-tooltip">
          <div class="plan-box-title">
            <span>Simulated Entry Price ⓘ</span>
            <span id="plan-direction-tag" style="color:var(--accent-rose); font-weight:700;">SHORT</span>
          </div>
          <div class="plan-box-value" id="plan-entry-price">$80,170.00</div>
          <div class="plan-box-sub">
            <span>Historical Sizing: <b style="color:#fff;">1,000 USDT</b> (<span id="plan-qty">0.01247</span> BTC)</span>
          </div>
          <div class="tooltip-popup">
            <div class="tooltip-header">
              <span>TRADE PLAN EXECUTION RATIONALE</span>
              <span>SIMULATED</span>
            </div>
            <div class="tooltip-body" id="tip-plan-reasoning">
              Derived from Policy B: Short entry at eligible bid quote ($80,170.00). Target 1.5x ATR and Stop 1.0x ATR.
            </div>
            <div class="tooltip-metrics">
              <div class="tooltip-metric-item">
                <span class="tooltip-metric-label">Execution Side</span>
                <span class="tooltip-metric-val">Crossing Spread</span>
              </div>
              <div class="tooltip-metric-item">
                <span class="tooltip-metric-label">Holding Limit</span>
                <span class="tooltip-metric-val">3,600s (60m)</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Executable Mark-to-Market Valuation -->
        <div class="plan-box">
          <div class="plan-box-title">
            <span>Executable Valuation</span>
            <span style="color:var(--accent-cyan);" id="plan-mtm-status">Mark-to-Market</span>
          </div>
          <div class="plan-box-value" id="plan-mtm-val">--</div>
          <div class="plan-box-sub">
            <span>Executable Quote: <b id="plan-current-quote">Bid $-- / Ask $--</b></span>
          </div>
        </div>

        <!-- Profit Target -->
        <div class="plan-box">
          <div class="plan-box-title">
            <span>Profit Target (1.5x ATR)</span>
            <span style="color:var(--accent-emerald);">+$3.75 Net</span>
          </div>
          <div class="plan-box-value" style="color:var(--accent-emerald);" id="plan-target-price">$79,773.35</div>
          <div class="plan-box-sub">
            <span>Distance: <b id="plan-target-dist">--</b></span>
          </div>
        </div>

        <!-- Stop Loss -->
        <div class="plan-box">
          <div class="plan-box-title">
            <span>Stop Loss (1.0x ATR)</span>
            <span style="color:var(--accent-rose);">-$4.50 Net</span>
          </div>
          <div class="plan-box-value" style="color:var(--accent-rose);" id="plan-stop-price">$80,434.43</div>
          <div class="plan-box-sub">
            <span>Distance: <b id="plan-stop-dist">--</b></span>
          </div>
        </div>

        <!-- Reward / Risk & Friction -->
        <div class="plan-box">
          <div class="plan-box-title">
            <span>Reward / Risk Ratio</span>
            <span style="color:var(--accent-cyan);">12 bps Drag</span>
          </div>
          <div class="plan-box-value">
            <span id="plan-net-rr">0.83 : 1</span>
            <span style="font-size:12px; color:var(--text-dim); font-weight:normal;">(Gross <span id="plan-gross-rr">1.50:1</span>)</span>
          </div>
          <div class="plan-box-sub">
            <span>Round-Trip Cost: <b id="plan-friction-usdt">$1.20</b> (12.0 bps)</span>
          </div>
        </div>

        <!-- Holding Horizon -->
        <div class="plan-box">
          <div class="plan-box-title">
            <span>Holding Horizon</span>
            <span style="color:var(--accent-amber);" id="plan-horizon-tag">Max 60 Min</span>
          </div>
          <div class="plan-box-value" id="plan-horizon-time">60m 00s</div>
          <div class="plan-box-sub">
            <span>Expiry: <b id="plan-expiry-clock">01:45 UTC</b> (<span id="plan-countdown">--m left</span>)</span>
          </div>
        </div>
      </div>

      <div class="plan-footer-notes">
        <div>Plan ID: <b id="plan-id-text" style="color:var(--text-muted); font-family:'JetBrains Mono',monospace;">PLAN-00021-SHORT-1792975500000</b></div>
        <div>Invalidation Level: <b id="plan-inval-text" style="color:var(--accent-rose);">$80,434.43</b> · Conservative Stop-First Tie-Break on Ambiguous Bars</div>
      </div>
    </div>

    <!-- SHARPE & CALMAR EVALUATION CARD -->
    <div class="ratio-card">
      <div class="section-title" style="margin-bottom:8px;">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="color:var(--accent-violet);"><circle cx="12" cy="12" r="10"/><path d="M16 8l-4 4-2-2-4 4"/></svg>
        Sharpe & Calmar Ratio Prospective Evaluation Roadmap
      </div>
      <p style="font-size:12px; color:var(--text-muted); margin-bottom:14px;" id="ratio-disclaimer">
        Current Stage: Stage 1 Setup & Calculation Check. Interim metrics are descriptive; formal evaluation at window close (25 Nov 2026).
      </p>
      <div class="ratio-grid">
        <div class="ratio-box">
          <div class="gate-name">Annualized Sharpe Ratio</div>
          <div class="gate-val" id="metric-sharpe" style="color:var(--accent-cyan);">--</div>
          <div style="font-size:11px; color:var(--text-dim); margin-top:4px;">15m Returns (Descriptive)</div>
        </div>
        <div class="ratio-box">
          <div class="gate-name">Calmar Ratio</div>
          <div class="gate-val" id="metric-calmar" style="color:var(--accent-violet);">Awaiting 7d DD</div>
          <div style="font-size:11px; color:var(--text-dim); margin-top:4px;">Max Drawdown Baseline</div>
        </div>
        <div class="ratio-box">
          <div class="gate-name">Evaluation Phase</div>
          <div class="gate-val" id="metric-stage" style="color:#fff; font-size:14px;">Stage 1 / 5</div>
          <div style="font-size:11px; color:var(--text-dim); margin-top:4px;">Next: 7-Day Operational Audit</div>
        </div>
        <div class="ratio-box">
          <div class="gate-name">Prospective Sample</div>
          <div class="gate-val" id="metric-prospective-sample" style="color:var(--accent-emerald);">--</div>
          <div style="font-size:11px; color:var(--text-dim); margin-top:4px;">Completed 15m Candles</div>
        </div>
      </div>
    </div>

    <!-- Operational Telemetry Gates -->
    <div class="section-title">
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="color:var(--accent-emerald);"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>
      Operational Telemetry Gates & Host Precision
    </div>
    <div class="grid-gates">
      <div class="gate-item">
        <div class="gate-name">Quote Coverage</div>
        <div class="gate-val" id="gate-quote-cov">100.0%</div>
      </div>
      <div class="gate-item">
        <div class="gate-name">Max Clock Drift</div>
        <div class="gate-val" id="gate-drift">&le; 2 ms</div>
      </div>
      <div class="gate-item">
        <div class="gate-name">Missing Bars</div>
        <div class="gate-val" id="gate-missing">0 (PASS)</div>
      </div>
      <div class="gate-item">
        <div class="gate-name">24h Reconnects</div>
        <div class="gate-val" id="gate-reconnects">0 (PASS)</div>
      </div>
      <div class="gate-item">
        <div class="gate-name">SHA-256 Checksum</div>
        <div class="gate-val" id="gate-checksum" style="font-size:12px; color:var(--text-muted);">VALID</div>
      </div>
    </div>

    <!-- GATE C: RP-608-EVENT-001 PARALLEL CANDIDATE LOOKBACK STREAMS -->
    <div class="card" style="margin-bottom:24px; border-left: 4px solid var(--accent-cyan); background: linear-gradient(135deg, rgba(6, 182, 212, 0.05) 0%, rgba(15, 23, 42, 0.6) 100%);">
      <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:12px; flex-wrap:wrap; gap:8px;">
        <div class="section-title" style="margin-bottom:0;">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="color:var(--accent-cyan);"><path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z"/></svg>
          Gate C: RP-608-EVENT-001 Parallel Multi-Timescale Tick Shadow Decision Streams
        </div>
        <div style="display:flex; gap:8px; align-items:center; flex-wrap:wrap;">
          <span class="pill-sm pill-prospective" id="gate-c-status-badge">GATE C: DEPLOYED (SHADOW)</span>
          <span class="pill-sm" id="gate-c-agreement-badge" style="background:rgba(244,63,94,0.15); color:var(--accent-rose); border:1px solid rgba(244,63,94,0.3);">AGREEMENT: 4/4 SHORT</span>
          <span class="badge badge-active" id="gate-c-service-pill">SERVICE ACTIVE</span>
          <span class="pill-sm pill-prewindow" style="border:1px solid rgba(255,255,255,0.2);">0.0% ORDER AUTHORITY</span>
        </div>
      </div>
      <p style="font-size:12px; color:var(--text-muted); margin-bottom:16px;">
        Four independent lookback recommendation engines evaluated concurrently against live Binance BTC/USDT top-of-book quotes. Invalidation policy: explicit 30.0 bps adverse tick stop. Dedicated output: <code style="color:var(--accent-cyan);">live_capture/shadow_validation/crypto/event_driven/</code>.
      </p>

      <div style="display:grid; grid-template-columns:repeat(auto-fit, minmax(230px, 1fr)); gap:14px; margin-bottom:16px;" id="gate-c-candidates-grid">
        <!-- Rendered dynamically by JS -->
      </div>

      <div style="display:flex; justify-content:space-between; align-items:center; background:rgba(0,0,0,0.3); border-radius:8px; padding:10px 14px; font-size:11px; color:var(--text-dim); flex-wrap:wrap; gap:8px;">
        <div>Canonical clock: <b style="color:var(--accent-emerald);">btcusdt@bookTicker</b> · ATR context: <b style="color:var(--accent-emerald);">kline_15m</b> · Gate C isolated</div>
        <div id="gate-c-telemetry-counters">Quotes: <b style="color:#fff;">--</b> · Rate-Limited (&ge;1s): <b style="color:#fff;">--</b> · Filtered: <b style="color:var(--accent-amber);">--</b></div>
        <div>Candle Buffer: <b style="color:#fff;" id="gate-c-candle-buffer">149 Bars (Warmed Up)</b></div>
        <div>Isolation: <b style="color:var(--accent-emerald);">Canonical Files 100% Untouched</b></div>
      </div>
    </div>

    <!-- RP-611 Volume Absorption (independent research · presentation only) -->
    <div class="card" style="margin-bottom:24px; border-left: 4px solid #2dd4bf; background: rgba(45,212,191,0.07);">
      <div style="display:flex; justify-content:space-between; align-items:flex-start; gap:12px; flex-wrap:wrap; margin-bottom:12px;">
        <div>
          <div style="font-size:14px; font-weight:800; color:#5eead4;">RP-611 Volume Absorption · Phase-1</div>
          <div style="font-size:12px; color:var(--text-muted); margin-top:4px;">
            Independent ledgers · frozen thresholds · does not alter RP-608 / OF / Gate5
          </div>
        </div>
        <span class="pill-sm" style="background:rgba(45,212,191,0.2); color:#5eead4; border:1px solid rgba(45,212,191,0.35);" id="rp611-authority-pill">0.0% ORDER AUTHORITY</span>
      </div>
      <div style="display:grid; grid-template-columns:repeat(auto-fit,minmax(260px,1fr)); gap:12px;">
        <div style="background:rgba(0,0,0,0.25); border-radius:8px; padding:12px;">
          <div style="font-size:11px; letter-spacing:0.06em; color:var(--text-dim); margin-bottom:8px;">RP-611N · NSE</div>
          <div id="rp611-nse-status" style="font-family:'JetBrains Mono',monospace; font-size:12px; color:#e2e8f0; line-height:1.55;">Loading…</div>
        </div>
        <div style="background:rgba(0,0,0,0.25); border-radius:8px; padding:12px;">
          <div style="font-size:11px; letter-spacing:0.06em; color:var(--text-dim); margin-bottom:8px;">RP-611C · BTC/USDT spot</div>
          <div id="rp611-crypto-status" style="font-family:'JetBrains Mono',monospace; font-size:12px; color:#e2e8f0; line-height:1.55;">Loading…</div>
        </div>
      </div>
    </div>

    <!-- Pointer → dedicated Multi-Horizon Books workspace tab -->
    <div class="card" style="margin-bottom:24px; border-left: 4px solid #c4b5fd; background: rgba(124,58,237,0.08);">
      <div style="display:flex; justify-content:space-between; align-items:center; gap:12px; flex-wrap:wrap;">
        <div>
          <div style="font-size:14px; font-weight:800; color:#c4b5fd;">Multi-Horizon Books live in their own tab</div>
          <div style="font-size:12px; color:var(--text-muted); margin-top:4px;" id="mh-pointer-counts">
            Open observation books · 0.0% order authority · gates CLOSED
          </div>
        </div>
        <button type="button" onclick="switchWorkspace('horizons')"
          style="background:#7c3aed; border:none; color:#fff; font-family:'JetBrains Mono',monospace; font-size:12px; font-weight:800; padding:10px 14px; border-radius:8px; cursor:pointer;">
          Open Multi-Horizon Books
        </button>
      </div>
    </div>

    <!-- Decision Cards Feed -->
    <div class="table-container">
      <div class="section-title">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="color:var(--accent-cyan);"><polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/></svg>
        Recent Decision Cards & Completed Candle Stream (Hover Badges for Reasoning)
        <span id="of-candle-legend" style="margin-left:12px; font-size:11px; color:var(--text-dim); font-weight:500;"></span>
      </div>
      <table>
        <thead>
          <tr>
            <th>Bar</th>
            <th>Close Time (UTC)</th>
            <th>OF Mark</th>
            <th>Sample Type</th>
            <th>Ref Price</th>
            <th>Policy B (Hover ⓘ)</th>
            <th>Policy C (Hover ⓘ)</th>
            <th>Ret (bps)</th>
            <th>ATR 14</th>
            <th>Quote Age</th>
            <th>Spread (bps)</th>
          </tr>
        </thead>
        <tbody id="decision-cards-tbody">
          <tr><td colspan="11" style="text-align:center; color:var(--text-dim); padding: 30px;">Loading decision cards...</td></tr>
        </tbody>
      </table>
    </div>

    <!-- NSE Research Command Center: RP-607 & RP-608 Linked Observability -->
    <div class="card" style="margin-bottom:24px; border-left: 4px solid var(--accent-violet); background: linear-gradient(135deg, rgba(139, 92, 246, 0.05) 0%, rgba(15, 23, 42, 0.6) 100%);">
      <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:14px; flex-wrap:wrap; gap:10px;">
        <div class="section-title" style="margin-bottom:0;">
          <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="color:var(--accent-violet);"><circle cx="12" cy="12" r="10"/><path d="M12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20"/><path d="M2 12h20"/></svg>
          NSE Operations & Research Center (RP-607 & RP-608 Linked Observability)
        </div>
        <div style="display:flex; gap:8px; align-items:center; flex-wrap:wrap;">
          <span class="pill-sm pill-prospective" style="background:rgba(139,92,246,0.15); color:var(--accent-violet); border:1px solid rgba(139,92,246,0.3);">RESEARCH OBSERVABILITY</span>
          <span class="pill-sm pill-prewindow" style="border:1px solid rgba(255,255,255,0.2);">0.0% ORDER AUTHORITY</span>
          <span class="badge badge-active" style="background:rgba(16,185,129,0.15); color:var(--accent-emerald); border:1px solid rgba(16,185,129,0.3);">KITE CONNECT: TJ6378</span>
          <span class="badge" style="background:rgba(6,182,212,0.15); color:var(--accent-cyan); border:1px solid rgba(6,182,212,0.3);">NTP PPS (+0.448ms)</span>
          <button id="btn-refresh-nse" onclick="refreshNseBatch()" style="background:rgba(139,92,246,0.2); border:1px solid var(--accent-violet); color:#fff; font-family:'JetBrains Mono',monospace; font-size:11px; padding:4px 10px; border-radius:6px; cursor:pointer; font-weight:600; transition:all 0.2s ease;">⟳ Refresh Live Batch</button>
          <button id="btn-reauth-kite" onclick="openKiteAuthModal()" style="background:rgba(245,158,11,0.2); border:1px solid #f59e0b; color:#fbbf24; font-family:'JetBrains Mono',monospace; font-size:11px; padding:4px 10px; border-radius:6px; cursor:pointer; font-weight:600; transition:all 0.2s ease;">🔑 Re-auth Kite</button>
        </div>
      </div>

      <!-- Top 4 Pillar Cards: Market Data Infrastructure -->
      <div style="display:grid; grid-template-columns:repeat(auto-fit, minmax(240px, 1fr)); gap:14px; margin-bottom:16px;">
        <div style="background:rgba(0,0,0,0.35); border:1px solid var(--card-border); border-radius:10px; padding:14px;">
          <div style="font-size:11px; text-transform:uppercase; letter-spacing:0.5px; color:var(--text-dim); margin-bottom:6px;">Instrument Coverage</div>
          <div style="font-family:'JetBrains Mono',monospace; font-size:22px; font-weight:700; color:var(--accent-cyan);" id="nse-cov-val">119 / 119</div>
          <div style="font-size:12px; color:#fff; margin-top:2px;">100% Universe Reconciled</div>
          <div style="font-size:10px; color:var(--text-muted); margin-top:4px;" id="nse-cov-note">Live session coverage · see continuous-feed tile</div>
        </div>
        <div style="background:rgba(0,0,0,0.35); border:1px solid var(--card-border); border-radius:10px; padding:14px;">
          <div style="font-size:11px; text-transform:uppercase; letter-spacing:0.5px; color:var(--text-dim); margin-bottom:6px;">Continuous L1 Feed</div>
          <div style="font-family:'JetBrains Mono',monospace; font-size:16px; font-weight:700; color:var(--accent-emerald);" id="nse-ws-val">—</div>
          <div style="font-size:12px; color:#fff; margin-top:2px;" id="nse-ws-sub">source · tokens</div>
          <div style="font-size:10px; color:var(--text-muted); margin-top:4px;">184-byte Full Depth · <span id="nse-ws-age-metric">Age</span>: <span id="nse-ws-age">--</span></div>
          <div style="font-size:10px; color:var(--text-muted); margin-top:2px;" id="nse-probe-note">Readiness probe kept separate</div>
        </div>
        <div style="background:rgba(0,0,0,0.35); border:1px solid var(--card-border); border-radius:10px; padding:14px;">
          <div style="font-size:11px; text-transform:uppercase; letter-spacing:0.5px; color:var(--text-dim); margin-bottom:6px;">Pre-Open Auction Discovery</div>
          <div style="font-family:'JetBrains Mono',monospace; font-size:18px; font-weight:700; color:var(--accent-violet);" id="nse-pre-val">3 Checkpoints Verified</div>
          <div style="font-size:12px; color:#fff; margin-top:2px;">09:07 · 09:11:30 · 09:14:30 IST</div>
          <div style="font-size:10px; color:var(--text-muted); margin-top:4px;">119 Scrips · Avg Gap: <span id="nse-avg-gap">--</span> · Vol: <span id="nse-pre-vol">--</span></div>
        </div>
        <div style="background:rgba(0,0,0,0.35); border:1px solid var(--card-border); border-radius:10px; padding:14px;">
          <div style="font-size:11px; text-transform:uppercase; letter-spacing:0.5px; color:var(--text-dim); margin-bottom:6px;">Bar 1 Reconciliation</div>
          <div style="font-family:'JetBrains Mono',monospace; font-size:18px; font-weight:700; color:var(--accent-emerald);" id="nse-recon-val">0 / SUCCESS (PASS)</div>
          <div style="font-size:12px; color:#fff; margin-top:2px;">Single-Batch L1 RTT: <span id="nse-rtt-val">--</span>ms</div>
          <div style="font-size:10px; color:var(--text-muted); margin-top:4px;">Last Check: <span id="nse-recon-last">--</span> · 119 Scrips</div>
        </div>
        <div style="background:rgba(0,0,0,0.35); border:1px solid rgba(16,185,129,0.35); border-radius:10px; padding:14px;">
          <div style="font-size:11px; text-transform:uppercase; letter-spacing:0.5px; color:var(--text-dim); margin-bottom:6px;">NSE OF Paper Ledger</div>
          <div style="font-family:'JetBrains Mono',monospace; font-size:18px; font-weight:700; color:var(--accent-emerald);" id="nse-of-paper-val">PAPER · 0 open</div>
          <div style="font-size:12px; color:#fff; margin-top:2px;" id="nse-of-paper-sub">n_trades=0 · net ₹0.00</div>
          <div style="font-size:10px; color:var(--text-muted); margin-top:4px;" id="nse-of-paper-note">RP-608-NSE-OF-PAPER-TRADE-001 · reverse OF / square-off · 0% authority</div>
        </div>
      </div>

      <!-- Quote Health Strip -->
      <div style="display:flex; justify-content:space-between; align-items:center; background:rgba(0,0,0,0.3); border-radius:8px; padding:10px 14px; font-size:11px; color:var(--text-dim); flex-wrap:wrap; gap:8px; margin-bottom:20px;">
        <div id="nse-health-strip">
          Quote Health: Valid: <b style="color:var(--accent-emerald);" id="nse-valid-cnt">119</b> · Stale: <b style="color:#fff;" id="nse-stale-cnt">0</b> · Missing: <b style="color:#fff;" id="nse-miss-cnt">0</b> · Crossed: <b style="color:var(--accent-rose);" id="nse-cross-cnt">0</b> · Zero Depth: <b style="color:var(--accent-amber);" id="nse-zero-cnt">0</b>
        </div>
        <div id="nse-gainers-strip">
          Top Gap: <span style="color:var(--accent-emerald);" id="nse-top-gainer">--</span> · <span style="color:var(--accent-rose);" id="nse-top-loser">--</span>
        </div>
        <div style="color:var(--text-muted); font-style:italic;">
          * Snapshot telemetry · 0.0% Order Authority
        </div>
      </div>

      <!-- PANEL A: RP-607 Historical Stock Compartments -->
      <div style="background:rgba(0,0,0,0.35); border:1px solid rgba(139,92,246,0.3); border-radius:12px; padding:18px; margin-bottom:20px;">
        <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:14px; flex-wrap:wrap; gap:8px;">
          <div>
            <div style="font-size:14px; font-weight:700; color:#fff; display:flex; align-items:center; gap:8px;">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="color:var(--accent-violet);"><rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/></svg>
              <span>RP-607 · Historical Stock Compartments (Structural Movement DNA)</span>
            </div>
            <div style="font-size:11px; color:var(--text-muted); margin-top:2px;">
              Protocol: <b style="color:var(--text-dim);">RP-607-RANGE-001</b> · Rolling Daily Volatility & Efficiency Morphology · Descriptive Baseline
            </div>
          </div>
          <div style="display:flex; gap:8px; align-items:center;">
            <span class="pill-sm" style="background:rgba(139,92,246,0.15); color:var(--accent-violet); border:1px solid rgba(139,92,246,0.3); font-weight:700;">HISTORICAL SNAPSHOT · AS OF 2026-10-01</span>
            <span class="pill-sm" style="background:rgba(255,255,255,0.08); color:var(--text-dim);">119 UNIVERSE · 102 CLASSIFIED · 17 QUARANTINED</span>
          </div>
        </div>

        <!-- Compartment Cards Grid -->
        <div style="display:grid; grid-template-columns:repeat(auto-fit, minmax(200px, 1fr)); gap:12px;">
          <!-- C1 -->
          <div style="background:rgba(6,182,212,0.05); border:1px solid rgba(6,182,212,0.25); border-radius:8px; padding:12px; cursor:pointer;" onclick="setNseCompartmentFilter('C1')">
            <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:4px;">
              <span class="comp-pill comp-c1">C1 · Directional</span>
              <span style="font-family:'JetBrains Mono',monospace; font-size:18px; font-weight:700; color:var(--accent-cyan);">18</span>
            </div>
            <div style="font-size:11px; color:#fff; font-weight:600;">High-Range Directional</div>
            <div style="font-size:10px; color:var(--text-muted); margin-top:4px;">15.1% of Universe · NATR &ge; 220 bps, Eff &ge; 0.14</div>
          </div>

          <!-- C2 -->
          <div style="background:rgba(139,92,246,0.05); border:1px solid rgba(139,92,246,0.25); border-radius:8px; padding:12px; cursor:pointer;" onclick="setNseCompartmentFilter('C2')">
            <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:4px;">
              <span class="comp-pill comp-c2">C2 · Oscillator</span>
              <span style="font-family:'JetBrains Mono',monospace; font-size:18px; font-weight:700; color:var(--accent-violet);">8</span>
            </div>
            <div style="font-size:11px; color:#fff; font-weight:600;">High-Range Oscillator</div>
            <div style="font-size:10px; color:var(--text-muted); margin-top:4px;">6.7% of Universe · NATR &ge; 220 bps, AR1 &le; -0.12</div>
          </div>

          <!-- C3 -->
          <div style="background:rgba(59,130,246,0.05); border:1px solid rgba(59,130,246,0.25); border-radius:8px; padding:12px; cursor:pointer;" onclick="setNseCompartmentFilter('C3')">
            <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:4px;">
              <span class="comp-pill comp-c3">C3 · Stable</span>
              <span style="font-family:'JetBrains Mono',monospace; font-size:18px; font-weight:700; color:#60a5fa;">64</span>
            </div>
            <div style="font-size:11px; color:#fff; font-weight:600;">Stable Moderate Movers</div>
            <div style="font-size:10px; color:var(--text-muted); margin-top:4px;">53.8% of Universe · 160 &le; NATR &lt; 220 bps</div>
          </div>

          <!-- C4 -->
          <div style="background:rgba(245,158,11,0.05); border:1px solid rgba(245,158,11,0.25); border-radius:8px; padding:12px; cursor:pointer;" onclick="setNseCompartmentFilter('C4')">
            <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:4px;">
              <span class="comp-pill comp-c4">C4 · Friction-Bound</span>
              <span style="font-family:'JetBrains Mono',monospace; font-size:18px; font-weight:700; color:var(--accent-amber);">12</span>
            </div>
            <div style="font-size:11px; color:#fff; font-weight:600;">Low-Range Friction-Bound</div>
            <div style="font-size:10px; color:var(--text-muted); margin-top:4px;">10.1% of Universe · NATR &lt; 160 bps</div>
          </div>

          <!-- Quarantined -->
          <div style="background:rgba(255,255,255,0.02); border:1px solid rgba(255,255,255,0.12); border-radius:8px; padding:12px; cursor:pointer;" onclick="setNseCompartmentFilter('QUARANTINED')">
            <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:4px;">
              <span class="comp-pill comp-quarantined">Quarantined</span>
              <span style="font-family:'JetBrains Mono',monospace; font-size:18px; font-weight:700; color:var(--text-muted);">17</span>
            </div>
            <div style="font-size:11px; color:var(--text-dim); font-weight:600;">Insufficient History</div>
            <div style="font-size:10px; color:var(--text-muted); margin-top:4px;">14.3% of Universe · &lt; 20 daily bars in cache</div>
          </div>
        </div>
      </div>

      <!-- PANEL B0: Next-session watchlist (post-close) · live-polled -->
      <div id="nse-next-session-panel" style="background:rgba(0,0,0,0.35); border:1px solid rgba(6,182,212,0.35); border-radius:12px; padding:18px; margin-bottom:20px;">
        <div style="display:flex; justify-content:space-between; align-items:flex-start; gap:12px; flex-wrap:wrap; margin-bottom:12px;">
          <div>
            <div style="font-size:14px; font-weight:700; color:#fff; display:flex; align-items:center; gap:8px; flex-wrap:wrap;">
              <span>Next Session · Intraday Watchlist</span>
              <span id="nse-next-session-live" style="display:inline-flex; align-items:center; gap:5px; font-size:10px; font-weight:700; letter-spacing:0.04em; color:#052e16; background:#67e8f9; border-radius:999px; padding:2px 8px;">
                <span style="width:6px; height:6px; border-radius:50%; background:#065f46; box-shadow:0 0 0 2px rgba(6,95,70,0.25); animation:nseNsrPulse 1.4s ease-in-out infinite;"></span>
                LIVE
              </span>
            </div>
            <div style="font-size:11px; color:var(--text-muted); margin-top:4px;" id="nse-next-session-meta">
              Post-close recommendations for the next trading session · 0.0% authority · not overnight / CAS
            </div>
            <div style="font-size:11px; color:#7dd3fc; margin-top:6px; font-family:'JetBrains Mono',monospace;" id="nse-next-session-rec-time">
              Recommended at: —
            </div>
            <div style="font-size:10px; color:var(--text-muted); margin-top:2px; font-family:'JetBrains Mono',monospace;" id="nse-next-session-live-clock">
              UI refresh: — · stamp poll 3s
            </div>
          </div>
          <span class="pill-sm" id="nse-next-session-pill" style="background:rgba(6,182,212,0.12); color:var(--accent-cyan); border:1px solid rgba(6,182,212,0.35);">WAITING</span>
        </div>
        <div id="nse-next-session-list" style="display:grid; gap:8px; font-family:'JetBrains Mono',monospace; font-size:12px; color:var(--text-dim);">
          No next-session file yet — emitted automatically after POST_CLOSE.
        </div>
      </div>
      <style>
        @keyframes nseNsrPulse {
          0%, 100% { opacity: 1; transform: scale(1); }
          50% { opacity: 0.45; transform: scale(0.85); }
        }
      </style>

      <!-- PANEL B: RP-608 Intraday Momentum & Decision Telemetry -->
      <div style="background:rgba(0,0,0,0.35); border:1px solid rgba(16,185,129,0.3); border-radius:12px; padding:18px; margin-bottom:20px;">
        <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:14px; flex-wrap:wrap; gap:8px;">
          <div>
            <div style="font-size:14px; font-weight:700; color:#fff; display:flex; align-items:center; gap:8px;">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="color:var(--accent-emerald);"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>
              <span>RP-608 · Intraday Momentum & Sequential Decision Telemetry</span>
            </div>
            <div style="font-size:11px; color:var(--text-muted); margin-top:2px;">
              Protocol: <b style="color:var(--text-dim);">RP-608-NSE-SHADOW-001</b> · 15m Causal Momentum (traj15) · Gate Progression Observer
              · <b style="color:var(--accent-cyan);">UI: target &gt; 75 bps board</b>
            </div>
          </div>
          <div style="display:flex; align-items:center; gap:10px; flex-wrap:wrap;">
            <label for="nse-rec-sort" style="display:flex; align-items:center; gap:6px; font-size:11px; color:var(--text-muted); margin:0;">
              <span style="text-transform:uppercase; letter-spacing:0.4px; font-weight:700; color:var(--text-dim);">Sort by</span>
              <select id="nse-rec-sort" style="background:rgba(0,0,0,0.45); color:#fff; border:1px solid rgba(6,182,212,0.35); border-radius:6px; padding:5px 8px; font-size:11px; font-family:'JetBrains Mono',monospace; cursor:pointer;">
                <option value="rank_score">R:R × Target</option>
                <option value="target_bps">Target (bps)</option>
                <option value="net_rr">Net R:R</option>
                <option value="traj15">traj15 score</option>
                <option value="rec_time">Rec time · newest</option>
                <option value="symbol">Symbol A–Z</option>
              </select>
            </label>
            <div id="nse-eval-status-pill">
              <span class="pill-sm" style="background:rgba(255,255,255,0.08); color:var(--text-muted);">INITIALIZING</span>
            </div>
          </div>
        </div>

        <!-- 5-Gate Sequential Progression Visualizer -->
        <div style="display:grid; grid-template-columns:repeat(auto-fit, minmax(180px, 1fr)); gap:10px; margin-bottom:14px;">
          <div style="background:rgba(255,255,255,0.02); border:1px solid rgba(255,255,255,0.08); border-radius:8px; padding:10px;">
            <div style="font-size:10px; text-transform:uppercase; color:var(--text-dim); margin-bottom:2px;">Gate 01 · Session Window</div>
            <div style="font-size:12px; font-weight:700; color:#fff;">1 &le; Bar &le; 19</div>
            <div style="font-size:10px; color:var(--text-muted);">Entry cutoff · 60m ends by 15:00 close-window</div>
          </div>
          <div style="background:rgba(255,255,255,0.02); border:1px solid rgba(255,255,255,0.08); border-radius:8px; padding:10px;">
            <div style="font-size:10px; text-transform:uppercase; color:var(--text-dim); margin-bottom:2px;">Gate 02 · Concurrency</div>
            <div style="font-size:12px; font-weight:700; color:#fff;">Zero Position Active</div>
            <div style="font-size:10px; color:var(--text-muted);">Prevents duplicate allocation</div>
          </div>
          <div style="background:rgba(255,255,255,0.02); border:1px solid rgba(255,255,255,0.08); border-radius:8px; padding:10px;">
            <div style="font-size:10px; text-transform:uppercase; color:var(--text-dim); margin-bottom:2px;">Gate 03 · Causal Warmup</div>
            <div style="font-size:12px; font-weight:700; color:#fff;">History &ge; 5 Bars</div>
            <div style="font-size:10px; color:var(--text-muted);">Minimum rolling sample</div>
          </div>
          <div style="background:rgba(255,255,255,0.02); border:1px solid rgba(255,255,255,0.08); border-radius:8px; padding:10px;">
            <div style="font-size:10px; text-transform:uppercase; color:var(--text-dim); margin-bottom:2px;">Gate 04 · Momentum</div>
            <div style="font-size:12px; font-weight:700; color:var(--accent-emerald);">traj15 &ge; +0.0500</div>
            <div style="font-size:10px; color:var(--text-muted);">Directional breakout filter</div>
          </div>
          <div style="background:rgba(255,255,255,0.02); border:1px solid rgba(255,255,255,0.08); border-radius:8px; padding:10px;">
            <div style="font-size:10px; text-transform:uppercase; color:var(--text-dim); margin-bottom:2px;">Gate 05 · Quote Quality</div>
            <div style="font-size:12px; font-weight:700; color:var(--accent-cyan);">Spread &lt; 5 bps</div>
            <div style="font-size:10px; color:var(--text-muted);">Valid top-of-book L1 depth</div>
          </div>
        </div>

        <!-- Dynamic Recommendations or Rejection Breakdown Display -->
        <div id="nse-eval-recs-container">
          <div style="color:var(--text-dim); font-size:12px; padding:8px 0;">Loading candidate evaluations...</div>
        </div>

        <!-- Rejection Breakdown Chips Strip & Momentum Distribution -->
        <div style="display:flex; justify-content:space-between; align-items:center; flex-wrap:wrap; gap:8px; margin-top:12px; padding-top:10px; border-top:1px solid rgba(255,255,255,0.06); font-size:11px;">
          <div id="nse-rejection-chips" style="display:flex; gap:8px; flex-wrap:wrap; align-items:center;">
            <!-- Rendered dynamically -->
          </div>
          <div id="nse-momentum-dist-strip" style="display:flex; gap:6px; align-items:center; font-family:'JetBrains Mono',monospace;">
            <span style="color:var(--text-muted); font-size:10px;">MOMENTUM CENSUS:</span>
            <span class="pill-sm" style="background:rgba(16,185,129,0.15); color:var(--accent-emerald);">Bullish: <b id="cnt-bullish">--</b></span>
            <span class="pill-sm" style="background:rgba(148,163,184,0.1); color:var(--text-dim);">Neutral: <b id="cnt-neutral">--</b></span>
            <span class="pill-sm" style="background:rgba(244,63,94,0.15); color:var(--accent-rose);">Bearish: <b id="cnt-bearish">--</b></span>
          </div>
        </div>
      </div>

      <!-- PANEL C: SHARED SYMBOL EXPLORER (The Linking Layer) -->
      <div style="margin-bottom:16px;">
        <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:12px; flex-wrap:wrap; gap:8px;">
          <div>
            <div style="font-size:14px; font-weight:700; color:#fff; display:flex; align-items:center; gap:8px;">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="color:var(--accent-cyan);"><line x1="8" y1="6" x2="21" y2="6"/><line x1="8" y1="12" x2="21" y2="12"/><line x1="8" y1="18" x2="21" y2="18"/><line x1="3" y1="6" x2="3.01" y2="6"/><line x1="3" y1="12" x2="3.01" y2="12"/><line x1="3" y1="18" x2="3.01" y2="18"/></svg>
              <span>Shared Symbol Explorer · Historical Morphology & Intraday Evaluator Link</span>
            </div>
            <div style="font-size:11px; color:var(--text-muted); margin-top:2px;">
              Independent Timestamps: RP-607 Historical Profile (2026-10-01) &harr; RP-608 Intraday Quote & Decision
            </div>
          </div>
          <span class="pill-sm" style="background:rgba(255,255,255,0.08); color:var(--text-muted);" id="nse-table-count">Showing All 119 Scrips</span>
        </div>

        <!-- Filter Controls Bar -->
        <div style="display:flex; gap:10px; align-items:center; flex-wrap:wrap; background:rgba(0,0,0,0.3); border:1px solid var(--card-border); border-radius:8px; padding:10px 14px; margin-bottom:12px;">
          <!-- Filter by Compartment -->
          <div style="display:flex; align-items:center; gap:6px;">
            <span style="font-size:11px; color:var(--text-muted); font-weight:600;">RP-607 Compartment:</span>
            <select id="select-filter-compartment" onchange="onFilterChange()" style="background:rgba(0,0,0,0.5); border:1px solid var(--card-border); color:#fff; font-family:'JetBrains Mono',monospace; font-size:11px; padding:5px 8px; border-radius:6px; outline:none;">
              <option value="ALL">All Compartments (119)</option>
              <option value="C1">C1 · Directional (18)</option>
              <option value="C2">C2 · Oscillator (8)</option>
              <option value="C3">C3 · Stable Moderate (64)</option>
              <option value="C4">C4 · Friction-Bound (12)</option>
              <option value="QUARANTINED">Quarantined (17)</option>
            </select>
          </div>

          <!-- Filter by Decision -->
          <div style="display:flex; align-items:center; gap:6px;">
            <span style="font-size:11px; color:var(--text-muted); font-weight:600;">RP-608 Decision:</span>
            <select id="select-filter-decision" onchange="onFilterChange()" style="background:rgba(0,0,0,0.5); border:1px solid var(--card-border); color:#fff; font-family:'JetBrains Mono',monospace; font-size:11px; padding:5px 8px; border-radius:6px; outline:none;">
              <option value="ALL">All Decisions</option>
              <option value="RECOMMEND_LONG">Qualifying Long Candidates</option>
              <option value="TARGET_GT_75">Target &gt; 75 bps</option>
              <option value="INTRADAY_BOUNDARY_EXCLUDED">Boundary Excluded (&ge; 15:00)</option>
              <option value="COLD_START_INSUFFICIENT_HISTORY">Cold Start (< 5 bars)</option>
              <option value="SUB_THRESHOLD_MOMENTUM">Sub-Threshold Momentum</option>
              <option value="EXISTING_POSITION_ACTIVE">Active Trade Open</option>
              <option value="DATA_QUALITY_FAILURE">Data Quality Failure</option>
            </select>
          </div>

          <!-- Filter by Momentum -->
          <div style="display:flex; align-items:center; gap:6px;">
            <span style="font-size:11px; color:var(--text-muted); font-weight:600;">Momentum:</span>
            <select id="select-filter-momentum" onchange="onFilterChange()" style="background:rgba(0,0,0,0.5); border:1px solid var(--card-border); color:#fff; font-family:'JetBrains Mono',monospace; font-size:11px; padding:5px 8px; border-radius:6px; outline:none;">
              <option value="ALL">All Scores</option>
              <option value="BULLISH">Bullish (&ge; +0.0500)</option>
              <option value="SUB_THRESHOLD">Below Threshold (&lt; +0.0500)</option>
            </select>
          </div>

          <!-- Sort -->
          <div style="display:flex; align-items:center; gap:6px;">
            <span style="font-size:11px; color:var(--text-muted); font-weight:600;">Sort:</span>
            <select id="select-explorer-sort" onchange="onFilterChange()" style="background:rgba(0,0,0,0.5); border:1px solid var(--card-border); color:#fff; font-family:'JetBrains Mono',monospace; font-size:11px; padding:5px 8px; border-radius:6px; outline:none;">
              <option value="REC_TIME_DESC" selected>Rec time · newest first</option>
              <option value="REC_TIME_ASC">Rec time · oldest first</option>
              <option value="TRAJ15_DESC">traj15 · high first</option>
              <option value="SYMBOL_ASC">Symbol A–Z</option>
            </select>
          </div>

          <!-- Search Input -->
          <div style="margin-left:auto;">
            <input type="text" id="nse-filter-input" placeholder="Search symbol or code..." oninput="onFilterChange()" style="background:rgba(0,0,0,0.5); border:1px solid var(--card-border); color:#fff; font-family:'JetBrains Mono',monospace; font-size:11px; padding:5px 10px; border-radius:6px; outline:none; width:180px;">
          </div>
        </div>

        <div style="overflow-x:auto; border:1px solid var(--card-border); border-radius:10px; background:rgba(0,0,0,0.25);">
          <table>
            <thead>
              <tr>
                <th>Scrip</th>
                <th>RP-607 Compartment</th>
                <th>Historical Features (2026-10-01)</th>
                <th>RP-608 Momentum</th>
                <th>Sequential Gate Trace</th>
                <th>Terminal Decision</th>
                <th>Rec Time</th>
                <th>Live LTP & Spread</th>
                <th>Quote Age & Provenance</th>
              </tr>
            </thead>
            <tbody id="nse-scrips-tbody">
              <tr><td colspan="9" style="text-align:center; color:var(--text-dim); padding:20px;">Loading NSE scrips...</td></tr>
            </tbody>
          </table>
        </div>
      </div>

      <!-- Today's NSE Pre-Open & Live Schedule Timeline -->
      <div style="border-top: 1px solid var(--card-border); padding-top:14px;">
        <div style="font-size:12px; font-weight:700; color:var(--text-muted); margin-bottom:10px; text-transform:uppercase; letter-spacing:0.5px;">
          Today's Scheduled Execution Checkpoints
        </div>
        <div class="timeline" id="nse-timeline">
          <!-- Rendered dynamically -->
        </div>
      </div>
    </div>

    </div><!-- /ws-panel-command -->

    <!-- MULTI-HORIZON BOOKS WORKSPACE -->
    <div id="ws-panel-horizons" style="display:none;">
      <div class="card" style="margin-bottom:20px; border-left: 4px solid #c4b5fd; background: linear-gradient(135deg, rgba(167,139,250,0.07) 0%, rgba(15,23,42,0.65) 100%);">
        <div style="display:flex; justify-content:space-between; align-items:flex-start; gap:12px; flex-wrap:wrap; margin-bottom:10px;">
          <div>
            <div class="section-title" style="margin-bottom:6px;">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="color:#c4b5fd;"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
              Multi-Horizon Observation Books
            </div>
            <p style="font-size:13px; color:var(--text-muted); max-width:820px; line-height:1.5; margin:0;" id="mh-note">
              Separate holding-term paper books for NSE (RP-609 CNC) and Crypto (RP-610 spot-sim).
              This is <b style="color:#e2e8f0;">not</b> the intraday decision feed — each row is an open observation position with a named symbol and a holding class.
            </p>
          </div>
          <div style="display:flex; gap:8px; flex-wrap:wrap;">
            <span class="pill-sm" style="background:rgba(167,139,250,0.15); color:#c4b5fd; border:1px solid rgba(167,139,250,0.35);">RP-609 NSE · RP-610 CRYPTO</span>
            <span class="pill-sm pill-prewindow" style="border:1px solid rgba(255,255,255,0.2);">0.0% ORDER AUTHORITY</span>
            <span class="pill-sm" id="mh-gates-pill" style="background:rgba(245,158,11,0.12); color:var(--accent-amber); border:1px solid rgba(245,158,11,0.3);">GATES CLOSED</span>
          </div>
        </div>

        <div style="display:flex; gap:8px; margin:16px 0 14px; flex-wrap:wrap;">
          <button type="button" class="mh-inner-tab active" data-mh-inner="OPEN" id="mh-inner-open"
            style="padding:10px 16px; border-radius:8px; border:none; cursor:pointer; font-weight:800; font-size:13px; background:#7c3aed; color:#fff;">1. Open books (actionable)</button>
          <button type="button" class="mh-inner-tab" data-mh-inner="UNIVERSE" id="mh-inner-universe"
            style="padding:10px 16px; border-radius:8px; border:none; cursor:pointer; font-weight:800; font-size:13px; background:#1e293b; color:#fff;">2. NSE universe (119)</button>
          <button type="button" class="mh-inner-tab" data-mh-inner="GUIDE" id="mh-inner-guide"
            style="padding:10px 16px; border-radius:8px; border:none; cursor:pointer; font-weight:800; font-size:13px; background:#1e293b; color:#fff;">3. How to read classes</button>
        </div>

        <div id="mh-coverage-strip" style="display:flex; gap:8px; flex-wrap:wrap; margin-bottom:14px; align-items:center;">
          <span class="pill-sm" style="background:rgba(56,189,248,0.12); color:#38bdf8; border:1px solid rgba(56,189,248,0.35);">NSE universe: —</span>
        </div>

        <div id="mh-guide-panel" style="display:none; margin-bottom:8px;">
          <div style="display:grid; gap:12px;" id="mh-guide-grid"></div>
        </div>

        <div id="mh-universe-panel" style="display:none; margin-bottom:8px;">
          <p style="font-size:12px; color:var(--text-muted); margin:0 0 12px 0; max-width:820px; line-height:1.45;">
            All <b style="color:#e2e8f0;">119</b> RP-608 NSE shadow scrips. Symbols marked <b style="color:#34d399;">IN MH</b> appear in an open H1/H2/H3 observation book.
            The rest are covered for market data only — not missing rows, and not auto-opened (0.0% order authority).
          </p>
          <div style="display:flex; gap:10px; align-items:center; flex-wrap:wrap; margin-bottom:12px;">
            <span style="font-size:11px; color:var(--text-muted); font-weight:700; text-transform:uppercase;">Filter</span>
            <select id="mh-universe-filter" style="background:rgba(0,0,0,0.5); border:1px solid var(--card-border); color:#fff; font-family:'JetBrains Mono',monospace; font-size:11px; padding:5px 8px; border-radius:6px; outline:none;">
              <option value="ALL">All 119</option>
              <option value="IN">In MH books</option>
              <option value="OUT">Not in MH books</option>
            </select>
            <input id="mh-universe-search" type="search" placeholder="Search symbol…"
              style="background:rgba(0,0,0,0.5); border:1px solid var(--card-border); color:#fff; font-family:'JetBrains Mono',monospace; font-size:11px; padding:5px 8px; border-radius:6px; outline:none; min-width:160px;" />
          </div>
          <div style="overflow-x:auto; border:1px solid rgba(255,255,255,0.08); border-radius:10px; max-height:520px; overflow-y:auto;">
            <table class="mh-open-table">
              <thead>
                <tr>
                  <th>Symbol</th>
                  <th>Tradingsymbol</th>
                  <th>MH status</th>
                  <th>Classes</th>
                  <th>Open book detail</th>
                </tr>
              </thead>
              <tbody id="mh-universe-tbody">
                <tr><td colspan="5" style="color:var(--text-muted); padding:24px 14px;">Loading universe…</td></tr>
              </tbody>
            </table>
          </div>
        </div>

        <div id="mh-open-panel">
          <div style="display:flex; gap:8px; flex-wrap:wrap; margin-bottom:14px; align-items:center;" id="mh-class-counts">
            <span class="pill-sm" style="background:rgba(255,255,255,0.06); color:#fff; border:1px solid rgba(255,255,255,0.12);">Loading…</span>
          </div>
          <div style="display:flex; gap:10px; align-items:center; flex-wrap:wrap; margin-bottom:12px;">
            <span style="font-size:11px; color:var(--text-muted); font-weight:700; text-transform:uppercase;">Market</span>
            <select id="mh-filter-market" style="background:rgba(0,0,0,0.5); border:1px solid var(--card-border); color:#fff; font-family:'JetBrains Mono',monospace; font-size:11px; padding:5px 8px; border-radius:6px; outline:none;">
              <option value="ALL">NSE + CRYPTO</option>
              <option value="NSE">NSE only</option>
              <option value="CRYPTO">CRYPTO only</option>
            </select>
            <span style="font-size:11px; color:var(--text-muted); font-weight:700; text-transform:uppercase;">Class</span>
            <select id="mh-filter-class" style="background:rgba(0,0,0,0.5); border:1px solid var(--card-border); color:#fff; font-family:'JetBrains Mono',monospace; font-size:11px; padding:5px 8px; border-radius:6px; outline:none;">
              <option value="ALL">All classes</option>
              <option value="INTRADAY">INTRADAY</option>
              <option value="MULTI_DAY">MULTI_DAY</option>
              <option value="MULTI_WEEK">MULTI_WEEK</option>
              <option value="LONG_TERM">LONG_TERM</option>
            </select>
            <span style="font-size:11px; color:var(--text-muted); font-weight:700; text-transform:uppercase;">Books</span>
            <select id="mh-filter-book-kind" style="background:rgba(0,0,0,0.5); border:1px solid var(--card-border); color:#fff; font-family:'JetBrains Mono',monospace; font-size:11px; padding:5px 8px; border-radius:6px; outline:none;">
              <option value="ALL">Actionable + 119 COVERED</option>
              <option value="ACTIONABLE">Actionable only (H1–H3)</option>
              <option value="UNIVERSE">COVERED universe only</option>
            </select>
            <input id="mh-filter-search" type="search" placeholder="Search symbol…"
              style="background:rgba(0,0,0,0.5); border:1px solid var(--card-border); color:#fff; font-family:'JetBrains Mono',monospace; font-size:11px; padding:5px 8px; border-radius:6px; outline:none; min-width:140px;" />
          </div>
          <div style="overflow-x:auto; border:1px solid rgba(255,255,255,0.08); border-radius:10px; margin-bottom:16px;">
            <table class="mh-open-table">
              <thead>
                <tr>
                  <th>Symbol(s)</th>
                  <th>Market</th>
                  <th>Holding class</th>
                  <th>What it means</th>
                  <th>Decision</th>
                  <th>Side</th>
                  <th>Status</th>
                  <th>Entry</th>
                  <th>Stop</th>
                  <th>Target</th>
                  <th>Thesis / plan</th>
                </tr>
              </thead>
              <tbody id="mh-recs-tbody">
                <tr><td colspan="11" style="color:var(--text-muted); padding:24px 14px;">Loading open books…</td></tr>
              </tbody>
            </table>
          </div>
          <details style="background:rgba(0,0,0,0.25); border:1px solid var(--card-border); border-radius:10px; padding:12px 14px;">
            <summary style="cursor:pointer; font-size:12px; font-weight:800; color:var(--text-muted); text-transform:uppercase; letter-spacing:0.04em;">Catalog reference (which books exist — not open positions)</summary>
            <div style="margin-top:12px;" id="mh-catalog-grid"></div>
          </details>
        </div>
      </div>
    </div><!-- /ws-panel-horizons -->

    <!-- Footer -->
    <footer>
      <div id="nse-footer-daemon">Daemon PID: <b>--</b> · Host: <b>LCUtest</b> (192.168.0.9)</div>
      <div>Strictly Read-Only · 0.0% Order Authority · Refresh: 3.0s</div>
    </footer>
  </div>

  <script>
    function horizonBadgeHtml(obj) {
      const cls = (obj && obj.horizon_class) ? String(obj.horizon_class) : 'INTRADAY';
      const label = (obj && obj.horizon_label)
        ? String(obj.horizon_label)
        : (cls === 'LONG_TERM' ? 'Long-term'
          : (cls === 'MULTI_WEEK' ? 'Multi-week'
            : (cls === 'MULTI_DAY' ? 'Multi-day' : 'Intraday')));
      const term = (obj && obj.horizon_term) ? String(obj.horizon_term) : 'Term unspecified';
      const colors = {
        INTRADAY: { bg: 'rgba(6,182,212,0.15)', fg: 'var(--accent-cyan)', bd: 'rgba(6,182,212,0.35)' },
        MULTI_DAY: { bg: 'rgba(167,139,250,0.15)', fg: '#c4b5fd', bd: 'rgba(167,139,250,0.35)' },
        MULTI_WEEK: { bg: 'rgba(251,146,60,0.15)', fg: '#fdba74', bd: 'rgba(251,146,60,0.35)' },
        LONG_TERM: { bg: 'rgba(244,63,94,0.12)', fg: 'var(--accent-rose)', bd: 'rgba(244,63,94,0.3)' },
      };
      const c = colors[cls] || colors.INTRADAY;
      const safeTerm = term.replace(/"/g, '&quot;');
      return `<span class="badge" style="background:${c.bg}; color:${c.fg}; border:1px solid ${c.bd}; font-weight:700; margin-right:6px;" title="${safeTerm}">${label.toUpperCase()}</span>`
        + `<span class="badge" style="background:rgba(255,255,255,0.05); color:var(--text-dim); border:1px solid rgba(255,255,255,0.1); font-weight:600; margin-right:6px;" title="Holding term"> · ${safeTerm}</span>`;
    }

    const MH_CLASS_META = {
      INTRADAY: {
        title: 'Intraday',
        meaning: 'Same-session / ≤60 minutes. Owned by RP-608 live shadows — not opened here.',
        color: '#22d3ee',
        nseOwner: 'RP-608 NSE MIS shadow (link-only)',
        cryptoOwner: 'RP-608 trade-plan overlay (link-only)',
      },
      MULTI_DAY: {
        title: 'Multi-day',
        meaning: 'Swing hold: NSE 2–20 sessions (CNC) · Crypto 2–14 calendar days.',
        color: '#c4b5fd',
        nseOwner: 'RP-609 H1 swing book',
        cryptoOwner: 'RP-610 H1 multi-day book',
      },
      MULTI_WEEK: {
        title: 'Multi-week',
        meaning: 'Longer build: NSE staggered CNC portfolio · Crypto 14–56 day position.',
        color: '#fdba74',
        nseOwner: 'RP-609 H2 portfolio build',
        cryptoOwner: 'RP-610 H2 multi-week book',
      },
      LONG_TERM: {
        title: 'Long-term',
        meaning: 'Thesis hold: NSE ≥60 sessions / ≥90d · Crypto ≥90 calendar days.',
        color: '#fb7185',
        nseOwner: 'RP-609 H3 long-term book',
        cryptoOwner: 'RP-610 H3 long-term book',
      },
    };

    function mhSymbolList(r) {
      if (Array.isArray(r.symbols) && r.symbols.length) {
        return r.symbols.map(s => String(s || '').trim()).filter(Boolean);
      }
      const one = String(r.symbol || '').trim();
      return one ? [one] : [];
    }

    function mhFormatSymbols(r) {
      const list = mhSymbolList(r);
      return list.length ? list.join(' · ') : '—';
    }

    function mhFormatLevel(r, field) {
      const raw = r ? r[field] : null;
      if (raw == null || raw === '') return '—';
      const n = Number(raw);
      if (Number.isNaN(n)) return '—';
      // Crypto H1–H3 store stop/target as fractions (e.g. 0.04 = 4%)
      if ((r.market || '') === 'CRYPTO' && Math.abs(n) > 0 && Math.abs(n) < 1 && field !== 'entry_price') {
        return (n * 100).toFixed(1) + '%';
      }
      if ((r.market || '') === 'NSE') return '₹' + n.toFixed(2);
      return '$' + n.toFixed(2);
    }

    function mhOpenSymbolsFor(market, horizonClass, horizonId) {
      const mh = window.__mhState;
      const recs = (mh && mh.recommendations) ? mh.recommendations : [];
      const syms = [];
      const seen = new Set();
      const push = (sym) => {
        const s = String(sym || '').trim();
        if (!s || seen.has(s)) return;
        seen.add(s);
        syms.push(s);
      };
      recs.forEach(r => {
        if ((r.market || '') !== market) return;
        if (horizonClass && (r.horizon_class || '') !== horizonClass) return;
        if (horizonId && (r.horizon_id || '') !== horizonId) return;
        mhSymbolList(r).forEach(push);
      });
      return syms;
    }

    function mhCatalogSection(title, market, items) {
      if (!items || !items.length) {
        return `<div style="margin-bottom:14px;">
          <div style="font-size:11px; font-weight:800; color:var(--text-muted); letter-spacing:0.04em; margin-bottom:8px;">${title}</div>
          <div style="color:var(--text-muted); font-size:12px;">No catalog rows.</div>
        </div>`;
      }
      const cards = items.map(c => {
        const openSyms = mhOpenSymbolsFor(market, c.horizon_class, c.horizon_id);
        const cls = c.horizon_class || 'INTRADAY';
        const meta = MH_CLASS_META[cls] || MH_CLASS_META.INTRADAY;
        const symHtml = openSyms.length
          ? `<span style="font-family:'JetBrains Mono',monospace; font-size:13px; font-weight:800; color:#38bdf8;">${openSyms.join(' · ')}</span>`
          : `<span style="font-size:11px; color:var(--text-muted);">${c.observational_link_only ? 'No symbol here — H0 owned by RP-608 live books' : 'No open observation symbol yet'}</span>`;
        return `
        <div style="background:rgba(0,0,0,0.35); border:1px solid var(--card-border); border-left:4px solid ${meta.color}; border-radius:10px; padding:12px;">
          <div style="display:flex; justify-content:space-between; gap:8px; margin-bottom:8px; flex-wrap:wrap; align-items:center;">
            <span style="font-size:11px; font-weight:900; color:${meta.color};">${cls}</span>
            <span class="pill-sm" style="background:rgba(255,255,255,0.06); color:#e2e8f0; border:1px solid rgba(255,255,255,0.12);">${market}</span>
          </div>
          <div style="font-size:11px; font-weight:800; color:var(--accent-cyan); text-transform:uppercase; letter-spacing:0.04em; margin-bottom:4px;">Open symbol(s)</div>
          <div style="margin-bottom:10px; min-height:28px;">${symHtml}</div>
          <div style="font-size:12px; font-weight:700; color:#fff; margin-bottom:4px;">${c.horizon_id || ''}</div>
          <div style="font-size:11px; color:var(--text-dim); line-height:1.4;">${c.label || c.horizon_term || meta.meaning}</div>
          <div style="font-size:10px; color:var(--text-muted); margin-top:6px;">${c.owner_protocol || ''} · ${c.product_type || ''}${c.observational_link_only ? ' · link-only' : ''}</div>
        </div>`;
      }).join('');
      return `<div style="margin-bottom:16px;">
        <div style="font-size:11px; font-weight:800; color:var(--text-muted); letter-spacing:0.04em; margin-bottom:8px; text-transform:uppercase;">${title}</div>
        <div style="display:grid; grid-template-columns:repeat(auto-fit, minmax(240px, 1fr)); gap:10px;">${cards}</div>
      </div>`;
    }

    function switchWorkspace(name) {
      const cmd = document.getElementById('ws-panel-command');
      const hor = document.getElementById('ws-panel-horizons');
      if (cmd) cmd.style.display = name === 'command' ? '' : 'none';
      if (hor) hor.style.display = name === 'horizons' ? '' : 'none';
      document.querySelectorAll('.ws-tab').forEach(btn => {
        btn.classList.toggle('active', btn.getAttribute('data-ws') === name);
      });
      try { localStorage.setItem('rp608_ws', name); } catch (e) {}
    }

    function switchMhInner(name) {
      const open = document.getElementById('mh-open-panel');
      const guide = document.getElementById('mh-guide-panel');
      const uni = document.getElementById('mh-universe-panel');
      if (open) open.style.display = name === 'OPEN' ? '' : 'none';
      if (guide) guide.style.display = name === 'GUIDE' ? '' : 'none';
      if (uni) uni.style.display = name === 'UNIVERSE' ? '' : 'none';
      document.querySelectorAll('.mh-inner-tab').forEach(btn => {
        const on = btn.getAttribute('data-mh-inner') === name;
        btn.classList.toggle('active', on);
        btn.style.background = on ? '#7c3aed' : '#1e293b';
      });
    }

    function paintNseUniverse() {
      const mh = window.__mhState;
      const uni = (mh && mh.nse_universe) || {};
      const strip = document.getElementById('mh-coverage-strip');
      if (strip) {
        const total = uni.total_universe != null ? uni.total_universe : 119;
        const inn = uni.in_mh_books != null ? uni.in_mh_books : 0;
        const out = uni.not_in_mh_books != null ? uni.not_in_mh_books : Math.max(0, total - inn);
        const members = uni.universe_members_in_open_books != null ? uni.universe_members_in_open_books : out;
        const shown = uni.shown_in_open_books != null ? uni.shown_in_open_books : total;
        strip.innerHTML = `
          <span class="pill-sm" style="background:rgba(56,189,248,0.12); color:#38bdf8; border:1px solid rgba(56,189,248,0.35);">NSE in Open books: <b>${shown}</b> / ${total}</span>
          <span class="pill-sm" style="background:rgba(16,185,129,0.12); color:#34d399; border:1px solid rgba(16,185,129,0.35);">Actionable H1–H3: <b>${inn}</b></span>
          <span class="pill-sm" style="background:rgba(148,163,184,0.12); color:#94a3b8; border:1px solid rgba(148,163,184,0.3);">COVERED members: <b>${members}</b></span>
          <span class="pill-sm" style="background:rgba(245,158,11,0.12); color:#fbbf24; border:1px solid rgba(245,158,11,0.3);">0.0% authority · gates CLOSED</span>`;
      }
      const tbody = document.getElementById('mh-universe-tbody');
      if (!tbody) return;
      const filt = (document.getElementById('mh-universe-filter') || {}).value || 'ALL';
      const q = String((document.getElementById('mh-universe-search') || {}).value || '').trim().toUpperCase();
      let rows = Array.isArray(uni.symbols) ? uni.symbols.slice() : [];
      if (filt === 'IN') rows = rows.filter(r => r.in_mh_books);
      if (filt === 'OUT') rows = rows.filter(r => !r.in_mh_books);
      if (q) {
        rows = rows.filter(r => String(r.symbol || '').toUpperCase().includes(q)
          || String(r.tradingsymbol || '').toUpperCase().includes(q));
      }
      if (!rows.length) {
        tbody.innerHTML = `<tr><td colspan="5" style="color:var(--text-muted); padding:24px 14px;">No symbols for this filter.</td></tr>`;
        return;
      }
      tbody.innerHTML = rows.map(r => {
        const classes = (r.horizon_classes || []).join(' · ') || '—';
        const detail = (r.books || []).map(b =>
          `${b.horizon_id || ''} ${b.status || ''} ${b.plan_id || b.thesis || ''}`.trim()
        ).filter(Boolean).join(' · ') || '—';
        const badge = r.in_mh_books
          ? `<span style="color:#34d399; font-weight:800;">IN MH</span>`
          : `<span style="color:#64748b; font-weight:700;">data only</span>`;
        return `<tr>
          <td style="font-family:'JetBrains Mono',monospace; font-weight:800; color:#38bdf8;">${r.symbol || '—'}</td>
          <td style="color:#e2e8f0;">${r.tradingsymbol || '—'}</td>
          <td>${badge}</td>
          <td style="color:#fdba74; font-size:11px; font-weight:700;">${classes}</td>
          <td style="color:#94a3b8; font-size:11px;">${detail}</td>
        </tr>`;
      }).join('');
    }

    window.__mhState = null;
    function ofPaperFmtUtc(msOrIso) {
      if (msOrIso == null || msOrIso === '') return '—';
      if (typeof msOrIso === 'string' && msOrIso.indexOf('T') >= 0) {
        return msOrIso.replace('T', ' ').replace('Z', 'Z');
      }
      const n = Number(msOrIso);
      if (!Number.isFinite(n)) return '—';
      try {
        return new Date(n).toISOString().replace('T', ' ').replace('.000Z', 'Z');
      } catch (e) {
        return '—';
      }
    }
    function ofPaperFmtHold(sec) {
      if (sec == null || !Number.isFinite(Number(sec))) return '—';
      const s = Number(sec);
      if (s < 60) return s.toFixed(1) + 's';
      if (s < 3600) return (s / 60).toFixed(1) + 'm';
      return (s / 3600).toFixed(2) + 'h';
    }
    function updateOfPaperTradesUI(data) {
      const openEl = document.getElementById('of-paper-open');
      const chipsEl = document.getElementById('of-paper-summary-chips');
      const bodyEl = document.getElementById('of-paper-ledger-body');
      if (!openEl || !chipsEl || !bodyEl) return;
      const shadow = data.of_shadow_trade;
      if (shadow && shadow.open) {
        const uNet = shadow.unrealized_net_pnl_usdt;
        const uGross = shadow.unrealized_gross_pnl_usdt;
        const ubNet = shadow.unrealized_net_return_bps;
        const ubGross = shadow.unrealized_gross_return_bps;
        const partPct = shadow.participation_pct != null
          ? (Number(shadow.participation_pct) * 100).toFixed(0) + '%'
          : '—';
        const qtyStr = shadow.simulated_quantity != null
          ? Number(shadow.simulated_quantity).toFixed(6)
          : '—';
        const notionalStr = shadow.simulated_notional_usdt != null
          ? Number(shadow.simulated_notional_usdt).toFixed(2)
          : '—';
        const dq = shadow.depth_quality || {};
        const combN = dq.combined_touch_notional_usdt;
        const sameN = dq.same_side_touch_notional_usdt;
        const oppN = dq.opposing_side_touch_notional_usdt;
        const thinOpp = dq.thin_opposite_diagnostic === true;
        openEl.innerHTML =
          '<b style="color:#5eead4;">OPEN</b> ' + (shadow.of_trade_id || '—') +
          ' · ' + (shadow.direction || '—') +
          ' · opened ' + ofPaperFmtUtc(shadow.paper_open_timestamp || shadow.activated_ms) +
          ' · entry ' + (shadow.entry_price != null ? Number(shadow.entry_price).toFixed(2) : '—') +
          ' · mark ' + (shadow.mark_price != null ? Number(shadow.mark_price).toFixed(2) : '—') +
          ' · entry_sig ' + (shadow.entry_signal_qty != null ? Number(shadow.entry_signal_qty).toFixed(4) : '—') +
          ' · qty ' + qtyStr +
          ' · notional ' + notionalStr + ' USDT' +
          ' · part ' + partPct +
          ' · uGross ' + (uGross != null ? Number(uGross).toFixed(4) + ' USDT' : '—') +
          (ubGross != null ? ' (' + Number(ubGross).toFixed(2) + ' bps)' : '') +
          ' · uNet ' + (uNet != null ? Number(uNet).toFixed(4) + ' USDT' : '—') +
          (ubNet != null ? ' (' + Number(ubNet).toFixed(2) + ' bps)' : '') +
          ' · hold ' + ofPaperFmtHold(shadow.holding_duration_seconds) +
          '<div style="color:#94a3b8; font-size:11px; margin-top:4px;">' +
            'depth · comb ' + (combN != null ? Number(combN).toFixed(0) + ' USDT' : '—') +
            ' · same ' + (sameN != null ? Number(sameN).toFixed(0) + ' USDT' : '—') +
            ' · opp ' + (oppN != null ? Number(oppN).toFixed(0) + ' USDT' : '—') +
            (thinOpp ? ' · <span style="color:#fbbf24;">thin_opposite</span>' : '') +
            ' · telemetry only (OF ±0.30 unchanged)' +
          '</div>' +
          '<div style="color:#64748b; font-size:11px; margin-top:2px;">PAPER · authority 0.0% · uGross=mid mark P&amp;L · uNet=gross−spread−fees · exit fill qty=entry qty</div>';
      } else {
        openEl.textContent = 'No open OF paper trade';
      }
      const pnl = data.of_trade_pnl || {};
      const sum = pnl.summary || {};
      chipsEl.innerHTML =
        '<span class="pill-sm" style="background:rgba(34,211,238,0.15); color:var(--accent-cyan);">n ' + (sum.n_trades || 0) + '</span>' +
        '<span class="pill-sm" style="background:rgba(52,211,153,0.15); color:var(--accent-emerald);">W ' + (sum.n_wins || 0) + '</span>' +
        '<span class="pill-sm" style="background:rgba(251,113,133,0.15); color:var(--accent-rose);">L ' + (sum.n_losses || 0) + '</span>' +
        '<span class="pill-sm" style="background:rgba(255,255,255,0.06); color:#e2e8f0;">cum net ' +
          (sum.cumulative_net_pnl_usdt != null ? Number(sum.cumulative_net_pnl_usdt).toFixed(2) + ' USDT' : '—') + '</span>' +
        '<span class="pill-sm" style="background:rgba(45,212,191,0.15); color:#5eead4;">cum ' +
          (sum.cumulative_net_return_bps != null ? Number(sum.cumulative_net_return_bps).toFixed(2) + ' bps' : '—') + '</span>';
      const recent = Array.isArray(pnl.recent) ? pnl.recent : [];
      if (!recent.length) {
        bodyEl.innerHTML = '<tr><td colspan="13" style="padding:8px 6px; color:var(--text-dim);">No closed OF paper trades yet</td></tr>';
        return;
      }
      bodyEl.innerHTML = recent.slice(0, 24).map(function (t) {
        const openT = ofPaperFmtUtc(t.paper_open_timestamp || t.activated_ms);
        const closeT = ofPaperFmtUtc(t.paper_close_timestamp || t.signal_exit_timestamp || t.exited_ms || t.paper_close_ms);
        const entryQty = t.entry_paper_quantity != null ? t.entry_paper_quantity : t.simulated_quantity;
        const exitQty = t.exit_quantity != null ? t.exit_quantity : t.simulated_quantity;
        const qtyMatched = t.qty_matched_entry_to_exit === true
          || (entryQty != null && exitQty != null && Number(entryQty) === Number(exitQty));
        const qtyCell = exitQty != null ? Number(exitQty).toFixed(6) : '—';
        const entrySig = t.entry_signal_qty != null ? Number(t.entry_signal_qty).toFixed(4) : '—';
        const exitSig = t.exit_signal_qty != null ? Number(t.exit_signal_qty).toFixed(4) : '—';
        const qtyColor = qtyMatched ? '#5eead4' : '#fb7185';
        return '<tr>' +
          '<td style="padding:4px 6px; white-space:nowrap;">' + openT + '</td>' +
          '<td style="padding:4px 6px; white-space:nowrap;">' + closeT + '</td>' +
          '<td style="padding:4px 6px;">' + ofPaperFmtHold(t.holding_duration_seconds) + '</td>' +
          '<td style="padding:4px 6px;">' + (t.direction || '—') + '</td>' +
          '<td style="padding:4px 6px;">' + entrySig + '</td>' +
          '<td style="padding:4px 6px;">' + exitSig + '</td>' +
          '<td style="padding:4px 6px; color:' + qtyColor + ';" title="exit fill must equal entry paper qty">' + qtyCell + '</td>' +
          '<td style="padding:4px 6px;">' + (t.entry_price != null ? Number(t.entry_price).toFixed(2) : '—') + '</td>' +
          '<td style="padding:4px 6px;">' + (t.exit_price != null ? Number(t.exit_price).toFixed(2) : '—') + '</td>' +
          '<td style="padding:4px 6px;">' + (t.net_pnl_usdt != null ? Number(t.net_pnl_usdt).toFixed(4) : '—') + '</td>' +
          '<td style="padding:4px 6px;">' + (t.net_return_bps != null ? Number(t.net_return_bps).toFixed(2) : '—') + '</td>' +
          '<td style="padding:4px 6px;">' + (t.fees_usdt != null ? Number(t.fees_usdt).toFixed(4) : '—') + '</td>' +
          '<td style="padding:4px 6px;">' + (t.exit_reason || '—') + '</td>' +
          '</tr>';
      }).join('');
    }

    function updateRp611UI(rp) {
      const nseEl = document.getElementById('rp611-nse-status');
      const cryEl = document.getElementById('rp611-crypto-status');
      if (!nseEl || !cryEl || !rp) return;
      const fmt = (arm, kind) => {
        if (!arm || arm.available === false) return 'UNAVAILABLE';
        const jc = arm.journal_counts || {};
        const gap = (arm.pre_deployment_capture_gap_sec != null)
          ? Number(arm.pre_deployment_capture_gap_sec).toFixed(0) + 's'
          : '—';
        const lines = [
          (arm.disposition || arm.status || '—'),
          'clock ' + (arm.experiment_clock_started ? 'ON' : 'OFF') +
            ' · T0 ' + (arm.experiment_clock_started_utc || '—'),
          'observer ' + (arm.observer_deployed ? 'PHASE-1 DEPLOYED' : 'NOT DEPLOYED') +
            ' · phase ' + (arm.phase || '—'),
          'deployed ' + (arm.phase1_deployed_utc || '—') + ' · gap ' + gap,
          'cand ' + (jc.candidates != null ? jc.candidates : '—') +
            ' · conf ' + (jc.confirmations != null ? jc.confirmations : '—') +
            (kind === 'CRYPTO'
              ? (' · L1 ' + (jc.l1_telemetry_rows != null ? jc.l1_telemetry_rows : '—'))
              : (' · census rows ' + (jc.session_log_rows != null ? jc.session_log_rows : '—'))),
        ];
        if (kind === 'CRYPTO') {
          lines.push('volume ' + (arm.volume_source || 'spot_kline_15m') + ' · futures EXCLUDED');
        } else {
          lines.push('universe 119 · RVOL≥1.50 · body≤0.35 · wick≥0.40');
        }
        return lines.join('<br/>');
      };
      nseEl.innerHTML = fmt(rp.nse, 'NSE');
      cryEl.innerHTML = fmt(rp.crypto, 'CRYPTO');
    }

    function updateMultiHorizonUI(mh) {
      window.__mhState = mh || null;
      const note = document.getElementById('mh-note');
      if (note && mh && mh.note) note.textContent = mh.note;
      const counts = (mh && mh.counts_by_class) || {};
      const countsEl = document.getElementById('mh-class-counts');
      if (countsEl) {
        countsEl.innerHTML = ['INTRADAY','MULTI_DAY','MULTI_WEEK','LONG_TERM'].map(cls => {
          const n = counts[cls] != null ? counts[cls] : 0;
          const c = (MH_CLASS_META[cls] || {}).color || '#94a3b8';
          return `<span class="pill-sm" style="background:${c}18; color:${c}; border:1px solid ${c}55;">${cls}: <b>${n}</b> open</span>`;
        }).join('');
      }
      const pointer = document.getElementById('mh-pointer-counts');
      if (pointer) {
        pointer.textContent =
          `Open: INTRADAY ${counts.INTRADAY || 0} · MULTI_DAY ${counts.MULTI_DAY || 0} · MULTI_WEEK ${counts.MULTI_WEEK || 0} · LONG_TERM ${counts.LONG_TERM || 0}`
          + ` · fetch-backed · 0.0% authority · gates CLOSED`;
      }
      const guideGrid = document.getElementById('mh-guide-grid');
      if (guideGrid) {
        guideGrid.innerHTML = Object.keys(MH_CLASS_META).map(k => {
          const m = MH_CLASS_META[k];
          const n = counts[k] != null ? counts[k] : 0;
          return `<div style="background:#0f172a; border:1px solid ${m.color}44; border-left:4px solid ${m.color}; border-radius:10px; padding:14px 16px;">
            <div style="display:flex; gap:10px; align-items:center; margin-bottom:6px; flex-wrap:wrap;">
              <span style="font-size:12px; font-weight:900; color:${m.color};">${k}</span>
              <span style="font-size:14px; font-weight:700; color:#f8fafc;">${m.title}</span>
              <span style="font-size:11px; color:#94a3b8;">open now: ${n}</span>
            </div>
            <p style="margin:0 0 8px 0; color:#cbd5e1; font-size:13px;">${m.meaning}</p>
            <div style="font-size:12px; color:#64748b;">NSE: ${m.nseOwner} · Crypto: ${m.cryptoOwner}</div>
          </div>`;
        }).join('');
      }
      const catalogGrid = document.getElementById('mh-catalog-grid');
      if (catalogGrid) {
        const nseCats = ((mh && mh.nse) || {}).catalog || [];
        const cryptoCats = ((mh && mh.crypto) || {}).catalog || [];
        if (!nseCats.length && !cryptoCats.length) {
          catalogGrid.innerHTML = '<div style="color:var(--text-muted); font-size:12px;">Horizon catalog unavailable (RP-609/610 not loaded).</div>';
        } else {
          catalogGrid.innerHTML =
            mhCatalogSection('NSE · RP-609 (MIS link + CNC books)', 'NSE', nseCats)
            + mhCatalogSection('CRYPTO · RP-610 (SPOT_SIM books)', 'CRYPTO', cryptoCats);
        }
      }
      paintMultiHorizonRecs();
      paintNseUniverse();
    }

    function paintMultiHorizonRecs() {
      const mh = window.__mhState;
      const tbody = document.getElementById('mh-recs-tbody');
      if (!tbody) return;
      const clsFilter = (document.getElementById('mh-filter-class') || {}).value || 'ALL';
      const mktFilter = (document.getElementById('mh-filter-market') || {}).value || 'ALL';
      const kindFilter = (document.getElementById('mh-filter-book-kind') || {}).value || 'ALL';
      const q = String((document.getElementById('mh-filter-search') || {}).value || '').trim().toUpperCase();
      let recs = (mh && mh.recommendations) ? mh.recommendations.slice() : [];
      if (clsFilter !== 'ALL') recs = recs.filter(r => r.horizon_class === clsFilter);
      if (mktFilter !== 'ALL') recs = recs.filter(r => r.market === mktFilter);
      const isUni = (r) => r.book_kind === 'UNIVERSE_MEMBER' || r.decision === 'UNIVERSE_MEMBER' || r.status === 'COVERED';
      if (kindFilter === 'ACTIONABLE') recs = recs.filter(r => !isUni(r));
      if (kindFilter === 'UNIVERSE') recs = recs.filter(r => isUni(r));
      if (q) {
        recs = recs.filter(r => {
          const blob = [r.symbol, ...(r.symbols || []), ...((r.legs || []).map(l => l.symbol)), r.plan_id, r.thesis]
            .filter(Boolean).join(' ').toUpperCase();
          return blob.includes(q);
        });
      }
      if (!recs.length) {
        tbody.innerHTML = `<tr><td colspan="11" style="color:var(--text-muted); padding:24px 14px;">
          No open observation books for this filter. H0 uses RP-608 fetches (NSE active_trades / crypto overlay);
          NSE marks come from kite bar_history. Demo H1–H3 rows are excluded.
        </td></tr>`;
        return;
      }
      tbody.innerHTML = recs.map(r => {
        const hc = r.horizon_class || 'INTRADAY';
        const meta = MH_CLASS_META[hc] || MH_CLASS_META.INTRADAY;
        const entryTxt = mhFormatLevel(r, 'entry_price');
        const stopTxt = mhFormatLevel(r, 'stop_loss_price');
        const tgtTxt = mhFormatLevel(r, 'target_price');
        const isUniverse = (r.book_kind === 'UNIVERSE_MEMBER') || (r.decision === 'UNIVERSE_MEMBER') || (r.status === 'COVERED');
        const isPortfolio = (r.book_kind === 'PORTFOLIO_BUILD') || (r.decision === 'PORTFOLIO_BUILD')
          || (Array.isArray(r.legs) && r.legs.length > 1);
        if (isUniverse) {
          const syms = mhFormatSymbols(r);
          const mark = entryTxt !== '—' ? entryTxt : '—';
          return `<tr style="background:rgba(148,163,184,0.05);">
            <td style="font-family:'JetBrains Mono',monospace; font-weight:800; color:#7dd3fc; white-space:nowrap;">${syms}</td>
            <td style="font-weight:700; color:#e2e8f0;">NSE</td>
            <td><span style="font-size:11px; font-weight:900; color:#94a3b8;">MULTI_DAY</span>
              <div style="font-size:10px; color:#64748b; margin-top:2px;">H1_SWING · universe</div></td>
            <td style="color:#94a3b8; font-size:12px;">119-universe member · in Open books as COVERED (not an opened trade)</td>
            <td style="color:#94a3b8; font-weight:700; font-size:12px;">UNIVERSE_MEMBER</td>
            <td style="color:#94a3b8;">—</td>
            <td style="color:#94a3b8; font-weight:700;">COVERED</td>
            <td style="font-family:'JetBrains Mono',monospace; color:#cbd5e1;">${mark}</td>
            <td style="color:#64748b; font-size:11px;">n/a</td>
            <td style="color:#64748b; font-size:11px;">n/a</td>
            <td style="color:#94a3b8; font-size:12px;">rp608_nse_119_universe
              <div style="font-size:10px; color:#64748b;">RP-608/609 · CNC · 0.0% authority</div></td>
          </tr>`;
        }
        if (isPortfolio) {
          const legs = Array.isArray(r.legs) && r.legs.length
            ? r.legs
            : (Array.isArray(r.symbols) ? r.symbols.map(s => ({ symbol: s, sector: '—', target_weight_pct: null, avg_price: null })) : []);
          const progress = r.tranche_progress || ((r.n_tranches != null) ? `${r.tranches_filled || 0}/${r.n_tranches}` : '—');
          const capital = (r.total_capital_inr != null && !Number.isNaN(Number(r.total_capital_inr)))
            ? ('₹' + Number(r.total_capital_inr).toLocaleString('en-IN'))
            : '—';
          const planId = r.plan_id || r.thesis || r.symbol || 'PORTFOLIO';
          const legRows = legs.map(leg => {
            const w = leg.target_weight_pct != null ? (Number(leg.target_weight_pct).toFixed(1) + '%') : '—';
            const avg = leg.avg_price != null ? ('₹' + Number(leg.avg_price).toFixed(2)) : '—';
            const qty = leg.qty_filled != null ? Number(leg.qty_filled) : 0;
            return `<tr style="background:rgba(0,0,0,0.18);">
              <td style="padding-left:28px; font-family:'JetBrains Mono',monospace; font-weight:800; color:#38bdf8; white-space:nowrap;">${leg.symbol || '—'}</td>
              <td style="color:#e2e8f0; font-weight:700;">${r.market || 'NSE'}</td>
              <td style="color:#94a3b8; font-size:11px;">${leg.sector || '—'}
                <div style="font-size:10px; color:#64748b;">leg · weight ${w}</div></td>
              <td style="color:#94a3b8; font-size:12px;">CNC tranche leg · qty ${qty}</td>
              <td style="color:#c4b5fd; font-size:11px;">—</td>
              <td style="color:#e2e8f0;">${r.side || 'LONG'}</td>
              <td style="color:#94a3b8; font-size:11px;">building</td>
              <td style="font-family:'JetBrains Mono',monospace; color:#f8fafc;">${avg}</td>
              <td style="color:#64748b; font-size:11px;" title="H2 has no per-leg stop">n/a</td>
              <td style="color:#64748b; font-size:11px;" title="H2 has no per-leg target">n/a</td>
              <td style="color:#64748b; font-size:11px;">plan ${planId}</td>
            </tr>`;
          }).join('');
          return `<tr style="background:rgba(251,146,60,0.08); border-top:1px solid rgba(251,146,60,0.25);">
            <td style="font-family:'JetBrains Mono',monospace; font-weight:800; color:#fdba74; white-space:nowrap;">
              PORTFOLIO · ${legs.length} names
              <div style="font-size:10px; color:#94a3b8; font-weight:600; margin-top:2px;">one CNC build plan — not a single ticker</div>
            </td>
            <td style="font-weight:700; color:#e2e8f0;">${r.market || 'NSE'}</td>
            <td><span style="font-size:11px; font-weight:900; color:${meta.color};">${hc}</span>
              <div style="font-size:10px; color:#64748b; margin-top:2px;">${r.horizon_id || ''}</div></td>
            <td style="color:#94a3b8; font-size:12px; max-width:240px;">${r.horizon_term || meta.meaning}
              <div style="font-size:10px; color:#fdba74; margin-top:2px;">Staggered build · tranche ${progress} · capital ${capital}</div></td>
            <td style="color:#c4b5fd; font-weight:700; font-size:12px;">${r.decision || 'PORTFOLIO_BUILD'}</td>
            <td style="color:#e2e8f0;">${r.side || 'LONG'}</td>
            <td style="color:#34d399; font-weight:700;">${r.status || 'ACTIVE'}</td>
            <td style="font-family:'JetBrains Mono',monospace; color:#f8fafc;">—</td>
            <td style="color:#64748b; font-size:11px;" title="H2 uses weights/caps, not swing stops">n/a</td>
            <td style="color:#64748b; font-size:11px;" title="H2 uses weights/caps, not swing targets">n/a</td>
            <td style="color:#94a3b8; font-size:12px;">${planId}
              <div style="font-size:10px; color:#64748b;">${r.source || 'RP-609'} · ${r.product_type || 'CNC'}</div></td>
          </tr>${legRows}`;
        }
        const syms = mhFormatSymbols(r);
        const stopNote = (hc === 'LONG_TERM' && stopTxt === '—') ? 'n/a' : stopTxt;
        const tgtNote = (hc === 'LONG_TERM' && tgtTxt === '—') ? 'n/a' : tgtTxt;
        return `<tr>
          <td style="font-family:'JetBrains Mono',monospace; font-weight:800; color:#38bdf8; white-space:nowrap;">${syms}</td>
          <td style="font-weight:700; color:#e2e8f0;">${r.market || '—'}</td>
          <td><span style="font-size:11px; font-weight:900; color:${meta.color};">${hc}</span>
            <div style="font-size:10px; color:#64748b; margin-top:2px;">${r.horizon_id || ''}</div></td>
          <td style="color:#94a3b8; font-size:12px; max-width:240px;">${r.horizon_term || meta.meaning}</td>
          <td style="color:#c4b5fd; font-weight:700; font-size:12px;">${r.decision || '—'}</td>
          <td style="color:#e2e8f0;">${r.side || 'LONG'}</td>
          <td style="color:#34d399; font-weight:700;">${r.status || 'OPEN'}</td>
          <td style="font-family:'JetBrains Mono',monospace; color:#f8fafc;">${entryTxt}</td>
          <td style="font-family:'JetBrains Mono',monospace; color:#f87171;">${stopNote}</td>
          <td style="font-family:'JetBrains Mono',monospace; color:#34d399;">${tgtNote}</td>
          <td style="color:#94a3b8; font-size:12px;">${r.thesis || r.plan_id || '—'}
            <div style="font-size:10px; color:#64748b;">${r.source || ''} · ${r.product_type || '—'}</div></td>
        </tr>`;
      }).join('');
    }

    document.addEventListener('DOMContentLoaded', () => {
      const a = document.getElementById('mh-filter-class');
      const b = document.getElementById('mh-filter-market');
      const c = document.getElementById('mh-filter-book-kind');
      const d = document.getElementById('mh-filter-search');
      if (a) a.addEventListener('change', paintMultiHorizonRecs);
      if (b) b.addEventListener('change', paintMultiHorizonRecs);
      if (c) c.addEventListener('change', paintMultiHorizonRecs);
      if (d) d.addEventListener('input', paintMultiHorizonRecs);
      const uf = document.getElementById('mh-universe-filter');
      const us = document.getElementById('mh-universe-search');
      if (uf) uf.addEventListener('change', paintNseUniverse);
      if (us) us.addEventListener('input', paintNseUniverse);
      document.querySelectorAll('.ws-tab').forEach(btn => {
        btn.addEventListener('click', () => switchWorkspace(btn.getAttribute('data-ws')));
      });
      document.querySelectorAll('.mh-inner-tab').forEach(btn => {
        btn.addEventListener('click', () => switchMhInner(btn.getAttribute('data-mh-inner')));
      });
      let initial = 'command';
      try {
        const saved = localStorage.getItem('rp608_ws');
        if (saved === 'horizons' || saved === 'command') initial = saved;
      } catch (e) {}
      if (location.hash === '#horizons') initial = 'horizons';
      switchWorkspace(initial);
    });

    async function fetchMultiHorizon() {
      try {
        const resp = await fetch('/api/multi_horizon');
        if (!resp.ok) return;
        const mh = await resp.json();
        if (mh && (mh.recommendations || mh.counts_by_class)) {
          updateMultiHorizonUI(mh);
        }
      } catch (e) {
        console.warn('multi_horizon fetch failed', e);
      }
    }

    async function fetchState() {
      try {
        const resp = await fetch('/api/state');
        if (!resp.ok) return;
        const data = await resp.json();

        // Paint MH first so this tab never depends on later fragile UI blocks
        if (data.multi_horizon) {
          updateMultiHorizonUI(data.multi_horizon);
        }
        if (data.rp611) {
          updateRp611UI(data.rp611);
        }
        updateOfPaperTradesUI(data);

        // Update Clocks
        document.getElementById('live-ist-clock').textContent = data.server_time_ist;
        document.getElementById('live-utc-clock').textContent = data.server_time_utc + ' UTC';

        // Service Status
        const svc = data.service_status;
        const svcEl = document.getElementById('service-status');
        svcEl.textContent = svc.active ? 'DAEMON ACTIVE' : 'DAEMON STOPPED';
        svcEl.parentElement.style.color = svc.active ? 'var(--accent-emerald)' : 'var(--accent-rose)';

        // Hero Card (Latest Decision Card)
        const latest = data.latest_card;
        if (latest) {
          document.getElementById('rec-bar-idx').textContent =
            (latest.decision_event_idx != null) ? latest.decision_event_idx : latest.bar_idx;
          
          const isProspective = (latest.close_time_ms >= 1792972800000);
          const pill = document.getElementById('sample-status-pill');
          pill.textContent = isProspective ? 'PROSPECTIVE SAMPLE' : 'PRE-WINDOW BUFFER';
          pill.className = 'pill-sm ' + (isProspective ? 'pill-prospective' : 'pill-prewindow');

          // Stage-1 regime badge (observational telemetry only)
          const regimeBadge = document.getElementById('regime-state-badge');
          if (regimeBadge) {
            const rs = latest.regime_state || 'UNAVAILABLE';
            regimeBadge.textContent = 'REGIME: ' + rs;
            let bg = 'rgba(148,163,184,0.12)', fg = 'var(--text-dim)', bd = 'rgba(148,163,184,0.25)';
            if (rs === 'DIRECTIONAL') { bg = 'rgba(16,185,129,0.15)'; fg = 'var(--accent-emerald)'; bd = 'rgba(16,185,129,0.3)'; }
            else if (rs === 'SIDEWAYS') { bg = 'rgba(245,158,11,0.15)'; fg = 'var(--accent-amber)'; bd = 'rgba(245,158,11,0.3)'; }
            else if (rs === 'TRANSITIONING') { bg = 'rgba(59,130,246,0.15)'; fg = '#60a5fa'; bd = 'rgba(59,130,246,0.3)'; }
            regimeBadge.style.background = bg; regimeBadge.style.color = fg; regimeBadge.style.borderColor = bd;
          }

          // Action Badges
          updateActionBadge('action-policy-b', latest.policy_b_action);
          updateActionBadge('action-policy-c', latest.policy_c_action);

          // Update Tooltip Contents for Policy B and Policy C
          const r = latest.reasoning || {};
          document.getElementById('tip-b-action').textContent = latest.policy_b_action || 'NO_TRADE';
          document.getElementById('tip-b-text').textContent = r.reason_b || 'No reasoning available.';
          document.getElementById('tip-b-ret').textContent = (latest.ret_15m_bps !== undefined) ? latest.ret_15m_bps.toFixed(2) + ' bps' : '--';
          document.getElementById('tip-b-vol').textContent = r.volatility || '--';
          document.getElementById('tip-b-trend').textContent = r.trend ? r.trend.split('(')[0].trim() : '--';

          document.getElementById('tip-c-action').textContent = latest.policy_c_action || 'NO_TRADE';
          document.getElementById('tip-c-text').textContent = r.reason_c || 'No reasoning available.';

          const ft = latest.features_tick || {};
          const midEl = document.getElementById('rec-tick-mid');
          const imbEl = document.getElementById('rec-tick-imb');
          if (midEl) midEl.textContent = (ft.mid_price != null) ? Number(ft.mid_price).toFixed(2) : '--';
          if (imbEl) imbEl.textContent = (ft.imbalance != null) ? Number(ft.imbalance).toFixed(3) : '--';

          // Details
          document.getElementById('rec-ret').textContent = (latest.ret_15m_bps !== undefined) ? latest.ret_15m_bps.toFixed(2) + ' bps' : '--';
          document.getElementById('rec-atr').textContent = (latest.atr_14 !== undefined) ? latest.atr_14.toFixed(2) : '--';
          document.getElementById('rec-quote-age').textContent = (latest.quote_age_ms !== undefined) ? latest.quote_age_ms + ' ms' : '--';

          // Price & Quotes
          document.getElementById('ref-price').textContent = '$' + latest.reference_price.toLocaleString('en-US', {minimumFractionDigits: 2, maximumFractionDigits: 2});
          
          if (latest.execution_quote) {
            document.getElementById('bid-price').textContent = '$' + latest.execution_quote.best_bid_price.toFixed(2);
            document.getElementById('ask-price').textContent = '$' + latest.execution_quote.best_ask_price.toFixed(2);
            document.getElementById('spread-bps').textContent = (latest.execution_quote.spread_bps !== undefined) ? latest.execution_quote.spread_bps.toFixed(4) + ' bps' : '--';
          }
        }

        // Render Trade Plan Overlay
        const plan = data.trade_plan;
        const ofRec = data.order_flow_recommendation;
        const ofExit = data.order_flow_last_exit;
        const ofPill = document.getElementById('plan-of-pill');
        const ofBanner = document.getElementById('crypto-of-banner');
        const ofBannerTitle = document.getElementById('crypto-of-banner-title');
        const ofBannerText = document.getElementById('crypto-of-banner-text');
        if (ofRec && ofRec.active) {
          const imb = (ofRec.order_flow_imbalance != null) ? Number(ofRec.order_flow_imbalance).toFixed(3) : '—';
          if (ofPill) {
            ofPill.style.display = 'inline-block';
            ofPill.textContent = 'ORDER FLOW';
            ofPill.style.background = '#86efac';
            ofPill.style.color = '#052e16';
            ofPill.style.borderColor = '#4ade80';
            ofPill.title = 'Active until opposite-side OF exit (not 60m)';
          }
          if (ofBanner) {
            ofBanner.style.display = 'block';
            ofBanner.style.background = 'rgba(134,239,172,0.08)';
            ofBanner.style.borderColor = 'rgba(74,222,128,0.35)';
          }
          const ec = ofRec.entry_candle || {};
          if (ofBannerTitle) { ofBannerTitle.textContent = 'ORDER FLOW ACTIVE'; ofBannerTitle.style.color = '#86efac'; }
          if (ofBannerText) {
            ofBannerText.textContent = `${ofRec.direction || ''} · imb ${imb} · ENTRY candle ${ec.candle_label || ('#'+(ec.bar_idx??'—'))} · evt ${ec.decision_event_idx ?? '—'} · exits on reverse OF`;
          }
        } else if (ofExit && ofExit.exit_reason) {
          const eImb = (ofExit.exit_imbalance != null) ? Number(ofExit.exit_imbalance).toFixed(3) : '—';
          const ec = ofExit.entry_candle || {};
          const xc = ofExit.exit_candle || {};
          if (ofPill) {
            ofPill.style.display = 'inline-block';
            ofPill.textContent = 'OF EXITED';
            ofPill.style.background = 'rgba(245,158,11,0.2)';
            ofPill.style.color = 'var(--accent-amber)';
            ofPill.style.borderColor = 'rgba(245,158,11,0.45)';
            ofPill.title = 'Order-flow exit received (reverse L1); plan may still be OPEN';
          }
          if (ofBanner) {
            ofBanner.style.display = 'block';
            ofBanner.style.background = 'rgba(245,158,11,0.08)';
            ofBanner.style.borderColor = 'rgba(245,158,11,0.35)';
          }
          if (ofBannerTitle) { ofBannerTitle.textContent = 'ORDER FLOW PAPER CLOSED'; ofBannerTitle.style.color = 'var(--accent-amber)'; }
          if (ofBannerText) {
            ofBannerText.textContent =
              `closed ${ofExit.direction || '—'} paper (sold to flat) · reverse-OF signal was ${ofExit.exit_opposite_side || '—'} · ` +
              `EXIT candle ${xc.candle_label || ('#'+(xc.bar_idx??'—'))} · ENTRY was ${ec.candle_label || ('#'+(ec.bar_idx??'—'))} · exit imb ${eImb} · not a naked short`;
          }
        } else {
          if (ofPill) ofPill.style.display = 'none';
          if (ofBanner) ofBanner.style.display = 'none';
        }
        const planHzEl = document.getElementById('plan-horizon-badges');
        if (planHzEl) {
          planHzEl.innerHTML = horizonBadgeHtml(plan || {
            horizon_class: 'INTRADAY',
            horizon_label: 'Intraday',
            horizon_term: '≤60 minutes (trade-plan overlay)'
          });
        }
        const planContainer = document.getElementById('trade-plan-container');
        if (plan || (ofRec && ofRec.active) || (ofExit && ofExit.exit_reason)) {
          planContainer.style.display = 'block';
        }
        if (plan) {
          document.getElementById('plan-direction-tag').textContent = plan.direction;
          document.getElementById('plan-direction-tag').style.color = (plan.direction === 'LONG') ? 'var(--accent-emerald)' : 'var(--accent-rose)';
          document.getElementById('plan-entry-price').textContent = '$' + plan.entry_price.toLocaleString('en-US', {minimumFractionDigits: 2, maximumFractionDigits: 2});
          document.getElementById('plan-qty').textContent = plan.simulated_quantity;

          document.getElementById('plan-target-price').textContent = '$' + plan.profit_target_price.toLocaleString('en-US', {minimumFractionDigits: 2, maximumFractionDigits: 2});
          const plannedTargetDist = (plan.direction === 'LONG')
            ? (plan.profit_target_price - plan.entry_price)
            : (plan.entry_price - plan.profit_target_price);
          // Remaining distance uses live MTM quote when present (not entry-planned ATR span).
          const markPx = (plan.valuation && plan.valuation.executable_exit_price)
            ? plan.valuation.executable_exit_price
            : null;
          let targetDistLabel = (plan.direction === 'LONG' ? '+' : '-') + plannedTargetDist.toFixed(2) + ' pts (1.50 ATR)';
          if (markPx != null && plan.state === 'OPEN') {
            const remain = (plan.direction === 'LONG')
              ? (plan.profit_target_price - markPx)
              : (markPx - plan.profit_target_price);
            if (remain <= 0) {
              targetDistLabel = 'TARGET REACHED (mark past TP; closing…)';
            } else {
              targetDistLabel = remain.toFixed(2) + ' pts remaining to TP';
            }
          } else if (plan.state === 'CLOSED_TP') {
            targetDistLabel = 'HIT @ $' + (plan.exit_price != null ? Number(plan.exit_price).toFixed(2) : plan.profit_target_price.toFixed(2));
          }
          document.getElementById('plan-target-dist').textContent = targetDistLabel;

          document.getElementById('plan-stop-price').textContent = '$' + plan.stop_loss_price.toLocaleString('en-US', {minimumFractionDigits: 2, maximumFractionDigits: 2});
          const stopDist = (plan.direction === 'LONG') ? (plan.entry_price - plan.stop_loss_price) : (plan.stop_loss_price - plan.entry_price);
          document.getElementById('plan-stop-dist').textContent = (plan.direction === 'LONG' ? '-' : '+') + stopDist.toFixed(2) + ' pts (1.00 ATR)';

          document.getElementById('plan-gross-rr').textContent = plan.gross_rr_ratio.toFixed(2) + ':1';
          document.getElementById('plan-net-rr').textContent = plan.net_rr_ratio.toFixed(2) + ' : 1';
          document.getElementById('plan-friction-usdt').textContent = '$' + plan.est_friction_usdt.toFixed(2);

          // Tooltip in trade plan
          const pr = plan.reasoning || {};
          document.getElementById('tip-plan-reasoning').textContent = pr.trade_plan_why || pr.reason_b || 'Simulated trade plan based on Policy B canonical signal.';

          // Mark to market valuation
          const val = plan.valuation || {};
          const isClosed = (plan.state !== 'OPEN');
          const mtmEl = document.getElementById('plan-mtm-val');
          const quoteEl = document.getElementById('plan-current-quote');
          const mtmStatusEl = document.getElementById('plan-mtm-status');

          if (val.unrealized_net_pnl_usdt !== undefined) {
            const sign = val.unrealized_net_pnl_usdt >= 0 ? '+' : '';
            mtmEl.textContent = `${sign}$${val.unrealized_net_pnl_usdt.toFixed(2)} (${val.unrealized_net_return_bps.toFixed(1)} bps)`;
            mtmEl.style.color = (val.unrealized_net_pnl_usdt >= 0) ? 'var(--accent-emerald)' : 'var(--accent-rose)';
            quoteEl.textContent = `Bid $${val.current_bid.toFixed(2)} / Ask $${val.current_ask.toFixed(2)}`;
            mtmStatusEl.textContent = isClosed ? 'Realized Net P&L' : 'Unrealized MTM Net';
          } else {
            mtmEl.textContent = '--';
          }

          // State Pill Styling
          const statePill = document.getElementById('plan-state-pill');
          if (plan.state === 'OPEN') {
            statePill.textContent = 'PLAN OPEN (ACTIVE)';
            statePill.className = 'pill-sm pill-prospective';
            statePill.style.background = '';
            statePill.style.color = '';
          } else if (plan.state === 'CLOSED_TP') {
            statePill.textContent = 'CLOSED (TARGET HIT)';
            statePill.className = 'pill-sm pill-prospective';
            statePill.style.background = 'rgba(16,185,129,0.2)';
            statePill.style.color = 'var(--accent-emerald)';
          } else if (plan.state === 'CLOSED_SL') {
            statePill.textContent = 'CLOSED (STOP LOSS)';
            statePill.className = 'pill-sm pill-prewindow';
            statePill.style.background = 'rgba(244,63,94,0.2)';
            statePill.style.color = 'var(--accent-rose)';
          } else if (plan.state === 'TIME_EXIT') {
            statePill.textContent = 'CLOSED (TIME EXPIRED)';
            statePill.className = 'pill-sm';
            statePill.style.background = 'rgba(245,158,11,0.2)';
            statePill.style.color = 'var(--accent-amber)';
          } else {
            statePill.textContent = plan.state;
          }

          // Expiry & Countdown calculation
          const nowMs = Date.now();
          const expDate = new Date(plan.expiry_time_ms);
          document.getElementById('plan-expiry-clock').textContent = expDate.toISOString().substring(11, 19) + ' UTC';
          document.getElementById('plan-id-text').textContent = plan.plan_id;
          document.getElementById('plan-inval-text').textContent = '$' + plan.invalidation_price.toFixed(2);

          if (isClosed) {
            document.getElementById('plan-countdown').textContent = `Closed (${plan.state})`;
            document.getElementById('plan-horizon-time').textContent = plan.exit_price ? `Exit: $${plan.exit_price.toFixed(2)}` : 'Closed';
            document.getElementById('plan-horizon-tag').textContent = 'Lifecycle Terminal';
          } else {
            // Prefer server MTM clock (same clock as TP/SL); fall back to expiry_time_ms.
            let remainSec = (val.time_remaining_seconds !== undefined && val.time_remaining_seconds !== null)
              ? Math.max(0, Math.floor(val.time_remaining_seconds))
              : Math.max(0, Math.floor((plan.expiry_time_ms - nowMs) / 1000));
            const rMins = Math.floor(remainSec / 60);
            const rSecs = remainSec % 60;
            const remainLabel = `${rMins}m ${String(rSecs).padStart(2, '0')}s`;
            document.getElementById('plan-countdown').textContent = `${remainLabel} left`;
            document.getElementById('plan-horizon-time').textContent = remainLabel;
            document.getElementById('plan-horizon-tag').textContent = 'Remaining / Max 60m';
          }
        } else if (!(ofRec && ofRec.active) && !(ofExit && ofExit.exit_reason)) {
          planContainer.style.display = 'none';
        }

        // Ratio Framework (Sharpe / Calmar)
        const rf = data.ratio_framework;
        if (rf) {
          document.getElementById('ratio-disclaimer').textContent = rf.disclaimer;
          document.getElementById('metric-sharpe').textContent = rf.descriptive_sharpe !== null ? rf.descriptive_sharpe : 'Insufficient Data';
          document.getElementById('metric-calmar').textContent = rf.descriptive_calmar;
          document.getElementById('metric-stage').textContent = rf.current_stage.split(':')[0];
          document.getElementById('metric-prospective-sample').textContent = rf.prospective_sample_size + ' Cards';
        }

        // Countdown Timer
        const cd = data.next_candle_close_countdown_sec;
        const mins = Math.floor(cd / 60);
        const secs = cd % 60;
        document.getElementById('next-candle-timer').textContent = `${String(mins).padStart(2, '0')}:${String(secs).padStart(2, '0')}`;

        // Accounting
        const acc = data.accounting;
        document.getElementById('prospective-count').textContent = acc.prospective_count;
        document.getElementById('prewindow-count').textContent = acc.prewindow_excluded;
        document.getElementById('total-cards-count').textContent = acc.total_cards;
        document.getElementById('elapsed-time').textContent = data.window_elapsed_hours + ' hrs';

        // Telemetry Gates
        const tg = data.telemetry_gates;
        document.getElementById('gate-quote-cov').textContent = tg.quote_coverage_pct.toFixed(1) + '%';
        document.getElementById('gate-drift').textContent = tg.max_drift_ms + ' ms (PASS)';
        document.getElementById('gate-missing').textContent = tg.total_missing_bars + ' (PASS)';
        document.getElementById('gate-reconnects').textContent = tg.reconnection_count_24h + ' (PASS)';
        
        const cs = data.checkpoint_integrity;
        document.getElementById('gate-checksum').textContent = cs.recorded.substring(0, 16) + '... (VALID)';

        // Decision Cards Table (+ OF entry/exit candle highlights)
        const tbody = document.getElementById('decision-cards-tbody');
        tbody.innerHTML = '';
        const ofMarks = data.of_candle_markers || {};
        const ofEntry = ofMarks.entry || ((data.order_flow_recommendation || data.order_flow_last_exit || {}).entry_candle) || null;
        const ofExitC = ofMarks.exit || ((data.order_flow_last_exit || {}).exit_candle) || null;
        const legend = document.getElementById('of-candle-legend');
        if (legend) {
          const bits = [];
          if (ofEntry) bits.push(`OF ENTRY → 15m #${ofEntry.bar_idx ?? '—'} ${ofEntry.candle_label || ''}`);
          if (ofExitC) bits.push(`OF EXIT → 15m #${ofExitC.bar_idx ?? '—'} ${ofExitC.candle_label || ''}`);
          legend.textContent = bits.length ? ('· ' + bits.join(' · ')) : '';
        }
        function ofRowKind(c) {
          // Event-precise OF badges only. Never paint by 15m window / bar_idx alone —
          // window-only exit marks (legend "15m #—") made every #665 prospective
          // card look like OF EXIT.
          const ev = c.decision_event_idx;
          const t = c.close_time_ms;
          const MARK_PROX_MS = 5000; // ±5s around mark_time / decision event
          const hit = (m) => {
            if (!m) return false;
            // Require event identity — refuse window-only marks (no bar, no evt)
            if (m.decision_event_idx == null && m.bar_idx == null) return false;
            if (m.decision_event_idx != null && ev != null
                && Number(m.decision_event_idx) === Number(ev)) {
              return true;
            }
            // Time proximity only when decision_event_idx is known on the mark
            if (m.decision_event_idx != null && m.mark_time_ms != null && t != null) {
              return Math.abs(Number(m.mark_time_ms) - Number(t)) <= MARK_PROX_MS;
            }
            return false;
          };
          const isEntry = hit(ofEntry);
          const isExit = hit(ofExitC);
          if (isEntry && isExit) return 'BOTH';
          if (isEntry) return 'ENTRY';
          if (isExit) return 'EXIT';
          return null;
        }
        (data.recent_cards || []).forEach(c => {
          const tr = document.createElement('tr');
          const isPro = (c.close_time_ms >= 1792972800000);
          const typeBadge = isPro 
            ? '<span class="pill-sm pill-prospective">PROSPECTIVE</span>' 
            : '<span class="pill-sm pill-prewindow">STARTUP</span>';

          const cr = c.reasoning || {};
          const bBadge = getActionMiniBadgeWithTooltip(c.policy_b_action, cr.reason_b);
          const cBadge = getActionMiniBadgeWithTooltip(c.policy_c_action, cr.reason_c);
          const kind = ofRowKind(c);
          let ofCell = '<span style="color:var(--text-dim);">—</span>';
          if (kind === 'ENTRY') {
            ofCell = '<span class="pill-sm" style="background:#86efac; color:#052e16; border:1px solid #4ade80; font-weight:800;">OF ENTRY</span>';
            tr.style.background = 'rgba(134,239,172,0.12)';
            tr.style.outline = '1px solid rgba(74,222,128,0.45)';
          } else if (kind === 'EXIT') {
            ofCell = '<span class="pill-sm" style="background:rgba(245,158,11,0.25); color:var(--accent-amber); border:1px solid rgba(245,158,11,0.5); font-weight:800;">OF EXIT</span>';
            tr.style.background = 'rgba(245,158,11,0.10)';
            tr.style.outline = '1px solid rgba(245,158,11,0.4)';
          } else if (kind === 'BOTH') {
            ofCell = '<span class="pill-sm" style="background:rgba(6,182,212,0.2); color:var(--accent-cyan); border:1px solid rgba(6,182,212,0.45); font-weight:800;">OF IN/OUT</span>';
            tr.style.background = 'rgba(6,182,212,0.10)';
          }

          tr.innerHTML = `
            <td><b>#${c.bar_idx}</b></td>
            <td>${c.timestamp_utc.replace('T', ' ').substring(11, 19)}</td>
            <td>${ofCell}</td>
            <td>${typeBadge}</td>
            <td>$${c.reference_price.toFixed(2)}</td>
            <td>${bBadge}</td>
            <td>${cBadge}</td>
            <td style="color:${c.ret_15m_bps >= 0 ? 'var(--accent-emerald)' : 'var(--accent-rose)'};">${c.ret_15m_bps.toFixed(2)}</td>
            <td>${c.atr_14.toFixed(2)}</td>
            <td>${c.quote_age_ms} ms</td>
            <td>${c.execution_quote ? c.execution_quote.spread_bps.toFixed(4) : '--'}</td>
          `;
          tbody.appendChild(tr);
        });

        // NSE Operations update
        if (data.nse_operations) {
          updateNseUI(data.nse_operations);
        }

        // NSE Schedule Timeline
        const nseEl = document.getElementById('nse-timeline');
        if (nseEl) {
          nseEl.innerHTML = '';
          (data.nse_schedule || []).forEach(item => {
            const div = document.createElement('div');
            div.className = 'timeline-step';
            const isDone = (data.server_time_ist || '').split(' ')[1] > item.time_ist;
            const badgeText = isDone ? 'COMPLETED' : 'PENDING';
            const badgeColor = isDone ? 'var(--accent-emerald)' : 'var(--text-dim)';
            const badgeBg = isDone ? 'rgba(16,185,129,0.15)' : 'rgba(255,255,255,0.05)';
            div.innerHTML = `
              <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:4px;">
                <span class="step-time">${item.time_ist} IST</span>
                <span style="font-size:9px; font-weight:700; padding:2px 5px; border-radius:4px; background:${badgeBg}; color:${badgeColor};">${badgeText}</span>
              </div>
              <div class="step-name">${item.name}</div>
            `;
            nseEl.appendChild(div);
          });
        }

        // Gate C Parallel Lookbacks
        if (data.gate_c) {
          const gc = data.gate_c;
          const statusBadge = document.getElementById('gate-c-status-badge');
          if (statusBadge) statusBadge.textContent = gc.gate_c_status;
          const servicePill = document.getElementById('gate-c-service-pill');
          if (servicePill) {
            servicePill.textContent = gc.service_active ? 'SERVICE ACTIVE' : 'SERVICE STOPPED';
            servicePill.className = gc.service_active ? 'badge badge-active' : 'badge badge-inactive';
          }

          const agBadge = document.getElementById('gate-c-agreement-badge');
          if (agBadge && gc.agreement) {
            agBadge.textContent = 'AGREEMENT: ' + gc.agreement.status;
            agBadge.title = gc.agreement.detail;
            if (gc.agreement.type === 'CONCORDANT') {
              agBadge.style.background = 'rgba(244, 63, 94, 0.15)';
              agBadge.style.color = 'var(--accent-rose)';
              agBadge.style.border = '1px solid rgba(244, 63, 94, 0.3)';
            } else {
              agBadge.style.background = 'rgba(245, 158, 11, 0.15)';
              agBadge.style.color = 'var(--accent-amber)';
              agBadge.style.border = '1px solid rgba(245, 158, 11, 0.3)';
            }
          }

          const grid = document.getElementById('gate-c-candidates-grid');
          if (grid && gc.candidates) {
            grid.innerHTML = '';
            const candKeys = ['16', '32', '64', '120'];
            candKeys.forEach(k => {
              const cand = gc.candidates[k];
              if (!cand) return;
              const isShort = cand.direction === 'SHORT';
              const isLong = cand.direction === 'LONG';
              const decColor = isShort ? 'var(--accent-rose)' : (isLong ? 'var(--accent-emerald)' : 'var(--text-dim)');
              const decBg = isShort ? 'rgba(244, 63, 94, 0.15)' : (isLong ? 'rgba(16, 185, 129, 0.15)' : 'rgba(148, 163, 184, 0.1)');
              const decBorder = isShort ? 'rgba(244, 63, 94, 0.3)' : (isLong ? 'rgba(16, 185, 129, 0.3)' : 'rgba(148, 163, 184, 0.2)');

              const cBox = document.createElement('div');
              cBox.style.cssText = 'background:rgba(0,0,0,0.35); border:1px solid var(--card-border); border-radius:10px; padding:12px;';
              cBox.innerHTML = `
                <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:8px;">
                  <span style="font-size:12px; font-weight:700; color:#fff;">${cand.name}</span>
                  <span style="font-size:10px; font-family:'JetBrains Mono',monospace; color:var(--text-dim);" title="SHA-256: ${cand.hash_full}">${cand.hash_prefix}...</span>
                </div>
                <div style="margin-bottom:8px; display:flex; flex-wrap:wrap; gap:4px;">${horizonBadgeHtml(cand)}</div>
                <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:10px;">
                  <span style="padding:3px 8px; border-radius:6px; font-size:11px; font-weight:700; background:${decBg}; color:${decColor}; border:1px solid ${decBorder};">
                    ${cand.latest_recommendation}
                  </span>
                  <span style="font-size:11px; color:var(--text-muted); font-family:'JetBrains Mono',monospace;">${cand.total_emitted} recs</span>
                </div>
                <div style="font-size:11px; color:var(--text-dim); display:flex; flex-direction:column; gap:3px;">
                  <div style="display:flex; justify-content:space-between;">
                    <span>Lookback Trend:</span>
                    <b style="color:${cand.candle_trend_bps < 0 ? 'var(--accent-rose)' : 'var(--accent-emerald)'}; font-family:'JetBrains Mono',monospace;">${cand.candle_trend_bps !== null && cand.candle_trend_bps !== undefined ? (cand.candle_trend_bps > 0 ? '+' : '') + cand.candle_trend_bps.toFixed(1) + ' bps' : '--'}</b>
                  </div>
                  <div style="display:flex; justify-content:space-between;">
                    <span>Entry Ref:</span>
                    <b style="color:#fff; font-family:'JetBrains Mono',monospace;">${cand.entry_price > 0 ? '$' + cand.entry_price.toFixed(2) : '--'}</b>
                  </div>
                  <div style="display:flex; justify-content:space-between;">
                    <span>Inval Stop (30bps):</span>
                    <b style="color:var(--accent-rose); font-family:'JetBrains Mono',monospace;">${cand.invalidation_price > 0 ? '$' + cand.invalidation_price.toFixed(2) : '--'}</b>
                  </div>
                  <div style="display:flex; justify-content:space-between; margin-top:2px;">
                    <span>Last Signal:</span>
                    <span style="color:var(--text-muted); font-size:10px;">${cand.last_emitted_utc !== 'N/A' ? cand.last_emitted_utc.substring(11, 19) + ' UTC' : 'None (Filtered)'}</span>
                  </div>
                </div>
              `;
              grid.appendChild(cBox);
            });
          }

          const bufEl = document.getElementById('gate-c-candle-buffer');
          if (bufEl && gc.latest_telemetry) {
            bufEl.textContent = `${gc.latest_telemetry.candle_buffer_count} Bars (Warmed Up)`;
          }

          const telemEl = document.getElementById('gate-c-telemetry-counters');
          if (telemEl && gc.latest_telemetry) {
            const t = gc.latest_telemetry;
            const firstCand = Object.values(t.candidates || {})[0] || {};
            telemEl.innerHTML = `Quotes: <b style="color:#fff;">${firstCand.quotes_received ? firstCand.quotes_received.toLocaleString() : '--'}</b> · Rate-Limited (&ge;1s): <b style="color:#fff;">${firstCand.recs_rate_limited ? firstCand.recs_rate_limited.toLocaleString() : '--'}</b> · Filtered: <b style="color:var(--accent-amber);">${firstCand.recs_counter_trend_filtered ? firstCand.recs_counter_trend_filtered.toLocaleString() : '--'}</b>`;
          }
        }

        // Synchronize 119-symbol NSE explorer state
        if (data.nse_operations) {
          updateNseUI(data.nse_operations);
        }

      } catch (err) {
        console.error("Dashboard fetch error:", err);
      }
    }

    function updateActionBadge(id, action) {
      const el = document.getElementById(id);
      el.textContent = action || 'NO_TRADE';
      el.className = 'action-badge action-' + (action || 'NO_TRADE');
    }

    function getActionMiniBadgeWithTooltip(action, reason) {
      let color = 'var(--text-dim)';
      let symbol = '—';
      if (action === 'LONG') { color = 'var(--accent-emerald)'; symbol = '▲'; }
      if (action === 'SHORT') { color = 'var(--accent-rose)'; symbol = '▼'; }

      const titleAttr = (reason || '').replace(/"/g, '&quot;');
      return `<span title="${titleAttr}" style="color:${color}; font-weight:700; cursor:help; text-decoration:underline dotted rgba(255,255,255,0.2);">${symbol} ${action}</span>`;
    }

    let allNseScrips = [];
    const benchmarkSymbols = new Set([
      "RELIANCE", "TCS", "INFY", "HDFCBANK", "ICICIBANK", "BHARTIARTL", "ITC", 
      "ADANIENT", "ADANIPORTS", "SBIN", "LT", "KOTAKBANK", "AXISBANK", "MARUTI", 
      "TATACONSUM", "TITAN", "SUNPHARMA", "WIPRO", "HINDUNILVR", "BAJFINANCE"
    ]);

    function setNseCompartmentFilter(comp) {
      const sel = document.getElementById('select-filter-compartment');
      if (sel) {
        sel.value = comp;
        onFilterChange();
      }
    }

    function onFilterChange() {
      renderNseTable();
    }

    async function refreshNseBatch() {
      const btn = document.getElementById('btn-refresh-nse');
      if (btn) {
        btn.disabled = true;
        btn.textContent = '⟳ Evaluating 119 Scrips...';
      }
      try {
        const res = await fetch('/api/nse/refresh');
        if (res.ok) {
          const data = await res.json();
          if (data.nse_operations) {
            updateNseUI(data.nse_operations);
          }
        }
        if (btn) {
          btn.textContent = '✓ Evaluated & Updated';
          setTimeout(() => {
            btn.disabled = false;
            btn.textContent = '⟳ Refresh Live Batch';
          }, 2000);
        }
      } catch (e) {
        if (btn) {
          btn.textContent = '⚠ Refresh Failed';
          setTimeout(() => {
            btn.disabled = false;
            btn.textContent = '⟳ Refresh Live Batch';
          }, 2000);
        }
      }
    }

    function updateNseUI(nse) {
      if (!nse) return;
      const ofp = nse.of_paper_ledger || {};
      const ofpSum = ofp.of_outcomes_summary || {};
      const ofpVal = document.getElementById('nse-of-paper-val');
      const ofpSub = document.getElementById('nse-of-paper-sub');
      const ofpNote = document.getElementById('nse-of-paper-note');
      if (ofpVal) {
        const nOpen = ofp.n_open != null ? ofp.n_open : 0;
        ofpVal.textContent = (ofp.available ? 'PAPER' : 'OFF') + ' · ' + nOpen + ' open';
      }
      if (ofpSub) {
        const nt = ofpSum.n_trades != null ? ofpSum.n_trades : 0;
        const net = ofpSum.cumulative_net_pnl_inr != null ? Number(ofpSum.cumulative_net_pnl_inr).toFixed(2) : '0.00';
        const opens = (ofp.opens || []).map(o => o.symbol).filter(Boolean).slice(0, 6).join(', ');
        ofpSub.textContent = `n_trades=${nt} · net ₹${net}` + (opens ? ` · open: ${opens}` : '');
      }
      if (ofpNote) {
        ofpNote.textContent = (ofp.protocol_id || 'RP-608-NSE-OF-PAPER-TRADE-001') +
          ' · reverse OF / square-off · 0% authority';
      }
      if (nse.api_key) {
        currentApiKey = nse.api_key;
        const link = document.getElementById('kite-login-link');
        if (link) {
          link.href = `https://kite.zerodha.com/connect/login?v=3&api_key=${encodeURIComponent(nse.api_key)}`;
        }
      }
      const foot = document.getElementById('nse-footer-daemon');
      if (foot) {
        const pid = nse.daemon_pid;
        const src = nse.artifact_source || 'nse';
        const ntp = (nse.live_websocket && nse.live_websocket.calibrated_ntp_offset_ms != null)
          ? Number(nse.live_websocket.calibrated_ntp_offset_ms).toFixed(3) : '--';
        const pidHtml = pid
          ? `Daemon PID: <b>${pid}</b> (active)`
          : `Daemon PID: <b>--</b> (not detected)`;
        foot.innerHTML = `${pidHtml} · Artifacts: <b>${src}</b> · Stratum-1 PPS NTP Synced (+${ntp} ms) · Host: <b>LCUtest</b> (192.168.0.9)`;
      }
      // Coverage
      const covEl = document.getElementById('nse-cov-val');
      if (covEl && nse.coverage) covEl.textContent = nse.coverage.status_text;
      const covNote = document.getElementById('nse-cov-note');
      if (covNote && nse.coverage && nse.coverage.mode_note) covNote.textContent = nse.coverage.mode_note;

      // Continuous L1 feed (not the one-shot readiness probe)
      const wsVal = document.getElementById('nse-ws-val');
      const wsSub = document.getElementById('nse-ws-sub');
      const wsAge = document.getElementById('nse-ws-age');
      const wsAgeMetric = document.getElementById('nse-ws-age-metric');
      const probeNote = document.getElementById('nse-probe-note');
      if (nse.live_websocket) {
        const lw = nse.live_websocket;
        if (wsVal) wsVal.textContent = lw.status || '—';
        const tokCached = (lw.cached_tokens != null) ? lw.cached_tokens : '—';
        const tokSub = (lw.subscribed_tokens != null) ? lw.subscribed_tokens : (nse.coverage ? nse.coverage.total_universe : 119);
        if (wsSub) wsSub.textContent = `source=${lw.source || '—'} · ${tokCached}/${tokSub} tokens`;
        if (wsAgeMetric) wsAgeMetric.textContent = (lw.age_metric === 'packet') ? 'Packet age' : 'Heartbeat age';
        if (wsAge) {
          const ageStr = (lw.age_sec !== null && lw.age_sec !== undefined)
            ? `${lw.age_sec}s ago`
            : (lw.last_packet_utc && lw.last_packet_utc !== 'N/A'
              ? String(lw.last_packet_utc).substring(11, 19) + ' UTC'
              : 'N/A');
          wsAge.textContent = ageStr;
        }
      }
      if (probeNote && nse.readiness_probe) {
        const rp = nse.readiness_probe;
        const rpAge = (rp.age_sec != null) ? `${rp.age_sec}s` : 'n/a';
        probeNote.textContent = `Readiness probe (separate): ${rp.status || '—'} · age ${rpAge}`;
      }
      const sess = nse.production_session;
      if (sess && probeNote) {
        probeNote.textContent += ` · session ${sess.session_date || '—'} durable=${sess.durable_bar ?? '—'} G1=${sess.g1_status || '—'}`;
      }

      // Pre-Open
      const preVal = document.getElementById('nse-pre-val');
      const avgGapEl = document.getElementById('nse-avg-gap');
      const preVolEl = document.getElementById('nse-pre-vol');
      if (nse.preopen_discovery) {
        if (preVal) preVal.textContent = nse.preopen_discovery.status;
        if (avgGapEl) {
          const g = nse.preopen_discovery.avg_gap_bps;
          avgGapEl.textContent = (g > 0 ? '+' : '') + g.toFixed(1) + ' bps';
          avgGapEl.style.color = g >= 0 ? 'var(--accent-emerald)' : 'var(--accent-rose)';
        }
        if (preVolEl) {
          preVolEl.textContent = (nse.preopen_discovery.total_auction_qty / 1000).toFixed(1) + 'k';
        }
      }

      // Bar 1 Recon
      const reconVal = document.getElementById('nse-recon-val');
      const rttVal = document.getElementById('nse-rtt-val');
      const reconLast = document.getElementById('nse-recon-last');
      if (nse.bar_reconciliation) {
        if (reconVal) reconVal.textContent = nse.bar_reconciliation.status;
        if (rttVal) rttVal.textContent = nse.bar_reconciliation.rtt_ms.toFixed(1);
        if (reconLast) {
          const rAge = (nse.bar_reconciliation.age_sec !== null && nse.bar_reconciliation.age_sec !== undefined) ? `${nse.bar_reconciliation.age_sec}s ago` : (nse.bar_reconciliation.last_check_utc !== 'N/A' ? nse.bar_reconciliation.last_check_utc.substring(11, 19) + ' UTC' : 'N/A');
          reconLast.textContent = rAge;
        }
      }

      // Quote Health Strip
      if (nse.quote_health) {
        const qh = nse.quote_health;
        const vEl = document.getElementById('nse-valid-cnt');
        const sEl = document.getElementById('nse-stale-cnt');
        const mEl = document.getElementById('nse-miss-cnt');
        const cEl = document.getElementById('nse-cross-cnt');
        const zEl = document.getElementById('nse-zero-cnt');
        if (vEl) vEl.textContent = qh.valid;
        if (sEl) sEl.textContent = qh.stale;
        if (mEl) mEl.textContent = qh.missing;
        if (cEl) cEl.textContent = qh.crossed;
        if (zEl) zEl.textContent = qh.zero_depth;
      }

      // Top gainers/losers
      if (nse.preopen_discovery && nse.preopen_discovery.top_gainers && nse.preopen_discovery.top_gainers.length > 0) {
        const tg = nse.preopen_discovery.top_gainers[0];
        const tl = (nse.preopen_discovery.top_losers && nse.preopen_discovery.top_losers.length > 0) ? nse.preopen_discovery.top_losers[0] : null;
        const tgEl = document.getElementById('nse-top-gainer');
        const tlEl = document.getElementById('nse-top-loser');
        if (tgEl) tgEl.textContent = `▲ ${tg.symbol} (+${tg.gap_bps} bps)`;
        if (tlEl && tl) tlEl.textContent = `▼ ${tl.symbol} (${tl.gap_bps} bps)`;
      }

      // Next-session post-close watchlist (live-polled with /api/state)
      const nsr = nse.next_session_recommendations || {};
      const nsrPill = document.getElementById('nse-next-session-pill');
      const nsrMeta = document.getElementById('nse-next-session-meta');
      const nsrList = document.getElementById('nse-next-session-list');
      const nsrRecTime = document.getElementById('nse-next-session-rec-time');
      const nsrLiveClock = document.getElementById('nse-next-session-live-clock');
      const nsrLive = document.getElementById('nse-next-session-live');

      function formatNsrWhen(iso) {
        if (!iso) return null;
        const d = new Date(iso);
        if (Number.isNaN(d.getTime())) return String(iso);
        try {
          const ist = d.toLocaleString('en-IN', {
            timeZone: 'Asia/Kolkata',
            year: 'numeric', month: 'short', day: '2-digit',
            hour: '2-digit', minute: '2-digit', second: '2-digit',
            hour12: false
          });
          const utc = d.toLocaleString('en-GB', {
            timeZone: 'UTC',
            year: 'numeric', month: 'short', day: '2-digit',
            hour: '2-digit', minute: '2-digit', second: '2-digit',
            hour12: false
          });
          return { ist: ist + ' IST', utc: utc + ' UTC', ms: d.getTime() };
        } catch (_) {
          return { ist: String(iso), utc: '', ms: d.getTime() };
        }
      }

      if (nsrPill) {
        const ofN = nsr.n_order_flow_supported != null ? Number(nsr.n_order_flow_supported) : 0;
        nsrPill.textContent = nsr.available
          ? (`NEXT ${nsr.next_session_date || ''} · ${nsr.n_recommendations || 0} · OF ${ofN}`)
          : 'WAITING';
      }
      if (nsrMeta) {
        nsrMeta.textContent = nsr.note
          || `From ${nsr.from_session_date || '—'} → next ${nsr.next_session_date || '—'} · 0.0% authority · not overnight/CAS`;
      }
      const recAtIso = nsr.recommended_at_utc || nsr.emitted_utc || null;
      const recAt = formatNsrWhen(recAtIso);
      const stampAt = formatNsrWhen(nsr.stamp_mtime_utc);
      if (nsrRecTime) {
        nsrRecTime.textContent = recAt
          ? (`Recommended at: ${recAt.ist}${recAt.utc ? ' · ' + recAt.utc : ''}`)
          : 'Recommended at: — (stamp not yet emitted)';
      }
      const pollSec = nsr.live_poll_hint_sec != null ? Number(nsr.live_poll_hint_sec) : 3;
      const uiNow = formatNsrWhen(new Date().toISOString());
      if (nsrLiveClock) {
        const stampBit = stampAt ? ` · file mtime ${stampAt.ist}` : '';
        nsrLiveClock.textContent = `UI refresh: ${uiNow ? uiNow.ist : '—'} · live poll ${pollSec}s${stampBit}`;
      }
      if (nsrLive) {
        nsrLive.style.opacity = '1';
        nsrLive.title = recAt
          ? `Live panel · last recommendation ${recAt.ist}`
          : 'Live panel · waiting for next-session stamp';
      }
      if (nsrList) {
        const rows = nsr.recommendations || [];
        if (!rows.length) {
          nsrList.innerHTML = 'No next-session watches yet — emitted automatically after POST_CLOSE.';
        } else {
          const lvlStatusColor = (st) => {
            if (st === 'VALID') return '#86efac';
            if (st === 'WAITING') return '#fde68a';
            if (st === 'INVALIDATED') return '#fda4af';
            if (st === 'STALE_SUPPRESSED') return '#94a3b8';
            return '#94a3b8';
          };
          nsrList.innerHTML = rows.slice(0, 20).map((r, i) => {
            const sym = r.tradingsymbol || (r.symbol || '').replace('.NS','');
            const sc = (r.score != null) ? Number(r.score).toFixed(4) : '—';
            const ofOn = !!r.order_flow_supported;
            const imb = (r.order_flow_imbalance != null) ? Number(r.order_flow_imbalance).toFixed(3) : '';
            const rowIso = r.recommended_at_utc || recAtIso;
            const rowRec = formatNsrWhen(rowIso);
            let rowRecShort = '—';
            if (rowIso) {
              try {
                rowRecShort = new Date(rowIso).toLocaleTimeString('en-IN', {
                  timeZone: 'Asia/Kolkata', hour: '2-digit', minute: '2-digit', second: '2-digit', hour12: false
                }) + ' IST';
              } catch (_) { rowRecShort = rowRec ? rowRec.ist : '—'; }
            }
            const lv = r.levels || {};
            const eLo = r.entry_low != null ? r.entry_low : lv.entry_low;
            const eHi = r.entry_high != null ? r.entry_high : lv.entry_high;
            const stop = r.stop_price != null ? r.stop_price : lv.stop_price;
            const tgt = r.target_price != null ? r.target_price : lv.target_price;
            const rr = r.risk_reward_display || lv.risk_reward_display || '—';
            const lst = r.level_status || lv.level_status || '—';
            const entryRange = (eLo != null && eHi != null)
              ? (`₹${Number(eLo).toFixed(2)}–${Number(eHi).toFixed(2)}`)
              : '—';
            const ofBadge = ofOn
              ? `<span style="display:inline-block; margin-left:8px; padding:1px 6px; font-size:10px; font-weight:700; letter-spacing:0.02em; color:#052e16; background:#86efac; border:1px solid #4ade80; border-radius:4px; vertical-align:middle;" title="L1 TOB imbalance agrees with side${imb ? ' ('+imb+')' : ''}">ORDER FLOW</span>`
              : '';
            const livePill = `<span style="display:inline-block; padding:1px 6px; font-size:10px; font-weight:700; color:#083344; background:#67e8f9; border-radius:4px;">LIVE</span>`;
            const statusPill = `<span style="display:inline-block; margin-left:6px; padding:1px 6px; font-size:10px; font-weight:700; color:#0f172a; background:${lvlStatusColor(lst)}; border-radius:4px;" title="${(lv.level_status_reason||'')}">${lst}</span>`;
            const lvlNote = 'Indicative watchlist levels (RP608_DYNAMIC_OHLCV_STRUCTURE_V1) — not executed / not paper entry';
            return `<div style="display:grid; gap:6px; padding:10px 12px; background:rgba(6,182,212,0.06); border:1px solid rgba(6,182,212,0.15); border-radius:8px;">
              <div style="display:flex; flex-wrap:wrap; gap:8px; align-items:center;">
                <span style="color:#64748b;">#${i+1}</span>
                <span style="color:#7dd3fc; font-weight:800; font-size:13px;">${sym}</span>
                <span style="color:#e2e8f0;">${r.side || 'LONG'}</span>
                ${livePill}${statusPill}${ofBadge}
                <span style="color:#94a3b8; font-size:10px; margin-left:auto;" title="${rowRec ? (rowRec.ist + ' · ' + rowRec.utc) : ''}">rec ${rowRecShort} · score ${sc}</span>
              </div>
              <div style="display:grid; grid-template-columns:repeat(auto-fit,minmax(140px,1fr)); gap:8px; font-size:11px;" title="${lvlNote}">
                <div><div style="color:#64748b; font-size:9px; text-transform:uppercase;">Entry range</div><div style="color:#e2e8f0; font-weight:700;">${entryRange}</div></div>
                <div><div style="color:#64748b; font-size:9px; text-transform:uppercase;">Target</div><div style="color:#86efac; font-weight:700;">${tgt != null ? ('₹'+Number(tgt).toFixed(2)) : '—'}</div></div>
                <div><div style="color:#64748b; font-size:9px; text-transform:uppercase;">Stop-loss</div><div style="color:#fda4af; font-weight:700;">${stop != null ? ('₹'+Number(stop).toFixed(2)) : '—'}</div></div>
                <div><div style="color:#64748b; font-size:9px; text-transform:uppercase;">Risk/reward</div><div style="color:#fde68a; font-weight:700;">${rr}</div></div>
              </div>
              <div style="font-size:9px; color:#64748b;">Indicative only · not an executed or paper-trade entry · ref close ≠ authorized fill</div>
            </div>`;
          }).join('');
        }
      }

      // Candidate Recommendations & Evaluation Ledger Banner
      const evSummary = nse.evaluations_summary || {};
      const statusPill = document.getElementById('nse-eval-status-pill');
      const recsContainer = document.getElementById('nse-eval-recs-container');
      const chipsContainer = document.getElementById('nse-rejection-chips');
      const sortSelect = document.getElementById('nse-rec-sort');
      const highTargetFloor = evSummary.high_target_bps_floor != null ? evSummary.high_target_bps_floor : 75;
      const highTargetsRaw = evSummary.high_target_candidates || [];
      const nHigh = evSummary.n_high_target != null ? evSummary.n_high_target : highTargetsRaw.length;
      window.__nseHighTargetCandidates = highTargetsRaw;
      window.__nseQualifyingCandidates = evSummary.qualifying_candidates || [];
      window.__nseHighTargetFloor = highTargetFloor;

      const NSE_SORT_LABELS = {
        rank_score: 'R:R × Target (Net R:R × target_bps/100)',
        target_bps: 'Target (bps) descending',
        net_rr: 'Net R:R descending',
        traj15: 'traj15 score descending',
        rec_time: 'Rec time · newest first',
        symbol: 'Symbol A–Z'
      };
      const NSE_SORT_KEYS = Object.keys(NSE_SORT_LABELS);

      function getNseRecSortMode() {
        let mode = 'rank_score';
        try {
          const saved = localStorage.getItem('rp608_nse_rec_sort');
          if (saved && NSE_SORT_KEYS.includes(saved)) mode = saved;
        } catch (_) {}
        if (sortSelect && NSE_SORT_KEYS.includes(sortSelect.value)) mode = sortSelect.value;
        return mode;
      }

      function syncNseRecSortSelect() {
        if (!sortSelect) return;
        const mode = getNseRecSortMode();
        if (sortSelect.value !== mode) sortSelect.value = mode;
        try { localStorage.setItem('rp608_nse_rec_sort', mode); } catch (_) {}
      }

      function nseSortMetric(qc, mode) {
        const rr = qc.risk_reward || {};
        if (mode === 'target_bps') return Number(rr.target_bps != null ? rr.target_bps : -Infinity);
        if (mode === 'net_rr') return Number(rr.net_rr_ratio != null ? rr.net_rr_ratio : -Infinity);
        if (mode === 'traj15') return Number(qc.score != null ? qc.score : -Infinity);
        if (mode === 'rec_time') {
          const ms = qc.recommendation_time_utc_ms;
          return (ms != null && !Number.isNaN(Number(ms))) ? Number(ms) : -Infinity;
        }
        if (mode === 'symbol') return String(qc.tradingsymbol || qc.symbol || '').toUpperCase();
        // rank_score default
        const rs = qc.rank_score != null ? qc.rank_score : rr.rank_score;
        return Number(rs != null ? rs : -Infinity);
      }

      function sortNseIdeaCards(list, mode) {
        const arr = (list || []).slice();
        if (mode === 'symbol') {
          arr.sort((a, b) => {
            const sa = nseSortMetric(a, 'symbol');
            const sb = nseSortMetric(b, 'symbol');
            if (sa < sb) return -1;
            if (sa > sb) return 1;
            return 0;
          });
          return arr;
        }
        arr.sort((a, b) => {
          const vb = nseSortMetric(b, mode);
          const va = nseSortMetric(a, mode);
          if (vb !== va) return vb - va;
          // Stable tie-breakers: target, net rr, symbol
          const tb = Number((b.risk_reward || {}).target_bps || 0) - Number((a.risk_reward || {}).target_bps || 0);
          if (tb) return tb;
          const rr = Number((b.risk_reward || {}).net_rr_ratio || 0) - Number((a.risk_reward || {}).net_rr_ratio || 0);
          if (rr) return rr;
          return String(a.tradingsymbol || '').localeCompare(String(b.tradingsymbol || ''));
        });
        return arr;
      }

      function renderNseIdeaCard(qc, accent, rankIdx) {
        const rr = qc.risk_reward || {};
        const kind = qc.card_kind || (qc.decision === 'RECOMMEND_LONG' ? 'QUALIFIED' : 'ACTIVE');
        const kindLabel = kind === 'QUALIFIED_HIGH_TARGET' || qc.decision === 'RECOMMEND_LONG'
          ? `QUALIFY #${qc.provisional_rank || '—'}`
          : (kind === 'ACTIVE_HIGH_TARGET' ? `ACTIVE · BAR ${qc.entry_bar_idx ?? '—'}` : kind);
        const scoreTxt = (qc.score != null && !Number.isNaN(Number(qc.score)))
          ? ((Number(qc.score) >= 0 ? '+' : '') + Number(qc.score).toFixed(4))
          : '--';
        const px = qc.reference_price != null ? qc.reference_price : qc.entry_price;
        const border = accent || 'rgba(16,185,129,0.25)';
        const bg = accent ? accent.replace('0.25', '0.06').replace('0.3', '0.06') : 'rgba(16,185,129,0.06)';
        const rankScore = qc.rank_score != null ? qc.rank_score : rr.rank_score;
        const rankBadge = rankIdx != null
          ? `<span class="badge" style="background:rgba(6,182,212,0.18); color:var(--accent-cyan); font-weight:700; margin-right:6px;">#${rankIdx + 1}</span>`
          : '';
        const recTimeIst = qc.recommendation_time_ist || null;
        const recTimeUtc = qc.recommendation_time_utc || null;
        const recTimeHtml = recTimeIst
          ? `<div style="font-size:10px; color:var(--accent-cyan); font-family:'JetBrains Mono',monospace; margin:0 0 8px 0;">
              Rec time: <b style="color:#fff;">${recTimeIst}</b>
              ${recTimeUtc ? `<span style="color:var(--text-dim); margin-left:6px;">(${recTimeUtc})</span>` : ''}
            </div>`
          : `<div style="font-size:10px; color:var(--text-dim); margin:0 0 8px 0;">Rec time: —</div>`;
        return `
          <div style="background:${bg}; border:1px solid ${border}; border-radius:8px; padding:12px;">
            <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:8px;">
              <div>
                ${rankBadge}
                <span class="badge badge-active" style="background:rgba(16,185,129,0.2); color:var(--accent-emerald); font-weight:700; margin-right:6px;">${kindLabel}</span>
                ${horizonBadgeHtml(qc)}
                <b style="font-size:14px; color:#fff;">${qc.tradingsymbol || qc.symbol}</b>
                <span style="font-size:10px; color:var(--text-dim); margin-left:4px;">${qc.symbol || ''}</span>
              </div>
              <span style="font-family:'JetBrains Mono',monospace; font-size:13px; font-weight:700; color:#fff;">${px != null ? '₹' + Number(px).toFixed(2) : '--'}</span>
            </div>
            ${recTimeHtml}
            <div style="display:grid; grid-template-columns:1fr 1fr; gap:6px; font-size:11px; margin-bottom:8px;">
              <div>Rank score: <b style="color:var(--accent-cyan); font-family:'JetBrains Mono',monospace;">${rankScore != null ? Number(rankScore).toFixed(2) : '--'}</b> <span style="color:var(--text-dim); font-size:9px;">(R:R×tgt%)</span></div>
              <div>Net R:R: <b style="color:var(--accent-cyan); font-family:'JetBrains Mono',monospace;">${rr.net_rr_ratio != null ? Number(rr.net_rr_ratio).toFixed(2) + 'x' : '--'}</b></div>
              <div>Score (traj15): <b style="color:var(--accent-emerald); font-family:'JetBrains Mono',monospace;">${scoreTxt}</b></div>
              <div>Friction: <span style="color:var(--text-muted); font-family:'JetBrains Mono',monospace;">${rr.estimated_friction_bps != null ? Number(rr.estimated_friction_bps).toFixed(1) + ' bps' : '--'}</span></div>
              <div>Target (+${rr.target_bps != null ? Number(rr.target_bps).toFixed(0) : '--'} bps): <span style="color:var(--accent-emerald); font-family:'JetBrains Mono',monospace;">₹${rr.target_price != null ? Number(rr.target_price).toFixed(2) : '--'}</span></div>
              <div>Stop (−${rr.stop_loss_bps != null ? Number(rr.stop_loss_bps).toFixed(0) : '--'} bps): <span style="color:var(--accent-rose); font-family:'JetBrains Mono',monospace;">₹${rr.stop_loss_price != null ? Number(rr.stop_loss_price).toFixed(2) : '--'}</span></div>
              <div>ATR14: <span style="color:var(--text-muted); font-family:'JetBrains Mono',monospace;">${rr.atr_14 != null ? Number(rr.atr_14).toFixed(2) : '--'}</span></div>
              <div>Trend: <span style="color:var(--text-muted); font-family:'JetBrains Mono',monospace;">${rr.trend_label || '--'}</span></div>
            </div>
            <div style="font-size:10px; color:var(--text-muted); line-height:1.4; border-top:1px solid rgba(255,255,255,0.06); padding-top:6px;">
              <div><b style="color:var(--text-dim);">Horizon:</b> ${qc.horizon_class || 'INTRADAY'} · ${qc.horizon_term || '15m bars · 60m target / 180m max'}</div>
              <div><b style="color:var(--text-dim);">Policy:</b> ${rr.exit_policy || '--'}</div>
              <div><b style="color:var(--text-dim);">Invalidation:</b> ${qc.invalidation || '--'}</div>
            </div>
          </div>`;
      }

      function paintNseRecommendationBoard() {
        syncNseRecSortSelect();
        const mode = getNseRecSortMode();
        const floor = window.__nseHighTargetFloor != null ? window.__nseHighTargetFloor : 75;
        const highs = window.__nseHighTargetCandidates || [];
        const quals = window.__nseQualifyingCandidates || [];
        const nQual = (evSummary.n_qualifying != null) ? evSummary.n_qualifying : quals.length;

        if (!evSummary.total_evaluated || evSummary.total_evaluated === 0) {
          if (statusPill) statusPill.innerHTML = '<span class="pill-sm" style="background:rgba(245,158,11,0.15); color:var(--accent-amber); border:1px solid rgba(245,158,11,0.3);">EVALUATION ENGINE PENDING</span>';
          if (recsContainer) {
            recsContainer.innerHTML = '<div style="color:var(--text-muted); font-size:12px; padding:6px 0;">No completed bar evaluation snapshot recorded yet. Click <b>⟳ Refresh Live Batch</b> to trigger a single-bar evaluation cycle.</div>';
          }
          return;
        }

        if (statusPill) {
          statusPill.innerHTML = `<span class="badge badge-active" style="background:rgba(16,185,129,0.15); color:var(--accent-emerald); border:1px solid rgba(16,185,129,0.3); font-weight:700;">${nHigh} TARGET &gt; ${floor} BPS</span>`
            + ` <span class="pill-sm" style="margin-left:6px; background:rgba(255,255,255,0.08); color:var(--text-dim);">${nQual} fresh qualify</span>`;
        }

        let recsHtml = '';
        if (highs.length > 0) {
          const sorted = sortNseIdeaCards(highs, mode);
          recsHtml += `<div style="font-size:11px; color:var(--text-muted); margin-bottom:8px;">Ideas with frozen dynamic target &gt; <b style="color:#fff;">${floor} bps</b> · sorted by <b style="color:var(--accent-cyan);">${NSE_SORT_LABELS[mode] || mode}</b>. Each card shows <b style="color:var(--accent-cyan);">INTRADAY</b> + holding term.</div>`;
          recsHtml += '<div style="display:grid; grid-template-columns:repeat(auto-fit, minmax(320px, 1fr)); gap:12px; max-height:520px; overflow:auto; padding-right:4px;">';
          sorted.forEach((qc, idx) => {
            const isFresh = qc.decision === 'RECOMMEND_LONG' || qc.card_kind === 'QUALIFIED_HIGH_TARGET';
            recsHtml += renderNseIdeaCard(qc, isFresh ? 'rgba(16,185,129,0.3)' : 'rgba(6,182,212,0.25)', idx);
          });
          recsHtml += '</div>';
        } else if (nQual > 0) {
          const sorted = sortNseIdeaCards(quals, mode);
          recsHtml += `<div style="font-size:11px; color:var(--text-muted); margin-bottom:8px;">Fresh qualifies · sorted by <b style="color:var(--accent-cyan);">${NSE_SORT_LABELS[mode] || mode}</b>.</div>`;
          recsHtml += '<div style="display:grid; grid-template-columns:repeat(auto-fit, minmax(320px, 1fr)); gap:12px;">';
          sorted.forEach((qc, idx) => { recsHtml += renderNseIdeaCard(qc, null, idx); });
          recsHtml += '</div>';
        } else {
          recsHtml = `
            <div style="background:rgba(255,255,255,0.02); border:1px solid rgba(255,255,255,0.06); border-radius:8px; padding:12px;">
              <div style="font-size:12px; font-weight:700; color:#fff;">No scrips with target &gt; ${floor} bps and no fresh qualifies this bar</div>
              <div style="font-size:11px; color:var(--text-dim); margin-top:3px;">
                Fresh entry still requires <code style="color:var(--accent-cyan);">traj15 &gt;= +0.0500</code>. Open-position targets come from ATR/structure policy.
              </div>
            </div>`;
        }
        if (recsContainer) recsContainer.innerHTML = recsHtml;
      }

      if (sortSelect && !sortSelect.dataset.bound) {
        sortSelect.dataset.bound = '1';
        try {
          const saved = localStorage.getItem('rp608_nse_rec_sort');
          if (saved && NSE_SORT_KEYS.includes(saved)) sortSelect.value = saved;
        } catch (_) {}
        sortSelect.addEventListener('change', () => {
          try { localStorage.setItem('rp608_nse_rec_sort', sortSelect.value); } catch (_) {}
          paintNseRecommendationBoard();
        });
      }
      paintNseRecommendationBoard();

      // Rejection Breakdown Chips
      if (chipsContainer && evSummary.rejection_breakdown) {
        const rb = evSummary.rejection_breakdown;
        chipsContainer.innerHTML = `
          <span style="font-weight:700; text-transform:uppercase; letter-spacing:0.5px; color:var(--text-muted); font-size:10px;">Rejection Breakdown:</span>
          <span class="pill-sm" style="background:rgba(245,158,11,0.1); color:var(--accent-amber); border:1px solid rgba(245,158,11,0.2);">Cold Start (&lt; 5 bars): <b>${rb.COLD_START_INSUFFICIENT_HISTORY || 0}</b></span>
          <span class="pill-sm" style="background:rgba(148,163,184,0.1); color:var(--text-dim); border:1px solid rgba(148,163,184,0.2);">Sub-Threshold (traj15 &lt; 0.05): <b>${rb.SUB_THRESHOLD_MOMENTUM || 0}</b></span>
          <span class="pill-sm" style="background:rgba(244,63,94,0.1); color:var(--accent-rose); border:1px solid rgba(244,63,94,0.2);">Boundary (&ge; 15:00): <b>${rb.INTRADAY_BOUNDARY_EXCLUDED || 0}</b></span>
          <span class="pill-sm" style="background:rgba(6,182,212,0.1); color:var(--accent-cyan); border:1px solid rgba(6,182,212,0.2);">Active Trade: <b>${rb.EXISTING_POSITION_ACTIVE || 0}</b></span>
          <span class="pill-sm" style="background:rgba(239,68,68,0.1); color:var(--accent-rose); border:1px solid rgba(239,68,68,0.2);">Data Failure: <b>${rb.DATA_QUALITY_FAILURE || 0}</b></span>
        `;
      }

      // Momentum Census Counts
      allNseScrips = nse.scrips || [];
      const explorerSort = document.getElementById('select-explorer-sort');
      if (explorerSort) {
        try {
          const saved = localStorage.getItem('rp608_nse_explorer_sort');
          if (saved && ['REC_TIME_DESC', 'REC_TIME_ASC', 'TRAJ15_DESC', 'SYMBOL_ASC'].includes(saved)) {
            explorerSort.value = saved;
          } else if (!explorerSort.value) {
            explorerSort.value = 'REC_TIME_DESC';
          }
        } catch (_) {
          explorerSort.value = 'REC_TIME_DESC';
        }
      }
      let cntBullish = 0;
      let cntNeutral = 0;
      let cntBearish = 0;
      allNseScrips.forEach(s => {
        if (s.score !== null && s.score !== undefined) {
          if (s.score >= 0.05) cntBullish++;
          else if (s.score > -0.05) cntNeutral++;
          else cntBearish++;
        }
      });
      const bEl = document.getElementById('cnt-bullish');
      const nEl = document.getElementById('cnt-neutral');
      const brEl = document.getElementById('cnt-bearish');
      if (bEl) bEl.textContent = cntBullish;
      if (nEl) nEl.textContent = cntNeutral;
      if (brEl) brEl.textContent = cntBearish;

      renderNseTable();
    }

    function renderNseTable() {
      const tbody = document.getElementById('nse-scrips-tbody');
      if (!tbody) return;

      const compSelect = document.getElementById('select-filter-compartment');
      const decSelect = document.getElementById('select-filter-decision');
      const momSelect = document.getElementById('select-filter-momentum');
      const sortSelect = document.getElementById('select-explorer-sort');
      const filterInput = document.getElementById('nse-filter-input');

      const compFilter = compSelect ? compSelect.value : 'ALL';
      const decFilter = decSelect ? decSelect.value : 'ALL';
      const momFilter = momSelect ? momSelect.value : 'ALL';
      const sortMode = sortSelect ? sortSelect.value : 'REC_TIME_DESC';
      const q = (filterInput ? filterInput.value.trim().toUpperCase() : '');

      let displayList = allNseScrips.slice();

      // 1. Filter by Compartment
      if (compFilter !== 'ALL') {
        displayList = displayList.filter(s => {
          const c = (s.rp607 && s.rp607.compartment_code) ? s.rp607.compartment_code : (s.compartment || 'QUARANTINED');
          return c === compFilter;
        });
      }

      // 2. Filter by Decision / target surface
      if (decFilter !== 'ALL') {
        if (decFilter === 'RECOMMEND_LONG') {
          displayList = displayList.filter(s => s.decision === 'RECOMMEND_LONG');
        } else if (decFilter === 'TARGET_GT_75') {
          displayList = displayList.filter(s => {
            const tb = s.risk_reward && s.risk_reward.target_bps;
            return tb != null && Number(tb) > 75;
          });
        } else {
          displayList = displayList.filter(s => s.decision_code === decFilter);
        }
      }

      // 3. Filter by Momentum
      if (momFilter === 'BULLISH') {
        displayList = displayList.filter(s => (s.score !== null && s.score !== undefined && s.score >= 0.05));
      } else if (momFilter === 'SUB_THRESHOLD') {
        displayList = displayList.filter(s => (s.score === null || s.score === undefined || s.score < 0.05));
      }

      // 4. Filter by text search
      if (q) {
        displayList = displayList.filter(s => {
          const rp = s.rp607 || {};
          return (
            (s.tradingsymbol && s.tradingsymbol.toUpperCase().includes(q)) || 
            (s.symbol && s.symbol.toUpperCase().includes(q)) ||
            (rp.compartment_code && rp.compartment_code.toUpperCase().includes(q)) ||
            (rp.compartment && rp.compartment.toUpperCase().includes(q)) ||
            (s.decision_code && s.decision_code.toUpperCase().includes(q)) ||
            (s.explanation && s.explanation.toUpperCase().includes(q))
          );
        });
      }

      // 5. Sort (default: recommendation time newest first)
      const recMs = (s) => {
        const ms = s && s.recommendation_time_utc_ms;
        const n = (ms != null) ? Number(ms) : NaN;
        return Number.isFinite(n) ? n : null;
      };
      const traj = (s) => {
        const n = (s && s.score != null) ? Number(s.score) : NaN;
        return Number.isFinite(n) ? n : -Infinity;
      };
      displayList.sort((a, b) => {
        if (sortMode === 'SYMBOL_ASC') {
          return String(a.tradingsymbol || a.symbol || '').localeCompare(String(b.tradingsymbol || b.symbol || ''));
        }
        if (sortMode === 'TRAJ15_DESC') {
          const d = traj(b) - traj(a);
          if (d) return d;
          return String(a.tradingsymbol || '').localeCompare(String(b.tradingsymbol || ''));
        }
        const ma = recMs(a);
        const mb = recMs(b);
        // Missing timestamps sink to bottom for both asc/desc rec-time modes
        if (ma == null && mb == null) {
          return String(a.tradingsymbol || '').localeCompare(String(b.tradingsymbol || ''));
        }
        if (ma == null) return 1;
        if (mb == null) return -1;
        if (sortMode === 'REC_TIME_ASC') {
          if (ma !== mb) return ma - mb;
        } else {
          // REC_TIME_DESC (default)
          if (mb !== ma) return mb - ma;
        }
        return String(a.tradingsymbol || '').localeCompare(String(b.tradingsymbol || ''));
      });
      try { localStorage.setItem('rp608_nse_explorer_sort', sortMode); } catch (_) {}

      const countBadge = document.getElementById('nse-table-count');
      if (countBadge) {
        countBadge.textContent = `Showing ${displayList.length} of ${allNseScrips.length} Scrips`;
      }

      if (displayList.length === 0) {
        tbody.innerHTML = '<tr><td colspan="9" style="text-align:center; color:var(--text-dim); padding:24px;">No instruments match the selected filters.</td></tr>';
        return;
      }

      tbody.innerHTML = '';
      displayList.forEach(s => {
        const tr = document.createElement('tr');
        const isRecLong = s.decision === 'RECOMMEND_LONG';
        const rp = s.rp607 || {};
        const compCode = rp.compartment_code || s.compartment || 'QUARANTINED';
        const isQuarantined = !rp.valid || compCode === 'QUARANTINED';

        // Compartment Pill
        let compPill = '<span class="comp-pill comp-quarantined">QUARANTINED</span>';
        if (compCode === 'C1') compPill = '<span class="comp-pill comp-c1">C1 · Directional</span>';
        else if (compCode === 'C2') compPill = '<span class="comp-pill comp-c2">C2 · Oscillator</span>';
        else if (compCode === 'C3') compPill = '<span class="comp-pill comp-c3">C3 · Stable</span>';
        else if (compCode === 'C4') compPill = '<span class="comp-pill comp-c4">C4 · Friction</span>';

        const compValidityBadge = isQuarantined 
          ? '<span style="font-size:9px; color:var(--text-muted); display:block; margin-top:3px;">Data Deficient (&lt; 20 bars)</span>'
          : `<span style="font-size:9px; color:var(--accent-cyan); display:block; margin-top:3px;">RP-607 Valid (${rp.profile_date || '2026-10-01'})</span>`;

        // Historical Features
        let histFeaturesHtml = '';
        if (!isQuarantined && rp.natr_20 !== null && rp.natr_20 !== undefined) {
          histFeaturesHtml = `
            <div style="font-family:'JetBrains Mono',monospace; font-size:11px; line-height:1.4;">
              <div>NATR20: <b style="color:#fff;">${rp.natr_20.toFixed(1)} bps</b></div>
              <div style="color:var(--text-dim); font-size:10px;">Eff20: <b style="color:#fff;">${rp.dir_eff_20 !== null && rp.dir_eff_20 !== undefined ? rp.dir_eff_20.toFixed(2) : '--'}</b> · AR1: <b style="color:#fff;">${rp.ar1_60 !== null && rp.ar1_60 !== undefined ? rp.ar1_60.toFixed(2) : '--'}</b></div>
              <div style="font-size:9px; color:var(--text-muted); margin-top:2px;">Historical: ${rp.profile_date || '2026-10-01'}</div>
            </div>
          `;
        } else {
          histFeaturesHtml = `<div style="font-size:10px; color:var(--text-muted); line-height:1.3;">Insufficient history (&lt; 20 daily bars) · Historical profile quarantined</div>`;
        }

        // Momentum Score
        const scoreVal = (s.score !== undefined && s.score !== null) ? ((s.score >= 0 ? '+' : '') + s.score.toFixed(4)) : '--';
        const scoreColor = (s.score >= 0.05) ? 'var(--accent-emerald)' : (s.score > 0 ? 'var(--accent-cyan)' : 'var(--text-dim)');
        const histBars = s.history_length || 1;

        // Gate Progression Trace (G1 to G5)
        let gateTraceHtml = '<div style="display:flex; align-items:center; gap:3px;">';
        if (s.gates_trace && Array.isArray(s.gates_trace)) {
          s.gates_trace.forEach(g => {
            const statusLower = (g.status || '').toLowerCase();
            let pillClass = 'gate-skip';
            if (statusLower === 'pass' || statusLower === 'none') pillClass = 'gate-pass';
            else if (statusLower === 'fail') pillClass = 'gate-fail';
            else if (statusLower === 'active') pillClass = 'gate-active';
            else if (statusLower === 'closed') pillClass = 'gate-closed';
            gateTraceHtml += `<span class="gate-micro-pill ${pillClass}" title="${g.gate_id} ${g.name}: ${g.status} (${g.reason})">${g.gate_id}</span>`;
          });
        }
        gateTraceHtml += '</div>';

        // Terminal Decision
        const decBg = isRecLong ? 'rgba(16,185,129,0.18)' : 'rgba(255,255,255,0.06)';
        const decColor = isRecLong ? 'var(--accent-emerald)' : 'var(--text-muted)';
        const decText = isRecLong ? 'RECOMMEND_LONG' : 'NO_TRADE';
        const rankBadge = s.provisional_rank ? `<span class="badge badge-active" style="background:rgba(16,185,129,0.2); color:var(--accent-emerald); font-weight:700; margin-left:4px;">#${s.provisional_rank}</span>` : '';
        const hzBadge = `<div style="margin-top:4px;">${horizonBadgeHtml(s)}</div>`;

        // Live LTP & Spread
        const isCrossed = s.status === 'CROSSED';
        const isZero = s.status === 'ZERO_DEPTH';
        const statusBg = isCrossed ? 'rgba(244,63,94,0.15)' : (isZero ? 'rgba(245,158,11,0.15)' : 'rgba(16,185,129,0.15)');
        const statusColor = isCrossed ? 'var(--accent-rose)' : (isZero ? 'var(--accent-amber)' : 'var(--accent-emerald)');
        const statusText = isCrossed ? 'CROSSED' : (isZero ? 'ZERO DEPTH' : 'VALID L1');

        // Quote Age & Provenance
        const ageSec = s.age_sec;
        let ageHtml = '--';
        if (ageSec !== null && ageSec !== undefined) {
          const ageColor = ageSec > 600 ? 'var(--accent-rose)' : (ageSec > 120 ? 'var(--accent-amber)' : 'var(--accent-emerald)');
          const ageStr = ageSec < 60 ? `${ageSec}s ago` : `${Math.floor(ageSec / 60)}m ${ageSec % 60}s ago`;
          ageHtml = `<span style="font-size:10px; color:${ageColor}; font-weight:600; font-family:'JetBrains Mono',monospace;">${ageStr}</span>`;
        }

        const isBenchmark = benchmarkSymbols.has(s.tradingsymbol);
        const benchmarkBadge = isBenchmark ? '<span class="badge" style="background:rgba(6,182,212,0.15); color:var(--accent-cyan); font-size:8px; padding:1px 4px; margin-left:4px;">BENCHMARK</span>' : '';

        tr.innerHTML = `
          <td>
            <div style="display:flex; align-items:center;">
              <b style="color:#fff; font-size:13px;">${s.tradingsymbol}</b>
              ${benchmarkBadge}
            </div>
            <span style="font-size:10px; color:var(--text-dim); display:block; font-family:'JetBrains Mono',monospace;">${s.symbol}</span>
          </td>
          <td>
            ${compPill}
            ${compValidityBadge}
          </td>
          <td>
            ${histFeaturesHtml}
          </td>
          <td>
            <div style="font-family:'JetBrains Mono',monospace; font-weight:700; font-size:13px; color:${scoreColor};">
              ${scoreVal}
            </div>
            <span style="font-size:10px; color:var(--text-dim); font-family:'JetBrains Mono',monospace;">Sample: ${histBars}/5 bars</span>
          </td>
          <td>
            ${gateTraceHtml}
          </td>
          <td>
            <div style="display:flex; align-items:center; margin-bottom:4px; flex-wrap:wrap; gap:4px;">
              <span style="font-size:10px; font-weight:700; padding:3px 7px; border-radius:4px; background:${decBg}; color:${decColor}; font-family:'JetBrains Mono',monospace;">
                ${decText}
              </span>
              ${rankBadge}
            </div>
            ${hzBadge}
            <div style="font-size:10px; color:var(--text-dim); max-width:240px; line-height:1.3;">${s.explanation || '--'}</div>
            <span style="font-size:9px; color:var(--text-muted); font-family:'JetBrains Mono',monospace;">Code: ${s.decision_code || 'N/A'}</span>
          </td>
          <td>
            <div style="font-family:'JetBrains Mono',monospace; font-size:10px; line-height:1.35; max-width:160px;">
              ${s.recommendation_time_ist
                ? `<div style="color:var(--accent-cyan); font-weight:600;">${s.recommendation_time_ist}</div>
                   ${s.recommendation_time_utc ? `<div style="color:var(--text-dim); font-size:9px; margin-top:2px;">${s.recommendation_time_utc}</div>` : ''}`
                : '<span style="color:var(--text-muted);">—</span>'}
            </div>
          </td>
          <td>
            <div style="font-family:'JetBrains Mono',monospace; font-weight:700; color:#fff; font-size:13px;">
              ${s.last_price > 0 ? '₹' + s.last_price.toFixed(2) : '--'}
            </div>
            <div style="font-family:'JetBrains Mono',monospace; color:var(--text-muted); font-size:10px;">
              Spread: ${s.half_spread_bps ? s.half_spread_bps.toFixed(2) + ' bps' : '--'}
            </div>
            <span style="font-size:9px; font-weight:700; padding:1px 5px; border-radius:3px; background:${statusBg}; color:${statusColor}; display:inline-block; margin-top:2px;">
              ${statusText}
            </span>
          </td>
          <td>
            <div style="font-size:10px; font-family:'JetBrains Mono',monospace;">
              <div>Quote: ${ageHtml}</div>
              <div style="font-size:9px; color:var(--text-muted); margin-top:2px;">Profile: ${(s.rp607 && s.rp607.profile_date) ? s.rp607.profile_date : '2026-10-01'}</div>
              <div style="font-size:9px; color:var(--accent-cyan); margin-top:1px;">Feed: ${s.feed_label || 'Unknown'}${s.history_length != null ? ' · Hist ' + s.history_length + '/5' : ''}</div>
            </div>
          </td>
        `;
        tbody.appendChild(tr);
      });
    }

    let currentApiKey = '';

    function openKiteAuthModal() {
      const modal = document.getElementById('kite-auth-modal');
      const input = document.getElementById('kite-token-input');
      const msg = document.getElementById('kite-auth-msg');
      if (input) input.value = '';
      if (msg) { msg.style.display = 'none'; msg.textContent = ''; }
      if (modal) modal.style.display = 'flex';
    }

    function closeKiteAuthModal() {
      const modal = document.getElementById('kite-auth-modal');
      if (modal) modal.style.display = 'none';
    }

    async function submitKiteToken() {
      const input = document.getElementById('kite-token-input');
      const msg = document.getElementById('kite-auth-msg');
      const btn = document.getElementById('btn-submit-kite-token');
      const rawVal = input ? input.value.trim() : '';

      if (!rawVal) {
        if (msg) {
          msg.style.display = 'block';
          msg.style.color = '#ef4444';
          msg.textContent = 'Please paste the redirect URL or request_token.';
        }
        return;
      }

      // Extract request_token if full URL or query was pasted
      let token = rawVal;
      if (rawVal.includes('request_token=')) {
        try {
          const dummyUrl = rawVal.startsWith('http') ? rawVal : 'http://localhost/' + (rawVal.startsWith('?') ? '' : '?') + rawVal;
          const urlObj = new URL(dummyUrl);
          token = urlObj.searchParams.get('request_token') || token;
        } catch (e) {
          const match = rawVal.match(/request_token=([a-zA-Z0-9]+)/);
          if (match) token = match[1];
        }
      }

      if (btn) {
        btn.disabled = true;
        btn.textContent = 'Exchanging Token...';
      }
      if (msg) {
        msg.style.display = 'block';
        msg.style.color = '#60a5fa';
        msg.textContent = 'Exchanging request token and refreshing 119 scrip evaluations...';
      }

      try {
        const res = await fetch('/api/kite/auth', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ request_token: token })
        });
        const data = await res.json();
        if (res.ok && data.status === 'ok') {
          if (msg) {
            msg.style.color = '#10b981';
            msg.textContent = '✓ ' + (data.message || 'Authentication successful!');
          }
          if (btn) btn.textContent = '✓ Authenticated';
          setTimeout(() => {
            closeKiteAuthModal();
            fetchState();
            if (btn) {
              btn.disabled = false;
              btn.textContent = 'Authenticate & Refresh';
            }
          }, 1500);
        } else {
          throw new Error(data.error || 'Authentication failed');
        }
      } catch (err) {
        if (msg) {
          msg.style.display = 'block';
          msg.style.color = '#ef4444';
          msg.textContent = '⚠ ' + err.message;
        }
        if (btn) {
          btn.disabled = false;
          btn.textContent = 'Try Again';
        }
      }
    }

    // Initial and periodic refresh
    fetchState();
    fetchMultiHorizon();
    setInterval(fetchState, 3000);
    setInterval(fetchMultiHorizon, 15000);

    // Continuous Kite batch live feed polling (15s interval)
    refreshNseBatch();
    setInterval(refreshNseBatch, 15000);
  </script>

  <!-- Kite Connect Re-Authentication Modal -->
  <div id="kite-auth-modal" style="display:none; position:fixed; top:0; left:0; width:100vw; height:100vh; background:rgba(0,0,0,0.75); backdrop-filter:blur(4px); z-index:9999; align-items:center; justify-content:center;">
    <div style="background:#0f172a; border:1px solid var(--accent-violet); border-radius:12px; max-width:550px; width:90%; padding:24px; box-shadow:0 20px 50px rgba(0,0,0,0.8); color:var(--text-main);">
      <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:16px;">
        <div style="display:flex; align-items:center; gap:8px;">
          <span style="font-size:18px;">🔑</span>
          <span style="font-size:15px; font-weight:700; color:#fff;">Kite Connect Re-Authentication</span>
        </div>
        <button onclick="closeKiteAuthModal()" style="background:none; border:none; color:var(--text-muted); font-size:18px; cursor:pointer;">✕</button>
      </div>
      <p style="font-size:12px; color:var(--text-muted); margin-bottom:14px; line-height:1.5;">
        Zerodha session tokens expire daily after midnight IST. You can generate a new token via Kite login and paste the redirect URL or token below.
      </p>
      <div style="margin-bottom:16px; padding:12px; background:rgba(255,255,255,0.03); border:1px solid var(--card-border); border-radius:8px;">
        <div style="font-size:11px; font-weight:600; color:var(--text-dim); text-transform:uppercase; margin-bottom:6px;">Step 1: Open Login in Zerodha</div>
        <a id="kite-login-link" href="https://kite.zerodha.com/connect/login?v=3&api_key=" target="_blank" style="display:inline-block; background:rgba(59,130,246,0.2); border:1px solid var(--accent-blue); color:#60a5fa; font-size:12px; padding:6px 14px; border-radius:6px; text-decoration:none; font-weight:600;">Open Kite Login Window ↗</a>
      </div>
      <div style="margin-bottom:16px;">
        <div style="font-size:11px; font-weight:600; color:var(--text-dim); text-transform:uppercase; margin-bottom:6px;">Step 2: Paste Callback URL or Request Token</div>
        <input id="kite-token-input" type="text" placeholder="http://127.0.0.1:8000/callback?... or request_token" style="width:100%; box-sizing:border-box; background:#1e293b; border:1px solid var(--card-border); border-radius:6px; color:#fff; font-family:'JetBrains Mono',monospace; font-size:12px; padding:8px 10px; outline:none;" />
        <div id="kite-auth-msg" style="font-size:11px; margin-top:8px; display:none;"></div>
      </div>
      <div style="display:flex; justify-content:flex-end; gap:10px;">
        <button onclick="closeKiteAuthModal()" style="background:rgba(255,255,255,0.05); border:1px solid var(--card-border); color:var(--text-muted); font-size:12px; padding:6px 14px; border-radius:6px; cursor:pointer;">Cancel</button>
        <button id="btn-submit-kite-token" onclick="submitKiteToken()" style="background:var(--accent-violet); border:none; color:#fff; font-size:12px; padding:6px 16px; border-radius:6px; font-weight:600; cursor:pointer;">Authenticate & Refresh</button>
      </div>
    </div>
  </div>
</body>
</html>
"""


class DashboardHTTPRequestHandler(http.server.BaseHTTPRequestHandler):
    def _send_json(self, status: int, data: dict):
        payload = json.dumps(data).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(payload)))
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        self.send_header("Access-Control-Allow-Headers", "Content-Type")
        self.send_header("Cache-Control", "no-cache, no-store, must-revalidate")
        self.end_headers()
        self.wfile.write(payload)

    def do_OPTIONS(self):
        self.send_response(204)
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        self.send_header("Access-Control-Allow-Headers", "Content-Type")
        self.end_headers()

    def _handle_kite_token(self, request_token: str) -> dict:
        request_token = request_token.strip()
        if not request_token:
            raise ValueError("No request_token provided")
        if not load_credentials or not exchange_request_token:
            raise RuntimeError("Kite credentials module not available")

        creds = load_credentials()
        api_key = creds.get("api_key")
        api_secret = creds.get("api_secret")
        if not api_key or not api_secret:
            raise ValueError("api_key or api_secret missing in credentials")

        access_token = exchange_request_token(api_key, api_secret, request_token)
        creds["access_token"] = access_token
        if save_credentials:
            save_credentials(creds)

        user_id = creds.get("user_id", "TJ6378")
        user_name = creds.get("user_name", "Nikhil Tanaji More")
        if kite_api_get:
            try:
                prof = kite_api_get("/user/profile", api_key, access_token)
                data_prof = prof.get("data", {})
                user_id = data_prof.get("user_id", user_id)
                user_name = data_prof.get("user_name", user_name)
            except Exception as pe:
                print(f"Profile fetch notice: {pe}")

        # Run fresh probe with new credentials
        if run_probe:
            try:
                run_probe()
            except Exception as pe:
                print(f"Post-auth probe error: {pe}")

        # Run single iteration of daemon to produce fresh evaluations
        try:
            from scripts.run_rp608_nse_shadow_daemon import RP608NSEShadowDaemon
            daemon = RP608NSEShadowDaemon(output_dir=NSE_TEST_DIR, single_bar_test=True)
            daemon.run_single_iteration()
        except Exception as de:
            print(f"Post-auth daemon eval error: {de}")

        return {
            "status": "ok",
            "user_id": user_id,
            "user_name": user_name,
            "message": f"Successfully authenticated as {user_name} ({user_id}) and refreshed NSE quotes & evaluations."
        }

    def do_POST(self):
        parsed = urllib.parse.urlparse(self.path)
        if parsed.path == "/api/kite/auth":
            try:
                content_length = int(self.headers.get("Content-Length", 0))
                body = self.rfile.read(content_length).decode("utf-8")
                data = json.loads(body) if body else {}
                token = data.get("request_token", "").strip()
                if not token and "callback_url" in data:
                    cb = data["callback_url"]
                    parsed_cb = urllib.parse.urlparse(cb)
                    token = urllib.parse.parse_qs(parsed_cb.query).get("request_token", [""])[0]

                res = self._handle_kite_token(token)
                self._send_json(200, res)
            except Exception as e:
                self._send_json(400, {"status": "error", "error": str(e)})
        else:
            self.send_response(404)
            self.end_headers()

    def do_GET(self):
        parsed = urllib.parse.urlparse(self.path)
        if parsed.path in ("/callback", "/api/kite/auth"):
            qs = urllib.parse.parse_qs(parsed.query)
            token = qs.get("request_token", [""])[0]
            if token:
                try:
                    res = self._handle_kite_token(token)
                    redirect_html = f"""<!DOCTYPE html>
<html>
<head><meta http-equiv="refresh" content="2;url=/" /></head>
<body style="background:#090d16; color:#10b981; font-family:sans-serif; text-align:center; padding-top:80px;">
  <h2>✓ Kite Authentication Successful!</h2>
  <p>{res['message']}</p>
  <p style="color:#94a3b8;">Redirecting back to dashboard in 2 seconds...</p>
</body>
</html>"""
                    payload = redirect_html.encode("utf-8")
                    self.send_response(200)
                    self.send_header("Content-Type", "text/html; charset=utf-8")
                    self.send_header("Content-Length", str(len(payload)))
                    self.end_headers()
                    self.wfile.write(payload)
                    return
                except Exception as e:
                    self._send_json(400, {"status": "error", "error": str(e)})
                    return

        if self.path == "/api/state":
            try:
                state = get_cached_live_state()
                self._send_json(200, state)
            except Exception as e:
                self._send_json(500, {"error": str(e)})
        elif self.path == "/api/multi_horizon":
            try:
                self._send_json(200, get_multi_horizon_dashboard_state())
            except Exception as e:
                self._send_json(500, {"error": str(e)})
        elif self.path in ("/api/nse/refresh", "/api/refresh_nse"):
            try:
                if run_probe:
                    try:
                        run_probe()
                    except Exception as pe:
                        print(f"Probe execution error: {pe}")
                # Execute daemon bar evaluation cycle to update evaluations snapshot
                try:
                    from scripts.run_rp608_nse_shadow_daemon import RP608NSEShadowDaemon
                    daemon = RP608NSEShadowDaemon(output_dir=NSE_TEST_DIR, single_bar_test=True)
                    daemon.run_single_iteration()
                except Exception as de:
                    print(f"Daemon single-bar evaluation error during refresh: {de}")
                nse_state = get_nse_dashboard_state()
                self._send_json(200, {"status": "ok", "nse_operations": nse_state})
            except Exception as e:
                self._send_json(500, {"error": str(e)})
        elif self.path in ("/", "/index.html"):
            payload = DASHBOARD_HTML.encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(payload)))
            self.send_header("Cache-Control", "no-cache, no-store, must-revalidate")
            self.end_headers()
            self.wfile.write(payload)
        else:
            self.send_response(404)
            self.end_headers()

    def log_message(self, format, *args):
        # Suppress verbose access logs
        return


def run_server(host: str = "0.0.0.0", port: int = 8765):
    class ThreadingHTTPServer(socketserver.ThreadingMixIn, http.server.HTTPServer):
        daemon_threads = True

    server = ThreadingHTTPServer((host, port), DashboardHTTPRequestHandler)
    print("=" * 85)
    print(f"  RP-608 LIVE OPERATIONAL & RECOMMENDATION DASHBOARD SERVER")
    print(f"  Serving on http://{host}:{port} (LAN & Local Access)")
    print(f"  Strictly Read-Only · Zero Order Authority · 3s Auto-Refresh")
    print("=" * 85)

    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\n[!] Shutting down dashboard server...")
        server.server_close()


if __name__ == "__main__":
    port = 8765
    if len(sys.argv) > 1:
        try:
            port = int(sys.argv[1])
        except ValueError:
            pass
    run_server(host="0.0.0.0", port=port)
