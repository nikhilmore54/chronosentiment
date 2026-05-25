# Phase 2 Decision Protocol
## Controlled Authority Reduction — Governance Rules

**Baseline anchor:** commit `881f4141`, tag `replay-governance-baseline-v1`  
**Governing discipline:** semantic clarity over maximal reduction  
**Phase 2 entry condition:** [`docs/PHASE1_GOVERNANCE_CHECKPOINT.md`](PHASE1_GOVERNANCE_CHECKPOINT.md) exists and `phase1-governance-visibility-restored` tag is applied

---

## Core Rule

Every Phase 2 authority reduction must be explainable relative to the sealed baseline
(`881f4141` / `replay-governance-baseline-v1`) across five mandatory axes:

1. **What authority signal was false or misleading** — why this surface emits a phantom or
   misleading authority signal relative to the baseline
2. **What canonical authority remains after reduction** — where the authoritative semantic
   surface lives after the change
3. **Why replay behavior is unchanged** — explicit justification that replay semantics are
   invariant across the reduction
4. **Whether lineage preservation is required** — whether the surface carries historical
   semantic lineage that must be archived rather than deleted
5. **Which baseline was used for comparison** — always `881f4141` unless a subsequent
   governance checkpoint supersedes it

If any axis cannot be answered, the surface is **not ready for Phase 2 reduction**. Classify
it using the four-category framework below and defer.

---

## Four-Category Classification Framework

| Category | Definition | Action |
|----------|------------|--------|
| Dormant but semantically valid | Surface is unused but carries legitimate deferred intent, lineage, or wiring anchor | annotate + preserve |
| Transitional duplicate | Surface duplicates canonical authority during a migration; the canonical form exists | consolidate toward canonical |
| Phantom authority | Surface emits an authority signal (module, re-export, stub, file name) that does not correspond to real semantic authority | archive / remove |
| Experimental lineage | Surface represents a completed or abandoned experiment whose lineage must be traceable | quarantine with justification |

The question is no longer "is this code dead?" It is:

> **What authority signal does this surface emit relative to the sealed baseline?**

---

## CRITICAL / TRANSITIONAL Surface Constraint

For any surface classified as CRITICAL or TRANSITIONAL in `AUTHORITY_MAP.md`:

**No authority reduction without replay comparison evidence.**

Even when the expectation is "no replay impact," that expectation must be validated, not assumed.
Acceptable validation forms:

- `cargo check` exit 0 after reduction
- Replay hash comparison against `replay-governance-baseline-v1`
- Fixture comparison (before/after output equivalence)
- Explicit Rule 8 compliance declaration

This operationalizes Rule 8 of `GOVERNANCE.md` at the Phase 2 action level.

---

## Phase 2 Decision Template

Every authority reduction action must complete this template before the change is committed.
File the completed template in `archive_justifications/` or inline in the commit message.

```
## Phase 2 Authority Reduction Decision

**Surface:**
  [file path, module name, struct/fn/const identifier]

**Classification:**
  [ ] Phantom authority
  [ ] Transitional duplicate
  [ ] Dormant but semantically valid
  [ ] Experimental lineage

**Baseline anchor:**
  881f4141 / replay-governance-baseline-v1

**False or misleading signal:**
  [What authority signal did this surface emit that was incorrect or misleading?]

**Canonical authority retained:**
  [What remains authoritative after this reduction, and where does it live?]

**Replay impact expectation:**
  [ ] None — no replay-sensitive paths affected
  [ ] Expected — replay behavior changes in the following documented way: ___
  [ ] Unknown — requires validation before commit

**Validation performed:**
  [ ] cargo check exit 0
  [ ] Replay hash comparison
  [ ] Fixture comparison
  [ ] Rule 8 compliance declaration
  [ ] Other: ___

**Lineage disposition:**
  [ ] Preserved (annotated in place)
  [ ] Archived (moved to archive_justifications/)
  [ ] Deleted (no lineage value; justification: ___)

**Governance rule references:**
  [Which GOVERNANCE.md rules justify this action?]
```

---

## What This Protocol Prevents

Without the baseline-anchor rule, "authority reduction" can silently become:

- subjective cleanup,
- aesthetic simplification,
- undocumented semantic removal,
- replay-risk drift.

With it, every Phase 2 action is auditable against a known governance state. The protocol
transforms consolidation from cleanup intuition into evidence-based governance.

---

## Highest-Value Phase 2 Targets

Phantom authority signals are the primary Phase 2 target because they distort future reasoning
by humans, auditors, contributors, and agents. Known candidates (from `AUTHORITY_MAP.md`):

| Surface | Location | Signal Type |
|---------|----------|-------------|
