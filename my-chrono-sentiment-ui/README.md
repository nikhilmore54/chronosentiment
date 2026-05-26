# ChronoSentiment Observatory UI
**Version:** Phase 2 — Propagation-Aware Causal Observatory  
**Last Updated:** 2026-05-26  
**Location:** `my-chrono-sentiment-ui/`

---

## System Identity

The Observatory UI is a **schema-bound causal replay instrument**. It projects backend-certified causal propagation topology, divergence state, and execution traces into navigable visual surfaces.

**Core invariant:** The frontend projects relationships — it does not invent relationships.

This is not an analytics dashboard. It is not a smart frontend explaining markets. It is a deterministic projection surface attached to a causal simulation system.

---

## Data Authority Model

All rendered data derives from backend-certified sources. The frontend never:
- Computes certification state
- Derives causal relationships independently
- Synthesizes divergence classification
- Fabricates operational state

### Canonical Schema References

| Schema | Authority |
|--------|-----------|
| `schemas/canonical/replay_response.schema.json` | Replay blocks, `certification_state`, `causal_chain`, `portfolio_state` |
| `schemas/canonical/observatory_state.schema.json` | `system_phase`, `kernel_state.*`, `governor_state.*`, `snapshot_sequence_id` |
| `schemas/canonical/governor_telemetry.schema.json` | Governor telemetry |

### Registered Bridge Artifacts (Sunset Conditions)

| ID | Description | Sunset Condition |
|----|-------------|-----------------|
| ARTIFACT-009 | `normalizeNarrativeBlock` — snake_case → camelCase bridge | Backend emits camelCase `narrative_blocks[]` natively |
| ARTIFACT-010 | `compareNarrativeBlocks` — frontend divergence analysis | Backend emits `divergence_analysis[]` in `CanonicalInspectResponse` |
| ARTIFACT-011 | `getExecutionSummary` — frontend execution summary | Backend emits `execution_summary` in `CanonicalInspectResponse` |

---

## Workspace Views

### Run GA (`/run-ga`)
Executes the genetic algorithm against the configured signal universe. Sidebar exposes signal filter controls (Top-K per asset, weak signal inclusion, strong-only filter) in pre-execution state. Results show strategy fitness scores and allow direct navigation to Inspect Strategy.

### Inspect Strategy (`/inspect-strategy`)
The primary causal replay surface. Accepts one or two strategy IDs for single or dual-mode inspection.

**Sidebar:**
- Replay position slider (Seq min → max)
- Certification badge: CERTIFIED / DEGRADED / PARTIAL / INVALID (from `certification_state`)
- Event count at current replay position
- "Jump to end" button when slider is not at maximum

**Main surface:**
- Causal trace header with inline certification badge and divergence accumulation summary
- Execution narrative stream (NarrativeBlock instances)
- Causal ancestry panel with depth counter and ancestry path label
- Forward propagation panel showing direct children of selected block

**Dual mode:**
- Side-by-side strategy columns
- Divergence accumulation summary: total count + type distribution at current replay position, filtered by `<= currentMaxSeqId`

### Compare Strategies (`/compare-strategies`)
Compares two or more strategies via `POST /compare_strategies`. Results derive from `comparison_summary` API fields — no hardcoded metrics. Ranking table, execution summary comparison, execution insights, and divergence analysis panels.

### Global Ranking (`/global-ranking`)
Fetches `GET /ga/global-ranking`. Shimmer skeleton during load; `.cs-empty` when no data; unified error display.

---

## Component Documentation

### `App.js`
Shell layout. Manages workspace routing, strategy selection state, and system status.

**`useSystemStatus()` hook:**
- Tries `GET /observatory` first (canonical `ObservatoryState` per `observatory_state.schema.json`)
- Falls back to `GET /health` with legacy field mapping
- Falls back to null defaults with `online: false` — no fabricated operational state
- Maps: `system_phase`, `throttle_state`, `cohort_id`, `active_cohort_size`, `queue_depth`, `fill_latency_ns`, `sync_ratio`, `events_per_second`, `snapshot_sequence_id`

**Operational awareness strip fields:**
- Phase (`system_phase`) with color-coded status dot
- Throttle (`governor_state.throttle_state`) — OPEN/THROTTLED/CLOSED
- Cohort (`governor_state.cohort_id`) with active cohort size
- Queue depth (`kernel_state.queue_depth`)
- Fill latency (`kernel_state.fill_latency_ns`) — formatted as ns/µs/ms
- Sync ratio (`kernel_state.sync_ratio`) — color-coded display threshold (≥0.95 green, ≥0.8 amber, <0.8 red)
- Events per second (`kernel_state.events_per_second`)
- Snapshot sequence ID (`snapshot_sequence_id`) — chronology anchor

### `StrategyInspector.js`
Primary causal replay orchestrator. Manages dual-mode inspection, replay slider, causal chain traversal, and divergence analysis.

**Key computations (all pure derivations from backend data):**
- `narratedExecutionTrace1/2`: slider-filtered view of `narrative_blocks[]`
- `activeChain`: ancestor set via `parentId` traversal through `eventMap`
- `divergenceStatements`: `compareNarrativeBlocks()` on two certified `narrative_blocks[]` arrays (ARTIFACT-010)
- `visibleDivergences`: `divergenceStatements` filtered by `<= currentMaxSeqId` — preserves temporal epistemic integrity
- `divergenceTypeCounts`: type distribution of `visibleDivergences`

### `StrategyColumn.js`
Per-strategy projection coordinator. Renders the execution narrative stream and causal inspection panels.

**Key helpers:**
- `isBlockDivergent(id)`: checks `divergenceStatements` for block involvement
- `getBlockDivergenceMessage(blockId)`: retrieves divergence message for a block
- `getBlockDivergenceType(blockId)`: retrieves divergence type for badge rendering
- `getForwardChildren(blockId)`: filters `eventMap` by `parentId === blockId` — pure derivation from backend-certified topology

**Causal ancestry panel:**
- Depth counter (`depth N`)
- Ancestry path label (`3 → 7 → 12`)
- Step-by-step chain (sorted descending by seq id)
- Forward propagation section: direct children, clickable for graph traversal, terminal node detection

### `NarrativeBlock.js`
Pure presentational component. Renders a single certified narrative block.

**Rendered fields:**
- `block.group` — event group label
- `block.id` — sequence ID (monospace)
- `block.timestamp` — temporal position (right-aligned monospace, conditionally rendered)
- `block.isKeyEvent` — filled purple badge (`★ KEY` or `block.keyEventMarker`)
- `block.narrative` — certified narrative text
- `block.parentId` — causal parent reference (`↳ derived from seq:N`)
- Divergence indicator: type badge + message (6-type taxonomy)

**Transition-aware causal arrow:**
- Red + `⚑` flag: next block is divergent
- Blue + `→ {group}`: group transition (next block has different group)
- Default `--tm`: same group, no divergence
- Priority: divergent > group transition > default

### `ComparisonPanels.js`
Pure presentational component. All data via props. No hardcoded values. Renders execution summary comparison, execution insights, final verdict, and divergence analysis.

### `CompareStrategies.js`
Comparison workspace. Fetches `POST /compare_strategies`. Results rendered from `comparison_summary` API fields. Loading skeleton; unified error display; `.cs-empty` pre-execution state.

### `GlobalRanking.js`
Ranking workspace. Fetches `GET /ga/global-ranking`. Shimmer skeleton (6 rows); `.cs-empty` empty state; unified error display.

### `RunGA.js`
GA execution workspace. Signal filter controls visible pre-execution. Loading skeleton; unified error display; store status indicator.

---

## CSS Design System

**Design tokens (`:root` in `index.css`):**

| Token | Value | Usage |
|-------|-------|-------|
| `--bred` | `rgba(220,38,38,.3)` | Red border |
| `--bamb` | `rgba(194,122,90,.3)` | Amber border |
| `--bgrn` | `rgba(61,122,94,.3)` | Green border |
| `--bblu` | `rgba(79,107,255,.3)` | Blue border |
| `--rdim` | `rgba(220,38,38,.1)` | Red background dim |
| `--adim` | `rgba(194,122,90,.1)` | Amber background dim |
| `--gdim` | `rgba(61,122,94,.1)` | Green background dim |
| `--bdim` | `rgba(79,107,255,.1)` | Blue background dim |
| `--r4/r8/r10/r12` | `4px/8px/10px/12px` | Border radius |

**Key CSS classes:**
- `.cs-nav-item` / `.cs-nav-item.active` — left-border accent nav pattern
- `.cs-status-dot` — animated status indicator (`.grn`, `.amb`, `.red`, `.blu` variants)
- `.cs-skeleton` — shimmer loading animation via `@keyframes cs-shimmer`
- `.cs-empty` / `.cs-empty-icon` / `.cs-empty-title` — unified empty/pending state
- `.cs-trace-block` — narrative block container (`.intent`, `.queue`, `.execution`, `.other` group modifiers)
- `.cs-causal-arrow` — inter-block transition indicator
- `.cs-causal-chain` / `.cs-causal-step` — ancestry chain container

---

## Observability Capability Matrix

| Principle | Status |
|-----------|--------|
| UI observes reality | VERY STRONG |
| Non-execution is valid | VERY STRONG |
| Outcomes are emergent | STRONG |
| Replay explainability | STRONG |
| Deterministic projection | VERY STRONG |
| Observatory identity | VERY STRONG |
| Causal continuity | STRONG |
| Event-driven cognition | STRONG |
| Forward propagation visibility | STRONG |
| Transition morphology | STRONG |
| Divergence accumulation | STRONG |
| Temporal epistemic integrity | VERY STRONG |

---

## Backend API Endpoints

| Endpoint | Method | Consumer |
|----------|--------|----------|
| `/observatory` | GET | `useSystemStatus()` — primary |
| `/health` | GET | `useSystemStatus()` — fallback |
| `/run_ga` | POST | `RunGA.js` |
| `/signals/latest` | GET | `RunGA.js` |
| `/ga/strategy-store` | GET | `RunGA.js` |
| `/inspect_strategy` | POST | `StrategyInspector.js` |
| `/compare_strategies` | POST | `CompareStrategies.js` |
| `/ga/global-ranking` | GET | `GlobalRanking.js` |

---

## Development

```bash
cd my-chrono-sentiment-ui
npm install
npm start
```

Backend expected at `http://localhost:8000`. All API calls fail gracefully — the UI renders null fields as `—` and shows `.cs-empty` states when data is absent.