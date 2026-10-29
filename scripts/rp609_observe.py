#!/usr/bin/env python3
"""
RP-609 observation CLI — swing / portfolio build / long-term CNC paper book.

Examples:
  python3 scripts/rp609_observe.py bootstrap
  python3 scripts/rp609_observe.py swing-open --symbol HDFCBANK --price 1650 --qty 10 \\
      --stop 1580 --target 1750 --thesis swing_observe --max-sessions 8
  python3 scripts/rp609_observe.py portfolio-plan --plan-id build_a --capital 500000
  python3 scripts/rp609_observe.py long-open --symbol RELIANCE --price 2800 --qty 5 \\
      --thesis "long-term observe" --version v1
  python3 scripts/rp609_observe.py eod
  python3 scripts/rp609_observe.py report
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

_WORKSPACE = Path(__file__).resolve().parent.parent
if str(_WORKSPACE) not in sys.path:
    sys.path.insert(0, str(_WORKSPACE))

from scripts.rp609_indian_retail_horizons import write_horizon_registry
from scripts.rp609_multi_horizon_engine import (
    DEFAULT_NSE_DIR,
    MultiHorizonEngine,
    load_price_map,
    normalize_symbol,
)


def _engine(args: argparse.Namespace) -> MultiHorizonEngine:
    return MultiHorizonEngine(nse_dir=Path(args.nse_dir))


def cmd_bootstrap(_args: argparse.Namespace) -> int:
    path = write_horizon_registry()
    eng = MultiHorizonEngine()
    eng.persist()
    print(json.dumps({"registry": str(path), "report": eng.report()}, indent=2))
    return 0


def cmd_swing_open(args: argparse.Namespace) -> int:
    eng = _engine(args)
    price = args.price
    if price is None:
        pxmap = load_price_map(Path(args.nse_dir))
        sym = normalize_symbol(args.symbol)
        if sym not in pxmap:
            raise SystemExit(f"no mark for {sym}; pass --price explicitly")
        price = float(pxmap[sym])
    stop = args.stop if args.stop is not None else round(price * 0.96, 2)
    target = args.target if args.target is not None else round(price * 1.06, 2)
    row = eng.register_swing_candidate(
        symbol=args.symbol,
        thesis_tag=args.thesis,
        reference_price=price,
        invalidation_price=stop,
        target_price=target,
        quantity=args.qty,
        max_sessions=args.max_sessions,
        sector=args.sector,
        notes=args.notes,
        auto_open=True,
        session_date=args.session_date,
    )
    print(json.dumps(row, indent=2, default=str))
    return 0


def cmd_portfolio_plan(args: argparse.Namespace) -> int:
    eng = _engine(args)
    # Default diversified seed legs if none provided via JSON file
    if args.legs_json:
        legs = json.loads(Path(args.legs_json).read_text(encoding="utf-8"))
    else:
        legs = [
            {"symbol": "RELIANCE.NS", "sector": "ENERGY", "target_weight_pct": 8.0},
            {"symbol": "TCS.NS", "sector": "IT", "target_weight_pct": 8.0},
            {"symbol": "HDFCBANK.NS", "sector": "BANKING", "target_weight_pct": 8.0},
            {"symbol": "ITC.NS", "sector": "FMCG", "target_weight_pct": 6.0},
            {"symbol": "LT.NS", "sector": "INDUSTRIALS", "target_weight_pct": 6.0},
        ]
    row = eng.register_portfolio_plan(
        plan_id=args.plan_id,
        total_capital_inr=args.capital,
        legs=legs,
        n_tranches=args.tranches,
        max_name_pct=args.max_name_pct,
        max_sector_pct=args.max_sector_pct,
        session_date=args.session_date,
    )
    print(json.dumps(row, indent=2, default=str))
    return 0


def cmd_long_open(args: argparse.Namespace) -> int:
    eng = _engine(args)
    price = args.price
    if price is None:
        pxmap = load_price_map(Path(args.nse_dir))
        sym = normalize_symbol(args.symbol)
        if sym not in pxmap:
            raise SystemExit(f"no mark for {sym}; pass --price explicitly")
        price = float(pxmap[sym])
    row = eng.open_long_term(
        symbol=args.symbol,
        thesis=args.thesis,
        thesis_version=args.version,
        entry_price=price,
        quantity=args.qty,
        sector=args.sector,
        review_cadence_sessions=args.review_sessions,
        session_date=args.session_date,
    )
    print(json.dumps(row, indent=2, default=str))
    return 0


def cmd_eod(args: argparse.Namespace) -> int:
    eng = _engine(args)
    summary = eng.run_eod(session_date=args.session_date)
    print(json.dumps({"eod": summary, "report": eng.report()}, indent=2, default=str))
    return 0


def cmd_report(args: argparse.Namespace) -> int:
    eng = _engine(args)
    print(json.dumps(eng.report(), indent=2, default=str))
    return 0


def cmd_review(args: argparse.Namespace) -> int:
    eng = _engine(args)
    row = eng.complete_long_term_review(args.hold_id, decision=args.decision)
    print(json.dumps(row, indent=2, default=str))
    return 0


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--nse-dir", default=str(DEFAULT_NSE_DIR))
    sp = p.add_subparsers(dest="cmd", required=True)

    sp.add_parser("bootstrap", help="Write registry + empty books")

    s = sp.add_parser("swing-open", help="Register + open H1 CNC swing")
    s.add_argument("--symbol", required=True)
    s.add_argument("--price", type=float, default=None, help="Defaults to NSE last mark")
    s.add_argument("--qty", type=float, required=True)
    s.add_argument("--stop", type=float, default=None, help="Defaults to -4 pct from entry")
    s.add_argument("--target", type=float, default=None, help="Defaults to +6 pct from entry")
    s.add_argument("--thesis", required=True)
    s.add_argument("--max-sessions", type=int, default=10)
    s.add_argument("--sector", default=None)
    s.add_argument("--notes", default=None)
    s.add_argument("--session-date", default=None)

    pp = sp.add_parser("portfolio-plan", help="Create H2 portfolio build plan")
    pp.add_argument("--plan-id", required=True)
    pp.add_argument("--capital", type=float, required=True)
    pp.add_argument("--tranches", type=int, default=4)
    pp.add_argument("--max-name-pct", type=float, default=10.0)
    pp.add_argument("--max-sector-pct", type=float, default=30.0)
    pp.add_argument("--legs-json", default=None)
    pp.add_argument("--session-date", default=None)

    lt = sp.add_parser("long-open", help="Open H3 long-term thesis hold")
    lt.add_argument("--symbol", required=True)
    lt.add_argument("--price", type=float, default=None, help="Defaults to NSE last mark")
    lt.add_argument("--qty", type=float, required=True)
    lt.add_argument("--thesis", required=True)
    lt.add_argument("--version", default="v1")
    lt.add_argument("--sector", default=None)
    lt.add_argument("--review-sessions", type=int, default=60)
    lt.add_argument("--session-date", default=None)

    e = sp.add_parser("eod", help="Mark opens, fill next portfolio tranche")
    e.add_argument("--session-date", default=None)

    r = sp.add_parser("review", help="Complete H3 review")
    r.add_argument("--hold-id", required=True)
    r.add_argument(
        "--decision",
        choices=["HOLD_CONTINUE", "THESIS_INVALIDATED"],
        default="HOLD_CONTINUE",
    )

    sp.add_parser("report", help="Print book summary")
    return p


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    dispatch = {
        "bootstrap": cmd_bootstrap,
        "swing-open": cmd_swing_open,
        "portfolio-plan": cmd_portfolio_plan,
        "long-open": cmd_long_open,
        "eod": cmd_eod,
        "report": cmd_report,
        "review": cmd_review,
    }
    return dispatch[args.cmd](args)


if __name__ == "__main__":
    raise SystemExit(main())
