# Transitional Artifacts Registry

**Authority:** Constitutional Governance Document §21  
**Status:** Living document — append-only, never delete entries  
**Purpose:** Track every stub, shim, bridge, and temporary computation that exists
in the system pending a canonical backend implementation.

---

## Registry Format

Each entry follows this schema:

```
ARTIFACT-NNN
Status:      ACTIVE | ELIMINATED | SUPERSEDED
Registered:  YYYY-MM-DD
Eliminated:  YYYY-MM-DD (if applicable)
Location:    file path(s)
Description: what it does
Sunset:      condition under which it can be removed
Dependent:   what depends on it
```

---

## Active Artifacts

### ARTIFACT-001
```
ARTIFACT-001
Status:      ACTIVE
Registered:  2026-05-24
Location:    my-chrono-sentiment-ui/src/components/GlobalRanking.js
Description: Frontend fallback resolution — reads execution_fitness from
             multiple possible field names (execution_fitness, execution_score,
             live_score, fitness, score) because backend field name was not
             yet stabilised.
Sunset:      Backend canonically emits execution_fitness in all ranking
             responses. Frontend reads only execution_fitness.
Dependent:   GlobalRanking.js score display, sorting, and comparison logic.
```

### ARTIFACT-002
```
ARTIFACT-002
Status:      ELIMINATED
Registered:  2026-05-24
Eliminated:  2026-05-25
Location:    my-chrono-sentiment-ui/src/components/StrategyInspector.js
Description: groupAndNarrateEvents() — frontend function that synthesised
             narrative_blocks[] from raw execution_trace events. Violated
             Law One (observer-only UI). Used PascalCase event type strings
             ('OrderIntent', 'OrderEnteredQueue', etc.) — non-canonical.
Sunset:      ACHIEVED — backend now emits certified narrative_blocks[] via
             POST /inspect_strategy. Frontend consumes directly.
Dependent:   getOrderId() — also eliminated 2026-05-25.
```

### ARTIFACT-003
```
ARTIFACT-003
Status:      ACTIVE
Registered:  2026-05-24
Location:    my-chrono-sentiment-ui/src/components/CompareStrategies.js
Description: Frontend fallback resolution for execution_fitness in strategy
             comparison — same multi-field fallback pattern as ARTIFACT-001.
Sunset:      Backend canonically emits execution_fitness in all comparison
             responses.
Dependent:   CompareStrategies.js fitness display and diff logic.
```

### ARTIFACT-004
```
ARTIFACT-004
Status:      ACTIVE
Registered:  2026-05-24
Location:    my-chrono-sentiment-ui/src/components/RunGA.js
Description: Frontend fallback resolution for execution_fitness in GA run
             results — same multi-field fallback pattern as ARTIFACT-001.
Sunset:      Backend canonically emits execution_fitness in all GA run
             responses.
Dependent:   RunGA.js fitness display and leaderboard logic.
```

### ARTIFACT-005
```
ARTIFACT-005
Status:      ACTIVE
Registered:  2026-05-24
Location:    services/api/src/handlers/inspect_strategy.rs
Description: Stub narrative_blocks[] generation in inspect_strategy_handler —
             generates placeholder narrative text from event types rather than
             a proper semantic narrative engine.
Sunset:      A canonical NarrativeEngine service generates certified
             narrative_blocks[] with proper semantic content, causal linkage,
             and key event detection.
Dependent:   StrategyInspector.js narrative display, NarrativeBlock.js
             rendering.
```

### ARTIFACT-006
```
ARTIFACT-006
Status:      ACTIVE
Registered:  2026-05-24
Location:    services/api/src/handlers/inspect_strategy.rs
Description: Stub execution_trace generation — generates synthetic trace
             events from strategy parameters rather than replaying actual
             market data through the strategy engine.
Sunset:      A canonical ReplayEngine replays actual market data through
             the strategy, producing a certified execution_trace[].
Dependent:   StrategyInspector.js trace display, execution summary.
```

### ARTIFACT-007
```
ARTIFACT-007
Status:      ACTIVE
Registered:  2026-05-24
Location:    services/api/src/handlers/inspect_strategy.rs
Description: Stub certification_state logic — assigns CERTIFIED/DEGRADED/
             UNCERTIFIED based on simple heuristics (event count, type
             presence) rather than cryptographic lineage verification.
Sunset:      A canonical CertificationEngine performs cryptographic lineage
             verification (causal closure, replay parity, chronology
             integrity) and emits a certified verdict.
Dependent:   StrategyInspector.js certification badge display.
```

### ARTIFACT-008
```
ARTIFACT-008
Status:      ACTIVE
Registered:  2026-05-24
Location:    services/api/src/certify.rs
Description: Stub trace_signature generation — computes SHA-256 over
             serialised execution_trace[] but does not include causal
             lineage, replay session ID, or kernel_signature chain.
Sunset:      trace_signature includes causal lineage hash, replay session
             ID, and kernel_signature chain — enabling full cryptographic
             replay verification.
Dependent:   StrategyInspector.js signature display, verify_manifest.py.
```

### ARTIFACT-009
```
ARTIFACT-009
Status:      ACTIVE
Registered:  2026-05-25
Location:    my-chrono-sentiment-ui/src/components/StrategyInspector.js
             (function: normalizeNarrativeBlock)
Description: Field-name bridge between backend snake_case narrative_blocks[]
             fields (sequence_id, parent_sequence_id, is_key_event,
             key_event_marker) and the camelCase shape expected by
             NarrativeBlock.js and StrategyColumn.js. Necessary because the
             backend emits snake_case JSON and the React components use
             camelCase property access.
Sunset:      Backend emits camelCase narrative_blocks[] natively, or a
             canonical JS SDK handles the snake_case → camelCase mapping
             at the API boundary. Frontend components read snake_case
             directly, or a shared normalisation layer is introduced.
Dependent:   NarrativeBlock.js (id, parentId, isKeyEvent, keyEventMarker),
             StrategyColumn.js (narrative block rendering),
             getCausalChain() in StrategyInspector.js.
```

### ARTIFACT-010
```
ARTIFACT-010
Status:      ACTIVE
Registered:  2026-05-25
Location:    my-chrono-sentiment-ui/src/components/StrategyInspector.js
             (function: compareNarrativeBlocks)
Description: Frontend divergence analysis between two backend-certified
             narrative_blocks[] arrays. Computes divergencePoints[] and
             convergenceScore by comparing group and narrative fields
             position-by-position. This computation belongs in the backend
             as a canonical divergence_analysis[] field in
             CanonicalInspectResponse.
Sunset:      Backend emits divergence_analysis[] in CanonicalInspectResponse
             when two strategy IDs are submitted for comparison. Frontend
             renders the backend-computed divergence data directly without
             local computation.
Dependent:   ComparisonPanels.js divergence display,
             showDivergenceAnalysis state in StrategyInspector.js.
```

### ARTIFACT-011
```
ARTIFACT-011
Status:      ACTIVE
Registered:  2026-05-25
Location:    my-chrono-sentiment-ui/src/components/StrategyInspector.js
             (functions: getExecutionSummary, CONFIDENCE_LEVELS, and
             finalVerdict/confidenceLevel computation block)
Description: Frontend execution summary and confidence verdict derived from
             backend narrative_blocks[]. Computes totalSteps, partialFills,
             queueProgressions, hasQueueProgression, totalFills, finalVerdict,
             confidenceLevel, and confidenceReason locally. These computations
             belong in the backend as canonical execution_summary{} and
             confidence_verdict{} fields in CanonicalInspectResponse.
Sunset:      Backend emits execution_summary{} and confidence_verdict{}
             objects in CanonicalInspectResponse. Frontend renders these
             fields directly without local computation.
Dependent:   ComparisonPanels.js execution summary display and verdict panel,
             executionSummary1/2 and finalVerdict state in StrategyInspector.js.
```

---

## Eliminated Artifacts

*(See ARTIFACT-002 above — entries remain in the Active section with
`Status: ELIMINATED` to preserve the append-only audit trail.)*

---

## Lifecycle Rules

1. **Never delete** an entry — set `Status: ELIMINATED` and record the date.
2. **Register before shipping** — every stub, shim, or bridge must be
   registered here before it reaches `main`.
3. **Sunset conditions are binding** — when the condition is met, the
   artifact must be eliminated within one sprint.
4. **In-source annotation required** — every artifact must have a
   `// ARTIFACT-NNN: ...` comment at its definition site.
5. **Automated expiration checks** — Phase 3 P6 will add CI checks that
   alert when a sunset condition has been met but the artifact has not
   been eliminated.

---

## Summary Table

| ID           | Status      | Location                          | Description (short)                        |
|--------------|-------------|-----------------------------------|--------------------------------------------|
| ARTIFACT-001 | ACTIVE      | GlobalRanking.js                  | Frontend fitness fallback resolution       |
| ARTIFACT-002 | ELIMINATED  | StrategyInspector.js              | groupAndNarrateEvents() frontend synthesis |
| ARTIFACT-003 | ACTIVE      | CompareStrategies.js              | Frontend fitness fallback resolution       |
| ARTIFACT-004 | ACTIVE      | RunGA.js                          | Frontend fitness fallback resolution       |
| ARTIFACT-005 | ACTIVE      | inspect_strategy.rs               | Stub narrative_blocks[] generation         |
| ARTIFACT-006 | ACTIVE      | inspect_strategy.rs               | Stub execution_trace generation            |
| ARTIFACT-007 | ACTIVE      | inspect_strategy.rs               | Stub certification_state logic             |
| ARTIFACT-008 | ACTIVE      | certify.rs                        | Stub trace_signature (no causal lineage)   |
| ARTIFACT-009 | ACTIVE      | StrategyInspector.js              | normalizeNarrativeBlock snake→camel bridge |
| ARTIFACT-010 | ACTIVE      | StrategyInspector.js              | compareNarrativeBlocks frontend divergence |
| ARTIFACT-011 | ACTIVE      | StrategyInspector.js              | getExecutionSummary/confidence verdict     |

---

*Last updated: 2026-05-25*