# ChronoSentiment — Post-QFTH Development Roadmap

> Branch: `post-qfth/cert-restore-and-gulf-validation`
> Branched from: `master` @ `caad7eeb` (qfth-submission commit)
> Do not merge to master until QFTH submission is complete and accepted.

---

## Architectural spine (do not compromise)

```
chronology truth first
inference second
```

Every feature decision should be evaluated against three questions:
1. Does this strengthen chronology integrity?
2. Does this strengthen replay authority or forensic explainability?
3. Does this strengthen execution governance?

If the answer to all three is no, defer it.

---

## Phase 1 — Restore certification integrity (weeks 1–2)

**Priority: highest. Do this before any new capability work.**

The backend cert loop is broken in two places. Without these fixes, the replay chain certification is empirically valid but cannot be audited programmatically — which breaks any technical due-diligence demo.

### 1a. Restore cross-runtime parity harness

**File:** `scripts/compare_ingest_parity.py`
**Problem:** After promotion of `cs-ingest` to canonical ingest, the `--python-ingest` flag was removed from `run_nse_cohort.py`. The parity harness now compares cs-ingest against itself (self-compare bug).
**Fix options:**
- Option A: Restore `scripts/legacy_python_ingest.py` as a thin wrapper that runs the old Python ingest path, and pass it via `--python-ingest`
- Option B: Switch to golden-archive regression — freeze a certified output archive from the last known-good Python run, and compare cs-ingest output against that frozen golden archive on every run

Option B is more durable. Option A is faster to restore.

### 1b. Restore `live_session_steps.jsonl` output

**File:** `scripts/run_live_session.py`
**Problem:** The new kernel runner (Phase A/B/C) does not write `live_session_steps.jsonl`. This breaks `post-soak-cert` and the full `certify all` equivalence path.
**Fix:** Append one JSONL line per cycle to `state_archive/metadata/live_session_steps.jsonl`:
```json
{"ts": "<barrier_timestamp>", "barrier_committed": true, "latency_ms": <optional>}
```

### 1c. Restore participation/lag logging

**File:** `scripts/run_live_session.py`
**Problem:** Participation rate, provider lag, and `sync_breakdown` were removed in the kernel runner rewrite. These are needed to diagnose Yahoo fragmentation on batch_902.
**Fix:** Thin metadata logging — log sync ratio, dispersion, and per-provider lag to a per-session JSONL file alongside the existing run output.

### 1d. Verify dedupe efficiency

**Confirm:** Each live cycle should log `persisted N | dedupe_skip M` where M > 0 after the first cycle. If M is always 0, the dedupe is not working and CPU cost is unbounded.

---

## Phase 2 — Gulf exchange validation (weeks 2–4)

**Requires: QFTH network access to ADX, DFM, or Tadawul data feeds.**

This is the single highest-value technical milestone. The batch_003/batch_910 finding is real and documented, but it is one data point on one market (NSE). Replicating it on a Gulf exchange moves the project from "interesting NSE phenomenon" to "generalizable execution-governance framework."

### Steps

1. Obtain Gulf exchange feed access (ADX preferred — Abu Dhabi Securities Exchange)
2. Define two cohorts:
   - Broad market cohort (equivalent to batch_003): 100–500 symbols, mixed liquidity
   - Control cohort (equivalent to batch_910): 10–20 high-liquidity symbols (banking/energy)
3. Instrument feeds through the existing ingest pipeline (`cs-ingest replay-step`)
4. Run the same synchronisation measurement: sync ratio, dispersion, provider lag
5. Document findings in `RESEARCH_LOG.md` using the same session-phase structure (open, midday, pre-close, close, post-close)
6. Compare: is the fragmentation pattern exchange-specific or structural?

### Expected outcome

If the broad-market Gulf cohort shows similar fragmentation (sync <70%, dispersion >2.0 at open), the finding is structural and the institutional pitch becomes: "this is a market microstructure property, not an NSE artifact." That is a much stronger claim.

---

## Phase 3 — Canonical frontend build in `services/ui/` (weeks 3–6)

**Per `docs/frontend_cleanup_strategy.md` Phase 1.**

The React app at `services/ui/` is the canonical frontend but is currently behind `observatory/suppression.html` in institutional legibility. Phase 1 work only — no new charts.

### 3a. 4-surface navigation in `services/ui/src/App.tsx`

Implement top-level routing for:
- `/observatory` — Observatory surface
- `/replay` — Replay Timeline surface
- `/trades` — Trade Inspector surface
- `/research` — Research Console surface

### 3b. Observatory surface wired to live governor state

Wire the Observatory surface to the backend API governor state endpoint. The surface should show: sync ratio, dispersion, provider lag, affected symbols, governor state (HALTED/THROTTLED/NOMINAL), cohort name, session phase.

### 3c. Suppression banner as first-class React component

Implement the suppression banner from `observatory/suppression.html` as a React component in `services/ui/`. This is the institutional screen — it should be the first thing visible when governor state is HALTED.

### Constraints

- No new charts
- No new data sources
- Focus: hierarchy, navigation, metric dedupe, color semantics, terminology consistency
- Color system: green (NOMINAL), amber (THROTTLED), red (HALTED), slate/blue (neutral telemetry)

---

## Phase 4 — Replay Timeline surface (weeks 6–10)

**This is the signature differentiator. The "flight recorder for automated financial execution."**

The Replay Timeline answers: "Why exactly did this happen?" across chronology, synchronisation, execution, and replay. Most systems can tell you what they did. Very few can prove why they did it with deterministic reconstruction.

### Build order

1. **Scrubber UI** — timeline scrubber over historical ingest cycles, replay-safe boundary markers shown as hard stops
2. **Synchronisation state overlay** — per-timestamp sync ratio, dispersion, governor state overlaid on the timeline
3. **Causal chain drill-down** — click any suppression event → expand the 6-step causal chain (feed received → sync measured → dispersion computed → threshold breach → HALT issued → execution suppressed)
4. **Replay reconstruction** — integrate with `services/api/src/replay.rs` for deterministic reconstruction of market view at any certified timestamp

### Progressive disclosure rule

```
Overview (sync ratio timeline)
→ drill-down (per-timestamp state)
→ causal detail (6-step chain)
→ deeper telemetry (provider-level breakdown)
```

Do not show all layers simultaneously.

---

## Phase 5 — Institutional design partner (ongoing)

**Target: one design partner agreement within 3 months of QFTH accelerator entry.**

A trading desk willing to instrument their execution pipeline with ChronoSentiment is the validation path from "empirical finding on NSE" to "production deployment at an institutional client."

### What to offer a design partner

- Free instrumentation of their execution pipeline
- Chronology integrity monitoring on their existing data feeds
- Replay-safe timestamp certification for their execution audit trail
- Co-authorship of a case study documenting the findings

### What to ask for

- Access to their live feed data (anonymised is acceptable)
- Permission to document the synchronisation fragmentation pattern (anonymised)
- A reference letter for DFSA regulatory dialogue

---

## Deferred (post-design-partner)

These are real items but not on the critical path:

| Item | Reason deferred |
|------|----------------|
| Rename `scripts/ecology_signature_atlas.py` | Legacy filename, non-operational, residue not active drift |
| Extend provider clustering to Gulf symbol sets | Requires Gulf feed data first |
| Build Trade Inspector surface | Requires live execution data from a design partner |
| DFSA regulatory dialogue | Requires Gulf validation finding + design partner reference |
| Make governor thresholds configurable per asset class/feed | Requires design partner feedback on threshold calibration |
| Build audit trail export surface (DFSA-facing compliance artifact) | Requires DFSA dialogue to understand format requirements |

---

## What not to build (ever, or until Phases 1–4 are complete)

- Agentic AI / autonomous strategy generation
- LLM cognition layers
- Adaptive inference systems
- Prediction signals or alpha generation
- Retail trading UI patterns (PnL casino, neon aesthetics, indicator walls)

These systems pressure architecture toward probabilistic shortcuts, hidden mutable state, and non-deterministic execution — the exact failure modes ChronoSentiment is designed to detect and prevent.

**The strongest advantage is:** chronology authority remains upstream of intelligence. Protect that boundary.

---

## Branch strategy

| Branch | Purpose |
|--------|---------|
| `master` | QFTH submission state — stable, do not modify |
| `post-qfth/cert-restore-and-gulf-validation` | This branch — all post-submission development |

When Phase 1 (cert restore) is complete and verified, consider creating sub-branches:
- `post-qfth/phase-2-gulf-validation`
- `post-qfth/phase-3-frontend-canonical`
- `post-qfth/phase-4-replay-timeline`
