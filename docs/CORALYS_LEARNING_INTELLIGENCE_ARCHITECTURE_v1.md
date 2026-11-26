# Coralys — Learning Intelligence Architecture

**Document ID:** CORALYS-LI-ARCH-001  
**Document type:** Platform Architecture (additive capability)  
**Version:** 1.0  
**Status:** Baseline draft  
**Date:** 2026-11-16  
**Owner:** Platform / Engineering  

---

## Document Governance

| Field | Value |
|-------|-------|
| Document Status | Baseline draft v1.0 |
| Review Trigger | Material change in Learning/Optimization/Decision contracts; first Learning backend registration; Coralys v2 planning |

**Relationship to other documents:**

- Complementary to: [`CORALYS_PLATFORM_ARCHITECTURE.md`](CORALYS_PLATFORM_ARCHITECTURE.md) — Knowledge Evolution Platform identity; Continuous Learning Engine (CLE) owns the **knowledge lifecycle**. This document defines **Learning Intelligence** as a separate computational capability (ML paradigms + model infrastructure).
- Complementary to: [`CORALYS_PLATFORM_STRATEGY.md`](CORALYS_PLATFORM_STRATEGY.md) — commercial hierarchy; Learning remains platform-internal unless a product surfaces it.
- Complementary to: [`CORALYS_DECISION_INTELLIGENCE_MVP_V01.md`](CORALYS_DECISION_INTELLIGENCE_MVP_V01.md) — Decision evaluates/selects and records; Learning may inform alternatives but does not allocate capital or bypass user-controlled execution.
- Complementary to: Coralys Optimization / `coralys-moga` — multi-objective evolutionary search remains independent; ML training is **not** embedded in the MOGA core.
- Research programme: [`docs/research/rp002_coralys_learning_architecture.md`](research/rp002_coralys_learning_architecture.md) (**RP-002**) — Coralys Learning Architecture; foundational refs Grefenstette (1993) + Schaffer & Grefenstette; multidimensional feedback as central research subject; authority 0%.
- Research Reading Register: [`docs/research/rp002/RP002_RESEARCH_READING_REGISTER.md`](research/rp002/RP002_RESEARCH_READING_REGISTER.md) (**RP-002-RRR**) — initial freeze of gap-mapped literature; **architecture not frozen** by the register.
- Domain adapters (ChronoSentiment, UltraCrew, future): own features, labels, rewards, constraints, and interpretation of learned outputs. Domain freezes (e.g. RP-608 Gate 5, OF ±0.30, OFX 0% authority) stay **domain governance**, not Learning core policy.

---

## Platform statement

Coralys should evolve into a **Learning and Decision Intelligence Platform** by introducing **Learning Intelligence** as a first-class platform capability alongside **Optimization**, **Decision Intelligence**, and the **Execution Framework**.

> **Coralys learns from data, discovers patterns, predicts outcomes, optimises alternatives, supports decisions and improves through observed results.**

This is an **additive** architectural evolution. Existing optimization capabilities, APIs, benchmarks, and platform boundaries are preserved wherever possible. The Knowledge Evolution Platform identity and the Continuous Learning Engine lifecycle remain in force; Learning Intelligence does **not** rename or replace the CLE.

---

## 1. Platform capability model

The Coralys Platform comprises four complementary capabilities:

| Capability | Responsibility |
| ---------- | -------------- |
| **Learning Intelligence** | Discover patterns, predict outcomes, train models and learn decision policies. |
| **Optimization** | Explore solution spaces and identify optimal or near-optimal solutions under objectives and constraints (including MOGA / evolutionary computation). |
| **Decision Intelligence** | Evaluate alternatives, assess trade-offs and select appropriate actions under governance. |
| **Execution Framework** | Execute decisions, observe outcomes and feed operational feedback into the platform. |

These capabilities remain independently defined but interoperable through stable platform contracts.

### Continuous Learning Engine vs Learning Intelligence

| Concept | Owns | Does not own |
| ------- | ---- | ------------ |
| **Continuous Learning Engine (CLE)** | Knowledge lifecycle: Workspace → Evidence → Hypothesis → Review → Outcome → Pattern → Knowledge Graph | Pluggable ML training frameworks; model registries as product APIs |
| **Learning Intelligence** | Supervised / unsupervised / RL paradigms; feature & dataset infra; training, evaluation, registration, inference, experiment tracking, monitoring | Domain vocabulary; MOGA operators; Decision seals; capital allocation |

CLE answers: *how knowledge evolves under governance.*  
Learning Intelligence answers: *how statistical / policy models are trained, versioned, and served for use by Optimization and Decision.*

---

## 2. Learning Intelligence

Learning Intelligence is a reusable, domain-independent platform capability supporting three principal learning paradigms:

- **Supervised Learning:** Learn relationships from labelled historical data to predict outcomes.
- **Unsupervised Learning:** Discover patterns, structures, clusters and relationships in unlabelled data.
- **Reinforcement Learning:** Learn decision policies through interactions, feedback and reward signals.

It provides common infrastructure for:

- Feature engineering and data preparation.
- Dataset management and validation.
- Model training and evaluation.
- Model registration, versioning and lifecycle management.
- Batch and real-time inference.
- Experiment tracking and reproducibility.
- Model monitoring and performance evaluation.

Learning algorithms are **pluggable**. The platform must not depend on a particular ML framework or programming language.

---

## 3. Integration with Optimization and Decision Intelligence

Learning and optimization **complement** rather than replace one another.

- Learning provides predictions, discovered patterns, estimated parameters and adaptive policies.
- Optimization uses learned information to evaluate candidate solutions and explore trade-offs.
- Decision Intelligence evaluates optimized alternatives against objectives, constraints and decision policies.
- Execution Framework implements selected decisions and captures resulting outcomes.

**Hard boundary:** The Optimization capability, including MOGA and evolutionary computation, retains its independent responsibilities. **Machine-learning training must not be embedded into the MOGA core.**

### MOGA lineage note (genetic algorithms as search over knowledge / policies)

Two classical references constrain where GA-style work sits in Coralys:

**A. Schaffer & Grefenstette — *Multi-Objective Learning via Genetic Algorithms* (LS-2 / vector-evaluated GA)**

- Scalar fitness can extinguish **specialists** needed for multi-aspect tasks.
- **Vector-valued performance** with complementary knowledge sharing improves multi-class / multi-objective learning via evolutionary search.
- Critic (reward/punishment) design dominates outcomes: too lax → indiscriminate guessing; too strict → do-nothing strategies.

**B. Grefenstette — *Genetic Algorithms and Machine Learning* (ACM COLT ’93 survey)**

- GAs are **population-based heuristic search**: candidate structures, fitness-proportional selection, genetic operators (optionally hybridized with local search).
- The same machinery has been applied to ML-shaped problems (concept learning, NN weight search, sequential decision rules)—but that does **not** make the GA core a supervised/RL training framework.
- **SAMUEL pattern:** chromosome encodes a set of condition–action rules; fitness is measured by evaluating the control strategy on a **simulator**. Multi-agent and **non-stationary** environments (including co-learning opponents) are Optimization evaluation concerns.
- Schema / implicit-parallelism reading (Holland): above-average subspaces receive exponentially more trials—explore/exploit allocation is an Optimization design property, not a Learning Intelligence training loop.

**Coralys placement:** that lineage belongs to **Optimization** (multi-objective fitness vectors, scenario aggregation, critic/fitness design, simulator-evaluated rule/policy search, non-stationary robustness)—not to Learning Intelligence’s pluggable supervised/unsupervised/RL training infrastructure. Learning Intelligence may *emit* features, predicted parameters, or policy priors that Optimization consumes as inputs or soft guidance; it does **not** absorb MOGA into an ML framework, and “GA learned a strategy on a simulator” remains Optimization + domain evaluation—not authority to train models inside `coralys-moga`.

**Research programme:** RP-002 elevates both papers as **foundational references** and treats Schaffer & Grefenstette’s multidimensional / per-component selection mechanism as a **central research subject** for learning-controller design (preserve complementary optimisation knowledge)—as a Coralys extension beyond the paper’s classification demonstration. See [`rp002_coralys_learning_architecture.md`](research/rp002_coralys_learning_architecture.md).

ChronoSentiment-style requirements (determinism of the strategy state machine; multi-scenario evaluation; aggregated fitness) remain **Optimization / evaluation** concerns. Stochastic ML training runs are isolated behind Learning contracts and must not inject non-determinism into the exchange/simulation event path.

---

## 4. Domain independence and extensibility

Learning Intelligence remains independent of application and industry semantics.

Domain-specific solution components define:

- Domain features and their representations.
- Training labels and target variables.
- Business objectives and constraints.
- Reward functions and evaluation criteria.
- Interpretation of predictions and learned patterns.

Coralys provides the reusable learning algorithms, model infrastructure, lifecycle management and execution mechanisms for inference.

This separation allows multiple solution domains to use the same learning infrastructure without duplicating algorithms or introducing application-specific dependencies into the platform.

---

## 5. Continuous intelligence cycle

The four platform capabilities form a continuous feedback cycle:

```text
Observe  →  Learn  →  Optimize  →  Decide  →  Execute
   ↑                                              │
   └──────────── operational feedback ────────────┘
```

| Stage | Capability | Role |
| ----- | ---------- | ---- |
| **Observe** | Execution / adapters | Collect data and operational outcomes |
| **Learn** | Learning Intelligence | Discover patterns and predict outcomes |
| **Optimize** | Optimization (MOGA) | Generate and evaluate candidate solutions |
| **Decide** | Decision Intelligence | Evaluate alternatives and select actions under governance |
| **Execute** | Execution Framework | Implement decisions and capture feedback |

Feedback returns to Observe and Learn. Learned outputs do **not** bypass Decision governance or hard constraints.

---

## 6. Architectural principles

1. **Learning Before Automation:** Use data-derived intelligence to improve decisions rather than relying exclusively on predefined rules.
2. **Optimization Remains Independent:** Learning informs optimization but does not replace its algorithms or responsibilities.
3. **Domain-Agnostic Platform:** Learning, optimization and decision capabilities remain reusable across solution domains.
4. **Pluggable Algorithms:** Support multiple learning paradigms, algorithms and execution technologies.
5. **Evidence-Based Improvement:** Model performance and decision quality must be measured against defined baselines.
6. **Governed Adaptation:** Learned outputs must respect hard constraints, policies, safety requirements and human oversight.
7. **Continuous Intelligence:** Operational outcomes provide feedback for future learning and decision improvement.

---

## 7. Platform contracts (sketch)

Stable contracts only—no framework lock-in. Backends register against these shapes.

| Contract | Purpose |
| -------- | ------- |
| `Dataset` | Versioned collection of records with schema, split, provenance, validation status |
| `FeatureSpec` | Named feature definitions, transforms, and domain-owned encodings |
| `Experiment` | Reproducible training run: dataset + feature set + algorithm backend + hyperparameters + metrics |
| `ModelCard` | Registered model identity: version, metrics, lineage, intended use, constraints, monitoring hooks |
| `InferenceRequest` / `InferenceResponse` | Batch or real-time prediction / policy query with model version and audit fields |
| `RewardSignal` | Domain-defined reward / critic feedback for RL or policy evaluation (not MOGA fitness vector itself) |
| `FeedbackEvent` | Execution outcome linked to decision/model versions for closed-loop learning |

**Non-goals for these contracts:**

- Embedding training loops inside `coralys-moga` or domain event engines.
- Granting unsupervised model outputs **authority** without Decision / human oversight.
- Encoding ChronoSentiment, UltraCrew, or clinical semantics into platform Learning types.
- Changing domain freezes (Gate 5, OF ±0.30, OFX authority 0%) from Learning alone.

---

## 8. Proposed architectural evolution

**Coralys evolves from an Optimization and Decision platform into a Learning, Optimization and Decision Intelligence platform.**

Intended outcome: a reusable platform capable of learning from data, discovering previously unknown patterns, predicting future conditions, optimizing complex decisions, and improving its intelligence through observed outcomes—while preserving MOGA independence, Decision governance, and domain-adapter vocabulary ownership.

**Implementation posture:** introduce Learning Intelligence through these explicit architecture contracts, then incremental implementation and validation. No production authority changes are implied by this document alone.

---

## Change log

| Version | Date | Notes |
| ------- | ---- | ----- |
| 1.0 | 2026-11-16 | Initial additive architecture: four-capability model; CLE vs Learning Intelligence; MOGA boundary; contract sketch; continuous cycle |
| 1.0.1 | 2026-11-16 | MOGA lineage expanded with Grefenstette COLT ’93 (GA∩ML survey, SAMUEL simulator fitness, non-stationary / multi-agent)—still Optimization, not ML-inside-MOGA |
| 1.0.2 | 2026-11-16 | Cross-link RP-002 (CLA); multidimensional feedback / complementary knowledge as research programme subject |
| 1.0.3 | 2026-11-16 | Cross-link RP-002-RRR Research Reading Register (initial freeze); architecture remains unfrozen |

---

*Coralys Learning Intelligence Architecture v1.0 | November 2026 | Status: Baseline draft*  
*Review trigger: first Learning backend registration; material Optimization/Decision contract change.*
