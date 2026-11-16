#!/usr/bin/env python3
"""
RP-608-OFX — NSE derived-aggressor volume PAPER trade book (research-only).

Parallel to baseline NSE L1 OF paper. Does NOT mutate OF ±0.30 / reverse-OF.
Authority 0%. Construct: DERIVED_VALIDATED (Stage 0D), never DIRECT.

Same OFX volume-paper logic as crypto (`rp608_ofx_volume_paper_trades.py`):
  Entry: directional decision (RECOMMEND_LONG/SHORT → LONG/SHORT) + entry tick
         volume ≥ 3× mean of prior 20 completed ticks (raw sizes only; no L1 OF /
         other order-flow signals).
  Exit:  opposing aggressor qty vs entry aggressor qty in −10%..+10% band
         (window primary; cumulative only if also in-band); capped clubbed
         completion after 15m; early-adverse abort; min-hold + persist.
         MIXED entry tempo suppressed (path analysis 2026-11-15).
         Per-symbol only (never pools scrips). Session square-off force-flats.
"""

from __future__ import annotations

import json
import os
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional, Set, Tuple

WORKSPACE = Path(__file__).resolve().parent.parent
import sys

sys.path.insert(0, str(WORKSPACE))

from research.rp608_ofx.executed_flow.aggregate import ExecutedFlowWindowResult  # noqa: E402
from research.rp608_ofx.executed_flow.nse_inferred import trades_from_nse_inferred_ticks  # noqa: E402
from research.rp608_ofx.executed_flow.volume_mark import (  # noqa: E402
    OFX_CLUBBED_MAX_OVERSHOOT,
    OFX_CLUBBED_MIN_HOLD_MS,
    OFX_CLUBBED_OPPOSING_EXIT_REASON,
    OFX_EARLY_ADVERSE_BPS,
    OFX_EARLY_ADVERSE_EXIT_REASON,
    OFX_EARLY_ADVERSE_WINDOW_MS,
    OFX_ENTRY_SURGE_LOOKBACK_TICKS,
    OFX_ENTRY_SURGE_MULTIPLE,
    OFX_TEMPO_LOOKBACK_TICKS,
    OFX_VOLUME_IMBALANCE_THRESHOLD,
    OFX_VOLUME_LABEL,
    OFX_VOLUME_MATCH_DEV_MAX,
    OFX_VOLUME_MATCH_DEV_MIN,
    classify_ofx_aggressor_tempo,
    classify_ofx_volume_support,
    entry_aggressor_qty_for_side,
    is_ofx_clubbed_opposing_completion,
    is_ofx_volume_match_exit,
    ofx_entry_qty_for_tempo,
    ofx_entry_surge_from_trades,
    opposite_aggressor_label,
    should_suppress_ofx_entry_for_tempo,
)
from scripts.rp608_ofx_nse_volume_flow_mark import (  # noqa: E402
    canonicalize_nse_symbol,
    global_nse_volume_flow_buffer,
)
from scripts.rp608_nse_equity_costs import calc_nse_of_paper_accounting  # noqa: E402
from scripts.rp608_nse_of_paper_ledger import (  # noqa: E402
    quote_as_book_dict,
    size_nse_of_paper_from_entry_signal,
)
from scripts.rp608_order_flow_mark import l1_from_nse_quote  # noqa: E402

PROTOCOL_ID = "RP-608-OFX-NSE-VOLUME-PAPER-001"
CONSTRUCT = "DERIVED_VALIDATED_AGGRESSOR_VOLUME"
MIN_HOLD_MS = 300_000
OPPOSITE_PERSIST_MS = 30_000
DEFAULT_FALLBACK_NOTIONAL_INR = 10_000.0
EXIT_REASON_MATCH = "OPPOSING_VOLUME_MATCH"
EXIT_REASON_CLUBBED = OFX_CLUBBED_OPPOSING_EXIT_REASON


def _iso(ms: int) -> str:
    return datetime.fromtimestamp(ms / 1000.0, tz=timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def _bid_ask(quote: Optional[Any]) -> Tuple[Optional[float], Optional[float]]:
    l1 = l1_from_nse_quote(quote)
    try:
        bid = float(l1.get("bid_price") or 0.0) or None
    except (TypeError, ValueError):
        bid = None
    try:
        ask = float(l1.get("ask_price") or 0.0) or None
    except (TypeError, ValueError):
        ask = None
    return bid, ask


def _mid_px(quote: Optional[Any]) -> Optional[float]:
    bid, ask = _bid_ask(quote)
    if bid is not None and ask is not None and bid > 0 and ask > 0:
        return (bid + ask) / 2.0
    if ask is not None and ask > 0:
        return ask
    if bid is not None and bid > 0:
        return bid
    return None


def _signed_bps(direction: str, entry: float, px: float) -> float:
    if entry <= 0 or px <= 0:
        return 0.0
    d = str(direction or "").upper()
    if d == "LONG":
        return (px - entry) / entry * 1e4
    if d == "SHORT":
        return (entry - px) / entry * 1e4
    return 0.0


def _entry_px(direction: str, quote: Optional[Any], fallback: Optional[float]) -> Optional[float]:
    bid, ask = _bid_ask(quote)
    d = str(direction).upper()
    if d == "LONG" and ask is not None:
        return ask
    if d == "SHORT" and bid is not None:
        return bid
    try:
        return float(fallback) if fallback is not None else None
    except (TypeError, ValueError):
        return None


def _exit_px(direction: str, quote: Optional[Any], fallback: Optional[float]) -> Optional[float]:
    bid, ask = _bid_ask(quote)
    d = str(direction).upper()
    if d == "LONG" and bid is not None:
        return bid
    if d == "SHORT" and ask is not None:
        return ask
    try:
        return float(fallback) if fallback is not None else None
    except (TypeError, ValueError):
        return None


def _result_from_snapshot(volume_snapshot: Dict[str, Any], *, now: int) -> ExecutedFlowWindowResult:
    return ExecutedFlowWindowResult(
        source=str(volume_snapshot.get("phase") or "nse_ofx_volume_snapshot"),
        side_classification_mode="DERIVED_VALIDATED",
        instrument_id=str(volume_snapshot.get("symbol") or ""),
        window_start_ms=int(volume_snapshot.get("window_start_ms") or 0),
        window_end_ms=int(volume_snapshot.get("window_end_ms") or now),
        window_ms=int(volume_snapshot.get("window_ms") or 0),
        buy_aggressor_qty=float(volume_snapshot.get("buy_aggressor_qty") or 0.0),
        sell_aggressor_qty=float(volume_snapshot.get("sell_aggressor_qty") or 0.0),
        total_classified_qty=float(volume_snapshot.get("total_classified_qty") or 0.0),
        signed_delta_qty=float(volume_snapshot.get("signed_delta_qty") or 0.0),
        classification_coverage=volume_snapshot.get("classification_coverage"),
        n_trades_in_window=int(volume_snapshot.get("buffer_rows_in_window") or 0),
        n_classified=1 if float(volume_snapshot.get("total_classified_qty") or 0) > 0 else 0,
        n_unclassified=0,
        n_invalid=0,
        n_duplicates_skipped=0,
        n_out_of_order=0,
        data_quality=str(volume_snapshot.get("data_quality") or "UNAVAILABLE"),
        status=str(volume_snapshot.get("status") or "UNAVAILABLE"),
    )


def _direction_from_eval(eval_record: Dict[str, Any]) -> Optional[str]:
    """Map NSE decision → LONG/SHORT (crypto Policy B parity)."""
    raw = str(
        (eval_record or {}).get("decision")
        or (eval_record or {}).get("direction")
        or ""
    ).upper()
    if raw in ("RECOMMEND_LONG", "LONG", "BUY"):
        return "LONG"
    if raw in ("RECOMMEND_SHORT", "SHORT", "SELL"):
        return "SHORT"
    return None


def _sync_match_band(rec: Dict[str, Any]) -> None:
    """Keep open stamps on the live crypto OFX ±10% + clubbed-completion exit."""
    rec["ofx_match_dev_min"] = OFX_VOLUME_MATCH_DEV_MIN
    rec["ofx_match_dev_max"] = OFX_VOLUME_MATCH_DEV_MAX
    rec["exit_rule"] = (
        f"per_symbol_opposing_volume_match_pm{int(OFX_VOLUME_MATCH_DEV_MAX * 100)}pct"
        f"|clubbed_opposing_completion"
        f"+min_hold_{MIN_HOLD_MS}ms+persist_{OPPOSITE_PERSIST_MS}ms"
    )
    rec["ofx_match_scope"] = "PER_SYMBOL_ONLY"
    if not rec.get("match_symbol"):
        rec["match_symbol"] = canonicalize_nse_symbol(rec.get("symbol"))


class OfxNseVolumePaperManager:
    """Multi-symbol OFX NSE volume paper trades under ofx_tick_capture_v1/."""

    def __init__(self, root: Path):
        self.root = Path(root)
        self.root.mkdir(parents=True, exist_ok=True)
        self.outcomes_path = self.root / "ofx_nse_volume_paper_outcomes.jsonl"
        self.opens_path = self.root / "ofx_nse_volume_paper_opens.jsonl"
        self.open_state_path = self.root / "ofx_nse_volume_paper_open.json"
        self.summary_path = self.root / "ofx_nse_volume_paper_summary.json"
        self.meta_path = self.root / "ofx_nse_volume_paper_meta.json"
        self.min_hold_ms = MIN_HOLD_MS
        self.opposite_persist_ms = OPPOSITE_PERSIST_MS
        self._opens: Dict[str, Dict[str, Any]] = {}
        self._known_ids: Set[str] = set()
        self._last_entry_bar_by_symbol: Dict[str, int] = {}
        self._load_known_ids()
        self._load_meta()
        self._restore_opens()
        self._rebuild_summary()

    def _load_known_ids(self) -> None:
        if not self.outcomes_path.exists():
            return
        with self.outcomes_path.open(encoding="utf-8") as fh:
            for line in fh:
                if not line.strip():
                    continue
                try:
                    row = json.loads(line)
                except json.JSONDecodeError:
                    continue
                tid = row.get("ofx_trade_id")
                if tid:
                    self._known_ids.add(str(tid))

    def _load_meta(self) -> None:
        if not self.meta_path.exists():
            return
        try:
            data = json.loads(self.meta_path.read_text(encoding="utf-8"))
            last = data.get("last_entry_bar_by_symbol") or {}
            if isinstance(last, dict):
                self._last_entry_bar_by_symbol = {
                    str(k): int(v) for k, v in last.items() if v is not None
                }
        except Exception:
            pass

    def _write_meta(self) -> None:
        payload = {
            "protocol_id": PROTOCOL_ID,
            "last_entry_bar_by_symbol": self._last_entry_bar_by_symbol,
        }
        tmp = self.meta_path.with_suffix(".tmp")
        with open(tmp, "w", encoding="utf-8") as f:
            json.dump(payload, f)
            f.flush()
            os.fsync(f.fileno())
        os.replace(tmp, self.meta_path)

    def _restore_opens(self) -> None:
        if not self.open_state_path.exists():
            return
        try:
            data = json.loads(self.open_state_path.read_text(encoding="utf-8"))
        except Exception:
            return
        trades = data.get("trades") if isinstance(data, dict) else None
        if not isinstance(trades, list):
            # single-trade legacy shape
            if isinstance(data, dict) and data.get("open") and isinstance(data.get("trade"), dict):
                trades = [data["trade"]]
            else:
                return
        for t in trades:
            if not isinstance(t, dict) or not t.get("active"):
                continue
            tid = str(t.get("ofx_trade_id") or "")
            if tid and tid in self._known_ids:
                continue
            sym = canonicalize_nse_symbol(t.get("symbol") or t.get("match_symbol"))
            if sym:
                t["symbol"] = sym
                t["match_symbol"] = canonicalize_nse_symbol(t.get("match_symbol") or sym)
                _sync_match_band(t)
                self._opens[sym] = t

    def _persist_opens(self) -> None:
        trades = list(self._opens.values())
        payload = {
            "open": bool(trades),
            "n_open": len(trades),
            "protocol_id": PROTOCOL_ID,
            "execution_mode": "PAPER",
            "live_order_authority": 0.0,
            "construct": CONSTRUCT,
            "side_classification_mode": "DERIVED_VALIDATED",
            "baseline_of_inert": True,
            "trades": trades,
        }
        tmp = self.open_state_path.with_name(
            f"{self.open_state_path.stem}.{os.getpid()}.{time.time_ns()}.tmp"
        )
        with open(tmp, "w", encoding="utf-8") as f:
            json.dump(payload, f, indent=2)
            f.flush()
            os.fsync(f.fileno())
        os.replace(tmp, self.open_state_path)

    def _append_jsonl(self, path: Path, row: Dict[str, Any]) -> None:
        with path.open("a", encoding="utf-8") as f:
            f.write(json.dumps(row, separators=(",", ":")) + "\n")
            f.flush()

    def _rebuild_summary(self) -> Dict[str, Any]:
        wins = losses = 0
        net = gross = friction = 0.0
        n = 0
        if self.outcomes_path.exists():
            with self.outcomes_path.open(encoding="utf-8") as fh:
                for line in fh:
                    if not line.strip():
                        continue
                    try:
                        row = json.loads(line)
                    except json.JSONDecodeError:
                        continue
                    n += 1
                    try:
                        pnl = float(row.get("net_pnl_inr") or 0.0)
                    except (TypeError, ValueError):
                        pnl = 0.0
                    if pnl > 0:
                        wins += 1
                    else:
                        losses += 1
                    net += pnl
                    try:
                        gross += float(row.get("gross_pnl_inr") or 0.0)
                    except (TypeError, ValueError):
                        pass
                    try:
                        friction += float(row.get("friction_inr") or 0.0)
                    except (TypeError, ValueError):
                        pass
        summary = {
            "protocol_id": PROTOCOL_ID,
            "construct": CONSTRUCT,
            "side_classification_mode": "DERIVED_VALIDATED",
            "baseline_of_inert": True,
            "live_order_authority": 0.0,
            "n_trades": n,
            "n_wins": wins,
            "n_losses": losses,
            "win_rate_pct": round(100.0 * wins / n, 2) if n else None,
            "cumulative_net_pnl_inr": round(net, 4),
            "cumulative_gross_pnl_inr": round(gross, 4),
            "cumulative_friction_inr": round(friction, 4),
            "n_open": len(self._opens),
            "updated_utc": datetime.now(timezone.utc).isoformat(),
            "open": bool(self._opens),
        }
        tmp = self.summary_path.with_suffix(".tmp")
        with open(tmp, "w", encoding="utf-8") as f:
            json.dump(summary, f, indent=2)
            f.flush()
            os.fsync(f.fileno())
        os.replace(tmp, self.summary_path)
        return summary

    def summary(self) -> Dict[str, Any]:
        if self.summary_path.exists():
            try:
                return json.loads(self.summary_path.read_text(encoding="utf-8"))
            except Exception:
                pass
        return self._rebuild_summary()

    def open_snapshot(self) -> List[Dict[str, Any]]:
        return list(self._opens.values())

    def try_open_from_eval(
        self,
        *,
        symbol: str,
        eval_record: Dict[str, Any],
        volume_snapshot: Dict[str, Any],
        quote: Optional[Any] = None,
        now_ms: Optional[int] = None,
    ) -> Optional[Dict[str, Any]]:
        """Open OFX volume paper when decision side + 3×/20-tick surge pass."""
        now = int(now_ms if now_ms is not None else time.time() * 1000)
        sym = canonicalize_nse_symbol(symbol)
        if not sym:
            return None

        direction = _direction_from_eval(eval_record)
        if direction not in ("LONG", "SHORT"):
            return None

        # Entry volume mark must be for this symbol only — never another scrip's window
        snap_sym = canonicalize_nse_symbol(
            (volume_snapshot or {}).get("symbol") or symbol
        )
        if snap_sym != sym:
            return None

        # Entry gate: raw tick surge only (no L1 OF / other order-flow signals)
        surge = self._entry_surge_gate(symbol=sym, direction=direction, now_ms=now)
        if not surge.get("passed"):
            return None

        trades = self._recent_executed_trades(symbol=sym)
        entry_side = "BUY" if direction == "LONG" else "SELL"
        opp_side = "SELL" if direction == "LONG" else "BUY"
        entry_tempo = classify_ofx_aggressor_tempo(
            trades, aggressor_side=entry_side, lookback=OFX_TEMPO_LOOKBACK_TICKS, as_of_ms=now
        )
        opp_tempo = classify_ofx_aggressor_tempo(
            trades, aggressor_side=opp_side, lookback=OFX_TEMPO_LOOKBACK_TICKS, as_of_ms=now
        )
        tempo_gate = should_suppress_ofx_entry_for_tempo(
            direction=direction, entry_side_tempo=entry_tempo, opposite_side_tempo=opp_tempo
        )
        if tempo_gate.get("suppress"):
            return None

        # Optional imbalance telemetry only (never gates entry)
        result = _result_from_snapshot({**(volume_snapshot or {}), "symbol": sym}, now=now)
        mark = classify_ofx_volume_support(direction, result)

        # Ghost reconcile / same-direction refresh / opposite close (crypto parity)
        existing = self._opens.get(sym)
        if existing and existing.get("ofx_trade_id"):
            if str(existing["ofx_trade_id"]) in self._known_ids:
                self._opens.pop(sym, None)
                existing = None
                self._persist_opens()
        if existing and existing.get("active"):
            if str(existing.get("direction")) == direction:
                existing["ofx_volume_imbalance"] = mark.get("ofx_volume_imbalance")
                existing["signed_delta_qty"] = mark.get("signed_delta_qty")
                existing["last_eval_ms"] = now
                existing["ofx_entry_surge"] = surge
                _sync_match_band(existing)
                self._persist_opens()
                return existing
            # Opposite open → try match-exit first (same as crypto evaluate_tick path)
            self.evaluate_open_tick(
                symbol=sym,
                volume_snapshot=volume_snapshot,
                quote=quote,
                now_ms=now,
            )
            existing = self._opens.get(sym)
            if existing and existing.get("active"):
                return existing

        bar_idx = eval_record.get("bar_idx")
        if bar_idx is not None and self._last_entry_bar_by_symbol.get(sym) == int(bar_idx):
            return None

        entry = _entry_px(direction, quote, eval_record.get("reference_price"))
        if entry is None or entry <= 0:
            return None

        size = size_nse_of_paper_from_entry_signal(entry_price=entry, quote=quote)
        qty = float(size.get("simulated_quantity") or 0.0)
        notional = float(size.get("simulated_notional_inr") or 0.0)
        if qty <= 0 or notional <= 0:
            qty = max(1.0, DEFAULT_FALLBACK_NOTIONAL_INR / float(entry))
            notional = qty * float(entry)

        bid, ask = _bid_ask(quote)
        tid = f"OFX-NSE-VOL-PAPER-{sym}-{direction}-{bar_idx}-{now}"
        if tid in self._known_ids:
            return None

        qty_stamp = ofx_entry_qty_for_tempo(
            surge_entry_volume=float(surge.get("entry_volume") or 0.0),
            entry_side_tempo=entry_tempo,
        )
        entry_agg_qty = float(qty_stamp.get("entry_aggressor_qty") or 0.0)
        if entry_agg_qty <= 0:
            entry_agg_qty = float(
                entry_aggressor_qty_for_side(
                    direction,
                    buy_aggressor_qty=float(mark.get("buy_aggressor_qty") or 0.0),
                    sell_aggressor_qty=float(mark.get("sell_aggressor_qty") or 0.0),
                )
                or 0.0
            )
        if entry_agg_qty <= 0:
            return None

        decision_label = (
            "RECOMMEND_LONG"
            if direction == "LONG"
            else "RECOMMEND_SHORT"
        )
        opp_agg = opposite_aggressor_label(direction)
        rec = {
            "active": True,
            "symbol": sym,
            "match_symbol": sym,
            "tradingsymbol": str(sym).replace(".NS", ""),
            "exchange": "NSE",
            "direction": direction,
            "ofx_trade_id": tid,
            "protocol_id": PROTOCOL_ID,
            "construct": CONSTRUCT,
            "side_classification_mode": "DERIVED_VALIDATED",
            "baseline_of_inert": True,
            "execution_mode": "PAPER",
            "live_order_authority": 0.0,
            "ofx_volume_label": OFX_VOLUME_LABEL,
            "ofx_volume_supported": bool(mark.get("ofx_volume_supported")),
            "ofx_volume_imbalance": mark.get("ofx_volume_imbalance"),
            "signed_delta_qty": mark.get("signed_delta_qty"),
            "buy_aggressor_qty": mark.get("buy_aggressor_qty"),
            "sell_aggressor_qty": mark.get("sell_aggressor_qty"),
            "entry_aggressor_qty": float(entry_agg_qty),
            "entry_aggressor_side": surge.get("entry_aggressor_side")
            or ("BUY" if direction == "LONG" else "SELL"),
            "entry_qty_source": qty_stamp.get("entry_qty_source"),
            "entry_side_tempo": entry_tempo,
            "opposite_side_tempo_at_entry": opp_tempo,
            "ofx_tempo_entry_gate": tempo_gate,
            "exit_match_aggressor_side": opp_agg,
            "ofx_entry_surge": surge,
            "ofx_entry_surge_multiple": OFX_ENTRY_SURGE_MULTIPLE,
            "ofx_entry_surge_lookback_ticks": OFX_ENTRY_SURGE_LOOKBACK_TICKS,
            "ofx_volume_imbalance_threshold": OFX_VOLUME_IMBALANCE_THRESHOLD,
            "ofx_match_dev_min": OFX_VOLUME_MATCH_DEV_MIN,
            "ofx_match_dev_max": OFX_VOLUME_MATCH_DEV_MAX,
            "ofx_match_scope": "PER_SYMBOL_ONLY",
            "decision": decision_label,
            "bar_idx": bar_idx,
            "session_idx": eval_record.get("session_idx"),
            "entry_price": entry,
            "entry_bid": bid,
            "entry_ask": ask,
            "simulated_notional_inr": round(notional, 4),
            "simulated_quantity": qty,
            "entry_paper_quantity": qty,
            "activated_ms": now,
            "entry_time_ms": now,
            "paper_open_timestamp": _iso(now),
            "opposite_exit_seen_ms": None,
            "exit_rule": (
                f"per_symbol_opposing_volume_match_pm{int(OFX_VOLUME_MATCH_DEV_MAX*100)}pct"
                f"|clubbed_opposing_completion"
                f"+min_hold_{self.min_hold_ms}ms+persist_{self.opposite_persist_ms}ms"
            ),
            "note": (
                "OFX NSE volume PAPER — entry: 3×/20 surge + tempo gate; staggered entry qty "
                "uses same-side lookback sum; exit: ±10% match OR clubbed opposing completion; "
                "baseline L1 OF inert; DERIVED_VALIDATED construct."
            ),
        }
        self._opens[sym] = rec
        self._persist_opens()
        self._append_jsonl(self.opens_path, dict(rec))
        if bar_idx is not None:
            self._last_entry_bar_by_symbol[sym] = int(bar_idx)
            self._write_meta()
        self._rebuild_summary()
        return rec

    def evaluate_open_tick(
        self,
        *,
        symbol: str,
        volume_snapshot: Dict[str, Any],
        quote: Optional[Any] = None,
        now_ms: Optional[int] = None,
        force_reason: Optional[str] = None,
    ) -> Optional[Dict[str, Any]]:
        now = int(now_ms if now_ms is not None else time.time() * 1000)
        sym = canonicalize_nse_symbol(symbol)
        if not sym:
            return None
        # Resolve open under canonical key (and one legacy raw-key fallback)
        prev = self._opens.get(sym)
        if prev is None and str(symbol) in self._opens:
            legacy = self._opens.pop(str(symbol))
            legacy["symbol"] = sym
            legacy["match_symbol"] = sym
            self._opens[sym] = legacy
            prev = legacy
        if not prev or not prev.get("active"):
            return None

        # Always restamp live crypto OFX ±10% band (legacy ±6% / 2–8% opens)
        _sync_match_band(prev)

        if force_reason:
            return self._close(
                symbol=sym,
                reason=str(force_reason),
                exit_meta={},
                quote=quote,
                now_ms=now,
                volume_snapshot=volume_snapshot,
            )

        # Refuse to evaluate against another symbol's volume snapshot
        snap_sym = canonicalize_nse_symbol(
            (volume_snapshot or {}).get("symbol") or symbol
        )
        if snap_sym and snap_sym != sym:
            prev["close_suppressed_reason"] = f"symbol_mismatch_snapshot_{snap_sym}_vs_{sym}"
            self._persist_opens()
            return prev

        direction = str(prev.get("direction") or "LONG").upper()
        try:
            entry_agg = float(prev.get("entry_aggressor_qty") or 0.0)
        except (TypeError, ValueError):
            entry_agg = 0.0
        if entry_agg <= 0:
            entry_agg = float(
                entry_aggressor_qty_for_side(
                    direction,
                    buy_aggressor_qty=float(prev.get("buy_aggressor_qty") or 0.0),
                    sell_aggressor_qty=float(prev.get("sell_aggressor_qty") or 0.0),
                )
                or 0.0
            )
            if entry_agg > 0:
                prev["entry_aggressor_qty"] = entry_agg
                prev["entry_aggressor_side"] = "BUY" if direction == "LONG" else "SELL"
                prev["exit_match_aggressor_side"] = opposite_aggressor_label(direction)

        activated = int(prev.get("activated_ms") or now)
        # Path telemetry + early-adverse abort (parity with crypto OFX paper)
        try:
            entry_px = float(prev.get("entry_price") or 0.0)
        except (TypeError, ValueError):
            entry_px = 0.0
        mid = _mid_px(quote)
        if entry_px > 0 and mid is not None and mid > 0:
            sb = _signed_bps(direction, entry_px, mid)
            mfe = float(prev.get("path_mfe_bps") or 0.0)
            mae = float(prev.get("path_mae_bps") or 0.0)
            early_mae = float(prev.get("early_mae_bps") or 0.0)
            if sb > mfe:
                mfe = sb
            if sb < mae:
                mae = sb
            age = now - activated
            if age <= OFX_EARLY_ADVERSE_WINDOW_MS and sb < early_mae:
                early_mae = sb
            prev["path_mfe_bps"] = round(mfe, 4)
            prev["path_mae_bps"] = round(mae, 4)
            prev["early_mae_bps"] = round(early_mae, 4)
            if (
                age >= OFX_EARLY_ADVERSE_WINDOW_MS
                and not prev.get("early_adverse_checked")
                and early_mae <= -float(OFX_EARLY_ADVERSE_BPS)
                and mfe < 10.0
            ):
                prev["early_adverse_checked"] = True
                return self._close(
                    symbol=sym,
                    reason=OFX_EARLY_ADVERSE_EXIT_REASON,
                    exit_meta={
                        "early_mae_bps": round(early_mae, 4),
                        "path_mfe_bps": round(mfe, 4),
                        "path_mae_bps": round(mae, 4),
                        "early_window_ms": OFX_EARLY_ADVERSE_WINDOW_MS,
                        "early_adverse_bps": OFX_EARLY_ADVERSE_BPS,
                    },
                    quote=quote,
                    now_ms=now,
                    volume_snapshot=volume_snapshot or {},
                )
            if age >= OFX_EARLY_ADVERSE_WINDOW_MS:
                prev["early_adverse_checked"] = True

        match_sym = canonicalize_nse_symbol(prev.get("match_symbol") or prev.get("symbol") or sym)
        cum_opp = self._sum_opposite_aggressor_since_entry(
            symbol=match_sym, direction=direction, since_ms=activated, now_ms=now
        )
        prev["opposite_aggressor_qty_since_entry"] = round(cum_opp, 8)
        prev["opposite_match_symbol"] = match_sym
        # Same as crypto: ±10% match OR capped clubbed opposing completion
        if direction == "LONG":
            window_opp = float((volume_snapshot or {}).get("sell_aggressor_qty") or 0.0)
            opp_side = "SELL"
        else:
            window_opp = float((volume_snapshot or {}).get("buy_aggressor_qty") or 0.0)
            opp_side = "BUY"
        prev["opposite_aggressor_qty_window"] = round(window_opp, 8)
        exit_sig = is_ofx_volume_match_exit(
            direction,
            entry_aggressor_qty=entry_agg,
            opposite_aggressor_qty_since_entry=window_opp,
            match_dev_min=OFX_VOLUME_MATCH_DEV_MIN,
            match_dev_max=OFX_VOLUME_MATCH_DEV_MAX,
        )
        exit_sig["match_qty_source"] = "window_opposing_aggressor"
        exit_reason = EXIT_REASON_MATCH
        if not exit_sig.get("exit") and cum_opp > 0:
            cum_sig = is_ofx_volume_match_exit(
                direction,
                entry_aggressor_qty=entry_agg,
                opposite_aggressor_qty_since_entry=cum_opp,
                match_dev_min=OFX_VOLUME_MATCH_DEV_MIN,
                match_dev_max=OFX_VOLUME_MATCH_DEV_MAX,
            )
            if cum_sig.get("exit"):
                exit_sig = cum_sig
                exit_sig["match_qty_source"] = "cumulative_since_entry_in_band"
        if not exit_sig.get("exit"):
            trades = self._recent_executed_trades(symbol=match_sym)
            opp_tempo = classify_ofx_aggressor_tempo(
                trades, aggressor_side=opp_side, lookback=OFX_TEMPO_LOOKBACK_TICKS, as_of_ms=now
            )
            prev["opposite_side_tempo_live"] = opp_tempo
            for opp_qty, src in ((window_opp, "window_clubbed"), (cum_opp, "cumulative_clubbed")):
                club = is_ofx_clubbed_opposing_completion(
                    entry_aggressor_qty=entry_agg,
                    opposite_aggressor_qty=opp_qty,
                    opposite_tempo=str(opp_tempo.get("tempo") or ""),
                    match_dev_max=OFX_VOLUME_MATCH_DEV_MAX,
                    max_overshoot=OFX_CLUBBED_MAX_OVERSHOOT,
                )
                if club.get("exit"):
                    exit_sig = {
                        **exit_sig,
                        **club,
                        "exit": True,
                        "opposite_side": "SHORT" if direction == "LONG" else "LONG",
                        "opposite_aggressor": opp_side,
                        "match_qty_source": src,
                        "match_overshoot": True,
                        "match_in_tolerance_band": False,
                    }
                    exit_reason = EXIT_REASON_CLUBBED
                    break
        prev["ofx_match_deviation"] = exit_sig.get("match_deviation")
        prev["ofx_match_reason"] = exit_sig.get("reason")
        prev["ofx_match_qty_source"] = exit_sig.get("match_qty_source")

        if not exit_sig.get("exit"):
            if prev.get("opposite_exit_seen_ms") is not None:
                prev["opposite_exit_seen_ms"] = None
            self._persist_opens()
            return prev

        hold_needed = int(self.min_hold_ms)
        if exit_reason == EXIT_REASON_CLUBBED:
            hold_needed = max(hold_needed, int(OFX_CLUBBED_MIN_HOLD_MS))
        if (now - activated) < hold_needed:
            prev["close_suppressed_reason"] = (
                f"min_hold_{(now - activated)}ms<{hold_needed}ms"
                + ("_clubbed" if exit_reason == EXIT_REASON_CLUBBED else "")
            )
            self._persist_opens()
            return prev

        seen = prev.get("opposite_exit_seen_ms")
        if seen is None:
            prev["opposite_exit_seen_ms"] = now
            self._persist_opens()
            return prev
        if (now - int(seen)) < self.opposite_persist_ms:
            prev["close_suppressed_reason"] = (
                f"opposite_persist_{(now - int(seen))}ms<{self.opposite_persist_ms}ms"
            )
            self._persist_opens()
            return prev

        return self._close(
            symbol=sym,
            reason=exit_reason,
            exit_meta={
                "opposite_side": exit_sig.get("opposite_side"),
                "opposite_aggressor": exit_sig.get("opposite_aggressor"),
                "entry_aggressor_qty": exit_sig.get("entry_aggressor_qty"),
                "opposite_aggressor_qty_since_entry": prev.get(
                    "opposite_aggressor_qty_since_entry"
                ),
                "opposite_aggressor_qty_matched": exit_sig.get("opposite_aggressor_qty"),
                "completion_floor_qty": exit_sig.get("completion_floor_qty"),
                "opposite_tempo": prev.get("opposite_side_tempo_live"),
                "match_qty_source": exit_sig.get("match_qty_source"),
                "match_deviation": exit_sig.get("match_deviation"),
                "match_in_tolerance_band": exit_sig.get("match_in_tolerance_band"),
                "match_overshoot": exit_sig.get("match_overshoot"),
                "ofx_volume_exit_label": exit_sig.get("ofx_volume_exit_label"),
                "match_symbol": match_sym,
                "ofx_match_scope": "PER_SYMBOL_ONLY",
            },
            quote=quote,
            now_ms=now,
            volume_snapshot=volume_snapshot,
        )

    def _recent_executed_trades(self, *, symbol: str, since_ms: Optional[int] = None):
        sym = canonicalize_nse_symbol(symbol)
        if not sym:
            return []
        buf = global_nse_volume_flow_buffer()
        rows = buf.rows_for(sym, since_ms=since_ms)
        return trades_from_nse_inferred_ticks(
            rows,
            min_confidence="MEDIUM",
            require_trade_like=True,
            instrument_ids={sym},
        )

    def _entry_surge_gate(
        self, *, symbol: str, direction: str, now_ms: Optional[int] = None
    ) -> Dict[str, Any]:
        trades = self._recent_executed_trades(symbol=symbol)
        return ofx_entry_surge_from_trades(
            trades,
            direction=direction,
            lookback=OFX_ENTRY_SURGE_LOOKBACK_TICKS,
            multiple=OFX_ENTRY_SURGE_MULTIPLE,
            as_of_ms=now_ms,
        )

    def _sum_opposite_aggressor_since_entry(
        self, *, symbol: str, direction: str, since_ms: int, now_ms: int
    ) -> float:
        """Cumulative opposing aggressor qty for ONE symbol only (never pooled)."""
        opp = opposite_aggressor_label(direction)
        sym = canonicalize_nse_symbol(symbol)
        if not opp or not sym:
            return 0.0
        trades = self._recent_executed_trades(symbol=sym, since_ms=int(since_ms))
        total = 0.0
        for t in trades:
            if canonicalize_nse_symbol(t.instrument_id) != sym:
                continue
            if t.timestamp_ms > int(now_ms):
                continue
            if str(t.side.aggressor).upper() != opp:
                continue
            total += float(t.quantity or 0.0)
        return total

    def square_off_all(
        self,
        *,
        quotes_by_symbol: Dict[str, Any],
        volume_by_symbol: Optional[Dict[str, Dict[str, Any]]] = None,
        now_ms: Optional[int] = None,
        reason: str = "SESSION_SQUARE_OFF",
    ) -> List[Dict[str, Any]]:
        now = int(now_ms if now_ms is not None else time.time() * 1000)
        closed: List[Dict[str, Any]] = []
        for sym in list(self._opens.keys()):
            snap = (volume_by_symbol or {}).get(sym) or {
                "symbol": sym,
                "buy_aggressor_qty": 0,
                "sell_aggressor_qty": 0,
                "total_classified_qty": 0,
                "signed_delta_qty": 0,
                "data_quality": "UNAVAILABLE",
                "status": "UNAVAILABLE",
            }
            out = self._close(
                symbol=sym,
                reason=reason,
                exit_meta={},
                quote=(quotes_by_symbol or {}).get(sym),
                now_ms=now,
                volume_snapshot=snap,
            )
            if out:
                closed.append(out)
        return closed

    def _close(
        self,
        *,
        symbol: str,
        reason: str,
        exit_meta: Dict[str, Any],
        quote: Optional[Any],
        now_ms: int,
        volume_snapshot: Dict[str, Any],
    ) -> Optional[Dict[str, Any]]:
        prev = self._opens.get(symbol)
        if not prev:
            return None
        tid = str(prev.get("ofx_trade_id") or "")
        if tid and tid in self._known_ids:
            self._opens.pop(symbol, None)
            self._persist_opens()
            return None

        direction = str(prev.get("direction") or "LONG")
        entry = float(prev["entry_price"])
        exit_price = _exit_px(direction, quote, entry)
        if exit_price is None:
            exit_price = entry
        qty = float(prev.get("entry_paper_quantity") or prev.get("simulated_quantity") or 0.0)
        bid, ask = _bid_ask(quote)
        acct = calc_nse_of_paper_accounting(
            entry_price=entry,
            exit_price=float(exit_price),
            quantity=qty,
            direction=direction,
        )
        closed = dict(prev)
        closed.update(
            {
                "active": False,
                "exit_reason": reason,
                "exited_ms": now_ms,
                "paper_close_ms": now_ms,
                "paper_close_timestamp": _iso(now_ms),
                "exit_price": float(exit_price),
                "exit_bid": bid,
                "exit_ask": ask,
                "exit_opposite_side": exit_meta.get("opposite_side"),
                "exit_opposite_aggressor": exit_meta.get("opposite_aggressor"),
                "exit_entry_aggressor_qty": exit_meta.get("entry_aggressor_qty"),
                "exit_opposite_aggressor_qty_since_entry": exit_meta.get(
                    "opposite_aggressor_qty_since_entry"
                ),
                "exit_opposite_aggressor_qty_matched": exit_meta.get(
                    "opposite_aggressor_qty_matched"
                ),
                "exit_match_qty_source": exit_meta.get("match_qty_source"),
                "exit_match_deviation": exit_meta.get("match_deviation"),
                "exit_match_in_tolerance_band": exit_meta.get("match_in_tolerance_band"),
                "exit_match_overshoot": exit_meta.get("match_overshoot"),
                "ofx_volume_exit_label": exit_meta.get("ofx_volume_exit_label"),
                "exit_match_symbol": exit_meta.get("match_symbol") or closed.get("match_symbol"),
                "ofx_match_scope": exit_meta.get("ofx_match_scope") or "PER_SYMBOL_ONLY",
                "ofx_match_dev_min": OFX_VOLUME_MATCH_DEV_MIN,
                "ofx_match_dev_max": OFX_VOLUME_MATCH_DEV_MAX,
                "exit_signed_delta_qty": volume_snapshot.get("signed_delta_qty"),
                "qty_matched_entry_to_exit": True,
                "exit_quantity": qty,
                "gross_pnl_inr": round(float(acct.get("touch_gross_pnl_inr") or 0.0), 4),
                "net_pnl_inr": round(float(acct.get("all_in_net_pnl_inr") or 0.0), 4),
                "friction_inr": round(float(acct.get("statutory_overhead_inr") or 0.0), 4),
                "accounting": acct,
                "quote_book": quote_as_book_dict(quote),
            }
        )

        self._append_jsonl(self.outcomes_path, closed)
        if tid:
            self._known_ids.add(tid)
        self._opens.pop(symbol, None)
        self._persist_opens()
        self._rebuild_summary()
        return closed
