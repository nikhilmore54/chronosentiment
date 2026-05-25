# ChronoSentiment — Documentation vs. UI Implementation Discrepancy Report

**Generated:** 2026-05-23
**Scope:** All `.md` documentation files across the project + all React UI components in `my-chrono-sentiment-ui/src/`
**Methodology:** Full read of every doc and every component, cross-referenced against each other.
**Status:** Constitutional governance artifact — architectural transition document for ChronoSentiment.

---

## Constitutional Architecture Laws (Established by This Report)

These laws are not guidelines. They are correctness constraints. Any system behavior that violates them is not a style issue — it is a correctness failure.

| Law | Statement | Governs |
|-----|-----------|---------|
| **Law Zero** | The UI must never invent reality | Philosophical root constraint — replay validity, observability trust, execution explainability |
| **Law One** | Every UI element must trace to a backend-emitted field in an authoritative schema | Operational enforcement of Law Zero — converts it into a testable engineering rule |
| **Law Two** | Every replay must be reconstructible from certified chronology alone | Deterministic replay guarantee — UI is a consumer of replay output, not a participant |
| **Law Three** | No derived certainty without kernel certification | Certification authority boundary — verdicts, confidence, divergence, causal attribution |

### Authority Layer Hierarchy

The four laws map to a five-layer authority structure. Each layer has exactly one authority role. No layer may assume the authority of another.

| Layer | Authority Role | What It Owns |
|-------|---------------|--------------|
| **Kernel** | Truth authority | Event ordering, state transitions, execution outcomes |
| **Schemas** | Transmission authority | What fields are emitted, their types, their semantics |
| **Replay Engine** | Reconstruction authority | Deterministic state reconstruction from event stream |
| **UI** | Observation authority | Display of backend-certified state — nothing more |
| **Experimental Tooling** | Non-authoritative exploration | Interaction ideas, UX prototyping — not canonical |

Any component that operates outside its layer's authority role is a constitutional violation. The current React UI (`my-chrono-sentiment-ui/`) operates at the Kernel and Replay Engine layers when it synthesizes narratives, computes verdicts, and derives causal chains. That is the root cause of the 93+ discrepancies documented below.

### Law Zero — The UI Must Never Invent Reality

> **The UI must never invent reality.**

This is the foundational constraint. It governs:

- **Replay validity** — a UI that synthesizes causality cannot be used to debug a deterministic kernel, because the UI's invented state may contradict the kernel's actual state
- **Observability trust** — hardcoded telemetry makes the system appear healthy regardless of actual state
- **Execution explainability** — client-derived narratives are not certified by the kernel and cannot be replayed

### Law One — Every UI Element Must Trace to Backend-Emitted Authority

> **No UI element should exist that cannot point to a specific field in a specific backend schema.**

This is the operational enforcement mechanism for Law Zero. It converts the philosophical constraint into a testable engineering rule. If a UI element cannot name its source schema and field, it is inventing reality.

### Law Two — Every Replay Must Be Reconstructible from Certified Chronology

> **Replay must produce identical system states for identical inputs, without UI participation.**

The UI is a consumer of replay output, not a participant in replay computation. Any replay logic that depends on UI state is a violation.

### Law Three — No Derived Certainty Without Kernel Certification

> **Verdicts, confidence levels, divergence classifications, and causal attributions must originate in the kernel, not the frontend.**

Client-side computation of these values (as currently implemented in [`StrategyInspector.js:246-273`](my-chrono-sentiment-ui/src/components/StrategyInspector.js:246) and [`ComparisonPanels.js`](my-chrono-sentiment-ui/src/components/ComparisonPanels.js)) violates this law.

Violations of all four laws are documented in Section 15. They are not cosmetic issues — they are correctness failures.

---

## Executive Summary

This report documents an **ontology divergence**, not merely implementation incompleteness. The 93+ discrepancies below are evidence of an unresolved identity split between two distinct system philosophies that evolved in parallel:

| Identity A (Documented) | Identity B (Implemented) |
|------------------------|--------------------------|
| Chronology observability infrastructure | GA optimization workstation |
| Replay authority | Strategy experimentation |
| Observer UI — backend-certified truth only | Analytical UI — client-derived interpretation |
| Deterministic causality | Client-synthesized narrative |

The correct response is **architectural succession**, not incremental repair. See Section 18 for the full strategic analysis.

The technical divergences fall into five categories:

1. **API endpoint mismatches** — components call hardcoded, non-standard URLs that don't match any documented API spec.
2. **Navigation / surface architecture mismatch** — the UI uses 4 tabs that don't align with the 5 surfaces defined in `docs/frontend_cleanup_strategy.md`.
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
---

## 18. Strategic Analysis — The Identity Split

*This section interprets the discrepancies above as evidence of a deeper architectural condition, not merely implementation drift.*

### 18.1 The Core Finding

The 93+ discrepancies documented above are not primarily a list of bugs to fix. They are evidence of an **unresolved identity split** inside ChronoSentiment itself.

Two distinct system identities have been evolving in parallel:

| System Identity A | System Identity B |
|-------------------|-------------------|
| Observability / chronology infrastructure | GA optimization workstation |
| Replay authority | Strategy experimentation |
| Execution governance | Trading tooling |
| Observer UI (backend-certified truth only) | Interactive analytical UI (client-derived interpretation) |
| Deterministic causality | Client-synthesized narrative |

The documentation (`docs/`) evolved toward **Identity A**.  
The React UI (`my-chrono-sentiment-ui/`) evolved toward **Identity B**.  
`app.py` is a transitional operational tool that belongs to neither cleanly.

These are not the same product category. Attempting to reconcile them incrementally would produce patchwork architecture and permanent ontology confusion.

---

### 18.2 Why the Architectural Violations Are Not Cosmetic

The violations listed in Section 15 are structurally significant:

| Violation | Consequence |
|-----------|-------------|
| Client-side verdict computation ([`StrategyInspector.js:246-273`](my-chrono-sentiment-ui/src/components/StrategyInspector.js:246)) | Breaks authoritative replay — two clients can produce different verdicts from the same event stream |
| Client-side narrative synthesis ([`groupAndNarrateEvents`](my-chrono-sentiment-ui/src/components/StrategyInspector.js:27)) | Breaks deterministic explainability — the narrative is not certified by the kernel |
| Hardcoded telemetry ([`App.js:97-117`](my-chrono-sentiment-ui/src/App.js:97)) | Creates fake observability — the system appears healthy regardless of actual state |
| GA launcher as primary surface ([`RunGA.js`](my-chrono-sentiment-ui/src/components/RunGA.js)) | Re-centers optimization as the product identity, contradicting the observability-first philosophy |
| Mock synchronization states ([`CompareStrategies.js:202-215`](my-chrono-sentiment-ui/src/components/CompareStrategies.js:202)) | Violates chronology truth — "CERTIFIED" and "DEGRADED" are not real system state |
| Dual-mode comparison logic ([`compareNarrativeBlocks`](my-chrono-sentiment-ui/src/components/StrategyInspector.js:99)) | Introduces undocumented interpretation machinery that the backend cannot validate or replay |

Each of these directly violates [`Service_Boundary_Definition.md`](docs/Service_Boundary_Definition.md), [`Event_Flow_Specification.md`](docs/Event_Flow_Specification.md), and the chronology authority principles in [`Backend_Architecture_Blueprint.md`](docs/Backend_Architecture_Blueprint.md).

---

### 18.3 The React UI Is Exploratory Prototype Lineage

The React UI was almost certainly built as:

- exploratory scaffolding to test interaction ideas,
- experimentation tooling to validate GA and inspection workflows,
- cognitive prototyping to discover what the product should feel like.

That is normal and valuable. The mistake would be treating it as canonical infrastructure and attempting 93 incremental fixes. That path leads to:

- patchwork architecture with no coherent identity,
- permanent ontology confusion between the two system identities,
- endless drift as new features are added to the wrong foundation.

---

### 18.4 Architectural Succession Declaration

The correct response is to declare **architectural succession**, not incremental repair:

| Artifact | Reclassification |
|----------|-----------------|
| `my-chrono-sentiment-ui/` | **Exploratory prototype lineage** — useful for interaction ideas, replay UX experiments, and forensic UI exploration. Not canonical infrastructure. Freeze, do not extend. |
| `services/ui` (to be built) | **Canonical observability frontend** — built against authoritative backend state only, no client-derived truth, replay-certified data, observability-first hierarchy. |
| `app.py` | **Transitional operational tooling** — useful for live diagnostics and paper trading monitoring. Not the product UI. Maintain separately. |
| `docs/` (Observatory, Replay, Trades, Research, Settings) | **Canonical product ontology** — the documented surfaces define the correct identity. |

---

### 18.5 The Canonical Surface Architecture (From Docs)

The documentation already defines the correct future. These five surfaces are coherent and should be built as `services/ui`:

| Surface | Purpose | Key Principle |
|---------|---------|---------------|
| **Observatory** | Chronology integrity, sync ratio, provider fragmentation, replay safety | Global system observability — "state of reality" layer |
| **Replay** | Deterministic reconstruction, 4-layer drill-down (Market → Strategy → Execution → Portfolio) | Causal reconstruction engine — the signature differentiator |
| **Trades** | Single-trade forensic analysis (Intent → Environment → Execution → Outcome tabs) | Why did this outcome occur? |
| **Research** | Regime comparisons, cohort studies, replay certification, topology persistence | Long-horizon behavioral analysis |
| **Settings** | Governance, configuration | System configuration |

---

### 18.6 The Single Most Important Principle Going Forward

> **The UI must never invent reality.**

This means:

- No hardcoded telemetry values
- No client-derived certainty (verdicts, confidence levels, divergence classifications)
- No narrative fabrication (client-side `groupAndNarrateEvents`)
- No client-side causal authority

The frontend must **observe, replay, and explain backend-certified chronology only**.

This is not a UX constraint. It is a correctness constraint. A UI that invents reality cannot be used to debug a deterministic system — because the UI's invented state may contradict the kernel's actual state, making the system appear to explain itself when it is actually explaining a client-side approximation.

---

### 18.7 The API Mismatch as a Diagnostic Signal

The fact that the React UI evolved against imagined APIs (`http://localhost:8000` endpoints that don't exist) is a diagnostic signal, not just a bug:

> UX exploration outpaced system stabilization.

This is the natural consequence of building a frontend before the backend API contract is stable. The solution is not to build the backend to match the frontend's assumptions — those assumptions encode the wrong identity (Identity B). The solution is to build the backend API to match the documented architecture (Identity A) and build the new frontend against that.

---

### 18.8 The Theme Mismatch as a Symbolic Indicator

The CSS theme divergence is not trivial:

| `app.py` Streamlit theme | React UI theme |
|--------------------------|----------------|
| Dark (`#0b0f19`) — infrastructure telemetry aesthetic | Light (`#ECEFF3`) — generic analytics workspace aesthetic |
| Observatory identity | Strategy workstation identity |

The two applications literally look like different products because they *are* different products. The theme is a visible symptom of the identity split.

The canonical `services/ui` should adopt the dark infrastructure-telemetry aesthetic described in [`docs/frontend_cleanup_strategy.md §12`](docs/frontend_cleanup_strategy.md) — not because dark themes are better, but because the visual language must communicate "this is a deterministic execution observatory" rather than "this is a trading analytics dashboard."

---

### 18.9 What ChronoSentiment Already Knows It Wants to Become

The documentation is remarkably coherent about the target identity. The system already knows what it wants to become through:

- The replay architecture in [`Event_Flow_Specification.md`](docs/Event_Flow_Specification.md)
- The observability philosophy in [`frontend_cleanup_strategy.md`](docs/frontend_cleanup_strategy.md)
- The chronology governance model in [`Backend_Architecture_Blueprint.md`](docs/Backend_Architecture_Blueprint.md)
- The service boundary laws in [`Service_Boundary_Definition.md`](docs/Service_Boundary_Definition.md)

The React UI simply has not caught up yet. This discrepancy report is the evidence that the transition from **exploratory quant tooling** to **institutional chronology observability infrastructure** is underway — and that the old UI belongs to the former, while the documented architecture belongs to the latter.

---

*End of Discrepancy Report.*
---

## 19. The Observer vs. Analyst Distinction — A Foundational Governance Principle

This distinction is the deepest architectural boundary established by this report. It must govern all future frontend decisions.

| Observer UI | Analytical UI |
|-------------|---------------|
| Shows certified state | Computes interpretation |
| Replay-authoritative | Interpretation-authoritative |
| Kernel-derived | Client-derived |
| Chronology truth | Analytical convenience |
| Passive instrument | Active synthesizer |

ChronoSentiment must remain **observer-first**. The moment the UI becomes an analyst, it becomes a second simulation layer — one that is neither deterministic nor certifiable.

The current React UI is an analytical UI. The canonical `services/ui` must be an observer UI.

The test for any proposed frontend feature is:

> Does this feature display backend-certified state, or does it compute its own interpretation of that state?

If it computes interpretation: it belongs in the backend, not the frontend.

---

## 20. The Next Architectural Layer — Before Frontend Implementation Begins

The feedback from this report is clear: the next step is **not** to begin building `services/ui` immediately. The next step is to define the authoritative backend schemas that the UI will observe.

The UI can only become truthful if the backend emits authoritative truth structures. Schema definition precedes frontend implementation.

### Required Schemas (in priority order)

| Schema | Purpose | Governs |
|--------|---------|---------|
| **Authoritative event schema** | Canonical event envelope with `sequence_id`, `timestamp`, `type`, `payload`, `parent_sequence_id` | All event stream consumers |
| **Replay response schema** | Certified replay session structure with state hash | Replay surface |
| **Observatory state schema** | Live telemetry: sync ratio, provider fragmentation, chronology integrity, propagation dispersion, replay safety, confidence state | Observatory surface |
| **Governor telemetry schema** | Kernel governor state, throttle state, activation conditions | Observatory + Settings surfaces |
| **`decision_trace` schema** | Per-trade explainability: signals observed, conditions evaluated, reason string | Trades surface |

Until these schemas are defined and the backend emits them, any frontend built against them will repeat the same mistake as the current React UI — evolving against imagined APIs.

### The Correct Build Sequence

```
1. Define authoritative backend schemas (above)
2. Implement backend endpoints that emit those schemas
3. Build services/ui as a pure observer of those schemas
4. Validate: every UI element must trace to a backend-emitted field
```

No UI element should exist that cannot point to a specific field in a specific backend schema. If it cannot, it is inventing reality.

---

## 21. Conclusion — ChronoSentiment as a Governed Infrastructure System

ChronoSentiment is no longer a collection of experiments. It is beginning to behave like a governed infrastructure system. Governed systems require:

- **Architectural succession** — clear lineage decisions about what is canonical and what is exploratory
- **Ontology discipline** — a single, consistent vocabulary for what the system is and does
- **Authoritative boundaries** — explicit rules about who can compute what, and where

This report successfully establishes those boundaries.

The transition from exploratory quant tooling to institutional chronology observability infrastructure is now documented, justified, and governed.

### The Succession Table (Final)

| Artifact | Status | Action |
|----------|--------|--------|
| `my-chrono-sentiment-ui/` | Exploratory prototype lineage | Freeze. Preserve as historical continuity. Do not extend. |
| `services/ui` | Canonical observability frontend | Build from scratch against authoritative schemas. Observer-only. |
| `app.py` | Transitional operational tooling | Maintain separately. Not the product UI. |
| `docs/` | Canonical product ontology | Authoritative. All implementation must trace to docs. |

### The Foundational Principle (Repeated for Governance)

> **The UI must never invent reality.**

This is the single most important constraint established by this report. It is not a UX guideline. It is a correctness constraint that governs replay validity, observability trust, and execution explainability.

Every future frontend decision must be evaluated against it.

---

*End of ChronoSentiment Discrepancy Report*  
*18 Technical Sections · 93+ Discrepancies · 3 Governance Sections · 1 Foundational Principle*
---

## 22. The Concept Extraction Pathway — What the Prototype Lineage Contributes

The architectural succession declared in Section 18 does not invalidate the current React UI. It reclassifies it. That distinction is important.

The current `my-chrono-sentiment-ui/` clearly produced valuable discoveries through exploratory development. These should be extracted, reinterpreted, and rebuilt inside the observer-first architecture — not discarded, and not incrementally patched into canonical infrastructure.

### Valuable Concepts Discovered in the Prototype Lineage

| Concept | Where It Appears | How to Reinterpret for `services/ui` |
|---------|-----------------|--------------------------------------|
| Replay position slider (sequence ID scrubber) | [`StrategyInspector.js:286-299`](my-chrono-sentiment-ui/src/components/StrategyInspector.js:286) | Rebuild as a backend-driven replay cursor — the slider requests a certified state snapshot at a given `sequence_id` from the Replay Engine; the UI does not filter events locally |
| Causal chain highlighting (click a block → highlight ancestors) | [`StrategyColumn.js:84-108`](my-chrono-sentiment-ui/src/components/StrategyColumn.js:84) | Rebuild as a backend-resolved causal query — the UI sends a `sequence_id` to the backend; the backend returns the certified causal chain; the UI renders it |
| Execution narrative stream (Intent → Queue → Execution grouping) | [`StrategyInspector.js:27-97`](my-chrono-sentiment-ui/src/components/StrategyInspector.js:27) | Rebuild as a backend-emitted narrative — the `decision_trace` schema (Section 20) should include pre-computed narrative groups; the UI renders them without synthesis |
| Divergence visualization (two strategies side by side) | [`ComparisonPanels.js`](my-chrono-sentiment-ui/src/components/ComparisonPanels.js) | Rebuild as a backend-certified comparison — the backend computes divergence between two certified replay sessions; the UI renders the diff |
| Divergence badge system (Overfit / Hidden Gem / Aligned) | [`RunGA.js:23-32`](my-chrono-sentiment-ui/src/components/RunGA.js:23) | Rebuild as a backend-classified field in the strategy schema — the kernel certifies the classification; the UI renders the badge |
| Operational awareness strip (system state, governor, cohort) | [`App.js:97-117`](my-chrono-sentiment-ui/src/App.js:97) | Rebuild as a live Observatory schema subscription — all values come from the governor telemetry schema (Section 20); nothing is hardcoded |

### The Transformation Pathway

| Old Mental Model | New Mental Model |
|-----------------|-----------------|
| Fix the existing UI | Extract concepts, rebuild architecture |
| Frontend feature parity | Observer-certified surfaces |
| React app with backend APIs | Chronology observability frontend |
| UI-driven workflows | Schema-driven observability |
| Client-side truth synthesis | Backend-certified truth display |

The prototype lineage answered the question: *what does the user need to see?*

The canonical `services/ui` must answer a different question: *what has the kernel certified as true, and how do we display it faithfully?*

Both questions are necessary. The prototype answered the first. The constitutional architecture now governs the second.

---

*End of ChronoSentiment Constitutional Discrepancy Report*  
*21 Technical + Governance Sections · 22 Total Sections · 93+ Discrepancies · 4 Constitutional Laws · 5-Layer Authority Hierarchy · 1 Foundational Principle*