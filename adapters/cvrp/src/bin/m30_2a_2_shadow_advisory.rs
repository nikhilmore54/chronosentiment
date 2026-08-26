use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use coralys_moga::traits::*;
use cvrp::moga_impl::{CvrpCrossoverRoutePreserving, CvrpEvaluator, CvrpRouteAwareMutator};
use cvrp::{CvrpCandidate, CvrpEvaluation, CvrpGenomeFactory, CvrpInstance};

#[derive(Clone, Hash, Eq, PartialEq)]
struct CvrpContextKey {
    route_count: usize,
    longest_route: usize,
    route_balance: i64,
}

impl CvrpContextKey {
    fn from_eval(eval: &CvrpEvaluation) -> Self {
        Self {
            route_count: eval.num_vehicles,
            longest_route: eval.routes.iter().map(|r| r.len()).max().unwrap_or(0),
            route_balance: compute_route_balance(&eval.routes),
        }
    }
}

fn compute_route_balance(routes: &[Vec<usize>]) -> i64 {
    if routes.is_empty() {
        return 0;
    }
    let mean = routes.iter().map(|r| r.len() as f64).sum::<f64>() / routes.len() as f64;
    let var = routes
        .iter()
        .map(|r| (r.len() as f64 - mean).powi(2))
        .sum::<f64>()
        / routes.len() as f64;
    var.sqrt().round() as i64
}

struct MemoryStats {
    obs: usize,
    champs: usize,
}

struct OpportunityMemory {
    map: HashMap<CvrpContextKey, MemoryStats>,
    alpha: f64,
    beta: f64,
}

impl OpportunityMemory {
    fn new(prior_weight: f64, global_rate: f64) -> Self {
        Self {
            map: HashMap::new(),
            alpha: global_rate * prior_weight,
            beta: (1.0 - global_rate) * prior_weight,
        }
    }

    fn score(&self, key: &CvrpContextKey) -> f64 {
        if let Some(stats) = self.map.get(key) {
            (stats.champs as f64 + self.alpha) / (stats.obs as f64 + self.alpha + self.beta)
        } else {
            self.alpha / (self.alpha + self.beta)
        }
    }

    fn record(&mut self, key: CvrpContextKey, is_champ: bool) {
        let stats = self
            .map
            .entry(key)
            .or_insert(MemoryStats { obs: 0, champs: 0 });
        stats.obs += 1;
        if is_champ {
            stats.champs += 1;
        }
    }
}

struct Offspring {
    genome: CvrpCandidate,
    parent_ctx: Option<CvrpContextKey>,
}

struct EvaluatedOffspring {
    eval: CvrpEvaluation,
    parent_ctx: Option<CvrpContextKey>,
}

fn tournament_selection<'a>(
    evals: &'a [EvaluatedOffspring],
    k: usize,
    rng: &mut StdRng,
) -> &'a EvaluatedOffspring {
    let mut best: Option<&'a EvaluatedOffspring> = None;
    for _ in 0..k {
        let idx = rng.gen_range(0..evals.len());
        let e = &evals[idx];
        if best.is_none() || e.eval.total_distance < best.unwrap().eval.total_distance {
            best = Some(e);
        }
    }
    best.unwrap()
}

fn run_shadow(seed: u64, out_csv: &mut File) {
    let instance = CvrpInstance::a_n32_k5();
    let evaluator = CvrpEvaluator {
        instance: instance.clone(),
    };
    let mutator = CvrpRouteAwareMutator {
        instance: instance.clone(),
    };
    let crossover = CvrpCrossoverRoutePreserving {
        instance: instance.clone(),
    };
    let factory = CvrpGenomeFactory {
        num_customers: instance.customers.len(),
    };

    let mut rng = StdRng::seed_from_u64(seed);

    let population_size = 100;
    let generations = 500;

    // Global Champion Rate ~0.00088
    let mut memory = OpportunityMemory::new(500.0, 0.00088);

    let mut population: Vec<Offspring> = (0..population_size)
        .map(|_| Offspring {
            genome: factory.create(&mut rng),
            parent_ctx: None,
        })
        .collect();

    let mut global_best: Option<CvrpEvaluation> = None;

    for generation in 0..generations {
        let mut evals: Vec<EvaluatedOffspring> = population
            .drain(..)
            .map(|off| EvaluatedOffspring {
                eval: evaluator
                    .evaluate(
                        &off.genome,
                        &coralys_moga::runtime::optimization::metric::MetricReport::default(),
                    )
                    .eval,
                parent_ctx: off.parent_ctx,
            })
            .collect();

        // Control ranking: always purely by fitness
        evals.sort_by(|a, b| {
            a.eval
                .total_distance
                .partial_cmp(&b.eval.total_distance)
                .unwrap_or(Ordering::Equal)
        });
        let gen_best = evals[0].eval.clone();

        let mut new_global_best = false;
        if global_best.is_none()
            || gen_best.total_distance < global_best.as_ref().unwrap().total_distance
        {
            global_best = Some(gen_best.clone());
            new_global_best = true;
        }

        // Shadow telemetry
        for (rank, e) in evals.iter().enumerate() {
            let is_champ = rank == 0 && new_global_best;
            let survived = rank < population_size / 2;

            if let Some(ctx) = &e.parent_ctx {
                let score = memory.score(ctx);
                let ctx_str = format!(
                    "{}-{}-{}",
                    ctx.route_count, ctx.longest_route, ctx.route_balance
                );
                if generation >= 100 {
                    writeln!(
                        out_csv,
                        "{},{},{},{},{},{}",
                        seed, generation, ctx_str, score, is_champ, survived
                    )
                    .unwrap();
                }
                memory.record(ctx.clone(), is_champ);
            }
        }

        let mut next_gen = Vec::with_capacity(population_size);
        for i in 0..std::cmp::min(5, evals.len()) {
            next_gen.push(Offspring {
                genome: evals[i].eval.candidate.clone(),
                parent_ctx: evals[i].parent_ctx.clone(),
            });
        }

        while next_gen.len() < population_size {
            let p1 = tournament_selection(&evals, 3, &mut rng);
            let p2 = tournament_selection(&evals, 3, &mut rng);

            let (mut c1, mut c2) = if rng.gen_bool(0.8) {
                crossover.crossover(&p1.eval.candidate, &p2.eval.candidate, &mut rng)
            } else {
                (p1.eval.candidate.clone(), p2.eval.candidate.clone())
            };

            mutator.mutate(&mut c1, &mut rng);
            mutator.mutate(&mut c2, &mut rng);

            next_gen.push(Offspring {
                genome: c1,
                parent_ctx: Some(CvrpContextKey::from_eval(&p1.eval)),
            });
            if next_gen.len() < population_size {
                next_gen.push(Offspring {
                    genome: c2,
                    parent_ctx: Some(CvrpContextKey::from_eval(&p2.eval)),
                });
            }
        }
        population = next_gen;
    }
}

fn main() {
    println!("Starting M30.2A.2 CVRP Shadow Advisory (10 Seeds, Context D)...");

    let mut out_csv = File::create("m30_2a_2_shadow_advisory.csv").unwrap();
    writeln!(
        out_csv,
        "seed,generation,parent_context,parent_score,became_champion,survived"
    )
    .unwrap();

    let seeds = vec![42, 100, 256, 512, 1024, 2048, 4096, 8192, 12345, 99999];

    for &seed in &seeds {
        println!("Running seed {}...", seed);
        run_shadow(seed, &mut out_csv);
    }

    println!("Done. Written to m30_2a_2_shadow_advisory.csv");
}
