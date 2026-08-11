use coralys_moga::ecology::{EcologyMemory, EcologySignal, EcologyPolicy};

pub type NurseId = usize;

/// Adapts the generic Coralys EcologyMemory to the Workforce scheduling domain.
#[derive(Clone)]
pub struct WorkforceEcologyAdapter {
    pub memory: EcologyMemory<NurseId>,
    pub policy: EcologyPolicy,
}

impl WorkforceEcologyAdapter {
    pub fn new(num_nurses: usize, alpha: f64) -> Self {
        let mut memory = EcologyMemory::new();
        // Initialize for all nurses so keys exist
        for n in 0..num_nurses {
            memory.accumulate(n, "assignments", 0.0);
            memory.accumulate(n, "weekends", 0.0);
        }
        Self {
            memory,
            policy: EcologyPolicy::new(alpha),
        }
    }

    pub fn accumulate_assignments(&mut self, nurse: NurseId, count: usize) {
        self.memory.accumulate(nurse, "assignments", count as f64);
    }
    
    pub fn accumulate_weekends(&mut self, nurse: NurseId, count: usize) {
        self.memory.accumulate(nurse, "weekends", count as f64);
    }
    
    pub fn get_assignments(&self, nurse: NurseId) -> f64 {
        self.memory.get_measure(nurse, "assignments")
    }

    pub fn get_weekends(&self, nurse: NurseId) -> f64 {
        self.memory.get_measure(nurse, "weekends")
    }

    /// Calculate the ecology signal (load pressure) for a specific nurse
    pub fn compute_signal(&self, nurse: NurseId, num_nurses: usize) -> EcologySignal {
        let total_assignments: f64 = (0..num_nurses)
            .map(|n| self.get_assignments(n))
            .sum();
            
        let mean = if num_nurses > 0 { total_assignments / num_nurses as f64 } else { 0.0 };
        
        if mean == 0.0 {
            return EcologySignal::new(0.0);
        }
        
        let nurse_load = self.get_assignments(nurse);
        let ratio = nurse_load / mean;
        
        // pressure > 0 means under-utilized (should receive more shifts)
        // pressure < 0 means over-utilized (should receive fewer shifts)
        let pressure = 1.0 - ratio;
        
        EcologySignal::new(pressure)
    }
}
