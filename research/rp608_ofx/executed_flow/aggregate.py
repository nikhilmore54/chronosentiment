"""
Windowed signed executed-volume aggregation (OFX research-only).

Buy aggressor → +qty; sell aggressor → −qty contribution to signed_delta_qty.
Resting L1 quantities and unsigned candle volume are never inputs.
"""

from __future__ import annotations

from dataclasses import asdict, dataclass, field
from typing import Any, Dict, Iterable, List, Optional, Sequence, Set


DEFAULT_WINDOW_MS = 60_000

# Side classification provenance (never silently promote)
SIDE_DIRECT = "DIRECT"
SIDE_DERIVED_VALIDATED = "DERIVED_VALIDATED"
SIDE_UNAVAILABLE = "UNAVAILABLE"

_SIGNABLE = frozenset({SIDE_DIRECT, SIDE_DERIVED_VALIDATED})


@dataclass(frozen=True)
class SideClass:
    """Aggressor direction with provenance — qty lives on the same record as side."""

    aggressor: str  # BUY | SELL | UNKNOWN
    classification: str  # DIRECT | DERIVED_VALIDATED | UNAVAILABLE
    evidence: str = ""


@dataclass(frozen=True)
class ExecutedTrade:
    """One executed print with side and quantity bound on the same object."""

    instrument_id: str
    timestamp_ms: int
    price: float
    quantity: float
    side: SideClass
    source: str
    sequence_id: Optional[str] = None
    raw_ref: Optional[str] = None


@dataclass(frozen=True)
class CandleContext:
    """Optional OHLC context — never used to decide aggressor side."""

    open: float
    high: float
    low: float
    close: float
    direction: str  # UP | DOWN | FLAT
    unsigned_volume: Optional[float] = None  # informational only; never signed into ΔQ


@dataclass
class ExecutedFlowWindowResult:
    source: str
    side_classification_mode: str
    instrument_id: str
    window_start_ms: int
    window_end_ms: int
    window_ms: int
    buy_aggressor_qty: float
    sell_aggressor_qty: float
    total_classified_qty: float
    signed_delta_qty: float
    classification_coverage: Optional[float]
    n_trades_in_window: int
    n_classified: int
    n_unclassified: int
    n_invalid: int
    n_duplicates_skipped: int
    n_out_of_order: int
    data_quality: str  # OK | INCOMPLETE | UNAVAILABLE
    status: str
    candle: Optional[Dict[str, Any]] = None
    notes: List[str] = field(default_factory=list)

    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)


def candle_direction_from_ohlc(open_: float, close: float, *, eps: float = 0.0) -> str:
    if close > open_ + eps:
        return "UP"
    if close < open_ - eps:
        return "DOWN"
    return "FLAT"


def _valid_qty(q: float) -> bool:
    try:
        return float(q) > 0.0 and float(q) == float(q)  # not NaN
    except (TypeError, ValueError):
        return False


def aggregate_executed_flow(
    trades: Sequence[ExecutedTrade],
    *,
    instrument_id: str,
    window_start_ms: int,
    window_end_ms: int,
    source: str,
    candle: Optional[CandleContext] = None,
    allow_derived: bool = True,
) -> ExecutedFlowWindowResult:
    """
    Aggregate signed executed volume over [window_start_ms, window_end_ms).

    Only trades whose SideClass.classification is DIRECT (always) or
    DERIVED_VALIDATED (when allow_derived=True) contribute to signed delta.
    Side and quantity are taken from the same ExecutedTrade — never zip'd.
    """
    notes: List[str] = []
    window_ms = int(window_end_ms) - int(window_start_ms)
    if window_ms <= 0:
        return ExecutedFlowWindowResult(
            source=source,
            side_classification_mode="none",
            instrument_id=instrument_id,
            window_start_ms=int(window_start_ms),
            window_end_ms=int(window_end_ms),
            window_ms=window_ms,
            buy_aggressor_qty=0.0,
            sell_aggressor_qty=0.0,
            total_classified_qty=0.0,
            signed_delta_qty=0.0,
            classification_coverage=None,
            n_trades_in_window=0,
            n_classified=0,
            n_unclassified=0,
            n_invalid=0,
            n_duplicates_skipped=0,
            n_out_of_order=0,
            data_quality="UNAVAILABLE",
            status="INVALID_WINDOW",
            candle=_candle_dict(candle),
            notes=["window_end_ms must be > window_start_ms"],
        )

    buy = 0.0
    sell = 0.0
    n_in = 0
    n_classified = 0
    n_unclassified = 0
    n_invalid = 0
    n_dup = 0
    n_ooo = 0
    modes: Set[str] = set()
    seen: Set[str] = set()
    last_ts: Optional[int] = None
    unsigned_attempted = 0.0

    # Sort by time for OOO detection; aggregation itself is commutative for ΔQ
    ordered = sorted(
        (t for t in trades if str(t.instrument_id) == str(instrument_id)),
        key=lambda t: (int(t.timestamp_ms), str(t.sequence_id or "")),
    )

    for t in ordered:
        ts = int(t.timestamp_ms)
        if ts < int(window_start_ms) or ts >= int(window_end_ms):
            continue
        n_in += 1

        if last_ts is not None and ts < last_ts:
            n_ooo += 1
        last_ts = ts

        dup_key = None
        if t.sequence_id is not None:
            dup_key = f"{t.instrument_id}|{t.sequence_id}"
        elif t.raw_ref is not None:
            dup_key = f"{t.instrument_id}|{t.raw_ref}"
        if dup_key is not None:
            if dup_key in seen:
                n_dup += 1
                continue
            seen.add(dup_key)

        if not _valid_qty(t.quantity):
            n_invalid += 1
            continue

        qty = float(t.quantity)
        unsigned_attempted += qty
        cls = str(t.side.classification)
        agg = str(t.side.aggressor).upper()
        modes.add(cls)

        signable = cls == SIDE_DIRECT or (cls == SIDE_DERIVED_VALIDATED and allow_derived)
        if not signable or agg not in ("BUY", "SELL"):
            n_unclassified += 1
            continue

        # Quantity stays bound to this trade's aggressor — no cross-record pairing
        if agg == "BUY":
            buy += qty
        else:
            sell += qty
        n_classified += 1

    total_classified = buy + sell
    signed = buy - sell
    coverage = None
    if unsigned_attempted > 0:
        coverage = total_classified / unsigned_attempted

    if n_in == 0:
        dq, status = "UNAVAILABLE", "NO_TRADES_IN_WINDOW"
        notes.append("No executed trades in observation window for instrument")
    elif n_classified == 0:
        dq, status = "UNAVAILABLE", "NO_CLASSIFIED_AGGRESSOR_VOLUME"
        notes.append("Executed qty present but aggressor side unavailable/unvalidated for all rows")
    elif coverage is not None and coverage < 1.0:
        dq, status = "INCOMPLETE", "PARTIAL_COVERAGE"
    else:
        dq, status = "OK", "OK"

    if candle is not None and candle.unsigned_volume is not None:
        notes.append(
            "candle.unsigned_volume is contextual metadata only — excluded from signed_delta_qty"
        )

    mode = ",".join(sorted(modes)) if modes else "none"
    return ExecutedFlowWindowResult(
        source=source,
        side_classification_mode=mode,
        instrument_id=str(instrument_id),
        window_start_ms=int(window_start_ms),
        window_end_ms=int(window_end_ms),
        window_ms=window_ms,
        buy_aggressor_qty=round(buy, 12),
        sell_aggressor_qty=round(sell, 12),
        total_classified_qty=round(total_classified, 12),
        signed_delta_qty=round(signed, 12),
        classification_coverage=None if coverage is None else round(coverage, 6),
        n_trades_in_window=n_in,
        n_classified=n_classified,
        n_unclassified=n_unclassified,
        n_invalid=n_invalid,
        n_duplicates_skipped=n_dup,
        n_out_of_order=n_ooo,
        data_quality=dq,
        status=status,
        candle=_candle_dict(candle),
        notes=notes,
    )


def _candle_dict(candle: Optional[CandleContext]) -> Optional[Dict[str, Any]]:
    if candle is None:
        return None
    return {
        "open": candle.open,
        "high": candle.high,
        "low": candle.low,
        "close": candle.close,
        "direction": candle.direction,
        "unsigned_volume": candle.unsigned_volume,
        "note": "direction/volume do not determine aggressor side",
    }


def iter_dicts(rows: Iterable[Dict[str, Any]]) -> Iterable[Dict[str, Any]]:
    for r in rows:
        if isinstance(r, dict):
            yield r
