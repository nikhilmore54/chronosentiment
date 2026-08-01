# Cleanup Register

**Document ID:** GOV-CLN-001
**Version:** 1.0
**Status:** Active
**Created:** 2026-08-01

---

## Purpose

Every consolidation, merge, archival, or deletion in the repository is recorded here with a unique identifier. This makes cleanup reproducible and reversible. No asset may be deleted or archived without a corresponding entry in this register.

**Rule:** Implement → Freeze → Inventory → Consolidate → Validate → Delete. Never delete while implementing.

---

## Register Format

Each entry contains:

- **ID** — unique cleanup identifier (CLN-NNN)
- **Type** — Archive | Delete | Merge | Rename | Consolidate
- **Asset(s)** — path(s) affected
- **Reason** — why this cleanup is needed
- **Canonical** — the asset that survives (if applicable)
- **Dependents** — assets that reference the affected asset and must be updated
- **Validation** — how to confirm the cleanup is safe
- **Status** — Pending | In Progress | Complete | Blocked
- **Date** — date completed (if applicable)

---

## Open Items

### CLN-001 — Archive superseded repository survey

| Field | Value |
|-------|-------|
| **ID** | CLN-001 |
| **Type** | Archive |
| **Asset** | `docs/REPOSITORY_SURVEY.md` |
| **Reason** | Superseded by `docs/governance/KNOWLEDGE_SURVEY.md` (GOV-KS-001), which covers all five knowledge systems rather than documents only |
| **Canonical** | `docs/governance/KNOWLEDGE_SURVEY.md` |
| **Dependents** | Any document that links to `docs/REPOSITORY_SURVEY.md` — search with `grep -r "REPOSITORY_SURVEY" docs/` |
| **Validation** | Confirm no active document links to `REPOSITORY_SURVEY.md`; confirm `KNOWLEDGE_SURVEY.md` covers all content |
| **Status** | Pending |
| **Date** | — |

---

### CLN-002 — Resolve RESEARCH_LINEAGE duplicate

| Field | Value |
|-------|-------|
| **ID** | CLN-002 |
| **Type** | Merge or Delete |
| **Asset A** | `docs/RESEARCH_LINEAGE.md` |
| **Asset B** | `docs/research/RESEARCH_LINEAGE.md` |
| **Reason** | Two files with identical names in different directories; likely duplicates |
| **Canonical** | TBD — read both files and compare content |
| **Dependents** | Any document linking to either path |
| **Validation** | Diff both files; if identical, delete one and update all references; if different, merge and note provenance |
| **Status** | Pending |
| **Date** | — |

---

### CLN-003 — Resolve ChronoSentiment Blueprint duplicate

| Field | Value |
|-------|-------|
| **ID** | CLN-003 |
| **Type** | Rename or Merge |
| **Asset A** | `docs/ChronoSentiment_Product_Blueprint_v1.md` |
| **Asset B** | `docs/ChronoSentiment_Personal_Blueprint_v1.md` |
| **Reason** | Overlapping names; unclear whether "Personal" vs "Product" is a meaningful distinction or a naming accident |
| **Canonical** | TBD — read both files and determine scope |
| **Dependents** | Any document linking to either path |
| **Validation** | If scopes are distinct, rename to make the distinction explicit; if overlapping, merge |
| **Status** | Pending |
| **Date** | — |

---

### CLN-004 — Resolve Codebase Assessment duplicate

| Field | Value |
|-------|-------|
| **ID** | CLN-004 |
| **Type** | Merge or Delete |
| **Asset A** | `docs/CODEBASE_ARCHITECTURE_ASSESSMENT.md` |
| **Asset B** | `docs/CODEBASE_ASSESSMENT.md` |
| **Reason** | Two files with similar names; likely one is a draft or earlier version of the other |
| **Canonical** | TBD — read both files and compare content |
| **Dependents** | Any document linking to either path |
| **Validation** | Diff both files; keep the more complete version; archive the other |
| **Status** | Pending |
| **Date** | — |

---

### CLN-005 — Resolve Milestone document duplicate

| Field | Value |
|-------|-------|
| **ID** | CLN-005 |
| **Type** | Merge or Delete |
| **Asset A** | `docs/EP-001_MILESTONE.md` |
| **Asset B** | `docs/P001_MILESTONE.md` |
| **Reason** | Two milestone documents; unclear if they cover the same milestone or different ones |
| **Canonical** | TBD — read both files and determine scope |
| **Dependents** | `docs/EP-002_ROADMAP.md` and any document referencing either milestone |
| **Validation** | If same milestone, merge; if different, rename to make the distinction explicit |
| **Status** | Pending |
| **Date** | — |

---

### CLN-006 — Survey docs/contracts/ vs docs/research/ contracts

| Field | Value |
|-------|-------|
| **ID** | CLN-006 |
| **Type** | Consolidate |
| **Asset A** | `docs/contracts/` (directory — not yet surveyed) |
| **Asset B** | 23 `*_CONTRACT_v1.md` files in `docs/research/` |
| **Reason** | Contracts may exist in both locations; canonical location is unclear |
| **Canonical** | TBD — survey `docs/contracts/` first |
| **Dependents** | Any document referencing contracts in either location |
| **Validation** | List all files in `docs/contracts/`; cross-reference with `docs/research/` contracts; identify duplicates |
| **Status** | Pending |
| **Date** | — |

---

## Completed Items

*No items completed yet.*

---

## Workstream Sequence

Per the reviewer's instruction, cleanup follows this sequence:

```
Implement
    ↓
Freeze
    ↓
Inventory  ← GOV-KS-001 (complete)
    ↓
Consolidate  ← this register governs consolidation
    ↓
Validate
    ↓
Delete
```

Workstreams execute in this order:

1. **WS1 — Duplicate Code Inventory** — identify duplicate helper functions, statistics calculations, CSV writers, logging, report generation, benchmark loaders, genome builders
2. **WS2 — Canonical Components** — for every duplicated capability, choose exactly one owner (harness modules are the canonical implementations)
3. **WS3 — Remove Dead Code** — only after WS1 and WS2; every deletion justified by the inventory
4. **WS4 — Module Boundaries** — verify harness/optimizer/research/GERAD responsibilities are clean
5. **WS5 — Public API Freeze** — freeze harness, statistics, reporting, and persistence APIs after cleanup

---

## Maintenance Protocol

1. Before deleting or archiving any asset, create an entry in this register.
2. Set status to Pending; get reviewer approval before proceeding.
3. After completing the cleanup, update status to Complete and record the date.
4. If a cleanup is blocked (e.g. a dependent cannot be updated), set status to Blocked and explain why.
5. Never delete an asset that is referenced by a frozen document without explicit reviewer approval.

---

*Last updated: 2026-08-01 | Maintained by: Repository Governance*