use std::sync::{Arc, Mutex};
use std::fs::File;
use std::io::Write;

use coralys_moga::config::EvolutionConfig;

use ultracrew::models::{Shift, Worker, Skill};
use ultracrew::ecology::WorkforceEcology;
use ultracrew::public_contracts::InrcScenario;
use ultracrew::optimization::{ScheduleContext, Observatory, run_partitioned_evolution, ScheduleGenome};
use ultracrew::partitioning::{Partitioner, Phase6CPartitioner, BoundaryReconciler, Partition};
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

fn compute_crossing_shifts(sorted_shifts: &[Shift], start_idx: usize, end_idx: usize, boundary_hour: u64) -> usize {
    let mut crossing = 0;
    for s in &sorted_shifts[start_idx..end_idx] {
        if s.start_hour + s.duration_hours > boundary_hour {
            crossing += 1;
        }
    }
    crossing
}

fn main() -> std::io::Result<()> {
    init_logging();
    
    let out_file_perf = Arc::new(Mutex::new(File::create("p6_c_factorial_performance.csv")?));
    {
        let mut f = out_file_perf.lock().unwrap();
        writeln!(f, "treatment,seed,total_partitions,l1_feasible,l2_boundary_conflicts,l3_reconciled_feasible,sum_pairwise_conflicts,non_local_gap").unwrap();
    }
    
    let out_file_bound = Arc::new(Mutex::new(File::create("p6_c_factorial_boundaries.csv")?));
    {
        let mut f = out_file_bound.lock().unwrap();
        writeln!(f, "treatment,seed,boundary_id,left_core_edges,right_core_edges,core_duration_hours,halo_after_hours,halo_span_partitions,nominal_crossing,selected_crossing,boundary_crossing_delta,actual_boundary_conflicts").unwrap();
    }
    
    let shifts_c2 = generate_family_c(0.60);
    let skill = Skill("Pilot".to_string());
    let workers: Vec<Worker> = (0..40).map(|id| Worker { id, skills: vec![skill.clone()] }).collect();
    
    let treatments = vec![
        ("A2", false, false),
        ("C1", true, false),
        ("C2", false, true),
        ("C3", true, true),
    ];
    
    for (treatment, span_aware, dynamic_halo) in treatments {
        println!("=== Running Treatment {} ===", treatment);
        
        for seed in 1..=10 {
            println!("  Seed {}", seed);
            let partitioner = Phase6CPartitioner { 
                max_core_edges: 50, 
                base_halo_hours: 24,
                enable_span_aware_cut: span_aware,
                enable_dynamic_halo: dynamic_halo,
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
            let result = run_partitioned_evolution(ctx.clone(), &partitioner, &reconciler, local_config, global_config);
            
            let mut total_pairwise_conflicts = 0;
            
            for i in 0..partitions.len().saturating_sub(1) {
                let p_left = &partitions[i];
                let p_right = &partitions[i+1];
                
                let g_left = &result.local_genomes[i];
                let g_right = &result.local_genomes[i+1];
                
                let left_core_start = p_left.core_shifts.first().map(|s| s.start_hour).unwrap_or(0);
                let left_core_end = p_left.core_shifts.iter().map(|s| s.start_hour + s.duration_hours).max().unwrap_or(0);
                let core_duration = left_core_end.saturating_sub(left_core_start);
                
                let halo_end = p_left.halo_shifts.iter().map(|s| s.start_hour + s.duration_hours).max().unwrap_or(left_core_end);
                let halo_after_hours = halo_end.saturating_sub(left_core_end);
                
                let mut halo_span_partitions = 0;
                for k in (i+1)..partitions.len() {
                    let k_start = partitions[k].core_shifts.first().map(|s| s.start_hour).unwrap_or(0);
                    if k_start < halo_end {
                        halo_span_partitions += 1;
                    } else {
                        break;
                    }
                }
                
                // Nominal vs Selected crossing
                // To compute nominal boundary hour, we simulate adding shifts to p_left until > 50 edges
                let mut start_idx = 0;
                for (idx, s) in shifts_c2.iter().enumerate() {
                    if s.id == p_left.core_shifts[0].id {
                        start_idx = idx; break;
                    }
                }
                let mut nominal_cut = start_idx;
                let mut edges = 0;
                while nominal_cut < shifts_c2.len() {
                    let mut new_edges = 0;
                    for j in start_idx..nominal_cut {
                        let s1 = &shifts_c2[j];
                        let s2 = &shifts_c2[nominal_cut];
                        let overlap = !(s1.start_hour + s1.duration_hours <= s2.start_hour || s2.start_hour + s2.duration_hours <= s1.start_hour);
                        let rest_violation = !overlap && (
                            (s1.start_hour + s1.duration_hours <= s2.start_hour && s2.start_hour - (s1.start_hour + s1.duration_hours) < 8) ||
                            (s2.start_hour + s2.duration_hours <= s1.start_hour && s1.start_hour - (s2.start_hour + s2.duration_hours) < 8)
                        );
                        if overlap || rest_violation { new_edges += 1; }
                    }
                    edges += new_edges;
                    if edges > partitioner.max_core_edges && nominal_cut > start_idx {
                        break;
                    }
                    nominal_cut += 1;
                }
                
                let nominal_boundary_hour = if nominal_cut < shifts_c2.len() { shifts_c2[nominal_cut].start_hour } else { u64::MAX };
                let nominal_crossing = compute_crossing_shifts(&shifts_c2, start_idx, nominal_cut, nominal_boundary_hour);
                
                let actual_boundary_hour = p_right.core_shifts.first().map(|s| s.start_hour).unwrap_or(u64::MAX);
                // The actual cut index for this boundary is start_idx + p_left.core_shifts.len()
                let actual_cut = start_idx + p_left.core_shifts.len();
                let selected_crossing = compute_crossing_shifts(&shifts_c2, start_idx, actual_cut, actual_boundary_hour);
                
                let boundary_crossing_delta = (nominal_crossing as i64) - (selected_crossing as i64);
                
                let actual_boundary_conflicts = evaluate_pair_conflicts(p_left, g_left, p_right, g_right);
                total_pairwise_conflicts += actual_boundary_conflicts;
                
                let mut f = out_file_bound.lock().unwrap();
                writeln!(f, "{},{},{},{},{},{},{},{},{},{},{},{}",
                    treatment, seed, i, compute_core_edges(&p_left.core_shifts), compute_core_edges(&p_right.core_shifts),
                    core_duration, halo_after_hours, halo_span_partitions, nominal_crossing, selected_crossing, boundary_crossing_delta, actual_boundary_conflicts
                ).unwrap();
            }
            
            let non_local_gap = (result.boundary_conflicts as i64) - (total_pairwise_conflicts as i64);
            
            let mut f = out_file_perf.lock().unwrap();
            writeln!(f, "{},{},{},{},{},{},{},{}",
                treatment, seed, partitions.len(), result.local_feasible_count, result.boundary_conflicts, result.reconciled_feasible,
                total_pairwise_conflicts, non_local_gap
            ).unwrap();
        }
    }
    
    Ok(())
}
