# V-006 Manifest ms Assumption Probe — Scope

**Lane:** 2 — prerequisite clarification (observational)  
**Status:** active scope declaration  
**Parent unit:** `V006_MANIFEST_MS_IMPACT_INVENTORY_SCOPE.md` — inventory mapped exposure; probe detects implicit semantic assumptions  
**Authority:** `AUTHORITY_MAP.md` calibration layer; `V006_MANIFEST_DIALECT_POLICY.md` timestamp doctrine

---

## Purpose

Answer:

```text
which consumers implicitly assume millisecond semantics?
```

without correcting manifests, rewriting chronology, aligning producers, extending schema law, or reopening the manifest ms correction tranche.

---

## Admissibility (four-question filter)

| Filter | Posture |
|--------|---------|
| Who defines meaning? | existing V-006 law only — probe does not alter dialect doctrine |
| What may mutate? | observational probe fixture + scope note only |
| What must remain replay-stable? | chronology bytes + existing manifest semantics on disk |
| What escalation becomes mandatory? | hidden ms normalization pressure, rewrite tooling, or producer convergence discovered during probe |

**Verdict:** admissible as observational prerequisite clarification.

---

## Operational shape

```text
1 scope note          (this document)
1 probe artifact      fixtures/lane2/manifest_ms_assumption_probe.json
1 execution probe     scripts/lane2_manifest_ms_assumption_probe.py
1 CI job              lane2-manifest-ms-assumption-probe (--check-reference)
0 migrations
0 producer edits
```

---

## Explicit non-goals

```text
no manifest rewrites
no producer edits
no schema normalization
no replay-law reinterpretation
no migration sequencing
no tranche preparation tooling
```

If probe work requires normalization helpers, compatibility layers, repair utilities, or manifest rewriting → **stop immediately**.

---

## Classification taxonomy (observational)

| Class | Meaning |
|-------|---------|
| `IMPLICIT_MS_SEMANTICS` | naming or pass-through asserts ms without dialect gate |
| `IMPLICIT_MS_ARITHMETIC` | unconditional `/ 1000` or ms-only window math on manifest bounds |
| `HEURISTIC_UNIT_AWARE` | threshold heuristic distinguishes seconds vs ms |
| `EXPLICIT_MS_LAW_CHECK` | explicit ms-law or dialect classification check |
| `PRODUCER_SECONDS_EMISSION` | producer emits seconds bounds (CD-2 class; not consumer assumption) |
| `NO_BOUNDS_CONSUMPTION` | does not interpret manifest bound units |

---

## Success criteria

Probe complete when the repo can state:

- which bounded consumer surfaces assume ms semantics,
- which use heuristic unit detection,
- which explicitly gate on dialect/ms law,
- which would mislead on Dialect B seconds bounds,
- what escalation triggers apply,

**without** implying migration order, rollout strategy, or correction inevitability.

---

## Escalation topology

| If probe reveals… | Action |
|-------------------|--------|
| new consumer silently normalizing bounds to ms | consumer inventory fix or dialect gate — stop probe expansion |
| rewrite/repair utility pressure | Lane 2 tranche scope doc required |
| schema or producer convergence pressure | stop probe — constitutional territory |

---

## Non-claims

This probe does **not**:

- authorize manifest ms correction tranche,
- ratify producers,
- alter replay meaning,
- define migration topology,
- imply that correction is operationally expected.
