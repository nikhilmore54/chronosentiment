# UltraRoster — Decision Support Roadmap

**Branch:** `governance-hardening`
**Status:** P2 FROZEN — P3 AUTHORIZED (Decision Selection UI)
**Last updated:** 2026-08-30

---

## Governance Constraints (frozen)

- No changes to `optimization.rs`, `decision_support.rs`, or any Coralys core module.
- P1 finding is frozen: alternatives-generation hypothesis failed.
- P1.1 (meaningful alternatives) is deferred until after P2 is complete.
- Each phase is bounded. No automatic escalation to the next phase.

---

## Phase Summary

### P1 — Decision Presentation Layer (COMPLETE, finding frozen)

**Objective:** Verify that the presentation layer surfaces meaningful alternatives to the planner.

**Finding:** Hypothesis failed. `run_partitioned_evolution` → `top_10` →
`DecisionSupportEngine::generate_decision_matrix` produces essentially identical solutions.
Only 1 meaningful alternative was found (warning `decision_result.alternatives.len() < 3` fires).

**Conclusion:** The alternatives-generation mechanism does not produce diversity at the
presentation layer. Root cause is upstream in the optimizer, not in the presentation layer.

**Files:**
- `adapters/ultracrew/src/bin/ultraroster_p1_demo.rs` — P1 demonstration binary
- `adapters/ultracrew/src/decision_support.rs` — `DecisionSupportEngine`, `DecisionResult`

---

### P1.1 — Meaningful Alternatives (DEFERRED)

**Objective:** Fix the optimizer to produce diverse alternatives for the planner.

**Status:** Deferred until after P2. Requires changes to the optimizer (Coralys core or
`optimization.rs`). Authorized as one bounded attempt when explicitly re-authorized.

---

### P2 — Decision Memory (IMPLEMENTED)

**Objective:** Build a memory model that captures the full decision lifecycle so that future
decisions can be informed by past experience.

**Commit:** `4c19355e2` — "UltraRoster P2: Decision Memory — model + demo + stage() bug fix"
**Files changed:** 3 files, 812 insertions

**Files:**
- `adapters/ultracrew/src/decision_memory.rs` — core memory model (430 lines)
- `adapters/ultracrew/src/bin/ultraroster_p2_demo.rs` — demonstration binary (200+ lines)
- `adapters/ultracrew/src/lib.rs` — `pub mod decision_memory` registered

**Lifecycle stages:**

| Stage | Name | Trigger |
|-------|------|---------|
| 1 | PRESENTED | System generated alternatives and recommendation |
| 2 | DECIDED | Planner selected an alternative (or rejected all) |
| 3 | MODIFIED | Planner modified the selected roster |
| 4 | APPROVED | Final roster approved for execution |
| 5 | OBSERVED | Outcome recorded after execution |

**Key types:**

- `SituationFingerprint` — compact description of the planning context; `similarity()` method
  returns [0.0, 1.0] across 5 dimensions (worker_count, shift_count, horizon_hours,
  weekend_ratio, locked_assignment_count). `scenario_id` mismatch short-circuits to 0.0.
- `AlternativeSnapshot` — metrics of one alternative as presented to the planner.
- `PlannerChoice` — `AcceptedRecommendation` | `AcceptedAlternative { id }` | `RejectedAll`.
- `AssignmentChange` — one shift reassignment made by the planner.
- `OutcomeQuality` — `Successful` | `MinorIssues` | `MajorIssues` | `Pending`.
- `ObservedOutcome` — actual coverage, violations, notes, timestamp.
- `DecisionRecord` — full lifecycle record; append-only stage advancement via methods.
- `DecisionMemory` — in-memory store; `append`, `get`, `get_mut`, `find_similar`,
  `completed_decisions`, `summary`, `to_json`, `from_json`.
- `MemorySummary` — aggregate stats: total, completed, accepted_unchanged, overrode,
  successful, mean_modifications.

**Design principles:**
1. Append-only: records are never modified after creation.
2. Serializable: all records persist to/from JSON via serde.
3. Queryable: `find_similar()` retrieves past decisions by situation fingerprint.
4. No optimizer dependency: memory is captured at the presentation layer only.

**Bug fix in `stage()` method:**
- MODIFIED gate uses `modification_count > 0` (not `decided_at_unix_ms.is_some()`).
- Decision with 0 modifications after `AcceptedRecommendation` correctly reports `DECIDED`.
- Decision with ≥1 modifications correctly reports `MODIFIED`.

**Demo output (all 10 checks PASS):**
```
[PASS] Stage 1 PRESENTED  — situation + alternatives captured
[PASS] Stage 2 DECIDED    — planner choice recorded
[PASS] Stage 3 MODIFIED   — assignment changes recorded
[PASS] Stage 4 APPROVED   — final roster stored
[PASS] Stage 5 OBSERVED   — outcome captured
[PASS] Partial records    — Stage 1-only record supported
[PASS] Similarity search  — scenario isolation verified
[PASS] Retrieve by ID     — O(n) lookup works
[PASS] Summary statistics — acceptance/override/outcome counts
[PASS] JSON round-trip    — serialize + deserialize consistent
```

**Build verification:**
```
cargo build --release -p ultracrew --bin ultraroster_p2_demo
# exit 0 — only pre-existing warnings, no new errors
```

---

## P3 — Decision Selection & Comparison UI (AUTHORIZED, not started)

**Objective:** Give the scheduler a UI to explore available alternatives, compare trade-offs,
and explicitly select one. The selected option feeds directly into the P2 decision record.

**Product loop this enables:**

```
Current situation → Generate decision → Explore choices → Planner decides →
Planner modifies → Approve → Observe outcome → Remember →
NEXT SIMILAR SITUATION → Use memory → Better recommendation
```

**The long-term UltraRoster differentiator:**
> "UltraRoster remembers how planners handled similar situations and what happened afterward,
> and uses that experience when recommending the next decision."

**Three concepts explicitly separated in the UI:**

- **Recommendation** — what UltraRoster thinks the planner should choose.
- **Alternative** — another feasible option available to the planner.
- **Decision** — what the planner actually chose (persisted into P2 memory).

**UI must handle the single-alternative case gracefully.** If the current engine returns only
one meaningfully distinct alternative, the UI must say so — it must not pretend there are
three alternatives when there aren't.

**IN SCOPE:**
- Update existing UltraRoster UI.
- Display recommendation + available alternatives.
- Compare alternatives (coverage, fairness, utilization, cost, changes from recommended).
- Allow scheduler to select an alternative.
- Persist selected option into P2 decision record (Stage 2 DECIDED).
- Preserve recommendation vs actual selection distinction.
- Support single-alternative case gracefully.

**OUT OF SCOPE:**
- Changing MOGA or any optimizer code.
- Diversity optimization.
- New optimizer experiments.
- P1.1 (meaningful alternatives) — remains separately deferred.
- Memory-based recommendations (P4).
- What-if optimization.
- New airline constraints.
- Any Coralys changes.

---

## Roadmap Status

| Phase | Description | Status |
|-------|-------------|--------|
| P1 | Decision Presentation Layer | COMPLETE (finding frozen) |
| P1.1 | Meaningful Alternatives | DEFERRED — separately authorized when ready |
| P2 | Decision Memory | **FROZEN** — commits `4c19355e2` + `4fe32a953` |
| P3 | Decision Selection & Comparison UI | **AUTHORIZED** — next phase |
| P4 | Memory-aware Recommendation | Not started |

**Product sequence:**
```
P1  Explore the Decision
        ↓
P2  Decision Memory  ← FROZEN
        ↓
P3  Decision Selection UI  ← NEXT
        ↓
     scheduler chooses
        ↓
P2 memory records choice
        ↓
P4  Memory-aware Recommendation
```

**P1.1 does not get smuggled into P3.** If meaningful alternatives are eventually tackled,
they get their own tightly scoped authorization.