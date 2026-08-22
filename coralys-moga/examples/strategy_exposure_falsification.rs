use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::cmp::Ordering;
use std::fs::File;
use std::io::Write;
use std::time::Instant;

use coralys_moga::ecology::{EcologyMemory, EcologyPolicy, EcologySignal, distribution_gini};
use coralys_moga::traits::{
    CrossoverOperator, Evaluated, FitnessEvaluator, Genome, GenomeFactory, MutationOperator,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    Off,
    HorizonLocal,
    FullEcology,
}

#[derive(Clone, Debug)]
struct Opportunity {
    capital: usize,
    expected_return: f64,
    risk_multiplier: f64,
}

#[derive(Clone)]
struct StrategyGenome {
    // Each opportunity is assigned to a strategy (0..50). Node 50 is "Unassigned".
    assignments: Vec<usize>,
}

impl coralys_core::Solution for StrategyGenome {}
impl Genome for StrategyGenome {}

#[derive(Clone)]
struct StrategyEvaluator {
    num_strategies: usize,
    capital_limit: usize,
    opportunities: Vec<Opportunity>,
}

#[derive(Clone)]
struct StrategyEvaluation {
    genome: StrategyGenome,
    hard_penalties: usize,
    total_expected_return: f64,
    fitness: f64,
}

impl coralys_core::Outcome for StrategyEvaluation {
    type Sol = StrategyGenome;

    fn objectives(&self) -> &[f64] {
        std::slice::from_ref(&self.fitness)
    }

    fn is_valid(&self) -> bool {
        self.hard_penalties == 0
    }

    fn solution(&self) -> &Self::Sol {
        &self.genome
    }
}

impl Evaluated for StrategyEvaluation {
    type Genome = StrategyGenome;

    fn fitness(&self) -> f64 {
        self.fitness
    }

    fn is_valid(&self) -> bool {
        self.hard_penalties == 0
    }

    fn genome(&self) -> &StrategyGenome {
        &self.genome
    }
}

impl FitnessEvaluator<StrategyGenome> for StrategyEvaluator {
    type Evaluation = StrategyEvaluation;

    fn evaluate(&self, genome: &StrategyGenome, _metrics: &coralys_moga::runtime::optimization::metric::MetricReport) -> StrategyEvaluation {
        let mut capital_allocated = vec![0; self.num_strategies];
        let mut total_expected_return = 0.0;

        for (i, &strategy) in genome.assignments.iter().enumerate() {
            if strategy < self.num_strategies {
                capital_allocated[strategy] += self.opportunities[i].capital;
                total_expected_return += self.opportunities[i].expected_return;
            }
        }

        let mut hard_penalties = 0;
        for &cap in &capital_allocated {
            if cap > self.capital_limit {
                hard_penalties += cap - self.capital_limit;
            }
        }

        let fitness = total_expected_return - (hard_penalties as f64 * 10000.0);
        StrategyEvaluation {
            genome: genome.clone(),
            hard_penalties,
            total_expected_return,
            fitness,
        }
    }
}

#[derive(Clone)]
struct StrategyEcologyAdapter {
    memory: EcologyMemory<usize>,
    policy: EcologyPolicy,
}

impl StrategyEcologyAdapter {
    fn new() -> Self {
        Self {
            memory: EcologyMemory::new(),
            policy: EcologyPolicy { alpha: 1.0 },
        }
    }

    fn accumulate_exposure(&mut self, strategy: usize, exposure: f64) {
        self.memory.accumulate(strategy, "risk_exposure", exposure);
    }

    fn get_exposure(&self, strategy: usize) -> f64 {
        self.memory.get_measure(strategy, "risk_exposure")
    }

    fn compute_signal(&self, strategy: usize, num_strategies: usize) -> EcologySignal {
        let exposure = self.get_exposure(strategy);
        let mut total = 0.0;
        for n in 0..num_strategies {
            total += self.get_exposure(n);
        }
        if total == 0.0 {
            return EcologySignal { pressure: 0.0 };
        }
        let avg = total / num_strategies as f64;
        EcologySignal {
            pressure: (exposure - avg) / avg,
        }
    }
}

#[derive(Clone)]
struct StrategyGenomeFactory {
    num_strategies: usize,
    num_opportunities: usize,
}

impl GenomeFactory<StrategyGenome> for StrategyGenomeFactory {
    fn create(&self, rng: &mut StdRng) -> StrategyGenome {
        let mut assignments = Vec::with_capacity(self.num_opportunities);
        for _ in 0..self.num_opportunities {
            assignments.push(rng.gen_range(0..=self.num_strategies));
        }
        StrategyGenome { assignments }
    }
}

#[derive(Clone)]
struct StrategyMutator {
    adapter: StrategyEcologyAdapter,
    num_strategies: usize,
    opportunities: Vec<Opportunity>,
    arm: Arm,
    ecology_weights: Option<Vec<f64>>,
}

impl StrategyMutator {
    fn pick_strategy(rng: &mut StdRng, weights: &[f64]) -> usize {
        let total: f64 = weights.iter().sum();
        if total <= 0.0 {
            return rng.gen_range(0..weights.len());
        }
        let mut val = rng.gen_range(0.0..total);
        for (i, &w) in weights.iter().enumerate() {
            if val < w {
                return i;
            }
            val -= w;
        }
        weights.len() - 1
    }
}

impl MutationOperator<StrategyGenome> for StrategyMutator {
    fn mutate(&self, genome: &mut StrategyGenome, rng: &mut StdRng) {
        let rate = 1.0 / (genome.assignments.len() as f64).max(1.0);

        let mut current_exposures = vec![0.0; self.num_strategies];
        if self.arm == Arm::HorizonLocal {
            for (i, &strategy) in genome.assignments.iter().enumerate() {
                if strategy < self.num_strategies {
                    let opp = &self.opportunities[i];
                    current_exposures[strategy] += opp.capital as f64 * opp.risk_multiplier;
                }
            }
        }

        for i in 0..genome.assignments.len() {
            if rng.gen_bool(rate) {
                let current_strategy = genome.assignments[i];
                let opp = &self.opportunities[i];
                let exposure = opp.capital as f64 * opp.risk_multiplier;

                let new_strategy = match self.arm {
                    Arm::Off => rng.gen_range(0..=self.num_strategies),
                    Arm::FullEcology => {
                        Self::pick_strategy(rng, self.ecology_weights.as_ref().unwrap())
                    }
                    Arm::HorizonLocal => {
                        let mut weights = vec![1.0; self.num_strategies + 1];
                        let sum: f64 = current_exposures.iter().sum();
                        let avg = sum / self.num_strategies as f64;
                        for n in 0..self.num_strategies {
                            let pressure = if avg > 0.0 {
                                (current_exposures[n] - avg) / avg
                            } else {
                                0.0
                            };
                            let w = 1.0 - self.adapter.policy.alpha * pressure;
                            weights[n] = w.max(0.1).min(2.0);
                        }
                        weights[self.num_strategies] = 1.0;
                        Self::pick_strategy(rng, &weights)
                    }
                };

                if new_strategy != current_strategy {
                    if self.arm == Arm::HorizonLocal {
                        if current_strategy < self.num_strategies {
                            current_exposures[current_strategy] -= exposure;
                        }
                        if new_strategy < self.num_strategies {
                            current_exposures[new_strategy] += exposure;
                        }
                    }
                    genome.assignments[i] = new_strategy;
                }
            }
        }
    }
}

#[derive(Clone)]
struct StrategyCrossover;

impl CrossoverOperator<StrategyGenome> for StrategyCrossover {
    fn crossover(
        &self,
        parent1: &StrategyGenome,
        parent2: &StrategyGenome,
        rng: &mut StdRng,
    ) -> (StrategyGenome, StrategyGenome) {
        let pt = rng.gen_range(0..parent1.assignments.len());
        let mut c1 = parent1.clone();
        let mut c2 = parent2.clone();
        for i in pt..parent1.assignments.len() {
            std::mem::swap(&mut c1.assignments[i], &mut c2.assignments[i]);
        }
        (c1, c2)
    }
}

fn tournament_selection<'a>(
    evals: &'a [StrategyEvaluation],
    k: usize,
    rng: &mut StdRng,
) -> &'a StrategyEvaluation {
    let mut best: Option<&'a StrategyEvaluation> = None;
    for _ in 0..k {
        let idx = rng.gen_range(0..evals.len());
        let eval = &evals[idx];
        if best.is_none() || eval.fitness() > best.unwrap().fitness() {
            best = Some(eval);
        }
    }
    best.unwrap()
}

fn generate_opportunities(rng: &mut StdRng, count: usize) -> Vec<Opportunity> {
    (0..count)
        .map(|_| Opportunity {
            capital: rng.gen_range(10..=100),
            expected_return: rng.gen_range(5.0..=50.0),
            risk_multiplier: rng.gen_range(1.0..=5.0),
        })
        .collect()
}

fn run_falsification(seed: u64, arm: Arm, out_csv: &mut File) {
    let num_strategies = 50;
    let num_opportunities = 1000;
    let num_horizons = 10;

    // Average capital per opp = 55. Total capital = 55,000.
    // 50 strategies. Average capital per strategy = 1,100.
    // We set capital_limit to 1200 to create some packing pressure, requiring optimization.
    let capital_limit = 1200;

    let mut adapter = StrategyEcologyAdapter::new();

    // Create a structural historical exposure imbalance
    for n in 0..num_strategies {
        if n < num_strategies / 2 {
            adapter.accumulate_exposure(n, 10000.0);
        } else {
            adapter.accumulate_exposure(n, 0.0);
        }
    }

    let crossover = StrategyCrossover;
    let factory = StrategyGenomeFactory {
        num_strategies,
        num_opportunities,
    };

    let mut rng_env = StdRng::seed_from_u64(seed);

    for h in 0..num_horizons {
        let opportunities = generate_opportunities(&mut rng_env, num_opportunities);
        let evaluator = StrategyEvaluator {
            num_strategies,
            capital_limit,
            opportunities: opportunities.clone(),
        };

        let mut mutator = StrategyMutator {
            adapter: adapter.clone(),
            num_strategies,
            opportunities: opportunities.clone(),
            arm,
            ecology_weights: None,
        };

        if arm == Arm::FullEcology {
            let mut weights = vec![1.0; num_strategies + 1];
            for n in 0..num_strategies {
                let signal = adapter.compute_signal(n, num_strategies);
                let w = 1.0 - adapter.policy.alpha * signal.pressure;
                weights[n] = w.max(0.1).min(2.0);
            }
            weights[num_strategies] = 1.0;
            mutator.ecology_weights = Some(weights);
        }

        let mut rng_search = StdRng::seed_from_u64(seed + h as u64 * 1000);
        let mut population: Vec<StrategyGenome> =
            (0..100).map(|_| factory.create(&mut rng_search)).collect();
        let mut best_overall: Option<StrategyEvaluation> = None;

        for _gen in 0..200 {
            let mut evals: Vec<StrategyEvaluation> =
                population.iter().map(|g| evaluator.evaluate(g, &coralys_moga::runtime::optimization::metric::MetricReport::default())).collect();

            evals.sort_by(|a, b| {
                b.fitness()
                    .partial_cmp(&a.fitness())
                    .unwrap_or(Ordering::Equal)
            });
            let gen_best = evals[0].clone();

            if best_overall.is_none()
                || gen_best.fitness() > best_overall.as_ref().unwrap().fitness()
            {
                best_overall = Some(gen_best.clone());
            }

            let mut next_gen = Vec::with_capacity(100);
            next_gen.extend(evals.iter().take(5).map(|e| e.genome().clone()));

            while next_gen.len() < 100 {
                let p1 = tournament_selection(&evals, 3, &mut rng_search);
                let p2 = tournament_selection(&evals, 3, &mut rng_search);
                let mut c1 = p1.genome().clone();
                let mut c2 = p2.genome().clone();
                if rng_search.gen_bool(0.8) {
                    let (new_c1, new_c2) = crossover.crossover(&c1, &c2, &mut rng_search);
                    c1 = new_c1;
                    c2 = new_c2;
                }
                mutator.mutate(&mut c1, &mut rng_search);
                mutator.mutate(&mut c2, &mut rng_search);
                next_gen.push(c1);
                if next_gen.len() < 100 {
                    next_gen.push(c2);
                }
            }
            population = next_gen;
        }

        let best = best_overall.unwrap();

        // Accumulate real exposures for this horizon
        let mut h_exposures = vec![0.0; num_strategies];
        for (i, &strategy) in best.genome.assignments.iter().enumerate() {
            if strategy < num_strategies {
                let opp = &opportunities[i];
                h_exposures[strategy] += opp.capital as f64 * opp.risk_multiplier;
            }
        }
        for n in 0..num_strategies {
            adapter.accumulate_exposure(n, h_exposures[n]);
        }

        if h == num_horizons - 1 {
            let mut total_exposures = vec![0.0; num_strategies];
            for n in 0..num_strategies {
                total_exposures[n] = adapter.get_exposure(n);
            }

            // Re-implement distribution_gini for f64 manually since the ecology module assumes integers/usizes currently
            total_exposures.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
            let n = total_exposures.len() as f64;
            let mut num = 0.0;
            let mut den = 0.0;
            for (i, &val) in total_exposures.iter().enumerate() {
                num += (i as f64 + 1.0) * val;
                den += val;
            }
            let gini = if den > 0.0 {
                (2.0 * num) / (n * den) - (n + 1.0) / n
            } else {
                0.0
            };

            let total_return = best.total_expected_return;
            let violations = best.hard_penalties;

            // Calculate unassigned capital just for tracking
            let mut unassigned_capital = 0;
            for (i, &strategy) in best.genome.assignments.iter().enumerate() {
                if strategy == num_strategies {
                    unassigned_capital += opportunities[i].capital;
                }
            }

            writeln!(
                out_csv,
                "{},{:?},{},{},{:.4},{:.2}",
                seed, arm, unassigned_capital, violations, gini, total_return
            )
            .unwrap();
        }
    }
}

fn main() {
    let seeds = 2000..2030; // 30 seeds
    let arms = vec![Arm::Off, Arm::HorizonLocal, Arm::FullEcology];
    let output_file = "strategy_exposure_falsification_30seed.csv";

    let mut file = File::create(output_file).unwrap();
    writeln!(
        file,
        "seed,arm,unassigned_capital,violations,gini,total_return"
    )
    .unwrap();

    println!("F.3B Strategy Exposure Balancing Falsification");
    println!("  Output: {}", output_file);
    println!();

    for seed in seeds {
        for &arm in &arms {
            let start = Instant::now();
            run_falsification(seed, arm, &mut file);
            let elapsed = start.elapsed();
            println!(
                "  Seed {} Arm {:?} completed in {:.1}s",
                seed,
                arm,
                elapsed.as_secs_f64()
            );
        }
    }

    println!("\nFalsification test completed successfully.");
}
