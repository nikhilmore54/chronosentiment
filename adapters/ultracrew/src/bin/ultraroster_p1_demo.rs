use std::sync::{Arc, Mutex};
use std::time::Instant;
use ultracrew::models::{Shift, Worker, Skill};
use ultracrew::ecology::WorkforceEcology;
use ultracrew::public_contracts::InrcScenario;
use ultracrew::optimization::{ScheduleContext, ScheduleOptimizer, Observatory};
use ultracrew::partitioning::{Partitioner, Phase6CPartitioner, BoundaryReconciler};
use ultracrew::decision_support::DecisionSupportEngine;
use coralys_moga::config::EvolutionConfig;

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
    println!("=== UltraRoster P1: Explore the Decision ===");
    println!("Initializing scheduling scenario...");

    let mut workers = vec![];
    for i in 0..40 {
        workers.push(Worker { id: (i + 1) as u64, skills: vec![Skill("Pilot".to_string())] });
    }
    let shifts_c2 = generate_family_c(0.6);

    let scenario = InrcScenario {
        planning_horizon_hours: Some(168.0),
        max_hours_per_worker: Some(40.0),
        minimum_rest_hours: Some(8),
        leave_requests: None,
    };
    
    let context = Arc::new(ScheduleContext {
        workers: Arc::new(workers),
        shifts: Arc::new(shifts_c2),
        ecology: WorkforceEcology::new(),
        rng_seed: 1,
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
        seed: Some(1),
        ..Default::default()
    };

    let global_config = EvolutionConfig {
        population_size: 100,
        generation_limit: 100,
        mutation_rate: 0.2,
        crossover_rate: 0.8,
        elite_count: 5,
        seed: Some(1),
        ..Default::default()
    };

    let partitioner = Phase6CPartitioner {
        max_core_edges: 50,
        base_halo_hours: 24,
        enable_span_aware_cut: false,
        enable_dynamic_halo: false,
    };

    let reconciler = BoundaryReconciler;

    println!("Executing A2 baseline (local partitions -> global reconciliation)...");
    let start_time = Instant::now();
    
    let result = ultracrew::optimization::run_partitioned_evolution(
        context.clone(),
        &partitioner,
        &reconciler,
        local_config,
        global_config,
    );

    let duration = start_time.elapsed();
    println!("Optimization completed in {:.2}s", duration.as_secs_f64());
    
    println!("Extracting candidate pool (top_10) for Decision Support...");
    let candidate_pool = result.global_result.top_10.clone();
    for (i, eval) in candidate_pool.iter().enumerate() {
        let hards = eval.hc1_violations + eval.hc2_violations + eval.hc3_violations + eval.rest_violations;
        println!("  Candidate {}: valid={}, hard_violations={}, fitness={}", i, eval.is_valid, hards, eval.fitness);
    }
    
    let ds_engine = DecisionSupportEngine::new();
    let decision_result = ds_engine.generate_decision_matrix(candidate_pool);

    println!("\n--- P1 DECISION RESULT JSON ---");
    let json_output = serde_json::to_string_pretty(&decision_result).unwrap();
    println!("{}", json_output);

    println!("\n--- HUMAN READABLE SUMMARY ---");
    if decision_result.alternatives.len() < 3 {
        println!("WARNING: Only {} meaningfully different feasible alternatives were found.", decision_result.alternatives.len());
    } else {
        println!("Found {} meaningfully different feasible alternatives.", decision_result.alternatives.len());
    }

    for alt in &decision_result.alternatives {
        println!("- {}: Coverage {:.1}%, Fairness Penalty: {:.2}, Utilization: {:.2}, Cost: {:.2}", 
            alt.id, alt.metrics.coverage * 100.0, alt.metrics.fairness_penalty, alt.metrics.utilization, alt.metrics.cost);
    }

    if let Some(rec) = decision_result.recommendation {
        println!("\nRECOMMENDED: {}", rec.recommended_id);
        println!("Why:");
        for reason in rec.why {
            println!("  • {}", reason);
        }
    }
}
