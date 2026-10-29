import requests
import json
import time
import math
from pathlib import Path
from datetime import datetime, timedelta

WORKSPACE = Path("/Users/nikhil/ChronoSentiment_MEGA_FINAL")
DATA_DIR = WORKSPACE / "datasets" / "e9" / "crypto"

def fetch_binance_klines(symbol, interval, start_time_ms, end_time_ms):
    url = "https://api.binance.com/api/v3/klines"
    all_klines = []
    current_start = start_time_ms
    
    print(f"Fetching {symbol} {interval} from {datetime.fromtimestamp(start_time_ms/1000)} to {datetime.fromtimestamp(end_time_ms/1000)}")
    
    while current_start < end_time_ms:
        params = {
            "symbol": symbol,
            "interval": interval,
            "startTime": current_start,
            "endTime": end_time_ms,
            "limit": 1000
        }
        resp = requests.get(url, params=params)
        if resp.status_code != 200:
            print(f"Error fetching data: {resp.text}")
            break
            
        data = resp.json()
        if not data:
            break
            
        all_klines.extend(data)
        current_start = data[-1][0] + 1
        time.sleep(0.1) # Rate limit protection
        
    return all_klines

def get_volatility_regimes():
    # 30 days ago
    end_dt = datetime.now()
    start_dt = end_dt - timedelta(days=30)
    
    end_ms = int(end_dt.timestamp() * 1000)
    start_ms = int(start_dt.timestamp() * 1000)
    
    klines = fetch_binance_klines("BTCUSDT", "1m", start_ms, end_ms)
    
    # Calculate 24h rolling realized volatility of 1m closes
    # Volatility formula: std dev of 1m log returns over the last 24h (1440 mins)
    
    closes = [float(k[4]) for k in klines]
    times = [k[0] for k in klines]
    
    log_returns = []
    for i in range(1, len(closes)):
        log_returns.append(math.log(closes[i] / closes[i-1]))
        
    # We want to evaluate non-overlapping 72h windows. 
    # 72h = 4320 minutes
    window_size = 4320
    
    windows = []
    for i in range(0, len(log_returns) - window_size, window_size):
        w_rets = log_returns[i : i + window_size]
        mean = sum(w_rets)/len(w_rets)
        var = sum((x - mean)**2 for x in w_rets) / len(w_rets)
        std = math.sqrt(var)
        
        # Approximate direction (trend vs reversal)
        start_px = closes[i]
        end_px = closes[i + window_size]
        min_px = min(closes[i : i + window_size])
        max_px = max(closes[i : i + window_size])
        
        # A simple reversal metric: distance from min/max to the end price compared to the total range
        trend_size = abs(end_px - start_px)
        total_range = max_px - min_px
        reversal_score = (total_range - trend_size) / total_range if total_range > 0 else 0
        
        windows.append({
            "start_idx": i,
            "start_time": times[i],
            "end_time": times[i + window_size],
            "volatility": std,
            "reversal_score": reversal_score,
            "trend_size": trend_size / start_px,
            "start_px": start_px,
            "end_px": end_px
        })
        
    # Sort by volatility
    windows.sort(key=lambda x: x["volatility"])
    
    low_vol = windows[0]
    high_vol = windows[-1]
    
    # Sort by reversal score (high volatility + high reversal score)
    reversal_candidates = [w for w in windows if w["volatility"] > statistics_median([x["volatility"] for x in windows])]
    reversal_candidates.sort(key=lambda x: x["reversal_score"], reverse=True)
    reversal = reversal_candidates[0] if reversal_candidates else windows[len(windows)//2]
    
    return low_vol, high_vol, reversal

def statistics_median(l):
    s = sorted(l)
    return s[len(s)//2]

def save_regime(name, window, symbols=["BTCUSDT", "ETHUSDT", "SOLUSDT"]):
    print(f"\nRegime: {name}")
    print(f"Start: {datetime.fromtimestamp(window['start_time']/1000)}")
    print(f"End:   {datetime.fromtimestamp(window['end_time']/1000)}")
    print(f"Vol:   {window['volatility']:.6f} | Rev: {window['reversal_score']:.2f}")
    
    for sym in symbols:
        data = fetch_binance_klines(sym, "1m", window['start_time'], window['end_time'])
        
        # Format as standard OHLCV dict for easy consumption
        formatted = []
        for d in data:
            formatted.append({
                "timestamp": int(d[0]/1000),
                "open": float(d[1]),
                "high": float(d[2]),
                "low": float(d[3]),
                "close": float(d[4]),
                "volume": float(d[5])
            })
            
        out_file = DATA_DIR / f"{name}_{sym}.json"
        with open(out_file, "w") as f:
            json.dump(formatted, f)
        print(f"Saved {len(formatted)} bars to {out_file}")

if __name__ == "__main__":
    low, high, rev = get_volatility_regimes()
    save_regime("low_vol", low)
    save_regime("high_vol", high)
    save_regime("reversal", rev)
