# Lane 4 — Experimental Consumer Labeling Audit Scope

**Lane:** 4 — authority boundary containment (observational)  
**Status:** active scope declaration  
**Companion:** `docs/RESEARCH_LINEAGE.md`  
**Authority:** `AUTHORITY_MAP.md` calibration layer; governance rule 5 (research preserves lineage without granting authority)

---

## Purpose

Detect research and experimental surfaces that duplicate or consume **CRITICAL** semantics — without fixing, consolidating, or canonicalizing them.

```text
documented ≠ operational authority
research preserved
authority bounded
```

---

## Admissibility (four-question filter)

| Filter | Posture |
|--------|---------|
| Who defines meaning? | existing authority map + Rust core only |
| What may mutate? | lineage doc + audit fixture only |
| Replay-stable? | all replay-bearing surfaces untouched |
| Escalation mandatory if… | audit implies convergence, canonical replacement, or remediation plans |

**Verdict:** admissible as containment hygiene during calibration pause.

---

## Operational shape

```text
1 lineage doc           docs/RESEARCH_LINEAGE.md
1 scope note            (this document)
1 audit script          scripts/lane4_experimental_consumer_labeling_audit.py
1 fixture               fixtures/lane4/experimental_consumer_labeling_audit.json
1 CI job                lane4-experimental-consumer-labeling (--check-reference)
0 convergence work
0 research relocation (unless separately authorized)
```

---

## Classification taxonomy (observational)

| Label | Meaning |
|-------|---------|
| `detached_archive` | under `/research_experiments/` — not in module tree |
| `semantic_duplicate` | duplicates CRITICAL formula or interpretation locally |
| `downstream_consumer` | consumes GA/replay outputs without authority claim |
| `observational_consumer` | read-only export/plot/verify — no semantic redefinition |
| `orphaned_lineage` | references former core paths or deprecated authority stubs |

---

## Explicit non-goals

No:

* prescribing convergence,
* assigning canonical replacements,
* auto-rewriting references,
* remediation plans,
* moving scripts to `/research_experiments/` (separate Phase 4 item),
* replay-law changes,
* semantic promotion of research artifacts.

If audit work requires “recommended fix” sections → **stop** — that is remediation, not labeling.

---

## Success criteria

Audit complete when the repo can state:

* which experimental surfaces exist,
* which label each carries,
* which duplicate CRITICAL semantics (if any),
* which are observational-only,
* which archived artifacts remain detached,

**without** implying consolidation order or operational obligation.

---

## Non-claims

This unit does **not**:

* authorize research script relocation,
* ratify research for operational use,
* alter replay meaning,
* merge Lane 4 into Lane 2 or Lane 3.
