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
    
    # 1. Native Sep-15
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
            trigger_time = datetime.datetime.fromtimestamp(path_bars[trigger_idx]["timestamp"], tz=IST).strftime("%H:%M")
            trigger_ret = (trigger_px - ep)/ep if direction == "LONG" else (ep - trigger_px)/ep
            
            subsequent_bars = path_bars[trigger_idx:]
            max_sub_px = max(b["close"] for b in subsequent_bars) if direction == "LONG" else min(b["close"] for b in subsequent_bars)
            max_sub_mfe = (max_sub_px - ep)/ep if direction == "LONG" else (ep - max_sub_px)/ep
            
            prod_exit_px = path_bars[-1]["close"]
            
            # Post-trigger specific returns (relative to entry price for consistent bps comparison)
            trigger_to_mfe = max_sub_mfe - trigger_ret
            trigger_to_prod = prod_ret - trigger_ret
            giveback = trigger_ret - prod_ret
            
            triggered_trades.append({
                "ticker": ticker,
                "direction": direction,
                "ep": ep,
                "trigger_px": trigger_px,
                "trigger_time": trigger_time,
                "trigger_bar": trigger_idx + 1,
                "trigger_ret": trigger_ret,
                "max_sub_px": max_sub_px,
                "max_sub_mfe": max_sub_mfe,
                "prod_exit_px": prod_exit_px,
                "prod_ret": prod_ret,
                "trigger_to_mfe": trigger_to_mfe,
                "trigger_to_prod": trigger_to_prod,
                "giveback": giveback
            })
            
    if not triggered_trades:
        print("No triggered trades found.")
        return

    # Write E7 Report
    md = "# E7: Post-Trigger Giveback Analysis (0.50% Threshold)\n\n"
    md += f"Analysis of the {len(triggered_trades)} trades from Sep-15 that successfully hit the +0.50% trigger.\n\n"
    
    mean_trig_ret = statistics.mean(t["trigger_ret"] for t in triggered_trades)
    mean_max_mfe = statistics.mean(t["max_sub_mfe"] for t in triggered_trades)
    mean_prod_ret = statistics.mean(t["prod_ret"] for t in triggered_trades)
    
    mean_trig_to_mfe = statistics.mean(t["trigger_to_mfe"] for t in triggered_trades)
    mean_giveback = statistics.mean(t["giveback"] for t in triggered_trades)
    
    md += "### Aggregate Post-Trigger Behavior\n"
    md += f"- **Mean Return at Trigger**: {mean_trig_ret*100:.3f}%\n"
    md += f"- **Mean Maximum Subsequent MFE**: {mean_max_mfe*100:.3f}%\n"
    md += f"- **Mean Eventual Production Return (H300)**: {mean_prod_ret*100:.3f}%\n"
    md += f"- **Mean Additional Upside Left on Table**: {mean_trig_to_mfe*10000:.1f} bps\n"
    md += f"- **Mean Giveback by H300**: {mean_giveback*10000:.1f} bps\n\n"
    
    pos_givebacks = [t for t in triggered_trades if t["giveback"] > 0]
    neg_givebacks = [t for t in triggered_trades if t["giveback"] < 0] # these actually improved after trigger
    
    md += f"Out of {len(triggered_trades)} triggered trades:\n"
    md += f"- **{len(pos_givebacks)} trades** surrendered gains (average giveback: {statistics.mean(t['giveback'] for t in pos_givebacks)*10000:.1f} bps)\n"
    if neg_givebacks:
        md += f"- **{len(neg_givebacks)} trades** improved after the trigger (average additional gain: {abs(statistics.mean(t['giveback'] for t in neg_givebacks))*10000:.1f} bps)\n"
    
    md += "\n### Trade-Level Details\n\n"
    md += "| Ticker | Trigger Time | Trigger Ret | Max Sub MFE | Prod Ret | Giveback vs Trigger | Additional Upside vs Trigger |\n"
    md += "|---|---|---:|---:|---:|---:|---:|\n"
    
    for t in sorted(triggered_trades, key=lambda x: x["giveback"], reverse=True):
        md += f"| {t['ticker']} | {t['trigger_time']} | {t['trigger_ret']*100:.2f}% | {t['max_sub_mfe']*100:.2f}% | {t['prod_ret']*100:.2f}% | {t['giveback']*10000:.1f} bps | {t['trigger_to_mfe']*10000:.1f} bps |\n"
        
    out = ARTIFACT_DIR / "post_trigger_giveback_report.md"
    with open(out, "w") as f:
        f.write(md)
    print(f"Report written to {out}")

if __name__ == "__main__":
    main()
