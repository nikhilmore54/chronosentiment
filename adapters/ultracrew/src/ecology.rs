use coralys_ecology::models::{MemoryState, CognitionGeometry};
use std::collections::HashMap;

/// Provides historical context (Fatigue) to the MOGA Engine using Coralys Ecology
#[derive(Debug, Clone)]
pub struct WorkforceEcology {
    // Maps worker_id to their MemoryState of past workload
    worker_history: HashMap<u64, MemoryState>,
}

impl WorkforceEcology {
    pub fn new() -> Self {
        Self {
            worker_history: HashMap::new(),
        }
    }

    pub fn record_historical_hours(&mut self, worker_id: u64, hours: f64) {
        let memory = self.worker_history.entry(worker_id).or_insert_with(|| {
            MemoryState::new(CognitionGeometry::RollingBounded { window: 4 })
        });
        
        memory.buffer.push(hours);
        if hours > memory.running_max {
            memory.running_max = hours;
        }
        if memory.buffer.len() > 4 { // Keep last 4 scheduling windows
            memory.buffer.remove(0);
        }
    }

    pub fn get_historical_fatigue(&self, worker_id: u64) -> f64 {
        if let Some(mem) = self.worker_history.get(&worker_id) {
            if mem.buffer.is_empty() {
                return 0.0;
            }
            // Simple moving average of historical hours in the buffer
            let sum: f64 = mem.buffer.iter().sum();
            sum / mem.buffer.len() as f64
        } else {
            0.0
        }
    }
}
