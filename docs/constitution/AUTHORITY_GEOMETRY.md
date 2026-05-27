# Constitutional Authority Geometry

The system enforces strict directional authority geometry through its compile-time topological structure. The dependency graph serves as mechanical enforcement of constitutional law.

## The Layer Geometry

Authority flows downward. Semantics must never migrate into mechanics.

| Layer                          | Responsibility                      | May Know                      |
|--------------------------------|-------------------------------------|-------------------------------|
| `infrastructure/core`          | Causal history + Replay equivalence | Chronology, replay, causality |
| `infrastructure/optimization`  | Deterministic search mechanics      | Scoring exists                |
| `infrastructure/observatory`   | Evidentiary attestation             | Replay evidence               |
| `financial/ese`                | Execution reality                   | Execution semantics           |
| `financial/strategies`         | Semantic intent + Orchestration     | Scoring meaning               |

## Optimization Substrate Constraint

The optimization layer must remain entirely domain-blind.

```text
Optimization may rank candidates
without understanding financial meaning.
```

It must **never**:
- Know finance
- Know alpha
- Know markets
- Know portfolios
- Know reward/risk semantics

It only knows:
- Candidate populations
- Ranking
- Mutation
- Convergence mechanics

The `FitnessEvaluator<T>` trait serves as the singular semantic evacuation gate, ensuring the optimization substrate relies entirely on `/financial/strategies` to supply meaning without taking ownership of it.
