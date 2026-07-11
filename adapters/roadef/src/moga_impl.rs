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
use std::sync::Arc;

use coralys_moga::traits::{
    Genome, GenomeFactory, FitnessEvaluator, Evaluated, MutationOperator, CrossoverOperator,
};

use crate::evaluator::RoadefEvaluator;
use crate::models::{Solution, SrPath};

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

/// Creates random genomes by assigning 0–1 random waypoints per demand.
/// The waypoints are drawn from the set of valid node IDs in the network.
pub struct RoadefGenomeFactory {
    pub num_demands: usize,
    pub num_time_slots: usize,
    /// All node IDs in the network (for random waypoint selection)
    pub node_ids: Vec<u64>,
}

impl GenomeFactory<RoadefGenome> for RoadefGenomeFactory {
    fn create(&self, rng: &mut StdRng) -> RoadefGenome {
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

        // Compute average MLU across time slots for reporting (only for valid solutions)
        let mlu = if valid {
            let mut total_mlu = 0.0;
            let mut count = 0;
            for t in 0..genome.num_time_slots {
                if let Some(loads) = self.evaluator.compute_loads(t, &solution) {
                    total_mlu += loads.mlu;
                    count += 1;
                }
            }
            if count > 0 { total_mlu / count as f64 } else { 0.0 }
        } else {
            f64::INFINITY
        };

        RoadefEvaluation {
            genome: genome.clone(),
            obj: result.obj,
            valid,
            mlu,
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
}

/// Run ROADEF evolution with active per-generation logging.
///
/// `log_sink`: any Write target (file, stderr, Vec<u8>).
/// Returns the best result found.
pub fn run_roadef_evolution(
    factory: &RoadefGenomeFactory,
    fitness_eval: &RoadefFitnessEvaluator,
    mutator: &RoadefMutator,
    crossover: &RoadefCrossover,
    config: &EvolutionRunConfig,
    instance_name: &str,
    log_sink: &mut dyn Write,
) -> EvolutionRunResult {
    let mut rng: StdRng = match config.seed {
        Some(s) => StdRng::seed_from_u64(s),
        None => StdRng::from_entropy(),
    };

    let t0 = Instant::now();

    // --- Initialize population ---
    let mut population: Vec<RoadefGenome> = (0..config.population_size)
        .map(|_| factory.create(&mut rng))
        .collect();

    let mut global_best: Option<RoadefEvaluation> = None;
    let mut best_found_at_gen = 0usize;
    let mut stagnation = 0usize;
    let mut gen = 0usize;
    let mut termination_reason = String::new(); // set before every break in the loop below

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

        // --- Evaluate population ---
        let mut evals: Vec<RoadefEvaluation> = population.iter()
            .map(|g| fitness_eval.evaluate(g))
            .collect();

        // Sort descending by fitness (higher = better)
        evals.sort_by(|a, b| b.fitness().partial_cmp(&a.fitness()).unwrap_or(std::cmp::Ordering::Equal));

        // --- Update global best ---
        let gen_best = &evals[0];
        let improved = match &global_best {
            None => true,
            Some(prev) => gen_best.fitness() > prev.fitness(),
        };

        if improved {
            let prev_obj = global_best.as_ref().map(|g| -g.fitness()).unwrap_or(f64::INFINITY);
            let new_obj = if gen_best.is_valid() { -gen_best.fitness() } else { f64::INFINITY };
            let _ = writeln!(log_sink,
                "[IMPROVE] Gen {:4}  obj: {:.4} → {:.4}  mlu: {:.4}  valid: {}",
                gen, prev_obj, new_obj, gen_best.mlu, gen_best.valid);
            global_best = Some(gen_best.clone());
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

        // --- Build next generation ---
        let elite_count = config.elite_count.min(evals.len());
        let mut next_pop: Vec<RoadefGenome> = evals[..elite_count]
            .iter()
            .map(|e| e.genome().clone())
            .collect();

        while next_pop.len() < config.population_size {
            // Tournament selection (k=3)
            let select = |rng: &mut StdRng| -> &RoadefEvaluation {
                let k = 3.min(evals.len());
                let mut best_idx = rng.gen_range(0..evals.len());
                for _ in 1..k {
                    let idx = rng.gen_range(0..evals.len());
                    if evals[idx].fitness() > evals[best_idx].fitness() {
                        best_idx = idx;
                    }
                }
                &evals[best_idx]
            };

            if rng.gen_bool(config.crossover_rate) && next_pop.len() + 1 < config.population_size {
                let pa = select(&mut rng).genome().clone();
                let pb = select(&mut rng).genome().clone();
                let (mut ca, mut cb) = crossover.crossover(&pa, &pb, &mut rng);
                if rng.gen_bool(config.mutation_rate) { mutator.mutate(&mut ca, &mut rng); }
                if rng.gen_bool(config.mutation_rate) { mutator.mutate(&mut cb, &mut rng); }
                next_pop.push(ca);
                if next_pop.len() < config.population_size { next_pop.push(cb); }
            } else {
                let mut child = select(&mut rng).genome().clone();
                mutator.mutate(&mut child, &mut rng);
                next_pop.push(child);
            }
        }

        population = next_pop;
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

    EvolutionRunResult {
        best_genome: best.map(|g| g.genome().clone()).unwrap_or_else(|| factory.create(&mut rng)),
        best_obj,
        best_mlu,
        valid,
        generations_run: gen,
        best_found_at_gen,
        termination_reason,
        runtime_ms,
    }
}