use std::sync::{Arc, Mutex};
use std::fs::File;
use std::io::Write;
use std::collections::{HashMap, HashSet};

use coralys_moga::config::EvolutionConfig;
use ultracrew::optimization::ScheduleGenome;

use ultracrew::models::{Shift, Worker, Skill};
use ultracrew::ecology::WorkforceEcology;
use ultracrew::public_contracts::InrcScenario;
use ultracrew::optimization::{ScheduleContext, Observatory, run_partitioned_evolution};
use ultracrew::partitioning::{Partitioner, Phase6CPartitioner, BoundaryReconciler, Partition, Reconciler};
use ultracrew::telemetry::init_logging;

fn generate_family_c(weekend_ratio: f64) -> Vec<Shift> {
    let skill = "Pilot".to_string();
    let total_hours = 1140; 
    let weekend_hours = (total_hours as f64 * weekend_ratio) as u64;
    let weekday_hours = total_hours - weekend_hours;
    
    let weekend_shifts = weekend_hours / 8;
    let weekday_shifts = weekday_hours / 8;
    
    let mut shifts = vec![];
    for i in 0..weekend_shifts {
        shifts.push(Shift { id: (i + 1) as u64, start_hour: 120 + ((i * 8) % 40), duration_hours: 8, required_skill: Skill(skill.clone()) });
    }
    for i in 0..weekday_shifts {
        shifts.push(Shift { id: (weekend_shifts + i + 1) as u64, start_hour: (i * 8) % 120, duration_hours: 8, required_skill: Skill(skill.clone()) });
    }
    shifts.sort_by_key(|s| s.start_hour);
    shifts
}


#[derive(Debug, Clone)]
struct Conflict {
    id: usize,
    worker_id: u64,
    shift1_id: u64,
    shift2_id: u64,
    violation_type: String,
    delta_p: usize,
    p1: usize,
    p2: usize,
    local1: Option<u64>,
    local2: Option<u64>,
    reconciled1: Option<u64>,
    reconciled2: Option<u64>,
    global1: Option<u64>,
    global2: Option<u64>,
}

fn extract_conflicts(
    genome: &ScheduleGenome,
    context: &ScheduleContext,
    shift_to_partition: &HashMap<u64, usize>,
    local_assignments: &HashMap<u64, u64>,
    reconciled_genome: &ScheduleGenome,
    global_genome: &ScheduleGenome,
    conflict_id_counter: &mut usize,
) -> Vec<Conflict> {
    let mut worker_shifts: HashMap<u64, Vec<&Shift>> = HashMap::new();
    let mut worker_hours: HashMap<u64, u64> = HashMap::new();
    
    for shift in context.shifts.iter() {
        if let Some(&worker_id) = genome.assignments.get(&shift.id) {
            worker_shifts.entry(worker_id).or_default().push(shift);
            *worker_hours.entry(worker_id).or_default() += shift.duration_hours;
        }
    }
    
    let min_rest = context.scenario.as_ref().and_then(|s| s.minimum_rest_hours).unwrap_or(10);
    let hc3_limit = context.scenario.as_ref().and_then(|s| s.max_hours_per_worker).map(|h| h as u64).unwrap_or(40);
    
    let mut conflicts = Vec::new();
    
    for (worker_id, shifts) in worker_shifts {
        // HC3 max hours check
        let total_hours = *worker_hours.get(&worker_id).unwrap_or(&0);
        if total_hours > hc3_limit {
            let mut sorted_shifts = shifts.clone();
            sorted_shifts.sort_by_key(|s| s.start_hour);
            // Form a chain of conflicts for HC3 for the dependency graph
            for i in 0..sorted_shifts.len().saturating_sub(1) {
                let s1 = sorted_shifts[i];
                let s2 = sorted_shifts[i + 1];
                let p1 = *shift_to_partition.get(&s1.id).unwrap_or(&0);
                let p2 = *shift_to_partition.get(&s2.id).unwrap_or(&0);
                let delta_p = p1.abs_diff(p2);
                
                conflicts.push(Conflict {
                    id: *conflict_id_counter,
                    worker_id,
                    shift1_id: s1.id,
                    shift2_id: s2.id,
                    violation_type: "MaxHours".to_string(),
                    delta_p,
                    p1,
                    p2,
                    local1: local_assignments.get(&s1.id).copied(),
                    local2: local_assignments.get(&s2.id).copied(),
                    reconciled1: reconciled_genome.assignments.get(&s1.id).copied(),
                    reconciled2: reconciled_genome.assignments.get(&s2.id).copied(),
                    global1: global_genome.assignments.get(&s1.id).copied(),
                    global2: global_genome.assignments.get(&s2.id).copied(),
                });
                *conflict_id_counter += 1;
            }
        }
        let mut sorted_shifts = shifts.clone();
        sorted_shifts.sort_by_key(|s| s.start_hour);
        
        for i in 0..sorted_shifts.len().saturating_sub(1) {
            let s1 = sorted_shifts[i];
            let s2 = sorted_shifts[i + 1];
            
            let mut is_conflict = false;
            let mut violation_type = String::new();
            
            if s1.start_hour + s1.duration_hours > s2.start_hour {
                is_conflict = true;
                violation_type = "Overlap".to_string();
            } else {
                let gap = s2.start_hour - (s1.start_hour + s1.duration_hours);
                if gap < min_rest {
                    is_conflict = true;
                    violation_type = "Rest".to_string();
                }
            }
            
            if is_conflict {
                let p1 = *shift_to_partition.get(&s1.id).unwrap_or(&0);
                let p2 = *shift_to_partition.get(&s2.id).unwrap_or(&0);
                let delta_p = p1.abs_diff(p2);
                
                conflicts.push(Conflict {
                    id: *conflict_id_counter,
                    worker_id,
                    shift1_id: s1.id,
                    shift2_id: s2.id,
                    violation_type,
                    delta_p,
                    p1,
                    p2,
                    local1: local_assignments.get(&s1.id).copied(),
                    local2: local_assignments.get(&s2.id).copied(),
                    reconciled1: reconciled_genome.assignments.get(&s1.id).copied(),
                    reconciled2: reconciled_genome.assignments.get(&s2.id).copied(),
                    global1: global_genome.assignments.get(&s1.id).copied(),
                    global2: global_genome.assignments.get(&s2.id).copied(),
                });
                *conflict_id_counter += 1;
            }
        }
    }
    
    conflicts
}

struct UnionFind {
    parent: Vec<usize>,
}

impl UnionFind {
    fn new(size: usize) -> Self {
        Self { parent: (0..size).collect() }
    }
    
    fn find(&mut self, i: usize) -> usize {
        if self.parent[i] == i {
            i
        } else {
            let root = self.find(self.parent[i]);
            self.parent[i] = root;
            root
        }
    }
    
    fn union(&mut self, i: usize, j: usize) {
        let root_i = self.find(i);
        let root_j = self.find(j);
        if root_i != root_j {
            self.parent[root_i] = root_j;
        }
    }
}

fn main() -> std::io::Result<()> {
    init_logging();
    
    let out_forensics = Arc::new(Mutex::new(File::create("p7_a_conflict_forensics.csv")?));
    {
        let mut f = out_forensics.lock().unwrap();
        writeln!(f, "seed,conflict_id,worker_id,shift1_id,shift2_id,violation_type,p1,p2,delta_p,local1,local2,reconciled1,reconciled2,global1,global2,stage").unwrap();
    }
    
    let out_summary = Arc::new(Mutex::new(File::create("p7_a_cluster_summary.csv")?));
    {
        let mut f = out_summary.lock().unwrap();
        writeln!(f, "seed,total_reconciled_conflicts,total_global_conflicts,r_global_refinement,num_components,cluster_sizes").unwrap();
    }
    
    let shifts_c2 = generate_family_c(0.60);
    let skill = Skill("Pilot".to_string());
    let workers: Vec<Worker> = (0..40).map(|id| Worker { id, skills: vec![skill.clone()] }).collect();
    
    println!("=== Running P7-A Reconciliation Forensics (A2 Partitioner) ===");
    
    for seed in 1..=10 {
        println!("  Seed {}", seed);
        let partitioner = Phase6CPartitioner { 
            max_core_edges: 50, 
            base_halo_hours: 24,
            enable_span_aware_cut: false,
            enable_dynamic_halo: false,
        };
        
        let scenario = InrcScenario {
            planning_horizon_hours: Some(168.0),
            max_hours_per_worker: Some(40.0),
            minimum_rest_hours: Some(8),
            leave_requests: None,
        };
        
        let ctx = Arc::new(ScheduleContext {
            workers: Arc::new(workers.clone()),
            shifts: Arc::new(shifts_c2.clone()),
            ecology: WorkforceEcology::new(),
            rng_seed: seed,
            observatory: Arc::new(Mutex::new(Observatory::new())),
            locked_assignments: None,
            scenario: Some(scenario),
            enable_fatigue: false,
            fatigue_weight: 0.0,
            hc3_aware_initialization: true,
            temporal_scarcity_construction: false,
            disable_global_constructor: false,
            constructor_budget_ms: Some(5000),
            precomputed_seeds: None,
        });
        
        let local_config = EvolutionConfig {
            population_size: 20,
            generation_limit: 50,
            mutation_rate: 0.2,
            crossover_rate: 0.8,
            elite_count: 2,
            seed: Some(seed),
            ..Default::default()
        };
        
        let global_config = EvolutionConfig {
            population_size: 100,
            generation_limit: 100,
            mutation_rate: 0.2,
            crossover_rate: 0.8,
            elite_count: 5,
            seed: Some(seed),
            ..Default::default()
        };
        
        let reconciler = BoundaryReconciler;
        let partitions = partitioner.partition(&shifts_c2, &workers);
        
        let mut shift_to_partition = HashMap::new();
        for p in &partitions {
            for s in &p.core_shifts {
                shift_to_partition.insert(s.id, p.id);
            }
        }
        
        let result = run_partitioned_evolution(ctx.clone(), &partitioner, &reconciler, local_config, global_config);
        
        let mut local_assignments = HashMap::new();
        for (i, p) in partitions.iter().enumerate() {
            let local_genome = &result.local_genomes[i];
            for s in &p.core_shifts {
                if let Some(&w) = local_genome.assignments.get(&s.id) {
                    local_assignments.insert(s.id, w);
                }
            }
        }
        
        let reconciled_genome = reconciler.reconcile(&partitions, &result.local_genomes);
        let final_genome = &result.global_result.global_best.schedule;
        
        let mut conflict_id_counter = 0;
        
        let reconciled_conflicts = extract_conflicts(
            &reconciled_genome,
            &ctx,
            &shift_to_partition,
            &local_assignments,
            &reconciled_genome,
            final_genome,
            &mut conflict_id_counter,
        );
        
        let global_conflicts = extract_conflicts(
            final_genome,
            &ctx,
            &shift_to_partition,
            &local_assignments,
            &reconciled_genome,
            final_genome,
            &mut conflict_id_counter,
        );
        
        {
            let mut f = out_forensics.lock().unwrap();
            for c in &reconciled_conflicts {
                writeln!(f, "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},reconciled",
                    seed, c.id, c.worker_id, c.shift1_id, c.shift2_id, c.violation_type, c.p1, c.p2, c.delta_p,
                    c.local1.map(|v| v.to_string()).unwrap_or_default(),
                    c.local2.map(|v| v.to_string()).unwrap_or_default(),
                    c.reconciled1.map(|v| v.to_string()).unwrap_or_default(),
                    c.reconciled2.map(|v| v.to_string()).unwrap_or_default(),
                    c.global1.map(|v| v.to_string()).unwrap_or_default(),
                    c.global2.map(|v| v.to_string()).unwrap_or_default(),
                ).unwrap();
            }
            for c in &global_conflicts {
                writeln!(f, "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},post-global",
                    seed, c.id, c.worker_id, c.shift1_id, c.shift2_id, c.violation_type, c.p1, c.p2, c.delta_p,
                    c.local1.map(|v| v.to_string()).unwrap_or_default(),
                    c.local2.map(|v| v.to_string()).unwrap_or_default(),
                    c.reconciled1.map(|v| v.to_string()).unwrap_or_default(),
                    c.reconciled2.map(|v| v.to_string()).unwrap_or_default(),
                    c.global1.map(|v| v.to_string()).unwrap_or_default(),
                    c.global2.map(|v| v.to_string()).unwrap_or_default(),
                ).unwrap();
            }
        }
        
        let n = reconciled_conflicts.len();
        let mut uf = UnionFind::new(n);
        
        for i in 0..n {
            for j in (i+1)..n {
                let c1 = &reconciled_conflicts[i];
                let c2 = &reconciled_conflicts[j];
                
                let shared_worker = c1.worker_id == c2.worker_id;
                let shared_shift = c1.shift1_id == c2.shift1_id || c1.shift1_id == c2.shift2_id ||
                                   c1.shift2_id == c2.shift1_id || c1.shift2_id == c2.shift2_id;
                                   
                if shared_worker || shared_shift {
                    uf.union(i, j);
                }
            }
        }
        
        let mut components: HashMap<usize, Vec<usize>> = HashMap::new();
        for i in 0..n {
            components.entry(uf.find(i)).or_default().push(i);
        }
        
        let num_components = components.len();
        let mut cluster_summaries: Vec<(usize, usize)> = components.values().map(|v| {
            let size = v.len();
            let mut min_p = usize::MAX;
            let mut max_p = 0;
            for &idx in v {
                let c = &reconciled_conflicts[idx];
                if c.p1 < min_p { min_p = c.p1; }
                if c.p1 > max_p { max_p = c.p1; }
                if c.p2 < min_p { min_p = c.p2; }
                if c.p2 > max_p { max_p = c.p2; }
            }
            let delta_p_max = if max_p >= min_p && min_p != usize::MAX { max_p - min_p } else { 0 };
            (size, delta_p_max)
        }).collect();
        
        cluster_summaries.sort_by(|a, b| b.0.cmp(&a.0));
        
        let cluster_sizes_str = cluster_summaries.iter()
            .map(|(s, dp)| format!("{}(dp={})", s, dp))
            .collect::<Vec<String>>()
            .join(";");
        
        let r_global_refinement = if n > 0 {
            (global_conflicts.len() as f64) / (n as f64)
        } else {
            0.0
        };
        
        {
            let mut f = out_summary.lock().unwrap();
            writeln!(f, "{},{},{},{:.3},{},{}",
                seed, n, global_conflicts.len(), r_global_refinement, num_components, cluster_sizes_str
            ).unwrap();
        }
    }
    
    Ok(())
}
