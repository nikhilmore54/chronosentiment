#!/usr/bin/env python3
"""
RP-608-ECON-EDGE-DIAG-001 — mutually exclusive primary *accounting* attribution.

Distinct from causal mechanism labels (G3 / reverse-OF / stops / time exits /
friction-as-cause / concentration-as-cause). Those remain UNRESOLVED until
counterfactual evidence supports them.

Input: session_export_*_closed.jsonl from rp608_econ_edge_session_export.py
Authority: 0%. Rules frozen. Books never pooled.
"""

from __future__ import annotations

import argparse
import json
from collections import Counter, defaultdict
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

BOOKS = (
    ("nse_estimand_a", "INR", "₹"),
    ("nse_of_paper", "INR", "₹"),
    ("crypto_recommendation_plan", "USDT", "$"),
    ("crypto_of_paper", "USDT", "$"),
)

MFE_EPS_BPS = 1e-9


def fnum(r: Dict[str, Any], *keys: str) -> Optional[float]:
    for k in keys:
        if r.get(k) is None:
            continue
        try:
            return float(r[k])
        except (TypeError, ValueError):
            continue
    return None


def trade_id(r: Dict[str, Any]) -> str:
    for k in ("trade_id", "of_trade_id", "plan_id"):
        if r.get(k) is not None:
            return str(r[k])
    return "?"


def pnl_fields(r: Dict[str, Any], currency: str) -> Tuple[Optional[float], Optional[float], Optional[float]]:
    if currency == "INR":
        g = fnum(r, "gross_pnl_inr", "gross_pnl", "touch_gross_pnl_inr")
        n = fnum(r, "net_pnl_inr", "net_pnl", "all_in_net_pnl_inr")
        fr = fnum(r, "friction_inr", "statutory_overhead_inr", "fees_inr", "total_friction_inr")
    else:
        g = fnum(r, "gross_pnl_usdt", "gross_pnl")
        n = fnum(r, "net_pnl_usdt", "net_pnl")
        fr = fnum(r, "friction_usdt", "fees_usdt", "est_friction_usdt")
    if fr is None and g is not None and n is not None:
        fr = g - n
    return g, fr, n


def primary_accounting(gross: Optional[float], net: Optional[float]) -> str:
    if gross is None or net is None:
        return "UNRESOLVED_PNL_FIELDS"
    if net > 0:
        return "NET_POSITIVE"
    if net == 0:
        return "NET_FLAT"
    if gross >= 0:
        return "FRICTION_DOMINATED"
    return "GROSS_ADVERSE"


def path_descriptor(r: Dict[str, Any]) -> str:
    mfe = fnum(r, "mfe_bps", "mfe")
    if mfe is None:
        return "PATH_METRICS_UNAVAILABLE"
    if mfe <= MFE_EPS_BPS:
        return "PATH_NEVER_FAVORABLE"
    return "PATH_HAD_FAVORABLE"


def observed_exit(r: Dict[str, Any]) -> str:
    return str(r.get("exit_reason") or r.get("close_reason") or "UNRESOLVED_EXIT_REASON")


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


def attribute_book(
    *,
    book_key: str,
    rows: List[Dict[str, Any]],
    currency: str,
) -> Dict[str, Any]:
    attributed: List[Dict[str, Any]] = []
    by_primary: Dict[str, Dict[str, float]] = defaultdict(
        lambda: {"n": 0.0, "gross": 0.0, "friction": 0.0, "net": 0.0}
    )
    path_within_ga: Counter = Counter()
    exit_within_primary: Dict[str, Counter] = defaultdict(Counter)
    by_symbol_net: Dict[str, float] = defaultdict(float)
    unresolved: List[Dict[str, Any]] = []

    for r in rows:
        g, fr, n = pnl_fields(r, currency)
        primary = primary_accounting(g, n)
        path = path_descriptor(r)
        exit_r = observed_exit(r)
        sym = str(r.get("symbol") or r.get("tradingsymbol") or "?")
        row_out = {
            "trade_id": trade_id(r),
            "symbol": sym,
            "side": str(r.get("side") or r.get("direction") or "?").upper(),
            "gross": None if g is None else round(g, 6),
            "friction": None if fr is None else round(fr, 6),
            "net": None if n is None else round(n, 6),
            "primary_accounting": primary,
            "path_descriptor": path,
            "observed_exit_reason": exit_r,
            # Causal / economic-mechanism labels intentionally withheld.
            "mechanism_status": "UNRESOLVED",
            "mechanism_candidates_not_established": [
                "signal_quality",
                "exit_quality",
                "friction_as_cause",
                "concentration_as_cause",
                "g3_policy_c",
                "reverse_of",
                "stops",
                "time_exits",
            ],
        }
        if primary == "UNRESOLVED_PNL_FIELDS":
            unresolved.append(row_out)
        attributed.append(row_out)

        b = by_primary[primary]
        b["n"] += 1
        if g is not None:
            b["gross"] += g
        if fr is not None:
            b["friction"] += fr
        if n is not None:
            b["net"] += n
            by_symbol_net[sym] += n
        exit_within_primary[primary][exit_r] += 1
        if primary == "GROSS_ADVERSE":
            path_within_ga[path] += 1

    # Round aggregates
    by_primary_out = {
        k: {
            "n": int(v["n"]),
            "gross": round(v["gross"], 4),
            "friction": round(v["friction"], 4),
            "net": round(v["net"], 4),
            "identity_gross_minus_friction_minus_net": round(v["gross"] - v["friction"] - v["net"], 6),
        }
        for k, v in sorted(by_primary.items())
    }

    total_g = sum(v["gross"] for v in by_primary.values())
    total_f = sum(v["friction"] for v in by_primary.values())
    total_n = sum(v["net"] for v in by_primary.values())

    loss_sum = sum(v for v in by_symbol_net.values() if v < 0)
    top_loss = sorted(by_symbol_net.items(), key=lambda kv: kv[1])[:8]
    top3_loss_share = None
    if loss_sum < 0:
        top3 = sum(v for _, v in top_loss[:3] if v < 0)
        top3_loss_share = round(top3 / loss_sum, 4)
    top3_syms = {s for s, v in top_loss[:3] if v < 0}

    for row in attributed:
        row["concentration_flag"] = (
            "IN_TOP3_LOSS_SYMBOLS" if row["symbol"] in top3_syms else "NOT_TOP3_LOSS_SYMBOL"
        )

    # Path split economic mass inside GROSS_ADVERSE (still not causal)
    ga_path_mass: Dict[str, Dict[str, float]] = defaultdict(
        lambda: {"n": 0.0, "gross": 0.0, "friction": 0.0, "net": 0.0}
    )
    for row in attributed:
        if row["primary_accounting"] != "GROSS_ADVERSE":
            continue
        p = ga_path_mass[row["path_descriptor"]]
        p["n"] += 1
        p["gross"] += row["gross"] or 0.0
        p["friction"] += row["friction"] or 0.0
        p["net"] += row["net"] or 0.0
    ga_path_out = {
        k: {
            "n": int(v["n"]),
            "gross": round(v["gross"], 4),
            "friction": round(v["friction"], 4),
            "net": round(v["net"], 4),
        }
        for k, v in sorted(ga_path_mass.items())
    }

    return {
        "book": book_key,
        "currency": currency,
        "n_closed": len(rows),
        "totals": {
            "gross": round(total_g, 4),
            "friction": round(total_f, 4),
            "net": round(total_n, 4),
            "identity_gross_minus_friction_minus_net": round(total_g - total_f - total_n, 6),
        },
        "primary_accounting_reconcile": by_primary_out,
        "gross_adverse_path_split": ga_path_out,
        "gross_adverse_path_counts": dict(path_within_ga),
        "observed_exit_within_primary": {
            k: dict(v) for k, v in sorted(exit_within_primary.items())
        },
        "concentration": {
            "top_symbols_by_net_loss": [
                {"symbol": s, "net": round(v, 4)} for s, v in top_loss
            ],
            "top3_symbols_share_of_total_losses": top3_loss_share,
            "note": (
                "Concentration is a distributional descriptor. "
                "It is not assigned as the primary accounting label of any trade's loss."
            ),
        },
        "unresolved_pnl_field_rows": unresolved,
        "n_unresolved_pnl_fields": len(unresolved),
        "rows": attributed,
        "caveats": [
            "primary_accounting is mutually exclusive and reconciles to gross/friction/net.",
            "path_descriptor uses MFE only when present; absence → PATH_METRICS_UNAVAILABLE.",
            "observed_exit_reason is descriptive metadata, not a causal attribution.",
            "mechanism_status remains UNRESOLVED for all rows in this deliverable.",
        ],
    }


def to_markdown(rep: Dict[str, Any]) -> str:
    day = rep["session_date_ist"]
    lines = [
        f"# RP-608 Accounting Attribution — {day}",
        "",
        f"**Generated:** {rep['generated_utc']}  ",
        "**Authority:** 0.0% · rules frozen · books not pooled  ",
        f"**Source export:** `{rep['source_export']}`  ",
        f"**Method:** `{rep['method']}`  ",
        "",
        "## Governing distinctions",
        "",
        "1. **Full-session disposition** — all four closed books red; rules frozen; authority 0%.",
        "2. **Primary accounting** — mutually exclusive partition of each closed row by gross/net identity.",
        "3. **Path / exit descriptors** — observational; not causal mechanism labels.",
        "4. **Mechanism status** — UNRESOLVED for every row (G3, reverse-OF, stops, time exits,",
        "   friction-as-cause, concentration-as-cause remain candidates only).",
        "",
        "## Primary accounting labels (mutually exclusive)",
        "",
        "| Label | Rule |",
        "| --- | --- |",
        "| `NET_POSITIVE` | net > 0 |",
        "| `NET_FLAT` | net == 0 |",
        "| `FRICTION_DOMINATED` | net < 0 and gross ≥ 0 |",
        "| `GROSS_ADVERSE` | net < 0 and gross < 0 |",
        "| `UNRESOLVED_PNL_FIELDS` | gross or net missing |",
        "",
    ]

    for key, _cur, sym in BOOKS:
        b = rep["books"][key]
        lines.append(f"## {key}")
        t = b["totals"]
        lines.append(
            f"- n={b['n_closed']} · gross {sym}{t['gross']} · friction {sym}{t['friction']} · net {sym}{t['net']} · "
            f"identity residual {t['identity_gross_minus_friction_minus_net']}"
        )
        lines.append("")
        lines.append("| primary_accounting | n | gross | friction | net | identity residual |")
        lines.append("| --- | ---: | ---: | ---: | ---: | ---: |")
        for label, v in b["primary_accounting_reconcile"].items():
            lines.append(
                f"| `{label}` | {v['n']} | {sym}{v['gross']} | {sym}{v['friction']} | {sym}{v['net']} | "
                f"{v['identity_gross_minus_friction_minus_net']} |"
            )
        lines.append("")
        if b["gross_adverse_path_split"]:
            lines.append("### GROSS_ADVERSE path split (MFE-based; not causal)")
            lines.append("| path_descriptor | n | gross | friction | net |")
            lines.append("| --- | ---: | ---: | ---: | ---: |")
            for label, v in b["gross_adverse_path_split"].items():
                lines.append(
                    f"| `{label}` | {v['n']} | {sym}{v['gross']} | {sym}{v['friction']} | {sym}{v['net']} |"
                )
            lines.append("")
        lines.append("### Observed exit reasons within primary (descriptive only)")
        for label, exits in b["observed_exit_within_primary"].items():
            lines.append(f"- `{label}`: `{exits}`")
        lines.append("")
        c = b["concentration"]
        lines.append(
            f"### Concentration (distributional only) · top3 loss share `{c['top3_symbols_share_of_total_losses']}`"
        )
        lines.append(f"- top losses: `{c['top_symbols_by_net_loss'][:5]}`")
        lines.append(f"- {c['note']}")
        lines.append(f"- unresolved PnL-field rows: **{b['n_unresolved_pnl_fields']}**")
        lines.append("")

    lines.extend(
        [
            "## What this does *not* establish",
            "",
            "- That G3 Policy C caused NSE A losses.",
            "- That reverse-OF exits caused NSE OF losses.",
            "- That stops caused crypto-plan negative expectancy.",
            "- That friction alone explains any book.",
            "- That any particular entry/exit change would have improved the day.",
            "",
            "## Next evidence required for mechanism labels",
            "",
            "- Entry-path / exit counterfactuals (reuse exit-falsification matrix gaps).",
            "- Recommendation-quality vs exit separation on matched cohorts.",
            "- OF incremental value on matched recommendation cohorts.",
            "- Multi-session consistency before any rule proposal.",
            "",
        ]
    )
    return "\n".join(lines)


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--day", default="2026-11-03")
    ap.add_argument("--export-dir", type=Path, required=True)
    ap.add_argument("--out-dir", type=Path, required=True)
    args = ap.parse_args()
    args.out_dir.mkdir(parents=True, exist_ok=True)

    books_out: Dict[str, Any] = {}
    for key, currency, _sym in BOOKS:
        path = args.export_dir / f"session_export_{args.day}_{key}_closed.jsonl"
        rows = load_jsonl(path)
        books_out[key] = attribute_book(book_key=key, rows=rows, currency=currency)

    rep = {
        "protocol_id": "RP-608-ECON-EDGE-DIAG-001",
        "deliverable_id": f"accounting_attribution_{args.day}",
        "session_date_ist": args.day,
        "generated_utc": datetime.now(timezone.utc).isoformat().replace("+00:00", "Z"),
        "live_order_authority": 0.0,
        "rules_frozen": True,
        "books_pooled": False,
        "source_export": str(args.export_dir / f"session_export_{args.day}.json"),
        "method": "docs/research/rp608_econ_edge_session_postmortem_method.md",
        "attribution_layer": "PRIMARY_ACCOUNTING_ONLY",
        "causal_mechanism_labels": "NOT_ASSIGNED",
        "books": books_out,
        "disposition": "docs/research/rp608_econ_edge_2026-11-03_full_session_disposition.md",
    }

    # Slim row tables (no mega nested dump in md)
    (args.out_dir / f"accounting_attribution_{args.day}.json").write_text(
        json.dumps(rep, indent=2, default=str)
    )
    (args.out_dir / f"accounting_attribution_{args.day}.md").write_text(to_markdown(rep))
    for key, _c, _s in BOOKS:
        path = args.out_dir / f"accounting_attribution_{args.day}_{key}_rows.jsonl"
        with path.open("w", encoding="utf-8") as f:
            for r in books_out[key]["rows"]:
                f.write(json.dumps(r, default=str) + "\n")
    print(to_markdown(rep))
    print(f"WROTE {args.out_dir}")


if __name__ == "__main__":
    main()
