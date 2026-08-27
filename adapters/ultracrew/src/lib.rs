pub mod ecology;
pub mod config;
pub mod models;
pub mod optimization;
pub mod public_contracts;
pub mod global_constructor;
pub mod inrc;
pub mod workforce;
pub mod compliance;

pub mod helpers {
    use super::ecology::WorkforceEcology;
    use super::models::{Shift, Skill, Worker};
    use super::optimization::{ScheduleContext, ScheduleOptimizer};
    use coralys_moga::config::EvolutionConfig;
    use coralys_moga::engine::{EvolutionEngine, GaResult};
    use crate::optimization::ScheduleEvaluation;
    use std::sync::Arc;

    pub fn generate_scenario(
        num_workers: usize,
        num_shifts: usize,
        pre_fatigue_workers: usize,
    ) -> Arc<ScheduleContext> {
        let mut workers = Vec::new();
        let all_skills = vec![
            Skill::new("Forklift"),
            Skill::new("GeneralLabor"),
            Skill::new("Supervisor"),
            Skill::new("FirstAid"),
        ];
        for i in 0..num_workers {
            workers.push(Worker {
                id: i as u64,
                skills: vec![
                    all_skills[i % all_skills.len()].clone(),
                    all_skills[(i + 1) % all_skills.len()].clone(),
                ],
            });
        }

        let mut shifts = Vec::new();
        for i in 0..num_shifts {
            let start = (i * 3) % 160; 
            shifts.push(Shift {
                id: i as u64,
                start_hour: start as u64,
                duration_hours: 8,
                required_skill: all_skills[i % all_skills.len()].clone(),
            });
        }

        let mut ecology = WorkforceEcology::new();
        for i in 0..pre_fatigue_workers {
            ecology.record_historical_hours(i as u64, 60.0);
            ecology.record_historical_hours(i as u64, 65.0);
        }

        Arc::new(ScheduleContext {
            workers: Arc::new(workers),
            shifts: Arc::new(shifts),
            ecology,
            rng_seed: 0,
            observatory: Arc::new(std::sync::Mutex::new(crate::optimization::Observatory::new())),
            locked_assignments: None,
            scenario: None,
            enable_fatigue: false,
            fatigue_weight: 0.0,
            hc3_aware_initialization: false,
            temporal_scarcity_construction: false,
            disable_global_constructor: false,
            precomputed_seeds: None,
        })
    }

    pub fn run_optimization(
        context: Arc<ScheduleContext>,
        config: EvolutionConfig,
    ) -> GaResult<ScheduleEvaluation> {
        let mutator = ScheduleOptimizer::new(context.clone());
        let crossover = ScheduleOptimizer::new(context.clone());
        let evaluator = ScheduleOptimizer::new(context.clone());
        
        let total_required: u64 = context.shifts.iter().map(|s| s.duration_hours).sum();
        let total_capacity: u64 = context.workers.len() as u64 * 40; // max hc3
        let utilization = total_required as f64 / total_capacity as f64;
        
        let mut seeded_factory = None;
        if !context.disable_global_constructor {
            let mut seed_opt = None;
            if let Some(ref seeds_arc) = context.precomputed_seeds {
                let seeds = seeds_arc.lock().unwrap();
                if !seeds.is_empty() {
                    seed_opt = Some(seeds[0].clone());
                }
            } else {
                seed_opt = crate::global_constructor::generate_feasible_seed(&context.shifts, &context.workers, 8, 40);
            }
            if let Some(seed) = seed_opt {
                let metrics = coralys_moga::runtime::optimization::metric::MetricReport::default();
                use coralys_moga::traits::FitnessEvaluator;
                let eval = evaluator.evaluate(&seed, &metrics);
                let hard_violations = eval.hc1_violations + eval.hc2_violations + eval.hc3_violations + eval.rest_violations;
                
                if eval.is_valid && hard_violations == 0 {
                    seeded_factory = Some(crate::optimization::SeededScheduleFactory {
                        seeds: std::cell::RefCell::new(vec![seed]),
                        base_optimizer: ScheduleOptimizer::new(context.clone()),
                    });
                }
            }
        }

        if let Some(factory) = seeded_factory {
            let mut engine = EvolutionEngine::new(evaluator, mutator, crossover, factory);
            engine.metric_engine = Some(Arc::new(crate::metrics::UltraCrewMetricEngine { context: context.clone() }));
            engine.run_ga_evolution(config).expect("Invalid EvolutionConfig – engine failed")
        } else {
            let factory = ScheduleOptimizer::new(context.clone());
            let mut engine = EvolutionEngine::new(evaluator, mutator, crossover, factory);
            engine.metric_engine = Some(Arc::new(crate::metrics::UltraCrewMetricEngine { context: context.clone() }));
            engine.run_ga_evolution(config).expect("Invalid EvolutionConfig – engine failed")
        }
    }
}


pub mod pipeline;
pub mod decision_intelligence;
pub mod schedule_solution;

pub mod constraint_engine;
pub mod recommendation;
pub mod generic_import;
pub mod generic_export;
pub mod strict_validator;
pub mod telemetry;
pub mod errors;
pub mod health;
pub mod repair;
pub mod observability;
pub mod metrics;
