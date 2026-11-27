#!/usr/bin/env python3
"""
RP-608-SCI-ADAPTER-REC — S0–S7 diagnostic ledger (authority 0%).

Diagnose-first charter: attribute exclusions and economics without remine.
Books never pooled. Open plans are not closed W/L.

Usage (on Pi or local journal mount):
  python3 scripts/research/rp608_sci_adapter_rec_diag_ledger.py
  python3 scripts/research/rp608_sci_adapter_rec_diag_ledger.py --book SCI
  python3 scripts/research/rp608_sci_adapter_rec_diag_ledger.py --json
"""

from __future__ import annotations

import argparse
import json
from collections import Counter
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional

WORKSPACE = Path(__file__).resolve().parents[2]
CRYPTO_ROOT = WORKSPACE / "live_capture/shadow_validation/crypto"

BOOKS = {
    "SCI": "trade_plan_overlay_sci_adapter_rec_v1",
    "SCI_CLEAN": "trade_plan_overlay_sci_adapter_rec_clean_v1",
    "SCI_ETH": "trade_plan_overlay_sci_adapter_rec_eth_v1",
    "PRIMARY": "trade_plan_overlay_v1",
    "ADAPTER_SELECT": "trade_plan_overlay_adapter_select_15m_v1",
    "GATE2E_B": "trade_plan_overlay_trend_agree_parallel_v1",
}

# Upstream / stack exclusions (not SCI cut ownership).
UPSTREAM_SUPPRESS = {
    "SUPPRESSED_GATE2E_DI_SKEW",
    "SUPPRESSED_GATE2E_REGIME_ALIGN",
    "SUPPRESSED_GATE2E_REGIME_DIRECTIONAL",
    "SUPPRESSED_GATE2E_PATH_INVALID",
    "SUPPRESSED_GATE2E_TREND_AGREE",
    "SUPPRESSED_OPEN_PLAN",
    "SUPPRESSED_ADAPTER_SELECT_15M",
}

SCI_OWNED_SUPPRESS = {
    "SUPPRESSED_SCI_ADAPTER_REC",
}


def _load_jsonl(path: Path, max_bytes: int = 32_000_000) -> List[Dict[str, Any]]:
    if not path.exists():
        return []
    rows: List[Dict[str, Any]] = []
    with open(path, "rb") as f:
        f.seek(0, 2)
        f.seek(max(0, f.tell() - int(max_bytes)))
        for line in f.read().decode("utf-8", "ignore").splitlines():
            if not line.strip():
                continue
            try:
                rows.append(json.loads(line))
            except json.JSONDecodeError:
                continue
    return rows


def _f(row: Dict[str, Any], *keys: str) -> Optional[float]:
    for k in keys:
        if row.get(k) is not None:
            try:
                return float(row[k])
            except (TypeError, ValueError):
                continue
    return None


def _s0_from_cards(cards: List[Dict[str, Any]]) -> Dict[str, Any]:
    """Primary Coralys/policy_b directional recommendation events."""
    pb = Counter(str(c.get("policy_b_action") or "NONE") for c in cards)
    disp = Counter(str(c.get("recommendation_disposition") or "NONE") for c in cards)
    directional = int(pb.get("LONG", 0) + pb.get("SHORT", 0))
    return {
        "decision_cards_scanned": len(cards),
        "policy_b_directional": directional,
        "policy_b_counts": dict(pb.most_common(8)),
        "disposition_counts": dict(disp.most_common(8)),
        "note": "S0 = policy_b LONG+SHORT in scanned decision-card tail (not pooled)",
    }


def ledger_for(
    root: Path,
    book_key: str,
    cards: Optional[List[Dict[str, Any]]] = None,
) -> Dict[str, Any]:
    rel = BOOKS[book_key]
    base = root / rel
    events = _load_jsonl(base / "trade_plan_events.jsonl")
    outcomes = _load_jsonl(base / "trade_plan_outcomes.jsonl")
    state_path = base / "overlay_state.json"
    state: Dict[str, Any] = {}
    if state_path.exists():
        try:
            state = json.loads(state_path.read_text())
        except json.JSONDecodeError:
            state = {}

    et_counts: Counter[str] = Counter()
    for e in events:
        et_counts[str(e.get("event_type") or "?")] += 1

    upstream = {k: et_counts[k] for k in sorted(UPSTREAM_SUPPRESS) if et_counts.get(k)}
    sci_owned = {k: et_counts[k] for k in sorted(SCI_OWNED_SUPPRESS) if et_counts.get(k)}
    plan_created = int(et_counts.get("PLAN_CREATED") or 0)
    # S2 proxy: events that imply SCI path was entered (SCI suppress or post-SCI stack).
    s2_proxy = (
        plan_created
        + int(et_counts.get("SUPPRESSED_SCI_ADAPTER_REC") or 0)
        + int(et_counts.get("SUPPRESSED_GATE2E_DI_SKEW") or 0)
        + int(et_counts.get("SUPPRESSED_GATE2E_PATH_INVALID") or 0)
        + int(et_counts.get("SUPPRESSED_GATE2E_REGIME_ALIGN") or 0)
    )
    s0 = _s0_from_cards(cards or [])

    # S4–S7 from outcomes (closed only).
    closed = [o for o in outcomes if o.get("exit_reason") or o.get("exit_time_ms")]
    wins: List[float] = []
    losses: List[float] = []
    gross_sum = 0.0
    friction_sum = 0.0
    net_sum = 0.0
    for o in closed:
        gross = _f(o, "gross_pnl_usdt", "gross_pnl") or 0.0
        fr = _f(o, "friction_usdt", "est_friction_usdt") or 0.0
        net = _f(o, "net_pnl_usdt", "net_pnl")
        if net is None:
            net = gross - fr
        gross_sum += gross
        friction_sum += fr
        net_sum += net
        if net > 0:
            wins.append(net)
        else:
            losses.append(abs(net))

    n = len(wins) + len(losses)
    p_w = (len(wins) / n) if n else None
    p_l = (len(losses) / n) if n else None
    avg_win = (sum(wins) / len(wins)) if wins else None
    avg_loss = (sum(losses) / len(losses)) if losses else None
    avg_friction = (friction_sum / n) if n else None
    gross_exp = None
    net_exp = None
    if p_w is not None and avg_win is not None and avg_loss is not None:
        gross_exp = p_w * avg_win - (p_l or 0.0) * avg_loss
        if avg_friction is not None:
            net_exp = gross_exp - avg_friction

    s1_upstream_blocks = sum(upstream.values())
    s3_accepted = plan_created
    open_n = 1 if (state.get("active_plan") or {}).get("state") == "OPEN" else 0
    s0_dir = int(s0.get("policy_b_directional") or 0)
    net_per_eligible = (
        round(net_sum / s0_dir, 6) if s0_dir > 0 else None
    )

    return {
        "book_key": book_key,
        "dir": rel,
        "armed": base.exists(),
        "generated_utc": datetime.now(timezone.utc).isoformat(),
        "stages": {
            "S0_coralys_primary_events": s0,
            "S1_upstream_eligible_proxy": {
                "note": "upstream suppress ledger (DI_SKEW/PATH/OPEN_PLAN); not adapter-owned",
                "upstream_suppress_total": s1_upstream_blocks,
                "exclusion_ledger": upstream,
            },
            "S2_evaluated_by_sci_proxy": {
                "note": "PLAN_CREATED + SCI_SUPPRESS + DI_SKEW + PATH + REGIME (post/with SCI path)",
                "n": s2_proxy,
            },
            "S3_accepted_by_sci": {"PLAN_CREATED": s3_accepted},
            "S4_closed_outcomes": {"n": n, "open_active": open_n},
            "S5_wl": {
                "W": len(wins),
                "L": len(losses),
                "wr": round(len(wins) / n, 4) if n else None,
            },
            "S6_gross_friction": {
                "gross_sum": round(gross_sum, 4),
                "friction_sum": round(friction_sum, 4),
                "avg_friction": round(avg_friction, 4) if avg_friction is not None else None,
                "avg_win_net": round(avg_win, 4) if avg_win is not None else None,
                "avg_loss_abs_net": round(avg_loss, 4) if avg_loss is not None else None,
            },
            "S7_net": {
                "net_sum": round(net_sum, 4),
                "gross_expectancy": round(gross_exp, 4) if gross_exp is not None else None,
                "net_expectancy": round(net_exp, 4) if net_exp is not None else None,
                "net_per_accepted_plan": round(net_sum / s3_accepted, 4) if s3_accepted else None,
                "net_per_eligible_s0_directional": net_per_eligible,
            },
        },
        "ownership": {
            "upstream_exclusion_ledger": upstream,
            "sci_owned_suppress": sci_owned,
            "other_event_top": et_counts.most_common(12),
        },
        "incomplete": [
            "S2 is a proxy until per-card SCI stamps are journaled on decision cards",
            "S0 uses decision-card file tail (max_bytes), not full history",
        ],
        "rules": "chronosentiment-core §3b; diagnose-first charter; never pool books",
    }


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", type=Path, default=CRYPTO_ROOT)
    ap.add_argument(
        "--cards",
        type=Path,
        default=None,
        help="decision cards jsonl (default: <root>/rp608_decision_cards.jsonl)",
    )
    ap.add_argument("--book", choices=sorted(BOOKS.keys()), default="SCI")
    ap.add_argument("--all", action="store_true")
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()

    cards_path = args.cards or (args.root / "rp608_decision_cards.jsonl")
    cards = _load_jsonl(cards_path, max_bytes=8_000_000)
    keys = list(BOOKS.keys()) if args.all else [args.book]
    reports = [ledger_for(args.root, k, cards=cards) for k in keys]
    if args.json:
        print(json.dumps(reports if args.all else reports[0], indent=2))
        return 0

    for r in reports:
        print(f"[DIAG-LEDGER] book={r['book_key']} armed={r['armed']} utc={r['generated_utc']}")
        s = r["stages"]
        s0 = s["S0_coralys_primary_events"]
        print(
            f"  S0 cards={s0.get('decision_cards_scanned')} "
            f"policy_b_directional={s0.get('policy_b_directional')} "
            f"pb={s0.get('policy_b_counts')}"
        )
        print(f"  S1 upstream_suppress_total={s['S1_upstream_eligible_proxy']['upstream_suppress_total']}")
        for k, v in (s["S1_upstream_eligible_proxy"]["exclusion_ledger"] or {}).items():
            print(f"    {k}={v}")
        print(f"  S2 sci_path_proxy={s['S2_evaluated_by_sci_proxy']['n']}")
        print(f"  S3 PLAN_CREATED={s['S3_accepted_by_sci']['PLAN_CREATED']}")
        s5 = s["S5_wl"]
        print(f"  S5 W/L={s5['W']}/{s5['L']} n={s['S4_closed_outcomes']['n']} wr={s5['wr']} open={s['S4_closed_outcomes']['open_active']}")
        s6 = s["S6_gross_friction"]
        s7 = s["S7_net"]
        print(
            f"  S6 gross={s6['gross_sum']} friction={s6['friction_sum']} "
            f"avg_fric={s6['avg_friction']} avg_win={s6['avg_win_net']} avg_loss={s6['avg_loss_abs_net']}"
        )
        print(
            f"  S7 net={s7['net_sum']} gross_exp={s7['gross_expectancy']} "
            f"net_exp={s7['net_expectancy']} net_per_plan={s7['net_per_accepted_plan']} "
            f"net_per_s0={s7['net_per_eligible_s0_directional']}"
        )
        print(f"  incomplete: {', '.join(r['incomplete'])}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
