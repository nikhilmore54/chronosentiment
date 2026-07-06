use serde::Serialize;
use std::collections::HashSet;
use cvrp::CvrpCandidate;
use std::cmp::Ordering;
use rand::SeedableRng;

use coralys_moga::traits::{FitnessEvaluator, MutationOperator, CrossoverOperator, GenomeFactory, Evaluated, Genome};
use coralys_core::memory::InnovationTracker;
use cvrp::{CvrpInstance, moga_impl::*};
use cvrp::CvrpGenomeFactory;

#[derive(Serialize, Clone, Default)]
struct RunTelemetryAverages {
    pop_diversity: f64,
    elite_diversity: f64,
    feasible_ratio: f64,
    parent_similarity: f64,
    offspring_novelty: f64,
    structural_damage: f64,
    elite_survival: f64,
    active_memory_size: f64,
    innovation_persistence: f64,
    rediscovery_ratio: f64,
}

#[derive(Serialize, Clone)]
struct SeedRunResult {
    seed: u64,
    best_final_distance: f64,
    generation_of_first_elite_improvement: usize,
    time_spent_in_final_basin: usize,
    avg_telemetry: RunTelemetryAverages,
    progress_at_1000: coralys_ecology::progress::ProgressObservation,
}

#[derive(Serialize)]
struct Report {
    top_10_percent_avg_telemetry: RunTelemetryAverages,
    bottom_10_percent_avg_telemetry: RunTelemetryAverages,
    top_10_percent_avg_distance: f64,
    bottom_10_percent_avg_distance: f64,
    top_10_avg_gen_first_improvement: f64,
    top_10_avg_gen_final_improvement: f64,
    bottom_10_avg_time_in_final_basin: f64,
    top_10_gen1000_basin_residency: f64,
    bottom_10_gen1000_basin_residency: f64,
    top_10_gen1000_improvement_rate_100: f64,
    bottom_10_gen1000_improvement_rate_100: f64,
    all_runs: Vec<SeedRunResult>,
}

fn aggregate_telemetry(runs: &[SeedRunResult]) -> RunTelemetryAverages {
    let mut avg = RunTelemetryAverages::default();
    if runs.is_empty() { return avg; }
    for r in runs {
        let t = &r.avg_telemetry;
        avg.pop_diversity += t.pop_diversity;
        avg.elite_diversity += t.elite_diversity;
        avg.feasible_ratio += t.feasible_ratio;
        avg.parent_similarity += t.parent_similarity;
        avg.offspring_novelty += t.offspring_novelty;
        avg.structural_damage += t.structural_damage;
        avg.elite_survival += t.elite_survival;
        avg.active_memory_size += t.active_memory_size;
        avg.innovation_persistence += t.innovation_persistence;
        avg.rediscovery_ratio += t.rediscovery_ratio;
    }
    let n = runs.len() as f64;
    avg.pop_diversity /= n;
    avg.elite_diversity /= n;
    avg.feasible_ratio /= n;
    avg.parent_similarity /= n;
    avg.offspring_novelty /= n;
    avg.structural_damage /= n;
    avg.elite_survival /= n;
    avg.active_memory_size /= n;
    avg.innovation_persistence /= n;
    avg.rediscovery_ratio /= n;
    avg
}

fn get_canonical_edges(routes: &Vec<Vec<usize>>) -> HashSet<(usize, usize)> {
    let mut edges = HashSet::new();
    let depot_id = 1;
    for r in routes {
        if r.is_empty() { continue; }
        let mut prev = depot_id;
        for &node in r {
            edges.insert((prev.min(node), prev.max(node)));
            prev = node;
        }
        edges.insert((prev.min(depot_id), prev.max(depot_id)));
    }
    edges
}

fn main() {
    let instance = CvrpInstance::a_n32_k5();
    let evaluator = CvrpEvaluator { instance: instance.clone() };
    let mut mutator = CvrpMutator::new(instance.clone(), cvrp::RadiusPolicy::Control);
    mutator.entropy_scale = 1.0;
    let crossover = cvrp::moga_impl::CvrpCrossoverVariant::OX1(cvrp::moga_impl::CvrpCrossover);
    let factory = CvrpGenomeFactory { num_customers: instance.customers.len() };

    let r_bks = vec![
        vec![15, 29, 12, 5, 24, 4, 3, 7],
        vec![27, 8, 14, 18, 20, 32, 22],
        vec![30, 19, 9, 10, 23, 16, 11, 26, 6, 21],
        vec![13, 2, 17, 31],
        vec![28, 25]
    ];
    let bks_edges = get_canonical_edges(&r_bks);

    let num_seeds = 3;
    let generations = 2000;
    let population_size = 100;

    let mut results = Vec::new();

    println!("=== M22-0: Elite Manifold Archive ===");
    println!("Running {} seeds for {} generations...", num_seeds, generations);

    for seed_idx in 0..num_seeds {
        let seed = 42 + seed_idx as u64 * 100;
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

        let mut population: Vec<_> = (0..population_size)
            .map(|_| factory.create(&mut rng))
            .collect();

        let mut global_best_distance = f64::INFINITY;
        let mut gen_first_improvement = 0;
        let mut gen_final_improvement = 0;

        let mut innovation_tracker = coralys_core::memory::InnovationTracker::new();
        let mut progress_tracker = coralys_ecology::progress::ProgressTracker::new();
        let mut gen_1000_progress = None;
        let mut sum_telemetry = RunTelemetryAverages::default();

        // (dist, genome, edges, added_generation)
        let mut passive_archive: Vec<(f64, CvrpCandidate, HashSet<(usize, usize)>, usize)> = Vec::new();
        let mut eviction_lifetimes = Vec::new();

        for generation in 1..=generations {
            let mut evals: Vec<_> = population
                .iter()
                .map(|c| evaluator.evaluate(c))
                .filter(|e| e.is_valid())
                .collect();

            if evals.is_empty() {
                population = (0..population_size).map(|_| factory.create(&mut rng)).collect();
                continue;
            }

            evals.sort_by(|a, b| b.fitness().partial_cmp(&a.fitness()).unwrap_or(Ordering::Equal));
            let gen_best = evals[0].clone();
            let best_dist = gen_best.eval.total_distance;
            
            if best_dist < global_best_distance {
                global_best_distance = best_dist;
                if gen_first_improvement == 0 {
                    gen_first_improvement = generation;
                }
                gen_final_improvement = generation;
            }

            // Passive Archive Logic
            for eval in &evals {
                let dist = eval.eval.total_distance;
                if dist <= 900.0 {
                    let edges = get_canonical_edges(&eval.eval.routes);
                    let mut novel = true;
                    for (_, _, arc_edges, _) in &passive_archive {
                        let diff = edges.symmetric_difference(arc_edges).count();
                        if diff < 10 {
                            novel = false;
                            break;
                        }
                    }
                    if novel && passive_archive.len() < 50 {
                        passive_archive.push((dist, eval.genome().clone(), edges, generation));
                    } else if novel {
                        // Find worst in archive, replace if dist is better
                        let mut worst_idx = 0;
                        let mut worst_dist = passive_archive[0].0;
                        for (i, (d, _, _, _)) in passive_archive.iter().enumerate() {
                            if *d > worst_dist {
                                worst_dist = *d;
                                worst_idx = i;
                            }
                        }
                        if dist < worst_dist {
                            let (_, _, _, added_gen) = passive_archive[worst_idx];
                            eviction_lifetimes.push(generation - added_gen);
                            passive_archive[worst_idx] = (dist, eval.genome().clone(), edges, generation);
                        }
                    }
                }
            }

            let mut unique_dists: Vec<_> = evals.iter().map(|e| (e.eval.total_distance * 1000.0).round() as i64).collect();
            unique_dists.sort_unstable();
            unique_dists.dedup();
            let diversity_score = unique_dists.len() as f64 / evals.len() as f64;

            let num_elites = (evals.len() / 5).max(1);
            let mut unique_elite_dists: Vec<_> = evals[0..num_elites].iter().map(|e| (e.eval.total_distance * 1000.0).round() as i64).collect();
            unique_elite_dists.sort_unstable();
            unique_elite_dists.dedup();
            let elite_diversity_score = unique_elite_dists.len() as f64 / num_elites as f64;
            let feasible_ratio = evals.len() as f64 / population_size as f64;

            let median_fitness = evals[evals.len() / 2].fitness();

            let mut next_gen = Vec::with_capacity(population_size);
            
            let mut elite_count = 0;
            let mut iter = evals.iter();
            while elite_count < 10 {
                if let Some(e) = iter.next() {
                    next_gen.push(e.genome().clone());
                    elite_count += 1;
                } else {
                    break;
                }
            }

            let mut total_parent_sim = 0.0;
            let mut total_offspring_nov = 0.0;
            let mut total_pairs = 0;
            let mut total_preserved_pairs = 0;
            let mut num_offspring = 0.0;
            let mut children_better_than_median = 0;

            while next_gen.len() < population_size {
                use rand::seq::SliceRandom;
                
                let tournament = |rng: &mut rand::rngs::StdRng| {
                    let mut best = evals.choose(rng).unwrap();
                    for _ in 0..4 {
                        let candidate = evals.choose(rng).unwrap();
                        if candidate.fitness() > best.fitness() { best = candidate; }
                    }
                    best
                };

                let p1_eval = tournament(&mut rng);
                
                if rand::Rng::gen_bool(&mut rng, 0.8) {
                    let p2_eval = tournament(&mut rng);
                    let (mut c1, mut c2) = crossover.crossover(p1_eval.genome(), p2_eval.genome(), &mut rng);
                    
                    let size = p1_eval.genome().permutation.len();
                    let mut identical_parents = 0;
                    for k in 0..size {
                        if p1_eval.genome().permutation[k] == p2_eval.genome().permutation[k] { identical_parents += 1; }
                    }
                    total_parent_sim += identical_parents as f64 / size as f64;

                    for c in [&c1, &c2] {
                        let mut ident_p1 = 0; let mut ident_p2 = 0;
                        for k in 0..size {
                            if c.permutation[k] == p1_eval.genome().permutation[k] { ident_p1 += 1; }
                            if c.permutation[k] == p2_eval.genome().permutation[k] { ident_p2 += 1; }
                        }
                        total_offspring_nov += 1.0 - (ident_p1 as f64 / size as f64).max(ident_p2 as f64 / size as f64);
                        num_offspring += 1.0;
                    }

                    mutator.mutate(&mut c1, &mut rng);
                    mutator.mutate(&mut c2, &mut rng);

                    let c1_eval = evaluator.evaluate(&c1);
                    let c2_eval = evaluator.evaluate(&c2);

                    if c1_eval.fitness() > median_fitness { children_better_than_median += 1; }
                    if c2_eval.fitness() > median_fitness { children_better_than_median += 1; }

                    for c_eval in [&c1_eval, &c2_eval] {
                        for route in &c_eval.eval.routes {
                            for window in route.windows(2) {
                                let a = window[0]; let b = window[1];
                                let mut preserved = false;
                                for p_eval in [&p1_eval, &p2_eval] {
                                    if p_eval.eval.routes.iter().any(|r| r.windows(2).any(|w| (w[0]==a && w[1]==b) || (w[0]==b && w[1]==a))) {
                                        preserved = true; break;
                                    }
                                }
                                total_pairs += 1;
                                if preserved { total_preserved_pairs += 1; }
                            }
                        }
                    }
                    next_gen.push(c1);
                    if next_gen.len() < population_size { next_gen.push(c2); }
                } else {
                    let mut child = p1_eval.genome().clone();
                    mutator.mutate(&mut child, &mut rng);
                    next_gen.push(child);
                }
            }
            
            let structural_damage = if total_pairs > 0 { 1.0 - (total_preserved_pairs as f64 / total_pairs as f64) } else { 0.0 };
            let parent_similarity = if num_offspring > 0.0 { total_parent_sim / (num_offspring / 2.0) } else { 0.0 };
            let offspring_novelty = if num_offspring > 0.0 { total_offspring_nov / num_offspring } else { 0.0 };
            let elite_survival = if num_offspring > 0.0 { children_better_than_median as f64 / num_offspring } else { 0.0 };

            let mut generation_signatures = Vec::new();
            for eval in &evals {
                for route in &eval.eval.routes {
                    let mut prev = 0;
                    for &node in route {
                        let sig = ((prev.min(node) as u64) << 32) | (prev.max(node) as u64);
                        generation_signatures.push(sig);
                        prev = node;
                    }
                    let sig = ((prev.min(0) as u64) << 32) | (prev.max(0) as u64);
                    generation_signatures.push(sig);
                }
            }
            
            if generation == 1000 {
                gen_1000_progress = Some(progress_tracker.observe_minimization(generation, best_dist));
            } else {
                progress_tracker.observe_minimization(generation, best_dist);
            }

            let mem = innovation_tracker.observe(&generation_signatures);

            sum_telemetry.pop_diversity += diversity_score;
            sum_telemetry.elite_diversity += elite_diversity_score;
            sum_telemetry.feasible_ratio += feasible_ratio;
            sum_telemetry.parent_similarity += parent_similarity;
            sum_telemetry.offspring_novelty += offspring_novelty;
            sum_telemetry.structural_damage += structural_damage;
            sum_telemetry.elite_survival += elite_survival;
            sum_telemetry.active_memory_size += mem.active_memory_size as f64;
            sum_telemetry.innovation_persistence += mem.persistence_ratio;
            sum_telemetry.rediscovery_ratio += mem.rediscovery_ratio;

            population = next_gen;
        }

        let gens_f64 = generations as f64;
        sum_telemetry.pop_diversity /= gens_f64;
        sum_telemetry.elite_diversity /= gens_f64;
        sum_telemetry.feasible_ratio /= gens_f64;
        sum_telemetry.parent_similarity /= gens_f64;
        sum_telemetry.offspring_novelty /= gens_f64;
        sum_telemetry.structural_damage /= gens_f64;
        sum_telemetry.elite_survival /= gens_f64;
        sum_telemetry.active_memory_size /= gens_f64;
        sum_telemetry.innovation_persistence /= gens_f64;
        sum_telemetry.rediscovery_ratio /= gens_f64;

        let time_in_basin = generations - gen_final_improvement;

        let mut best_archive_dist = 9999.0;
        let mut max_bks_overlap = 0;
        for (d, _, edges, added_gen) in &passive_archive {
            if *d < best_archive_dist { best_archive_dist = *d; }
            let overlap = edges.intersection(&bks_edges).count();
            if overlap > max_bks_overlap { max_bks_overlap = overlap; }
            eviction_lifetimes.push(generations - added_gen + 1); // For the ones surviving till the end
        }
        
        let avg_lifetime = if eviction_lifetimes.is_empty() { 0.0 } else { 
            eviction_lifetimes.iter().sum::<usize>() as f64 / eviction_lifetimes.len() as f64 
        };

        println!("Seed {} -> Dist: {:.2} | Archive Best: {:.2} | Archive Families: {} | Max BKS Overlap: {} | Avg Lifetime: {:.1}", 
            seed, global_best_distance, best_archive_dist, passive_archive.len(), max_bks_overlap, avg_lifetime);

        results.push(SeedRunResult {
            seed,
            best_final_distance: global_best_distance,
            generation_of_first_elite_improvement: gen_first_improvement,
            time_spent_in_final_basin: time_in_basin,
            avg_telemetry: sum_telemetry,
            progress_at_1000: gen_1000_progress.unwrap(),
        });
    }

    results.sort_by(|a, b| a.best_final_distance.partial_cmp(&b.best_final_distance).unwrap());

    let top_10_count = num_seeds / 10;
    let top_10 = &results[0..top_10_count];
    let bottom_10 = &results[(results.len() - top_10_count)..];

    let report = Report {
        top_10_percent_avg_telemetry: aggregate_telemetry(top_10),
        bottom_10_percent_avg_telemetry: aggregate_telemetry(bottom_10),
        top_10_percent_avg_distance: top_10.iter().map(|r| r.best_final_distance).sum::<f64>() / top_10_count as f64,
        bottom_10_percent_avg_distance: bottom_10.iter().map(|r| r.best_final_distance).sum::<f64>() / top_10_count as f64,
        top_10_avg_gen_first_improvement: top_10.iter().map(|r| r.generation_of_first_elite_improvement as f64).sum::<f64>() / top_10_count as f64,
        top_10_avg_gen_final_improvement: top_10.iter().map(|r| r.generation_of_first_elite_improvement as f64).sum::<f64>() / top_10_count as f64,
        bottom_10_avg_time_in_final_basin: bottom_10.iter().map(|r| r.time_spent_in_final_basin as f64).sum::<f64>() / bottom_10.len() as f64,
        top_10_gen1000_basin_residency: top_10.iter().map(|r| r.progress_at_1000.basin_residency_ratio).sum::<f64>() / top_10.len() as f64,
        bottom_10_gen1000_basin_residency: bottom_10.iter().map(|r| r.progress_at_1000.basin_residency_ratio).sum::<f64>() / bottom_10.len() as f64,
        top_10_gen1000_improvement_rate_100: top_10.iter().map(|r| r.progress_at_1000.improvement_rate_100).sum::<f64>() / top_10.len() as f64,
        bottom_10_gen1000_improvement_rate_100: bottom_10.iter().map(|r| r.progress_at_1000.improvement_rate_100).sum::<f64>() / bottom_10.len() as f64,
        all_runs: results.clone(),
    };

    let json = serde_json::to_string_pretty(&results).unwrap();
    std::fs::write("m8g_cvrp_report.json", json).unwrap();
    println!("Report written to m8g_cvrp_report.json");
}
