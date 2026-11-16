#!/usr/bin/env python3
"""
RP-608 actionable favorable-excursion grades (four books separate).

Narrow question: for losing trades, was there a meaningful, actionable favorable
opportunity before the loss — or was the path never net-green after costs?

Binary MFE-present is insufficient. Grades are observational, not causal.
Does not change rules. Authority 0%. No Step 3+ / OF / Stage 2 authorization.

Peak MFE/MAE only — no intra-trade time order. "Reversal co-occurrence" means
both extrema recorded; it does NOT prove favorable-then-adverse sequencing.
"""

from __future__ import annotations

import argparse
import json
from collections import Counter
from pathlib import Path
from typing import Any, Dict, List, Optional, Sequence, Tuple

BOOKS = [
    ("nse_estimand_a", "NSE Estimand A", "INR"),
    ("nse_of_paper", "NSE OF paper", "INR"),
    ("crypto_recommendation_plan", "Crypto plan", "USDT"),
    ("crypto_of_paper", "Crypto OF paper", "USDT"),
]

MFE_EPS_BPS = 1e-9
# Peak mark after friction must clear this multiple of friction to count as
# "plausible alternative exit" (breakeven-after-costs alone is not enough).
PLAUSIBLE_FRICTION_MULTIPLE = 2.0
# Also require peak net mark >= this many bps of notional when notional known.
PLAUSIBLE_MIN_NET_BPS = 5.0


def fnum(r: Dict[str, Any], *keys: str) -> Optional[float]:
    for k in keys:
        if r.get(k) is None:
            continue
        try:
            return float(r[k])
        except (TypeError, ValueError):
            continue
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
        return abs(fr)
    g, n = gross_of(r, ccy), net_of(r, ccy)
    if g is not None and n is not None:
        return abs(g - n)
    return None


def mfe_mae_bps(r: Dict[str, Any]) -> Tuple[Optional[float], Optional[float], str]:
    mfe = fnum(r, "mfe_bps")
    mae = fnum(r, "mae_bps")
    if mfe is not None or mae is not None:
        return mfe, mae, "bps"
    entry = fnum(r, "entry_price")
    mfe_px = fnum(r, "mfe_exec_price")
    mae_px = fnum(r, "mae_exec_price")
    side = str(r.get("side") or r.get("direction") or "").upper()
    if entry and entry > 0 and (mfe_px is not None or mae_px is not None):

        def fav_bps(px: Optional[float]) -> Optional[float]:
            if px is None:
                return None
            if "SHORT" in side:
                return (entry - px) / entry * 10_000.0
            return (px - entry) / entry * 10_000.0

        mfe_b = fav_bps(mfe_px)
        mae_b = fav_bps(mae_px)
        # MAE as adverse magnitude (positive number).
        mae_abs = None if mae_b is None else abs(mae_b)
        return mfe_b, mae_abs, "bps_from_exec_price"
    return None, None, "unavailable"


def notional_of(r: Dict[str, Any], ccy: str) -> Optional[float]:
    if ccy == "USDT":
        n = fnum(r, "simulated_notional_usdt")
        if n is not None:
            return abs(n)
        qty = fnum(r, "simulated_quantity", "quantity")
        px = fnum(r, "entry_price")
        if qty is not None and px is not None:
            return abs(qty * px)
        return None
    qty = fnum(r, "quantity", "entry_paper_quantity", "exit_quantity")
    px = fnum(r, "entry_price")
    if qty is not None and px is not None:
        return abs(qty * px)
    return None


def grade_trade(
    *,
    mfe_bps: Optional[float],
    mae_bps: Optional[float],
    mfe_ccy: Optional[float],
    friction: Optional[float],
    notional: Optional[float],
    path_unit: str,
) -> Tuple[str, Dict[str, Any]]:
    """
    Mutually exclusive favorable-opportunity grade (observational).

    Ladder (peak MFE only):
      PATH_UNAVAILABLE
      NO_FAVORABLE_EXCURSION
      SMALL_NEVER_COVERED_COSTS      — peak mark never clears recorded friction
      COVERED_COSTS_NOT_LOSS_SCALE   — peak clears friction but not plausible buffer
      PLAUSIBLE_ALT_EXIT_SUPPORT     — peak clears friction with meaningful buffer

    Separate flag (not exclusive): reversal_extrema_cooccur = mfe>0 and mae>0
    """
    meta: Dict[str, Any] = {
        "mfe_bps": mfe_bps,
        "mae_bps": mae_bps,
        "mfe_ccy": mfe_ccy,
        "friction": friction,
        "notional": notional,
        "path_unit": path_unit,
        "peak_net_mark": None,
        "plausible_floor": None,
        "reversal_extrema_cooccur": False,
    }
    if mfe_bps is None or path_unit == "unavailable" or mfe_ccy is None or friction is None:
        return "PATH_UNAVAILABLE", meta

    meta["reversal_extrema_cooccur"] = (mfe_bps > MFE_EPS_BPS) and (
        mae_bps is not None and mae_bps > MFE_EPS_BPS
    )
    peak_net = mfe_ccy - friction
    meta["peak_net_mark"] = peak_net

    floor_from_friction = PLAUSIBLE_FRICTION_MULTIPLE * friction - friction  # == 1x friction buffer
    # peak_net >= friction  iff  mfe_ccy >= 2*friction
    floor = floor_from_friction
    if notional is not None and notional > 0:
        floor = max(floor, notional * PLAUSIBLE_MIN_NET_BPS / 10_000.0)
    meta["plausible_floor"] = floor

    if mfe_bps <= MFE_EPS_BPS or mfe_ccy <= MFE_EPS_BPS:
        return "NO_FAVORABLE_EXCURSION", meta
    if peak_net <= MFE_EPS_BPS:
        return "SMALL_NEVER_COVERED_COSTS", meta
    if peak_net < floor:
        return "COVERED_COSTS_NOT_LOSS_SCALE", meta
    return "PLAUSIBLE_ALT_EXIT_SUPPORT", meta


def enrich_row(r: Dict[str, Any], ccy: str) -> Optional[Dict[str, Any]]:
    net = net_of(r, ccy)
    if net is None:
        return None
    gross = gross_of(r, ccy)
    friction = friction_of(r, ccy)
    mfe_bps, mae_bps, unit = mfe_mae_bps(r)
    notional = notional_of(r, ccy)
    mfe_ccy = None
    if mfe_bps is not None and notional is not None:
        mfe_ccy = notional * mfe_bps / 10_000.0
    grade, meta = grade_trade(
        mfe_bps=mfe_bps,
        mae_bps=mae_bps,
        mfe_ccy=mfe_ccy,
        friction=friction,
        notional=notional,
        path_unit=unit,
    )
    return {
        "symbol": str(r.get("symbol") or "?"),
        "exit_reason": str(r.get("exit_reason") or r.get("exit_rule") or "UNKNOWN"),
        "net": net,
        "gross": gross,
        "win": net > 0,
        "loss": net < 0,
        "flat": net == 0,
        "grade": grade,
        **meta,
    }


def share(n: int, d: int) -> Optional[float]:
    return None if d == 0 else round(n / d, 4)


def summarize(rows: List[Dict[str, Any]]) -> Dict[str, Any]:
    grades = [
        "PATH_UNAVAILABLE",
        "NO_FAVORABLE_EXCURSION",
        "SMALL_NEVER_COVERED_COSTS",
        "COVERED_COSTS_NOT_LOSS_SCALE",
        "PLAUSIBLE_ALT_EXIT_SUPPORT",
    ]

    def block(subset: List[Dict[str, Any]]) -> Dict[str, Any]:
        c = Counter(e["grade"] for e in subset)
        rev = sum(1 for e in subset if e.get("reversal_extrema_cooccur"))
        path_ok = [e for e in subset if e["grade"] != "PATH_UNAVAILABLE"]
        return {
            "n": len(subset),
            "grade_counts": {g: c.get(g, 0) for g in grades},
            "grade_shares_of_subset": {g: share(c.get(g, 0), len(subset)) for g in grades},
            "grade_shares_among_path_available": {
                g: share(c.get(g, 0), len(path_ok)) for g in grades if g != "PATH_UNAVAILABLE"
            },
            "reversal_extrema_cooccur_n": rev,
            "reversal_extrema_cooccur_share": share(rev, len(subset)),
            "path_available_n": len(path_ok),
        }

    wins = [e for e in rows if e["win"]]
    losses = [e for e in rows if e["loss"]]
    # Among losers with path: actionable vs entry-like
    loss_path = [e for e in losses if e["grade"] != "PATH_UNAVAILABLE"]
    entry_like = sum(
        1
        for e in loss_path
        if e["grade"] in ("NO_FAVORABLE_EXCURSION", "SMALL_NEVER_COVERED_COSTS")
    )
    exit_like = sum(
        1
        for e in loss_path
        if e["grade"] in ("COVERED_COSTS_NOT_LOSS_SCALE", "PLAUSIBLE_ALT_EXIT_SUPPORT")
    )
    return {
        "n_closed": len(rows),
        "all": block(rows),
        "wins": block(wins),
        "losses": block(losses),
        "loss_path_available_n": len(loss_path),
        "among_path_available_losses_entry_like_share": share(entry_like, len(loss_path)),
        "among_path_available_losses_exit_hold_like_share": share(exit_like, len(loss_path)),
        "definitions": {
            "NO_FAVORABLE_EXCURSION": "mfe_bps <= 0 (or mfe currency ~0)",
            "SMALL_NEVER_COVERED_COSTS": "mfe_ccy > 0 but peak_net_mark = mfe_ccy - friction <= 0",
            "COVERED_COSTS_NOT_LOSS_SCALE": (
                "peak_net_mark > 0 but < plausible_floor "
                f"(floor = max(friction buffer from {PLAUSIBLE_FRICTION_MULTIPLE}x costs, "
                f"{PLAUSIBLE_MIN_NET_BPS} bps of notional))"
            ),
            "PLAUSIBLE_ALT_EXIT_SUPPORT": (
                "peak_net_mark >= plausible_floor — peak was meaningfully net-green after costs; "
                "does NOT prove an alternate exit policy would have captured it"
            ),
            "reversal_extrema_cooccur": (
                "mfe>0 and mae>0 recorded; NOT time-ordered favorable-then-adverse proof"
            ),
            "entry_like": "NO_FAVORABLE + SMALL_NEVER_COVERED_COSTS",
            "exit_hold_like": "COVERED_COSTS_NOT_LOSS_SCALE + PLAUSIBLE_ALT_EXIT_SUPPORT",
        },
    }


def render_md(day: str, books_out: Dict[str, Any]) -> str:
    lines = [
        f"# RP-608 Actionable Favorable-Excursion Grades — {day}",
        "",
        "**Authority:** 0.0% · books never pooled · rules frozen",
        "**Question:** Did losing trades ever produce a meaningful, actionable favorable opportunity before they lost?",
        "**Not authorized:** threshold/OF changes · Stage 2 classifier · Gate R · Step 3+",
        "",
        "## Method (observational)",
        "",
        "Peak MFE converted to currency via notional × mfe_bps / 1e4; compared to recorded friction.",
        "Mutually exclusive ladder on peak mark only — not a claim that an alternate exit was executable.",
        "OF books without MFE/MAE fields are `PATH_UNAVAILABLE` (entry vs exit unresolved).",
        "",
        "| Grade | Meaning |",
        "| --- | --- |",
        "| `NO_FAVORABLE_EXCURSION` | No favorable peak |",
        "| `SMALL_NEVER_COVERED_COSTS` | Favorable peak never clears realistic costs |",
        "| `COVERED_COSTS_NOT_LOSS_SCALE` | Peak clears costs but thin vs plausible buffer |",
        "| `PLAUSIBLE_ALT_EXIT_SUPPORT` | Peak meaningfully net-green after costs (hypothesis support only) |",
        "| `PATH_UNAVAILABLE` | No MFE/notional/friction path fields |",
        "",
    ]
    titles = {k: t for k, t, _ in BOOKS}
    for key, _, _ in BOOKS:
        b = books_out[key]
        title = titles[key]
        lines += [f"## {title}", ""]
        if b["loss_path_available_n"] == 0:
            lines += [
                f"Closed: {b['n_closed']}. **Path unavailable** for this export — "
                "entry vs exit/hold cannot be graded from peak MFE.",
                "",
            ]
            continue
        L = b["losses"]
        W = b["wins"]
        lines += [
            f"Closed: {b['n_closed']} · path-available losses: {b['loss_path_available_n']}",
            "",
            f"Among path-available **losses**: entry-like share "
            f"**{b['among_path_available_losses_entry_like_share']}** · "
            f"exit/hold-like share **{b['among_path_available_losses_exit_hold_like_share']}**",
            "",
            "### Losses by grade",
            "",
            "| Grade | n | share of losses |",
            "| --- | ---: | ---: |",
        ]
        for g, n in L["grade_counts"].items():
            lines.append(f"| `{g}` | {n} | {L['grade_shares_of_subset'].get(g)} |")
        lines += [
            "",
            f"Reversal extrema co-occur among losses: {L['reversal_extrema_cooccur_n']} "
            f"({L['reversal_extrema_cooccur_share']}) — not time-ordered.",
            "",
            "### Wins by grade (control)",
            "",
            "| Grade | n | share of wins |",
            "| --- | ---: | ---: |",
        ]
        for g, n in W["grade_counts"].items():
            lines.append(f"| `{g}` | {n} | {W['grade_shares_of_subset'].get(g)} |")
        lines.append("")

    lines += [
        "## Cross-book note",
        "",
        "Do not pool books. Shared W/L weakness does not imply a shared corrective action.",
        "PLAUSIBLE_ALT_EXIT_SUPPORT among losers prioritizes exit/hold investigation; "
        "it does not authorize an execution-policy change.",
        "",
        "## Governance",
        "",
        "Rules frozen · authority 0% · Step 3+ held · Gate R unreached · OFX Stage 2 classifier held.",
        "",
    ]
    return "\n".join(lines)


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--day", required=True)
    ap.add_argument("--export-dir", default="artifacts/rp608_econ_edge_diag")
    args = ap.parse_args()
    export_dir = Path(args.export_dir)
    books_out: Dict[str, Any] = {}
    for key, title, ccy in BOOKS:
        path = export_dir / f"session_export_{args.day}_{key}_closed.jsonl"
        rows = load_jsonl(path)
        enriched = [e for e in (enrich_row(r, ccy) for r in rows) if e is not None]
        books_out[key] = summarize(enriched)
        books_out[key]["title"] = title
        books_out[key]["currency"] = ccy
        books_out[key]["source"] = str(path)

    payload = {
        "day": args.day,
        "protocol": "RP-608-ECON-EDGE-ACTIONABLE-MFE-GRADES",
        "authority_pct": 0.0,
        "books": books_out,
    }
    md_path = export_dir / f"actionable_mfe_grades_{args.day}.md"
    json_path = export_dir / f"actionable_mfe_grades_{args.day}.json"
    md_path.write_text(render_md(args.day, books_out), encoding="utf-8")
    json_path.write_text(json.dumps(payload, indent=2, default=str), encoding="utf-8")
    print(json.dumps({k: {
        "n": books_out[k]["n_closed"],
        "loss_path_n": books_out[k]["loss_path_available_n"],
        "entry_like": books_out[k]["among_path_available_losses_entry_like_share"],
        "exit_like": books_out[k]["among_path_available_losses_exit_hold_like_share"],
        "loss_grades": books_out[k]["losses"]["grade_counts"],
    } for k, _, _ in BOOKS}, indent=2))
    print(f"WROTE {md_path}")


if __name__ == "__main__":
    main()
