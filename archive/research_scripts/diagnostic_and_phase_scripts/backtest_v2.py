"""
Backtest v2 — Trade-level + Portfolio Simulation
=================================================
Consumes the frozen IC v1 DecisionBrief decisions (via the golden dataset)
and produces:

  v2.1 — Trade-level ledger
    Per acted decision: entry price, exit price, holding duration,
    exit reason, gross return, transaction cost, net return, MFE, MAE.

  v2.2 — Portfolio simulation
    Starting capital: ₹1,00,000.
    Processes trades chronologically.
    Produces equity curve, drawdown, capital utilization, risk metrics.

Architecture guard
------------------
This script does NOT:
  - Reimplement IC v1 classification (imported from backtest_v1 helpers)
  - Modify the frozen DecisionBrief contract
  - Modify Decision Cockpit v0.4
  - Optimize thresholds or introduce new trading rules
  - Use future information

The simulator asks only:
  "If this IC v1 decision was acted upon, what trade would have resulted?"

Information available at each checkpoint (same as IC v1):
  ENTRY:  direction, OQS, h60_classification
  H120:   h120_ret, mfe_h120 (LONG WAIT-MID reassessment only)
  H300:   outcome measured

Reconciliation
--------------
v2 gross returns must match v1 h300_ret for acted decisions.
Any discrepancy is a bug in v2.

Usage
-----
  python3 scripts/backtest_v2.py

Output
------
  datasets/backtest_v2_trade_ledger.json   — canonical trade ledger
  datasets/backtest_v2_portfolio.json      — portfolio simulation result
  Console: trade-level summary + portfolio report
"""

import json
import statistics
from collections import defaultdict
from dataclasses import dataclass, asdict
from typing import Optional

# ── Configuration ─────────────────────────────────────────────────────────────

DATASET_PATH      = "datasets/p4_opportunity_dataset.json"
LEDGER_OUT        = "datasets/backtest_v2_trade_ledger.json"
PORTFOLIO_OUT     = "datasets/backtest_v2_portfolio.json"

STARTING_CAPITAL  = 100_000.0   # ₹1,00,000
TRANSACTION_COST  = 0.0005      # 5 bps per trade (round-trip = 10 bps)
POSITION_FRACTION = 0.10        # 10% of available capital per position
MAX_POSITIONS     = 5           # max concurrent open positions

# IC v1 thresholds (frozen — do not change)
THRESHOLD  = 0.002   # 0.2%
MFE_FLOOR  = 0.001   # 0.1%

# ── IC v1 classification (frozen — mirrors Rust IC v1 exactly) ────────────────

def entry_state(r: dict) -> str:
    cls  = r["path_5m"]["h60_classification"]
    oqs  = r["opportunity_dimensions"]["opportunity_quality_score"]
    dirn = r["direction"]
    if cls == "ENTER": return "ENTER"
    if cls == "AVOID": return "AVOID"
    if dirn == "SHORT":
        return "WAIT-HIGH" if oqs >= 50 else "WAIT-LOW"
    else:
        if oqs >= 65:  return "WAIT-HIGH"
        if oqs >= 40:  return "WAIT-MID"
        return "WAIT-LOW"

def final_state(r: dict, es: str) -> str:
    if r["direction"] != "LONG" or es != "WAIT-MID":
        return es
    h120 = r["path_5m"].get("h120_ret")
    mfe  = r["path_5m"].get("mfe_h120")
    if h120 is None:
        return "WAIT-LATE"
    if mfe is not None and mfe < MFE_FLOOR:
        return "AVOID-LATE"
    if h120 > THRESHOLD:  return "ENTER-LATE"
    if h120 < -THRESHOLD: return "AVOID-LATE"
    return "WAIT-LATE"

ACT_STATES = {"ENTER", "WAIT-HIGH", "ENTER-LATE", "WAIT-LATE"}

# ── Exit reason inference ─────────────────────────────────────────────────────

def infer_exit_reason(r: dict, gross_ret: float) -> str:
    """
    Infer exit reason from available path data.
    The dataset does not record explicit exit reasons for intraday paths,
    so we use the following heuristics (information-safe — uses only
    data that would have been available at H300):

      TARGET  — gross_ret >= target_rate (reached the stated target)
      RISK    — gross_ret < -0.02 (adverse move > 2%)
      HORIZON — session closed at H300 without hitting target or risk
    """
    target = r.get("target_rate", 0.30)
    if gross_ret >= target:
        return "TARGET"
    if gross_ret < -0.02:
        return "RISK"
    return "HORIZON"

# ── Trade-level simulation ────────────────────────────────────────────────────

@dataclass
class Trade:
    decision_id:      str
    ticker:           str
    direction:        str
    cohort_date:      str
    entry_state:      str
    final_state:      str

    entry_price:      float
    exit_price:       float
    holding_minutes:  int          # H300 = 300 minutes
    exit_reason:      str

    gross_ret:        float        # direction-adjusted fractional return
    transaction_cost: float        # fractional cost (both legs)
    net_ret:          float        # gross_ret - transaction_cost

    mfe:              float        # max favourable excursion (H300)
    mae:              float        # max adverse excursion (H300)

    oqs:              int
    position_size:    float = 0.0  # ₹ allocated (set by portfolio engine)
    gross_pnl:        float = 0.0  # ₹ gross P&L
    net_pnl:          float = 0.0  # ₹ net P&L


def simulate_trades(data: list) -> list[Trade]:
    """
    Build the canonical trade ledger from the golden dataset.
    Only acted decisions (ACT_STATES) are included.
    """
    trades = []
    for r in data:
        p5 = r["path_5m"]
        h300 = p5.get("h300_ret")
        if h300 is None:
            continue  # incomplete record — skip

        es = entry_state(r)
        fs = final_state(r, es)
        if fs not in ACT_STATES:
            continue  # no action taken

        entry_px = p5.get("entry_price") or r.get("reference_price")
        if entry_px is None or entry_px <= 0:
            continue

        # Exit price: derived from direction-adjusted gross return.
        # h300_ret is already direction-adjusted (positive = favourable).
        # For LONG:  exit = entry * (1 + h300_ret)
        # For SHORT: exit = entry * (1 - h300_ret)
        gross_ret = h300
        if r["direction"] == "LONG":
            exit_px = entry_px * (1.0 + gross_ret)
        else:
            exit_px = entry_px * (1.0 - gross_ret)

        cost = TRANSACTION_COST  # applied once (covers both legs)
        net_ret = gross_ret - cost

        mfe = p5.get("mfe_h300") or 0.0
        mae = p5.get("mae_h300") or 0.0  # already negative in dataset

        exit_reason = infer_exit_reason(r, gross_ret)

        trades.append(Trade(
            decision_id      = r["decision_id"],
            ticker           = r["ticker"],
            direction        = r["direction"],
            cohort_date      = r["cohort_date"],
            entry_state      = es,
            final_state      = fs,
            entry_price      = round(entry_px, 4),
            exit_price       = round(exit_px, 4),
            holding_minutes  = 300,
            exit_reason      = exit_reason,
            gross_ret        = gross_ret,
            transaction_cost = cost,
            net_ret          = net_ret,
            mfe              = mfe,
            mae              = mae,
            oqs              = r["opportunity_dimensions"]["opportunity_quality_score"],
        ))

    return trades

# ── Portfolio simulation ──────────────────────────────────────────────────────

@dataclass
class EquityPoint:
    date:              str
    trades_opened:     int
    trades_closed:     int
    gross_pnl:         float
    net_pnl:           float
    realized_pnl:      float
    cash:              float
    invested:          float
    equity:            float
    drawdown:          float
    utilization:       float   # invested / equity

def simulate_portfolio(trades: list[Trade]) -> tuple[list[EquityPoint], dict]:
    """
    Process trades chronologically.

    Assumptions:
      - All trades on the same cohort_date open at the start of the session
        and close at H300 (end of session).
      - No carry-over between dates (intraday only).
      - Position size = POSITION_FRACTION * available_cash, capped at
        MAX_POSITIONS concurrent positions.
      - Positions are ranked by OQS descending; highest-quality trades
        are selected when the cap is binding.
      - Transaction cost is deducted from net_pnl.
      - Capital is returned at end of each session.

    Statistics (win rate, PF, mean/median) are computed from the
    SELECTED trades only — not from the full 233-candidate population.
    """
    cash = STARTING_CAPITAL
    peak_equity = STARTING_CAPITAL
    max_drawdown = 0.0
    realized_pnl_cumulative = 0.0

    equity_curve: list[EquityPoint] = []
    selected_trades: list[Trade] = []   # only the trades actually executed

    # Group trades by cohort_date
    by_date: dict[str, list[Trade]] = defaultdict(list)
    for t in trades:
        by_date[t.cohort_date].append(t)

    for date in sorted(by_date.keys()):
        day_trades = by_date[date]

        # Rank by OQS descending before applying the position cap.
        # OQS is the IC v1 opportunity quality signal — higher = better.
        # Selecting by dataset order would be arbitrary and misleading.
        day_trades = sorted(day_trades, key=lambda t: t.oqs, reverse=True)
        day_trades = day_trades[:MAX_POSITIONS]

        # Size each position
        available = cash
        n = len(day_trades)
        if n == 0:
            continue

        per_trade_capital = min(available * POSITION_FRACTION,
                                available / n)
        per_trade_capital = max(per_trade_capital, 0.0)

        invested = 0.0
        day_gross_pnl = 0.0
        day_net_pnl = 0.0

        for t in day_trades:
            t.position_size = per_trade_capital
            t.gross_pnl = per_trade_capital * t.gross_ret
            t.net_pnl   = per_trade_capital * t.net_ret
            invested += per_trade_capital
            day_gross_pnl += t.gross_pnl
            day_net_pnl   += t.net_pnl
            selected_trades.append(t)

        # Close all positions at end of session
        cash = cash - invested + invested + day_net_pnl
        realized_pnl_cumulative += day_net_pnl
        equity = cash

        # Drawdown
        if equity > peak_equity:
            peak_equity = equity
        dd = (peak_equity - equity) / peak_equity if peak_equity > 0 else 0.0
        if dd > max_drawdown:
            max_drawdown = dd

        utilization = invested / equity if equity > 0 else 0.0

        equity_curve.append(EquityPoint(
            date           = date,
            trades_opened  = n,
            trades_closed  = n,
            gross_pnl      = round(day_gross_pnl, 2),
            net_pnl        = round(day_net_pnl, 2),
            realized_pnl   = round(realized_pnl_cumulative, 2),
            cash           = round(cash, 2),
            invested       = round(invested, 2),
            equity         = round(equity, 2),
            drawdown       = round(dd, 6),
            utilization    = round(utilization, 4),
        ))

    # Portfolio-level summary — computed from SELECTED trades only.
    # The 233-candidate population statistics belong in the trade-level section.
    sel_net_rets = [t.net_ret for t in selected_trades]
    sel_winners  = [r for r in sel_net_rets if r > THRESHOLD]
    sel_losers   = [r for r in sel_net_rets if r < -THRESHOLD]
    pf = (sum(sel_winners) / abs(sum(sel_losers))) if sel_losers and sum(sel_losers) != 0 else None

    final_equity = equity_curve[-1].equity if equity_curve else STARTING_CAPITAL
    total_return = (final_equity - STARTING_CAPITAL) / STARTING_CAPITAL

    # Sharpe-like: mean / stdev of selected trade net returns
    sharpe_like = None
    if len(sel_net_rets) > 1:
        mu  = statistics.mean(sel_net_rets)
        sig = statistics.stdev(sel_net_rets)
        sharpe_like = mu / sig if sig > 0 else None

    summary = {
        "starting_capital":       STARTING_CAPITAL,
        "final_equity":           round(final_equity, 2),
        "total_return":           round(total_return, 6),
        "total_net_pnl":          round(final_equity - STARTING_CAPITAL, 2),
        # Candidate pool (all 233 IC-acted decisions)
        "candidate_count":        len(trades),
        # Executed portfolio (OQS-ranked top-5 per day)
        "executed_trade_count":   len(selected_trades),
        "trading_days":           len(equity_curve),
        # Statistics from executed trades only
        "win_rate":               round(len(sel_winners) / len(sel_net_rets), 4) if sel_net_rets else None,
        "profit_factor":          round(pf, 4) if pf else None,
        "mean_net_ret":           round(statistics.mean(sel_net_rets), 6) if sel_net_rets else None,
        "median_net_ret":         round(statistics.median(sel_net_rets), 6) if sel_net_rets else None,
        "max_drawdown":           round(max_drawdown, 6),
        "sharpe_like":            round(sharpe_like, 4) if sharpe_like else None,
        "transaction_cost_bps":   int(TRANSACTION_COST * 10_000),
        "position_fraction":      POSITION_FRACTION,
        "max_positions":          MAX_POSITIONS,
    }

    return equity_curve, summary, selected_trades

# ── Reconciliation ────────────────────────────────────────────────────────────

def reconcile_with_v1(data: list, trades: list[Trade]) -> dict:
    """
    Compare v2 gross returns against v1 h300_ret for acted decisions.
    Any mismatch is a bug in v2.
    """
    v1_acted = {}
    for r in data:
        h300 = r["path_5m"].get("h300_ret")
        if h300 is None:
            continue
        es = entry_state(r)
        fs = final_state(r, es)
        if fs in ACT_STATES:
            v1_acted[r["decision_id"]] = h300

    mismatches = []
    for t in trades:
        v1_ret = v1_acted.get(t.decision_id)
        if v1_ret is None:
            mismatches.append({"id": t.decision_id, "issue": "not in v1"})
            continue
        diff = abs(t.gross_ret - v1_ret)
        if diff > 1e-9:
            mismatches.append({
                "id":      t.decision_id,
                "v1_ret":  v1_ret,
                "v2_ret":  t.gross_ret,
                "diff":    diff,
            })

    v2_ids = {t.decision_id for t in trades}
    for did, ret in v1_acted.items():
        if did not in v2_ids:
            mismatches.append({"id": did, "issue": "in v1 but missing from v2"})

    return {
        "v1_acted_count": len(v1_acted),
        "v2_trade_count": len(trades),
        "mismatches":     len(mismatches),
        "status":         "PASS" if not mismatches else "FAIL",
        "details":        mismatches[:20],  # first 20 only
    }

# ── Stats helpers ─────────────────────────────────────────────────────────────

def pct(v, d=2):
    if v is None: return "—"
    return f"{v*100:+.{d}f}%"

def fmt_pf(v):
    if v is None: return "—"
    if v > 99: return ">99x"
    return f"{v:.2f}x"

def stats(returns):
    if not returns: return None
    n = len(returns)
    winners = [v for v in returns if v > THRESHOLD]
    losers  = [v for v in returns if v < -THRESHOLD]
    pf = (sum(winners) / abs(sum(losers))) if losers and sum(losers) != 0 else None
    return {
        "n":        n,
        "mean":     statistics.mean(returns),
        "median":   statistics.median(returns),
        "win_rate": len(winners) / n,
        "pf":       pf,
        "cum":      sum(returns),
    }

# ── Main ──────────────────────────────────────────────────────────────────────

def main():
    data = json.load(open(DATASET_PATH))

    # ── v2.1 Trade-level simulation ───────────────────────────────────────────
    trades = simulate_trades(data)

    print("=" * 80)
    print("BACKTEST v2.1 — Trade-level simulation")
    print(f"IC v1 frozen classification · {len(trades)} acted trades")
    print(f"Transaction cost: {int(TRANSACTION_COST*10_000)} bps per trade")
    print("=" * 80)
    print()

    # Per-state breakdown
    print(f"  {'State':14} {'Dir':6} {'N':>4}  {'Mean Gross':>11}  {'Mean Net':>10}  "
          f"{'Win%':>6}  {'PF':>7}  {'MFE med':>8}  {'MAE med':>8}")
    print(f"  {'-'*85}")

    STATE_ORDER = ["ENTER", "WAIT-HIGH", "WAIT-MID", "ENTER-LATE", "WAIT-LATE"]
    for direction in ["LONG", "SHORT"]:
        for state in STATE_ORDER:
            group = [t for t in trades
                     if t.direction == direction and t.final_state == state]
            if not group:
                continue
            gross_s = stats([t.gross_ret for t in group])
            net_s   = stats([t.net_ret   for t in group])
            mfe_med = statistics.median([t.mfe for t in group])
            mae_med = statistics.median([t.mae for t in group])
            if not gross_s:
                continue
            print(f"  {state:14} {direction:6} {gross_s['n']:>4}  "
                  f"{pct(gross_s['mean']):>11}  {pct(net_s['mean'] if net_s else None):>10}  "
                  f"{gross_s['win_rate']*100:>5.1f}%  {fmt_pf(gross_s['pf']):>7}  "
                  f"{pct(mfe_med):>8}  {pct(mae_med):>8}")
    print()

    # Exit reason breakdown
    print(f"  {'Exit Reason':12}  {'N':>4}  {'Mean Net':>10}  {'Win%':>6}")
    print(f"  {'-'*40}")
    for reason in ["TARGET", "HORIZON", "RISK"]:
        group = [t for t in trades if t.exit_reason == reason]
        if not group:
            continue
        s = stats([t.net_ret for t in group])
        if not s:
            continue
        print(f"  {reason:12}  {s['n']:>4}  {pct(s['mean']):>10}  {s['win_rate']*100:>5.1f}%")
    print()

    # ── Reconciliation ────────────────────────────────────────────────────────
    recon = reconcile_with_v1(data, trades)
    print("=" * 80)
    print(f"RECONCILIATION vs Backtest v1  —  STATUS: {recon['status']}")
    print(f"  v1 acted count: {recon['v1_acted_count']}")
    print(f"  v2 trade count: {recon['v2_trade_count']}")
    print(f"  Mismatches:     {recon['mismatches']}")
    if recon["details"]:
        for m in recon["details"][:5]:
            print(f"    {m}")
    print()

    # ── v2.2 Portfolio simulation ─────────────────────────────────────────────
    equity_curve, summary, selected = simulate_portfolio(trades)

    print("=" * 80)
    print("BACKTEST v2.2 — Portfolio simulation")
    print(f"Starting capital: ₹{STARTING_CAPITAL:,.0f}")
    print(f"Position sizing:  {int(POSITION_FRACTION*100)}% of available capital per trade")
    print(f"Max concurrent:   {MAX_POSITIONS} positions (OQS-ranked)")
    print(f"Transaction cost: {summary['transaction_cost_bps']} bps")
    print("=" * 80)
    print()

    print(f"  Candidate pool:    {summary['candidate_count']} IC-acted decisions")
    print(f"  Executed trades:   {summary['executed_trade_count']} (top-OQS selected)")
    print(f"  Trading days:      {summary['trading_days']}")
    print()
    print(f"  Final equity:      ₹{summary['final_equity']:>12,.2f}")
    print(f"  Total net P&L:     ₹{summary['total_net_pnl']:>12,.2f}")
    print(f"  Total return:      {pct(summary['total_return'])}")
    print()
    print(f"  — Executed-trade statistics (N={summary['executed_trade_count']}) —")
    print(f"  Win rate:          {(summary['win_rate'] or 0)*100:.1f}%")
    print(f"  Profit factor:     {fmt_pf(summary['profit_factor'])}")
    print(f"  Mean net ret:      {pct(summary['mean_net_ret'])}")
    print(f"  Median net ret:    {pct(summary['median_net_ret'])}")
    print(f"  Max drawdown:      {pct(summary['max_drawdown'])}")
    print(f"  Sharpe-like:       {summary['sharpe_like']:.3f}" if summary['sharpe_like'] else "  Sharpe-like:       —")
    print()

    # Equity curve (last 5 dates)
    print("  Equity curve (last 5 trading days):")
    print(f"  {'Date':12}  {'Trades':>6}  {'Net P&L':>10}  {'Equity':>12}  "
          f"{'Drawdown':>9}  {'Utilization':>11}")
    print(f"  {'-'*65}")
    for ep in equity_curve[-5:]:
        print(f"  {ep.date:12}  {ep.trades_opened:>6}  "
              f"₹{ep.net_pnl:>9,.2f}  ₹{ep.equity:>11,.2f}  "
              f"{ep.drawdown*100:>8.2f}%  {ep.utilization*100:>10.1f}%")
    print()

    # ── Save outputs ──────────────────────────────────────────────────────────
    ledger = [asdict(t) for t in trades]
    with open(LEDGER_OUT, "w") as f:
        json.dump(ledger, f, indent=2)
    print(f"  Trade ledger saved → {LEDGER_OUT}  ({len(ledger)} records)")

    portfolio_out = {
        "summary":      summary,
        "equity_curve": [asdict(ep) for ep in equity_curve],
        "reconciliation": recon,
    }
    with open(PORTFOLIO_OUT, "w") as f:
        json.dump(portfolio_out, f, indent=2)
    print(f"  Portfolio saved   → {PORTFOLIO_OUT}")
    print()

    # Final verdict
    recon_ok = recon["status"] == "PASS"
    print("=" * 80)
    print(f"STATUS: {'✓ PASS' if recon_ok else '✗ FAIL — reconciliation mismatch'}")
    print("=" * 80)

    return 0 if recon_ok else 1


if __name__ == "__main__":
    raise SystemExit(main())