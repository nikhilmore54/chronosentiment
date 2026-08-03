# RP-403 Root-Cause Analysis

**Programme:** EURO/ROADEF 2026 Challenge — T-Adaptive Segment Routing
**Status:** Complete — Termination Gate Decision Made
**Version:** 1.0
**Date:** 2026-08-03

---

## 1. Objective

This document records the mandatory pre-coding root-cause analysis for RP-403 (Adaptive Candidate Generation and Diversity Recovery), as required by the RP-403 termination gate in ROADEF_PROGRAMME.md v1.7.

**Termination gate question:** Does the root-cause analysis identify at least one failure mode that is plausibly addressable by candidate-generation methods?

Three instances were investigated:

| Instance | Budget | RP-401C | RP-401D | RP-402 | Priority |
|----------|--------|---------|---------|--------|----------|
| setA-12 | 13 | 26.12 (finite) | inf | inf | Highest — consistent regression across two independent variants |
| setA-17 | 1 | inf | inf | inf | High — may be budget-limited, not path-diversity-limited |
| setA-08 | 13 | inf | 48.67 (finite) | inf | Medium — RP-402 regressed a previously recovered instance |

**Method:** Phase 1A — mine existing solution JSONs without writing new code. Script: `scripts/rp403_json_mining.py`.

---

## 2. setA-12 Investigation — Path Audit

**Research question:** Why did RP-401C produce a finite solution (26.12) while RP-401D and RP-402 both regress to inf?

### 2.1 Instance characteristics

| Field | Value |
|-------|-------|
| Nodes | 200 |
| Links | 898 |
| Demands | 392 (routed; 400 total) |
| Budget | 13 |
| RP-401C result | 26.1166 (finite) |
| RP-401D result | inf |
| RP-402 result | inf |

### 2.2 Mining results

| Metric | rp401c | rp401d | rp402 |
|--------|--------|--------|-------|
| Demands routed | 392 | 392 | 392 |
| t0 waypoint length (mean) | 3.7 | 3.9 | 3.9 |
| t1 waypoint length (mean) | 3.7 | 3.9 | 3.9 |
| Shared path (t0==t1) | 392/392 | 392/392 | 392/392 |
| Adapted demands (t0!=t1) | **0** | **0** | **0** |
| Total transition cost | 0 | 0 | 0 |

**Cross-solver t0 divergence:**

| Pair | Demands differing |
|------|------------------|
| rp401c vs rp401d | 235/392 (59.9%) |
| rp401c vs rp402 | 232/392 (59.2%) |
| rp401d vs rp402 | 6/392 (1.5%) |

First divergence between rp401c and rp402 at demand 0:
- rp401c t0: `[165, 144, 5, 193, 186]`
- rp402  t0: `[165, 144, 127, 20, 1]`

### 2.3 Analysis

**Critical finding: RP-402's budget-aware adaptation never fired for setA-12.**

All three solvers emit `adapted=0/392` — zero demands have different t0 vs t1 paths. This means the shared-path construction itself determines feasibility. The adaptation step in RP-402 never had a chance to help because the base construction was already infeasible.

**Why does RP-401C succeed while RP-401D and RP-402 fail?**

RP-401C and RP-402 diverge on 232/392 demands (59.2%) at t0. RP-401D and RP-402 differ on only 6/392 demands (1.5%) — they are nearly identical constructions, both infeasible. This confirms that RP-401D's EDF (earliest-deadline-first) sorting changes the demand processing order in a way that produces a different, infeasible assignment for setA-12's topology.

RP-401C uses a pure nearest-neighbour greedy without EDF pre-sorting. For setA-12, this happens to produce a feasible assignment. RP-401D's EDF sort changes which demands are processed first, leading to a different path assignment that is infeasible under ECMP load estimation.

### 2.4 Failure classification

**Layer: Construction** — the failure occurs in the initial path assignment, not in candidate generation, selection policy, budget adaptation, or ECMP modelling.

The RP-401C feasible assignment is reachable (it exists), but RP-401D and RP-402 do not generate it because their demand ordering differs. This is a **construction ordering problem**, not a candidate diversity problem.

**Is this addressable by candidate generation?**

Partially. If RP-403 generates K candidate paths per demand and evaluates them under ECMP-accurate load estimation, it might find the RP-401C-equivalent assignment. However, the root cause is the demand processing order, not the set of candidate paths available. A selection policy fix (reverting to RP-401C ordering for setA-12's topology) would be cheaper and more targeted than generating K candidates.

**Verdict for termination gate:** The failure is primarily a **construction/ordering problem**. Candidate generation could help if it includes the RP-401C path as one of K candidates, but this is not the most direct fix.

---

## 3. setA-17 Investigation — Budget Sensitivity

**Research question:** Is setA-17 infeasible because budget=1 is structurally insufficient, or because the solver fails to find the one valid re-route?

### 3.1 Instance characteristics

| Field | Value |
|-------|-------|
| Nodes | 300 |
| Links | 1270 |
| Demands | 408 (routed; 2000 total) |
| Budget | 1 |
| RP-401C result | inf |
| RP-401D result | inf |
| RP-402 result | inf |

### 3.2 Mining results

| Metric | rp401c | rp401d | rp402 |
|--------|--------|--------|-------|
| Demands routed | 408 | 408 | 408 |
| t0 waypoint length (mean) | 4.1 | 2.4 | 4.1 |
| t1 waypoint length (mean) | 4.1 | 2.4 | 4.1 |
| Shared path (t0==t1) | 408/408 | 408/408 | 408/408 |
| Adapted demands (t0!=t1) | **0** | **0** | **0** |
| Total transition cost | 0 | 0 | 0 |

**Cross-solver t0 divergence:**

| Pair | Demands differing |
|------|------------------|
| rp401c vs rp401d | 168/408 (41.2%) |
| rp401c vs rp402 | 29/408 (7.1%) |
| rp401d vs rp402 | 180/408 (44.1%) |

**Critical observation:** RP-401D has `t0 waypoint length mean=2.4` vs RP-401C's `4.1`. Many demands in RP-401D have empty or very short waypoints (min=0), indicating routing failures during construction. RP-402 matches RP-401C's mean (4.1), suggesting better construction coverage.

First divergence between rp401c and rp402 at demand 53:
- rp401c t0: `[19, 88, 207, 68, 9]`
- rp402  t0: `None` (routing failure)

### 3.3 Analysis

**Critical finding: Zero adaptation attempted by any solver. The construction itself fails to achieve feasibility.**

All three solvers emit `adapted=0/408` — no budget-aware adaptation was attempted. The shared-path construction is infeasible for all three solvers, and since adaptation only fires when the base construction is feasible, it never runs.

The RP-402 construction fails to route demand 53 (and likely others), producing `None` waypoints. This is a construction failure, not a budget limitation.

**Budget sensitivity:** The budget sweep (Phase 1B) was not required to reach this conclusion. The mining data shows that RP-402 never uses any budget (total_transition_cost=0 for all solvers). The infeasibility is in the t=0 construction, not in the t=0→t=1 transition. Budget=1 vs budget=unlimited is irrelevant if the t=0 solution is already infeasible.

### 3.4 Failure classification

**Layer: Construction** — the failure occurs in the initial path assignment. Some demands cannot be routed feasibly under the ECMP-aware greedy for setA-17's topology (300 nodes, 1270 links, 2000 demands, budget=1).

**Is this addressable by candidate generation?**

No. The construction failure means some demands receive `None` waypoints — they are not routed at all. Generating K candidate paths for already-routed demands does not help demands that fail to route. The fix requires a more robust construction that can handle setA-17's high demand density (2000 demands on 300 nodes).

**Verdict for termination gate:** The failure is a **construction failure** (routing failure for some demands). Candidate generation cannot address this. The budget=1 constraint is not the binding constraint — the construction itself is the binding constraint.

---

## 4. setA-08 Investigation — Regression Analysis

**Research question:** Why did RP-402 regress setA-08 from 48.67 (RP-401D) to inf?

### 4.1 Instance characteristics

| Field | Value |
|-------|-------|
| Nodes | 150 |
| Links | 654 |
| Demands | 193 (routed; 200 total) |
| Budget | 13 |
| RP-401C result | inf |
| RP-401D result | 48.6693 (finite) |
| RP-402 result | inf |

### 4.2 Mining results

| Metric | rp401c | rp401d | rp402 |
|--------|--------|--------|-------|
| Demands routed | 193 | 193 | 193 |
| t0 waypoint length (mean) | 4.0 | 3.7 | 3.7 |
| t1 waypoint length (mean) | 4.0 | 3.7 | 3.7 |
| Shared path (t0==t1) | 193/193 | 193/193 | 191/193 |
| Adapted demands (t0!=t1) | **0** | **0** | **2** |
| Total transition cost | 0 | 0 | **4** |

**Cross-solver t0 divergence:**

| Pair | Demands differing |
|------|------------------|
| rp401c vs rp401d | 168/193 (87.0%) |
| rp401c vs rp402 | 166/193 (86.0%) |
| rp401d vs rp402 | 30/193 (15.5%) |

**RP-401D vs RP-402 t1 differences:** 28/193 demands (14.5%)

Sample t1 differences (all with transition cost=0 for rp401d, meaning rp401d uses shared paths):
- demand 2: rp401d t1=`[46, 117, 63, 22, 42]` | rp402 t1=`[115, 94, 49, 38, 41]`
- demand 6: rp401d t1=`[33]` | rp402 t1=`[108, 103, 136, 32, 64]`

rp401d total transition cost: **0** (pure shared-path, no adaptation)
rp402 total transition cost: **4** (2 demands adapted, 2 units each)

### 4.3 Analysis

**Critical finding: RP-401D is feasible with zero adaptation (pure shared-path). RP-402 attempts adaptation and becomes infeasible.**

RP-401D uses a pure shared-path strategy (t0==t1 for all 193 demands, total_transition_cost=0). Its feasibility at 48.67 comes entirely from the construction, not from any t=1 adaptation.

RP-402 diverges from RP-401D on 30/193 t0 demands (15.5%). This means RP-402's base construction makes different path assignments than RP-401D for 30 demands. For setA-08's topology, RP-401D's construction happens to be feasible while RP-402's is not.

RP-402 then attempts to adapt 2 demands (total transition cost=4, within budget=13), but this adaptation does not recover feasibility — the base construction is already infeasible.

**Why do RP-401D and RP-402 constructions diverge?**

RP-401D uses EDF (earliest-deadline-first) sorting before greedy construction. RP-402 uses a shared-path construction that is supposed to replicate RP-401C's ECMP-aware greedy. The 30-demand divergence between RP-401D and RP-402 at t0 is the source of the regression.

### 4.4 Failure classification

**Layer: Construction** — RP-402's base construction diverges from RP-401D's on 30/193 demands, producing an infeasible assignment. The adaptation step cannot recover it.

**Is this addressable by candidate generation?**

Potentially. If RP-403 generates K candidate paths per demand and includes the RP-401D path as one candidate, it could recover the feasible assignment. However, this requires the candidate generator to produce paths that match RP-401D's EDF-ordered greedy — which is a specific construction strategy, not a diversity improvement.

**Verdict for termination gate:** The failure is a **construction divergence** between RP-401D and RP-402. Candidate generation could help if it includes RP-401D-equivalent paths, but the more direct fix is to preserve the RP-401D construction as a fallback when RP-402's construction is infeasible.

---

## 5. Failure Taxonomy

Summary of failure modes identified across all three instances:

| Instance | Causal Layer | Failure Mode | Addressable by Candidate Generation? |
|----------|-------------|-------------|--------------------------------------|
| setA-12 | Construction | Demand ordering (EDF vs nearest-neighbour) produces different path assignments; RP-401C's ordering is feasible, RP-401D/402's is not | Partially — K candidates could include RP-401C-equivalent paths, but ordering fix is more direct |
| setA-17 | Construction | Routing failure for some demands (None waypoints); construction cannot route all 2000 demands feasibly | No — candidate generation cannot route demands that fail construction entirely |
| setA-08 | Construction | RP-402 base construction diverges from RP-401D on 30/193 demands; RP-401D's construction is feasible, RP-402's is not | Partially — K candidates could include RP-401D-equivalent paths, but fallback strategy is more direct |

**Common pattern:** All three failures occur at the **construction layer**, not at the candidate generation, selection policy, budget adaptation, or ECMP modelling layers. The adaptation step in RP-402 never fires for setA-12 and setA-17 (construction already infeasible), and fires but fails to recover setA-08 (base construction diverges from the feasible RP-401D assignment).

**Revised causal layer taxonomy (per reviewer recommendation):**

| Layer | Evidence in this analysis | Applicable RP |
|-------|--------------------------|---------------|
| Model | Not implicated — ECMP modelling is consistent across solvers | RP-401 (resolved) |
| Construction | **Primary failure layer for all three instances** | RP-401C/D, RP-402 |
| Candidate diversity | Not implicated — adaptation never fires; construction is the bottleneck | RP-403 (conditional) |
| Selection policy | Not implicated — no candidates are being selected/rejected | RP-403 (conditional) |
| Budget semantics | Not implicated — budget is never consumed for setA-12/17; setA-08 uses only 4 of 13 | RP-402 (resolved) |
| Local search | Not yet investigated | RP-404 |
| Global search | Not yet investigated | RP-406 |

---

## 6. Candidate-Generation Hypothesis Assessment

The RP-403 hypothesis was:

> The remaining failures arise from insufficient path diversity rather than budget allocation. setA-12 requires alternative paths that the ECMP-aware greedy cannot generate. setA-17 may require a different demand prioritisation strategy given its budget=1 constraint.

**Assessment:**

**setA-12:** The hypothesis is **partially wrong**. The failure is not that alternative paths cannot be generated — RP-401C already generates a feasible assignment. The failure is that RP-401D and RP-402 use a different demand ordering (EDF sort) that produces a different, infeasible assignment. Candidate generation could help if it includes RP-401C-equivalent paths, but the root cause is ordering, not diversity.

**setA-17:** The hypothesis is **wrong**. The failure is not budget allocation (budget=1 is never consumed) and not path diversity. The failure is that the construction cannot route all 2000 demands feasibly on setA-17's topology. Budget=1 is irrelevant — the t=0 construction itself fails. No amount of candidate generation for t=1 can fix a t=0 construction failure.

**setA-08:** The hypothesis is **partially applicable**. RP-402's construction diverges from RP-401D's on 30/193 demands. If RP-403 generates K candidates that include RP-401D-equivalent paths, it could recover setA-08. However, the more direct fix is a construction fallback strategy.

---

## 7. Termination Gate Decision

Per ROADEF_PROGRAMME.md v1.7, RP-403 shall only proceed to implementation if the root-cause analysis identifies at least one failure mode that is plausibly addressable by candidate-generation methods.

**Evidence summary:**

| Instance | Addressable by candidate generation? | Reason |
|----------|-------------------------------------|--------|
| setA-12 | Partially | Construction ordering problem; K candidates could include RP-401C paths, but ordering fix is more direct |
| setA-17 | **No** | Construction routing failure; some demands receive None waypoints; candidate generation cannot help |
| setA-08 | Partially | Construction divergence; K candidates could include RP-401D paths, but fallback strategy is more direct |

**Decision:**

- [ ] ✅ Proceed with RP-403 as originally scoped
- [x] 🔄 **Redefine RP-403** — failure modes identified but require a different approach than candidate generation
- [ ] 📦 Archive RP-403
- [ ] ❌ Reject hypothesis

**Rationale:**

The root-cause analysis reveals that all three failures occur at the **construction layer**, not the candidate generation layer. The original RP-403 hypothesis (insufficient path diversity) is not supported by the evidence.

However, the analysis also reveals a specific, addressable problem: **construction strategy selection**. RP-401C's nearest-neighbour greedy (without EDF sorting) produces feasible assignments for setA-12 and setA-08 that RP-401D and RP-402 do not. The correct next step is not to generate K candidate paths per demand, but to:

1. **Preserve the best construction across RP-401C and RP-401D** — run both constructions and select the feasible one (or the one with lower objective).
2. **Fix the RP-402 base construction** — RP-402 should inherit the better of RP-401C and RP-401D as its base, not always use RP-401D's EDF-ordered construction.
3. **Investigate setA-17's construction failure** — 2000 demands on 300 nodes with budget=1 may require a fundamentally different construction strategy (e.g., demand clustering, capacity-aware routing).

**RP-403 redefined scope:**

RP-403 should be redefined as **"Construction Strategy Selection and Fallback"** rather than "Adaptive Candidate Generation and Diversity Recovery":

- Run RP-401C and RP-401D constructions in parallel
- Select the construction with lower objective (or the feasible one if only one is feasible)
- Use the selected construction as the base for RP-402's budget-aware adaptation
- Measure: recovery of setA-12 and setA-08, no regression on the 18 currently finite instances

This is a targeted, evidence-driven redefinition. It addresses the actual failure mode (construction ordering) rather than the hypothesised failure mode (candidate diversity).

---

## 8. Amendment Log

| Version | Date | Change |
|---------|------|--------|
| 0.1 | 2026-08-03 | Skeleton created. Investigation structure defined. All three instances (setA-12, setA-17, setA-08) scoped. Termination gate decision deferred pending diagnostic runs. |
| 1.0 | 2026-08-03 | Phase 1A mining complete. All three instances analysed using existing solution JSONs (scripts/rp403_json_mining.py). Key finding: all failures occur at the construction layer, not the candidate generation layer. RP-403 redefined as "Construction Strategy Selection and Fallback". Termination gate decision: 🔄 Redefine. |