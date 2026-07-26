# Coralys — Platform Architecture
## The Knowledge Evolution Platform

**Document type:** Platform Architecture
**Version:** 1.5
**Status:** Baseline
**Date:** 2026-07-26
**Owner:** Platform / Engineering

---

## Document Governance

| Field | Value |
|-------|-------|
| Document Status | Baseline v1.5 |
| Review Trigger | Material change in platform capability; new domain adapter added; Coralys v2 architecture baseline |

**Relationship to other documents:**
- Realised by: ChronoSentiment Personal Blueprint v1.1 (investment research adapter)
- Realised by: ChronoSentiment Product Blueprint v1.0 (enterprise decision management adapter)
- Realised by: UltraCrew (workforce scheduling adapter)
- Informs: ChronoSentiment Product Strategy v1.2 (Section 9 — platform hierarchy)
- Complementary to: `ARCHITECTURE_EVOLUTION.md` (engineering constitution — crate structure, dependency rules, planning capability, invariants). This document operates at the product/strategy layer; `ARCHITECTURE_EVOLUTION.md` operates at the engineering/implementation layer. Neither supersedes the other.
- Complementary to: `PLATFORM_CRATE_RESPONSIBILITIES.md` (engineering — per-crate responsibility boundaries)
- Complementary to: `DOMAIN_EXTENSION_GUIDE.md` (engineering — how to add a new domain adapter)

---

## Platform Identity

Coralys is a **Knowledge Evolution Platform**.

It is not an investment platform. It is not a scheduling platform. It is not a research platform. It is a domain-neutral platform that captures, governs, evolves, and reuses structured knowledge — across any domain where hypotheses are formed, tested against evidence, and refined through experience.

Domain adapters supply the domain vocabulary. Coralys supplies the lifecycle, governance, and knowledge evolution capabilities.

**Platform statement:**

> Coralys manages the lifecycle of structured knowledge: capturing it in Workspaces, evolving it through evidence and review, and making it reusable through pattern extraction and the Knowledge Graph.

**Three-part model:**

> The Platform governs the lifecycle.
> The Continuous Learning Engine drives improvement.
> The Knowledge Graph preserves memory.

**Platform layering:**

```
Knowledge Evolution Platform
    owns
        Lifecycle
        Platform Primitives
        Knowledge Graph
        Domain Adapter Model
    powered by
        Continuous Learning Engine
```

**The Continuous Learning Engine** is Coralys' core computational capability — a subsystem of the platform, not the platform itself. It is the mechanism by which knowledge evolves: driving the lifecycle from hypothesis creation through evidence gathering, execution, review, outcome recording, and pattern extraction. The engine enables knowledge to improve over time.

The distinction matters: if you remove the Continuous Learning Engine, the platform loses its ability to improve. If you remove Workspaces, Evidence, Hypotheses, Reviews, and Outcomes, the platform ceases to exist. The platform is the lifecycle. The engine is what makes that lifecycle productive.

**Canonical platform diagram:**

```
                    Applications
    ┌──────────────────────────────────────────┐
    │  UltraCrew  │  ChronoSentiment  │ Future │
    └──────────────────────────────────────────┘
                          ▲
                          │
                   Domain Adapters
                   (vocabulary only)
                          ▲
                          │
    ┌──────────────────────────────────────────┐
    │      Coralys Knowledge Evolution         │
    │              Platform                    │
    │                                          │
    │  Workspace · Actor · Intent              │
    │  Subject · Context · Evidence            │
    │  Hypothesis · Review · Timeline          │
    │  Outcome · Learning · Pattern            │
    │  Knowledge Graph                         │
    └──────────────────────────────────────────┘
                          ▲
                          │
            Continuous Learning Engine
            (core computational mechanism)
```

---

## Architectural Principle

> **Coralys owns the lifecycle. Adapters own the vocabulary. Neither owns the other.**

This principle has three implications:

**1. Coralys never becomes domain-specific.** No investment terminology, scheduling terminology, or clinical terminology enters the Coralys core. The core operates on Workspaces, Actors, Evidence, Hypotheses, Reviews, Outcomes, Patterns, and Learning — always.

**2. Adapters never re-implement the lifecycle.** No adapter builds its own evidence management, hypothesis versioning, review orchestration, or pattern extraction. These capabilities are provided by Coralys. The adapter configures them; it does not replace them.

**3. The Knowledge Graph is shared infrastructure.** Within a domain, the Knowledge Graph accumulates across all Workspaces. Cross-domain knowledge propagation is a future capability — not in scope for the current platform, but architecturally possible because the Knowledge Graph is a platform primitive, not an adapter-specific feature.

---

## The Continuous Learning Engine

The Continuous Learning Engine is the core computational subsystem of the Coralys platform — not an emergent pattern across Workspaces, Reviews, Timelines, and Outcomes, but a designed capability that owns the complete knowledge lifecycle.

### The lifecycle

```
Create
  A new Workspace is opened. A subject is identified.
  The work begins.
        ↓
Gather Evidence
  Evidence is accumulated in the Workspace.
  Sources are attached. Observations are recorded.
        ↓
Form Hypothesis
  A structured hypothesis is articulated.
  Assumptions are documented. Risks are identified.
  The hypothesis is versioned.
        ↓
Execute
  A decision is made. An action is taken.
  The execution is linked to the hypothesis.
        ↓
Observe
  The world responds. Outcomes begin to emerge.
  Observations are recorded against the hypothesis.
        ↓
Review
  A structured review is conducted.
  The hypothesis is compared against observations.
  Assumptions are evaluated. The hypothesis is revised.
        ↓
Learn
  The outcome is recorded.
  Lessons are captured.
  The hypothesis is closed or continued.
        ↓
Generalise
  Patterns are extracted across multiple completed cycles.
  Cross-workspace knowledge is identified.
  The Knowledge Graph is updated.
        ↓
Reuse
  Future Workspaces benefit from accumulated patterns.
  The Knowledge Graph informs new hypotheses.
  The cycle begins again.
```

This lifecycle is completely domain-neutral. The Continuous Learning Engine does not know what a "stock" is, what a "roster" is, or what a "clinical trial" is. It knows what a Workspace, Evidence, Hypothesis, Review, Outcome, Pattern, and Learning are.

Domain adapters supply the rest.

---

## Core Platform Primitives

### Workspace

The persistent environment where a unit of work lives throughout its complete lifecycle. A Workspace accumulates evidence, hosts hypothesis versioning, records reviews, links to execution, and produces an Outcome record.

A Workspace is not a document. It is not a form. It is a structured environment that persists from the creation of a hypothesis through its completion and archival.

**Workspace as transaction boundary.** Everything that happens in a Coralys lifecycle happens inside exactly one Workspace. Evidence, Reviews, Timeline entries, Pattern contributions, and the Outcome all belong to exactly one Workspace. This boundary is the unit of provenance, the unit of archival, and the unit of access control. A Workspace is never shared between two independent units of work — if two Actors are working on related but distinct problems, they have distinct Workspaces that may be linked in the Knowledge Graph.

### Actor

The person, team, or system that owns a Hypothesis within a Workspace. The Actor is responsible for forming the Hypothesis, gathering Evidence, conducting Reviews, and recording the Outcome.

Actor is a platform primitive because ownership matters for provenance, permissions, collaboration, and accountability. Without Actor, there is no answer to "who believed this?" or "who is responsible for this decision?"

Domain adapter examples:

| Domain | Actor |
|--------|-------|
| ChronoSentiment Personal | Individual investor |
| ChronoSentiment Enterprise | Portfolio manager / Investment committee |
| UltraCrew | Scheduler / Operations manager |
| Medical research | Clinician / Research team |
| Engineering | Design engineer / Review board |

### Intent

The reason a Workspace was opened. Intent captures why the work exists — the question being investigated, the problem being solved, or the decision being prepared.

Intent is a platform primitive because it enables search, analytics, and knowledge reuse. A Workspace without Intent is an archive entry. A Workspace with Intent is a retrievable unit of reasoning that can be surfaced when a future Actor faces a similar question.

Domain adapter examples:

| Domain | Intent |
|--------|--------|
| ChronoSentiment Personal | "Research potential investment in Reliance Industries" |
| ChronoSentiment Enterprise | "Evaluate new position in HDFC Bank for the India Growth Fund" |
| UltraCrew | "Generate crew roster for July 2026 — SunAir base BOM" |
| Medical research | "Evaluate treatment protocol for cohort A" |
| Engineering | "Validate thermal performance of component X under load" |

### Subject

The entity that a Workspace is about. In ChronoSentiment Personal, the Subject is a Company. In ChronoSentiment Enterprise, the Subject is an Investment Opportunity. In UltraCrew, the Subject is a Scheduling Period. In medical research, the Subject is a Patient or Trial.

### Context

The environment in which the Subject exists. In ChronoSentiment Personal, the Context is the investor's Portfolio. In UltraCrew, the Context is the operational environment (airport, route network, crew base). In medical research, the Context is the patient population or clinical setting.

### Evidence

Structured records of information that inform a hypothesis. Evidence is attached to a Workspace with a timestamp and a source record. Evidence is immutable once recorded — it cannot be revised to match a later hypothesis.

### Hypothesis

A structured, versioned statement of what the Actor believes to be true and why. A Hypothesis has assumptions, risks, and an articulated reasoning chain. It is versioned — each revision is timestamped and linked to the evidence that caused it.

### Review

A structured, periodic evaluation of a Hypothesis against accumulated Evidence. A Review produces a Review Record — a timestamped snapshot of the Hypothesis state at that point in time. The Timeline is built from Review Records.

### Timeline

The complete history of how a Hypothesis evolved — every revision, every Review, every new piece of Evidence, every assumption update. The Timeline is the Workspace's commit history.

### Outcome

The recorded result of a completed Hypothesis cycle. An Outcome captures what actually happened, compared against what the Hypothesis predicted. Outcomes are the raw material for Pattern Extraction.

### Pattern

A generalisation extracted across multiple Outcomes. Patterns are cross-Workspace — they represent knowledge that has been validated across multiple hypothesis cycles and is available to inform future Workspaces.

### Learning

The computational process by which Outcomes are analysed, Patterns are extracted, and the results are made available to future Workspaces. Learning computes — it does not store. The Knowledge Graph stores the results of Learning.

Learning is not a passive archive. It is an active capability that surfaces relevant past experience when a new Workspace is created or a new Hypothesis is formed.

### Knowledge Graph

The network of linked Subjects, Actors, Hypotheses, Evidence, Intents, and Patterns that accumulates over time. The Knowledge Graph is the platform's long-term structural memory — the store of what Learning has computed across all Workspaces.

**Responsibility boundary:** Learning computes. The Knowledge Graph stores. These are distinct responsibilities. Learning analyses Outcomes and extracts Patterns; the Knowledge Graph persists those Patterns and makes them queryable. Neither owns the other's responsibility.

### Pattern

A generalisation extracted by Learning across multiple Outcomes. Patterns are cross-Workspace — they represent knowledge that has been validated across multiple hypothesis cycles and is available to inform future Workspaces.

**Pattern Extraction as a future engine.** Pattern extraction will eventually require sophisticated capabilities: clustering, embeddings, similarity search, graph mining, behavioural sequence mining. These capabilities are not in scope for the current platform. They are noted here as a future first-class engine — Pattern Extraction — that will sit between Learning and the Knowledge Graph, owned by the platform, not by any adapter.

### Question (candidate future primitive)

Evidence exists to answer questions. Hypotheses answer questions. Reviews generate questions. Research begins with questions.

Question is not yet a first-class platform primitive. It is noted here as a candidate for future promotion. As the platform matures, the ability to explicitly represent open questions — attach them to Workspaces, link them to Evidence, and surface them across the Knowledge Graph — may prove as important as Hypothesis itself.

---

## Domain Adapters

A domain adapter configures Coralys for a specific domain by supplying:

1. **Vocabulary mapping** — translating Coralys primitives into domain-specific terms
2. **Workflow configuration** — defining the specific sequence of steps within each lifecycle stage
3. **Evidence types** — defining what counts as evidence in this domain
4. **Review structure** — defining the questions asked at each Review
5. **Pattern types** — defining what kinds of patterns are meaningful in this domain
6. **Knowledge Graph schema** — defining how Subjects relate to each other in this domain

The adapter does not modify the Continuous Learning Engine. It configures it.

---

## Adapter Vocabulary Tables

### ChronoSentiment Personal (Investment Research Adapter)

| Coralys Primitive | ChronoSentiment Personal |
|------------------|--------------------------|
| Workspace | Research Workspace |
| Actor | Individual investor |
| Intent | Research objective (e.g. "Evaluate Reliance Industries as a long-term holding") |
| Subject | Company |
| Context | Portfolio |
| Evidence | Research Sources (annual reports, earnings calls, AI conversations, news) |
| Hypothesis | Investment Thesis |
| Hypothesis version | Thesis version (v1, v2, v3...) |
| Review | Quarterly Research Review |
| Timeline | Research Timeline |
| Outcome | Investment Outcome |
| Pattern | Investor Behaviour Pattern / Research Quality Score |
| Learning | Personal Investment Learning Loop |
| Knowledge Graph | Personal Investment Knowledge Graph |

**Continuous Learning Engine realization:** Personal Investment Learning Loop

---

### ChronoSentiment Enterprise (Institutional Decision Management Adapter)

| Coralys Primitive | ChronoSentiment Enterprise |
|------------------|---------------------------|
| Workspace | Decision Workspace |
| Actor | Portfolio manager / Investment committee |
| Intent | Investment evaluation objective (e.g. "Evaluate new position for India Growth Fund") |
| Subject | Investment opportunity |
| Context | Fund / mandate |
| Evidence | Decision Evidence (research, market data, AI conversations) |
| Hypothesis | Investment Thesis |
| Hypothesis version | Thesis version |
| Review | Committee Review |
| Timeline | Decision Timeline |
| Outcome | Decision Outcome |
| Pattern | Organisational Decision Pattern |
| Learning | Organisational Decision Learning Loop |
| Knowledge Graph | Institutional Decision Knowledge Graph |

**Continuous Learning Engine realization:** Organisational Decision Learning Loop

---

### UltraCrew (Workforce Scheduling Adapter)

| Coralys Primitive | UltraCrew |
|------------------|-----------|
| Workspace | Scheduling Workspace |
| Actor | Scheduler / Operations manager |
| Intent | Scheduling objective (e.g. "Generate crew roster — SunAir BOM base, July 2026") |
| Subject | Scheduling period / route / crew base |
| Context | Operational environment (airport, network, constraints) |
| Evidence | Operational data (disruptions, KPIs, crew availability, regulations) |
| Hypothesis | Roster Strategy |
| Hypothesis version | Roster version |
| Review | Schedule Review |
| Timeline | Scheduling Timeline |
| Outcome | Operational KPIs |
| Pattern | Workforce Behaviour Pattern |
| Learning | Workforce Operations Learning Loop |
| Knowledge Graph | Operational Knowledge Graph |

**Continuous Learning Engine realization:** Workforce Operations Learning Loop

---

### Medical Research Adapter (Illustrative)

| Coralys Primitive | Medical Research |
|------------------|-----------------|
| Workspace | Clinical Research Workspace |
| Subject | Patient / trial cohort |
| Context | Clinical setting / patient population |
| Evidence | Clinical data (lab results, imaging, patient history, literature) |
| Hypothesis | Clinical Hypothesis |
| Hypothesis version | Protocol version |
| Review | Clinical Review |
| Timeline | Clinical Timeline |
| Outcome | Patient Outcome |
| Pattern | Clinical Pattern |
| Learning | Clinical Learning Loop |
| Knowledge Graph | Clinical Knowledge Graph |

**Continuous Learning Engine realization:** Clinical Learning Loop

---

### Engineering Design Adapter (Illustrative)

| Coralys Primitive | Engineering Design |
|------------------|-------------------|
| Workspace | Design Workspace |
| Subject | Design problem / component / system |
| Context | Engineering constraints / requirements |
| Evidence | Design evidence (test results, simulations, specifications, failure reports) |
| Hypothesis | Design Hypothesis |
| Hypothesis version | Design revision |
| Review | Design Review |
| Timeline | Design Timeline |
| Outcome | Design Outcome (performance, failure, success) |
| Pattern | Engineering Pattern |
| Learning | Engineering Learning Loop |
| Knowledge Graph | Engineering Knowledge Graph |

**Continuous Learning Engine realization:** Engineering Learning Loop

---

## The Knowledge Graph as Platform Primitive

The Knowledge Graph is not a feature of any individual adapter. It is a first-class platform primitive that accumulates across all Workspaces within a domain.

In ChronoSentiment Personal, the Knowledge Graph links companies, industries, research dossiers, assumptions, and cross-company relationships. An insight about Jio's ARPU growth informs the research on Bharti Airtel. An understanding of 5G infrastructure informs the research on tower companies.

In UltraCrew, the Knowledge Graph links rosters, airports, disruption patterns, crew constraints, and operational policies. A disruption pattern at one airport informs scheduling decisions at connected airports.

In medical research, the Knowledge Graph links patients, treatments, outcomes, and clinical evidence. A treatment outcome for one patient cohort informs the hypothesis for a related cohort.

The Knowledge Graph is what transforms the Continuous Learning Engine from a record-keeping system into a genuine intelligence platform. It is the structural representation of what has been learned — not just stored, but connected, weighted, and made available to future work.

---

## Platform Applications

Coralys is not a Decision Intelligence Platform. Decision Intelligence is one application of the platform — the application realised by ChronoSentiment Enterprise.

Coralys is a **Knowledge Evolution Platform**: a domain-neutral platform that captures, governs, evolves, and reuses structured knowledge across any domain where hypotheses are formed, tested against evidence, and refined through experience.

**Platform applications:**

| Application | Domain | Continuous Learning Engine Realization |
|-------------|--------|---------------------------------------|
| ChronoSentiment Personal | Investment research | Personal Investment Learning Loop |
| ChronoSentiment Enterprise | Institutional decision management | Organisational Decision Learning Loop |
| UltraCrew | Workforce scheduling | Workforce Operations Learning Loop |
| Future: Medical | Clinical research | Clinical Learning Loop |
| Future: Engineering | Design and testing | Engineering Learning Loop |
| Future: Corporate strategy | Strategic planning | Strategic Learning Loop |
| Future: M&A | Deal evaluation | Deal Learning Loop |

The platform does not change between applications. The adapter supplies the vocabulary. The Continuous Learning Engine drives the lifecycle. The Knowledge Graph supplies the memory.

---

## Platform Primitive Summary

| Primitive | Role | Responsibility |
|-----------|------|---------------|
| Workspace | Transaction boundary | Contains all lifecycle activity for one unit of work |
| Actor | Ownership | Who owns the Hypothesis; provenance and accountability |
| Intent | Purpose | Why the Workspace was opened; enables search and reuse |
| Subject | Focus | What the Workspace is about |
| Context | Environment | The environment in which the Subject exists |
| Evidence | Information | Immutable records that inform the Hypothesis |
| Hypothesis | Belief | Versioned statement of what the Actor believes and why |
| Review | Evaluation | Structured periodic comparison of Hypothesis against Evidence |
| Timeline | History | Complete record of how the Hypothesis evolved |
| Outcome | Result | What actually happened; raw material for Learning |
| Learning | Computation | Analyses Outcomes; extracts Patterns; does not store |
| Pattern | Generalisation | Cross-Workspace knowledge extracted by Learning |
| Knowledge Graph | Memory | Stores Patterns and links; queryable by future Workspaces |
| Question *(future)* | Inquiry | Open questions attached to Workspaces and Evidence |
| Pattern Extraction Engine *(future)* | Computation | Clustering, embeddings, similarity, graph mining |

---

## Coralys v2 — Future Evolution

The following observations are recorded as candidates for Coralys v2. They are not gaps in v1.2 — v1.2 is the architectural baseline. They are directions the platform is likely to evolve as products mature and evidence accumulates.

**Observation 1 — Mental Model as a primitive.** Between Evidence and Hypothesis there may eventually be a third object: a worldview or mental model that shapes how an Actor interprets evidence. An investor's belief that "interest rates will remain elevated" is not evidence and not a hypothesis — it is a model. Models evolve over years. Their evolution is valuable. Candidate future lifecycle: Evidence → Model → Hypothesis → Decision.

**Observation 2 — Confidence History.** Reviews produce not just revised hypotheses but revised confidence levels. A Hypothesis may move from confidence 0.42 → 0.81 → 0.31 across three reviews. That trajectory is independently valuable — separate from the Timeline, which records what changed, Confidence History records how certain the Actor was at each point. Candidate future primitive: Confidence History, owned by the platform, not by any adapter.

**Observation 3 — Pattern Maturity.** Patterns are not binary (exists / does not exist). They evolve: Candidate → Observed → Repeated → Validated → Institutionalised. A Pattern Maturity model would make the Knowledge Graph significantly more powerful — surfacing only validated patterns for high-stakes decisions while still preserving candidate patterns for exploration.

**Observation 4 — Evidence Quality Grades.** Evidence is currently immutable and undifferentiated. In practice, evidence differs in reliability, freshness, independence, and traceability. An annual report and a social media post are both evidence — but they are not equivalent. Evidence quality grades are domain-neutral (every domain has evidence quality) and would strengthen provenance across all adapters.

**Observation 5 — Reflection as a lifecycle stage.** Between Observe and Review there is a cognitive step that is currently implicit: Reflection — the interpretation of raw observations into meaning. "Revenue declined" is an observation. "The decline came from exports rather than pricing" is a reflection. Without an explicit Reflection stage, Review becomes overloaded with both interpretation and evaluation. Candidate future lifecycle: Observe → Reflect → Review → Learn.

**Observation 6 — Pattern Extraction Engine.** Learning will eventually require sophisticated computation: clustering, embeddings, similarity search, graph mining, behavioural sequence mining. These capabilities are already noted as a future engine in the Platform Primitive Summary. The v2 direction is to make Pattern Extraction a first-class engine with its own lifecycle, sitting between Learning and the Knowledge Graph.

**Observation 7 — Knowledge Graph Services.** The Knowledge Graph is currently described as a platform primitive (a store). As the platform matures, the Knowledge Graph will require a richer service layer: persistence (durable, versioned storage of Patterns and links), traversal (graph navigation across Subjects, Actors, and Hypotheses), semantic retrieval (surfacing relevant past Workspaces given a new Intent), and contextual enrichment (annotating new Hypotheses with related Patterns at creation time). These four capabilities are noted as a v2 elaboration — not new primitives, but a Knowledge Graph Services layer that makes the primitive operationally complete.

**Observation 8 — Platform Realisations terminology.** The current document uses "Platform Applications" to describe UltraCrew, ChronoSentiment, and future products. "Applications" implies software products built on top of the platform. "Platform Realisations" is a more precise term — each product is a realisation of the platform in a specific domain, not merely an application that happens to use it. This terminology change is a v2 candidate: it more accurately reflects the relationship between the platform and its domain expressions, and aligns with the adapter model (adapters configure the platform; they do not build on top of it).

**Observation 9 — Platform Invariants.** As the platform is implemented, a set of invariants should be made explicit and enforced. Candidate invariants: (a) every Workspace has exactly one Intent; (b) Evidence is immutable once recorded — no adapter may mutate historical Evidence; (c) every Outcome belongs to exactly one Workspace; (d) Learning never mutates historical Evidence — it only produces Patterns; (e) the Knowledge Graph stores but does not infer without traceability — every link must have a recorded source (Learning computation, explicit Actor assertion, or adapter configuration). These invariants are not yet formally specified in v1. Making them explicit in v2 would make the platform's guarantees auditable and enforceable.

**Unifying observation.** Everything in Coralys evolves: Evidence evolves into understanding, Hypotheses evolve through versioning, Patterns evolve through maturity, Knowledge evolves through the graph. The Continuous Learning Engine is therefore acting as an evolution engine for structured knowledge — managing state transitions of knowledge through governed lifecycle stages. This mental model unifies all future platform capabilities under a single architectural idea.

---

*Coralys Platform Architecture v1.5 | July 2026 | Status: Baseline*
*v1.1: Actor and Intent added as platform primitives; Workspace elevated as transaction boundary; Learning vs Knowledge Graph responsibilities clarified; Pattern Extraction noted as future engine; Question noted as candidate future primitive; Architectural Principle elevated to top of document; Coralys v2 future evolution observations recorded.*
*v1.2: Platform identity revised — Coralys is a Knowledge Evolution Platform; Continuous Learning Engine repositioned as the platform's core computational mechanism, not its identity.*
*v1.3: Product Portfolio Positioning section added (subsequently moved to CORALYS_PLATFORM_STRATEGY.md in v1.4).*
*v1.4: Platform layering diagram and canonical architecture diagram added; three-part model elevated (Platform governs / Engine drives / Knowledge Graph preserves); "Coralys supplies the lifecycle" refined to "lifecycle, governance, and knowledge evolution capabilities"; Product Portfolio Positioning moved to CORALYS_PLATFORM_STRATEGY.md to keep architecture timeless.*
*v1.5: Continuous Learning Engine terminology corrected — "first-class platform primitive" replaced with "core computational subsystem"; three v2 candidates added (Observation 7: Knowledge Graph Services; Observation 8: Platform Realisations terminology; Observation 9: Platform Invariants).*
*Domain adapters: ChronoSentiment Personal, ChronoSentiment Enterprise, UltraCrew, and future products.*
*Review trigger: Material change in platform capability; new domain adapter added; Coralys v2 planning begins.*