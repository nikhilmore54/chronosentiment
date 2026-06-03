pub mod ecology;
pub mod models;
pub mod optimization;
pub mod inrc;

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
        let all_skills = vec![Skill::Forklift, Skill::GeneralLabor, Skill::Supervisor, Skill::FirstAid];
        for i in 0..num_workers {
            workers.push(Worker {
                id: i as u64,
                skills: vec![
                    all_skills[i % all_skills.len()],
                    all_skills[(i + 1) % all_skills.len()],
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
                required_skill: all_skills[i % all_skills.len()],
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
        })
    }

    pub fn run_optimization(
        context: Arc<ScheduleContext>,
        config: EvolutionConfig,
    ) -> GaResult<ScheduleEvaluation> {
        let factory = ScheduleOptimizer { context: context.clone() };
        let mutator = ScheduleOptimizer { context: context.clone() };
        let crossover = ScheduleOptimizer { context: context.clone() };
        let evaluator = ScheduleOptimizer { context: context.clone() };

        let engine = EvolutionEngine::new(evaluator, mutator, crossover, factory);
        engine.run_ga_evolution(config)
    }
}
