import yfinance as yf
import datetime

spy = yf.download('SPY', period='60d', interval='5m')
qqq = yf.download('QQQ', period='60d', interval='5m')
nvda = yf.download('NVDA', period='60d', interval='5m')
tsla = yf.download('TSLA', period='60d', interval='5m')
gme = yf.download('GME', period='60d', interval='5m')

def scan_upward_ruptures(df, name):
    if df.empty:
        return
        
    closes = df['Close']
    if hasattr(closes, "columns") and len(closes.columns) > 0:
        closes = closes.iloc[:, 0]
        
    max_upward = 0
    best_time = None
    
    for i in range(len(closes) - 12):
        window = closes.iloc[i:i+12]
        time_diff = window.index[-1] - window.index[0]
        if time_diff.total_seconds() > 90 * 60:
            continue
            
        upward_move = window.max() - window.min()
        # Ensure it's a net positive move from start to end (so it's a rally, not a crash and recover)
        if window.iloc[-1] > window.iloc[0]:
            pct_upward = upward_move / window.min() * 100
            
            if pct_upward > max_upward:
                max_upward = pct_upward
                best_time = window.index[0]
            
    print(f"{name} Max 60m Upward Rally (5m timeframe): {max_upward:.2f}% starting at {best_time}")

scan_upward_ruptures(spy, "SPY")
scan_upward_ruptures(qqq, "QQQ")
scan_upward_ruptures(nvda, "NVDA")
scan_upward_ruptures(tsla, "TSLA")
scan_upward_ruptures(gme, "GME")
