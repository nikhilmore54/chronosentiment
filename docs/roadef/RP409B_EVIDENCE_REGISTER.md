# RP-409B Evidence Register

**Version:** 1.0
**Status:** Active — updated as campaigns complete
**Scope:** All evidence supporting research contributions C-1 through C-5 in RC-001 A/B Report v0.7

---

## Purpose

This register provides a permanent, traceable index of every piece of benchmark evidence that supports a research contribution. Every claim in the RC-001 A/B Report Section 13 must have at least one entry here before SR-001 can be passed.

The register is the canonical reference for the Claim Traceability Matrix in SR-001.

---

## Evidence Index

| Evidence ID | Source Document | Source Campaign | Benchmark Instances | Supports Claim | Evidence Status |
|-------------|----------------|-----------------|---------------------|----------------|-----------------|
| E-001 | RC-001 A/B Report §6 | rc001_ab_v2.3 | All 20 setA instances | C-1 (greedy superiority) | ✅ Confirmed |
| E-002 | RC-001 A/B Report §6, Score Summary | rc001_ab_v2.3 | 9 instances (both valid) | C-1 (100% win rate when both valid) | ✅ Confirmed |
| E-003 | RC-001 A/B Report §9 | rc001_ab_v2.3 | setA-01..20 | C-2 (three-class taxonomy) | ✅ Confirmed |
| E-004 | RC-001 A/B Report §12, ms/eval table | rc001_ab_v2.3 | setA-01, 10, 13, 16, 17 | C-3 (evaluation cost scaling) | ✅ Confirmed |
| E-005 | RC-001 A/B Report §12, state-dependent cost | rc001_ab_v2.3 | setA-16 (Greedy vs Random) | C-3 (4.4× same-instance difference) | ✅ Confirmed |
| E-006 | RC-001 A/B Report §12, throughput collapse | rc001_ab_v2.3 | setA-16, 19, 20 | C-4 (evaluation dominates scalability) | ✅ Confirmed |
| E-007 | RC-001 A/B Report §9, Class C analysis | rc001_ab_v2.3 | setA-17 (arc 66, 67, 163, 606, 968) | C-5 (recurring structural bottlenecks) | ✅ Confirmed |
| E-008 | RC-003 report | rc003_lex_v1.0 | All 20 setA instances | C-1 (objective alignment) | ⏳ Pending RC-003 |
| E-009 | RC-004A report | rc004a_v1.0 | All 20 setA instances | C-3 (ms/eval baseline) | ⏳ Pending RC-004A |
| E-010 | RC-004B report | rc004b_v1.0 | setA-16 (profiling target) | C-3, C-4 (evaluator internals) | ⏳ Pending RC-004B |
| E-011 | RC-001B report | rc001b_v1.0 | All 20 setA instances | C-2, C-5 (topology classifier) | ⏳ Pending RC-001B |
| E-012 | RC-006A report | rc006a_v1.0 | setA-18, setA-20 | C-2 (EA correctness) | ⏳ Pending RC-006A |

**Evidence Status values:** ✅ Confirmed — present in completed campaign, reviewed. ⏳ Pending — will be produced by named future campaign. ❌ Rejected — evidence was collected but does not support the claim. 🔄 Superseded — replaced by a later, stronger evidence entry (original entry retained for audit trail).

---

## Claim Coverage Summary

| Claim | Required Evidence | Available Now | Pending |
|-------|------------------|---------------|---------|
| C-1: Greedy superiority | E-001, E-002, E-008 | E-001, E-002 | E-008 (RC-003) |
| C-2: Three-class taxonomy | E-003, E-011, E-012 | E-003 | E-011 (RC-001B), E-012 (RC-006A) |
| C-3: State-dependent eval cost | E-004, E-005, E-009, E-010 | E-004, E-005 | E-009 (RC-004A), E-010 (RC-004B) |
| C-4: Evaluation bottleneck | E-006, E-010 | E-006 | E-010 (RC-004B) |
| C-5: Structural bottlenecks | E-007, E-011 | E-007 | E-011 (RC-001B) |

---

## Notes

- Evidence IDs are permanent. Once assigned, they are never reused or renumbered.
- "Confirmed" status means the evidence is present in a completed campaign report and has been reviewed.
- "Pending" status means the evidence will be produced by the named future campaign.
- SR-001 cannot be passed until all entries in the "Pending" column are resolved.

---

*Register created: 2026-08-07. Maintained by: RC-001 A/B programme.*