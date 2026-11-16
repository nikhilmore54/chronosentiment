#!/usr/bin/env python3
"""
RP-608-ECON-EDGE-DIAG-001 — authoritative same-day four-book session export.

Books never pooled. Open marks distinct from realized.
Does not change production rules. Authority 0%.
"""

from __future__ import annotations

import argparse
import json
import math
from collections import Counter
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any, Dict, Iterable, List, Optional, Sequence, Tuple

IST = timezone(timedelta(hours=5, minutes=30))


def pdt(v: Any) -> Optional[datetime]:
    if v is None:
        return None
    if isinstance(v, (int, float)):
        ms = int(v)
        if ms > 10_000_000_000:
            ms //= 1000
        try:
            return datetime.fromtimestamp(ms, tz=timezone.utc)
        except (OSError, OverflowError, ValueError):
            return None
    try:
        dt = datetime.fromisoformat(str(v).replace("Z", "+00:00"))
        if dt.tzinfo is None:
            dt = dt.replace(tzinfo=timezone.utc)
        return dt
    except Exception:
        return None


def on_day(dt: Optional[datetime], day: str) -> bool:
    return dt is not None and dt.astimezone(IST).date().isoformat() == day


def load_jsonl(path: Path) -> List[Dict[str, Any]]:
    if not path.exists():
        return []
    out: List[Dict[str, Any]] = []
    for line in path.read_text(encoding="utf-8", errors="ignore").splitlines():
        if not line.strip():
            continue
        try:
            out.append(json.loads(line))
        except Exception:
            continue
    return out


def fnum(r: Dict[str, Any], *keys: str) -> Optional[float]:
    for k in keys:
        if r.get(k) is None:
            continue
        try:
            return float(r[k])
        except (TypeError, ValueError):
            continue
    return None


def quantiles(xs: Sequence[float]) -> Dict[str, float]:
    if not xs:
        return {}
    s = sorted(xs)

    def q(p: float) -> float:
        return s[int(round((len(s) - 1) * p))]

    return {
        "min": s[0],
        "p25": q(0.25),
        "p50": q(0.5),
        "p75": q(0.75),
        "max": s[-1],
        "mean": sum(s) / len(s),
    }


def pf(xs: Sequence[float]) -> Optional[Any]:
    gains = sum(x for x in xs if x > 0)
    losses = -sum(x for x in xs if x < 0)
    if losses <= 0:
        return None if gains <= 0 else "inf"
    return round(gains / losses, 4)


def summarize_closed(
    rows: List[Dict[str, Any]],
    *,
    currency: str,
    net_keys: Sequence[str],
    gross_keys: Sequence[str],
    friction_keys: Sequence[str],
    mae_keys: Sequence[str],
    mfe_keys: Sequence[str],
    hold_keys: Sequence[str],
    entry_keys: Sequence[str],
    side_key: str,
    reason_key: str,
) -> Dict[str, Any]:
    nets: List[float] = []
    gross: List[float] = []
    fric: List[float] = []
    mae: List[float] = []
    mfe: List[float] = []
    holds: List[float] = []
    hours: Counter = Counter()
    reasons: Counter = Counter()
    sides: Counter = Counter()
    symbols: Counter = Counter()

    for r in rows:
        n = fnum(r, *net_keys)
        g = fnum(r, *gross_keys)
        fr = fnum(r, *friction_keys)
        if fr is None and g is not None and n is not None:
            fr = g - n
        if n is not None:
            nets.append(n)
        if g is not None:
            gross.append(g)
        if fr is not None:
            fric.append(fr)
        a = fnum(r, *mae_keys)
        b = fnum(r, *mfe_keys)
        if a is not None:
            mae.append(abs(a))
        if b is not None:
            mfe.append(abs(b))
        h = fnum(r, *hold_keys)
        if h is not None:
            holds.append(h)
        et = None
        for k in entry_keys:
            et = pdt(r.get(k))
            if et:
                break
        if et:
            hours[et.astimezone(IST).strftime("%H")] += 1
        reasons[str(r.get(reason_key) or r.get("close_reason") or "?")] += 1
        sides[str(r.get(side_key) or r.get("direction") or "?").upper()] += 1
        symbols[str(r.get("symbol") or r.get("tradingsymbol") or "?")] += 1

    # concentration: top symbols by |net|
    by_sym: Dict[str, float] = {}
    for r in rows:
        n = fnum(r, *net_keys) or 0.0
        sym = str(r.get("symbol") or r.get("tradingsymbol") or "?")
        by_sym[sym] = by_sym.get(sym, 0.0) + n
    top_loss = sorted(by_sym.items(), key=lambda kv: kv[1])[:8]
    top_win = sorted(by_sym.items(), key=lambda kv: kv[1], reverse=True)[:5]
    loss_sum = sum(x for x in nets if x < 0)
    top3_loss_share = None
    if loss_sum < 0:
        top3 = sum(v for _, v in sorted(by_sym.items(), key=lambda kv: kv[1])[:3] if v < 0)
        top3_loss_share = round(top3 / loss_sum, 4)

    return {
        "currency": currency,
        "n_closed": len(rows),
        "realized_net_sum": None if not nets else round(sum(nets), 4),
        "realized_gross_sum": None if not gross else round(sum(gross), 4),
        "friction_sum": None if not fric else round(sum(fric), 4),
        "win_rate_pct": None if not nets else round(100.0 * sum(1 for x in nets if x > 0) / len(nets), 2),
        "n_wins": sum(1 for x in nets if x > 0),
        "n_losses": sum(1 for x in nets if x <= 0),
        "expectancy_net": None if not nets else round(sum(nets) / len(nets), 4),
        "profit_factor": None if not nets else pf(nets),
        "sides": dict(sides),
        "exit_reasons": dict(reasons),
        "entry_hour_ist": dict(sorted(hours.items())),
        "mae": {k: round(v, 4) for k, v in quantiles(mae).items()},
        "mfe": {k: round(v, 4) for k, v in quantiles(mfe).items()},
        "hold": {k: round(v, 4) for k, v in quantiles(holds).items()},
        "symbol_count": len(symbols),
        "top_symbols_by_count": dict(symbols.most_common(10)),
        "top_symbols_by_net_loss": [{ "symbol": s, "net": round(v, 4)} for s, v in top_loss],
        "top_symbols_by_net_win": [{ "symbol": s, "net": round(v, 4)} for s, v in top_win],
        "top3_symbols_share_of_total_losses": top3_loss_share,
    }


def normalize_nse_a(r: Dict[str, Any]) -> Dict[str, Any]:
    out = dict(r)
    fr = out.get("friction")
    if isinstance(fr, dict) and out.get("friction_inr") is None:
        out["friction_inr"] = fr.get("total_friction") or fr.get("total_friction_inr")
    if out.get("friction_inr") is None and out.get("gross_pnl_inr") is not None and out.get("net_pnl_inr") is not None:
        out["friction_inr"] = float(out["gross_pnl_inr"]) - float(out["net_pnl_inr"])
    if out.get("hold_bars") is None and out.get("entry_bar_idx") is not None and out.get("exit_bar_idx") is not None:
        try:
            out["hold_bars"] = float(out["exit_bar_idx"]) - float(out["entry_bar_idx"])
        except Exception:
            pass
    return out


def normalize_crypto_of(r: Dict[str, Any]) -> Dict[str, Any]:
    out = dict(r)
    if out.get("friction_usdt") is None and out.get("gross_pnl_usdt") is not None and out.get("net_pnl_usdt") is not None:
        out["friction_usdt"] = float(out["gross_pnl_usdt"]) - float(out["net_pnl_usdt"])
    return out


def normalize_nse_of(r: Dict[str, Any]) -> Dict[str, Any]:
    out = dict(r)
    if out.get("gross_pnl_inr") is None:
        out["gross_pnl_inr"] = fnum(out, "gross_pnl", "touch_gross_pnl_inr")
    if out.get("net_pnl_inr") is None:
        out["net_pnl_inr"] = fnum(out, "net_pnl")
    if out.get("friction_inr") is None:
        fr = fnum(out, "statutory_overhead_inr", "fees_inr", "total_friction_inr", "friction_usdt")
        if fr is None and out.get("gross_pnl_inr") is not None and out.get("net_pnl_inr") is not None:
            fr = float(out["gross_pnl_inr"]) - float(out["net_pnl_inr"])
        out["friction_inr"] = fr
    return out


def normalize_plan(r: Dict[str, Any]) -> Dict[str, Any]:
    out = dict(r)
    if out.get("friction_usdt") is None and out.get("gross_pnl_usdt") is not None and out.get("net_pnl_usdt") is not None:
        out["friction_usdt"] = float(out["gross_pnl_usdt"]) - float(out["net_pnl_usdt"])
    if out.get("holding_duration_seconds") is None:
        a, b = pdt(out.get("entry_time_ms")), pdt(out.get("exit_time_ms"))
        if a and b:
            out["holding_duration_seconds"] = (b - a).total_seconds()
    if not out.get("symbol"):
        out["symbol"] = "BTCUSDT"
    return out


def export_row_nse_a(r: Dict[str, Any]) -> Dict[str, Any]:
    et, xt = pdt(r.get("entry_timestamp_utc_ms")), pdt(r.get("exit_timestamp_utc_ms"))
    return {
        "trade_id": r.get("trade_id"),
        "symbol": r.get("symbol"),
        "side": r.get("side"),
        "entry_utc": None if not et else et.isoformat().replace("+00:00", "Z"),
        "exit_utc": None if not xt else xt.isoformat().replace("+00:00", "Z"),
        "entry_bar_idx": r.get("entry_bar_idx"),
        "exit_bar_idx": r.get("exit_bar_idx"),
        "entry_price": r.get("entry_price"),
        "exit_price": r.get("exit_price"),
        "quantity": r.get("quantity"),
        "gross_pnl_inr": r.get("gross_pnl_inr"),
        "friction_inr": r.get("friction_inr"),
        "net_pnl_inr": r.get("net_pnl_inr"),
        "net_return_bps": r.get("net_return_bps"),
        "mae_bps": r.get("mae_bps"),
        "mfe_bps": r.get("mfe_bps"),
        "exit_reason": r.get("exit_reason"),
        "hold_bars": r.get("hold_bars"),
        "stop_loss_price": r.get("stop_loss_price"),
        "target_price": r.get("target_price"),
    }


def build_export(*, root: Path, day: str) -> Dict[str, Any]:
    nse_root = root / "live_capture/shadow_validation/nse"
    crypto_root = root / "live_capture/shadow_validation/crypto/trade_plan_overlay_v1"

    # NSE A closed
    nse_a = [normalize_nse_a(r) for r in load_jsonl(nse_root / "rp608_nse_trade_journal.jsonl")]
    nse_a_closed = []
    for r in nse_a:
        if r.get("exit_price") is None:
            continue
        if on_day(pdt(r.get("exit_timestamp_utc_ms")), day):
            nse_a_closed.append(r)
    nse_a_closed = list({str(r.get("trade_id")): r for r in nse_a_closed}.values())

    # NSE A open
    pend = {}
    pp = nse_root / "rp608_nse_pending_evaluations.json"
    if pp.exists():
        pend = json.loads(pp.read_text())
    active = pend.get("active_trades") or {}
    evals = {}
    ep = nse_root / "rp608_nse_latest_evaluations.json"
    if ep.exists():
        evals = (json.loads(ep.read_text()).get("evaluations") or {})
    open_a = []
    mtm = 0.0
    for sym, t in active.items():
        if not isinstance(t, dict):
            continue
        et = pdt(t.get("entry_timestamp_utc_ms"))
        if not on_day(et, day):
            continue
        e = evals.get(sym)
        e = e[-1] if isinstance(e, list) and e else e
        px = (e or {}).get("reference_price") if isinstance(e, dict) else None
        entry, qty = t.get("entry_price"), t.get("quantity")
        unreal = None
        if px is not None and entry is not None and qty is not None:
            unreal = (float(px) - float(entry)) * float(qty)
            mtm += unreal
        open_a.append(
            {
                "symbol": sym,
                "side": t.get("side") or "LONG",
                "entry_utc": None if not et else et.isoformat().replace("+00:00", "Z"),
                "entry_bar_idx": t.get("entry_bar_idx"),
                "entry_price": entry,
                "quantity": qty,
                "mark_price": px,
                "open_gross_mtm_inr": None if unreal is None else round(unreal, 4),
                "note": "gross MTM before exit friction; not realized",
            }
        )

    # NSE OF
    of_paths = [nse_root / "of_paper_ledger_v1/of_trade_outcomes.jsonl"]
    of_rows = []
    for p in of_paths:
        for r in load_jsonl(p):
            if on_day(pdt(r.get("paper_close_timestamp") or r.get("exited_ms") or r.get("exit_time_ms")), day):
                of_rows.append(normalize_nse_of(r))
    of_rows = list({str(r.get("of_trade_id")): r for r in of_rows}.values())
    of_open = {}
    op = nse_root / "of_paper_ledger_v1/of_paper_trades_open.json"
    if op.exists():
        of_open = json.loads(op.read_text())

    # Crypto OF (live + any archives under root)
    couts = []
    c_paths = [crypto_root / "of_trade_outcomes.jsonl"]
    arch = root / "crypto_all_arch"
    if arch.exists():
        c_paths.extend(sorted(arch.glob("**/of_trade_outcomes.jsonl")))
    arch2 = root / "crypto_archives"
    if arch2.exists():
        c_paths.extend(sorted(arch2.glob("**/of_trade_outcomes.jsonl")))
    for p in c_paths:
        for r in load_jsonl(p):
            if on_day(pdt(r.get("paper_close_timestamp") or r.get("exited_ms")), day):
                couts.append(normalize_crypto_of(r))
    couts = list({str(r.get("of_trade_id")): r for r in couts}.values())
    crypto_open = {}
    cp = crypto_root / "of_paper_trades_open.json"
    if cp.exists():
        crypto_open = json.loads(cp.read_text())

    # Crypto plan
    plans = []
    for r in load_jsonl(crypto_root / "trade_plan_outcomes.jsonl"):
        if on_day(pdt(r.get("exit_time_ms")), day):
            plans.append(normalize_plan(r))
    plans = list({str(r.get("plan_id")): r for r in plans}.values())

    nse_a_summary = summarize_closed(
        nse_a_closed,
        currency="INR",
        net_keys=["net_pnl_inr"],
        gross_keys=["gross_pnl_inr"],
        friction_keys=["friction_inr"],
        mae_keys=["mae_bps"],
        mfe_keys=["mfe_bps"],
        hold_keys=["hold_bars"],
        entry_keys=["entry_timestamp_utc_ms"],
        side_key="side",
        reason_key="exit_reason",
    )
    nse_of_summary = summarize_closed(
        of_rows,
        currency="INR",
        net_keys=["net_pnl_inr", "net_pnl"],
        gross_keys=["gross_pnl_inr", "gross_pnl", "touch_gross_pnl_inr"],
        friction_keys=["friction_inr", "statutory_overhead_inr"],
        mae_keys=["mae_bps", "mae"],
        mfe_keys=["mfe_bps", "mfe"],
        hold_keys=["holding_duration_seconds"],
        entry_keys=["paper_open_timestamp", "activated_ms", "entry_time_ms"],
        side_key="direction",
        reason_key="exit_reason",
    )
    crypto_of_summary = summarize_closed(
        couts,
        currency="USDT",
        net_keys=["net_pnl_usdt"],
        gross_keys=["gross_pnl_usdt"],
        friction_keys=["friction_usdt", "fees_usdt"],
        mae_keys=["mae_bps"],
        mfe_keys=["mfe_bps"],
        hold_keys=["holding_duration_seconds"],
        entry_keys=["paper_open_timestamp", "activated_ms"],
        side_key="direction",
        reason_key="exit_reason",
    )
    crypto_plan_summary = summarize_closed(
        plans,
        currency="USDT",
        net_keys=["net_pnl_usdt"],
        gross_keys=["gross_pnl_usdt"],
        friction_keys=["friction_usdt", "est_friction_usdt"],
        mae_keys=["mae_bps"],
        mfe_keys=["mfe_bps"],
        hold_keys=["holding_duration_seconds"],
        entry_keys=["entry_time_ms"],
        side_key="direction",
        reason_key="exit_reason",
    )

    realized_a = nse_a_summary["realized_net_sum"] or 0.0
    return {
        "protocol_id": "RP-608-ECON-EDGE-DIAG-001",
        "export_id": f"session_export_{day}",
        "session_date_ist": day,
        "generated_utc": datetime.now(timezone.utc).isoformat().replace("+00:00", "Z"),
        "live_order_authority": 0.0,
        "method": "docs/research/rp608_econ_edge_session_postmortem_method.md",
        "rules_frozen": True,
        "books_pooled": False,
        "postmortem_questions": [
            "signal_quality",
            "exit_quality",
            "friction",
            "concentration",
        ],
        "non_attribution_rule": (
            "Do not assign the same loss to multiple causes by default. "
            "Reverse-OF exit ≠ automatic OF failure without entry-edge and OF-incremental tests."
        ),
        "books": {
            "nse_estimand_a": {
                "summary_closed": nse_a_summary,
                "open": {
                    "n_open_entered_session": len(open_a),
                    "open_gross_mtm_inr": round(mtm, 4),
                    "realized_net_plus_open_gross_mtm_inr": round(realized_a + mtm, 4),
                    "note": "Open gross MTM is not realized PnL and excludes exit friction.",
                    "positions": open_a,
                },
                "closed_trades": [export_row_nse_a(r) for r in sorted(
                    nse_a_closed, key=lambda r: r.get("entry_timestamp_utc_ms") or 0
                )],
            },
            "nse_of_paper": {
                "summary_closed": nse_of_summary,
                "open": {
                    "n_open": of_open.get("n_open"),
                    "trades": of_open.get("trades") or of_open.get("open_trades") or [],
                },
                "closed_trades": of_rows,
            },
            "crypto_recommendation_plan": {
                "summary_closed": crypto_plan_summary,
                "closed_trades": plans,
            },
            "crypto_of_paper": {
                "summary_closed": crypto_of_summary,
                "open": {
                    "open": crypto_open.get("open"),
                    "of_trade_id": crypto_open.get("of_trade_id"),
                    "trade": crypto_open.get("trade"),
                },
                "closed_trades": couts,
            },
        },
        "attribution_status": "EXPORT_ONLY_NO_MECHANISM_LABELS_YET",
        "next": "Produce mutually exclusive primary-mechanism labels from this export under the four-question method.",
    }


def to_markdown(rep: Dict[str, Any]) -> str:
    day = rep["session_date_ist"]
    lines = [
        f"# RP-608 Session Export — {day}",
        "",
        f"**Generated:** {rep['generated_utc']}  ",
        "**Authority:** 0.0% · rules frozen · books not pooled  ",
        f"**Method:** `{rep['method']}`  ",
        "",
        "## Postmortem questions",
        "1. Signal quality",
        "2. Exit quality",
        "3. Friction",
        "4. Concentration",
        "",
        f"**Non-attribution rule:** {rep['non_attribution_rule']}",
        "",
        "## Book summaries (closed)",
    ]
    for key, title in [
        ("nse_estimand_a", "NSE Estimand A"),
        ("nse_of_paper", "NSE OF paper"),
        ("crypto_recommendation_plan", "Crypto recommendation / trade-plan"),
        ("crypto_of_paper", "Crypto OF paper"),
    ]:
        s = rep["books"][key]["summary_closed"]
        cur = "₹" if s["currency"] == "INR" else "$"
        lines.append(f"### {title}")
        lines.append(
            f"- n={s['n_closed']} · net {cur}{s['realized_net_sum']} · gross {cur}{s['realized_gross_sum']} · friction {cur}{s['friction_sum']}"
        )
        lines.append(
            f"- W/L {s['n_wins']}/{s['n_losses']} ({s['win_rate_pct']}%) · expectancy {cur}{s['expectancy_net']} · PF {s['profit_factor']}"
        )
        lines.append(f"- exits `{s['exit_reasons']}`")
        lines.append(f"- entry hour IST `{s['entry_hour_ist']}`")
        lines.append(f"- concentration top losses `{s['top_symbols_by_net_loss'][:5]}`")
        lines.append(f"- top3 share of total losses: `{s['top3_symbols_share_of_total_losses']}`")
        lines.append("")
    a_open = rep["books"]["nse_estimand_a"]["open"]
    lines.append("## NSE Estimand A open book")
    lines.append(
        f"- open entered session: **{a_open['n_open_entered_session']}** · open gross MTM **₹{a_open['open_gross_mtm_inr']}**"
    )
    lines.append(
        f"- realized net + open gross MTM: **₹{a_open['realized_net_plus_open_gross_mtm_inr']}** (not a realized result)"
    )
    lines.append("")
    lines.append("## Status")
    lines.append(f"- `{rep['attribution_status']}`")
    lines.append(f"- {rep['next']}")
    lines.append("")
    return "\n".join(lines)


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--day", default="2026-11-03")
    ap.add_argument("--root", type=Path, required=True, help="Extracted journal tree root")
    ap.add_argument("--out-dir", type=Path, required=True)
    args = ap.parse_args()
    args.out_dir.mkdir(parents=True, exist_ok=True)
    rep = build_export(root=args.root, day=args.day)
    (args.out_dir / f"session_export_{args.day}.json").write_text(json.dumps(rep, indent=2, default=str))
    (args.out_dir / f"session_export_{args.day}.md").write_text(to_markdown(rep))
    # also slim event CSVs-ish jsonl per book for downstream attribution
    for key in ("nse_estimand_a", "nse_of_paper", "crypto_recommendation_plan", "crypto_of_paper"):
        rows = rep["books"][key].get("closed_trades") or []
        path = args.out_dir / f"session_export_{args.day}_{key}_closed.jsonl"
        with path.open("w", encoding="utf-8") as f:
            for r in rows:
                f.write(json.dumps(r, default=str) + "\n")
    print(to_markdown(rep))
    print(f"WROTE {args.out_dir}")


if __name__ == "__main__":
    main()
