use coralys_moga::runtime::optimization::metric::{MetricEngine, MetricReport, MetricValue};
use crate::optimization::{ScheduleGenome, ScheduleContext};
use std::sync::Arc;
use std::collections::HashMap;

pub struct UltraCrewMetricEngine {
    pub context: Arc<ScheduleContext>,
}

impl MetricEngine<ScheduleGenome> for UltraCrewMetricEngine {
    fn evaluate(&self, genome: &ScheduleGenome) -> MetricReport {
        let mut metrics = HashMap::new();
        
        let min_rest = self.context.scenario
            .as_ref()
            .and_then(|s| s.minimum_rest_hours)
            .unwrap_or(10); 
            
        let mut min_rest_margin = f64::MAX;
        
        let mut worker_shifts: HashMap<u64, Vec<&crate::models::Shift>> = HashMap::new();
        for shift in self.context.shifts.iter() {
            if let Some(worker_id) = genome.assignments.get(&shift.id) {
                worker_shifts.entry(*worker_id).or_default().push(shift);
            }
        }
        
        let mut skill_coverage = 0;
        let total_shifts = self.context.shifts.len();

        for (worker_id, shifts) in &worker_shifts {
            let mut sorted_shifts = shifts.clone();
            sorted_shifts.sort_by_key(|s| s.start_hour);
            
            for i in 0..sorted_shifts.len().saturating_sub(1) {
                let s_i = sorted_shifts[i];
                let s_next = sorted_shifts[i + 1];
                let gap = if s_next.start_hour >= s_i.end_hour() {
                    s_next.start_hour - s_i.end_hour()
                } else { 0 };
                
                let margin = (gap as f64) - (min_rest as f64);
                if margin < min_rest_margin {
                    min_rest_margin = margin;
                }
            }
        }
        
        for shift in self.context.shifts.iter() {
            if let Some(worker_id) = genome.assignments.get(&shift.id) {
                if let Some(worker) = self.context.workers.iter().find(|w| w.id == *worker_id) {
                    if worker.skills.contains(&shift.required_skill) {
                        skill_coverage += 1;
                    }
                }
            }
        }

        if min_rest_margin == f64::MAX {
            min_rest_margin = 0.0;
        }

        metrics.insert("rest_margin".to_string(), MetricValue::Float(min_rest_margin));
        metrics.insert("skill_coverage".to_string(), MetricValue::Float(skill_coverage as f64 / total_shifts.max(1) as f64));
        
        MetricReport { metrics }
    }
}
