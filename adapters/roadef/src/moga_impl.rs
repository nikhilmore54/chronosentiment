/// ROADEF 2026 MOGA Implementation
///
/// Wires the coralys-moga evolution engine to the ROADEF SR-path solution space.
///
/// Genome: RoadefGenome — a flat Vec<Vec<u64>> where index d is the waypoint list
///         for demand d (applied uniformly across all time slots in this baseline).
///         Empty waypoints = ECMP default path.
///
/// This is the M19 baseline. Per-time-slot waypoints are a Phase IV enhancement.

use rand::rngs::StdRng;
use rand::Rng;
use rand::seq::SliceRandom;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::sync::{Arc, RwLock};

use coralys_moga::traits::{
    Genome, GenomeFactory, FitnessEvaluator, Evaluated, MutationOperator, CrossoverOperator,
};

use crate::ecmp::expand_sr_path;
use crate::evaluator::RoadefEvaluator;
use crate::models::{Network, Solution, SrPath};
use crate::telemetry::{
    TelemetrySink, MoveRecord, GenerationRecord, ConstructionRecord, CandidateRecord,
    ZoneDeltas, sorted_load_vector, compute_sdi, ComparatorMode,
};

// ---------------------------------------------------------------------------
// RP-408A: Comparator trait + implementations
// ---------------------------------------------------------------------------

/// Trait for comparing two evaluated genomes.
///
/// Selection never knows which comparator it is using — both `ScalarComparator`
/// and `LexicographicComparator` implement this interface identically. The
/// comparator is selected once per run via `EvolutionRunConfig::comparator_mode`
/// and passed through the evolution loop as a `&dyn EvalComparator`.
///
/// The method returns `Ordering::Greater` when `lhs` is **better** than `rhs`
/// (i.e. should be preferred by selection). This matches the convention used
/// by `sort_by` with `b.cmp(a)` (descending sort, best first).
pub trait EvalComparator: Send + Sync {
    fn is_better(&self, lhs: &RoadefEvaluation, rhs: &RoadefEvaluation) -> bool;
    fn cmp_evals(&self, lhs: &RoadefEvaluation, rhs: &RoadefEvaluation) -> Ordering;
}

/// Scalar comparator: compares by `fitness()` (= −obj) only.
/// Invalid solutions always lose to valid ones.
/// This is the RP-406C baseline behaviour.
pub struct ScalarComparator;

impl EvalComparator for ScalarComparator {
    fn is_better(&self, lhs: &RoadefEvaluation, rhs: &RoadefEvaluation) -> bool {
        lhs.fitness() > rhs.fitness()
    }
    fn cmp_evals(&self, lhs: &RoadefEvaluation, rhs: &RoadefEvaluation) -> Ordering {
        lhs.fitness().partial_cmp(&rhs.fitness()).unwrap_or(Ordering::Equal)
    }
}

/// Lexicographic comparator: compares by the full sorted load vector
/// (descending), breaking ties zone by zone (Peak → Shoulder → Transition → Tail).
///
/// Invalid solutions always lose to valid ones (same as scalar).
/// Among valid solutions, the comparison is element-wise on the sorted load
/// vector: lower load at rank k is better (minimisation). The first rank at
/// which the two vectors differ determines the winner.
///
/// Introduced in RP-408.
pub struct LexicographicComparator;

impl EvalComparator for LexicographicComparator {
    fn is_better(&self, lhs: &RoadefEvaluation, rhs: &RoadefEvaluation) -> bool {
        self.cmp_evals(lhs, rhs) == Ordering::Greater
    }
    fn cmp_evals(&self, lhs: &RoadefEvaluation, rhs: &RoadefEvaluation) -> Ordering {
        // Invalid always loses to valid.
        match (lhs.valid, rhs.valid) {
            (false, false) => Ordering::Equal,
            (true,  false) => Ordering::Greater,
            (false, true)  => Ordering::Less,
            (true,  true)  => {
                // Lexicographic comparison on sorted load vector (lower = better).
                // We want "better" to map to Ordering::Greater (for sort_by descending).
                let lv = &lhs.load_vector;
                let rv = &rhs.load_vector;
                let len = lv.len().max(rv.len());
                for i in 0..len {
                    let l = lv.get(i).copied().unwrap_or(0.0);
                    let r = rv.get(i).copied().unwrap_or(0.0);
                    // Lower load is better → lhs better if l < r → return Greater
                    match r.partial_cmp(&l).unwrap_or(Ordering::Equal) {
                        Ordering::Equal => continue,
                        ord => return ord, // r > l → lhs better (Greater); r < l → lhs worse (Less)
                    }
                }
                Ordering::Equal
            }
        }
    }
}

/// Construct the comparator for a given `ComparatorMode`.
pub fn make_comparator(mode: ComparatorMode) -> Box<dyn EvalComparator> {
    match mode {
        ComparatorMode::Scalar       => Box::new(ScalarComparator),
        ComparatorMode::Lexicographic => Box::new(LexicographicComparator),
    }
}

// ---------------------------------------------------------------------------
// Genome
// ---------------------------------------------------------------------------

/// SR-path genome: one waypoint list per demand, applied to all time slots.
/// waypoints[d] = list of intermediate node IDs for demand d.
/// Empty = use ECMP default path.
#[derive(Clone, Debug)]
pub struct RoadefGenome {
    /// waypoints[d] = waypoint sequence for demand d
    pub waypoints: Vec<Vec<u64>>,
    /// Number of time slots (needed to expand into Solution)
    pub num_time_slots: usize,
}

impl Genome for RoadefGenome {}

impl RoadefGenome {
    /// Expand genome into a full Solution (one SrPath per demand per time slot).
    pub fn to_solution(&self) -> Solution {
        let mut srpaths = Vec::new();
        for (d, wps) in self.waypoints.iter().enumerate() {
            if wps.is_empty() {
                // Empty waypoints = ECMP default; no SrPath entry needed
                continue;
            }
            for t in 0..self.num_time_slots {
                srpaths.push(SrPath {
                    d,
                    t,
                    w: wps.clone(),
                });
            }
        }
        Solution { srpaths }
    }
}

// ---------------------------------------------------------------------------
// GenomeFactory
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// RC-001: Construction mode selection
// ---------------------------------------------------------------------------

/// Construction strategy for the initial population.
///
/// `Random` is the CB-000 baseline: 70% ECMP default, 30% one random waypoint.
/// `GreedyLoadAware` is the RC-001 improvement: volume-sorted, load-aware
/// Dijkstra with additive saturation penalty (RP-401C algorithm), with
/// per-genome demand-order perturbation for population diversity.
#[derive(Clone, Debug)]
pub enum ConstructionMode {
    /// CB-000 baseline: random waypoints, no topology awareness.
    Random,
    /// RC-001: load-aware greedy construction (RP-401C algorithm).
    GreedyLoadAware,
}

/// Data required by the greedy load-aware constructor.
/// Held inside `RoadefGenomeFactory` when `mode == GreedyLoadAware`.
pub struct GreedyConstructorData {
    /// Network topology (for adjacency and link capacities).
    pub network: Network,
    /// Evaluator (for `compute_loads` to update saturation after each demand).
    pub evaluator: Arc<RoadefEvaluator>,
    /// Demands as (demand_index, src, dst, max_volume_across_slots).
    /// Pre-sorted descending by volume; shuffled within bands per genome.
    pub demands_by_volume: Vec<(usize, u64, u64, f64)>,
    /// Maximum number of SR segments allowed (from Scenario::max_segments).
    pub max_segments: usize,
    /// Arc capacity map keyed by directed arc ID (same ID space as `arc_flows` from `compute_loads`).
    /// CORRECTNESS: Must be built from `evaluator.graph.arcs`, NOT from `net.links`.
    /// `arc_flows` uses `graph.arcs[i].id`; `net.links[i].id` is a different ID space on some topologies.
    /// Using `net.links` caused `link_capacity.get(arc_id)` to return None → cap defaulted to 1.0
    /// → sat = flow / 1.0 = flow → max_sat = 22.766 on setA-05 (RC-001A bug).
    pub link_capacity: HashMap<u64, f64>,
}

/// Creates random genomes by assigning 0–1 random waypoints per demand.
/// The waypoints are drawn from the set of valid node IDs in the network.
pub struct RoadefGenomeFactory {
    pub num_demands: usize,
    pub num_time_slots: usize,
    /// All node IDs in the network (for random waypoint selection in Random mode)
    pub node_ids: Vec<u64>,
    /// RC-001: construction mode. Default is `Random` (CB-000 baseline).
    pub mode: ConstructionMode,
    /// RC-001: greedy constructor data. Required when `mode == GreedyLoadAware`.
    pub greedy_data: Option<Arc<GreedyConstructorData>>,
}

impl GenomeFactory<RoadefGenome> for RoadefGenomeFactory {
    fn create(&self, rng: &mut StdRng) -> RoadefGenome {
        match self.mode {
            ConstructionMode::Random => self.create_random(rng),
            ConstructionMode::GreedyLoadAware => {
                if let Some(ref gd) = self.greedy_data {
                    self.create_greedy(rng, gd)
                } else {
                    // Fallback: greedy_data not provided, use random
                    self.create_random(rng)
                }
            }
        }
    }
}

impl RoadefGenomeFactory {
    // -----------------------------------------------------------------------
    // CB-000 baseline constructor (Random mode)
    // -----------------------------------------------------------------------
    fn create_random(&self, rng: &mut StdRng) -> RoadefGenome {
        let waypoints = (0..self.num_demands)
            .map(|_| {
                // 70% chance of empty (ECMP default), 30% chance of 1 random waypoint
                if rng.gen_bool(0.70) || self.node_ids.is_empty() {
                    vec![]
                } else {
                    let idx = rng.gen_range(0..self.node_ids.len());
                    vec![self.node_ids[idx]]
                }
            })
            .collect();
        RoadefGenome {
            waypoints,
            num_time_slots: self.num_time_slots,
        }
    }

    // -----------------------------------------------------------------------
    // RC-001: Load-aware greedy constructor (RP-401C algorithm)
    //
    // Algorithm:
    //   1. Sort demands by volume descending (with per-genome band shuffle for
    //      population diversity — demands within 10% of each other's volume
    //      are shuffled randomly).
    //   2. For each demand in order:
    //      a. Run load-aware Dijkstra with additive saturation penalty.
    //      b. Commit the chosen path.
    //      c. Update arc saturations via compute_loads on the partial solution.
    //   3. Return a genome whose waypoints are the resulting paths.
    //
    // This is the same algorithm as RP-401C (rp403_construction_portfolio.rs),
    // ported into the MOGA factory so the entire initial population benefits
    // from topology-aware construction rather than random assignment.
    // -----------------------------------------------------------------------
    fn create_greedy(&self, rng: &mut StdRng, gd: &GreedyConstructorData) -> RoadefGenome {
        // Build a demand order with band-level shuffling for diversity.
        // Demands within 10% of the maximum volume are considered one band.
        let mut ordered = gd.demands_by_volume.clone();
        if ordered.len() > 1 {
            // Shuffle within volume bands: group demands whose volume is within
            // 10% of the current band leader, then shuffle each group.
            let mut i = 0;
            while i < ordered.len() {
                let band_vol = ordered[i].3;
                let threshold = band_vol * 0.90;
                let mut j = i + 1;
                while j < ordered.len() && ordered[j].3 >= threshold {
                    j += 1;
                }
                // Shuffle the band [i..j)
                ordered[i..j].shuffle(rng);
                i = j;
            }
        }

        // Initialise saturation map from link capacities (all zero load).
        let mut ecmp_saturation: HashMap<u64, f64> = gd.link_capacity.keys()
            .map(|&id| (id, 0.0_f64))
            .collect();

        let mut partial_srpaths: Vec<SrPath> = Vec::new();
        let mut waypoints: Vec<Vec<u64>> = vec![vec![]; self.num_demands];

        // RC-001A2 FIX: Use the union of disabled links across ALL time slots.
        // Previously only t=0 disabled links were used. The genome is evaluated at
        // all time slots using the same waypoints — if a t=1 intervention disables
        // a link that the t=0 route traverses, expand_sr_path fails at t=1 →
        // compute_loads returns None → evaluator marks genome invalid → IFR=0.
        //
        // Conservative fix: avoid any link disabled at any time slot. This ensures
        // the routed paths are valid for all slots. It is suboptimal (may avoid links
        // that are only disabled at one slot) but guarantees cross-slot feasibility.
        let disabled_links: HashSet<u64> = gd.evaluator.scenario.interventions
            .iter()
            .flat_map(|iv| iv.links.iter().copied())
            .collect();
        let n_disabled_links = disabled_links.len();

        // RC-001A3 FIX: Precompute the worst-case time slot (highest total demand volume).
        // Using t=0 only for the Dijkstra saturation penalty is insufficient when t=1
        // volumes are much higher (e.g. setA-05: v1/v0 ratio max=103). The Dijkstra
        // penalty must reflect the slot where congestion is worst.
        //
        // Cost: one pass over demands × slots (O(D×T)) — negligible vs. O(D²) construction.
        // This replaces the previous approach of calling compute_loads() for all slots
        // after each demand (O(D²×T) total), which was too slow for setA-02 (10s budget).
        let worst_slot: usize = {
            let mut best_ts = 0usize;
            let mut best_vol = 0.0f64;
            for ts in 0..self.num_time_slots {
                let total_vol: f64 = gd.evaluator.tm.demands.iter()
                    .map(|d| d.v.get(ts).copied().unwrap_or(0.0))
                    .sum();
                if total_vol > best_vol {
                    best_vol = total_vol;
                    best_ts = ts;
                }
            }
            best_ts
        };

        // RC-001 SCALABILITY FIX: Precompute disabled arcs per time slot once per genome.
        // expand_sr_path takes a HashSet<u64> of disabled arc IDs (same ID space as arc.id).
        // The evaluator's scenario.interventions use link IDs, which equal arc IDs in the
        // undirected→directed expansion (each undirected link becomes two directed arcs with
        // the same ID as the link).
        //
        // RC-001A4 FIX: Precompute for ALL slots, not just worst_slot.
        // The greedy previously only measured saturation at worst_slot, so it missed overloads
        // at other slots (e.g. setA-02: greedy max_sat=0.811 at t=0, but evaluator sees
        // sat=1.098 on arc 40 at t=1 because t=1 has different disabled arcs and demand volumes).
        // Cost: O(T) precomputation — negligible.
        let disabled_arcs_per_slot: Vec<HashSet<u64>> = (0..self.num_time_slots)
            .map(|ts| {
                gd.evaluator.scenario.interventions
                    .iter()
                    .filter(|iv| iv.t == ts)
                    .flat_map(|iv| iv.links.iter().copied())
                    .collect()
            })
            .collect();
        // RC-001 SCALABILITY FIX: Maintain running arc-flow accumulators per time slot.
        // Instead of cloning partial_srpaths and calling compute_loads() (O(D×arcs) per demand,
        // O(D²×arcs) total), we call expand_sr_path() once per new demand per slot (O(path_len)
        // per demand per slot, O(D×T×path_len) total). For T=2 this is 2× the single-slot cost.
        //
        // RC-001A4 FIX: Track flows for ALL slots so saturation measurement matches the
        // evaluator, which checks all slots. Previously only worst_slot was tracked, causing
        // the greedy to miss overloads at other slots.
        //
        // On setA-06 (500 demands, 500 links, T=2): O(D²) = 250K → O(D×T) = 1K iterations.
        // Expected speedup vs. original: ~250× for the saturation update step.
        let mut running_arc_flows_per_slot: Vec<HashMap<u64, f64>> = (0..self.num_time_slots)
            .map(|_| {
                gd.evaluator.graph.arcs
                    .iter()
                    .map(|a| (a.id, 0.0_f64))
                    .collect()
            })
            .collect();

        // Telemetry counters for constructor diagnostics (emitted to stderr on invalid candidate).
        let mut n_load_aware_routes: usize = 0;
        let mut n_fallback_routes: usize = 0;
        let mut n_routing_failures: usize = 0;
        let mut n_segment_truncations: usize = 0;
        let mut max_saturation_seen: f64 = 0.0;

        for (d_idx, src, dst, _vol) in &ordered {
            // Load-aware Dijkstra with additive saturation penalty (RP-401C).
            // RC-001B: metric_noise_pct=0.20 injects ±20% per-link metric noise
            // drawn from rng so each genome explores a different cost landscape.
            // This breaks the deterministic argmin that caused g0dup=50 (diversity
            // collapse) while preserving the load-aware routing signal.
            let load_aware_result = greedy_load_aware_dijkstra(
                &gd.network,
                *src,
                *dst,
                &disabled_links,
                &ecmp_saturation,
                100.0,
                0.20,
                rng,
            );

            let full_path = if load_aware_result.is_some() {
                n_load_aware_routes += 1;
                load_aware_result
            } else {
                let fallback = greedy_shortest_path(&gd.network, *src, *dst, &disabled_links);
                if fallback.is_some() {
                    n_fallback_routes += 1;
                }
                fallback
            };

            if let Some(fp) = full_path {
                let raw_wps: Vec<u64> = if fp.len() > 2 {
                    fp[1..fp.len() - 1].to_vec()
                } else {
                    vec![]
                };
                let wps = path_to_waypoints_rc001(&fp, gd.max_segments);
                if raw_wps.len() > wps.len() {
                    n_segment_truncations += 1;
                    // TRUNCATION = CONSTRUCTION FAILURE.
                    //
                    // path_to_waypoints_rc001() silently dropped waypoints because the
                    // path requires more intermediate nodes than gd.max_segments allows.
                    // The resulting `wps` is an incomplete route: the commodity would not
                    // actually reach its destination under SR forwarding.
                    //
                    // Storing a truncated route and then calling expand_sr_path() on it
                    // reserves arc capacity for a partial path, causing those arcs to
                    // accumulate flow from every truncated commodity — producing the
                    // observed 22× overload on arcs 362/363 while failures=0.
                    //
                    // Correct semantics: treat truncation as a routing failure.
                    // Do NOT store waypoints, do NOT reserve arc flows.
                    // waypoints[*d_idx] remains empty (default), which the evaluator
                    // will route via ECMP — a valid fallback that respects capacity.
                    n_routing_failures += 1;
                    continue;
                }
                // RC-001A3 FIX: Emit SR paths for ALL time slots in partial_srpaths,
                // not just t=0. This ensures compute_loads(ts, &partial_sol) uses the
                // actual constructed waypoints at every slot rather than falling back to
                // ECMP default routing. ECMP default at t≥1 may route through disabled
                // links (e.g. setA-05 link 337 disabled at t=1), causing compute_loads
                // to return None → saturation update skipped → Dijkstra has no load
                // signal → routes pile onto same links → IFR=0.
                //
                // Emitting all-slot SR paths also ensures the saturation update reflects
                // the actual worst-case load the genome will experience at evaluation time.
                if !wps.is_empty() {
                    for ts in 0..self.num_time_slots {
                        partial_srpaths.push(SrPath { d: *d_idx, t: ts, w: wps.clone() });
                    }
                }
                waypoints[*d_idx] = wps;

                // Update saturation from partial solution using the worst-case time slot.
                //
                // RC-001 SCALABILITY FIX: Incremental arc-flow update via expand_sr_path().
                //
                // Previously: clone partial_srpaths (O(D)) + compute_loads() (O(D×arcs))
                //             = O(D²×arcs) total for the full population construction.
                // Now: call expand_sr_path() once for the new demand only (O(path_len))
                //      = O(D×path_len) total — effectively O(D) since path_len << D.
                //
                // On setA-06 (500 demands, 500 links): 500× speedup for this step.
                // On setA-04 (200 demands, 250 links): 200× speedup.
                //
                // Correctness: running_arc_flows accumulates flows from all demands routed
                // so far at worst_slot. expand_sr_path ADDS to the map (does not reset it),
                // so the accumulator is monotonically correct. ecmp_saturation is derived
                // from running_arc_flows after each demand, giving the same signal as before.
                //
                // RC-001A4 FIX: Measure saturation across ALL time slots, not just worst_slot.
                //
                // Previously only worst_slot was measured, so the greedy missed overloads at
                // other slots (e.g. setA-02: t=1 has different disabled arcs and demand volumes
                // than t=0, causing arc 40 to overload at t=1 even though t=0 was fine).
                //
                // For each slot: use that slot's disabled arcs and demand volume.
                // The Dijkstra penalty (ecmp_saturation) is updated from worst_slot flows only
                // (to keep the load signal consistent with path selection). The max_saturation_seen
                // tracker covers all slots so the greedy knows the true worst-case saturation.
                for ts in 0..self.num_time_slots {
                    let demand_vol = gd.evaluator.tm.demands[*d_idx].v
                        .get(ts).copied().unwrap_or(0.0);
                    if demand_vol == 0.0 { continue; }
                    let ok = expand_sr_path(
                        &gd.evaluator.graph,
                        *src,
                        *dst,
                        &waypoints[*d_idx],
                        &disabled_arcs_per_slot[ts],
                        demand_vol,
                        &mut running_arc_flows_per_slot[ts],
                    );
                    if ok {
                        for (arc_id, &flow) in &running_arc_flows_per_slot[ts] {
                            let cap = gd.link_capacity.get(arc_id).copied().unwrap_or(1.0);
                            let sat = if cap > 0.0 { flow / cap } else { f64::INFINITY };
                            if sat > max_saturation_seen { max_saturation_seen = sat; }
                            // Update Dijkstra penalty only from worst_slot flows, so path
                            // selection remains consistent with the worst-case load signal.
                            if ts == worst_slot {
                                let entry = ecmp_saturation.entry(*arc_id).or_insert(0.0);
                                if sat > *entry { *entry = sat; }
                            }
                        }
                    }
                    // If expand_sr_path returns false (disconnected at this slot due to a
                    // disabled arc), skip the saturation update for this slot — the Dijkstra
                    // penalty will be based on the previous state (conservative).
                }
            } else {
                n_routing_failures += 1;
            }
        }

        // Emit constructor diagnostics to stderr (always, for observability).
        // This is cheap (one eprintln per genome) and invaluable for debugging.
        // Note: load_aware count = num_demands × num_time_slots because the constructor
        // routes each demand independently for each time slot (t=0 saturation model).
        // For a 200-demand, 2-slot instance: load_aware=400 is expected and correct.
        eprintln!(
            "[greedy] demands={} slots={} disabled_links={} load_aware={} fallback={} failures={} truncations={} max_sat={:.3}",
            ordered.len(), self.num_time_slots, n_disabled_links,
            n_load_aware_routes, n_fallback_routes, n_routing_failures,
            n_segment_truncations, max_saturation_seen
        );

        // RC-001A1: When any arc's final saturation exceeds 1.0, emit top-5 saturated arcs.
        // CORRECTNESS: Use the final ecmp_saturation map (not max_saturation_seen, which
        // accumulates intermediate values during construction and can be stale).
        // Only emit once per genome (not 50× per population) — caller controls frequency.
        let actual_max_sat = ecmp_saturation.values().cloned().fold(0.0_f64, f64::max);
        if actual_max_sat > 1.0 {
            let mut sat_vec: Vec<(u64, f64, f64, f64)> = ecmp_saturation.iter()
                .filter(|(_, &sat)| sat > 0.0)
                .map(|(&arc_id, &sat)| {
                    let cap = gd.link_capacity.get(&arc_id).copied().unwrap_or(1.0);
                    (arc_id, sat * cap, cap, sat) // (arc_id, flow, capacity, sat)
                })
                .collect();
            sat_vec.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));
            sat_vec.truncate(5);
            eprintln!("[greedy] ⚠ actual_max_sat={:.3} — top saturated arcs (arc_id, flow, cap, sat):",
                actual_max_sat);
            for (arc_id, flow, cap, sat) in &sat_vec {
                eprintln!("[greedy]   arc={} flow={:.3} cap={:.3} sat={:.3}", arc_id, flow, cap, sat);
            }
        }

        // RC-001A5 FIX: If the constructed solution is infeasible (max_saturation_seen > 1.0),
        // return empty waypoints so the evaluator routes via ECMP (the default fallback).
        //
        // Rationale: the greedy constructor may deterministically route all demands through
        // the same bottleneck arcs (e.g. setA-02: arc 40 at t=1, sat=1.098), producing a
        // genome that the evaluator always marks invalid. Returning the infeasible SR waypoints
        // causes the entire initial population to be invalid (IFR=0), leaving repair with no
        // valid fallback and the GA stuck.
        //
        // Returning empty waypoints (ECMP) gives the evaluator a different routing to try.
        // ECMP may or may not be feasible, but it breaks the deterministic lock and allows
        // the rejection-sampling loop to find feasible genomes via random construction.
        if max_saturation_seen > 1.0 {
            eprintln!(
                "[greedy] ⚠ max_sat={:.3} > 1.0 — returning empty waypoints (ECMP fallback)",
                max_saturation_seen
            );
            return RoadefGenome {
                waypoints: vec![vec![]; self.num_demands],
                num_time_slots: self.num_time_slots,
            };
        }

        RoadefGenome {
            waypoints,
            num_time_slots: self.num_time_slots,
        }
    }
}

// ---------------------------------------------------------------------------
// RC-001 helper: extract intermediate waypoints from a full node path.
// ---------------------------------------------------------------------------
fn path_to_waypoints_rc001(full_path: &[u64], max_segments: usize) -> Vec<u64> {
    if full_path.len() <= 2 {
        return vec![];
    }
    let waypoints: Vec<u64> = full_path[1..full_path.len() - 1].to_vec();
    // max_segments is the maximum number of SR segments (hops between waypoints + 1).
    // A path with k intermediate waypoints uses k+1 segments.
    // If max_segments == 0, SR is disabled entirely — return empty (ECMP default).
    // If max_segments == 1, only direct paths are allowed — no intermediate waypoints.
    if max_segments == 0 {
        return vec![];
    }
    // max intermediate waypoints = max_segments - 1
    let max_waypoints = max_segments.saturating_sub(1);
    if max_waypoints == 0 {
        return vec![];
    }
    if waypoints.len() > max_waypoints {
        waypoints[..max_waypoints].to_vec()
    } else {
        waypoints
    }
}

// ---------------------------------------------------------------------------
// RC-001 helper: load-aware Dijkstra with additive saturation penalty.
// Matches RP-401C (rp403_construction_portfolio.rs load_aware_path_ecmp_rp401c).
//
// Penalty formula (load_penalty = 100.0):
//   sat >= 1.0  → penalty = 1e9  (effectively blocked)
//   sat >  0.8  → penalty = load_penalty * (1/(1-sat) - 1) * 10.0
//   else        → penalty = load_penalty * sat
//
// RC-001B: metric_noise_pct adds per-link metric perturbation drawn from
// link_rng to break ties differently across genomes. Each link gets
// metric * (1 + ε) where ε ~ Uniform(-metric_noise_pct, +metric_noise_pct).
// This preserves the load-aware signal while restoring population diversity.
// Set metric_noise_pct = 0.0 to disable (deterministic mode).
// ---------------------------------------------------------------------------
fn greedy_load_aware_dijkstra(
    net: &Network,
    src: u64,
    dst: u64,
    disabled_links: &HashSet<u64>,
    ecmp_saturation: &HashMap<u64, f64>,
    load_penalty: f64,
    metric_noise_pct: f64,
    link_rng: &mut StdRng,
) -> Option<Vec<u64>> {
    if src == dst {
        return Some(vec![src]);
    }

    // Build adjacency with additive penalty and optional metric noise.
    // Each link is added individually — the evaluator sees the same graph.
    // For multigraph networks with parallel links, all links are included;
    // Dijkstra will naturally select the best (lowest effective metric) path.
    let mut adj: HashMap<u64, Vec<(u64, f64)>> = HashMap::new();
    for link in &net.links {
        if disabled_links.contains(&link.id) {
            continue;
        }
        let sat = ecmp_saturation.get(&link.id).copied().unwrap_or(0.0);
        let penalty = if sat >= 1.0 {
            1e9
        } else if sat > 0.8 {
            load_penalty * (1.0 / (1.0 - sat) - 1.0) * 10.0
        } else {
            load_penalty * sat
        };
        // RC-001B: per-link metric noise for diversity. Noise is proportional to
        // base metric so it doesn't dominate the load penalty on congested links.
        let noise = if metric_noise_pct > 0.0 {
            let eps: f64 = link_rng.gen_range(-metric_noise_pct..=metric_noise_pct);
            link.metric * eps
        } else {
            0.0
        };
        let effective_metric = link.metric + penalty + noise;
        adj.entry(link.from).or_default().push((link.to, effective_metric));
    }

    let mut dist: HashMap<u64, u64> = HashMap::new();
    let mut prev: HashMap<u64, u64> = HashMap::new();
    let mut heap: BinaryHeap<(std::cmp::Reverse<u64>, u64)> = BinaryHeap::new();

    dist.insert(src, 0);
    heap.push((std::cmp::Reverse(0), src));

    while let Some((std::cmp::Reverse(cost), node)) = heap.pop() {
        if dist.get(&node).copied().unwrap_or(u64::MAX) < cost {
            continue;
        }
        if node == dst {
            break;
        }
        if let Some(neighbors) = adj.get(&node) {
            for &(next, em) in neighbors {
                let new_cost = cost + (em * 1000.0) as u64;
                if dist.get(&next).copied().unwrap_or(u64::MAX) > new_cost {
                    dist.insert(next, new_cost);
                    prev.insert(next, node);
                    heap.push((std::cmp::Reverse(new_cost), next));
                }
            }
        }
    }

    if !dist.contains_key(&dst) {
        return None;
    }

    let mut path = vec![dst];
    let mut cur = dst;
    while cur != src {
        match prev.get(&cur) {
            Some(&p) => { path.push(p); cur = p; }
            None => return None,
        }
    }
    path.reverse();
    Some(path)
}

// ---------------------------------------------------------------------------
// RC-001 helper: plain shortest-path Dijkstra (fallback when load-aware fails).
// ---------------------------------------------------------------------------
fn greedy_shortest_path(
    net: &Network,
    src: u64,
    dst: u64,
    disabled_links: &HashSet<u64>,
) -> Option<Vec<u64>> {
    if src == dst {
        return Some(vec![src]);
    }

    let mut adj: HashMap<u64, Vec<(u64, f64)>> = HashMap::new();
    for link in &net.links {
        if disabled_links.contains(&link.id) {
            continue;
        }
        adj.entry(link.from).or_default().push((link.to, link.metric));
    }

    let mut dist: HashMap<u64, u64> = HashMap::new();
    let mut prev: HashMap<u64, u64> = HashMap::new();
    let mut heap: BinaryHeap<(std::cmp::Reverse<u64>, u64)> = BinaryHeap::new();

    dist.insert(src, 0);
    heap.push((std::cmp::Reverse(0), src));

    while let Some((std::cmp::Reverse(cost), node)) = heap.pop() {
        if dist.get(&node).copied().unwrap_or(u64::MAX) < cost {
            continue;
        }
        if node == dst {
            break;
        }
        if let Some(neighbors) = adj.get(&node) {
            for &(next, m) in neighbors {
                let new_cost = cost + (m * 1000.0) as u64;
                if dist.get(&next).copied().unwrap_or(u64::MAX) > new_cost {
                    dist.insert(next, new_cost);
                    prev.insert(next, node);
                    heap.push((std::cmp::Reverse(new_cost), next));
                }
            }
        }
    }

    if !dist.contains_key(&dst) {
        return None;
    }

    let mut path = vec![dst];
    let mut cur = dst;
    while cur != src {
        match prev.get(&cur) {
            Some(&p) => { path.push(p); cur = p; }
            None => return None,
        }
    }
    path.reverse();
    Some(path)
}

// ---------------------------------------------------------------------------
// Evaluation
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct RoadefEvaluation {
    pub genome: RoadefGenome,
    pub obj: f64,
    pub valid: bool,
    pub mlu: f64,
    /// Sorted arc-saturation load vector (descending). Used by RP-410 telemetry.
    pub load_vector: Vec<f64>,
    /// Origin operator tag for RP-410 telemetry.
    /// Values: "crossover", "crossover+mutation", "mutation", "elite", "initial"
    pub operator: &'static str,
    /// RC-002: maximum arc saturation observed across all time slots.
    /// For valid genomes: max_sat ≤ 1.0 + 1e-6 (by evaluator invariant).
    /// For invalid genomes: max_sat > 1.0 + 1e-6, used to classify overload severity:
    ///   epsilon: sat ≤ 1.0 + 1e-5  (float rounding artefact, physically feasible)
    ///   minor:   sat ≤ 1.01        (≤1% overload, likely crossover accumulation)
    ///   major:   sat > 1.01        (>1% overload, structural capacity violation)
    /// 0.0 when compute_loads() returns None (structural failure before load computation).
    pub max_sat: f64,
}

impl Evaluated for RoadefEvaluation {
    type Genome = RoadefGenome;

    fn fitness(&self) -> f64 {
        if !self.valid {
            -1_000_000.0
        } else {
            -self.obj  // lower obj = higher fitness
        }
    }

    fn is_valid(&self) -> bool {
        self.valid
    }

    fn genome(&self) -> &RoadefGenome {
        &self.genome
    }
}

pub struct RoadefFitnessEvaluator {
    pub evaluator: Arc<RoadefEvaluator>,
}

impl FitnessEvaluator<RoadefGenome> for RoadefFitnessEvaluator {
    type Evaluation = RoadefEvaluation;

    /// Evaluation Invariants
    ///
    /// A solution is valid iff:
    ///   1. Structural constraints satisfied (budget, max_segments, connectivity).
    ///   2. Objective is finite (no arc saturation ≥ 1.0 in the inverse load cost).
    ///
    /// Therefore: valid == true ⇒ obj.is_finite()
    ///
    /// This invariant is enforced here so that fitness() remains trivial (-obj)
    /// and FeasibilityCertificate (M20) maps cleanly to a binary pass/fail.
    fn evaluate(&self, genome: &RoadefGenome) -> RoadefEvaluation {
        let solution = genome.to_solution();
        // M20 Phase 3: use cached evaluator as production path (E-001 validated).
        // Timings discarded here; profiling uses eval_profiler binary directly.
        let (result, _) = self.evaluator.evaluate_solution_cached(&solution);

        // Enforce evaluation invariant: infinite objective ⇒ invalid
        let valid = result.valid && result.obj.is_finite();

        // Compute average MLU and sorted load vector across time slots.
        // For valid genomes: full computation for RP-410 telemetry.
        // For invalid genomes: compute_loads() to extract max_sat for RC-002 overload
        // classification. The [diag] block has been moved to the evolution loop call
        // site where the operator tag is known, so it can be emitted with origin context.
        let mut total_mlu = 0.0;
        let mut mlu_count = 0;
        let mut all_sats: Vec<f64> = Vec::new();
        let mut max_sat: f64 = 0.0;

        for t in 0..genome.num_time_slots {
            if let Some(loads) = self.evaluator.compute_loads(t, &solution) {
                if valid {
                    total_mlu += loads.mlu;
                    mlu_count += 1;
                    for sat in loads.arc_saturations.values() {
                        all_sats.push(*sat);
                    }
                }
                // RC-002: track max_sat for ALL genomes (valid and invalid) so the
                // call site can classify overload severity without re-calling compute_loads.
                for sat in loads.arc_saturations.values() {
                    if *sat > max_sat { max_sat = *sat; }
                }
            }
        }

        let mlu = if valid && mlu_count > 0 { total_mlu / mlu_count as f64 } else { f64::INFINITY };
        let load_vector = if valid { sorted_load_vector(&all_sats) } else { Vec::new() };

        RoadefEvaluation {
            genome: genome.clone(),
            obj: result.obj,
            valid,
            mlu,
            load_vector,
            operator: "initial",
            max_sat,
        }
    }
}

// ---------------------------------------------------------------------------
// Mutation
// ---------------------------------------------------------------------------

/// Mutates one randomly chosen demand's waypoint list.
/// Operations: clear (→ ECMP), set to 1 random node, or swap existing waypoint.
pub struct RoadefMutator {
    pub node_ids: Vec<u64>,
}

impl MutationOperator<RoadefGenome> for RoadefMutator {
    fn mutate(&self, genome: &mut RoadefGenome, rng: &mut StdRng) {
        if genome.waypoints.is_empty() {
            return;
        }
        let d = rng.gen_range(0..genome.waypoints.len());
        let op = rng.gen_range(0u8..3);
        match op {
            0 => {
                // Clear waypoints → ECMP default
                genome.waypoints[d].clear();
            }
            1 => {
                // Set to 1 random waypoint
                if !self.node_ids.is_empty() {
                    let idx = rng.gen_range(0..self.node_ids.len());
                    genome.waypoints[d] = vec![self.node_ids[idx]];
                }
            }
            _ => {
                // Replace existing waypoint or add one
                if genome.waypoints[d].is_empty() {
                    if !self.node_ids.is_empty() {
                        let idx = rng.gen_range(0..self.node_ids.len());
                        genome.waypoints[d] = vec![self.node_ids[idx]];
                    }
                } else {
                    let wp_idx = rng.gen_range(0..genome.waypoints[d].len());
                    if !self.node_ids.is_empty() {
                        let idx = rng.gen_range(0..self.node_ids.len());
                        genome.waypoints[d][wp_idx] = self.node_ids[idx];
                    }
                }
            }
        }
    }
}
// ---------------------------------------------------------------------------
// RP-409B: Peak-Targeted Mutation
// ---------------------------------------------------------------------------

/// Mutation strategy selector for RP-409B A/B experiment.
///
/// `Uniform`      — baseline: perturbs one uniformly random demand (RP-406C behaviour).
/// `PeakTargeted` — experimental: with probability `peak_bias`, perturbs a demand
///                  that routes through the current Peak arc; falls back to uniform
///                  if no such demands are known.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MutationStrategy {
    Uniform,
    PeakTargeted,
}

/// Shared state updated by the evolution loop after each global-best improvement.
/// Contains the indices (into `genome.waypoints`) of demands whose waypoints
/// include at least one node on the path through the highest-loaded arc.
///
/// The evolution loop calls `update_peak_demands()` after each global-best update.
/// The mutator reads this on each `mutate()` call.
pub type PeakDemandSet = Arc<RwLock<Vec<usize>>>;

/// Create an empty shared peak-demand set.
pub fn new_peak_demand_set() -> PeakDemandSet {
    Arc::new(RwLock::new(Vec::new()))
}

/// Update the peak-demand set from the current global-best genome and its load vector.
///
/// Strategy: the Peak arc is the arc with the highest saturation (rank-1 of the
/// sorted load vector). We identify demands whose waypoint list is non-empty
/// (i.e. they are SR-routed rather than ECMP-default). Among those, we select
/// the ones most likely to influence the Peak arc by checking whether their
/// waypoint sequence contains any node that appears in the top-K loaded arcs.
///
/// Since we do not have direct arc→demand routing information without calling
/// `compute_loads()` per demand (expensive), we use a heuristic: any demand
/// with a non-empty waypoint list is a candidate. We rank them by waypoint
/// count (more waypoints = more routing influence) and take the top-N.
///
/// This is a conservative approximation. A future version can use
/// `compute_loads()` arc_flows to identify exact demand→arc routing.
pub fn update_peak_demands(
    peak_set: &PeakDemandSet,
    genome: &RoadefGenome,
    _load_vector: &[f64],
) {
    // Collect all demands with non-empty waypoints (SR-routed demands).
    // These are the only demands that can be perturbed to change routing.
    let mut candidates: Vec<(usize, usize)> = genome.waypoints.iter()
        .enumerate()
        .filter(|(_, wps)| !wps.is_empty())
        .map(|(d, wps)| (d, wps.len()))
        .collect();

    // Sort by waypoint count descending (more waypoints = more routing influence).
    candidates.sort_by(|a, b| b.1.cmp(&a.1));

    // Take top-20% of SR-routed demands, minimum 1, maximum 50.
    let n = ((candidates.len() as f64 * 0.20).ceil() as usize).max(1).min(50);
    let peak_demands: Vec<usize> = candidates.iter().take(n).map(|(d, _)| *d).collect();

    if let Ok(mut guard) = peak_set.write() {
        *guard = peak_demands;
    }
}

/// Peak-targeted mutator for RP-409B.
///
/// With probability `peak_bias` (default 0.7), selects a demand from the
/// shared `peak_demand_set` and applies the same three operations as
/// `RoadefMutator` (clear, set-to-random-node, swap). Falls back to uniform
/// random demand selection if the peak set is empty or the bias roll fails.
pub struct PeakTargetedMutator {
    pub node_ids: Vec<u64>,
    pub peak_demand_set: PeakDemandSet,
    /// Probability of targeting a Peak-arc demand rather than a random demand.
    pub peak_bias: f64,
}

impl MutationOperator<RoadefGenome> for PeakTargetedMutator {
    fn mutate(&self, genome: &mut RoadefGenome, rng: &mut StdRng) {
        if genome.waypoints.is_empty() {
            return;
        }

        // Decide whether to target a Peak-arc demand.
        let use_peak = rng.gen_bool(self.peak_bias);
        let d = if use_peak {
            // Try to pick from the peak demand set.
            let guard = self.peak_demand_set.read().ok();
            let peak_d = guard.as_ref().and_then(|v| {
                if v.is_empty() { None }
                else {
                    let idx = rng.gen_range(0..v.len());
                    Some(v[idx])
                }
            });
            // Fall back to uniform if peak set is empty or out of bounds.
            match peak_d {
                Some(d) if d < genome.waypoints.len() => d,
                _ => rng.gen_range(0..genome.waypoints.len()),
            }
        } else {
            rng.gen_range(0..genome.waypoints.len())
        };

        // Apply the same three operations as RoadefMutator.
        let op = rng.gen_range(0u8..3);
        match op {
            0 => {
                // Clear waypoints → ECMP default
                genome.waypoints[d].clear();
            }
            1 => {
                // Set to 1 random waypoint
                if !self.node_ids.is_empty() {
                    let idx = rng.gen_range(0..self.node_ids.len());
                    genome.waypoints[d] = vec![self.node_ids[idx]];
                }
            }
            _ => {
                // Replace existing waypoint or add one
                if genome.waypoints[d].is_empty() {
                    if !self.node_ids.is_empty() {
                        let idx = rng.gen_range(0..self.node_ids.len());
                        genome.waypoints[d] = vec![self.node_ids[idx]];
                    }
                } else {
                    let wp_idx = rng.gen_range(0..genome.waypoints[d].len());
                    if !self.node_ids.is_empty() {
                        let idx = rng.gen_range(0..self.node_ids.len());
                        genome.waypoints[d][wp_idx] = self.node_ids[idx];
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Crossover
// ---------------------------------------------------------------------------

/// Uniform crossover: for each demand, randomly inherit waypoints from parent A or B.
pub struct RoadefCrossover;

impl CrossoverOperator<RoadefGenome> for RoadefCrossover {
    fn crossover(&self, parent_a: &RoadefGenome, parent_b: &RoadefGenome, rng: &mut StdRng) -> (RoadefGenome, RoadefGenome) {
        let n = parent_a.waypoints.len().min(parent_b.waypoints.len());
        let mut child_a = parent_a.clone();
        let mut child_b = parent_b.clone();
        for d in 0..n {
            if rng.gen_bool(0.5) {
                child_a.waypoints[d] = parent_b.waypoints[d].clone();
                child_b.waypoints[d] = parent_a.waypoints[d].clone();
            }
        }
        (child_a, child_b)
    }
}

// ---------------------------------------------------------------------------
// Custom Evolution Loop with Active Logging
// ---------------------------------------------------------------------------
//
// The MOGA engine's run_ga_evolution() is a monolithic loop with no
// per-generation callback. This custom loop uses the same building blocks
// (evaluate, mutate, crossover, tournament selection) but adds:
//
//   Level 1 — Progress every LOG_INTERVAL generations
//   Level 2 — Improvement events (global best changes)
//   Level 3 — Termination reason
//   Level 4 — Population health every HEALTH_INTERVAL generations
//
// All logging goes to the provided log_sink (typically a per-instance file).
// This stays entirely within the ROADEF adapter — no MOGA modifications.

use rand::SeedableRng;
use std::io::Write;
use std::time::Instant;

pub struct EvolutionRunConfig {
    pub population_size: usize,
    pub elite_count: usize,
    pub generation_limit: usize,
    pub mutation_rate: f64,
    pub crossover_rate: f64,
    pub no_improvement_limit: usize,
    pub seed: Option<u64>,
    pub log_interval: usize,
    pub health_interval: usize,
    /// Wall-clock time budget per instance. Execution policy — not an EA parameter.
    /// When elapsed >= max_runtime, terminates with reason TimeLimit.
    /// None = no time limit (generation_limit and no_improvement_limit govern termination).
    pub max_runtime: Option<std::time::Duration>,
    /// RP-408A: Which objective comparator to use for this run.
    /// `Scalar` (default) reproduces the RP-406C baseline behaviour.
    /// `Lexicographic` activates the RP-408 experimental comparator.
    pub comparator_mode: ComparatorMode,
    /// RP-409B: Optional shared peak-demand set for PeakTargetedMutator.
    /// When `Some`, the evolution loop calls `update_peak_demands()` after
    /// each global-best improvement. `None` (default) disables the update.
    pub peak_demand_set: Option<PeakDemandSet>,
}

impl Default for EvolutionRunConfig {
    fn default() -> Self {
        Self {
            population_size: 80,
            elite_count: 8,
            generation_limit: 200,
            mutation_rate: 0.3,
            crossover_rate: 0.7,
            no_improvement_limit: 20,
            seed: None,
            log_interval: 10,
            health_interval: 20,
            max_runtime: None,
            comparator_mode: ComparatorMode::Scalar,
            peak_demand_set: None,
        }
    }
}

pub struct EvolutionRunResult {
    pub best_genome: RoadefGenome,
    pub best_obj: f64,
    pub best_mlu: f64,
    pub valid: bool,
    pub generations_run: usize,
    pub best_found_at_gen: usize,
    pub termination_reason: String,
    pub runtime_ms: u128,
    /// RC-001: Initial Feasibility Rate of generation 0 (fraction of initial
    /// population that was feasible). Baseline (CB-000): mean 10.6%.
    pub initial_feasibility_rate: f64,
    /// RC-001: Best objective value in generation 0 (before any evolution).
    /// Distinguishes "good constructor → good start → EA improved it" from
    /// "poor constructor → EA repaired it". f64::INFINITY if no valid individual.
    pub gen0_best_obj: f64,
    /// RC-001: Mean objective value across all *valid* individuals in generation 0.
    /// f64::INFINITY if no valid individual in generation 0.
    pub gen0_mean_obj: f64,
    /// RC-001: Number of feasible individuals in generation 0 (= IFR × pop_size).
    pub gen0_feasible_count: usize,
    /// RC-001: Number of distinct objective values among valid gen-0 individuals.
    /// Low value (e.g. 1–3 out of 50) indicates population diversity collapse:
    /// the constructor is producing nearly identical genomes, which causes
    /// premature convergence even when IFR is high.
    pub gen0_unique_obj_count: usize,
    /// RC-001: Number of gen-0 individuals whose waypoint vector is identical to
    /// at least one other individual in the initial population (exact duplicates).
    /// High value confirms diversity collapse hypothesis.
    pub gen0_duplicate_genome_count: usize,
}

/// Run ROADEF evolution with active per-generation logging.
///
/// `log_sink`: any Write target (file, stderr, Vec<u8>).
/// Returns the best result found.
pub fn run_roadef_evolution<M>(
    factory: &RoadefGenomeFactory,
    fitness_eval: &RoadefFitnessEvaluator,
    mutator: &M,
    crossover: &RoadefCrossover,
    config: &EvolutionRunConfig,
    instance_name: &str,
    log_sink: &mut dyn Write,
    telemetry: &mut dyn TelemetrySink,
) -> EvolutionRunResult
where
    M: MutationOperator<RoadefGenome>,
{
    let mut rng: StdRng = match config.seed {
        Some(s) => StdRng::seed_from_u64(s),
        None => StdRng::from_entropy(),
    };

    // RP-408A: generate a unique run identifier and build the comparator.
    // run_uuid groups all telemetry records from this execution without relying
    // on filenames or directory structure.
    let run_uuid = {
        // Simple UUID v4 from random bytes (no uuid crate dependency).
        let b: [u8; 16] = rng.gen();
        format!(
            "{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}",
            u32::from_le_bytes([b[0], b[1], b[2], b[3]]),
            u16::from_le_bytes([b[4], b[5]]),
            u16::from_le_bytes([b[6], b[7]]) & 0x0fff,
            (u16::from_le_bytes([b[8], b[9]]) & 0x3fff) | 0x8000,
            {
                let hi = u32::from_le_bytes([b[10], b[11], b[12], b[13]]) as u64;
                let lo = u16::from_le_bytes([b[14], b[15]]) as u64;
                (hi << 16) | lo
            }
        )
    };
    let comparator = make_comparator(config.comparator_mode);

    let t0 = Instant::now();

    // --- Initialize population and evaluate gen-0 (tagged "initial") ---
    // RC-006A fix: check time budget after each individual to prevent constructor
    // from consuming the entire time budget on large instances (setA-18: D=2000,
    // setA-20: D=6000). Without this check, 50 greedy individuals × ~8s each = 430s,
    // which exceeds the 60s budget before the evolution loop is entered.
    //
    // Budget policy: use at most 50% of the time budget for initial population
    // construction. If the budget is exceeded after any individual, stop building
    // and proceed with the population built so far (minimum 1 individual).
    // This ensures at least one generation of evolution can run.
    let init_budget_fraction = 0.5_f64;
    let init_deadline: Option<std::time::Duration> = config.max_runtime
        .map(|b| b.mul_f64(init_budget_fraction));

    // RC-003 initialization repair: rejection sampling.
    // If a constructed genome evaluates as invalid, retry construction up to
    // MAX_INIT_RETRIES times before accepting it. This directly addresses the
    // IFR problem: both constructors produce waypoints that the evaluator marks
    // invalid because the constructor's routing and the evaluator's routing diverge.
    //
    // Budget policy: retries count against the init budget. If the budget is
    // exhausted during retries, accept the best genome found so far (valid if
    // any retry succeeded, otherwise the last invalid genome).
    const MAX_INIT_RETRIES: usize = 10;

    let mut evals: Vec<RoadefEvaluation> = Vec::with_capacity(config.population_size);
    let mut n_init_retries: usize = 0;
    let mut n_init_retry_successes: usize = 0;
    for i in 0..config.population_size {
        let g = factory.create(&mut rng);
        let mut ev = fitness_eval.evaluate(&g);
        ev.operator = "initial";

        // Rejection sampling: retry if invalid and budget allows.
        if !ev.is_valid() {
            for _retry in 0..MAX_INIT_RETRIES {
                n_init_retries += 1;
                // Check budget before each retry.
                if let Some(deadline) = init_deadline {
                    if t0.elapsed() >= deadline { break; }
                }
                let g2 = factory.create(&mut rng);
                let mut ev2 = fitness_eval.evaluate(&g2);
                ev2.operator = "initial";
                if ev2.is_valid() {
                    ev = ev2;
                    n_init_retry_successes += 1;
                    break;
                }
                // Keep the best (lowest max_sat) invalid genome seen so far.
                if ev2.max_sat < ev.max_sat { ev = ev2; }
            }
        }

        evals.push(ev);

        // Check time budget after each individual (except the first — we always
        // build at least one individual so global_best can be initialized).
        if i > 0 {
            if let Some(deadline) = init_deadline {
                if t0.elapsed() >= deadline {
                    let _ = writeln!(log_sink,
                        "[init] time budget {:.0}% consumed after {} individuals — stopping early (RC-006A fix)",
                        init_budget_fraction * 100.0, i + 1);
                    break;
                }
            }
        }
    }
    if n_init_retries > 0 {
        let _ = writeln!(log_sink,
            "[init] rejection sampling: {} retries, {} successes ({:.0}% repair rate)",
            n_init_retries, n_init_retry_successes,
            if n_init_retries > 0 { n_init_retry_successes as f64 / n_init_retries as f64 * 100.0 } else { 0.0 });
    }
    evals.sort_by(|a, b| comparator.cmp_evals(b, a).then(
        b.fitness().partial_cmp(&a.fitness()).unwrap_or(Ordering::Equal)
    ));

    // RP-407: record valid count at generation 0 before any selection or variation.
    // This is the Initial Feasibility Rate metric — evaluates constructor quality.
    let generation0_valid_count: usize = evals.iter().filter(|e| e.is_valid()).count();
    // RC-001: hoist IFR so it can be returned in EvolutionRunResult.
    let initial_feasibility_rate: f64 = if config.population_size > 0 {
        generation0_valid_count as f64 / config.population_size as f64
    } else {
        0.0
    };

    // RC-001: generation-0 objective diagnostics.
    // These distinguish "good constructor → good start" from "EA repaired a poor start".
    // evals is sorted best-first (valid before invalid, lower obj before higher).
    let gen0_feasible_count = generation0_valid_count;
    let gen0_best_obj: f64 = evals.iter()
        .find(|e| e.is_valid())
        .map(|e| e.obj)
        .unwrap_or(f64::INFINITY);
    let gen0_mean_obj: f64 = {
        let valid_objs: Vec<f64> = evals.iter()
            .filter(|e| e.is_valid())
            .map(|e| e.obj)
            .collect();
        if valid_objs.is_empty() {
            f64::INFINITY
        } else {
            valid_objs.iter().sum::<f64>() / valid_objs.len() as f64
        }
    };

    // RC-001: population diversity metrics for gen-0.
    // These directly test the diversity-collapse hypothesis: if the greedy constructor
    // produces nearly identical genomes, unique_obj_count will be low and
    // duplicate_genome_count will be high, explaining premature convergence.
    //
    // gen0_unique_obj_count: count distinct objective values (rounded to 4 decimal places
    // to avoid floating-point noise treating near-identical values as distinct).
    let gen0_unique_obj_count: usize = {
        use std::collections::HashSet;
        let mut seen: HashSet<u64> = HashSet::new();
        for e in evals.iter().filter(|e| e.is_valid()) {
            // Round to 4 decimal places for deduplication.
            let key = (e.obj * 10_000.0).round() as u64;
            seen.insert(key);
        }
        seen.len()
    };
    // gen0_duplicate_genome_count: count genomes whose waypoint vector is identical
    // to at least one other genome in the population.
    // Uses a hash of the waypoint structure for O(n) detection.
    let gen0_duplicate_genome_count: usize = {
        use std::collections::HashMap;
        let mut counts: HashMap<Vec<Vec<u64>>, usize> = HashMap::new();
        for e in &evals {
            *counts.entry(e.genome.waypoints.clone()).or_insert(0) += 1;
        }
        // Count genomes that appear more than once (i.e. are duplicates).
        counts.values().filter(|&&c| c > 1).map(|&c| c).sum()
    };

    // RP-412 Phase 2: emit ConstructionRecord once per run, immediately after initial population.
    // capacity_violation_count = number of invalid individuals in the initial population.
    // The evaluator marks an individual invalid when any structural constraint is violated
    // (capacity, budget, connectivity). We cannot distinguish violation types without
    // per-individual breakdown from the evaluator, so we attribute all invalids to
    // capacity_violation_count as the dominant observed violation type (per RP-410C telemetry).
    // budget_violation_count and repair_* remain 0 — they require evaluator-level instrumentation
    // that is not yet available. This is the honest Phase 2 baseline state.
    {
        let invalid_count = config.population_size.saturating_sub(generation0_valid_count);
        let ifr = initial_feasibility_rate;
        let construction_rec = ConstructionRecord {
            record_type: "construction",
            run_uuid: run_uuid.clone(),
            comparator_mode: config.comparator_mode,
            instance: instance_name.to_string(),
            seed: config.seed.unwrap_or(0),
            population_size: config.population_size,
            valid_count: generation0_valid_count,
            invalid_count,
            initial_feasibility_rate: ifr,
            any_feasible: generation0_valid_count > 0,
            // RP-412 Phase 2: invalid_count is used as a proxy for capacity_violation_count.
            // All invalid individuals are assumed to have at least one capacity violation,
            // which is consistent with the RP-410C observation that all CapacityViolation
            // records are in the Tail zone. Per-constraint breakdown requires evaluator changes.
            capacity_violation_count: invalid_count as u32,
            // RP-412 Phase 2: budget_violation_count requires per-individual evaluator breakdown.
            // Remains 0 until the evaluator exposes constraint-level diagnostics.
            budget_violation_count: 0,
            // RP-412 Phase 2: repair is not yet a separate phase in this harness.
            repair_attempts: 0,
            repair_successes: 0,
        };
        telemetry.emit_construction(&construction_rec);
    }

    let mut global_best: Option<RoadefEvaluation> = None;
    let mut best_found_at_gen = 0usize;
    let mut stagnation = 0usize;
    let mut gen = 0usize;
    let mut termination_reason = String::new(); // set before every break in the loop below
    // RP-410C: monotonically increasing candidate counter for genealogy reconstruction.
    let mut candidate_counter: u64 = 0;

    let started = chrono::Utc::now().to_rfc3339();
    let _ = writeln!(log_sink, "=========================================");
    let _ = writeln!(log_sink, "ROADEF Campaign — Research Harness");
    let _ = writeln!(log_sink, "Instance      : {}", instance_name);
    let _ = writeln!(log_sink, "Population    : {}", config.population_size);
    let _ = writeln!(log_sink, "Elite         : {}", config.elite_count);
    let _ = writeln!(log_sink, "Generations   : {}", config.generation_limit);
    let _ = writeln!(log_sink, "NoImprove     : {}", config.no_improvement_limit);
    let _ = writeln!(log_sink, "Mutation rate : {}", config.mutation_rate);
    let _ = writeln!(log_sink, "Crossover rate: {}", config.crossover_rate);
    let _ = writeln!(log_sink, "Crossover     : Uniform (per-demand)");
    let _ = writeln!(log_sink, "Seed          : {}", config.seed.map(|s| s.to_string()).unwrap_or("random".to_string()));
    let _ = writeln!(log_sink, "Started       : {}", started);
    let _ = writeln!(log_sink, "=========================================");
    let _ = writeln!(log_sink, "");

    loop {
        // --- Termination check ---
        if gen >= config.generation_limit {
            termination_reason = format!("GenerationLimit({})", config.generation_limit);
            break;
        }
        if stagnation >= config.no_improvement_limit {
            termination_reason = format!("NoImprovement({})", config.no_improvement_limit);
            break;
        }
        if let Some(budget) = config.max_runtime {
            if t0.elapsed() >= budget {
                termination_reason = format!("TimeLimit({:.1}s)", budget.as_secs_f64());
                break;
            }
        }

        // RP-411: per-generation wall-clock timers (milliseconds)
        let gen_start = Instant::now();
        let mut t_selection_ms: f64 = 0.0;
        let mut t_crossover_ms: f64 = 0.0;
        let mut t_mutation_ms: f64 = 0.0;
        let mut t_eval_ms: f64 = 0.0;
        let mut t_telemetry_ms: f64 = 0.0;

        // RP-410: per-generation improvement histogram counters (reset each generation)
        let mut gen_moves_peak: u32 = 0;
        let mut gen_moves_shoulder: u32 = 0;
        let mut gen_moves_transition: u32 = 0;
        let mut gen_moves_tail: u32 = 0;
        let mut gen_moves_mixed: u32 = 0;
        let mut gen_moves_neutral: u32 = 0;
        // RP-410: per-generation operator usage counters (reset each generation)
        let mut gen_crossover_count: u32 = 0;
        let mut gen_mutation_count: u32 = 0;

        // `evals` is pre-populated and sorted:
        //   gen 0  → initialized before the loop, tagged "initial"
        //   gen N+1 → replaced at the bottom of each iteration, tagged with operator

        // --- Update global best ---
        let gen_best = &evals[0];
        let improved = match &global_best {
            None => true,
            // RP-408A: use pluggable comparator for global-best improvement detection.
            Some(prev) => comparator.is_better(gen_best, prev),
        };

        if improved {
            let prev_obj = global_best.as_ref().map(|g| -g.fitness()).unwrap_or(f64::INFINITY);
            let new_obj = if gen_best.is_valid() { -gen_best.fitness() } else { f64::INFINITY };
            let _ = writeln!(log_sink,
                "[IMPROVE] Gen {:4}  obj: {:.4} → {:.4}  mlu: {:.4}  valid: {}",
                gen, prev_obj, new_obj, gen_best.mlu, gen_best.valid);
            // RP-410: emit MoveRecord for this accepted improvement
            if let Some(ref prev) = global_best {
                let deltas = ZoneDeltas::compute(&prev.load_vector, &gen_best.load_vector);
                let move_class = deltas.classify(1e-9).to_string();
                let new_sdi = compute_sdi(&gen_best.load_vector);
                let move_rec = MoveRecord {
                    record_type: "move",
                    run_uuid: run_uuid.clone(),
                    comparator_mode: config.comparator_mode,
                    instance: instance_name.to_string(),
                    seed: config.seed.unwrap_or(0),
                    generation: gen as u32,
                    operator: gen_best.operator,
                    deltas,
                    move_class,
                    new_obj: if gen_best.is_valid() { -gen_best.fitness() } else { f64::INFINITY },
                    prev_obj: if prev.is_valid() { -prev.fitness() } else { f64::INFINITY },
                    new_mlu: gen_best.mlu,
                    new_sdi,
                };
                telemetry.emit_move(&move_rec);
                // Accumulate histogram (use move_rec.move_class since move_class was moved into the struct)
                match move_rec.move_class.as_str() {
                    "peak"       => gen_moves_peak += 1,
                    "shoulder"   => gen_moves_shoulder += 1,
                    "transition" => gen_moves_transition += 1,
                    "tail"       => gen_moves_tail += 1,
                    "mixed"      => gen_moves_mixed += 1,
                    _            => gen_moves_neutral += 1,
                }
            }
            global_best = Some(gen_best.clone());
            // RP-409B: update peak-demand set for PeakTargetedMutator.
            if let Some(ref pds) = config.peak_demand_set {
                update_peak_demands(pds, gen_best.genome(), &gen_best.load_vector);
            }
            best_found_at_gen = gen;
            stagnation = 0;
        } else {
            stagnation += 1;
        }

        // --- Progress log (Level 1) ---
        if gen % config.log_interval == 0 {
            let best_obj = global_best.as_ref()
                .map(|g| if g.is_valid() { -g.fitness() } else { f64::INFINITY })
                .unwrap_or(f64::INFINITY);
            let best_mlu = global_best.as_ref().map(|g| g.mlu).unwrap_or(f64::INFINITY);
            let valid_count = evals.iter().filter(|e| e.is_valid()).count();
            let elapsed = t0.elapsed().as_secs_f64();
            let _ = writeln!(log_sink,
                "Gen {:4}/{} | best_obj={:.4} mlu={:.4} | valid={}/{} | stagnation={} | {:.1}s",
                gen, config.generation_limit,
                best_obj, best_mlu,
                valid_count, config.population_size,
                stagnation, elapsed);
        }

        // --- Population health (Level 4) ---
        if gen % config.health_interval == 0 && gen > 0 {
            let unique: std::collections::HashSet<String> = evals.iter()
                .map(|e| format!("{:.6}", e.fitness()))
                .collect();
            let avg_waypoints: f64 = evals.iter()
                .map(|e| e.genome().waypoints.iter().filter(|w| !w.is_empty()).count() as f64)
                .sum::<f64>() / evals.len() as f64;
            let _ = writeln!(log_sink,
                "  [HEALTH] unique_fitness={}/{}  avg_nonempty_waypoints={:.2}",
                unique.len(), evals.len(), avg_waypoints);
        }

        // RP-411 Phase 2: GenerationRecord is emitted AFTER the operator and eval phases
        // so that all timing accumulators (t_selection_ms, t_crossover_ms, t_mutation_ms,
        // t_eval_ms) are populated before the record is written. See emit block below.

        // --- Build next generation ---
        // RC-003 fix: elite selection must only carry forward VALID individuals.
        // Previously evals[..elite_count] was taken unconditionally — invalid individuals
        // that sorted into the top slots (due to crossover producing high-fitness-but-invalid
        // offspring) were preserved as elites and re-logged identically every generation.
        // Fix: collect only valid individuals from the sorted population, capped at elite_count.
        let valid_elites: Vec<&RoadefEvaluation> = evals.iter()
            .filter(|e| e.is_valid())
            .take(config.elite_count)
            .collect();
        let elite_count = valid_elites.len();

        // RP-410C Phase 2: next_pop carries full lineage metadata alongside each genome.
        // Tuple: (genome, operator_tag, parent1_id, parent2_id, tournament_id, won_tournament)
        // Elite individuals are carried forward unchanged; their tournament_id is 0 (no
        // tournament), won_tournament is true (they survived the previous generation's sort).
        let mut next_pop: Vec<(RoadefGenome, &'static str, u64, u64, u64, bool)> = valid_elites
            .iter()
            .map(|e| {
                candidate_counter += 1;
                (e.genome().clone(), "elite", candidate_counter, 0u64, 0u64, true)
            })
            .collect();

        // RP-411 Phase 2: separate Instant timers for selection, crossover, mutation.
        // Selection = time spent inside run_tournament! macro calls.
        // Crossover/mutation = time spent applying operators after tournament winners are chosen.
        // These are accumulated per-generation and reset at the top of each iteration.

        // RP-410C Phase 2: tournament counter for this generation.
        let mut tournament_counter: u64 = 0;

        // RP-410C Phase 2: tournament selection with full telemetry.
        // The select_with_id closure returns (winner_ref, winner_candidate_id, tournament_id).
        // It also emits CandidateRecords for every tournament loser so the full
        // tournament funnel is observable.
        //
        // Implementation note: evals is sorted descending by fitness. Each individual
        // in evals already has a stable candidate_id from the previous generation's
        // emit loop. We track those IDs in a parallel Vec<u64> that is rebuilt each
        // generation alongside evals.
        //
        // For Phase 2 we use a simpler approach: assign a fresh candidate_id to every
        // tournament participant at the point of selection, and emit loser records
        // immediately. Winners carry their ID into next_pop.

        // RP-410C Phase 2: inline tournament selection macro.
        // Runs a k=3 tournament, emits CandidateRecords for every loser immediately,
        // and returns (winner_index, winner_candidate_id).
        // Uses a macro instead of a closure to avoid borrow-checker conflicts with
        // `telemetry` (which is &mut dyn TelemetrySink and cannot be passed as
        // &dyn TelemetrySink through a closure parameter).
        macro_rules! run_tournament {
            ($tourn_id:expr) => {{
                let k = 3.min(evals.len());
                let mut best_idx = rng.gen_range(0..evals.len());
                candidate_counter += 1;
                let mut winner_cid = candidate_counter;
                let mut participant_idxs = vec![best_idx];
                for _ in 1..k {
                    let idx = rng.gen_range(0..evals.len());
                    participant_idxs.push(idx);
                    // RP-408A: use pluggable comparator instead of raw fitness().
                    if comparator.is_better(&evals[idx], &evals[best_idx]) {
                        best_idx = idx;
                        candidate_counter += 1;
                        winner_cid = candidate_counter;
                    }
                }
                // Emit loser records for all non-winner participants.
                for &idx in &participant_idxs {
                    if idx == best_idx { continue; }
                    candidate_counter += 1;
                    let loser_cid = candidate_counter;
                    let ev = &evals[idx];
                    let (deltas, move_class) = if let Some(ref best) = global_best {
                        let d = ZoneDeltas::compute(&best.load_vector, &ev.load_vector);
                        let mc = d.classify(1e-9).to_string();
                        (d, mc)
                    } else {
                        (ZoneDeltas { delta_rank1: 0.0, delta_2_20: 0.0, delta_21_100: 0.0, delta_tail: 0.0 },
                         "neutral".to_string())
                    };
                    let loser_rec = CandidateRecord {
                        record_type: "candidate",
                        run_uuid: run_uuid.clone(),
                        comparator_mode: config.comparator_mode,
                        instance: instance_name.to_string(),
                        seed: config.seed.unwrap_or(0),
                        generation: gen as u32,
                        candidate_id: loser_cid,
                        parent1: 0,
                        parent2: 0,
                        operator: ev.operator,
                        tournament_id: ($tourn_id as u32),
                        deltas,
                        move_class,
                        obj: if ev.is_valid() { -ev.fitness() } else { f64::INFINITY },
                        valid: ev.is_valid(),
                        won_tournament: false,
                        population_slot: None,
                        elite_slot: None,
                        became_global_best: false,
                        decision_stage: "Tournament",
                        reason: Some("LostTournament"),
                    };
                    telemetry.emit_candidate(&loser_rec);
                }
                (best_idx, winner_cid)
            }};
        }

        while next_pop.len() < config.population_size {
            tournament_counter += 1;
            let tourn_id = tournament_counter;

            if rng.gen_bool(config.crossover_rate) && next_pop.len() + 1 < config.population_size {
                // RP-411 Phase 2: time tournament selection separately from operator application.
                let t_sel_a = Instant::now();
                let (pa_idx, pa_cid) = run_tournament!(tourn_id);
                t_selection_ms += t_sel_a.elapsed().as_secs_f64() * 1000.0;

                tournament_counter += 1;
                let tourn_id_b = tournament_counter;

                let t_sel_b = Instant::now();
                let (pb_idx, pb_cid) = run_tournament!(tourn_id_b);
                t_selection_ms += t_sel_b.elapsed().as_secs_f64() * 1000.0;

                // RP-411 Phase 2: time crossover + optional mutation.
                // RC-003 pipeline fix: Crossover → Mutation → Repair → Evaluate.
                // Repair fires AFTER all destructive operators so it catches infeasibility
                // from both crossover and mutation in a single pass.
                let t_xo = Instant::now();
                let pa = evals[pa_idx].genome().clone();
                let pb = evals[pb_idx].genome().clone();
                let (mut ca, mut cb) = crossover.crossover(&pa, &pb, &mut rng);
                let mut ca_tag: &'static str = "crossover";
                let mut cb_tag: &'static str = "crossover";

                // Step 1: optional mutation (before repair).
                if rng.gen_bool(config.mutation_rate) {
                    mutator.mutate(&mut ca, &mut rng);
                    ca_tag = "crossover+mutation";
                }
                if rng.gen_bool(config.mutation_rate) {
                    mutator.mutate(&mut cb, &mut rng);
                    cb_tag = "crossover+mutation";
                }

                // Step 2: repair — evaluate after all destructive operators.
                // If the child is invalid, fall back in priority order:
                //   1. Valid parent (pa for child A, pb for child B)
                //   2. Other valid parent
                //   3. Best valid individual in current evals (previous generation)
                //   4. global_best (always valid if set; survives across generations)
                //   5. Keep as-is (population fully infeasible, no valid reference exists)
                // Tag is updated to "crossover_repaired" so the repair rate is observable.
                let ca_valid = fitness_eval.evaluate(&ca).is_valid();
                if !ca_valid {
                    ca = if evals[pa_idx].is_valid() {
                        pa.clone()
                    } else if evals[pb_idx].is_valid() {
                        pb.clone()
                    } else if let Some(best_valid) = evals.iter().find(|e| e.is_valid()) {
                        best_valid.genome().clone()
                    } else if let Some(ref gb) = global_best {
                        gb.genome().clone() // global_best is always valid
                    } else {
                        pa.clone() // no valid reference exists — keep as-is
                    };
                    ca_tag = "crossover_repaired";
                }
                let cb_valid = fitness_eval.evaluate(&cb).is_valid();
                if !cb_valid {
                    cb = if evals[pb_idx].is_valid() {
                        pb.clone()
                    } else if evals[pa_idx].is_valid() {
                        pa.clone()
                    } else if let Some(best_valid) = evals.iter().find(|e| e.is_valid()) {
                        best_valid.genome().clone()
                    } else if let Some(ref gb) = global_best {
                        gb.genome().clone() // global_best is always valid
                    } else {
                        pb.clone() // no valid reference exists — keep as-is
                    };
                    cb_tag = "crossover_repaired";
                }
                t_crossover_ms += t_xo.elapsed().as_secs_f64() * 1000.0;

                gen_crossover_count += 1;
                next_pop.push((ca, ca_tag, pa_cid, pb_cid, tourn_id, true));
                if next_pop.len() < config.population_size {
                    next_pop.push((cb, cb_tag, pa_cid, pb_cid, tourn_id_b, true));
                }
            } else {
                // RP-411 Phase 2: time tournament selection separately from mutation.
                let t_sel = Instant::now();
                let (pa_idx, pa_cid) = run_tournament!(tourn_id);
                t_selection_ms += t_sel.elapsed().as_secs_f64() * 1000.0;

                // RP-411 Phase 2: time mutation.
                let t_mut = Instant::now();
                let pa = evals[pa_idx].genome().clone();
                let mut child = pa;
                mutator.mutate(&mut child, &mut rng);
                t_mutation_ms += t_mut.elapsed().as_secs_f64() * 1000.0;

                // RC-003 pipeline fix: repair after mutation.
                // Fall back in priority order:
                //   1. Valid parent
                //   2. Best valid individual in current evals (previous generation)
                //   3. global_best (always valid if set; survives across generations)
                //   4. Keep as-is (no valid reference exists)
                let mut child_tag = "mutation";
                if !fitness_eval.evaluate(&child).is_valid() {
                    child = if evals[pa_idx].is_valid() {
                        evals[pa_idx].genome().clone()
                    } else if let Some(best_valid) = evals.iter().find(|e| e.is_valid()) {
                        best_valid.genome().clone()
                    } else if let Some(ref gb) = global_best {
                        gb.genome().clone() // global_best is always valid
                    } else {
                        child // no valid reference exists — keep as-is
                    };
                    child_tag = "mutation_repaired";
                }

                gen_mutation_count += 1;
                next_pop.push((child, child_tag, pa_cid, 0u64, tourn_id, true));
            }
        }
        // RP-411 Phase 2: t_selection_ms, t_crossover_ms, t_mutation_ms are now
        // measured directly with separate Instant timers inside the loop above.
        // No approximation is applied here.

        // RP-411: time evaluation phase
        let t_eval_start = Instant::now();
        // RP-409C: Evaluate next generation and carry full lineage metadata (cid, p1, p2, tid)
        // alongside each RoadefEvaluation so the post-sort emit block can record them.
        // Previously these were discarded at the eval map boundary; this is the fix.
        //
        // Each tuple: (RoadefEvaluation, candidate_id, parent1_id, parent2_id, tournament_id)
        // RC-002: per-generation invalid-by-origin counters.
        // Classified by overload severity:
        //   epsilon:    max_sat ≤ 1.0 + 1e-5  (float rounding, physically feasible)
        //   minor:      max_sat ≤ 1.01         (≤1% overload, likely crossover accumulation)
        //   major:      max_sat > 1.01         (>1% overload, structural capacity violation)
        //   structural: max_sat == 0.0         (compute_loads() returned None — structural failure)
        let mut rc002_inv: [u32; 16] = [0u32; 16]; // [origin*4 + class]: origin=0..3, class=0..3
        // origin index: 0=initial, 1=crossover, 2=mutation, 3=elite
        // class index:  0=epsilon, 1=minor, 2=major, 3=structural

        let mut new_evals_with_meta: Vec<(RoadefEvaluation, u64, u64, u64, u64)> =
            next_pop.into_iter().map(|(g, tag, p1, p2, tid, _won)| {
                let mut ev = fitness_eval.evaluate(&g);
                ev.operator = tag;
                candidate_counter += 1;
                let cid = candidate_counter;

                // RC-002: emit [diag] with origin tag for invalid genomes that have waypoints.
                // Moved here from evaluate() so the operator tag is available for origin context.
                if !ev.valid {
                    let has_waypoints = ev.genome.waypoints.iter().any(|w| !w.is_empty());
                    if has_waypoints {
                        let sat = ev.max_sat;
                        let (overload_class, class_idx) = if sat == 0.0 {
                            ("structural", 3usize)
                        } else if sat <= 1.0 + 1e-5 {
                            ("epsilon", 0usize)
                        } else if sat <= 1.01 {
                            ("minor", 1usize)
                        } else {
                            ("major", 2usize)
                        };
                        let solution = ev.genome.to_solution();
                        let diag_reason = fitness_eval.evaluator.diagnose_failure(&solution)
                            .unwrap_or_else(|| format!("arc overloaded max_sat={:.9}", sat));
                        eprintln!("[diag] gen={} origin={} overload={} max_sat={:.9} | {}",
                            gen, tag, overload_class, sat, diag_reason);

                        // Accumulate per-generation counters.
                        // "crossover+mutation" counts as crossover for origin.
                        let origin_idx: usize = if tag.starts_with("crossover") { 1 }
                            else if tag == "mutation" { 2 }
                            else if tag == "elite"    { 3 }
                            else                      { 0 }; // "initial"
                        rc002_inv[origin_idx * 4 + class_idx] += 1;
                    }
                }

                (ev, cid, p1, p2, tid)
            }).collect();

        // Sort descending by comparator order (best first).
        // RP-408A: use pluggable comparator instead of raw fitness().
        new_evals_with_meta.sort_by(|(a, ..), (b, ..)| comparator.cmp_evals(b, a).then(
            b.fitness().partial_cmp(&a.fitness()).unwrap_or(Ordering::Equal)
        ));
        t_eval_ms += t_eval_start.elapsed().as_secs_f64() * 1000.0;
// RC-002: emit per-generation invalid-by-origin summary when any invalids occurred.
        // Format: [rc002] gen=N  initial: eps=A min=B maj=C str=D  crossover: ...  mutation: ...  elite: ...
        // Only emitted when total invalid count > 0 to keep logs clean for healthy generations.
        {
            let total_inv: u32 = rc002_inv.iter().sum();
            if total_inv > 0 {
                eprintln!(
                    "[rc002] gen={:4}  initial: eps={} min={} maj={} str={}  crossover: eps={} min={} maj={} str={}  mutation: eps={} min={} maj={} str={}  elite: eps={} min={} maj={} str={}",
                    gen,
                    rc002_inv[0], rc002_inv[1], rc002_inv[2], rc002_inv[3],     // initial
                    rc002_inv[4], rc002_inv[5], rc002_inv[6], rc002_inv[7],     // crossover
                    rc002_inv[8], rc002_inv[9], rc002_inv[10], rc002_inv[11],   // mutation
                    rc002_inv[12], rc002_inv[13], rc002_inv[14], rc002_inv[15], // elite
                );
            }
        }

        // RP-409C: emit winner CandidateRecords now that population_slot is known.
        // parent1, parent2, tournament_id, and candidate_id are all correctly propagated
        // from the next_pop build phase through the evaluation map.
        for (slot, (ev, cid, p1, p2, tid)) in new_evals_with_meta.iter().enumerate() {
            let (deltas, move_class) = if let Some(ref best) = global_best {
                let d = ZoneDeltas::compute(&best.load_vector, &ev.load_vector);
                let mc = d.classify(1e-9).to_string();
                (d, mc)
            } else {
                (ZoneDeltas { delta_rank1: 0.0, delta_2_20: 0.0, delta_21_100: 0.0, delta_tail: 0.0 },
                 "neutral".to_string())
            };

            let became_global_best = match &global_best {
                None => ev.is_valid(),
                // RP-408A: use pluggable comparator for global-best comparison.
                Some(best) => comparator.is_better(ev, best),
            };

            let pop_slot = Some(slot);
            let el_slot = if slot < elite_count { Some(slot) } else { None };

            // RP-409C: full decision-stage classification.
            // Evaluation → infeasible candidate eliminated before selection.
            // GlobalBest  → candidate improved the global best.
            // Elite       → candidate entered the elite archive.
            // Population  → candidate entered the population but not the elite.
            let (decision_stage, reason): (&'static str, Option<&'static str>) =
                if !ev.is_valid() {
                    ("Evaluation", Some("CapacityViolation"))
                } else if became_global_best {
                    ("GlobalBest", None)
                } else if el_slot.is_some() {
                    ("Elite", Some("EnteredElite"))
                } else {
                    ("Population", Some("EnteredPopulation"))
                };

            let cand_rec = CandidateRecord {
                record_type: "candidate",
                run_uuid: run_uuid.clone(),
                comparator_mode: config.comparator_mode,
                instance: instance_name.to_string(),
                seed: config.seed.unwrap_or(0),
                generation: gen as u32,
                candidate_id: *cid,
                parent1: *p1,   // RP-409C: correctly propagated from next_pop build phase
                parent2: *p2,   // RP-409C: correctly propagated from next_pop build phase
                operator: ev.operator,
                tournament_id: *tid as u32, // RP-409C: correctly propagated from next_pop build phase
                deltas,
                move_class,
                obj: if ev.is_valid() { -ev.fitness() } else { f64::INFINITY },
                valid: ev.is_valid(),
                won_tournament: true, // all candidates in new_evals_with_meta won their tournament
                population_slot: pop_slot,
                elite_slot: el_slot,
                became_global_best,
                decision_stage,
                reason,
            };
            telemetry.emit_candidate(&cand_rec);
        }

        // Extract plain evals for the rest of the loop (GenerationRecord, next iteration).
        let new_evals: Vec<RoadefEvaluation> = new_evals_with_meta.into_iter().map(|(ev, ..)| ev).collect();

        // --- RP-410 / RP-411 Phase 2: emit GenerationRecord ---
        // Emitted here (after selection, crossover, mutation, eval, and sort) so that
        // all per-phase timing accumulators are populated with the current generation's
        // measured values. valid_count and unique_fitness_count reflect the new population.
        {
            let best_sdi = global_best.as_ref()
                .map(|g| compute_sdi(&g.load_vector))
                .unwrap_or(0.0);
            let top20_prefix: Vec<f64> = global_best.as_ref()
                .map(|g| g.load_vector.iter().take(20).cloned().collect())
                .unwrap_or_default();
            let unique_fitness_count = {
                let unique: std::collections::HashSet<String> = new_evals.iter()
                    .map(|e| format!("{:.6}", e.fitness()))
                    .collect();
                unique.len()
            };
            // RP-411 Phase 2: all timing accumulators are now populated.
            // total_gen_time_ms = wall-clock from gen_start to now (includes emit overhead).
            let total_so_far_ms = gen_start.elapsed().as_secs_f64() * 1000.0;
            let accounted_ms = t_selection_ms + t_crossover_ms + t_mutation_ms + t_eval_ms;
            let other_ms = (total_so_far_ms - accounted_ms).max(0.0);

            let t_tel_start = Instant::now();
            let gen_rec = GenerationRecord {
                record_type: "generation",
                run_uuid: run_uuid.clone(),
                comparator_mode: config.comparator_mode,
                instance: instance_name.to_string(),
                seed: config.seed.unwrap_or(0),
                generation: gen as u32,
                best_obj: global_best.as_ref()
                    .map(|g| if g.is_valid() { -g.fitness() } else { f64::INFINITY })
                    .unwrap_or(f64::INFINITY),
                best_mlu: global_best.as_ref().map(|g| g.mlu).unwrap_or(f64::INFINITY),
                best_sdi,
                top20_prefix,
                valid_count: new_evals.iter().filter(|e| e.is_valid()).count(),
                population_size: config.population_size,
                unique_fitness_count,
                stagnation,
                moves_peak: gen_moves_peak,
                moves_shoulder: gen_moves_shoulder,
                moves_transition: gen_moves_transition,
                moves_tail: gen_moves_tail,
                moves_mixed: gen_moves_mixed,
                moves_neutral: gen_moves_neutral,
                crossover_count: gen_crossover_count,
                mutation_count: gen_mutation_count,
                // RP-407: only meaningful at gen 0; zero for all subsequent generations.
                generation0_valid_count: if gen == 0 { generation0_valid_count } else { 0 },
                // RP-411 Phase 2: per-phase timing fields — measured directly this generation.
                eval_time_ms: t_eval_ms,
                crossover_time_ms: t_crossover_ms,
                mutation_time_ms: t_mutation_ms,
                repair_time_ms: 0.0, // repair is not yet a separate phase in this harness
                selection_time_ms: t_selection_ms,
                telemetry_time_ms: 0.0, // approximation: emit cost not yet measured
                other_time_ms: other_ms,
                total_gen_time_ms: total_so_far_ms,
            };
            telemetry.emit_generation(&gen_rec);
            t_telemetry_ms += t_tel_start.elapsed().as_secs_f64() * 1000.0;
        }

        evals = new_evals;

        gen += 1;
    }

    let runtime_ms = t0.elapsed().as_millis();

    // --- Termination summary (Level 3) ---
    let best = global_best.as_ref();
    let best_obj = best.map(|g| if g.is_valid() { -g.fitness() } else { f64::INFINITY }).unwrap_or(f64::INFINITY);
    let best_mlu = best.map(|g| g.mlu).unwrap_or(f64::INFINITY);
    let valid = best.map(|g| g.is_valid()).unwrap_or(false);

    let _ = writeln!(log_sink, "");
    let _ = writeln!(log_sink, "[TERMINATION]");
    let _ = writeln!(log_sink, "  Reason       : {}", termination_reason);
    let _ = writeln!(log_sink, "  Generations  : {}", gen);
    let _ = writeln!(log_sink, "  Best obj     : {:.4}", best_obj);
    let _ = writeln!(log_sink, "  Best MLU     : {:.4}", best_mlu);
    let _ = writeln!(log_sink, "  Valid        : {}", valid);
    let _ = writeln!(log_sink, "  Best at gen  : {}", best_found_at_gen);
    let _ = writeln!(log_sink, "  Runtime      : {}ms", runtime_ms);
    let _ = writeln!(log_sink, "");
    let _ = writeln!(log_sink, "=========================================");
    let _ = writeln!(log_sink, "Finished      : {}", chrono::Utc::now().to_rfc3339());
    let _ = writeln!(log_sink, "Runtime       : {}ms", runtime_ms);
    let _ = writeln!(log_sink, "Termination   : {}", termination_reason);
    let _ = writeln!(log_sink, "Best Objective: {:.4}", best_obj);
    let _ = writeln!(log_sink, "Best MLU      : {:.4}", best_mlu);
    let _ = writeln!(log_sink, "Valid         : {}", valid);
    let _ = writeln!(log_sink, "=========================================");

    telemetry.flush();

    EvolutionRunResult {
        best_genome: best.map(|g| g.genome().clone()).unwrap_or_else(|| factory.create(&mut rng)),
        best_obj,
        best_mlu,
        valid,
        generations_run: gen,
        best_found_at_gen,
        termination_reason,
        runtime_ms,
        initial_feasibility_rate,
        gen0_best_obj,
        gen0_mean_obj,
        gen0_feasible_count,
        gen0_unique_obj_count,
        gen0_duplicate_genome_count,
    }
}
// ---------------------------------------------------------------------------
// RP-408A: Comparator unit tests + scalar equivalence regression
// ---------------------------------------------------------------------------

#[cfg(test)]
mod comparator_tests {
    use super::*;
    use std::cmp::Ordering;

    // Helper: construct a minimal RoadefEvaluation with a given load vector and validity.
    // The genome fields are irrelevant for comparator tests.
    fn make_eval(load_vector: Vec<f64>, valid: bool) -> RoadefEvaluation {
        let obj = if valid { load_vector.first().copied().unwrap_or(0.0) } else { f64::INFINITY };
        RoadefEvaluation {
            genome: RoadefGenome { waypoints: vec![], num_time_slots: 0 },
            obj,
            valid,
            mlu: load_vector.first().copied().unwrap_or(0.0),
            load_vector,
            operator: "test",
        }
    }

    // -----------------------------------------------------------------------
    // ScalarComparator tests
    // -----------------------------------------------------------------------

    #[test]
    fn scalar_valid_beats_invalid() {
        let cmp = ScalarComparator;
        let valid   = make_eval(vec![0.9], true);
        let invalid = make_eval(vec![0.5], false);
        assert!(cmp.is_better(&valid, &invalid),
            "valid should beat invalid under ScalarComparator");
        assert!(!cmp.is_better(&invalid, &valid),
            "invalid should not beat valid under ScalarComparator");
    }

    #[test]
    fn scalar_lower_obj_wins() {
        let cmp = ScalarComparator;
        // fitness = -obj, so lower obj = higher fitness = better
        let better = make_eval(vec![0.8], true);  // obj=0.8, fitness=-0.8
        let worse  = make_eval(vec![0.9], true);  // obj=0.9, fitness=-0.9
        assert!(cmp.is_better(&better, &worse));
        assert!(!cmp.is_better(&worse, &better));
    }

    #[test]
    fn scalar_equal_fitness_is_equal() {
        let cmp = ScalarComparator;
        let a = make_eval(vec![0.7], true);
        let b = make_eval(vec![0.7], true);
        assert_eq!(cmp.cmp_evals(&a, &b), Ordering::Equal);
    }

    // -----------------------------------------------------------------------
    // LexicographicComparator tests
    // -----------------------------------------------------------------------

    #[test]
    fn lex_valid_beats_invalid() {
        let cmp = LexicographicComparator;
        let valid   = make_eval(vec![0.9, 0.8], true);
        let invalid = make_eval(vec![0.1, 0.1], false);
        assert!(cmp.is_better(&valid, &invalid),
            "valid should beat invalid under LexicographicComparator");
        assert!(!cmp.is_better(&invalid, &valid));
    }

    #[test]
    fn lex_both_invalid_is_equal() {
        let cmp = LexicographicComparator;
        let a = make_eval(vec![0.9], false);
        let b = make_eval(vec![0.1], false);
        assert_eq!(cmp.cmp_evals(&a, &b), Ordering::Equal,
            "two invalid solutions should compare Equal");
    }

    #[test]
    fn lex_first_rank_decides() {
        // [100, 80, 70] vs [101, 10, 10]: first has lower rank-1 → first wins
        let cmp = LexicographicComparator;
        let better = make_eval(vec![100.0, 80.0, 70.0], true);
        let worse  = make_eval(vec![101.0, 10.0, 10.0], true);
        assert!(cmp.is_better(&better, &worse),
            "[100,80,70] should beat [101,10,10] (lower rank-1 load)");
        assert!(!cmp.is_better(&worse, &better));
    }

    #[test]
    fn lex_tie_at_rank1_second_rank_decides() {
        // [100, 81, 60] vs [100, 82, 10]: tie at rank-1, first wins at rank-2
        let cmp = LexicographicComparator;
        let better = make_eval(vec![100.0, 81.0, 60.0], true);
        let worse  = make_eval(vec![100.0, 82.0, 10.0], true);
        assert!(cmp.is_better(&better, &worse),
            "[100,81,60] should beat [100,82,10] (lower rank-2 load)");
    }

    #[test]
    fn lex_equal_vectors_is_equal() {
        let cmp = LexicographicComparator;
        let a = make_eval(vec![0.9, 0.8, 0.7], true);
        let b = make_eval(vec![0.9, 0.8, 0.7], true);
        assert_eq!(cmp.cmp_evals(&a, &b), Ordering::Equal,
            "identical load vectors should compare Equal");
    }

    #[test]
    fn lex_shorter_vector_treated_as_zero_padded() {
        // [0.9] vs [0.9, 0.5]: tie at rank-1, second has 0.5 at rank-2 vs 0.0 → first wins
        let cmp = LexicographicComparator;
        let shorter = make_eval(vec![0.9], true);
        let longer  = make_eval(vec![0.9, 0.5], true);
        // shorter has 0.0 at rank-2 (missing), longer has 0.5 → shorter is better
        assert!(cmp.is_better(&shorter, &longer),
            "shorter vector (zero-padded) should beat longer with non-zero tail");
    }

    // -----------------------------------------------------------------------
    // Scalar equivalence regression: ScalarComparator must agree with the
    // pre-RP-408A fitness()-based comparison on all orderings.
    // -----------------------------------------------------------------------

    #[test]
    fn scalar_agrees_with_fitness_comparison_valid_vs_valid() {
        let cmp = ScalarComparator;
        let cases: Vec<(f64, f64)> = vec![
            (0.5, 0.9),   // a better
            (0.9, 0.5),   // b better
            (0.7, 0.7),   // equal
            (0.0, 1.0),   // a much better
            (1.0, 0.0),   // b much better
        ];
        for (obj_a, obj_b) in cases {
            let a = make_eval(vec![obj_a], true);
            let b = make_eval(vec![obj_b], true);
            // Pre-RP-408A: a better iff a.fitness() > b.fitness() iff -obj_a > -obj_b iff obj_a < obj_b
            let legacy_a_better = a.fitness() > b.fitness();
            let cmp_a_better    = cmp.is_better(&a, &b);
            assert_eq!(cmp_a_better, legacy_a_better,
                "ScalarComparator disagrees with legacy fitness() for obj_a={obj_a} obj_b={obj_b}");
        }
    }

    #[test]
    fn scalar_agrees_with_fitness_comparison_valid_vs_invalid() {
        let cmp = ScalarComparator;
        let valid   = make_eval(vec![0.9], true);
        let invalid = make_eval(vec![0.1], false);
        // Legacy: valid.fitness() = -0.9, invalid.fitness() = -1_000_000.0
        // valid is better
        assert_eq!(cmp.is_better(&valid, &invalid), valid.fitness() > invalid.fitness());
        assert_eq!(cmp.is_better(&invalid, &valid), invalid.fitness() > valid.fitness());
    }

    #[test]
    fn scalar_sort_order_matches_legacy_sort() {
        let cmp = ScalarComparator;
        let mut evals = vec![
            make_eval(vec![0.9], true),
            make_eval(vec![0.5], true),
            make_eval(vec![0.7], true),
            make_eval(vec![0.3], false),
            make_eval(vec![0.1], true),
        ];
        // Sort using ScalarComparator (descending: best first)
        evals.sort_by(|a, b| cmp.cmp_evals(b, a).then(
            b.fitness().partial_cmp(&a.fitness()).unwrap_or(Ordering::Equal)
        ));
        let cmp_order: Vec<f64> = evals.iter().map(|e| e.obj).collect();

        // Sort using legacy fitness() comparison
        let mut legacy = vec![
            make_eval(vec![0.9], true),
            make_eval(vec![0.5], true),
            make_eval(vec![0.7], true),
            make_eval(vec![0.3], false),
            make_eval(vec![0.1], true),
        ];
        legacy.sort_by(|a, b| b.fitness().partial_cmp(&a.fitness()).unwrap_or(Ordering::Equal));
        let legacy_order: Vec<f64> = legacy.iter().map(|e| e.obj).collect();

        assert_eq!(cmp_order, legacy_order,
            "ScalarComparator sort order must match legacy fitness() sort order");
    }
}
