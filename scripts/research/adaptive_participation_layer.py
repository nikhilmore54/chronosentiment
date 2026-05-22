import json

class AdaptiveParticipationLayer:
    def __init__(self, profiles_path="scripts/ecology_profiles.json"):
        with open(profiles_path, 'r') as f:
            self.ecology_profiles = json.load(f)

    def evaluate_policy(self, asset, state, collapse_risk):
        """
        Determines the execution policy based on the asset's ecology profile, 
        current execution state, and real-time collapse risk score.
        """
        ecology = self.ecology_profiles.get(asset, {})
        ecology_type = ecology.get("type", "unknown")
        
        policy_action = "MAINTAIN_EXPOSURE"
        sizing_multiplier = 1.0
        threshold_modifier = 1.0
        reason = "Environment is stable and permissive."

        # BTC-USD Policy (Fragile Explosive)
        if ecology_type == "fragile_explosive":
            if collapse_risk > 0.45:
                policy_action = "REDUCE_EXPOSURE"
                sizing_multiplier = 0.30  # Reduce size by 70%
                threshold_modifier = 1.5  # Widen thresholds
                reason = f"BTC abruptly collapses. Elevated risk ({collapse_risk:.2f}) triggers severe defensive sizing."
            elif state == "COMPRESSING":
                policy_action = "REDUCE_EXPOSURE"
                sizing_multiplier = 0.50
                reason = "BTC entered compression. Reducing exposure early due to high transition risk to COLLAPSED."

        # ETH-USD Policy (Collapse Trap)
        elif ecology_type == "collapse_trap":
            if state == "COMPRESSING" or collapse_risk > 0.60:
                policy_action = "LIQUIDATE_POSITIONS"
                sizing_multiplier = 0.0
                reason = "ETH entered compression trap. Statistical recovery is slow (avg 5 ticks). Hard abort."

        # SOL-USD Policy (Elastic Resilient)
        elif ecology_type == "elastic_resilient":
            if state == "COMPRESSING":
                policy_action = "MAINTAIN_EXPOSURE"
                sizing_multiplier = 1.0
                reason = "SOL is elastic. Compression is highly survivable. Maintaining participation."
            elif collapse_risk > 0.85: # Extremely high threshold for SOL
                policy_action = "REDUCE_EXPOSURE"
                sizing_multiplier = 0.50
                reason = "SOL showing rare structural breakdown. Halving exposure."

        return {
            "asset": asset,
            "ecology": ecology_type,
            "state": state,
            "risk_score": collapse_risk,
            "action": policy_action,
            "sizing_multiplier": sizing_multiplier,
            "threshold_modifier": threshold_modifier,
            "reason": reason
        }

if __name__ == "__main__":
    layer = AdaptiveParticipationLayer()
    
    # Test cases mapped to our recent topologies
    scenarios = [
        ("BTC-USD", "EXPANSIVE", 0.52),    # High risk but expansive
        ("ETH-USD", "COMPRESSING", 0.30),  # Low risk but compressing trap
        ("SOL-USD", "COMPRESSING", 0.60),  # Moderate risk but elastic
    ]
    
    print("\n" + "="*80)
    print("🛡️ PHASE 2C.1: ADAPTIVE PARTICIPATION LAYER")
    print("="*80)
    
    for asset, state, risk in scenarios:
        decision = layer.evaluate_policy(asset, state, risk)
        print(f"\n[{asset}] State: {state} | Risk: {risk:.2f}")
        print(f"  Action   : {decision['action']}")
        print(f"  Sizing   : {decision['sizing_multiplier']}x")
        print(f"  Reasoning: {decision['reason']}")
