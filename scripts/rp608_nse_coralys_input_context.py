#!/usr/bin/env python3
"""
RP-608 NSE — Coralys decision-time input context (adapter-side).

Enriches Estimand A entry decisions with:
  - session phase / time-of-action
    (OPEN / EARLY_JITTER / EARLY / MID / LATE)
  - pre-open gap + imbalance (RP-608-NSE-PREOPEN-OBS-001 snapshots)
  - prior-session watch support (next-session recommendations)
  - pre-open / open jitter + trend-alignment stand-aside

Does NOT touch coralys-* crates. Does NOT change OF ±0.30.
Does NOT change frozen stops/targets/holds (Gate 5).
Authority: 0.0%. Experimental entry gate for shadow observation.

Prior freshness (V3):
  OK              — next_session_date == session_date (actionable)
  STALE_RECENT    — next_session_date lags ≤6 calendar days (weekend +
                    unseen holiday buffer); symbols still loaded, weak prior
  STALE_EXPIRED   — lag > STALE_RECENT_MAX_CALENDAR_DAYS
  MISMATCH_FUTURE — next_session_date after session_date
  MISSING         — no file / unreadable

Time-of-action (V6 · IST):
  OPEN          ≤09:15  — auction / open print (jitter-sensitive)
  EARLY_JITTER  <09:20  — settle hold after open
  EARLY         <09:45  — constructive window (trend + gap aligned)
  MID / LATE    as before
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
# V6 (2026-11-20): time-of-action + trend vs pre-open jitter.
# Keeps V5: preopen MISSING → always stand aside.
# Per-symbol session re-entry bans removed — churn is a pattern feature
# (see rp608_nse_trade_pattern_* / Gate 2E).
POLICY_ID = "CORALYS_INPUT_GATE_V6"

# Continuous-session traj floors (adapter gate on top of frozen Estimand A ≥0.05).
CONTINUOUS_TRAJ_FLOOR = 0.10
CONTINUOUS_TRAJ_FLOOR_PRIOR_WEAK = 0.12
CONTINUOUS_TRAJ_FLOOR_NONPOSITIVE_GAP = 0.12
CONTINUOUS_TRAJ_FLOOR_OPPOSED_GAP = 0.15
LATE_TRAJ_FLOOR = 0.15

# Prior watch freshness ladder.
STALE_RECENT_MAX_CALENDAR_DAYS = 6  # weekend + unseen holiday buffer; still STALE_RECENT

# Pre-open / open turbulence (hard jitter) thresholds.
TURBULENCE_ABS_GAP_BPS = 75.0
TURBULENCE_ABS_IMB = 0.45
TURBULENCE_COMBO_GAP_BPS = 40.0
TURBULENCE_OPPOSED_GAP_BPS = 50.0
TURBULENCE_OPPOSED_IMB = 0.35

# Medium pre-open jitter (OPEN / EARLY_JITTER standaside even without full turbulence).
JITTER_MED_GAP_BPS = 35.0
JITTER_MED_IMB = 0.30

# Time-of-action floors / alignment (LONG book).
OPEN_TRAJ_FLOOR = 0.15
EARLY_JITTER_TRAJ_FLOOR = 0.18
EARLY_TRAJ_FLOOR = 0.12
EARLY_TURBULENT_TRAJ_FLOOR = 0.18
ALIGN_GAP_BPS = 20.0
OPPOSED_GAP_BPS = -25.0
OPPOSED_EARLY_TRAJ_FLOOR = 0.18


def gate_enabled() -> bool:
    raw = os.environ.get("RP608_NSE_CORALYS_INPUT_GATE", "1").strip().lower()
    return raw not in {"0", "false", "off", "no"}


def session_phase_ist(ts: datetime) -> str:
    """Map decision time to time-of-action phase (IST)."""
    if ts.tzinfo is None:
        ts = ts.replace(tzinfo=IST)
    else:
        ts = ts.astimezone(IST)
    hm = ts.hour * 60 + ts.minute
    if hm <= 9 * 60 + 15:
        return "OPEN"
    if hm < 9 * 60 + 20:
        return "EARLY_JITTER"
    if hm < 9 * 60 + 45:
        return "EARLY"
    if hm < 11 * 60:
        return "MID"
    return "LATE"


def is_preopen_jittery(
    gap_bps: Optional[float],
    imbalance_ratio: Optional[float],
) -> Tuple[bool, str]:
    """
    Medium pre-open jitter for LONG book (below full turbulence).

    Clean constructive gap-up + supportive bid is NOT jitter.
    Flags gap-down noise, gap/imbalance opposition, and gap-up with ask-heavy book.
    """
    if gap_bps is None:
        return False, ""
    gap = float(gap_bps)
    imb = float(imbalance_ratio) if imbalance_ratio is not None else None
    if gap <= -JITTER_MED_GAP_BPS:
        return True, "MED_GAP_DOWN"
    if (
        gap >= JITTER_MED_GAP_BPS
        and imb is not None
        and imb <= -JITTER_MED_IMB
    ):
        return True, "MED_GAP_UP_ASK_HEAVY"
    if (
        imb is not None
        and abs(imb) >= JITTER_MED_IMB
        and abs(gap) >= 20.0
        and (gap * imb) < 0.0
    ):
        return True, "MED_GAP_IMB_OPPOSED"
    return False, ""


def long_trend_aligned(gap_bps: float, traj15: float) -> bool:
    """LONG book: traj and opening gap should not fight."""
    if gap_bps >= ALIGN_GAP_BPS and traj15 >= OPEN_TRAJ_FLOOR:
        return True
    if gap_bps >= 0.0 and traj15 >= EARLY_JITTER_TRAJ_FLOOR:
        return True
    return False


def _parse_day(s: str) -> Optional[str]:
    s = (s or "").strip()
    return s[:10] if len(s) >= 10 else None


def classify_prior_watch_status(
    session_date: str,
    next_session_date: Optional[str],
) -> str:
    """
    Classify prior-watch freshness for the live session.

    OK: actionable for today.
    STALE_RECENT: recent lag (≤6 calendar days) — usable with weak-prior floors.
    STALE_EXPIRED: too old to use as support.
    MISMATCH_FUTURE: file points past today.
    MISSING: no next_session_date.
    """
    sess = _parse_day(session_date)
    nxt = _parse_day(next_session_date or "")
    if not nxt:
        return "MISSING"
    if not sess:
        return "MISSING"
    if nxt == sess:
        return "OK"
    try:
        sess_d = datetime.strptime(sess, "%Y-%m-%d").date()
        nxt_d = datetime.strptime(nxt, "%Y-%m-%d").date()
    except ValueError:
        return "STALE_EXPIRED"
    if nxt_d > sess_d:
        return "MISMATCH_FUTURE"
    lag = (sess_d - nxt_d).days
    if lag <= STALE_RECENT_MAX_CALENDAR_DAYS:
        return "STALE_RECENT"
    return "STALE_EXPIRED"


def is_preopen_turbulent(
    gap_bps: Optional[float],
    imbalance_ratio: Optional[float],
) -> Tuple[bool, str]:
    """
    Detect pre-open / open turbulence from gap + L1 imbalance.
    Returns (turbulent, reason_tag).
    """
    if gap_bps is None:
        return False, ""
    gap = float(gap_bps)
    imb = float(imbalance_ratio) if imbalance_ratio is not None else None

    if abs(gap) >= TURBULENCE_ABS_GAP_BPS:
        return True, "LARGE_ABS_GAP"
    if (
        imb is not None
        and abs(imb) >= TURBULENCE_ABS_IMB
        and abs(gap) >= TURBULENCE_COMBO_GAP_BPS
    ):
        return True, "GAP_IMBALANCE_SPIKE"
    if gap <= -TURBULENCE_OPPOSED_GAP_BPS and imb is not None and imb <= -TURBULENCE_OPPOSED_IMB:
        return True, "GAP_DOWN_ASK_HEAVY"
    if gap >= TURBULENCE_OPPOSED_GAP_BPS and imb is not None and imb >= TURBULENCE_OPPOSED_IMB:
        return True, "GAP_UP_BID_HEAVY"
    return False, ""


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

    def invalidate_watch_cache(self) -> None:
        """Clear cached watch after an on-disk refresh."""
        self._watch_by_day.clear()
        self._watch_meta = {}

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
            try:
                text = path.read_text(encoding="utf-8")
            except OSError:
                continue
            for line in text.splitlines():
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
        """Load recommendations under OK / STALE_RECENT freshness."""
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
            status = classify_prior_watch_status(session_date, next_d)
            meta = {
                "path": str(path),
                "status": status,
                "next_session_date": next_d,
                "from_session_date": doc.get("from_session_date"),
                "n_recommendations": doc.get("n_recommendations"),
            }
            # Actionable membership only for OK + STALE_RECENT.
            if status in {"OK", "STALE_RECENT"}:
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
        prior_status = str(watch_meta.get("status") or "MISSING")
        # Support signal only when watch membership is loaded (OK / STALE_RECENT).
        if prior_status in {"OK", "STALE_RECENT"}:
            prior_support = bool(on_watch)
        else:
            prior_support = None  # unavailable

        gap = po.opening_gap_bps if po else None
        imb = po.imbalance_ratio if po else None
        preopen_status = "OK" if po is not None else "MISSING"
        turbulent, turb_tag = is_preopen_turbulent(gap, imb)
        jittery, jitter_tag = is_preopen_jittery(gap, imb)

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
                "preopen_turbulent": bool(turbulent),
                "preopen_turbulence_tag": turb_tag or None,
                "preopen_jittery": bool(jittery),
                "preopen_jitter_tag": jitter_tag or None,
                "prior_status": prior_status,
                "prior_support": prior_support,
                "on_next_session_watch": on_watch,
                "traj15": round(float(traj15), 6),
                "allow_long_entry": False,
                "standaside_reason": reason,
                "baseline_of_inert": True,
                "coralys_crates_untouched": True,
            }

        allow = True
        reason = "PASS_BASELINE_SIGNAL"
        t = float(traj15)

        if gate_enabled():
            if preopen_status != "OK" or gap is None:
                # V5 retained: never enter without preopen context.
                allow = False
                reason = "STANDASIDE_PREOPEN_MISSING"
            elif gap <= -75.0:
                # Large gap-down: hard standaside in all phases (jitter / trend break).
                allow = False
                reason = "STANDASIDE_PREOPEN_LARGE_GAP_DOWN"
            elif phase in {"OPEN", "EARLY_JITTER"} and turbulent:
                # V6: hard standaside on full turbulence during open/jitter hold.
                allow = False
                reason = f"STANDASIDE_PREOPEN_TURBULENCE_{turb_tag}"
            elif phase in {"OPEN", "EARLY_JITTER"} and jittery:
                # Medium gap/imbalance jitter — do not chase auction prints.
                allow = False
                reason = f"STANDASIDE_PREOPEN_JITTER_{jitter_tag}"
            elif phase == "OPEN":
                # Auction/open: require constructive gap + trend align + prior.
                aligned = gap >= ALIGN_GAP_BPS and (imb is None or imb >= -0.25)
                if not aligned:
                    allow = False
                    reason = "STANDASIDE_OPEN_PREOPEN_NOT_ALIGNED"
                elif not long_trend_aligned(gap, t):
                    allow = False
                    reason = "STANDASIDE_OPEN_TREND_NOT_ALIGNED"
                elif t < OPEN_TRAJ_FLOOR:
                    allow = False
                    reason = "STANDASIDE_OPEN_WEAK_TRAJ"
                elif prior_status == "STALE_EXPIRED" or prior_status == "MISMATCH_FUTURE":
                    if t < CONTINUOUS_TRAJ_FLOOR_PRIOR_WEAK:
                        allow = False
                        reason = "STANDASIDE_OPEN_PRIOR_EXPIRED_WEAK_TRAJ"
                    else:
                        allow = True
                        reason = "PASS_OPEN_PRIOR_EXPIRED_STRONG_TRAJ"
                elif prior_status == "STALE_RECENT":
                    if prior_support is False and t < CONTINUOUS_TRAJ_FLOOR_PRIOR_WEAK:
                        allow = False
                        reason = "STANDASIDE_OPEN_STALE_PRIOR_NO_WATCH"
                    elif t < OPEN_TRAJ_FLOOR:
                        allow = False
                        reason = "STANDASIDE_OPEN_STALE_PRIOR_WEAK_TRAJ"
                    else:
                        allow = True
                        reason = "PASS_OPEN_STALE_RECENT_PRIOR"
                elif prior_support is False:
                    allow = False
                    reason = "STANDASIDE_OPEN_NO_PRIOR_WATCH"
                else:
                    allow = True
                    reason = "PASS_OPEN_PRIOR_PREOPEN_ALIGN"
            elif phase == "EARLY_JITTER":
                # 09:15–09:20 settle hold: only strong, aligned, prior-supported longs.
                if gap < OPPOSED_GAP_BPS:
                    allow = False
                    reason = "STANDASIDE_EARLY_JITTER_GAP_OPPOSED"
                elif not long_trend_aligned(gap, t) or t < EARLY_JITTER_TRAJ_FLOOR:
                    allow = False
                    reason = "STANDASIDE_EARLY_JITTER_WEAK_OR_MISALIGNED"
                elif prior_support is False:
                    allow = False
                    reason = "STANDASIDE_EARLY_JITTER_NO_PRIOR_WATCH"
                else:
                    allow = True
                    reason = "PASS_EARLY_JITTER_ALIGNED"
            elif phase == "EARLY":
                # 09:20–09:45 constructive window — trend + gap; hard on turbulence.
                if turbulent and t < EARLY_TURBULENT_TRAJ_FLOOR:
                    allow = False
                    reason = f"STANDASIDE_EARLY_TURBULENCE_{turb_tag}"
                elif gap < OPPOSED_GAP_BPS and t < OPPOSED_EARLY_TRAJ_FLOOR:
                    allow = False
                    reason = "STANDASIDE_EARLY_PREOPEN_OPPOSED_WEAK_TRAJ"
                elif t < EARLY_TRAJ_FLOOR:
                    allow = False
                    reason = "STANDASIDE_EARLY_WEAK_TRAJ"
                elif prior_status == "STALE_RECENT" and prior_support is False and t < CONTINUOUS_TRAJ_FLOOR_PRIOR_WEAK:
                    allow = False
                    reason = "STANDASIDE_EARLY_STALE_PRIOR_NO_WATCH"
                elif gap < 0.0 and not long_trend_aligned(gap, t):
                    allow = False
                    reason = "STANDASIDE_EARLY_TREND_NOT_ALIGNED"
                else:
                    allow = True
                    reason = "PASS_EARLY"
            else:
                # MID / LATE: raise traj floor; stale prior / non-positive gap need more strength.
                # Does NOT change frozen stops/targets/holds (Gate 5). Entry selection only.
                prior_weak = prior_status != "OK"
                late_floor = LATE_TRAJ_FLOOR if phase == "LATE" else CONTINUOUS_TRAJ_FLOOR
                if t < late_floor:
                    allow = False
                    reason = (
                        "STANDASIDE_LATE_WEAK_TRAJ"
                        if phase == "LATE"
                        else "STANDASIDE_CONTINUOUS_WEAK_TRAJ"
                    )
                elif prior_weak and t < CONTINUOUS_TRAJ_FLOOR_PRIOR_WEAK:
                    allow = False
                    reason = "STANDASIDE_CONTINUOUS_PRIOR_UNAVAILABLE_WEAK_TRAJ"
                elif prior_support is False and t < CONTINUOUS_TRAJ_FLOOR_PRIOR_WEAK:
                    allow = False
                    reason = "STANDASIDE_CONTINUOUS_NO_PRIOR_WEAK_TRAJ"
                elif gap < 0.0 and t < CONTINUOUS_TRAJ_FLOOR_NONPOSITIVE_GAP:
                    allow = False
                    reason = "STANDASIDE_CONTINUOUS_NONPOSITIVE_GAP_WEAK_TRAJ"
                elif gap < -20.0 and t < CONTINUOUS_TRAJ_FLOOR_OPPOSED_GAP:
                    allow = False
                    reason = "STANDASIDE_CONTINUOUS_PREOPEN_OPPOSED_WEAK_TRAJ"
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
            "preopen_turbulent": bool(turbulent),
            "preopen_turbulence_tag": turb_tag or None,
            "preopen_jittery": bool(jittery),
            "preopen_jitter_tag": jitter_tag or None,
            "trend_aligned_long": bool(
                gap is not None and long_trend_aligned(float(gap), t)
            )
            if signal_long and gap is not None
            else None,
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
                "stale_recent_max_calendar_days": STALE_RECENT_MAX_CALENDAR_DAYS,
            },
        }


def decision_ts_from_bar_ms(timestamp_utc_ms: int) -> datetime:
    return datetime.fromtimestamp(int(timestamp_utc_ms) / 1000.0, tz=timezone.utc).astimezone(IST)
