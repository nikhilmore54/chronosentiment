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
