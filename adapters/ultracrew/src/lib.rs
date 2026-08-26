pub mod ecology;
pub mod config;
pub mod models;
pub mod optimization;
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
        })
    }

    pub fn run_optimization(
        context: Arc<ScheduleContext>,
        config: EvolutionConfig,
    ) -> GaResult<ScheduleEvaluation> {
        let factory = ScheduleOptimizer::new(context.clone());
        let mutator = ScheduleOptimizer::new(context.clone());
        let crossover = ScheduleOptimizer::new(context.clone());
        let evaluator = ScheduleOptimizer::new(context.clone());

        let mut engine = EvolutionEngine::new(evaluator, mutator, crossover, factory);
        engine.metric_engine = Some(Arc::new(crate::metrics::UltraCrewMetricEngine { context: context.clone() }));
        
        // let mut satisfaction_engine = coralys_moga::runtime::optimization::satisfaction::DefaultRepairEngine::new(
        //     coralys_moga::runtime::optimization::constraint::ConstraintSatisfactionConfig::default()
        // ).with_metric_engine(Arc::new(crate::metrics::UltraCrewMetricEngine { context: context.clone() }));
        // satisfaction_engine.add_model(Box::new(crate::repair::RestConstraint { context: context.clone() }));
        // satisfaction_engine.add_model(Box::new(crate::repair::SkillConstraint { context: context.clone() }));
        // satisfaction_engine.add_operator(Box::new(crate::repair::ReassignRepairOperator { context: context.clone() }));
        // engine.satisfaction_engine = Some(Box::new(satisfaction_engine));

        // Temporary compatibility shim.
        // EvolutionEngine now returns Result to surface configuration validation errors.
        // For the demo/pilot we unwrap here because configs are generated internally.
        // TODO: Propagate this Result through the UltraCrew API when production‑ready.
        engine.run_ga_evolution(config)
            .expect("Invalid EvolutionConfig – engine failed")
    }
}


pub mod pipeline;
pub mod decision_intelligence;
pub mod schedule_solution;
pub mod public_contracts;
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
