import re

with open("AUTHORITY_MAP.md", "r") as f:
    content = f.read()

new_header = """# AUTHORITY_MAP.md
# Coralys — Canonical Authority Registry
# Last updated: 2026-06-03

## Architectural Principle

Coralys is the authoritative platform.

ChronoSentiment is the first domain implementation built on Coralys.

No reusable optimization, simulation, ecology, decision, or recommendation logic may originate in a domain implementation.

Reusable capabilities must be promoted into Coralys.

---

# Coralys Authorities

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

Ecology is a universal concept and is not finance-specific.

---

## Decision Authority

Owns:
* Candidate evaluation
* Consensus formation
* Decision generation
* Conviction scoring
* Decision auditing

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

Owns only finance-specific concepts.

## Market Authority

Owns:
* Market events
* Price data
* Candles
* Order books
* Exchange adapters

---

## Trading Authority

Owns:
* Orders
* Trades
* Positions
* Portfolios
* Execution fills

---

## Financial Strategy Authority

Owns:
* TWAP
* AMM
* Auction mechanics
* Financial evaluators
* Trading policies

---

# UltraCrew Authorities

Owns:
* Workers
* Shifts
* Coverage constraints
* Labor rules
* Fatigue models
* Workforce policies

---

# Promotion Rule

When a capability can be reused by multiple domains without changing its meaning, authority must move to Coralys.

When a capability depends on domain vocabulary or domain semantics, authority remains within the domain implementation.

**Explicit Rule:** ChronoSentiment may depend on Coralys. Coralys must never depend on ChronoSentiment.

---

## CALIBRATION PRESERVATION LAYER
"""

# Replace everything before CALIBRATION PRESERVATION LAYER
content = re.sub(r'^.*?(?=## CALIBRATION PRESERVATION LAYER)', new_header, content, flags=re.DOTALL)

with open("AUTHORITY_MAP.md", "w") as f:
    f.write(content)

