#!/usr/bin/env python3
"""
RP-608 NSE — Coralys decision-time input context (adapter-side).

Enriches Estimand A entry decisions with:
  - session phase (OPEN / EARLY / MID / LATE)
  - pre-open gap + imbalance (RP-608-NSE-PREOPEN-OBS-001 snapshots)
  - prior-session watch support (next-session recommendations)

Does NOT modify coralys-* crates. Does NOT change OF ±0.30.
Authority: 0.0%. Experimental entry gate for shadow observation.
"""

from __future__ import annotations

import json
import os
from dataclasses import dataclass
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any, Dict, Optional, Tuple

IST = timezone(timedelta(hours=5, minutes=30))

PROTOCOL_ID = "RP-608-NSE-CORALYS-INPUT-GATE-001"
POLICY_ID = "CORALYS_INPUT_GATE_V1"

# Env: 1/true enables stand-aside; 0 keeps stamp-only.
def gate_enabled() -> bool:
    raw = os.environ.get("RP608_NSE_CORALYS_INPUT_GATE", "1").strip().lower()
    return raw not in {"0", "false", "off", "no"}


def session_phase_ist(ts: datetime) -> str:
    """Map decision time to session phase."""
    if ts.tzinfo is None:
        ts = ts.replace(tzinfo=IST)
    else:
        ts = ts.astimezone(IST)
    hm = ts.hour * 60 + ts.minute
    if hm <= 9 * 60 + 15:
        return "OPEN"
    if hm < 9 * 60 + 45:
        return "EARLY"
    if hm < 11 * 60:
        return "MID"
    return "LATE"


def _parse_day(s: str) -> Optional[str]:
    s = (s or "").strip()
    return s[:10] if len(s) >= 10 else None


@dataclass
class PreopenRow:
    symbol: str
    opening_gap_bps: Optional[float]
    imbalance_ratio: Optional[float]
    indicative_source: Optional[str]
    previous_close: Optional[float]


class CoralysInputContextStore:
    """Session-cached prior + pre-open inputs."""

    def __init__(self, nse_dir: Path):
        self.nse_dir = Path(nse_dir)
        self._preopen_by_day: Dict[str, Dict[str, PreopenRow]] = {}
        self._watch_by_day: Dict[str, Dict[str, Dict[str, Any]]] = {}
        self._watch_meta: Dict[str, Any] = {}

    def _candidate_preopen_paths(self, session_date: str) -> list[Path]:
        day = session_date.replace("-", "")
        names = [
            f"preopen_discovery_snapshot_{day}T091430.jsonl",
            f"preopen_discovery_snapshot_{day}T091431.jsonl",
            f"preopen_discovery_snapshot_{day}T091130.jsonl",
            f"preopen_discovery_snapshot_{day}T090701.jsonl",
            f"preopen_discovery_snapshot_{day}T090700.jsonl",
        ]
        roots = [
            self.nse_dir / "test_run",
            self.nse_dir,
            self.nse_dir.parent / "test_run",
            self.nse_dir.parent,
        ]
        out: list[Path] = []
        for root in roots:
            for name in names:
                out.append(root / name)
        return out

    def load_preopen(self, session_date: str) -> Dict[str, PreopenRow]:
        if session_date in self._preopen_by_day:
            return self._preopen_by_day[session_date]
        rows: Dict[str, PreopenRow] = {}
        for path in self._candidate_preopen_paths(session_date):
            if not path.exists():
                continue
            with path.open("r", encoding="utf-8") as fh:
                for line in fh:
                    line = line.strip()
                    if not line:
                        continue
                    try:
                        r = json.loads(line)
                    except json.JSONDecodeError:
                        continue
                    sym = str(r.get("symbol") or "")
                    if not sym:
                        continue
                    gap = r.get("opening_gap_bps")
                    imb = r.get("imbalance_ratio")
                    rows[sym] = PreopenRow(
                        symbol=sym,
                        opening_gap_bps=float(gap) if gap is not None else None,
                        imbalance_ratio=float(imb) if imb is not None else None,
                        indicative_source=r.get("indicative_source"),
                        previous_close=float(r["previous_close"])
                        if r.get("previous_close") is not None
                        else None,
                    )
            if rows:
                break
        self._preopen_by_day[session_date] = rows
        return rows

    def _candidate_watch_paths(self) -> list[Path]:
        # Daemon output_dir is usually .../nse/test_run; file often lives in .../nse/.
        return [
            self.nse_dir / "rp608_nse_next_session_recommendations.json",
            self.nse_dir.parent / "rp608_nse_next_session_recommendations.json",
        ]

    def load_watchlist_for_session(self, session_date: str) -> Dict[str, Dict[str, Any]]:
        """Recommendations whose actionable/next_session_date matches session_date."""
        if session_date in self._watch_by_day:
            return self._watch_by_day[session_date]
        by_sym: Dict[str, Dict[str, Any]] = {}
        meta: Dict[str, Any] = {"path": None, "status": "MISSING"}
        for path in self._candidate_watch_paths():
            if not path.exists():
                continue
            try:
                doc = json.loads(path.read_text(encoding="utf-8"))
            except (OSError, json.JSONDecodeError):
                continue
            next_d = _parse_day(str(doc.get("next_session_date") or ""))
            meta = {
                "path": str(path),
                "status": "OK" if next_d == session_date else "STALE_OR_MISMATCH",
                "next_session_date": next_d,
                "from_session_date": doc.get("from_session_date"),
                "n_recommendations": doc.get("n_recommendations"),
            }
            if next_d == session_date:
                for rec in doc.get("recommendations") or []:
                    sym = str(rec.get("symbol") or "")
                    if sym:
                        by_sym[sym] = rec
            break
        self._watch_meta = meta
        self._watch_by_day[session_date] = by_sym
        return by_sym

    def evaluate(
        self,
        *,
        symbol: str,
        session_date: str,
        decision_ts_ist: datetime,
        traj15: float,
        signal_long: bool,
    ) -> Dict[str, Any]:
        phase = session_phase_ist(decision_ts_ist)
        preopen_map = self.load_preopen(session_date)
        watch = self.load_watchlist_for_session(session_date)
        po = preopen_map.get(symbol)
        watch_meta = dict(self._watch_meta)
        on_watch = symbol in watch
        prior_status = watch_meta.get("status", "MISSING")
        if prior_status == "OK":
            prior_support = bool(on_watch)
        else:
            prior_support = None  # unavailable — do not hard-require

        gap = po.opening_gap_bps if po else None
        imb = po.imbalance_ratio if po else None
        preopen_status = "OK" if po is not None else "MISSING"

        allow = True
        reason = "PASS_NO_SIGNAL"
        if not signal_long:
            return {
                "protocol_id": PROTOCOL_ID,
                "policy_id": POLICY_ID,
                "gate_enabled": gate_enabled(),
                "session_phase": phase,
                "preopen_status": preopen_status,
                "opening_gap_bps": gap,
                "imbalance_ratio": imb,
                "indicative_source": po.indicative_source if po else None,
                "prior_status": prior_status,
                "prior_support": prior_support,
                "on_next_session_watch": on_watch,
                "traj15": round(float(traj15), 6),
                "allow_long_entry": False,
                "standaside_reason": reason,
                "baseline_of_inert": True,
                "coralys_crates_untouched": True,
            }

        # Default allow; apply experimental gate when enabled.
        allow = True
        reason = "PASS_BASELINE_SIGNAL"

        if gate_enabled():
            if preopen_status != "OK" or gap is None:
                allow = True
                reason = "PASS_PREOPEN_MISSING_FAIL_OPEN"
            elif gap <= -75.0 and float(traj15) < 0.12:
                allow = False
                reason = "STANDASIDE_PREOPEN_LARGE_GAP_DOWN"
            elif phase == "OPEN":
                # Open requires constructive pre-open; prior watch when available.
                aligned = gap >= 20.0 and (imb is None or imb >= -0.25)
                if not aligned:
                    allow = False
                    reason = "STANDASIDE_OPEN_PREOPEN_NOT_ALIGNED"
                elif prior_support is False:
                    allow = False
                    reason = "STANDASIDE_OPEN_NO_PRIOR_WATCH"
                else:
                    allow = True
                    reason = "PASS_OPEN_PRIOR_PREOPEN_ALIGN"
            elif phase == "EARLY":
                if gap < -40.0 and float(traj15) < 0.08:
                    allow = False
                    reason = "STANDASIDE_EARLY_PREOPEN_OPPOSED_WEAK_TRAJ"
                else:
                    allow = True
                    reason = "PASS_EARLY"
            else:
                # MID / LATE (includes post-10:45): opposed pre-open needs stronger traj15
                if gap < -20.0 and float(traj15) < 0.08:
                    allow = False
                    reason = "STANDASIDE_CONTINUOUS_PREOPEN_OPPOSED_WEAK_TRAJ"
                elif gap < -20.0 and prior_support is False and float(traj15) < 0.10:
                    allow = False
                    reason = "STANDASIDE_CONTINUOUS_NO_PRIOR_WEAK_TRAJ"
                else:
                    allow = True
                    reason = "PASS_CONTINUOUS"

        return {
            "protocol_id": PROTOCOL_ID,
            "policy_id": POLICY_ID,
            "gate_enabled": gate_enabled(),
            "session_phase": phase,
            "preopen_status": preopen_status,
            "opening_gap_bps": gap,
            "imbalance_ratio": imb,
            "indicative_source": po.indicative_source if po else None,
            "prior_status": prior_status,
            "prior_support": prior_support,
            "on_next_session_watch": on_watch,
            "traj15": round(float(traj15), 6),
            "allow_long_entry": bool(allow),
            "standaside_reason": reason,
            "baseline_of_inert": True,
            "coralys_crates_untouched": True,
            "watch_meta": {
                "status": prior_status,
                "next_session_date": watch_meta.get("next_session_date"),
            },
        }


def decision_ts_from_bar_ms(timestamp_utc_ms: int) -> datetime:
    return datetime.fromtimestamp(int(timestamp_utc_ms) / 1000.0, tz=timezone.utc).astimezone(IST)
