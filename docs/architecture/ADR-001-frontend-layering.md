# ADR-001: Frontend Layering

## Status
Accepted

## Context
The UltraCrew Pilot Portal frontend was originally developed as a monolithic React application, with business logic, view rendering, state management, and API calls all concentrated within `App.js`. This structure led to cognitive complexity and introduced React Hook violations (error #310) due to conditional hook execution within IIFEs. 

To restore stability and align the frontend with Coralys's architectural principles, the application was refactored into a structured, layered architecture under **RP-601**.

## Decisions

### 1. Decomposition of App.js
**Decision:** `App.js` has been decomposed into distinct UI components (`ResultsWorkspace`, `DecisionWorkspace`, `GanttChart`, etc.).
**Rationale:** This ensures that `App.js` only handles high-level state and orchestration. Each bounded component now owns its specific responsibility, reducing cognitive overhead and establishing clear component ownership.

### 2. Adapters vs. Models
**Decision:** We established an `adapters/` directory instead of a `models/` directory for data transformation logic.
**Rationale:** The frontend does not own the canonical domain models—the Coralys backend does. The frontend modules exist strictly to adapt backend DTOs into frontend view models (and vice versa). Calling them "adapters" enforces this boundary and respects the RP-600 governance principle against duplicating top-level concepts.

### 3. Deliberate Exclusion of React Context
**Decision:** React Context and external state management frameworks were purposefully not introduced. State is passed via props.
**Rationale:** Context is valuable when dozens of deeply nested, unrelated components need to read the same global state. For a single-page application with a shallow component tree, prop drilling is simpler and keeps data dependencies explicit. We chose not to over-engineer the state layer.

### 4. Unified Planning and Recovery Service Layer
**Decision:** Both optimization and recovery workflows share the same service layer (`runOptimization`).
**Rationale:** Planning and recovery are mathematically identical optimization problems in Coralys. Recovery simply introduces a different request payload containing locked assignments and disruption constraints. Maintaining this symmetry in the frontend preserves the unified optimization architecture established in the backend.

### 5. ResultsWorkspace Ownership
**Decision:** `ResultsWorkspace` acts as a container that owns everything related to an optimization result (including `DecisionWorkspace`, metrics, and constraints), rather than being a sibling to them under `App.js`.
**Rationale:** This establishes a clean dependency graph (`App -> ResultsWorkspace(result) -> Everything Else`). Features like `DecisionWorkspace` only exist once a solution has been produced, so placing them under the Result container prevents `App.js` from passing the same `result` object to multiple siblings.

## Consequences
- **Positive:** Complete elimination of React hook violations, improved readability, explicit component ownership, and architectural consistency with the Coralys backend.
- **Negative:** Slightly increased file count and prop passing, which is an acceptable trade-off for architectural clarity and maintainability.
