class DecisionBrief:
    def __init__(self, admission_decision, state_snapshot, ic_output):
        """
        Immutable record freezing the T0 context, the state, and the evidence.
        """
        self.timestamp = admission_decision.timestamp
        self.action = admission_decision.action
        self.direction = admission_decision.direction
        self.state_snapshot = state_snapshot.to_dict() if state_snapshot else None
        self.ic_evidence = ic_output.to_dict() if ic_output else None
        
    def to_dict(self):
        return {
            "timestamp": self.timestamp,
            "action": self.action,
            "direction": self.direction,
            "state_snapshot": self.state_snapshot,
            "ic_evidence": self.ic_evidence
        }
