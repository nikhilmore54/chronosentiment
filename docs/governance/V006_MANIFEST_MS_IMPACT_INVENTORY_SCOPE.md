# V-006 Manifest ms Correction — Impact Inventory Scope

**Lane:** 2 — prerequisite clarification (observational)  
**Status:** active scope declaration  
**Authority:** `AUTHORITY_MAP.md` calibration layer; `V006_MANIFEST_DIALECT_POLICY.md` timestamp doctrine  
**Burden:** reduce future constitutional uncertainty **without** changing present replay law

---

## Purpose

Produce a bounded, evidence-only inventory answering:

- where seconds/ms ambiguity exists in manifest bounds,
- which persisted manifests are affected,
- which tools observe or assume bound units,
- which byte fixtures would constrain a future migration tranche,
- what conditions would trigger Lane 2 escalation.

This unit **maps exposure**. It does **not** design migration architecture.

---

## Admissibility (four-question filter)

| Filter | Outcome |
|--------|---------|
| Who defines meaning? | unchanged — inventory does not alter dialect law |
| What may mutate? | inventory artifacts + scope note only |
| What remains replay-stable? | chronology JSONL bytes untouched |
| What escalation becomes mandatory? | any schema/producer convergence pressure discovered during inventory |

---

## Explicit non-goals

No:

- migration implementation,
- schema normalization,
- producer alignment,
- auto-conversion logic,
- compatibility rewrite,
- shared serialization abstractions,
- tranche bundling,
- manifest rewriting on disk,
- `"while inventorying we also fixed..."` drift.

Especially no:

- `capture_types.rs`,
- dialect collapsing,
- batch seconds→ms rewrite without authorized tranche scope doc.

If work requires adapters, normalization helpers, repair utilities, or manifest rewriting → **stop immediately** (constitutional replay-law territory).

---

## Operational shape

```text
1 scope note          (this document)
1 inventory artifact  fixtures/lane2/manifest_ms_impact_inventory.json
1 verifier/probe      scripts/lane2_manifest_ms_impact_inventory.py
1 CI job              lane2-manifest-ms-inventory (--check-reference)
0 migrations
0 producer edits
```

---

## Success criteria

Inventory complete when the repo can state exactly:

- where seconds/ms ambiguity exists,
- which manifests are affected (by dialect and bounds unit),
- which replay/cert surfaces observe manifest bounds,
- which fixtures constrain future migration,
- what triggers Lane 2 escalation,

**without** proposing migration, unifying producers, changing schemas, or redefining replay meaning.

---

## Known defect (pre-declared)

Per `V006_MANIFEST_DIALECT_POLICY.md` and `V006_LIVE_CAPTURE_AUTHORITY_DECISION.md`:

- **Forward law:** manifest `capture_start` / `capture_end` must be **milliseconds**.
- **Historical defect (Dialect B):** live rotation manifests store bounds in **seconds** while tick JSONL uses ms.
- **Correction class:** manifest metadata migration — JSONL unchanged → `chronology_hash` unchanged; catalog window interpretation changes.

---

## Escalation topology (inventory output only)

| Trigger | Escalation |
|---------|------------|
| Batch manifest bounds rewrite | Lane 2 tranche scope doc + `AUTHORITY_MAP` update |
| `capture_daemon` producer alignment | producer ratification + replay-scope declaration |
| Tool assumes ms on Dialect B without classification | consumer inventory fix or explicit dialect gate |
| Schema field semantic weight added | Lane 2 or map update — stop inventory expansion |

---

## Non-claims

This scope does **not**:

- authorize manifest ms correction tranche,
- reopen V-006 migration,
- ratify any producer,
- alter Tier 1 or Lane 3 operational surfaces,
- define replay meaning beyond existing V-006 law.
