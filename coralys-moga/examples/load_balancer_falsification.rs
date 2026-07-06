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

#[derive(Clone)]
struct LbGenome {
    // Each task is assigned to a node (0..100). Node 100 is "Unassigned".
    assignments: Vec<usize>,
}

impl coralys_core::Solution for LbGenome {}
impl Genome for LbGenome {}

#[derive(Clone)]
struct LbEvaluator {
    num_nodes: usize,
    capacity: usize,
    tasks: Vec<usize>, // Cost of each task
}

#[derive(Clone)]
struct LbEvaluation {
    genome: LbGenome,
    hard_penalties: usize,
    unassigned_cost: usize,
    fitness: f64,
}

impl coralys_core::Outcome for LbEvaluation {
    type Sol = LbGenome;

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

impl Evaluated for LbEvaluation {
    type Genome = LbGenome;

    fn fitness(&self) -> f64 {
        self.fitness
    }

    fn is_valid(&self) -> bool {
        self.hard_penalties == 0
    }

    fn genome(&self) -> &LbGenome {
        &self.genome
    }
}

impl FitnessEvaluator<LbGenome> for LbEvaluator {
    type Evaluation = LbEvaluation;

    fn evaluate(&self, genome: &LbGenome) -> LbEvaluation {
        let mut loads = vec![0; self.num_nodes];
        let mut unassigned_cost = 0;

        for (i, &node) in genome.assignments.iter().enumerate() {
            if node < self.num_nodes {
                loads[node] += self.tasks[i];
            } else {
                unassigned_cost += self.tasks[i];
            }
        }

        let mut hard_penalties = 0;
        for &load in &loads {
            if load > self.capacity {
                hard_penalties += load - self.capacity;
            }
        }

        let fitness = -((hard_penalties as f64 * 1000.0) + unassigned_cost as f64);
        LbEvaluation {
            genome: genome.clone(),
            hard_penalties,
            unassigned_cost,
            fitness,
        }
    }
}

#[derive(Clone)]
struct NodeEcologyAdapter {
    memory: EcologyMemory<usize>,
    policy: EcologyPolicy,
}

impl NodeEcologyAdapter {
    fn new() -> Self {
        Self {
            memory: EcologyMemory::new(),
            policy: EcologyPolicy { alpha: 1.0 },
        }
    }

    fn accumulate_load(&mut self, node: usize, load: usize) {
        self.memory.accumulate(node, "cpu_load", load as f64);
    }

    fn get_load(&self, node: usize) -> f64 {
        self.memory.get_measure(node, "cpu_load")
    }

    fn compute_signal(&self, node: usize, num_nodes: usize) -> EcologySignal {
        let load = self.get_load(node);
        let mut total = 0.0;
        for n in 0..num_nodes {
            total += self.get_load(n);
        }
        if total == 0.0 {
            return EcologySignal { pressure: 0.0 };
        }
        let avg = total / num_nodes as f64;
        EcologySignal {
            pressure: (load - avg) / avg,
        }
    }
}

#[derive(Clone)]
struct LbGenomeFactory {
    num_nodes: usize,
    num_tasks: usize,
}

impl GenomeFactory<LbGenome> for LbGenomeFactory {
    fn create(&self, rng: &mut StdRng) -> LbGenome {
        let mut assignments = Vec::with_capacity(self.num_tasks);
        for _ in 0..self.num_tasks {
            // Randomly assign to a node or leave unassigned
            assignments.push(rng.gen_range(0..=self.num_nodes));
        }
        LbGenome { assignments }
    }
}

#[derive(Clone)]
struct LbMutator {
    adapter: NodeEcologyAdapter,
    num_nodes: usize,
    tasks: Vec<usize>,
    arm: Arm,
    ecology_weights: Option<Vec<f64>>,
}

impl LbMutator {
    fn pick_node(rng: &mut StdRng, weights: &[f64]) -> usize {
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

impl MutationOperator<LbGenome> for LbMutator {
    fn mutate(&self, genome: &mut LbGenome, rng: &mut StdRng) {
        let rate = 1.0 / (genome.assignments.len() as f64).max(1.0);

        let mut current_loads = vec![0.0; self.num_nodes];
        if self.arm == Arm::HorizonLocal {
            for (i, &node) in genome.assignments.iter().enumerate() {
                if node < self.num_nodes {
                    current_loads[node] += self.tasks[i] as f64;
                }
            }
        }

        for i in 0..genome.assignments.len() {
            if rng.gen_bool(rate) {
                let current_node = genome.assignments[i];
                let cost = self.tasks[i] as f64;

                let new_node = match self.arm {
                    Arm::Off => rng.gen_range(0..=self.num_nodes),
                    Arm::FullEcology => {
                        Self::pick_node(rng, self.ecology_weights.as_ref().unwrap())
                    }
                    Arm::HorizonLocal => {
                        let mut weights = vec![1.0; self.num_nodes + 1];
                        let sum: f64 = current_loads.iter().sum();
                        let avg = sum / self.num_nodes as f64;
                        for n in 0..self.num_nodes {
                            let pressure = if avg > 0.0 {
                                (current_loads[n] - avg) / avg
                            } else {
                                0.0
                            };
                            let w = 1.0 - self.adapter.policy.alpha * pressure;
                            weights[n] = w.max(0.1).min(2.0);
                        }
                        weights[self.num_nodes] = 1.0;
                        Self::pick_node(rng, &weights)
                    }
                };

                if new_node != current_node {
                    if self.arm == Arm::HorizonLocal {
                        if current_node < self.num_nodes {
                            current_loads[current_node] -= cost;
                        }
                        if new_node < self.num_nodes {
                            current_loads[new_node] += cost;
                        }
                    }
                    genome.assignments[i] = new_node;
                }
            }
        }
    }
}

#[derive(Clone)]
struct LbCrossover;

impl CrossoverOperator<LbGenome> for LbCrossover {
    fn crossover(
        &self,
        parent1: &LbGenome,
        parent2: &LbGenome,
        rng: &mut StdRng,
    ) -> (LbGenome, LbGenome) {
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
    evals: &'a [LbEvaluation],
    k: usize,
    rng: &mut StdRng,
) -> &'a LbEvaluation {
    let mut best: Option<&'a LbEvaluation> = None;
    for _ in 0..k {
        let idx = rng.gen_range(0..evals.len());
        let eval = &evals[idx];
        if best.is_none() || eval.fitness() > best.unwrap().fitness() {
            best = Some(eval);
        }
    }
    best.unwrap()
}

fn generate_tasks(rng: &mut StdRng, count: usize) -> Vec<usize> {
    (0..count).map(|_| rng.gen_range(1..=20)).collect()
}

fn run_falsification(seed: u64, arm: Arm, out_csv: &mut File) {
    let num_nodes = 100;
    let capacity = 100;
    let num_tasks = 1000;
    let num_horizons = 10;

    let mut adapter = NodeEcologyAdapter::new();

    // Create a structural historical imbalance
    for n in 0..num_nodes {
        if n < num_nodes / 2 {
            adapter.accumulate_load(n, 500);
        } else {
            adapter.accumulate_load(n, 0);
        }
    }
    let crossover = LbCrossover;
    let factory = LbGenomeFactory {
        num_nodes,
        num_tasks,
    };

    let mut rng_env = StdRng::seed_from_u64(seed);

    for h in 0..num_horizons {
        let tasks = generate_tasks(&mut rng_env, num_tasks);
        let evaluator = LbEvaluator {
            num_nodes,
            capacity,
            tasks: tasks.clone(),
        };
        let mut mutator = LbMutator {
            adapter: adapter.clone(),
            num_nodes,
            tasks: tasks.clone(),
            arm,
            ecology_weights: None,
        };

        if arm == Arm::FullEcology {
            let mut weights = vec![1.0; num_nodes + 1];
            for n in 0..num_nodes {
                let signal = adapter.compute_signal(n, num_nodes);
                let w = 1.0 - adapter.policy.alpha * signal.pressure;
                weights[n] = w.max(0.1).min(2.0);
            }
            weights[num_nodes] = 1.0;
            mutator.ecology_weights = Some(weights);
        }

        let mut rng_search = StdRng::seed_from_u64(seed + h as u64 * 1000);
        let mut population: Vec<LbGenome> =
            (0..100).map(|_| factory.create(&mut rng_search)).collect();
        let mut best_overall: Option<LbEvaluation> = None;

        for _gen in 0..200 {
            let mut evals: Vec<LbEvaluation> =
                population.iter().map(|g| evaluator.evaluate(g)).collect();

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

        // Accumulate real loads for this horizon
        let mut h_loads = vec![0; num_nodes];
        for (i, &node) in best.genome.assignments.iter().enumerate() {
            if node < num_nodes {
                h_loads[node] += tasks[i];
            }
        }
        for n in 0..num_nodes {
            adapter.accumulate_load(n, h_loads[n]);
        }

        if h == num_horizons - 1 {
            let mut total_loads = vec![0; num_nodes];
            for n in 0..num_nodes {
                total_loads[n] = adapter.get_load(n) as usize;
            }

            let gini = distribution_gini(&total_loads);
            let mean = total_loads.iter().sum::<usize>() as f64 / num_nodes as f64;
            let cv = (total_loads
                .iter()
                .map(|&x| (x as f64 - mean).powi(2))
                .sum::<f64>()
                / num_nodes as f64)
                .sqrt()
                / mean;

            let unassigned = best.unassigned_cost;
            let violations = best.hard_penalties;

            writeln!(
                out_csv,
                "{},{:?},{},{},{:.4},{:.4},{:.2}",
                seed, arm, unassigned, violations, gini, cv, mean
            )
            .unwrap();
        }
    }
}

fn main() {
    let seeds = 2000..2030; // 30 seeds
    let arms = vec![Arm::Off, Arm::HorizonLocal, Arm::FullEcology];
    let output_file = "load_balancer_falsification_30seed.csv";

    let mut file = File::create(output_file).unwrap();
    writeln!(
        file,
        "seed,arm,unassigned,violations,gini,cv,mean_utilization"
    )
    .unwrap();

    println!("F.3 Cross-Domain Falsification: Multi-Horizon Load Balancer");
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
