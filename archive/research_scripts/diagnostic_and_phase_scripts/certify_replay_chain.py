#!/usr/bin/env python3
"""
Replay certification chain — frozen full replay, optional live equivalence, cross-runtime parity.

Certification protocol (chronosentiment-core):
  - Always --fresh on certification archives (never reuse stale gzip/dedupe).
  - cs-ingest is canonical for --from-frozen production ingest.

Typical flows:

  # A) Full frozen replay (canonical cs-ingest)
  python3 scripts/certify_replay_chain.py full-replay --batch-id 3 \
    --run-label cert_full --max-intervals 50 --fresh

  # B) Live vs replay equivalence (after live soak + freeze + replay_equiv on LIVE window)
  python3 scripts/certify_replay_chain.py equiv-vs-live --batch-id 900 \
    --live-label lse_replay --replay-label replay_equiv

  # C) Post-soak: freeze → replay live window → equivalence (end-to-end loop)
  python3 scripts/certify_replay_chain.py post-soak-cert --batch-id 900 \
    --live-label lse_replay --replay-label replay_equiv --freeze
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))

from candle_substrate import load_frozen_cohort
from compare_replay_equivalence import load_live_ts_window
from run_nse_cohort import resolve_archive_dir


def run(cmd: list[str], label: str) -> int:
    print("\n" + "=" * 72)
    print(label)
    print("=" * 72)
    print("  $", " ".join(cmd), "\n")
    return subprocess.run(cmd, cwd=ROOT).returncode


def cmd_full_replay(args: argparse.Namespace) -> int:
    cmd = [
        sys.executable,
        str(ROOT / "scripts" / "run_nse_cohort.py"),
        "--batch-id",
        str(args.batch_id),
        "--from-frozen",
        "--run-label",
        args.run_label,
        "--start-interval",
        str(args.start_interval),
    ]
    if args.fresh:
        cmd.append("--fresh")
    if args.max_intervals is not None:
        cmd.extend(["--max-intervals", str(args.max_intervals)])
    return run(cmd, f"CERTIFY — FULL FROZEN REPLAY (batch {args.batch_id:03d})")


def cmd_equiv_vs_live(args: argparse.Namespace) -> int:
    cmd = [
        sys.executable,
        str(ROOT / "scripts" / "compare_replay_equivalence.py"),
        "--batch-id",
        str(args.batch_id),
        "--live-label",
        args.live_label,
        "--replay-label",
        args.replay_label,
    ]
    ts_min = getattr(args, "ts_min", 0) or 0
    ts_max = getattr(args, "ts_max", 0) or 0
    if ts_min:
        cmd.extend(["--ts-min", str(ts_min)])
    if ts_max:
        cmd.extend(["--ts-max", str(ts_max)])
    return run(cmd, f"CERTIFY — LIVE vs REPLAY EQUIVALENCE (batch {args.batch_id:03d})")





def live_ts_interval_window(batch_id: int, live_ts: set[int]) -> tuple[list[int], int, int]:
    """Map committed live barriers to [start_interval, max_intervals) on frozen timeline."""
    cohort = ROOT / f"cohorts/batch_{batch_id:03d}.txt"
    symbols = [line.strip() for line in cohort.read_text().splitlines() if line.strip()]
    data, _manifest = load_frozen_cohort(batch_id, symbols)
    all_ts: set[int] = set()
    for df in data.values():
        for ts in df.index:
            all_ts.add(int(ts.timestamp()))
    timeline = sorted(all_ts)
    missing = sorted(live_ts - set(timeline))
    if missing:
        raise ValueError(
            f"{len(missing)} live barrier(s) missing from frozen substrate "
            f"(e.g. {missing[:3]}). Re-run with --freeze after soak."
        )
    indices = [i for i, t in enumerate(timeline) if t in live_ts]
    start = indices[0]
    end_exclusive = indices[-1] + 1
    window_ts = timeline[start:end_exclusive]
    return window_ts, start, end_exclusive - start


def cmd_post_soak_cert(args: argparse.Namespace) -> int:
    """Freeze (optional) → deterministic replay on live barrier window → equivalence."""
    live_dir = resolve_archive_dir(args.batch_id, False, args.live_label)
    steps_path = live_dir / "metadata" / "live_session_steps.jsonl"
    live_ts, committed = load_live_ts_window(steps_path)
    if not live_ts:
        print(f"❌ No committed barriers in {steps_path}", file=sys.stderr)
        return 1

    print("=" * 72)
    print("POST-SOAK REPLAY CERTIFICATION")
    print("=" * 72)
    print(f"  Batch         : {args.batch_id:03d}")
    print(f"  Live archive  : {live_dir}")
    print(f"  Live barriers : {len(live_ts)}")
    print(f"  ts range      : {min(live_ts)} → {max(live_ts)}")
    print("=" * 72)

    steps: list[tuple[str, int]] = []

    if args.freeze:
        code = run(
            [
                sys.executable,
                str(ROOT / "scripts" / "freeze_cohort_candles.py"),
                "--batch-id",
                str(args.batch_id),
                "--max-workers",
                str(args.freeze_workers),
            ],
            "POST-SOAK — FREEZE COHORT (substrate must cover live window)",
        )
        steps.append(("freeze", code))
        if code != 0:
            return _summarize(steps)

    try:
        window_ts, start_interval, max_intervals = live_ts_interval_window(
            args.batch_id, live_ts
        )
    except ValueError as e:
        print(f"❌ {e}", file=sys.stderr)
        return 1

    print(
        f"\n  Frozen replay window: interval [{start_interval}:{start_interval + max_intervals}) "
        f"({max_intervals} barriers, {len(window_ts)} unique ts)"
    )

    replay_args = argparse.Namespace(
        batch_id=args.batch_id,
        run_label=args.replay_label,
        start_interval=start_interval,
        max_intervals=max_intervals,
        fresh=True,
    )
    steps.append(("replay-live-window", cmd_full_replay(replay_args)))

    equiv_args = argparse.Namespace(
        batch_id=args.batch_id,
        live_label=args.live_label,
        replay_label=args.replay_label,
        ts_min=0,
        ts_max=0,
    )
    steps.append(("equiv-vs-live", cmd_equiv_vs_live(equiv_args)))

    return _summarize(steps)


def _summarize(steps: list[tuple[str, int]]) -> int:
    print("\n" + "=" * 72)
    print("CERTIFY — CHAIN SUMMARY")
    print("=" * 72)
    failed = [name for name, code in steps if code != 0]
    for name, code in steps:
        print(f"  {name:20s} {'PASS' if code == 0 else f'FAIL (exit {code})'}")
    print("=" * 72)
    return 1 if failed else 0


def cmd_all(args: argparse.Namespace) -> int:
    steps: list[tuple[str, int]] = []
    steps.append(("full-replay", cmd_full_replay(args)))
    live_dir = (
        ROOT
        / "state_archive"
        / "batches"
        / f"batch_{args.batch_id:03d}"
        / "runs"
        / args.live_label
    )
    steps_path = live_dir / "metadata" / "live_session_steps.jsonl"
    if args.skip_equiv:
        print("\n⚠️  Skipping equiv-vs-live (--skip-equiv)")
    elif live_dir.exists() and steps_path.exists():
        replay_dir = (
            ROOT / "state_archive" / "batches" / f"batch_{args.batch_id:03d}"
            / "runs" / args.replay_label
        )
        if replay_dir.resolve() == (
            ROOT / "state_archive" / "batches" / f"batch_{args.batch_id:03d}"
            / "runs" / args.run_label
        ).resolve() and args.max_intervals:
            print(
                "\n⚠️  Skipping equiv-vs-live: --run-label replay was just built as "
                f"[0:{args.max_intervals}) barriers, not the live soak window.\n"
                "    After live soak: freeze → run --from-frozen --run-label replay_equiv "
                "(full timeline or live ts window) → equiv-vs-live."
            )
        else:
            steps.append(("equiv-vs-live", cmd_equiv_vs_live(args)))
    else:
        print(
            f"\n⚠️  Skipping equiv-vs-live: no live steps at {steps_path}"
        )

    return _summarize(steps)


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(description="Replay certification orchestrator")
    sub = p.add_subparsers(dest="command", required=True)

    common = argparse.ArgumentParser(add_help=False)
    common.add_argument("--batch-id", type=int, required=True)

    fr = sub.add_parser("full-replay", parents=[common], help="Canonical frozen replay ingest")
    fr.add_argument("--run-label", default="cert_full")
    fr.add_argument("--start-interval", type=int, default=0)
    fr.add_argument("--max-intervals", type=int, default=None)
    fr.add_argument("--fresh", action="store_true", default=True)
    fr.add_argument("--no-fresh", action="store_false", dest="fresh")

    ev = sub.add_parser("equiv-vs-live", parents=[common], help="compare_replay_equivalence")
    ev.add_argument("--live-label", default="live")
    ev.add_argument("--replay-label", default="cert_full")
    ev.add_argument("--ts-min", type=int, default=0)
    ev.add_argument("--ts-max", type=int, default=0)



    al = sub.add_parser("all", parents=[common], help="Parity + full replay + live equiv if present")
    al.add_argument("--run-label", default="cert_full")
    al.add_argument("--replay-label", default=None, help="Defaults to --run-label")
    al.add_argument("--live-label", default="live")
    al.add_argument("--start-interval", type=int, default=0)
    al.add_argument("--max-intervals", type=int, default=50)
    al.add_argument("--fresh", action="store_true", default=True)
    al.add_argument("--no-fresh", action="store_false", dest="fresh")
    al.add_argument("--skip-equiv", action="store_true", help="Skip live vs replay (e.g. cert_full is not live-window replay)")
    al.add_argument("--ts-min", type=int, default=0)
    al.add_argument("--ts-max", type=int, default=0)

    ps = sub.add_parser(
        "post-soak-cert",
        parents=[common],
        help="Freeze → replay live barrier window → equiv-vs-live",
    )
    ps.add_argument("--live-label", required=True)
    ps.add_argument("--replay-label", default="replay_equiv")
    ps.add_argument(
        "--freeze",
        action="store_true",
        help="Re-freeze cohort candles before replay (required if substrate predates soak)",
    )
    ps.add_argument("--freeze-workers", type=int, default=2)

    return p


def main() -> int:
    p = build_parser()
    args = p.parse_args()
    if args.command == "all" and args.replay_label is None:
        args.replay_label = args.run_label

    handlers = {
        "full-replay": cmd_full_replay,
        "equiv-vs-live": cmd_equiv_vs_live,
        "all": cmd_all,
        "post-soak-cert": cmd_post_soak_cert,
    }
    return handlers[args.command](args)


if __name__ == "__main__":
    sys.exit(main())
