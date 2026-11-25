#!/usr/bin/env python3
"""
scripts/rp608_crypto_tick_policy.py

RP-608-SHADOW-TICK-001 — tick-native Policy B/C evidence + evaluation helpers.

Decision clock: Binance BTC/USDT bookTicker (L1).
ATR / trade-plan geometry remains sourced from completed 15m bars (caller-supplied).
Strictly 0.0% order authority. Deterministic: same quote sequence → same factors/actions.
"""

from __future__ import annotations

from collections import deque
from dataclasses import dataclass, field
from typing import Any, Deque, Dict, List, Optional, Sequence, Tuple

# Frozen microstructure thresholds (aligned with Gate C EVENT-POLICY-V1 constants).
MAX_VALID_SPREAD_BPS = 5.0
MAX_QUOTE_AGE_MS = 500.0
MIN_HISTORY_WARMUP_SEC = 5.0
MIN_REFRACTORY_MS = 1000.0

IMBALANCE_THRESHOLD = 0.30
VELOCITY_5S_THRESHOLD_BPS = 1.0
VELOCITY_30S_CONFIRM_BPS = 0.0
TREND_FILTER_BPS = 15.0
TICK_VOL_WINDOW_SEC = 30.0

PROTOCOL_ID = "RP-608-SHADOW-TICK-001"
DECISION_CLOCK = "tick_book_ticker"


@dataclass
class TickQuote:
    update_id: int
    event_time_ms: int
    receive_time_ms: int
    bid_price: float
    bid_qty: float
    ask_price: float
    ask_qty: float
    symbol: str = "BTCUSDT"

    @property
    def mid_price(self) -> float:
        return (self.bid_price + self.ask_price) / 2.0

    @property
    def spread_bps(self) -> float:
        mid = self.mid_price
        if mid <= 0:
            return 0.0
        return ((self.ask_price - self.bid_price) / mid) * 10000.0

    @property
    def imbalance(self) -> float:
        denom = self.bid_qty + self.ask_qty
        if denom <= 0:
            return 0.0
        return (self.bid_qty - self.ask_qty) / denom

    @property
    def micro_price(self) -> float:
        denom = self.bid_qty + self.ask_qty
        if denom <= 0:
            return self.mid_price
        return (self.bid_price * self.ask_qty + self.ask_price * self.bid_qty) / denom


@dataclass
class TickFeatureSnapshot:
    mid_price: float
    spread_bps: float
    imbalance: float
    micro_vs_mid_bps: float
    velocity_5s_bps: Optional[float]
    velocity_30s_bps: Optional[float]
    tick_vol_30s_bps: Optional[float]
    macro_trend_16_bps: Optional[float]
    macro_trend_64_bps: Optional[float]
    history_sec: float
    quote_age_ms: float
    update_id: int
    event_time_ms: int
    receive_time_ms: int


@dataclass
class TickFeatureBuffer:
    """Rolling mid-price buffer for velocity / short-horizon realized range."""

    window_sec: float = 65.0
    _buf: Deque[TickQuote] = field(default_factory=deque)

    def append(self, quote: TickQuote) -> None:
        self._buf.append(quote)
        cutoff = quote.event_time_ms - int(self.window_sec * 1000)
        while self._buf and self._buf[0].event_time_ms < cutoff:
            self._buf.popleft()

    def quotes(self) -> List[TickQuote]:
        """Frozen window contents (oldest→newest) for Rust tick-eval buffer_quotes."""
        return list(self._buf)

    def history_sec(self) -> float:
        if len(self._buf) < 2:
            return 0.0
        return (self._buf[-1].event_time_ms - self._buf[0].event_time_ms) / 1000.0

    def velocity_bps(self, lookback_sec: float) -> Optional[float]:
        if not self._buf:
            return None
        latest = self._buf[-1]
        target_ms = latest.event_time_ms - int(lookback_sec * 1000)
        if self._buf[0].event_time_ms > target_ms:
            return None
        hist = None
        for q in self._buf:
            if q.event_time_ms >= target_ms:
                hist = q
                break
        if hist is None or hist.mid_price <= 0:
            return None
        return ((latest.mid_price - hist.mid_price) / hist.mid_price) * 10000.0

    def realized_range_bps(self, lookback_sec: float = TICK_VOL_WINDOW_SEC) -> Optional[float]:
        if not self._buf:
            return None
        latest = self._buf[-1]
        cutoff = latest.event_time_ms - int(lookback_sec * 1000)
        mids = [q.mid_price for q in self._buf if q.event_time_ms >= cutoff and q.mid_price > 0]
        if len(mids) < 2:
            return None
        lo, hi = min(mids), max(mids)
        mid = latest.mid_price
        if mid <= 0:
            return None
        return ((hi - lo) / mid) * 10000.0


def macro_trend_bps(closes: Sequence[float], lookback_bars: int) -> Optional[float]:
    """Return bps from close[-lookback] to close[-1] using completed 15m closes."""
    if lookback_bars < 1 or len(closes) < lookback_bars:
        return None
    past = float(closes[-lookback_bars])
    latest = float(closes[-1])
    if past <= 0:
        return None
    return ((latest - past) / past) * 10000.0


def validate_tick_quote(
    quote: TickQuote,
    *,
    prev_update_id: Optional[int] = None,
    max_age_ms: float = MAX_QUOTE_AGE_MS,
) -> Tuple[bool, str]:
    if quote.bid_price <= 0 or quote.ask_price <= 0:
        return False, "INVALID_PRICE"
    if quote.ask_price <= quote.bid_price:
        return False, "INVERTED_BOOK"
    if quote.bid_qty <= 0 or quote.ask_qty <= 0:
        return False, "INVALID_QTY"
    if quote.spread_bps > MAX_VALID_SPREAD_BPS:
        return False, "EXCESSIVE_SPREAD"
    age = quote.receive_time_ms - quote.event_time_ms
    if age > max_age_ms:
        return False, "STALE_QUOTE"
    if prev_update_id is not None and quote.update_id < prev_update_id:
        return False, "OUT_OF_ORDER"
    return True, "VALID"


def build_tick_snapshot(
    quote: TickQuote,
    buf: TickFeatureBuffer,
    completed_closes: Sequence[float],
) -> TickFeatureSnapshot:
    mid = quote.mid_price
    micro_bps = 0.0
    if mid > 0:
        micro_bps = ((quote.micro_price - mid) / mid) * 10000.0
    return TickFeatureSnapshot(
        mid_price=mid,
        spread_bps=quote.spread_bps,
        imbalance=quote.imbalance,
        micro_vs_mid_bps=micro_bps,
        velocity_5s_bps=buf.velocity_bps(5.0),
        velocity_30s_bps=buf.velocity_bps(30.0),
        tick_vol_30s_bps=buf.realized_range_bps(TICK_VOL_WINDOW_SEC),
        macro_trend_16_bps=macro_trend_bps(completed_closes, 16),
        macro_trend_64_bps=macro_trend_bps(completed_closes, 64),
        history_sec=buf.history_sec(),
        quote_age_ms=float(quote.receive_time_ms - quote.event_time_ms),
        update_id=quote.update_id,
        event_time_ms=quote.event_time_ms,
        receive_time_ms=quote.receive_time_ms,
    )


def tick_features_to_evidence(snap: TickFeatureSnapshot, alphabet: str) -> List[Dict[str, Any]]:
    """
    Map tick snapshot → concept factors for evaluate_policy_rules genomes.

    Policy B alphabet: TickBase
    Policy C alphabet: TickWithExhaustion
    """
    factors: List[Dict[str, Any]] = []

    # Macro Trend (prefer 64-bar; fall back to 16)
    macro = snap.macro_trend_64_bps
    if macro is None:
        macro = snap.macro_trend_16_bps
    if macro is None:
        trend_dir = "Neutral"
    elif macro > TREND_FILTER_BPS:
        trend_dir = "Bullish"
    elif macro < -TREND_FILTER_BPS:
        trend_dir = "Bearish"
    else:
        trend_dir = "Neutral"
    factors.append({"concept": "Trend", "present": True, "direction": trend_dir})

    # Micro Momentum from 5s velocity
    v5 = snap.velocity_5s_bps
    if v5 is None:
        mom_dir = "Neutral"
    elif v5 >= VELOCITY_5S_THRESHOLD_BPS:
        mom_dir = "Positive"
    elif v5 <= -VELOCITY_5S_THRESHOLD_BPS:
        mom_dir = "Negative"
    else:
        mom_dir = "Neutral"
    factors.append({"concept": "Momentum", "present": True, "direction": mom_dir})

    # Volatility / activity gate
    vol_present = False
    if snap.tick_vol_30s_bps is not None and snap.tick_vol_30s_bps >= 2.0:
        vol_present = True
    if abs(snap.imbalance) >= IMBALANCE_THRESHOLD:
        vol_present = True
    if v5 is not None and abs(v5) >= VELOCITY_5S_THRESHOLD_BPS:
        vol_present = True
    factors.append({"concept": "Volatility", "present": vol_present, "direction": None})

    # Microstructure confirm (imbalance + microprice side)
    micro_long = snap.imbalance >= IMBALANCE_THRESHOLD and snap.micro_vs_mid_bps >= 0.0
    micro_short = snap.imbalance <= -IMBALANCE_THRESHOLD and snap.micro_vs_mid_bps <= 0.0
    if micro_long:
        factors.append({"concept": "MicroConfirm", "present": True, "direction": "BidHeavy"})
    elif micro_short:
        factors.append({"concept": "MicroConfirm", "present": True, "direction": "AskHeavy"})
    else:
        factors.append({"concept": "MicroConfirm", "present": False, "direction": None})

    if alphabet == "TickWithExhaustion":
        # Exhaustion stretch via micro vs mid + imbalance extremity
        if snap.imbalance >= 0.55 and snap.micro_vs_mid_bps > 1.0:
            factors.append({"concept": "PriceRangeStretch", "present": True, "direction": "ExhaustedHigh"})
        elif snap.imbalance <= -0.55 and snap.micro_vs_mid_bps < -1.0:
            factors.append({"concept": "PriceRangeStretch", "present": True, "direction": "ExhaustedLow"})
        else:
            factors.append({"concept": "PriceRangeStretch", "present": False, "direction": None})

        v30 = snap.velocity_30s_bps
        if v30 is not None and v30 > 8.0:
            factors.append({"concept": "VWAPStretch", "present": True, "direction": "Overbought"})
        elif v30 is not None and v30 < -8.0:
            factors.append({"concept": "VWAPStretch", "present": True, "direction": "Oversold"})
        else:
            factors.append({"concept": "VWAPStretch", "present": False, "direction": None})

        burst = abs(snap.imbalance) >= 0.50
        factors.append({
            "concept": "VolumeBurst",
            "present": burst,
            "direction": "Burst" if burst else None,
        })

    return factors


def predicate_matches(pred: Dict[str, Any], factors: List[Dict[str, Any]]) -> bool:
    target_concept = pred["concept"]
    factor = next((f for f in factors if f["concept"] == target_concept), None)
    pred_present = pred.get("present")
    pred_dir = pred.get("direction")

    if pred_present is False and factor is None:
        return pred_dir is None
    if pred_present is False and factor is not None:
        return (not factor["present"]) and (pred_dir is None)
    if pred_present is True and factor is not None and factor["present"]:
        if pred_dir is None:
            return True
        return factor.get("direction") == pred_dir
    if pred_present is True and (factor is None or not factor["present"]):
        return False
    if pred_present is None and factor is not None:
        if pred_dir is None:
            return True
        return factor["present"] and (factor.get("direction") == pred_dir)
    if pred_present is None and factor is None:
        return pred_dir is None
    return False


def evaluate_policy_rules(genome: Dict[str, Any], factors: List[Dict[str, Any]]) -> str:
    rules = genome.get("rules", [])
    for rule in rules:
        when = rule.get("when", [])
        if all(predicate_matches(p, factors) for p in when):
            return rule.get("action", "NO_TRADE")
    return genome.get("unmatched_action", "NO_TRADE")


def evaluate_tick_policies(
    snap: TickFeatureSnapshot,
    policy_b_genome: Dict[str, Any],
    policy_c_genome: Dict[str, Any],
) -> Tuple[str, str, List[Dict[str, Any]], List[Dict[str, Any]]]:
    factors_b = tick_features_to_evidence(snap, "TickBase")
    factors_c = tick_features_to_evidence(snap, "TickWithExhaustion")
    action_b = evaluate_policy_rules(policy_b_genome, factors_b)
    action_c = evaluate_policy_rules(policy_c_genome, factors_c)
    return action_b, action_c, factors_b, factors_c


REC_HYGIENE_PROTOCOL_ID = "RP-608-SHADOW-TICK-REC-HYGIENE-001"
REC_HYGIENE_REV = 4
REC_CONFIRM_TICKS = 3
# Live evidence 2026-11-25: 3 WS packets pass confirm in <100ms → SHORT every sticky cycle.
# Rev3 soak: even with 1.5s gaps, live max wall span≈2.3s — still never hit 8s.
REC_CONFIRM_MIN_SPAN_MS = 5_000
REC_CONFIRM_GAP_TOLERANCE_MS = 5_000
REC_STICKY_MS = 12_000
# After sticky expires, block re-arm so L1 AskHeavy cannot re-fire immediately.
REC_REARM_COOLDOWN_MS = 30_000


def _is_tradeable(action: str) -> bool:
    return str(action or "").upper() in ("LONG", "SHORT")


def microconfirm_gate_b(
    action_b: str, factors_b: List[Dict[str, Any]]
) -> Tuple[str, bool]:
    """Downgrade soft B LONG/SHORT lacking aligned MicroConfirm."""
    a = str(action_b or "NO_TRADE").upper()
    if not _is_tradeable(a):
        return a, False
    mc = next((f for f in factors_b if f.get("concept") == "MicroConfirm"), None)
    if a == "LONG":
        ok = bool(mc and mc.get("present") and mc.get("direction") == "BidHeavy")
    elif a == "SHORT":
        ok = bool(mc and mc.get("present") and mc.get("direction") == "AskHeavy")
    else:
        ok = False
    return (a, False) if ok else ("NO_TRADE", True)


@dataclass
class RecommendationStabilizer:
    """
    Track C v3: MicroConfirm + N tradeable evals spanning min wall-clock,
    with brief gap tolerance (L1 MC flicker), sticky on drop, rearm cooldown.
    Matches Rust RecommendationStabilizer.
    """

    confirm_ticks: int = REC_CONFIRM_TICKS
    confirm_min_span_ms: int = REC_CONFIRM_MIN_SPAN_MS
    confirm_gap_tolerance_ms: int = REC_CONFIRM_GAP_TOLERANCE_MS
    sticky_ms: int = REC_STICKY_MS
    rearm_cooldown_ms: int = REC_REARM_COOLDOWN_MS
    candidate_b: Optional[str] = None
    candidate_streak: int = 0
    candidate_first_ms: int = 0
    last_tradeable_ms: int = 0
    sticky_b: Optional[str] = None
    sticky_c: Optional[str] = None
    sticky_until_ms: int = 0
    rearm_after_ms: int = 0

    def update(
        self,
        now_ms: int,
        raw_b: str,
        raw_c: str,
        factors_b: List[Dict[str, Any]],
    ) -> Tuple[str, str, Dict[str, Any]]:
        now_ms = int(now_ms)
        gated_b, mc_gated = microconfirm_gate_b(raw_b, factors_b)
        stamp: Dict[str, Any] = {
            "protocol_id": REC_HYGIENE_PROTOCOL_ID,
            "hygiene_rev": REC_HYGIENE_REV,
            "microconfirm_gated": mc_gated,
            "confirm_pending": False,
            "confirm_streak": 0,
            "confirm_span_ms": 0,
            "gap_tolerated": False,
            "rearm_blocked": False,
            "sticky_held": False,
            "raw_b": raw_b,
            "raw_c": raw_c,
            "out_b": gated_b,
            "out_c": raw_c,
        }
        if _is_tradeable(gated_b):
            # Already confirmed this side — keep emitting while signal holds.
            if self.sticky_b == gated_b:
                self.sticky_c = raw_c
                self.sticky_until_ms = 0
                stamp["out_b"] = gated_b
                stamp["out_c"] = raw_c
                return gated_b, raw_c, stamp
            if self.candidate_b == gated_b:
                self.candidate_streak += 1
            else:
                self.candidate_b = gated_b
                self.candidate_streak = 1
                self.candidate_first_ms = now_ms
            self.last_tradeable_ms = now_ms
            span = max(0, now_ms - int(self.candidate_first_ms))
            stamp["confirm_streak"] = self.candidate_streak
            stamp["confirm_span_ms"] = span
            rearm_blocked = now_ms < int(self.rearm_after_ms or 0)
            stamp["rearm_blocked"] = rearm_blocked
            span_ok = span >= int(self.confirm_min_span_ms)
            ticks_ok = self.candidate_streak >= int(self.confirm_ticks)
            if ticks_ok and span_ok and not rearm_blocked:
                self.sticky_b = gated_b
                self.sticky_c = raw_c
                # Sticky arms on drop, not while continuously tradeable.
                self.sticky_until_ms = 0
                # Drop candidate so gap-tolerance cannot override sticky.
                self.candidate_b = None
                self.candidate_streak = 0
                self.candidate_first_ms = 0
                self.last_tradeable_ms = 0
                stamp["out_b"] = gated_b
                stamp["out_c"] = raw_c
                return gated_b, raw_c, stamp
            stamp["confirm_pending"] = True
            stamp["out_b"] = "NO_TRADE"
            return "NO_TRADE", raw_c, stamp

        # Non-tradeable: tolerate brief MC gaps without resetting the candidate.
        # Only while still confirming (not after sticky_b is armed).
        # Live: span often crosses the threshold only on gap ticks (tradeable max
        # span stayed <5s) — allow confirm when streak+span already met.
        if (
            self.candidate_b
            and not self.sticky_b
            and int(self.last_tradeable_ms or 0) > 0
            and (now_ms - int(self.last_tradeable_ms))
            <= int(self.confirm_gap_tolerance_ms)
        ):
            span = max(0, now_ms - int(self.candidate_first_ms))
            stamp["confirm_streak"] = self.candidate_streak
            stamp["confirm_span_ms"] = span
            stamp["gap_tolerated"] = True
            rearm_blocked = now_ms < int(self.rearm_after_ms or 0)
            stamp["rearm_blocked"] = rearm_blocked
            span_ok = span >= int(self.confirm_min_span_ms)
            ticks_ok = self.candidate_streak >= int(self.confirm_ticks)
            if ticks_ok and span_ok and not rearm_blocked:
                confirmed = self.candidate_b
                self.sticky_b = confirmed
                self.sticky_c = raw_c
                self.sticky_until_ms = 0
                self.candidate_b = None
                self.candidate_streak = 0
                self.candidate_first_ms = 0
                self.last_tradeable_ms = 0
                stamp["confirm_pending"] = False
                stamp["out_b"] = confirmed
                stamp["out_c"] = raw_c
                return confirmed, raw_c, stamp
            stamp["confirm_pending"] = True
            stamp["out_b"] = "NO_TRADE"
            return "NO_TRADE", raw_c, stamp

        self.candidate_b = None
        self.candidate_streak = 0
        self.candidate_first_ms = 0
        self.last_tradeable_ms = 0
        if self.sticky_b and not _is_tradeable(gated_b):
            if int(self.sticky_until_ms or 0) <= 0:
                self.sticky_until_ms = now_ms + int(self.sticky_ms)
            if now_ms <= int(self.sticky_until_ms):
                stamp["sticky_held"] = True
                stamp["out_b"] = self.sticky_b
                stamp["out_c"] = self.sticky_c or raw_c
                return stamp["out_b"], stamp["out_c"], stamp
            self.rearm_after_ms = now_ms + int(self.rearm_cooldown_ms)
        self.sticky_b = None
        self.sticky_c = None
        self.sticky_until_ms = 0
        stamp["out_b"] = gated_b
        stamp["out_c"] = raw_c
        return gated_b, raw_c, stamp


def evaluate_tick_policies_hygiened(
    snap: TickFeatureSnapshot,
    policy_b_genome: Dict[str, Any],
    policy_c_genome: Dict[str, Any],
    stab: RecommendationStabilizer,
) -> Tuple[str, str, List[Dict[str, Any]], List[Dict[str, Any]], Dict[str, Any]]:
    raw_b, raw_c, factors_b, factors_c = evaluate_tick_policies(
        snap, policy_b_genome, policy_c_genome
    )
    now_ms = int(getattr(snap, "receive_time_ms", 0) or 0)
    b, c, stamp = stab.update(now_ms, raw_b, raw_c, factors_b)
    return b, c, factors_b, factors_c, stamp


@dataclass
class TickEmissionGate:
    """Refractory + action-change emission control (deterministic)."""

    refractory_ms: float = MIN_REFRACTORY_MS
    last_emit_ms: int = 0
    last_action_b: Optional[str] = None
    last_action_c: Optional[str] = None

    def should_emit(self, now_ms: int, action_b: str, action_c: str) -> Tuple[bool, str]:
        changed = (action_b != self.last_action_b) or (action_c != self.last_action_c)
        if not changed:
            # Suppress identical spam (including sustained NO_TRADE).
            return False, "UNCHANGED"
        if self.last_emit_ms > 0 and (now_ms - self.last_emit_ms) < self.refractory_ms:
            # Directional flip inside refractory: still allow (action change).
            # Only block if we somehow double-fire; plan: emit on action change even in refractory
            # for flips, but Gate C uses refractory on all active emits.
            # Spec: min refractory 1.0s — apply to all emits including changes.
            return False, "REFRACTORY"
        return True, "OK"

    def mark_emitted(self, now_ms: int, action_b: str, action_c: str) -> None:
        self.last_emit_ms = now_ms
        self.last_action_b = action_b
        self.last_action_c = action_c


def snapshot_to_dict(snap: TickFeatureSnapshot) -> Dict[str, Any]:
    return {
        "mid_price": round(snap.mid_price, 8),
        "spread_bps": round(snap.spread_bps, 6),
        "imbalance": round(snap.imbalance, 6),
        "micro_vs_mid_bps": round(snap.micro_vs_mid_bps, 4),
        "velocity_5s_bps": None if snap.velocity_5s_bps is None else round(snap.velocity_5s_bps, 4),
        "velocity_30s_bps": None if snap.velocity_30s_bps is None else round(snap.velocity_30s_bps, 4),
        "tick_vol_30s_bps": None if snap.tick_vol_30s_bps is None else round(snap.tick_vol_30s_bps, 4),
        "macro_trend_16_bps": None if snap.macro_trend_16_bps is None else round(snap.macro_trend_16_bps, 4),
        "macro_trend_64_bps": None if snap.macro_trend_64_bps is None else round(snap.macro_trend_64_bps, 4),
        "history_sec": round(snap.history_sec, 3),
        "quote_age_ms": round(snap.quote_age_ms, 2),
        "update_id": snap.update_id,
        "event_time_ms": snap.event_time_ms,
        "receive_time_ms": snap.receive_time_ms,
    }
