#!/usr/bin/env python3
"""
RP-608-GATE2X-002 — NSE A journal-anchored OHLC ambiguous → TP.

Frozen entries. Production journal read-only. Authority 0%.
Policies predeclared in docs/research/rp608_gate2x_002_nse_a_ohlc_ambiguous_scoped.md.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from collections import defaultdict
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional

PROTOCOL_ID = "RP-608-GATE2X-002"
IST = timezone(timedelta(hours=5, minutes=30))
HOLD_INSPECTED_DAY = "2026-11-16"


def _sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def _ts_ist(ms: Any) -> Optional[datetime]:
    try:
        return datetime.fromtimestamp(int(ms) / 1000, tz=timezone.utc).astimezone(IST)
    except Exception:
        return None


def _day_ist(ms: Any) -> str:
    dt = _ts_ist(ms)
    return dt.strftime("%Y-%m-%d") if dt else "UNKNOWN"


def _fric_total(fric: Any) -> float:
    if isinstance(fric, dict):
        return float(fric.get("total_friction") or 0.0)
    try:
        return float(fric or 0.0)
    except (TypeError, ValueError):
        return 0.0


def _load_jsonl(path: Path) -> List[Dict[str, Any]]:
    out: List[Dict[str, Any]] = []
    with path.open() as f:
        for line in f:
            if line.strip():
                out.append(json.loads(line))
    return out


def _is_ambiguous_ohlc_sl(t: Dict[str, Any]) -> bool:
    if t.get("exit_path") != "BAR_OHLC_PATH":
        return False
    if t.get("exit_reason") != "STOP_LOSS":
        return False
    try:
        ep = float(t.get("entry_price") or 0)
        sl = float(t.get("stop_loss_price") or 0)
        tp = float(t.get("target_price") or 0)
        mfe_p = float(t.get("mfe_price") or ep)
        mae_p = float(t.get("mae_price") or ep)
    except (TypeError, ValueError):
        return False
    if ep <= 0 or sl <= 0 or tp <= 0:
        return False
    return mae_p <= sl and mfe_p >= tp


def _net_g_ohlc(t: Dict[str, Any]) -> float:
    """Production net, or TP recomputed net when ambiguous OHLC SL."""
    prod = float(t.get("net_pnl_inr") or 0.0)
    if not _is_ambiguous_ohlc_sl(t):
        return prod
    ep = float(t["entry_price"])
    tp = float(t["target_price"])
    qty = float(t.get("quantity") or 0)
    if qty <= 0:
        return prod
    gross = (tp - ep) * qty
    return gross - _fric_total(t.get("friction"))


def _net_g0(t: Dict[str, Any]) -> float:
    prod = float(t.get("net_pnl_inr") or 0.0)
    if _is_ambiguous_ohlc_sl(t):
        return 0.0
    return prod


def _summarize(nets: List[float]) -> Dict[str, Any]:
    n = len(nets)
    wins = sum(1 for x in nets if x > 0)
    losses = sum(1 for x in nets if x < 0)
    avg_w = (sum(x for x in nets if x > 0) / wins) if wins else None
    avg_l = (sum(abs(x) for x in nets if x < 0) / losses) if losses else None
    payoff = (avg_w / avg_l) if (avg_w is not None and avg_l and avg_l > 0) else None
    total = sum(nets)
    return {
        "n": n,
        "wins": wins,
        "losses": losses,
        "W/L": f"{wins}/{n}",
        "WR_pct": round(100.0 * wins / n, 2) if n else None,
        "payoff_net": None if payoff is None else round(payoff, 3),
        "net": round(total, 2),
        "expectancy": round(total / n, 2) if n else None,
    }


def _session_tables(
    rows: List[Dict[str, Any]],
) -> Dict[str, Any]:
    by_day: Dict[str, List[Dict[str, Any]]] = defaultdict(list)
    for t in rows:
        by_day[_day_ist(t.get("entry_timestamp_utc_ms"))].append(t)

    sessions = []
    for day in sorted(by_day):
        xs = by_day[day]
        a_nets = [float(t.get("net_pnl_inr") or 0.0) for t in xs]
        g_nets = [_net_g_ohlc(t) for t in xs]
        g0_nets = [_net_g0(t) for t in xs]
        flipped = sum(1 for t in xs if _is_ambiguous_ohlc_sl(t))
        a = _summarize(a_nets)
        g = _summarize(g_nets)
        g0 = _summarize(g0_nets)
        d_net = round(g["net"] - a["net"], 2)
        sessions.append(
            {
                "session_ist": day,
                "n": len(xs),
                "n_flipped": flipped,
                "A": a,
                "G_ohlc": g,
                "G0": g0,
                "delta_net_G_minus_A": d_net,
                "delta_nonneg": d_net >= 0,
            }
        )
    return {"n_sessions": len(sessions), "sessions": sessions}


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--journal",
        type=Path,
        default=Path("live_capture/shadow_validation/nse/rp608_nse_trade_journal.jsonl"),
    )
    ap.add_argument(
        "--out-dir",
        type=Path,
        default=Path("artifacts/rp608_gate2x_002"),
    )
    args = ap.parse_args()

    rows = _load_jsonl(args.journal)
    # Estimand A closed trades only (journal is A lifecycle).
    rows = [t for t in rows if t.get("entry_timestamp_utc_ms") and t.get("exit_timestamp_utc_ms")]

    a_all = _summarize([float(t.get("net_pnl_inr") or 0.0) for t in rows])
    g_all = _summarize([_net_g_ohlc(t) for t in rows])
    g0_all = _summarize([_net_g0(t) for t in rows])
    n_flip = sum(1 for t in rows if _is_ambiguous_ohlc_sl(t))

    sess = _session_tables(rows)
    sessions = sess["sessions"]
    discovery = [s for s in sessions if s["session_ist"] != HOLD_INSPECTED_DAY]
    hold = [s for s in sessions if s["session_ist"] == HOLD_INSPECTED_DAY]

    def _robust(slist: List[Dict[str, Any]]) -> Dict[str, Any]:
        if not slist:
            return {"n_sessions": 0, "n_delta_nonneg": 0, "share_nonneg": None, "sum_delta_net": 0.0}
        nn = sum(1 for s in slist if s["delta_nonneg"])
        return {
            "n_sessions": len(slist),
            "n_delta_nonneg": nn,
            "share_nonneg": round(nn / len(slist), 3),
            "sum_delta_net": round(sum(s["delta_net_G_minus_A"] for s in slist), 2),
            "sum_net_A": round(sum(s["A"]["net"] for s in slist), 2),
            "sum_net_G": round(sum(s["G_ohlc"]["net"] for s in slist), 2),
        }

    disc_r = _robust(discovery)
    hold_r = _robust(hold)
    all_r = _robust(sessions)

    # ADVANCE rule on discovery sessions (exclude previously inspected day).
    n_sess = disc_r["n_sessions"]
    need = 0.5 if n_sess < 10 else 0.6
    share = disc_r["share_nonneg"] or 0.0
    beats_net = (disc_r["sum_net_G"] - disc_r["sum_net_A"]) > 0
    advance = bool(n_sess >= 3 and beats_net and share >= need and (g_all["net"] > a_all["net"]))

    disposition = "ADVANCE_CANDIDATE" if advance else "DO_NOT_ADVANCE"
    # G0 never advances
    note = []
    if not beats_net:
        note.append("discovery sum net G does not beat A")
    if share < need:
        note.append(f"session robustness share {share} < required {need}")
    if n_sess < 3:
        note.append("fewer than 3 discovery sessions")

    out = {
        "protocol_id": PROTOCOL_ID,
        "authority_pct": 0.0,
        "journal_path": str(args.journal),
        "journal_sha256": _sha256_file(args.journal) if args.journal.exists() else None,
        "n_trades": len(rows),
        "n_flipped_eligible": n_flip,
        "hold_inspected_day": HOLD_INSPECTED_DAY,
        "full_sample": {"A": a_all, "G_ohlc": g_all, "G0": g0_all, "delta_net_G_minus_A": round(g_all["net"] - a_all["net"], 2)},
        "discovery_ex_inspected_day": disc_r,
        "holdout_inspected_day": hold_r,
        "all_sessions_robustness": all_r,
        "advance_rule": {
            "exclude_day_from_discovery_sign": HOLD_INSPECTED_DAY,
            "min_sessions": 3,
            "share_nonneg_required": need,
            "primary": "net",
        },
        "disposition_G_ohlc": disposition,
        "disposition_G0": "DIAGNOSTIC_ONLY",
        "disposition_notes": note,
        "sessions": sessions,
    }

    args.out_dir.mkdir(parents=True, exist_ok=True)
    json_path = args.out_dir / "gate2x_002_nse_a_ohlc_ambiguous_results.json"
    json_path.write_text(json.dumps(out, indent=2) + "\n")

    md_path = args.out_dir / "gate2x_002_nse_a_ohlc_ambiguous_results.md"
    lines = [
        f"# {PROTOCOL_ID} results",
        "",
        f"**Disposition G_ohlc:** `{disposition}` · G0: DIAGNOSTIC_ONLY · authority 0%",
        f"**n:** {len(rows)} · flipped eligible: {n_flip}",
        f"**Journal sha256:** `{out['journal_sha256']}`",
        "",
        "## Full sample",
        f"- A: W/L {a_all['W/L']} · net {a_all['net']} · E={a_all['expectancy']}",
        f"- G_ohlc: W/L {g_all['W/L']} · net {g_all['net']} · E={g_all['expectancy']} · Δnet {out['full_sample']['delta_net_G_minus_A']}",
        f"- G0: W/L {g0_all['W/L']} · net {g0_all['net']} (cannot ADVANCE)",
        "",
        f"## Discovery (exclude {HOLD_INSPECTED_DAY})",
        f"- sessions={disc_r['n_sessions']} · Δnet≥0 in {disc_r['n_delta_nonneg']} ({disc_r['share_nonneg']}) · ΣΔnet={disc_r['sum_delta_net']}",
        f"- Σnet A={disc_r.get('sum_net_A')} · Σnet G={disc_r.get('sum_net_G')}",
        "",
        f"## Hold / previously inspected ({HOLD_INSPECTED_DAY})",
        f"- sessions={hold_r['n_sessions']} · ΣΔnet={hold_r['sum_delta_net']} · Σnet A={hold_r.get('sum_net_A')} · Σnet G={hold_r.get('sum_net_G')}",
        "",
        "## Per session Δnet(G−A)",
    ]
    for s in sessions:
        lines.append(
            f"- {s['session_ist']}: n={s['n']} flip={s['n_flipped']} "
            f"A_net={s['A']['net']} G_net={s['G_ohlc']['net']} Δ={s['delta_net_G_minus_A']} "
            f"WL_A={s['A']['W/L']} WL_G={s['G_ohlc']['W/L']}"
        )
    lines.extend(
        [
            "",
            "## Notes",
            f"- Advance notes: {note or ['passed discovery robustness + net']}",
            "- No live daemon change. Gate 5 required before production OHLC order change.",
            "",
        ]
    )
    md_path.write_text("\n".join(lines))
    print(json.dumps({k: out[k] for k in out if k != "sessions"}, indent=2))
    print("WROTE", json_path)
    print("WROTE", md_path)


if __name__ == "__main__":
    main()
