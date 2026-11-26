#!/usr/bin/env python3
"""
RP-608-EXIT-FALSIFICATION-001 — Phase 1
60-minute counterfactual for square-off + max-hold Estimand A trades.

Read-only on production journals. Writes only under artifacts/rp608_exit_falsification_001/.
Authority: 0.0%. No production / daemon changes.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from collections import Counter
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

WORKSPACE = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(WORKSPACE))
sys.path.insert(0, str(WORKSPACE / "scripts"))

from rp608_nse_equity_costs import calc_indian_equity_friction  # noqa: E402

PROTOCOL_ID = "RP-608-EXIT-FALSIFICATION-001"
PARENT_PROTOCOL = "RP-608-NSE-SHADOW-001"
COHORT_REASONS = {
    "MANDATORY_INTRADAY_SQUARE_OFF",
    "MAX_HOLD_HORIZON_180M",
    "TARGET_HORIZON_180M",
}
BARS_TO_60M = 4  # production: bars_held >= 4


def _sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def _day_utc(ms: Any) -> Optional[str]:
    try:
        return datetime.fromtimestamp(int(ms) / 1000, tz=timezone.utc).strftime("%Y-%m-%d")
    except Exception:
        return None


def _load_jsonl(path: Path) -> List[Dict[str, Any]]:
    rows: List[Dict[str, Any]] = []
    with path.open() as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            rows.append(json.loads(line))
    return rows


def _friction_total(fric: Any) -> float:
    if isinstance(fric, dict):
        return float(fric.get("total_friction") or 0.0)
    return float(fric or 0.0)


def _find_bar_window(
    bars: List[Dict[str, Any]],
    *,
    entry_bar_idx: Optional[int],
    entry_ts_ms: int,
) -> Tuple[Optional[int], Optional[int], str]:
    """
    Locate entry bar and +60m bar indices inside the symbol bar list.
    Prefer stamped entry_bar_idx when it indexes the list; else timestamp match.
    """
    if not bars:
        return None, None, "NO_BARS"

    if entry_bar_idx is not None and 0 <= int(entry_bar_idx) < len(bars):
        e = int(entry_bar_idx)
        t60 = e + BARS_TO_60M
        if t60 < len(bars):
            return e, t60, "ENTRY_BAR_IDX"
        return e, None, "INSUFFICIENT_BARS_AFTER_ENTRY_IDX"

    # Timestamp: first bar with ts >= entry_ts, then +4 bars forward in list
    idxs = [i for i, b in enumerate(bars) if int(b.get("timestamp_utc_ms") or 0) >= entry_ts_ms]
    if not idxs:
        return None, None, "NO_BAR_AT_OR_AFTER_ENTRY_TS"
    e = idxs[0]
    t60 = e + BARS_TO_60M
    if t60 < len(bars):
        return e, t60, "ENTRY_TS_MATCH"
    return e, None, "INSUFFICIENT_BARS_AFTER_ENTRY_TS"


def _path_excursions(
    bars: List[Dict[str, Any]],
    *,
    entry_list_idx: int,
    t60_list_idx: int,
    entry_price: float,
) -> Tuple[float, float, float, float]:
    """MFE/MAE over (entry, +60m] using bar high/low; LONG only."""
    mfe_px = entry_price
    mae_px = entry_price
    # Forward bars after entry through the 60m bar (inclusive of t60)
    for b in bars[entry_list_idx + 1 : t60_list_idx + 1]:
        hi = float(b.get("high") or entry_price)
        lo = float(b.get("low") or entry_price)
        if hi > mfe_px:
            mfe_px = hi
        if lo < mae_px:
            mae_px = lo
    mfe_bps = ((mfe_px - entry_price) / entry_price) * 10000.0 if entry_price else 0.0
    mae_bps = ((entry_price - mae_px) / entry_price) * 10000.0 if entry_price else 0.0
    return mfe_px, mae_px, mfe_bps, mae_bps


def reconstruct_trade(
    trade: Dict[str, Any],
    bars_by_sym: Dict[str, List[Dict[str, Any]]],
) -> Dict[str, Any]:
    sym = str(trade.get("symbol") or "")
    entry_p = float(trade["entry_price"])
    qty = float(trade.get("quantity") or 0.0)
    entry_ts = int(trade["entry_timestamp_utc_ms"])
    entry_bar_idx = trade.get("entry_bar_idx")
    half_spread = float(trade.get("half_spread_bps") or 2.5)
    bars = bars_by_sym.get(sym) or []

    e_idx, t60_idx, align = _find_bar_window(
        bars, entry_bar_idx=entry_bar_idx if entry_bar_idx is not None else None, entry_ts_ms=entry_ts
    )

    out: Dict[str, Any] = {
        "protocol_id": PROTOCOL_ID,
        "parent_protocol_id": PARENT_PROTOCOL,
        "phase": 1,
        "trade_id": trade.get("trade_id"),
        "symbol": sym,
        "side": trade.get("side") or "LONG",
        "exit_reason_realized": trade.get("exit_reason"),
        "entry_price": entry_p,
        "entry_timestamp_utc_ms": entry_ts,
        "entry_bar_idx_stamped": entry_bar_idx,
        "quantity": qty,
        "realized_exit_price": float(trade.get("exit_price") or 0.0),
        "realized_gross_pnl_inr": float(trade.get("gross_pnl_inr") or 0.0),
        "realized_net_pnl_inr": float(trade.get("net_pnl_inr") or 0.0),
        "realized_friction_inr": _friction_total(trade.get("friction")),
        "realized_mfe_bps_full_path": trade.get("mfe_bps"),
        "realized_mae_bps_full_path": trade.get("mae_bps"),
        "bar_align_method": align,
        "reconstructible": False,
    }

    if e_idx is None or t60_idx is None:
        out["non_reconstructible_reason"] = align
        return out

    bar_60 = bars[t60_idx]
    mtm = float(bar_60.get("close") or 0.0)
    if mtm <= 0:
        out["non_reconstructible_reason"] = "BAD_60M_CLOSE"
        out["entry_list_idx"] = e_idx
        out["t60_list_idx"] = t60_idx
        return out

    mfe_px, mae_px, mfe_bps, mae_bps = _path_excursions(
        bars, entry_list_idx=e_idx, t60_list_idx=t60_idx, entry_price=entry_p
    )
    gross = (mtm - entry_p) * qty
    fric = calc_indian_equity_friction(
        buy_price=entry_p,
        sell_price=mtm,
        quantity=qty,
        half_spread_bps=half_spread,
    )
    net = gross - float(fric.get("total_friction") or 0.0)
    realized_net = float(trade.get("net_pnl_inr") or 0.0)

    out.update(
        {
            "reconstructible": True,
            "entry_list_idx": e_idx,
            "t60_list_idx": t60_idx,
            "t60_bar_timestamp_utc_ms": int(bar_60.get("timestamp_utc_ms") or 0),
            "mtm_price_60m": round(mtm, 4),
            "mtm_source": "BAR_CLOSE_ENTRY_PLUS_4",
            "gross_mtm_60m_inr": round(gross, 2),
            "friction_60m": fric,
            "net_mtm_60m_inr": round(net, 2),
            "mfe_price_60m": round(mfe_px, 4),
            "mae_price_60m": round(mae_px, 4),
            "mfe_60m_bps": round(mfe_bps, 2),
            "mae_60m_bps": round(mae_bps, 2),
            "would_horizon_green": bool(mtm > entry_p),
            "delta_vs_realized_net_inr": round(net - realized_net, 2),
            "hold_to_60m_bars": BARS_TO_60M,
        }
    )
    return out


def summarize(rows: List[Dict[str, Any]]) -> Dict[str, Any]:
    ok = [r for r in rows if r.get("reconstructible")]
    bad = [r for r in rows if not r.get("reconstructible")]
    n = len(ok)

    def _sum(key: str) -> float:
        return sum(float(r.get(key) or 0.0) for r in ok)

    green = sum(1 for r in ok if r.get("would_horizon_green"))
    net_pos = sum(1 for r in ok if float(r.get("net_mtm_60m_inr") or 0.0) > 0)
    delta_pos = sum(1 for r in ok if float(r.get("delta_vs_realized_net_inr") or 0.0) > 0)

    by_reason: Dict[str, Dict[str, Any]] = {}
    for reason in sorted({r.get("exit_reason_realized") for r in ok}):
        sub = [r for r in ok if r.get("exit_reason_realized") == reason]
        by_reason[str(reason)] = {
            "n": len(sub),
            "realized_net_inr": round(sum(float(r.get("realized_net_pnl_inr") or 0) for r in sub), 2),
            "cf_net_mtm_60m_inr": round(sum(float(r.get("net_mtm_60m_inr") or 0) for r in sub), 2),
            "delta_vs_realized_net_inr": round(
                sum(float(r.get("delta_vs_realized_net_inr") or 0) for r in sub), 2
            ),
            "would_horizon_green_n": sum(1 for r in sub if r.get("would_horizon_green")),
            "cf_net_win_n": sum(1 for r in sub if float(r.get("net_mtm_60m_inr") or 0) > 0),
        }

    return {
        "protocol_id": PROTOCOL_ID,
        "phase": 1,
        "n_cohort": len(rows),
        "n_reconstructible": n,
        "n_non_reconstructible": len(bad),
        "non_reconstructible_reasons": dict(Counter(r.get("non_reconstructible_reason") for r in bad)),
        "realized_net_inr": round(_sum("realized_net_pnl_inr"), 2),
        "cf_gross_mtm_60m_inr": round(_sum("gross_mtm_60m_inr"), 2),
        "cf_net_mtm_60m_inr": round(_sum("net_mtm_60m_inr"), 2),
        "delta_vs_realized_net_inr": round(_sum("delta_vs_realized_net_inr"), 2),
        "would_horizon_green_n": green,
        "would_horizon_green_rate": round(green / n, 4) if n else None,
        "cf_net_win_n": net_pos,
        "cf_net_win_rate": round(net_pos / n, 4) if n else None,
        "delta_positive_n": delta_pos,
        "by_exit_reason": by_reason,
        "avg_mfe_60m_bps": round(sum(float(r.get("mfe_60m_bps") or 0) for r in ok) / n, 2) if n else None,
        "avg_mae_60m_bps": round(sum(float(r.get("mae_60m_bps") or 0) for r in ok) / n, 2) if n else None,
    }


def write_markdown(summary: Dict[str, Any], preflight: Dict[str, Any], path: Path) -> None:
    lines = [
        f"# {PROTOCOL_ID} — Phase 1: 60m Counterfactual",
        "",
        f"**Generated (UTC):** {preflight.get('generated_utc')}",
        f"**Session filter:** `{preflight.get('session_date')}`",
        f"**Authority:** 0.0% · production journal untouched",
        "",
        "## Preflight cutoff",
        "",
        "```json",
        json.dumps(preflight, indent=2, sort_keys=True),
        "```",
        "",
        "## Headline",
        "",
        f"- Cohort n = **{summary['n_cohort']}** (reconstructible **{summary['n_reconstructible']}**)",
        f"- Realized net (production exits) = **₹{summary['realized_net_inr']:+,.2f}**",
        f"- Counterfactual net MTM at +60m = **₹{summary['cf_net_mtm_60m_inr']:+,.2f}**",
        f"- Δ (CF − realized) = **₹{summary['delta_vs_realized_net_inr']:+,.2f}**",
        f"- Would have been horizon-green at +60m: **{summary['would_horizon_green_n']}/{summary['n_reconstructible']}**"
        f" ({(summary['would_horizon_green_rate'] or 0)*100:.1f}%)",
        f"- CF net winners: **{summary['cf_net_win_n']}/{summary['n_reconstructible']}**",
        f"- Avg MFE₆₀ / MAE₆₀ (bps): **{summary['avg_mfe_60m_bps']}** / **{summary['avg_mae_60m_bps']}**",
        "",
        "## By realized exit reason",
        "",
        "| Exit reason | n | Realized net ₹ | CF net@60m ₹ | Δ ₹ | Horizon-green | CF net wins |",
        "| --- | ---: | ---: | ---: | ---: | ---: | ---: |",
    ]
    for reason, block in (summary.get("by_exit_reason") or {}).items():
        lines.append(
            f"| `{reason}` | {block['n']} | {block['realized_net_inr']:+,.0f} | "
            f"{block['cf_net_mtm_60m_inr']:+,.0f} | {block['delta_vs_realized_net_inr']:+,.0f} | "
            f"{block['would_horizon_green_n']} | {block['cf_net_win_n']} |"
        )
    lines.extend(
        [
            "",
            "## Method notes",
            "",
            "- 60m mark = bar close at `entry_bar_idx + 4` (production `bars_held >= 4`).",
            "- MFE/MAE₆₀ from forward bar high/low on `(entry_bar, entry_bar+4]` — not full-path journal excursions.",
            "- Friction = `calc_indian_equity_friction` (same statutory helper as Estimand A close).",
            "- This is **Phase 1 evidence only** — policies A–E are not ranked here.",
            "",
            "## Non-reconstructible",
            "",
            f"```json\n{json.dumps(summary.get('non_reconstructible_reasons') or {}, indent=2)}\n```",
            "",
        ]
    )
    path.write_text("\n".join(lines) + "\n")


def main() -> int:
    ap = argparse.ArgumentParser(description=f"{PROTOCOL_ID} Phase 1")
    ap.add_argument(
        "--journal",
        type=Path,
        default=WORKSPACE / "live_capture/shadow_validation/nse/rp608_nse_trade_journal.jsonl",
    )
    ap.add_argument(
        "--bar-history",
        type=Path,
        default=WORKSPACE / "live_capture/shadow_validation/nse/rp608_nse_bar_history.json",
    )
    ap.add_argument("--session-date", type=str, default="2026-11-02")
    ap.add_argument(
        "--out-dir",
        type=Path,
        default=WORKSPACE / "artifacts/rp608_exit_falsification_001",
    )
    args = ap.parse_args()

    out_dir: Path = args.out_dir
    out_dir.mkdir(parents=True, exist_ok=True)

    journal: Path = args.journal
    bars_path: Path = args.bar_history
    if not journal.is_file():
        raise SystemExit(f"journal missing: {journal}")
    if not bars_path.is_file():
        raise SystemExit(f"bar history missing: {bars_path}")

    rows = _load_jsonl(journal)
    cohort = [
        r
        for r in rows
        if _day_utc(r.get("exit_timestamp_utc_ms")) == args.session_date
        and str(r.get("exit_reason") or "") in COHORT_REASONS
        and str(r.get("protocol_id") or PARENT_PROTOCOL) == PARENT_PROTOCOL
    ]
    bars_by_sym: Dict[str, List[Dict[str, Any]]] = json.loads(bars_path.read_text())

    preflight = {
        "protocol_id": PROTOCOL_ID,
        "phase": 1,
        "generated_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "session_date": args.session_date,
        "journal_path": str(journal),
        "journal_bytes": journal.stat().st_size,
        "journal_sha256": _sha256_file(journal),
        "bar_history_path": str(bars_path),
        "bar_history_bytes": bars_path.stat().st_size,
        "bar_history_sha256": _sha256_file(bars_path),
        "n_journal_rows": len(rows),
        "n_cohort": len(cohort),
        "cohort_exit_reason_counts": dict(Counter(r.get("exit_reason") for r in cohort)),
        "max_exit_timestamp_utc_ms": max((int(r["exit_timestamp_utc_ms"]) for r in cohort), default=None),
        "production_journal_mutated": False,
        "authority_pct": 0.0,
    }

    results = [reconstruct_trade(r, bars_by_sym) for r in cohort]
    summary = summarize(results)
    summary["preflight"] = preflight

    # Persist cohort snapshot (research copy only — not a journal rewrite)
    cohort_path = out_dir / f"phase1_cohort_{args.session_date}.jsonl"
    with cohort_path.open("w") as f:
        for r in cohort:
            f.write(json.dumps(r, sort_keys=True) + "\n")

    jsonl_path = out_dir / f"phase1_60m_counterfactual_{args.session_date}.jsonl"
    with jsonl_path.open("w") as f:
        for r in results:
            f.write(json.dumps(r, sort_keys=True) + "\n")

    summary_path = out_dir / f"phase1_60m_summary_{args.session_date}.json"
    summary_path.write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n")

    md_path = out_dir / f"phase1_60m_report_{args.session_date}.md"
    write_markdown(summary, preflight, md_path)

    preflight_path = out_dir / f"phase1_preflight_{args.session_date}.json"
    preflight_path.write_text(json.dumps(preflight, indent=2, sort_keys=True) + "\n")

    print(json.dumps({"ok": True, "summary": summary, "artifacts": {
        "cohort": str(cohort_path),
        "jsonl": str(jsonl_path),
        "summary": str(summary_path),
        "report": str(md_path),
        "preflight": str(preflight_path),
    }}, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
