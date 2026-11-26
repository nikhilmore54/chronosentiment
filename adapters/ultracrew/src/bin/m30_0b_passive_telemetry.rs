use std::cmp::Ordering;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use coralys_moga::traits::*;
use ultracrew::ecology::WorkforceEcology;
use ultracrew::inrc::optimization::{InrcContext, InrcEvaluation, InrcGenome, InrcOptimizer};
use ultracrew::inrc::parser::{parse_history, parse_scenario, parse_week_data};

struct TelemetryRecord {
    generation: usize,
    parent_fitness: f64,
    parent_hc_cov: usize,
    parent_hc_skills: usize,
    parent_hc_1shift: usize,
    parent_hc_forb: usize,
    parent_soft_pen: i32,
    child_genome: InrcGenome,
}

fn tournament_selection<'a>(
    evals: &'a [InrcEvaluation],
    k: usize,
    rng: &mut StdRng,
) -> &'a InrcEvaluation {
    let mut best: Option<&'a InrcEvaluation> = None;
    for _ in 0..k {
        let idx = rng.gen_range(0..evals.len());
        let eval = &evals[idx];
        if best.is_none() || eval.fitness() > best.unwrap().fitness() {
            best = Some(eval);
        }
    }
    best.unwrap()
}

fn genome_distance(a: &InrcGenome, b: &InrcGenome) -> usize {
    a.bits
        .iter()
        .zip(b.bits.iter())
        .filter(|(x, y)| x != y)
        .count()
}

fn main() {
    let seed = 42;
    let base_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/n030w4");

    let scenario = parse_scenario(base_dir.join("Sc-n030w4.json")).unwrap();
    let current_history = parse_history(base_dir.join("H0-n030w4-0.json")).unwrap();
    let week_data = parse_week_data(base_dir.join("WD-n030w4-0.json")).unwrap();

    let context = Arc::new(InrcContext::new(
        scenario,
        week_data,
        current_history,
        WorkforceEcology::new(),
    ));
    let evaluator = InrcOptimizer {
        context: context.clone(),
    };

    let mut rng = StdRng::seed_from_u64(seed);

    let population_size = 100;
    let generations = 500;

    let mut population = (0..population_size)
        .map(|_| evaluator.create(&mut rng))
        .collect::<Vec<_>>();
    let mut global_best: Option<InrcEvaluation> = None;

    let mut out_csv = File::create("m30_0b_passive_telemetry.csv").unwrap();
    writeln!(out_csv, "generation,parent_fitness,parent_hc_cov,parent_hc_skills,parent_hc_1shift,parent_hc_forb,parent_soft_pen,objective_delta,survived_generation,became_champion").unwrap();

    println!("Starting M30.0B Passive Telemetry Run on UltraCrew...");

    let mut pending_telemetry: Vec<TelemetryRecord> = Vec::new();

    for gen in 0..generations {
        let mut evals: Vec<InrcEvaluation> = population
            .iter()
            .map(|g| {
                evaluator.evaluate(
                    g,
                    &coralys_moga::runtime::optimization::metric::MetricReport::default(),
                )
            })
            .filter(|e| e.is_valid())
            .collect();

        if evals.is_empty() {
            population = (0..population_size)
                .map(|_| evaluator.create(&mut rng))
                .collect();
            continue;
        }

        evals.sort_by(|a, b| {
            b.fitness()
                .partial_cmp(&a.fitness())
                .unwrap_or(Ordering::Equal)
        });
        let gen_best = evals[0].clone();

        let mut new_global_best = false;
        if global_best.is_none() || gen_best.fitness() > global_best.as_ref().unwrap().fitness() {
            global_best = Some(gen_best.clone());
            new_global_best = true;
        }

        // Resolve pending telemetry from the previous generation
        for record in pending_telemetry.drain(..) {
            // Find the child in `evals` to get its fitness
            let mut child_fitness = 0.0;
            let mut found = false;
            let mut survived_generation = false;
            let mut became_champion = false;

            for (rank, eval) in evals.iter().enumerate() {
                if genome_distance(&record.child_genome, eval.genome()) == 0 {
                    child_fitness = eval.fitness();
                    found = true;
                    // Survival: Let's say top 50% survive tournament selection on average
                    if rank < population_size / 2 {
                        survived_generation = true;
                    }
                    if rank == 0 && new_global_best {
                        became_champion = true;
                    }
                    break;
                }
            }

            if found {
                let objective_delta = child_fitness - record.parent_fitness;
                writeln!(
                    out_csv,
                    "{},{},{},{},{},{},{},{},{},{}",
                    record.generation,
                    record.parent_fitness,
                    record.parent_hc_cov,
                    record.parent_hc_skills,
                    record.parent_hc_1shift,
                    record.parent_hc_forb,
                    record.parent_soft_pen,
                    objective_delta,
                    survived_generation,
                    became_champion
                )
                .unwrap();
            }
        }

        let mut next_gen = Vec::with_capacity(population_size);
        next_gen.extend(evals.iter().take(5).map(|e| e.genome().clone()));

        while next_gen.len() < population_size {
            let parent1 = tournament_selection(&evals, 3, &mut rng);
            let parent2 = tournament_selection(&evals, 3, &mut rng);

            let mut c1 = parent1.genome().clone();
            let mut c2 = parent2.genome().clone();

            if rng.gen_bool(0.8) {
                evaluator.crossover(&mut c1, &mut c2, &mut rng);
            }
            evaluator.mutate(&mut c1, &mut rng);
            evaluator.mutate(&mut c2, &mut rng);

            // Log telemetry record (awaiting evaluation in next gen)
            pending_telemetry.push(TelemetryRecord {
                generation: gen,
                parent_fitness: parent1.fitness(),
                parent_hc_cov: parent1.hc_coverage,
                parent_hc_skills: parent1.hc_skills,
                parent_hc_1shift: parent1.hc_one_shift_per_day,
                parent_hc_forb: parent1.hc_forbidden_successions,
                parent_soft_pen: parent1.soft_report.total_penalty,
                child_genome: c1.clone(),
            });

            pending_telemetry.push(TelemetryRecord {
                generation: gen,
                parent_fitness: parent2.fitness(),
                parent_hc_cov: parent2.hc_coverage,
                parent_hc_skills: parent2.hc_skills,
                parent_hc_1shift: parent2.hc_one_shift_per_day,
                parent_hc_forb: parent2.hc_forbidden_successions,
                parent_soft_pen: parent2.soft_report.total_penalty,
                child_genome: c2.clone(),
            });

            next_gen.push(c1);
            if next_gen.len() < population_size {
                next_gen.push(c2);
            }
        }
        population = next_gen;
    }

    println!("Done. See m30_0b_passive_telemetry.csv");
}
