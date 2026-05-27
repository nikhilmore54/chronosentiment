import yfinance as yf
import datetime

# Fetch 60 days of 5m data
spy = yf.download('SPY', period='60d', interval='5m')
qqq = yf.download('QQQ', period='60d', interval='5m')
nvda = yf.download('NVDA', period='60d', interval='5m')

def scan_ruptures(df, name):
    if df.empty:
        print(f"No data for {name}")
        return
        
    closes = df['Close']
    if hasattr(closes, "columns") and len(closes.columns) > 0:
        closes = closes.iloc[:, 0]
        
    max_rupture = 0
    best_time = None
    
    # 5m data, 12 bars = 60 minutes
    for i in range(len(closes) - 12):
        window = closes.iloc[i:i+12]
        
        # Check if continuous (less than 90 minutes between first and last)
        time_diff = window.index[-1] - window.index[0]
        if time_diff.total_seconds() > 90 * 60:
            continue
            
        rupture = window.max() - window.min()
        pct_rupture = rupture / window.min() * 100
        
        if pct_rupture > max_rupture:
            max_rupture = pct_rupture
            best_time = window.index[0]
            
    print(f"{name} Max 60m Rupture (5m timeframe): {max_rupture:.2f}% starting at {best_time}")

scan_ruptures(spy, "SPY")
scan_ruptures(qqq, "QQQ")
scan_ruptures(nvda, "NVDA")
