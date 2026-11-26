#!/usr/bin/env python3
"""
RP-608 same-day W/L-first attribution (four books separate).

Win rate is the lead diagnostic; gross/friction/net support it.
Does not change rules. Authority 0%. No Stage 2 / OF threshold implications.
"""

from __future__ import annotations

import argparse
import json
from collections import Counter, defaultdict
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional, Sequence, Tuple

IST = timezone(timedelta(hours=5, minutes=30))

BOOKS = [
    ("nse_estimand_a", "NSE Estimand A", "INR"),
    ("nse_of_paper", "NSE OF paper", "INR"),
    ("crypto_recommendation_plan", "Crypto plan", "USDT"),
    ("crypto_of_paper", "Crypto OF paper", "USDT"),
]


def fnum(r: Dict[str, Any], *keys: str) -> Optional[float]:
    for k in keys:
        if r.get(k) is None:
            continue
        try:
            return float(r[k])
        except (TypeError, ValueError):
            continue
    return None


def pdt(v: Any) -> Optional[datetime]:
    if v is None:
        return None
    if isinstance(v, (int, float)):
        ms = int(v)
        if ms > 10_000_000_000:
            ms //= 1000
        try:
            return datetime.fromtimestamp(ms, tz=timezone.utc)
        except Exception:
            return None
    try:
        dt = datetime.fromisoformat(str(v).replace("Z", "+00:00"))
        if dt.tzinfo is None:
            dt = dt.replace(tzinfo=timezone.utc)
        return dt
    except Exception:
        return None


def load_jsonl(path: Path) -> List[Dict[str, Any]]:
    if not path.exists():
        return []
    out = []
    for line in path.read_text(encoding="utf-8", errors="ignore").splitlines():
        if not line.strip():
            continue
        try:
            out.append(json.loads(line))
        except Exception:
            continue
    return out


def net_of(r: Dict[str, Any], ccy: str) -> Optional[float]:
    if ccy == "INR":
        return fnum(r, "net_pnl_inr", "all_in_net_pnl_inr", "net_pnl")
    return fnum(r, "net_pnl_usdt", "net_pnl")


def gross_of(r: Dict[str, Any], ccy: str) -> Optional[float]:
    if ccy == "INR":
        return fnum(r, "gross_pnl_inr", "touch_gross_pnl_inr", "gross_pnl")
    return fnum(r, "gross_pnl_usdt", "gross_pnl")


def friction_of(r: Dict[str, Any], ccy: str) -> Optional[float]:
    if ccy == "INR":
        fr = fnum(r, "friction_inr", "friction")
    else:
        fr = fnum(r, "friction_usdt", "est_friction_usdt", "friction")
    if fr is not None:
        return fr
    g, n = gross_of(r, ccy), net_of(r, ccy)
    if g is not None and n is not None:
        return g - n
    return None


def entry_dt(r: Dict[str, Any]) -> Optional[datetime]:
    return pdt(r.get("entry_utc") or r.get("entry_time_ms") or r.get("opened_ms") or r.get("entry_ms"))


def mfe_mae(r: Dict[str, Any]) -> Tuple[Optional[float], Optional[float], str]:
    """Return (mfe, mae, unit_hint). Prefer bps; else exec-price excursions."""
    mfe = fnum(r, "mfe_bps")
    mae = fnum(r, "mae_bps")
    if mfe is not None or mae is not None:
        return mfe, mae, "bps"
    # crypto plan exec prices vs entry
    entry = fnum(r, "entry_price")
    mfe_px = fnum(r, "mfe_exec_price")
    mae_px = fnum(r, "mae_exec_price")
    side = str(r.get("side") or r.get("direction") or "").upper()
    if entry and entry > 0 and (mfe_px is not None or mae_px is not None):
        # convert to signed favorable bps if possible
        def fav_bps(px: Optional[float]) -> Optional[float]:
            if px is None:
                return None
            if "SHORT" in side:
                return (entry - px) / entry * 10_000.0
            return (px - entry) / entry * 10_000.0

        return fav_bps(mfe_px), (None if mae_px is None else abs(fav_bps(mae_px) or 0.0)), "bps_from_exec_price"
    return None, None, "unavailable"


def path_bucket(mfe: Optional[float], mae: Optional[float], net: float) -> str:
    """Descriptive entry-path bucket — not causal."""
    if mfe is None:
        return "PATH_UNAVAILABLE"
    if net > 0:
        return "WIN"
    # losing trade
    if mfe <= 0:
        return "LOSS_ENTRY_PATH_NEVER_FAVORABLE"
    return "LOSS_HAD_FAVORABLE_EXCURSION"


def max_consecutive_losses(nets: Sequence[float]) -> int:
    best = cur = 0
    for n in nets:
        if n <= 0:
            cur += 1
            best = max(best, cur)
        else:
            cur = 0
    return best


def avg(xs: Sequence[float]) -> Optional[float]:
    return None if not xs else round(sum(xs) / len(xs), 6)


def analyze_book(rows: List[Dict[str, Any]], *, ccy: str) -> Dict[str, Any]:
    enriched = []
    for r in rows:
        n = net_of(r, ccy)
        if n is None:
            continue
        g = gross_of(r, ccy)
        fr = friction_of(r, ccy)
        mfe, mae, munit = mfe_mae(r)
        edt = entry_dt(r)
        hour = edt.astimezone(IST).strftime("%H") if edt else "??"
        exit_reason = str(r.get("exit_reason") or r.get("exit_rule") or "UNKNOWN")
        sym = str(r.get("symbol") or r.get("tradingsymbol") or "?")
        bucket = path_bucket(mfe, mae, n)
        enriched.append(
            {
                "net": n,
                "gross": g,
                "friction": fr,
                "mfe": mfe,
                "mae": mae,
                "path_unit": munit,
                "path_bucket": bucket,
                "exit_reason": exit_reason,
                "symbol": sym,
                "entry_hour_ist": hour,
                "win": n > 0,
                "flat": n == 0,
                "loss": n < 0,
            }
        )

    n = len(enriched)
    wins = [e for e in enriched if e["win"]]
    losses = [e for e in enriched if e["loss"]]
    flats = [e for e in enriched if e["flat"]]
    nets = [e["net"] for e in enriched]
    win_nets = [e["net"] for e in wins]
    loss_nets = [e["net"] for e in losses]

    by_exit_w = Counter()
    by_exit_l = Counter()
    by_exit_n = Counter()
    for e in enriched:
        by_exit_n[e["exit_reason"]] += 1
        if e["win"]:
            by_exit_w[e["exit_reason"]] += 1
        elif e["loss"]:
            by_exit_l[e["exit_reason"]] += 1

    exit_table = []
    for reason, cnt in by_exit_n.most_common():
        w = by_exit_w[reason]
        l = by_exit_l[reason]
        exit_table.append(
            {
                "exit_reason": reason,
                "n": cnt,
                "wins": w,
                "losses": l,
                "win_rate_pct": round(100.0 * w / cnt, 2) if cnt else None,
            }
        )

    by_hour = defaultdict(lambda: {"n": 0, "wins": 0, "losses": 0})
    for e in enriched:
        h = by_hour[e["entry_hour_ist"]]
        h["n"] += 1
        h["wins"] += int(e["win"])
        h["losses"] += int(e["loss"])
    hour_table = []
    for hour in sorted(by_hour.keys()):
        h = by_hour[hour]
        hour_table.append(
            {
                "entry_hour_ist": hour,
                "n": h["n"],
                "wins": h["wins"],
                "losses": h["losses"],
                "win_rate_pct": round(100.0 * h["wins"] / h["n"], 2) if h["n"] else None,
            }
        )

    by_sym = defaultdict(lambda: {"n": 0, "wins": 0, "losses": 0, "net": 0.0})
    for e in enriched:
        s = by_sym[e["symbol"]]
        s["n"] += 1
        s["wins"] += int(e["win"])
        s["losses"] += int(e["loss"])
        s["net"] += e["net"]
    worst_wr = sorted(
        (
            {
                "symbol": sym,
                "n": v["n"],
                "wins": v["wins"],
                "losses": v["losses"],
                "win_rate_pct": round(100.0 * v["wins"] / v["n"], 2),
                "net": round(v["net"], 4),
            }
            for sym, v in by_sym.items()
            if v["n"] >= 3
        ),
        key=lambda x: (x["win_rate_pct"], x["net"]),
    )[:8]

    path_counts = Counter(e["path_bucket"] for e in enriched)
    loss_path = Counter(e["path_bucket"] for e in losses)

    # payoff asymmetry
    avg_win = avg(win_nets)
    avg_loss = avg(loss_nets)  # negative
    payoff = None
    if avg_win is not None and avg_loss is not None and avg_loss != 0:
        payoff = round(abs(avg_win / avg_loss), 4)

    gross_sum = sum(e["gross"] or 0.0 for e in enriched)
    fr_sum = sum(e["friction"] or 0.0 for e in enriched)
    net_sum = sum(nets)

    # supporting friction-among-losses
    loss_gross_pos = sum(1 for e in losses if (e["gross"] is not None and e["gross"] >= 0))
    loss_gross_neg = sum(1 for e in losses if (e["gross"] is not None and e["gross"] < 0))

    mfe_loss = [e["mfe"] for e in losses if e["mfe"] is not None]
    mfe_win = [e["mfe"] for e in wins if e["mfe"] is not None]

    return {
        "currency": ccy,
        "n_closed": n,
        "n_wins": len(wins),
        "n_losses": len(losses),
        "n_breakeven": len(flats),
        "win_rate_pct": round(100.0 * len(wins) / n, 2) if n else None,
        "loss_rate_pct": round(100.0 * len(losses) / n, 2) if n else None,
        "max_consecutive_losses": max_consecutive_losses(nets),
        "avg_win_net": avg_win,
        "avg_loss_net": avg_loss,
        "payoff_ratio_abs_avg_win_over_avg_loss": payoff,
        "expectancy_net": avg(nets),
        "realized_gross_sum": round(gross_sum, 4),
        "realized_friction_sum": round(fr_sum, 4),
        "realized_net_sum": round(net_sum, 4),
        "path_buckets_all": dict(path_counts),
        "path_buckets_among_losses": dict(loss_path),
        "loss_with_never_favorable_share": (
            round(loss_path.get("LOSS_ENTRY_PATH_NEVER_FAVORABLE", 0) / len(losses), 4) if losses else None
        ),
        "loss_with_favorable_excursion_share": (
            round(loss_path.get("LOSS_HAD_FAVORABLE_EXCURSION", 0) / len(losses), 4) if losses else None
        ),
        "path_metric": enriched[0]["path_unit"] if enriched else None,
        "mfe_among_wins_mean": avg(mfe_win),
        "mfe_among_losses_mean": avg(mfe_loss),
        "losses_with_gross_ge_0": loss_gross_pos,
        "losses_with_gross_lt_0": loss_gross_neg,
        "wl_by_exit_reason": exit_table,
        "wl_by_entry_hour_ist": hour_table,
        "worst_symbols_by_win_rate_min_n3": worst_wr,
        "non_claims": [
            "win_rate_is_descriptive_not_a_rule_change_trigger",
            "path_buckets_are_not_causal_mechanism_labels",
            "friction_does_not_explain_low_win_rate_by_itself",
        ],
    }


def render_md(day: str, books: Dict[str, Any]) -> str:
    lines = [
        f"# RP-608 W/L-First Session Attribution — {day}",
        "",
        "**Authority:** 0.0% · books never pooled · rules frozen  ",
        "**Priority:** win/loss diagnostics first; gross / friction / net supportive  ",
        "**Not authorized:** threshold/OF changes · Stage 2 classifier · Gate R",
        "",
        "## Session disposition",
        "",
        "Disappointing full-session outcome. Morning open MTM is not a realized result.",
        "W/L weakness and friction are investigated as **separate** questions.",
        "",
    ]
    for key, title, _ccy in BOOKS:
        b = books[key]
        lines += [
            f"## {title}",
            "",
            f"| Metric | Value |",
            f"| --- | ---: |",
            f"| Closed | {b['n_closed']} |",
            f"| W / L / BE | {b['n_wins']} / {b['n_losses']} / {b['n_breakeven']} |",
            f"| **Win rate** | **{b['win_rate_pct']}%** |",
            f"| Loss rate | {b['loss_rate_pct']}% |",
            f"| Max consecutive losses | {b['max_consecutive_losses']} |",
            f"| Avg win net | {b['avg_win_net']} |",
            f"| Avg loss net | {b['avg_loss_net']} |",
            f"| Abs avg-win / abs avg-loss | {b['payoff_ratio_abs_avg_win_over_avg_loss']} |",
            f"| Expectancy net | {b['expectancy_net']} |",
            f"| Gross sum | {b['realized_gross_sum']} |",
            f"| Friction sum | {b['realized_friction_sum']} |",
            f"| Net sum | {b['realized_net_sum']} |",
            f"| Losses with gross≥0 (friction can dominate) | {b['losses_with_gross_ge_0']} |",
            f"| Losses with gross<0 (directionally adverse) | {b['losses_with_gross_lt_0']} |",
            f"| Among losses: never-favorable path share | {b['loss_with_never_favorable_share']} |",
            f"| Among losses: had favorable excursion share | {b['loss_with_favorable_excursion_share']} |",
            f"| Path metric | `{b['path_metric']}` |",
            "",
            "### W/L by exit reason",
            "",
            "| Exit | n | W | L | Win% |",
            "| --- | ---: | ---: | ---: | ---: |",
        ]
        for row in b["wl_by_exit_reason"]:
            lines.append(
                f"| `{row['exit_reason']}` | {row['n']} | {row['wins']} | {row['losses']} | {row['win_rate_pct']} |"
            )
        lines += [
            "",
            "### W/L by entry hour (IST)",
            "",
            "| Hour | n | W | L | Win% |",
            "| --- | ---: | ---: | ---: | ---: |",
        ]
        for row in b["wl_by_entry_hour_ist"]:
            lines.append(
                f"| {row['entry_hour_ist']} | {row['n']} | {row['wins']} | {row['losses']} | {row['win_rate_pct']} |"
            )
        lines += [
            "",
            "### Weakest symbols (n≥3, by win rate then net)",
            "",
            "| Symbol | n | W | L | Win% | Net |",
            "| --- | ---: | ---: | ---: | ---: | ---: |",
        ]
        for row in b["worst_symbols_by_win_rate_min_n3"]:
            lines.append(
                f"| {row['symbol']} | {row['n']} | {row['wins']} | {row['losses']} | {row['win_rate_pct']} | {row['net']} |"
            )
        lines.append("")
        # tentative separation note
        never = b.get("loss_with_never_favorable_share")
        fav = b.get("loss_with_favorable_excursion_share")
        if never is None and fav is None:
            note = "Path fields unavailable — entry vs exit separation incomplete for this book."
        elif never is not None and never >= 0.5:
            note = (
                "Among losers, ≥50% never showed favorable excursion in available path fields "
                "→ **entry-side weakness is a leading hypothesis** (not proven causal)."
            )
        elif fav is not None and fav >= 0.5:
            note = (
                "Among losers, ≥50% had some favorable excursion before finishing negative "
                "→ **exit/hold policy is a leading hypothesis** (not proven causal)."
            )
        else:
            note = "Loss paths mixed between never-favorable and had-excursion — do not collapse to one cause."
        lines += [f"**Entry vs exit (descriptive):** {note}", ""]

    lines += [
        "## Cross-book note",
        "",
        "Do not pool books. Shared narrative of 'disappointing session' is allowed; shared causal claim is not.",
        "Multi-session W/L consistency requires comparing this export to prior locked session exports (e.g. 3 Nov).",
        "",
        "## Governance",
        "",
        "Rules frozen · authority 0% · Step 3+ held · Gate R unreached · OFX Stage 2 classifier held.",
        "",
    ]
    return "\n".join(lines)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--day", required=True)
    ap.add_argument("--export-dir", type=Path, default=Path("artifacts/rp608_econ_edge_diag"))
    args = ap.parse_args()
    day = args.day
    out_books = {}
    for key, _title, ccy in BOOKS:
        path = args.export_dir / f"session_export_{day}_{key}_closed.jsonl"
        rows = load_jsonl(path)
        out_books[key] = analyze_book(rows, ccy=ccy)
        out_books[key]["source"] = str(path)

    payload = {
        "protocol_id": "RP-608-ECON-EDGE-DIAG-001",
        "phase": "WL_FIRST_SAME_DAY_ATTRIBUTION",
        "day": day,
        "generated_utc": datetime.now(timezone.utc).isoformat(),
        "authority_pct": 0.0,
        "books": out_books,
        "governance": {
            "rule_changes": False,
            "of_threshold_changes": False,
            "stage2_classifier": "HELD",
            "step3_plus": "HELD",
            "gate_r": "UNREACHED",
        },
    }
    json_path = args.export_dir / f"wl_first_attribution_{day}.json"
    md_path = args.export_dir / f"wl_first_attribution_{day}.md"
    json_path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    md_path.write_text(render_md(day, out_books), encoding="utf-8")
    # compact stdout
    summary = {
        k: {
            "n": out_books[k]["n_closed"],
            "W/L": f"{out_books[k]['n_wins']}/{out_books[k]['n_losses']}",
            "win_rate_pct": out_books[k]["win_rate_pct"],
            "max_consec_L": out_books[k]["max_consecutive_losses"],
            "payoff": out_books[k]["payoff_ratio_abs_avg_win_over_avg_loss"],
            "loss_never_fav_share": out_books[k]["loss_with_never_favorable_share"],
            "loss_had_fav_share": out_books[k]["loss_with_favorable_excursion_share"],
            "net": out_books[k]["realized_net_sum"],
        }
        for k, _, _ in BOOKS
    }
    print(json.dumps(summary, indent=2))
    print(f"WROTE {md_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
