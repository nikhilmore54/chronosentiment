#!/usr/bin/env python3
"""
RP-608-WL-IMPROVE-001 Phase 2 — NSE OF paper entry-quality diagnostic.

At-entry fields from closed exports. Path MFE/MAE labeled UNAVAILABLE.
Baseline frozen (OF ±0.30 unchanged). Authority 0%. Books never pooled.
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
        if line.strip():
            try:
                out.append(json.loads(line))
            except Exception:
                pass
    return out


def imb_bucket(imb: Optional[float]) -> str:
    if imb is None:
        return "IMB_UNAVAILABLE"
    a = abs(imb)
    if a < 0.40:
        return "ABS_IMB_0.30_0.40"  # entries should be ≥0.30 by rule
    if a < 0.55:
        return "ABS_IMB_0.40_0.55"
    if a < 0.70:
        return "ABS_IMB_0.55_0.70"
    if a < 0.85:
        return "ABS_IMB_0.70_0.85"
    return "ABS_IMB_≥0.85"


def touch_bucket(notional: Optional[float]) -> str:
    if notional is None:
        return "TOUCH_NOTIONAL_UNAVAILABLE"
    if notional < 5_000:
        return "TOUCH_<5k"
    if notional < 15_000:
        return "TOUCH_5k_15k"
    if notional < 50_000:
        return "TOUCH_15k_50k"
    return "TOUCH_≥50k"


def spread_bucket(bps: Optional[float]) -> str:
    if bps is None:
        return "SPREAD_UNAVAILABLE"
    if bps < 2:
        return "HALF_SPREAD_<2bps"
    if bps < 5:
        return "HALF_SPREAD_2_5bps"
    if bps < 10:
        return "HALF_SPREAD_5_10bps"
    return "HALF_SPREAD_≥10bps"


def enrich(r: Dict[str, Any], day: str) -> Optional[Dict[str, Any]]:
    net = fnum(r, "net_pnl_inr", "all_in_net_pnl_inr")
    if net is None:
        return None
    imb = fnum(r, "order_flow_imbalance")
    dq = r.get("depth_quality_entry") or r.get("depth_quality") or {}
    if not isinstance(dq, dict):
        dq = {}
    touch = fnum(dq, "combined_touch_notional_usdt")
    thin = dq.get("thin_opposite_diagnostic")
    edt = pdt(r.get("entry_time_ms") or r.get("paper_open_timestamp"))
    return {
        "day": day,
        "net": net,
        "win": net > 0,
        "direction": str(r.get("direction") or "?").upper(),
        "symbol": str(r.get("tradingsymbol") or r.get("symbol") or "?"),
        "entry_hour_ist": edt.astimezone(IST).strftime("%H") if edt else "??",
        "imb_bucket": imb_bucket(imb),
        "imbalance": imb,
        "abs_imb": None if imb is None else abs(imb),
        "of_supported": bool(r.get("order_flow_supported")),
        "of_label": str(r.get("order_flow_label") or "OF_NONE"),
        "touch_bucket": touch_bucket(touch),
        "touch_notional": touch,
        "thin_opposite": bool(thin) if thin is not None else None,
        "spread_bucket": spread_bucket(fnum(r, "half_spread_bps")),
        "half_spread_bps": fnum(r, "half_spread_bps"),
        "exit_reason": str(r.get("exit_reason") or r.get("order_flow_exit_label") or "?"),
        "path_response": "PATH_UNAVAILABLE",  # NSE OF exports lack MFE/MAE
        "holding_s": fnum(r, "holding_duration_seconds"),
    }


def wl(rows: Sequence[Dict[str, Any]], key: str) -> List[Dict[str, Any]]:
    buckets: Dict[str, List[float]] = defaultdict(list)
    for r in rows:
        buckets[str(r[key])].append(r["net"])
    out = []
    for k, nets in buckets.items():
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
    return None if not xs else round(sum(xs) / len(xs), 6)


def contrast(rows: Sequence[Dict[str, Any]]) -> Dict[str, Any]:
    wins = [r for r in rows if r["win"]]
    losses = [r for r in rows if not r["win"]]

    def dist(subset: Sequence[Dict[str, Any]], field: str) -> Dict[str, int]:
        c: Dict[str, int] = defaultdict(int)
        for r in subset:
            c[str(r[field])] += 1
        return dict(sorted(c.items(), key=lambda kv: -kv[1])[:12])

    return {
        "n_wins": len(wins),
        "n_losses": len(losses),
        "mean_abs_imb_wins": mean([r["abs_imb"] for r in wins if r["abs_imb"] is not None]),
        "mean_abs_imb_losses": mean([r["abs_imb"] for r in losses if r["abs_imb"] is not None]),
        "mean_touch_wins": mean([r["touch_notional"] for r in wins if r["touch_notional"] is not None]),
        "mean_touch_losses": mean([r["touch_notional"] for r in losses if r["touch_notional"] is not None]),
        "mean_spread_wins": mean([r["half_spread_bps"] for r in wins if r["half_spread_bps"] is not None]),
        "mean_spread_losses": mean([r["half_spread_bps"] for r in losses if r["half_spread_bps"] is not None]),
        "mean_hold_wins": mean([r["holding_s"] for r in wins if r["holding_s"] is not None]),
        "mean_hold_losses": mean([r["holding_s"] for r in losses if r["holding_s"] is not None]),
        "imb_wins": dist(wins, "imb_bucket"),
        "imb_losses": dist(losses, "imb_bucket"),
        "dir_wins": dist(wins, "direction"),
        "dir_losses": dist(losses, "direction"),
        "thin_wins": dist(wins, "thin_opposite"),
        "thin_losses": dist(losses, "thin_opposite"),
        "path_note": "PATH_UNAVAILABLE for all NSE OF rows in this export (no MFE/MAE)",
    }


def derive_leads(payload: Dict[str, Any]) -> List[str]:
    leads = [
        "Path MFE/MAE UNAVAILABLE on NSE OF export — cannot separate never-favorable entry vs exit-path failure here."
    ]
    by_dir = {r["key"]: r for r in payload["by_direction"]}
    if "LONG" in by_dir and "SHORT" in by_dir:
        lg, sh = by_dir["LONG"], by_dir["SHORT"]
        if lg["n"] >= 30 and sh["n"] >= 30:
            leads.append(
                f"Direction: LONG WR {lg['win_rate_pct']}% (n={lg['n']}) vs SHORT WR {sh['win_rate_pct']}% "
                f"(n={sh['n']}) — descriptive; not a filter."
            )
    imbs = [r for r in payload["by_imb"] if r["n"] >= 20 and "UNAVAILABLE" not in r["key"]]
    if len(imbs) >= 2:
        best = max(imbs, key=lambda r: r["win_rate_pct"] or -1)
        worst = min(imbs, key=lambda r: r["win_rate_pct"] or 999)
        leads.append(
            f"Abs-imbalance strata: best `{best['key']}` WR {best['win_rate_pct']}% (n={best['n']}) vs "
            f"worst `{worst['key']}` WR {worst['win_rate_pct']}% (n={worst['n']}) — "
            "NOT an OF ±0.30 change proposal (`.cursor/rules/rp608-ofx.mdc` / baseline freeze)."
        )
    touches = [r for r in payload["by_touch"] if r["n"] >= 20 and "UNAVAILABLE" not in r["key"]]
    if len(touches) >= 2:
        best = max(touches, key=lambda r: r["win_rate_pct"] or -1)
        worst = min(touches, key=lambda r: r["win_rate_pct"] or 999)
        leads.append(
            f"Touch notional: `{best['key']}` WR {best['win_rate_pct']}% vs `{worst['key']}` WR {worst['win_rate_pct']}%."
        )
    hours = [r for r in payload["by_hour"] if r["n"] >= 20]
    if hours:
        best = max(hours, key=lambda r: r["win_rate_pct"] or -1)
        worst = min(hours, key=lambda r: r["win_rate_pct"] or 999)
        if (best["win_rate_pct"] or 0) - (worst["win_rate_pct"] or 0) >= 10:
            leads.append(
                f"Hour: best `{best['key']}` WR {best['win_rate_pct']}% (n={best['n']}) vs "
                f"worst `{worst['key']}` WR {worst['win_rate_pct']}% (n={worst['n']})."
            )
    return leads


def render(payload: Dict[str, Any]) -> str:
    c = payload["contrast"]
    lines = [
        "# RP-608-WL-IMPROVE-001 Phase 2 — NSE OF Entry Quality",
        "",
        "**Sessions:** 2026-11-03 … 2026-11-05 · **Authority:** 0% · baseline frozen (OF ±0.30)",
        "**Scope:** NSE OF paper book only (never pooled with Estimand A / crypto).",
        "**At-entry fields:** signed imbalance, direction, hour, depth touch notional, thin-opposite flag, half-spread.",
        "**Post-entry path:** **PATH_UNAVAILABLE** (no MFE/MAE on this export) — Phase 3 path split blocked for this book until remediated.",
        "",
        f"Closed n={payload['n']} · W/L {payload['wins']}/{payload['losses']} · WR **{payload['win_rate_pct']}%**",
        "",
        "## Winner vs loser contrast (descriptive)",
        "",
        f"| Metric | Wins (n={c['n_wins']}) | Losses (n={c['n_losses']}) |",
        "| --- | ---: | ---: |",
        f"| Mean abs OF imb | {c['mean_abs_imb_wins']} | {c['mean_abs_imb_losses']} |",
        f"| Mean touch notional | {c['mean_touch_wins']} | {c['mean_touch_losses']} |",
        f"| Mean half-spread bps | {c['mean_spread_wins']} | {c['mean_spread_losses']} |",
        f"| Mean hold seconds | {c['mean_hold_wins']} | {c['mean_hold_losses']} |",
        "",
        f"- Imb buckets — wins: `{c['imb_wins']}` · losses: `{c['imb_losses']}`",
        f"- Direction — wins: `{c['dir_wins']}` · losses: `{c['dir_losses']}`",
        f"- Thin opposite — wins: `{c['thin_wins']}` · losses: `{c['thin_losses']}`",
        f"- Path: {c['path_note']}",
        "",
    ]

    def tbl(title: str, rows: List[Dict[str, Any]]) -> None:
        lines.append(f"## {title}")
        lines.append("")
        lines.append("| Key | n | W | L | WR% | Payoff | Net ₹ | small_n |")
        lines.append("| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |")
        for r in rows[:25]:
            lines.append(
                f"| `{r['key']}` | {r['n']} | {r['wins']} | {r['losses']} | {r['win_rate_pct']} | "
                f"{r['payoff']} | {r['net_sum']} | {r['small_n']} |"
            )
        lines.append("")

    for title, key in [
        ("W/L by abs imbalance strata (at entry)", "by_imb"),
        ("W/L by direction", "by_direction"),
        ("W/L by touch notional (depth_quality_entry)", "by_touch"),
        ("W/L by thin-opposite diagnostic", "by_thin"),
        ("W/L by half-spread", "by_spread"),
        ("W/L by entry hour IST", "by_hour"),
        ("W/L by exit reason / label", "by_exit"),
        ("W/L by session", "by_day"),
    ]:
        tbl(title, payload[key])

    lines += ["## Candidate leads (NOT Gate 2 / NOT filters)", ""]
    for lead in payload.get("leads", []):
        lines.append(f"- {lead}")
    lines += [
        "",
        "## Governance",
        "",
        "Baseline frozen · authority 0% · do not change OF ±0.30 · Step 3+ held · Gate R unreached · OFX Stage 2 held.",
        "Gate 1 PASS (narrow) only. Gate 2 not locked from this book alone.",
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
        path = export_dir / f"session_export_{day}_nse_of_paper_closed.jsonl"
        for r in load_jsonl(path):
            e = enrich(r, day)
            if e:
                rows.append(e)
    wins = sum(1 for r in rows if r["win"])
    losses = len(rows) - wins
    # thin as string key for wl
    for r in rows:
        r["thin_key"] = "THIN" if r["thin_opposite"] is True else ("NOT_THIN" if r["thin_opposite"] is False else "THIN_NA")
    payload = {
        "protocol": "RP-608-WL-IMPROVE-001",
        "phase": 2,
        "book": "nse_of_paper",
        "n": len(rows),
        "wins": wins,
        "losses": losses,
        "win_rate_pct": round(100 * wins / len(rows), 2) if rows else None,
        "contrast": contrast(rows),
        "by_imb": wl(rows, "imb_bucket"),
        "by_direction": wl(rows, "direction"),
        "by_touch": wl(rows, "touch_bucket"),
        "by_thin": wl(rows, "thin_key"),
        "by_spread": wl(rows, "spread_bucket"),
        "by_hour": wl(rows, "entry_hour_ist"),
        "by_exit": wl(rows, "exit_reason"),
        "by_day": wl(rows, "day"),
        "by_session": {
            d: {
                "n": sum(1 for r in rows if r["day"] == d),
                "wins": sum(1 for r in rows if r["day"] == d and r["win"]),
                "losses": sum(1 for r in rows if r["day"] == d and not r["win"]),
            }
            for d in DAYS
        },
    }
    payload["leads"] = derive_leads(payload)
    out = export_dir / "wl_phase2_nse_of_entry_quality_2026-11-03_to_05"
    out.with_suffix(".json").write_text(json.dumps(payload, indent=2, default=str), encoding="utf-8")
    out.with_suffix(".md").write_text(render(payload), encoding="utf-8")
    print(json.dumps({"n": payload["n"], "wr": payload["win_rate_pct"], "leads": payload["leads"]}, indent=2))
    print(f"WROTE {out}.md")


if __name__ == "__main__":
    main()
