use std::sync::{Arc, Mutex};
use std::fs::File;
use std::io::Write;

use coralys_moga::config::EvolutionConfig;

use ultracrew::models::{Shift, Worker, Skill};
use ultracrew::ecology::WorkforceEcology;
use ultracrew::public_contracts::InrcScenario;
use ultracrew::optimization::{ScheduleContext, Observatory, run_partitioned_evolution, ScheduleGenome};
use ultracrew::partitioning::{
    Partitioner, AdaptiveTemporalPartitioner, BoundaryReconciler, Partition
};
use ultracrew::telemetry::init_logging;

fn generate_family_c(weekend_ratio: f64) -> Vec<Shift> {
    let skill = "Pilot".to_string();
    let total_hours = 1140; // 95% of 1200
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
    shifts
}

fn compute_crossover_edges(core: &[Shift], halo: &[Shift]) -> usize {
    let mut crossover_edges = 0;
    for s1 in core {
        for s2 in halo {
            let overlap = !(s1.start_hour + s1.duration_hours <= s2.start_hour || s2.start_hour + s2.duration_hours <= s1.start_hour);
            let rest_violation = !overlap && (
                (s1.start_hour + s1.duration_hours <= s2.start_hour && s2.start_hour - (s1.start_hour + s1.duration_hours) < 8) ||
                (s2.start_hour + s2.duration_hours <= s1.start_hour && s1.start_hour - (s2.start_hour + s2.duration_hours) < 8)
            );
            if overlap || rest_violation { crossover_edges += 1; }
        }
    }
    crossover_edges
}

fn compute_core_edges(core: &[Shift]) -> usize {
    let mut core_edges = 0;
    for x in 0..core.len() {
        for y in (x+1)..core.len() {
            let s1 = &core[x];
            let s2 = &core[y];
            let overlap = !(s1.start_hour + s1.duration_hours <= s2.start_hour || s2.start_hour + s2.duration_hours <= s1.start_hour);
            let rest_violation = !overlap && (
                (s1.start_hour + s1.duration_hours <= s2.start_hour && s2.start_hour - (s1.start_hour + s1.duration_hours) < 8) ||
                (s2.start_hour + s2.duration_hours <= s1.start_hour && s1.start_hour - (s2.start_hour + s2.duration_hours) < 8)
            );
            if overlap || rest_violation { core_edges += 1; }
        }
    }
    core_edges
}

fn evaluate_pair_conflicts(
    p1: &Partition,
    g1: &ScheduleGenome,
    p2: &Partition,
    g2: &ScheduleGenome,
) -> usize {
    let mut conflicts = 0;
    for s1 in &p1.core_shifts {
        for s2 in &p2.core_shifts {
            if let (Some(w1), Some(w2)) = (g1.assignments.get(&s1.id), g2.assignments.get(&s2.id)) {
                if w1 == w2 {
                    let overlap = !(s1.start_hour + s1.duration_hours <= s2.start_hour || s2.start_hour + s2.duration_hours <= s1.start_hour);
                    let rest_violation = !overlap && (
                        (s1.start_hour + s1.duration_hours <= s2.start_hour && s2.start_hour - (s1.start_hour + s1.duration_hours) < 8) ||
                        (s2.start_hour + s2.duration_hours <= s1.start_hour && s1.start_hour - (s2.start_hour + s2.duration_hours) < 8)
                    );
                    
                    if overlap || rest_violation {
                        conflicts += 1;
                    }
                }
            }
        }
    }
    conflicts
}

fn main() -> std::io::Result<()> {
    init_logging();
    
    let out_file = Arc::new(Mutex::new(File::create("p6_b_boundary_forensics.csv")?));
    {
        let mut f = out_file.lock().unwrap();
        writeln!(f, "seed,boundary_id,boundary_hour,distance_from_peak,left_partition,right_partition,left_core_edges,right_core_edges,left_shift_count,right_shift_count,shifts_crossing_boundary,halo_crossover_edges,actual_boundary_conflicts").unwrap();
    }
    
    let shifts_c2 = generate_family_c(0.60);
    let skill = Skill("Pilot".to_string());
    let workers: Vec<Worker> = (0..40).map(|id| Worker { id, skills: vec![skill.clone()] }).collect();
    
    let num_seeds = 10;
    
    for seed in 1..=num_seeds {
        println!("Running Seed {}...", seed);
        let partitioner = AdaptiveTemporalPartitioner { max_core_edges: 50, halo_hours: 24 };
        
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
        let result = run_partitioned_evolution(ctx.clone(), &partitioner, &reconciler, local_config, global_config);
        
        // Find the peak density hour (most concurrent shifts)
        let mut hour_density = vec![0; 168];
        for s in &shifts_c2 {
            for h in s.start_hour..(s.start_hour + s.duration_hours) {
                if h < 168 { hour_density[h as usize] += 1; }
            }
        }
        let peak_hour = hour_density.iter().enumerate().max_by_key(|&(_, count)| count).map(|(i, _)| i).unwrap_or(0) as i64;
        
        let mut total_pairwise_conflicts = 0;
        
        for i in 0..partitions.len().saturating_sub(1) {
            let p_left = &partitions[i];
            let p_right = &partitions[i+1];
            
            let g_left = &result.local_genomes[i];
            let g_right = &result.local_genomes[i+1];
            
            let boundary_hour = p_right.core_shifts.first().map(|s| s.start_hour).unwrap_or(0);
            let distance_from_peak = (boundary_hour as i64 - peak_hour).abs();
            
            let left_core_edges = compute_core_edges(&p_left.core_shifts);
            let right_core_edges = compute_core_edges(&p_right.core_shifts);
            
            let left_shift_count = p_left.core_shifts.len();
            let right_shift_count = p_right.core_shifts.len();
            
            let mut shifts_crossing_boundary = 0;
            for s in &p_left.core_shifts {
                if s.start_hour + s.duration_hours > boundary_hour {
                    shifts_crossing_boundary += 1;
                }
            }
            
            let halo_crossover_edges = compute_crossover_edges(&p_left.core_shifts, &p_right.core_shifts);
            
            let actual_boundary_conflicts = evaluate_pair_conflicts(p_left, g_left, p_right, g_right);
            total_pairwise_conflicts += actual_boundary_conflicts;
            
            let mut f = out_file.lock().unwrap();
            writeln!(f, "{},{},{},{},{},{},{},{},{},{},{},{},{}",
                seed, i, boundary_hour, distance_from_peak, p_left.id, p_right.id,
                left_core_edges, right_core_edges, left_shift_count, right_shift_count,
                shifts_crossing_boundary, halo_crossover_edges, actual_boundary_conflicts
            ).unwrap();
        }
        
        println!("Seed {}: Total Pairwise Conflicts = {}, Reconciled Boundary Conflicts = {}", seed, total_pairwise_conflicts, result.boundary_conflicts);
    }
    
    Ok(())
}
