# SURFACE HASH CONTRACT v1

## 1. Constitutional Authority
This contract governs the cryptographic immutability of Ecological Survivability Surfaces. A Surface is a frozen scientific artifact that compiles the chronological, topological, and economic traces of a strategy. It is not dynamically generated; it is statically compiled.

## 2. Deterministic Compilation Rule
The `survivability_surface_builder.py` compiler must be perfectly deterministic. For a given:
- Canonical Substrate (`batch_id`)
- Topology Matrix
- Strategy Physics Ledger
- Strategy Economic Ledger
- Replay Artifact Hash

The compiler must ALWAYS yield an identical `surface_hash`. 

## 3. Replay Equivalence Guarantee
The `surface_hash` serves as the final seal of causal isolation. 
If the underlying execution physics, chronology, or alpha generation changes, the Physics Ledger hash will change, the Economic Ledger hash will change, and the `surface_hash` will fracture. This guarantees that any visualization or interpretation of the surface is strictly anchored to a verified and reproducible laboratory condition.

## 4. Prohibition on Mutable State
Surfaces cannot be "updated" or "refreshed". If a new topology or new strategy variant is tested, a completely new Surface Artifact with a new `surface_hash` is generated. The history of surfaces remains append-only and immutable.
