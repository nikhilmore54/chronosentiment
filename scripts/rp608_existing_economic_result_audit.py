#!/usr/bin/env python3
"""
RP-608 — Existing economic-result audit (WR vs profitability).

Uses frozen artifacts / crypto plan outcomes only. Does NOT open entry hypotheses,
change stops/targets/execution, or thr-scan.

Usage:
  python3 scripts/research/rp608_existing_economic_result_audit.py \
    --p4 artifacts/rp608_econ_edge_diag/rp608_h2_crypto_plan_session_regime_p4.json \
    --outcomes live_capture/.../trade_plan_outcomes.jsonl \
    --out-dir artifacts/rp608_econ_edge_diag
"""

from __future__ import annotations

import argparse
import json
from collections import defaultdict
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional, Sequence, Tuple

IST = timezone(timedelta(hours=5, minutes=30))
PROTOCOL = "RP-608-EXISTING-ECONOMIC-RESULT-AUDIT-001"
CONFIRMATION_START = "2026-11-06"
DISCOVERY_END = "2026-10-31"
VALIDATION_START = "2026-11-01"
VALIDATION_END = "2026-11-05"


def load_json(path: Path) -> Dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def load_jsonl(path: Path) -> List[Dict[str, Any]]:
    rows = []
    if not path.exists():
        return rows
    for line in path.read_text(encoding="utf-8", errors="ignore").splitlines():
        if not line.strip():
            continue
        try:
            rows.append(json.loads(line))
        except Exception:
            pass
    return rows


def day_ist(ms: Any) -> Optional[str]:
    if ms is None:
        return None
    try:
        return datetime.fromtimestamp(int(ms) / 1000, tz=timezone.utc).astimezone(IST).date().isoformat()
    except Exception:
        return None


def window(day: str) -> str:
    if day <= DISCOVERY_END:
        return "discovery"
    if VALIDATION_START <= day <= VALIDATION_END:
        return "validation"
    if day >= CONFIRMATION_START:
        return "confirmation"
    return "other"


def slice_econ(rows: Sequence[Dict[str, Any]]) -> Dict[str, Any]:
    wins = [float(r["net_pnl_usdt"]) for r in rows if float(r["net_pnl_usdt"]) > 0]
    losses = [float(r["net_pnl_usdt"]) for r in rows if float(r["net_pnl_usdt"]) <= 0]
    fric = [float(r.get("friction_usdt") or 0) for r in rows]
    gross = [float(r.get("gross_pnl_usdt") or 0) for r in rows if r.get("gross_pnl_usdt") is not None]
    n = len(rows)
    avg_w = sum(wins) / len(wins) if wins else None
    avg_l = sum(abs(x) for x in losses) / len(losses) if losses else None
    payoff = round(avg_w / avg_l, 4) if avg_w is not None and avg_l and avg_l > 0 else None
    # loss concentration: top 10% of losses by magnitude share of total loss mass
    loss_sorted = sorted((abs(x) for x in losses), reverse=True)
    loss_mass = sum(loss_sorted) or 1.0
    k = max(1, int(round(0.1 * len(loss_sorted)))) if loss_sorted else 0
    top10_share = round(100.0 * sum(loss_sorted[:k]) / loss_mass, 2) if loss_sorted else None
    return {
        "n": n,
        "wins": len(wins),
        "losses": len(losses),
        "wr_pct": round(100.0 * len(wins) / n, 2) if n else None,
        "avg_win_usdt": None if avg_w is None else round(avg_w, 4),
        "avg_loss_abs_usdt": None if avg_l is None else round(avg_l, 4),
        "payoff": payoff,
        "net_usdt": round(sum(float(r["net_pnl_usdt"]) for r in rows), 4) if rows else None,
        "gross_usdt": round(sum(gross), 4) if gross else None,
        "friction_usdt": round(sum(fric), 4) if rows else None,
        "friction_per_trade_usdt": round(sum(fric) / n, 4) if n else None,
        "top10pct_losses_share_of_loss_mass_pct": top10_share,
        "wr_vs_econ": (
            "WR_NEAR_COINFLIP_BUT_NET_NEGATIVE"
            if n and (40 <= (100.0 * len(wins) / n) <= 60) and sum(float(r["net_pnl_usdt"]) for r in rows) < 0
            else "WR_LOW_AND_NET_NEGATIVE"
            if n and (100.0 * len(wins) / n) < 40 and sum(float(r["net_pnl_usdt"]) for r in rows) < 0
            else "NET_NON_NEGATIVE"
            if n and sum(float(r["net_pnl_usdt"]) for r in rows) >= 0
            else "OTHER"
        ),
    }


def run(p4_path: Path, outcomes_path: Path) -> Dict[str, Any]:
    p4 = load_json(p4_path) if p4_path.exists() else {}
    rows = [
        r
        for r in load_jsonl(outcomes_path)
        if r.get("entry_time_ms") is not None and r.get("net_pnl_usdt") is not None
    ]
    for r in rows:
        r["_day"] = day_ist(r.get("entry_time_ms"))
        r["_win"] = window(r["_day"]) if r["_day"] else "other"

    by_w: Dict[str, List[Dict[str, Any]]] = defaultdict(list)
    by_day: Dict[str, List[Dict[str, Any]]] = defaultdict(list)
    for r in rows:
        by_w[r["_win"]].append(r)
        by_day[str(r["_day"])].append(r)

    # Prefer P4 frozen aggregates when present (canonical)
    frozen_windows = {}
    for w in ("discovery", "validation", "confirmation"):
        if p4.get(w):
            frozen_windows[w] = {
                "source": "p4_frozen_json",
                "aggregate": p4[w]["aggregate"],
                "dispersion": p4[w].get("dispersion"),
                "concentration": p4[w].get("concentration"),
            }

    live_windows = {w: slice_econ(by_w.get(w, [])) for w in ("discovery", "validation", "confirmation")}

    session_econ = []
    for day in sorted(by_day):
        session_econ.append({"session_day_ist": day, "window": window(day), **slice_econ(by_day[day])})

    # Narrative factors from frozen P4 validation + discovery
    factors = []
    if p4.get("validation"):
        va = p4["validation"]["aggregate"]
        if va.get("payoff") is not None and va["payoff"] < 1:
            factors.append(
                {
                    "factor": "payoff_asymmetry_against_system",
                    "evidence": f"validation payoff={va['payoff']} < 1 (avg loss exceeds avg win)",
                }
            )
        if va.get("net_usdt") is not None and va["net_usdt"] < 0:
            factors.append(
                {
                    "factor": "negative_validation_net",
                    "evidence": f"validation net={va['net_usdt']} USDT on n={va['n']}",
                }
            )
    if p4.get("discovery", {}).get("concentration", {}).get("top_n_share_pct", 0) >= 60:
        c = p4["discovery"]["concentration"]
        factors.append(
            {
                "factor": "session_concentration",
                "evidence": (
                    f"discovery top session {c.get('top_session_by_n')} share "
                    f"{c.get('top_n_share_pct')}% of trades"
                ),
            }
        )
    if p4.get("validation", {}).get("aggregate", {}).get("friction_usdt"):
        factors.append(
            {
                "factor": "modeled_friction_load",
                "evidence": (
                    f"validation friction_usdt={p4['validation']['aggregate']['friction_usdt']} "
                    "(modeled; not observed slippage)"
                ),
            }
        )

    literature_notes = [
        {
            "item": "NSE A 2026-11-05 WR vs net divergence",
            "note": "Prior session reporting: ~51.8% WR with negative net — WR≠economics (ledger/session docs).",
            "status": "referenced_not_recomputed_here",
        },
        {
            "item": "Crypto plan low-WR session example",
            "note": "Prior reporting cited ~16.5% WR with negative net on a weak day — consistent with session-regime + payoff issues.",
            "status": "referenced_not_recomputed_here",
        },
        {
            "item": "H2 P4 validation",
            "note": "Frozen: WR 33.11%, payoff 0.53, net −747.14 USDT",
            "status": "from_frozen_p4",
        },
    ]

    return {
        "protocol": PROTOCOL,
        "authority": "0.0%",
        "gate2": "CLOSED",
        "opens_entry_hypothesis": False,
        "changes_stops_targets_execution": False,
        "p4_path": str(p4_path),
        "outcomes_path": str(outcomes_path),
        "outcomes_eligible_n": len(rows),
        "frozen_p4_windows": frozen_windows,
        "live_recompute_windows": live_windows,
        "live_vs_frozen_note": (
            "If live_recompute differs from frozen_p4_windows, prefer frozen P4 "
            "(canonical Pi snapshot). Investigate dataset staleness — do not thr-scan."
        ),
        "session_economics": session_econ,
        "candidate_economic_factors": factors,
        "prior_result_notes": literature_notes,
        "conclusion": {
            "wr_and_profitability_diverge": True,
            "primary_frozen_evidence": "H2 P4 validation payoff<1 with material negative net despite mid-30s WR",
            "not_established": [
                "Which single lever (size of losses vs friction vs session mix) is causal",
                "Any authorized change to stops/targets/execution",
            ],
            "next_permitted": (
                "Optional deeper accounting on frozen exports only; no entry-feature search; "
                "no system change without governance"
            ),
        },
    }


def render_md(rep: Dict[str, Any]) -> str:
    lines = [
        "# RP-608 — Existing Economic-Result Audit (WR vs Profitability)",
        "",
        f"**Protocol:** `{rep['protocol']}` · Gate 2 {rep['gate2']} · no entry hypothesis opened",
        "",
        "## Conclusion",
        "",
        f"- WR and profitability **diverge** on frozen evidence: `{rep['conclusion']['primary_frozen_evidence']}`",
        f"- Not established: {rep['conclusion']['not_established']}",
        f"- Next permitted: {rep['conclusion']['next_permitted']}",
        "",
        "## Frozen P4 windows",
        "",
    ]
    for w, block in (rep.get("frozen_p4_windows") or {}).items():
        a = block["aggregate"]
        lines.append(
            f"- **{w}**: n={a.get('n')} WR={a.get('wr_pct')}% W/L={a.get('wins')}/{a.get('losses')} "
            f"payoff={a.get('payoff')} net={a.get('net_usdt')} friction={a.get('friction_usdt')}"
        )
    lines.extend(["", "## Candidate factors (descriptive)", ""])
    for f in rep.get("candidate_economic_factors") or []:
        lines.append(f"- **{f['factor']}**: {f['evidence']}")
    lines.extend(["", "## Prior referenced results", ""])
    for n in rep.get("prior_result_notes") or []:
        lines.append(f"- {n['item']}: {n['note']} _{n['status']}_")
    lines.extend(
        [
            "",
            "## Governance",
            "",
            "- No stops/targets/horizon/execution changes",
            "- No feature search / thr scan",
            "- Modeled friction remains modeled",
            "",
        ]
    )
    return "\n".join(lines)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--p4",
        default="artifacts/rp608_econ_edge_diag/rp608_h2_crypto_plan_session_regime_p4.json",
    )
    ap.add_argument(
        "--outcomes",
        default="live_capture/shadow_validation/crypto/trade_plan_overlay_v1/trade_plan_outcomes.jsonl",
    )
    ap.add_argument("--out-dir", default="artifacts/rp608_econ_edge_diag")
    args = ap.parse_args()
    out = Path(args.out_dir)
    out.mkdir(parents=True, exist_ok=True)
    rep = run(Path(args.p4), Path(args.outcomes))
    stem = out / "rp608_existing_economic_result_audit"
    stem.with_suffix(".json").write_text(json.dumps(rep, indent=2), encoding="utf-8")
    stem.with_suffix(".md").write_text(render_md(rep), encoding="utf-8")
    print(
        json.dumps(
            {
                "status": "OK",
                "outcomes_n": rep["outcomes_eligible_n"],
                "factors": [f["factor"] for f in rep["candidate_economic_factors"]],
                "md": str(stem.with_suffix(".md")),
            },
            indent=2,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
