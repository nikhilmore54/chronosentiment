#!/usr/bin/env python3
import json
import csv
import datetime
from pathlib import Path
import statistics

WORKSPACE = Path("/Users/nikhil/ChronoSentiment_MEGA_FINAL")
BARS_DIR = WORKSPACE / "intraday_capture" / "yahoo_cache_1m"
TM_OBS_DIR = WORKSPACE / "time_machine" / "observations"
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

def find_first_observable_idx(bars, t0_iso, ep, bars_held, target_ret, direction):
    t0_dt = datetime.datetime.fromisoformat(t0_iso.replace("Z", "+00:00"))
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
    if not TM_OBS_DIR.exists():
        print(f"Directory {TM_OBS_DIR} does not exist.")
        return
        
    dates = sorted([d.name for d in TM_OBS_DIR.iterdir() if d.is_dir() and d.name.isdigit()])
    
    date_reports = []
    all_valid_trades = []
    
    csv_rows = []
    
    total_act = 0
    total_valid = 0
    total_incomplete = 0
    total_failures = 0

    for d in dates:
        obs_dir = TM_OBS_DIR / d
        files = list(obs_dir.glob("*.json"))
        
        cohort_act = 0
        cohort_valid = 0
        cohort_failures = 0
        cohort_incomplete = 0
        
        cohort_prod_rets = []
        cohort_shadow_rets = []
        cohort_deltas = []
        cohort_triggered = 0
        
        for fpath in files:
            with open(fpath) as f:
                obs = json.load(f)
                
            if obs.get("action") not in ["Buy", "Sell"]:
                continue
                
            cohort_act += 1
            ticker = obs["ticker"].replace(".NS", "_NS")
            direction = obs["direction"]
            ep = obs["reference_price"]
            prod_ret = obs["realized_return"]
            prod_bars_held = obs["n_bars_in_horizon"]
            t0_iso = obs["as_of"]
            
            bars = load_bars(ticker)
            if not bars:
                cohort_failures += 1
                csv_rows.append({
                    "time_machine_date": d,
                    "ticker": ticker,
                    "validation_status": "RECONCILIATION_FAILURE",
                    "trigger_bar": "", "shadow_exit_time": "", "shadow_exit_price": "",
                    "shadow_return": "", "production_return": f"{prod_ret:.6f}", "return_delta_bps": ""
                })
                continue
                
            start_idx = find_first_observable_idx(bars, t0_iso, ep, prod_bars_held, prod_ret, direction)
            if start_idx is None:
                last_cache_ts = bars[-1]["timestamp"] if bars else 0
                t0 = int(datetime.datetime.fromisoformat(t0_iso.replace("Z", "+00:00")).timestamp())
                expected_exit = t0 + (prod_bars_held * 60)
                
                if expected_exit > last_cache_ts:
                    cohort_incomplete += 1
                    status = "DATA_INCOMPLETE"
                else:
                    cohort_failures += 1
                    status = "RECONCILIATION_FAILURE"
                    
                csv_rows.append({
                    "time_machine_date": d,
                    "ticker": ticker,
                    "validation_status": status,
                    "trigger_bar": "", "shadow_exit_time": "", "shadow_exit_price": "",
                    "shadow_return": "", "production_return": f"{prod_ret:.6f}", "return_delta_bps": ""
                })
                continue
                
            # Valid path
            cohort_valid += 1
            path_bars = bars[start_idx : start_idx + prod_bars_held + 1]
            
            shadow_exit_idx = prod_bars_held
            shadow_mfe = 0.0
            shadow_mae = 0.0
            triggered = False
            
            for i, b in enumerate(path_bars):
                px = b["close"]
                bar_ret = (px - ep) / ep if direction == "LONG" else (ep - px) / ep
                
                if bar_ret > shadow_mfe: shadow_mfe = bar_ret
                if bar_ret < shadow_mae: shadow_mae = bar_ret
                
                if bar_ret >= 0.0050:
                    shadow_exit_idx = i
                    triggered = True
                    break
                    
            shadow_exit_px = path_bars[shadow_exit_idx]["close"]
            shadow_exit_time = datetime.datetime.fromtimestamp(path_bars[shadow_exit_idx]["timestamp"], tz=IST).strftime("%H:%M")
            shadow_return = (shadow_exit_px - ep) / ep if direction == "LONG" else (ep - shadow_exit_px) / ep
            shadow_bars_held = shadow_exit_idx + 1 if triggered else prod_bars_held
            delta_bps = (shadow_return - prod_ret) * 10000
            
            cohort_prod_rets.append(prod_ret)
            cohort_shadow_rets.append(shadow_return)
            cohort_deltas.append(delta_bps)
            if triggered: cohort_triggered += 1
            
            csv_rows.append({
                "time_machine_date": d,
                "ticker": ticker,
                "validation_status": "VALID",
                "trigger_bar": shadow_bars_held, 
                "shadow_exit_time": shadow_exit_time, 
                "shadow_exit_price": f"{shadow_exit_px:.6f}",
                "shadow_return": f"{shadow_return:.6f}", 
                "production_return": f"{prod_ret:.6f}", 
                "return_delta_bps": f"{delta_bps:.2f}"
            })
            
            all_valid_trades.append({
                "delta": shadow_return - prod_ret,
                "prod_ret": prod_ret,
                "shadow_ret": shadow_return
            })
            
        total_act += cohort_act
        total_valid += cohort_valid
        total_incomplete += cohort_incomplete
        total_failures += cohort_failures
        
        prod_mean = statistics.mean(cohort_prod_rets) if cohort_valid > 0 else 0
        shadow_mean = statistics.mean(cohort_shadow_rets) if cohort_valid > 0 else 0
        delta_mean = statistics.mean(cohort_deltas) if cohort_valid > 0 else 0
        trig_rate = (cohort_triggered / cohort_valid * 100) if cohort_valid > 0 else 0
        
        date_reports.append({
            "Date": d,
            "ACT": cohort_act,
            "Valid": cohort_valid,
            "Incomplete": cohort_incomplete,
            "Reconciliation failures": cohort_failures,
            "Trigger rate": trig_rate,
            "Production mean": prod_mean,
            "E4 mean": shadow_mean,
            "Δ bps": delta_mean
        })
        
    # Write CSV
    csv_path = ARTIFACT_DIR / "time_machine_e4_shadow_all_dates.csv"
    with open(csv_path, "w", newline="") as f:
        fieldnames = ["time_machine_date", "ticker", "validation_status", "trigger_bar", 
                      "shadow_exit_time", "shadow_exit_price", "shadow_return", 
                      "production_return", "return_delta_bps"]
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        for row in csv_rows:
            writer.writerow(row)
            
    # Write Markdown Report
    md = "# Time Machine E4-0.50 Evaluation Report\n\n"
    
    md += "## 1. Date-Level Report\n\n"
    md += "| Date | ACT | Valid | Incomplete | Reconciliation failures | Trigger rate | Production mean | E4 mean | Δ bps |\n"
    md += "|---|---:|---:|---:|---:|---:|---:|---:|---:|\n"
    for r in date_reports:
        md += f"| {r['Date']} | {r['ACT']} | {r['Valid']} | {r['Incomplete']} | {r['Reconciliation failures']} | {r['Trigger rate']:.1f}% | {r['Production mean']*100:.3f}% | {r['E4 mean']*100:.3f}% | {r['Δ bps']:.1f} |\n"
        
    md += "\n## 2. Integrity Report\n\n"
    md += "Explicit separation of states across all evaluated dates:\n\n"
    md += f"- **Total ACT Decisions Inventory**: {total_act}\n"
    md += f"- **VALID** (Fully reconciled, complete path): {total_valid}\n"
    md += f"- **DATA_INCOMPLETE** (Market data path missing/future): {total_incomplete}\n"
    md += f"- **RECONCILIATION_FAILURE** (Mismatch / exact T0 broken): {total_failures}\n\n"
    
    md += f"> {total_valid}/{total_act} valid; {total_act - total_valid} excluded due to specific integrity status.\n\n"
    
    if all_valid_trades:
        mean_prod = statistics.mean(t["prod_ret"] for t in all_valid_trades)
        mean_shad = statistics.mean(t["shadow_ret"] for t in all_valid_trades)
        mean_delta = statistics.mean(t["delta"] for t in all_valid_trades) * 10000
        
        md += "### Aggregate Valid Performance\n"
        md += f"- **Overall Production Mean**: {mean_prod*100:.3f}%\n"
        md += f"- **Overall Shadow Mean**: {mean_shad*100:.3f}%\n"
        md += f"- **Overall Mean Return Delta**: {mean_delta:.1f} bps\n"
        
    md_path = ARTIFACT_DIR / "time_machine_e4_all_dates_report.md"
    with open(md_path, "w") as f:
        f.write(md)
        
    print(f"CSV written to {csv_path}")
    print(f"Report written to {md_path}")

if __name__ == "__main__":
    main()
