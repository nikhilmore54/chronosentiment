use std::fs;
use std::sync::Arc;
use std::time::Instant;
use std::collections::HashMap;
use chrono::Utc;
use serde::{Serialize, Deserialize};

use cvrp::{CvrpInstance, CvrpGenomeFactory};
use cvrp::moga_impl::{CvrpMutator, CvrpCrossoverRoutePreserving, CvrpLocalSearch, CvrpEvaluator};
use coralys_moga::{
    EvolutionConfig, EvolutionEngineBuilder, ProcessingMetricsCollector, MogaBenchmarkReport,
    SolutionQuality, ExecutionMetrics, EngineMetrics, ConvergenceMetrics,
};

const BKS_DISTANCE: f64 = 784.0;
const BENCHMARK_DIR: &str = "benchmarks/history";

#[derive(Serialize, Deserialize, Clone, Debug)]
struct BenchmarkIndexEntry {
    milestone: String,
    timestamp: String,
    best_distance: f64,
    gap_to_bks: f64,
    runtime_ms: u128,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct BenchmarkIndex {
    entries: Vec<BenchmarkIndexEntry>,
}

fn main() {
    println!("Starting Benchmark-Driven Development (BDD) validation run...");

    // Create benchmarks/history directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(BENCHMARK_DIR) {
        eprintln!("Failed to create benchmark directory: {}", e);
        std::process::exit(1);
    }

    // 1. Setup the Canonical CVRP A-n32-k5 Instance with Official TSPLIB metric
    let mut instance = CvrpInstance::a_n32_k5();
    instance.distance_metric = cvrp::DistanceMetric::TspLibEuc2D;
    let evaluator = CvrpEvaluator { instance: instance.clone() };
    let mutator = CvrpMutator::new(instance.clone(), cvrp::RadiusPolicy::Control);
    let crossover = CvrpCrossoverRoutePreserving { instance: instance.clone() };
    let factory = CvrpGenomeFactory { num_customers: instance.customers.len() };
    let local_search = CvrpLocalSearch { instance: instance.clone() };

    let evo_config = EvolutionConfig {
        population_size: 200,
        elite_count: 20,
        generation_limit: 150,
        mutation_rate: 0.2,
        crossover_rate: 0.8,
        seed: Some(42),
        tournament_size: Some(5),
        ..Default::default()
    };

    // Attach passive metrics observer
    let observer_metrics = Arc::new(ProcessingMetricsCollector::new());

    let engine = EvolutionEngineBuilder::new()
        .with_evaluator(evaluator)
        .with_mutator(mutator)
        .with_crossover(crossover)
        .with_factory(factory)
        .with_improvement(local_search)
        .with_observer(observer_metrics.clone())
        .enable_metrics()
        .build()
        .expect("Failed to build EvolutionEngine");

    // 2. Execute GA & Measure
    let start_time = Instant::now();
    let _ga_res = engine.run_ga_evolution(evo_config.clone())
        .expect("GA execution failed");
    let total_runtime = start_time.elapsed();

    // 3. Load structured metrics from collector
    let m = engine.metrics_snapshot().expect("metrics should be enabled");

    // Solution Quality (translating from GA fitness y = 100000.0 - distance)
    let best_distance = 100000.0 - m.best_fitness;
    let average_distance = 100000.0 - m.average_fitness;
    let worst_distance = 100000.0 - m.worst_fitness;
    let gap_to_bks = ((best_distance - BKS_DISTANCE) / BKS_DISTANCE) * 100.0;
    
    // Median distance from final fitnesses: wait, since we don't have final_fitnesses in m, we can calculate it from ga_res, or just estimate. Wait! We do have ga_res!
    let final_fitnesses = &_ga_res.final_fitnesses;
    let mut sorted_fitnesses = final_fitnesses.clone();
    sorted_fitnesses.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median_fitness = if sorted_fitnesses.len() % 2 == 0 {
        let mid = sorted_fitnesses.len() / 2;
        (sorted_fitnesses[mid - 1] + sorted_fitnesses[mid]) / 2.0
    } else {
        sorted_fitnesses[sorted_fitnesses.len() / 2]
    };
    let median_distance = 100000.0 - median_fitness;

    // Convergence
    let best_distances_per_gen: Vec<f64> = m.best_history.iter()
        .map(|&f| 100000.0 - f)
        .collect();
    let avg_distances_per_gen: Vec<f64> = m.average_history.iter()
        .map(|&f| 100000.0 - f)
        .collect();

    // Engine metrics from EvolutionMetrics map
    let num_processors = engine.processor_count();
    let mut proc_times = HashMap::new();
    let mut proc_calls = HashMap::new();
    let mut total_proc_time_ms = 0.0;

    for i in 0..num_processors {
        if let Some(proc_m) = m.processors.get(&i) {
            let time_ms = proc_m.total_runtime.as_secs_f64() * 1000.0;
            proc_times.insert(i, time_ms);
            proc_calls.insert(i, proc_m.invocation_count);
            total_proc_time_ms += time_ms;
        } else {
            proc_times.insert(i, 0.0);
            proc_calls.insert(i, 0);
        }
    }

    // Determine milestone numbering
    let milestone_num = get_next_milestone_number();
    let milestone_name = format!("milestone-{:03}", milestone_num);

    let report = MogaBenchmarkReport {
        milestone: milestone_name.clone(),
        timestamp: Utc::now().to_rfc3339(),
        solution_quality: SolutionQuality {
            best_fitness: best_distance,
            average_fitness: average_distance,
            worst_fitness: worst_distance,
            gap_to_bks,
        },
        execution_metrics: ExecutionMetrics {
            runtime_ms: total_runtime.as_millis(),
            evaluations: m.evaluation_count,
            generations: m.generation + 1,
            population_size: evo_config.population_size,
        },
        engine_metrics: EngineMetrics {
            num_processors_executed: num_processors,
            processor_execution_time_ms: proc_times,
            processor_invocation_counts: proc_calls,
            processing_overhead_ms: total_proc_time_ms,
            observer_overhead_ms: 0.0,
        },
        convergence_metrics: ConvergenceMetrics {
            best_fitness_per_generation: best_distances_per_gen,
            average_fitness_per_generation: avg_distances_per_gen,
            diversity: Some(m.fitness_stddev),
            stagnation_generation: m.stagnation_generations,
        },
    };

    // Load previous milestone for comparison
    let mut prev_report: Option<MogaBenchmarkReport> = None;
    if milestone_num > 1 {
        let prev_name = format!("milestone-{:03}", milestone_num - 1);
        let prev_path = format!("{}/{}.json", BENCHMARK_DIR, prev_name);
        if let Ok(content) = fs::read_to_string(&prev_path) {
            if let Ok(parsed) = serde_json::from_str::<MogaBenchmarkReport>(&content) {
                prev_report = Some(parsed);
            }
        }
    }

    // Detect Regression
    let mut regression = false;
    let mut improvement = false;
    if let Some(ref prev) = prev_report {
        let delta = report.solution_quality.best_fitness - prev.solution_quality.best_fitness;
        if delta > 0.01 {
            regression = true;
        } else if delta < -0.01 {
            improvement = true;
        }
    }

    // Save report to milestones dir
    let report_path = format!("{}/{}.json", BENCHMARK_DIR, milestone_name);
    let report_json = serde_json::to_string_pretty(&report).unwrap();
    fs::write(&report_path, &report_json).expect("Failed to write report JSON file");

    // Update Index
    update_benchmark_index(&report);

    // Extract dual distance values from the best solution
    let best_eval = &_ga_res.global_best.eval;
    let best_dist_integer = best_eval.total_distance_integer;
    let best_dist_float = best_eval.total_distance_float;

    // Write Markdown Milestone Report Output
    let md_report = generate_markdown_report(
        &report,
        prev_report.as_ref(),
        regression,
        improvement,
        m.fitness_stddev,
        median_distance,
        best_dist_integer,
        best_dist_float,
    );
    let md_report_path = format!("{}/{}.md", BENCHMARK_DIR, milestone_name);
    fs::write(&md_report_path, &md_report).expect("Failed to write report Markdown file");

    println!("{}", md_report);

    // ── A-n46-k7 Baseline Run ──────────────────────────────────────────────────
    // Identical optimizer configuration. BKS=914 (TspLibEuc2D), 45 customers, 7 vehicles.
    // Source: https://galgos.inf.puc-rio.br/cvrplib/index.php/en/download/instance/18
    println!("=== A-n46-k7 Baseline (identical config) ===\n");
    const BKS_A_N46_K7: f64 = 914.0;

    let mut instance_46 = CvrpInstance::a_n46_k7();
    instance_46.distance_metric = cvrp::DistanceMetric::TspLibEuc2D;
    let evaluator_46 = CvrpEvaluator { instance: instance_46.clone() };
    let mutator_46 = CvrpMutator::new(instance_46.clone(), cvrp::RadiusPolicy::Control);
    let crossover_46 = CvrpCrossoverRoutePreserving { instance: instance_46.clone() };
    let factory_46 = CvrpGenomeFactory { num_customers: instance_46.customers.len() };
    let local_search_46 = CvrpLocalSearch { instance: instance_46.clone() };

    let engine_46 = EvolutionEngineBuilder::new()
        .with_evaluator(evaluator_46)
        .with_mutator(mutator_46)
        .with_crossover(crossover_46)
        .with_factory(factory_46)
        .with_improvement(local_search_46)
        .enable_metrics()
        .build()
        .expect("Failed to build EvolutionEngine for A-n46-k7");

    let start_46 = Instant::now();
    let ga_res_46 = engine_46.run_ga_evolution(evo_config.clone())
        .expect("GA execution failed for A-n46-k7");
    let runtime_46 = start_46.elapsed();

    let m46 = engine_46.metrics_snapshot().expect("metrics should be enabled");
    let best_46 = 100000.0 - m46.best_fitness;
    let avg_46  = 100000.0 - m46.average_fitness;
    let worst_46 = 100000.0 - m46.worst_fitness;
    let gap_46 = (best_46 - BKS_A_N46_K7) / BKS_A_N46_K7 * 100.0;

    let mut sorted_46 = ga_res_46.final_fitnesses.clone();
    sorted_46.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median_46 = if sorted_46.len() % 2 == 0 {
        let mid = sorted_46.len() / 2;
        (sorted_46[mid - 1] + sorted_46[mid]) / 2.0
    } else {
        sorted_46[sorted_46.len() / 2]
    };
    let median_dist_46 = 100000.0 - median_46;

    let best_eval_46 = &ga_res_46.global_best.eval;
    let feasible_46 = best_eval_46.total_distance_integer < 99000.0;

    let num_proc_46 = engine_46.processor_count();
    let mut proc_stats_46 = String::new();
    for i in 0..num_proc_46 {
        if let Some(pm) = m46.processors.get(&i) {
            let time_ms = pm.total_runtime.as_secs_f64() * 1000.0;
            let avg_ms = if pm.invocation_count > 0 { time_ms / pm.invocation_count as f64 } else { 0.0 };
            proc_stats_46.push_str(&format!(
                "| Processor #{} | {} | {:.4} ms | {:.2} ms |\n",
                i, pm.invocation_count, avg_ms, time_ms
            ));
        }
    }

    println!(
        r#"| Metric | Value |
| :--- | :--- |
| **Instance** | A-n46-k7 |
| **BKS (TspLibEuc2D)** | {bks:.2} |
| **Best Distance** | {best:.4} |
| **Average Distance** | {avg:.4} |
| **Worst Distance** | {worst:.4} |
| **Median Distance** | {median:.4} |
| **Std Dev** | {stddev:.4} |
| **Gap to BKS** | {gap:.2}% |
| **Feasible** | {feasible} |
| **Runtime** | {runtime} ms |
| **Evaluations** | {evals} |
| **Generations** | {gens} |
| **Stagnation Gen** | {stagnation} |
| **Population** | {pop} |
| **Elite** | {elite} |
| **Seed** | 42 |

## Processor Statistics (A-n46-k7)
| Processor | Invocations | Average Runtime | Total Runtime |
| :--- | :--- | :--- | :--- |
{proc_stats}
"#,
        bks = BKS_A_N46_K7,
        best = best_46,
        avg = avg_46,
        worst = worst_46,
        median = median_dist_46,
        stddev = m46.fitness_stddev,
        gap = gap_46,
        feasible = if feasible_46 { "YES" } else { "NO (INFEASIBLE)" },
        runtime = runtime_46.as_millis(),
        evals = m46.evaluation_count,
        gens = m46.generation + 1,
        stagnation = m46.stagnation_generations,
        pop = evo_config.population_size,
        elite = evo_config.elite_count,
        proc_stats = proc_stats_46,
    );

    // Save A-n46-k7 baseline to benchmarks/history
    let baseline_46_path = format!("{}/a_n46_k7_baseline.json", BENCHMARK_DIR);
    let baseline_46_json = serde_json::json!({
        "instance": "A-n46-k7",
        "bks": BKS_A_N46_K7,
        "best_distance": best_46,
        "average_distance": avg_46,
        "worst_distance": worst_46,
        "median_distance": median_dist_46,
        "std_dev": m46.fitness_stddev,
        "gap_to_bks_pct": gap_46,
        "feasible": feasible_46,
        "runtime_ms": runtime_46.as_millis(),
        "evaluations": m46.evaluation_count,
        "generations": m46.generation + 1,
        "stagnation_generation": m46.stagnation_generations,
        "config": {
            "population_size": evo_config.population_size,
            "elite_count": evo_config.elite_count,
            "generation_limit": evo_config.generation_limit,
            "mutation_rate": evo_config.mutation_rate,
            "crossover_rate": evo_config.crossover_rate,
            "seed": 42,
            "tournament_size": 5
        }
    });
    fs::write(&baseline_46_path, serde_json::to_string_pretty(&baseline_46_json).unwrap())
        .expect("Failed to write A-n46-k7 baseline JSON");
    println!("A-n46-k7 baseline saved to {}", baseline_46_path);
}

fn get_next_milestone_number() -> usize {
    let mut max_num = 0;
    if let Ok(entries) = fs::read_dir(BENCHMARK_DIR) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                if stem.starts_with("milestone-") {
                    if let Ok(num) = stem["milestone-".len()..].parse::<usize>() {
                        if num > max_num {
                            max_num = num;
                        }
                    }
                }
            }
        }
    }
    max_num + 1
}

fn update_benchmark_index(report: &MogaBenchmarkReport) {
    let index_path = format!("{}/index.json", BENCHMARK_DIR);
    let mut index = if let Ok(content) = fs::read_to_string(&index_path) {
        serde_json::from_str::<BenchmarkIndex>(&content).unwrap_or_default()
    } else {
        BenchmarkIndex::default()
    };

    index.entries.push(BenchmarkIndexEntry {
        milestone: report.milestone.clone(),
        timestamp: report.timestamp.clone(),
        best_distance: report.solution_quality.best_fitness,
        gap_to_bks: report.solution_quality.gap_to_bks,
        runtime_ms: report.execution_metrics.runtime_ms,
    });

    let index_json = serde_json::to_string_pretty(&index).unwrap();
    fs::write(&index_path, &index_json).expect("Failed to write index.json");
}

fn generate_markdown_report(
    report: &MogaBenchmarkReport,
    prev: Option<&MogaBenchmarkReport>,
    regression: bool,
    improvement: bool,
    stddev: f64,
    median_distance: f64,
    best_dist_integer: f64,
    best_dist_float: f64,
) -> String {
    let prev_best_str = prev.map(|p| format!("{:.4}", p.solution_quality.best_fitness))
        .unwrap_or_else(|| "N/A".to_string());
    
    let regression_str = if regression { "⚠️ YES (REGRESSION DETECTED)" } else { "NO" };
    let improvement_str = if improvement { "✅ YES" } else { "NO" };

    let mut proc_stats = String::new();
    for i in 0..report.engine_metrics.num_processors_executed {
        let count = report.engine_metrics.processor_invocation_counts.get(&i).unwrap_or(&0);
        let time = report.engine_metrics.processor_execution_time_ms.get(&i).unwrap_or(&0.0);
        let avg = if *count > 0 { time / (*count as f64) } else { 0.0 };
        proc_stats.push_str(&format!(
            "| Processor #{} | {} | {:.4} ms | {:.2} ms |\n",
            i, count, avg, time
        ));
    }

    format!(
        r#"# BDD Milestone Report: {milestone}

| Metric | Current Value | Previous Value |
| :--- | :--- | :--- |
| **Milestone** | {milestone} | - |
| **Timestamp** | {timestamp} | - |
| **Dataset** | CVRP A-n32-k5 | - |
| **Official TSPLIB (Benchmark)** | {best_dist_integer:.2} | - |
| **Floating Euclidean (Research)** | {best_dist_float:.4} | - |
| **Best Distance (Active Metric)** | {best_dist:.4} | {prev_best} |
| **Average Distance**| {avg_dist:.4} | - |
| **Worst Distance**  | {worst_dist:.4} | - |
| **Median Distance** | {median_dist:.4} | - |
| **Std Dev (Diversity)**| {stddev:.4} | - |
| **Gap to BKS (784.0)** | {gap:.2}% | - |
| **Runtime** | {runtime} ms | - |
| **Evaluations** | {evals} | - |
| **Generations** | {gens} | - |
| **Stagnation Gen** | {stagnation} | - |
| **Regression** | {regression} | - |
| **Improvement** | {improvement} | - |

## Processor Statistics
| Processor | Invocations | Average Runtime | Total Runtime |
| :--- | :--- | :--- | :--- |
{proc_stats}
## Observer Statistics
- Total Processing Overhead: {processing_overhead:.2} ms
- Observer Overhead: {observer_overhead:.2} ms

## Notes
- Evolved using Benchmark-Driven Development protocols.
- Stopping criteria: 150 generations.
- Random seed: 42.
"#,
        milestone = report.milestone,
        timestamp = report.timestamp,
        best_dist = report.solution_quality.best_fitness,
        prev_best = prev_best_str,
        avg_dist = report.solution_quality.average_fitness,
        worst_dist = report.solution_quality.worst_fitness,
        median_dist = median_distance,
        stddev = stddev,
        gap = report.solution_quality.gap_to_bks,
        runtime = report.execution_metrics.runtime_ms,
        evals = report.execution_metrics.evaluations,
        gens = report.execution_metrics.generations,
        stagnation = report.convergence_metrics.stagnation_generation,
        regression = regression_str,
        improvement = improvement_str,
        proc_stats = proc_stats,
        processing_overhead = report.engine_metrics.processing_overhead_ms,
        observer_overhead = report.engine_metrics.observer_overhead_ms,
        best_dist_integer = best_dist_integer,
        best_dist_float = best_dist_float,
    )
}
