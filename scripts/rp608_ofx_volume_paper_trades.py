#!/usr/bin/env python3
"""
RP-608-OFX — Volume/aggressor-direction PAPER trade book (research-only).

Parallel to baseline L1 OF paper. Does NOT mutate OF_SUP / ±0.30 / reverse-OF
ledger under trade_plan_overlay_v1. Authority 0%.

Entry: Policy B LONG/SHORT + OFX entry tick volume ≥ 3× mean of prior 20
      completed ticks (raw sizes only; no L1 OF / other order-flow signals).
Exit: opposing aggressor volume matches entry aggressor volume within −10%..+10%
      (buy closed by sell qty ≈ entry buy qty; sell closed by buy qty ≈ entry sell qty),
      or capped clubbed opposing completion after 15m min-hold; early-adverse abort
      after 60s if first-minute MAE ≤ −8 bps and MFE never ≥ +10 bps.
Entry tempo: MIXED suppressed (path analysis 2026-11-15).
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

from research.rp608_ofx.executed_flow import trades_from_aggtrade_records  # noqa: E402
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
from scripts.rp608_ofx_crypto_volume_flow_mark import (  # noqa: E402
    _load_recent_journal_rows,
    _rows_to_aggtrade_shaped,
)
from scripts.rp608_trade_plan_overlay import compute_of_paper_cost_breakdown  # noqa: E402

PROTOCOL_ID = "RP-608-OFX-VOLUME-PAPER-001"
DEFAULT_NOTIONAL = 100.0
MIN_HOLD_MS = 300_000
OPPOSITE_PERSIST_MS = 30_000
# Paper safety: tiny surge stamps can leave opposing qty forever past ±10%/1.5× band.
OFX_MAX_HOLD_MS = 6 * 60 * 60 * 1000  # 6h
FEE_BPS_PER_LEG = 6.0
EXIT_REASON_MATCH = "OPPOSING_VOLUME_MATCH"
EXIT_REASON_CLUBBED = OFX_CLUBBED_OPPOSING_EXIT_REASON
EXIT_REASON_OVERSHOOT = "OPPOSING_VOLUME_OVERSHOOT_EXHAUSTED"
EXIT_REASON_MAX_HOLD = "RECONCILED_MAX_HOLD"


def _iso(ms: int) -> str:
    return datetime.fromtimestamp(ms / 1000.0, tz=timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def _bid_ask(quote: Optional[Dict[str, Any]]) -> Tuple[Optional[float], Optional[float]]:
    q = quote if isinstance(quote, dict) else {}
    try:
        bid = float(q["best_bid_price"]) if q.get("best_bid_price") is not None else None
    except (TypeError, ValueError, KeyError):
        bid = None
    try:
        ask = float(q["best_ask_price"]) if q.get("best_ask_price") is not None else None
    except (TypeError, ValueError, KeyError):
        ask = None
    return bid, ask


def _mid_px(quote: Optional[Dict[str, Any]]) -> Optional[float]:
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


def _entry_px(direction: str, quote: Optional[Dict[str, Any]], fallback: Optional[float]) -> Optional[float]:
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


def _exit_px(direction: str, quote: Optional[Dict[str, Any]], fallback: Optional[float]) -> Optional[float]:
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


class OfxVolumePaperManager:
    """Durable OFX volume paper trades under ofx_aggtrade_capture_v1/."""

    def __init__(self, root: Path, *, notional_usdt: float = DEFAULT_NOTIONAL):
        self.root = Path(root)
        self.root.mkdir(parents=True, exist_ok=True)
        self.outcomes_path = self.root / "ofx_volume_paper_outcomes.jsonl"
        self.opens_path = self.root / "ofx_volume_paper_opens.jsonl"
        self.open_state_path = self.root / "ofx_volume_paper_open.json"
        self.summary_path = self.root / "ofx_volume_paper_summary.json"
        self.notional = float(notional_usdt)
        self.min_hold_ms = MIN_HOLD_MS
        self.opposite_persist_ms = OPPOSITE_PERSIST_MS
        self.active: Optional[Dict[str, Any]] = self._read_open()
        self._meta_path = self.root / "ofx_volume_paper_meta.json"
        self.last_entry_bar_idx: Optional[int] = self._read_last_bar()
        self._known_ids: Set[str] = set()
        self._load_known_ids()
        self._rebuild_summary()

    def _read_last_bar(self) -> Optional[int]:
        if not self._meta_path.exists():
            return None
        try:
            data = json.loads(self._meta_path.read_text(encoding="utf-8"))
            b = data.get("last_entry_bar_idx")
            return int(b) if b is not None else None
        except Exception:
            return None

    def _write_last_bar(self, bar_idx: Optional[int]) -> None:
        payload = {"last_entry_bar_idx": bar_idx, "protocol_id": PROTOCOL_ID}
        tmp = self._meta_path.with_suffix(".tmp")
        with open(tmp, "w", encoding="utf-8") as f:
            json.dump(payload, f)
            f.flush()
            os.fsync(f.fileno())
        os.replace(tmp, self._meta_path)

    def _load_known_ids(self) -> None:
        if not self.outcomes_path.exists():
            return
        for line in self.outcomes_path.open(encoding="utf-8"):
            if not line.strip():
                continue
            try:
                row = json.loads(line)
            except json.JSONDecodeError:
                continue
            tid = row.get("ofx_trade_id")
            if tid:
                self._known_ids.add(str(tid))

    def _read_open(self) -> Optional[Dict[str, Any]]:
        if not self.open_state_path.exists():
            return None
        try:
            data = json.loads(self.open_state_path.read_text(encoding="utf-8"))
        except Exception:
            return None
        if isinstance(data, dict) and data.get("open") and isinstance(data.get("trade"), dict):
            return data["trade"]
        return None

    def _write_open(self, rec: Optional[Dict[str, Any]]) -> None:
        tmp = self.open_state_path.with_name(
            f"{self.open_state_path.stem}.{os.getpid()}.{time.time_ns()}.tmp"
        )
        if rec is None:
            payload = {
                "open": False,
                "ofx_trade_id": None,
                "protocol_id": PROTOCOL_ID,
                "execution_mode": "PAPER",
                "live_order_authority": 0.0,
                "construct": "EXECUTED_AGGRESSOR_VOLUME",
                "baseline_of_inert": True,
            }
        else:
            payload = {
                "open": True,
                "protocol_id": PROTOCOL_ID,
                "execution_mode": "PAPER",
                "live_order_authority": 0.0,
                "construct": "EXECUTED_AGGRESSOR_VOLUME",
                "baseline_of_inert": True,
                "ofx_trade_id": rec.get("ofx_trade_id"),
                "trade": rec,
            }
        with open(tmp, "w", encoding="utf-8") as f:
            json.dump(payload, f, indent=2)
            f.flush()
            os.fsync(f.fileno())
        os.replace(tmp, self.open_state_path)

    def _append_jsonl(self, path: Path, row: Dict[str, Any]) -> None:
        with path.open("a", encoding="utf-8") as f:
            f.write(json.dumps(row, separators=(",", ":")) + "\n")
            f.flush()

    def _rebuild_summary(self) -> None:
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
                        pnl = float(row.get("net_pnl_usdt") or 0.0)
                    except (TypeError, ValueError):
                        pnl = 0.0
                    if pnl > 0:
                        wins += 1
                    else:
                        losses += 1
                    net += pnl
                    try:
                        gross += float(row.get("gross_pnl_usdt") or 0.0)
                    except (TypeError, ValueError):
                        pass
                    try:
                        friction += float(row.get("friction_usdt") or 0.0)
                    except (TypeError, ValueError):
                        pass
        summary = {
            "protocol_id": PROTOCOL_ID,
            "construct": "EXECUTED_AGGRESSOR_VOLUME",
            "baseline_of_inert": True,
            "live_order_authority": 0.0,
            "n_trades": n,
            "n_wins": wins,
            "n_losses": losses,
            "win_rate_pct": round(100.0 * wins / n, 2) if n else None,
            "cumulative_net_pnl_usdt": round(net, 4),
            "cumulative_gross_pnl_usdt": round(gross, 4),
            "cumulative_friction_usdt": round(friction, 4),
            "updated_utc": datetime.now(timezone.utc).isoformat(),
            "open": bool(self.active and self.active.get("active")),
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

    def load_recent(self, n: int = 24) -> List[Dict[str, Any]]:
        rows: List[Dict[str, Any]] = []
        if not self.outcomes_path.exists():
            return rows
        for line in self.outcomes_path.open(encoding="utf-8"):
            if not line.strip():
                continue
            try:
                rows.append(json.loads(line))
            except json.JSONDecodeError:
                continue
        return rows[-n:][::-1]

    def try_open_from_decision(
        self,
        *,
        direction: str,
        volume_snapshot: Dict[str, Any],
        quote: Optional[Dict[str, Any]] = None,
        plan_id: Optional[str] = None,
        bar_idx: Optional[int] = None,
        decision_event_idx: Optional[int] = None,
        entry_price: Optional[float] = None,
        now_ms: Optional[int] = None,
    ) -> Optional[Dict[str, Any]]:
        """Open OFX volume paper when Policy B side + 3×/20-tick surge pass."""
        d = str(direction or "").upper()
        if d not in ("LONG", "SHORT"):
            return self.active
        now = int(now_ms if now_ms is not None else time.time() * 1000)

        # Ghost reconcile
        if self.active and self.active.get("ofx_trade_id"):
            if str(self.active["ofx_trade_id"]) in self._known_ids:
                self.active = None
                self._write_open(None)

        # Entry gate: raw tick surge only (no L1 OF / other order-flow signals)
        symbol = str((volume_snapshot or {}).get("symbol") or "BTCUSDT")
        surge = self._entry_surge_gate(direction=d, symbol=symbol, now_ms=now)
        if not surge.get("passed"):
            return self.active

        trades = self._recent_executed_trades(symbol=symbol)
        entry_side = "BUY" if d == "LONG" else "SELL"
        opp_side = "SELL" if d == "LONG" else "BUY"
        entry_tempo = classify_ofx_aggressor_tempo(
            trades, aggressor_side=entry_side, lookback=OFX_TEMPO_LOOKBACK_TICKS, as_of_ms=now
        )
        opp_tempo = classify_ofx_aggressor_tempo(
            trades, aggressor_side=opp_side, lookback=OFX_TEMPO_LOOKBACK_TICKS, as_of_ms=now
        )
        tempo_gate = should_suppress_ofx_entry_for_tempo(
            direction=d, entry_side_tempo=entry_tempo, opposite_side_tempo=opp_tempo
        )
        if tempo_gate.get("suppress"):
            return None

        # Optional telemetry from snapshot imbalance (never gates entry)
        from research.rp608_ofx.executed_flow.aggregate import ExecutedFlowWindowResult

        result = ExecutedFlowWindowResult(
            source=str((volume_snapshot or {}).get("journal_path") or "ofx_volume_snapshot"),
            side_classification_mode="DIRECT",
            instrument_id=symbol,
            window_start_ms=int((volume_snapshot or {}).get("window_start_ms") or 0),
            window_end_ms=int((volume_snapshot or {}).get("window_end_ms") or now),
            window_ms=int((volume_snapshot or {}).get("window_ms") or 0),
            buy_aggressor_qty=float((volume_snapshot or {}).get("buy_aggressor_qty") or 0.0),
            sell_aggressor_qty=float((volume_snapshot or {}).get("sell_aggressor_qty") or 0.0),
            total_classified_qty=float((volume_snapshot or {}).get("total_classified_qty") or 0.0),
            signed_delta_qty=float((volume_snapshot or {}).get("signed_delta_qty") or 0.0),
            classification_coverage=(volume_snapshot or {}).get("classification_coverage"),
            n_trades_in_window=int(((volume_snapshot or {}).get("journal_rows_loaded") or 0)),
            n_classified=1 if float((volume_snapshot or {}).get("total_classified_qty") or 0) > 0 else 0,
            n_unclassified=0,
            n_invalid=0,
            n_duplicates_skipped=0,
            n_out_of_order=0,
            data_quality=str((volume_snapshot or {}).get("data_quality") or "UNAVAILABLE"),
            status=str((volume_snapshot or {}).get("status") or "UNAVAILABLE"),
        )
        mark = classify_ofx_volume_support(d, result)

        if self.active and self.active.get("active"):
            if str(self.active.get("direction")) == d:
                self.active["ofx_volume_imbalance"] = mark.get("ofx_volume_imbalance")
                self.active["signed_delta_qty"] = mark.get("signed_delta_qty")
                self.active["last_eval_ms"] = now
                self.active["ofx_entry_surge"] = surge
                self._write_open(self.active)
                return self.active
            # opposite open → close first via evaluate
            self.evaluate_tick(volume_snapshot=volume_snapshot, quote=quote, now_ms=now)

        if self.active and self.active.get("active"):
            return self.active

        # Anti-churn: at most one OFX volume paper open per 15m bar
        if bar_idx is not None and self.last_entry_bar_idx is not None:
            if int(bar_idx) == int(self.last_entry_bar_idx):
                return None

        entry = _entry_px(d, quote, entry_price)
        if entry is None or entry <= 0:
            return None
        qty = self.notional / float(entry)
        bid, ask = _bid_ask(quote)
        tid = f"OFX-VOL-PAPER-{(plan_id or 'NOPLAN')}-{now}"
        if tid in self._known_ids:
            return None
        # Entry aggressor qty: staggered → same-side lookback sum; else surge tick
        qty_stamp = ofx_entry_qty_for_tempo(
            surge_entry_volume=float(surge.get("entry_volume") or 0.0),
            entry_side_tempo=entry_tempo,
        )
        entry_agg_qty = float(qty_stamp.get("entry_aggressor_qty") or 0.0)
        if entry_agg_qty <= 0:
            entry_agg_qty = float(
                entry_aggressor_qty_for_side(
                    d,
                    buy_aggressor_qty=float(mark.get("buy_aggressor_qty") or 0.0),
                    sell_aggressor_qty=float(mark.get("sell_aggressor_qty") or 0.0),
                )
                or 0.0
            )
        if entry_agg_qty <= 0:
            return None
        opp_agg = opposite_aggressor_label(d)
        rec = {
            "active": True,
            "symbol": "BTCUSDT",
            "direction": d,
            "ofx_trade_id": tid,
            "protocol_id": PROTOCOL_ID,
            "construct": "EXECUTED_AGGRESSOR_VOLUME",
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
            or ("BUY" if d == "LONG" else "SELL"),
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
            "plan_id": plan_id,
            "bar_idx": bar_idx,
            "decision_event_idx": decision_event_idx,
            "entry_price": entry,
            "entry_bid": bid,
            "entry_ask": ask,
            "simulated_notional_usdt": round(self.notional, 4),
            "simulated_quantity": qty,
            "entry_paper_quantity": qty,
            "activated_ms": now,
            "entry_time_ms": now,
            "paper_open_timestamp": _iso(now),
            "opposite_exit_seen_ms": None,
            "exit_rule": (
                f"opposing_volume_match_pm{int(OFX_VOLUME_MATCH_DEV_MAX*100)}pct"
                f"|clubbed_opposing_completion"
                f"+min_hold_{self.min_hold_ms}ms+persist_{self.opposite_persist_ms}ms"
            ),
            "note": (
                "OFX volume PAPER — entry: 3×/20 surge + tempo gate; staggered entry qty "
                "uses same-side lookback sum; exit: ±10% match OR clubbed opposing completion."
            ),
        }
        self.active = rec
        self._write_open(rec)
        self._append_jsonl(self.opens_path, dict(rec))
        if bar_idx is not None:
            self.last_entry_bar_idx = int(bar_idx)
            self._write_last_bar(self.last_entry_bar_idx)
        return rec

    def _recent_executed_trades(self, *, symbol: str = "BTCUSDT"):
        journal = self.root / "aggtrades.jsonl"
        rows = _load_recent_journal_rows(journal, max_rows=5_000)
        return trades_from_aggtrade_records(
            _rows_to_aggtrade_shaped(rows, symbol), default_symbol=symbol
        )

    def _entry_surge_gate(
        self, *, direction: str, symbol: str = "BTCUSDT", now_ms: Optional[int] = None
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
        self, *, direction: str, since_ms: int, now_ms: int, symbol: str = "BTCUSDT"
    ) -> float:
        """Cumulative opposite aggressor qty from durable aggTrade journal since entry."""
        opp = opposite_aggressor_label(direction)
        if not opp:
            return 0.0
        trades = self._recent_executed_trades(symbol=symbol)
        total = 0.0
        for t in trades:
            if t.timestamp_ms < int(since_ms) or t.timestamp_ms > int(now_ms):
                continue
            if str(t.side.aggressor).upper() != opp:
                continue
            total += float(t.quantity or 0.0)
        return total

    def evaluate_tick(
        self,
        *,
        volume_snapshot: Dict[str, Any],
        quote: Optional[Dict[str, Any]] = None,
        now_ms: Optional[int] = None,
    ) -> Optional[Dict[str, Any]]:
        """Exit on ±10% opposing match, or capped clubbed opposing completion."""
        now = int(now_ms if now_ms is not None else time.time() * 1000)
        if not self.active or not self.active.get("active"):
            return None

        # Restamp live OFX ±10% band + capped clubbed completion exit
        self.active["ofx_match_dev_min"] = OFX_VOLUME_MATCH_DEV_MIN
        self.active["ofx_match_dev_max"] = OFX_VOLUME_MATCH_DEV_MAX
        self.active["ofx_clubbed_max_overshoot"] = OFX_CLUBBED_MAX_OVERSHOOT
        self.active["ofx_clubbed_min_hold_ms"] = OFX_CLUBBED_MIN_HOLD_MS
        self.active["exit_rule"] = (
            f"opposing_volume_match_pm{int(OFX_VOLUME_MATCH_DEV_MAX * 100)}pct"
            f"|clubbed_opposing_completion_cap{int(OFX_CLUBBED_MAX_OVERSHOOT * 100)}pct"
            f"+clubbed_min_hold_{OFX_CLUBBED_MIN_HOLD_MS}ms"
            f"+min_hold_{self.min_hold_ms}ms+persist_{self.opposite_persist_ms}ms"
            f"|early_adverse_{int(OFX_EARLY_ADVERSE_BPS)}bps_{OFX_EARLY_ADVERSE_WINDOW_MS}ms"
        )

        direction = str(self.active.get("direction") or "")
        try:
            entry_agg = float(self.active.get("entry_aggressor_qty") or 0.0)
        except (TypeError, ValueError):
            entry_agg = 0.0
        if entry_agg <= 0:
            # Legacy open without stamp — derive from stored buy/sell at entry
            entry_agg = float(
                entry_aggressor_qty_for_side(
                    direction,
                    buy_aggressor_qty=float(self.active.get("buy_aggressor_qty") or 0.0),
                    sell_aggressor_qty=float(self.active.get("sell_aggressor_qty") or 0.0),
                )
                or 0.0
            )
            if entry_agg > 0:
                self.active["entry_aggressor_qty"] = entry_agg
                self.active["entry_aggressor_side"] = "BUY" if direction == "LONG" else "SELL"
                self.active["exit_match_aggressor_side"] = opposite_aggressor_label(direction)

        activated = int(self.active.get("activated_ms") or now)
        # Path telemetry + early-adverse abort (bad-entry correction from path analysis)
        try:
            entry_px = float(self.active.get("entry_price") or 0.0)
        except (TypeError, ValueError):
            entry_px = 0.0
        mid = _mid_px(quote)
        if entry_px > 0 and mid is not None and mid > 0:
            sb = _signed_bps(direction, entry_px, mid)
            mfe = float(self.active.get("path_mfe_bps") or 0.0)
            mae = float(self.active.get("path_mae_bps") or 0.0)
            early_mae = float(self.active.get("early_mae_bps") or 0.0)
            if sb > mfe:
                mfe = sb
            if sb < mae:
                mae = sb
            age = now - activated
            if age <= OFX_EARLY_ADVERSE_WINDOW_MS and sb < early_mae:
                early_mae = sb
            self.active["path_mfe_bps"] = round(mfe, 4)
            self.active["path_mae_bps"] = round(mae, 4)
            self.active["early_mae_bps"] = round(early_mae, 4)
            if (
                age >= OFX_EARLY_ADVERSE_WINDOW_MS
                and not self.active.get("early_adverse_checked")
                and early_mae <= -float(OFX_EARLY_ADVERSE_BPS)
                and mfe < 10.0
            ):
                self.active["early_adverse_checked"] = True
                return self._close(
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
                    volume_snapshot=volume_snapshot,
                )
            if age >= OFX_EARLY_ADVERSE_WINDOW_MS:
                self.active["early_adverse_checked"] = True

        # Absolute paper max-hold — unblocks geometric exit traps (tiny entry stamp).
        age_ms = now - activated
        if age_ms >= int(OFX_MAX_HOLD_MS):
            return self._close(
                reason=EXIT_REASON_MAX_HOLD,
                exit_meta={
                    "max_hold_ms": int(OFX_MAX_HOLD_MS),
                    "held_ms": age_ms,
                    "reconcile": True,
                    "note": "PAPER max-hold reconcile — opposing match band missed",
                },
                quote=quote,
                now_ms=now,
                volume_snapshot=volume_snapshot or {},
            )

        symbol = str(self.active.get("symbol") or volume_snapshot.get("symbol") or "BTCUSDT")
        cum_opp = self._sum_opposite_aggressor_since_entry(
            direction=direction,
            since_ms=activated,
            now_ms=now,
            symbol=symbol,
        )
        self.active["opposite_aggressor_qty_since_entry"] = round(cum_opp, 8)
        # Strict exit match: live window opposing qty vs entry aggressor qty (−10%..+10%)
        if direction == "LONG":
            window_opp = float(volume_snapshot.get("sell_aggressor_qty") or 0.0)
            opp_side = "SELL"
        else:
            window_opp = float(volume_snapshot.get("buy_aggressor_qty") or 0.0)
            opp_side = "BUY"
        self.active["opposite_aggressor_qty_window"] = round(window_opp, 8)
        exit_sig = is_ofx_volume_match_exit(
            direction,
            entry_aggressor_qty=entry_agg,
            opposite_aggressor_qty_since_entry=window_opp,
            match_dev_min=OFX_VOLUME_MATCH_DEV_MIN,
            match_dev_max=OFX_VOLUME_MATCH_DEV_MAX,
        )
        exit_sig["match_qty_source"] = "window_opposing_aggressor"
        exit_reason = EXIT_REASON_MATCH
        # Also accept cumulative only if it itself sits inside the ±10% band
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
        # Clubbed opposing completion: dump/lift overshoots ±band but fills entry floor
        if not exit_sig.get("exit"):
            trades = self._recent_executed_trades(symbol=symbol)
            opp_tempo = classify_ofx_aggressor_tempo(
                trades, aggressor_side=opp_side, lookback=OFX_TEMPO_LOOKBACK_TICKS, as_of_ms=now
            )
            self.active["opposite_side_tempo_live"] = opp_tempo
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
        # Past clubbed cap with no match possible — opposing flow already exhausted entry.
        if (
            not exit_sig.get("exit")
            and entry_agg > 0
            and cum_opp >= entry_agg * (1.0 + float(OFX_CLUBBED_MAX_OVERSHOOT))
            and age_ms >= int(self.min_hold_ms)
        ):
            exit_sig = {
                "exit": True,
                "opposite_side": "SHORT" if direction == "LONG" else "LONG",
                "opposite_aggressor": opp_side,
                "entry_aggressor_qty": entry_agg,
                "opposite_aggressor_qty": cum_opp,
                "match_qty_source": "cumulative_overshoot_exhausted",
                "match_overshoot": True,
                "match_in_tolerance_band": False,
                "match_deviation": (cum_opp / entry_agg) - 1.0 if entry_agg else None,
            }
            exit_reason = EXIT_REASON_OVERSHOOT
        self.active["ofx_match_deviation"] = exit_sig.get("match_deviation")
        self.active["ofx_match_reason"] = exit_sig.get("reason")
        self.active["ofx_match_qty_source"] = exit_sig.get("match_qty_source")

        if not exit_sig.get("exit"):
            if self.active.get("opposite_exit_seen_ms") is not None:
                self.active["opposite_exit_seen_ms"] = None
            self._write_open(self.active)
            return self.active

        hold_needed = int(self.min_hold_ms)
        if exit_reason == EXIT_REASON_CLUBBED:
            hold_needed = max(hold_needed, int(OFX_CLUBBED_MIN_HOLD_MS))
        if (now - activated) < hold_needed:
            self.active["close_suppressed_reason"] = (
                f"min_hold_{(now - activated)}ms<{hold_needed}ms"
                + ("_clubbed" if exit_reason == EXIT_REASON_CLUBBED else "")
            )
            self._write_open(self.active)
            return self.active

        # Overshoot/max-hold class exits: no 30s persist (already past completion geometry).
        if exit_reason in (EXIT_REASON_OVERSHOOT, EXIT_REASON_MAX_HOLD):
            return self._close(
                reason=exit_reason,
                exit_meta={
                    "opposite_side": exit_sig.get("opposite_side"),
                    "opposite_aggressor": exit_sig.get("opposite_aggressor"),
                    "entry_aggressor_qty": exit_sig.get("entry_aggressor_qty") or entry_agg,
                    "opposite_aggressor_qty_since_entry": self.active.get(
                        "opposite_aggressor_qty_since_entry"
                    ),
                    "opposite_aggressor_qty_matched": exit_sig.get("opposite_aggressor_qty"),
                    "match_qty_source": exit_sig.get("match_qty_source"),
                    "match_deviation": exit_sig.get("match_deviation"),
                    "match_overshoot": True,
                    "reconcile": True,
                },
                quote=quote,
                now_ms=now,
                volume_snapshot=volume_snapshot or {},
            )

        seen = self.active.get("opposite_exit_seen_ms")
        if seen is None:
            self.active["opposite_exit_seen_ms"] = now
            self._write_open(self.active)
            return self.active
        if (now - int(seen)) < self.opposite_persist_ms:
            self.active["close_suppressed_reason"] = (
                f"opposite_persist_{(now - int(seen))}ms<{self.opposite_persist_ms}ms"
            )
            self._write_open(self.active)
            return self.active

        return self._close(
            reason=exit_reason,
            exit_meta={
                "opposite_side": exit_sig.get("opposite_side"),
                "opposite_aggressor": exit_sig.get("opposite_aggressor"),
                "entry_aggressor_qty": exit_sig.get("entry_aggressor_qty"),
                "opposite_aggressor_qty_since_entry": self.active.get(
                    "opposite_aggressor_qty_since_entry"
                )
                if self.active
                else None,
                "opposite_aggressor_qty_matched": exit_sig.get("opposite_aggressor_qty"),
                "match_qty_source": exit_sig.get("match_qty_source"),
                "match_deviation": exit_sig.get("match_deviation"),
                "match_in_tolerance_band": exit_sig.get("match_in_tolerance_band"),
                "match_overshoot": exit_sig.get("match_overshoot"),
                "ofx_volume_exit_label": exit_sig.get("ofx_volume_exit_label"),
                "opposite_tempo": (self.active or {}).get("opposite_side_tempo_live"),
                "completion_floor_qty": exit_sig.get("completion_floor_qty"),
            },
            quote=quote,
            now_ms=now,
            volume_snapshot=volume_snapshot,
        )

    def force_close_active(
        self,
        *,
        reason: str = EXIT_REASON_MAX_HOLD,
        quote: Optional[Dict[str, Any]] = None,
        now_ms: Optional[int] = None,
        note: Optional[str] = None,
    ) -> Optional[Dict[str, Any]]:
        """Operator/paper reconcile: close active OFX volume trade into outcomes."""
        if not self.active or not self.active.get("active"):
            return None
        now = int(now_ms if now_ms is not None else time.time() * 1000)
        activated = int(self.active.get("activated_ms") or now)
        return self._close(
            reason=reason,
            exit_meta={
                "reconcile": True,
                "held_ms": now - activated,
                "max_hold_ms": int(OFX_MAX_HOLD_MS),
                "note": note
                or "force_close_active — paper reconcile of stuck OFX volume open",
            },
            quote=quote,
            now_ms=now,
            volume_snapshot={},
        )

    def _close(
        self,
        *,
        reason: str,
        exit_meta: Dict[str, Any],
        quote: Optional[Dict[str, Any]],
        now_ms: int,
        volume_snapshot: Dict[str, Any],
    ) -> Optional[Dict[str, Any]]:
        prev = self.active
        if not prev:
            return None
        tid = str(prev.get("ofx_trade_id") or "")
        if tid and tid in self._known_ids:
            self.active = None
            self._write_open(None)
            return None

        direction = str(prev.get("direction") or "")
        entry = float(prev["entry_price"])
        exit_price = _exit_px(direction, quote, entry)
        if exit_price is None:
            exit_price = entry
        qty = float(prev.get("entry_paper_quantity") or prev.get("simulated_quantity") or 0.0)
        bid, ask = _bid_ask(quote)
        pnl = compute_of_paper_cost_breakdown(
            direction=direction,
            entry_price=entry,
            exit_price=float(exit_price),
            notional=float(prev.get("simulated_notional_usdt") or self.notional),
            entry_bid=prev.get("entry_bid"),
            entry_ask=prev.get("entry_ask"),
            exit_bid=bid,
            exit_ask=ask,
            fee_bps_per_leg=FEE_BPS_PER_LEG,
            quantity=qty,
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
                "exit_signed_delta_qty": volume_snapshot.get("signed_delta_qty"),
                "qty_matched_entry_to_exit": True,
                "exit_quantity": qty,
                **{k: pnl.get(k) for k in (
                    "gross_pnl_usdt",
                    "net_pnl_usdt",
                    "friction_usdt",
                    "fees_usdt",
                    "spread_cost_usdt",
                    "slippage_usdt",
                    "net_return_bps",
                    "simulated_quantity",
                    "simulated_notional_usdt",
                )},
            }
        )
        self._append_jsonl(self.outcomes_path, closed)
        if tid:
            self._known_ids.add(tid)
        self.active = None
        self._write_open(None)
        self._rebuild_summary()
        return closed
