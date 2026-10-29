#!/usr/bin/env python3
import csv
import json
import argparse
import datetime
from pathlib import Path
import re
import statistics

WORKSPACE = Path("/Users/nikhil/ChronoSentiment_MEGA_FINAL")
DATASET_DIR = WORKSPACE / "datasets"
BARS_DIR = WORKSPACE / "intraday_capture" / "yahoo_cache_1m"
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

def find_first_observable_idx(bars, d_str, ep, bars_held, target_ret, direction):
    if "-" in d_str: d_str = d_str.replace("-", "")
    t0 = int(datetime.datetime.strptime(d_str + " 09:15", "%Y%m%d %H:%M").replace(tzinfo=IST).timestamp())
    t1 = int(datetime.datetime.strptime(d_str + " 15:30", "%Y%m%d %H:%M").replace(tzinfo=IST).timestamp())
    
    for i, b in enumerate(bars):
        if t0 <= b["timestamp"] <= t1:
            if i + bars_held < len(bars):
                exit_px = bars[i + bars_held]["close"]
                ret = (exit_px - ep)/ep if direction == "LONG" else (ep - exit_px)/ep
                if abs(ret - target_ret) < 1e-5:
                    return i
    return None

def extract_date(row, filename):
    if "date" in row: return row["date"].replace("-", "")
    if "decision_id" in row:
        m = re.search(r"2026\d{4}", row["decision_id"])
        if m: return m.group(0)
    
    name_parts = Path(filename).stem.split("_")
    for p in name_parts:
        if p.isdigit() and len(p) == 8:
            return p
    return "20260915"

def parse_iso_or_hm(time_str, default_t):
    if not time_str: return default_t
    if "T" in time_str:
        return int(datetime.datetime.fromisoformat(time_str).timestamp())
    else:
        # Assumes %H:%M today if no T is present, but this is a rough proxy
        return default_t

def main():
    parser = argparse.ArgumentParser(description="Run E4-0.50 Shadow Pipeline")
    parser.add_argument("--input", type=str, required=True, help="Path to production CSV ledger")
    parser.add_argument("--output", type=str, help="Path to output shadow CSV ledger")
    args = parser.parse_args()

    input_path = Path(args.input)
    if not input_path.exists():
        print(f"Error: {input_path} does not exist.")
        return

    date_str = "20260915"
    name_parts = input_path.stem.split("_")
    for p in name_parts:
        if p.isdigit() and len(p) == 8:
            date_str = p

    output_path = Path(args.output) if args.output else DATASET_DIR / f"shadow_ledger_e4_050_{date_str}.csv"
    
    fieldnames = [
        "decision_id", "ticker", "date", "direction", "entry_price",
        "prod_exit_time", "prod_exit_price", "prod_exit_reason", 
        "prod_return", "prod_mfe", "prod_mae", "prod_bars_held",
        "shadow_exit_time", "shadow_exit_price", "shadow_exit_reason", 
        "shadow_return", "shadow_mfe", "shadow_mae", "shadow_bars_held",
        "shadow_vs_production_return_bps"
    ]

    success_count = 0
    failure_count = 0
    incomplete_count = 0
    
    # Aggregates
    trigger_bars = []
    triggered_returns = []
    untriggered_returns = []
    all_prod_returns = []
    all_shadow_returns = []
    all_deltas = []
    all_mfe_caps = []

    with open(input_path, newline="") as infile, open(output_path, "w", newline="") as outfile:
        reader = list(csv.DictReader(infile))
        writer = csv.DictWriter(outfile, fieldnames=fieldnames)
        writer.writeheader()

        for r in reader:
            if r.get("exit_reason", "").upper() != "HORIZON":
                continue

            ticker = r["ticker"].replace("_NS", ".NS")
            direction = r["direction"]
            ep = float(r["entry_price"])
            prod_ret = float(r["realized_return"])
            prod_bars_held = int(r["bars_held"])
            
            trade_date = extract_date(r, input_path.name)
            decision_id = r.get("decision_id", f"LIVE-{trade_date}-{ticker}")

            bars = load_bars(ticker)
            if not bars:
                print(f"RECONCILIATION_FAILURE: {ticker} on {trade_date} - No market data found")
                failure_count += 1
                continue

            start_idx = find_first_observable_idx(bars, trade_date, ep, prod_bars_held, prod_ret, direction)
            if start_idx is None:
                # Differentiate incomplete vs failure based on expected exit time
                exit_time_str = r.get("exit_time_ist", r.get("exit_timestamp"))
                last_cache_ts = bars[-1]["timestamp"] if bars else 0
                expected_exit_ts = parse_iso_or_hm(exit_time_str, last_cache_ts + 999999)
                
                if expected_exit_ts > last_cache_ts:
                    print(f"DATA_INCOMPLETE: {ticker} on {trade_date} - Required future observations do not yet exist")
                    incomplete_count += 1
                else:
                    print(f"RECONCILIATION_FAILURE: {ticker} on {trade_date} - Could not establish exact T0 sequence")
                    failure_count += 1
                continue

            path_bars = bars[start_idx : start_idx + prod_bars_held + 1]
            
            shadow_exit_idx = prod_bars_held
            shadow_exit_reason = "HORIZON"
            shadow_mfe = 0.0
            shadow_mae = 0.0
            
            triggered = False
            for i, b in enumerate(path_bars):
                px = b["close"]
                bar_ret = (px - ep) / ep if direction == "LONG" else (ep - px) / ep
                
                if bar_ret > shadow_mfe: shadow_mfe = bar_ret
                if bar_ret < shadow_mae: shadow_mae = bar_ret
                
                if bar_ret >= 0.0050: # close-only E4 0.50% threshold
                    shadow_exit_idx = i
                    shadow_exit_reason = "MFE_PROTECTION"
                    triggered = True
                    break

            shadow_exit_px = path_bars[shadow_exit_idx]["close"]
            shadow_exit_time = datetime.datetime.fromtimestamp(path_bars[shadow_exit_idx]["timestamp"], tz=IST).strftime("%H:%M")
            shadow_return = (shadow_exit_px - ep) / ep if direction == "LONG" else (ep - shadow_exit_px) / ep
            
            shadow_bars_held = shadow_exit_idx + 1 if triggered else prod_bars_held
            prod_exit_time = r.get("exit_time_ist", r.get("exit_timestamp", datetime.datetime.fromtimestamp(path_bars[-1]["timestamp"], tz=IST).strftime("%H:%M")))
            prod_exit_px = float(r["exit_price"])
            delta_bps = (shadow_return - prod_ret) * 10000

            writer.writerow({
                "decision_id": decision_id,
                "ticker": ticker.replace(".NS", "_NS"),
                "date": trade_date,
                "direction": direction,
                "entry_price": f"{ep:.6f}",
                "prod_exit_time": prod_exit_time,
                "prod_exit_price": f"{prod_exit_px:.6f}",
                "prod_exit_reason": r.get("exit_reason"),
                "prod_return": f"{prod_ret:.6f}",
                "prod_mfe": r.get("max_favourable_excursion", r.get("MFE", f"{shadow_mfe:.6f}")),
                "prod_mae": r.get("max_adverse_excursion", r.get("MAE", f"{shadow_mae:.6f}")),
                "prod_bars_held": prod_bars_held,
                "shadow_exit_time": shadow_exit_time,
                "shadow_exit_price": f"{shadow_exit_px:.6f}",
                "shadow_exit_reason": shadow_exit_reason,
                "shadow_return": f"{shadow_return:.6f}",
                "shadow_mfe": f"{shadow_mfe:.6f}",
                "shadow_mae": f"{shadow_mae:.6f}",
                "shadow_bars_held": shadow_bars_held,
                "shadow_vs_production_return_bps": f"{delta_bps:.2f}"
            })
            
            success_count += 1
            if triggered:
                trigger_bars.append(shadow_bars_held)
                triggered_returns.append(shadow_return)
            else:
                untriggered_returns.append(shadow_return)
                
            all_prod_returns.append(prod_ret)
            all_shadow_returns.append(shadow_return)
            all_deltas.append(shadow_return - prod_ret)
            
            if shadow_mfe > 0.001:
                all_mfe_caps.append(shadow_return / shadow_mfe)

    print(f"\n==============================================")
    print(f"Shadow Pipeline Complete (E4-0.50)")
    print(f"==============================================")
    print(f"Successfully reconciled and simulated: {success_count} trades")
    if incomplete_count > 0:
        print(f"PENDING (DATA_INCOMPLETE): {incomplete_count} trades")
    if failure_count > 0:
        print(f"RECONCILIATION_FAILURE: {failure_count} trades")
        
    if success_count > 0:
        mean_shadow = statistics.mean(all_shadow_returns)
        mean_prod = statistics.mean(all_prod_returns)
        med_shadow = statistics.median(all_shadow_returns)
        med_prod = statistics.median(all_prod_returns)
        mean_delta = statistics.mean(all_deltas)
        trig_rate = len(triggered_returns) / success_count
        med_trig_bar = statistics.median(trigger_bars) if trigger_bars else 0
        mean_trig_bar = statistics.mean(trigger_bars) if trigger_bars else 0
        mean_cap = statistics.mean(all_mfe_caps) if all_mfe_caps else 0
        sum_trig = sum(triggered_returns)
        sum_untrig = sum(untriggered_returns)
        
        print(f"\n--- Shadow Aggregate Report ---")
        print(f"Trigger Rate:                {trig_rate*100:.1f}%")
        print(f"Median Trigger Bar:          {med_trig_bar:.1f}")
        print(f"Mean Trigger Bar:            {mean_trig_bar:.1f}")
        print(f"Opp. Capture vs MFE (>0.1%): {mean_cap*100:.1f}%")
        print(f"Production Mean Return:      {mean_prod*100:.3f}%")
        print(f"Shadow Mean Return:          {mean_shadow*100:.3f}%")
        print(f"Mean Return Delta:           {mean_delta*10000:.1f} bps")
        print(f"Sum Return (Triggered):      {sum_trig*100:.2f}% ({len(triggered_returns)} trades)")
        print(f"Sum Return (Untriggered):    {sum_untrig*100:.2f}% ({len(untriggered_returns)} trades)")
        
    print(f"\nOutput written to {output_path}")

if __name__ == "__main__":
    main()
