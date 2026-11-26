# Glossary – Constitutional Terminology

> **Purpose** – This is the single source of truth for all architectural terms used in ChronoSentiment. Any PR that introduces a term not listed will be rejected by CI (initially in warning mode). 

## Canonical Terms Table

| Term | Definition (canonical) | Allowed Synonyms / Aliases |
|------|------------------------|----------------------------|
| **authority** | The bounded domain of control defined in `AUTHORITY_MAP.md`. It delineates which components may modify which pieces of state and enforces the “no‑leakage” rule between layers. | **(no synonyms)** – any synonym must be explicitly mapped here. |
| **certification** | A formally verified property of a component (e.g., replay certification, sweep certification) that guarantees deterministic behavior under the defined topology. Certified artifacts are tagged and immutable. | **verification**, **validation** (only when referring to a formal certification process). |
| **replay** | The deterministic reconstruction of a complete market timeline given the same seed, inputs, and execution order. Replay produces an immutable hash that must remain unchanged across runs. | **deterministic replay**, **replay hash**. |
| **orchestration** | The coordination layer that schedules and binds together certified components (e.g., reporting, sweep scheduling). It is the highest‑risk surface because it interacts with replay and certification boundaries. | **coordination**, **scheduler** (only when the component truly manages execution order). |
| **topology** | The acyclic, four‑layer architecture (optimization → chronology → strategy → observatory) that defines permissible dependency directions. Changes to topology require full replay recertification and a new tag. | **architecture**, **layering** (when describing the canonical four‑layer structure). |
| **semantic** | The meaning attached to data and operations that must remain stable across replay. Semantic drift occurs when the same term acquires multiple meanings. | **meaning**, **interpretation** (when describing the property, not as a code identifier). |
| **deterministic** | Guarantees that given identical inputs and the same execution order, the output (including replay hash) is identical. Determinism is a prerequisite for all certifications. | **repeatable**, **predictable** (when describing the property, not as a code identifier). |
| **projection** | The canonical operation that maps a pure candidate (bytes from the optimizer) into a concrete strategy execution within the `financial/strategies` layer. It is the only approved term for this mapping; synonyms such as *snapshot* or *execution state* are prohibited unless formally added here. | **(no synonyms)** – use *projection* exclusively. |
| **snapshot** | **Prohibited** unless explicitly defined as a synonym of *projection* in this table. Use only when a formal, audited transformation is introduced. |
| **execution state** | **Prohibited** as a term for the outcome of a projection. Must be expressed as *projection* or *replay* depending on context. |

## Term Categories

| Category | Description | Example Terms |
|----------|-------------|---------------|
| **Constitutional** | Extremely stable terms that form the immutable surface of the system. Changes require replay recertification and topology review. | authority, replay, topology, projection, certification |
| **Operational** | Stable but implementation‑bound terms that may evolve with code changes, but do not affect constitutional invariants. | queue, scheduler, runtime, task, adapter |
| **Experimental** | Non‑authoritative terms used in temporary prototypes or research branches. They may evolve rapidly and are not required in the main‑line glossary. |

## Term Stability Levels

| Stability | Meaning |
|----------|---------|
| **Constitutional** | Extremely stable; any change triggers replay recertification, topology review, and a new tag. |
| **Operational** | Stable for the lifetime of a component; changes are allowed after reviewer acknowledgment and CI warning clearance. |
| **Experimental** | Fluid; may be added, renamed, or removed without formal review, but must never appear in constitutional documents. |

## Enforcement Guidelines (CI Semantic Linting)

1. **No undefined terms** – The CI workflow runs a script that extracts all words matching the *Term* column of this glossary. If a changed file introduces a term not present, the job emits a **warning** and requires a reviewer to acknowledge the issue in the PR description. 
2. **Glossary updates required** – When a PR adds a new constitutional term, the contributor must also add a row to this table with a clear definition. 
3. **Synonym mapping** – Any allowed synonym used in code or docs must be listed in the *Allowed Synonyms / Aliases* column. 
4. **Prohibited terms** – Use of `snapshot` or `execution state` outside an explicit glossary mapping fails the CI check (warning mode initially). 
5. **Gradual hardening** – After the stabilization window, the CI job will be switched from *warning* to *hard‑fail* to enforce strict compliance.

---

*Keep this file up‑to‑date — it is the single source of truth for semantic governance.*
