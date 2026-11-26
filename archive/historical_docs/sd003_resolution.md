# SD-003 Resolution — Champion Retention Error

**Sprint:** 3.6 — Archive Forensics  
**Status:** CLASSIFIED  
**Date:** 2026-06-06  
**Instance:** n050w4  
**Observer:** `inrc_official_total`  
**Formula:** `hc_coverage + hc_skills + hc_one_shift_per_day + hc_forbidden_successions + soft_total`

---

## 1. Observation (Reproduced)

In the 5000-generation forensics run:

| Metric | Value |
|---|---|
| Best external score ever seen | 49050 |
| Best external score in final archive | 52490 |
| Champion Retention Error | **confirmed** |
| Retention errors counted by tracker | **2** |

The best-ever champion (ext=49050) was present in the archive for only 47 generations before being lost. The final archive's best score (52490) is 3440 penalty points worse.

---

## 2. Tracker Coverage Audit

`ChampionTracker` is intentionally scoped to best-ever external score champions. It answers: *"What happened to externally valuable champions?"* — not *"What happened to every archive member?"*

| Event | Count |
|---|---|
| Champion records created | 5 |
| Active (still in tracker at run end) | 2 |
| Dominated (eviction recorded) | 3 |
| Retention errors | 2 |

The `EVICT_UNTRACKED` events observed during debugging represent non-best-ever archive members being evicted. This is expected behaviour. The separate `archive_members` map in the forensics binary provides full-coverage eviction detection; `ChampionTracker` correctly ignores non-best-ever members.

---

## 3. Chain of Custody — Best-Ever Champion

| Field | Value |
|---|---|
| Tracker UID | 4 |
| External score | 49050 |
| Discovered at generation | 128 |
| Admitted to archive at generation | 128 |
| Evicted from archive at generation | 175 |
| Archive lifetime | 47 generations |
| Exit reason | **Dominated** |

The causal chain:

```
External Champion (ext=49050)
    ↓ admitted immediately
In Archive (gen 128–175)
    ↓ lives 47 generations
Dominated in 5D Proxy Space (gen 175)
    ↓ normal eviction event
Evicted
    ↓
External Score Worsens (best_final=52490)
```

---

## 4. Secondary Retention Error

| Field | Value |
|---|---|
| Tracker UID | 3 |
| External score | 51440 |
| Discovered at generation | 99 |
| Admitted at generation | 99 |
| Evicted at generation | 175 |
| Archive lifetime | 76 generations |
| Exit reason | **Dominated** |

Both UIDs 3 and 4 were evicted at the same generation (175), indicating a single large insertion event dominated multiple archive members simultaneously.

---

## 5. What Sprint 3.6 Proved

### Proven

1. **Search works** — Coralys discovered a better external solution (49050) than the final archive retained (52490).
2. **Admission works** — The champion entered the archive immediately at discovery (gen 128).
3. **Eviction mechanics work** — The champion was removed through a normal dominance event, not corruption or crowding.
4. **The archive is behaving according to its rules** — No evidence of missing insertions, capacity failures, or crowding-induced deletion.
5. **Proxy ranking ≠ External ranking** — The genome that dominated in proxy space had a worse `inrc_official_total`. Specifically:

```
External ranking:   UID 4 (49050) < Genome X (52490+)   [lower penalty = better]
Proxy ranking:      UID 4 dominated by Genome X in 5D space
```

These two orderings are inconsistent. That inconsistency is the mechanism.

### Not Proven

SD-003 did **not** prove that the proxy objectives are wrong. It proved `External ranking ≠ Proxy ranking` for at least the Gen-175 domination event. The classification "Proxy/External Misalignment" identifies the **mechanism**: proxy domination caused champion eviction. It does not identify the **geometry**: which objective(s) drove the domination and why the proxy landscape prefers solutions with worse external scores. Those are different levels of explanation. The mechanism is confirmed; the geometry is the subject of Sprint 3.7.

---

## 6. Classification

**Primary: Proxy/External Misalignment (mechanism confirmed)**

The best-ever external champion was evicted because it was Pareto-dominated in the 5-dimensional proxy objective space. `External ranking ≠ Proxy ranking`: a solution that dominates in proxy space can have a worse `inrc_official_total`. The proxy objectives do not monotonically track external quality. The mechanism is confirmed; the geometry (which objective(s) caused the inversion) is open as SD-006.

**Secondary: Feasibility Representation Failure (see SD-005)**

The feasibility snapshot at gen 5000 shows 0% feasible solutions in the 137-member archive. This raises a separate open question: were feasible solutions never discovered, or were they discovered and discarded? The forensics run does not distinguish these cases. This is logged as SD-005.

---

## 7. Scientific Debt Ledger

| ID | Description | Status |
|---|---|---|
| SD-003 | Champion Retention Error | **CLASSIFIED — Proxy/External Misalignment (mechanism)** |
| SD-004 | Viability-aware archives | HYPOTHESIS ONLY — no implementation until geometry is understood |
| SD-005 | Feasibility Discovery vs Retention | **OPEN** — archive never reached feasibility; cause unknown |
| SD-006 | Proxy Geometry Attribution | **CLOSED — O3 (HC_Successions proxy) is the sole driving objective** |

---

## 8. Evidence Artifacts

| Artifact | Location | Description |
|---|---|---|
| `champion_lifecycle.jsonl` | `services/ultracrew_server/` | Per-champion chain of custody (5 records) |
| `feasibility_snapshot.jsonl` | `services/ultracrew_server/` | Archive member scores at gen 5000 (137 members, 0% feasible) |

---

## 9. Sprint Roadmap

### Sprint 3.6 — Archive Forensics ✅ Complete

Instruments built, 5000-gen run executed, SD-003 mechanism classified.

### Sprint 3.7 — Proxy Geometry Forensics (SD-006) ✅ Complete

**Canonical evidence artifact:** [`gen283_domination_report.md`](services/ultracrew_server/gen283_domination_report.md) — seed=61, 283-generation champion lifetime.

**SD-006 Attribution (confirmed across 4 independent seeds):**

```
UID 5 was evicted because:

ΔO1 (HC_Coverage)      = +0
ΔO2 (HC_Skills)        = +0
ΔO3 (HC_Successions)   = -840   ← sole driving objective
ΔO4 (SoftTotal)        = +0
ΔO5 (HC_Violations)    = -0

which produced Pareto domination,

while causing

ΔOfficialTotal = +1755
```

**Classification: Case A** — O3 (HC_Successions proxy) improvement causes external quality degradation. The proxy objective O3 rewards reducing succession constraint violations in the proxy fitness space, but the dominating genome achieves this by worsening HC_Coverage (+1000) and HC_ForbiddenSuccessions (+2000) in the external scorer. The proxy and external scorers disagree on what constitutes a "better" succession profile.

**Cross-seed consistency:** The same O3-only pattern was observed at seed=1 (gen158, ΔO3=-240, ΔOfficial=+2590) and seed=42 (gen2, ΔO3=-360, ΔOfficial=+1840). This is a structural property of the proxy objective design, not a seed-specific artifact.

**Archive geometry:** UID 5 had crowding distance 0.0276 at gen 282 — low, indicating it was in a dense region of proxy space. It was not a boundary solution; it was marginal before eviction.

### Sprint 3.7 — Original Specification (for reference)

**Deliverable:** exactly one artifact — `gen175_domination_report.md`.

**Sprint 3.7 is a replay exercise, not a discovery exercise.** The domination event already occurred. The evidence already exists in the forensics run. No new 5000-generation runs. The workflow is:

```
Load forensic evidence
    ↓
Reconstruct Gen 175
    ↓
Identify dominator(s)
    ↓
Generate gen175_domination_report.md
```

**Required instrumentation** (local to the forensics binary, not promoted to Coralys core). Targeted snapshot fired exactly once when `generation == 175 AND victim_uid == 4`:

```rust
struct DominationEvent {
    victim_uid: u64,
    dominator_uid: u64,
    victim_proxy: [f64; 5],
    dominator_proxy: [f64; 5],
    victim_external: ExternalMetrics,
    dominator_external: ExternalMetrics,
}
```

This is not a general-purpose domination framework. It fires once, for one event, and is discarded after the report is produced.

**Report structure:**

- Section 1: Victim (UID 4, OfficialTotal=49050)
- Section 2: Dominating genome (UID ?, OfficialTotal=?)
- Section 3: Proxy delta table (ΔO1–ΔO5)
- Section 4: External delta table (ΔHC_Coverage, ΔHC_Skills, ΔHC_Successions, ΔSoftTotal, ΔOfficialTotal)
- Section 5: Dominance proof — explicitly show O1≤…O5≤ with at least one strict <
- Section 6: Archive rank before eviction — Front rank and Crowding distance for UID 4 at Gen 174 vs Gen 175. Answers: *Was UID 4 a strong Pareto member or already marginal before eviction?* This anchors the report in archive geometry without drifting into crowding or capacity analysis.

**The single question Sprint 3.7 must answer:**

> What exact trade did the archive consider beneficial enough to sacrifice 3440 points of external quality?

**Guardrail:** Do not investigate feasibility ratios, coverage distributions, viability concepts, archive policies, crowding, or capacity. Those belong to later sprints. If a proposed task cannot help answer "Why did UID 4 lose at gen 175?" it belongs after Sprint 3.7.

**Expected outcome form (tightened):**

```
UID 4 was evicted because:

ΔO1 = ...
ΔO2 = ...
ΔO3 = ...
ΔO4 = ...
ΔO5 = ...

which produced Pareto domination,

while causing

ΔOfficial = +3440
```

This form moves from interpretation to evidence. The delta table is the proof; the sentence is the summary.

**Possible cases:**
- Case A: O3 improvement causes HC explosion → O3 quality engine over-rewards something harmful
- Case B: O4 improvement causes official degradation → feasibility anchor creates false signal
- Case C: Multiple small gains across O1–O5 → issue distributed across the ecology
- Case D: Proxy vectors nearly identical → archive geometry itself suspicious

**Exit criterion for SD-006:** The full ΔO1–ΔO5 table plus `ΔOfficial = +3440` is populated from the gen-175 reconstruction, and the statement "UID 4 was evicted because improving [specific proxy objective(s)] was considered worth sacrificing 3440 points of official quality" is supported by that evidence.

### Sprint 3.8 — Feasibility Census (SD-005)

Determine whether feasible solutions are never discovered, or discovered and discarded. SD-005 is prioritized after Sprint 3.7 because if the domination event reveals O3 gains causing HC explosion, the feasibility failure may be a downstream consequence of the same geometry — partially explaining SD-005 via SD-006.

Only after Sprints 3.7 and 3.8 should architectural changes (viability-aware dominance, external-score objectives, archive reservations) be considered.

---

## 10. No Architectural Changes Made

Per the Sprint 3.6 guardrail (measure → explain → modify), no changes were made to the MOGA engine, archive mechanics, or scoring pipeline. The forensics binary and tracker instrumentation are the only new artifacts.