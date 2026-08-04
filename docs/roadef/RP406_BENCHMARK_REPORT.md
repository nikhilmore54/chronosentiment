# RP-406A Benchmark Report: setA-17 Feasibility Frontier Investigation

**Programme:** RP-406A (Instance Diagnostics)
**Status:** COMPLETE
**Date:** 2026-08-04
**Author:** Research Programme (automated)

---

## 1. Programme Context

### 1.1 Research Programme Lineage

| Programme | Description | Status |
|-----------|-------------|--------|
| RP-401C | ECMP-aware greedy construction (repair operator) | Complete |
| RP-403 | Construction portfolio baseline | Complete |
| RP-404A–D | LNS framework — five destroy operators | Complete |
| RP-405 | Adaptive operator selection | Complete |
| **RP-406A** | **setA-17 feasibility frontier investigation (this programme)** | **Complete** |

### 1.2 Motivation

setA-17 produced `inf` objective across all 120 evaluations in RP-404 (five fixed operators × 20 instances each) and RP-405 (adaptive operator selection). The RP-405 reviewer noted that further gains are unlikely to come from changing the destroy-operator selection policy, and that the remaining frontier lies in understanding the structural cause of the persistent infeasibility of setA-17.

RP-406A is explicitly framed as an **instance diagnostics programme** rather than an optimisation programme. The goal is to determine whether setA-17 is structurally recoverable before investing in stronger search.

### 1.3 RP-406A Hypothesis

> The infeasibility of setA-17 is not caused by graph disconnection or aggregate capacity shortage. Instead, it originates within the evaluator's treatment of unassigned demands, which are routed via ECMP shortest paths by default. Identifying which objective component first evaluates to infinity will distinguish between an intentional hard-feasibility rule and an implementation defect.

---

## 2. Diagnostic Method

### 2.1 Diagnostic Binary

**Binary:** [`rp406_setA17_diag`](adapters/roadef/src/bin/rp406_setA17_diag.rs)
**Commit:** `f7a0e2bd`
**Cargo check:** Clean (0 errors)

The binary performs the following checks in sequence:

| Check | Purpose |
|-------|---------|
| Load instance files | Verify parser/input correctness |
| Network capacity statistics | Rule out aggregate capacity shortage |
| Demand volume statistics | Compute volume/capacity ratio |
| Reachability at t=0 | Pure graph connectivity |
| Reachability at t=1 (intervention applied) | Identify demands broken by intervention |
| Evaluate empty solution | Detect evaluator behaviour with no paths assigned |
| Evaluate RP-403 solution | Confirm baseline objective/validity |
| Evaluate RP-405 solution | Confirm adaptive solution objective/validity |
| Link utilisation (RP-403, t=0 and t=1) | Identify saturated resources |

### 2.2 Evaluator Source Analysis

The evaluator source ([`adapters/roadef/src/evaluator.rs`](adapters/roadef/src/evaluator.rs)) was inspected to understand the objective computation. Key findings:

- [`expand_sr_path`](adapters/roadef/src/ecmp.rs) with empty waypoints routes a demand via ECMP shortest path (backward Dijkstra + ECMP routing).
- For each link: `sat = flow / capacity` if `capacity > 0`, else `f64::INFINITY`.
- If `sat >= 1.0`: `inv_load_cost += f64::INFINITY`.
- The objective is `sum(mlu + inv_load_cost)` across all time slots.
- A single overloaded link makes the objective `f64::INFINITY`.

---

## 3. Instance Characteristics

| Property | Value |
|----------|-------|
| Nodes | 300 |
| Links | 1270 |
| Demands | 2000 |
| Time slots | 2 |
| Max segments | 6 |
| Intervention | t=1, link 1074 disabled (1 link) |
| Total link capacity | 966,501,968 |
| Min/Mean/Max link capacity | 219 / 761,025 / 6,082,785 |
| Zero-capacity links | 0 |
| Total demand volume t=0 | 1,818,258 |
| Total demand volume t=1 | 1,852,892 |
| Volume/Capacity ratio t=0 | 0.0019 |
| Volume/Capacity ratio t=1 | 0.0019 |

setA-17 is the largest instance in the benchmark (300 nodes, 1270 links, 2000 demands). The volume/capacity ratio of 0.0019 indicates enormous aggregate spare capacity — the instance is not infeasible due to insufficient total capacity.

---

## 4. Diagnostic Results

### 4.1 Reachability

| Check | Result |
|-------|--------|
| Demands unreachable at t=0 | 0 / 2000 |
| Demands unreachable at t=1 (intervention applied) | 0 / 2000 |
| Demands unreachable at both t | 0 / 2000 |

**Finding:** All 2000 demands are reachable at both time slots. Graph disconnection is not the cause of infeasibility.

### 4.2 Empty Solution Evaluation

| Solution | obj | valid |
|----------|-----|-------|
| Empty (no paths assigned) | inf | true |
| RP-403 (425 paths assigned) | inf | true |
| RP-405 (425 paths assigned) | inf | true |

**Critical observation:** The empty solution evaluates to `obj=inf, valid=true`. This means the infeasibility is not caused by the LNS operators or repair — it is a property of the instance under the evaluator's default routing behaviour.

The `valid=true` result confirms that "valid" checks only structural correctness of the solution representation (SR path syntax, segment count), not objective finiteness. The objective independently assigns an infinite penalty when any link is overloaded.

### 4.3 Link Utilisation (RP-403 Solution)

| Time slot | Overloaded links (sat ≥ 1.0) | Max utilisation | Bottleneck link |
|-----------|------------------------------|-----------------|-----------------|
| t=0 | 1 | 1.0001 | Link 1173 (12→36, cap=1513, flow=1513.11) |
| t=1 | 0 | 0.4445 | Link 1173 (12→36, cap=1513, flow=672.50) |

**Finding:** Only one link is overloaded, and only at t=0, by 0.11 units on a 1513-capacity link. The overflow is marginal in absolute terms but sufficient to trigger `inv_load_cost = f64::INFINITY`.

### 4.4 Root Cause Identification

The evaluator routes unassigned demands via ECMP shortest paths (empty waypoints → `backward_dijkstra` + `route_ecmp`). With 2000 demands all routed via ECMP shortest paths (empty solution), link 1173 (12→36, cap=1513) becomes overloaded.

The RP-403 repair operator assigns SR paths to only 425/2000 demands (21.25%). The remaining 1575 demands (78.75%) are routed via default ECMP shortest paths. These 1575 unassigned demands concentrate traffic on link 1173, causing `sat >= 1.0` → `inv_load_cost = f64::INFINITY`.

**Root cause:** The repair operator (RP-401C) assigns SR paths to an insufficient fraction of demands. The 1575 unassigned demands route via ECMP shortest paths, which concentrate traffic on link 1173 (12→36, cap=1513), overloading it by 0.11 units and making the objective infinite.

---

## 5. Evaluator Behaviour Classification

The diagnostic distinguishes between three possible failure modes:

| Failure mode | Evidence | Conclusion |
|-------------|----------|------------|
| Graph disconnection | 0/2000 demands unreachable at t=0 and t=1 | **Ruled out** |
| Aggregate capacity shortage | Volume/capacity ratio = 0.0019 | **Ruled out** |
| ECMP concentration on bottleneck link | Link 1173 overloaded by 0.11 units at t=0 | **Confirmed** |

The infeasibility is **not** inherent to the instance. The instance has sufficient aggregate capacity and full graph connectivity. The infeasibility is caused by the repair operator's failure to assign SR paths to enough demands to prevent ECMP concentration on link 1173.

---

## 6. Objective Component Analysis

The evaluator objective is:

```
obj = sum over time slots of (MLU + inv_load_cost)
```

where:
- `MLU` = maximum link utilisation across all links
- `inv_load_cost` = sum of `1/(1-sat) - 1` for each link with `0 < sat < 1`, plus `f64::INFINITY` for each link with `sat >= 1.0`

For setA-17 at t=0:
- `MLU` = 1.0001 (link 1173)
- `inv_load_cost` = `f64::INFINITY` (link 1173, sat=1.0001 >= 1.0)
- `obj` = `f64::INFINITY`

The first objective component to become infinite is `inv_load_cost` on link 1173 at t=0. This is an intentional hard-feasibility rule in the evaluator, not an implementation defect.

---

## 7. Fix Direction for RP-406B

To achieve a finite objective on setA-17, the repair operator must ensure that link 1173 (12→36, cap=1513) is not overloaded at t=0. This requires one or both of:

1. **Assign SR paths to all demands that traverse link 1173 under default ECMP routing** — reroute them via explicit waypoints that avoid link 1173.
2. **Assign SR paths to all 2000 demands** — eliminate default ECMP routing entirely, giving the solver full control over traffic distribution.

The current RP-401C repair operator assigns paths to only 425/2000 demands. The RP-406B programme should implement a repair operator that assigns SR paths to all demands, or at minimum to all demands that traverse the bottleneck link under default ECMP routing.

---

## 8. Programme Progression

| Programme | Focus | Status |
|-----------|-------|--------|
| RP-406A | Instance diagnostics — identify root cause | **Complete (this report)** |
| RP-406B | Algorithm design — full-coverage repair operator | Pending |
| RP-406C | Benchmark — evaluate RP-406B on setA-17 and full setA | Pending |

---

## 9. Commits

| Hash | Description |
|------|-------------|
| `f7a0e2bd` | `rp406_setA17_diag.rs` diagnostic binary + Cargo.toml update |

---

## 10. Amendment Log

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| v1.0 | 2026-08-04 | Research Programme | Initial report — RP-406A diagnostics complete |