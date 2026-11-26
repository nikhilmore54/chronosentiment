#!/usr/bin/env python3
"""
RP-608 NSE A — PATH_ONLY diagnostic on 2026-11-16 post-13:00 IST closed trades.

Frozen journal entries. Authority 0%. Single-session → DIAGNOSTIC_ONLY
(not Gate 2X ADVANCE per rp608_gate2_amendment_v2.md §2.2).

Question: how much of today's bad W/L is next-bar BAR_OHLC path-through
geometry vs selection, and what journal-anchored exit counterfactuals do?
"""

from __future__ import annotations

import json
from collections import Counter, defaultdict
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional

IST = timezone(timedelta(hours=5, minutes=30))
CUTOFF = datetime(2026, 11, 16, 13, 0, 0, tzinfo=IST)
PROTOCOL = "RP-608-NSE-A-POST1PM-GATE2X-PATH-DIAG"


def ts_ist(ms: Any) -> Optional[datetime]:
    try:
        return datetime.fromtimestamp(int(ms) / 1000, tz=timezone.utc).astimezone(IST)
    except Exception:
        return None


def fric_total(t: Dict[str, Any]) -> float:
    fr = t.get("friction") or {}
    if isinstance(fr, dict):
        return float(fr.get("total_friction") or 0.0)
    try:
        return float(fr or 0.0)
    except (TypeError, ValueError):
        return 0.0


def summarize(nets: List[float], label: str) -> Dict[str, Any]:
    n = len(nets)
    wins = sum(1 for x in nets if x > 0)
    losses = sum(1 for x in nets if x < 0)
    flat = n - wins - losses
    avg_w = (sum(x for x in nets if x > 0) / wins) if wins else None
    avg_l = (sum(abs(x) for x in nets if x < 0) / losses) if losses else None
    payoff = (avg_w / avg_l) if (avg_w is not None and avg_l and avg_l > 0) else None
    total = sum(nets)
    return {
        "label": label,
        "n": n,
        "W/L": f"{wins}/{n}",
        "WR_pct": round(100.0 * wins / n, 1) if n else None,
        "loss_rate_pct": round(100.0 * losses / n, 1) if n else None,
        "flat": flat,
        "payoff_net": None if payoff is None else round(payoff, 3),
        "avg_win": None if avg_w is None else round(avg_w, 2),
        "avg_loss": None if avg_l is None else round(avg_l, 2),
        "net": round(total, 2),
        "expectancy": round(total / n, 2) if n else None,
    }


def main() -> None:
    root = Path("live_capture/shadow_validation/nse")
    journal = root / "rp608_nse_trade_journal.jsonl"
    rows: List[Dict[str, Any]] = []
    with journal.open() as f:
        for line in f:
            if not line.strip():
                continue
            o = json.loads(line)
            ts = ts_ist(o.get("entry_timestamp_utc_ms"))
            if ts is None or ts < CUTOFF:
                continue
            if str(o.get("estimand") or "ESTIMAND_A") not in ("ESTIMAND_A", ""):
                # journal rows are Estimand A lifecycle
                pass
            rows.append(o)

    prod_nets = [float(t.get("net_pnl_inr") or 0.0) for t in rows]
    prod = summarize(prod_nets, "PRODUCTION")

    # Ambiguous OHLC: both stop and target sides touched in recorded MFE/MAE.
    ambiguous = []
    for t in rows:
        ep = float(t.get("entry_price") or 0)
        sl = float(t.get("stop_loss_price") or 0)
        tp = float(t.get("target_price") or 0)
        mfe_p = float(t.get("mfe_price") or ep)
        mae_p = float(t.get("mae_price") or ep)
        if ep <= 0 or sl <= 0 or tp <= 0:
            continue
        hit_sl = mae_p <= sl
        hit_tp = mfe_p >= tp
        if hit_sl and hit_tp:
            ambiguous.append(t)

    # Counterfactual G: if production SL on BAR_OHLC and path also touched TP → flip to TP net.
    # Journal-anchored: use target exit price and recompute net ≈ gross_at_tp - friction_at_tp proxy
    # using recorded friction scale (approx): friction stays similar order; use calc if available.
    g_nets: List[float] = []
    flipped = 0
    for t in rows:
        net = float(t.get("net_pnl_inr") or 0.0)
        ep = float(t.get("entry_price") or 0)
        sl = float(t.get("stop_loss_price") or 0)
        tp = float(t.get("target_price") or 0)
        mfe_p = float(t.get("mfe_price") or ep)
        mae_p = float(t.get("mae_price") or ep)
        qty = float(t.get("quantity") or 0)
        reason = t.get("exit_reason")
        path = t.get("exit_path")
        if (
            path == "BAR_OHLC_PATH"
            and reason == "STOP_LOSS"
            and ep > 0
            and qty > 0
            and mae_p <= sl
            and mfe_p >= tp
        ):
            # Flip ambiguous SL→TP (crypto-G analogue for NSE OHLC same-bar both-touch).
            gross = (tp - ep) * qty
            # Re-scale friction roughly by turnover change vs recorded.
            fr = fric_total(t)
            exit_px = float(t.get("exit_price") or sl)
            # crude: keep same friction (statutory dominates; spread on exit similar)
            net = gross - fr
            flipped += 1
        g_nets.append(net)
    g_sum = summarize(g_nets, "CF_G_AMBIGUOUS_SL_TO_TP")

    # Counterfactual: void all BAR_OHLC exits that are pure SL with bars_held==1
    # and replace net with 0 (scratch@exit-bar) — optimistic diagnostic bound only.
    scratch_nets: List[float] = []
    scratched = 0
    for t in rows:
        net = float(t.get("net_pnl_inr") or 0.0)
        entry_bar = int(t.get("entry_bar_idx") or -1)
        exit_bar = int(t.get("exit_bar_idx") or -1)
        held = exit_bar - entry_bar
        if (
            t.get("exit_path") == "BAR_OHLC_PATH"
            and t.get("exit_reason") == "STOP_LOSS"
            and held == 1
        ):
            scratch_nets.append(0.0)
            scratched += 1
        else:
            scratch_nets.append(net)
    scratch_sum = summarize(scratch_nets, "CF_SCRATCH_NEXTBAR_OHLC_SL")

    # Cohort: WS_LAST vs LAST entry marks (same exit stack)
    by_src: Dict[str, List[float]] = defaultdict(list)
    for t in rows:
        src = str(t.get("entry_mark_source") or "NONE")
        by_src[src].append(float(t.get("net_pnl_inr") or 0.0))

    # Exit stack mix
    exit_mix = Counter((t.get("exit_path"), t.get("exit_reason")) for t in rows)

    out = {
        "protocol_id": PROTOCOL,
        "authority_pct": 0.0,
        "disposition": "DIAGNOSTIC_ONLY_SINGLE_SESSION",
        "not_gate2x_advance": True,
        "cutoff_ist": CUTOFF.isoformat(),
        "n": len(rows),
        "production": prod,
        "exit_mix": {f"{a}|{b}": c for (a, b), c in exit_mix.items()},
        "n_ambiguous_both_touch": len(ambiguous),
        "cf_g": g_sum,
        "cf_g_flipped": flipped,
        "cf_scratch_nextbar_ohlc_sl": scratch_sum,
        "cf_scratch_n": scratched,
        "by_entry_mark_source": {
            k: summarize(v, k) for k, v in sorted(by_src.items())
        },
        "interpretation": [
            "Primary W/L failure today is next-bar BAR_OHLC_PATH (almost all exits).",
            "Tick/WS entry mark change did not repair selection or exit geometry.",
            "CF_G (ambiguous SL→TP) is journal-anchored analogue of crypto Gate2X-G; single-session only.",
            "CF_SCRATCH next-bar OHLC SL is an optimistic bound (void loss), not a fill model.",
            "No live daemon / Gate 5 change authorized from this memo.",
        ],
    }

    art = Path("artifacts/rp608_econ_edge_diag")
    art.mkdir(parents=True, exist_ok=True)
    out_path = art / "rp608_nse_a_post1pm_gate2x_path_diag.json"
    out_path.write_text(json.dumps(out, indent=2) + "\n")
    md = art / "rp608_nse_a_post1pm_gate2x_path_diag.md"
    lines = [
        f"# {PROTOCOL}",
        "",
        f"**Disposition:** `{out['disposition']}` · authority 0% · not Gate 2X ADVANCE",
        f"**Window:** entries ≥ {CUTOFF.isoformat()} · n={len(rows)}",
        "",
        "## Production (W/L first)",
        f"- W/L **{prod['W/L']}** · WR {prod['WR_pct']}% · loss rate {prod['loss_rate_pct']}%",
        f"- Payoff net {prod['payoff_net']} · net **{prod['net']}** · E[net]={prod['expectancy']}",
        f"- Exit mix: `{out['exit_mix']}`",
        f"- Ambiguous both-touch (MFE≥TP ∧ MAE≤SL): **{len(ambiguous)}**",
        "",
        "## Journal-anchored counterfactuals (frozen entries)",
        f"- **CF_G** ambiguous SL→TP: flipped={flipped} · W/L {g_sum['W/L']} · net {g_sum['net']} · E={g_sum['expectancy']}",
        f"- **CF_SCRATCH** next-bar OHLC SL→0: scratched={scratched} · W/L {scratch_sum['W/L']} · net {scratch_sum['net']} · E={scratch_sum['expectancy']}",
        "",
        "## By entry mark source",
    ]
    for k, s in out["by_entry_mark_source"].items():
        lines.append(
            f"- `{k}`: W/L {s['W/L']} · net {s['net']} · E={s['expectancy']}"
        )
    lines.extend(
        [
            "",
            "## Locked next steps (no live edit)",
            "1. NSE A Gate 2X path package needs multi-session CLOSE_GRID (bar history restore/enrich on Pi).",
            "2. Gate 2E remains separate: decision-time selection — prior NSE A discovery still DO NOT ADVANCE under v1; needs new sealed 2E package under v2.",
            "3. Do not change traj15 / stops / OF ±0.30 without Gate 5.",
            "",
        ]
    )
    md.write_text("\n".join(lines))
    print(json.dumps(out, indent=2))
    print("WROTE", out_path)
    print("WROTE", md)


if __name__ == "__main__":
    main()
