import yfinance as yf

symbols = ['NVDA', 'AMD']
data = {}
for sym in symbols:
    df = yf.download(sym, period='60d', interval='5m')
    if not df.empty:
        closes = df['Close']
        if hasattr(closes, "columns") and len(closes.columns) > 0:
            data[sym] = closes.iloc[:, 0]
        else:
            data[sym] = closes

if 'NVDA' in data and 'AMD' in data:
    boba = data['NVDA']
    bmah = data['AMD']
    
    common_indices = boba.index.intersection(bmah.index)
    
    for i in range(len(common_indices) - 12):
        start_idx = common_indices[i]
        end_idx = common_indices[i+11]
        
        time_diff = end_idx - start_idx
        if time_diff.total_seconds() > 90 * 60:
            continue
            
        boba_pct = (boba[end_idx] - boba[start_idx]) / boba[start_idx] * 100
        bmah_pct = (bmah[end_idx] - bmah[start_idx]) / bmah[start_idx] * 100
        
        # Weak Sync Rally
        if 0.5 <= boba_pct < 1.0 and 0.5 <= bmah_pct < 1.0:
            # check if we have enough ticks after
            if i + 60 < len(common_indices) and (common_indices[i+60] - start_idx).total_seconds() < 7 * 3600:
                print(f"Weak Sync Rally at {start_idx}: NVDA {boba_pct:.2f}%, AMD {bmah_pct:.2f}%")
                
        # Moderate Sync Rally
        if 1.0 <= boba_pct < 2.0 and 1.0 <= bmah_pct < 2.0:
            if i + 60 < len(common_indices) and (common_indices[i+60] - start_idx).total_seconds() < 7 * 3600:
                print(f"Moderate Sync Rally at {start_idx}: NVDA {boba_pct:.2f}%, AMD {bmah_pct:.2f}%")
