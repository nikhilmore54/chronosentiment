#!/usr/bin/env python3
"""
RP-608 NSE — next-session intraday watchlist (post-close).

After continuous session close, rank the 119-universe for *next session*
intraday observation using today's completed 15m bars (fetch-backed).

Governance: 0.0% order authority. Not executable overnight / into CAS.
Status: WATCH_NEXT_SESSION — actionable only after next-session warmup
(bar ≥5, within MAX_NEW_ENTRY_BAR), subject to live traj15 / quote gates.
"""

from __future__ import annotations

import json
import math
from dataclasses import asdict, dataclass
from datetime import date, datetime, timedelta, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional, Sequence, Set

PROTOCOL_ID = "RP-608-NSE-NEXT-SESSION-001"
LIVE_ORDER_AUTHORITY_PCT = 0.0
TRAJ15_ENTRY_THRESHOLD = 0.05
TOP_N_DEFAULT = 20
MIN_BARS = 5

# Align with close-window adaptation
MAX_NEW_ENTRY_BAR = 19
MANDATORY_SQUARE_OFF_BAR = 23


@dataclass
class NextSessionCandidate:
    symbol: str
    tradingsymbol: str
    side: str
    decision: str
    status: str
    score: float
    traj15_eod: float
    session_return_bps: float
    close_price: float
    session_bars: int
    thesis: str
    actionable_from_session_date: str
    actionable_from_bar: int
    live_order_authority: float = LIVE_ORDER_AUTHORITY_PCT
    protocol_id: str = PROTOCOL_ID
    horizon_class: str = "INTRADAY"
    horizon_id: str = "H0_NSE_NEXT_SESSION_WATCH"
    source: str = "RP-608-NSE-EOD-FETCH"


def _traj15(closes: Sequence[float]) -> float:
    if len(closes) < 2:
        return 0.0
    window = list(closes[-5:]) if len(closes) >= 5 else list(closes)
    if len(window) < 2 or window[0] <= 0:
        return 0.0
    ret = (window[-1] / window[0]) - 1.0
    denom = 0.01 * math.sqrt(max(len(window) - 1, 1))
    if denom <= 0:
        return 0.0
    return float(math.tanh(ret / denom))


def _session_return_bps(closes: Sequence[float]) -> float:
    if len(closes) < 2 or closes[0] <= 0:
        return 0.0
    return ((float(closes[-1]) / float(closes[0])) - 1.0) * 10000.0


def next_weekday_session(d: date, holidays: Optional[Set[date]] = None) -> date:
    """Next Mon–Fri session date, skipping optional exchange holidays."""
    hol = holidays or set()
    cur = d + timedelta(days=1)
    for _ in range(14):
        if cur.weekday() < 5 and cur not in hol:
            return cur
        cur += timedelta(days=1)
    return d + timedelta(days=1)


def bar_closes_from_history(bars: Sequence[Any]) -> List[float]:
    out: List[float] = []
    for b in bars:
        if isinstance(b, dict):
            c = b.get("close")
        else:
            c = getattr(b, "close", None)
        if c is None:
            continue
        try:
            out.append(float(c))
        except (TypeError, ValueError):
            continue
    return out


def build_next_session_recommendations(
    price_history: Dict[str, Sequence[Any]],
    *,
    from_session_date: str,
    next_session_date: str,
    top_n: int = TOP_N_DEFAULT,
    now_utc: Optional[datetime] = None,
) -> Dict[str, Any]:
    """
    Rank symbols for next-session LONG watch from today's completed bars.
    Deterministic: same histories → same ranked list.
    """
    now = now_utc or datetime.now(timezone.utc)
    candidates: List[NextSessionCandidate] = []

    for sym, bars in sorted(price_history.items()):
        closes = bar_closes_from_history(bars)
        if len(closes) < MIN_BARS:
            continue
        traj = _traj15(closes)
        sess_bps = _session_return_bps(closes)
        # Rank score: emphasize end-of-day traj15 (same family as live entry),
        # with session return as soft tie-break (scaled).
        score = traj + (sess_bps / 10000.0) * 0.25
        if traj < TRAJ15_ENTRY_THRESHOLD and sess_bps < 25.0:
            continue
        side = "LONG"
        thesis = (
            f"EOD traj15={traj:+.4f}, session={sess_bps:+.1f} bps over {len(closes)} bars — "
            f"watch for next-session intraday continuation after warmup"
        )
        candidates.append(
            NextSessionCandidate(
                symbol=str(sym),
                tradingsymbol=str(sym).replace(".NS", ""),
                side=side,
                decision="WATCH_NEXT_SESSION",
                status="WATCH_NEXT_SESSION",
                score=round(score, 6),
                traj15_eod=round(traj, 6),
                session_return_bps=round(sess_bps, 4),
                close_price=round(closes[-1], 4),
                session_bars=len(closes),
                thesis=thesis,
                actionable_from_session_date=next_session_date,
                actionable_from_bar=MIN_BARS,
            )
        )

    candidates.sort(key=lambda c: (-c.score, c.symbol))
    top = candidates[: max(0, int(top_n))]

    return {
        "protocol_id": PROTOCOL_ID,
        "live_order_authority": LIVE_ORDER_AUTHORITY_PCT,
        "emitted_utc": now.isoformat().replace("+00:00", "Z"),
        "from_session_date": from_session_date,
        "next_session_date": next_session_date,
        "closing_window_ist": "15:00-15:15",
        "mandatory_square_off_bar": MANDATORY_SQUARE_OFF_BAR,
        "max_new_entry_bar_next_session": MAX_NEW_ENTRY_BAR,
        "note": (
            "Post-close next-session intraday watchlist from today's kite/15m fetches. "
            "Not executable overnight or in CAS. Next session: re-validate traj15 + L1 "
            f"after bar {MIN_BARS}; no new entries after bar {MAX_NEW_ENTRY_BAR}; "
            f"square-off bar {MANDATORY_SQUARE_OFF_BAR} (15:00)."
        ),
        "universe_scored": len(candidates),
        "n_recommendations": len(top),
        "recommendations": [asdict(c) for c in top],
    }


def write_next_session_recommendations(path: Path, payload: Dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_suffix(".tmp")
    tmp.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    tmp.replace(path)


def load_price_history_json(path: Path) -> Dict[str, List[Dict[str, Any]]]:
    if not path.exists():
        return {}
    try:
        raw = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return {}
    if not isinstance(raw, dict):
        return {}
    out: Dict[str, List[Dict[str, Any]]] = {}
    for k, v in raw.items():
        if isinstance(v, list):
            out[str(k)] = [x for x in v if isinstance(x, dict)]
    return out
