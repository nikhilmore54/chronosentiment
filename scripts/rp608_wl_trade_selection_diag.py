#!/usr/bin/env python3
"""
RP-608-WL-IMPROVE-001 Phase 1 — multi-session trade-selection diagnostic.

Books never pooled. Baseline frozen. Authority 0%.
Descriptive only — not causal filters, not execution changes.
"""

from __future__ import annotations

import argparse
import json
import math
from collections import defaultdict
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

# Prefer explicit fields; gaps labeled UNAVAILABLE in report.
ENTRY_CONDITION_KEYS = [
    "exit_reason",  # not entry — excluded in builder
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


def entry_dt(r: Dict[str, Any]) -> Optional[datetime]:
    return pdt(
        r.get("entry_utc")
        or r.get("entry_time_ms")
        or r.get("paper_open_timestamp")
        or r.get("activated_ms")
        or r.get("opened_ms")
        or r.get("entry_ms")
    )


def direction_of(r: Dict[str, Any]) -> str:
    d = str(r.get("side") or r.get("direction") or "").upper()
    if "SHORT" in d:
        return "SHORT"
    if "LONG" in d:
        return "LONG"
    return "UNKNOWN"


def entry_condition_of(r: Dict[str, Any], book: str) -> str:
    """Best-effort entry-condition label from recorded fields (descriptive)."""
    if book == "nse_estimand_a":
        # Estimand A exports lack rich entry-condition tags; use entry_bar bucket.
        bar = r.get("entry_bar_idx")
        if bar is None:
            return "ENTRY_COND_UNAVAILABLE"
        try:
            b = int(bar)
        except Exception:
            return "ENTRY_COND_UNAVAILABLE"
        if b <= 2:
            return "ENTRY_BAR_EARLY_≤2"
        if b <= 8:
            return "ENTRY_BAR_MID_3_8"
        if b <= 16:
            return "ENTRY_BAR_LATE_9_16"
        return "ENTRY_BAR_VERY_LATE_≥17"
    if book in ("nse_of_paper", "crypto_of_paper"):
        imb = fnum(r, "order_flow_imbalance", "order_flow_imbalance_entry", "last_book_imbalance")
        label = str(r.get("order_flow_label") or "").upper() or "OF_LABEL_MISSING"
        if imb is None:
            return f"OF|{label}|IMB_NA"
        # Bucket imbalance magnitude (sign follows direction separately)
        a = abs(imb)
        if a < 0.30:
            mag = "IMB_<0.30"
        elif a < 0.50:
            mag = "IMB_0.30_0.50"
        elif a < 0.70:
            mag = "IMB_0.50_0.70"
        else:
            mag = "IMB_≥0.70"
        return f"OF|{label}|{mag}"
    if book == "crypto_recommendation_plan":
        of_lab = str(r.get("order_flow_label") or "OF_NONE")
        of_sup = r.get("order_flow_supported")
        hz = str(r.get("horizon_id") or r.get("horizon_label") or "HZ_NA")
        dq = str(r.get("data_quality_status") or "DQ_NA")
        return f"PLAN|{hz}|OF_SUP={of_sup}|{of_lab}|{dq}"
    return "ENTRY_COND_UNAVAILABLE"


def wl_table(rows: Sequence[Dict[str, Any]], key_fn, *, min_n: int = 1) -> List[Dict[str, Any]]:
    buckets: Dict[str, List[float]] = defaultdict(list)
    for r in rows:
        n = r["net"]
        buckets[str(key_fn(r))].append(n)
    out = []
    for k, nets in buckets.items():
        w = sum(1 for x in nets if x > 0)
        l = sum(1 for x in nets if x < 0)
        be = sum(1 for x in nets if x == 0)
        nn = len(nets)
        wr = round(100.0 * w / nn, 2) if nn else None
        wins = [x for x in nets if x > 0]
        losses = [x for x in nets if x < 0]
        aw = sum(wins) / len(wins) if wins else None
        al = sum(losses) / len(losses) if losses else None
        payoff = (abs(aw / al) if aw is not None and al not in (None, 0) else None)
        # Wilson-ish crude uncertainty: flag small n
        small = nn < 10
        out.append(
            {
                "key": k,
                "n": nn,
                "wins": w,
                "losses": l,
                "breakeven": be,
                "win_rate_pct": wr,
                "net_sum": round(sum(nets), 4),
                "avg_win": None if aw is None else round(aw, 4),
                "avg_loss": None if al is None else round(al, 4),
                "payoff": None if payoff is None else round(payoff, 4),
                "small_n": small,
            }
        )
    out.sort(key=lambda r: (r["win_rate_pct"] is None, r["win_rate_pct"] if r["win_rate_pct"] is not None else 999, -r["n"]))
    return [r for r in out if r["n"] >= min_n]


def enrich(rows: List[Dict[str, Any]], book: str, ccy: str, day: str) -> List[Dict[str, Any]]:
    out = []
    for r in rows:
        n = net_of(r, ccy)
        if n is None:
            continue
        edt = entry_dt(r)
        out.append(
            {
                "day": day,
                "net": n,
                "win": n > 0,
                "loss": n < 0,
                "symbol": str(r.get("symbol") or r.get("tradingsymbol") or "?"),
                "direction": direction_of(r),
                "entry_hour_ist": edt.astimezone(IST).strftime("%H") if edt else "??",
                "entry_condition": entry_condition_of(r, book),
                "exit_reason": str(r.get("exit_reason") or r.get("exit_rule") or "UNKNOWN"),
            }
        )
    return out


def session_summary(rows: Sequence[Dict[str, Any]]) -> Dict[str, Any]:
    n = len(rows)
    w = sum(1 for r in rows if r["win"])
    l = sum(1 for r in rows if r["loss"])
    return {
        "n": n,
        "wins": w,
        "losses": l,
        "win_rate_pct": None if not n else round(100.0 * w / n, 2),
        "net_sum": round(sum(r["net"] for r in rows), 4),
    }


def persistence(by_day: Dict[str, List[Dict[str, Any]]], dim: str, *, min_sessions: int = 2, min_n_per: int = 3) -> List[Dict[str, Any]]:
    """Keys that appear weak (WR<40) or strong (WR>=55) in ≥min_sessions with enough n."""
    # Collect per-day tables
    day_maps: Dict[str, Dict[str, Dict[str, Any]]] = {}
    for day, rows in by_day.items():
        tab = wl_table(rows, lambda r: r[dim], min_n=1)
        day_maps[day] = {t["key"]: t for t in tab}

    keys = set()
    for m in day_maps.values():
        keys |= set(m.keys())

    out = []
    for k in keys:
        sessions = []
        for day, m in sorted(day_maps.items()):
            t = m.get(k)
            if t and t["n"] >= min_n_per:
                sessions.append({"day": day, **t})
        if len(sessions) < min_sessions:
            continue
        wrs = [s["win_rate_pct"] for s in sessions if s["win_rate_pct"] is not None]
        if not wrs:
            continue
        weak = sum(1 for wr in wrs if wr < 40.0)
        strong = sum(1 for wr in wrs if wr >= 55.0)
        label = None
        if weak >= min_sessions:
            label = "PERSISTENT_WEAK_WR"
        elif strong >= min_sessions:
            label = "PERSISTENT_STRONG_WR"
        if label:
            out.append(
                {
                    "key": k,
                    "label": label,
                    "n_sessions": len(sessions),
                    "win_rates": wrs,
                    "mean_wr": round(sum(wrs) / len(wrs), 2),
                    "sessions": sessions,
                }
            )
    out.sort(key=lambda r: (0 if r["label"].startswith("PERSISTENT_WEAK") else 1, r["mean_wr"], -r["n_sessions"]))
    return out


def analyze_book(export_dir: Path, book: str, title: str, ccy: str, days: Sequence[str]) -> Dict[str, Any]:
    by_day: Dict[str, List[Dict[str, Any]]] = {}
    all_rows: List[Dict[str, Any]] = []
    for day in days:
        path = export_dir / f"session_export_{day}_{book}_closed.jsonl"
        raw = load_jsonl(path)
        rows = enrich(raw, book, ccy, day)
        by_day[day] = rows
        all_rows.extend(rows)

    return {
        "book": book,
        "title": title,
        "currency": ccy,
        "days": list(days),
        "overall": session_summary(all_rows),
        "by_session": {d: session_summary(rs) for d, rs in by_day.items()},
        "wl_by_entry_condition": wl_table(all_rows, lambda r: r["entry_condition"], min_n=1)[:40],
        "wl_by_symbol": wl_table(all_rows, lambda r: r["symbol"], min_n=3)[:40],
        "wl_by_entry_hour_ist": wl_table(all_rows, lambda r: r["entry_hour_ist"], min_n=1),
        "wl_by_direction": wl_table(all_rows, lambda r: r["direction"], min_n=1),
        "persistence_entry_condition": persistence(by_day, "entry_condition"),
        "persistence_symbol": persistence(by_day, "symbol", min_n_per=2)[:30],
        "persistence_entry_hour": persistence(by_day, "entry_hour_ist", min_n_per=5),
        "persistence_direction": persistence(by_day, "direction", min_n_per=5),
        "notes": {
            "entry_condition_definition": {
                "nse_estimand_a": "entry_bar_idx buckets (rich entry tags unavailable in export)",
                "nse_of_paper": "order_flow_label + |imbalance| buckets",
                "crypto_of_paper": "order_flow_label + |imbalance| buckets",
                "crypto_recommendation_plan": "horizon + OF supported/label + data_quality",
            },
            "not_causal": True,
            "not_filter_authorization": True,
        },
    }


def render_md(payload: Dict[str, Any]) -> str:
    lines = [
        "# RP-608 W/L Trade-Selection Diagnostic — 2026-11-03 → 2026-11-05",
        "",
        "**Protocol:** `RP-608-WL-IMPROVE-001` Phase 1",
        "**Authority:** 0.0% · books never pooled · baseline frozen",
        "**Purpose:** Identify recurring losing-entry patterns vs winners across sessions.",
        "**Not authorized:** OF threshold / stop / target / universal filters / Stage 2",
        "",
        "Small-n rows (n<10) are flagged — do not promote to filters without persistence + Gate 3.",
        "",
    ]
    for book, title, _ in BOOKS:
        b = payload["books"][book]
        o = b["overall"]
        lines += [
            f"## {title}",
            "",
            f"Overall (3 sessions pooled **for display only**; never for causal pooling): "
            f"n={o['n']} · W/L {o['wins']}/{o['losses']} · WR **{o['win_rate_pct']}%** · net {o['net_sum']}",
            "",
            "### By session",
            "",
            "| Day | n | W/L | WR% | Net |",
            "| --- | ---: | --- | ---: | ---: |",
        ]
        for d, s in b["by_session"].items():
            lines.append(f"| {d} | {s['n']} | {s['wins']}/{s['losses']} | {s['win_rate_pct']} | {s['net_sum']} |")

        def dump_table(name: str, rows: List[Dict[str, Any]], limit: int = 15) -> None:
            lines.append("")
            lines.append(f"### {name}")
            lines.append("")
            lines.append("| Key | n | W | L | WR% | Payoff | Net | small_n |")
            lines.append("| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |")
            for r in rows[:limit]:
                lines.append(
                    f"| `{r['key']}` | {r['n']} | {r['wins']} | {r['losses']} | {r['win_rate_pct']} | "
                    f"{r['payoff']} | {r['net_sum']} | {r['small_n']} |"
                )

        dump_table("W/L by entry condition", b["wl_by_entry_condition"])
        dump_table("W/L by symbol (n≥3)", b["wl_by_symbol"])
        dump_table("W/L by entry hour IST", b["wl_by_entry_hour_ist"])
        dump_table("W/L by direction", b["wl_by_direction"])

        lines.append("")
        lines.append("### Multi-session persistence (descriptive)")
        lines.append("")
        pers = b["persistence_entry_condition"][:12] + b["persistence_entry_hour"][:8] + b["persistence_symbol"][:12]
        if not pers:
            lines.append("_No persistent weak/strong keys under current thresholds._")
        else:
            lines.append("| Dim key | Label | Sessions | Mean WR | Session WRs |")
            lines.append("| --- | --- | ---: | ---: | --- |")
            for p in pers:
                lines.append(
                    f"| `{p['key']}` | {p['label']} | {p['n_sessions']} | {p['mean_wr']} | {p['win_rates']} |"
                )
        lines.append("")

    lines += [
        "## Cross-book note",
        "",
        "Do not pool books for a shared corrective. Crypto plan WR failure ≠ NSE A payoff failure.",
        "Next: Phase 2 entry-quality at decision time for the highest-urgency book (Crypto plan), "
        "only after Gate 1 review of this diagnostic.",
        "",
        "## Governance",
        "",
        "Baseline frozen · authority 0% · Step 3+ held · Gate R unreached · OFX Stage 2 held · Gate 3 experiment not authorized.",
        "",
    ]
    return "\n".join(lines)


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--export-dir", default="artifacts/rp608_econ_edge_diag")
    ap.add_argument("--days", nargs="+", default=["2026-11-03", "2026-11-04", "2026-11-05"])
    args = ap.parse_args()
    export_dir = Path(args.export_dir)
    books_out = {}
    for book, title, ccy in BOOKS:
        books_out[book] = analyze_book(export_dir, book, title, ccy, args.days)
    payload = {
        "protocol": "RP-608-WL-IMPROVE-001",
        "phase": 1,
        "authority_pct": 0.0,
        "days": args.days,
        "books": books_out,
    }
    stem = export_dir / "wl_trade_selection_diag_2026-11-03_to_05"
    stem.with_suffix(".json").write_text(json.dumps(payload, indent=2, default=str), encoding="utf-8")
    stem.with_suffix(".md").write_text(render_md(payload), encoding="utf-8")
    # brief stdout
    summary = {
        k: {
            "overall_wr": books_out[k]["overall"]["win_rate_pct"],
            "n": books_out[k]["overall"]["n"],
            "persistent_weak_hours": [
                p["key"] for p in books_out[k]["persistence_entry_hour"] if p["label"].startswith("PERSISTENT_WEAK")
            ][:5],
            "persistent_weak_conds": [
                p["key"] for p in books_out[k]["persistence_entry_condition"] if p["label"].startswith("PERSISTENT_WEAK")
            ][:5],
        }
        for k, _, _ in BOOKS
    }
    print(json.dumps(summary, indent=2))
    print(f"WROTE {stem}.md")


if __name__ == "__main__":
    main()
