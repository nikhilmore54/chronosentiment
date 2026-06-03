use ultracrew::helpers::run_optimization;
use ultracrew::models::{Shift, Skill, Worker};
use ultracrew::optimization::ScheduleContext;
use ultracrew::ecology::WorkforceEcology;
use coralys_moga::config::EvolutionConfig;
use std::sync::Arc;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

fn generate_sequential_scenario(week: usize, ecology: WorkforceEcology, rng: &mut StdRng) -> Arc<ScheduleContext> {
    let mut workers = Vec::new();
    let all_skills = vec![Skill::Forklift, Skill::GeneralLabor, Skill::Supervisor, Skill::FirstAid];
    for i in 0..20 {
        workers.push(Worker {
            id: i as u64,
            skills: vec![all_skills[i % 4], all_skills[(i + 1) % 4]],
        });
    }

    let mut shifts = Vec::new();
    for i in 0..50 {
        shifts.push(Shift {
            id: (week * 1000 + i) as u64,
            start_hour: rng.gen_range(0..160) as u64,
            duration_hours: 8,
            required_skill: all_skills[rng.gen_range(0..4)],
        });
    }

    Arc::new(ScheduleContext {
        workers: Arc::new(workers),
        shifts: Arc::new(shifts),
        ecology,
    })
}

fn run_4_weeks(enable_memory: bool, base_seed: u64) -> (f64, f64) {
    let mut rng = StdRng::seed_from_u64(base_seed);
    let mut ecology = WorkforceEcology::new();
    
    // Track actual cumulative hours worked by each worker over the 4 weeks
    let mut cumulative_hours = vec![0.0; 20];

    let config = EvolutionConfig {
        population_size: 50,
        generation_limit: 50,
        mutation_rate: 0.1,
        crossover_rate: 0.7,
        elite_count: 5,
        seed: Some(base_seed),
    };

    for week in 0..4 {
        if !enable_memory {
            ecology = WorkforceEcology::new();
        }

        let context = generate_sequential_scenario(week, ecology.clone(), &mut rng);
        let result = run_optimization(context.clone(), config.clone());
        let best = result.global_best;

        let mut week_hours = vec![0.0; 20];
        for (shift_id, worker_id) in &best.schedule.assignments {
            week_hours[*worker_id as usize] += 8.0;
            cumulative_hours[*worker_id as usize] += 8.0;
        }

        if enable_memory {
            for i in 0..20 {
                ecology.record_historical_hours(i as u64, week_hours[i]);
            }
        }
    }

    // Metric 1: True Long-Term Fairness (Variance of cumulative hours)
    let mean_hours = cumulative_hours.iter().sum::<f64>() / 20.0;
    let variance = cumulative_hours.iter().map(|h| (h - mean_hours).powi(2)).sum::<f64>() / 20.0;

    // Metric 2: True Long-Term Fatigue (Max hours worked by any single worker)
    let max_hours = cumulative_hours.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    (max_hours, variance)
}

#[test]
fn test_level4_ecology_ablation() {
    let num_runs = 30;

    let mut mem_max_fatigue = Vec::new();
    let mut mem_variance = Vec::new();
    
    let mut no_mem_max_fatigue = Vec::new();
    let mut no_mem_variance = Vec::new();

    for seed in 1..=num_runs {
        let (max_fatigue_mem, var_mem) = run_4_weeks(true, seed as u64);
        let (max_fatigue_no_mem, var_no_mem) = run_4_weeks(false, seed as u64);

        mem_max_fatigue.push(max_fatigue_mem);
        mem_variance.push(var_mem);
        
        no_mem_max_fatigue.push(max_fatigue_no_mem);
        no_mem_variance.push(var_no_mem);
    }

    let mean_mem_max_fatigue: f64 = mem_max_fatigue.iter().sum::<f64>() / num_runs as f64;
    let mean_no_mem_max_fatigue: f64 = no_mem_max_fatigue.iter().sum::<f64>() / num_runs as f64;

    let mean_mem_variance: f64 = mem_variance.iter().sum::<f64>() / num_runs as f64;
    let mean_no_mem_variance: f64 = no_mem_variance.iter().sum::<f64>() / num_runs as f64;

    let fatigue_improvement = ((mean_no_mem_max_fatigue - mean_mem_max_fatigue) / mean_no_mem_max_fatigue) * 100.0;
    let fairness_improvement = ((mean_no_mem_variance - mean_mem_variance) / mean_no_mem_variance) * 100.0;

    println!("--- LEVEL 4: ECOLOGY ABLATION (30 PAIRED RUNS) ---");
    println!("Memory Disabled - Mean Max 4-Week Hours: {:.2}, Mean 4-Week Variance: {:.2}", mean_no_mem_max_fatigue, mean_no_mem_variance);
    println!("Memory Enabled  - Mean Max 4-Week Hours: {:.2}, Mean 4-Week Variance: {:.2}", mean_mem_max_fatigue, mean_mem_variance);
    println!("Fatigue (Max Hours) Improvement:  {:.2}%", fatigue_improvement);
    println!("Fairness (Variance) Improvement: {:.2}%", fairness_improvement);

    assert!(
        fairness_improvement >= 10.0,
        "MemoryState failed to improve long-term fairness by at least 10%. Actual: {:.2}%",
        fairness_improvement
    );
}
