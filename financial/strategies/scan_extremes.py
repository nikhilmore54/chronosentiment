import yfinance as yf
import datetime

symbols = ['SPY', 'QQQ', 'NVDA', 'TSLA', 'GME', 'AMC', 'MSTR']
def scan_extremes():
    for sym in symbols:
        df = yf.download(sym, period='60d', interval='5m')
        if df.empty:
            continue
            
        closes = df['Close']
        if hasattr(closes, "columns"):
            closes = closes.iloc[:, 0]
            
        max_up = 0
        up_time = None
        max_down = 0
        down_time = None
        
        for i in range(len(closes) - 12):
            window = closes.iloc[i:i+12]
            time_diff = window.index[-1] - window.index[0]
            if time_diff.total_seconds() > 90 * 60:
                continue
                
            move = window.iloc[-1] - window.iloc[0]
            pct_move = move / window.iloc[0] * 100
            
            if pct_move > max_up:
                max_up = pct_move
                up_time = window.index[0]
            elif pct_move < max_down:
                max_down = pct_move
                down_time = window.index[0]
                
        print(f"{sym}: Max Up = {max_up:.2f}% at {up_time}, Max Down = {max_down:.2f}% at {down_time}")

scan_extremes()
