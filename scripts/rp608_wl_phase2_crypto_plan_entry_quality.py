#!/usr/bin/env python3
"""
RP-608-WL-IMPROVE-001 Phase 2 — Crypto plan entry-quality (decision-time only).

Narrow question:
  Using information available at the entry decision, which observable
  conditions distinguish trades that subsequently win from trades that lose?

FORBIDDEN as predictors: post-entry MFE/MAE, exit reason/time, later marks.
Baseline frozen. Authority 0%. Books never pooled. Gate 3 unauthorized.
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
DAYS = ["2026-11-03", "2026-11-04", "2026-11-05"]
SMALL_N = 15  # Phase 2 combination floor (stricter than Phase 1)


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


def wilson_low(wins: int, n: int, z: float = 1.96) -> Optional[float]:
    if n <= 0:
        return None
    p = wins / n
    denom = 1 + z * z / n
    centre = p + z * z / (2 * n)
    adj = z * math.sqrt(p * (1 - p) / n + z * z / (4 * n * n))
    return round(100 * max(0.0, (centre - adj) / denom), 2)


def atr_bucket(atr: Optional[float], entry: Optional[float]) -> str:
    if atr is None or entry is None or entry <= 0:
        return "ATR_REL_UNAVAILABLE"
    rel = atr / entry
    if rel < 0.002:
        return "ATR_REL_<20bps"
    if rel < 0.004:
        return "ATR_REL_20_40bps"
    if rel < 0.008:
        return "ATR_REL_40_80bps"
    return "ATR_REL_≥80bps"


def imb_bucket(imb: Optional[float]) -> str:
    if imb is None:
        return "IMB_UNAVAILABLE"
    a = abs(imb)
    if a < 0.30:
        return "ABS_IMB_<0.30"
    if a < 0.50:
        return "ABS_IMB_0.30_0.50"
    if a < 0.70:
        return "ABS_IMB_0.50_0.70"
    return "ABS_IMB_≥0.70"


def rr_bucket(rr: Optional[float], prefix: str) -> str:
    if rr is None:
        return f"{prefix}_UNAVAILABLE"
    if rr < 1.0:
        return f"{prefix}_<1"
    if rr < 1.5:
        return f"{prefix}_1_1.5"
    if rr < 2.0:
        return f"{prefix}_1.5_2"
    return f"{prefix}_≥2"


def stop_dist_bps(r: Dict[str, Any]) -> Optional[float]:
    entry = fnum(r, "entry_price", "reference_price")
    stop = fnum(r, "stop_loss_price", "invalidation_price")
    if entry is None or stop is None or entry <= 0:
        return None
    return abs(entry - stop) / entry * 1e4


def target_dist_bps(r: Dict[str, Any]) -> Optional[float]:
    entry = fnum(r, "entry_price", "reference_price")
    tgt = fnum(r, "profit_target_price")
    if entry is None or tgt is None or entry <= 0:
        return None
    return abs(tgt - entry) / entry * 1e4


def dist_bucket(bps: Optional[float], prefix: str) -> str:
    if bps is None:
        return f"{prefix}_UNAVAILABLE"
    if bps < 30:
        return f"{prefix}_<30bps"
    if bps < 60:
        return f"{prefix}_30_60bps"
    if bps < 100:
        return f"{prefix}_60_100bps"
    return f"{prefix}_≥100bps"


def enrich(r: Dict[str, Any], day: str) -> Optional[Dict[str, Any]]:
    """Decision-time features only. Outcome label is win/loss from net."""
    net = fnum(r, "net_pnl_usdt", "net_pnl")
    if net is None:
        return None
    entry = fnum(r, "entry_price", "reference_price")
    atr = fnum(r, "atr_14")
    imb = fnum(r, "order_flow_imbalance")
    edt = pdt(r.get("entry_time_ms") or r.get("source_timestamp_utc"))
    of_sup = bool(r.get("order_flow_supported"))
    of_label = str(r.get("order_flow_label") or "OF_NONE")
    horizon = str(r.get("horizon_id") or "H_UNKNOWN")
    dq = str(r.get("data_quality_status") or "DQ_NA")
    direction = str(r.get("direction") or "?").upper()
    # H0 plan condition tag (decision-time identity of the plan)
    h0_of = f"H0={horizon}|OF_SUP={of_sup}|OF={of_label}|DQ={dq}"
    return {
        "day": day,
        "book": "crypto_recommendation_plan",
        "plan_id": r.get("plan_id"),
        "symbol": str(r.get("symbol") or "BTCUSDT"),
        "entry_time_utc": edt.isoformat() if edt else None,
        "entry_hour_ist": edt.astimezone(IST).strftime("%H") if edt else "??",
        "net": net,
        "win": net > 0,
        # --- decision-time features ---
        "direction": direction,
        "horizon_id": horizon,
        "h0_of_condition": h0_of,
        "of_supported": "OF_SUP=True" if of_sup else "OF_SUP=False",
        "of_label": of_label,
        "of_latched_at_open": "LATCHED" if r.get("order_flow_latched_at_open") else "NOT_LATCHED",
        "imb_bucket": imb_bucket(imb),
        "atr_bucket": atr_bucket(atr, entry),
        "net_rr_bucket": rr_bucket(fnum(r, "net_rr_ratio"), "NET_RR"),
        "gross_rr_bucket": rr_bucket(fnum(r, "gross_rr_ratio"), "GROSS_RR"),
        "stop_dist_bucket": dist_bucket(stop_dist_bps(r), "STOP_DIST"),
        "target_dist_bucket": dist_bucket(target_dist_bps(r), "TGT_DIST"),
        "dq": dq,
        "signal_cadence": str(r.get("signal_cadence") or "CADENCE_NA"),
        # Combinations (still decision-time)
        "dir_x_of": f"{direction}|{('OF_SUP=True' if of_sup else 'OF_SUP=False')}",
        "dir_x_imb": f"{direction}|{imb_bucket(imb)}",
        "dir_x_atr": f"{direction}|{atr_bucket(atr, entry)}",
        "of_x_atr": f"{('OF_SUP=True' if of_sup else 'OF_SUP=False')}|{atr_bucket(atr, entry)}",
        # Explicit UNAVAILABLE dimensions (not reconstructed without look-ahead-safe bars)
        "trend": "TREND_UNAVAILABLE",
        "momentum": "MOMENTUM_UNAVAILABLE",
        "price_location": "PRICE_LOCATION_UNAVAILABLE",
        "signal_persistence_multi_bar": "SIGNAL_PERSISTENCE_UNAVAILABLE",
    }


def summarize(rows: Sequence[Dict[str, Any]]) -> Dict[str, Any]:
    n = len(rows)
    w = sum(1 for r in rows if r["win"])
    l = n - w
    nets = [r["net"] for r in rows]
    wins = [x for x in nets if x > 0]
    losses = [x for x in nets if x < 0]
    aw = sum(wins) / len(wins) if wins else None
    al = sum(losses) / len(losses) if losses else None
    payoff = abs(aw / al) if aw is not None and al not in (None, 0) else None
    return {
        "n": n,
        "wins": w,
        "losses": l,
        "win_rate_pct": round(100 * w / n, 2) if n else None,
        "wilson_low_95_pct": wilson_low(w, n),
        "net_sum": round(sum(nets), 4) if nets else None,
        "payoff": None if payoff is None else round(payoff, 4),
        "coverage_vs_book_pct": None,  # filled by caller
        "small_n": n < SMALL_N,
    }


def wl_by(
    rows: Sequence[Dict[str, Any]],
    key: str,
    book_n: int,
) -> List[Dict[str, Any]]:
    buckets: Dict[str, List[Dict[str, Any]]] = defaultdict(list)
    for r in rows:
        buckets[str(r[key])].append(r)
    out = []
    for k, rs in buckets.items():
        s = summarize(rs)
        s["key"] = k
        s["coverage_vs_book_pct"] = round(100 * s["n"] / book_n, 2) if book_n else None
        # session stability
        sess = {}
        for d in DAYS:
            sub = [r for r in rs if r["day"] == d]
            if not sub:
                sess[d] = {"n": 0, "win_rate_pct": None}
            else:
                ss = summarize(sub)
                sess[d] = {"n": ss["n"], "win_rate_pct": ss["win_rate_pct"]}
        rates = [v["win_rate_pct"] for v in sess.values() if v["n"] >= 5 and v["win_rate_pct"] is not None]
        s["session"] = sess
        s["sessions_with_n≥5"] = len(rates)
        s["session_wr_range_pp"] = (
            None if len(rates) < 2 else round(max(rates) - min(rates), 2)
        )
        s["single_session_dominant"] = False
        if s["n"] >= SMALL_N:
            # flag if ≥70% of slice n comes from one day
            day_ns = [sess[d]["n"] for d in DAYS]
            if day_ns and max(day_ns) / s["n"] >= 0.70:
                s["single_session_dominant"] = True
        out.append(s)
    out.sort(key=lambda r: (r["win_rate_pct"] is None, r["win_rate_pct"] or 999, -r["n"]))
    return out


def winner_loser_means(rows: Sequence[Dict[str, Any]]) -> Dict[str, Any]:
    """Descriptive means of decision-time continuous proxies — not predictors involving path."""
    wins = [r for r in rows if r["win"]]
    losses = [r for r in rows if not r["win"]]

    def dist(subset: Sequence[Dict[str, Any]], field: str) -> Dict[str, int]:
        c: Dict[str, int] = defaultdict(int)
        for r in subset:
            c[str(r[field])] += 1
        return dict(sorted(c.items(), key=lambda kv: -kv[1]))

    return {
        "n_wins": len(wins),
        "n_losses": len(losses),
        "direction_wins": dist(wins, "direction"),
        "direction_losses": dist(losses, "direction"),
        "of_supported_wins": dist(wins, "of_supported"),
        "of_supported_losses": dist(losses, "of_supported"),
        "atr_wins": dist(wins, "atr_bucket"),
        "atr_losses": dist(losses, "atr_bucket"),
        "imb_wins": dist(wins, "imb_bucket"),
        "imb_losses": dist(losses, "imb_bucket"),
        "h0_wins": dist(wins, "h0_of_condition"),
        "h0_losses": dist(losses, "h0_of_condition"),
    }


def association_leads(payload: Dict[str, Any], book_n: int) -> List[str]:
    """Descriptive association leads only — not Gate 2 / not filters."""
    leads: List[str] = []
    baseline_wr = payload["overall"]["win_rate_pct"] or 0

    def consider(title: str, rows: List[Dict[str, Any]], min_n: int = SMALL_N) -> None:
        eligible = [
            r
            for r in rows
            if r["n"] >= min_n
            and not r["small_n"]
            and "UNAVAILABLE" not in r["key"]
            and not r.get("single_session_dominant")
        ]
        if len(eligible) < 2:
            return
        best = max(eligible, key=lambda r: r["win_rate_pct"] or -1)
        worst = min(eligible, key=lambda r: r["win_rate_pct"] or 999)
        gap = (best["win_rate_pct"] or 0) - (worst["win_rate_pct"] or 0)
        if gap < 10:
            return
        # coverage / trade-count reduction if one kept the "best" slice only
        reduction = round(100 * (1 - best["n"] / book_n), 1)
        leads.append(
            f"{title}: `{best['key']}` WR {best['win_rate_pct']}% "
            f"(n={best['n']}, cov {best['coverage_vs_book_pct']}%, "
            f"wilson_low {best['wilson_low_95_pct']}%, "
            f"session_range_pp {best['session_wr_range_pp']}) vs "
            f"`{worst['key']}` WR {worst['win_rate_pct']}% (n={worst['n']}). "
            f"Selecting only best slice would drop ~{reduction}% of trades — not success by itself."
        )

    consider("Direction", payload["by_direction"])
    consider("OF supported", payload["by_of_supported"])
    consider("H0×OF condition", payload["by_h0_of"])
    consider("Abs imbalance", payload["by_imb"])
    consider("ATR regime", payload["by_atr"])
    consider("Direction × OF", payload["by_dir_x_of"])
    consider("Direction × imbalance", payload["by_dir_x_imb"])
    consider("Net RR geometry", payload["by_net_rr"])
    consider("Entry hour IST", payload["by_hour"], min_n=20)

    # Flag hour slices that are single-session dominant even if gap exists
    hour_dom = [r for r in payload["by_hour"] if r.get("single_session_dominant") and r["n"] >= 15]
    if hour_dom:
        keys = ", ".join(f"`{r['key']}`" for r in hour_dom[:6])
        leads.append(
            f"Hour caution: single-session-dominant slices present ({keys}) — "
            "do not treat as stable entry-quality."
        )

    leads.append(
        "UNAVAILABLE at decision time in this export (no look-ahead-safe reconstruction): "
        "trend, momentum, multi-bar signal persistence, price location vs structure."
    )
    leads.append(
        f"Book baseline WR {baseline_wr}% (n={book_n}). Associations are descriptive only; "
        "Gate 2 / Gate 3 not authorized from this artifact alone."
    )
    return leads


def render(payload: Dict[str, Any]) -> str:
    o = payload["overall"]
    c = payload["contrast"]
    lines = [
        "# RP-608-WL-IMPROVE-001 Phase 2 — Crypto Plan Entry Quality (Decision-Time Only)",
        "",
        "**Sessions:** 2026-11-03 … 2026-11-05 · **Authority:** 0% · baseline frozen",
        "**Book:** crypto recommendation / trade-plan only (never pooled).",
        "",
        "## Narrow question",
        "",
        "> Using information available at the entry decision, which observable conditions "
        "distinguish trades that subsequently win from trades that lose?",
        "",
        "## Predictor policy",
        "",
        "| Allowed (decision-time) | Forbidden as predictors |",
        "| --- | --- |",
        "| Entry timestamp, H0/horizon, OF state/label/imbalance, direction, ATR(14), "
        "net/gross RR, stop/target distance, DQ, cadence, latch flag | "
        "Post-entry MFE/MAE, exit reason/time, later marks, holding path |",
        "",
        "## Unavailable (labeled, not imputed)",
        "",
        "- Trend · momentum · price location · multi-bar signal persistence "
        "(no Nov-2026 look-ahead-safe bar join in this run)",
        "",
        f"**Overall:** n={o['n']} · W/L {o['wins']}/{o['losses']} · WR **{o['win_rate_pct']}%** · "
        f"wilson_low95 **{o['wilson_low_95_pct']}%** · payoff {o['payoff']}",
        "",
        "### By session (integrity)",
        "",
        "| Day | n | WR% |",
        "| --- | ---: | ---: |",
    ]
    for d, s in payload["by_session_overall"].items():
        lines.append(f"| {d} | {s['n']} | {s['win_rate_pct']} |")

    lines += [
        "",
        "## Winner vs loser — decision-time distributions",
        "",
        f"- Direction — wins `{c['direction_wins']}` · losses `{c['direction_losses']}`",
        f"- OF supported — wins `{c['of_supported_wins']}` · losses `{c['of_supported_losses']}`",
        f"- ATR — wins `{c['atr_wins']}` · losses `{c['atr_losses']}`",
        f"- Imbalance — wins `{c['imb_wins']}` · losses `{c['imb_losses']}`",
        "",
    ]

    def tbl(title: str, rows: List[Dict[str, Any]]) -> None:
        lines.append(f"## {title}")
        lines.append("")
        lines.append(
            "| Key | n | cov% | W | L | WR% | wilson_lo | Payoff | sess_range_pp | "
            "single_sess? | small_n |"
        )
        lines.append(
            "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- | --- |"
        )
        for r in rows:
            lines.append(
                f"| `{r['key']}` | {r['n']} | {r['coverage_vs_book_pct']} | {r['wins']} | {r['losses']} | "
                f"{r['win_rate_pct']} | {r['wilson_low_95_pct']} | {r['payoff']} | "
                f"{r['session_wr_range_pp']} | {r['single_session_dominant']} | {r['small_n']} |"
            )
        lines.append("")

    for title, key in [
        ("W/L by H0 × OF condition (at entry)", "by_h0_of"),
        ("W/L by direction", "by_direction"),
        ("W/L by OF supported", "by_of_supported"),
        ("W/L by direction × OF", "by_dir_x_of"),
        ("W/L by abs imbalance", "by_imb"),
        ("W/L by direction × imbalance", "by_dir_x_imb"),
        ("W/L by ATR regime (volatility proxy)", "by_atr"),
        ("W/L by direction × ATR", "by_dir_x_atr"),
        ("W/L by net RR geometry", "by_net_rr"),
        ("W/L by stop distance", "by_stop_dist"),
        ("W/L by entry hour IST", "by_hour"),
        ("Unavailable placeholders", "by_unavailable"),
    ]:
        tbl(title, payload[key])

    lines += [
        "## Association leads (descriptive only — not Gate 2 / not filters)",
        "",
    ]
    for lead in payload["leads"]:
        lines.append(f"- {lead}")
    lines += [
        "",
        "## Success criterion reminder",
        "",
        "A defensible, repeatable explanation of entry win probability — "
        "**not** a higher WR obtained by reducing trade count or selecting favorable historical slices.",
        "",
        "## Governance",
        "",
        "Gate 1 PASS · Gate 2 not reached · Gate 3 unauthorized · OF ±0.30 frozen · "
        "other books' Phase 2 deferred · Step 3+ held · Gate R unreached · OFX Stage 2 held.",
        "",
        "Next authorized steps: cross-session replication beyond 3–5 Nov → one falsifiable "
        "hypothesis → Gate 2 review.",
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
        path = export_dir / f"session_export_{day}_crypto_recommendation_plan_closed.jsonl"
        for r in load_jsonl(path):
            e = enrich(r, day)
            if e:
                rows.append(e)

    book_n = len(rows)
    overall = summarize(rows)
    overall["key"] = "ALL"
    overall["coverage_vs_book_pct"] = 100.0
    overall["session"] = {}
    overall["sessions_with_n≥5"] = 3
    overall["session_wr_range_pp"] = None
    overall["single_session_dominant"] = False

    by_session_overall = {}
    for d in DAYS:
        sub = [r for r in rows if r["day"] == d]
        by_session_overall[d] = summarize(sub)

    # unavailable placeholder table
    for r in rows:
        r["unavailable_tag"] = (
            f"{r['trend']}|{r['momentum']}|{r['price_location']}|{r['signal_persistence_multi_bar']}"
        )

    payload: Dict[str, Any] = {
        "protocol": "RP-608-WL-IMPROVE-001",
        "phase": 2,
        "book": "crypto_recommendation_plan",
        "predictor_policy": "decision_time_only",
        "forbidden_predictors": ["mfe", "mae", "exit_reason", "exit_time", "holding_path"],
        "overall": overall,
        "by_session_overall": by_session_overall,
        "contrast": winner_loser_means(rows),
        "by_h0_of": wl_by(rows, "h0_of_condition", book_n),
        "by_direction": wl_by(rows, "direction", book_n),
        "by_of_supported": wl_by(rows, "of_supported", book_n),
        "by_dir_x_of": wl_by(rows, "dir_x_of", book_n),
        "by_imb": wl_by(rows, "imb_bucket", book_n),
        "by_dir_x_imb": wl_by(rows, "dir_x_imb", book_n),
        "by_atr": wl_by(rows, "atr_bucket", book_n),
        "by_dir_x_atr": wl_by(rows, "dir_x_atr", book_n),
        "by_net_rr": wl_by(rows, "net_rr_bucket", book_n),
        "by_stop_dist": wl_by(rows, "stop_dist_bucket", book_n),
        "by_hour": wl_by(rows, "entry_hour_ist", book_n),
        "by_unavailable": wl_by(rows, "unavailable_tag", book_n),
    }
    # session range on overall
    rates = [
        v["win_rate_pct"]
        for v in by_session_overall.values()
        if v["n"] >= 5 and v["win_rate_pct"] is not None
    ]
    if len(rates) >= 2:
        overall["session_wr_range_pp"] = round(max(rates) - min(rates), 2)

    payload["leads"] = association_leads(payload, book_n)

    stem = export_dir / "wl_phase2_crypto_plan_entry_quality_2026-11-03_to_05"
    stem.with_suffix(".json").write_text(json.dumps(payload, indent=2, default=str), encoding="utf-8")
    stem.with_suffix(".md").write_text(render(payload), encoding="utf-8")

    # Row-level decision-time feature store (for replication / Gate 2 prep)
    feats = export_dir / "wl_phase2_crypto_plan_decision_time_rows_2026-11-03_to_05.jsonl"
    with feats.open("w", encoding="utf-8") as fh:
        for r in rows:
            fh.write(json.dumps(r, default=str) + "\n")

    print(
        json.dumps(
            {
                "n": overall["n"],
                "wr": overall["win_rate_pct"],
                "wilson_low": overall["wilson_low_95_pct"],
                "n_leads": len(payload["leads"]),
                "wrote": str(stem) + ".md",
            },
            indent=2,
        )
    )


if __name__ == "__main__":
    main()
