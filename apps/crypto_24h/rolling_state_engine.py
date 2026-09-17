import math

class RollingStateSnapshot:
    def __init__(self, timestamp, volatility_1m_std_24h, upside_excursion_24h, downside_excursion_24h, trend_dir,
                 trend_return_15m, trend_return_60m, trend_return_240m,
                 volatility_std_60m, volatility_std_240m, volatility_ratio_60m_24h, volatility_ratio_240m_24h,
                 persistence_60m, persistence_240m, volume_acceleration_60m):
        self.timestamp = timestamp
        self.volatility_1m_std_24h = volatility_1m_std_24h
        self.upside_excursion_24h = upside_excursion_24h
        self.downside_excursion_24h = downside_excursion_24h
        self.trend_dir = trend_dir
        
        self.trend_return_15m = trend_return_15m
        self.trend_return_60m = trend_return_60m
        self.trend_return_240m = trend_return_240m
        
        self.volatility_std_60m = volatility_std_60m
        self.volatility_std_240m = volatility_std_240m
        self.volatility_ratio_60m_24h = volatility_ratio_60m_24h
        self.volatility_ratio_240m_24h = volatility_ratio_240m_24h
        
        self.persistence_60m = persistence_60m
        self.persistence_240m = persistence_240m
        self.volume_acceleration_60m = volume_acceleration_60m
        
    def to_dict(self):
        return {
            "timestamp": self.timestamp,
            "volatility_1m_std_24h": self.volatility_1m_std_24h,
            "upside_excursion_24h": self.upside_excursion_24h,
            "downside_excursion_24h": self.downside_excursion_24h,
            "trend_dir": self.trend_dir,
            "trend_return_15m": self.trend_return_15m,
            "trend_return_60m": self.trend_return_60m,
            "trend_return_240m": self.trend_return_240m,
            "volatility_std_60m": self.volatility_std_60m,
            "volatility_std_240m": self.volatility_std_240m,
            "volatility_ratio_60m_24h": self.volatility_ratio_60m_24h,
            "volatility_ratio_240m_24h": self.volatility_ratio_240m_24h,
            "persistence_60m": self.persistence_60m,
            "persistence_240m": self.persistence_240m,
            "volume_acceleration_60m": self.volume_acceleration_60m
        }

class RollingStateEngine:
    def __init__(self):
        """
        crypto_state_v0.2
        Synchronous state estimator. 
        Updates on each observation, using only observations <= t.
        
        Provenance note: volume_acceleration_60m is an informational state coordinate,
        not an admission rule. It passed Stages A-F (Feature Census, Temporal Stability, 
        Conditioning, Prospective Validation, Directional Decomposition, Directional Mapping).
        """
        self.state_history = []
        
    def _calc_volatility(self, returns_slice):
        if len(returns_slice) < 2:
            return 0.0
        mean = sum(returns_slice) / len(returns_slice)
        var = sum((x - mean)**2 for x in returns_slice) / len(returns_slice)
        return math.sqrt(var)

    def _calc_persistence(self, returns_slice):
        if not returns_slice:
            return None
        sum_abs_rets = sum(abs(r) for r in returns_slice)
        if sum_abs_rets == 0:
            return None
        return abs(sum(returns_slice)) / sum_abs_rets

    def update(self, obs_store):
        """
        Calculates the state snapshot strictly using the current observation store.
        """
        recent = obs_store.get_recent(1440) # Last 24 hours of 1m bars
        
        # 1. Warm-up gate: requires exactly 1440 bars
        if not recent or len(recent) < 1440:
            return None
            
        # 2. Continuity check: Ensure there are no missing minutes
        # Binance timestamps are in milliseconds. 1 minute = 60000 ms.
        expected_duration = (1440 - 1) * 60000
        actual_duration = recent[-1]["timestamp"] - recent[0]["timestamp"]
        if actual_duration != expected_duration:
            return None
            
        current_obs = recent[-1]
        closes = [b["close"] for b in recent]
        
        if len(closes) > 1:
            returns_24h = [math.log(closes[i] / closes[i-1]) for i in range(1, len(closes))]
        else:
            returns_24h = []
            
        # Volatilities
        volatility_1m_std_24h = self._calc_volatility(returns_24h)
        volatility_std_60m = self._calc_volatility(returns_24h[-60:]) if len(returns_24h) >= 60 else 0.0
        volatility_std_240m = self._calc_volatility(returns_24h[-240:]) if len(returns_24h) >= 240 else 0.0
        
        volatility_ratio_60m_24h = (volatility_std_60m / volatility_1m_std_24h) if volatility_1m_std_24h > 0 else None
        volatility_ratio_240m_24h = (volatility_std_240m / volatility_1m_std_24h) if volatility_1m_std_24h > 0 else None
        
        # Trend returns (using window slices relative to current price)
        def calc_trend_return(window):
            if len(closes) > window:
                return math.log(closes[-1] / closes[-(window+1)])
            return None
            
        trend_return_15m = calc_trend_return(15)
        trend_return_60m = calc_trend_return(60)
        trend_return_240m = calc_trend_return(240)
        
        # Persistence
        persistence_60m = self._calc_persistence(returns_24h[-60:]) if len(returns_24h) >= 60 else None
        persistence_240m = self._calc_persistence(returns_24h[-240:]) if len(returns_24h) >= 240 else None
        
        # Volume Acceleration (v0.2)
        vol_last_30m = sum(c["volume"] for c in recent[-30:])
        vol_prev_30m = sum(c["volume"] for c in recent[-60:-30])
        volume_acceleration_60m = (
            vol_last_30m / vol_prev_30m
            if vol_prev_30m > 0
            else 1.0
        )
        
        # Existing features
        highs = [b["high"] for b in recent]
        lows = [b["low"] for b in recent]
        
        start_px = closes[0]
        end_px = closes[-1]
        
        upside_excursion_24h = (max(highs) - start_px) / start_px if start_px > 0 else 0
        downside_excursion_24h = (min(lows) - start_px) / start_px if start_px > 0 else 0
        
        trend_dir = 1 if end_px > start_px else (-1 if end_px < start_px else 0)
        
        snapshot = RollingStateSnapshot(
            timestamp=current_obs["timestamp"],
            volatility_1m_std_24h=volatility_1m_std_24h,
            upside_excursion_24h=upside_excursion_24h,
            downside_excursion_24h=downside_excursion_24h,
            trend_dir=trend_dir,
            trend_return_15m=trend_return_15m,
            trend_return_60m=trend_return_60m,
            trend_return_240m=trend_return_240m,
            volatility_std_60m=volatility_std_60m,
            volatility_std_240m=volatility_std_240m,
            volatility_ratio_60m_24h=volatility_ratio_60m_24h,
            volatility_ratio_240m_24h=volatility_ratio_240m_24h,
            persistence_60m=persistence_60m,
            persistence_240m=persistence_240m,
            volume_acceleration_60m=volume_acceleration_60m
        )
        
        self.state_history.append(snapshot)
        return snapshot
