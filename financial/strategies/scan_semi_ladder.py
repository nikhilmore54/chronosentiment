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
        
        # Weak Sync Drop (-0.5% to -1.0%)
        if -1.0 < boba_pct <= -0.5 and -1.0 < bmah_pct <= -0.5:
            print(f"Weak Sync Drop at {start_idx}: NVDA {boba_pct:.2f}%, AMD {bmah_pct:.2f}%")
            
        # Moderate Sync Drop (-1.0% to -2.0%)
        if -2.0 < boba_pct <= -1.0 and -2.0 < bmah_pct <= -1.0:
            print(f"Moderate Sync Drop at {start_idx}: NVDA {boba_pct:.2f}%, AMD {bmah_pct:.2f}%")
            
        # Absorb / Gap Down Stabilization
        # Drop then return in same window
        # To do this, check min vs start and end vs start
        min_boba = min((boba[start_idx:end_idx] - boba[start_idx]) / boba[start_idx] * 100)
        min_bmah = min((bmah[start_idx:end_idx] - bmah[start_idx]) / bmah[start_idx] * 100)
        
        if min_boba < -1.5 and boba_pct > -0.5 and min_bmah < -1.5 and bmah_pct > -0.5:
            print(f"Absorbed Sync Drop at {start_idx}: NVDA Min {min_boba:.2f}%, End {boba_pct:.2f}%; AMD Min {min_bmah:.2f}%, End {bmah_pct:.2f}%")
