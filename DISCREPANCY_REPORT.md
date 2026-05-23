# ChronoSentiment — Documentation vs. UI Implementation Discrepancy Report

**Generated:** 2026-05-23  
**Scope:** All `.md` documentation files across the project + all React UI components in `my-chrono-sentiment-ui/src/`  
**Methodology:** Full read of every doc and every component, cross-referenced against each other.

---

## Executive Summary

The ChronoSentiment React UI (`my-chrono-sentiment-ui`) has diverged significantly from the architecture and UX direction described in the documentation. The divergences fall into five categories:

1. **API endpoint mismatches** — components call hardcoded, non-standard URLs that don't match any documented API spec.
2. **Navigation / surface architecture mismatch** — the UI uses 4 tabs that don't align with the 4 surfaces defined in `docs/frontend_cleanup_strategy.md`.
3. **Component prop/data contract divergence** — every component's props and data model differ from what the specs describe.
4. **Backend identity mismatch** — `app.py` is a Streamlit analytics dashboard, not the Flask REST API the React UI expects.
5. **Design system / theme divergence** — the CSS uses a light theme; docs call for a dark, infrastructure-grade aesthetic.

---

## 1. Navigation Architecture

### Documented (docs/frontend_cleanup_strategy.md §13)

The frontend should converge into **5 top-level navigation items**:

```
1. Observatory
2. Replay
3. Trades
4. Research
5. Settings
```

Everything else becomes secondary navigation, contextual panels, inspectors, or overlays.

### Actual Implementation (my-chrono-sentiment-ui/src/App.js:7-12)

```js
const TABS = [
  { id: 'run-ga',             label: 'Run GA' },
  { id: 'inspect-strategy',   label: 'Inspect Strategy' },
  { id: 'compare-strategies', label: 'Compare Strategies' },
  { id: 'global-ranking',     label: 'Global Ranking' },
];
```

### Discrepancies

| # | Doc Requirement | Actual |
|---|----------------|--------|
| 1.1 | Surface 1: **Observatory** (global system observability, sync ratio, chronology integrity, replay safety) | **Absent** — no Observatory surface exists |
| 1.2 | Surface 2: **Replay Timeline** (causal reconstruction, 4-layer drill-down: Market → Strategy → Execution → Portfolio) | **Absent** — no Replay Timeline surface exists |
| 1.3 | Surface 3: **Trade Inspector** (single-trade forensic analysis with 4 tabs: Intent, Environment, Execution, Outcome) | Partially present as "Inspect Strategy" but with a completely different structure |
| 1.4 | Surface 4: **Research / Analytics** (regime comparisons, cohort studies, replay certification) | **Absent** — no Research surface exists |
| 1.5 | **Settings** tab | **Absent** |
| 1.6 | "Run GA" tab | **Not in any doc** — GA execution is a backend concern; docs say UI should be an observability instrument, not a GA launcher |
| 1.7 | "Compare Strategies" tab | Not defined as a primary surface in any doc |
| 1.8 | "Global Ranking" tab | Not defined as a primary surface in any doc |

---

## 2. API Endpoint Mismatches

### Documented (docs/platform/api.md)

The API spec stub mentions: `/orders`, `/sessions`, `/ga`, `/analytics`. No detailed endpoint contracts are defined in the docs.

### Actual Backend (app.py)

`app.py` is a **Streamlit** application — it is not a REST API server at all. It renders a Streamlit dashboard with tabs for diagnostics, live logs, and paper trading. It has no HTTP endpoints that the React UI could call.

### Actual React UI Calls

| Component | Endpoint Called | Method | Notes |
|-----------|----------------|--------|-------|
| `RunGA.js:166` | `http://localhost:8000/run_ga` | GET | Hardcoded absolute URL |
| `RunGA.js:170` | `http://localhost:8000/signals/latest` | GET | Hardcoded absolute URL |
| `RunGA.js:146` | `http://localhost:8000/ga/strategy-store` | GET | Hardcoded absolute URL |
| `GlobalRanking.js:41` | `http://localhost:8000/ga/global-ranking` | GET | Hardcoded absolute URL |
| `CompareStrategies.js:51` | `http://localhost:8000/compare_strategies` | POST | Hardcoded absolute URL |
| `StrategyInspector.js:153` | `http://localhost:8000/inspect_strategy` | POST | Hardcoded absolute URL |

### Discrepancies

| # | Issue |
|---|-------|
| 2.1 | All API URLs are **hardcoded** to `http://localhost:8000` — no environment variable, no config, no relative paths |
| 2.2 | `app.py` is a Streamlit app; it **cannot serve** any of these REST endpoints |
| 2.3 | `/run_ga` is called with GET but sends no parameters — `populationSize`, `generations`, `mutationRate`, `seed` are collected in the form but **never sent** to the API |
| 2.4 | `/compare_strategies` is called with POST; docs (platform/api.md stub) suggest GET-style query params |
| 2.5 | `/ga/global-ranking` endpoint is not documented anywhere |
| 2.6 | `/inspect_strategy` endpoint is not documented anywhere |
| 2.7 | `/signals/latest` endpoint is not documented anywhere |
| 2.8 | `/ga/strategy-store` endpoint is not documented anywhere |

---

## 3. RunGA Component

### Documented Behavior

- `docs/MVP_Scope_Document_v2_1.md §4.4`: GA optimization must operate **outside** the simulation boundary; each simulation run must be independent and deterministic.
- `docs/SDS_v2_0.md §15.3`: Core orchestrators (`run_ga_orchestration`) must exist in the Core crate; API is a thin adapter.
- `docs/frontend_cleanup_strategy.md §18`: Before adding any frontend feature, ask: "Does this improve causal clarity, observability, replay understanding, or execution explainability?" A GA launcher does not.

### Actual Implementation (my-chrono-sentiment-ui/src/components/RunGA.js)

- Collects `populationSize`, `generations`, `mutationRate`, `seed` in form fields.
- Calls `GET /run_ga` — **none of the form parameters are sent** in the request body or query string.
- After GA completes, fetches `/signals/latest` and renders a "Derived execution topology" signals table.
- Fetches `/ga/strategy-store` on mount and after each run.
- Shows `execution_fitness`, `ga_fitness`, divergence badges (`Overfit`, `Hidden Gem`, `Aligned`).
- Shows signal topology with `asset`, `action`, `entry_zone`, `target`, `stop_loss`, `confidence`.

### Discrepancies

| # | Doc Requirement | Actual |
|---|----------------|--------|
| 3.1 | GA parameters (`populationSize`, `generations`, `mutationRate`, `seed`) should be sent to the backend | Parameters are **collected but never sent** — `GET /run_ga` has no body or query params |
| 3.2 | UI should be an observability instrument, not a GA launcher | The entire tab is a GA launcher — contradicts `frontend_cleanup_strategy.md` |
| 3.3 | Signals (`/signals/latest`) are not part of any documented API | Endpoint is undocumented; signal schema (`asset`, `action`, `entry_zone`, `target`, `stop_loss`, `composite_score`, `confidence`, `scenario_pnl`) is not in any spec |
| 3.4 | `ga/strategy-store` endpoint not documented | Endpoint and its response schema are entirely undocumented |
| 3.5 | `divergenceBadge` logic (comparing `execution_fitness` vs normalized `ga_fitness`) is not in any spec | Business logic for "Overfit / Hidden Gem / Aligned" classification is undocumented |
| 3.6 | `buildAssetRollups` and `topSignalsPerAsset` utility functions | Not specified in any doc |

---

## 4. GlobalRanking Component

### Documented Behavior

`docs/frontend_cleanup_strategy.md` describes a Research surface with cohort studies and regime comparisons. No specific "Global Ranking" table spec exists in any document.

### Actual Implementation (my-chrono-sentiment-ui/src/components/GlobalRanking.js)

- Fetches `GET http://localhost:8000/ga/global-ranking`.
- Displays columns: `#`, `Strategy`, `Avg PnL`, `Std Dev`, `Exec Fitness`, `GA Fitness`, `Classification`.
- Sorts by `execution_fitness` descending.
- Shows classification badges: `stable` → green, `volatile` → amber, `fragile` → red.
- No filter controls (no cohort, limit, or sortBy inputs).

### Discrepancies

| # | Doc Requirement | Actual |
|---|----------------|--------|
| 4.1 | No documented spec for a Global Ranking surface | Entire component is undocumented |
| 4.2 | Columns shown: `avg`, `std`, `execution_fitness`, `ga_fitness`, `classification` | No column spec exists in any doc |
| 4.3 | Classification system (`stable`, `volatile`, `fragile`) | Not defined in any doc |
| 4.4 | `resolveExecutionFitness` fallback chain (`execution_fitness` → `fitness` → `score` → `final_fitness`) | Indicates backend response schema is unstable/inconsistent; not documented |
| 4.5 | No filter controls | If this maps to a Research surface, cohort/regime filters would be expected |

---

## 5. CompareStrategies Component

### Documented Behavior

`docs/frontend_cleanup_strategy.md §7` describes a Trade Inspector surface for single-trade forensic analysis. No "Compare Strategies" surface is defined. The closest concept is the Replay Timeline's drill-down capability.

### Actual Implementation (my-chrono-sentiment-ui/src/components/CompareStrategies.js)

- Accepts comma-separated strategy IDs (supports 2+).
- Has a `seed` parameter (default 42).
- Calls `POST http://localhost:8000/compare_strategies` with `{ strategies: [...], scenarios: [], seed }`.
- Uses `parseStrategyParamsFromId` from `../utils/strategyId` to parse strategy config from ID string.
- Shows a Ranking table with `Exec Fitness` and `GA Fitness`.
- Shows a "Structural comparison" panel with hardcoded mock data (`queue_depth=12`, `fill_latency=42ms`, `sync_ratio=0.91`).
- Shows "Replay Cert: VALID", "Replay Integrity: CERTIFIED", "Timestamp Cohesion: VALID", "Synchronization State: DEGRADED", "Governor Action: THROTTLED" — all **hardcoded strings**.
- Has `setSelectedStrategyForInspection` prop for cross-tab navigation.

### Discrepancies

| # | Doc Requirement | Actual |
|---|----------------|--------|
| 5.1 | No "Compare Strategies" surface defined in any doc | Entire surface is undocumented |
| 5.2 | POST to `/compare_strategies` | No documented endpoint; method and payload schema undocumented |
| 5.3 | `parseStrategyParamsFromId` utility | Not documented; implies strategy IDs encode config parameters — not in any spec |
| 5.4 | `scenarios: []` in payload | Scenarios concept not explained in UI docs |
| 5.5 | Structural comparison panel shows **hardcoded mock data** | `queue_depth=12`, `fill_latency=42ms`, `sync_ratio=0.91` are static strings, not real data |
| 5.6 | "Replay Cert: VALID", "CERTIFIED", "DEGRADED", "THROTTLED" are hardcoded | These should be live system state from the backend, not static UI strings |
| 5.7 | `setSelectedStrategyForInspection` prop enables cross-tab navigation | Cross-tab navigation pattern not defined in any doc |
| 5.8 | `seed` parameter in comparison | Not in any documented comparison API |

---

## 6. StrategyInspector Component

### Documented Behavior

`docs/frontend_cleanup_strategy.md §7` defines the Trade Inspector with 4 tabs:
- **Tab 1 — Intent**: what strategy observed, why order formed, confidence context
- **Tab 2 — Environment**: synchronization state, temporal fragmentation, liquidity context
- **Tab 3 — Execution**: latency, queue, fills, slippage, divergence
- **Tab 4 — Outcome**: realized result, capture efficiency, suppression reasons, causal chain

`docs/MVP_Scope_Document_v2_1.md §3.1.6` defines the Trade Inspector as presenting each trade across 3 layers: Decision, Execution, Outcome.

### Actual Implementation (my-chrono-sentiment-ui/src/components/StrategyInspector.js)

- Accepts `strategyId`, `seed`, `strategyId2`, `seed2`, `onReset` props.
- Calls `POST http://localhost:8000/inspect_strategy` with `{ strategy_id, seed }`.
- Supports **dual-mode** (two strategies side by side) — not in any spec.
- Has a **replay position slider** (sequence ID range scrubber) — partially aligns with Replay spec.
- Renders `StrategyColumn` components with execution narrative stream.
- Renders `ComparisonPanels` when in dual mode.
- Computes `finalVerdict`, `confidenceLevel`, `confidenceColorClass` locally in the component.
- Has `showRawEvents` toggle for raw JSON event dump.
- Has `selectedSeqId` state for causal chain highlighting.

### Discrepancies

| # | Doc Requirement | Actual |
|---|----------------|--------|
| 6.1 | 4-tab structure (Intent, Environment, Execution, Outcome) | **No tabs** — single scrollable view with execution narrative stream |
| 6.2 | Tab 2 — Environment: synchronization state, temporal fragmentation, liquidity context | **Absent** — no environment/synchronization data shown |
| 6.3 | Tab 1 — Intent: what strategy observed, why order formed | Partially present via `OrderIntent` narrative block, but no confidence context |
| 6.4 | Tab 4 — Outcome: capture efficiency, suppression reasons | **Absent** |
| 6.5 | Single-trade forensic analysis | Actual component inspects an entire execution trace (all events for a strategy run), not a single trade |
| 6.6 | Dual-mode comparison (two strategies side by side) | Not in any spec; this is a novel feature |
| 6.7 | `POST /inspect_strategy` endpoint | Not documented anywhere |
| 6.8 | Verdict/confidence logic computed client-side | Business logic for execution quality assessment should be backend-computed per architecture docs |
| 6.9 | `showRawEvents` raw JSON dump | Not in any spec; contradicts `frontend_cleanup_strategy.md §9` which says to avoid raw technical vocabulary everywhere |
| 6.10 | Replay slider scrubs by `sequence_id` range | Partially aligns with Replay spec but is a simplified version |

---

## 7. StrategyColumn Component

### Documented Behavior

No direct spec for `StrategyColumn` exists. The closest is the Trade Inspector spec in `docs/frontend_cleanup_strategy.md §7`.

### Actual Implementation (my-chrono-sentiment-ui/src/components/StrategyColumn.js)

Props: `strategyNum`, `strategyId`, `seed`, `inspectionResult`, `narratedExecutionTrace`, `rawEventRefs`, `activeChain`, `eventMap`, `showRawEvents`, `getGroupColorClass`, `divergenceStatements`, `setSelectedSeqId`, `selectedSeqId`.

Renders:
- Strategy context header (ID, seed, total trades)
- Execution Narrative stream (list of `NarrativeBlock` components)
- Causal Chain panel (when a sequence ID is selected)
- Raw Execution Trace (JSON dump, when `showRawEvents` is true)

### Discrepancies

| # | Doc Requirement | Actual |
|---|----------------|--------|
| 7.1 | No spec for this component | Entire component is undocumented |
| 7.2 | `docs/frontend_cleanup_strategy.md §9`: avoid raw technical vocabulary everywhere | Raw JSON event dump (`showRawEvents`) directly contradicts this |
| 7.3 | `docs/frontend_cleanup_strategy.md §9`: avoid metric walls | 13 props passed to this component; high complexity |
| 7.4 | Causal chain panel shows `block.group · Seq {block.id}` and `block.narrative` | Causal chain visualization is partially aligned with the event causality principle but not formally specified |
| 7.5 | `divergenceStatements` prop enables cross-strategy divergence highlighting | Not in any spec |

---

## 8. ComparisonPanels Component

### Documented Behavior

No direct spec. The closest is `docs/frontend_cleanup_strategy.md §7` Trade Inspector and the general principle of execution explainability.

### Actual Implementation (my-chrono-sentiment-ui/src/components/ComparisonPanels.js)

Props: `isDualMode`, `allNarrativeBlocks1`, `allNarrativeBlocks2`, `strategyId`, `strategyId2`, `executionSummary1`, `executionSummary2`, `finalVerdict`, `confidenceLevel`, `confidenceColorClass`, `confidenceReason`, `divergenceStatements`.

Renders:
- Execution Summary Comparison (steps, partial fills, queue progression, full fills per strategy)
- Execution Insights (auto-generated text comparing the two strategies)
- Final Execution Verdict (alert with confidence level)
- Execution Divergence Analysis (list of divergence statements)

### Discrepancies

| # | Doc Requirement | Actual |
|---|----------------|--------|
| 8.1 | No spec for this component | Entire component is undocumented |
| 8.2 | Verdict/confidence computed in `StrategyInspector.js` and passed as props | Business logic for execution quality assessment is client-side; docs say backend is the single source of truth |
| 8.3 | `executionSummary` derived from narrative blocks in `StrategyInspector.js` | Derived metrics (steps, partial fills, queue progression) are computed client-side from event narrative, not from backend analytics |
| 8.4 | Divergence analysis compares narrative blocks step-by-step | This is a novel client-side algorithm not specified anywhere |

---

## 9. NarrativeBlock Component

### Documented Behavior

`docs/MVP_Scope_Document_v2_1.md §2.3` requires that every outcome be traceable and that users can inspect any trade and understand the sequence of events. `docs/Event_Flow_Specification.md §13` says the UI is an observer of the event stream.

### Actual Implementation (my-chrono-sentiment-ui/src/components/NarrativeBlock.js)

Props: `block`, `blockIndex`, `narratedExecutionTraceLength`, `activeChain`, `getGroupColorClass`, `isBlockDivergent`, `blockDivergenceMessage`, `setSelectedSeqId`.

The `block` object has: `id` (sequence_id), `group` (Intent/Queue Entry/Queue Progression/Execution/Other), `narrative` (human-readable string), `parentId`, `isKeyEvent`, `keyEventMarker`.

Renders:
- Group label + key event marker + sequence ID
- Narrative text
- Parent derivation arrow (`↳ derived from seq:{parentId}`)
- Divergence marker (`⚑ {message}`)
- Causal arrow between blocks (`↓`)

### Discrepancies

| # | Doc Requirement | Actual |
|---|----------------|--------|
| 9.1 | No spec for this component | Entire component is undocumented |
| 9.2 | Event groups (`Intent`, `Queue Entry`, `Queue Progression`, `Execution`, `Other`) | Partially aligns with `docs/Event_Flow_Specification.md §7` event types (`OrderIntentCreated`, `OrderEnteredQueue`, `QueueProgression`, `PartialFill`, `OrderFilled`) |
| 9.3 | `parentId` causal chain | Aligns with the causal event model in `docs/SRS_v1_6.md §6.2` |
| 9.4 | Narrative strings are generated client-side in `groupAndNarrateEvents()` | Docs say explainability should come from the backend (`docs/SDS_v2_0.md §8`); the `decision_trace.reason` field should come from the engine |

---

## 10. App.js Shell

### Documented Behavior

`docs/frontend_cleanup_strategy.md §12` specifies:
- Restrained typography, strong spacing, low-noise layout
- Color philosophy: stable/healthy = muted green, degraded = amber, invalid = red, neutral = slate/blue
- Dark theme aesthetic (infrastructure telemetry, not retail trading UI)

`docs/frontend_cleanup_strategy.md §13` specifies minimal top-level navigation.

### Actual Implementation (my-chrono-sentiment-ui/src/App.js)

- Left rail navigation (220px sidebar) with 4 workspace tabs.
- "Operational Awareness Strip" showing hardcoded values: `System State: Nominal`, `Chronology Engine: Synchronized (1.00x)`, `Governor: Active`, `Cohort: NSE_ALPHA_01`.
- IST clock in header and footer.
- Cross-tab navigation: clicking a strategy in CompareStrategies navigates to StrategyInspector.

### Discrepancies

| # | Doc Requirement | Actual |
|---|----------------|--------|
| 10.1 | Dark theme (infrastructure-grade, observability tooling aesthetic) | **Light theme** — `index.css` uses `--bg: #ECEFF3`, `--card: #F7F8FA` (light gray) |
| 10.2 | Color: stable/healthy = muted green | `--grn: #4F6BFF` is actually **blue**, not green |
| 10.3 | Color: neutral telemetry = slate/blue | `--blu: #4F6BFF` — same value as `--grn`, making them **identical** |
| 10.4 | Operational Awareness Strip values are **hardcoded** | `Synchronized (1.00x)`, `Active`, `NSE_ALPHA_01` are static strings, not live data |
| 10.5 | Navigation should be minimal (5 items max) | 4 tabs, but wrong surfaces (GA, Inspect, Compare, Ranking vs Observatory, Replay, Trades, Research, Settings) |
| 10.6 | `docs/frontend_cleanup_strategy.md §9`: avoid multi-ontology screens | The Operational Awareness Strip mixes observability telemetry with navigation chrome |

---

## 11. CSS Design System

### Documented Behavior

`docs/frontend_cleanup_strategy.md §12`:
- Dark theme
- Color: stable = muted green, degraded = amber, invalid/chronology failure = red, neutral = slate/blue
- IBM Plex Sans + IBM Plex Mono (actually implemented correctly)
- No neon aesthetics, no crypto-style visual overload

### Actual Implementation (my-chrono-sentiment-ui/src/index.css)

```css
:root {
  --bg:    #ECEFF3;   /* light gray — NOT dark */
  --card:  #F7F8FA;   /* near-white */
  --grn:   #4F6BFF;   /* blue, not green */
  --blu:   #4F6BFF;   /* same as --grn */
  --amb:   #C27A5A;   /* amber-ish — OK */
  --red:   #B05A5A;   /* muted red — OK */
  --r4: 0px; --r8: 0px; --r10: 0px; --r12: 0px;  /* all border-radius = 0 */
}
```

### Discrepancies

| # | Doc Requirement | Actual |
|---|----------------|--------|
| 11.1 | Dark theme | **Light theme** (`#ECEFF3` background) |
| 11.2 | `--grn` should be muted green | `--grn: #4F6BFF` is **blue** |
| 11.3 | `--blu` and `--grn` should be distinct | Both are `#4F6BFF` — **identical values** |
| 11.4 | Rounded corners expected for card-based UI | All `--r*` variables are `0px` — **no border radius** |
| 11.5 | `app.py` uses a dark Streamlit theme (`#0b0f19` background) | React UI uses light theme — the two UIs have **opposite themes** |

---

## 12. Backend Identity Mismatch

### Documented Expectation

The React UI expects a REST API server at `http://localhost:8000` with endpoints:
- `GET /run_ga`
- `GET /signals/latest`
- `GET /ga/strategy-store`
- `GET /ga/global-ranking`
- `POST /compare_strategies`
- `POST /inspect_strategy`

### Actual Backend

`app.py` is a **Streamlit** application that:
- Runs on Streamlit's default port (8501, not 8000)
- Renders a multi-tab Streamlit dashboard
- Reads from `data/experiments.jsonl`, `analysis/` log directories
- Has no HTTP REST endpoints
- Is a completely separate product from what the React UI expects

### Discrepancies

| # | Issue |
|---|-------|
| 12.1 | `app.py` is Streamlit, not Flask/FastAPI — **no REST API exists** |
| 12.2 | Port mismatch: Streamlit defaults to 8501; React UI calls 8000 |
| 12.3 | `app.py` reads from `data/experiments.jsonl` and log files; React UI expects JSON API responses |
| 12.4 | The two applications appear to be **entirely separate products** that happen to share a repository |
| 12.5 | `app.py` has its own CSS design system (dark theme, `#0b0f19`) that contradicts the React UI's light theme |

---

## 13. Missing Documented Features (Not Yet Implemented)

The following capabilities are specified in docs but have **no corresponding UI implementation**:

| # | Documented Feature | Source Doc |
|---|-------------------|-----------|
| 13.1 | Observatory surface (sync ratio, provider fragmentation, chronology integrity, propagation dispersion, replay safety, confidence state) | `frontend_cleanup_strategy.md §5` |
| 13.2 | Replay Timeline with 4-layer drill-down (Market, Strategy, Execution, Portfolio) | `frontend_cleanup_strategy.md §6` |
| 13.3 | Trade Inspector Tab 2 — Environment (synchronization state, temporal fragmentation, liquidity context) | `frontend_cleanup_strategy.md §7` |
| 13.4 | Trade Inspector Tab 4 — Outcome (capture efficiency, suppression reasons) | `frontend_cleanup_strategy.md §7` |
| 13.5 | Research surface (regime comparisons, cohort studies, replay certification, synchronization drift, topology persistence) | `frontend_cleanup_strategy.md §8` |
| 13.6 | Settings surface | `frontend_cleanup_strategy.md §13` |
| 13.7 | WebSocket real-time event streaming to UI | `Event_Flow_Specification.md §12` |
| 13.8 | Portfolio view (portfolio events projection) | `Event_Flow_Specification.md §13.2` |
| 13.9 | Timeline component (navigate across simulation, identify key moments) | `MVP_Scope_Document_v2_1.md §3.1.7` |
| 13.10 | Replay engine UI (step-by-step navigation, 10x–50x accelerated playback) | `MVP_Scope_Document_v2_1.md §3.1.5` |
| 13.11 | Explainability from backend `decision_trace.reason` field | `SDS_v2_0.md §8` |
| 13.12 | Progressive disclosure (Overview → drill-down → causal detail) | `frontend_cleanup_strategy.md §6` |

---

## 14. Undocumented Features Present in UI (No Spec Coverage)

The following features exist in the UI but have **no documentation**:

| # | Feature | Component |
|---|---------|-----------|
| 14.1 | GA parameter form (population, generations, mutation rate, seed) | `RunGA.js` |
| 14.2 | Signals topology table (asset, action, entry_zone, target, stop_loss) | `RunGA.js` |
| 14.3 | Divergence badge system (Overfit / Hidden Gem / Aligned) | `RunGA.js` |
| 14.4 | Strategy store panel | `RunGA.js` |
| 14.5 | Global ranking table with classification badges | `GlobalRanking.js` |
| 14.6 | Multi-strategy comparison via comma-separated IDs | `CompareStrategies.js` |
| 14.7 | `parseStrategyParamsFromId` utility (strategy config encoded in ID string) | `CompareStrategies.js` |
| 14.8 | Hardcoded structural comparison mock data | `CompareStrategies.js:224-235` |
| 14.9 | Dual-mode strategy inspection (two strategies side by side) | `StrategyInspector.js` |
| 14.10 | Sequence ID replay slider | `StrategyInspector.js` |
| 14.11 | Client-side execution verdict / confidence computation | `StrategyInspector.js:246-273` |
| 14.12 | Client-side narrative generation from raw events (`groupAndNarrateEvents`) | `StrategyInspector.js:27-97` |
| 14.13 | Client-side divergence detection (`compareNarrativeBlocks`) | `StrategyInspector.js:99-114` |
| 14.14 | Causal chain highlighting (click a block to highlight its ancestors) | `StrategyColumn.js` |
| 14.15 | Raw events JSON dump toggle | `StrategyColumn.js` |
| 14.16 | IST clock in header/footer | `App.js` |
| 14.17 | Hardcoded Operational Awareness Strip | `App.js:97-117` |
| 14.18 | Cross-tab navigation (CompareStrategies → StrategyInspector) | `App.js:41-52` |

---

## 15. Architectural Principle Violations

The following implementations violate core architectural principles stated in the docs:

| # | Principle (Source) | Violation |
|---|-------------------|-----------|
| 15.1 | "UI does not compute reality — it observes reality" (`Event_Flow_Specification.md §13.3`) | Client-side narrative generation (`groupAndNarrateEvents`), verdict computation, divergence detection, and execution summary derivation all compute business logic in the UI |
| 15.2 | "Every action MUST produce a `decision_trace`" (`SDS_v2_0.md §8`) | No `decision_trace` is rendered; explainability comes from client-side narrative strings, not backend-provided traces |
| 15.3 | "UI must represent uncertainty — not eliminate it" (`Service_Boundary_Definition.md §5.3`) | Hardcoded "CERTIFIED", "VALID", "DEGRADED" strings in `CompareStrategies.js` eliminate uncertainty with false certainty |
| 15.4 | "No state change without an event" (`Service_Boundary_Definition.md §6.1`) | Client-side state (verdict, confidence, divergence) is computed from derived data, not from events |
| 15.5 | "Optimization must not violate causal isolation" (`Service_Boundary_Definition.md §8`)
| 15.5 | "Optimization must not violate causal isolation" (`Service_Boundary_Definition.md §8`) | The GA launcher UI (`RunGA.js`) directly triggers GA execution from the UI, blurring the boundary between optimization and simulation |
| 15.6 | "Reads must NEVER influence event generation" (`Service_Boundary_Definition.md §5.2`) | The UI calls `/run_ga` which presumably triggers a simulation run — the UI is causing state changes, not just observing |
| 15.7 | "No direct cross-service state access" (`Service_Boundary_Definition.md §4`) | The UI directly calls multiple backend endpoints that expose internal state (`/ga/strategy-store`, `/ga/global-ranking`) |

---

## 16. Summary Statistics

| Category | Count |
|----------|-------|
| Navigation surface mismatches | 8 |
| API endpoint mismatches / undocumented endpoints | 8 |
| Component prop/data contract divergences | 35+ |
| Missing documented features | 12 |
| Undocumented features present in UI | 18 |
| Architectural principle violations | 7 |
| Design system / theme violations | 5 |
| **Total discrepancies identified** | **93+** |

---

## 17. Priority Remediation Recommendations

### Critical (Blocks architectural integrity)

1. **Identify or build the actual REST API backend** — `app.py` is Streamlit and cannot serve the React UI. A separate FastAPI/Flask server must exist or be built to serve `localhost:8000`.
2. **Move business logic out of the UI** — `groupAndNarrateEvents`, `compareNarrativeBlocks`, verdict/confidence computation, and execution summary derivation all belong in the backend per the architecture docs.
3. **Fix GA parameter submission** — `RunGA.js` collects `populationSize`, `generations`, `mutationRate`, `seed` but never sends them to the API.
4. **Remove hardcoded mock data** — `CompareStrategies.js` shows static `queue_depth=12`, `fill_latency=42ms`, `sync_ratio=0.91` values that are not real data.

### High (Significant spec divergence)

5. **Implement the 4 documented surfaces** — Observatory, Replay Timeline, Trade Inspector (with 4 tabs), Research — replacing the current Run GA / Inspect / Compare / Ranking tabs.
6. **Fix the color system** — `--grn` and `--blu` are identical (`#4F6BFF`); `--grn` should be a muted green per the color philosophy.
7. **Implement WebSocket streaming** — the Event Flow Specification requires real-time event streaming to the UI; currently all data is fetched via polling REST calls.
8. **Replace hardcoded Operational Awareness Strip** — `System State`, `Chronology Engine`, `Governor`, `Cohort` values must come from live backend data.

### Medium (UX alignment)

9. **Align theme** — either commit to the documented dark theme or update the docs to reflect the light theme decision.
10. **Add Trade Inspector tabs** — Intent, Environment, Execution, Outcome tabs are missing; the current single-scroll view lacks the Environment and Outcome layers.
11. **Document the undocumented features** — 18 features exist in the UI with no spec coverage; either document them or remove them.
12. **Externalize API URLs** — all 6 hardcoded `http://localhost:8000` URLs should use environment variables or a config file.

### Low (Polish / consistency)

13. **Add border-radius** — all `--r*` CSS variables are `0px`; the flat design may be intentional but contradicts the card-based UI shown in the components.
14. **Align `--grn` semantic** — the variable named `--grn` (green) is used for "success/positive" states but its value is blue; this creates semantic confusion.
15. **Document `parseStrategyParamsFromId`** — the utility that parses strategy configuration from an ID string is a significant undocumented contract between UI and backend.

---

*Report generated by cross-referencing all `.md` files in `docs/`, root-level `.md` files, and all React components in `my-chrono-sentiment-ui/src/`.*