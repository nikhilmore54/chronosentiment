#!/usr/bin/env python3
"""
RP-608 NSE time-of-day observation snapshot (read-only).

Intraday entry universe remains the FULL eligible session (Gate 1 window).
The 11:00 IST split is a DIAGNOSTIC contrast only — it does NOT filter,
suppress, or limit entries to >=11:00 IST.

Does NOT modify strategy, daemon config, or live order authority.

Usage (on Pi or with NSE_DIR pointing at shadow journals):
  python3 scripts/rp608_nse_tod_observation_snapshot.py
  python3 scripts/rp608_nse_tod_observation_snapshot.py --session-date 2026-10-29
"""

from __future__ import annotations

import argparse
import collections
import json
from datetime import datetime, timezone, timedelta
from pathlib import Path
from typing import Any, Dict, List, Optional

IST = timezone(timedelta(hours=5, minutes=30))
WORKSPACE = Path(__file__).resolve().parent.parent
DEFAULT_NSE_DIR = WORKSPACE / "live_capture" / "shadow_validation" / "nse"
DEFAULT_OUT = (
    WORKSPACE
    / "artifacts"
    / "rp608_regime_breakout"
    / "prospective_v1"
    / "journals"
    / "NSE_tod_observation_log.jsonl"
)
CUTOFF_HOUR_IST = 11
PROTOCOL = "RP-608-NSE-SHADOW-001"
OBSERVATION_ID = "RP608_NSE_TOD_OBSERVE_V1"


def ms_ist(ms: Optional[int]) -> Optional[datetime]:
    if not ms:
        return None
    return datetime.fromtimestamp(ms / 1000.0, tz=timezone.utc).astimezone(IST)


def load_closed(nse_dir: Path) -> List[Dict[str, Any]]:
    path = nse_dir / "rp608_nse_trade_journal.jsonl"
    rows: List[Dict[str, Any]] = []
    if not path.exists():
        return rows
    with path.open(encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                rows.append(json.loads(line))
            except json.JSONDecodeError:
                continue
    return rows


def load_active(nse_dir: Path) -> List[Dict[str, Any]]:
    path = nse_dir / "rp608_nse_pending_evaluations.json"
    if not path.exists():
        return []
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return []
    return list((payload.get("active_trades") or {}).values())


def load_checkpoint(nse_dir: Path) -> Dict[str, Any]:
    path = nse_dir / "rp608_nse_checkpoint.json"
    if not path.exists():
        return {}
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return {}


def entry_hour_bucket(trade: Dict[str, Any]) -> Optional[str]:
    ts = ms_ist(trade.get("entry_timestamp_utc_ms"))
    if ts is None:
        return None
    return "ge_11ist" if ts.hour >= CUTOFF_HOUR_IST else "lt_11ist"


def summarize(trades: List[Dict[str, Any]]) -> Dict[str, Any]:
    n = len(trades)
    if n == 0:
        return {
            "n_closed": 0,
            "wins": 0,
            "win_rate": None,
            "net_pnl_inr": 0.0,
            "gross_pnl_inr": 0.0,
            "avg_net_pnl_inr": None,
            "avg_mfe_bps": None,
            "avg_mae_bps": None,
            "mfe0_count": 0,
            "exit_reasons": {},
            "exit_net_pnl_inr": {},
            "by_entry_hour": [],
        }
    wins = sum(1 for t in trades if t.get("win_label") == 1)
    nets = [float(t.get("net_pnl_inr") or 0.0) for t in trades]
    gross = [float(t.get("gross_pnl_inr") or 0.0) for t in trades]
    mfe = [float(t.get("mfe_bps") or 0.0) for t in trades]
    mae = [float(t.get("mae_bps") or 0.0) for t in trades]
    reasons = collections.Counter(t.get("exit_reason") for t in trades)
    by_r: Dict[str, float] = collections.defaultdict(float)
    for t in trades:
        by_r[str(t.get("exit_reason") or "?")] += float(t.get("net_pnl_inr") or 0.0)
    by_hour: Dict[str, List[Dict[str, Any]]] = collections.defaultdict(list)
    for t in trades:
        ts = ms_ist(t.get("entry_timestamp_utc_ms"))
        if ts is None:
            continue
        by_hour[ts.strftime("%H:00")].append(t)
    hour_rows = []
    for hour, rows in sorted(by_hour.items()):
        hn = [float(t.get("net_pnl_inr") or 0.0) for t in rows]
        hw = sum(1 for t in rows if t.get("win_label") == 1)
        hour_rows.append(
            {
                "hour_ist": hour,
                "n": len(rows),
                "net_pnl_inr": round(sum(hn), 2),
                "win_rate": round(hw / len(rows), 4),
            }
        )
    return {
        "n_closed": n,
        "wins": wins,
        "win_rate": round(wins / n, 4),
        "net_pnl_inr": round(sum(nets), 2),
        "gross_pnl_inr": round(sum(gross), 2),
        "avg_net_pnl_inr": round(sum(nets) / n, 2),
        "avg_mfe_bps": round(sum(mfe) / n, 2),
        "avg_mae_bps": round(sum(mae) / n, 2),
        "mfe0_count": sum(1 for x in mfe if x == 0.0),
        "exit_reasons": dict(reasons),
        "exit_net_pnl_inr": {k: round(v, 2) for k, v in by_r.items()},
        "by_entry_hour": hour_rows,
    }


def session_date_for_trade(trade: Dict[str, Any], fallback: Optional[str]) -> Optional[str]:
    # Prefer explicit session fields; else IST calendar date of entry.
    for key in ("session_date", "current_session_date"):
        if trade.get(key):
            return str(trade.get(key))
    ts = ms_ist(trade.get("entry_timestamp_utc_ms"))
    if ts is not None:
        return ts.strftime("%Y-%m-%d")
    return fallback


def build_snapshot(
    nse_dir: Path,
    session_date: Optional[str] = None,
) -> Dict[str, Any]:
    cp = load_checkpoint(nse_dir)
    closed = load_closed(nse_dir)
    active = load_active(nse_dir)
    fallback_date = session_date or cp.get("current_session_date")
    if not fallback_date:
        fallback_date = datetime.now(IST).strftime("%Y-%m-%d")

    # Filter closed to the requested session calendar date when possible.
    day_closed: List[Dict[str, Any]] = []
    for t in closed:
        sd = session_date_for_trade(t, None)
        # session_idx alone is ambiguous across days; prefer date when present.
        if sd == fallback_date:
            day_closed.append(t)
        elif sd is None and session_date is None and t.get("session_idx") == cp.get("session_idx"):
            day_closed.append(t)

    if not day_closed and session_date is None:
        # Last resort: use latest session_idx in journal matching checkpoint.
        sid = cp.get("session_idx")
        day_closed = [t for t in closed if t.get("session_idx") == sid]

    ge = [t for t in day_closed if entry_hour_bucket(t) == "ge_11ist"]
    lt = [t for t in day_closed if entry_hour_bucket(t) == "lt_11ist"]
    open_ge = [t for t in active if entry_hour_bucket(t) == "ge_11ist"]
    open_lt = [t for t in active if entry_hour_bucket(t) == "lt_11ist"]

    full_s = summarize(day_closed)
    full_s["n_open"] = len(active)
    ge_s = summarize(ge)
    lt_s = summarize(lt)
    ge_s["n_open"] = len(open_ge)
    lt_s["n_open"] = len(open_lt)

    return {
        "observation_id": OBSERVATION_ID,
        "protocol_id": PROTOCOL,
        "frozen_strategy": True,
        "live_order_authority": 0.0,
        # Hard policy: never restrict intraday entries to post-11.
        "entry_policy": "FULL_SESSION_UNRESTRICTED",
        "tod_split_is_diagnostic_only": True,
        "entry_filter_applied": False,
        "cutoff_hour_ist": CUTOFF_HOUR_IST,
        "hypothesis": (
            "Diagnostic only: compare pre-11 vs >=11 IST cohorts inside the full "
            "session book. Intraday entries remain allowed across the entire Gate-1 "
            "window; do not conclude or enforce a post-11-only policy from this split."
        ),
        "decision_status": "OBSERVE_ONLY_NO_CONCLUSION",
        "session_date": fallback_date,
        "session_idx": cp.get("session_idx"),
        "last_durable_bar_idx": cp.get("last_durable_bar_idx"),
        "heartbeat_iso": cp.get("last_heartbeat_iso"),
        "snapshot_utc": datetime.now(timezone.utc).isoformat(),
        "n_closed_session": len(day_closed),
        "n_open_session": len(active),
        "full_session": full_s,
        "lt_11ist": lt_s,
        "ge_11ist": ge_s,
        "contrast": {
            "net_ge11_minus_lt11": round(ge_s["net_pnl_inr"] - lt_s["net_pnl_inr"], 2),
            "wr_ge11": ge_s["win_rate"],
            "wr_lt11": lt_s["win_rate"],
            "note": "Contrast buckets only; entry universe is full_session",
        },
        "source_dir": str(nse_dir),
    }


def upsert_log(out_path: Path, snap: Dict[str, Any]) -> None:
    out_path.parent.mkdir(parents=True, exist_ok=True)
    existing: List[Dict[str, Any]] = []
    if out_path.exists():
        with out_path.open(encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if not line:
                    continue
                try:
                    existing.append(json.loads(line))
                except json.JSONDecodeError:
                    continue
    # One durable row per session_date (latest snapshot wins).
    kept = [r for r in existing if r.get("session_date") != snap.get("session_date")]
    kept.append(snap)
    kept.sort(key=lambda r: str(r.get("session_date") or ""))
    with out_path.open("w", encoding="utf-8") as f:
        for row in kept:
            f.write(json.dumps(row, default=str) + "\n")


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--nse-dir", type=Path, default=Path(
        __import__("os").environ.get("NSE_DIR", str(DEFAULT_NSE_DIR))
    ))
    ap.add_argument("--out", type=Path, default=Path(
        __import__("os").environ.get("TOD_OBS_OUT", str(DEFAULT_OUT))
    ))
    ap.add_argument("--session-date", type=str, default=None)
    ap.add_argument("--print-only", action="store_true")
    args = ap.parse_args()

    snap = build_snapshot(args.nse_dir, session_date=args.session_date)
    if not args.print_only:
        upsert_log(args.out, snap)
    print(json.dumps({
        "wrote": None if args.print_only else str(args.out),
        "session_date": snap["session_date"],
        "entry_policy": snap.get("entry_policy"),
        "tod_split_is_diagnostic_only": snap.get("tod_split_is_diagnostic_only"),
        "n_closed_session": snap["n_closed_session"],
        "full_session_net": snap["full_session"]["net_pnl_inr"],
        "full_session_wr": snap["full_session"]["win_rate"],
        "lt_11ist_net": snap["lt_11ist"]["net_pnl_inr"],
        "ge_11ist_net": snap["ge_11ist"]["net_pnl_inr"],
        "lt_11ist_wr": snap["lt_11ist"]["win_rate"],
        "ge_11ist_wr": snap["ge_11ist"]["win_rate"],
        "contrast_net": snap["contrast"]["net_ge11_minus_lt11"],
        "decision_status": snap["decision_status"],
    }, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
