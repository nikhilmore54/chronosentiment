class CryptoICOutput:
    def __init__(self, timestamp, raw_signal, confidence, model_version="stub_v1", feature_version="stub_v1"):
        self.timestamp = timestamp
        self.raw_signal = raw_signal        # float: positive for LONG, negative for SHORT
        self.confidence = confidence        # float: 0.0 to 1.0, or None if uncalibrated
        self.model_version = model_version
        self.feature_version = feature_version
        
    def to_dict(self):
        return {
            "timestamp": self.timestamp,
            "raw_signal": self.raw_signal,
            "confidence": self.confidence,
            "model_version": self.model_version,
            "feature_version": self.feature_version
        }

class NativeCryptoIC:
    def __init__(self):
        """
        Research/Validation layer for Crypto signal generation.
        Currently a stub to establish the exact evaluation contract.
        """
        pass
        
    def evaluate(self, state_snapshot) -> CryptoICOutput:
        """
        Causal evaluation.

        Inputs:
            State(t), constructed exclusively from observations <= t.

        Outputs:
            raw_signal in [-1, +1]
            confidence in [0, 1] once calibrated (None during early research)

        Forbidden:
            future observations
            future returns
            lifecycle state
            E4 outcomes
            admission decisions
            position state
        """
        if not state_snapshot:
            return None
            
        # STUB LOGIC: For V1 scaffolding, we just generate a deterministic mock signal 
        # based on the trend direction to prove the pipeline flows correctly.
        # This is NOT a real predictive strategy.
        raw_signal = float(state_snapshot.trend_dir) * 0.5 
        
        # Confidence is None until empirically calibrated from model probability
        confidence = None
        
        return CryptoICOutput(
            timestamp=state_snapshot.timestamp,
            raw_signal=raw_signal,
            confidence=confidence,
            model_version="stub_v1",
            feature_version="stub_v1"
        )
