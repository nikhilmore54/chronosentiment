use std::fs;
use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::time::Instant;
use serde::{Serialize, Deserialize};

use cvrp::{CvrpInstance, CvrpGenomeFactory, DistanceMetric, RadiusPolicy, Node};
use cvrp::moga_impl::{CvrpEvaluator, CvrpMutator, CvrpCrossover, CvrpLocalSearch};
use coralys_moga::{EvolutionConfig, EvolutionEngineBuilder};

#[derive(Serialize, Deserialize, Clone, Debug)]
struct InstanceMetadata {
    instance_id: usize,
    bks_id: usize,
    family: String,
    name: String,
    customers: usize,
    vehicles: usize,
    capacity: usize,
    bks: f64,
    instance_url: String,
    bks_url: String,
}

struct BaselineTask {
    metadata: InstanceMetadata,
    vrp_content: String,
}

struct BaselineResult {
    metadata: InstanceMetadata,
    coralys_result: f64,
    runtime_sec: f64,
    status: String,
}

fn parse_vrp_file(content: &str) -> CvrpInstance {
    let mut capacity = 0;
    let mut coords = Vec::new();
    let mut demands = Vec::new();
    
    let mut section = "";
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }
        if line.starts_with("CAPACITY") {
            let parts: Vec<&str> = line.split(':').collect();
            capacity = parts[1].trim().parse().unwrap();
        } else if line.starts_with("NODE_COORD_SECTION") {
            section = "coords";
            continue;
        } else if line.starts_with("DEMAND_SECTION") {
            section = "demands";
            continue;
        } else if line.starts_with("DEPOT_SECTION") || line.starts_with("EOF") {
            section = "";
            continue;
        }
        
        if section == "coords" {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let id: usize = parts[0].parse().unwrap();
                let x: f64 = parts[1].parse().unwrap();
                let y: f64 = parts[2].parse().unwrap();
                coords.push((id, x, y));
            }
        } else if section == "demands" {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let id: usize = parts[0].parse().unwrap();
                let demand: i32 = parts[1].parse().unwrap();
                demands.push((id, demand));
            }
        }
    }
    
    // Find depot (typically node with id 1)
    let depot_coord = coords.iter().find(|(id, _, _)| *id == 1).unwrap();
    let depot_demand = demands.iter().find(|(id, _)| *id == 1).unwrap();
    let depot = Node { id: depot_coord.0, x: depot_coord.1, y: depot_coord.2, demand: depot_demand.1 };
    
    let mut customers = Vec::new();
    for coord in coords {
        if coord.0 == 1 { continue; }
        let demand = demands.iter().find(|(id, _)| *id == coord.0).unwrap();
        customers.push(Node { id: coord.0, x: coord.1, y: coord.2, demand: demand.1 });
    }
    
    CvrpInstance {
        capacity,
        depot,
        customers,
        distance_metric: DistanceMetric::TspLibEuc2D,
        max_vehicles: None,
    }
}

fn main() {
    println!("Starting official CVRPLIB Baseline Evaluation...");
    
    // Load metadata
    let metadata_path = "/Users/nikhil/.gemini/antigravity/brain/262ffe5d-aed4-43c6-a002-28b6911113bc/scratch/instances.json";
    let metadata_json = fs::read_to_string(metadata_path).expect("Failed to read instances.json");
    let all_instances: Vec<InstanceMetadata> = serde_json::from_str(&metadata_json).expect("Failed to parse instances.json");
    
    // Filter for Augerat A, B, and P
    let augerat_instances: Vec<InstanceMetadata> = all_instances.into_iter()
        .filter(|inst| inst.family == "A" || inst.family == "B" || inst.family == "P")
        .collect();
        
    println!("Found {} Augerat instances to process.", augerat_instances.len());
    
    // Construct tasks
    let mut tasks = Vec::new();
    let instances_dir = "/Users/nikhil/ChronoSentiment_MEGA_FINAL/adapters/cvrp/data/instances";
    for inst in augerat_instances {
        let vrp_path = format!("{}/{}.vrp", instances_dir, inst.name);
        if Path::new(&vrp_path).exists() {
            let vrp_content = fs::read_to_string(&vrp_path).unwrap();
            tasks.push(BaselineTask {
                metadata: inst,
                vrp_content,
            });
        } else {
            eprintln!("Warning: instance file not found for {}", inst.name);
        }
    }
    
    let num_tasks = tasks.len();
    println!("Loaded {} valid tasks for execution.", num_tasks);
    
    // Set up thread pool channels
    let (tx_task, rx_task) = mpsc::channel::<Option<BaselineTask>>();
    let (tx_res, rx_res) = mpsc::channel::<BaselineResult>();
    
    let rx_task = std::sync::Arc::new(std::sync::Mutex::new(rx_task));
    let num_workers = 8; // Concurrency limit
    
    let mut workers = Vec::new();
    for worker_id in 0..num_workers {
        let rx_task = rx_task.clone();
        let tx_res = tx_res.clone();
        
        let handle = thread::spawn(move || {
            loop {
                // Get next task
                let task_opt = {
                    let lock = rx_task.lock().unwrap();
                    lock.recv().unwrap()
                };
                
                let task = match task_opt {
                    Some(t) => t,
                    None => break, // Poison pill
                };
                
                let start = Instant::now();
                let mut instance = parse_vrp_file(&task.vrp_content);
                instance.max_vehicles = Some(task.metadata.vehicles);
                
                let evaluator = CvrpEvaluator { instance: instance.clone() };
                let mutator = CvrpMutator::new(instance.clone(), RadiusPolicy::Control);
                let crossover = CvrpCrossover;
                let factory = CvrpGenomeFactory { num_customers: instance.customers.len() };
                let local_search = CvrpLocalSearch { instance: instance.clone() };
                
                let config = EvolutionConfig {
                    population_size: 200,
                    elite_count: 20,
                    generation_limit: 50,
                    mutation_rate: 0.2,
                    crossover_rate: 0.8,
                    seed: Some(42),
                    tournament_size: Some(5),
                    ..Default::default()
                };
                
                let engine_res = EvolutionEngineBuilder::new()
                    .with_evaluator(evaluator)
                    .with_mutator(mutator)
                    .with_crossover(crossover)
                    .with_factory(factory)
                    .with_improvement(local_search)
                    .build();
                    
                match engine_res {
                    Ok(engine) => {
                        match engine.run_ga_evolution(config) {
                            Ok(res) => {
                                let coralys_result = res.global_best.eval.total_distance_integer;
                                let runtime_sec = start.elapsed().as_secs_f64();
                                
                                // Rigorous feasibility check
                                let mut is_feasible = true;
                                let mut visited = std::collections::HashSet::new();
                                let mut total_visited = 0;
                                let mut capacity_ok = true;
                                
                                for route in &res.global_best.eval.routes {
                                    let mut load = 0;
                                    for &node_id in route {
                                        if node_id == instance.depot.id {
                                            is_feasible = false;
                                        }
                                        if let Some(cust) = instance.customers.iter().find(|c| c.id == node_id) {
                                            load += cust.demand;
                                            visited.insert(node_id);
                                            total_visited += 1;
                                        } else {
                                            is_feasible = false;
                                        }
                                    }
                                    if load > instance.capacity {
                                        capacity_ok = false;
                                    }
                                }
                                
                                let customer_coverage_ok = visited.len() == instance.customers.len() && total_visited == instance.customers.len();
                                let vehicle_count_ok = res.global_best.eval.routes.len() <= task.metadata.vehicles;
                                
                                let status = if !customer_coverage_ok || !capacity_ok || !vehicle_count_ok || !is_feasible {
                                    "INFEASIBLE".to_string()
                                } else {
                                    let diff = coralys_result - task.metadata.bks;
                                    if diff.abs() < 0.01 {
                                        "MATCH".to_string()
                                    } else if diff > 0.0 {
                                        "ABOVE BKS".to_string()
                                    } else {
                                        "BELOW BKS".to_string()
                                    }
                                };
                                
                                tx_res.send(BaselineResult {
                                    metadata: task.metadata,
                                    coralys_result,
                                    runtime_sec,
                                    status,
                                }).unwrap();
                            }
                            Err(e) => {
                                eprintln!("GA error on {}: {}", task.metadata.name, e);
                                tx_res.send(BaselineResult {
                                    metadata: task.metadata,
                                    coralys_result: 0.0,
                                    runtime_sec: start.elapsed().as_secs_f64(),
                                    status: "FAILED".to_string(),
                                }).unwrap();
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Engine build error on {}: {}", task.metadata.name, e);
                        tx_res.send(BaselineResult {
                            metadata: task.metadata,
                            coralys_result: 0.0,
                            runtime_sec: start.elapsed().as_secs_f64(),
                            status: "FAILED".to_string(),
                        }).unwrap();
                    }
                }
            }
        });
        workers.push(handle);
    }
    
    // Dispatch tasks
    for task in tasks {
        tx_task.send(Some(task)).unwrap();
    }
    // Poison pills
    for _ in 0..num_workers {
        tx_task.send(None).unwrap();
    }
    
    // Collect results
    let mut results = Vec::new();
    for _ in 0..num_tasks {
        let res = rx_res.recv().unwrap();
        results.push(res);
        println!("Completed [{}/{}] {}", results.len(), num_tasks, results.last().unwrap().metadata.name);
    }
    
    // Wait for worker threads
    for handle in workers {
        handle.join().unwrap();
    }
    
    // Sort results by Family and Name
    results.sort_by(|a, b| {
        let a_fam = &a.metadata.family;
        let b_fam = &b.metadata.family;
        if a_fam != b_fam {
            a_fam.cmp(b_fam)
        } else {
            // Sort numerically if possible
            a.metadata.name.cmp(&b.metadata.name)
        }
    });
    
    // Generate Report
    let mut table = String::new();
    table.push_str("| Family | Instance | Customers | Vehicles | Published BKS | Coralys Result | Gap (%) | Runtime | Status | Official Source |\n");
    table.push_str("| :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |\n");
    
    let mut total_instances = 0;
    let mut successful_runs = 0;
    let mut exact_matches = 0;
    let mut total_gap = 0.0;
    let mut gaps = Vec::new();
    let mut runtimes = Vec::new();
    let mut failed_instances = Vec::new();
    
    for r in &results {
        total_instances += 1;
        let bks = r.metadata.bks;
        let coralys = r.coralys_result;
        
        let is_valid = r.status != "FAILED" && r.status != "INFEASIBLE";
        let gap_str = if is_valid {
            let g = ((coralys - bks) / bks) * 100.0;
            format!("{:.2}%", g)
        } else {
            "N/A".to_string()
        };

        if is_valid {
            successful_runs += 1;
            let g = ((coralys - bks) / bks) * 100.0;
            total_gap += g;
            gaps.push(g);
            if r.status == "MATCH" || r.status == "BELOW BKS" {
                exact_matches += 1;
            }
        } else if r.status == "FAILED" {
            failed_instances.push(r.metadata.name.clone());
        }
        runtimes.push(r.runtime_sec);

        table.push_str(&format!(
            "| {} | {} | {} | {} | {:.2} | {:.2} | {} | {:.2} s | {} | [Source]({}) |\n",
            r.metadata.family,
            r.metadata.name,
            r.metadata.customers,
            r.metadata.vehicles,
            bks,
            coralys,
            gap_str,
            r.runtime_sec,
            r.status,
            r.metadata.instance_url
        ));
    }
    
    gaps.sort_by(|a, b| a.partial_cmp(b).unwrap());
    runtimes.sort_by(|a, b| a.partial_cmp(b).unwrap());
    
    let avg_gap = if successful_runs > 0 { total_gap / (successful_runs as f64) } else { 0.0 };
    let median_gap = if !gaps.is_empty() { gaps[gaps.len() / 2] } else { 0.0 };
    let worst_gap = if !gaps.is_empty() { gaps[gaps.len() - 1] } else { 0.0 };
    let best_gap = if !gaps.is_empty() { gaps[0] } else { 0.0 };
    let avg_runtime = if !runtimes.is_empty() { runtimes.iter().sum::<f64>() / (runtimes.len() as f64) } else { 0.0 };
    
    let summary = format!(
        r#"
## Final Summary Statistics:
- **Total Instances**: {total_instances}
- **Successful Runs**: {successful_runs}
- **Exact Matches**: {exact_matches}
- **Average Gap**: {avg_gap:.2}%
- **Median Gap**: {median_gap:.2}%
- **Worst Gap**: {worst_gap:.2}%
- **Best Gap**: {best_gap:.2}%
- **Average Runtime**: {avg_runtime:.2} s
- **Failed Instances**: {failed_instances:?}
"#,
        total_instances = total_instances,
        successful_runs = successful_runs,
        exact_matches = exact_matches,
        avg_gap = avg_gap,
        median_gap = median_gap,
        worst_gap = worst_gap,
        best_gap = best_gap,
        avg_runtime = avg_runtime,
        failed_instances = failed_instances.join(", ")
    );
    
    let report_content = format!("{}\n{}", table, summary);
    fs::write("/Users/nikhil/.gemini/antigravity/brain/262ffe5d-aed4-43c6-a002-28b6911113bc/cvrplib_baseline_report.md", &report_content).unwrap();
    
    println!("Baseline Evaluation complete. Report written to cvrplib_baseline_report.md");
}
