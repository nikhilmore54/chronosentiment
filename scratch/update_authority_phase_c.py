import re

with open("AUTHORITY_MAP.md", "r") as f:
    content = f.read()

new_header = """# AUTHORITY_MAP.md
# Coralys — Canonical Authority Registry
# Last updated: 2026-06-03

## Coralys Extraction Status

Phase A: Authority Freeze
Status: Complete

Phase B: Workspace Restructuring
Status: Complete

Phase C: Trait Migration
Status: Active

Phase D: Implementation Migration
Status: Pending

Phase E: Adapter Finalization
Status: Pending

---

## Architectural Principle

Coralys is the authoritative platform.

ChronoSentiment is the first domain implementation built on Coralys.

No reusable optimization, simulation, ecology, decision, or recommendation logic may originate in a domain implementation.

Reusable capabilities must be promoted into Coralys.

## Trait Before Implementation Rule

No implementation may be moved into a Coralys crate
until the corresponding abstract trait compiles independently
of ChronoSentiment.

Authority follows abstraction first, implementation second.

---

# Coralys Authorities

## Deferred Authority: Observation

Status: Deferred

Reason:
The current observation-related types contain a mixture of
generic event abstractions and ChronoSentiment-specific market
abstractions.

No Observation Authority may be created until a domain-neutral
event model has been proven through trait migration.

Examples currently under review:

- CanonicalEvent
- ChronologyEvent
- TimelineEvent
- EventStreamResponse
- SymbolicCandle
- SubstrateTick

Temporary Authority:
ChronoSentiment

---

## MOGA Authority

Owns:
* Genome definitions
* Population management
* Mutation operators
* Crossover operators
* Selection operators
* Pareto ranking
* Fitness evaluation interfaces
* Evolution state
* Elite archives

Must not contain:
* Trade, Order, Position, Portfolio, Price
* Worker, Shift, Vehicle, Route

---

## Simulation Authority

Owns:
* Replay engines
* Scenario execution
* Environment modeling
* State transitions
* Execution abstractions
* Outcome generation

Must remain domain-agnostic.

---

## Ecology Authority

Owns:
* Geometry construction
* Clustering
* Archetypes
* Regimes
* Topology fields
* Transition analysis
* Divergence detection
* Behavioral characterization
* EdgeTransfer
* TrajectoryTransitionMatcher

Ecology is a universal concept and is not finance-specific.

---

## Decision Authority

Owns:
* CandidateEvaluation
* Decision
* Consensus
* Conviction
* Gate Decisions
* Decision Auditing

This is now the bridge layer between Ecology -> Decision -> Recommendation.

---

## Recommendation Authority

Owns:
* Candidate ranking
* Recommendation generation
* Explanation generation
* Recommendation scoring

Recommendations consume decisions and ecological state.

---

# ChronoSentiment Authorities
"""

# We want to replace everything from the beginning to the line containing "# ChronoSentiment Authorities"
# Using regex:
pattern = re.compile(r'^.*?# ChronoSentiment Authorities\n', re.DOTALL)
content = pattern.sub(new_header, content)

with open("AUTHORITY_MAP.md", "w") as f:
    f.write(content)

