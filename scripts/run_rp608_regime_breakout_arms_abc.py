#!/usr/bin/env python3
"""
RP-608-REGIME-BREAKOUT-001 Phase 4: Reject-preserving Arms A/B/C harness (IS only).

Sessions/time-blocks 1–40 only. Holdout 41+ sealed.
Reports taken-trade and opportunity-level metrics separately.
Paired incremental returns vs Arm A counterfactual.
Live order authority: 0.0%.
"""

from __future__ import annotations

import json
import math
import random
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional, Sequence, Tuple

WORKSPACE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE))

from scripts.breakout_lifecycle import LifecycleState, evaluate_quote, pack_bars_for_breakout_eval
from scripts.market_regime_classifier import classify_market_regime
from scripts.nse_market_context import apply_market_chop_to_regime, compute_market_context
from scripts.rp608_arm_a_baseline import (
    NseBar,
    nse_arm_a_decide_entry,
    nse_arm_a_simulate_trade,
)

BOOTSTRAP_SEED = 20261028
HOLDOUT_START_IDX = 40  # 0-based: sessions[40:] sealed


def holm_bonferroni(p_values: Sequence[float], alpha: float = 0.05) -> List[Dict[str, Any]]:
    indexed = sorted(enumerate(p_values), key=lambda x: x[1])
    m = len(p_values)
    out = [{"raw_p": p, "reject": False, "adjusted_alpha": None} for p in p_values]
    for rank, (idx, p) in enumerate(indexed):
        adj_alpha = alpha / (m - rank)
        out[idx]["adjusted_alpha"] = adj_alpha
        out[idx]["reject"] = p < adj_alpha
        if not out[idx]["reject"]:
            # stop step-down
            break
    return out


def paired_bootstrap_p(
    deltas: Sequence[float],
    n_boot: int = 10000,
    seed: int = BOOTSTRAP_SEED,
) -> float:
    """One-sided p for mean(delta) > 0 using paired bootstrap."""
    if not deltas:
        return 1.0
    rng = random.Random(seed)
    n = len(deltas)
    mean0 = sum(deltas) / n
    if mean0 <= 0:
        # still compute for reporting
        pass
    count = 0
    for _ in range(n_boot):
        sample = [deltas[rng.randrange(n)] for _ in range(n)]
        if (sum(sample) / n) <= 0:
            count += 1
    return count / n_boot


def profit_factor(nets: Sequence[float]) -> Any:
    wins = [x for x in nets if x > 0]
    losses = [x for x in nets if x < 0]
    if not wins and not losses:
        return None
    if wins and not losses:
        return float("inf")
    return sum(wins) / abs(sum(losses))


def run_synthetic_smoke() -> Dict[str, Any]:
    """
    Deterministic synthetic opportunity universe for harness validation.
    Not an IS promotion result — labeled diagnostic_replay.
    """
    opportunities: List[Dict[str, Any]] = []
    # Build mild uptrend bars
    bars = []
    px = 1000.0
    for i in range(1, 30):
        px *= 1.003
        bars.append(NseBar(i, px * 0.999, px * 1.001, px * 0.998, px))

    for entry_bar in (8, 10, 12, 14, 16):
        closes = [b.close for b in bars[:entry_bar]]
        decision = nse_arm_a_decide_entry(closes, bar_idx=entry_bar)
        if not decision.get("signal_entry"):
            continue
        trade = nse_arm_a_simulate_trade(entry_bar, bars[entry_bar - 1].close, bars[entry_bar:])
        regime = classify_market_regime(
            [{"open": b.open, "high": b.high, "low": b.low, "close": b.close} for b in bars[:entry_bar]]
        )
        mctx = compute_market_context([20.0, -5.0, 15.0, 8.0, -2.0])
        regime = apply_market_chop_to_regime(regime, mctx)

        # Breakout evaluate at entry — same packing as historical NSE/crypto harness
        completed = [
            {"open": b.open, "high": b.high, "low": b.low, "close": b.close} for b in bars[:entry_bar]
        ]
        lc = evaluate_quote(
            state=LifecycleState(),
            regime=regime.get("regime", "UNAVAILABLE"),
            bars_completed_plus_forming=pack_bars_for_breakout_eval(completed),
            ask=bars[entry_bar - 1].close * 1.001,
            bid=bars[entry_bar - 1].close * 0.999,
            quote_age_ms=100.0,
            session_date="SYNTH",
            symbol="TEST.NS",
            half_spread_cap_bps=1.80,
            target_distance=150.0,
            stop_distance=100.0,
            friction=10.0,
        )

        arm_a_net = trade["net_return_bps"]
        take_b = regime.get("regime") == "DIRECTIONAL"
        take_c = take_b and lc.state == "TRADE_QUALIFIED"

        def outcome(taken: bool) -> Tuple[float, str, str]:
            if taken:
                return arm_a_net, "taken", "taken"
            return arm_a_net, "arm_a_counterfactual", "regime_reject" if not take_b else "breakout_reject"

        b_net, b_src, b_disp = outcome(take_b)
        c_net, c_src, c_disp = outcome(take_c)
        if take_b:
            b_disp = "taken"
        if take_c:
            c_disp = "taken"

        opportunities.append({
            "opportunity_id": f"SYNTH|{entry_bar}|LONG",
            "arm_a_net_bps": arm_a_net,
            "arm_b": {"taken": take_b, "opp_net_bps": b_net, "outcome_source": b_src, "disposition": b_disp},
            "arm_c": {"taken": take_c, "opp_net_bps": c_net, "outcome_source": c_src, "disposition": c_disp},
            "regime": regime.get("regime"),
            "breakout_state": lc.state,
        })

    # Metrics
    a_nets = [o["arm_a_net_bps"] for o in opportunities]
    b_opp = [o["arm_b"]["opp_net_bps"] for o in opportunities]
    c_opp = [o["arm_c"]["opp_net_bps"] for o in opportunities]
    delta_b = [b - a for a, b in zip(a_nets, b_opp)]
    delta_c = [c - a for a, c in zip(a_nets, c_opp)]
    delta_cb = [c - b for b, c in zip(b_opp, c_opp)]

    p_b = paired_bootstrap_p(delta_b)
    p_c_inc = paired_bootstrap_p(delta_cb)
    p_c_a = paired_bootstrap_p(delta_c)
    holm = holm_bonferroni([p_b, p_c_inc, p_c_a])

    n_univ = len(opportunities)
    cov_b = sum(1 for o in opportunities if o["arm_b"]["taken"]) / n_univ if n_univ else 0.0
    cov_c = sum(1 for o in opportunities if o["arm_c"]["taken"]) / n_univ if n_univ else 0.0

    report = {
        "protocol": "RP-608-REGIME-BREAKOUT-001",
        "phase": 4,
        "label": "diagnostic_replay",
        "live_order_authority": "0.0%",
        "holdout": "SEALED_UNTOUCHED",
        "partition": "synthetic_smoke_not_is_promotion",
        "n_universe": n_univ,
        "arm_a": {
            "opp_expectancy_bps": round(sum(a_nets) / n_univ, 4) if n_univ else None,
            "profit_factor": profit_factor(a_nets),
        },
        "arm_b": {
            "coverage": round(cov_b, 4),
            "opp_expectancy_bps": round(sum(b_opp) / n_univ, 4) if n_univ else None,
            "mean_delta_vs_a": round(sum(delta_b) / n_univ, 4) if n_univ else None,
            "bootstrap_p_one_sided": p_b,
        },
        "arm_c": {
            "coverage": round(cov_c, 4),
            "opp_expectancy_bps": round(sum(c_opp) / n_univ, 4) if n_univ else None,
            "mean_delta_vs_a": round(sum(delta_c) / n_univ, 4) if n_univ else None,
            "mean_delta_vs_b": round(sum(delta_cb) / n_univ, 4) if n_univ else None,
            "bootstrap_p_H_C_inc": p_c_inc,
            "bootstrap_p_H_C_vs_A": p_c_a,
        },
        "holm_bonferroni": {
            "family": ["H_B", "H_C_inc", "H_C_vs_A"],
            "results": holm,
        },
        "promotion": "INCONCLUSIVE",
        "promotion_reason": "Synthetic smoke harness only; not IS Sessions 1-40 evaluation",
        "opportunities": opportunities,
    }
    return report


def main() -> None:
    out_dir = WORKSPACE / "artifacts" / "rp608_regime_breakout"
    out_dir.mkdir(parents=True, exist_ok=True)
    report = run_synthetic_smoke()
    path = out_dir / "phase4_arms_abc_smoke_report.json"
    path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(f"Wrote {path}")
    print(f"promotion={report['promotion']} n={report['n_universe']}")


if __name__ == "__main__":
    main()
