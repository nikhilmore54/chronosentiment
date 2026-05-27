import yfinance as yf
import datetime

symbols = ['MAHABANK.NS', 'BANKBARODA.NS', 'IDEA.NS', 'PNB.NS', 'CANBK.NS']
data = {}
for sym in symbols:
    df = yf.download(sym, period='60d', interval='5m')
    if not df.empty:
        closes = df['Close']
        if hasattr(closes, "columns") and len(closes.columns) > 0:
            data[sym] = closes.iloc[:, 0]
        else:
            data[sym] = closes

# Let's find windows where both Bank of Baroda and Bank of Maharashtra dropped
if 'BANKBARODA.NS' in data and 'MAHABANK.NS' in data:
    boba = data['BANKBARODA.NS']
    bmah = data['MAHABANK.NS']
    
    max_down = 0
    best_time = None
    
    # We just need to find a window where both exist
    common_indices = boba.index.intersection(bmah.index)
    
    for i in range(len(common_indices) - 12):
        start_idx = common_indices[i]
        end_idx = common_indices[i+11]
        
        # Ensure it's continuous
        time_diff = end_idx - start_idx
        if time_diff.total_seconds() > 90 * 60:
            continue
            
        boba_pct = (boba[end_idx] - boba[start_idx]) / boba[start_idx] * 100
        bmah_pct = (bmah[end_idx] - bmah[start_idx]) / bmah[start_idx] * 100
        
        # Looking for a synchronized downward move
        if boba_pct < -2.0 and bmah_pct < -2.0:
            combined = boba_pct + bmah_pct
            if combined < max_down:
                max_down = combined
                best_time = start_idx
                print(f"Sync Drop at {start_idx}: BANKBARODA {boba_pct:.2f}%, MAHABANK {bmah_pct:.2f}%")

