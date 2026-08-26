use chronosentiment_optimization::{
    Candidate, CandidateEvaluation, FitnessEvaluator, GaConfig, crossover, initialize_population,
    mutate_candidate, tournament_selection,
};
use rand::SeedableRng;
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};

const MAX_GENERATIONS: usize = 1000;
const POPULATION_SIZE: usize = 50;

#[derive(Serialize)]
struct SeedProgressReport {
    seed: u64,
    final_score: f64,
    progress_at_100: coralys_ecology::progress::ProgressObservation,
}

struct SyntheticEvaluator;

impl FitnessEvaluator<Candidate> for SyntheticEvaluator {
    type Evaluation = CandidateEvaluation;

    fn evaluate(&self, candidate: &Candidate) -> CandidateEvaluation {
        // Normalize variables to [-2.0, 2.0] based on their typical ranges
        let x = (candidate.queue_threshold as f64 / 2500.0) - 2.0;
        let y = (candidate.take_profit as f64 / 250.0) - 2.0;

        // Rosenbrock function: f(x,y) = (1-x)^2 + 100(y-x^2)^2
        let rosenbrock = (1.0 - x).powi(2) + 100.0 * (y - x.powi(2)).powi(2);

        // Map to [0, 1] range: e^(-rosenbrock / 100)
        let fitness = (-rosenbrock / 100.0).exp();

        let mut eval = CandidateEvaluation::default();
        eval.candidate = candidate.clone();
        eval.fitness = fitness;
        eval.evaluation_valid = true;
        eval.win_rate = fitness * 0.7;
        eval.payoff = 1.2 + fitness * 0.5;
        eval
    }
}

fn win_rate_proxy(c: &Candidate) -> f64 {
    let raw = c.selectivity as f64 / 100.0 * c.edge_ratio as f64 / 250.0;
    raw.min(1.0)
}

fn main() {
    let seeds: Vec<u64> = (1..=3).collect();

    println!("=== M8G: ChronoSentiment Progress Validation ===");
    println!("Generations : {}", MAX_GENERATIONS);
    println!("Seeds       : {} total", seeds.len());
    println!();

    let mut all_reports: Vec<SeedProgressReport> = Vec::new();

    let evaluator = SyntheticEvaluator;
    let config = GaConfig {
        population_size: POPULATION_SIZE,
        generations: MAX_GENERATIONS,
        crossover_rate: 0.8,
        mutation_rate: 0.1,
        seed: 0,
    };

    for seed in &seeds {
        println!("--- Running seed {} ---", seed);

        let mut rng = StdRng::seed_from_u64(*seed);
        let mut population = initialize_population(&config, &mut rng);

        let mut progress_tracker = coralys_ecology::progress::ProgressTracker::new();
        let mut progress_at_100 = None;
        let mut final_score = 0.0;

        let mut history_log = Vec::new();

        for g in 1..=MAX_GENERATIONS {
            let mut evaluations: Vec<CandidateEvaluation> =
                population.iter().map(|c| evaluator.evaluate(c)).collect();

            evaluations.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());
            let global_best_fitness = evaluations[0].fitness;

            // Approximate diversity: std dev of fitness
            let mean_fit =
                evaluations.iter().map(|e| e.fitness).sum::<f64>() / POPULATION_SIZE as f64;
            let div = (evaluations
                .iter()
                .map(|e| (e.fitness - mean_fit).powi(2))
                .sum::<f64>()
                / POPULATION_SIZE as f64)
                .sqrt();

            if g <= 100 || g % 100 == 0 {
                history_log.push(format!(
                    "Gen {:>4} | Best: {:.4} | Div: {:.6}",
                    g, global_best_fitness, div
                ));
            }

            let progress_telemetry = progress_tracker.observe_maximization(g, global_best_fitness);

            if g == 100 {
                progress_at_100 = Some(progress_telemetry);
            }
            if g == MAX_GENERATIONS {
                final_score = global_best_fitness;
            }

            let mut next_population = Vec::with_capacity(POPULATION_SIZE);
            for i in 0..2 {
                next_population.push(evaluations[i].candidate.clone());
            }
            while next_population.len() < POPULATION_SIZE {
                let p1 = tournament_selection(&evaluations, 3, &mut rng);
                let p2 = tournament_selection(&evaluations, 3, &mut rng);
                let mut child = crossover(&p1.candidate, &p2.candidate, &mut rng);
                mutate_candidate(&mut child, &mut rng, 1.0);
                next_population.push(child);
            }
            population = next_population;
        }

        println!("--- Seed {} Audit Log ---", seed);
        for line in &history_log {
            println!("{}", line);
        }

        all_reports.push(SeedProgressReport {
            seed: *seed,
            final_score,
            progress_at_100: progress_at_100.unwrap(),
        });
    }

    let json = serde_json::to_string_pretty(&all_reports).unwrap();
    let filename = "m8g_cs_report.json";
    std::fs::write(&filename, &json).expect("Failed to write seed JSON");
    println!("Wrote report to {}", filename);
}
