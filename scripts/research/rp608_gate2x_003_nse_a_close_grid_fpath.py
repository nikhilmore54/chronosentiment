#!/usr/bin/env python3
"""
RP-608-GATE2X-003 — NSE A multi-session CLOSE_GRID + F_path.

Frozen journal entries. Kite 15m backfill bars. Authority 0%.
No live daemon edits.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import sys
from collections import Counter, defaultdict
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

WORKSPACE = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(WORKSPACE))
sys.path.insert(0, str(WORKSPACE / "scripts"))

from rp608_nse_equity_costs import calc_indian_equity_friction  # noqa: E402

import importlib.util

_spec = importlib.util.spec_from_file_location(
    "rp608_exit_falsification_001_phase2_path_only",
    WORKSPACE / "scripts/research/rp608_exit_falsification_001_phase2_path_only.py",
)
_ef = importlib.util.module_from_spec(_spec)
assert _spec.loader is not None
_spec.loader.exec_module(_ef)
BARS_180M = _ef.BARS_180M
BARS_60M = _ef.BARS_60M
MANDATORY_SQUARE_OFF_BAR = _ef.MANDATORY_SQUARE_OFF_BAR
PARENT_PROTOCOL = _ef.PARENT_PROTOCOL
_align_entry = _ef._align_entry
_pnl = _ef._pnl
_session_bar_number = _ef._session_bar_number
replay_policy = _ef.replay_policy

PROTOCOL_ID = "RP-608-GATE2X-003"
IST = timezone(timedelta(hours=5, minutes=30))
POLICIES = ("A", "C", "E", "F_path")


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def load_jsonl(path: Path) -> List[Dict[str, Any]]:
    return [json.loads(l) for l in path.open() if l.strip()]


def day_ist(ms: Any) -> str:
    try:
        return (
            datetime.fromtimestamp(int(ms) / 1000, tz=timezone.utc)
            .astimezone(IST)
            .strftime("%Y-%m-%d")
        )
    except Exception:
        return "UNKNOWN"


def stats(nets: List[float]) -> Dict[str, Any]:
    n = len(nets)
    wins = sum(1 for x in nets if x > 0)
    losses = sum(1 for x in nets if x < 0)
    avg_w = (sum(x for x in nets if x > 0) / wins) if wins else 0.0
    avg_l = (sum(abs(x) for x in nets if x < 0) / losses) if losses else 0.0
    payoff = (avg_w / avg_l) if avg_l > 0 else None
    return {
        "n": n,
        "W": wins,
        "L": losses,
        "wr_pct": round(100.0 * wins / n, 2) if n else None,
        "net": round(sum(nets), 4),
        "payoff": round(payoff, 4) if payoff is not None else None,
    }


def replay_f_path(trade: Dict[str, Any], bars: List[Dict[str, Any]]) -> Dict[str, Any]:
    """A CLOSE_GRID with friction-aware continue at 60m green."""
    entry = float(trade["entry_price"])
    qty = float(trade["quantity"])
    half = float(trade.get("half_spread_bps") or 2.5)
    stop = float(trade["stop_loss_price"])
    tp = float(trade["target_price"])
    entry_bar_stamped = int(trade["entry_bar_idx"])
    entry_ts = int(trade["entry_timestamp_utc_ms"])
    e_idx, align = _align_entry(bars, entry_bar_stamped, entry_ts)
    base = {
        "policy": "F_path",
        "trade_id": trade.get("trade_id"),
        "symbol": trade.get("symbol"),
        "bar_align_method": align,
        "path_mode": "CLOSE_GRID",
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
                }
            )
            return out

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

        if bars_held >= BARS_60M and close > entry:
            trial = _pnl(entry, close, qty, half)
            if float(trial["net_pnl_inr"]) > 0:
                out = {**base, **trial}
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
            # F_path: post-friction red green → continue
            continue

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
        }
    )
    return out


def paired(control: List[Dict[str, Any]], chall: List[Dict[str, Any]]) -> Dict[str, Any]:
    c_day: Dict[str, float] = defaultdict(float)
    x_day: Dict[str, float] = defaultdict(float)
    for r in control:
        if r.get("reconstructible") and r.get("session_ist") != "UNKNOWN":
            c_day[str(r["session_ist"])] += float(r["net_pnl_inr"])
    for r in chall:
        if r.get("reconstructible") and r.get("session_ist") != "UNKNOWN":
            x_day[str(r["session_ist"])] += float(r["net_pnl_inr"])
    days = sorted(set(c_day) | set(x_day))
    ok = sum(1 for d in days if x_day.get(d, 0.0) >= c_day.get(d, 0.0))
    return {
        "n_sessions": len(days),
        "delta_nonneg": ok,
        "share": round(ok / len(days), 4) if days else None,
        "sum_delta": round(sum(x_day.get(d, 0.0) - c_day.get(d, 0.0) for d in days), 4),
    }


def promote(a: Dict[str, Any], x: Dict[str, Any], paired_info: Dict[str, Any]) -> Tuple[bool, List[str]]:
    fails = []
    if x["net"] <= a["net"]:
        fails.append("net_not_better_than_A")
    share = paired_info.get("share") or 0
    n_sess = paired_info.get("n_sessions") or 0
    need = 0.6 if n_sess >= 10 else 0.5
    if share < need:
        fails.append(f"session_share_{share}_lt_{need}")
    if n_sess < 3:
        fails.append("n_sessions_lt_3")
    return len(fails) == 0, fails


def main() -> int:
    ap = argparse.ArgumentParser(description=PROTOCOL_ID)
    ap.add_argument(
        "--journal",
        type=Path,
        default=WORKSPACE / "live_capture/shadow_validation/nse/rp608_nse_trade_journal.jsonl",
    )
    ap.add_argument(
        "--bar-history",
        type=Path,
        default=WORKSPACE
        / "artifacts/rp608_nse_kite_hist_bars/rp608_nse_bar_history_backfill.json",
    )
    ap.add_argument(
        "--out-dir",
        type=Path,
        default=WORKSPACE / "artifacts/rp608_gate2x_003",
    )
    args = ap.parse_args()
    out_dir = args.out_dir.resolve()
    out_dir.mkdir(parents=True, exist_ok=True)

    journal = args.journal.resolve()
    bars_path = args.bar_history.resolve()
    rows = load_jsonl(journal)
    raw = [
        r
        for r in rows
        if str(r.get("protocol_id") or PARENT_PROTOCOL) == PARENT_PROTOCOL
        and str(r.get("side") or "LONG").upper() == "LONG"
        and r.get("stop_loss_price") is not None
        and r.get("target_price") is not None
        and r.get("entry_bar_idx") is not None
        and r.get("entry_timestamp_utc_ms") is not None
        and r.get("entry_price") is not None
        and r.get("quantity") is not None
    ]
    by_tid: Dict[str, List[Dict[str, Any]]] = defaultdict(list)
    for r in raw:
        by_tid[str(r.get("trade_id"))].append(r)
    trades = []
    for tid, group in sorted(by_tid.items()):
        group_sorted = sorted(group, key=lambda r: int(r.get("exit_timestamp_utc_ms") or 0))
        trades.append(group_sorted[0])

    bars_by_sym: Dict[str, List[Dict[str, Any]]] = json.loads(bars_path.read_text())
    print(f"[gate2x003] trades={len(trades)} symbols_bars={len(bars_by_sym)}", flush=True)

    by_policy: Dict[str, List[Dict[str, Any]]] = {p: [] for p in POLICIES}
    n_recon = 0
    for i, t in enumerate(trades):
        if i and i % 200 == 0:
            print(f"[gate2x003] replayed {i}/{len(trades)}", flush=True)
        sym = str(t.get("symbol") or "")
        bars = bars_by_sym.get(sym) or []
        sess = day_ist(t.get("entry_timestamp_utc_ms"))
        for pol in ("A", "C", "E"):
            out = replay_policy(t, bars, pol)
            out["session_ist"] = sess
            by_policy[pol].append(out)
        out_f = replay_f_path(t, bars)
        out_f["session_ist"] = sess
        by_policy["F_path"].append(out_f)
        if out_f.get("reconstructible"):
            n_recon += 1

    summary: Dict[str, Any] = {}
    a_rows = [r for r in by_policy["A"] if r.get("reconstructible")]
    a_stats = stats([float(r["net_pnl_inr"]) for r in a_rows])
    summary["A"] = {
        "stats": a_stats,
        "n_reconstructible": len(a_rows),
        "exit_reasons": dict(Counter(r.get("exit_reason") for r in a_rows)),
    }

    dispositions = {}
    for pol in ("C", "E", "F_path"):
        rows_p = [r for r in by_policy[pol] if r.get("reconstructible")]
        st = stats([float(r["net_pnl_inr"]) for r in rows_p])
        pr = paired(a_rows, rows_p)
        ok, fails = promote(a_stats, st, pr)
        dispositions[pol] = {
            "advance": ok,
            "fails": fails,
            "stats": st,
            "delta_net": round(st["net"] - a_stats["net"], 4),
            "paired": pr,
            "exit_reasons": dict(Counter(r.get("exit_reason") for r in rows_p)),
        }
        label = "ADVANCE_CANDIDATE" if ok else "DO_NOT_ADVANCE"
        dispositions[pol]["disposition"] = label

    overall = "DO_NOT_ADVANCE"
    adv = [p for p, d in dispositions.items() if d["advance"]]
    if adv:
        overall = "ADVANCE_CANDIDATE:" + ",".join(adv)

    report = {
        "protocol_id": PROTOCOL_ID,
        "generated_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "authority_pct": 0.0,
        "journal_path": str(journal),
        "journal_sha256": sha256_file(journal),
        "bar_history_path": str(bars_path),
        "bar_history_sha256": sha256_file(bars_path),
        "n_trades": len(trades),
        "n_reconstructible_F_path": n_recon,
        "summary_A": summary["A"],
        "challengers": dispositions,
        "disposition": overall,
        "non_goals": ["no_live_edit", "no_OF_change", "no_traj15_change"],
    }

    json_path = out_dir / "gate2x_003_nse_a_close_grid_fpath_results.json"
    md_path = out_dir / "gate2x_003_nse_a_close_grid_fpath_results.md"
    json_path.write_text(json.dumps(report, indent=2) + "\n")

    lines = [
        f"# {PROTOCOL_ID} results",
        "",
        f"**Disposition:** `{overall}` · authority 0%",
        f"**n trades:** {len(trades)} · reconstructible F_path: {n_recon}",
        "",
        "## Control A",
        f"- net {a_stats['net']} · WR {a_stats['wr_pct']}% · payoff {a_stats['payoff']} · W/L {a_stats['W']}/{a_stats['L']}",
        f"- exits: `{summary['A']['exit_reasons']}`",
        "",
        "## Challengers",
        "",
        "| Policy | disposition | net | Δnet | WR% | payoff | paired share |",
        "| --- | --- | ---: | ---: | ---: | ---: | ---: |",
    ]
    for pol in ("C", "E", "F_path"):
        d = dispositions[pol]
        st = d["stats"]
        lines.append(
            f"| {pol} | {d['disposition']} | {st['net']} | {d['delta_net']} | {st['wr_pct']} | {st['payoff']} | {d['paired'].get('share')} |"
        )
    lines.extend(["", "## Notes", "", "- CLOSE_GRID on Kite 15m hist backfill (not ticks).", "- No live daemon change.", ""])
    md_path.write_text("\n".join(lines) + "\n")

    print(json.dumps({"disposition": overall, "out": str(out_dir), "A_net": a_stats["net"], "challengers": {p: dispositions[p]["disposition"] for p in dispositions}}, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
