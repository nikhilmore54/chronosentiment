# Orchestration Execution‑Order Certification

## Purpose

Define and enforce the deterministic execution ordering for the orchestration layer. This document captures the **canonicalization rule**, the **projection schema**, and the **adversarial fixture design** that together constitute the certification of orchestration stability.

---
### 1. Orchestration Canonicalization Rule

> **Rule:** *Multi‑asset orchestration must execute against a **canonicalized asset ordering**.*
>
> Execution order may never depend on:
> - hash‑map (or any unordered collection) iteration order
> - insertion nondeterminism
> - thread scheduling or runtime scheduling nondeterminism
> - external enumeration order supplied by callers
>
> **Implementation guidance:**
> - Sort assets by a stable key (e.g., asset identifier) before any loop.
> - Preserve the sorted order throughout the orchestration pipeline.
> - If a specific ordering is required by business logic, encode it explicitly as a deterministic sequence.

---
### 2. ExecutionProjection Schema

The certification functions return an `ExecutionProjection` that captures the deterministic snapshot of orchestration execution.

```rust
#[derive(Serialize, PartialEq, Debug, Hash, Clone)]
pub struct ExecutionProjection {
    /// Canonical ordered list of asset identifiers used during orchestration.
    pub ordered_assets: Vec<String>,
    /// Hash of the full execution trace (e.g., serialized events, decisions).
    pub execution_trace_hash: String,
    /// Human‑readable sequence of high‑level orchestration steps.
    pub execution_sequence: Vec<String>,
    /// Schema version – enables evolution of the projection format.
    pub projection_version: String,
}
```

- **`ordered_assets`** reflects the canonical ordering after sorting.
- **`execution_trace_hash`** is a SHA‑256 digest of the serialized execution trace, providing a stable fingerprint.
- **`execution_sequence`** records the ordered identifiers of orchestration phases (e.g., `"reporting"`, `"sweep"`, `"aggregation"`).
- **`projection_version`** allows safe migration of the projection format.

---
### 3. Adversarial Fixture Design (`asset_permutation_fixture`)

Only a limited, high‑coverage set of permutations is required:

1. **Canonical ordering** – assets sorted by identifier.
2. **Shuffled ordering** – a deterministic shuffle (e.g., reverse, rotate).
3. **Reverse ordering** – descending order.
4. **Duplicated ordering** – repeat one asset to test handling of redundancies.
5. **Sparse ordering** – a subset of assets (e.g., only the first two).

The fixture returns a `Vec<Vec<Asset>>` where each inner vector represents one permutation. Tests iterate over these variants, invoke the certification functions twice, and assert:

- `Projection1 == Projection2`
- Serialized JSON strings are identical.
- SHA‑256 hashes of the serialized projections match.

---
### 4. Certification Test (`tests/orchestration_execution_order.rs`)

The test exercise follows the steps outlined in the fixture description and uses the `ExecutionProjection` defined above. It lives in the `tests/` directory and follows the invariant‑oriented naming convention.

---
### 5. Governance

- All certification code resides under `src/pipeline/certification/` and **must not** be imported by production orchestration modules.
- The `pub(crate)` visibility together with `#[cfg(test)]` guards ensure the code is compiled only for testing.
- Any change to the `ExecutionProjection` schema requires a bump of `projection_version` and an accompanying update to this certification document.

---
*Document author: Antigravity (AI assistant)*
*Last updated: 2026‑05‑28*
