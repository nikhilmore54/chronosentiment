# BENCHMARK-GOVERNANCE.md

**Document class:** Governance  
**Status:** ACTIVE  
**Scope:** CVD-001 benchmark stream and Coralys benchmark adapter pattern  
**Branch:** `governance-hardening`  
**Date:** 2026-07-17

---

## 1. Purpose

This document records the governance principles that govern the CVD-001 benchmark stream and the Coralys benchmark adapter pattern. It is intended for future contributors who need to understand the relationship between benchmark infrastructure and the Coralys platform.

The CVD-001 benchmark stream is **complete**. The artifacts produced are permanent validation infrastructure, not an active research project.

---

## 2. Benchmark Stream Status

The CVD-001 benchmark program is declared **COMPLETE** as of commit `021f9ce0` (tag `M3-FROZEN`).

### Complete frozen artifact chain

| Commit | Tag | Artifact | Role |
|--------|-----|---------|------|
| `721c086c` | — | `SPRINT10-CLOSURE-REPORT-v1.0.md` | Evidence acquisition closure |
| `e0407ded` | — | `WP-M2.1-MATHEMATICAL-BENCHMARK-MODEL-v1.0.md` | Mathematical foundation |
| `eb38a8d0` | — | `WP-M2.2-CREDITED-WORKLOAD-EQUATION-R1.md` | R1 reconstruction |
| `34c84327` | — | `WP-M2.3-OBJECTIVE-FUNCTION-R2.md` | R2 reconstruction |
| `58ceae03` | — | `WP-M2.4-HC3-MATHEMATICAL-DEFINITION-R3.md` | R3 reconstruction |
| `75269f4d` | — | `WP-M2.5-BASE-CAP-ENFORCEMENT-R4.md` | R4 reconstruction |
| `d3e3303a` | `M2-FROZEN` | `WP-M2.6-INTERNAL-CONSISTENCY-VALIDATION.md` | Milestone 2 validation |
| `781499c4` | — | `BENCHMARK-SEMANTICS-v1.0.md` | Implementation semantics |
| `021f9ce0` | `M3-FROZEN` | `BENCHMARK-REFERENCE-SPECIFICATION-v1.0.md` + `adapters/cvd001/` | Engineering spec + reference adapter |

### Benchmark lineage

```
Primary Evidence (GERAD G-2014-22, generator source files)
        │
        ▼
Sprint 10 Evidence Acquisition (S0–S4b)
        │
        ▼
M2-FROZEN: Mathematical Reconstruction (R1–R4, WP-M2.1–WP-M2.6)
        │
        ▼
BENCHMARK-SEMANTICS-v1.0 (implementation semantics, scope boundary)
        │
        ▼
BENCHMARK-REFERENCE-SPECIFICATION-v1.0 (engineering contracts)
        │
        ▼
M3-FROZEN: adapters/cvd001/ (reference adapter, 32/32 oracle tests)
        │
        ▼
Permanent Regression Infrastructure
```

---

## 3. Governance Principles

### Principle 1 — Frozen benchmark lineage

M2 and M3 artifacts are **immutable** except through deliberate versioned revisions.

A versioned revision requires:
- A new work package document (e.g. `WP-M2.2-v2.0.md`) explaining the change and its evidence basis
- A new version of the affected downstream artifacts (semantics, specification, adapter)
- A new git tag (e.g. `M2-FROZEN-v2`, `M3-FROZEN-v2`)
- All oracle tests passing at the new version

Corrections to typographical errors or formatting do not require a versioned revision, but must not change any mathematical definition, implementation decision, or oracle expected value.

### Principle 2 — Reference adapter authority

`adapters/cvd001/` is the **executable conformance reference** for the CVD-001 benchmark.

The oracle test suite (`cargo test -p cvd001`) is the normative correctness gate. Any change to the adapter that causes an oracle test to fail is a breaking change and requires a versioned revision per Principle 1.

### Principle 3 — Regression requirement

Changes to the Coralys evaluation framework that affect evaluation behaviour must preserve oracle parity with the standalone adapter, or be accompanied by a deliberate versioned benchmark update.

The regression criterion is:

> For all oracle inputs O1–O8, OI1–OI2: `standalone_adapter(input) == integrated_adapter(input)`

This criterion must be maintained as a permanent CI gate once Milestone 4 (Coralys Evaluation Framework) is implemented.

### Principle 4 — Platform independence

Coralys evolves independently of benchmark semantics. Benchmarks validate Coralys; they do not define Coralys.

Specifically:
- The CVD-001 objective function (Z = Σ_n Δ_n, α=0, β=1) is a benchmark default, not a Coralys constraint. Coralys may use different objective models.
- The HC3 hard constraint is a benchmark constraint, not a universal Coralys constraint. Coralys may support additional or different constraint models.
- The `FlightLeg → Duty → CrewMember` domain model is the CVD-001 domain model. Coralys may extend or generalise it.

### Principle 5 — Adapter pattern

The CVD-001 adapter establishes the **canonical pattern** for future benchmark adapters. Future benchmarks (INRC, CVRP, rail scheduling, workforce scheduling, etc.) should follow the same structure:

```
adapters/<benchmark-name>/
├── Cargo.toml
└── src/
    ├── lib.rs          # crate root; re-exports public API
    ├── types.rs        # benchmark-specific domain types
    ├── ...             # benchmark-specific computation modules
    └── evaluator.rs    # evaluate() — implements Coralys evaluation interface
```

Each adapter must:
- Implement the Coralys evaluation framework interfaces (defined in Milestone 4)
- Carry its own oracle test suite
- Maintain oracle parity with the standalone adapter when integrated

---

## 4. Coralys Evaluation Framework Architecture

The intended architecture for Milestone 4 and beyond:

```
                Coralys Evaluation Framework
                          │
        ┌─────────────────┼─────────────────┐
        │                 │                 │
   CVD001 Adapter    INRC Adapter     Future Adapters
```

Coralys owns:
- Evaluation pipeline
- Constraint evaluation interface
- Objective evaluation interface
- Common `EvaluationResult` type
- Adapter registration

Adapters implement those interfaces. They do not own them.

This separation ensures that adding a new benchmark adapter does not require changes to the Coralys core, and that changes to the Coralys core do not require changes to benchmark adapters (beyond interface conformance).

---

## 5. Author Correspondence (Optional)

If contact with the original benchmark authors (Kasirzadeh, Saddoune, Soumis) is pursued, it should be framed as:

- Presenting the reconstructed benchmark for validation
- Requesting clarification where public artifacts were incomplete
- Validating or correcting specific implementation assumptions (R1–R4)

This is an evidence-improvement exercise (potential ER-010+ entries in `BENCHMARK-KNOWLEDGE-MATRIX-v1.1`), not a prerequisite for Coralys development.

---

## 6. Relationship to Other Documents

| Document | Role | Status |
|----------|------|--------|
| `data/cvd001/BENCHMARK-SEMANTICS-v1.0.md` | Implementation semantics | Frozen (781499c4) |
| `docs/BENCHMARK-REFERENCE-SPECIFICATION-v1.0.md` | Engineering contracts | Frozen (021f9ce0) |
| `adapters/cvd001/` | Reference adapter | Frozen (021f9ce0, M3-FROZEN) |
| This document | Governance principles | Active |

---

*End of BENCHMARK-GOVERNANCE.md*