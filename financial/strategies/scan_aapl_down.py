import yfinance as yf
import datetime

aapl = yf.download('AAPL', period='60d', interval='5m')
if not aapl.empty:
    closes = aapl['Close'].iloc[:, 0] if hasattr(aapl['Close'], 'columns') else aapl['Close']
    max_down = 0
    down_time = None
    for i in range(len(closes) - 12):
        window = closes.iloc[i:i+12]
        time_diff = window.index[-1] - window.index[0]
        if time_diff.total_seconds() > 90 * 60:
            continue
        move = window.iloc[-1] - window.iloc[0]
        pct = move / window.iloc[0] * 100
        if pct < max_down:
            max_down = pct
            down_time = window.index[0]
    print(f"AAPL Max Down: {max_down:.2f}% at {down_time}")
