import yfinance as yf
import pandas as pd
import numpy as np

def scan_oscillatory_fragmentation(symbol):
    df = yf.download(symbol, period='60d', interval='5m')
    if df.empty:
        return []
        
    df.index = df.index.tz_convert('America/New_York')
    df['Date'] = df.index.date
    
    dates = df['Date'].unique()
    matches = []
    
    for d in dates:
        day_data = df[df['Date'] == d].copy()
        
        # We need a 60-tick window, let's take 09:30 to 14:30 (approx 60 ticks)
        # To be safe, just take the first 60 ticks of the day
        if len(day_data) < 60:
            continue
            
        window = day_data.iloc[:60].copy()
        
        try:
            opens = window['Open'].squeeze() if isinstance(window['Open'], pd.DataFrame) else window['Open']
            closes = window['Close'].squeeze() if isinstance(window['Close'], pd.DataFrame) else window['Close']
            highs = window['High'].squeeze() if isinstance(window['High'], pd.DataFrame) else window['High']
            lows = window['Low'].squeeze() if isinstance(window['Low'], pd.DataFrame) else window['Low']
            vols = window['Volume'].squeeze() if isinstance(window['Volume'], pd.DataFrame) else window['Volume']
            
            first_open = float(opens.iloc[0])
            last_close = float(closes.iloc[-1])
            
            # Net displacement
            net_disp = abs(last_close - first_open) / first_open * 100
            
            # Total absolute path (sum of bar ranges)
            abs_path = (highs - lows).sum() / first_open * 100
            
            # Oscillatory ratio
            if net_disp == 0:
                continue
            ratio = abs_path / net_disp
            
            # Average Volume
            avg_vol = vols.mean()
            
            # Criteria for violent oscillatory fragmentation:
            # 1. High absolute path (lots of movement, > 10%)
            # 2. Low net displacement (ends near where it started, < 1.5%)
            # 3. High ratio (> 10)
            
            if abs_path > 8.0 and net_disp < 1.5 and ratio > 6.0:
                matches.append({
                    'date': str(d),
                    'abs_path_pct': abs_path,
                    'net_disp_pct': net_disp,
                    'ratio': ratio,
                    'avg_vol': avg_vol
                })
        except Exception as e:
            continue
            
    return matches

print("Scanning NVDA for Oscillatory Fragmentation...")
nvda_matches = scan_oscillatory_fragmentation('NVDA')

print("\nScanning AMD for Oscillatory Fragmentation...")
amd_matches = scan_oscillatory_fragmentation('AMD')

print("\n--- NVDA Matches ---")
for m in nvda_matches:
    print(m)
    
print("\n--- AMD Matches ---")
for m in amd_matches:
    print(m)
