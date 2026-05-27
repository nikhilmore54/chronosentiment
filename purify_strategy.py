import os
import re

TARGETS = [
    "infrastructure/optimization/src/evolution_engine.rs",
    "financial/strategies/src/edge_decay.rs",
    "financial/strategies/src/edge_half_life_estimator.rs",
    "financial/strategies/src/ensemble.rs",
    "financial/strategies/src/paper.rs",
    "financial/strategies/src/pipeline.rs",
    "financial/strategies/src/reco.rs",
    "financial/strategies/src/strategy_id.rs",
    "financial/strategies/src/strategy_ranking.rs",
    "financial/strategies/src/exit.rs",
    "financial/strategies/src/pnl_overlay.rs",
    "financial/strategies/src/replay_evaluator.rs",
    "infrastructure/core/src/lib.rs",
    "infrastructure/core/src/ese.rs",
    "financial/strategies/src/bin/financial_replay.rs"
]

REPLACEMENTS = [
    (r"\bStrategy\b", "Candidate"),
    (r"\bstrategy\b", "candidate"),
    (r"\bstrategies\b", "candidates"),
    (r"\bStrategies\b", "Candidates"),
]

def main():
    for filepath in TARGETS:
        if not os.path.exists(filepath):
            continue
        with open(filepath, 'r') as f:
            content = f.read()
            
        new_content = content
        for pattern, replacement in REPLACEMENTS:
            new_content = re.sub(pattern, replacement, new_content)
            
        if new_content != content:
            with open(filepath, 'w') as f:
                f.write(new_content)
            print(f"Updated {filepath}")

if __name__ == "__main__":
    main()
