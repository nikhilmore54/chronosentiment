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

def extract_valid_paths_native(csv_path, date_str):
    valid_paths = []
    with open(csv_path, newline="") as f:
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
            
        t0_dt = datetime.datetime.strptime(date_str + " 09:15", "%Y%m%d %H:%M").replace(tzinfo=IST)
        start_idx = find_first_observable_idx(bars, t0_dt, ep, bars_held, prod_ret, direction)
        if start_idx is None: continue
            
        path_bars = bars[start_idx : start_idx + bars_held + 1]
        valid_paths.append({
            "ticker": ticker,
            "direction": direction,
            "ep": ep,
            "prod_ret": prod_ret,
            "bars_held": bars_held,
            "path": path_bars
        })
    return valid_paths

def extract_valid_paths_tm(date_str):
    valid_paths = []
    obs_dir = WORKSPACE / "time_machine" / "observations" / date_str
    if not obs_dir.exists(): return []
    
    for fpath in obs_dir.glob("*.json"):
        with open(fpath) as f:
            obs = json.load(f)
            
        if obs.get("action") not in ["Buy", "Sell"]: continue
            
        ticker = obs["ticker"].replace(".NS", "_NS")
        direction = obs["direction"]
        ep = obs["reference_price"]
        prod_ret = obs["realized_return"]
        bars_held = obs["n_bars_in_horizon"]
        t0_iso = obs["as_of"]
        
        bars = load_bars(ticker)
        if not bars: continue
            
        t0_dt = datetime.datetime.fromisoformat(t0_iso.replace("Z", "+00:00"))
        start_idx = find_first_observable_idx(bars, t0_dt, ep, bars_held, prod_ret, direction)
        if start_idx is None: continue
            
        path_bars = bars[start_idx : start_idx + bars_held + 1]
        valid_paths.append({
            "ticker": ticker,
            "direction": direction,
            "ep": ep,
            "prod_ret": prod_ret,
            "bars_held": bars_held,
            "path": path_bars
        })
    return valid_paths

def eval_threshold(paths, threshold):
    results = []
    for p in paths:
        path_bars = p["path"]
        ep = p["ep"]
        direction = p["direction"]
        
        shadow_exit_idx = p["bars_held"]
        triggered = False
        
        for i, b in enumerate(path_bars):
            px = b["close"]
            ret = (px - ep)/ep if direction == "LONG" else (ep - px)/ep
            if ret >= threshold:
                shadow_exit_idx = i
                triggered = True
                break
                
        shadow_px = path_bars[shadow_exit_idx]["close"]
        shadow_ret = (shadow_px - ep)/ep if direction == "LONG" else (ep - shadow_px)/ep
        shadow_bars = shadow_exit_idx + 1 if triggered else p["bars_held"]
        
        # MFE over the full horizon
        full_mfe = max([(b["close"] - ep)/ep if direction == "LONG" else (ep - b["close"])/ep for b in path_bars])
        
        results.append({
            "prod_ret": p["prod_ret"],
            "shadow_ret": shadow_ret,
            "triggered": triggered,
            "shadow_bars": shadow_bars,
            "delta": shadow_ret - p["prod_ret"],
            "mfe_cap": shadow_ret / full_mfe if full_mfe > 0.001 else 0
        })
    return results

def format_row(threshold, res):
    if not res: return f"| {threshold*100:.2f}% | 0 | 0.0% | 0.000% | 0.0 |"
    
    mean_ret = statistics.mean([r["shadow_ret"] for r in res])
    trig_rate = sum(1 for r in res if r["triggered"]) / len(res)
    deltas = statistics.mean([r["delta"] for r in res])
    
    trig_rets = [r["shadow_ret"] for r in res if r["triggered"]]
    untrig_rets = [r["shadow_ret"] for r in res if not r["triggered"]]
    
    sum_trig = sum(trig_rets)
    sum_untrig = sum(untrig_rets)
    
    return f"| {threshold*100:.2f}% | {trig_rate*100:.1f}% | {mean_ret*100:.3f}% | {deltas*10000:.1f} bps | {sum_trig*100:.2f}% | {sum_untrig*100:.2f}% |"

def main():
    native_paths = extract_valid_paths_native(WORKSPACE / "datasets" / "paper_trader_v2_20260915.csv", "20260915")
    tm_paths = extract_valid_paths_tm("20260910") + extract_valid_paths_tm("20260911")
    
    thresholds = [0.0025, 0.0035, 0.0050, 0.0065, 0.0075, 0.0100, 0.0125, 0.0150]
    
    md = "# E5 & E6: Threshold Stability and Regime Analysis\n\n"
    
    def render_table(title, paths):
        t_md = f"### {title} (n={len(paths)})\n\n"
        t_md += "| Threshold | Trigger Rate | Mean Return | Δ vs H300 | Sum Ret (Trig) | Sum Ret (Untrig) |\n"
        t_md += "|---|---:|---:|---:|---:|---:|\n"
        for th in thresholds:
            res = eval_threshold(paths, th)
            t_md += format_row(th, res) + "\n"
        return t_md + "\n"
        
    md += render_table("Pooled Results (All Regimes)", native_paths + tm_paths)
    md += render_table("Favorable Native Regime (Sep-15)", native_paths)
    md += render_table("Adverse Historical Regime (Sep-10, Sep-11)", tm_paths)
    
    out = ARTIFACT_DIR / "threshold_stability_report.md"
    with open(out, "w") as f:
        f.write(md)
    print(f"Report written to {out}")

if __name__ == "__main__":
    main()
