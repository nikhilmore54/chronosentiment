# SEMANTIC_LINT_POSTURE.md
# ChronoSentiment — Semantic Lint Governance Posture
# Last updated: 2026-05-28
# Purpose: Define the constitutional boundary between observational governance tooling,
#          terminology compression, and human interpretive authority.
#          Prevents tool-mediated semantic authority from accumulating silently.

---

## POSTURE DECLARATION

Semantic lint in this repository is **observational governance only**.

It surfaces drift. It does not adjudicate meaning.

```text
tooling surfaces drift
humans adjudicate meaning
replay law remains authoritative
CI does not silently canonize interpretation
```

This is not a temporary posture. It is a constitutional property of the repo.

---

## THREE-LAYER BOUNDARY

| Layer              | Role                        | May                                              | May Not                                                    |
|--------------------|-----------------------------|--------------------------------------------------|------------------------------------------------------------|
| **semantic lint**  | observational governance    | detect, flag, measure, compare, surface warnings | infer ontology; canonize interpretation; hard-fail on meaning |
| **glossary**       | terminology compression     | reduce friction; compress repeated terms         | define canonical meaning; expand authority; grow unboundedly |
| **humans/reviewers** | interpretive authority    | adjudicate meaning; ratify semantic expansion    | delegate interpretive authority to tooling or CI           |

This boundary is aligned with the **Observational authority class** in [`AUTHORITY_MAP.md`](AUTHORITY_MAP.md):

> Observational: detect, measure, replay, compare, attest — NOT infer ontology; silent canonization.

---

## WHAT SEMANTIC LINT MAY DO

- Flag terminology inconsistencies against the declared glossary
- Surface repeated use of undefined or ambiguous terms
- Warn when a term appears in a new context without a declared mapping
- Measure glossary coverage across documentation surfaces
- Report drift between current usage and last-ratified terminology state

## WHAT SEMANTIC LINT MAY NOT DO

- Hard-fail CI on semantic grounds without explicit human ratification
- Treat warning suppression as implicit canonization
- Accumulate glossary entries without human review
- Infer that repeated usage constitutes canonical meaning
- Acquire interpretive authority over replay semantics, strategy identity, or chronology law

**Rule:** `Repeated observation ≠ canonical meaning` (per `AUTHORITY_MAP.md` semantic escalation protocol).

---

## GLOSSARY POSTURE

The glossary is **compression tooling** — not a semantic authority surface.

| Allowed                                      | Forbidden                                              |
|----------------------------------------------|--------------------------------------------------------|
| Compress repeated terms into stable shorthand | Define replay semantics or strategy identity           |
| Reduce contributor friction                  | Grow unboundedly without ratification pressure review  |
| Reference authoritative surfaces             | Become an authoritative surface itself                 |
| Flag undefined terms for human review        | Resolve undefined terms autonomously                   |

**Glossary growth is a warning signal.** Excessive additions indicate either:
- terminology instability (contributors gaming wording), or
- scope creep pressure (new concepts entering without governance escalation).

Both require human review — not automated resolution.

---

## CI POSTURE

Semantic CI jobs are **observational-only** in the current equilibrium validation phase.

| CI behavior          | Allowed | Forbidden                                      |
|----------------------|---------|------------------------------------------------|
| Emit warnings        | yes     | —                                              |
| Surface drift counts | yes     | —                                              |
| Hard-fail on meaning | **no**  | CI must not block merge on semantic grounds alone |
| Auto-update glossary | **no**  | Glossary mutations require human commit        |
| Canonize by silence  | **no**  | No implicit ratification through CI pass       |

**Escalation condition:** if semantic CI warnings are consistently suppressed or ignored, that is a governance signal — not a CI configuration problem. Escalate to human review before adjusting thresholds.

---

## OBSERVATIONAL SOAK PROTOCOL

After the governance layer is merged, observe for several days before any enforcement changes:

| Signal                          | Healthy                              | Warning                                              |
|---------------------------------|--------------------------------------|------------------------------------------------------|
| Lint warning volume             | Low, stable, actionable              | High noise → contributors ignoring output            |
| Glossary addition rate          | Slow, deliberate                     | Rapid growth → scope pressure or gaming              |
| Contributor friction            | Minimal, ergonomic                   | Complaints about CI → tooling-led interpretation risk |
| CI ergonomics                   | Warnings visible, non-blocking       | Suppression patterns → silent canonization risk      |
| Replay cadence stability        | Unchanged from pre-governance state  | Degradation → governance overhead too high           |
| Interpretive disputes           | Resolved by humans, not tooling      | Tooling cited as authority → escalate immediately    |

**Do not adjust enforcement posture during the soak period.** Observation only.

---

## ESCALATION CONDITIONS

The following conditions require explicit human ratification before any posture change:

| Condition                                                        | Required action                                      |
|------------------------------------------------------------------|------------------------------------------------------|
| Semantic lint warning proposed for hard-fail enforcement         | Human ratification + map update in `AUTHORITY_MAP.md` |
| Glossary entry proposed as canonical definition                  | Governance escalation per semantic escalation protocol |
| CI job proposed to block merge on semantic grounds               | Lane/tranche authorization required                  |
| Lint tooling proposed to infer ontology from usage patterns      | Reject — observational class violation               |
| Contributor optimizing for CI pass rather than clarity           | Human review — governance drift signal               |

---

## RELATIONSHIP TO AUTHORITY_MAP.md

This document is a **bounded governance posture declaration** — not an authority surface.

It does not modify, extend, or supersede any entry in [`AUTHORITY_MAP.md`](AUTHORITY_MAP.md).

It operationalizes the **Observational authority class** for the semantic lint layer specifically:

> Observational surfaces verify replay law; they do not redefine replay law.

The calibration-preservation doctrine in `AUTHORITY_MAP.md` remains the constitutional substrate.
Human interpretive primacy is preserved. Bounded human ratification governs all semantic expansion.

---

## CURRENT PHASE

**Equilibrium validation** — not expansion mode.

Allowed: CI observation, replay cadence checks, terminology stabilization, documentation clarity,
onboarding improvements, governance ergonomics.

Avoid: orchestration evolution, replay semantic mutation, topology extraction, new authority surfaces,
runtime behavioral changes, hard-fail enforcement on semantic lint.

Reassess before any enforcement escalation.