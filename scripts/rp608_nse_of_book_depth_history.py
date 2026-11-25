#!/usr/bin/env python3
"""
RP-608-NSE-OF-BOOK-INTELLIGENCE-001 — Phase 1 durable L1–L5 depth snapshots.

Observational only. Does not change OF ±0.30 marks, reverse-OF paper exits,
Estimand A, or live order authority (0%).

Objective served: retain timestamped five-level history so bid persistence,
replenishment, and price response can later be measured while price is still weak.
"""

from __future__ import annotations

import json
import os
import threading
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Mapping, Optional, Sequence

PROTOCOL_ID = "RP-608-NSE-OF-BOOK-INTELLIGENCE-001"
DEFAULT_MIN_INTERVAL_MS = 5_000


def _depth_writer_mode() -> str:
    try:
        from scripts.rp608_nse_depth_journal_client import depth_writer_mode

        return depth_writer_mode()
    except Exception:
        raw = (os.environ.get("RP608_NSE_DEPTH_WRITER") or "python").strip().lower()
        if raw in {"compare", "side_by_side", "sbs", "rust", "rs"}:
            return "compare" if raw not in {"rust", "rs"} else "rust"
        return "python"


def _level_tuple(levels: Any) -> List[Dict[str, float]]:
    out: List[Dict[str, float]] = []
    if not isinstance(levels, list):
        return [{"price": 0.0, "quantity": 0.0} for _ in range(5)]
    for i in range(5):
        row = levels[i] if i < len(levels) and isinstance(levels[i], dict) else {}
        try:
            px = float(row.get("price") or 0.0)
        except (TypeError, ValueError):
            px = 0.0
        try:
            qty = float(row.get("quantity") or 0.0)
        except (TypeError, ValueError):
            qty = 0.0
        out.append({"price": round(px, 4), "quantity": round(qty, 4)})
    return out


def snapshot_row_from_tick(
    *,
    symbol: str,
    tick: Mapping[str, Any],
    sample_utc_ms: Optional[int] = None,
) -> Dict[str, Any]:
    """Build one durable depth row from a retained full-mode cache tick."""
    now_ms = int(sample_utc_ms if sample_utc_ms is not None else time.time() * 1000)
    depth_buy = _level_tuple(tick.get("depth_buy"))
    depth_sell = _level_tuple(tick.get("depth_sell"))
    bid_nz = sum(1 for lv in depth_buy if float(lv["quantity"]) > 0)
    ask_nz = sum(1 for lv in depth_sell if float(lv["quantity"]) > 0)
    try:
        exch_sec = int(tick.get("exchange_timestamp_sec") or 0)
    except (TypeError, ValueError):
        exch_sec = 0
    exch_ms = exch_sec * 1000 if exch_sec > 0 else None
    try:
        receipt_ms = int(tick.get("receipt_utc_ms") or 0) or None
    except (TypeError, ValueError):
        receipt_ms = None
    try:
        age_ms = float(tick.get("quote_age_ms")) if tick.get("quote_age_ms") is not None else None
    except (TypeError, ValueError):
        age_ms = None
    stale = bool(age_ms is not None and age_ms > 60_000.0)
    missing_levels = bid_nz < 5 or ask_nz < 5
    return {
        "protocol_id": PROTOCOL_ID,
        "record_type": "depth_snapshot_l1_l5",
        "symbol": str(symbol),
        "instrument_token": int(tick.get("instrument_token") or 0) or None,
        "sample_utc_ms": now_ms,
        "sample_iso": datetime.fromtimestamp(now_ms / 1000.0, tz=timezone.utc).isoformat(),
        "exchange_timestamp_sec": exch_sec or None,
        "exchange_timestamp_utc_ms": exch_ms,
        "receipt_utc_ms": receipt_ms,
        "quote_age_ms": age_ms,
        "stale": stale,
        "missing_level_qty": missing_levels,
        "bid_levels_qty_nonzero": bid_nz,
        "ask_levels_qty_nonzero": ask_nz,
        "last_price": float(tick.get("last_price") or 0.0) or None,
        "best_bid": float(tick.get("best_bid") or 0.0) or None,
        "best_ask": float(tick.get("best_ask") or 0.0) or None,
        "bid_qty": float(tick.get("bid_qty") or 0.0),
        "ask_qty": float(tick.get("ask_qty") or 0.0),
        "volume": float(tick.get("volume") or 0.0) or None,
        "depth_buy": depth_buy,
        "depth_sell": depth_sell,
        "quote_source": tick.get("quote_source") or "kite_ws_full",
    }


class NseOfBookDepthHistoryJournal:
    """Append-only L1–L5 snapshots; rate-limited; OF-rule inert."""

    def __init__(
        self,
        root: Path,
        *,
        min_interval_ms: int = DEFAULT_MIN_INTERVAL_MS,
        enabled: bool = True,
    ):
        self.root = Path(root)
        self.root.mkdir(parents=True, exist_ok=True)
        self.path = self.root / "depth_snapshots.jsonl"
        # Track B side-path only — never dual-write authoritative file.
        self.rust_side_path = self.root / "depth_snapshots.rust.jsonl"
        self.coverage_path = self.root / "depth_capture_coverage.json"
        self.min_interval_ms = int(min_interval_ms)
        self.enabled = bool(enabled)
        self._lock = threading.Lock()
        self._last_sample_mono_ms: float = 0.0
        self._stats: Dict[str, Any] = {
            "protocol_id": PROTOCOL_ID,
            "phase": "PHASE_1_DURABLE_L1_L5_HISTORY",
            "snapshots_written": 0,
            "rust_side_snapshots_written": 0,
            "samples_taken": 0,
            "samples_skipped_interval": 0,
            "rows_missing_level_qty": 0,
            "rows_stale": 0,
            "rows_with_l2_plus_bid_qty": 0,
            "rows_with_l2_plus_ask_qty": 0,
            "last_sample_utc": None,
            "last_symbols_sampled": 0,
            "depth_writer_mode": _depth_writer_mode(),
        }

    def maybe_sample(
        self,
        *,
        ticks_by_symbol: Mapping[str, Mapping[str, Any]],
        now_ms: Optional[int] = None,
    ) -> int:
        """
        Write one snapshot per symbol if min interval elapsed.
        Returns number of rows appended (0 if skipped / disabled).
        """
        if not self.enabled:
            return 0
        now = int(now_ms if now_ms is not None else time.time() * 1000)
        mono_ms = time.monotonic() * 1000.0
        with self._lock:
            if self._last_sample_mono_ms > 0 and (mono_ms - self._last_sample_mono_ms) < self.min_interval_ms:
                self._stats["samples_skipped_interval"] = int(self._stats["samples_skipped_interval"]) + 1
                self._write_coverage_unlocked()
                return 0
            self._last_sample_mono_ms = mono_ms
            self._stats["samples_taken"] = int(self._stats["samples_taken"]) + 1

            rows: List[Dict[str, Any]] = []
            for sym, tick in ticks_by_symbol.items():
                if not isinstance(tick, Mapping):
                    continue
                row = snapshot_row_from_tick(symbol=str(sym), tick=tick, sample_utc_ms=now)
                rows.append(row)
                if row["missing_level_qty"]:
                    self._stats["rows_missing_level_qty"] = int(self._stats["rows_missing_level_qty"]) + 1
                if row["stale"]:
                    self._stats["rows_stale"] = int(self._stats["rows_stale"]) + 1
                if int(row["bid_levels_qty_nonzero"]) >= 2:
                    self._stats["rows_with_l2_plus_bid_qty"] = int(
                        self._stats["rows_with_l2_plus_bid_qty"]
                    ) + 1
                if int(row["ask_levels_qty_nonzero"]) >= 2:
                    self._stats["rows_with_l2_plus_ask_qty"] = int(
                        self._stats["rows_with_l2_plus_ask_qty"]
                    ) + 1

            if not rows:
                self._write_coverage_unlocked()
                return 0

            mode = _depth_writer_mode()
            self._stats["depth_writer_mode"] = mode
            # Authoritative path: Python unless explicit rust flip (post-soak only).
            write_python_auth = mode != "rust"
            if write_python_auth:
                with self.path.open("a", encoding="utf-8") as f:
                    for row in rows:
                        f.write(json.dumps(row, separators=(",", ":")) + "\n")
                    f.flush()
                self._stats["snapshots_written"] = int(self._stats["snapshots_written"]) + len(
                    rows
                )

            if mode in {"compare", "rust"}:
                rust_n = self._append_rust_side_unlocked(
                    ticks_by_symbol=ticks_by_symbol, sample_utc_ms=now
                )
                if mode == "rust" and rust_n <= 0:
                    # Fail-closed: do not leave a hole — fall back to Python auth write.
                    print(
                        "[nse_depth_journal] rust flip failed; writing Python authoritative rows",
                        flush=True,
                    )
                    with self.path.open("a", encoding="utf-8") as f:
                        for row in rows:
                            f.write(json.dumps(row, separators=(",", ":")) + "\n")
                        f.flush()
                    self._stats["snapshots_written"] = int(
                        self._stats["snapshots_written"]
                    ) + len(rows)

            self._stats["last_sample_utc"] = datetime.fromtimestamp(
                now / 1000.0, tz=timezone.utc
            ).isoformat()
            self._stats["last_symbols_sampled"] = len(rows)
            self._write_coverage_unlocked()
            return len(rows)

    def _append_rust_side_unlocked(
        self,
        *,
        ticks_by_symbol: Mapping[str, Mapping[str, Any]],
        sample_utc_ms: int,
    ) -> int:
        """Append Rust-built rows to side path (compare) or auth path (rust flip)."""
        try:
            from scripts.rp608_nse_depth_journal_client import rust_depth_row
        except Exception as exc:
            print(f"[nse_depth_journal] rust client import failed: {exc}", flush=True)
            return 0
        mode = _depth_writer_mode()
        out_path = self.path if mode == "rust" else self.rust_side_path
        n = 0
        try:
            with out_path.open("a", encoding="utf-8") as f:
                for sym, tick in ticks_by_symbol.items():
                    if not isinstance(tick, Mapping):
                        continue
                    row = rust_depth_row(
                        symbol=str(sym),
                        tick=dict(tick),
                        sample_utc_ms=int(sample_utc_ms),
                    )
                    if not row:
                        continue
                    f.write(json.dumps(row, separators=(",", ":")) + "\n")
                    n += 1
                f.flush()
        except Exception as exc:
            print(f"[nse_depth_journal] rust side append failed: {exc}", flush=True)
            return 0
        if n:
            self._stats["rust_side_snapshots_written"] = int(
                self._stats.get("rust_side_snapshots_written") or 0
            ) + n
        return n

    def _write_coverage_unlocked(self) -> None:
        payload = dict(self._stats)
        payload["path"] = str(self.path)
        payload["min_interval_ms"] = self.min_interval_ms
        payload["enabled"] = self.enabled
        payload["updated_utc"] = datetime.now(timezone.utc).isoformat()
        tmp = self.coverage_path.with_suffix(".tmp")
        tmp.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
        tmp.replace(self.coverage_path)


def collect_ticks_by_symbol(
    cache: Any,
    token_to_symbol: Mapping[int, str],
) -> Dict[str, Dict[str, Any]]:
    """Pull latest retained ticks (incl. depth arrays) for journaling."""
    out: Dict[str, Dict[str, Any]] = {}
    if cache is None or not hasattr(cache, "get_l1"):
        return out
    for tok, sym in token_to_symbol.items():
        try:
            tick = cache.get_l1(int(tok))
        except Exception:
            tick = None
        if not tick:
            continue
        out[str(sym)] = tick
    return out
