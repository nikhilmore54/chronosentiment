#!/usr/bin/env python3
"""
Run independent per-symbol live_engine pipelines.

Aligned with .cursor/rules/chronosentiment-core.mdc:
- deterministic process topology (one lane per symbol)
- no strategy mutation
- symbol isolation before engine ingestion
"""

from __future__ import annotations

import argparse
import os
import signal
import subprocess
import sys
from pathlib import Path


def parse_args() -> argparse.Namespace:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--symbols",
        default="BTC-USD,ETH-USD,SOL-USD",
        help="Comma-separated symbols (default: BTC-USD,ETH-USD,SOL-USD)",
    )
    ap.add_argument("--interval", default="1m", help="Streamer interval (default: 1m)")
    ap.add_argument("--n-candles", type=int, default=2, help="Candles per fetch call")
    ap.add_argument("--cadence-seconds", type=float, default=1.0, help="Streamer cadence")
    ap.add_argument(
        "--log-dir",
        default="analysis/live_multi",
        help="Output directory for per-symbol logs",
    )
    ap.add_argument(
        "--blue-green-max-bytes",
        type=int,
        default=5 * 1024 * 1024,
        help="Blue/green rotate threshold per symbol",
    )
    return ap.parse_args()


def main() -> int:
    args = parse_args()
    symbols = [s.strip() for s in args.symbols.split(",") if s.strip()]
    if not symbols:
        print("[MULTI] no symbols provided", file=sys.stderr)
        return 1

    root = Path(__file__).resolve().parents[1]
    log_dir = (root / args.log_dir).resolve()
    log_dir.mkdir(parents=True, exist_ok=True)

    procs: list[subprocess.Popen[str]] = []
    try:
        for sym in symbols:
            safe = sym.replace("-", "_")
            base_log = log_dir / f"live_{safe}.log"
            cmd = (
                f"python3 scripts/fetch_stream_loop.py "
                f"--symbols \"{sym}\" "
                f"--interval {args.interval} "
                f"--n-candles {args.n_candles} "
                f"--cadence-seconds {args.cadence_seconds} "
                f"| cargo run --release --example live_engine 2>&1 "
                f"| python3 scripts/blue_green_log_writer.py \"{base_log}\" "
                f"--max-bytes {args.blue_green_max_bytes}"
            )
            print(f"[MULTI] starting {sym} -> {base_log}")
            p = subprocess.Popen(
                cmd,
                shell=True,
                cwd=str(root),
                text=True,
                preexec_fn=os.setsid,
            )
            procs.append(p)

        # Wait until one exits; then stop all (fail-fast deterministic supervision).
        while True:
            for p in procs:
                rc = p.poll()
                if rc is not None:
                    print(f"[MULTI] child exited pid={p.pid} rc={rc}, stopping all")
                    raise RuntimeError("child exited")
            signal.pause()
    except KeyboardInterrupt:
        print("[MULTI] stopping all pipelines (ctrl+c)")
    except RuntimeError:
        pass
    finally:
        for p in procs:
            if p.poll() is None:
                try:
                    os.killpg(os.getpgid(p.pid), signal.SIGTERM)
                except OSError:
                    pass
        for p in procs:
            try:
                p.wait(timeout=5)
            except Exception:
                pass

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
