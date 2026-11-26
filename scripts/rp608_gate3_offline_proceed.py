#!/usr/bin/env python3
"""
RP-608 Gate 3 offline proceed:
  - 3X crypto policy G (ambiguous→TP) validation
  - 3E NSE BAR_ge_12 confound audit

Authority 0%. Journals read-only. No live daemon edits.
"""

from __future__ import annotations

import hashlib
import json
import math
from collections import Counter, defaultdict
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

WORKSPACE = Path(__file__).resolve().parents[2]
IST = timezone(timedelta(hours=5, minutes=30))
OUT = WORKSPACE / "artifacts/rp608_gate3_offline"
SQUARE_OFF_BAR = 23


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for c in iter(lambda: f.read(1 << 20), b""):
            h.update(c)
    return h.hexdigest()


def load_jsonl(path: Path) -> List[Dict[str, Any]]:
    return [json.loads(l) for l in path.open() if l.strip()]


def day_ist(ms: Any) -> str:
    try:
        return (
            datetime.fromtimestamp(int(ms) / 1000, tz=timezone.utc)
            .astimezone(IST)
            .strftime("%Y-%m-%d")
        )
    except Exception:
        return "UNKNOWN"


def chrono_split_days(days: List[str]) -> Tuple[List[str], List[str]]:
    days = sorted(d for d in days if d != "UNKNOWN")
    cut = max(2, int(math.floor(len(days) * 0.60))) if len(days) >= 4 else max(1, len(days) // 2)
    return days[:cut], days[cut:]


def stats(nets: List[float]) -> Dict[str, Any]:
    n = len(nets)
    wins = sum(1 for x in nets if x > 0)
    losses = sum(1 for x in nets if x < 0)
    avg_w = (sum(x for x in nets if x > 0) / wins) if wins else 0.0
    avg_l = (sum(abs(x) for x in nets if x < 0) / losses) if losses else 0.0
    payoff = (avg_w / avg_l) if avg_l > 0 else None
    return {
        "n": n,
        "wr_pct": round(100 * wins / n, 2) if n else None,
        "net": round(sum(nets), 4),
        "payoff": round(payoff, 4) if payoff is not None else None,
    }


def crypto_fric(entry: float, exit_px: float, qty: float) -> float:
    return (entry * qty + exit_px * qty) * (6.0 / 10000.0)


def crypto_net_at(direction: str, entry: float, exit_px: float, qty: float) -> float:
    if direction == "LONG":
        gross = (exit_px - entry) * qty
    else:
        gross = (entry - exit_px) * qty
    return round(gross - crypto_fric(entry, exit_px, qty), 2)


# ─── Gate 3X: Policy G ────────────────────────────────────────────────


def run_3x_g(crypto: List[Dict[str, Any]]) -> Dict[str, Any]:
    rows = []
    for t in crypto:
        d = str(t.get("direction") or "").upper()
        entry = float(t["entry_price"])
        qty = float(t.get("simulated_quantity") or 0.0)
        prod = float(t["net_pnl_usdt"])
        reason = str(t.get("exit_reason") or "")
        sess = day_ist(t.get("entry_time_ms"))
        tp = float(t.get("profit_target_price") or entry)
        sl = float(t.get("stop_loss_price") or entry)
        g_net = prod
        g_mid_net = prod
        applied = False
        if reason == "AMBIGUOUS_STOP_TARGET":
            applied = True
            g_net = crypto_net_at(d, entry, tp, qty)
            mid = (entry + tp) / 2.0
            g_mid_net = crypto_net_at(d, entry, mid, qty)
        rows.append(
            {
                "session_ist": sess,
                "prod": prod,
                "g": g_net,
                "g_mid": g_mid_net,
                "applied": applied,
                "direction": d,
                "reason": reason,
            }
        )

    days = sorted({r["session_ist"] for r in rows})
    disc_d, hold_d = chrono_split_days(days)
    disc_set, hold_set = set(disc_d), set(hold_d)

    def book_net(rs: List[Dict[str, Any]], key: str) -> float:
        return sum(float(r[key]) for r in rs)

    def paired_share(rs: List[Dict[str, Any]], key: str) -> Dict[str, Any]:
        c_day: Dict[str, float] = defaultdict(float)
        x_day: Dict[str, float] = defaultdict(float)
        for r in rs:
            c_day[r["session_ist"]] += r["prod"]
            x_day[r["session_ist"]] += r[key]
        dd = sorted(set(c_day) | set(x_day) - {"UNKNOWN"})
        ok = sum(1 for d in dd if x_day[d] >= c_day[d])
        return {
            "n_sessions": len(dd),
            "delta_nonneg": ok,
            "share": round(ok / len(dd), 4) if dd else None,
        }

    full = rows
    hold = [r for r in rows if r["session_ist"] in hold_set]
    disc = [r for r in rows if r["session_ist"] in disc_set]
    amb = [r for r in rows if r["applied"]]

    def block(rs: List[Dict[str, Any]], label: str) -> Dict[str, Any]:
        return {
            "label": label,
            "n": len(rs),
            "n_ambiguous": sum(1 for r in rs if r["applied"]),
            "net_A": round(book_net(rs, "prod"), 4),
            "net_G": round(book_net(rs, "g"), 4),
            "net_G_mid": round(book_net(rs, "g_mid"), 4),
            "delta_G": round(book_net(rs, "g") - book_net(rs, "prod"), 4),
            "delta_G_mid": round(book_net(rs, "g_mid") - book_net(rs, "prod"), 4),
            "paired_G": paired_share(rs, "g"),
            "paired_G_mid": paired_share(rs, "g_mid"),
            "stats_A": stats([r["prod"] for r in rs]),
            "stats_G": stats([r["g"] for r in rs]),
        }

    full_b = block(full, "full")
    hold_b = block(hold, "holdout")
    disc_b = block(disc, "discovery")

    # Promote rule: holdout ΔG>0, paired share≥0.5, n_amb on holdout≥5
    promote = (
        hold_b["delta_G"] > 0
        and (hold_b["paired_G"]["share"] or 0) >= 0.5
        and hold_b["n_ambiguous"] >= 5
        and hold_b["paired_G"]["n_sessions"] >= 3
    )
    # Conservative haircut still positive?
    promote_conservative = (
        hold_b["delta_G_mid"] > 0
        and (hold_b["paired_G_mid"]["share"] or 0) >= 0.5
    )

    if promote and promote_conservative:
        disposition = "PASS_GATE3X_DRAFT_GATE5_PACKET"
    elif promote:
        disposition = "PASS_TP_FILL_ONLY_HAIRCUT_FRAGILE"
    else:
        disposition = "DO_NOT_PROMOTE"

    return {
        "protocol": "RP-608-GATE3X-CRYPTO-G-001",
        "discovery_days": disc_d,
        "holdout_days": hold_d,
        "n_ambiguous_total": len(amb),
        "by_direction_ambiguous": dict(Counter(r["direction"] for r in amb)),
        "full": full_b,
        "discovery": disc_b,
        "holdout": hold_b,
        "disposition": disposition,
        "promote_tp_fill": promote,
        "promote_mid_haircut": promote_conservative,
    }


# ─── Gate 3E: BAR_ge_12 confound ──────────────────────────────────────


def run_3e_bar12(nse: List[Dict[str, Any]]) -> Dict[str, Any]:
    rows = []
    for t in nse:
        bar = t.get("entry_bar_idx")
        if bar is None:
            continue
        bar_i = int(bar)
        ms = t.get("entry_timestamp_utc_ms")
        rows.append(
            {
                "session_ist": day_ist(ms),
                "symbol": t.get("symbol"),
                "entry_bar": bar_i,
                "bars_to_sqoff": SQUARE_OFF_BAR - bar_i,
                "late": bar_i >= 12,
                "net": float(t["net_pnl_inr"]),
                "exit_reason": str(t.get("exit_reason") or ""),
                "trade_id": t.get("trade_id"),
            }
        )

    days = sorted({r["session_ist"] for r in rows})
    disc_d, hold_d = chrono_split_days(days)
    hold_set = set(hold_d)
    hold = [r for r in rows if r["session_ist"] in hold_set]
    disc = [r for r in rows if r["session_ist"] in set(disc_d)]

    def cohort(rs: List[Dict[str, Any]], late: bool) -> Dict[str, Any]:
        sub = [r for r in rs if r["late"] is late]
        s = stats([r["net"] for r in sub])
        s["exit_mix"] = dict(Counter(r["exit_reason"] for r in sub))
        s["mean_bars_to_sqoff"] = (
            round(sum(r["bars_to_sqoff"] for r in sub) / len(sub), 2) if sub else None
        )
        return s

    # Bar curve on holdout
    by_bar: Dict[int, List[float]] = defaultdict(list)
    by_bar_ex: Dict[int, Counter] = defaultdict(Counter)
    for r in hold:
        by_bar[r["entry_bar"]].append(r["net"])
        by_bar_ex[r["entry_bar"]][r["exit_reason"]] += 1
    bar_curve = []
    for b in sorted(by_bar):
        st = stats(by_bar[b])
        bar_curve.append(
            {
                "entry_bar": b,
                **st,
                "top_exit": by_bar_ex[b].most_common(2),
            }
        )

    # Same-day same-symbol: late vs early mean net
    pairs = []
    by_ds: Dict[Tuple[str, str], List[Dict[str, Any]]] = defaultdict(list)
    for r in hold:
        by_ds[(r["session_ist"], r["symbol"])].append(r)
    for (day, sym), grp in by_ds.items():
        early = [r for r in grp if not r["late"]]
        late = [r for r in grp if r["late"]]
        if early and late:
            pairs.append(
                {
                    "day": day,
                    "symbol": sym,
                    "early_n": len(early),
                    "late_n": len(late),
                    "early_net": round(sum(r["net"] for r in early), 2),
                    "late_net": round(sum(r["net"] for r in late), 2),
                    "delta_late_minus_early": round(
                        sum(r["net"] for r in late) / len(late)
                        - sum(r["net"] for r in early) / len(early),
                        2,
                    ),
                }
            )

    pair_delta_pos = sum(1 for p in pairs if p["delta_late_minus_early"] > 0)

    # Exit-mix shift: share of STOP vs TARGET vs SQUARE_OFF
    def exit_share(rs: List[Dict[str, Any]], reason_prefix: str) -> float:
        if not rs:
            return 0.0
        return sum(1 for r in rs if r["exit_reason"].startswith(reason_prefix) or r["exit_reason"] == reason_prefix) / len(rs)

    late_h = [r for r in hold if r["late"]]
    early_h = [r for r in hold if not r["late"]]
    mix_shift = {
        "late_stop_share": round(exit_share(late_h, "STOP_LOSS"), 4),
        "early_stop_share": round(exit_share(early_h, "STOP_LOSS"), 4),
        "late_square_share": round(exit_share(late_h, "MANDATORY_INTRADAY_SQUARE_OFF"), 4),
        "early_square_share": round(exit_share(early_h, "MANDATORY_INTRADAY_SQUARE_OFF"), 4),
        "late_horizon60_share": round(exit_share(late_h, "TARGET_HORIZON_60M"), 4),
        "early_horizon60_share": round(exit_share(early_h, "TARGET_HORIZON_60M"), 4),
        "late_maxhold_share": round(exit_share(late_h, "MAX_HOLD_HORIZON_180M"), 4),
        "early_maxhold_share": round(exit_share(early_h, "MAX_HOLD_HORIZON_180M"), 4),
    }

    hold_late = cohort(hold, True)
    hold_early = cohort(hold, False)
    disc_late = cohort(disc, True)
    disc_early = cohort(disc, False)

    # Confound dominant if: (1) late has much higher square-off / truncated path mix explaining net
    # or (2) within-day pairs don't favor late, or (3) bar curve not monotone / only last bars
    square_inflate = mix_shift["late_square_share"] - mix_shift["early_square_share"]
    maxhold_deflate = mix_shift["early_maxhold_share"] - mix_shift["late_maxhold_share"]
    pairs_ok = len(pairs) >= 10 and (pair_delta_pos / len(pairs) >= 0.55)
    econ_ok = (
        hold_late.get("payoff") is not None
        and hold_late["payoff"] >= 1.0
        and hold_late.get("net", 0) > 0
        and (hold_late.get("wr_pct") or 0) > (hold_early.get("wr_pct") or 0)
    )

    if not econ_ok:
        label = "INCONCLUSIVE"
        reason = "Holdout late cohort no longer clears WR+payoff+net jointly in this audit pass"
    elif square_inflate >= 0.15 or (not pairs_ok and square_inflate >= 0.08):
        label = "CONFOUND_DOMINANT"
        reason = (
            f"Late entries shift exit mix (Δsquare_off share={square_inflate:+.2f}); "
            f"within-day pairs favor late in {pair_delta_pos}/{len(pairs)}"
        )
    elif pairs_ok and econ_ok and maxhold_deflate >= 0:
        label = "EDGE_SURVIVES_AUDIT"
        reason = (
            f"Holdout late net={hold_late['net']} payoff={hold_late['payoff']}; "
            f"within-day pairs {pair_delta_pos}/{len(pairs)} favor late"
        )
    else:
        label = "INCONCLUSIVE"
        reason = (
            f"Mixed: pairs {pair_delta_pos}/{len(pairs)}; square_inflate={square_inflate:+.2f}; "
            f"econ_ok={econ_ok}"
        )

    return {
        "protocol": "RP-608-GATE3E-NSE-BAR-GE12-001",
        "discovery_days": disc_d,
        "holdout_days": hold_d,
        "holdout_late": hold_late,
        "holdout_early": hold_early,
        "discovery_late": disc_late,
        "discovery_early": disc_early,
        "exit_mix_shift": mix_shift,
        "bar_curve_holdout": bar_curve,
        "within_day_pairs_n": len(pairs),
        "within_day_pairs_late_better": pair_delta_pos,
        "within_day_pair_share": round(pair_delta_pos / len(pairs), 4) if pairs else None,
        "sample_pairs": sorted(pairs, key=lambda p: -abs(p["delta_late_minus_early"]))[:8],
        "disposition": label,
        "reason": reason,
    }


def main() -> int:
    OUT.mkdir(parents=True, exist_ok=True)
    crypto_p = (
        WORKSPACE
        / "live_capture/shadow_validation/crypto/trade_plan_overlay_v1/trade_plan_outcomes.jsonl"
    )
    nse_p = WORKSPACE / "live_capture/shadow_validation/nse/rp608_nse_trade_journal.jsonl"
    crypto = load_jsonl(crypto_p)
    nse = load_jsonl(nse_p)

    x3 = run_3x_g(crypto)
    e3 = run_3e_bar12(nse)

    report = {
        "generated_utc": datetime.now(timezone.utc).isoformat(),
        "authority_pct": 0.0,
        "fingerprints": {
            "crypto": {"sha256": sha256_file(crypto_p), "n": len(crypto)},
            "nse": {"sha256": sha256_file(nse_p), "n": len(nse)},
        },
        "gate_3x_crypto_g": x3,
        "gate_3e_nse_bar_ge12": e3,
    }
    (OUT / "gate3_offline_proceed.json").write_text(json.dumps(report, indent=2))

    lines = [
        "# RP-608 Gate 3 Offline Proceed",
        "",
        f"**Generated:** {report['generated_utc']}  ",
        "**Authority:** 0% · no live edits  ",
        "",
        "## Gate 3X — Crypto Policy G (ambiguous → TP)",
        "",
        f"**Disposition:** `{x3['disposition']}`  ",
        f"Ambiguous trades: {x3['n_ambiguous_total']} · by side `{x3['by_direction_ambiguous']}`",
        "",
        "| Window | n | amb | net A | net G | ΔG | ΔG mid-haircut | sess Δ≥0 share |",
        "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
    ]
    for key in ("discovery", "holdout", "full"):
        b = x3[key]
        lines.append(
            f"| {key} | {b['n']} | {b['n_ambiguous']} | {b['net_A']} | {b['net_G']} | "
            f"{b['delta_G']} | {b['delta_G_mid']} | {b['paired_G']['share']} |"
        )
    lines += [
        "",
        f"Holdout days: `{x3['holdout_days']}`",
        "",
        "## Gate 3E — NSE `BAR_ge_12` confound audit",
        "",
        f"**Disposition:** `{e3['disposition']}`  ",
        f"**Reason:** {e3['reason']}",
        "",
        f"| Cohort | n | WR% | net | payoff |",
        f"| --- | ---: | ---: | ---: | ---: |",
        f"| Holdout late (≥12) | {e3['holdout_late']['n']} | {e3['holdout_late']['wr_pct']} | {e3['holdout_late']['net']} | {e3['holdout_late']['payoff']} |",
        f"| Holdout early (<12) | {e3['holdout_early']['n']} | {e3['holdout_early']['wr_pct']} | {e3['holdout_early']['net']} | {e3['holdout_early']['payoff']} |",
        "",
        "### Exit-mix shift (holdout)",
        "",
        "```json",
        json.dumps(e3["exit_mix_shift"], indent=2),
        "```",
        "",
        f"Within-day same-symbol pairs: {e3['within_day_pairs_n']} · late better {e3['within_day_pairs_late_better']} "
        f"(share={e3['within_day_pair_share']})",
        "",
        "## Governance",
        "",
        "- Gate 5 / daemon changes: **not** authorized by this run alone.",
        "- Next: if 3X PASS → draft Gate 5 packet for G only; if 3E EDGE_SURVIVES → separate packet for bar cutoff research.",
        "",
    ]
    (OUT / "gate3_offline_proceed.md").write_text("\n".join(lines))
    print(f"wrote {OUT/'gate3_offline_proceed.md'}")
    print(
        json.dumps(
            {
                "gate_3x": x3["disposition"],
                "gate_3e": e3["disposition"],
                "3x_holdout_delta_G": x3["holdout"]["delta_G"],
                "3e_reason": e3["reason"],
            },
            indent=2,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
