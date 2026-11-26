#!/usr/bin/env python3
"""
RP-608-CROSS-SESSION-WARMUP-DIAG-001

Diagnostic-only comparison of same-session warmup vs cross-session warmup
with overnight-gap handling. Does NOT change live G3/G4 thresholds or
deploy a new entry policy.

Policies compared (frozen threshold traj15 >= +0.05; history_ready >= 5):
  A) same_session          — current live semantics (session-causal only)
  B) cross_naive           — prior+today closes concatenated (gap artefact probe)
  C) cross_gap_neutral     — prior+today, overnight return zeroed in traj15

Authority: 0.0%. Output is an observational artifact only.
"""

from __future__ import annotations

import argparse
import json
import math
import sys
from collections import Counter
from dataclasses import dataclass
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional, Sequence, Tuple

WORKSPACE_DIR = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(WORKSPACE_DIR))

IST = timezone(timedelta(hours=5, minutes=30))
TRAJ_THRESHOLD = 0.05
HISTORY_READY_N = 5
MAX_NEW_ENTRY_BAR = 19

# Next-session watchlist (observational) — default diagnostic universe.
DEFAULT_SYMBOLS: Tuple[str, ...] = (
    "TORNTPHARM.NS",
    "GODREJPROP.NS",
    "COALINDIA.NS",
    "ONGC.NS",
    "GAIL.NS",
    "ITC.NS",
    "GODREJCP.NS",
    "RELIANCE.NS",
    "MARICO.NS",
    "PETRONET.NS",
    "SUNPHARMA.NS",
    "MUTHOOTFIN.NS",
    "SRF.NS",
    "PAGEIND.NS",
    "ADANIPORTS.NS",
    "CIPLA.NS",
    "TATACONSUM.NS",
    "ICICIPRULI.NS",
    "TITAN.NS",
    "LUPIN.NS",
)


@dataclass(frozen=True)
class Bar:
    timestamp_utc_ms: int
    open: float
    high: float
    low: float
    close: float
    volume: float
    session_date: str


def _ist_date(ts_ms: int) -> str:
    return datetime.fromtimestamp(ts_ms / 1000.0, tz=timezone.utc).astimezone(IST).date().isoformat()


def _bar_idx_for_session(ts_ms: int, session_date: str) -> int:
    """1-based 15m bar index within NSE cash session 09:15–15:30 IST."""
    dt = datetime.fromtimestamp(ts_ms / 1000.0, tz=timezone.utc).astimezone(IST)
    if dt.date().isoformat() != session_date:
        return 0
    open_dt = dt.replace(hour=9, minute=15, second=0, microsecond=0)
    elapsed_min = (dt - open_dt).total_seconds() / 60.0
    if elapsed_min < 0:
        return 0
    # Candle labeled by close time ≈ end of interval; use floor of elapsed/15.
    idx = int(elapsed_min // 15) + 1
    return max(1, min(idx, 25))


def traj15_from_closes(closes: Sequence[float]) -> float:
    """Exact daemon formula (scripts/run_rp608_nse_shadow_daemon.py)."""
    if len(closes) < 2:
        return 0.0
    lookback = min(len(closes), 15)
    ret = (closes[-1] - closes[-lookback]) / (closes[-lookback] + 1e-9)
    diffs = [
        (closes[i] - closes[i - 1]) / (closes[i - 1] + 1e-9)
        for i in range(-lookback + 1, 0)
    ]
    vol = (sum(d * d for d in diffs) / len(diffs)) ** 0.5 if diffs else 1e-3
    denom = (vol * (lookback ** 0.5) * 2.5) + 1e-9
    return float(math.tanh(ret / denom))


def traj15_gap_neutral(bars: Sequence[Bar]) -> Tuple[float, Optional[float]]:
    """
    Traj15 over closes with overnight step return forced to 0.

    Method (documented): within a session, chain close-to-close returns normally.
    At a session boundary, replace the overnight close→close jump with the
    open→close move of the first bar of the new session (gap excluded).

    Returns (traj15, overnight_gap_bps) for the latest boundary in the window.
    """
    if len(bars) < 2:
        return 0.0, None
    lookback = min(len(bars), 15)
    window = list(bars[-lookback:])
    adj_closes = [window[0].close]
    overnight_gap_bps: Optional[float] = None
    for i in range(1, len(window)):
        prev, cur = window[i - 1], window[i]
        if cur.session_date != prev.session_date:
            overnight_gap_bps = ((cur.open - prev.close) / (prev.close + 1e-9)) * 1e4
            if cur.open > 0:
                adj_closes.append(adj_closes[-1] * (cur.close / cur.open))
            else:
                adj_closes.append(adj_closes[-1])
        elif prev.close > 0:
            adj_closes.append(adj_closes[-1] * (cur.close / prev.close))
        else:
            adj_closes.append(adj_closes[-1])
    return traj15_from_closes(adj_closes), overnight_gap_bps


def evaluate_at(
    *,
    policy: str,
    history: Sequence[Bar],
    bar_idx: int,
) -> Dict[str, Any]:
    """Evaluate G3 readiness + G4 momentum for one policy at one bar."""
    n = len(history)
    g1 = 1 <= bar_idx <= MAX_NEW_ENTRY_BAR
    if policy == "same_session":
        g3 = n >= HISTORY_READY_N
        traj = traj15_from_closes([b.close for b in history]) if n >= 2 else 0.0
        gap_bps = None
        score_kind = "calculated" if n >= 2 else "cold_start_default_zero"
    elif policy == "cross_naive":
        g3 = n >= HISTORY_READY_N
        traj = traj15_from_closes([b.close for b in history]) if n >= 2 else 0.0
        gap_bps = None
        if n >= 2:
            # Report last overnight gap inside history if present.
            for i in range(1, n):
                if history[i].session_date != history[i - 1].session_date:
                    gap_bps = ((history[i].open - history[i - 1].close) / (history[i - 1].close + 1e-9)) * 1e4
        score_kind = "calculated_naive_cross" if n >= 2 else "cold_start_default_zero"
    elif policy == "cross_gap_neutral":
        g3 = n >= HISTORY_READY_N
        traj, gap_bps = traj15_gap_neutral(history) if n >= 2 else (0.0, None)
        score_kind = "calculated_gap_neutral" if n >= 2 else "cold_start_default_zero"
    else:
        raise ValueError(policy)

    # Live NSE entry is long-only at +0.05; short side is observational for stability only.
    g4_long = bool(g3 and traj >= TRAJ_THRESHOLD)
    g4_short_obs = bool(g3 and traj <= -TRAJ_THRESHOLD)
    direction = (
        "LONG"
        if traj >= TRAJ_THRESHOLD
        else ("SHORT_OBS" if traj <= -TRAJ_THRESHOLD else "FLAT")
    )
    return {
        "policy": policy,
        "bar_idx": bar_idx,
        "history_length": n,
        "g1_eligible": g1,
        "g3_ready": g3,
        "traj15": round(float(traj), 4),
        "direction": direction,
        "score_kind": score_kind,
        "overnight_gap_bps": None if gap_bps is None else round(float(gap_bps), 2),
        "g4_pass": g4_long,
        "g4_short_obs": g4_short_obs,
        "signal": bool(g1 and g3 and g4_long),
    }


def split_sessions(bars: Sequence[Bar]) -> Dict[str, List[Bar]]:
    out: Dict[str, List[Bar]] = {}
    for b in bars:
        out.setdefault(b.session_date, []).append(b)
    for k in out:
        out[k].sort(key=lambda x: x.timestamp_utc_ms)
    return out


def prior_session_date(session_map: Dict[str, List[Bar]], today: str) -> Optional[str]:
    dates = sorted(session_map.keys())
    prev = [d for d in dates if d < today]
    return prev[-1] if prev else None


def bars_from_kite(symbol: str, from_utc: datetime, to_utc: datetime) -> List[Bar]:
    from scripts.run_rp608_nse_shadow_daemon import KiteDataFeedAdapter

    adapter = KiteDataFeedAdapter((symbol,))
    raw = adapter.fetch_historical_15m_bars(symbol, from_utc, to_utc)
    out: List[Bar] = []
    for b in raw:
        ts = int(b.timestamp_utc_ms)
        out.append(
            Bar(
                timestamp_utc_ms=ts,
                open=float(b.open),
                high=float(b.high),
                low=float(b.low),
                close=float(b.close),
                volume=float(b.volume),
                session_date=_ist_date(ts),
            )
        )
    out.sort(key=lambda x: x.timestamp_utc_ms)
    return out


def _rows_by_policy(rows: Sequence[Dict[str, Any]], policy: str) -> Dict[int, Dict[str, Any]]:
    return {int(r["bar_idx"]): r for r in rows if r["policy"] == policy}


def classify_early_vs_baseline(
    *,
    candidate_policy: str,
    rows: Sequence[Dict[str, Any]],
) -> Dict[str, Any]:
    """
    Distinguish recovered early opportunities from mere timing advances.

    - incremental_early_only: candidate signals in bars 1–4; baseline never signals later
    - timing_advance: candidate signals in 1–4; baseline also signals at bar >= 5
    - no_early_candidate: no candidate signal in bars 1–4
    """
    base = _rows_by_policy(rows, "same_session")
    cand = _rows_by_policy(rows, candidate_policy)
    early_bars = sorted(b for b, r in cand.items() if b <= 4 and r.get("signal"))
    later_base = sorted(b for b, r in base.items() if b >= 5 and r.get("signal"))
    any_base = sorted(b for b, r in base.items() if r.get("signal"))

    if not early_bars:
        return {
            "class": "no_early_candidate",
            "early_signal_bars": [],
            "baseline_first_signal_bar": any_base[0] if any_base else None,
        }
    if later_base or (any_base and any_base[0] >= 5):
        return {
            "class": "timing_advance",
            "early_signal_bars": early_bars,
            "early_first_bar": early_bars[0],
            "baseline_first_signal_bar": any_base[0] if any_base else None,
        }
    if any_base and any_base[0] <= 4:
        # Baseline cannot normally signal before bar 5; if it does, mark separately.
        return {
            "class": "both_early_unexpected",
            "early_signal_bars": early_bars,
            "baseline_first_signal_bar": any_base[0],
        }
    return {
        "class": "incremental_early_only",
        "early_signal_bars": early_bars,
        "early_first_bar": early_bars[0],
        "baseline_first_signal_bar": None,
    }


def subsequent_trajectory(
    *,
    policy: str,
    rows: Sequence[Dict[str, Any]],
    from_bar: int,
    horizon: int = 4,
) -> Dict[str, Any]:
    """Track traj/direction/G4 persistence after an early signal."""
    by_bar = _rows_by_policy(rows, policy)
    if from_bar not in by_bar:
        return {"available": False}
    start = by_bar[from_bar]
    path = []
    hold_g4 = 0
    sign_flip = False
    start_sign = 1 if start["traj15"] >= 0 else -1
    for b in range(from_bar, from_bar + horizon + 1):
        r = by_bar.get(b)
        if not r:
            break
        path.append(
            {
                "bar_idx": b,
                "traj15": r["traj15"],
                "direction": r["direction"],
                "g4_pass": r["g4_pass"],
                "signal": r["signal"],
            }
        )
        if r["g4_pass"]:
            hold_g4 += 1
        cur_sign = 1 if r["traj15"] >= 0 else -1
        if cur_sign != start_sign and abs(r["traj15"]) >= TRAJ_THRESHOLD:
            sign_flip = True
    return {
        "available": True,
        "from_bar": from_bar,
        "start_traj15": start["traj15"],
        "start_direction": start["direction"],
        "bars_observed": len(path),
        "bars_g4_held": hold_g4,
        "sign_flip_to_opposite_threshold": sign_flip,
        "still_signal_at_horizon": bool(path and path[-1]["signal"]),
        "path": path,
    }


def agreement_after_bar5(
    *,
    candidate_policy: str,
    rows: Sequence[Dict[str, Any]],
) -> Dict[str, Any]:
    base = _rows_by_policy(rows, "same_session")
    cand = _rows_by_policy(rows, candidate_policy)
    bars = sorted(b for b in set(base) & set(cand) if b >= 5)
    if not bars:
        return {"n_bars": 0, "agree_n": 0, "agree_rate": None, "disagreements": []}
    agree = 0
    disagreements = []
    for b in bars:
        if bool(base[b]["signal"]) == bool(cand[b]["signal"]):
            agree += 1
        else:
            disagreements.append(
                {
                    "bar_idx": b,
                    "baseline_signal": base[b]["signal"],
                    "candidate_signal": cand[b]["signal"],
                    "baseline_traj15": base[b]["traj15"],
                    "candidate_traj15": cand[b]["traj15"],
                }
            )
    return {
        "n_bars": len(bars),
        "agree_n": agree,
        "agree_rate": round(agree / len(bars), 4),
        "disagreements": disagreements,
    }


def replay_symbol(symbol: str, all_bars: Sequence[Bar], today: str) -> Dict[str, Any]:
    by_sess = split_sessions(all_bars)
    today_bars = by_sess.get(today, [])
    prev = prior_session_date(by_sess, today)
    prev_bars = by_sess.get(prev, []) if prev else []

    rows: List[Dict[str, Any]] = []
    for i, b in enumerate(today_bars):
        bar_idx = _bar_idx_for_session(b.timestamp_utc_ms, today)
        if bar_idx < 1 or bar_idx > MAX_NEW_ENTRY_BAR:
            continue
        today_prefix = today_bars[: i + 1]
        hist_same = today_prefix
        hist_cross = list(prev_bars) + today_prefix
        for policy, hist in (
            ("same_session", hist_same),
            ("cross_naive", hist_cross),
            ("cross_gap_neutral", hist_cross),
        ):
            rows.append(evaluate_at(policy=policy, history=hist, bar_idx=bar_idx))

    def _first_signal(policy: str) -> Optional[int]:
        for r in rows:
            if r["policy"] == policy and r["signal"]:
                return int(r["bar_idx"])
        return None

    def _g3_first(policy: str) -> Optional[int]:
        for r in rows:
            if r["policy"] == policy and r["g3_ready"]:
                return int(r["bar_idx"])
        return None

    # Per-bar readiness/signals for dashboard-style tables
    g3_by_bar: Dict[str, Dict[str, bool]] = {}
    g4_by_bar: Dict[str, Dict[str, bool]] = {}
    for r in rows:
        b = str(r["bar_idx"])
        g3_by_bar.setdefault(b, {})[r["policy"]] = bool(r["g3_ready"])
        g4_by_bar.setdefault(b, {})[r["policy"]] = bool(r["g4_pass"])

    early_class_naive = classify_early_vs_baseline(candidate_policy="cross_naive", rows=rows)
    early_class_gap = classify_early_vs_baseline(candidate_policy="cross_gap_neutral", rows=rows)

    persistence = {}
    for policy, cls in (
        ("cross_naive", early_class_naive),
        ("cross_gap_neutral", early_class_gap),
    ):
        fb = cls.get("early_first_bar")
        persistence[policy] = (
            subsequent_trajectory(policy=policy, rows=rows, from_bar=int(fb), horizon=4)
            if fb is not None
            else {"available": False, "reason": "no_early_signal"}
        )

    return {
        "symbol": symbol,
        "today": today,
        "prior_session": prev,
        "n_today_bars": len(today_bars),
        "n_prior_bars": len(prev_bars),
        "max_bar_idx_available": (
            max(_rows_by_policy(rows, "same_session").keys())
            if _rows_by_policy(rows, "same_session")
            else 0
        ),
        "first_g3_ready_bar": {
            "same_session": _g3_first("same_session"),
            "cross_naive": _g3_first("cross_naive"),
            "cross_gap_neutral": _g3_first("cross_gap_neutral"),
        },
        "first_signal_bar": {
            "same_session": _first_signal("same_session"),
            "cross_naive": _first_signal("cross_naive"),
            "cross_gap_neutral": _first_signal("cross_gap_neutral"),
        },
        "g3_ready_by_bar": g3_by_bar,
        "g4_pass_by_bar": g4_by_bar,
        "early_vs_baseline": {
            "cross_naive": early_class_naive,
            "cross_gap_neutral": early_class_gap,
        },
        "agreement_after_bar5": {
            "cross_naive": agreement_after_bar5(candidate_policy="cross_naive", rows=rows),
            "cross_gap_neutral": agreement_after_bar5(
                candidate_policy="cross_gap_neutral", rows=rows
            ),
        },
        "subsequent_trajectory_after_early": persistence,
        "bar_rows": rows,
    }


def summarize(symbol_reports: Sequence[Dict[str, Any]]) -> Dict[str, Any]:
    policies = ("same_session", "cross_naive", "cross_gap_neutral")
    summary: Dict[str, Any] = {
        "n_symbols": len(symbol_reports),
        "max_bar_idx_available_median": _median(
            [int(r.get("max_bar_idx_available") or 0) for r in symbol_reports]
        ),
        "by_policy": {},
        "incremental_framework": {},
        "agreement_after_bar5": {},
        "persistence_after_early": {},
    }
    for p in policies:
        g3_bars = [r["first_g3_ready_bar"].get(p) for r in symbol_reports]
        sig_bars = [r["first_signal_bar"].get(p) for r in symbol_reports]
        early_g3 = sum(1 for b in g3_bars if b is not None and b <= 4)
        signals_b1_4 = sum(1 for b in sig_bars if b is not None and b <= 4)
        signals_any = sum(1 for b in sig_bars if b is not None)
        summary["by_policy"][p] = {
            "symbols_g3_ready_by_bar4": early_g3,
            "symbols_with_any_signal_bars_1_19": signals_any,
            "symbols_with_signal_bars_1_4": signals_b1_4,
            "median_first_g3_bar": _median([b for b in g3_bars if b is not None]),
            "median_first_signal_bar": _median([b for b in sig_bars if b is not None]),
        }

    disagree = 0
    gap_sensitive_examples = []
    for r in symbol_reports:
        naive = _rows_by_policy(r["bar_rows"], "cross_naive")
        neut = _rows_by_policy(r["bar_rows"], "cross_gap_neutral")
        hit = False
        for bidx in sorted(set(naive) & set(neut)):
            if bidx <= 4 and naive[bidx]["signal"] != neut[bidx]["signal"]:
                hit = True
                if len(gap_sensitive_examples) < 12:
                    gap_sensitive_examples.append(
                        {
                            "symbol": r["symbol"],
                            "bar_idx": bidx,
                            "naive_traj15": naive[bidx]["traj15"],
                            "gap_neutral_traj15": neut[bidx]["traj15"],
                            "overnight_gap_bps": neut[bidx].get("overnight_gap_bps"),
                            "naive_signal": naive[bidx]["signal"],
                            "gap_neutral_signal": neut[bidx]["signal"],
                        }
                    )
                break
        if hit:
            disagree += 1
    summary["symbols_where_naive_vs_gap_neutral_signal_differs_bars_1_4"] = disagree
    summary["overnight_gap_sensitivity_examples"] = gap_sensitive_examples

    for cand in ("cross_naive", "cross_gap_neutral"):
        classes = [r["early_vs_baseline"][cand]["class"] for r in symbol_reports]
        c = Counter(classes)
        summary["incremental_framework"][cand] = {
            "incremental_early_only": c.get("incremental_early_only", 0),
            "timing_advance": c.get("timing_advance", 0),
            "no_early_candidate": c.get("no_early_candidate", 0),
            "both_early_unexpected": c.get("both_early_unexpected", 0),
            "note": (
                "timing_advance = early candidate signal that later appears under same_session; "
                "not counted as an additional opportunity"
            ),
        }

        agrees = [r["agreement_after_bar5"][cand] for r in symbol_reports]
        usable = [a for a in agrees if a.get("n_bars")]
        if usable:
            total_bars = sum(a["n_bars"] for a in usable)
            total_agree = sum(a["agree_n"] for a in usable)
            summary["agreement_after_bar5"][cand] = {
                "symbols_with_bar5_plus": len(usable),
                "bar_comparisons": total_bars,
                "agree_n": total_agree,
                "agree_rate": round(total_agree / total_bars, 4) if total_bars else None,
            }
        else:
            summary["agreement_after_bar5"][cand] = {
                "symbols_with_bar5_plus": 0,
                "bar_comparisons": 0,
                "agree_n": 0,
                "agree_rate": None,
                "note": "bar 5+ not yet available in this replay window",
            }

        pers = [r["subsequent_trajectory_after_early"][cand] for r in symbol_reports]
        avail = [p for p in pers if p.get("available")]
        if avail:
            summary["persistence_after_early"][cand] = {
                "n_early_signals": len(avail),
                "still_signal_at_horizon_n": sum(1 for p in avail if p.get("still_signal_at_horizon")),
                "sign_flip_n": sum(1 for p in avail if p.get("sign_flip_to_opposite_threshold")),
                "mean_bars_g4_held": round(
                    sum(float(p.get("bars_g4_held") or 0) for p in avail) / len(avail), 3
                ),
            }
        else:
            summary["persistence_after_early"][cand] = {"n_early_signals": 0}

    return summary


def _median(vals: List[int]) -> Optional[float]:
    if not vals:
        return None
    s = sorted(vals)
    mid = len(s) // 2
    if len(s) % 2:
        return float(s[mid])
    return (s[mid - 1] + s[mid]) / 2.0


def main() -> int:
    parser = argparse.ArgumentParser(description="RP-608 cross-session warmup diagnostic (0% authority)")
    parser.add_argument("--today", default=None, help="IST session date YYYY-MM-DD (default: today IST)")
    parser.add_argument("--lookback-days", type=int, default=5, help="Calendar days of 15m history to fetch")
    parser.add_argument("--symbols", nargs="*", default=None, help="Symbols (default: 20-name watchlist)")
    parser.add_argument(
        "--out",
        default=str(
            WORKSPACE_DIR
            / "artifacts"
            / "rp608_cross_session_warmup"
            / "diagnostic_report.json"
        ),
    )
    parser.add_argument("--max-symbols", type=int, default=20)
    args = parser.parse_args()

    now_ist = datetime.now(timezone.utc).astimezone(IST)
    today = args.today or now_ist.date().isoformat()
    symbols = tuple(args.symbols) if args.symbols else DEFAULT_SYMBOLS[: args.max_symbols]

    from_ist = datetime.fromisoformat(today).replace(tzinfo=IST) - timedelta(days=int(args.lookback_days))
    to_ist = datetime.fromisoformat(today).replace(hour=15, minute=30, second=0, microsecond=0, tzinfo=IST)
    from_utc = from_ist.astimezone(timezone.utc)
    to_utc = to_ist.astimezone(timezone.utc)

    reports: List[Dict[str, Any]] = []
    errors: Dict[str, str] = {}
    for sym in symbols:
        try:
            bars = bars_from_kite(sym, from_utc, to_utc)
            # Keep only cash-session bars 09:15–15:30
            filtered: List[Bar] = []
            for b in bars:
                dt = datetime.fromtimestamp(b.timestamp_utc_ms / 1000.0, tz=timezone.utc).astimezone(IST)
                mins = dt.hour * 60 + dt.minute
                if 9 * 60 + 15 <= mins <= 15 * 60 + 30:
                    filtered.append(b)
            reports.append(replay_symbol(sym, filtered, today))
            print(f"[ok] {sym} bars={len(filtered)} prior={reports[-1]['prior_session']} today_n={reports[-1]['n_today_bars']}")
        except Exception as exc:
            errors[sym] = f"{type(exc).__name__}: {exc}"
            print(f"[err] {sym}: {errors[sym]}")

    payload = {
        "protocol_id": "RP-608-CROSS-SESSION-WARMUP-DIAG-001",
        "authority": "0.0%",
        "disposition": "INVESTIGATION_ONLY_NO_LIVE_DEPLOY",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "today_session": today,
        "threshold_traj15": TRAJ_THRESHOLD,
        "history_ready_n": HISTORY_READY_N,
        "policies": [
            "same_session",
            "cross_naive",
            "cross_gap_neutral",
        ],
        "notes": [
            "G1/G2/G4/G5 rules unchanged; only G3 history provenance differs across policies.",
            "cross_naive exposes overnight-gap artefacts; cross_gap_neutral zeros the overnight return.",
            "RP-607 profiles are not used as candle substitutes.",
            "Incremental early_only ≠ timing_advance: early signals that later appear under same_session are not additional opportunities.",
            "Live NSE entry remains long-only; SHORT_OBS is diagnostic direction only.",
        ],
        "summary": summarize(reports) if reports else {},
        "errors": errors,
        "symbols": reports,
    }

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(payload, indent=2), encoding="utf-8")
    print(f"[wrote] {out_path}")
    print(json.dumps(payload.get("summary"), indent=2))
    return 0 if reports else 2


if __name__ == "__main__":
    raise SystemExit(main())
