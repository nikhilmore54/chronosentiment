# Fleet Semantics Registry v1.0

**Document ID:** FLEET-SEMANTICS-REGISTRY-v1.0  
**Status:** Active  
**Created:** 2026-07-09  
**Milestone:** M18.2 — Fleet Semantics Registry  
**Governance reference:** GOV-008 v1.2, EVIDENCE-v1.5.md (P5)

---

## Purpose

Every benchmark comparison in Coralys depends on a correct interpretation of the
fleet constraint. The same vehicle count K can mean two different things:

```
ATMOST(K)  →  routes_used ≤ K   (optimizer may use fewer vehicles)
EXACT(K)   →  routes_used = K   (optimizer must use exactly K vehicles)
```

If Coralys applies ATMOST semantics to a benchmark that requires EXACT semantics,
a solution using more vehicles than K would be:
- Accepted by Coralys as valid
- Rejected by the benchmark as infeasible
- Reported as a gap that is NOT COMPARABLE

This registry records the fleet constraint type for each benchmark family, the
evidence basis for that assignment, and the current verification status.

---

## Schema

Each family file is a TOML document with the following sections:

| Section | Required | Description |
|---------|----------|-------------|
| `[family]` | Yes | Identity: id, full name |
| `[constraint]` | Yes | Type (ATMOST/EXACT/Unspecified/Unknown), description, formal definition |
| `[source]` | Yes | Primary source for the constraint interpretation |
| `[publication]` | Yes | Original publication reference |
| `[notes]` | Yes | Derivation reasoning, campaign observations |
| `[verification]` | Yes | Status, confidence, method, date, open items |
| `[instance_exceptions]` | No | Per-instance overrides for genuine exceptions |

---

## Constraint Type Values

| Type | Formal | Meaning |
|------|--------|---------|
| `ATMOST` | `routes_used ≤ K` | Optimizer may use fewer vehicles than K |
| `EXACT` | `routes_used = K` | Optimizer must use exactly K vehicles |
| `Unspecified` | Unknown | Constraint type not definitively established; under investigation |
| `Unknown` | Unknown | Constraint type not investigated (out of scope) |

---

## Verification Status Values

| Status | Meaning |
|--------|---------|
| `Hypothesis` | Plausible interpretation based on literature consensus; not confirmed from original paper |
| `Unspecified` | Constraint type not definitively established; active investigation required |
| `Unknown` | Not investigated — out of scope |

## Confidence Values

| Confidence | Meaning |
|------------|---------|
| `High` | Strong literature consensus; no contradicting evidence; campaign results consistent |
| `Medium` | Reasonable basis but some uncertainty; negative gaps observed |
| `Low` | Significant uncertainty; negative gaps observed; original paper review required |
| `None` | No investigation performed |

---

## Registry Index

| File | Family | Constraint | Verification Status | Confidence | Priority |
|------|--------|------------|---------------------|------------|----------|
| [`A.toml`](A.toml) | Augerat A | ATMOST | Hypothesis | High | Low |
| [`B.toml`](B.toml) | Augerat B | ATMOST | Hypothesis | High | Low |
| [`CMT.toml`](CMT.toml) | Christofides-Mingozzi-Toth | Unspecified | Unspecified | Low | **Critical** |
| [`E.toml`](E.toml) | Augerat E | ATMOST | Hypothesis | High | Low |
| [`F.toml`](F.toml) | Fisher | ATMOST | Hypothesis | High | Low |
| [`Golden.toml`](Golden.toml) | Golden et al. 1998 | ATMOST | Hypothesis | Medium | None (out of scope) |
| [`Li.toml`](Li.toml) | Li et al. 2005 | Unknown | Unknown | None | None (out of scope) |
| [`M.toml`](M.toml) | Christofides M (large) | Unspecified | Unspecified | Low | **High** |
| [`P.toml`](P.toml) | Augerat P | ATMOST | Hypothesis | High | Medium (1 suspect instance) |
| [`Tai.toml`](Tai.toml) | Taillard 1993 | ATMOST | Hypothesis | Medium | **High** |
| [`X.toml`](X.toml) | Uchoa et al. 2017 | ATMOST | Hypothesis | High | Low |

---

## Open Items (P5)

P5 is the fleet semantics verification activity. The following families require
external confirmation before their fleet constraint assignments can be promoted
from Hypothesis to Verified.

### Critical Priority

**CMT** — 7 of 14 instances show negative gaps (up to −32.65%). Fleet semantics
are Unspecified. The 1979 paper may have used EXACT(K) semantics. Resolution
requires review of Christofides et al. 1979.

### High Priority

**M** — 2 of 2 instances show negative gaps (−2.09%, −2.99%). Same publication
as CMT — resolution is coupled.

**Tai** — 5 of 13 instances show negative gaps (up to −7.61%). The 1993 paper
predates widespread ATMOST convention adoption. Resolution requires review of
Taillard 1993.

### Medium Priority

**P** — 1 of 24 instances shows a negative gap (−2.04%, P-n55-k8). Family-level
ATMOST hypothesis is high-confidence; this is an isolated anomaly requiring
Stage B investigation.

### Low Priority (confirm but not blocking)

**A, B, E, F, X** — No negative gaps observed. ATMOST hypothesis is high-confidence.
Formal confirmation from original papers is good practice but not blocking.

---

## Relationship to Provenance Registry

Fleet semantics and provenance are complementary:

| Registry | Answers |
|----------|---------|
| `provenance/` | Where did the BKS come from? Is the distance metric correct? |
| `fleet_semantics/` | What does the vehicle count K mean? ATMOST or EXACT? |

Both must be resolved before a gap measurement can be reported as a qualified comparison.
A gap can be invalid due to BKS provenance issues (P4) OR fleet semantics issues (P5) OR both.

---

## Relationship to FCS (Fleet Constraint Semantics)

The FCS module in [`adapters/cvrp/src/bin/cvrplib_registry.rs`](../../adapters/cvrp/src/bin/cvrplib_registry.rs)
implements the runtime fleet constraint check. It uses the `FleetSemantics` enum
(`Minimum`, `Maximum`, `Exact`, `Unknown`) to classify each family.

This registry is the external-facing companion to FCS — it records the evidence
basis for each FCS assignment and tracks the verification status of each hypothesis.

When P5 verification is complete, the FCS assignments in `cvrplib_registry.rs`
should be updated to reflect the verified semantics, and the verification status
in this registry should be promoted from `Hypothesis` to `Verified`.

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| v1.0 | 2026-07-09 | Initial registry — 10 family files created as part of M18.2 |