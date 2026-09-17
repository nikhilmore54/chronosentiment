import math

class RollingStateSnapshot:
    def __init__(self, timestamp, volatility_5m_std_24h, upside_excursion_24h, downside_excursion_24h, trend_dir):
        self.timestamp = timestamp
        self.volatility_5m_std_24h = volatility_5m_std_24h
        self.upside_excursion_24h = upside_excursion_24h
        self.downside_excursion_24h = downside_excursion_24h
        self.trend_dir = trend_dir
        
    def to_dict(self):
        return {
            "timestamp": self.timestamp,
            "volatility_5m_std_24h": self.volatility_5m_std_24h,
            "upside_excursion_24h": self.upside_excursion_24h,
            "downside_excursion_24h": self.downside_excursion_24h,
            "trend_dir": self.trend_dir
        }

class RollingStateEngine:
    def __init__(self):
        """
        Synchronous state estimator for 5m substrate.
        Updates on each observation, using only observations <= t.
        """
        self.state_history = []
        
    def update(self, obs_store):
        """
        Calculates the state snapshot strictly using the current observation store.
        """
        recent = obs_store.get_recent(288) # Last 24 hours of 5m bars
        
        # 1. Warm-up gate: requires exactly 288 bars
        if not recent or len(recent) < 288:
            return None
            
        # 2. Continuity check: Ensure there are no missing 5m candles
        # Binance timestamps are in milliseconds. 5 minute = 300000 ms.
        expected_duration = (288 - 1) * 300000
        actual_duration = recent[-1]["timestamp"] - recent[0]["timestamp"]
        if actual_duration != expected_duration:
            return None
            
        current_obs = recent[-1]
        
        # Calculate some basic state features
        closes = [b["close"] for b in recent]
        if len(closes) > 1:
            returns = [math.log(closes[i] / closes[i-1]) for i in range(1, len(closes))]
            mean = sum(returns) / len(returns)
            var = sum((x - mean)**2 for x in returns) / len(returns)
            volatility_5m_std_24h = math.sqrt(var)
        else:
            volatility_5m_std_24h = 0.0
            
        highs = [b["high"] for b in recent]
        lows = [b["low"] for b in recent]
        
        start_px = closes[0]
        end_px = closes[-1]
        
        upside_excursion_24h = (max(highs) - start_px) / start_px if start_px > 0 else 0
        downside_excursion_24h = (min(lows) - start_px) / start_px if start_px > 0 else 0
        
        trend_dir = 1 if end_px > start_px else (-1 if end_px < start_px else 0)
        
        snapshot = RollingStateSnapshot(
            timestamp=current_obs["timestamp"],
            volatility_5m_std_24h=volatility_5m_std_24h,
            upside_excursion_24h=upside_excursion_24h,
            downside_excursion_24h=downside_excursion_24h,
            trend_dir=trend_dir
        )
        
        self.state_history.append(snapshot)
        return snapshot
