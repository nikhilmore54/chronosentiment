import yfinance as yf
import time

symbols = ['NVDA', 'AMD']
data = {}
vols = {}
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
        volumes = df['Volume']
        if hasattr(closes, "columns") and len(closes.columns) > 0:
            data[sym] = closes.iloc[:, 0]
            vols[sym] = volumes.iloc[:, 0]
        else:
            data[sym] = closes
            vols[sym] = volumes
        success = True

if 'NVDA' in data and 'AMD' in data:
    boba = data['NVDA']
    bmah = data['AMD']
    
    vboba = vols['NVDA']
    vbmah = vols['AMD']
    
    common_indices = boba.index.intersection(bmah.index)
    
    for i in range(len(common_indices) - 61):
        start_idx = common_indices[i]
        end_idx = common_indices[i+60]
        
        time_diff = end_idx - start_idx
        if time_diff.total_seconds() > 7 * 3600:
            continue
            
        boba_window = boba[start_idx:end_idx]
        bmah_window = bmah[start_idx:end_idx]
        
        vboba_window = vboba[start_idx:end_idx]
        vbmah_window = vbmah[start_idx:end_idx]
        
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
        
        boba_max_idx = boba_window.argmax()
        bmah_max_idx = bmah_window.argmax()
        
        if boba_max_idx > 0 and bmah_max_idx > 0:
            vol_boba_early = vboba_window.iloc[:boba_max_idx].mean()
            vol_boba_late = vboba_window.iloc[boba_max_idx:].mean()
            vol_bmah_early = vbmah_window.iloc[:bmah_max_idx].mean()
            vol_bmah_late = vbmah_window.iloc[bmah_max_idx:].mean()
            
            boba_vol_drop = (vol_boba_late - vol_boba_early) / vol_boba_early * 100
            bmah_vol_drop = (vol_bmah_late - vol_bmah_early) / vol_bmah_early * 100
            
            # Midday vacuum: early move, progressive decay/drift
            if boba_up >= 0.8 and -1.0 <= boba_fade <= -0.4 and bmah_up >= 0.8 and -1.0 <= bmah_fade <= -0.4:
                # Volume must drop significantly
                if boba_vol_drop <= -30.0 and bmah_vol_drop <= -30.0:
                    # Occurs midday/late morning
                    if start_idx.hour >= 13 and start_idx.hour <= 18:
                        if 10 < boba_max_idx < 30 and 10 < bmah_max_idx < 30:
                            print(f"Vacuum at {start_idx}: NVDA (Up {boba_up:.2f}%, Fade {boba_fade:.2f}%, Vol Drop {boba_vol_drop:.2f}%), AMD (Up {bmah_up:.2f}%, Fade {bmah_fade:.2f}%, Vol Drop {bmah_vol_drop:.2f}%)")

