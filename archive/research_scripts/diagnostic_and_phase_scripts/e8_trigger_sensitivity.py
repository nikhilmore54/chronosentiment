#!/usr/bin/env python3
import csv
import json
import datetime
import statistics
from pathlib import Path

WORKSPACE = Path("/Users/nikhil/ChronoSentiment_MEGA_FINAL")
BARS_DIR = WORKSPACE / "intraday_capture" / "yahoo_cache_1m"
ARTIFACT_DIR = Path("/Users/nikhil/.gemini/antigravity-ide/brain/ac8b3565-b88c-4945-8459-7e723a29389a")
IST = datetime.timezone(datetime.timedelta(hours=5, minutes=30))

_bar_cache = {}

def load_bars(ticker_ns: str):
    if ticker_ns in _bar_cache: return _bar_cache[ticker_ns]
    p = BARS_DIR / (ticker_ns.replace("_NS", ".NS") + ".json")
    if not p.exists(): return []
    with open(p) as f: 
        bars = [b for b in json.load(f) if b.get("volume", 0) > 0]
    bars.sort(key=lambda b: b["timestamp"])
    _bar_cache[ticker_ns] = bars
    return bars

def find_first_observable_idx(bars, t0_dt, ep, bars_held, target_ret, direction):
    t0 = int(t0_dt.timestamp())
    for i, b in enumerate(bars):
        if b["timestamp"] >= t0:
            if i + bars_held < len(bars):
                exit_px = bars[i + bars_held]["close"]
                ret = (exit_px - ep)/ep if direction == "LONG" else (ep - exit_px)/ep
                if abs(ret - target_ret) < 1e-5:
                    return i
    return None

def main():
    triggered_trades = []
    
    # Native Sep-15
    with open(WORKSPACE / "datasets" / "paper_trader_v2_20260915.csv", newline="") as f:
        reader = list(csv.DictReader(f))
        
    for r in reader:
        if r.get("exit_reason", "").upper() != "HORIZON": continue
        
        ticker = r["ticker"].replace("_NS", ".NS")
        direction = r["direction"]
        ep = float(r["entry_price"])
        prod_ret = float(r["realized_return"])
        bars_held = int(r["bars_held"])
        
        bars = load_bars(ticker)
        if not bars: continue
            
        t0_dt = datetime.datetime.strptime("20260915 09:15", "%Y%m%d %H:%M").replace(tzinfo=IST)
        start_idx = find_first_observable_idx(bars, t0_dt, ep, bars_held, prod_ret, direction)
        if start_idx is None: continue
            
        path_bars = bars[start_idx : start_idx + bars_held + 1]
        
        trigger_idx = None
        for i, b in enumerate(path_bars):
            px = b["close"]
            ret = (px - ep)/ep if direction == "LONG" else (ep - px)/ep
            if ret >= 0.0050:
                trigger_idx = i
                break
                
        if trigger_idx is not None:
            trigger_px = path_bars[trigger_idx]["close"]
            trigger_ret = (trigger_px - ep)/ep if direction == "LONG" else (ep - trigger_px)/ep
            
            overshoot = trigger_ret - 0.0050
            
            triggered_trades.append({
                "ticker": ticker,
                "trigger_ret": trigger_ret,
                "overshoot": overshoot
            })
            
    if not triggered_trades:
        print("No triggered trades found.")
        return

    # Calculate metrics
    rets = sorted([t["trigger_ret"] for t in triggered_trades])
    n = len(rets)
    
    median_ret = statistics.median(rets)
    mean_ret = statistics.mean(rets)
    
    # Simple percentile calculation
    p25_ret = rets[int(n * 0.25)]
    p75_ret = rets[int(n * 0.75)]
    
    mean_overshoot = statistics.mean([t["overshoot"] for t in triggered_trades])
    
    b_050_060 = sum(1 for t in triggered_trades if 0.0050 <= t["trigger_ret"] < 0.0060)
    b_060_075 = sum(1 for t in triggered_trades if 0.0060 <= t["trigger_ret"] < 0.0075)
    b_075_100 = sum(1 for t in triggered_trades if 0.0075 <= t["trigger_ret"] < 0.0100)
    b_100_plus = sum(1 for t in triggered_trades if t["trigger_ret"] >= 0.0100)
    
    md = "# E8: Trigger-Price Sensitivity and Overshoot Analysis\n\n"
    md += f"Analysis of the {n} trades from Sep-15 that hit the +0.50% trigger.\n\n"
    
    md += "### Aggregate Trigger Returns\n"
    md += f"- **Mean Trigger Return**: {mean_ret*100:.3f}%\n"
    md += f"- **Median Trigger Return**: {median_ret*100:.3f}%\n"
    md += f"- **P25 Trigger Return**: {p25_ret*100:.3f}%\n"
    md += f"- **P75 Trigger Return**: {p75_ret*100:.3f}%\n"
    md += f"- **Mean Overshoot**: {mean_overshoot*10000:.1f} bps\n\n"
    
    md += "### Trigger Overshoot Distribution\n"
    md += "| Bucket | Trades | Percentage |\n"
    md += "|---|---:|---:|\n"
    md += f"| 0.50% – 0.60% | {b_050_060} | {b_050_060/n*100:.1f}% |\n"
    md += f"| 0.60% – 0.75% | {b_060_075} | {b_060_075/n*100:.1f}% |\n"
    md += f"| 0.75% – 1.00% | {b_075_100} | {b_075_100/n*100:.1f}% |\n"
    md += f"| > 1.00% | {b_100_plus} | {b_100_plus/n*100:.1f}% |\n\n"
    
    md += "> [!NOTE]\n"
    md += "> The close-only rule means E4 does not instantaneously capture exactly +0.50%. It captures the first completed 1-minute close *after* the market moves at least 0.50%. This creates a natural execution overshoot depending on market volatility.\n"
    
    out = ARTIFACT_DIR / "trigger_sensitivity_report.md"
    with open(out, "w") as f:
        f.write(md)
    print(f"Report written to {out}")

if __name__ == "__main__":
    main()
