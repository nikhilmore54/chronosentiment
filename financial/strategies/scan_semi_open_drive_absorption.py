import yfinance as yf
import pandas as pd
from datetime import timedelta

def scan_open_drive_absorption(symbol):
    df = yf.download(symbol, period='60d', interval='5m')
    if df.empty:
        return []
        
    df.index = df.index.tz_convert('America/New_York')
    df['Date'] = df.index.date
    df['Time'] = df.index.time
    
    dates = df['Date'].unique()
    matches = []
    
    for d in dates:
        day_data = df[df['Date'] == d].copy()
        if len(day_data) < 30: # need enough data for the day
            continue
            
        # Define phases
        # Phase 1: Open Drive (09:30 to 10:30)
        open_drive = day_data.between_time('09:30', '10:30')
        # Phase 2: Absorption (10:30 to 13:00)
        absorption = day_data.between_time('10:35', '13:00')
        
        if open_drive.empty or absorption.empty:
            continue
            
        try:
            open_price = float(open_drive['Open'].iloc[0].iloc[0] if isinstance(open_drive['Open'], pd.DataFrame) else open_drive['Open'].iloc[0])
            drive_high = float(open_drive['High'].max().iloc[0] if isinstance(open_drive['High'], pd.DataFrame) else open_drive['High'].max())
            drive_low = float(open_drive['Low'].min().iloc[0] if isinstance(open_drive['Low'], pd.DataFrame) else open_drive['Low'].min())
            drive_close = float(open_drive['Close'].iloc[-1].iloc[0] if isinstance(open_drive['Close'], pd.DataFrame) else open_drive['Close'].iloc[-1])
        except Exception as e:
            continue
            
        drive_move_pct = (drive_close - open_price) / open_price * 100
        
        drive_move_pct = (drive_close - open_price) / open_price * 100
        
        try:
            drive_vol = float(open_drive['Volume'].mean().iloc[0] if isinstance(open_drive['Volume'], pd.DataFrame) else open_drive['Volume'].mean())
            abs_high = float(absorption['High'].max().iloc[0] if isinstance(absorption['High'], pd.DataFrame) else absorption['High'].max())
            abs_low = float(absorption['Low'].min().iloc[0] if isinstance(absorption['Low'], pd.DataFrame) else absorption['Low'].min())
            abs_vol = float(absorption['Volume'].mean().iloc[0] if isinstance(absorption['Volume'], pd.DataFrame) else absorption['Volume'].mean())
        except Exception:
            continue
        
        vol_drop_pct = (drive_vol - abs_vol) / drive_vol * 100
        stall_range_pct = (abs_high - abs_low) / abs_low * 100
        
        matches.append({
            'date': str(d),
            'dir': 'UP' if drive_move_pct > 0 else 'DOWN',
            'drive_pct': drive_move_pct,
            'vol_drop_pct': vol_drop_pct,
            'stall_range_pct': stall_range_pct,
            'drive_close': drive_close,
            'abs_high': abs_high,
            'abs_low': abs_low
        })
            
    return matches

print("Scanning NVDA for Open-Drive Absorption...")
nvda_matches = scan_open_drive_absorption('NVDA')
print("\nScanning AMD for Open-Drive Absorption...")
amd_matches = scan_open_drive_absorption('AMD')

print("\n--- NVDA Matches ---")
for m in nvda_matches:
    print(m)
    
print("\n--- AMD Matches ---")
for m in amd_matches:
    print(m)
