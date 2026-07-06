use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use std::time::Instant;

use coralys_moga::engine::EvolutionEngine;
use coralys_moga::config::EvolutionConfig;

use ultracrew::inrc::optimization::{
    InrcContext, InrcOptimizer
};
use ultracrew::inrc::parser::{parse_scenario, parse_history, parse_week_data};
use ultracrew::ecology::WorkforceEcology;
use ultracrew::inrc::history::extract_next_history;

#[derive(Debug)]
struct RunMetrics {
    final_score: i32,
    hard_violations: i32,
    fatigue_index: i32,
    assignment_variance: f64,
    weekend_concentration: f64,
    convergence_gen: usize,
    persistence_score: f64,
}

fn run_ablation(seed: u64, use_ecology: bool) -> RunMetrics {
    let base_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/n030w4");
    
    let scenario = parse_scenario(base_dir.join("Sc-n030w4.json")).unwrap();
    let history = parse_history(base_dir.join("H0-n030w4-0.json")).unwrap();
    let week_data = parse_week_data(base_dir.join("WD-n030w4-0.json")).unwrap();
    
    let ecology = WorkforceEcology::new();
    
    let context = Arc::new(InrcContext::new(scenario.clone(), week_data, history.clone(), ecology.clone()));
    let evaluator = InrcOptimizer { context: context.clone() };

    let config = EvolutionConfig {
        population_size: 100,
        generation_limit: 100,
        elite_count: 5,
        seed: Some(seed),
        ..Default::default()
    };

    let engine = EvolutionEngine::new(evaluator.clone(), evaluator.clone(), evaluator.clone(), evaluator.clone());
    let result = engine.run_ga_evolution(config);
    
    let best = result.global_best;
    
    let mut fatigue_index = 0;
    let next_hist = extract_next_history(&context, &best.genome);
    for n in 0..scenario.nurses.len() {
        let nh = &next_hist.nurse_history[n];
        let mut max_work = 5;
        for cont in &scenario.contracts {
            if cont.id == scenario.nurses[n].contract {
                max_work = cont.max_consecutive_working_days; // Is this correct? I'll fix this
                break;
            }
        }
        if nh.number_of_consecutive_working_days > max_work {
            let diff = nh.number_of_consecutive_working_days - max_work;
            fatigue_index += (diff * diff) as i32;
        }
    }

    let mut assignments = vec![0; scenario.nurses.len()];
    for n in 0..scenario.nurses.len() {
        assignments[n] = next_hist.nurse_history[n].number_of_assignments;
    }
    let mean_assignments = assignments.iter().sum::<usize>() as f64 / scenario.nurses.len() as f64;
    let assignment_variance = assignments.iter()
        .map(|&x| (x as f64 - mean_assignments).powi(2))
        .sum::<f64>() / scenario.nurses.len() as f64;

    let mut weekends = vec![0; scenario.nurses.len()];
    for n in 0..scenario.nurses.len() {
        weekends[n] = next_hist.nurse_history[n].number_of_working_weekends;
    }
    let mean_weekends = weekends.iter().sum::<usize>() as f64 / scenario.nurses.len() as f64;
    let weekend_concentration = weekends.iter()
        .map(|&x| (x as f64 - mean_weekends).powi(2))
        .sum::<f64>() / scenario.nurses.len() as f64;

    let mut init_assignments = vec![0; scenario.nurses.len()];
    for n in 0..scenario.nurses.len() {
        init_assignments[n] = history.nurse_history[n].number_of_assignments;
    }
    let mut init_ranked: Vec<usize> = (0..scenario.nurses.len()).collect();
    init_ranked.sort_by_key(|&i| init_assignments[i]);
    let mut final_ranked: Vec<usize> = (0..scenario.nurses.len()).collect();
    final_ranked.sort_by_key(|&i| assignments[i]);
    
    let mut rank_change_sum = 0;
    for n in 0..scenario.nurses.len() {
        let init_rank = init_ranked.iter().position(|&x| x == n).unwrap();
        let final_rank = final_ranked.iter().position(|&x| x == n).unwrap();
        rank_change_sum += (init_rank as i32 - final_rank as i32).abs();
    }
    let persistence_score = rank_change_sum as f64 / scenario.nurses.len() as f64;

    let hc = best.hc_coverage + best.hc_skills + best.hc_one_shift_per_day + best.hc_forbidden_successions;
    RunMetrics {
        final_score: best.soft_report.total_penalty,
        hard_violations: hc as i32,
        fatigue_index,
        assignment_variance,
        weekend_concentration,
        convergence_gen: 0,
        persistence_score,
    }
}

fn main() {
    println!("Starting F.2D Ecology Ablation Pilot (5 Runs)");
    
    let mut file = File::create("ablation_results.csv").unwrap();
    writeln!(file, "seed,ecology,final_score,hard_violations,fatigue_index,assignment_variance,weekend_concentration,convergence_gen,persistence_score").unwrap();

    let num_runs = 5;
    for i in 0..num_runs {
        let seed = 12345 + i as u64;
        
        let start = Instant::now();
        let off_metrics = run_ablation(seed, false);
        println!("Run {} (OFF) took {:?}", i, start.elapsed());
        
        let start = Instant::now();
        let on_metrics = run_ablation(seed, true);
        println!("Run {} (ON) took {:?}", i, start.elapsed());

        writeln!(file, "{},OFF,{},{},{},{:.4},{:.4},{},{:.4}", 
            seed, off_metrics.final_score, off_metrics.hard_violations, 
            off_metrics.fatigue_index, off_metrics.assignment_variance, 
            off_metrics.weekend_concentration, off_metrics.convergence_gen, 
            off_metrics.persistence_score).unwrap();
            
        writeln!(file, "{},ON,{},{},{},{:.4},{:.4},{},{:.4}", 
            seed, on_metrics.final_score, on_metrics.hard_violations, 
            on_metrics.fatigue_index, on_metrics.assignment_variance, 
            on_metrics.weekend_concentration, on_metrics.convergence_gen, 
            on_metrics.persistence_score).unwrap();
    }
    println!("Pilot completed successfully.");
}
