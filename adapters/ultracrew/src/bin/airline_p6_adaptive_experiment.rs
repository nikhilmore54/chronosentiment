use std::fs::File;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use ultracrew::models::{Shift, Worker, Skill};
use ultracrew::ecology::WorkforceEcology;
use ultracrew::public_contracts::InrcScenario;
use ultracrew::optimization::{ScheduleContext, Observatory, run_partitioned_evolution};
use ultracrew::partitioning::{Partitioner, TemporalPartitioner, AdaptiveTemporalPartitioner, BoundaryReconciler};
use coralys_moga::config::EvolutionConfig;

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
    shifts.sort_by_key(|s| s.start_hour);
    shifts
}

fn compute_edges(core: &[Shift], halo: &[Shift]) -> (usize, usize) {
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
    (core_edges, crossover_edges)
}

fn main() -> std::io::Result<()> {
    let shifts_c2 = generate_family_c(0.60);
    let skill = Skill("Pilot".to_string());
    let workers: Vec<Worker> = (0..40).map(|id| Worker { id, skills: vec![skill.clone()] }).collect();
    
    let global_file = Arc::new(Mutex::new(File::create("p6_adaptive_experiment_global.csv")?));
    let parts_file = Arc::new(Mutex::new(File::create("p6_adaptive_experiment_partitions.csv")?));
    
    {
        let mut gf = global_file.lock().unwrap();
        writeln!(gf, "treatment,seed,K,local_feasible_pct,boundary_conflicts,boundary_conflicts_per_boundary,reconciled_feasible,global_feasible_gen_0,final_feasible,optimization_ms").unwrap();
        let mut pf = parts_file.lock().unwrap();
        writeln!(pf, "treatment,seed,partition_id,core_start,core_end,duration_hours,core_shifts,core_shift_hours,core_conflict_edges,crossover_conflict_edges,PU_core,PU_boundary,local_feasible").unwrap();
    }
    
    let num_seeds = 10;
    
    // P2 Baseline
    println!("Running P2 Baseline...");
    for seed in 1..=num_seeds {
        println!("  Seed {}", seed);
        let partitioner = TemporalPartitioner { num_partitions: 8, halo_hours: 24 };
        run_treatment(global_file.clone(), parts_file.clone(), "P2", seed, &shifts_c2, &workers, Box::new(partitioner));
    }
    
    // A2 Adaptive
    println!("Running A2 Adaptive...");
    for seed in 1..=num_seeds {
        println!("  Seed {}", seed);
        let partitioner = AdaptiveTemporalPartitioner { max_core_edges: 50, halo_hours: 24 };
        run_treatment(global_file.clone(), parts_file.clone(), "A2", seed, &shifts_c2, &workers, Box::new(partitioner));
    }
    
    Ok(())
}

fn run_treatment(
    global_file: Arc<Mutex<File>>,
    parts_file: Arc<Mutex<File>>,
    treatment: &str,
    seed: u64,
    shifts: &[Shift],
    workers: &[Worker],
    partitioner: Box<dyn Partitioner>
) {
    let scenario = InrcScenario {
        planning_horizon_hours: Some(168.0),
        max_hours_per_worker: Some(40.0),
        minimum_rest_hours: Some(8),
        leave_requests: None,
    };

    let ctx = Arc::new(ScheduleContext {
        workers: Arc::new(workers.to_vec()),
        shifts: Arc::new(shifts.to_vec()),
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
        generation_limit: 100, // global refinement
        mutation_rate: 0.2,
        crossover_rate: 0.8,
        elite_count: 5,
        seed: Some(seed),
        ..Default::default()
    };
    
    let start = Instant::now();
    let reconciler = BoundaryReconciler;
    let result = run_partitioned_evolution(ctx.clone(), &*partitioner, &reconciler, local_config, global_config);
    let duration_ms = start.elapsed().as_millis();
    
    let partitions = partitioner.partition(shifts, workers);
    let k = partitions.len();
    let local_feasible_pct = (result.local_feasible_count as f64 / result.total_partitions as f64) * 100.0;
    
    let obs = ctx.observatory.lock().unwrap();
    let (global_feasible_gen_0, final_feasible) = if obs.reports.is_empty() {
        (false, false)
    } else {
        let initial = &obs.reports[0];
        let last = obs.reports.last().unwrap();
        (initial.hard_violations == 0, last.hard_violations == 0)
    };
    
    let boundary_conflicts_per_boundary = if k > 1 {
        result.boundary_conflicts as f64 / (k - 1) as f64
    } else {
        0.0
    };
    
    {
        let mut gf = global_file.lock().unwrap();
        writeln!(gf, "{},{},{},{:.1},{},{:.2},{},{},{},{}",
            treatment, seed, k, local_feasible_pct, 
            result.boundary_conflicts, boundary_conflicts_per_boundary, result.reconciled_feasible,
            global_feasible_gen_0, final_feasible, duration_ms
        ).unwrap();
    }
    
    {
        let mut pf = parts_file.lock().unwrap();
        for p in &partitions {
            let min_start = p.core_shifts.iter().map(|s| s.start_hour).min().unwrap_or(0);
            let max_end = p.core_shifts.iter().map(|s| s.start_hour + s.duration_hours).max().unwrap_or(0);
            let duration = if max_end > min_start { max_end - min_start } else { 1 };
            
            let core_shift_hours: u64 = p.core_shifts.iter().map(|s| s.duration_hours).sum();
            
            let (core_edges, crossover_edges) = compute_edges(&p.core_shifts, &p.halo_shifts);
            let pu_core = core_edges as f64 / 50.0;
            let pu_bound = crossover_edges as f64 / 32.0;
            
            let local_feasible = !result.local_failures.contains(&p.id);
            
            writeln!(pf, "{},{},{},{},{},{},{},{},{},{},{:.3},{:.3},{}",
                treatment, seed, p.id,
                min_start, max_end, duration,
                p.core_shifts.len(), core_shift_hours,
                core_edges, crossover_edges,
                pu_core, pu_bound, local_feasible
            ).unwrap();
        }
    }
}
