import json
import sys
from datetime import datetime, timezone
from pathlib import Path
import math

sys.path.append("/Users/nikhil/ChronoSentiment_MEGA_FINAL/scripts")
import candidate_validation_time_machine_v0_1 as cv

# Hack simulate_trade to also return exit_price and exit_timestamp
original_simulate_trade = cv.simulate_trade

def sim_trade_detailed(decision, target_mul, stop_mul):
    ticker = decision["ticker"]
    as_of = decision["as_of"]
    as_of_dt = datetime.strptime(as_of, "%Y-%m-%dT%H:%M:%SZ").replace(tzinfo=timezone.utc)
    as_of_ts = as_of_dt.timestamp()
    
    direction = decision["direction"]
    entry_price = float(decision["entry_price"]) if "entry_price" in decision else float(decision.get("reference_price", 0.0))
    base_target = float(decision.get("adaptive_target", 0.0))
    base_risk = float(decision.get("adaptive_risk", 0.0))
    
    target_price = entry_price + (base_target - entry_price) * target_mul if target_mul else base_target
    stop_price = entry_price + (base_risk - entry_price) * stop_mul if stop_mul else base_risk
    
    horizon_bars = cv.get_horizon(decision)
    bars = cv.load_bars(ticker)
    
    post_bars = [b for b in bars if b["timestamp"] > as_of_ts]
    window = post_bars[:horizon_bars]
    
    sim_exit = "HORIZON"
    sim_ret = 0.0
    hit_exit = False
    exit_price = 0.0
    exit_ts = 0
    
    for i, b in enumerate(window):
        o, h, l, c = b["open"], b["high"], b["low"], b["close"]
        ts = b["timestamp"]
        
        if direction == "LONG":
            if i == 0 and o >= target_price:
                sim_ret = (o - entry_price) / entry_price
                sim_exit = "TARGET_GAP_THROUGH"
                hit_exit = True
                exit_price = o
                exit_ts = ts
                break
            if i == 0 and o <= stop_price:
                sim_ret = (o - entry_price) / entry_price
                sim_exit = "RISK_GAP_THROUGH"
                hit_exit = True
                exit_price = o
                exit_ts = ts
                break
            if h >= target_price:
                sim_ret = (target_price - entry_price) / entry_price
                sim_exit = "TARGET"
                hit_exit = True
                exit_price = target_price
                exit_ts = ts
                break
            if l <= stop_price:
                sim_ret = (stop_price - entry_price) / entry_price
                sim_exit = "RISK"
                hit_exit = True
                exit_price = stop_price
                exit_ts = ts
                break
                
        elif direction == "SHORT":
            if i == 0 and o <= target_price:
                sim_ret = (entry_price - o) / entry_price
                sim_exit = "TARGET_GAP_THROUGH"
                hit_exit = True
                exit_price = o
                exit_ts = ts
                break
            if i == 0 and o >= stop_price:
                sim_ret = (entry_price - o) / entry_price
                sim_exit = "RISK_GAP_THROUGH"
                hit_exit = True
                exit_price = o
                exit_ts = ts
                break
            if l <= target_price:
                sim_ret = (entry_price - target_price) / entry_price
                sim_exit = "TARGET"
                hit_exit = True
                exit_price = target_price
                exit_ts = ts
                break
            if h >= stop_price:
                sim_ret = (entry_price - stop_price) / entry_price
                sim_exit = "RISK"
                hit_exit = True
                exit_price = stop_price
                exit_ts = ts
                break

    if not hit_exit:
        if window:
            last_close = window[-1]["close"]
            sim_ret = (last_close - entry_price) / entry_price if direction == "LONG" else (entry_price - last_close) / entry_price
            exit_price = last_close
            exit_ts = window[-1]["timestamp"]
        else:
            sim_ret = 0.0
            
    return {
        "sim_return": sim_ret,
        "sim_exit_reason": sim_exit,
        "exit_price": exit_price,
        "exit_ts": exit_ts,
        "decision": decision,
        "target_price": target_price
    }


decisions = []
for d_dir in (cv.TIME_MACHINE_DIR / "ledger").glob("*"):
    if not d_dir.is_dir(): continue
    for f in (d_dir / "entries").glob("*.json"):
        try:
            decisions.append(json.loads(f.read_text()))
        except: pass

valid_decisions = []
for v in decisions:
    signal = v.get("signal", v.get("direction", v.get("action", "")))
    if signal in ["Buy", "Sell", "LONG", "SHORT"]:
        v["direction"] = "LONG" if signal in ["Buy", "LONG"] else "SHORT"
        v["entry_price"] = v.get("entry_price", v.get("reference_price", 0.0))
        valid_decisions.append(v)

base_sims = [sim_trade_detailed(d, 1.0, 1.0) for d in valid_decisions]
cand_sims = [sim_trade_detailed(d, 1.25, 1.0) for d in valid_decisions]

print(f"{'Ticker':<15} | {'Dir':<5} | {'Ref Px':<9} | {'Base Tgt':<9} | {'Cand Tgt':<9} | {'B Exit R':<18} | {'C Exit R':<18} | {'B Exit Px':<9} | {'C Exit Px':<9} | {'B Ret':<8} | {'C Ret':<8} | {'B Exit TS':<19} | {'C Exit TS':<19}")
print("-" * 180)

for b, c in zip(base_sims, cand_sims):
    # Only print trades where the return or the reason changed
    if b["sim_exit_reason"] != c["sim_exit_reason"] or abs(b["sim_return"] - c["sim_return"]) > 1e-6:
        dec = b["decision"]
        tick = dec["ticker"]
        dir = dec["direction"]
        ref = dec["entry_price"]
        
        btgt = b["target_price"]
        ctgt = c["target_price"]
        
        bexr = b["sim_exit_reason"]
        cexr = c["sim_exit_reason"]
        
        bexp = b["exit_price"]
        cexp = c["exit_price"]
        
        bret = b["sim_return"] * 100
        cret = c["sim_return"] * 100
        
        bts = datetime.fromtimestamp(b["exit_ts"], timezone.utc).strftime("%Y-%m-%d %H:%M") if b["exit_ts"] else "None"
        cts = datetime.fromtimestamp(c["exit_ts"], timezone.utc).strftime("%Y-%m-%d %H:%M") if c["exit_ts"] else "None"
        
        print(f"{tick:<15} | {dir:<5} | {ref:<9.2f} | {btgt:<9.2f} | {ctgt:<9.2f} | {bexr:<18} | {cexr:<18} | {bexp:<9.2f} | {cexp:<9.2f} | {bret:<7.2f}% | {cret:<7.2f}% | {bts:<19} | {cts:<19}")
