#!/usr/bin/env python3
"""
RP-610 Crypto multi-horizon observe CLI (paper spot-sim only).

Examples:
  python3 scripts/rp610_crypto_observe.py bootstrap
  python3 scripts/rp610_crypto_observe.py day-open --side LONG --thesis multi_day_observe --price 68000
  python3 scripts/rp610_crypto_observe.py week-open --side LONG --thesis multi_week_observe --price 68000
  python3 scripts/rp610_crypto_observe.py long-open --side LONG --thesis long_term_observe --version v1 --price 68000
  python3 scripts/rp610_crypto_observe.py eod --price 69000
  python3 scripts/rp610_crypto_observe.py amend --candidate-id <id> --reason new_info \\
      --target-pct 0.12 --stop-pct 0.05
  python3 scripts/rp610_crypto_observe.py amend --candidate-id <id> --reason extend_swing \\
      --new-horizon H2_MULTI_WEEK --mark-price 69000
  python3 scripts/rp610_crypto_observe.py report
"""

from __future__ import annotations

import argparse
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

_WORKSPACE = Path(__file__).resolve().parent.parent
if str(_WORKSPACE) not in sys.path:
    sys.path.insert(0, str(_WORKSPACE))

from scripts.rp610_crypto_multi_horizon import (  # noqa: E402
    LIVE_ORDER_AUTHORITY,
    PROTOCOL_ID,
    write_horizon_registry,
)
from scripts.rp610_crypto_multi_horizon_engine import (  # noqa: E402
    admit_candidate,
    amend_hold,
    bootstrap_demo_candidates,
    build_candidate,
    find_open_hold,
    open_holds_from_book,
    summarize_books,
    write_daily_observation,
)

UTC = timezone.utc


def _parse_ts(s: str | None) -> datetime:
    if not s:
        return datetime.now(UTC)
    raw = s.strip()
    if raw.endswith("Z"):
        raw = raw[:-1] + "+00:00"
    dt = datetime.fromisoformat(raw)
    return dt if dt.tzinfo else dt.replace(tzinfo=UTC)


def _open_horizon(
    *,
    horizon_id: str,
    symbol: str,
    side: str,
    price: float,
    thesis: str,
    notional: float,
    entry_ts: str | None,
    source_signal_id: str,
    stop_pct: float | None,
    target_pct: float | None,
    thesis_version: str = "",
) -> dict:
    c = build_candidate(
        symbol=symbol,
        side=side,
        entry_price=float(price),
        entry_ts=_parse_ts(entry_ts),
        horizon_id=horizon_id,
        thesis=thesis if not thesis_version else f"{thesis} [{thesis_version}]",
        source_signal_id=source_signal_id,
        notional_usdt=float(notional),
        stop_pct=stop_pct,
        target_pct=target_pct,
    )
    if horizon_id == "H3_LONG_TERM":
        c["status"] = "HOLDING"
        c["thesis_version"] = thesis_version or "v1"
    admit_candidate(c)
    return c


def cmd_bootstrap(args: argparse.Namespace) -> int:
    path = write_horizon_registry()
    demo = []
    if args.with_demo:
        demo = bootstrap_demo_candidates(
            symbol=args.symbol,
            entry_price=float(args.price),
            entry_ts=_parse_ts(args.entry_ts) if args.entry_ts else None,
        )
    print(
        json.dumps(
            {
                "registry": str(path),
                "protocol_id": PROTOCOL_ID,
                "live_order_authority": LIVE_ORDER_AUTHORITY,
                "report": summarize_books(),
                "demo_admitted": len(demo),
            },
            indent=2,
        )
    )
    return 0


def cmd_day_open(args: argparse.Namespace) -> int:
    row = _open_horizon(
        horizon_id="H1_MULTI_DAY",
        symbol=args.symbol,
        side=args.side,
        price=args.price,
        thesis=args.thesis,
        notional=args.notional,
        entry_ts=args.entry_ts,
        source_signal_id=args.source_signal_id or "",
        stop_pct=args.stop_pct,
        target_pct=args.target_pct,
    )
    print(json.dumps(row, indent=2))
    return 0


def cmd_week_open(args: argparse.Namespace) -> int:
    row = _open_horizon(
        horizon_id="H2_MULTI_WEEK",
        symbol=args.symbol,
        side=args.side,
        price=args.price,
        thesis=args.thesis,
        notional=args.notional,
        entry_ts=args.entry_ts,
        source_signal_id=args.source_signal_id or "",
        stop_pct=args.stop_pct,
        target_pct=args.target_pct,
    )
    print(json.dumps(row, indent=2))
    return 0


def cmd_long_open(args: argparse.Namespace) -> int:
    row = _open_horizon(
        horizon_id="H3_LONG_TERM",
        symbol=args.symbol,
        side=args.side,
        price=args.price,
        thesis=args.thesis,
        notional=args.notional,
        entry_ts=args.entry_ts,
        source_signal_id=args.source_signal_id or "",
        stop_pct=args.stop_pct,
        target_pct=None,  # LT: no default target; thesis review
        thesis_version=args.version,
    )
    print(json.dumps(row, indent=2))
    return 0


def cmd_eod(args: argparse.Namespace) -> int:
    marks = {args.symbol.upper(): float(args.price)}
    summary = write_daily_observation(
        mark_ts=_parse_ts(args.ts),
        marks=marks,
        notes=args.notes or "",
    )
    print(json.dumps(summary, indent=2))
    return 0


def cmd_report(_args: argparse.Namespace) -> int:
    write_horizon_registry()
    payload = {
        "protocol_id": PROTOCOL_ID,
        "live_order_authority": LIVE_ORDER_AUTHORITY,
        "summary": summarize_books(),
        "opens": {
            hid: open_holds_from_book(hid)
            for hid in ("H1_MULTI_DAY", "H2_MULTI_WEEK", "H3_LONG_TERM")
        },
    }
    print(json.dumps(payload, indent=2, default=str))
    return 0


def cmd_amend(args: argparse.Namespace) -> int:
    if not args.reason:
        raise SystemExit("--reason is required (why the plan changed)")
    if find_open_hold(args.candidate_id) is None:
        raise SystemExit(f"no open hold for {args.candidate_id}; run report")
    out = amend_hold(
        args.candidate_id,
        reason=args.reason,
        stop_pct=args.stop_pct,
        target_pct=args.target_pct,
        hold_days_max=args.hold_days_max,
        hold_days_min=args.hold_days_min,
        max_exit_ts=args.max_exit_ts,
        thesis=args.thesis,
        thesis_version=args.thesis_version,
        source_signal_id=args.source_signal_id,
        new_horizon_id=args.new_horizon,
        mark_price=args.mark_price,
        mark_ts=_parse_ts(args.mark_ts) if args.mark_ts else None,
    )
    print(json.dumps(out, indent=2, default=str))
    return 0


def _add_open_args(p: argparse.ArgumentParser, *, require_price: bool = True) -> None:
    p.add_argument("--symbol", default="BTCUSDT")
    p.add_argument("--side", default="LONG", choices=["LONG", "SHORT"])
    p.add_argument("--price", required=require_price, type=float)
    p.add_argument("--entry-ts", default=None)
    p.add_argument("--notional", default=1000.0, type=float)
    p.add_argument("--stop-pct", default=None, type=float)
    p.add_argument("--target-pct", default=None, type=float)
    p.add_argument("--thesis", default="observe")
    p.add_argument("--source-signal-id", default="")


def main() -> int:
    p = argparse.ArgumentParser(description="RP-610 crypto multi-horizon observe (paper only)")
    sub = p.add_subparsers(dest="cmd", required=True)

    s = sub.add_parser("bootstrap", help="Write registry + closed gates; optional demo admits")
    s.add_argument("--with-demo", action="store_true")
    s.add_argument("--symbol", default="BTCUSDT")
    s.add_argument("--price", default=68000.0, type=float)
    s.add_argument("--entry-ts", default=None)
    s.set_defaults(func=cmd_bootstrap)

    s = sub.add_parser("day-open", help="Admit H1 multi-day paper hold")
    _add_open_args(s)
    s.set_defaults(func=cmd_day_open)

    s = sub.add_parser("week-open", help="Admit H2 multi-week paper hold")
    _add_open_args(s)
    s.set_defaults(func=cmd_week_open)

    s = sub.add_parser("long-open", help="Admit H3 long-term thesis hold")
    _add_open_args(s)
    s.add_argument("--version", default="v1")
    s.set_defaults(func=cmd_long_open)

    s = sub.add_parser("eod", help="Daily mark + deterministic H1/H2 exits")
    s.add_argument("--symbol", default="BTCUSDT")
    s.add_argument("--price", required=True, type=float)
    s.add_argument("--ts", default=None)
    s.add_argument("--notes", default="")
    s.set_defaults(func=cmd_eod)

    s = sub.add_parser("report", help="Open books + closed PnL summary")
    s.set_defaults(func=cmd_report)

    s = sub.add_parser(
        "amend",
        help="Append-only update to an open hold (target/stop/thesis/horizon)",
    )
    s.add_argument("--candidate-id", required=True)
    s.add_argument("--reason", required=True, help="Why parameters changed")
    s.add_argument("--stop-pct", default=None, type=float)
    s.add_argument("--target-pct", default=None, type=float)
    s.add_argument("--hold-days-min", default=None, type=int)
    s.add_argument("--hold-days-max", default=None, type=int)
    s.add_argument("--max-exit-ts", default=None)
    s.add_argument("--thesis", default=None)
    s.add_argument("--thesis-version", default=None)
    s.add_argument("--source-signal-id", default=None)
    s.add_argument(
        "--new-horizon",
        default=None,
        choices=["H1_MULTI_DAY", "H2_MULTI_WEEK", "H3_LONG_TERM"],
        help="Roll hold to another horizon (same entry; parent SUPERSEDED)",
    )
    s.add_argument("--mark-price", default=None, type=float, help="Mark at horizon roll")
    s.add_argument("--mark-ts", default=None)
    s.set_defaults(func=cmd_amend)

    args = p.parse_args()
    return int(args.func(args))


if __name__ == "__main__":
    raise SystemExit(main())
