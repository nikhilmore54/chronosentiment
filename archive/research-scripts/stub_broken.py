import os
import re

def aggressive_stub():
    # Stub pipeline.rs functions that need D3 rewrite
    path = "financial/strategies/src/pipeline.rs"
    if os.path.exists(path):
        with open(path, "r") as f:
            code = f.read()
        
        # Replace generate_latest_signals_with_thresholds body
        pattern = r"(pub fn generate_latest_signals_with_thresholds.*?\)\s*->\s*SignalsSnapshot\s*\{).*?(?=\n\n(?:pub fn|fn))"
        code = re.sub(pattern, r"\1\n    unimplemented!()\n}", code, flags=re.DOTALL)
        
        with open(path, "w") as f:
            f.write(code)

    # Stub reco.rs functions that need D2 rewrite
    path = "financial/strategies/src/reco.rs"
    if os.path.exists(path):
        with open(path, "r") as f:
            code = f.read()
            
        pattern = r"(pub fn evaluate_cluster.*?\)\s*->\s*Option<RecoResult>\s*\{).*?(?=\n\n(?:pub fn|fn))"
        code = re.sub(pattern, r"\1\n    unimplemented!()\n}", code, flags=re.DOTALL)
        
        with open(path, "w") as f:
            f.write(code)

if __name__ == "__main__":
    aggressive_stub()
