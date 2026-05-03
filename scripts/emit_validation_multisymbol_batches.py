#!/usr/bin/env python3
"""
Deterministic stdin batches for live_engine (JSON array per line, BTC/ETH/SOL).
Enough steps for history_len >= 300 and sparse trending drift for momentum bootstrap testing.
"""
from __future__ import annotations

import json
import sys

SYMS = ("BTC-USD", "ETH-USD", "SOL-USD")
BASE = {"BTC-USD": 67240.0, "ETH-USD": 3500.0, "SOL-USD": 145.0}
STEPS = int(__import__("os").environ.get("VALIDATION_BATCH_STEPS", "420"))
T0 = 1714543200000


def main() -> None:
    step_scale = 60_000  # 1 min ms between batches
    for i in range(STEPS):
        ts = T0 + i * step_scale
        batch = []
        for sym in SYMS:
            b = BASE[sym]
            # Mild deterministic drift + slow oscillation (same inputs → same tape).
            drift = 1.0 + 1.2e-5 * (i - 100) + 3e-6 * ((i % 37) - 18)
            close = round(b * drift, 6)
            batch.append(
                {
                    "symbol": sym,
                    "timestamp": ts,
                    "open": close,
                    "high": close * 1.0002,
                    "low": close * 0.9998,
                    "close": close,
                    "volume": 1200.0 + (i % 50) * 3.0,
                }
            )
        print(json.dumps(batch), flush=True)


if __name__ == "__main__":
    main()
