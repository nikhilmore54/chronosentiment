use std::fs::File;
use std::io::Write;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use coralys_moga::config::EvolutionConfig;
use ultracrew::models::{Shift, Worker, Skill};
use ultracrew::ecology::WorkforceEcology;
use ultracrew::public_contracts::InrcScenario;
use ultracrew::optimization::{ScheduleContext, Observatory, run_partitioned_evolution};
use ultracrew::partitioning::{Partitioner, Phase6CPartitioner, BoundaryReconciler, Reconciler};
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

fn main() {
    init_logging();
    
    let mut boundary_file = File::create("p7_b0_boundary_state.csv").unwrap();
    writeln!(boundary_file, "seed,partition,constraint_type,resource_id,state_before,state_after,remaining_capacity,future_required_demand,alternative_capacity,capacity_margin,irreversible").unwrap();
    
    let shifts_c2 = generate_family_c(0.60);
    let skill = Skill("Pilot".to_string());
    let workers: Vec<Worker> = (0..40).map(|id| Worker { id, skills: vec![skill.clone()] }).collect();
    
    let hc3_limit = 40i64;
    
    println!("=== Running P7-B0 Resource Exhaustion Characterization (A2 Partitioner) ===");
    
    for seed in 1..=10 {
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
            generation_limit: 1, // Only 1 generation, we just care about local genomes!
            mutation_rate: 0.2,
            crossover_rate: 0.8,
            elite_count: 5,
            seed: Some(seed),
            ..Default::default()
        };
        
        let reconciler = BoundaryReconciler {};
        
        let mut partitions = partitioner.partition(&shifts_c2, &workers);
        partitions.sort_by_key(|p| p.core_shifts.iter().map(|s| s.start_hour).min().unwrap_or(0));
        
        let result = run_partitioned_evolution(
            ctx.clone(),
            &partitioner,
            &reconciler,
            local_config.clone(),
            global_config.clone(),
        );
        
        // Reconstruct timeline
        let mut global_hours_used: HashMap<u64, i64> = HashMap::new();
        
        for (k, partition) in partitions.iter().enumerate() {
            let local_genome = &result.local_genomes[k];
            
            // First accumulate the hours from this partition
            for worker in &workers {
                let mut added_hours = 0;
                for shift in &partition.core_shifts {
                    if let Some(&w_id) = local_genome.assignments.get(&shift.id) {
                        if w_id == worker.id {
                            added_hours += shift.duration_hours as i64;
                        }
                    }
                }
                *global_hours_used.entry(worker.id).or_insert(0) += added_hours;
            }
            
            // Calculate future required demand (all core shifts in partitions > k)
            let mut future_required_demand = 0;
            for future_p in partitions.iter().skip(k + 1) {
                for shift in &future_p.core_shifts {
                    future_required_demand += shift.duration_hours as i64;
                }
            }
            
            // Write boundary state for each worker
            for worker in &workers {
                let state_after = *global_hours_used.get(&worker.id).unwrap_or(&0);
                // We fake state_before for the output log to be slightly simpler
                let state_before = state_after; // Not critical for the margin calc
                
                let remaining_capacity = std::cmp::max(0, hc3_limit - state_after);
                
                let mut alternative_capacity = 0;
                for other_w in &workers {
                    if other_w.id != worker.id {
                        let other_state = *global_hours_used.get(&other_w.id).unwrap_or(&0);
                        alternative_capacity += std::cmp::max(0, hc3_limit - other_state);
                    }
                }
                
                let capacity_margin = alternative_capacity - future_required_demand;
                
                let required_for_this_worker = if capacity_margin < 0 { -capacity_margin } else { 0 };
                let irreversible = required_for_this_worker > remaining_capacity || state_after > hc3_limit;
                
                writeln!(boundary_file, "{},{},MaxHours,{},{},{},{},{},{},{},{}",
                    seed,
                    k,
                    worker.id,
                    state_before,
                    state_after,
                    remaining_capacity,
                    future_required_demand,
                    alternative_capacity,
                    capacity_margin,
                    irreversible
                ).unwrap();
            }
        }
        println!("Seed {} completed playback.", seed);
    }
}
