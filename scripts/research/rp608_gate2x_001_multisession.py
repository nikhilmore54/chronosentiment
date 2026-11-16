#!/usr/bin/env python3
"""
RP-608-GATE2X-001 — Multi-session journal-anchored exit/cost counterfactuals.

Frozen entries. Production journals read-only. Authority 0%.
Policies predeclared in docs/research/rp608_gate2x_001_scoped.md.
Books never pooled.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from collections import defaultdict
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

WORKSPACE = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(WORKSPACE))
sys.path.insert(0, str(WORKSPACE / "scripts"))

from rp608_nse_equity_costs import calc_indian_equity_friction  # noqa: E402

PROTOCOL_ID = "RP-608-GATE2X-001"
IST = timezone(timedelta(hours=5, minutes=30))


def _sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def _load_jsonl(path: Path) -> List[Dict[str, Any]]:
    out: List[Dict[str, Any]] = []
    with path.open() as f:
        for line in f:
            if line.strip():
                out.append(json.loads(line))
    return out


def _pdt_ms(ms: Any) -> Optional[datetime]:
    try:
        return datetime.fromtimestamp(int(ms) / 1000, tz=timezone.utc)
    except Exception:
        return None


def _day_ist(ms: Any) -> str:
    dt = _pdt_ms(ms)
    if not dt:
        return "UNKNOWN"
    return dt.astimezone(IST).strftime("%Y-%m-%d")


def _fric_total(fric: Any) -> float:
    if isinstance(fric, dict):
        return float(fric.get("total_friction") or 0.0)
    try:
        return float(fric or 0.0)
    except (TypeError, ValueError):
        return 0.0


def _summarize(
    rows: List[Dict[str, Any]], net_key: str
) -> Dict[str, Any]:
    nets = [float(r[net_key]) for r in rows]
    wins = sum(1 for n in nets if n > 0)
    losses = sum(1 for n in nets if n < 0)
    n = len(nets)
    avg_w = (sum(n for n in nets if n > 0) / wins) if wins else 0.0
    avg_l = (sum(abs(n) for n in nets if n < 0) / losses) if losses else 0.0
    payoff = (avg_w / avg_l) if avg_l > 0 else None
    by_day: Dict[str, List[float]] = defaultdict(list)
    for r in rows:
        by_day[str(r.get("session_ist") or "UNKNOWN")].append(float(r[net_key]))
    day_delta_pos = 0
    day_n = 0
    day_rows = []
    for d, vals in sorted(by_day.items()):
        if d == "UNKNOWN":
            continue
        day_n += 1
        s = sum(vals)
        if s >= 0:
            day_delta_pos += 1
        day_rows.append({"day": d, "n": len(vals), "net": round(s, 4)})
    return {
        "n": n,
        "wins": wins,
        "losses": losses,
        "wr_pct": round(100.0 * wins / n, 2) if n else None,
        "net": round(sum(nets), 4),
        "avg_win": round(avg_w, 4),
        "avg_loss_abs": round(avg_l, 4),
        "payoff": round(payoff, 4) if payoff is not None else None,
        "n_sessions": day_n,
        "sessions_nonneg_net": day_delta_pos,
        "session_nonneg_share": round(day_delta_pos / day_n, 4) if day_n else None,
        "worst_days": sorted(day_rows, key=lambda x: x["net"])[:5],
        "best_days": sorted(day_rows, key=lambda x: x["net"], reverse=True)[:5],
    }


# ─── NSE A ─────────────────────────────────────────────────────────────


def nse_policy_rows(trades: List[Dict[str, Any]], policy: str) -> List[Dict[str, Any]]:
    rows: List[Dict[str, Any]] = []
    for t in trades:
        entry = float(t["entry_price"])
        qty = float(t["quantity"])
        half = float(t.get("half_spread_bps") or 2.5)
        prod_net = float(t["net_pnl_inr"])
        prod_gross = float(t.get("gross_pnl_inr") or 0.0)
        reason = str(t.get("exit_reason") or "")
        session = _day_ist(t.get("entry_timestamp_utc_ms"))
        base = {
            "trade_id": t.get("trade_id"),
            "symbol": t.get("symbol"),
            "session_ist": session,
            "policy": policy,
            "prod_exit_reason": reason,
            "prod_net_pnl_inr": prod_net,
        }
        if policy == "A":
            rows.append(
                {
                    **base,
                    "exit_reason": reason,
                    "net_pnl_inr": prod_net,
                    "gross_pnl_inr": prod_gross,
                    "friction_inr": _fric_total(t.get("friction")),
                    "cf_applied": False,
                }
            )
            continue

        # F / F0: friction-eaten TARGET_HORIZON_60M
        eaten = reason == "TARGET_HORIZON_60M" and prod_net < 0
        if not eaten:
            rows.append(
                {
                    **base,
                    "exit_reason": reason,
                    "net_pnl_inr": prod_net,
                    "gross_pnl_inr": prod_gross,
                    "friction_inr": _fric_total(t.get("friction")),
                    "cf_applied": False,
                }
            )
            continue

        if policy == "F0":
            rows.append(
                {
                    **base,
                    "exit_reason": "FRICTION_HORIZON_VOID",
                    "net_pnl_inr": 0.0,
                    "gross_pnl_inr": 0.0,
                    "friction_inr": 0.0,
                    "cf_applied": True,
                }
            )
            continue

        if policy == "F":
            fric = calc_indian_equity_friction(
                buy_price=entry, sell_price=entry, quantity=qty, half_spread_bps=half
            )
            fric_inr = float(fric.get("total_friction") or 0.0)
            rows.append(
                {
                    **base,
                    "exit_reason": "FRICTION_HORIZON_SCRATCH",
                    "net_pnl_inr": round(0.0 - fric_inr, 2),
                    "gross_pnl_inr": 0.0,
                    "friction_inr": round(fric_inr, 2),
                    "cf_applied": True,
                }
            )
            continue

        raise ValueError(f"unknown NSE policy {policy}")
    return rows


# ─── Crypto plan ───────────────────────────────────────────────────────


def _crypto_friction(entry: float, exit_px: float, qty: float) -> float:
    return (entry * qty + exit_px * qty) * (6.0 / 10000.0)


def crypto_policy_rows(trades: List[Dict[str, Any]], policy: str) -> List[Dict[str, Any]]:
    rows: List[Dict[str, Any]] = []
    for t in trades:
        direction = str(t.get("direction") or "").upper()
        entry = float(t["entry_price"])
        qty = float(t.get("simulated_quantity") or 0.0)
        prod_net = float(t["net_pnl_usdt"])
        reason = str(t.get("exit_reason") or "")
        hold = float(t.get("holding_duration_seconds") or 0.0)
        session = _day_ist(t.get("entry_time_ms"))
        base = {
            "plan_id": t.get("plan_id"),
            "direction": direction,
            "session_ist": session,
            "policy": policy,
            "prod_exit_reason": reason,
            "prod_net_pnl_usdt": prod_net,
            "holding_duration_seconds": hold,
        }
        if policy == "A":
            rows.append(
                {
                    **base,
                    "exit_reason": reason,
                    "net_pnl_usdt": prod_net,
                    "cf_applied": False,
                }
            )
            continue

        if policy == "G" and reason == "AMBIGUOUS_STOP_TARGET":
            tp = float(t["profit_target_price"])
            if direction == "LONG":
                gross = (tp - entry) * qty
            else:
                gross = (entry - tp) * qty
            fric = _crypto_friction(entry, tp, qty)
            rows.append(
                {
                    **base,
                    "exit_reason": "AMBIGUOUS_RESOLVED_TP",
                    "net_pnl_usdt": round(gross - fric, 2),
                    "cf_applied": True,
                }
            )
            continue

        early = False
        if policy == "H60" and reason == "STOP_LOSS_HIT" and hold < 60.0:
            early = True
        if policy == "H120" and reason == "STOP_LOSS_HIT" and hold < 120.0:
            early = True
        if early:
            fric = _crypto_friction(entry, entry, qty)
            rows.append(
                {
                    **base,
                    "exit_reason": "EARLY_SL_SCRATCH",
                    "net_pnl_usdt": round(0.0 - fric, 2),
                    "cf_applied": True,
                }
            )
            continue

        rows.append(
            {
                **base,
                "exit_reason": reason,
                "net_pnl_usdt": prod_net,
                "cf_applied": False,
            }
        )
    return rows


def _paired_session_delta(
    control_rows: List[Dict[str, Any]],
    chall_rows: List[Dict[str, Any]],
    net_key: str,
) -> Dict[str, Any]:
    c_day: Dict[str, float] = defaultdict(float)
    x_day: Dict[str, float] = defaultdict(float)
    for r in control_rows:
        d = str(r.get("session_ist") or "UNKNOWN")
        if d != "UNKNOWN":
            c_day[d] += float(r[net_key])
    for r in chall_rows:
        d = str(r.get("session_ist") or "UNKNOWN")
        if d != "UNKNOWN":
            x_day[d] += float(r[net_key])
    days = sorted(set(c_day) | set(x_day))
    improved = sum(1 for d in days if x_day.get(d, 0.0) >= c_day.get(d, 0.0))
    return {
        "n_sessions": len(days),
        "sessions_delta_nonneg": improved,
        "session_delta_nonneg_share": round(improved / len(days), 4) if days else None,
    }


def _verdict(
    control: Dict[str, Any],
    chall: Dict[str, Any],
    policy: str,
    *,
    paired: Dict[str, Any],
) -> Dict[str, Any]:
    """Gate 2X advance heuristics for this tranche (journal-anchored)."""
    if policy == "F0":
        return {
            "eligible_for_advance": False,
            "label": "DIAGNOSTIC_ONLY",
            "reason": "F0 is an optimistic void bound, not a production proposal",
            "paired": paired,
        }
    d_net = float(chall["net"]) - float(control["net"])
    n_sess = int(paired.get("n_sessions") or 0)
    share = paired.get("session_delta_nonneg_share")
    sess_ok = n_sess >= 4 and share is not None and share >= 0.5
    better_net = d_net > 0
    if better_net and sess_ok:
        label = "ADVANCE_CANDIDATE"
        reason = (
            f"Δnet={d_net:+.2f}; paired session Δnet≥0 share={share} "
            f"({paired.get('sessions_delta_nonneg')}/{n_sess}); Gate 5 still required"
        )
    elif better_net and not sess_ok:
        label = "IMPROVES_NET_SESSION_FRAGILE"
        reason = f"Δnet={d_net:+.2f} but paired session share={share} (need ≥0.5 on ≥4 sessions)"
    else:
        label = "DO_NOT_ADVANCE"
        reason = f"Δnet={d_net:+.2f}; does not clear net+session bar"
    return {
        "eligible_for_advance": label == "ADVANCE_CANDIDATE",
        "label": label,
        "delta_net": round(d_net, 4),
        "reason": reason,
        "paired": paired,
    }


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--nse-journal",
        type=Path,
        default=WORKSPACE
        / "live_capture/shadow_validation/nse/rp608_nse_trade_journal.jsonl",
    )
    ap.add_argument(
        "--crypto-journal",
        type=Path,
        default=WORKSPACE
        / "live_capture/shadow_validation/crypto/trade_plan_overlay_v1/trade_plan_outcomes.jsonl",
    )
    ap.add_argument(
        "--out-dir",
        type=Path,
        default=WORKSPACE / "artifacts/rp608_gate2x_001",
    )
    args = ap.parse_args()
    out_dir: Path = args.out_dir
    out_dir.mkdir(parents=True, exist_ok=True)
    inputs = out_dir / "inputs"
    inputs.mkdir(exist_ok=True)

    report: Dict[str, Any] = {
        "protocol_id": PROTOCOL_ID,
        "generated_utc": datetime.now(timezone.utc).isoformat(),
        "authority_pct": 0.0,
        "scope": "docs/research/rp608_gate2x_001_scoped.md",
        "gate2_amendment": "docs/research/rp608_gate2_amendment_v2.md",
        "books": {},
    }

    # ── NSE ──
    nse_trades = _load_jsonl(args.nse_journal)
    nse_fp = {
        "path": str(args.nse_journal),
        "sha256": _sha256_file(args.nse_journal) if args.nse_journal.exists() else None,
        "n": len(nse_trades),
        "bytes": args.nse_journal.stat().st_size if args.nse_journal.exists() else 0,
    }
    nse_policies = {}
    nse_rowsets: Dict[str, List[Dict[str, Any]]] = {}
    nse_control = None
    for pol in ("A", "F", "F0"):
        rows = nse_policy_rows(nse_trades, pol)
        nse_rowsets[pol] = rows
        summ = _summarize(rows, "net_pnl_inr")
        summ["n_cf_applied"] = sum(1 for r in rows if r.get("cf_applied"))
        nse_policies[pol] = summ
        if pol == "A":
            nse_control = summ
    nse_verdicts = {
        pol: _verdict(
            nse_control,  # type: ignore[arg-type]
            nse_policies[pol],
            pol,
            paired=_paired_session_delta(nse_rowsets["A"], nse_rowsets[pol], "net_pnl_inr"),
        )
        for pol in ("F", "F0")
    }
    report["books"]["nse_estimand_a"] = {
        "ccy": "INR",
        "fingerprint": nse_fp,
        "policies": nse_policies,
        "verdicts": nse_verdicts,
        "note": "Journal-anchored; CLOSE_GRID A–E multi-session blocked until Pi bar history restored",
    }

    # ── Crypto ──
    crypto_trades = _load_jsonl(args.crypto_journal)
    crypto_fp = {
        "path": str(args.crypto_journal),
        "sha256": _sha256_file(args.crypto_journal) if args.crypto_journal.exists() else None,
        "n": len(crypto_trades),
        "bytes": args.crypto_journal.stat().st_size if args.crypto_journal.exists() else 0,
    }
    crypto_policies = {}
    crypto_rowsets: Dict[str, List[Dict[str, Any]]] = {}
    crypto_control = None
    for pol in ("A", "G", "H60", "H120"):
        rows = crypto_policy_rows(crypto_trades, pol)
        crypto_rowsets[pol] = rows
        summ = _summarize(rows, "net_pnl_usdt")
        summ["n_cf_applied"] = sum(1 for r in rows if r.get("cf_applied"))
        crypto_policies[pol] = summ
        if pol == "A":
            crypto_control = summ
    crypto_verdicts = {
        pol: _verdict(
            crypto_control,  # type: ignore[arg-type]
            crypto_policies[pol],
            pol,
            paired=_paired_session_delta(
                crypto_rowsets["A"], crypto_rowsets[pol], "net_pnl_usdt"
            ),
        )
        for pol in ("G", "H60", "H120")
    }
    report["books"]["crypto_recommendation_plan"] = {
        "ccy": "USDT",
        "fingerprint": crypto_fp,
        "policies": crypto_policies,
        "verdicts": crypto_verdicts,
        "note": "Journal-anchored; scratch CF uses modeled 6bps/side RT friction",
    }

    json_path = out_dir / "gate2x_001_multisession_results.json"
    json_path.write_text(json.dumps(report, indent=2))

    # Markdown
    lines = [
        "# RP-608-GATE2X-001 — Multi-Session Results",
        "",
        f"**Generated:** {report['generated_utc']}  ",
        f"**Authority:** 0% · journals read-only · Gate 2 v2  ",
        f"**Scope:** `docs/research/rp608_gate2x_001_scoped.md`",
        "",
        "## NSE Estimand A",
        "",
        f"Fingerprint n={nse_fp['n']} sha256=`{nse_fp['sha256']}`",
        "",
        "| Policy | n | WR% | net INR | payoff | sessions | sess nonneg share | CF applied |",
        "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
    ]
    for pol, s in nse_policies.items():
        lines.append(
            f"| **{pol}** | {s['n']} | {s['wr_pct']} | {s['net']} | {s['payoff']} | "
            f"{s['n_sessions']} | {s['session_nonneg_share']} | {s.get('n_cf_applied')} |"
        )
    lines += ["", "### Verdicts", ""]
    for pol, v in nse_verdicts.items():
        lines.append(f"- **{pol}**: `{v['label']}` — {v['reason']}")

    lines += [
        "",
        "## Crypto recommendation plan",
        "",
        f"Fingerprint n={crypto_fp['n']} sha256=`{crypto_fp['sha256']}`",
        "",
        "| Policy | n | WR% | net USDT | payoff | sessions | sess nonneg share | CF applied |",
        "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
    ]
    for pol, s in crypto_policies.items():
        lines.append(
            f"| **{pol}** | {s['n']} | {s['wr_pct']} | {s['net']} | {s['payoff']} | "
            f"{s['n_sessions']} | {s['session_nonneg_share']} | {s.get('n_cf_applied')} |"
        )
    lines += ["", "### Verdicts", ""]
    for pol, v in crypto_verdicts.items():
        lines.append(f"- **{pol}**: `{v['label']}` — {v['reason']}")

    lines += [
        "",
        "## Governance",
        "",
        "- No live exit/entry changes (Gate 5 required).",
        "- CLOSE_GRID A–E multi-session still blocked: restore Pi `rp608_nse_bar_history.json`.",
        "- F0 cannot ADVANCE.",
        "",
    ]
    md_path = out_dir / "gate2x_001_multisession_results.md"
    md_path.write_text("\n".join(lines))
    print(f"wrote {json_path}")
    print(f"wrote {md_path}")
    print(json.dumps({"nse_verdicts": nse_verdicts, "crypto_verdicts": crypto_verdicts}, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
