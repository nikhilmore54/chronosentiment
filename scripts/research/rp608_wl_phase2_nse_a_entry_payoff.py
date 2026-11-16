#!/usr/bin/env python3
"""
RP-608-WL-IMPROVE-001 Phase 2 — NSE Estimand A entry/payoff concentration.

Primary question for this book: why WR can be ~50% while payoff <<1
(winners small / losses large). Feature-join (trend/momentum) UNAVAILABLE
without Nov-2026 bar archive join — labeled explicitly.
Baseline frozen. Authority 0%. Books never pooled.
"""

from __future__ import annotations

import argparse
import json
from collections import defaultdict
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional, Sequence

IST = timezone(timedelta(hours=5, minutes=30))
DAYS = ["2026-11-03", "2026-11-04", "2026-11-05"]


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
    s = str(v).replace("Z", "+00:00")
    try:
        dt = datetime.fromisoformat(s)
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
        if line.strip():
            try:
                out.append(json.loads(line))
            except Exception:
                pass
    return out


def path_bucket(mfe: Optional[float], mae: Optional[float]) -> str:
    if mfe is None:
        return "PATH_UNAVAILABLE"
    if mfe <= 0:
        return "NEVER_FAVORABLE"
    if mfe < 5:
        return "TINY_FAV_<5bps"
    if mfe < 20:
        return "FAV_5_20bps"
    return "FAV_≥20bps"


def hold_bucket(bars: Optional[float]) -> str:
    if bars is None:
        return "HOLD_UNAVAILABLE"
    if bars <= 1:
        return "HOLD_≤1bar"
    if bars <= 2:
        return "HOLD_2bars"
    if bars <= 4:
        return "HOLD_3_4bars"
    return "HOLD_≥5bars"


def enrich(r: Dict[str, Any], day: str) -> Optional[Dict[str, Any]]:
    net = fnum(r, "net_pnl_inr")
    if net is None:
        return None
    mfe = fnum(r, "mfe_bps")
    mae = fnum(r, "mae_bps")
    edt = pdt(r.get("entry_utc") or r.get("entry_time_ms"))
    side = str(r.get("side") or r.get("direction") or "LONG").upper()
    return {
        "day": day,
        "net": net,
        "win": net > 0,
        "direction": side,
        "symbol": str(r.get("symbol") or "?"),
        "entry_hour_ist": edt.astimezone(IST).strftime("%H") if edt else "??",
        "exit_reason": str(r.get("exit_reason") or "?"),
        "entry_bar": str(r.get("entry_bar_idx") if r.get("entry_bar_idx") is not None else "BAR_NA"),
        "hold_bucket": hold_bucket(fnum(r, "hold_bars")),
        "hold_bars": fnum(r, "hold_bars"),
        "path_response": path_bucket(mfe, mae),
        "mfe_bps": mfe,
        "mae_bps": mae,
        "gross": fnum(r, "gross_pnl_inr"),
        "friction": fnum(r, "friction_inr"),
        "feature_join": "FEATURE_JOIN_UNAVAILABLE",
    }


def wl(rows: Sequence[Dict[str, Any]], key: str) -> List[Dict[str, Any]]:
    buckets: Dict[str, List[Dict[str, Any]]] = defaultdict(list)
    for r in rows:
        buckets[str(r[key])].append(r)
    out = []
    for k, rs in buckets.items():
        nets = [r["net"] for r in rs]
        w = sum(1 for x in nets if x > 0)
        l = sum(1 for x in nets if x < 0)
        n = len(nets)
        wins = [x for x in nets if x > 0]
        losses = [x for x in nets if x < 0]
        aw = sum(wins) / len(wins) if wins else None
        al = sum(losses) / len(losses) if losses else None
        payoff = abs(aw / al) if aw is not None and al not in (None, 0) else None
        out.append(
            {
                "key": k,
                "n": n,
                "wins": w,
                "losses": l,
                "win_rate_pct": round(100 * w / n, 2) if n else None,
                "net_sum": round(sum(nets), 2),
                "avg_win": None if aw is None else round(aw, 2),
                "avg_loss": None if al is None else round(al, 2),
                "payoff": None if payoff is None else round(payoff, 4),
                "small_n": n < 15,
            }
        )
    out.sort(key=lambda r: (r["win_rate_pct"] is None, r["win_rate_pct"] or 999, -r["n"]))
    return out


def mean(xs: Sequence[float]) -> Optional[float]:
    return None if not xs else round(sum(xs) / len(xs), 4)


def concentration(rows: Sequence[Dict[str, Any]]) -> Dict[str, Any]:
    losses = sorted([r for r in rows if not r["win"]], key=lambda r: r["net"])
    wins = sorted([r for r in rows if r["win"]], key=lambda r: -r["net"])
    total_loss = sum(r["net"] for r in losses) or -1e-9
    total_win = sum(r["net"] for r in wins) or 1e-9

    def share(subset: Sequence[Dict[str, Any]], total: float, k: int) -> Dict[str, Any]:
        top = subset[:k]
        s = sum(r["net"] for r in top)
        return {
            "k": k,
            "sum": round(s, 2),
            "share_of_book_side_pct": round(100 * s / total, 2),
            "symbols": [r["symbol"] for r in top[:10]],
        }

    return {
        "n_losses": len(losses),
        "n_wins": len(wins),
        "avg_win": mean([r["net"] for r in wins]),
        "avg_loss": mean([r["net"] for r in losses]),
        "payoff_overall": None
        if not wins or not losses
        else round(abs(mean([r["net"] for r in wins]) / mean([r["net"] for r in losses])), 4),  # type: ignore
        "mean_mfe_wins": mean([r["mfe_bps"] for r in wins if r["mfe_bps"] is not None]),
        "mean_mfe_losses": mean([r["mfe_bps"] for r in losses if r["mfe_bps"] is not None]),
        "mean_mae_wins": mean([r["mae_bps"] for r in wins if r["mae_bps"] is not None]),
        "mean_mae_losses": mean([r["mae_bps"] for r in losses if r["mae_bps"] is not None]),
        "top5_loss_concentration": share(losses, total_loss, 5),
        "top10_loss_concentration": share(losses, total_loss, 10),
        "top5_win_concentration": share(wins, total_win, 5),
        "top10_win_concentration": share(wins, total_win, 10),
    }


def derive_leads(payload: Dict[str, Any]) -> List[str]:
    c = payload["concentration"]
    leads = [
        f"Overall payoff {c['payoff_overall']} with avg_win {c['avg_win']} vs avg_loss {c['avg_loss']} — "
        "payoff/concentration is the Estimand A primary research target (WR secondary).",
        f"Top-5 losses = {c['top5_loss_concentration']['share_of_book_side_pct']}% of loss mass "
        f"({c['top5_loss_concentration']['symbols']}).",
        f"Top-5 wins = {c['top5_win_concentration']['share_of_book_side_pct']}% of win mass "
        f"({c['top5_win_concentration']['symbols']}).",
        "Trend/momentum/regime FEATURE_JOIN_UNAVAILABLE (no Nov-2026 local bar join) — not inferred.",
    ]
    by_path = {r["key"]: r for r in payload["by_path"]}
    if "NEVER_FAVORABLE" in by_path:
        nf = by_path["NEVER_FAVORABLE"]
        leads.append(f"Never-favorable path: n={nf['n']} WR {nf['win_rate_pct']}% net {nf['net_sum']}.")
    by_exit = [r for r in payload["by_exit"] if r["n"] >= 20]
    if by_exit:
        worst = min(by_exit, key=lambda r: r["payoff"] if r["payoff"] is not None else 999)
        leads.append(
            f"Exit-reason payoff floor (n≥20): `{worst['key']}` payoff {worst['payoff']} "
            f"WR {worst['win_rate_pct']}% (n={worst['n']})."
        )
    return leads


def render(payload: Dict[str, Any]) -> str:
    c = payload["concentration"]
    lines = [
        "# RP-608-WL-IMPROVE-001 Phase 2 — NSE Estimand A Entry / Payoff",
        "",
        "**Sessions:** 2026-11-03 … 2026-11-05 · **Authority:** 0% · baseline frozen",
        "**Scope:** NSE Estimand A only (never pooled).",
        "**At-entry available:** side, entry bar idx, hour, symbol.",
        "**Path available:** mfe_bps / mae_bps on this export.",
        "**Unavailable:** trend / momentum / regime feature join (FEATURE_JOIN_UNAVAILABLE).",
        "",
        f"Closed n={payload['n']} · W/L {payload['wins']}/{payload['losses']} · WR **{payload['win_rate_pct']}%** · "
        f"payoff **{c['payoff_overall']}**",
        "",
        "## Payoff & concentration",
        "",
        f"| Metric | Value |",
        f"| --- | ---: |",
        f"| Avg win ₹ | {c['avg_win']} |",
        f"| Avg loss ₹ | {c['avg_loss']} |",
        f"| Payoff | {c['payoff_overall']} |",
        f"| Mean MFE bps wins | {c['mean_mfe_wins']} |",
        f"| Mean MFE bps losses | {c['mean_mfe_losses']} |",
        f"| Mean MAE bps wins | {c['mean_mae_wins']} |",
        f"| Mean MAE bps losses | {c['mean_mae_losses']} |",
        f"| Top-5 loss share of loss mass | {c['top5_loss_concentration']['share_of_book_side_pct']}% |",
        f"| Top-10 loss share | {c['top10_loss_concentration']['share_of_book_side_pct']}% |",
        f"| Top-5 win share of win mass | {c['top5_win_concentration']['share_of_book_side_pct']}% |",
        "",
        f"Top-5 loss symbols: `{c['top5_loss_concentration']['symbols']}`",
        f"Top-5 win symbols: `{c['top5_win_concentration']['symbols']}`",
        "",
    ]

    def tbl(title: str, rows: List[Dict[str, Any]]) -> None:
        lines.append(f"## {title}")
        lines.append("")
        lines.append("| Key | n | W | L | WR% | Payoff | Net ₹ | small_n |")
        lines.append("| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |")
        for r in rows[:30]:
            lines.append(
                f"| `{r['key']}` | {r['n']} | {r['wins']} | {r['losses']} | {r['win_rate_pct']} | "
                f"{r['payoff']} | {r['net_sum']} | {r['small_n']} |"
            )
        lines.append("")

    for title, key in [
        ("W/L by path response (MFE)", "by_path"),
        ("W/L by exit reason", "by_exit"),
        ("W/L by hold bars", "by_hold"),
        ("W/L by entry hour IST", "by_hour"),
        ("W/L by entry bar idx", "by_bar"),
        ("W/L by session", "by_day"),
        ("W/L by symbol (n≥8)", "by_symbol"),
    ]:
        rows = payload[key]
        if key == "by_symbol":
            rows = [r for r in rows if r["n"] >= 8]
        tbl(title, rows)

    lines += ["## Candidate leads (NOT Gate 2 / NOT filters)", ""]
    for lead in payload.get("leads", []):
        lines.append(f"- {lead}")
    lines += [
        "",
        "## Governance",
        "",
        "Baseline frozen · authority 0% · Step 3+ held · Gate R unreached · OFX Stage 2 held.",
        "Do not ban symbols from top-loss lists alone (plan forbidden).",
        "",
    ]
    return "\n".join(lines)


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--export-dir", default="artifacts/rp608_econ_edge_diag")
    args = ap.parse_args()
    export_dir = Path(args.export_dir)
    rows: List[Dict[str, Any]] = []
    for day in DAYS:
        path = export_dir / f"session_export_{day}_nse_estimand_a_closed.jsonl"
        for r in load_jsonl(path):
            e = enrich(r, day)
            if e:
                rows.append(e)
    wins = sum(1 for r in rows if r["win"])
    losses = len(rows) - wins
    payload = {
        "protocol": "RP-608-WL-IMPROVE-001",
        "phase": 2,
        "book": "nse_estimand_a",
        "n": len(rows),
        "wins": wins,
        "losses": losses,
        "win_rate_pct": round(100 * wins / len(rows), 2) if rows else None,
        "concentration": concentration(rows),
        "by_path": wl(rows, "path_response"),
        "by_exit": wl(rows, "exit_reason"),
        "by_hold": wl(rows, "hold_bucket"),
        "by_hour": wl(rows, "entry_hour_ist"),
        "by_bar": wl(rows, "entry_bar"),
        "by_day": wl(rows, "day"),
        "by_symbol": wl(rows, "symbol"),
        "by_direction": wl(rows, "direction"),
    }
    payload["leads"] = derive_leads(payload)
    out = export_dir / "wl_phase2_nse_a_entry_payoff_2026-11-03_to_05"
    out.with_suffix(".json").write_text(json.dumps(payload, indent=2, default=str), encoding="utf-8")
    out.with_suffix(".md").write_text(render(payload), encoding="utf-8")
    print(
        json.dumps(
            {
                "n": payload["n"],
                "wr": payload["win_rate_pct"],
                "payoff": payload["concentration"]["payoff_overall"],
                "leads": payload["leads"],
            },
            indent=2,
        )
    )
    print(f"WROTE {out}.md")


if __name__ == "__main__":
    main()
