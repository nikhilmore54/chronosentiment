use crate::models::{Shift, Worker};
use crate::optimization::ScheduleGenome;
use std::collections::HashMap;

/// A structural subproblem extracted from the global feasibility landscape.
#[derive(Debug, Clone)]
pub struct Partition {
    pub id: usize,
    /// Shifts that this partition explicitly owns and must solve.
    pub core_shifts: Vec<Shift>,
    /// Shifts included only to evaluate constraints crossing the boundary.
    pub halo_shifts: Vec<Shift>,
    /// The worker pool eligible to take shifts in this partition.
    /// In the primary Phase 5 experiment (P2), this is the entire global worker pool.
    pub eligible_workers: Vec<Worker>,
}

impl Partition {
    /// Returns all shifts (core + halo) visible to this partition.
    pub fn all_shifts(&self) -> Vec<Shift> {
        let mut all = self.core_shifts.clone();
        all.extend(self.halo_shifts.clone());
        // Sort to maintain temporal coherence
        all.sort_by_key(|s| s.start_hour);
        all
    }
}

pub trait Partitioner: Send + Sync {
    /// Partitions the global problem into K structural regions.
    fn partition(&self, shifts: &[Shift], workers: &[Worker]) -> Vec<Partition>;
}

/// P1 (No Halo) and P2 (With Halo) Primary Experiment Partitioner
pub struct TemporalPartitioner {
    pub num_partitions: usize,
    pub halo_hours: u64,
}

impl Partitioner for TemporalPartitioner {
    fn partition(&self, shifts: &[Shift], workers: &[Worker]) -> Vec<Partition> {
        if shifts.is_empty() { return vec![]; }
        
        let mut sorted_shifts = shifts.to_vec();
        sorted_shifts.sort_by_key(|s| s.start_hour);
        
        let total_duration = sorted_shifts.last().unwrap().start_hour + sorted_shifts.last().unwrap().duration_hours - sorted_shifts.first().unwrap().start_hour;
        let window_size = (total_duration as f64 / self.num_partitions as f64).ceil() as u64;
        let base_start = sorted_shifts.first().unwrap().start_hour;

        let mut partitions = Vec::new();
        
        for k in 0..self.num_partitions {
            let window_start = base_start + k as u64 * window_size;
            let window_end = if k == self.num_partitions - 1 {
                u64::MAX
            } else {
                window_start + window_size
            };
            
            let mut core_shifts = Vec::new();
            let mut halo_shifts = Vec::new();
            
            for s in &sorted_shifts {
                if s.start_hour >= window_start && s.start_hour < window_end {
                    core_shifts.push(s.clone());
                } else if self.halo_hours > 0 {
                    // Check if shift falls within halo boundaries
                    let halo_start = window_start.saturating_sub(self.halo_hours);
                    let halo_end = if window_end == u64::MAX { u64::MAX } else { window_end.saturating_add(self.halo_hours) };
                    
                    if (s.start_hour >= halo_start && s.start_hour < window_start) || 
                       (s.start_hour >= window_end && s.start_hour < halo_end) {
                        halo_shifts.push(s.clone());
                    }
                }
            }
            
            partitions.push(Partition {
                id: k,
                core_shifts,
                halo_shifts,
                eligible_workers: workers.to_vec(), // Global eligibility
            });
        }
        
        partitions
    }
}

/// P4 Control Experiment Partitioner (Resource Clustering / Disjoint Isolation)
pub struct ResourceClusterPartitioner {
    pub num_partitions: usize,
}

impl Partitioner for ResourceClusterPartitioner {
    fn partition(&self, shifts: &[Shift], workers: &[Worker]) -> Vec<Partition> {
        let mut partitions = vec![
            Partition {
                id: 0,
                core_shifts: vec![],
                halo_shifts: vec![],
                eligible_workers: vec![],
            }; 
            self.num_partitions
        ];
        
        for (i, p) in partitions.iter_mut().enumerate() {
            p.id = i;
        }
        
        for (i, shift) in shifts.iter().enumerate() {
            partitions[i % self.num_partitions].core_shifts.push(shift.clone());
        }
        for (i, worker) in workers.iter().enumerate() {
            partitions[i % self.num_partitions].eligible_workers.push(worker.clone());
        }
        partitions
    }
}

pub trait Reconciler: Send + Sync {
    /// Reconciles local genomes into a global genome.
    /// Extracts assignments ONLY for each partition's core_shifts.
    fn reconcile(&self, partitions: &[Partition], local_genomes: &[ScheduleGenome]) -> ScheduleGenome;
}

pub struct BoundaryReconciler;

impl Reconciler for BoundaryReconciler {
    fn reconcile(&self, partitions: &[Partition], local_genomes: &[ScheduleGenome]) -> ScheduleGenome {
        let mut global_assignments = HashMap::new();
        
        // Only extract the core assignments from each partition
        for (partition, genome) in partitions.iter().zip(local_genomes.iter()) {
            for shift in &partition.core_shifts {
                if let Some(&worker_id) = genome.assignments.get(&shift.id) {
                    global_assignments.insert(shift.id, worker_id);
                }
            }
        }
        
        ScheduleGenome {
            assignments: global_assignments,
        }
    }
}

/// Phase 6 Adaptive Temporal Partitioner
pub struct AdaptiveTemporalPartitioner {
    pub max_core_edges: usize,
    pub halo_hours: u64,
}

impl Partitioner for AdaptiveTemporalPartitioner {
    fn partition(&self, shifts: &[Shift], workers: &[Worker]) -> Vec<Partition> {
        if shifts.is_empty() { return vec![]; }
        
        let mut sorted_shifts = shifts.to_vec();
        sorted_shifts.sort_by_key(|s| s.start_hour);
        
        let mut partitions = Vec::new();
        let mut start_idx = 0;
        let mut partition_id = 0;
        
        let mut i = 0;
        while i < sorted_shifts.len() {
            // Evaluate test_core = sorted_shifts[start_idx..=i]
            let test_core = &sorted_shifts[start_idx..=i];
            let window_start = test_core.first().unwrap().start_hour;
            let window_end = sorted_shifts.get(i + 1).map(|s| s.start_hour).unwrap_or(u64::MAX);
            
            let halo_start = window_start.saturating_sub(self.halo_hours);
            let halo_end = if window_end == u64::MAX { u64::MAX } else { window_end.saturating_add(self.halo_hours) };
            
            let mut test_halo = Vec::new();
            for s in &sorted_shifts[..start_idx] {
                if s.start_hour >= halo_start { test_halo.push(s.clone()); }
            }
            if i + 1 < sorted_shifts.len() {
                for s in &sorted_shifts[i+1..] {
                    if s.start_hour < halo_end { test_halo.push(s.clone()); }
                    else { break; }
                }
            }
            
            // Compute D_core
            let mut core_edges = 0;
            for x in 0..test_core.len() {
                for y in (x+1)..test_core.len() {
                    let s1 = &test_core[x];
                    let s2 = &test_core[y];
                    let overlap = !(s1.start_hour + s1.duration_hours <= s2.start_hour || s2.start_hour + s2.duration_hours <= s1.start_hour);
                    let rest_violation = !overlap && (
                        (s1.start_hour + s1.duration_hours <= s2.start_hour && s2.start_hour - (s1.start_hour + s1.duration_hours) < 8) ||
                        (s2.start_hour + s2.duration_hours <= s1.start_hour && s1.start_hour - (s2.start_hour + s2.duration_hours) < 8)
                    );
                    if overlap || rest_violation { core_edges += 1; }
                }
            }
            
            // Compute D_crossover
            let mut crossover_edges = 0;
            for s1 in test_core {
                for s2 in &test_halo {
                    let overlap = !(s1.start_hour + s1.duration_hours <= s2.start_hour || s2.start_hour + s2.duration_hours <= s1.start_hour);
                    let rest_violation = !overlap && (
                        (s1.start_hour + s1.duration_hours <= s2.start_hour && s2.start_hour - (s1.start_hour + s1.duration_hours) < 8) ||
                        (s2.start_hour + s2.duration_hours <= s1.start_hour && s1.start_hour - (s2.start_hour + s2.duration_hours) < 8)
                    );
                    if overlap || rest_violation { crossover_edges += 1; }
                }
            }
            
            // If threshold exceeded AND we have at least 1 shift already accepted (so we don't trap on a single massive shift)
            if core_edges > self.max_core_edges && i > start_idx {
                // Reject s_{i}, close partition with start_idx..i
                let actual_core = &sorted_shifts[start_idx..i];
                let actual_window_start = actual_core.first().unwrap().start_hour;
                let actual_window_end = sorted_shifts[i].start_hour;
                
                let actual_halo_start = actual_window_start.saturating_sub(self.halo_hours);
                let actual_halo_end = actual_window_end.saturating_add(self.halo_hours);
                
                let mut actual_halo = Vec::new();
                for s in &sorted_shifts[..start_idx] {
                    if s.start_hour >= actual_halo_start { actual_halo.push(s.clone()); }
                }
                for s in &sorted_shifts[i..] {
                    if s.start_hour < actual_halo_end { actual_halo.push(s.clone()); }
                    else { break; }
                }
                
                partitions.push(Partition {
                    id: partition_id,
                    core_shifts: actual_core.to_vec(),
                    halo_shifts: actual_halo,
                    eligible_workers: workers.to_vec(),
                });
                partition_id += 1;
                start_idx = i;
                // DO NOT increment i, so next iteration evaluates s_{i} as the first shift of the new partition
            } else {
                i += 1;
            }
        }
        
        // Push the final partition
        if start_idx < sorted_shifts.len() {
            let actual_core = &sorted_shifts[start_idx..];
            let actual_window_start = actual_core.first().unwrap().start_hour;
            let actual_window_end = u64::MAX;
            
            let actual_halo_start = actual_window_start.saturating_sub(self.halo_hours);
            
            let mut actual_halo = Vec::new();
            for s in &sorted_shifts[..start_idx] {
                if s.start_hour >= actual_halo_start { actual_halo.push(s.clone()); }
            }
            
            partitions.push(Partition {
                id: partition_id,
                core_shifts: actual_core.to_vec(),
                halo_shifts: actual_halo,
                eligible_workers: workers.to_vec(),
            });
        }
        
        partitions
    }
}

pub struct Phase6CPartitioner {
    pub max_core_edges: usize,
    pub base_halo_hours: u64,
    pub enable_span_aware_cut: bool,
    pub enable_dynamic_halo: bool,
}

impl Partitioner for Phase6CPartitioner {
    fn partition(&self, shifts: &[Shift], workers: &[Worker]) -> Vec<Partition> {
        let mut sorted_shifts = shifts.to_vec();
        sorted_shifts.sort_by_key(|s| s.start_hour);
        
        let mut partitions = Vec::new();
        let mut start_idx = 0;
        let mut partition_id = 0;
        
        while start_idx < sorted_shifts.len() {
            let mut i = start_idx;
            let mut core_edges = 0;
            
            while i < sorted_shifts.len() {
                // Add shift i to core and check edges
                let mut new_edges = 0;
                for j in start_idx..i {
                    let s1 = &sorted_shifts[j];
                    let s2 = &sorted_shifts[i];
                    
                    let overlap = !(s1.start_hour + s1.duration_hours <= s2.start_hour || s2.start_hour + s2.duration_hours <= s1.start_hour);
                    let rest_violation = !overlap && (
                        (s1.start_hour + s1.duration_hours <= s2.start_hour && s2.start_hour - (s1.start_hour + s1.duration_hours) < 8) ||
                        (s2.start_hour + s2.duration_hours <= s1.start_hour && s1.start_hour - (s2.start_hour + s2.duration_hours) < 8)
                    );
                    
                    if overlap || rest_violation {
                        new_edges += 1;
                    }
                }
                
                core_edges += new_edges;
                
                if core_edges > self.max_core_edges && i > start_idx {
                    break;
                }
                
                i += 1;
            }
            
            let mut cut_idx = i;
            
            if self.enable_span_aware_cut && i < sorted_shifts.len() && i > start_idx {
                let nominal_boundary_hour = sorted_shifts[i].start_hour;
                let min_boundary_hour = nominal_boundary_hour.saturating_sub(4);
                
                let mut min_crossing = usize::MAX;
                let mut best_cut = i;
                
                for j in (start_idx + 1 ..= i).rev() {
                    let candidate_boundary_hour = if j < sorted_shifts.len() {
                        sorted_shifts[j].start_hour
                    } else {
                        u64::MAX
                    };
                    
                    if candidate_boundary_hour < min_boundary_hour {
                        break; // Out of 4h search radius
                    }
                    
                    let mut crossing = 0;
                    for k in start_idx..j {
                        let s = &sorted_shifts[k];
                        if s.start_hour + s.duration_hours > candidate_boundary_hour {
                            crossing += 1;
                        }
                    }
                    
                    // If crossing < min_crossing, update best_cut. 
                    // This naturally favors the larger j (later cut) in case of ties because we iterate backwards!
                    if crossing < min_crossing {
                        min_crossing = crossing;
                        best_cut = j;
                    }
                }
                
                cut_idx = best_cut;
            }
            
            // Construct the core
            let core_shifts = sorted_shifts[start_idx..cut_idx].to_vec();
            
            let actual_window_start = core_shifts.first().map(|s| s.start_hour).unwrap_or(0);
            let mut actual_window_end = actual_window_start;
            for s in &core_shifts {
                let end = s.start_hour + s.duration_hours;
                if end > actual_window_end {
                    actual_window_end = end;
                }
            }
            let core_duration = actual_window_end.saturating_sub(actual_window_start);
            
            let actual_halo_hours = if self.enable_dynamic_halo {
                std::cmp::min(self.base_halo_hours, core_duration)
            } else {
                self.base_halo_hours
            };
            
            let halo_start = actual_window_start.saturating_sub(actual_halo_hours);
            let halo_end = actual_window_end.saturating_add(actual_halo_hours);
            
            let mut halo_shifts = Vec::new();
            for s in shifts {
                let s_end = s.start_hour + s.duration_hours;
                // Exclude core shifts
                let is_core = core_shifts.iter().any(|cs| cs.id == s.id);
                if !is_core {
                    if (s.start_hour >= halo_start && s.start_hour < halo_end) || 
                       (s_end > halo_start && s_end <= halo_end) ||
                       (s.start_hour <= halo_start && s_end >= halo_end) {
                        halo_shifts.push(s.clone());
                    }
                }
            }
            
            partitions.push(Partition {
                id: partition_id,
                core_shifts,
                halo_shifts,
                eligible_workers: workers.to_vec(),
            });
            
            partition_id += 1;
            start_idx = cut_idx;
        }
        
        partitions
    }
}
