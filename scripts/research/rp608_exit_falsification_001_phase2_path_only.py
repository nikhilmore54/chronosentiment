#!/usr/bin/env python3
"""
RP-608-EXIT-FALSIFICATION-001 — Phase 2 PATH_ONLY

Replay frozen Estimand A entries under exit policies A–E.
No parameter tuning. Production journal read-only. Authority 0%.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from collections import Counter, defaultdict
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

WORKSPACE = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(WORKSPACE))
sys.path.insert(0, str(WORKSPACE / "scripts"))

from rp608_nse_equity_costs import calc_indian_equity_friction  # noqa: E402

PROTOCOL_ID = "RP-608-EXIT-FALSIFICATION-001"
PARENT_PROTOCOL = "RP-608-NSE-SHADOW-001"
MANDATORY_SQUARE_OFF_BAR = 23
BARS_60M = 4
BARS_180M = 12
POLICIES = ("A", "B", "C", "D", "E")

CALENDAR_REASONS = {
    "MANDATORY_INTRADAY_SQUARE_OFF",
    "MAX_HOLD_HORIZON_180M",
    "TARGET_HORIZON_180M",
}


def _sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def _day_utc(ms: Any) -> Optional[str]:
    try:
        return datetime.fromtimestamp(int(ms) / 1000, tz=timezone.utc).strftime("%Y-%m-%d")
    except Exception:
        return None


def _load_jsonl(path: Path) -> List[Dict[str, Any]]:
    out: List[Dict[str, Any]] = []
    with path.open() as f:
        for line in f:
            line = line.strip()
            if line:
                out.append(json.loads(line))
    return out


def _fric_total(fric: Any) -> float:
    if isinstance(fric, dict):
        return float(fric.get("total_friction") or 0.0)
    return float(fric or 0.0)


def _align_entry(
    bars: List[Dict[str, Any]], entry_bar_idx: Optional[int], entry_ts_ms: int
) -> Tuple[Optional[int], str]:
    if entry_bar_idx is not None and 0 <= int(entry_bar_idx) < len(bars):
        return int(entry_bar_idx), "ENTRY_BAR_IDX"
    idxs = [i for i, b in enumerate(bars) if int(b.get("timestamp_utc_ms") or 0) >= entry_ts_ms]
    if not idxs:
        return None, "NO_BAR_AT_OR_AFTER_ENTRY_TS"
    return idxs[0], "ENTRY_TS_MATCH"


def _session_bar_number(list_idx: int, entry_list_idx: int, entry_bar_idx_stamped: int) -> int:
    """Map list index → daemon session bar_idx using stamped entry_bar_idx as anchor."""
    return int(entry_bar_idx_stamped) + (list_idx - entry_list_idx)


def _pnl(entry: float, exit_px: float, qty: float, half_spread: float) -> Dict[str, Any]:
    gross = (exit_px - entry) * qty
    fric = calc_indian_equity_friction(
        buy_price=entry, sell_price=exit_px, quantity=qty, half_spread_bps=half_spread
    )
    net = gross - float(fric.get("total_friction") or 0.0)
    return {
        "exit_price": round(exit_px, 4),
        "gross_pnl_inr": round(gross, 2),
        "friction": fric,
        "friction_inr": round(float(fric.get("total_friction") or 0.0), 2),
        "net_pnl_inr": round(net, 2),
    }


def _d_target(entry: float, prod_target: float) -> float:
    d = float(prod_target) - float(entry)
    return float(entry) + 0.75 * d


def replay_policy(
    trade: Dict[str, Any],
    bars: List[Dict[str, Any]],
    policy: str,
    *,
    path_mode: str = "CLOSE_GRID",
) -> Dict[str, Any]:
    """
    Forward path replay for one policy.

    Primary path_mode=CLOSE_GRID (normative for this protocol):
      Decision and fill on completed bar **close** only — matches Phase 1 MTM
      and avoids wick-invented SL/TP that diverge from daemon tick p_ref.

    Per bar after entry, production-compatible order:
      1) stop if close <= stop
      2) target if close >= target
      3) mandatory square-off if session bar >= 23
      4) max-hold if bars_held >= 12
      5) policy-specific 60m rule on close

    Fill: bar.close (SL/TP also at close when the close-grid trigger fires).
    """
    if path_mode != "CLOSE_GRID":
        raise ValueError(f"unsupported path_mode={path_mode!r}; only CLOSE_GRID authorized")

    entry = float(trade["entry_price"])
    qty = float(trade["quantity"])
    half = float(trade.get("half_spread_bps") or 2.5)
    stop = float(trade["stop_loss_price"])
    prod_tp = float(trade["target_price"])
    tp = _d_target(entry, prod_tp) if policy == "D" else prod_tp
    entry_bar_stamped = int(trade["entry_bar_idx"])
    entry_ts = int(trade["entry_timestamp_utc_ms"])

    e_idx, align = _align_entry(bars, entry_bar_stamped, entry_ts)
    base = {
        "policy": policy,
        "trade_id": trade.get("trade_id"),
        "symbol": trade.get("symbol"),
        "bar_align_method": align,
        "path_mode": path_mode,
        "reconstructible": False,
    }
    if e_idx is None:
        base["non_reconstructible_reason"] = align
        return base

    for list_idx in range(e_idx + 1, len(bars)):
        b = bars[list_idx]
        close = float(b.get("close") or entry)
        if close <= 0:
            continue
        sess_bar = _session_bar_number(list_idx, e_idx, entry_bar_stamped)
        bars_held = sess_bar - entry_bar_stamped

        # 1) Stop on close
        if close <= stop:
            out = {**base, **_pnl(entry, close, qty, half)}
            out.update(
                {
                    "reconstructible": True,
                    "exit_reason": "STOP_LOSS",
                    "exit_list_idx": list_idx,
                    "exit_bar_idx": sess_bar,
                    "bars_held": bars_held,
                    "exit_fill_source": "BAR_CLOSE",
                }
            )
            return out

        # 2) Target on close
        if close >= tp:
            out = {**base, **_pnl(entry, close, qty, half)}
            out.update(
                {
                    "reconstructible": True,
                    "exit_reason": "TAKE_PROFIT",
                    "exit_list_idx": list_idx,
                    "exit_bar_idx": sess_bar,
                    "bars_held": bars_held,
                    "exit_fill_source": "BAR_CLOSE",
                    "target_price_used": round(tp, 4),
                }
            )
            return out

        # 3) Square-off
        if sess_bar >= MANDATORY_SQUARE_OFF_BAR:
            out = {**base, **_pnl(entry, close, qty, half)}
            out.update(
                {
                    "reconstructible": True,
                    "exit_reason": "MANDATORY_INTRADAY_SQUARE_OFF",
                    "exit_list_idx": list_idx,
                    "exit_bar_idx": sess_bar,
                    "bars_held": bars_held,
                    "exit_fill_source": "BAR_CLOSE",
                }
            )
            return out

        # 4) Max-hold 180m
        if bars_held >= BARS_180M:
            out = {**base, **_pnl(entry, close, qty, half)}
            out.update(
                {
                    "reconstructible": True,
                    "exit_reason": "MAX_HOLD_HORIZON_180M",
                    "exit_list_idx": list_idx,
                    "exit_bar_idx": sess_bar,
                    "bars_held": bars_held,
                    "exit_fill_source": "BAR_CLOSE",
                }
            )
            return out

        # 5) 60m policy branch
        if bars_held >= BARS_60M:
            if policy == "B":
                out = {**base, **_pnl(entry, close, qty, half)}
                out.update(
                    {
                        "reconstructible": True,
                        "exit_reason": "UNCONDITIONAL_HORIZON_60M",
                        "exit_list_idx": list_idx,
                        "exit_bar_idx": sess_bar,
                        "bars_held": bars_held,
                        "exit_fill_source": "BAR_CLOSE",
                    }
                )
                return out
            if policy == "E" and close <= entry:
                out = {**base, **_pnl(entry, close, qty, half)}
                out.update(
                    {
                        "reconstructible": True,
                        "exit_reason": "ADVERSE_CUT_60M",
                        "exit_list_idx": list_idx,
                        "exit_bar_idx": sess_bar,
                        "bars_held": bars_held,
                        "exit_fill_source": "BAR_CLOSE",
                    }
                )
                return out
            if policy in ("A", "D", "E") and close > entry:
                out = {**base, **_pnl(entry, close, qty, half)}
                out.update(
                    {
                        "reconstructible": True,
                        "exit_reason": "TARGET_HORIZON_60M",
                        "exit_list_idx": list_idx,
                        "exit_bar_idx": sess_bar,
                        "bars_held": bars_held,
                        "exit_fill_source": "BAR_CLOSE",
                    }
                )
                return out
            # C: no profit-conditioned horizon — continue
            # A/D with close <= entry: continue toward max-hold/square-off

    last = bars[-1]
    close = float(last.get("close") or entry)
    out = {**base, **_pnl(entry, close, qty, half)}
    out.update(
        {
            "reconstructible": True,
            "exit_reason": "PATH_END_NO_RULE",
            "exit_list_idx": len(bars) - 1,
            "exit_bar_idx": _session_bar_number(len(bars) - 1, e_idx, entry_bar_stamped),
            "bars_held": _session_bar_number(len(bars) - 1, e_idx, entry_bar_stamped)
            - entry_bar_stamped,
            "exit_fill_source": "BAR_CLOSE",
            "warning": "exhausted_bar_history",
        }
    )
    return out


def control_from_journal(trade: Dict[str, Any]) -> Dict[str, Any]:
    return {
        "policy": "A",
        "trade_id": trade.get("trade_id"),
        "symbol": trade.get("symbol"),
        "reconstructible": True,
        "bar_align_method": "JOURNAL_CONTROL",
        "exit_reason": trade.get("exit_reason"),
        "exit_bar_idx": trade.get("exit_bar_idx"),
        "bars_held": (
            int(trade["exit_bar_idx"]) - int(trade["entry_bar_idx"])
            if trade.get("exit_bar_idx") is not None and trade.get("entry_bar_idx") is not None
            else None
        ),
        "exit_price": float(trade.get("exit_price") or 0.0),
        "gross_pnl_inr": float(trade.get("gross_pnl_inr") or 0.0),
        "friction_inr": _fric_total(trade.get("friction")),
        "net_pnl_inr": float(trade.get("net_pnl_inr") or 0.0),
        "exit_fill_source": "JOURNAL",
    }


def _agg(rows: List[Dict[str, Any]]) -> Dict[str, Any]:
    n = len(rows)
    if not n:
        return {"n": 0}
    nets = [float(r.get("net_pnl_inr") or 0.0) for r in rows]
    gross = [float(r.get("gross_pnl_inr") or 0.0) for r in rows]
    fric = [float(r.get("friction_inr") or 0.0) for r in rows]
    return {
        "n": n,
        "net_pnl_inr": round(sum(nets), 2),
        "gross_pnl_inr": round(sum(gross), 2),
        "friction_inr": round(sum(fric), 2),
        "avg_net_pnl_inr": round(sum(nets) / n, 2),
        "net_win_n": sum(1 for x in nets if x > 0),
        "net_win_rate": round(sum(1 for x in nets if x > 0) / n, 4),
        "exit_reason_counts": dict(Counter(r.get("exit_reason") for r in rows)),
    }


def _symbol_concentration(deltas: List[Tuple[str, float]]) -> Dict[str, Any]:
    """Share of |Δ| mass in top symbols."""
    by = defaultdict(float)
    for sym, d in deltas:
        by[sym] += float(d)
    total_abs = sum(abs(v) for v in by.values()) or 1.0
    ranked = sorted(by.items(), key=lambda kv: abs(kv[1]), reverse=True)
    top1 = ranked[0] if ranked else ("", 0.0)
    top5_abs = sum(abs(v) for _, v in ranked[:5])
    return {
        "n_symbols": len(by),
        "top1_symbol": top1[0],
        "top1_delta_inr": round(top1[1], 2),
        "top1_share_of_abs_delta": round(abs(top1[1]) / total_abs, 4),
        "top5_share_of_abs_delta": round(top5_abs / total_abs, 4),
        "top5": [
            {"symbol": s, "delta_net_inr": round(v, 2)} for s, v in ranked[:5]
        ],
    }


def summarize_phase2(
    trades: List[Dict[str, Any]],
    by_policy: Dict[str, List[Dict[str, Any]]],
) -> Dict[str, Any]:
    control = {r["trade_id"]: r for r in by_policy["A"]}
    out: Dict[str, Any] = {
        "protocol_id": PROTOCOL_ID,
        "phase": 2,
        "mode": "PATH_ONLY",
        "n_entries": len(trades),
        "session_clustered_ci": "UNDERPOWERED",
        "session_clustered_ci_reason": "single_session_sample",
        "policies": {},
    }

    # Index production realized reason for cohort slices
    realized_reason = {t["trade_id"]: t.get("exit_reason") for t in trades}

    for pol in POLICIES:
        rows = by_policy[pol]
        ok = [r for r in rows if r.get("reconstructible")]
        block: Dict[str, Any] = {
            "headline": _agg(ok),
            "n_non_reconstructible": sum(1 for r in rows if not r.get("reconstructible")),
        }
        if pol != "A":
            paired = []
            for r in ok:
                tid = r["trade_id"]
                c = control.get(tid)
                if not c:
                    continue
                d_net = float(r.get("net_pnl_inr") or 0) - float(c.get("net_pnl_inr") or 0)
                paired.append(
                    {
                        "trade_id": tid,
                        "symbol": r.get("symbol"),
                        "delta_net_inr": round(d_net, 2),
                        "policy_exit_reason": r.get("exit_reason"),
                        "control_exit_reason": c.get("exit_reason"),
                        "realized_exit_reason": realized_reason.get(tid),
                    }
                )
            block["paired_vs_A"] = {
                "n": len(paired),
                "mean_delta_net_inr": round(
                    sum(p["delta_net_inr"] for p in paired) / len(paired), 2
                )
                if paired
                else None,
                "sum_delta_net_inr": round(sum(p["delta_net_inr"] for p in paired), 2),
                "n_improved": sum(1 for p in paired if p["delta_net_inr"] > 0),
                "n_worsened": sum(1 for p in paired if p["delta_net_inr"] < 0),
                "n_unchanged": sum(1 for p in paired if p["delta_net_inr"] == 0),
                "symbol_concentration": _symbol_concentration(
                    [(p["symbol"], p["delta_net_inr"]) for p in paired]
                ),
                # Exit-category migration: control reason → policy reason counts
                "exit_migration": dict(
                    Counter(
                        f"{p['control_exit_reason']}->{p['policy_exit_reason']}" for p in paired
                    )
                ),
            }

            # Stratify by *realized production* calendar cohorts
            for label, reasons in (
                ("realized_square_off", {"MANDATORY_INTRADAY_SQUARE_OFF"}),
                ("realized_max_hold_180m", {"MAX_HOLD_HORIZON_180M", "TARGET_HORIZON_180M"}),
                ("realized_calendar_64", CALENDAR_REASONS),
                ("realized_horizon_60m", {"TARGET_HORIZON_60M"}),
                ("realized_stop_loss", {"STOP_LOSS"}),
            ):
                sub_p = [
                    p
                    for p in paired
                    if realized_reason.get(p["trade_id"]) in reasons
                ]
                if not sub_p:
                    block["paired_vs_A"][label] = {"n": 0}
                    continue
                block["paired_vs_A"][label] = {
                    "n": len(sub_p),
                    "sum_delta_net_inr": round(sum(p["delta_net_inr"] for p in sub_p), 2),
                    "mean_delta_net_inr": round(
                        sum(p["delta_net_inr"] for p in sub_p) / len(sub_p), 2
                    ),
                    "n_improved": sum(1 for p in sub_p if p["delta_net_inr"] > 0),
                    "n_worsened": sum(1 for p in sub_p if p["delta_net_inr"] < 0),
                    "policy_exit_reason_counts": dict(
                        Counter(p["policy_exit_reason"] for p in sub_p)
                    ),
                }

        out["policies"][pol] = block
    return out


def write_report(summary: Dict[str, Any], preflight: Dict[str, Any], path: Path) -> None:
    lines = [
        f"# {PROTOCOL_ID} — Phase 2 PATH_ONLY (A–E)",
        "",
        f"**Generated (UTC):** {preflight.get('generated_utc')}",
        f"**Session:** `{preflight.get('session_date')}`",
        f"**Mode:** PATH_ONLY · **Authority:** 0% · journal untouched · no live implementation",
        f"**Session-clustered CI:** `{summary.get('session_clustered_ci')}` — {summary.get('session_clustered_ci_reason')}",
        "",
        "## Preflight",
        "",
        "```json",
        json.dumps(preflight, indent=2, sort_keys=True),
        "```",
        "",
        "## Headline net P&L by policy (identical entries)",
        "",
        "| Policy | n | Net ₹ | Gross ₹ | Friction ₹ | Net WR | Sum Δ vs A ₹ | Mean Δ ₹ | Improved | Worsened |",
        "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
    ]
    for pol in POLICIES:
        h = summary["policies"][pol]["headline"]
        paired = summary["policies"][pol].get("paired_vs_A") or {}
        lines.append(
            f"| **{pol}** | {h.get('n', 0)} | {h.get('net_pnl_inr', 0):+,.0f} | "
            f"{h.get('gross_pnl_inr', 0):+,.0f} | {h.get('friction_inr', 0):,.0f} | "
            f"{100 * (h.get('net_win_rate') or 0):.1f}% | "
            f"{(paired.get('sum_delta_net_inr') if pol != 'A' else 0) or 0:+,.0f} | "
            f"{(paired.get('mean_delta_net_inr') if pol != 'A' else 0) or 0:+,.1f} | "
            f"{paired.get('n_improved', '—')} | {paired.get('n_worsened', '—')} |"
        )

    lines.extend(["", "## Exit-reason mix (category move check)", ""])
    for pol in POLICIES:
        h = summary["policies"][pol]["headline"]
        lines.append(f"- **{pol}:** `{h.get('exit_reason_counts')}`")

    lines.extend(
        [
            "",
            "## Stratified paired Δ vs A (by *realized production* exit cohort)",
            "",
            "Phase 1 distinction preserved: square-off vs max-hold are reported separately.",
            "",
            "| Policy | Square-off Δ ₹ (n) | Max-hold Δ ₹ (n) | Calendar-64 Δ ₹ | Horizon-60m Δ ₹ | Stop Δ ₹ |",
            "| --- | ---: | ---: | ---: | ---: | ---: |",
        ]
    )
    for pol in ("B", "C", "D", "E"):
        p = summary["policies"][pol]["paired_vs_A"]
        def cell(key: str) -> str:
            b = p.get(key) or {}
            if not b.get("n"):
                return "—"
            return f"{b['sum_delta_net_inr']:+,.0f} ({b['n']})"

        lines.append(
            f"| **{pol}** | {cell('realized_square_off')} | {cell('realized_max_hold_180m')} | "
            f"{cell('realized_calendar_64')} | {cell('realized_horizon_60m')} | {cell('realized_stop_loss')} |"
        )

    lines.extend(["", "## Symbol concentration of Δ (top abs mass)", ""])
    for pol in ("B", "C", "D", "E"):
        c = summary["policies"][pol]["paired_vs_A"]["symbol_concentration"]
        lines.append(
            f"- **{pol}:** top1 `{c['top1_symbol']}` Δ={c['top1_delta_inr']:+.0f} "
            f"({100 * c['top1_share_of_abs_delta']:.1f}% of |Δ|); "
            f"top5 share {100 * c['top5_share_of_abs_delta']:.1f}%"
        )

    be = summary.get("policy_B_vs_E") or {}
    lines.extend(
        [
            "",
            "## Policy E vs B (frozen-definition collapse check)",
            "",
            f"- Identical economics on CLOSE_GRID: **{be.get('n_identical_economics')}/{be.get('n')}**",
            f"- Note: {be.get('note')}",
            "",
            "## Method",
            "",
            "- **A** = journaled production control (not path-rewritten).",
            "- **B–E** = forward **CLOSE_GRID** replay from identical entries (SL/TP/time on bar.close).",
            "- Universe = first exit per `trade_id` (journal duplicate closes dropped; see preflight).",
            "- Wick/OHLC first-touch is **not** used (invents stops vs daemon tick `p_ref`).",
            "- Friction = `calc_indian_equity_friction` (same helper as Estimand A).",
            "- No parameter tuning / no post-hoc threshold selection.",
            "- Single session → session-clustered inference **UNDERPOWERED**.",
            "- Results do **not** authorize live exit changes.",
            "",
            "## A_PATH fidelity diagnostic (not a challenger)",
            "",
            "```json",
            json.dumps(summary.get("a_path_diagnostic_vs_journal"), indent=2),
            "```",
            "",
        ]
    )
    path.write_text("\n".join(lines) + "\n")


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--journal",
        type=Path,
        default=WORKSPACE
        / "artifacts/rp608_exit_falsification_001/inputs/rp608_nse_trade_journal.jsonl",
    )
    ap.add_argument(
        "--bar-history",
        type=Path,
        default=WORKSPACE
        / "artifacts/rp608_exit_falsification_001/inputs/rp608_nse_bar_history.json",
    )
    ap.add_argument("--session-date", default="2026-11-02")
    ap.add_argument(
        "--out-dir",
        type=Path,
        default=WORKSPACE / "artifacts/rp608_exit_falsification_001",
    )
    args = ap.parse_args()
    out_dir: Path = args.out_dir
    out_dir.mkdir(parents=True, exist_ok=True)

    journal = args.journal
    bars_path = args.bar_history
    rows = _load_jsonl(journal)
    raw_trades = [
        r
        for r in rows
        if _day_utc(r.get("exit_timestamp_utc_ms")) == args.session_date
        and str(r.get("protocol_id") or PARENT_PROTOCOL) == PARENT_PROTOCOL
        and str(r.get("side") or "LONG").upper() == "LONG"
    ]
    # Journal anomaly: some trade_ids are closed twice (near-duplicate exits).
    # PATH_ONLY universe = first exit per trade_id (earliest exit_timestamp_utc_ms).
    by_tid: Dict[str, List[Dict[str, Any]]] = defaultdict(list)
    for r in raw_trades:
        by_tid[str(r.get("trade_id"))].append(r)
    trades = []
    n_dup_rows = 0
    for tid, group in sorted(by_tid.items(), key=lambda kv: kv[0]):
        group_sorted = sorted(group, key=lambda r: int(r.get("exit_timestamp_utc_ms") or 0))
        trades.append(group_sorted[0])
        n_dup_rows += max(0, len(group_sorted) - 1)
    bars_by_sym: Dict[str, List[Dict[str, Any]]] = json.loads(bars_path.read_text())

    preflight = {
        "protocol_id": PROTOCOL_ID,
        "phase": 2,
        "mode": "PATH_ONLY",
        "path_mode": "CLOSE_GRID",
        "generated_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "session_date": args.session_date,
        "journal_path": str(journal),
        "journal_bytes": journal.stat().st_size,
        "journal_sha256": _sha256_file(journal),
        "bar_history_path": str(bars_path),
        "bar_history_bytes": bars_path.stat().st_size,
        "bar_history_sha256": _sha256_file(bars_path),
        "n_raw_session_rows": len(raw_trades),
        "n_duplicate_trade_id_rows_dropped": n_dup_rows,
        "n_entries": len(trades),
        "dedupe_rule": "first_exit_per_trade_id",
        "realized_exit_reason_counts": dict(Counter(t.get("exit_reason") for t in trades)),
        "policies_frozen": list(POLICIES),
        "production_journal_mutated": False,
        "authority_pct": 0.0,
        "live_implementation_authorized": False,
        "parameter_tuning": False,
        "policy_E_note": (
            "Under production profit-conditioned 60m horizon, E's green branch "
            "exits at the same bar as B; CLOSE_GRID economics collapse B≡E."
        ),
    }

    by_policy: Dict[str, List[Dict[str, Any]]] = {p: [] for p in POLICIES}
    per_trade_rows: List[Dict[str, Any]] = []

    for t in trades:
        sym = str(t.get("symbol") or "")
        bars = bars_by_sym.get(sym) or []
        control = control_from_journal(t)
        by_policy["A"].append(control)
        trade_rec: Dict[str, Any] = {
            "trade_id": t.get("trade_id"),
            "symbol": sym,
            "realized_exit_reason": t.get("exit_reason"),
            "entry_price": t.get("entry_price"),
            "quantity": t.get("quantity"),
            "policies": {"A": control},
        }
        for pol in ("B", "C", "D", "E"):
            # Also replay A-equivalent path as sanity? Only B-E challengers path-replay.
            rep = replay_policy(t, bars, pol)
            by_policy[pol].append(rep)
            trade_rec["policies"][pol] = rep
        # Path-replay of production rules for diagnostics (labeled A_PATH, not used as control)
        a_path = replay_policy(t, bars, "A")
        trade_rec["policies"]["A_PATH_DIAGNOSTIC"] = a_path
        per_trade_rows.append(trade_rec)

    summary = summarize_phase2(trades, by_policy)
    summary["preflight"] = preflight

    # A_PATH diagnostic fidelity vs journal control
    a_path_deltas = []
    for rec in per_trade_rows:
        c = rec["policies"]["A"]
        p = rec["policies"]["A_PATH_DIAGNOSTIC"]
        if p.get("reconstructible"):
            a_path_deltas.append(
                float(p.get("net_pnl_inr") or 0) - float(c.get("net_pnl_inr") or 0)
            )
    reason_match = 0
    reason_mig: Counter = Counter()
    for rec in per_trade_rows:
        c = rec["policies"]["A"]
        p = rec["policies"]["A_PATH_DIAGNOSTIC"]
        if not p.get("reconstructible"):
            continue
        if c.get("exit_reason") == p.get("exit_reason"):
            reason_match += 1
        reason_mig[f"{c.get('exit_reason')}->{p.get('exit_reason')}"] += 1
    n_diag = len(a_path_deltas)
    summary["a_path_diagnostic_vs_journal"] = {
        "path_mode": "CLOSE_GRID",
        "n": n_diag,
        "sum_delta_net_inr": round(sum(a_path_deltas), 2),
        "mean_delta_net_inr": round(sum(a_path_deltas) / n_diag, 2) if n_diag else None,
        "exit_reason_match_n": reason_match,
        "exit_reason_match_rate": round(reason_match / n_diag, 4) if n_diag else None,
        "exit_reason_migration_top": reason_mig.most_common(12),
        "note": (
            "CLOSE_GRID replay of production rules vs journaled tick control; "
            "not a challenger. Residual Δ expected from intrabar p_ref vs bar close."
        ),
    }

    n_be_same = 0
    for rec in per_trade_rows:
        b = rec["policies"]["B"]
        e = rec["policies"]["E"]
        if not b.get("reconstructible") or not e.get("reconstructible"):
            continue
        if float(b.get("net_pnl_inr") or 0) == float(e.get("net_pnl_inr") or 0) and float(
            b.get("exit_price") or 0
        ) == float(e.get("exit_price") or 0):
            n_be_same += 1
    summary["policy_B_vs_E"] = {
        "n": len(per_trade_rows),
        "n_identical_economics": n_be_same,
        "note": preflight["policy_E_note"],
    }

    jsonl_path = out_dir / f"phase2_path_only_per_trade_{args.session_date}.jsonl"
    with jsonl_path.open("w") as f:
        for rec in per_trade_rows:
            f.write(json.dumps(rec, sort_keys=True) + "\n")

    summary_path = out_dir / f"phase2_path_only_summary_{args.session_date}.json"
    summary_path.write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n")

    report_path = out_dir / f"phase2_path_only_report_{args.session_date}.md"
    write_report(summary, preflight, report_path)

    preflight_path = out_dir / f"phase2_preflight_{args.session_date}.json"
    preflight_path.write_text(json.dumps(preflight, indent=2, sort_keys=True) + "\n")

    print(
        json.dumps(
            {
                "ok": True,
                "headline": {
                    pol: summary["policies"][pol]["headline"].get("net_pnl_inr")
                    for pol in POLICIES
                },
                "sum_delta_vs_A": {
                    pol: (summary["policies"][pol].get("paired_vs_A") or {}).get(
                        "sum_delta_net_inr"
                    )
                    for pol in ("B", "C", "D", "E")
                },
                "a_path_diagnostic": summary["a_path_diagnostic_vs_journal"],
                "artifacts": {
                    "jsonl": str(jsonl_path),
                    "summary": str(summary_path),
                    "report": str(report_path),
                    "preflight": str(preflight_path),
                },
            },
            indent=2,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
