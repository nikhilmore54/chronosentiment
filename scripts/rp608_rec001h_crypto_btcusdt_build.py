#!/usr/bin/env python3
"""Rebuild BTCUSDT REC-001-H JSONL from Binance 15m bars (adapter research).

Output: datasets/recommendation/historical_crypto/BTCUSDT.jsonl
coralys-* untouched — evidence for frozen RecommendationEngineV1 only.
"""

from __future__ import annotations

import json
from collections import Counter
from datetime import datetime, timezone
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
SRC = REPO / "datasets/crypto_15m/binance_btcusdt_2024_15m.json"
OUT = REPO / "datasets/recommendation/historical_crypto/BTCUSDT.jsonl"

MIN_HIST = 20
FORWARD = 10
TARGET_ATR = 2.0
RISK_ATR = 1.0
STRIDE = 4


def main() -> None:
    bars = json.loads(SRC.read_text())
    n = len(bars)
    OUT.parent.mkdir(parents=True, exist_ok=True)

    def atr14(i: int) -> float:
        trs = []
        for j in range(i - 13, i + 1):
            h, l, prev_c = bars[j]["high"], bars[j]["low"], bars[j - 1]["close"]
            trs.append(max(h - l, abs(h - prev_c), abs(l - prev_c)))
        atr = trs[0]
        for tr in trs[1:]:
            atr = (atr * 13 + tr) / 14
        return atr

    def rel_vol20(i: int) -> float:
        vols = [bars[j]["volume"] for j in range(i - 20, i)]
        med = sorted(vols)[len(vols) // 2]
        return bars[i]["volume"] / med if med > 0 else 1.0

    def labels_at(i: int):
        rets = []
        for j in range(i - 4, i + 1):
            c0, c1 = bars[j - 1]["close"], bars[j]["close"]
            rets.append((c1 - c0) / c0 if c0 else 0.0)
        mom = sum(rets) / len(rets)
        pressure = 0.0
        for j in range(i - 4, i + 1):
            o, c = bars[j]["open"], bars[j]["close"]
            pressure += ((c - o) / o) if o else 0.0
        rvol = (sum(x * x for x in rets) / len(rets)) ** 0.5
        prior = []
        for k in range(max(MIN_HIST, i - 40), i):
            rr = []
            for j in range(k - 4, k + 1):
                c0, c1 = bars[j - 1]["close"], bars[j]["close"]
                rr.append((c1 - c0) / c0 if c0 else 0.0)
            prior.append((sum(x * x for x in rr) / len(rr)) ** 0.5)
        med = sorted(prior)[len(prior) // 2] if prior else rvol
        trend = (
            "Bullish"
            if mom > 0 and pressure >= 0
            else ("Bearish" if mom < 0 and pressure < 0 else "absent")
        )
        momentum = "Positive" if mom >= 0 else "Negative"
        volatility = "present" if rvol >= med else "absent"
        direction = "LONG" if mom >= 0 else "SHORT"
        return trend, momentum, volatility, direction

    def forward_mfe_mae(i: int, direction: str, ref: float):
        mfe = [0.0] * FORWARD
        mae = [0.0] * FORWARD
        peak_fav = 0.0
        peak_adv = 0.0
        for s in range(FORWARD):
            j = i + 1 + s
            if j >= n:
                return None, None
            h, l = bars[j]["high"], bars[j]["low"]
            if direction == "LONG":
                fav = (h - ref) / ref * 100.0
                adv = (l - ref) / ref * 100.0
            else:
                fav = (ref - l) / ref * 100.0
                adv = (ref - h) / ref * 100.0
            peak_fav = max(peak_fav, fav)
            peak_adv = min(peak_adv, adv)
            mfe[s] = peak_fav
            mae[s] = peak_adv
        return mfe, mae

    def outcome_for(direction: str, ref: float, atr: float, mfe, mae):
        tgt_pct = TARGET_ATR * atr / ref * 100.0
        risk_pct = RISK_ATR * atr / ref * 100.0
        sessions_to = None
        outcome = "HORIZON"
        for s in range(FORWARD):
            hit_tgt = mfe[s] >= tgt_pct
            hit_risk = abs(mae[s]) >= risk_pct
            if hit_tgt or hit_risk:
                if hit_tgt and hit_risk:
                    outcome = "RISK_BEFORE_TARGET"
                elif hit_tgt:
                    outcome = "TARGET_BEFORE_RISK"
                else:
                    outcome = "RISK_BEFORE_TARGET"
                sessions_to = s + 1
                break
        return outcome, sessions_to or FORWARD, tgt_pct, risk_pct

    rows = 0
    with OUT.open("w") as f:
        for i in range(MIN_HIST + 14, n - FORWARD - 1, STRIDE):
            atr = atr14(i)
            if atr <= 0:
                continue
            ref = bars[i]["close"]
            trend, momentum, volatility, direction = labels_at(i)
            mfe, mae = forward_mfe_mae(i, direction, ref)
            if mfe is None:
                continue
            outcome, sto, tgt_pct, risk_pct = outcome_for(direction, ref, atr, mfe, mae)
            ts = bars[i]["close_time_ms"] // 1000
            dt = datetime.fromtimestamp(ts, tz=timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
            for direction_i, mfe_i, mae_i in (
                (direction, mfe, mae),
                (
                    "SHORT" if direction == "LONG" else "LONG",
                    *forward_mfe_mae(i, "SHORT" if direction == "LONG" else "LONG", ref),
                ),
            ):
                if mfe_i is None:
                    continue
                outcome_i, sto_i, tgt_i, risk_i = outcome_for(
                    direction_i, ref, atr, mfe_i, mae_i
                )
                rec = {
                    "ticker": "BTCUSDT",
                    "date": dt,
                    "timestamp_unix": ts,
                    "reference_price": ref,
                    "open": bars[i]["open"],
                    "high": bars[i]["high"],
                    "low": bars[i]["low"],
                    "volume": bars[i]["volume"],
                    "relative_volume_20": rel_vol20(i),
                    "trend": trend,
                    "momentum": momentum,
                    "volatility": volatility,
                    "direction": direction_i,
                    "atr_14": atr,
                    "target_distance_pct": tgt_i,
                    "risk_distance_pct": risk_i,
                    "indicative_target": (
                        ref * (1 + tgt_i / 100.0)
                        if direction_i == "LONG"
                        else ref * (1 - tgt_i / 100.0)
                    ),
                    "indicative_risk": (
                        ref * (1 - risk_i / 100.0)
                        if direction_i == "LONG"
                        else ref * (1 + risk_i / 100.0)
                    ),
                    "mfe_pct": mfe_i,
                    "mae_pct": mae_i,
                    "sessions_available": FORWARD,
                    "outcome": outcome_i,
                    "sessions_to_outcome": float(sto_i),
                }
                f.write(json.dumps(rec, separators=(",", ":")) + "\n")
                rows += 1

    c_dir: Counter = Counter()
    for line in OUT.open():
        c_dir[json.loads(line)["direction"]] += 1
    print(f"wrote {rows} rows -> {OUT} dir={dict(c_dir)}")


if __name__ == "__main__":
    main()
