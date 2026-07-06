use cvrp::{CvrpDecisionPlugin, CvrpInstance, CvrpGenomeFactory, CvrpState, CvrpCandidate};
use cvrp::moga_impl::{CvrpMutator, CvrpRouteAwareMutator, CvrpCrossover, CvrpCrossoverRoutePreserving};
use coralys_moga::{EvolutionConfig, MutationOperator, CrossoverOperator};
use coralys_moga::engine::EvolutionEngine;
use coralys_moga::engine::PluginFitnessEvaluator;
use coralys_core::DecisionPlugin;
use std::cmp::Ordering;

struct RunResult {
    desc: String,
    distance: f64,
}

fn evaluate_config<M, C>(
    instance: &CvrpInstance,
    plugin: &CvrpDecisionPlugin,
    state: &CvrpState,
    mutator: M,
    crossover: C,
    evo_config: EvolutionConfig,
) -> f64
where
    M: MutationOperator<CvrpCandidate> + 'static,
    C: CrossoverOperator<CvrpCandidate> + 'static,
{
    let evaluator = PluginFitnessEvaluator {
        plugin,
        state,
        _marker: std::marker::PhantomData,
    };
    let factory = CvrpGenomeFactory { num_customers: instance.customers.len() };
    let mut engine = EvolutionEngine::new(evaluator, mutator, crossover, factory);
    
    match engine.run_ga_evolution(evo_config) {
        Ok(ga_res) => {
            if let Some(&total) = ga_res.global_best.result.metrics.get("total_distance") {
                total
            } else {
                f64::MAX
            }
        }
        Err(_) => f64::MAX,
    }
}

fn main() {
    let instance = CvrpInstance::a_n32_k5();
    let plugin = CvrpDecisionPlugin::new(instance.clone());
    let state = plugin.current_state();

    println!("Starting Grid Search to find top 20 configurations...");
    
    let mut results = Vec::new();

    let pop_sizes = [100, 200, 400];
    let elite_counts = [10, 20, 40];
    let gen_limits = [50, 100, 200];
    let mut_rates = [0.1, 0.2, 0.4];
    let cross_rates = [0.7, 0.8, 0.9];
    let tourney_sizes = [3, 5, 7];

    for &pop in &pop_sizes {
        for &elite in &elite_counts {
            if elite >= pop { continue; }
            for &g_limit in &gen_limits {
                for &mut_rate in &mut_rates {
                    for &cross_rate in &cross_rates {
                        for &tourney in &tourney_sizes {
                            let evo_config = EvolutionConfig {
                                population_size: pop,
                                elite_count: elite,
                                generation_limit: g_limit,
                                mutation_rate: mut_rate,
                                crossover_rate: cross_rate,
                                seed: Some(42),
                                tournament_size: Some(tourney),
                            };

                            // Opt 1: Control + Basic Crossover
                            {
                                let mutator = CvrpMutator::new(instance.clone(), cvrp::RadiusPolicy::Control);
                                let crossover = CvrpCrossover;
                                let dist = evaluate_config(&instance, &plugin, &state, mutator, crossover, evo_config.clone());
                                if dist < 5000.0 {
                                    results.push(RunResult {
                                        desc: format!("Control + CvrpCrossover, Pop: {}, Elite: {}, Gen: {}, Mut: {}, Cross: {}, Tourney: {}", pop, elite, g_limit, mut_rate, cross_rate, tourney),
                                        distance: dist,
                                    });
                                }
                            }

                            // Opt 2: LocalBiased + Basic Crossover
                            {
                                let mutator = CvrpMutator::new(instance.clone(), cvrp::RadiusPolicy::LocalBiased);
                                let crossover = CvrpCrossover;
                                let dist = evaluate_config(&instance, &plugin, &state, mutator, crossover, evo_config.clone());
                                if dist < 5000.0 {
                                    results.push(RunResult {
                                        desc: format!("LocalBiased + CvrpCrossover, Pop: {}, Elite: {}, Gen: {}, Mut: {}, Cross: {}, Tourney: {}", pop, elite, g_limit, mut_rate, cross_rate, tourney),
                                        distance: dist,
                                    });
                                }
                            }

                            // Opt 3: RouteAware + RoutePreserving
                            {
                                let mutator = CvrpRouteAwareMutator { instance: instance.clone() };
                                let crossover = CvrpCrossoverRoutePreserving { instance: instance.clone() };
                                let dist = evaluate_config(&instance, &plugin, &state, mutator, crossover, evo_config.clone());
                                if dist < 5000.0 {
                                    results.push(RunResult {
                                        desc: format!("RouteAware + RoutePreserving, Pop: {}, Elite: {}, Gen: {}, Mut: {}, Cross: {}, Tourney: {}", pop, elite, g_limit, mut_rate, cross_rate, tourney),
                                        distance: dist,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Sort by ascending distance (best first)
    results.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap_or(Ordering::Equal));

    println!("==========================================================================");
    println!("TOP 20 BEST CONFIGURATIONS");
    println!("==========================================================================");
    for (i, r) in results.iter().take(20).enumerate() {
        println!("{:2}. {:.4} | {}", i + 1, r.distance, r.desc);
    }
    println!("==========================================================================");
}
