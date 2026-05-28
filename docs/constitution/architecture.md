# Constitutional Architecture of ChronoSentiment

## 1. The Core Inversion Principle
ChronoSentiment operates on a strict separation between **Mechanical Optimization** and **Semantic Evaluation**. The Genetic Algorithm (GA) is mathematically pure and domain-blind. It knows only bytes, mutations, and abstract fitness scores. 

All financial semantics—assets, regimes, classification, PnL, signaling, and execution—are explicitly owned by the runtime domain layers (`strategies` and `financial_core`).

## 2. Dependency Direction
The crate topology strictly enforces acyclic dependencies flowing downward into the mechanical abstraction:
- `api` → depends on `strategies`
- `strategies` → depends on `financial_core`
- `financial_core` → depends on `optimization` (for pure strategy structures only)
- `optimization` → depends on **NOTHING** (No financial awareness)

## 3. Layer Authority and Forbidden Operations
### `infrastructure/optimization`
**Authority**: Candidate evolution, crossover, mutation, ranking, and population mechanics.
**Forbidden**: Importing `chronosentiment_strategies` or `chronosentiment_financial_core`. Containing any domain vocabulary (`PnL`, `Trade`, `Bull/Bear`, `Replay`).

### `financial/core`
**Authority**: Market chronology, event streams, timestamp monotonicity, deterministic latency, and replay mechanics.
**Forbidden**: Knowing about orchestration, pipelines, or strategy evaluation semantics.

### `financial/strategies`
**Authority**: The Semantic Bridge. Maps mechanical eval to domain logic. Owns `FinancialEvaluator`, signal generation, asset/regime loops, and semantic reporting.
**Forbidden**: Re-implementing mathematical GA evolution mechanics.

## 4. Test Ownership and Golden Replay
Testing is layered strictly by architectural ownership:
- **Optimization**: Evolution stability and seed determinism.
- **Financial Core**: Causality and execution sequence.
- **Strategies**: Evaluation scoring boundaries and semantic labeling.
- **Golden Fixtures**: Found in `fixtures/replay/`, these freeze captured slices of market data alongside their verified output, ensuring the pipeline immune system detects any silent semantic drift.

## 5. Violation Rules
Any code change that requires adding a reverse dependency edge (e.g., passing a `Regime` into the `Candidate` mutation path) is a **Constitutional Violation** and must be entirely rejected during CI gates.

## 6. Constitutional Rule: Semantic Drift Requires Explicit Authorization
Any change that modifies:
- Replay hashes
- Signal outputs
- Evaluation ordering
- Classification boundaries
- Deterministic execution traces
- Orchestration aggregation semantics

**Must**:
1. Explicitly document the semantic delta.
2. Regenerate fixture expectations (`.expected.json`) intentionally.
3. Include rationale for the drift.
4. Pass replay recertification.

Silent semantic drift is considered a constitutional violation.


## 7. Constitutional Violations
| Violation | Severity |
| :--- | :--- |
| dependency inversion breach | critical |
| silent semantic drift | critical |
| nondeterministic replay | critical |
| fixture mutation without recertification | critical |
| vocabulary leakage into optimization | critical |

## Infrastructure Migration Discipline
**No semantic authority extraction may proceed until replay equivalence is re-certified.**
Every architectural decomposition step must be accompanied by a formal migration manifest (e.g., `docs/migrations/phase5_reporting_extraction.md`) and must pass the complete adversarial replay certification loop.

## Orchestration Canonicalization Invariant

All orchestration execution must operate on a canonicalized asset ordering prior to iteration.

Canonicalization must:

* be deterministic,
* be independent of insertion order,
* be independent of thread scheduling,
* be independent of unordered collection iteration behavior.

Any change affecting:

* asset ordering,
* orchestration traversal,
* execution projection generation,
* canonicalization policy,
* replay-to-orchestration mapping,

requires full re‑certification against:

* replay hashes,
* orchestration projection hashes,
* adversarial permutation fixtures,
* serialization boundary equivalence.

This binds execution order to constitutional certification.
