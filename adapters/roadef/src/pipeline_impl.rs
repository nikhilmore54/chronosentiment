use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rayon::prelude::*;
use std::cmp::Ordering;
use std::io::Write;
use std::time::Instant;

use crate::constraints::RoadefConstraintModel;
use crate::ecmp::dijkstra_cache_reset;
use crate::moga_impl::{
    make_comparator, update_peak_demands, EvolutionRunConfig, EvolutionRunResult, RoadefCrossover,
    RoadefEvaluation, RoadefFitnessEvaluator, RoadefGenome, RoadefGenomeFactory,
};
use crate::telemetry::compute_sdi;
use crate::telemetry::{
    CandidateRecord, ConstructionRecord, GenerationRecord, MoveRecord, TelemetrySink, ZoneDeltas,
};
use coralys_core::operators::ConstraintModel;
use coralys_moga::traits::{
    CrossoverOperator, Evaluated, FitnessEvaluator, GenomeFactory, MutationOperator,
};

pub fn run_pipeline_evolution<M>(
    factory: &RoadefGenomeFactory,
    fitness_eval: &RoadefFitnessEvaluator,
    mutator: &M,
    crossover: &RoadefCrossover,
    pipeline: &coralys_core::pipeline::EvolutionaryPipeline<
        RoadefGenome,
        RoadefConstraintModel,
        crate::operators::OperatorError,
    >,
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

    let run_uuid = {
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

    let mut evaluation_cache: std::collections::HashMap<RoadefGenome, RoadefEvaluation> =
        std::collections::HashMap::new();
    let t0 = Instant::now();

    let init_budget_fraction = 0.5_f64;
    let init_deadline: Option<std::time::Duration> =
        config.max_runtime.map(|b| b.mul_f64(init_budget_fraction));
    const MAX_INIT_RETRIES: usize = 10;

    let mut evals: Vec<RoadefEvaluation> = Vec::with_capacity(config.population_size);
    let mut n_init_retries: usize = 0;
    let mut n_init_retry_successes: usize = 0;
    for i in 0..config.population_size {
        let g = factory.create(&mut rng);
        let mut ev = fitness_eval.evaluate(
            &g,
            &coralys_moga::runtime::optimization::metric::MetricReport::default(),
        );
        ev.operator = "initial";

        if !ev.is_valid() {
            for _retry in 0..MAX_INIT_RETRIES {
                n_init_retries += 1;
                if let Some(deadline) = init_deadline {
                    if t0.elapsed() >= deadline {
                        break;
                    }
                }
                let g2 = factory.create(&mut rng);
                let mut ev2 = fitness_eval.evaluate(
                    &g2,
                    &coralys_moga::runtime::optimization::metric::MetricReport::default(),
                );
                ev2.operator = "initial";
                if ev2.is_valid() {
                    ev = ev2;
                    n_init_retry_successes += 1;
                    break;
                }
                if ev2.max_sat < ev.max_sat {
                    ev = ev2;
                }
            }
        }
        evals.push(ev);

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
        let _ = writeln!(
            log_sink,
            "[init] rejection sampling: {} retries, {} successes ({:.0}% repair rate)",
            n_init_retries,
            n_init_retry_successes,
            if n_init_retries > 0 {
                n_init_retry_successes as f64 / n_init_retries as f64 * 100.0
            } else {
                0.0
            }
        );
    }

    evals.sort_by(|a, b| {
        comparator.cmp_evals(b, a).then(
            b.fitness()
                .partial_cmp(&a.fitness())
                .unwrap_or(Ordering::Equal),
        )
    });

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
    let gen0_best_obj: f64 = evals
        .iter()
        .find(|e| e.is_valid())
        .map(|e| e.obj)
        .unwrap_or(f64::INFINITY);
    let gen0_mean_obj: f64 = {
        let valid_objs: Vec<f64> = evals
            .iter()
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
        let invalid_count = config
            .population_size
            .saturating_sub(generation0_valid_count);
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
    let _ = writeln!(
        log_sink,
        "Seed          : {}",
        config
            .seed
            .map(|s| s.to_string())
            .unwrap_or("random".to_string())
    );
    let _ = writeln!(log_sink, "Started       : {}", started);
    let _ = writeln!(log_sink, "=========================================");
    let _ = writeln!(log_sink, "");

    let mut trajectory: Vec<crate::moga_impl::GenerationSummary> = Vec::new();
    loop {
        let gen_start = std::time::Instant::now();

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

        // P9-H6-revised: reset per-thread Dijkstra cache at each generation boundary.
        // disabled_arcs is scenario-driven (constant per ts within a generation) so
        // cached results from the previous generation must not be reused.
        dijkstra_cache_reset();

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

        // P10-B: per-generation repair decomposition counters (reset each generation).
        // These are observational only — no repair behavior is changed.
        let mut p10b_infeasible_entering_repair: u32 = 0;
        let mut p10b_feasible_entering_repair: u32 = 0;
        let mut p10b_repair_attempts: u32 = 0;
        let mut p10b_repair_successes: u32 = 0;
        let mut p10b_repair_failures: u32 = 0;
        let mut p10b_repair_ms: f64 = 0.0;
        let mut p10b_improve_ms: f64 = 0.0;

        // P10-C0: per-generation repair-effectiveness counters (reset each generation).
        // These are observational only — no repair behavior is changed.
        let mut p10c0_genome_changed_count: u32 = 0;
        let mut p10c0_genome_unchanged_count: u32 = 0;
        let mut p10c0_violation_count_improved: u32 = 0;
        let mut p10c0_violation_count_unchanged: u32 = 0;
        let mut p10c0_violation_count_worsened: u32 = 0;
        let mut p10c0_sum_max_sat_before: f64 = 0.0;
        let mut p10c0_sum_max_sat_after: f64 = 0.0;

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
            let prev_obj = global_best
                .as_ref()
                .map(|g| -g.fitness())
                .unwrap_or(f64::INFINITY);
            let new_obj = if gen_best.is_valid() {
                -gen_best.fitness()
            } else {
                f64::INFINITY
            };
            let _ = writeln!(
                log_sink,
                "[IMPROVE] Gen {:4}  obj: {:.4} → {:.4}  mlu: {:.4}  valid: {}",
                gen, prev_obj, new_obj, gen_best.mlu, gen_best.valid
            );
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
                    new_obj: if gen_best.is_valid() {
                        -gen_best.fitness()
                    } else {
                        f64::INFINITY
                    },
                    prev_obj: if prev.is_valid() {
                        -prev.fitness()
                    } else {
                        f64::INFINITY
                    },
                    new_mlu: gen_best.mlu,
                    new_sdi,
                };
                telemetry.emit_move(&move_rec);
                // Accumulate histogram (use move_rec.move_class since move_class was moved into the struct)
                match move_rec.move_class.as_str() {
                    "peak" => gen_moves_peak += 1,
                    "shoulder" => gen_moves_shoulder += 1,
                    "transition" => gen_moves_transition += 1,
                    "tail" => gen_moves_tail += 1,
                    "mixed" => gen_moves_mixed += 1,
                    _ => gen_moves_neutral += 1,
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
            let best_obj = global_best
                .as_ref()
                .map(|g| {
                    if g.is_valid() {
                        -g.fitness()
                    } else {
                        f64::INFINITY
                    }
                })
                .unwrap_or(f64::INFINITY);
            let best_mlu = global_best.as_ref().map(|g| g.mlu).unwrap_or(f64::INFINITY);
            let valid_count = evals.iter().filter(|e| e.is_valid()).count();
            let elapsed = t0.elapsed().as_secs_f64();
            let _ = writeln!(
                log_sink,
                "Gen {:4}/{} | best_obj={:.4} mlu={:.4} | valid={}/{} | stagnation={} | {:.1}s",
                gen,
                config.generation_limit,
                best_obj,
                best_mlu,
                valid_count,
                config.population_size,
                stagnation,
                elapsed
            );
        }

        // --- Population health (Level 4) ---
        if gen % config.health_interval == 0 && gen > 0 {
            let unique: std::collections::HashSet<String> = evals
                .iter()
                .map(|e| format!("{:.6}", e.fitness()))
                .collect();
            let avg_waypoints: f64 = evals
                .iter()
                .map(|e| {
                    e.genome()
                        .waypoints
                        .iter()
                        .filter(|w| !w.is_empty())
                        .count() as f64
                })
                .sum::<f64>()
                / evals.len() as f64;
            let _ = writeln!(
                log_sink,
                "  [HEALTH] unique_fitness={}/{}  avg_nonempty_waypoints={:.2}",
                unique.len(),
                evals.len(),
                avg_waypoints
            );
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
        let valid_elites: Vec<&RoadefEvaluation> = evals
            .iter()
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
                (
                    e.genome().clone(),
                    "elite",
                    candidate_counter,
                    0u64,
                    0u64,
                    true,
                )
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
                    if idx == best_idx {
                        continue;
                    }
                    candidate_counter += 1;
                    let loser_cid = candidate_counter;
                    let ev = &evals[idx];
                    let (deltas, move_class) = if let Some(ref best) = global_best {
                        let d = ZoneDeltas::compute(&best.load_vector, &ev.load_vector);
                        let mc = d.classify(1e-9).to_string();
                        (d, mc)
                    } else {
                        (
                            ZoneDeltas {
                                delta_rank1: 0.0,
                                delta_2_20: 0.0,
                                delta_21_100: 0.0,
                                delta_tail: 0.0,
                            },
                            "neutral".to_string(),
                        )
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
                        obj: if ev.is_valid() {
                            -ev.fitness()
                        } else {
                            f64::INFINITY
                        },
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

                // Step 2: repair — evaluate after all destructive operators using pipeline.
                let ca_was_feasible = pipeline.constraint_model.is_feasible(&ca);
                // P10-C0: snapshot pre-repair state for infeasible offspring (observational only).
                let (ca_waypoints_before, ca_violations_before) = if !ca_was_feasible {
                    let wp = ca.waypoints.clone();
                    let vv = pipeline.constraint_model.evaluate_violations(&ca);
                    (Some(wp), Some(vv))
                } else {
                    (None, None)
                };
                // P10-B: count and time process_offspring separately for infeasible vs feasible paths.
                let t_proc_ca = Instant::now();
                match pipeline.process_offspring(&mut ca) {
                    Ok(true) => {
                        if !ca_was_feasible {
                            ca_tag = "repair_succeeded";
                            p10b_infeasible_entering_repair += 1;
                            p10b_repair_attempts += 1;
                            p10b_repair_successes += 1;
                            p10b_repair_ms += t_proc_ca.elapsed().as_secs_f64() * 1000.0;
                        } else {
                            ca_tag = "improvement_applied";
                            p10b_feasible_entering_repair += 1;
                            p10b_improve_ms += t_proc_ca.elapsed().as_secs_f64() * 1000.0;
                        }
                    }
                    Ok(false) => {
                        // P10-C0: measure repair effectiveness BEFORE resetting ca to pa.
                        // This is the only window where the post-repair genome is accessible.
                        if let (Some(wp_before), Some(vv_before)) =
                            (ca_waypoints_before, ca_violations_before)
                        {
                            let vv_after =
                                pipeline.constraint_model.evaluate_violations(&ca);
                            // Genome changed? Compare waypoint fingerprints.
                            if ca.waypoints != wp_before {
                                p10c0_genome_changed_count += 1;
                            } else {
                                p10c0_genome_unchanged_count += 1;
                            }
                            // Violation count delta.
                            let v_before = vv_before.len();
                            let v_after = vv_after.len();
                            if v_after < v_before {
                                p10c0_violation_count_improved += 1;
                            } else if v_after == v_before {
                                p10c0_violation_count_unchanged += 1;
                            } else {
                                p10c0_violation_count_worsened += 1;
                            }
                            // Max capacity saturation before/after (for Capacity violations).
                            let max_sat_before = vv_before
                                .iter()
                                .filter_map(|v| {
                                    if let crate::constraints::RoadefViolation::Capacity {
                                        sat, ..
                                    } = v
                                    {
                                        Some(*sat)
                                    } else {
                                        None
                                    }
                                })
                                .fold(f64::NEG_INFINITY, f64::max);
                            let max_sat_after = vv_after
                                .iter()
                                .filter_map(|v| {
                                    if let crate::constraints::RoadefViolation::Capacity {
                                        sat, ..
                                    } = v
                                    {
                                        Some(*sat)
                                    } else {
                                        None
                                    }
                                })
                                .fold(f64::NEG_INFINITY, f64::max);
                            if max_sat_before.is_finite() {
                                p10c0_sum_max_sat_before += max_sat_before;
                            }
                            if max_sat_after.is_finite() {
                                p10c0_sum_max_sat_after += max_sat_after;
                            }
                        }
                        ca = pa.clone();
                        ca_tag = "repair_failed";
                        p10b_infeasible_entering_repair += 1;
                        p10b_repair_attempts += 1;
                        p10b_repair_failures += 1;
                        p10b_repair_ms += t_proc_ca.elapsed().as_secs_f64() * 1000.0;
                    }
                    Err(e) => {
                        let _ = writeln!(log_sink, "[pipeline] operator error on ca: {}", e);
                        ca = pa.clone();
                        ca_tag = "operator_error";
                        if !ca_was_feasible {
                            p10b_infeasible_entering_repair += 1;
                            p10b_repair_attempts += 1;
                            p10b_repair_failures += 1;
                            p10b_repair_ms += t_proc_ca.elapsed().as_secs_f64() * 1000.0;
                        } else {
                            p10b_feasible_entering_repair += 1;
                            p10b_improve_ms += t_proc_ca.elapsed().as_secs_f64() * 1000.0;
                        }
                    }
                }

                let cb_was_feasible = pipeline.constraint_model.is_feasible(&cb);
                // P10-C0: snapshot pre-repair state for infeasible offspring (observational only).
                let (cb_waypoints_before, cb_violations_before) = if !cb_was_feasible {
                    let wp = cb.waypoints.clone();
                    let vv = pipeline.constraint_model.evaluate_violations(&cb);
                    (Some(wp), Some(vv))
                } else {
                    (None, None)
                };
                // P10-B: count and time process_offspring separately for infeasible vs feasible paths.
                let t_proc_cb = Instant::now();
                match pipeline.process_offspring(&mut cb) {
                    Ok(true) => {
                        if !cb_was_feasible {
                            cb_tag = "repair_succeeded";
                            p10b_infeasible_entering_repair += 1;
                            p10b_repair_attempts += 1;
                            p10b_repair_successes += 1;
                            p10b_repair_ms += t_proc_cb.elapsed().as_secs_f64() * 1000.0;
                        } else {
                            cb_tag = "improvement_applied";
                            p10b_feasible_entering_repair += 1;
                            p10b_improve_ms += t_proc_cb.elapsed().as_secs_f64() * 1000.0;
                        }
                    }
                    Ok(false) => {
                        // P10-C0: measure repair effectiveness BEFORE resetting cb to pb.
                        if let (Some(wp_before), Some(vv_before)) =
                            (cb_waypoints_before, cb_violations_before)
                        {
                            let vv_after =
                                pipeline.constraint_model.evaluate_violations(&cb);
                            if cb.waypoints != wp_before {
                                p10c0_genome_changed_count += 1;
                            } else {
                                p10c0_genome_unchanged_count += 1;
                            }
                            let v_before = vv_before.len();
                            let v_after = vv_after.len();
                            if v_after < v_before {
                                p10c0_violation_count_improved += 1;
                            } else if v_after == v_before {
                                p10c0_violation_count_unchanged += 1;
                            } else {
                                p10c0_violation_count_worsened += 1;
                            }
                            let max_sat_before = vv_before
                                .iter()
                                .filter_map(|v| {
                                    if let crate::constraints::RoadefViolation::Capacity {
                                        sat, ..
                                    } = v
                                    {
                                        Some(*sat)
                                    } else {
                                        None
                                    }
                                })
                                .fold(f64::NEG_INFINITY, f64::max);
                            let max_sat_after = vv_after
                                .iter()
                                .filter_map(|v| {
                                    if let crate::constraints::RoadefViolation::Capacity {
                                        sat, ..
                                    } = v
                                    {
                                        Some(*sat)
                                    } else {
                                        None
                                    }
                                })
                                .fold(f64::NEG_INFINITY, f64::max);
                            if max_sat_before.is_finite() {
                                p10c0_sum_max_sat_before += max_sat_before;
                            }
                            if max_sat_after.is_finite() {
                                p10c0_sum_max_sat_after += max_sat_after;
                            }
                        }
                        cb = pb.clone();
                        cb_tag = "repair_failed";
                        p10b_infeasible_entering_repair += 1;
                        p10b_repair_attempts += 1;
                        p10b_repair_failures += 1;
                        p10b_repair_ms += t_proc_cb.elapsed().as_secs_f64() * 1000.0;
                    }
                    Err(e) => {
                        let _ = writeln!(log_sink, "[pipeline] operator error on cb: {}", e);
                        cb = pb.clone();
                        cb_tag = "operator_error";
                        if !cb_was_feasible {
                            p10b_infeasible_entering_repair += 1;
                            p10b_repair_attempts += 1;
                            p10b_repair_failures += 1;
                            p10b_repair_ms += t_proc_cb.elapsed().as_secs_f64() * 1000.0;
                        } else {
                            p10b_feasible_entering_repair += 1;
                            p10b_improve_ms += t_proc_cb.elapsed().as_secs_f64() * 1000.0;
                        }
                    }
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

                // pipeline repair/improvement
                let child_was_feasible = pipeline.constraint_model.is_feasible(&child);
                // P10-C0: snapshot pre-repair state for infeasible offspring (observational only).
                let (child_waypoints_before, child_violations_before) = if !child_was_feasible {
                    let wp = child.waypoints.clone();
                    let vv = pipeline.constraint_model.evaluate_violations(&child);
                    (Some(wp), Some(vv))
                } else {
                    (None, None)
                };
                let mut child_tag = "mutation";
                // P10-B: count and time process_offspring separately for infeasible vs feasible paths.
                let t_proc_child = Instant::now();
                match pipeline.process_offspring(&mut child) {
                    Ok(true) => {
                        if !child_was_feasible {
                            child_tag = "repair_succeeded";
                            p10b_infeasible_entering_repair += 1;
                            p10b_repair_attempts += 1;
                            p10b_repair_successes += 1;
                            p10b_repair_ms += t_proc_child.elapsed().as_secs_f64() * 1000.0;
                        } else {
                            child_tag = "improvement_applied";
                            p10b_feasible_entering_repair += 1;
                            p10b_improve_ms += t_proc_child.elapsed().as_secs_f64() * 1000.0;
                        }
                    }
                    Ok(false) => {
                        // P10-C0: measure repair effectiveness BEFORE resetting child to parent.
                        if let (Some(wp_before), Some(vv_before)) =
                            (child_waypoints_before, child_violations_before)
                        {
                            let vv_after =
                                pipeline.constraint_model.evaluate_violations(&child);
                            if child.waypoints != wp_before {
                                p10c0_genome_changed_count += 1;
                            } else {
                                p10c0_genome_unchanged_count += 1;
                            }
                            let v_before = vv_before.len();
                            let v_after = vv_after.len();
                            if v_after < v_before {
                                p10c0_violation_count_improved += 1;
                            } else if v_after == v_before {
                                p10c0_violation_count_unchanged += 1;
                            } else {
                                p10c0_violation_count_worsened += 1;
                            }
                            let max_sat_before = vv_before
                                .iter()
                                .filter_map(|v| {
                                    if let crate::constraints::RoadefViolation::Capacity {
                                        sat, ..
                                    } = v
                                    {
                                        Some(*sat)
                                    } else {
                                        None
                                    }
                                })
                                .fold(f64::NEG_INFINITY, f64::max);
                            let max_sat_after = vv_after
                                .iter()
                                .filter_map(|v| {
                                    if let crate::constraints::RoadefViolation::Capacity {
                                        sat, ..
                                    } = v
                                    {
                                        Some(*sat)
                                    } else {
                                        None
                                    }
                                })
                                .fold(f64::NEG_INFINITY, f64::max);
                            if max_sat_before.is_finite() {
                                p10c0_sum_max_sat_before += max_sat_before;
                            }
                            if max_sat_after.is_finite() {
                                p10c0_sum_max_sat_after += max_sat_after;
                            }
                        }
                        child = evals[pa_idx].genome().clone();
                        child_tag = "repair_failed";
                        p10b_infeasible_entering_repair += 1;
                        p10b_repair_attempts += 1;
                        p10b_repair_failures += 1;
                        p10b_repair_ms += t_proc_child.elapsed().as_secs_f64() * 1000.0;
                    }
                    Err(e) => {
                        let _ = writeln!(log_sink, "[pipeline] operator error on mut: {}", e);
                        child = evals[pa_idx].genome().clone();
                        child_tag = "operator_error";
                        if !child_was_feasible {
                            p10b_infeasible_entering_repair += 1;
                            p10b_repair_attempts += 1;
                            p10b_repair_failures += 1;
                            p10b_repair_ms += t_proc_child.elapsed().as_secs_f64() * 1000.0;
                        } else {
                            p10b_feasible_entering_repair += 1;
                            p10b_improve_ms += t_proc_child.elapsed().as_secs_f64() * 1000.0;
                        }
                    }
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

        let t_eval_start = std::time::Instant::now();
        let t_eval_start = std::time::Instant::now();
        let mut new_evals_with_meta: Vec<(RoadefEvaluation, u64, u64, u64, u64)> = next_pop
            .into_iter()
            .map(|(g, tag, p1, p2, tid, _won)| {
                let mut ev = fitness_eval.evaluate(
                    &g,
                    &coralys_moga::runtime::optimization::metric::MetricReport::default(),
                );
                ev.operator = tag;
                candidate_counter += 1;
                let cid = candidate_counter;

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
                        let diag_reason = fitness_eval
                            .evaluator
                            .diagnose_failure(&solution)
                            .unwrap_or_else(|| format!("arc overloaded max_sat={:.9}", sat));
                        eprintln!(
                            "[diag] origin={} overload={} max_sat={:.9} | {}",
                            tag, overload_class, sat, diag_reason
                        );

                        let origin_idx: usize = if tag.starts_with("crossover") {
                            1
                        } else if tag == "mutation" {
                            2
                        } else if tag == "elite" {
                            3
                        } else {
                            0
                        };
                        rc002_inv[origin_idx * 4 + class_idx] += 1;
                    }
                }

                (ev, cid, p1, p2, tid)
            })
            .collect();
        let t_eval_ms = t_eval_start.elapsed().as_secs_f64() * 1000.0;

        // Sort descending by comparator order (best first).
        // RP-408A: use pluggable comparator instead of raw fitness().
        new_evals_with_meta.sort_by(|(a, ..), (b, ..)| {
            comparator.cmp_evals(b, a).then(
                b.fitness()
                    .partial_cmp(&a.fitness())
                    .unwrap_or(Ordering::Equal),
            )
        });

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
                (
                    ZoneDeltas {
                        delta_rank1: 0.0,
                        delta_2_20: 0.0,
                        delta_21_100: 0.0,
                        delta_tail: 0.0,
                    },
                    "neutral".to_string(),
                )
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
            let (decision_stage, reason): (&'static str, Option<&'static str>) = if !ev.is_valid() {
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
                parent1: *p1, // RP-409C: correctly propagated from next_pop build phase
                parent2: *p2, // RP-409C: correctly propagated from next_pop build phase
                operator: ev.operator,
                tournament_id: *tid as u32, // RP-409C: correctly propagated from next_pop build phase
                deltas,
                move_class,
                obj: if ev.is_valid() {
                    -ev.fitness()
                } else {
                    f64::INFINITY
                },
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
        let new_evals: Vec<RoadefEvaluation> =
            new_evals_with_meta.into_iter().map(|(ev, ..)| ev).collect();

        // --- RP-410 / RP-411 Phase 2: emit GenerationRecord ---
        // Emitted here (after selection, crossover, mutation, eval, and sort) so that
        // all per-phase timing accumulators are populated with the current generation's
        // measured values. valid_count and unique_fitness_count reflect the new population.
        {
            let best_sdi = global_best
                .as_ref()
                .map(|g| compute_sdi(&g.load_vector))
                .unwrap_or(0.0);
            let top20_prefix: Vec<f64> = global_best
                .as_ref()
                .map(|g| g.load_vector.iter().take(20).cloned().collect())
                .unwrap_or_default();
            let unique_fitness_count = {
                let unique: std::collections::HashSet<String> = new_evals
                    .iter()
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
                best_obj: global_best
                    .as_ref()
                    .map(|g| {
                        if g.is_valid() {
                            -g.fitness()
                        } else {
                            f64::INFINITY
                        }
                    })
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
                // P10-B: repair_time_ms is now measured directly from process_offspring calls
                // for infeasible individuals. Previously 0.0 (placeholder).
                repair_time_ms: p10b_repair_ms,
                selection_time_ms: t_selection_ms,
                telemetry_time_ms: 0.0, // approximation: emit cost not yet measured
                other_time_ms: other_ms,
                total_gen_time_ms: total_so_far_ms,
                // P10-B repair decomposition counters.
                p10b_infeasible_entering_repair,
                p10b_feasible_entering_repair,
                p10b_repair_attempts,
                p10b_repair_successes,
                p10b_repair_failures,
                p10b_repair_ms,
                p10b_improve_ms,
                p10b_repair_ms_per_infeasible: if p10b_infeasible_entering_repair > 0 {
                    p10b_repair_ms / p10b_infeasible_entering_repair as f64
                } else {
                    f64::NAN
                },
                p10c0_genome_changed_count,
                p10c0_genome_unchanged_count,
                p10c0_violation_count_improved,
                p10c0_violation_count_unchanged,
                p10c0_violation_count_worsened,
                p10c0_sum_max_sat_before,
                p10c0_sum_max_sat_after,
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
    let best_obj = best
        .map(|g| {
            if g.is_valid() {
                -g.fitness()
            } else {
                f64::INFINITY
            }
        })
        .unwrap_or(f64::INFINITY);
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
    let _ = writeln!(
        log_sink,
        "Finished      : {}",
        chrono::Utc::now().to_rfc3339()
    );
    let _ = writeln!(log_sink, "Runtime       : {}ms", runtime_ms);
    let _ = writeln!(log_sink, "Termination   : {}", termination_reason);
    let _ = writeln!(log_sink, "Best Objective: {:.4}", best_obj);
    let _ = writeln!(log_sink, "Best MLU      : {:.4}", best_mlu);
    let _ = writeln!(log_sink, "Valid         : {}", valid);
    let _ = writeln!(log_sink, "=========================================");

    telemetry.flush();

    EvolutionRunResult {
        trajectory: vec![],
        best_genome: best
            .map(|g| g.genome().clone())
            .unwrap_or_else(|| factory.create(&mut rng)),
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
    use crate::moga_impl::{EvalComparator, LexicographicComparator, ScalarComparator};
    use std::cmp::Ordering;

    // Helper: construct a minimal RoadefEvaluation with a given load vector and validity.
    // The genome fields are irrelevant for comparator tests.
    fn make_eval(load_vector: Vec<f64>, valid: bool) -> RoadefEvaluation {
        let obj = if valid {
            load_vector.first().copied().unwrap_or(0.0)
        } else {
            f64::INFINITY
        };
        RoadefEvaluation {
            genome: RoadefGenome {
                waypoints: vec![],
                num_time_slots: 0,
            },
            obj,
            valid,
            mlu: load_vector.first().copied().unwrap_or(0.0),
            load_vector,
            operator: "test",
            max_sat: 0.0,
        }
    }

    // -----------------------------------------------------------------------
    // ScalarComparator tests
    // -----------------------------------------------------------------------

    #[test]
    fn scalar_valid_beats_invalid() {
        let cmp = ScalarComparator;
        let valid = make_eval(vec![0.9], true);
        let invalid = make_eval(vec![0.5], false);
        assert!(
            cmp.is_better(&valid, &invalid),
            "valid should beat invalid under ScalarComparator"
        );
        assert!(
            !cmp.is_better(&invalid, &valid),
            "invalid should not beat valid under ScalarComparator"
        );
    }

    #[test]
    fn scalar_lower_obj_wins() {
        let cmp = ScalarComparator;
        // fitness = -obj, so lower obj = higher fitness = better
        let better = make_eval(vec![0.8], true); // obj=0.8, fitness=-0.8
        let worse = make_eval(vec![0.9], true); // obj=0.9, fitness=-0.9
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
        let valid = make_eval(vec![0.9, 0.8], true);
        let invalid = make_eval(vec![0.1, 0.1], false);
        assert!(
            cmp.is_better(&valid, &invalid),
            "valid should beat invalid under LexicographicComparator"
        );
        assert!(!cmp.is_better(&invalid, &valid));
    }

    #[test]
    fn lex_both_invalid_is_equal() {
        let cmp = LexicographicComparator;
        let a = make_eval(vec![0.9], false);
        let b = make_eval(vec![0.1], false);
        assert_eq!(
            cmp.cmp_evals(&a, &b),
            Ordering::Equal,
            "two invalid solutions should compare Equal"
        );
    }

    #[test]
    fn lex_first_rank_decides() {
        // [100, 80, 70] vs [101, 10, 10]: first has lower rank-1 → first wins
        let cmp = LexicographicComparator;
        let better = make_eval(vec![100.0, 80.0, 70.0], true);
        let worse = make_eval(vec![101.0, 10.0, 10.0], true);
        assert!(
            cmp.is_better(&better, &worse),
            "[100,80,70] should beat [101,10,10] (lower rank-1 load)"
        );
        assert!(!cmp.is_better(&worse, &better));
    }

    #[test]
    fn lex_tie_at_rank1_second_rank_decides() {
        // [100, 81, 60] vs [100, 82, 10]: tie at rank-1, first wins at rank-2
        let cmp = LexicographicComparator;
        let better = make_eval(vec![100.0, 81.0, 60.0], true);
        let worse = make_eval(vec![100.0, 82.0, 10.0], true);
        assert!(
            cmp.is_better(&better, &worse),
            "[100,81,60] should beat [100,82,10] (lower rank-2 load)"
        );
    }

    #[test]
    fn lex_equal_vectors_is_equal() {
        let cmp = LexicographicComparator;
        let a = make_eval(vec![0.9, 0.8, 0.7], true);
        let b = make_eval(vec![0.9, 0.8, 0.7], true);
        assert_eq!(
            cmp.cmp_evals(&a, &b),
            Ordering::Equal,
            "identical load vectors should compare Equal"
        );
    }

    #[test]
    fn lex_shorter_vector_treated_as_zero_padded() {
        // [0.9] vs [0.9, 0.5]: tie at rank-1, second has 0.5 at rank-2 vs 0.0 → first wins
        let cmp = LexicographicComparator;
        let shorter = make_eval(vec![0.9], true);
        let longer = make_eval(vec![0.9, 0.5], true);
        // shorter has 0.0 at rank-2 (missing), longer has 0.5 → shorter is better
        assert!(
            cmp.is_better(&shorter, &longer),
            "shorter vector (zero-padded) should beat longer with non-zero tail"
        );
    }

    // -----------------------------------------------------------------------
    // Scalar equivalence regression: ScalarComparator must agree with the
    // pre-RP-408A fitness()-based comparison on all orderings.
    // -----------------------------------------------------------------------

    #[test]
    fn scalar_agrees_with_fitness_comparison_valid_vs_valid() {
        let cmp = ScalarComparator;
        let cases: Vec<(f64, f64)> = vec![
            (0.5, 0.9), // a better
            (0.9, 0.5), // b better
            (0.7, 0.7), // equal
            (0.0, 1.0), // a much better
            (1.0, 0.0), // b much better
        ];
        for (obj_a, obj_b) in cases {
            let a = make_eval(vec![obj_a], true);
            let b = make_eval(vec![obj_b], true);
            // Pre-RP-408A: a better iff a.fitness() > b.fitness() iff -obj_a > -obj_b iff obj_a < obj_b
            let legacy_a_better = a.fitness() > b.fitness();
            let cmp_a_better = cmp.is_better(&a, &b);
            assert_eq!(
                cmp_a_better, legacy_a_better,
                "ScalarComparator disagrees with legacy fitness() for obj_a={obj_a} obj_b={obj_b}"
            );
        }
    }

    #[test]
    fn scalar_agrees_with_fitness_comparison_valid_vs_invalid() {
        let cmp = ScalarComparator;
        let valid = make_eval(vec![0.9], true);
        let invalid = make_eval(vec![0.1], false);
        // Legacy: valid.fitness() = -0.9, invalid.fitness() = -1_000_000.0
        // valid is better
        assert_eq!(
            cmp.is_better(&valid, &invalid),
            valid.fitness() > invalid.fitness()
        );
        assert_eq!(
            cmp.is_better(&invalid, &valid),
            invalid.fitness() > valid.fitness()
        );
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
        evals.sort_by(|a, b| {
            cmp.cmp_evals(b, a).then(
                b.fitness()
                    .partial_cmp(&a.fitness())
                    .unwrap_or(Ordering::Equal),
            )
        });
        let cmp_order: Vec<f64> = evals.iter().map(|e| e.obj).collect();

        // Sort using legacy fitness() comparison
        let mut legacy = vec![
            make_eval(vec![0.9], true),
            make_eval(vec![0.5], true),
            make_eval(vec![0.7], true),
            make_eval(vec![0.3], false),
            make_eval(vec![0.1], true),
        ];
        legacy.sort_by(|a, b| {
            b.fitness()
                .partial_cmp(&a.fitness())
                .unwrap_or(Ordering::Equal)
        });
        let legacy_order: Vec<f64> = legacy.iter().map(|e| e.obj).collect();

        assert_eq!(
            cmp_order, legacy_order,
            "ScalarComparator sort order must match legacy fitness() sort order"
        );
    }
}

pub fn run_pipeline_evolution_v2<M>(
    factory: &RoadefGenomeFactory,
    fitness_eval: &RoadefFitnessEvaluator,
    mutator: &M,
    crossover: &RoadefCrossover,
    pipeline_obj: &coralys_core::pipeline::EvolutionaryPipeline<
        RoadefGenome,
        RoadefConstraintModel,
        crate::operators::OperatorError,
    >,
    config: &EvolutionRunConfig,
    initial_population: crate::moga_impl::InitialPopulation<RoadefGenome>,
    instance_name: &str,
    log_sink: &mut dyn Write,
    telemetry: &mut dyn TelemetrySink,
    // Phase 3: when true, cache-miss evaluations are dispatched in parallel via Rayon.
    // When false, the L1 sequential baseline is used (identical search trajectory).
    use_rayon: bool,
) -> EvolutionRunResult
where
    M: MutationOperator<RoadefGenome>,
{
    let mut rng: StdRng = match config.seed {
        Some(s) => StdRng::seed_from_u64(s),
        None => StdRng::from_entropy(),
    };

    let run_uuid = {
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

    let mut evaluation_cache: std::collections::HashMap<RoadefGenome, RoadefEvaluation> =
        std::collections::HashMap::new();
    let t0 = Instant::now();

    // Use deterministic provided population
    let mut evals: Vec<RoadefEvaluation> = Vec::with_capacity(config.population_size);
    for g in initial_population.genomes.iter() {
        let ev = if let Some(cached) = evaluation_cache.get(g) {
            cached.clone()
        } else {
            let e = fitness_eval.evaluate(
                g,
                &coralys_moga::runtime::optimization::metric::MetricReport::default(),
            );
            evaluation_cache.insert(g.clone(), e.clone());
            e
        };
        let mut ev = ev;
        ev.operator = "initial";
        evals.push(ev);
    }

    evals.sort_by(|a, b| {
        comparator.cmp_evals(b, a).then(
            b.fitness()
                .partial_cmp(&a.fitness())
                .unwrap_or(Ordering::Equal),
        )
    });

    let generation0_valid_count: usize = evals.iter().filter(|e| e.is_valid()).count();
    let initial_feasibility_rate: f64 = if config.population_size > 0 {
        generation0_valid_count as f64 / config.population_size as f64
    } else {
        0.0
    };

    let gen0_feasible_count = generation0_valid_count;
    let gen0_best_obj: f64 = evals
        .iter()
        .find(|e| e.is_valid())
        .map(|e| e.obj)
        .unwrap_or(f64::INFINITY);
    let gen0_mean_obj: f64 = {
        let valid_objs: Vec<f64> = evals
            .iter()
            .filter(|e| e.is_valid())
            .map(|e| e.obj)
            .collect();
        if valid_objs.is_empty() {
            f64::INFINITY
        } else {
            valid_objs.iter().sum::<f64>() / valid_objs.len() as f64
        }
    };
    let gen0_unique_obj_count: usize = {
        let mut unq: std::collections::HashSet<String> = std::collections::HashSet::new();
        for e in evals.iter().filter(|e| e.is_valid()) {
            unq.insert(format!("{:.6}", e.obj));
        }
        unq.len()
    };
    let gen0_duplicate_genome_count: usize = {
        let mut seen = std::collections::HashSet::new();
        let mut dups = 0;
        for e in &evals {
            let wp_hash = {
                use std::hash::{Hash, Hasher};
                let mut h = std::collections::hash_map::DefaultHasher::new();
                for w in &e.genome.waypoints {
                    w.hash(&mut h);
                }
                h.finish()
            };
            if !seen.insert(wp_hash) {
                dups += 1;
            }
        }
        dups
    };

    let mut global_best: Option<RoadefEvaluation> = None;
    let mut best_found_at_gen = 0;
    if !evals.is_empty() && evals[0].is_valid() {
        global_best = Some(evals[0].clone());
    }

    let mut t_health = Instant::now();
    let mut no_improvement_count = 0;
    let mut generations_run = 0;
    let mut termination_reason = "GenerationLimit".to_string();
    let mut rng = StdRng::seed_from_u64(config.seed.unwrap_or(0));

    let mut population = evals;
    let mut trajectory = Vec::new();

    while generations_run < config.generation_limit {
        let gen_start = Instant::now();
        let mut t_eval_ms = 0.0;
        let mut cache_hits = 0;
        let mut actual_evals = 0;
        let mut t_cache_lookup_ms = 0.0;
        let mut t_cache_hit_materialize_ms = 0.0;
        let mut t_cache_insert_ms = 0.0;
        // Phase 8: per-operator timing accumulators.
        let mut t_crossover_ms: f64 = 0.0;
        let mut t_mutation_ms: f64 = 0.0;
        let mut t_repair_ms: f64 = 0.0;
        let mut t_improve_ms: f64 = 0.0;
        let mut t_sort_ms: f64 = 0.0;
        let mut t_selection_ms: f64 = 0.0;
        let mut t_feasibility_ms: f64 = 0.0;
        let mut t_staging_ms: f64 = 0.0;

        generations_run += 1;
        if let Some(b) = config.max_runtime {
            if t0.elapsed() >= b {
                termination_reason = format!("TimeLimit({:.1}s)", b.as_secs_f64());
                break;
            }
        }
        if no_improvement_count >= config.no_improvement_limit {
            termination_reason = format!("NoImprovement({})", config.no_improvement_limit);
            break;
        }

        let old_best_obj = global_best.as_ref().map(|b| b.obj).unwrap_or(f64::INFINITY);

        let mut next_generation = Vec::with_capacity(config.population_size);
        for i in 0..config.elite_count.min(population.len()) {
            let mut elite = population[i].clone();
            elite.operator = "elite";
            next_generation.push(elite);
        }

        let mut n_crossover = 0;
        let mut n_mutation = 0;
        let mut n_cross_repair = 0;
        let mut n_mut_repair = 0;
        let mut n_cross_maj = 0;
        let mut n_mut_maj = 0;

        // -----------------------------------------------------------------------
        // Phase 3 (Rayon): parallel evaluation of the offspring batch.
        //
        // Governance invariant: the search trajectory must be identical to the L1
        // baseline. This is guaranteed because:
        //   (a) All RNG-driven operations (selection, crossover, mutation, repair)
        //       remain sequential in Phase A below — the RNG sequence is unchanged.
        //   (b) The evaluator (fitness_eval.evaluate) is a pure function of the
        //       genome: same genome → same result on every call, every thread.
        //   (c) The L1 cache is consulted sequentially (Phase A) before the
        //       parallel phase, so cache-hit counts are identical to the baseline.
        //
        // Implementation:
        //   Phase A (sequential): produce all offspring genomes via the existing
        //     RNG-driven loop. For each offspring, check the L1 cache. If a hit,
        //     record the cached result immediately. If a miss, record the genome
        //     for parallel evaluation. Collect into a staging Vec.
        //   Phase B (parallel): evaluate all cache-miss genomes in parallel using
        //     rayon::par_iter(). fitness_eval is Arc-wrapped (Send+Sync).
        //   Phase C (sequential): merge parallel results back into the staging Vec,
        //     insert new results into the L1 cache, accumulate timing stats.
        // -----------------------------------------------------------------------

        // Staging entry: one per offspring slot.
        // Variants:
        //   CacheHit(ev)          — L1 hit; result already known.
        //   FallbackClone(ev)     — repair failed; clone of parent used.
        //   NeedsEval(genome, tag, slot_in_miss_vec) — cache miss; genome queued for parallel eval.
        enum OffspringStage {
            CacheHit(RoadefEvaluation),
            FallbackClone(RoadefEvaluation),
            NeedsEval(RoadefGenome, &'static str),
        }

        let needed = config.population_size.saturating_sub(next_generation.len());

        // Phase A: sequential RNG loop — produce genomes, check L1 cache.
        let mut staging: Vec<OffspringStage> = Vec::with_capacity(needed);
        // Track operator counts and cache stats (same as before).
        for _ in 0..needed {
            if rng.gen_bool(config.crossover_rate) && population.len() >= 2 {
                // Phase 8: time parent selection (index sampling only — no eval).
                let t_sel_start = Instant::now();
                let p1_idx = rng.gen_range(0..population.len());
                let p2_idx = rng.gen_range(0..population.len());
                t_selection_ms += t_sel_start.elapsed().as_secs_f64() * 1000.0;

                let p1 = &population[p1_idx].genome;
                let p2 = &population[p2_idx].genome;

                // Phase 8: time crossover operator call only.
                let t_xover_start = Instant::now();
                let (mut child, _child2) = crossover.crossover(p1, p2, &mut rng);
                t_crossover_ms += t_xover_start.elapsed().as_secs_f64() * 1000.0;

                if rng.gen_bool(config.mutation_rate) {
                    // Phase 8: time mutation operator call only.
                    let t_mut_start = Instant::now();
                    mutator.mutate(&mut child, &mut rng);
                    t_mutation_ms += t_mut_start.elapsed().as_secs_f64() * 1000.0;
                }

                // P9-H3: removed standalone is_feasible() pre-check.
                // process_offspring calls is_feasible() internally as its first action
                // (coralys-core/src/pipeline.rs line 25), so the pre-check was a
                // redundant evaluate_violations() call per offspring.
                // Tag and timing attribution now derived from process_offspring result only.
                let mut tag = "crossover_mutation";
                // Phase 8: time process_offspring (repair + improve) call only.
                let t_proc_start = Instant::now();
                let proc_result = pipeline_obj.process_offspring(&mut child);
                let t_proc_elapsed = t_proc_start.elapsed().as_secs_f64() * 1000.0;
                let success = match proc_result {
                    Ok(true) => {
                        t_improve_ms += t_proc_elapsed;
                        tag = "pipeline_improved";
                        true
                    }
                    Ok(false) => {
                        t_repair_ms += t_proc_elapsed;
                        tag = "pipeline_repair_failed";
                        false
                    }
                    Err(_) => {
                        t_repair_ms += t_proc_elapsed;
                        tag = "pipeline_operator_error";
                        false
                    }
                };

                n_crossover += 1;
                if success {
                    let lookup_start = Instant::now();
                    let cached_opt = evaluation_cache.get(&child);
                    t_cache_lookup_ms += lookup_start.elapsed().as_secs_f64() * 1000.0;

                    if let Some(cached) = cached_opt {
                        cache_hits += 1;
                        let mat_start = Instant::now();
                        let mut cloned = cached.clone();
                        t_cache_hit_materialize_ms += mat_start.elapsed().as_secs_f64() * 1000.0;
                        cloned.operator = tag;
                        staging.push(OffspringStage::CacheHit(cloned));
                    } else {
                        staging.push(OffspringStage::NeedsEval(child, tag));
                    }
                } else {
                    let mut fallback = population[rng.gen_range(0..population.len())].clone();
                    fallback.operator = tag;
                    staging.push(OffspringStage::FallbackClone(fallback));
                }
            } else {
                // Phase 8: time parent selection.
                let t_sel_start = Instant::now();
                let p1_idx = rng.gen_range(0..population.len());
                t_selection_ms += t_sel_start.elapsed().as_secs_f64() * 1000.0;

                let p = &population[p1_idx].genome;
                let mut child = p.clone();

                // Phase 8: time mutation operator call only.
                let t_mut_start = Instant::now();
                mutator.mutate(&mut child, &mut rng);
                t_mutation_ms += t_mut_start.elapsed().as_secs_f64() * 1000.0;

                // P9-H3: removed standalone is_feasible() pre-check (same as crossover path).
                let mut tag = "mutation";
                // Phase 8: time process_offspring (repair + improve) call only.
                let t_proc_start = Instant::now();
                let proc_result = pipeline_obj.process_offspring(&mut child);
                let t_proc_elapsed = t_proc_start.elapsed().as_secs_f64() * 1000.0;
                let success = match proc_result {
                    Ok(true) => {
                        t_improve_ms += t_proc_elapsed;
                        tag = "pipeline_improved";
                        true
                    }
                    Ok(false) => {
                        t_repair_ms += t_proc_elapsed;
                        tag = "pipeline_repair_failed";
                        false
                    }
                    Err(_) => {
                        t_repair_ms += t_proc_elapsed;
                        tag = "pipeline_operator_error";
                        false
                    }
                };

                n_mutation += 1;
                if success {
                    let lookup_start = Instant::now();
                    let cached_opt = evaluation_cache.get(&child);
                    t_cache_lookup_ms += lookup_start.elapsed().as_secs_f64() * 1000.0;

                    if let Some(cached) = cached_opt {
                        cache_hits += 1;
                        let mat_start = Instant::now();
                        let mut cloned = cached.clone();
                        t_cache_hit_materialize_ms += mat_start.elapsed().as_secs_f64() * 1000.0;
                        cloned.operator = tag;
                        staging.push(OffspringStage::CacheHit(cloned));
                    } else {
                        staging.push(OffspringStage::NeedsEval(child, tag));
                    }
                } else {
                    let mut fallback = population[p1_idx].clone();
                    fallback.operator = tag;
                    staging.push(OffspringStage::FallbackClone(fallback));
                }
            }
        }

        // Phase B: parallel evaluation of all cache-miss genomes.
        // Collect (index_in_staging, genome, tag) for every NeedsEval entry.
        // Phase 8: time staging collection overhead.
        let t_staging_start = Instant::now();
        let miss_indices: Vec<usize> = staging
            .iter()
            .enumerate()
            .filter_map(|(i, s)| {
                if matches!(s, OffspringStage::NeedsEval(..)) {
                    Some(i)
                } else {
                    None
                }
            })
            .collect();

        // Extract genomes for parallel evaluation (avoids borrow of staging).
        let miss_genomes: Vec<(RoadefGenome, &'static str)> = miss_indices
            .iter()
            .map(|&i| {
                if let OffspringStage::NeedsEval(ref g, tag) = staging[i] {
                    (g.clone(), tag)
                } else {
                    unreachable!()
                }
            })
            .collect();
        t_staging_ms += t_staging_start.elapsed().as_secs_f64() * 1000.0;

        // Phase B: evaluate cache-miss genomes.
        // When use_rayon=true: parallel via rayon::par_iter() — fitness_eval is Arc<RoadefEvaluator> (Send+Sync).
        // When use_rayon=false: sequential L1 baseline — identical search trajectory, used for A/B comparison.
        let t_par_eval_start = Instant::now();
        let parallel_results: Vec<RoadefEvaluation> = if use_rayon {
            miss_genomes
                .par_iter()
                .map(|(genome, tag)| {
                    let mut ev = fitness_eval.evaluate(
                        genome,
                        &coralys_moga::runtime::optimization::metric::MetricReport::default(),
                    );
                    ev.operator = *tag;
                    ev
                })
                .collect()
        } else {
            miss_genomes
                .iter()
                .map(|(genome, tag)| {
                    let mut ev = fitness_eval.evaluate(
                        genome,
                        &coralys_moga::runtime::optimization::metric::MetricReport::default(),
                    );
                    ev.operator = *tag;
                    ev
                })
                .collect()
        };
        let t_par_eval_elapsed = t_par_eval_start.elapsed().as_secs_f64() * 1000.0;
        t_eval_ms += t_par_eval_elapsed;
        actual_evals += parallel_results.len();

        // Phase C: merge parallel results back into staging, update L1 cache.
        let ins_start = Instant::now();
        for (result_idx, &staging_idx) in miss_indices.iter().enumerate() {
            let ev = parallel_results[result_idx].clone();
            // Insert into L1 cache (sequential — HashMap is not Send).
            if let OffspringStage::NeedsEval(ref genome, _) = staging[staging_idx] {
                evaluation_cache.insert(genome.clone(), ev.clone());
            }
            staging[staging_idx] = OffspringStage::CacheHit(ev);
        }
        t_cache_insert_ms += ins_start.elapsed().as_secs_f64() * 1000.0;

        // Flatten staging into next_generation.
        // Phase 8: time staging flatten overhead.
        let t_flatten_start = Instant::now();
        for entry in staging {
            let ev = match entry {
                OffspringStage::CacheHit(ev) => ev,
                OffspringStage::FallbackClone(ev) => ev,
                OffspringStage::NeedsEval(..) => unreachable!("all NeedsEval resolved in Phase C"),
            };
            next_generation.push(ev);
        }
        t_staging_ms += t_flatten_start.elapsed().as_secs_f64() * 1000.0;

        // Phase 8: time sort.
        let t_sort_start = Instant::now();
        next_generation.sort_by(|a, b| {
            comparator.cmp_evals(b, a).then(
                b.fitness()
                    .partial_cmp(&a.fitness())
                    .unwrap_or(Ordering::Equal),
            )
        });
        t_sort_ms += t_sort_start.elapsed().as_secs_f64() * 1000.0;
        population = next_generation;

        let cur_best = &population[0];
        if cur_best.is_valid() && cur_best.obj < old_best_obj {
            global_best = Some(cur_best.clone());
            best_found_at_gen = generations_run;
            no_improvement_count = 0;
            if let Some(ref peak_set) = config.peak_demand_set {
                crate::moga_impl::update_peak_demands(peak_set, &cur_best.genome, &[]);
            }
        } else {
            no_improvement_count += 1;
        }

        trajectory.push(crate::moga_impl::GenerationSummary {
            generation: generations_run,
            n_eval: actual_evals,
            generation_runtime_ms: gen_start.elapsed().as_secs_f64() * 1000.0,
            evaluation_runtime_ms: t_eval_ms,
            best_obj: global_best.as_ref().map(|b| b.obj).unwrap_or(f64::INFINITY),
            duplicate_genomes: cache_hits,
            cache_hits,
            cache_lookup_ms: t_cache_lookup_ms,
            cache_hit_materialize_ms: t_cache_hit_materialize_ms,
            cache_insert_ms: t_cache_insert_ms,
            // Phase 8: operator timing fields.
            crossover_ms: t_crossover_ms,
            mutation_ms: t_mutation_ms,
            repair_ms: t_repair_ms,
            improve_ms: t_improve_ms,
            sort_ms: t_sort_ms,
            selection_ms: t_selection_ms,
            feasibility_ms: t_feasibility_ms,
            staging_ms: t_staging_ms,
        });

        if generations_run % config.log_interval == 0 {
            if let Some(ref best) = global_best {
                let _ = writeln!(log_sink,
                    "[{}] gen={:<4} obj={:<9.4} mlu={:.4} | xover={} xrep={} xmaj={} | mut={} mrep={} mmaj={}",
                    instance_name, generations_run, best.obj, best.mlu,
                    n_crossover, n_cross_repair, n_cross_maj,
                    n_mutation, n_mut_repair, n_mut_maj);
            }
        }
    }
    let mut result = EvolutionRunResult {
        trajectory,

        best_genome: factory.create(&mut rng),
        best_obj: f64::INFINITY,
        best_mlu: f64::INFINITY,
        valid: false,
        generations_run,
        best_found_at_gen,
        termination_reason,
        runtime_ms: t0.elapsed().as_millis(),
        initial_feasibility_rate,
        gen0_best_obj,
        gen0_mean_obj,
        gen0_feasible_count,
        gen0_unique_obj_count,
        gen0_duplicate_genome_count,
    };

    if let Some(best) = global_best {
        result.best_genome = best.genome;
        result.best_obj = best.obj;
        result.best_mlu = best.mlu;
        result.valid = true;
    }
    result
}
