// Pipeline module for UltraCrew internal scheduling workflow

use std::sync::Arc;
use std::error::Error;
use crate::helpers::run_optimization;
use coralys_moga::config::EvolutionConfig;
use crate::optimization::ScheduleContext;
use crate::schedule_solution::ScheduleSolution;

/// Runs the complete internal scheduling pipeline:
/// 1. Execute the genetic algorithm optimization.
/// 2. Convert the best evaluation into a `ScheduleSolution`.
/// Returns the solution or an error.
pub fn run_pipeline(
    context: Arc<ScheduleContext>,
    config: EvolutionConfig,
) -> Result<ScheduleSolution, Box<dyn Error>> {
    // Initialize the observatory with the population size
    context.observatory.lock().unwrap().start_run(config.population_size);

    // Execute the optimizer
    let ga_result = run_optimization(context.clone(), config);

    // Access the best evaluation directly from the GA result.
    let best_evaluation = ga_result.global_best;

    // Build the internal solution representation from the evaluation.
    let mut solution = ScheduleSolution::from_evaluation(&best_evaluation);

    // Generate recommendations using the ConstraintEngine and RecommendationEngine
    let constraint_engine = crate::constraint_engine::ConstraintEngine::new(context.clone());
    let report = constraint_engine.evaluate(&best_evaluation.schedule);
    let recommendation_engine = crate::recommendation::RecommendationEngine::new();
    let recs = recommendation_engine.generate_recommendations(&report);
    solution.recommendations = Some(recs);

    // Populate telemetry
    let reports = context.observatory.lock().unwrap().reports.clone();
    solution.telemetry = Some(crate::optimization::OptimizationReport { generations: reports });

    Ok(solution)
}

/// Application-facing entry point that accepts tunable parameters directly,
/// building `EvolutionConfig` internally so the application layer does not
/// need to import `coralys_moga` types.
///
/// All parameters are optional; absent values fall back to `EvolutionConfig::default()`.
pub fn run_pipeline_from_request(
    context: Arc<ScheduleContext>,
    generation_limit: Option<usize>,
    tournament_size: Option<usize>,
    population_size: Option<usize>,
    mutation_rate: Option<f64>,
    crossover_rate: Option<f64>,
    elite_count: Option<usize>,
) -> Result<ScheduleSolution, Box<dyn Error>> {
    let mut config = EvolutionConfig::default();
    if let Some(v) = generation_limit { config.generation_limit = v; }
    if let Some(v) = tournament_size  { config.tournament_size = Some(v); }
    if let Some(v) = population_size  { config.population_size = v; }
    if let Some(v) = mutation_rate    { config.mutation_rate = v; }
    if let Some(v) = crossover_rate   { config.crossover_rate = v; }
    if let Some(v) = elite_count      { config.elite_count = v; }
    run_pipeline(context, config)
}

/// Runs the INRC startup pipeline: loads a scenario from `scenario_path` and
/// week data from `week_data_path`, seeds the Pareto engine with a baseline
/// schedule, runs `steps` evolution steps, and returns the best schedule plus
/// the full Pareto frontier as application-facing types.
///
/// This function encapsulates all `coralys_moga::engine_proof` usage so the
/// application layer does not need to import platform crates directly.
pub fn run_inrc_startup_pipeline(
    scenario_path: &std::path::Path,
    week_data_path: &std::path::Path,
    steps: usize,
) -> Result<crate::public_contracts::InrcStartupResult, Box<dyn Error>> {
    use crate::inrc::parser::{parse_scenario, parse_week_data};
    use crate::inrc::schedule_optimizer::{ScheduleGenome, UltraCrewEvaluator, UltraCrewMutator};
    use crate::inrc::baseline::generate_baseline_schedule;
    use coralys_moga::engine_proof::EvolutionEngine;
    use crate::public_contracts::{InrcParetoSolution, InrcStartupResult};

    let scenario = parse_scenario(scenario_path)?;
    let week_data = parse_week_data(week_data_path)?;
    let num_days = (scenario.number_of_weeks * 7) as usize;

    let baseline_genome = generate_baseline_schedule(&scenario, &week_data.requirements)
        .unwrap_or_else(|_| ScheduleGenome {
            slots: Vec::new(),
            num_days,
            nurses: scenario.nurses.iter().map(|n| n.id.clone()).collect(),
        });

    let evaluator = UltraCrewEvaluator { scenario: scenario.clone() };
    let mutator = UltraCrewMutator::new(scenario.clone());

    let mut engine = EvolutionEngine::new(evaluator, mutator);
    let fallback_schedule = baseline_genome.to_flat_schedule();
    engine.seed(baseline_genome);

    for _ in 0..steps {
        engine.step();
    }

    let pareto_solutions: Vec<InrcParetoSolution> = engine.archive.solutions.iter().map(|sol| {
        InrcParetoSolution {
            s6_assignment_penalty:  sol.fitness.get(0).copied().unwrap_or(0.0),
            s7_weekend_penalty:     sol.fitness.get(1).copied().unwrap_or(0.0),
            recovery_penalty:       sol.fitness.get(2).copied().unwrap_or(0.0),
            workload_balance:       sol.fitness.get(3).copied().unwrap_or(0.0),
            temporal_load_balance:  sol.fitness.get(4).copied().unwrap_or(0.0),
            schedule:               sol.genome.to_flat_schedule(),
        }
    }).collect();

    let schedule = if !engine.archive.solutions.is_empty() {
        engine.archive.solutions[0].genome.to_flat_schedule()
    } else {
        fallback_schedule
    };

    Ok(InrcStartupResult { schedule, pareto_solutions })
}
