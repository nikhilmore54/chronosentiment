use coralys_moga::ecology::metrics::{compute_pearson, compute_spearman, rank_array};
use coralys_moga::ecology::{EcologyObserver, ExternalObserver};
use coralys_moga::engine_proof::{Evaluator, EvolutionEngine, ParetoSolution};
use coralys_moga::traits::FitnessEvaluator;
use rand::Rng;
use rand::distributions::{Distribution, WeightedIndex};
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::sync::Arc;
use ultracrew::inrc::models::InrcScenario;
use ultracrew::inrc::optimization::{InrcContext, InrcGenome, InrcOptimizer};
use ultracrew::inrc::parser::{parse_history, parse_scenario, parse_week_data};
use ultracrew_server::optimizer::{ScheduleGenome, UltraCrewEvaluator, UltraCrewMutator};
use ultracrew_server::simulation::generate_baseline_schedule;
use ultracrew_server::tracker::ObservabilityTracker;

struct EvaluationPolicy {
    internal_enabled: bool,
    external_enabled: bool,
    external_frequency: usize,
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

fn to_inrc_genome(genome: &ScheduleGenome, scenario: &InrcScenario) -> InrcGenome {
    let num_nurses = scenario.nurses.len();
    let num_days = scenario.number_of_weeks * 7;
    let num_shift_types = scenario.shift_types.len();
    let size = num_nurses * num_days * num_shift_types;
    let mut bits = vec![false; size];

    let nurse_map: HashMap<String, usize> = scenario
        .nurses
        .iter()
        .enumerate()
        .map(|(i, n)| (n.id.clone(), i))
        .collect();
    let shift_map: HashMap<String, usize> = scenario
        .shift_types
        .iter()
        .enumerate()
        .map(|(i, s)| (s.id.clone(), i))
        .collect();

    for slot in &genome.slots {
        if slot.assigned_nurse.is_empty() {
            continue;
        }
        if let Some(&nurse_idx) = nurse_map.get(&slot.assigned_nurse) {
            if let Some(&shift_idx) = shift_map.get(&slot.shift_type) {
                let index = nurse_idx * (num_days * num_shift_types)
                    + slot.day * num_shift_types
                    + shift_idx;
                bits[index] = true;
            }
        }
    }

    InrcGenome { bits }
}

struct InrcExternalScorer {
    inrc_optimizer: InrcOptimizer,
    scenario: InrcScenario,
}

impl ExternalObserver<ScheduleGenome> for InrcExternalScorer {
    fn observe(&self, genome: &ScheduleGenome) -> f64 {
        self.score_with_components(genome).0
    }
}

impl InrcExternalScorer {
    fn score_with_components(&self, genome: &ScheduleGenome) -> (f64, Vec<f64>) {
        let i_genome = to_inrc_genome(genome, &self.scenario);
        let off_eval = self.inrc_optimizer.evaluate(
            &i_genome,
            &coralys_moga::runtime::optimization::metric::MetricReport::default(),
        );
        let escore = ((off_eval.hc_coverage
            + off_eval.hc_skills
            + off_eval.hc_one_shift_per_day
            + off_eval.hc_forbidden_successions) as i64
            + off_eval.soft_report.total_penalty as i64) as f64;
        let ext_objs = vec![
            off_eval.hc_coverage as f64,
            off_eval.hc_skills as f64,
            off_eval.hc_forbidden_successions as f64,
            off_eval.soft_report.optimal_coverage_penalty as f64,
            off_eval.soft_report.assignment_penalty as f64,
            off_eval.soft_report.weekend_penalty as f64,
            off_eval.soft_report.total_penalty as f64,
            escore,
        ];
        (escore, ext_objs)
    }
}

fn compute_monotonicity_violations(x: &[f64], y: &[f64]) -> usize {
    let mut violations = 0;
    let n = x.len();
    for i in 0..n {
        for j in (i + 1)..n {
            let dx = x[i] - x[j];
            let dy = y[i] - y[j];
            if (dx > 0.0 && dy < 0.0) || (dx < 0.0 && dy > 0.0) {
                violations += 1;
            }
        }
    }
    violations
}

fn compute_r_squared(x: &[f64], y: &[f64]) -> f64 {
    let pearson = compute_pearson(x, y);
    pearson * pearson
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let ablate_o3 = args.contains(&"--ablate-o3".to_string());
    let observer_only = args.contains(&"--observer-only".to_string());

    let instance = "n050w4";
    let base_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../adapters/ultracrew/tests/data/{}", instance));
    let scenario = parse_scenario(base_dir.join(format!("Sc-{}.json", instance))).unwrap();
    let week_data = parse_week_data(base_dir.join(format!("WD-{}-0.json", instance))).unwrap();
    let history = parse_history(base_dir.join(format!("H0-{}-0.json", instance))).unwrap();

    let inrc_context = InrcContext::new(
        scenario.clone(),
        week_data.clone(),
        history,
        ultracrew::ecology::WorkforceEcology::new(),
    );
    let inrc_optimizer = InrcOptimizer {
        context: Arc::new(inrc_context),
    };

    let external_scorer = InrcExternalScorer {
        inrc_optimizer,
        scenario: scenario.clone(),
    };

    println!(
        "Starting External Fitness Validation Pass for {}...",
        instance
    );

    let baseline_genome = generate_baseline_schedule(&scenario, &week_data.requirements).unwrap();

    println!("--- SANITY CHECK ---");
    let (s1, comps1) = external_scorer.score_with_components(&baseline_genome);
    let (s2, comps2) = external_scorer.score_with_components(&baseline_genome);
    let (s3, comps3) = external_scorer.score_with_components(&baseline_genome);
    println!("Run 1: {} | {:?}", s1, comps1);
    println!("Run 2: {} | {:?}", s2, comps2);
    println!("Run 3: {} | {:?}", s3, comps3);
    println!("--- END SANITY CHECK ---");
    let evaluator = UltraCrewEvaluator {
        scenario: scenario.clone(),
    };
    let mut mutator = UltraCrewMutator::new(scenario.clone());
    let mut engine = EvolutionEngine::new(evaluator, mutator);

    let mut tracker = ObservabilityTracker::new();
    let mut ecology_observer = EcologyObserver::new();

    let policy = EvaluationPolicy {
        internal_enabled: true,
        external_enabled: true,
        external_frequency: 1, // evaluate external fitness every time
    };

    let mut eval_counter = 0;
    let mut csv_file = File::create("validation_alignment.csv").unwrap();
    writeln!(csv_file, "Generation,UID,InternalFitness,ExternalFitness,ArchiveMember,Admitted,Dominated,Age,Novelty").unwrap();

    let mut evaluate_candidate =
        |genome: &ScheduleGenome,
         _g: u64,
         _parent_discovery_gen: u64,
         engine: &mut EvolutionEngine<ScheduleGenome, UltraCrewEvaluator, UltraCrewMutator>,
         external_scorer: &InrcExternalScorer,
         policy: &EvaluationPolicy,
         eval_counter: &mut usize|
         -> (Vec<f64>, Vec<f64>, f64, f64, Vec<f64>) {
            let logging_fitness = engine.evaluator.evaluate(genome);
            let mut selection_fitness = logging_fitness.clone();

            if ablate_o3 {
                selection_fitness[2] = 0.0;
            }

            let num_objs = selection_fitness.len();

            let mut e_score = -1.0;
            let mut e_objs = Vec::new();
            *eval_counter += 1;
            if policy.external_enabled
                && (*eval_counter % policy.external_frequency == 0 || *eval_counter == 1)
            {
                let (s, objs) = external_scorer.score_with_components(genome);
                e_score = s;
                e_objs = objs;
            }

            let archive_size = engine.archive.solutions.len();
            let novelty = if archive_size == 0 {
                0.0
            } else {
                let mut min_dist = f64::INFINITY;
                for sol in &engine.archive.solutions {
                    let dist = (0..num_objs)
                        .map(|d| (selection_fitness[d] - sol.fitness[d]).powi(2))
                        .sum::<f64>()
                        .sqrt();
                    if dist < min_dist {
                        min_dist = dist;
                    }
                }
                min_dist
            };

            (selection_fitness, logging_fitness, e_score, novelty, e_objs)
        };

    // Baseline Evaluate
    let i_genome = to_inrc_genome(&baseline_genome, &external_scorer.scenario);
    let off_eval = external_scorer.inrc_optimizer.evaluate(
        &i_genome,
        &coralys_moga::runtime::optimization::metric::MetricReport::default(),
    );
    let base_fitness = engine.evaluator.evaluate(&baseline_genome);
    let base_escore = ((off_eval.hc_coverage
        + off_eval.hc_skills
        + off_eval.hc_one_shift_per_day
        + off_eval.hc_forbidden_successions) as i64
        * 100_000
        + off_eval.soft_report.total_penalty as i64) as f64;
    let base_external_objs = vec![
        off_eval.hc_coverage as f64,
        off_eval.hc_skills as f64,
        off_eval.hc_forbidden_successions as f64,
        off_eval.soft_report.optimal_coverage_penalty as f64,
        off_eval.soft_report.assignment_penalty as f64,
        off_eval.soft_report.weekend_penalty as f64,
        base_escore,
    ];
    let base_uid = calculate_hash(&baseline_genome);
    let base_pscore = base_fitness.iter().sum::<f64>();

    tracker.record_evaluation(
        base_uid,
        0,
        base_pscore,
        base_escore,
        true,
        true,
        false,
        0,
        0.0,
        base_fitness.clone(),
        base_external_objs,
    );
    writeln!(
        csv_file,
        "{},{},{:.2},{:.2},{},{},{},{},{:.4}",
        0, base_uid, base_pscore, base_escore, true, true, false, 0, 0.0
    )
    .unwrap();

    engine.archive.add(ParetoSolution {
        genome: baseline_genome,
        fitness: base_fitness,
        uid: base_uid,
        parent_uid: 0,
    });

    let max_generations = 5000;

    for g in 1..=max_generations {
        let archive_size = engine.archive.solutions.len();
        if archive_size == 0 {
            break;
        }
        let num_objs = engine.archive.solutions[0].fitness.len();

        let mut idx = 0;
        let mut min_vals = vec![f64::INFINITY; num_objs];
        let mut max_vals = vec![0.0_f64; num_objs];
        let mut ranges = vec![1e-9; num_objs];

        for d in 0..num_objs {
            let vals: Vec<f64> = engine
                .archive
                .solutions
                .iter()
                .map(|s| s.fitness[d])
                .collect();
            min_vals[d] = vals.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            max_vals[d] = vals.iter().fold(0.0_f64, |a, &b| a.max(b));
            ranges[d] = max_vals[d] - min_vals[d] + 1e-9;
        }

        let mut weights = Vec::with_capacity(archive_size);
        if archive_size == 1 {
            weights.push(1.0);
        } else {
            let mut normalized_coords = vec![vec![0.0; num_objs]; archive_size];
            for d in 0..num_objs {
                for i in 0..archive_size {
                    normalized_coords[i][d] =
                        (engine.archive.solutions[i].fitness[d] - min_vals[d]) / ranges[d];
                }
            }
            for i in 0..archive_size {
                let mut min_dist = f64::INFINITY;
                for j in 0..archive_size {
                    if i == j {
                        continue;
                    }
                    let dist = (0..num_objs)
                        .map(|d| (normalized_coords[i][d] - normalized_coords[j][d]).powi(2))
                        .sum::<f64>()
                        .sqrt();
                    if dist < min_dist {
                        min_dist = dist;
                    }
                }
                weights.push((min_dist + 1e-9).powf(0.5));
            }
            let total_weight: f64 = weights.iter().sum();
            for w in weights.iter_mut() {
                *w /= total_weight;
            }
            let dist = WeightedIndex::new(&weights).unwrap();
            let mut rng = rand::thread_rng();
            idx = dist.sample(&mut rng);
        }

        let parent = engine.archive.solutions[idx].clone();
        let parent_discovery_gen = tracker
            .lifecycles
            .get(&parent.uid)
            .map(|l| l.discovery_generation)
            .unwrap_or(0);
        let age = g - parent_discovery_gen;

        let mut rng = rand::thread_rng();
        let num_offspring = 5;

        let mut candidates = Vec::new();

        let calc_raw_energy =
            |fitness: &[f64]| -> f64 { fitness.iter().map(|v| v.powi(2)).sum::<f64>().sqrt() };

        for _ in 0..num_offspring {
            let candidate_genome = engine
                .mutator
                .mutate_with_tier(&parent.genome, rng.gen_bool(0.8));
            let (sel_fitness, log_fitness, e_score, novelty, e_objs) = evaluate_candidate(
                &candidate_genome,
                g,
                parent_discovery_gen,
                &mut engine,
                &external_scorer,
                &policy,
                &mut eval_counter,
            );
            let energy = calc_raw_energy(&sel_fitness);
            candidates.push((
                candidate_genome,
                sel_fitness,
                log_fitness,
                e_score,
                novelty,
                energy,
                e_objs,
            ));
        }

        let mut best_candidate = candidates
            .into_iter()
            .min_by(|a, b| a.5.partial_cmp(&b.5).unwrap())
            .unwrap();

        let mut t = 1000.0;
        let alpha = 0.95;
        let mut sa_candidates = Vec::new();

        for _ in 0..20 {
            let neighbor_genome = engine
                .mutator
                .mutate_with_tier(&best_candidate.0, rng.gen_bool(0.8));
            let (sel_fitness, log_fitness, e_score, novelty, e_objs) = evaluate_candidate(
                &neighbor_genome,
                g,
                parent_discovery_gen,
                &mut engine,
                &external_scorer,
                &policy,
                &mut eval_counter,
            );
            let neighbor_energy = calc_raw_energy(&sel_fitness);

            sa_candidates.push((
                neighbor_genome.clone(),
                sel_fitness.clone(),
                log_fitness.clone(),
                e_score,
                novelty,
                neighbor_energy,
                e_objs.clone(),
            ));

            let delta = neighbor_energy - best_candidate.5;

            if delta < 0.0 || rng.gen_range(0.0..1.0) < (-delta / t).exp() {
                best_candidate = (
                    neighbor_genome,
                    sel_fitness,
                    log_fitness,
                    e_score,
                    novelty,
                    neighbor_energy,
                    e_objs,
                );
            }
            t *= alpha;
        }

        let final_uid = calculate_hash(&best_candidate.0);
        let final_pscore = best_candidate.2.iter().sum::<f64>();

        let old_uids: Vec<u64> = engine.archive.solutions.iter().map(|s| s.uid).collect();
        let was_inserted = engine.archive.add(ParetoSolution {
            genome: best_candidate.0.clone(),
            fitness: best_candidate.1.clone(), // use sel_fitness for Pareto dominance
            uid: final_uid,
            parent_uid: parent.uid,
        });

        // Use log_fitness (best_candidate.2) for the tracker!
        tracker.record_evaluation(
            final_uid,
            g,
            final_pscore,
            best_candidate.3,
            was_inserted,
            was_inserted,
            !was_inserted,
            age,
            best_candidate.4,
            best_candidate.2.clone(),
            best_candidate.6.clone(),
        );
        writeln!(
            csv_file,
            "{},{},{:.2},{:.2},{},{},{},{},{:.4}",
            g,
            final_uid,
            final_pscore,
            best_candidate.3,
            was_inserted,
            was_inserted,
            !was_inserted,
            age,
            best_candidate.4
        )
        .unwrap();

        // Evictions
        let new_uids: Vec<u64> = engine.archive.solutions.iter().map(|s| s.uid).collect();
        for old_uid in old_uids {
            if !new_uids.contains(&old_uid) {
                tracker.record_eviction(old_uid, g);
            }
        }

        if g % 1000 == 0 {
            let archive_external_fitnesses: Vec<f64> = engine
                .archive
                .solutions
                .iter()
                .map(|s| external_scorer.observe(&s.genome))
                .collect();
            tracker.snapshot_archive(g, &archive_external_fitnesses);
            let snap = tracker.snapshots.last().unwrap();
            println!(
                "Gen {}: Archive {}, Best External: {}, Median: {}, Worst: {}",
                g,
                engine.archive.solutions.len(),
                snap.best_fitness,
                snap.p50,
                snap.worst_fitness
            );

            // Ecology Observer
            let proxy_objs: Vec<Vec<f64>> = engine
                .archive
                .solutions
                .iter()
                .map(|s| s.fitness.clone())
                .collect();
            ecology_observer.compute_and_record_correlations(
                g,
                &proxy_objs,
                &archive_external_fitnesses,
            );
            ecology_observer.record_best_external(g, snap.best_fitness);
        }
    }

    println!(
        "\nValidation Complete. {} Final Candidates Evaluated.",
        tracker.evaluations
    );
    if let Some(best) = &tracker.best_ever {
        println!(
            "Best Ever External Fitness: {} (Found at Gen {})",
            best.fitness, best.generation
        );
    }

    // Output external champion curve
    let mut curve_file = File::create("external_champion_curve.csv").unwrap();
    writeln!(curve_file, "Generation,UID,ExternalFitness,InternalFitness").unwrap();

    let mut current_best = f64::MAX;
    for r in &tracker.alignment_records {
        if r.external_fitness < current_best {
            current_best = r.external_fitness;
            writeln!(
                curve_file,
                "{},{},{},{:.2}",
                r.generation, r.uid, r.external_fitness, r.internal_fitness_sum
            )
            .unwrap();
        }
    }

    // Compute Champion Retention Error
    let best_ever = tracker.best_ever.as_ref().map(|b| b.fitness).unwrap_or(0.0);
    let best_ever_uid = tracker.best_ever.as_ref().map(|b| b.uid).unwrap_or(0);
    let final_archive_uids: Vec<u64> = engine.archive.solutions.iter().map(|s| s.uid).collect();
    let retention_error = if final_archive_uids.contains(&best_ever_uid) {
        0
    } else {
        1
    };
    let best_final_archive = tracker
        .snapshots
        .last()
        .map(|s| s.best_fitness)
        .unwrap_or(0.0);
    let alignment_loss = best_final_archive - best_ever;

    // Compute Correlations
    let p_scores: Vec<f64> = tracker
        .alignment_records
        .iter()
        .map(|r| r.internal_fitness_sum)
        .collect();
    let e_scores: Vec<f64> = tracker
        .alignment_records
        .iter()
        .map(|r| r.external_fitness)
        .collect();

    let pearson = compute_pearson(&p_scores, &e_scores);
    let spearman = compute_spearman(&p_scores, &e_scores);

    let r_squared = compute_r_squared(&p_scores, &e_scores);
    let monotonicity_violations = compute_monotonicity_violations(&p_scores, &e_scores);

    // Champion Agreement
    let mut ranked = tracker.alignment_records.clone();

    let n = ranked.len();
    let total_pairs = (n * (n - 1)) / 2;
    let top1_count = (n as f64 * 0.01).ceil() as usize;
    let top5_count = (n as f64 * 0.05).ceil() as usize;

    // Top Internal
    ranked.sort_by(|a, b| {
        a.internal_fitness_sum
            .partial_cmp(&b.internal_fitness_sum)
            .unwrap()
    });
    let top1_internal: Vec<u64> = ranked.iter().take(top1_count).map(|r| r.uid).collect();
    let top5_internal: Vec<u64> = ranked.iter().take(top5_count).map(|r| r.uid).collect();

    // Top External
    ranked.sort_by(|a, b| a.external_fitness.partial_cmp(&b.external_fitness).unwrap());
    let top1_external: Vec<u64> = ranked.iter().take(top1_count).map(|r| r.uid).collect();
    let top5_external: Vec<u64> = ranked.iter().take(top5_count).map(|r| r.uid).collect();

    let top1_overlap = top1_internal
        .iter()
        .filter(|id| top1_external.contains(id))
        .count() as f64
        / top1_count as f64;
    let top5_overlap = top5_internal
        .iter()
        .filter(|id| top5_external.contains(id))
        .count() as f64
        / top5_count as f64;

    let mut obj_spearmans = Vec::new();
    let num_objs = tracker.alignment_records[0].fitness_objs.len();
    for o in 0..num_objs {
        let o_scores: Vec<f64> = tracker
            .alignment_records
            .iter()
            .map(|r| r.fitness_objs[o])
            .collect();
        let spearman_o = compute_spearman(&o_scores, &e_scores);
        obj_spearmans.push(spearman_o);
    }

    let soft_scores: Vec<f64> = tracker
        .alignment_records
        .iter()
        .map(|r| r.external_objs[6])
        .collect();
    let spearman_soft = compute_spearman(&p_scores, &soft_scores);

    println!("\n--- Alignment Metrics ---");
    println!("Pearson Correlation: {:.4}", pearson);
    println!("Spearman Correlation (vs Total Official): {:.4}", spearman);
    println!(
        "Spearman Correlation (vs Soft Official Only): {:.4}",
        spearman_soft
    );
    println!("R²: {:.4}", r_squared);
    println!(
        "Monotonicity Violations: {} / {} ({:.2}%)",
        monotonicity_violations,
        total_pairs,
        (monotonicity_violations as f64 / total_pairs as f64) * 100.0
    );
    println!("Top 1% Champion Agreement: {:.2}%", top1_overlap * 100.0);
    println!("Top 5% Champion Agreement: {:.2}%", top5_overlap * 100.0);
    println!("\n--- Individual Objective Spearman vs Official ---");
    for (i, &s) in obj_spearmans.iter().enumerate() {
        println!("Objective {}: {:.4}", i + 1, s);
    }

    let mut heatmap = String::new();
    let ext_names = [
        "HC_Coverage",
        "HC_Skills",
        "HC_Successions",
        "S1_OptCoverage",
        "S6_Assignment",
        "S7_Weekend",
        "SoftTotal",
    ];
    heatmap.push_str("\n--- Official Component Attribution Heatmap (Spearman vs Proxy) ---\n");
    heatmap.push_str(
        format!(
            "{:>4} | {:>10} | {:>10} | {:>10} | {:>10} | {:>10} | {:>10} | {:>10}\n",
            "",
            ext_names[0],
            ext_names[1],
            ext_names[2],
            ext_names[3],
            ext_names[4],
            ext_names[5],
            ext_names[6]
        )
        .as_str(),
    );

    for (i, _) in obj_spearmans.iter().enumerate() {
        let o_scores: Vec<f64> = tracker
            .alignment_records
            .iter()
            .map(|r| r.fitness_objs[i])
            .collect();
        let mut row = format!("O{:<3} |", i + 1);
        for ext_idx in 0..7 {
            let ext_vals: Vec<f64> = tracker
                .alignment_records
                .iter()
                .map(|r| r.external_objs[ext_idx])
                .collect();
            let mut all_zero = true;
            for &v in &ext_vals {
                if v != 0.0 {
                    all_zero = false;
                    break;
                }
            }
            if all_zero {
                row.push_str(&format!(" {:>10} |", "N/A"));
            } else {
                let s = compute_spearman(&o_scores, &ext_vals);
                row.push_str(&format!(" {:>10.4} |", s));
            }
        }
        heatmap.push_str(&row);
        heatmap.push('\n');
    }
    println!("{}", heatmap);

    println!("\n--- Archive Quality Drift ---");
    let mut drift_file = File::create("archive_quality_drift.csv").unwrap();
    writeln!(drift_file, "Generation,Best,P10,P25,P50,P75,P90,Worst").unwrap();
    for snap in &tracker.snapshots {
        println!(
            "Gen {}: Best={}, P25={}, P50={}, P75={}, Worst={}",
            snap.generation, snap.best_fitness, snap.p25, snap.p50, snap.p75, snap.worst_fitness
        );
        writeln!(
            drift_file,
            "{},{},{},{},{},{},{},{}",
            snap.generation,
            snap.best_fitness,
            snap.p10,
            snap.p25,
            snap.p50,
            snap.p75,
            snap.p90,
            snap.worst_fitness
        )
        .unwrap();
    }

    println!(
        "\nAlignment Loss: {} (Best Final: {} - Best Ever: {})",
        alignment_loss, best_final_archive, best_ever
    );
    println!("Champion Retention Error: {}", retention_error);

    println!("\n--- Ecology Observer Temporal Matrices ---");
    for (gen_num, spearmans) in &ecology_observer.correlation_history {
        println!("Generation {}:", gen_num);
        for (i, s) in spearmans.iter().enumerate() {
            println!("  O{} ↔ Ext = {:.4}", i + 1, s);
        }
    }

    println!("\n--- External Discovery Velocity ---");
    let velocities = ecology_observer.calculate_discovery_velocity();
    for (start, end, diff) in velocities {
        println!(
            "{:>4}-{:>4} : {:>8} pts / 1000 gen",
            start, end, diff as i64
        );
    }

    // Write validation_summary.txt
    let mut summary = File::create("validation_summary.txt").unwrap();
    writeln!(summary, "Validation Summary - {}", instance).unwrap();
    writeln!(summary, "-----------------------------").unwrap();
    writeln!(summary, "Pearson: {:.4}", pearson).unwrap();
    writeln!(summary, "Spearman: {:.4}", spearman).unwrap();
    writeln!(summary, "R²: {:.4}", r_squared).unwrap();
    writeln!(
        summary,
        "Monotonicity Violations: {} / {} ({:.2}%)",
        monotonicity_violations,
        total_pairs,
        (monotonicity_violations as f64 / total_pairs as f64) * 100.0
    )
    .unwrap();
    writeln!(
        summary,
        "Top 1% Champion Agreement: {:.2}%",
        top1_overlap * 100.0
    )
    .unwrap();
    writeln!(
        summary,
        "Top 5% Champion Agreement: {:.2}%",
        top5_overlap * 100.0
    )
    .unwrap();
    writeln!(summary, "Alignment Loss: {}", alignment_loss).unwrap();
    writeln!(summary, "Champion Retention Error: {}", retention_error).unwrap();
    writeln!(summary, "\nSuccess Criteria check:").unwrap();
    if spearman >= 0.70 && top5_overlap >= 0.60 && (alignment_loss / best_ever) <= 0.10 {
        writeln!(summary, "Result: GREEN (Proceed to league_table.rs)").unwrap();
    } else if spearman >= 0.40 && top5_overlap >= 0.30 {
        writeln!(
            summary,
            "Result: YELLOW (Run objective-space investigation before 60-run matrix)"
        )
        .unwrap();
    } else {
        writeln!(
            summary,
            "Result: RED (Stop immediately. Re-evaluate objective space.)"
        )
        .unwrap();
    }
}
