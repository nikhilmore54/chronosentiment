#!/usr/bin/env python3
import json
import csv
import statistics
from pathlib import Path

WORKSPACE = Path("/Users/nikhil/ChronoSentiment_MEGA_FINAL")
CRYPTO_DIR = WORKSPACE / "datasets" / "e9" / "crypto"
US_EQ_DIR = WORKSPACE / "datasets" / "e9" / "us_equities"
REPORTS_DIR = WORKSPACE / "reports" / "e9"

def eval_path(bars, t0_idx, direction, h300_bars=300, threshold=0.0050):
    if t0_idx + h300_bars >= len(bars):
        return None # Incomplete horizon
        
    ep = bars[t0_idx]["close"]
    horizon_bars = bars[t0_idx + 1 : t0_idx + 1 + h300_bars]
    
    prod_exit_px = horizon_bars[-1]["close"]
    prod_ret = (prod_exit_px - ep)/ep if direction == "LONG" else (ep - prod_exit_px)/ep
    
    shadow_exit_idx = len(horizon_bars) - 1
    triggered = False
    
    max_px = max(b["close"] for b in horizon_bars) if direction == "LONG" else min(b["close"] for b in horizon_bars)
    min_px = min(b["close"] for b in horizon_bars) if direction == "LONG" else max(b["close"] for b in horizon_bars)
    mfe = (max_px - ep)/ep if direction == "LONG" else (ep - max_px)/ep
    mae = (min_px - ep)/ep if direction == "LONG" else (ep - min_px)/ep
    
    trigger_ret = 0.0
    for i, b in enumerate(horizon_bars):
        px = b["close"]
        ret = (px - ep)/ep if direction == "LONG" else (ep - px)/ep
        if ret >= threshold:
            shadow_exit_idx = i
            triggered = True
            trigger_ret = ret
            break
            
    shadow_px = horizon_bars[shadow_exit_idx]["close"]
    shadow_ret = (shadow_px - ep)/ep if direction == "LONG" else (ep - shadow_px)/ep
    
    giveback = 0.0
    if triggered:
        giveback = trigger_ret - prod_ret
        
    return {
        "direction": direction,
        "triggered": triggered,
        "trigger_timing": shadow_exit_idx + 1 if triggered else None,
        "trigger_ret": trigger_ret if triggered else None,
        "shadow_ret": shadow_ret,
        "prod_ret": prod_ret,
        "delta": shadow_ret - prod_ret,
        "giveback": giveback,
        "mfe": mfe,
        "mae": mae
    }

def process_dataset(filepath, step_mins=60):
    with open(filepath) as f:
        bars = json.load(f)
        
    results = []
    for t0_idx in range(0, len(bars), step_mins):
        res_long = eval_path(bars, t0_idx, "LONG")
        if res_long: results.append(res_long)
        res_short = eval_path(bars, t0_idx, "SHORT")
        if res_short: results.append(res_short)
    return results

def format_report(market, regime, results):
    if not results: return None
    
    n = len(results)
    trig_res = [r for r in results if r["triggered"]]
    
    trig_rate = len(trig_res) / n
    e4_mean = statistics.mean(r["shadow_ret"] for r in results)
    h300_mean = statistics.mean(r["prod_ret"] for r in results)
    uncond_delta = statistics.mean(r["delta"] for r in results)
    
    mfe_mean = statistics.mean(r["mfe"] for r in results)
    mae_mean = statistics.mean(r["mae"] for r in results)
    
    cond_delta = statistics.mean(r["delta"] for r in trig_res) if trig_res else 0.0
    giveback = statistics.mean(r["giveback"] for r in trig_res) if trig_res else 0.0
    mean_trig_bar = statistics.mean(r["trigger_timing"] for r in trig_res) if trig_res else 0.0
    
    return {
        "Market": market,
        "Regime": regime,
        "N": n,
        "H300 mean": f"{h300_mean*100:.3f}%",
        "E4 mean": f"{e4_mean*100:.3f}%",
        "Paired Δ": f"{uncond_delta*10000:.1f} bps",
        "Trigger": f"{trig_rate*100:.1f}%",
        "Trig Bar": f"{mean_trig_bar:.1f}",
        "MFE": f"{mfe_mean*100:.2f}%",
        "MAE": f"{mae_mean*100:.2f}%",
        "Giveback": f"{giveback*10000:.1f} bps",
        "Cond Δ": f"{cond_delta*10000:.1f} bps"
    }

def process_directory(directory, out_file_name, title):
    if not directory.exists():
        return
        
    files = list(directory.glob("*.json"))
    if not files:
        return
        
    reports = []
    for fpath in files:
        name_parts = fpath.stem.split("_")
        market = name_parts[-1]
        regime = "_".join(name_parts[:-1]) if len(name_parts) > 1 else "recent_5d"
        
        res = process_dataset(fpath, step_mins=60)
        report = format_report(market, regime, res)
        if report:
            reports.append(report)
            
    reports.sort(key=lambda x: (x["Market"], x["Regime"]))
    
    md = f"# {title}\n\n"
    md += "| Market | Regime | N | H300 mean | E4 mean | Paired Δ | Trigger | Trig Bar | MFE | MAE | Giveback | Cond Δ |\n"
    md += "|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|\n"
    for r in reports:
        md += f"| {r['Market']} | {r['Regime']} | {r['N']} | {r['H300 mean']} | {r['E4 mean']} | {r['Paired Δ']} | {r['Trigger']} | {r['Trig Bar']} | {r['MFE']} | {r['MAE']} | {r['Giveback']} | {r['Cond Δ']} |\n"
        
    out = REPORTS_DIR / out_file_name
    with open(out, "w") as f:
        f.write(md)
    print(f"Report written to {out}")

def main():
    process_directory(CRYPTO_DIR, "crypto_portability_report.md", "E9 Crypto Portability Diagnostic")
    process_directory(US_EQ_DIR, "us_equities_portability_report.md", "E9 US Equities Portability Diagnostic")

if __name__ == "__main__":
    main()
