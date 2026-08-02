# RP-401C — ECMP-Aware Construction Report

**Status:** Complete (binary written; awaiting execution results)  
**Date:** 2026-08-02  
**Experiment:** RP-401C (ECMP-Aware Load Estimation During Construction)  
**Binary:** `adapters/roadef/src/bin/rp401c_ecmp_construction.rs`

---

## 1. Research Question

Does replacing the heuristic link-load model with the ECMP oracle during
greedy construction improve solution quality on Dataset A?

---

## 2. Design

### 2.1 Change from Baseline

| Aspect | Baseline (`campaign_engine`) | RP-401C |
|--------|------------------------------|---------|
| Load model during construction | Heuristic: full demand volume on chosen path | ECMP oracle: `compute_loads()` on partial solution |
| Path selection | Load-aware Dijkstra with heuristic saturation | Load-aware Dijkstra with ECMP saturation |
| Penalty function | Identical | Identical |
| Path strategy | Shared-path (t=0 = t=1) | Shared-path (t=0 = t=1) |
| Budget guarantee | Zero (shared-path) | Zero (shared-path) |

### 2.2 Algorithm

```
partial_solution = []
ecmp_saturation = {link_id: 0.0 for all links}

for demand in sorted_by_volume_desc:
    path = load_aware_dijkstra(ecmp_saturation)   // ECMP-accurate input
    partial_solution.append(path)
    loads = evaluator.compute_loads(ts, partial_solution)  // ECMP oracle
    ecmp_saturation = {link: loads.arc_flows[link] / capacity[link]}
```

### 2.3 Complexity

- Baseline: O(D × Dijkstra) construction
- RP-401C: O(D × (Dijkstra + compute_loads)) = O(D × (Dijkstra + D × Dijkstra))
  = O(D² × Dijkstra)

For Dataset A (D ≤ 200, |V| ≤ 128): acceptable. Estimated runtime < 5s per
instance vs < 0.1s for baseline.

---

## 3. Expected Results

Based on the RP-401B divergence analysis:

| Instance class | Expected improvement |
|----------------|---------------------|
| Small (setA-01–05) | Minimal (low ECMP fan-out) |
| Medium (setA-06–12) | 5–20% objective reduction |
| Large (setA-13–20) | 10–40% objective reduction |

Instances currently falling back to empty (due to heuristic over-saturation)
may become solvable with ECMP-accurate construction.

---

## 4. Evidence Record

| Field | Value |
|-------|-------|
| Experiment | RP-401C |
| Status | Binary written; execution pending |
| Research Question | Does ECMP-oracle construction improve Dataset A results? |
| Baseline | `campaign_engine.rs` heuristic construction |
| Metric | Objective value per instance; count of improved/regressed instances |
| Result | Pending execution of `rp401c_ecmp_construction` binary |
| Runtime | Estimated < 5s per instance (O(D²) oracle calls) |
| Statistical Confidence | Deterministic — same random seed not applicable (greedy) |
| Platform Impact | If confirmed: ECMP-aware construction becomes standard for RP-402+ |
| Decision | Pending results; proceed to RP-401D in parallel |
| Key Files | `adapters/roadef/src/bin/rp401c_ecmp_construction.rs` |

---

## 5. Implementation Notes

### 5.1 Oracle Call Frequency

`compute_loads()` is called once per demand assignment. For D=200 demands,
this is 200 oracle calls per instance. Each oracle call runs O(D × Dijkstra),
so total construction cost is O(D² × Dijkstra). This is acceptable for
research purposes; a production solver would cache intermediate results.

### 5.2 Partial Solution Consistency

The partial solution passed to `compute_loads()` contains only the demands
assigned so far. Unassigned demands are routed via ECMP default (no waypoints).
This means the oracle sees a mix of assigned and unassigned demands, which
accurately reflects the state of the network at each construction step.

### 5.3 Fallback Behaviour

If `compute_loads()` returns `None` (disconnected demand), the previous
saturation map is retained. This prevents a single disconnected demand from
corrupting the saturation state for subsequent demands.

### 5.4 Output Files

The binary writes `setA-{inst}-srpaths-rp401c.json` alongside the existing
`setA-{inst}-srpaths.json` (baseline). This allows direct comparison without
overwriting the competition submission.