#!/usr/bin/env python3
"""
Symbol Health Layer — universe integrity management for upstream acquisition.

Treats delisted / missing symbols as expected infrastructure events, not hard failures.
"""

from __future__ import annotations

import contextlib
import io
import json
import logging
import time
from dataclasses import dataclass, field
from datetime import datetime, timezone
from enum import Enum
from pathlib import Path
from typing import Callable

import pandas as pd

# Quiet yfinance noise in live loops
logging.getLogger("yfinance").setLevel(logging.CRITICAL)


class SymbolState(str, Enum):
    ACTIVE = "ACTIVE"
    DEGRADED = "DEGRADED"
    INVALID = "INVALID"
    QUARANTINED = "QUARANTINED"


INVALID_MARKERS = (
    "possibly delisted",
    "no price data found",
    "delisted",
    "not found",
    "no data found",
)

FAILURES_TO_QUARANTINE = 3
INVALID_RETRY_SEC = 6 * 3600  # 6 hours


@dataclass
class SymbolRecord:
    state: SymbolState = SymbolState.ACTIVE
    consecutive_failures: int = 0
    last_failure_reason: str = ""
    last_failure_at: float = 0.0
    last_success_at: float = 0.0
    quarantined_at: float = 0.0


class SymbolHealthRegistry:
    """Tracks symbol validity; emits deterministic universe events."""

    def __init__(self, state_path: Path, events_path: Path):
        self.state_path = state_path
        self.events_path = events_path
        self.records: dict[str, SymbolRecord] = {}
        self.events_path.parent.mkdir(parents=True, exist_ok=True)
        self._load()

    def _load(self) -> None:
        if not self.state_path.exists():
            return
        try:
            raw = json.loads(self.state_path.read_text())
            for sym, rec in raw.get("symbols", {}).items():
                self.records[sym] = SymbolRecord(
                    state=SymbolState(rec.get("state", "ACTIVE")),
                    consecutive_failures=int(rec.get("consecutive_failures", 0)),
                    last_failure_reason=rec.get("last_failure_reason", ""),
                    last_failure_at=float(rec.get("last_failure_at", 0)),
                    last_success_at=float(rec.get("last_success_at", 0)),
                    quarantined_at=float(rec.get("quarantined_at", 0)),
                )
        except Exception:
            pass

    def save(self) -> None:
        payload = {
            "version": 1,
            "updated_at_utc": datetime.now(timezone.utc).isoformat(),
            "symbols": {
                sym: {
                    "state": rec.state.value,
                    "consecutive_failures": rec.consecutive_failures,
                    "last_failure_reason": rec.last_failure_reason,
                    "last_failure_at": rec.last_failure_at,
                    "last_success_at": rec.last_success_at,
                    "quarantined_at": rec.quarantined_at,
                }
                for sym, rec in self.records.items()
            },
        }
        tmp = self.state_path.with_suffix(".tmp")
        with open(tmp, "w") as f:
            json.dump(payload, f, indent=2)
        tmp.replace(self.state_path)

    def emit(self, event_type: str, symbol: str, reason: str = "", extra: dict | None = None) -> None:
        event = {
            "type": event_type,
            "symbol": symbol,
            "reason": reason,
            "timestamp": int(time.time()),
            "completed_at_utc": datetime.now(timezone.utc).isoformat(),
        }
        if extra:
            event.update(extra)
        with open(self.events_path, "a") as f:
            f.write(json.dumps(event, sort_keys=True) + "\n")

    def _record(self, symbol: str) -> SymbolRecord:
        if symbol not in self.records:
            self.records[symbol] = SymbolRecord()
        return self.records[symbol]

    @staticmethod
    def classify_failure(stderr_text: str, exc: Exception | None = None) -> str:
        blob = (stderr_text or "").lower()
        if exc:
            blob += " " + str(exc).lower()
        for marker in INVALID_MARKERS:
            if marker in blob:
                return "NO_PRICE_DATA" if "delisted" in marker or "no price" in marker else marker.replace(" ", "_").upper()
        if exc:
            return type(exc).__name__
        return "EMPTY_RESPONSE"

    def should_skip_fetch(self, symbol: str) -> bool:
        rec = self._record(symbol)
        now = time.time()
        if rec.state == SymbolState.ACTIVE:
            return False
        if rec.state == SymbolState.DEGRADED:
            return False
        if rec.state in (SymbolState.INVALID, SymbolState.QUARANTINED):
            if now - rec.quarantined_at >= INVALID_RETRY_SEC:
                self.emit("SymbolRecovered", symbol, "REVALIDATION_WINDOW", {"prior_state": rec.state.value})
                rec.state = SymbolState.DEGRADED
                rec.consecutive_failures = 0
                return False
            return True
        return False

    def note_success(self, symbol: str) -> None:
        rec = self._record(symbol)
        if rec.state != SymbolState.ACTIVE:
            self.emit(
                "SymbolRecovered",
                symbol,
                "FETCH_OK",
                {"prior_state": rec.state.value},
            )
        rec.state = SymbolState.ACTIVE
        rec.consecutive_failures = 0
        rec.last_success_at = time.time()
        rec.last_failure_reason = ""
        self.save()

    def note_failure(self, symbol: str, reason: str, stderr_text: str = "") -> None:
        rec = self._record(symbol)
        now = time.time()
        rec.consecutive_failures += 1
        rec.last_failure_at = now
        rec.last_failure_reason = reason

        self.emit(
            "SymbolFetchFailed",
            symbol,
            reason,
            {"failures": rec.consecutive_failures, "stderr": stderr_text[:200]},
        )

        if reason == "NO_PRICE_DATA" or any(m in (stderr_text or "").lower() for m in ("possibly delisted", "no price data found")):
            if rec.state != SymbolState.INVALID:
                rec.state = SymbolState.INVALID
                rec.quarantined_at = now
                self.emit("SymbolQuarantined", symbol, reason, {"failures": rec.consecutive_failures})
        elif rec.consecutive_failures >= FAILURES_TO_QUARANTINE:
            if rec.state != SymbolState.QUARANTINED:
                rec.state = SymbolState.QUARANTINED
                rec.quarantined_at = now
                self.emit("SymbolQuarantined", symbol, reason, {"failures": rec.consecutive_failures})
        elif rec.consecutive_failures >= 1:
            if rec.state == SymbolState.ACTIVE:
                rec.state = SymbolState.DEGRADED
                self.emit("SymbolDegraded", symbol, reason, {"failures": rec.consecutive_failures})

        self.save()

    def active_universe(self, symbols: list[str]) -> list[str]:
        return [s for s in symbols if not self.should_skip_fetch(s)]

    def summary(self) -> dict[str, int]:
        counts: dict[str, int] = {s.value: 0 for s in SymbolState}
        for rec in self.records.values():
            counts[rec.state.value] += 1
        return counts


def health_paths(archive_dir: Path) -> tuple[Path, Path]:
    meta = archive_dir / "metadata"
    return meta / "symbol_health.json", meta / "universe_events.jsonl"


@contextlib.contextmanager
def _capture_stderr():
    buf = io.StringIO()
    with contextlib.redirect_stderr(buf):
        yield buf
    buf.seek(0)
    

def fetch_ticker_with_health(
    symbol: str,
    registry: SymbolHealthRegistry,
    interval: str = "5m",
    period: str = "1d",
) -> pd.DataFrame:
    """Fetch one symbol; update health registry; never raise for universe events."""
    if registry.should_skip_fetch(symbol):
        return pd.DataFrame()

    from candle_substrate import download_ticker_with_stderr

    try:
        df, stderr_text = download_ticker_with_stderr(symbol, interval, period)
    except Exception as e:
        reason = registry.classify_failure("", e)
        registry.note_failure(symbol, reason, "")
        return pd.DataFrame()

    if df is None or df.empty:
        reason = registry.classify_failure(stderr_text)
        registry.note_failure(symbol, reason, stderr_text)
        return pd.DataFrame()

    registry.note_success(symbol)
    return df


def fetch_universe_parallel(
    symbols: list[str],
    registry: SymbolHealthRegistry,
    interval: str = "5m",
    period: str = "1d",
    max_workers: int = 15,
    quorum_ratio: float = 0.15,
) -> tuple[dict[str, pd.DataFrame], dict]:
    """
    Parallel fetch with symbol health filtering.
    Proceeds if active symbols return data >= quorum_ratio of attempted fetches.
    """
    import concurrent.futures

    active = registry.active_universe(symbols)
    skipped = len(symbols) - len(active)
    data: dict[str, pd.DataFrame] = {}

    if not active:
        return data, {
            "attempted": 0,
            "success": 0,
            "skipped_quarantined": skipped,
            "quorum_met": False,
        }

    def _one(sym: str) -> tuple[str, pd.DataFrame]:
        return sym, fetch_ticker_with_health(sym, registry, interval, period)

    with concurrent.futures.ThreadPoolExecutor(max_workers=max_workers) as ex:
        for sym, df in ex.map(_one, active):
            if df is not None and not df.empty:
                data[sym] = df

    attempted = len(active)
    success = len(data)
    quorum_met = success >= max(1, int(attempted * quorum_ratio))

    stats = {
        "attempted": attempted,
        "success": success,
        "skipped_quarantined": skipped,
        "quorum_met": quorum_met,
        "health": registry.summary(),
    }

    if skipped > 0 or registry.summary().get("INVALID", 0) > 0:
        registry.emit(
            "UniverseChanged",
            "",
            "FETCH_CYCLE",
            {
                "active_success": success,
                "attempted": attempted,
                "skipped_quarantined": skipped,
            },
        )

    registry.save()
    return data, stats
