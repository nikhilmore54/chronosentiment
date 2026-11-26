#!/usr/bin/env python3
"""
NSE special-session (Muhurat) calendar helpers.

Regular cash: 09:15–15:30 IST on non-holiday weekdays.
Muhurat 2026-11-08 (Diwali): continuous cash 14:20–15:30 IST (Sunday special).

Does not change Estimand A / OF ±0.30 rules — only session clock gating.
"""

from __future__ import annotations

from dataclasses import dataclass
from datetime import date, datetime, time, timedelta
from typing import Optional, Tuple

IST_OFFSET = timedelta(hours=5, minutes=30)


@dataclass(frozen=True)
class NseSessionWindow:
    session_date: date
    open_ist: time
    close_ist: time
    kind: str  # REGULAR | MUHURAT
    warmup_completed_bars: int
    total_bars: int
    square_off_bar: int
    label: str


# Official NSE Diwali Muhurat 2026 (circular NSE/CMTR/76941): CTS 14:20–15:30 IST
NSE_MUHURAT_SESSIONS = {
    date(2026, 11, 8): NseSessionWindow(
        session_date=date(2026, 11, 8),
        open_ist=time(14, 20),
        close_ist=time(15, 30),
        kind="MUHURAT",
        warmup_completed_bars=1,  # ~70m session; 5-bar regular warmup impossible
        total_bars=5,
        square_off_bar=4,
        label="DIWALI_MUHURAT_2026",
    ),
}


def to_ist_naive(dt: datetime) -> datetime:
    """Normalize to naive IST wall-clock for session comparisons."""
    if dt.tzinfo is not None:
        # Assume caller already shifted to IST wall; drop tz for combine() math.
        return dt.replace(tzinfo=None)
    return dt


def muhurat_window_for(d: date) -> Optional[NseSessionWindow]:
    return NSE_MUHURAT_SESSIONS.get(d)


def resolve_session_window(
    now_ist: datetime,
    *,
    exchange_holidays: Optional[set] = None,
) -> Tuple[Optional[NseSessionWindow], str]:
    """
    Returns (window, closed_reason).
    closed_reason empty when a window applies for this calendar date (even if outside hours).
    """
    now_ist = to_ist_naive(now_ist)
    d = now_ist.date()
    muhurat = muhurat_window_for(d)
    if muhurat is not None:
        return muhurat, ""

    if now_ist.weekday() >= 5:
        return None, "WEEKEND_CLOSED"
    if exchange_holidays is not None and d in exchange_holidays:
        return None, "EXCHANGE_HOLIDAY"

    return (
        NseSessionWindow(
            session_date=d,
            open_ist=time(9, 15),
            close_ist=time(15, 30),
            kind="REGULAR",
            warmup_completed_bars=5,
            total_bars=25,
            square_off_bar=23,
            label="REGULAR_CASH",
        ),
        "",
    )


def session_phase(
    now_ist: datetime,
    window: NseSessionWindow,
) -> Tuple[bool, int, str]:
    """(is_active, bar_idx, phase). bar_idx 1..N while active."""
    now_ist = to_ist_naive(now_ist)
    open_dt = datetime.combine(window.session_date, window.open_ist)
    close_dt = datetime.combine(window.session_date, window.close_ist)
    if now_ist < open_dt:
        return False, 0, f"PRE_OPEN_{window.kind}"
    if now_ist >= close_dt:
        return False, 0, f"POST_CLOSE_{window.kind}"
    elapsed_min = (now_ist - open_dt).total_seconds() / 60.0
    bar_idx = int(elapsed_min // 15) + 1
    bar_idx = min(max(bar_idx, 1), int(window.total_bars))
    phase = "ACTIVE_MUHURAT_SESSION" if window.kind == "MUHURAT" else "ACTIVE_SESSION"
    return True, bar_idx, phase


def completed_bars_since_open(now_ist: datetime, window: NseSessionWindow) -> int:
    now_ist = to_ist_naive(now_ist)
    open_dt = datetime.combine(window.session_date, window.open_ist)
    if now_ist < open_dt:
        return 0
    elapsed_sec = (now_ist - open_dt).total_seconds()
    n = int(elapsed_sec // (15 * 60))
    return min(max(n, 0), int(window.total_bars))
