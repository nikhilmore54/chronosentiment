use crate::ecmp::{backward_dijkstra, expand_sr_path, route_ecmp, DijkstraResult};
use crate::graph::Digraph;
use crate::models::{Network, Scenario, Solution, TrafficMatrix};
use crate::path::SrPathBit;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::Instant;

/// Type alias for the Phase 6 cross-evaluation L2 Dijkstra cache.
///
/// Key: `(target_node, time_slot)` — genome-independent (disabled_arcs is scenario-only).
/// Value: `DijkstraResult` — cloned on insertion, served by reference on hit.
///
/// Thread-safe: `Arc<RwLock<...>>` allows concurrent reads (Rayon) and exclusive writes.
/// No eviction: run-local, bounded by `num_unique_targets × num_time_slots`.
pub type L2DijkstraCache = Arc<RwLock<HashMap<(u64, usize), DijkstraResult>>>;

// ---------------------------------------------------------------------------
// M20 Phase 1 — Evaluator Performance Model instrumentation
// M20 Phase 2 — Cache statistics and routing sub-stage timers
// Passive: records nanoseconds per stage, no logic changes to evaluate_solution().
// ---------------------------------------------------------------------------

/// Accumulated timing counters for one call to `evaluate_solution_timed()` or
/// `evaluate_solution_cached()`.
/// All fields are nanoseconds measured with `std::time::Instant` (monotonic).
#[derive(Debug, Default, Clone)]
pub struct EvalTimings {
    /// Segment-count constraint check (O(srpaths)).
    pub segment_check_ns: u64,
    /// Budget-cost check including SrPathBit construction and dist() calls
    /// (O(demands × time_slots)).
    pub budget_check_ns: u64,
    /// SR-path expansion via backward Dijkstra + ECMP flow routing
    /// (O(demands × dijkstra_runs)); the dominant cost.
    pub routing_ns: u64,
    /// Objective computation: saturation, MLU, Jain, inv-load-cost (O(links)).
    pub objective_ns: u64,
    /// Number of time slots processed (early-exit reduces this).
    pub time_slots_processed: u32,
    /// Number of demands routed across all time slots.
    pub demand_route_calls: u64,
    /// Number of Dijkstra invocations requested (= segments routed).
    pub dijkstra_calls: u64,

    // --- M20 Phase 2: routing sub-stage timers ---
    /// Time spent inside `backward_dijkstra()` only (subset of routing_ns).
    pub dijkstra_ns: u64,
    /// Time spent inside `route_ecmp()` only (subset of routing_ns).
    pub ecmp_ns: u64,

    // --- M20 Phase 2: cache statistics (only populated by evaluate_solution_cached) ---
    /// Dijkstra results served from cache (no recomputation).
    pub dijkstra_cache_hits: u64,
    /// Dijkstra results computed and inserted into cache.
    pub dijkstra_cache_misses: u64,
}

impl EvalTimings {
    pub fn total_ns(&self) -> u64 {
        self.segment_check_ns + self.budget_check_ns + self.routing_ns + self.objective_ns
    }

    /// Cache hit rate: fraction of Dijkstra requests served from cache.
    /// Returns 0.0 if no cache was used.
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.dijkstra_cache_hits + self.dijkstra_cache_misses;
        if total == 0 {
            0.0
        } else {
            self.dijkstra_cache_hits as f64 / total as f64
        }
    }

    /// Fraction of routing time spent in backward_dijkstra specifically.
    /// Returns 0.0 if routing_ns == 0.
    pub fn dijkstra_fraction(&self) -> f64 {
        if self.routing_ns == 0 {
            0.0
        } else {
            self.dijkstra_ns as f64 / self.routing_ns as f64
        }
    }

    /// Print a human-readable Performance Model table to stderr.
    pub fn print_report(&self, label: &str) {
        let total = self.total_ns().max(1) as f64;
        eprintln!("=== Evaluator Performance Model: {} ===", label);
        eprintln!("  {:30}  {:>10}  {:>6}", "Stage", "µs", "%");
        eprintln!("  {}", "-".repeat(52));
        let stages = [
            ("segment_check", self.segment_check_ns),
            ("budget_check", self.budget_check_ns),
            ("routing", self.routing_ns),
            ("  ↳ dijkstra", self.dijkstra_ns),
            ("  ↳ ecmp", self.ecmp_ns),
            ("objective", self.objective_ns),
        ];
        for (name, ns) in &stages {
            eprintln!(
                "  {:30}  {:>10.1}  {:>5.1}%",
                name,
                *ns as f64 / 1_000.0,
                *ns as f64 / total * 100.0
            );
        }
        eprintln!("  {}", "-".repeat(52));
        eprintln!("  {:30}  {:>10.1}", "TOTAL", total / 1_000.0);
        eprintln!(
            "  time_slots_processed: {}  demand_route_calls: {}  dijkstra_calls: {}",
            self.time_slots_processed, self.demand_route_calls, self.dijkstra_calls
        );
        if self.dijkstra_cache_hits + self.dijkstra_cache_misses > 0 {
            eprintln!(
                "  cache_hits: {}  cache_misses: {}  hit_rate: {:.1}%  dijkstra_fraction: {:.1}%",
                self.dijkstra_cache_hits,
                self.dijkstra_cache_misses,
                self.cache_hit_rate() * 100.0,
                self.dijkstra_fraction() * 100.0,
            );
        }
        eprintln!();
    }
}

pub struct RoadefEvaluator {
    pub graph: Digraph,
    pub tm: TrafficMatrix,
    pub scenario: Scenario,
}

pub struct EvaluationResult {
    pub valid: bool,
    pub obj: f64,
}

pub struct TimeSlotLoads {
    pub arc_flows: HashMap<u64, f64>,
    pub arc_saturations: HashMap<u64, f64>,
    pub mlu: f64,
    pub jain_index: f64,
    pub inv_load_cost: f64,
}

impl RoadefEvaluator {
    pub fn new(network: &Network, tm: TrafficMatrix, scenario: Scenario) -> Self {
        Self {
            graph: Digraph::new(network),
            tm,
            scenario,
        }
    }

    pub fn compute_loads(&self, time_slot: usize, solution: &Solution) -> Option<TimeSlotLoads> {
        let mut arc_flows: HashMap<u64, f64> = HashMap::new();

        let mut disabled_arcs = HashSet::new();
        if let Some(intervention) = self
            .scenario
            .interventions
            .iter()
            .find(|i| i.t == time_slot)
        {
            for &link_id in &intervention.links {
                disabled_arcs.insert(link_id);
            }
        }

        // Initialize flows to 0 for all arcs
        for arc in &self.graph.arcs {
            arc_flows.insert(arc.id, 0.0);
        }

        for (d_id, demand) in self.tm.demands.iter().enumerate() {
            let flow = demand.v[time_slot];
            if flow <= 0.0 {
                continue; // no traffic
            }

            // Find SR path for this demand at this time slot
            let mut waypoints: &[u64] = &[];
            if let Some(srpath) = solution
                .srpaths
                .iter()
                .find(|p| p.d == d_id && p.t == time_slot)
            {
                waypoints = &srpath.w;
            }

            let ok = expand_sr_path(
                &self.graph,
                demand.s,
                demand.t,
                waypoints,
                &disabled_arcs,
                flow,
                &mut arc_flows,
            );

            if !ok {
                // Disconnected demand
                return None;
            }
        }

        let mut arc_saturations = HashMap::new();
        let mut mlu = 0.0;
        let mut sum_sat = 0.0;
        let mut sum_sq_sat = 0.0;
        let mut inv_load_cost = 0.0;
        let mut count_sat = 0;

        for arc in &self.graph.arcs {
            let flow = *arc_flows.get(&arc.id).unwrap_or(&0.0);
            let capacity = arc.capacity;
            let sat = if capacity > 0.0 {
                flow / capacity
            } else {
                f64::INFINITY
            };
            arc_saturations.insert(arc.id, sat);
            if sat > mlu {
                mlu = sat;
            }

            sum_sat += sat;
            sum_sq_sat += sat * sat;
            count_sat += 1;

            if sat > 0.0 {
                // RC-001 FIX: use sat > 1.0 + 1e-6 instead of sat >= 1.0.
                // Floating-point accumulation in ECMP routing can produce sat=1.000000686
                // (flow=583.0004, cap=583.0000) which is physically feasible but was
                // incorrectly rejected. The 1e-6 epsilon matches the ROADEF checker's
                // tolerance and prevents false infeasibility from float rounding.
                if sat > 1.0 + 1e-6 {
                    inv_load_cost += f64::INFINITY;
                } else {
                    // Clamp sat to [0, 1-eps] before computing inv_load_cost to avoid
                    // division by zero or negative values from float near-equality.
                    let sat_clamped = sat.min(1.0 - 1e-9);
                    let f_sat = sat_clamped as f32;
                    let cost = (1.0 / (1.0 - f_sat as f64)) - 1.0;
                    inv_load_cost += cost;
                }
            }
        }

        let jain_index = if sum_sq_sat == 0.0 {
            0.0
        } else {
            let n = count_sat as f64;
            (sum_sat * sum_sat) / (n * sum_sq_sat)
        };

        Some(TimeSlotLoads {
            arc_flows,
            arc_saturations,
            mlu,
            jain_index,
            inv_load_cost,
        })
    }

    /// Diagnostic: returns a human-readable string describing the first reason
    /// a solution is invalid. Used to instrument greedy genome failures.
    /// Returns None if the solution is valid.
    pub fn diagnose_failure(&self, solution: &Solution) -> Option<String> {
        // Stage 1: segment check
        if self.scenario.max_segments >= 0 {
            for path in &solution.srpaths {
                if path.w.len() + 1 > self.scenario.max_segments as usize {
                    return Some(format!(
                        "segment_limit: demand={} t={} waypoints={} max_segments={}",
                        path.d,
                        path.t,
                        path.w.len(),
                        self.scenario.max_segments
                    ));
                }
            }
        }

        let mut prev_paths: HashMap<u64, SrPathBit> = HashMap::new();
        for ts in 0..self.tm.num_time_slots {
            // Stage 2: budget check
            let mut budget_cost = 0;
            let mut curr_paths: HashMap<u64, SrPathBit> = HashMap::new();
            for (d_id, demand) in self.tm.demands.iter().enumerate() {
                let d_id_u64 = d_id as u64;
                let mut bitpath = SrPathBit::new_uninitialized();
                if let Some(srpath) = solution.srpaths.iter().find(|p| p.d == d_id && p.t == ts) {
                    bitpath = SrPathBit::new_explicit(demand.s, demand.t, &srpath.w);
                }
                if ts > 0 {
                    let uninit = SrPathBit::new_uninitialized();
                    let prev_bitpath = prev_paths.get(&d_id_u64).unwrap_or(&uninit);
                    budget_cost += bitpath.dist(prev_bitpath);
                }
                curr_paths.insert(d_id_u64, bitpath);
            }
            if ts > 0 {
                let budget_val = self
                    .scenario
                    .budget
                    .iter()
                    .find(|b| b.t == ts)
                    .map(|b| b.value)
                    .unwrap_or(0);
                if budget_cost > budget_val {
                    return Some(format!(
                        "budget_exceeded: t={} budget_cost={} budget_val={}",
                        ts, budget_cost, budget_val
                    ));
                }
            }
            prev_paths = curr_paths;

            // Stage 3: routing check
            let mut disabled_arcs = HashSet::new();
            if let Some(intervention) = self.scenario.interventions.iter().find(|i| i.t == ts) {
                for &link_id in &intervention.links {
                    disabled_arcs.insert(link_id);
                }
            }
            let mut arc_flows: HashMap<u64, f64> = HashMap::new();
            for arc in &self.graph.arcs {
                arc_flows.insert(arc.id, 0.0);
            }
            for (d_id, demand) in self.tm.demands.iter().enumerate() {
                let flow = demand.v[ts];
                if flow <= 0.0 {
                    continue;
                }
                let mut waypoints: &[u64] = &[];
                if let Some(srpath) = solution.srpaths.iter().find(|p| p.d == d_id && p.t == ts) {
                    waypoints = &srpath.w;
                }
                let ok = expand_sr_path(
                    &self.graph,
                    demand.s,
                    demand.t,
                    waypoints,
                    &disabled_arcs,
                    flow,
                    &mut arc_flows,
                );
                if !ok {
                    return Some(format!(
                        "routing_failure: t={} demand={} src={} dst={} waypoints={:?} disabled_arcs={:?}",
                        ts, d_id, demand.s, demand.t, waypoints, disabled_arcs
                    ));
                }
            }

            // Stage 4: check for overloaded arcs (obj=inf)
            // Use sat > 1.0 - 1e-6 to catch floating-point near-equality at sat=1.0.
            // The evaluator's inv_load_cost formula 1/(1-sat)-1 diverges at sat=1.0,
            // so sat >= 1.0 produces obj=inf → valid=false.
            for arc in &self.graph.arcs {
                let flow = *arc_flows.get(&arc.id).unwrap_or(&0.0);
                let sat = if arc.capacity > 0.0 {
                    flow / arc.capacity
                } else {
                    f64::INFINITY
                };
                if sat >= 1.0 - 1e-6 {
                    return Some(format!(
                        "arc_overloaded: t={} arc={} flow={:.9} cap={:.9} sat={:.9}",
                        ts, arc.id, flow, arc.capacity, sat
                    ));
                }
            }
        }
        None // solution is valid
    }

    /// Instrumented variant of `evaluate_solution()`.
    /// Identical logic; wraps each stage with `Instant::now()` to populate
    /// `EvalTimings`. The unmodified `evaluate_solution()` is unchanged.
    pub fn evaluate_solution_timed(&self, solution: &Solution) -> (EvaluationResult, EvalTimings) {
        let mut t = EvalTimings::default();

        // --- Stage 1: segment check ---
        let t0 = Instant::now();
        if self.scenario.max_segments >= 0 {
            for path in &solution.srpaths {
                if path.w.len() + 1 > self.scenario.max_segments as usize {
                    t.segment_check_ns += t0.elapsed().as_nanos() as u64;
                    return (
                        EvaluationResult {
                            valid: false,
                            obj: f64::INFINITY,
                        },
                        t,
                    );
                }
            }
        }
        t.segment_check_ns += t0.elapsed().as_nanos() as u64;

        let mut total_obj = 0.0;
        let mut prev_paths: HashMap<u64, SrPathBit> = HashMap::new();

        for ts in 0..self.tm.num_time_slots {
            // --- Stage 2: budget check ---
            let t1 = Instant::now();
            let mut budget_cost = 0;
            let mut curr_paths: HashMap<u64, SrPathBit> = HashMap::new();

            for (d_id, demand) in self.tm.demands.iter().enumerate() {
                let d_id_u64 = d_id as u64;
                let mut bitpath = SrPathBit::new_uninitialized();
                if let Some(srpath) = solution.srpaths.iter().find(|p| p.d == d_id && p.t == ts) {
                    bitpath = SrPathBit::new_explicit(demand.s, demand.t, &srpath.w);
                }
                if ts > 0 {
                    let uninit = SrPathBit::new_uninitialized();
                    let prev_bitpath = prev_paths.get(&d_id_u64).unwrap_or(&uninit);
                    budget_cost += bitpath.dist(prev_bitpath);
                }
                curr_paths.insert(d_id_u64, bitpath);
            }
            if ts > 0 {
                let budget_val = self
                    .scenario
                    .budget
                    .iter()
                    .find(|b| b.t == ts)
                    .map(|b| b.value)
                    .unwrap_or(0);
                if budget_cost > budget_val {
                    t.budget_check_ns += t1.elapsed().as_nanos() as u64;
                    t.time_slots_processed += ts as u32;
                    return (
                        EvaluationResult {
                            valid: false,
                            obj: f64::INFINITY,
                        },
                        t,
                    );
                }
            }
            prev_paths = curr_paths;
            t.budget_check_ns += t1.elapsed().as_nanos() as u64;

            // --- Stage 3: routing (expand_sr_path / Dijkstra) ---
            let t2 = Instant::now();
            let mut arc_flows: HashMap<u64, f64> = HashMap::new();
            let mut disabled_arcs = HashSet::new();
            if let Some(intervention) = self.scenario.interventions.iter().find(|i| i.t == ts) {
                for &link_id in &intervention.links {
                    disabled_arcs.insert(link_id);
                }
            }
            for arc in &self.graph.arcs {
                arc_flows.insert(arc.id, 0.0);
            }

            let mut routing_ok = true;
            for (d_id, demand) in self.tm.demands.iter().enumerate() {
                let flow = demand.v[ts];
                if flow <= 0.0 {
                    continue;
                }
                let mut waypoints: &[u64] = &[];
                if let Some(srpath) = solution.srpaths.iter().find(|p| p.d == d_id && p.t == ts) {
                    waypoints = &srpath.w;
                }
                // Count Dijkstra calls: 1 per segment (waypoints.len() + 1)
                t.dijkstra_calls += (waypoints.len() as u64) + 1;
                t.demand_route_calls += 1;
                let ok = expand_sr_path(
                    &self.graph,
                    demand.s,
                    demand.t,
                    waypoints,
                    &disabled_arcs,
                    flow,
                    &mut arc_flows,
                );
                if !ok {
                    routing_ok = false;
                    break;
                }
            }
            t.routing_ns += t2.elapsed().as_nanos() as u64;

            if !routing_ok {
                t.time_slots_processed += ts as u32 + 1;
                return (
                    EvaluationResult {
                        valid: false,
                        obj: f64::INFINITY,
                    },
                    t,
                );
            }

            // --- Stage 4: objective computation ---
            let t3 = Instant::now();
            let mut mlu = 0.0f64;
            let mut inv_load_cost = 0.0f64;
            for arc in &self.graph.arcs {
                let flow = *arc_flows.get(&arc.id).unwrap_or(&0.0);
                let capacity = arc.capacity;
                let sat = if capacity > 0.0 {
                    flow / capacity
                } else {
                    f64::INFINITY
                };
                if sat > mlu {
                    mlu = sat;
                }
                if sat > 0.0 {
                    if sat > 1.0 + 1e-6 {
                        inv_load_cost += f64::INFINITY;
                    } else {
                        let sat_clamped = sat.min(1.0 - 1e-9);
                        let f_sat = sat_clamped as f32;
                        inv_load_cost += (1.0 / (1.0 - f_sat as f64)) - 1.0;
                    }
                }
            }
            total_obj += mlu + inv_load_cost;
            t.objective_ns += t3.elapsed().as_nanos() as u64;
        }

        t.time_slots_processed = self.tm.num_time_slots as u32;
        (
            EvaluationResult {
                valid: true,
                obj: total_obj,
            },
            t,
        )
    }

    /// M20 Phase 2 — Cached evaluator.
    ///
    /// Semantically identical to `evaluate_solution()`. Caches `DijkstraResult`
    /// per `(target_node, time_slot)` within a single evaluation call, eliminating
    /// redundant shortest-path computation for demands sharing the same destination.
    ///
    /// Cache key validity: `disabled_arcs` is determined solely by
    /// `self.scenario.interventions` (scenario data, immutable, genome-independent),
    /// so the result for `(target_node, time_slot)` is identical for all demands
    /// in that time slot. See RP-310-DESIGN-SPEC-v1.0.md §4.1.
    ///
    /// Returns `(EvaluationResult, EvalTimings)` with cache statistics populated.
    pub fn evaluate_solution_cached(&self, solution: &Solution) -> (EvaluationResult, EvalTimings) {
        let mut t = EvalTimings::default();

        // --- Stage 1: segment check ---
        let t0 = Instant::now();
        if self.scenario.max_segments >= 0 {
            for path in &solution.srpaths {
                if path.w.len() + 1 > self.scenario.max_segments as usize {
                    t.segment_check_ns += t0.elapsed().as_nanos() as u64;
                    return (
                        EvaluationResult {
                            valid: false,
                            obj: f64::INFINITY,
                        },
                        t,
                    );
                }
            }
        }
        t.segment_check_ns += t0.elapsed().as_nanos() as u64;

        let mut total_obj = 0.0;
        let mut prev_paths: HashMap<u64, SrPathBit> = HashMap::new();

        for ts in 0..self.tm.num_time_slots {
            // --- Stage 2: budget check (unchanged from reference evaluator) ---
            let t1 = Instant::now();
            let mut budget_cost = 0;
            let mut curr_paths: HashMap<u64, SrPathBit> = HashMap::new();

            for (d_id, demand) in self.tm.demands.iter().enumerate() {
                let d_id_u64 = d_id as u64;
                let mut bitpath = SrPathBit::new_uninitialized();
                if let Some(srpath) = solution.srpaths.iter().find(|p| p.d == d_id && p.t == ts) {
                    bitpath = SrPathBit::new_explicit(demand.s, demand.t, &srpath.w);
                }
                if ts > 0 {
                    let uninit = SrPathBit::new_uninitialized();
                    let prev_bitpath = prev_paths.get(&d_id_u64).unwrap_or(&uninit);
                    budget_cost += bitpath.dist(prev_bitpath);
                }
                curr_paths.insert(d_id_u64, bitpath);
            }
            if ts > 0 {
                let budget_val = self
                    .scenario
                    .budget
                    .iter()
                    .find(|b| b.t == ts)
                    .map(|b| b.value)
                    .unwrap_or(0);
                if budget_cost > budget_val {
                    t.budget_check_ns += t1.elapsed().as_nanos() as u64;
                    t.time_slots_processed += ts as u32;
                    return (
                        EvaluationResult {
                            valid: false,
                            obj: f64::INFINITY,
                        },
                        t,
                    );
                }
            }
            prev_paths = curr_paths;
            t.budget_check_ns += t1.elapsed().as_nanos() as u64;

            // --- Stage 3: routing with per-time-slot Dijkstra cache ---
            //
            // Cache key: target_node (u64).
            // disabled_arcs is scenario-only, fixed for this time slot.
            // One cache per time slot; dropped at end of loop iteration.
            let t2 = Instant::now();

            let mut disabled_arcs: HashSet<u64> = HashSet::new();
            if let Some(intervention) = self.scenario.interventions.iter().find(|i| i.t == ts) {
                for &link_id in &intervention.links {
                    disabled_arcs.insert(link_id);
                }
            }

            // Per-time-slot Dijkstra cache: target_node → DijkstraResult
            let mut dijkstra_cache: HashMap<u64, DijkstraResult> = HashMap::new();

            let mut arc_flows: HashMap<u64, f64> = HashMap::new();
            for arc in &self.graph.arcs {
                arc_flows.insert(arc.id, 0.0);
            }

            let mut routing_ok = true;

            for (d_id, demand) in self.tm.demands.iter().enumerate() {
                let flow = demand.v[ts];
                if flow <= 0.0 {
                    continue;
                }

                let mut waypoints: &[u64] = &[];
                if let Some(srpath) = solution.srpaths.iter().find(|p| p.d == d_id && p.t == ts) {
                    waypoints = &srpath.w;
                }

                t.demand_route_calls += 1;

                // Build the segment list: [source, wp0, wp1, ..., target]
                // Route each consecutive pair using cached Dijkstra.
                let mut segment_nodes: Vec<u64> = Vec::with_capacity(waypoints.len() + 2);
                segment_nodes.push(demand.s);
                segment_nodes.extend_from_slice(waypoints);
                segment_nodes.push(demand.t);

                let mut demand_ok = true;
                for i in 0..segment_nodes.len() - 1 {
                    let seg_src = segment_nodes[i];
                    let seg_tgt = segment_nodes[i + 1];
                    if seg_src == seg_tgt {
                        continue;
                    }

                    t.dijkstra_calls += 1;

                    // Cache lookup
                    if !dijkstra_cache.contains_key(&seg_tgt) {
                        // Cache miss: run Dijkstra and store result
                        t.dijkstra_cache_misses += 1;
                        let td = Instant::now();
                        let result = backward_dijkstra(&self.graph, seg_tgt, &disabled_arcs);
                        t.dijkstra_ns += td.elapsed().as_nanos() as u64;
                        dijkstra_cache.insert(seg_tgt, result);
                    } else {
                        t.dijkstra_cache_hits += 1;
                    }

                    let dijkstra_result = dijkstra_cache.get(&seg_tgt).unwrap();

                    // ECMP flow routing using cached result
                    let te = Instant::now();
                    let ok = route_ecmp(
                        &self.graph,
                        dijkstra_result,
                        seg_src,
                        seg_tgt,
                        flow,
                        &mut arc_flows,
                    );
                    t.ecmp_ns += te.elapsed().as_nanos() as u64;

                    if !ok {
                        demand_ok = false;
                        break;
                    }
                }

                if !demand_ok {
                    routing_ok = false;
                    break;
                }
            }

            t.routing_ns += t2.elapsed().as_nanos() as u64;

            if !routing_ok {
                t.time_slots_processed += ts as u32 + 1;
                return (
                    EvaluationResult {
                        valid: false,
                        obj: f64::INFINITY,
                    },
                    t,
                );
            }

            // --- Stage 4: objective computation (identical to reference) ---
            let t3 = Instant::now();
            let mut mlu = 0.0f64;
            let mut inv_load_cost = 0.0f64;
            for arc in &self.graph.arcs {
                let flow = *arc_flows.get(&arc.id).unwrap_or(&0.0);
                let capacity = arc.capacity;
                let sat = if capacity > 0.0 {
                    flow / capacity
                } else {
                    f64::INFINITY
                };
                if sat > mlu {
                    mlu = sat;
                }
                if sat > 0.0 {
                    if sat > 1.0 + 1e-6 {
                        inv_load_cost += f64::INFINITY;
                    } else {
                        let sat_clamped = sat.min(1.0 - 1e-9);
                        let f_sat = sat_clamped as f32;
                        inv_load_cost += (1.0 / (1.0 - f_sat as f64)) - 1.0;
                    }
                }
            }
            total_obj += mlu + inv_load_cost;
            t.objective_ns += t3.elapsed().as_nanos() as u64;
        }

        t.time_slots_processed = self.tm.num_time_slots as u32;
        (
            EvaluationResult {
                valid: true,
                obj: total_obj,
            },
            t,
        )
    }

    /// M20 Phase 6 — L2 cross-evaluation Dijkstra cache.
    ///
    /// Semantically identical to `evaluate_solution_cached()`. Additionally accepts a
    /// run-level `L2DijkstraCache` that persists `DijkstraResult` across evaluations.
    ///
    /// Cache key validity: `(target_node, time_slot)` is genome-independent because
    /// `disabled_arcs` is determined solely by `scenario.interventions[time_slot]`
    /// (immutable scenario data). See GERAD_PHASE5_EVALUATOR_PROFILE.md §4.3.
    ///
    /// L2 hit: serve cached `DijkstraResult` (read lock, no recomputation).
    /// L2 miss: run `backward_dijkstra`, clone result into L2 cache (write lock).
    ///
    /// The within-evaluation L1 Dijkstra cache (from `evaluate_solution_cached`) is
    /// retained as a fast path for the common case where multiple demands share the
    /// same target node within a single evaluation call.
    pub fn evaluate_solution_l2cached(
        &self,
        solution: &Solution,
        l2_cache: &L2DijkstraCache,
    ) -> (EvaluationResult, EvalTimings) {
        let mut t = EvalTimings::default();

        // --- Stage 1: segment check ---
        let t0 = Instant::now();
        if self.scenario.max_segments >= 0 {
            for path in &solution.srpaths {
                if path.w.len() + 1 > self.scenario.max_segments as usize {
                    t.segment_check_ns += t0.elapsed().as_nanos() as u64;
                    return (
                        EvaluationResult {
                            valid: false,
                            obj: f64::INFINITY,
                        },
                        t,
                    );
                }
            }
        }
        t.segment_check_ns += t0.elapsed().as_nanos() as u64;

        let mut total_obj = 0.0;
        let mut prev_paths: HashMap<u64, SrPathBit> = HashMap::new();

        for ts in 0..self.tm.num_time_slots {
            // --- Stage 2: budget check ---
            let t1 = Instant::now();
            let mut budget_cost = 0;
            let mut curr_paths: HashMap<u64, SrPathBit> = HashMap::new();

            for (d_id, demand) in self.tm.demands.iter().enumerate() {
                let d_id_u64 = d_id as u64;
                let mut bitpath = SrPathBit::new_uninitialized();
                if let Some(srpath) = solution.srpaths.iter().find(|p| p.d == d_id && p.t == ts) {
                    bitpath = SrPathBit::new_explicit(demand.s, demand.t, &srpath.w);
                }
                if ts > 0 {
                    let uninit = SrPathBit::new_uninitialized();
                    let prev_bitpath = prev_paths.get(&d_id_u64).unwrap_or(&uninit);
                    budget_cost += bitpath.dist(prev_bitpath);
                }
                curr_paths.insert(d_id_u64, bitpath);
            }
            if ts > 0 {
                let budget_val = self
                    .scenario
                    .budget
                    .iter()
                    .find(|b| b.t == ts)
                    .map(|b| b.value)
                    .unwrap_or(0);
                if budget_cost > budget_val {
                    t.budget_check_ns += t1.elapsed().as_nanos() as u64;
                    t.time_slots_processed += ts as u32;
                    return (
                        EvaluationResult {
                            valid: false,
                            obj: f64::INFINITY,
                        },
                        t,
                    );
                }
            }
            prev_paths = curr_paths;
            t.budget_check_ns += t1.elapsed().as_nanos() as u64;

            // --- Stage 3: routing with L2 cross-evaluation + L1 within-evaluation Dijkstra cache ---
            let t2 = Instant::now();

            let mut disabled_arcs: HashSet<u64> = HashSet::new();
            if let Some(intervention) = self.scenario.interventions.iter().find(|i| i.t == ts) {
                for &link_id in &intervention.links {
                    disabled_arcs.insert(link_id);
                }
            }

            // Within-evaluation L1 Dijkstra cache (target_node → DijkstraResult).
            // Serves as a fast path for demands sharing the same target within this call.
            let mut local_cache: HashMap<u64, DijkstraResult> = HashMap::new();

            let mut arc_flows: HashMap<u64, f64> = HashMap::new();
            for arc in &self.graph.arcs {
                arc_flows.insert(arc.id, 0.0);
            }

            let mut routing_ok = true;

            for (d_id, demand) in self.tm.demands.iter().enumerate() {
                let flow = demand.v[ts];
                if flow <= 0.0 {
                    continue;
                }

                let mut waypoints: &[u64] = &[];
                if let Some(srpath) = solution.srpaths.iter().find(|p| p.d == d_id && p.t == ts) {
                    waypoints = &srpath.w;
                }

                t.demand_route_calls += 1;

                let mut segment_nodes: Vec<u64> = Vec::with_capacity(waypoints.len() + 2);
                segment_nodes.push(demand.s);
                segment_nodes.extend_from_slice(waypoints);
                segment_nodes.push(demand.t);

                let mut demand_ok = true;
                for i in 0..segment_nodes.len() - 1 {
                    let seg_src = segment_nodes[i];
                    let seg_tgt = segment_nodes[i + 1];
                    if seg_src == seg_tgt {
                        continue;
                    }

                    t.dijkstra_calls += 1;

                    // L1 within-evaluation cache check (fast path, no lock).
                    if !local_cache.contains_key(&seg_tgt) {
                        // L1 miss: check L2 cross-evaluation cache.
                        let l2_key = (seg_tgt, ts);
                        let l2_hit = {
                            let guard = l2_cache.read().unwrap();
                            guard.get(&l2_key).cloned()
                        };

                        let result = if let Some(cached) = l2_hit {
                            // L2 hit: serve from cross-evaluation cache.
                            t.dijkstra_cache_hits += 1;
                            cached
                        } else {
                            // L2 miss: run Dijkstra, insert into both caches.
                            t.dijkstra_cache_misses += 1;
                            let td = Instant::now();
                            let r = backward_dijkstra(&self.graph, seg_tgt, &disabled_arcs);
                            t.dijkstra_ns += td.elapsed().as_nanos() as u64;
                            // Insert into L2 (cross-evaluation, shared).
                            {
                                let mut guard = l2_cache.write().unwrap();
                                guard.entry(l2_key).or_insert_with(|| r.clone());
                            }
                            r
                        };
                        local_cache.insert(seg_tgt, result);
                    } else {
                        // L1 hit (within-evaluation).
                        t.dijkstra_cache_hits += 1;
                    }

                    let dijkstra_result = local_cache.get(&seg_tgt).unwrap();

                    let te = Instant::now();
                    let ok = route_ecmp(
                        &self.graph,
                        dijkstra_result,
                        seg_src,
                        seg_tgt,
                        flow,
                        &mut arc_flows,
                    );
                    t.ecmp_ns += te.elapsed().as_nanos() as u64;

                    if !ok {
                        demand_ok = false;
                        break;
                    }
                }

                if !demand_ok {
                    routing_ok = false;
                    break;
                }
            }

            t.routing_ns += t2.elapsed().as_nanos() as u64;

            if !routing_ok {
                t.time_slots_processed += ts as u32 + 1;
                return (
                    EvaluationResult {
                        valid: false,
                        obj: f64::INFINITY,
                    },
                    t,
                );
            }

            // --- Stage 4: objective computation ---
            let t3 = Instant::now();
            let mut mlu = 0.0f64;
            let mut inv_load_cost = 0.0f64;
            for arc in &self.graph.arcs {
                let flow = *arc_flows.get(&arc.id).unwrap_or(&0.0);
                let capacity = arc.capacity;
                let sat = if capacity > 0.0 {
                    flow / capacity
                } else {
                    f64::INFINITY
                };
                if sat > mlu {
                    mlu = sat;
                }
                if sat > 0.0 {
                    if sat > 1.0 + 1e-6 {
                        inv_load_cost += f64::INFINITY;
                    } else {
                        let sat_clamped = sat.min(1.0 - 1e-9);
                        let f_sat = sat_clamped as f32;
                        inv_load_cost += (1.0 / (1.0 - f_sat as f64)) - 1.0;
                    }
                }
            }
            total_obj += mlu + inv_load_cost;
            t.objective_ns += t3.elapsed().as_nanos() as u64;
        }

        t.time_slots_processed = self.tm.num_time_slots as u32;
        (
            EvaluationResult {
                valid: true,
                obj: total_obj,
            },
            t,
        )
    }

    pub fn evaluate_solution(&self, solution: &Solution) -> EvaluationResult {
        // maxSegments check
        if self.scenario.max_segments >= 0 {
            for path in &solution.srpaths {
                // waypoints length + 1 (since segments = waypoints + 1 if we consider the full path... wait, no.)
                // Actually the number of segments is len(waypoints) + 1.
                if path.w.len() + 1 > self.scenario.max_segments as usize {
                    return EvaluationResult {
                        valid: false,
                        obj: f64::INFINITY,
                    };
                }
            }
        }

        let mut total_obj = 0.0;
        let mut prev_paths: HashMap<u64, SrPathBit> = HashMap::new();

        for t in 0..self.tm.num_time_slots {
            // Budget check
            let mut budget_cost = 0;
            let mut curr_paths: HashMap<u64, SrPathBit> = HashMap::new();

            for (d_id, demand) in self.tm.demands.iter().enumerate() {
                let d_id_u64 = d_id as u64;
                let mut bitpath = SrPathBit::new_uninitialized();

                if let Some(srpath) = solution.srpaths.iter().find(|p| p.d == d_id && p.t == t) {
                    bitpath = SrPathBit::new_explicit(demand.s, demand.t, &srpath.w);
                }

                if t > 0 {
                    let uninit = SrPathBit::new_uninitialized();
                    let prev_bitpath = prev_paths.get(&d_id_u64).unwrap_or(&uninit);
                    budget_cost += bitpath.dist(prev_bitpath);
                }

                curr_paths.insert(d_id_u64, bitpath);
            }

            if t > 0 {
                let budget_val = self
                    .scenario
                    .budget
                    .iter()
                    .find(|b| b.t == t)
                    .map(|b| b.value)
                    .unwrap_or(0);
                if budget_cost > budget_val {
                    return EvaluationResult {
                        valid: false,
                        obj: f64::INFINITY,
                    };
                }
            }

            prev_paths = curr_paths;

            // Load and connectivity check
            if let Some(loads) = self.compute_loads(t, solution) {
                total_obj += loads.mlu + loads.inv_load_cost;
            } else {
                // Connectivity failed
                return EvaluationResult {
                    valid: false,
                    obj: f64::INFINITY,
                };
            }
        }

        EvaluationResult {
            valid: true,
            obj: total_obj,
        }
    }

    /// RC-003: Compute the official ROADEF lexicographic objective vector for a solution.
    ///
    /// The official objective is the sorted (descending) vector of all per-link saturations
    /// across all time slots. Two solutions are compared by this vector lexicographically:
    /// the solution with the lower value at the first differing rank wins.
    ///
    /// Returns `None` if the solution is infeasible (connectivity failure or budget violation).
    /// Returns `Some(vec)` where `vec` is sorted descending (rank-1 = highest saturation first).
    ///
    /// This method is used by RC-003 to validate that the surrogate scalar objective
    /// (`Σ_t MLU_t + inv_load_cost_t`) preserves the official lexicographic ordering.
    pub fn compute_lex_vector(&self, solution: &Solution) -> Option<Vec<f64>> {
        // Segment count check (same as evaluate_solution)
        if self.scenario.max_segments >= 0 {
            for path in &solution.srpaths {
                if path.w.len() + 1 > self.scenario.max_segments as usize {
                    return None;
                }
            }
        }

        let mut all_saturations: Vec<f64> = Vec::new();
        let mut prev_paths: HashMap<u64, SrPathBit> = HashMap::new();

        for t in 0..self.tm.num_time_slots {
            // Budget check (same as evaluate_solution)
            let mut budget_cost = 0;
            let mut curr_paths: HashMap<u64, SrPathBit> = HashMap::new();

            for (d_id, demand) in self.tm.demands.iter().enumerate() {
                let d_id_u64 = d_id as u64;
                let mut bitpath = SrPathBit::new_uninitialized();

                if let Some(srpath) = solution.srpaths.iter().find(|p| p.d == d_id && p.t == t) {
                    bitpath = SrPathBit::new_explicit(demand.s, demand.t, &srpath.w);
                }

                if t > 0 {
                    let uninit = SrPathBit::new_uninitialized();
                    let prev_bitpath = prev_paths.get(&d_id_u64).unwrap_or(&uninit);
                    budget_cost += bitpath.dist(prev_bitpath);
                }

                curr_paths.insert(d_id_u64, bitpath);
            }

            if t > 0 {
                let budget_val = self
                    .scenario
                    .budget
                    .iter()
                    .find(|b| b.t == t)
                    .map(|b| b.value)
                    .unwrap_or(0);
                if budget_cost > budget_val {
                    return None;
                }
            }

            prev_paths = curr_paths;

            // Compute loads and collect all arc saturations for this time slot.
            match self.compute_loads(t, solution) {
                None => return None, // connectivity failure
                Some(loads) => {
                    for sat in loads.arc_saturations.values() {
                        all_saturations.push(*sat);
                    }
                }
            }
        }

        // Sort descending: rank-1 (highest saturation) first.
        all_saturations.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        Some(all_saturations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loader::{load_network, load_scenario, load_solution, load_traffic_matrix};

    #[test]
    fn test_compute_loads_set_a_01_empty_solution() {
        let net = load_network("repo/challenge-roadef-2026-main/setA/setA-01-net.json").unwrap();
        let tm =
            load_traffic_matrix("repo/challenge-roadef-2026-main/setA/setA-01-tm.json").unwrap();
        let scenario =
            load_scenario("repo/challenge-roadef-2026-main/setA/setA-01-scenario.json").unwrap();

        let evaluator = RoadefEvaluator::new(&net, tm, scenario);
        let empty_solution = Solution { srpaths: vec![] };

        let loads_t0 = evaluator.compute_loads(0, &empty_solution).unwrap();

        // We know from the C++ checker output:
        // "Maximum Link Utilization (MLU) at 0 : 1.0000006861063464"
        assert!((loads_t0.mlu - 1.000000686106).abs() < 1e-6);

        let loads_t1 = evaluator.compute_loads(1, &empty_solution).unwrap();
        // "Maximum Link Utilization (MLU) at 1 : 0.5663266666666666"
        assert!((loads_t1.mlu - 0.566326666666).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_solution_set_a_01() {
        let net = load_network("repo/challenge-roadef-2026-main/setA/setA-01-net.json").unwrap();
        let tm =
            load_traffic_matrix("repo/challenge-roadef-2026-main/setA/setA-01-tm.json").unwrap();
        let scenario =
            load_scenario("repo/challenge-roadef-2026-main/setA/setA-01-scenario.json").unwrap();

        let evaluator = RoadefEvaluator::new(&net, tm, scenario);
        let empty_solution = Solution { srpaths: vec![] };

        let result = evaluator.evaluate_solution(&empty_solution);
        assert!(result.valid);
        // The C++ checker says: Objective value: 64.99616053303649
        // This includes Jain's index, let's see if we get close.
        // Wait, C++ uses sum(mlu) + sum(inv_load) - (actually jain doesn't get summed in obj? Let's not assert exact obj yet)
    }
}
