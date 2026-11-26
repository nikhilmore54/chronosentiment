# ARCH-011: Coralys Canonical Platform Architecture

## Overview

This document defines the canonical architecture for the Coralys platform as it transitions from an airline schedule optimizer to a governed, domain-agnostic operational runtime. It establishes the 4-layer platform model, concept ownership, and the deprecation path for legacy implementations.

---

## 1. The 4-Layer Architecture Model

The Coralys ecosystem is organized into four distinct layers. Every crate, module, and concept must belong to exactly one of these layers.

### Layer 1: Coralys Platform (Core)
**Purpose:** Build the domain-agnostic operational optimization runtime.
- **Components:** `coralys-moga` (Runtime Contracts, Optimization Engine, Decision Lineage, Explainability).
- **Scope:** Defines `OperationalModel`, `ConstraintModel`, `ObjectiveModel`, `OptimizationEngine`.
- **Status:** **STABLE**. No domain-specific concepts (e.g., shifts, portfolios) may enter this layer.

### Layer 2: Generic Research (Research)
**Purpose:** Evolve generic platform capabilities before they are stabilized into the Core platform.
- **Components:** `coralys-ecology`, `Reference OEN` (future).
- **Scope:** MOGA improvements, Decision Intelligence algorithms, Adaptation models.

### Layer 3: Domain Research (Adapters)
**Purpose:** Validate the Coralys platform across specific operational domains.
- **Components:** `CS-*` (ChronoSentiment), `UC-*` (UltraCrew), `RO-*` (ROADEF).
- **Scope:** Domain models (e.g., Market Simulation, DGCA Rules).

### Layer 4: Commercial Products (Applications)
**Purpose:** User-facing applications built on top of the domain adapters.
- **Components:** `ultracrew_server`, `cvrp_server`, UI portals.

---

## 2. Concept Ownership Matrix

To prevent ambiguity and architectural leakage, concepts are strictly owned by specific layers:

| Concept | Canonical Owner (Crate/Layer) |
| :--- | :--- |
| **Runtime Contracts** | Coralys Core (`coralys-moga`) |
| **Optimization Engine** | Coralys Core (`coralys-moga`) |
| **Reference OEN** | Coralys Research |
| **Ecology / Survival** | Coralys Research (`coralys-ecology`) |
| **Decision Intelligence**| Application Layer / Adapters |
| **Airline Rules** | UltraCrew (`adapters/ultracrew`) |
| **Market Simulation** | ChronoSentiment (`adapters/chronosentiment`) |
| **ROADEF Dataset** | ROADEF (`adapters/roadef`) |

---

## 3. Deprecation and Consolidation Roadmap

As part of RP-600, several legacy concepts and implementations have been classified for eventual archival or consolidation.

### Preserved Concepts
- **Ecology:** Remains in the Research layer (`coralys-ecology`) as it represents long-term system adaptation and resilience, distinct from a standard `ObjectiveModel`.
- **Decision Intelligence:** Remains in the Application layer, encompassing recommendation logic, insights, and telemetry. UltraCrew-specific implementations stay in `adapters/ultracrew`.

### Compatibility Layer
The following concepts are domain-specific implementations that will remain supported as **Compatibility** models until their respective applications migrate to the Native OEN:
- `ScheduleGenome`
- `Shift` (Domain Activity)
- `Worker/Crew` (Domain Resource)

### Deprecated Crates (Pending Archival)
The following research crates are deprecated. Their functional concepts will be migrated to `coralys-moga` or the relevant domain adapter before they are deleted:
- `coralys-simulation` (Functionality migrating to ChronoSentiment)
- `coralys-policy` (Functionality migrating to ChronoSentiment/Manufacturing)
- `coralys-decision`, `coralys-eval`, `coralys-matching`, `coralys-planning`, `coralys-recommendation`, `coralys-v2`, `coralys-core`

*Note: No crate will be deleted until its references are cleanly removed and its underlying concept is intentionally retired or absorbed elsewhere.*

---

## Conclusion

This document formalizes **Coralys Platform v2.0 Baseline**. Future platform evolution (RP-* series) will exclusively target Layer 1 and Layer 2, ensuring domain independence. Domain research (CS-*, UC-*) will exclusively target Layer 3 and Layer 4.
