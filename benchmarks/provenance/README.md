# Provenance Registry v1.0

**Document ID:** PROVENANCE-REGISTRY-v1.0  
**Status:** Active  
**Created:** 2026-07-09  
**Milestone:** M18.1 — Provenance Registry  
**Governance reference:** GOV-008 v1.2, EVIDENCE-v1.5.md (P4)

---

## Purpose

Every benchmark comparison in Coralys must be traceable to its authoritative source.
This registry records the provenance of each benchmark family: where the instances
came from, what publication defined them, how distances are computed, where BKS values
originate, and what the current verification status is.

Without this registry, a gap measurement like `+0.82%` is a number without a referent.
With it, the same measurement carries a full chain of custody:

```
Instance:    Tai100d
BKS:         1575.03
BKS source:  CVRPLIB catalog (galgos.inf.puc-rio.br)
Publication: Taillard 1993 (Networks, 23(8):661–673)
Distance:    EUC_2D, TSPLIB integer rounding
Rounding:    nint(sqrt((x1-x2)^2 + (y1-y2)^2))
Status:      Under Investigation (P4 — negative gaps observed)
```

---

## Schema

Each family file is a TOML document with the following top-level sections:

| Section               | Required | Description |
|-----------------------|----------|-------------|
| `[family]`            | Yes      | Identity: id, full name, instance count, scope |
| `[source]`            | Yes      | Primary source URL, mirror, access notes |
| `[publication]`       | Yes      | Authors, title, year, journal/book, DOI |
| `[objective]`         | Yes      | Objective function, multi-objective flag, fleet cost |
| `[coordinate_type]`   | Yes      | EUC_2D / EXPLICIT / other; coordinate source |
| `[rounding]`          | Yes      | Rounding rule, formula, verified flag |
| `[bks]`               | Yes      | BKS source, extraction method, value type, notes |
| `[verification]`      | Yes      | Status, method, date, campaign reference, open items |
| `[instance_exceptions]` | No    | Per-instance overrides for genuine exceptions |
| `[instance_vehicle_counts]` | No | Per-instance vehicle counts (when not in .vrp file) |
| `[distance_exceptions]` | No   | Per-instance distance metric overrides |
| `[vehicle_count_encoding]` | No | How vehicle count is encoded (e.g. name suffix) |

---

## Verification Status Values

| Status | Meaning |
|--------|---------|
| `Verified` | Cross-checked against authoritative external source |
| `Verified (with one suspect instance)` | Family verified; one instance flagged for investigation |
| `Under Investigation` | Systematic anomalies observed; root cause not yet determined |
| `Publication Only` | Sourced from original publication; not cross-checked against CVRPLIB |

---

## Qualification Level Values

| Level | Meaning |
|-------|---------|
| `Provisionally Qualified` | No systematic negative gaps; BKS provenance verified; Stage B pending |
| `Under Investigation` | Negative gaps observed; root cause not yet determined |
| `Excluded` | Outside current campaign scope (e.g. >200 customers) |

---

## Registry Index

| File | Family | Instances | In Scope | Verification Status | Qualification Level |
|------|--------|-----------|----------|---------------------|---------------------|
| [`A.toml`](A.toml) | Augerat A | 27 | Yes | Verified | Provisionally Qualified |
| [`B.toml`](B.toml) | Augerat B | 23 | Yes | Verified | Provisionally Qualified |
| [`CMT.toml`](CMT.toml) | Christofides-Mingozzi-Toth | 14 | Yes | Under Investigation | Under Investigation |
| [`E.toml`](E.toml) | Augerat E | 13 | Yes | Verified | Provisionally Qualified |
| [`F.toml`](F.toml) | Fisher | 3 | Yes | Verified | Provisionally Qualified |
| [`Golden.toml`](Golden.toml) | Golden et al. 1998 | 20 | No | Verified (registry only) | Excluded |
| [`Li.toml`](Li.toml) | Li et al. 2005 | 12 | No | Publication Only | Excluded |
| [`M.toml`](M.toml) | Christofides M (large) | 2 | Yes | Under Investigation | Under Investigation |
| [`P.toml`](P.toml) | Augerat P | 24 | Yes | Verified (1 suspect) | Provisionally Qualified |
| [`Tai.toml`](Tai.toml) | Taillard 1993 | 13 | Yes | Under Investigation | Under Investigation |
| [`X.toml`](X.toml) | Uchoa et al. 2017 | 100 | Yes (28) | Verified | Provisionally Qualified |

---

## Open Items (P4)

The following families have negative gaps under investigation. These are the primary
targets of the P4 provenance verification activity.

| Family | Instances with Negative Gap | Worst Gap | Priority |
|--------|-----------------------------|-----------|----------|
| CMT | 7 of 14 | −32.65% (CMT13) | Critical |
| Tai | 5 of 13 | −7.61% (Tai75d) | High |
| M | 2 of 2 | −2.99% (M-n200-k17) | High |
| P | 1 of 24 | −2.04% (P-n55-k8) | Medium |

Root cause hypotheses (from `benchmark_qualification_spec.md`):
1. Fleet semantics mismatch — Coralys using more vehicles than benchmark K
2. BKS provenance — registry values differ from values used by comparison papers
3. Distance rounding — float vs integer objective representation

Resolution requires Stage B certificates (route count, capacity, customer coverage)
for each affected instance. See `fleet_semantics/` registry for P5 status.

---

## Relationship to Other Registries

| Registry | Location | Relationship |
|----------|----------|--------------|
| Fleet Semantics Registry | `benchmarks/fleet_semantics/` | Complements provenance — records ATMOST/EXACT constraint per family |
| CVRPLIB Operational Registry | `adapters/cvrp/src/bin/cvrplib_registry.rs` | Runtime registry used by the solver; provenance registry is the external-facing companion |
| Campaign Evidence | `benchmarks/campaign/EVIDENCE-v1.5.md` | Records measured campaign results; provenance registry explains what those results are compared against |

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| v1.0 | 2026-07-09 | Initial registry — 10 family files created as part of M18.1 |