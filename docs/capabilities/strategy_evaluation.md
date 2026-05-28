# Capability: Strategy Evaluation

**Owner**: `financial/strategies/src/evaluation`
**Consumers**: `financial/strategies/src/pipeline`

## Description
Bridges the gap between mechanical evaluation and semantic domain intent via `FinancialEvaluator`.

## Invariants
- Accurately translates candidate fitness to actual market PnL.
- Ensures valid classifications of candidate behaviors.
- Generates `SemanticEvaluationReport` without passing execution engine bloat upstream.