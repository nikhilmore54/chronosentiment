#!/usr/bin/env python3
"""
RP-608 Crypto recommendation-plan — past-trade pattern library miner.

Book: crypto trade-plan overlay (BTCUSDT plan) only — never pooled with NSE/OF/OFX.

Two uses:
  1) ENTRY (Gate 2E): decision-time OF/ATR/session/direction + candle vol_surge →
     ALLOW / STANDASIDE_ADVERSE / ABSTAIN from historical W/L.
  2) EXIT  (Gate 2X observational): MFE + reverse-after-green + exit_vol_surge →
     HOLD / EXIT_REVERSE stamps (frozen TP/SL/60m unless Gate 5).

Volume = 15m OHLCV burst vs SMA20 (not OFX aggression). No symbol in keys.
Authority 0%. OF ±0.30 unchanged; OFX inert here.
"""

from __future__ import annotations

import argparse
import json
from collections import Counter, defaultdict
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional, Sequence

WORKSPACE = Path(__file__).resolve().parent.parent

PROTOCOL_ID = "RP-608-CRYPTO-PLAN-TRADE-PATTERN-V1"
LIBRARY_SCHEMA = "crypto_plan_trade_pattern_library_v2"
ADVERSE_MIN_N = 30
ADVERSE_MAX_WR = 0.35
EXIT_REVERSE_MIN_N = 20
HOLDOUT_SESSIONS = 2

ENTRY_BACKOFF_DROP_ORDER = (
    "vol_surge",
    "gross_rr_bucket",
    "target_dist_bucket",
    "stop_dist_bucket",
    "imb_bucket",
    "atr_bucket",
    "of_supported",
    "prior_stop_same_utc_day_dir",
)


def utc_session_phase(ts_ms: int) -> str:
    """Crypto 24h phases (UTC)."""
    h = datetime.fromtimestamp(int(ts_ms) / 1000.0, tz=timezone.utc).hour
    if 0 <= h < 8:
        return "ASIA"
    if 8 <= h < 13:
        return "EU"
    if 13 <= h < 21:
        return "US"
    return "LATE_UTC"


def session_date_utc(ts_ms: int) -> str:
    return datetime.fromtimestamp(int(ts_ms) / 1000.0, tz=timezone.utc).date().isoformat()


def imb_bucket(imb: Optional[float]) -> str:
    if imb is None:
        return "UNK"
    x = float(imb)
    if x <= -0.30:
        return "LE_M30"
    if x < 0.0:
        return "M30_0"
    if x < 0.30:
        return "0_30"
    return "GE_30"


def atr_pct_bucket(atr: Optional[float], entry: Optional[float]) -> str:
    if atr is None or entry is None or float(entry) <= 0:
        return "UNK"
    pct = float(atr) / float(entry) * 100.0
    if pct < 0.3:
        return "ATR_LT_0_3"
    if pct < 0.6:
        return "ATR_0_3_0_6"
    if pct < 1.0:
        return "ATR_0_6_1_0"
    return "ATR_GE_1_0"


def dist_bps_bucket(entry: float, level: Optional[float]) -> str:
    if level is None or entry <= 0:
        return "UNK"
    bps = abs(float(level) - entry) / entry * 10000.0
    if bps < 40:
        return "LT_40"
    if bps < 80:
        return "40_80"
    if bps < 150:
        return "80_150"
    return "GE_150"


def rr_bucket(rr: Optional[float]) -> str:
    if rr is None:
        return "UNK"
    x = float(rr)
    if x < 1.0:
        return "RR_LT_1"
    if x < 1.5:
        return "RR_1_1_5"
    if x < 2.0:
        return "RR_1_5_2"
    return "RR_GE_2"


def mfe_bps_from_plan(plan: Dict[str, Any]) -> Optional[float]:
    entry = float(plan.get("entry_price") or 0)
    mfe_px = plan.get("mfe_exec_price")
    if entry <= 0 or mfe_px is None:
        return None
    direction = str(plan.get("direction") or "LONG").upper()
    mfe = float(mfe_px)
    if direction == "SHORT":
        return max(0.0, (entry - mfe) / entry * 10000.0)
    return max(0.0, (mfe - entry) / entry * 10000.0)


def mfe_bucket(mfe_bps: Optional[float]) -> str:
    if mfe_bps is None:
        return "UNK"
    x = float(mfe_bps)
    if x >= 80:
        return "MFE_GE80"
    if x >= 40:
        return "MFE_40_80"
    if x >= 15:
        return "MFE_15_40"
    return "MFE_LT15"


def hold_bucket(seconds: Optional[float]) -> str:
    if seconds is None:
        return "UNK"
    s = float(seconds)
    if s < 600:
        return "H_LT_10M"
    if s < 1800:
        return "H_10_30M"
    if s < 3600:
        return "H_30_60M"
    return "H_GE_60M"


def vol_surge_bucket(ratio: Optional[float]) -> str:
    """Candle volume vs prior SMA — same buckets as NSE A (not OFX aggression)."""
    if ratio is None:
        return "UNK"
    r = float(ratio)
    if r < 0.8:
        return "LOW"
    if r < 1.5:
        return "MED"
    return "HIGH"


def bool_token(v: Any) -> str:
    return "T" if bool(v) else "F"


def build_entry_pattern_key(feats: Dict[str, Any]) -> str:
    parts = [
        str(feats.get("utc_session_phase") or "UNK"),
        str(feats.get("direction") or "UNK"),
        "OF" + bool_token(feats.get("of_supported")),
        str(feats.get("imb_bucket") or "UNK"),
        str(feats.get("atr_bucket") or "UNK"),
        "S" + str(feats.get("stop_dist_bucket") or "UNK"),
        "T" + str(feats.get("target_dist_bucket") or "UNK"),
        "R" + str(feats.get("gross_rr_bucket") or "UNK"),
        "P" + bool_token(feats.get("prior_stop_same_utc_day_dir")),
        "V" + str(feats.get("vol_surge") or "UNK"),
    ]
    key = "|".join(parts)
    assert "symbol" not in key.lower()
    assert "btcusdt" not in key.lower()
    return key


def build_exit_pattern_key(feats: Dict[str, Any]) -> str:
    parts = [
        str(feats.get("utc_session_phase") or "UNK"),
        str(feats.get("direction") or "UNK"),
        str(feats.get("mfe_bucket") or "UNK"),
        str(feats.get("first_reverse") or "UNK"),
        str(feats.get("hold_bucket") or "UNK"),
        "V" + str(feats.get("exit_vol_surge") or "UNK"),
    ]
    return "|".join(parts)


def entry_key_softened(feats: Dict[str, Any], dropped: Sequence[str]) -> str:
    f = dict(feats)
    for ax in dropped:
        if ax in {
            "gross_rr_bucket",
            "target_dist_bucket",
            "stop_dist_bucket",
            "imb_bucket",
            "atr_bucket",
            "vol_surge",
        }:
            f[ax] = "UNK"
        elif ax == "of_supported":
            f[ax] = False
        elif ax == "prior_stop_same_utc_day_dir":
            f[ax] = False
    return build_entry_pattern_key(f)


def load_jsonl(path: Path) -> List[Dict[str, Any]]:
    rows: List[Dict[str, Any]] = []
    if not path.exists():
        return rows
    with path.open("r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                rows.append(json.loads(line))
            except json.JSONDecodeError:
                continue
    return rows


def plan_win(plan: Dict[str, Any]) -> int:
    if "win" in plan:
        return 1 if int(plan["win"]) == 1 else 0
    net = plan.get("net_pnl_usdt")
    if net is None:
        net = plan.get("net_pnl")
    return 1 if float(net or 0) > 0 else 0


def enrich_plan(
    plan: Dict[str, Any],
    *,
    prior_stop: bool,
) -> Dict[str, Any]:
    et = int(plan.get("entry_time_ms") or 0)
    entry = float(plan.get("entry_price") or 0)
    direction = str(plan.get("direction") or "UNK").upper()
    of_sup = bool(plan.get("order_flow_supported"))
    imb = plan.get("order_flow_imbalance")
    atr = plan.get("atr_14")
    sl = plan.get("stop_loss_price")
    tp = plan.get("profit_target_price")
    rr = plan.get("gross_rr_ratio")
    mfe_bps = mfe_bps_from_plan(plan)
    reason = str(plan.get("exit_reason") or plan.get("state") or "")
    if mfe_bps is not None and mfe_bps >= 15 and "STOP" in reason:
        rev = "REV_AFTER_GREEN"
    elif "PROFIT" in reason or "TARGET" in reason:
        rev = "NONE"
    elif "STOP" in reason:
        rev = "REV_NO_MFE"
    elif "TIME" in reason or "EXPIRED" in reason:
        rev = "TIME"
    else:
        rev = "UNK"
    entry_vol = plan.get("vol_surge")
    if entry_vol not in {"LOW", "MED", "HIGH", "UNK"}:
        raw = plan.get("vol_burst_20")
        entry_vol = vol_surge_bucket(float(raw) if raw is not None else None)
    exit_vol = plan.get("exit_vol_surge")
    if exit_vol not in {"LOW", "MED", "HIGH", "UNK"}:
        raw_x = plan.get("exit_vol_burst_20")
        if raw_x is not None:
            exit_vol = vol_surge_bucket(float(raw_x))
        else:
            exit_vol = entry_vol if entry_vol else "UNK"
    feats = {
        "utc_session_phase": utc_session_phase(et) if et else "UNK",
        "direction": direction,
        "of_supported": of_sup,
        "imb_bucket": imb_bucket(float(imb) if imb is not None else None),
        "atr_bucket": atr_pct_bucket(
            float(atr) if atr is not None else None,
            entry if entry > 0 else None,
        ),
        "stop_dist_bucket": dist_bps_bucket(entry, float(sl) if sl is not None else None),
        "target_dist_bucket": dist_bps_bucket(entry, float(tp) if tp is not None else None),
        "gross_rr_bucket": rr_bucket(float(rr) if rr is not None else None),
        "prior_stop_same_utc_day_dir": bool(prior_stop),
        "vol_surge": str(entry_vol or "UNK"),
        "mfe_bucket": mfe_bucket(mfe_bps),
        "first_reverse": rev,
        "hold_bucket": hold_bucket(
            float(plan["holding_duration_seconds"])
            if plan.get("holding_duration_seconds") is not None
            else None
        ),
        "exit_vol_surge": str(exit_vol or "UNK"),
        "win": plan_win(plan),
        "net_pnl_usdt": float(plan.get("net_pnl_usdt") or plan.get("net_pnl") or 0.0),
        "gross_pnl_usdt": float(plan.get("gross_pnl_usdt") or 0.0),
        "session_date": session_date_utc(et) if et else None,
        "_plan_id": str(plan.get("plan_id") or ""),
        "_symbol": str(plan.get("symbol") or ""),
    }
    feats["entry_pattern_id"] = build_entry_pattern_key(feats)
    feats["exit_pattern_id"] = build_exit_pattern_key(feats)
    return feats


def _agg_metrics(rows: List[Dict[str, Any]]) -> Dict[str, Any]:
    n = len(rows)
    if n == 0:
        return {
            "n": 0,
            "wins": 0,
            "losses": 0,
            "wr": None,
            "avg_win": None,
            "avg_loss": None,
            "payoff": None,
            "net_sum": 0.0,
            "net_mean": None,
        }
    wins = [r for r in rows if int(r.get("win") or 0) == 1]
    losses = [r for r in rows if int(r.get("win") or 0) != 1]
    avg_win = (
        sum(float(r["net_pnl_usdt"]) for r in wins) / len(wins) if wins else None
    )
    avg_loss = (
        sum(abs(float(r["net_pnl_usdt"])) for r in losses) / len(losses) if losses else None
    )
    payoff = None
    if avg_win is not None and avg_loss is not None and avg_loss > 0:
        payoff = avg_win / avg_loss
    net_sum = sum(float(r["net_pnl_usdt"]) for r in rows)
    return {
        "n": n,
        "wins": len(wins),
        "losses": len(losses),
        "wr": round(len(wins) / n, 4),
        "avg_win": round(avg_win, 4) if avg_win is not None else None,
        "avg_loss": round(avg_loss, 4) if avg_loss is not None else None,
        "payoff": round(payoff, 3) if payoff is not None else None,
        "net_sum": round(net_sum, 4),
        "net_mean": round(net_sum / n, 4),
    }


def classify_entry_label(m: Dict[str, Any]) -> str:
    n = int(m.get("n") or 0)
    wr = m.get("wr")
    if n < ADVERSE_MIN_N or wr is None:
        return "INSUFFICIENT"
    if wr <= ADVERSE_MAX_WR:
        return "ADVERSE"
    return "NEUTRAL"


def classify_exit_label(m: Dict[str, Any], rows: List[Dict[str, Any]]) -> str:
    n = int(m.get("n") or 0)
    if n < EXIT_REVERSE_MIN_N:
        return "INSUFFICIENT"
    rev_rows = [r for r in rows if str(r.get("first_reverse") or "NONE") not in {"NONE", "UNK"}]
    if len(rev_rows) < max(10, EXIT_REVERSE_MIN_N // 2) and str(
        rows[0].get("first_reverse") if rows else ""
    ) in {"NONE", "UNK", "TIME"}:
        # cohort itself may be reverse-typed via key
        pass
    wr = m.get("wr")
    if wr is not None and wr <= 0.40:
        return "REVERSE_STANDASIDE_HOLD"
    if wr is not None and wr >= 0.55:
        return "HOLD_FAVORABLE"
    return "NEUTRAL"


def mine_library(plans: List[Dict[str, Any]]) -> Dict[str, Any]:
    # Dedupe by plan_id keep last
    by_id: Dict[str, Dict[str, Any]] = {}
    for p in plans:
        pid = str(p.get("plan_id") or "")
        if not pid:
            continue
        if not p.get("entry_time_ms"):
            continue
        by_id[pid] = p
    closed = sorted(by_id.values(), key=lambda r: int(r["entry_time_ms"]))

    sessions: List[str] = []
    seen_s = set()
    for p in closed:
        d = session_date_utc(int(p["entry_time_ms"]))
        if d not in seen_s:
            seen_s.add(d)
            sessions.append(d)
    holdout_set = set(sessions[-HOLDOUT_SESSIONS:]) if len(sessions) > HOLDOUT_SESSIONS else set()
    train_sessions = [s for s in sessions if s not in holdout_set]

    enriched: List[Dict[str, Any]] = []
    day_dir_stops: Dict[str, set] = defaultdict(set)

    for p in closed:
        sess = session_date_utc(int(p["entry_time_ms"]))
        direction = str(p.get("direction") or "UNK").upper()
        family = f"{direction}"
        prior = family in day_dir_stops[sess]
        feats = enrich_plan(p, prior_stop=prior)
        feats["_split"] = "holdout" if sess in holdout_set else "train"
        enriched.append(feats)
        reason = str(p.get("exit_reason") or "")
        if "STOP" in reason:
            day_dir_stops[sess].add(family)

    train_rows = [r for r in enriched if r["_split"] == "train"]
    holdout_rows = [r for r in enriched if r["_split"] == "holdout"]

    def _group(rows: List[Dict[str, Any]], key_field: str) -> Dict[str, List[Dict[str, Any]]]:
        g: Dict[str, List[Dict[str, Any]]] = defaultdict(list)
        for r in rows:
            g[str(r[key_field])].append(r)
        return g

    entry_train = _group(train_rows, "entry_pattern_id")
    exit_train = _group(train_rows, "exit_pattern_id")
    entry_hold = _group(holdout_rows, "entry_pattern_id")
    exit_hold = _group(holdout_rows, "exit_pattern_id")

    entry_patterns = {}
    for pid, rows in entry_train.items():
        m = _agg_metrics(rows)
        entry_patterns[pid] = {
            "pattern_id": pid,
            "kind": "ENTRY",
            "label": classify_entry_label(m),
            "train": m,
            "holdout": _agg_metrics(entry_hold.get(pid, [])),
            "coverage_train_share": round(len(rows) / max(len(train_rows), 1), 4),
        }

    exit_patterns = {}
    for pid, rows in exit_train.items():
        m = _agg_metrics(rows)
        exit_patterns[pid] = {
            "pattern_id": pid,
            "kind": "EXIT",
            "label": classify_exit_label(m, rows),
            "train": m,
            "holdout": _agg_metrics(exit_hold.get(pid, [])),
            "coverage_train_share": round(len(rows) / max(len(train_rows), 1), 4),
        }

    return {
        "protocol_id": PROTOCOL_ID,
        "schema": LIBRARY_SCHEMA,
        "book": "CRYPTO_RECOMMENDATION_PLAN",
        "authority_pct": 0.0,
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "rules": {
            "adverse_min_n": ADVERSE_MIN_N,
            "adverse_max_wr": ADVERSE_MAX_WR,
            "exit_reverse_min_n": EXIT_REVERSE_MIN_N,
            "holdout_sessions": HOLDOUT_SESSIONS,
            "symbol_in_key": False,
            "books_pooled": False,
            "baseline_of_threshold_unchanged": True,
            "ofx_inert": True,
            "entry_uses": "Gate2E decision-time standaside vs allow",
            "exit_uses": "Gate2X observational reverse timing (no Gate5 apply)",
        },
        "sessions": {
            "all": sessions,
            "train": train_sessions,
            "holdout": sorted(holdout_set),
        },
        "counts": {
            "trades_total": len(enriched),
            "trades_train": len(train_rows),
            "trades_holdout": len(holdout_rows),
            "entry_patterns": len(entry_patterns),
            "exit_patterns": len(exit_patterns),
            "adverse_entry_patterns": sum(
                1 for p in entry_patterns.values() if p["label"] == "ADVERSE"
            ),
            "reverse_exit_patterns": sum(
                1
                for p in exit_patterns.values()
                if p["label"] == "REVERSE_STANDASIDE_HOLD"
            ),
        },
        "entry_patterns": entry_patterns,
        "exit_patterns": exit_patterns,
        "summary_wl": {
            "train": _agg_metrics(train_rows),
            "holdout": _agg_metrics(holdout_rows),
        },
        "exit_reason_counts": dict(
            Counter(str(p.get("exit_reason") or "") for p in closed).most_common()
        ),
    }


def default_library_path(crypto_dir: Optional[Path] = None) -> Path:
    if crypto_dir is None:
        crypto_dir = WORKSPACE / "live_capture" / "shadow_validation" / "crypto"
    return Path(crypto_dir) / "rp608_crypto_plan_trade_pattern_library.json"


def main() -> int:
    ap = argparse.ArgumentParser(description="Mine crypto plan entry+exit pattern library")
    ap.add_argument(
        "--outcomes",
        action="append",
        default=[],
        help="trade_plan_outcomes.jsonl paths (repeatable)",
    )
    ap.add_argument("--output", default="")
    args = ap.parse_args()
    paths = [Path(p) for p in (args.outcomes or [])]
    if not paths:
        paths = [
            WORKSPACE
            / "live_capture"
            / "shadow_validation"
            / "crypto"
            / "trade_plan_overlay_v1"
            / "trade_plan_outcomes.jsonl",
            WORKSPACE
            / "artifacts"
            / "rp608_econ_edge_diag"
            / "source_journals_2026-11-05"
            / "crypto"
            / "trade_plan_outcomes.jsonl",
        ]
    plans: List[Dict[str, Any]] = []
    for p in paths:
        rows = load_jsonl(p)
        print(f"[crypto_pattern_library] {p}: {len(rows)}")
        plans.extend(rows)
    lib = mine_library(plans)
    out = Path(args.output) if args.output else default_library_path()
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(lib, indent=2) + "\n", encoding="utf-8")
    print(
        f"[crypto_pattern_library] wrote {out} "
        f"entry={lib['counts']['entry_patterns']} "
        f"adverse={lib['counts']['adverse_entry_patterns']} "
        f"exit={lib['counts']['exit_patterns']} "
        f"reverse={lib['counts']['reverse_exit_patterns']}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
