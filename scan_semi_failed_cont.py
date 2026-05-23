import yfinance as yf
import time

symbols = ['NVDA', 'AMD']
data = {}
for sym in symbols:
    success = False
    retries = 3
    while not success and retries > 0:
        df = yf.download(sym, period='60d', interval='5m')
        if df.empty:
            print(f"Empty df for {sym}, retrying...")
            retries -= 1
            time.sleep(2)
            continue
            
        closes = df['Close']
        if hasattr(closes, "columns") and len(closes.columns) > 0:
            data[sym] = closes.iloc[:, 0]
        else:
            data[sym] = closes
        success = True

if 'NVDA' in data and 'AMD' in data:
    boba = data['NVDA']
    bmah = data['AMD']
    
    common_indices = boba.index.intersection(bmah.index)
    
    for i in range(len(common_indices) - 61):
        start_idx = common_indices[i]
        end_idx = common_indices[i+60]
        
        time_diff = end_idx - start_idx
        if time_diff.total_seconds() > 7 * 3600:
            continue
            
        boba_window = boba[start_idx:end_idx]
        bmah_window = bmah[start_idx:end_idx]
        
        if len(boba_window) < 61 or len(bmah_window) < 61:
            continue
            
        boba_start = boba_window.iloc[0]
        boba_max = boba_window.max()
        boba_end = boba_window.iloc[-1]
        
        bmah_start = bmah_window.iloc[0]
        bmah_max = bmah_window.max()
        bmah_end = bmah_window.iloc[-1]
        
        boba_up = (boba_max - boba_start) / boba_start * 100
        boba_fade = (boba_end - boba_max) / boba_max * 100
        
        bmah_up = (bmah_max - bmah_start) / bmah_start * 100
        bmah_fade = (bmah_end - bmah_max) / bmah_max * 100
        
        # Upward move >= 1.0%, then stalled (fade is between -0.5% and 0.0%)
        if boba_up >= 1.0 and -0.7 <= boba_fade <= -0.1 and bmah_up >= 1.0 and -0.7 <= bmah_fade <= -0.1:
            boba_max_idx = boba_window.argmax()
            bmah_max_idx = bmah_window.argmax()
            # Max reached relatively early, then stalls
            if 5 < boba_max_idx < 35 and 5 < bmah_max_idx < 35:
                print(f"Failed Continuation at {start_idx}: NVDA (Up {boba_up:.2f}%, Stall {boba_fade:.2f}%), AMD (Up {bmah_up:.2f}%, Stall {bmah_fade:.2f}%)")
