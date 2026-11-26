# ChronoSentiment UI/UX Synchronization Status
**Last Updated:** 2026-05-29
**Status:** Phase 3 complete — canonical schema alignment & dual-mode surface
**Identity:** The frontend is a schema-bound causal replay instrument, not an analytics dashboard.

> **Pass 6 classification:** LOW architectural expansion · HIGH architectural alignment.
> Pass 6 does not introduce new authority surfaces, new ontology, or new replay semantics.
> It tightens the coupling between the projection layer and the canonical schema layer,
> reducing translation ambiguity across the Schema → API → UI → Documentation chain.

---

## Core Invariant

> The frontend projects relationships — it does not invent relationships.

All rendered data derives from backend-certified sources. No client-side synthesis of backend-owned data. The frontend materializes explicit backend-certified graph structure — it does not infer influence, predict propagation, or cluster semantic descendants.

---

## UI Identity

The frontend has crossed from "replay viewer with annotations" into "propagation-aware causal replay instrument." This transition was achieved across five passes of UI/UX synchronization without violating Law One at any point.

**What the UI is:**
- A schema-bound observability projection surface
- A deterministic replay traversal instrument
- A causal propagation topology viewer
- A divergence-aware inspection console

**What the UI is not:**
- An interpretive analytics platform
- A smart frontend explaining markets
- A dashboard with runtime conveniences
- A system that synthesizes operational ontology

---

## Architectural Invariants

### Law One Compliance
All frontend data derives from backend-certified sources. The frontend never computes certification, never derives causal relationships independently, never synthesizes divergence classification.

### Temporal Epistemic Integrity
The `<= currentMaxSeqId` filter on divergence accumulation preserves causal temporal locality. The UI only exposes divergences already encountered within replay progression. Future divergence knowledge is not leaked backward through replay state.

### Service Boundary Authority
- `parentId` topology: owned by backend, projected by frontend
- `divergenceStatements`: owned by backend (ARTIFACT-010 bridge), filtered and counted by frontend
- `certification_state`: owned by backend, displayed by frontend
- `narratedExecutionTrace`: owned by backend, traversed by frontend for lookahead
- `system_phase`, `kernel_state.*`, `snapshot_sequence_id`: owned by `observatory_state` schema, projected by `useSystemStatus()`

### Projection Purity
The frontend emphasizes backend-certified transition conditions. It does not decide what is dangerous, invalid, or important — it only surfaces what the backend has certified. Display thresholds (e.g. `sync_ratio` color coding) are perceptual gradients only — they do not redefine system truth.

---

## Synchronization Pass Progression

| Pass | Outcome |
|------|---------|
| Pass 1 | Remove mock state |
| Pass 2 | Replay observability |
| Pass 3 | Canonical observatory telemetry |
| Pass 4 | Divergence visibility |
| Pass 5 | Propagation visibility |
| Pass 6 | Canonical schema convergence |

---

## Synchronization Passes

### Pass 1 — Foundation Audit & CSS/Layout Fixes

| # | Issue | Resolution |
|---|-------|------------|
| 1 | CSS variables `--bred`, `--bamb`, `--bgrn`, `--bblu` missing from `:root` | Added to global CSS root |
| 2 | `--grn` was identical to `--blu` (`#4F6BFF`) — copy-paste error | Fixed to `#3D7A5E` |
| 3 | Border-radius tokens all `0px` | Corrected: `--r4:4px`, `--r8:8px`, `--r10:10px`, `--r12:12px` |
| 4 | `.cs-main` had `max-width:1600px; margin:0 auto` inside flex container | Removed; replaced with `width:100%; overflow-x:hidden` |
| 5 | Left-rail nav active state missing | Added `.cs-nav-item` / `.cs-nav-item.active` with left-border accent |
| 6 | Operational awareness strip hardcoded | Replaced with `useSystemStatus()` hook |
| 7 | RunGA signal filter controls hidden pre-execution | Surfaced in always-visible sidebar section |
| 8 | CompareStrategies hardcoded mock data (`queue_depth=12`, `fill_latency=42ms`, `sync_ratio=0.91`) | Replaced with dynamic rendering from `comparison_summary` API fields |
| 9 | Empty/loading states inconsistent across 4 workspace views | Unified `.cs-empty` pattern; shimmer skeletons added to all four views |
| 10 | Footer clock duplicating header clock | Footer clock removed; footer shows version + cohort name |

### Pass 2 — Schema Alignment & Canonical Field Names

| # | Issue | Resolution |
|---|-------|------------|
| 11 | `useSystemStatus()` using non-canonical field names (`state`, `engine`, `governor`, `cohort`) | Aligned to `system_phase`, `kernel_state.*`, `snapshot_sequence_id` per `observatory_state.schema.json` |
| 12 | Awareness strip not surfacing kernel metrics | Expanded to show `queue_depth`, `fill_latency_ns`, `sync_ratio`, `events_per_second`, `snapshot_sequence_id`, `throttle_state` |
| 13 | CompareStrategies pre-execution idle block: hardcoded "ARMED"/"Deterministic"/"Fitness parity: 100%" | Replaced with `.cs-empty` pattern |
| 14 | StrategyInspector pre-execution idle block: hardcoded "ONLINE"/"Locked" | Replaced with `.cs-empty` pattern |
| 15 | `useSystemStatus()` endpoint strategy: only tried `/health` | Now tries `GET /observatory` first (canonical `ObservatoryState`), falls back to `GET /health`, falls back to null defaults with `online: false` |

### Pass 3 — Replay Observability & Certification Surface

| # | Issue | Resolution |
|---|-------|------------|
| 16 | `certification_state` badge not surfaced | Added CERTIFIED/DEGRADED/PARTIAL/INVALID badge in sidebar replay position block and inline in causal trace header |
| 17 | Causal chain panel lacked depth counter and ancestry path label | Added depth counter (`depth N`), ancestry path breadcrumb (`3 → 7 → 12`), visual hierarchy with card background |
| 18 | Replay position context strip absent | Added event count, seq range, "Jump to end" button, dual-mode event counts |

### Pass 4 — Narrative Block Enhancement & Divergence Visibility

| # | Issue | Resolution |
|---|-------|------------|
| 19 | `block.timestamp` not surfaced in NarrativeBlock | Added right-aligned monospace timestamp display, conditionally rendered |
| 20 | `isKeyEvent` not visually prominent | Replaced inline color text with filled purple badge (`★ KEY`) |
| 21 | Divergence type labels undifferentiated | Added `DIVERGENCE_TYPE_LABELS` map with 6 canonical types, color-coded bordered badges; `blockDivergenceType` prop wired through `StrategyColumn` |

### Pass 5 — Causal Propagation Topology

| # | Issue | Resolution |
|---|-------|------------|
| 22 | Forward propagation not visible — UI only showed historical lineage | Added forward propagation panel in `StrategyColumn`: `getForwardChildren(blockId)` filters `eventMap` by `parentId === blockId`; shows direct children with group label, seq id, divergence flag; terminal nodes render "No downstream events — terminal node"; child rows are clickable for causal graph traversal |
| 23 | Causal arrow was purely decorative — no transition signal | Transition-aware arrow in `NarrativeBlock`: red for divergent next block, blue for group transition, default otherwise; shows group label and `⚑` flag inline; priority: divergent > group transition > default |
| 24 | Divergence accumulation not visible at replay position | Added divergence accumulation summary in `StrategyInspector`: `visibleDivergences` filtered by `<= currentMaxSeqId`; `divergenceTypeCounts` per type; inline badge strip in dual mode; replay-position-aware |

### Pass 6 — Canonical Schema Alignment & Dual-Mode Surface

| # | Issue | Resolution |
|---|-------|------------|
| 25 | `App.js` footer used `sysStatus.cohort` (non-canonical) | Fixed to `sysStatus.cohort_id` per `observatory_state.schema.json` `governor_state.cohort_id` field |
| 26 | `ComparisonPanels` imported in `StrategyInspector.js` but never rendered | Added `ComparisonPanels` render after dual-column grid; wired with all required props (`executionSummary1/2`, `finalVerdict`, `confidenceLevel`, `confidenceColorClass`, `confidenceReason`, `divergenceStatements`) |
| 27 | `StrategyInspector.js` sidebar missing Strategy 2 input fields | Added labeled "Strategy 1" / "Strategy 2 (optional)" sections with ID + Seed inputs; Strategy 2 uses `onBlur` guard (`if (strategyId2)`) to avoid spurious fetches |
| 28 | `StrategyInspector.js` sidebar missing `showRawEvents` toggle | Added `showRawEvents` checkbox toggle in sidebar below the Reconstruct button |
| 29 | `StrategyInspector.js` sidebar missing `error2` display for Strategy 2 errors | Added separate `error2` display block below `error` in sidebar |
| 30 | `normalizeNarrativeBlock()` used `block.timestamp` (legacy field) | Updated to use canonical `timestamp_ns` per `event.schema.json` (`chrono:schema:event:v1`); falls back to `block.timestamp` for backward compatibility (ARTIFACT-009 bridge) |
| 31 | `NarrativeBlock.js` rendered `block.timestamp` (legacy field) | Updated to render `block.timestamp_ns` (canonical nanosecond field) |
| 32 | `getGroupColorClass()` only handled legacy mixed-case group names | Updated to use `group.toUpperCase()` matching canonical `decision_trace.schema.json` enum values (`INTENT`, `QUEUE`, `EXECUTION`, `SETTLEMENT`, `GOVERNANCE`); legacy values still handled |
| 33 | `normalizeNarrativeBlock()` did not map canonical `decision_trace.schema.json` fields | Added `blockType` (from `block_type`), `divergenceScore` (from `divergence_score`) to normalized output |

---

## Observability Capability Matrix

| Capability | Nature | Status |
|------------|--------|--------|
| Replay state | Static | ✓ Complete |
| Divergence state | Categorical | ✓ Complete |
| Lineage (ancestry) | Historical | ✓ Complete |
| Telemetry | Operational | ✓ Complete |
| Forward propagation | Directional | ✓ Complete |
| Transition morphology | Dynamic | ✓ Complete |
| Replay-window divergence accumulation | Temporal | ✓ Complete |
| Downstream topology | Graph-oriented | ✓ Complete |
| Propagation intensity / branching density | Visualization-depth | — Not yet addressed |
| Topology pressure / deformation gradients | Visualization-depth | — Not yet addressed |
| Replay turbulence accumulation | Visualization-depth | — Not yet addressed |

The remaining items are visualization-depth enhancements, not architectural gaps. They remain grounded in backend-certified data when implemented.

---

## Component Status

### `App.js`
- `useSystemStatus()` hook: canonical `observatory_state` schema field mapping; tries `/observatory` then `/health`; null defaults with `online: false` on failure
- Operational awareness strip: 8 fields from canonical schema (`system_phase`, `throttle_state`, `cohort_id`, `active_cohort_size`, `queue_depth`, `fill_latency_ns`, `sync_ratio`, `events_per_second`, `snapshot_sequence_id`)
- Nav: `.cs-nav-item` / `.cs-nav-item.active` with left-border accent and emoji icons
- Rail footer: active `selectedStrategyId` context; `cohort_id` field correctly sourced from `sysStatus.cohort_id`

### `StrategyInspector.js`
- Certification badge: reads `certification_state` + `certification_reason` from API response; color-coded; conditionally rendered
- Replay position context strip: event count, seq range, "Jump to end" button
- Divergence accumulation summary: `visibleDivergences` filtered by `<= currentMaxSeqId`; type distribution badge strip; dual-mode only
- Loading skeleton; error block; idle/results state guards tightened
- Sidebar: labeled "Strategy 1" / "Strategy 2 (optional)" sections with ID + Seed inputs; `showRawEvents` checkbox toggle; `error2` display for Strategy 2 errors
- `ComparisonPanels` rendered after dual-column grid in dual-mode
- `normalizeNarrativeBlock()`: canonical `timestamp_ns` field (falls back to `timestamp`); maps `block_type → blockType`, `divergence_score → divergenceScore`
- `getGroupColorClass()`: handles canonical uppercase group enum values (`INTENT`, `QUEUE`, `EXECUTION`, `SETTLEMENT`, `GOVERNANCE`) and legacy mixed-case values

### `StrategyColumn.js`
- `getForwardChildren(blockId)`: filters `eventMap` by `parentId === blockId`
- `getBlockDivergenceType(blockId)`: reads `.type` from matching divergence statement
- Causal ancestry panel: depth counter, ancestry path label, card background
- Forward propagation panel: direct children, clickable, terminal node detection
- Empty state: `.cs-empty` pattern

### `NarrativeBlock.js`
- `block.timestamp_ns`: right-aligned monospace, conditionally rendered (canonical nanosecond field per `event.schema.json`)
- `isKeyEvent`: filled purple badge with `★ KEY` label
- Divergence type labels: 6-type taxonomy with color-coded bordered badges
- Transition-aware arrow: color + group label + `⚑` flag based on next block state

### `CompareStrategies.js`
- Pre-execution idle block: `.cs-empty` pattern (no hardcoded status)
- Comparison results: dynamic from `comparison_summary` API fields
- Loading skeleton; error display unified

### `GlobalRanking.js`
- Loading skeleton (6 shimmer rows)
- `.cs-empty` empty state
- Error display unified

### `RunGA.js`
- Signal filter controls surfaced pre-execution
- Loading skeleton; error display unified

### `ComparisonPanels.js`
- Confirmed clean — pure presentational, all data via props, no hardcoded values

---

## Schema References

All canonical field names and data structures are governed by:

- [`schemas/canonical/replay_response.schema.json`](../../schemas/canonical/replay_response.schema.json) — replay blocks, execution traces, divergence statements, `certification_state`
- [`schemas/canonical/observatory_state.schema.json`](../../schemas/canonical/observatory_state.schema.json) — `system_phase`, `kernel_state.*`, `governor_state.*`, `snapshot_sequence_id`
- [`schemas/canonical/governor_telemetry.schema.json`](../../schemas/canonical/governor_telemetry.schema.json) — governor telemetry

No frontend component may synthesize data that these schemas define as backend-owned.

---

## Registered Artifacts (Sunset Conditions)

| ID | Description | Sunset Condition |
|----|-------------|-----------------|
| ARTIFACT-009 | `normalizeNarrativeBlock` — snake_case → camelCase bridge; also maps canonical `decision_trace.schema.json` fields (`block_type`, `divergence_score`, `timestamp_ns`) | Backend emits camelCase `narrative_blocks[]` natively |
| ARTIFACT-010 | `compareNarrativeBlocks` — frontend divergence analysis | Backend emits `divergence_analysis[]` in `CanonicalInspectResponse` |
| ARTIFACT-011 | `getExecutionSummary` — frontend execution summary from `narrative_blocks[]` | Backend emits `execution_summary` object in `CanonicalInspectResponse` |
