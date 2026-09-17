class AdmissionDecision:
    def __init__(self, timestamp, action, direction=None):
        self.timestamp = timestamp
        self.action = action            # "ACT" or "WAIT"
        self.direction = direction      # "LONG" or "SHORT" or None
        
    def to_dict(self):
        return {
            "timestamp": self.timestamp,
            "action": self.action,
            "direction": self.direction
        }

class AdmissionGate:
    def __init__(self, signal_threshold=0.3, confidence_threshold=0.5):
        """
        Event-driven T0 generator. 
        Evaluates IC output against stable thresholds.
        """
        self.signal_threshold = signal_threshold
        self.confidence_threshold = confidence_threshold
        
    def evaluate(self, ic_output):
        if not ic_output:
            return AdmissionDecision(None, "WAIT")
            
        if ic_output.confidence >= self.confidence_threshold:
            if ic_output.raw_signal >= self.signal_threshold:
                return AdmissionDecision(ic_output.timestamp, "ACT", "LONG")
            elif ic_output.raw_signal <= -self.signal_threshold:
                return AdmissionDecision(ic_output.timestamp, "ACT", "SHORT")
                
        return AdmissionDecision(ic_output.timestamp, "WAIT")
