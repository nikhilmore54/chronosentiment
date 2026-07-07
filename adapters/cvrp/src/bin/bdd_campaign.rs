use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Instant, Duration};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use cvrp::{CvrpInstance, CvrpGenomeFactory, DistanceMetric, RadiusPolicy, Node};
use cvrp::moga_impl::{CvrpEvaluator, CvrpMutator, CvrpCrossover, CvrpLocalSearch, CvrpViolation};
use coralys_moga::{EvolutionConfig, EvolutionEngineBuilder, RepairStats};

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

#[derive(Serialize, Deserialize, Clone, Debug)]
struct InstanceCampaignResult {
    name: String,
    family: String,
    customers: usize,
    vehicles: usize,
    capacity: usize,
    bks: f64,
    coralys_cost: f64,
    absolute_gap: f64,
    percentage_gap: f64,
    runtime_sec: f64,
    generations: usize,
    status: String, // "COMPLETED", "TIMEOUT", "FAILED", "PENDING"
    feasible: bool,
    best_permutation: Vec<usize>,
    repair_stats: Option<RepairStatsSnapshot>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct RepairStatsSnapshot {
    total_invocations: usize,
    successful_repairs: usize,
    failed_repairs: usize,
    total_iterations: usize,
    violations_encountered: HashMap<String, usize>,
    heuristic_successes: HashMap<String, usize>,
    heuristic_attempts: HashMap<String, usize>,
}

fn parse_vrp_file(content: &str) -> Option<CvrpInstance> {
    let mut capacity = 0;
    let mut coords = Vec::new();
    let mut demands = Vec::new();
    
    let mut section = "";
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }
        if line.starts_with("CAPACITY") {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 2 {
                capacity = parts[1].trim().parse().ok()?;
            }
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
                let id: usize = parts[0].parse().ok()?;
                let x: f64 = parts[1].parse().ok()?;
                let y: f64 = parts[2].parse().ok()?;
                coords.push((id, x, y));
            }
        } else if section == "demands" {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let id: usize = parts[0].parse().ok()?;
                let demand: i32 = parts[1].parse().ok()?;
                demands.push((id, demand));
            }
        }
    }
    
    let depot_coord = coords.iter().find(|(id, _, _)| *id == 1)?;
    let depot_demand = demands.iter().find(|(id, _)| *id == 1)?;
    let depot = Node { id: depot_coord.0, x: depot_coord.1, y: depot_coord.2, demand: depot_demand.1 };
    
    let mut customers = Vec::new();
    for coord in coords {
        if coord.0 == 1 { continue; }
        let demand = demands.iter().find(|(id, _)| *id == coord.0)?;
        customers.push(Node { id: coord.0, x: coord.1, y: coord.2, demand: demand.1 });
    }
    
    Some(CvrpInstance {
        capacity,
        depot,
        customers,
        distance_metric: DistanceMetric::TspLibEuc2D,
        max_vehicles: None,
    })
}

fn run_single_instance(meta: &InstanceMetadata, vrp_content: &str) -> Result<InstanceCampaignResult, String> {
    let start = Instant::now();
    let mut instance = parse_vrp_file(vrp_content).ok_or_else(|| "Failed to parse VRP file".to_string())?;
    instance.max_vehicles = Some(meta.vehicles);
    
    let evaluator = CvrpEvaluator { instance: instance.clone() };
    let mutator = CvrpMutator::new(instance.clone(), RadiusPolicy::Control);
    let crossover = CvrpCrossover;
    let factory = CvrpGenomeFactory { num_customers: instance.customers.len() };
    
    // Feasibility Repair Framework
    use coralys_moga::FeasibilityRepairFramework;
    let checker = cvrp::moga_impl::CvrpConstraintChecker { instance: instance.clone() };
    let limit_heuristic = cvrp::moga_impl::VehicleLimitRepairHeuristic { instance: instance.clone() };
    let bp_heuristic = cvrp::moga_impl::BinPackingRepairHeuristic { instance: instance.clone() };
    let spatial_bp_heuristic = cvrp::moga_impl::SpatialBinPackingRepairHeuristic { instance: instance.clone() };
    
    let mut repair_framework = FeasibilityRepairFramework::new(10);
    repair_framework.add_checker(Box::new(checker));
    repair_framework.add_heuristic(Box::new(limit_heuristic));
    repair_framework.add_heuristic(Box::new(bp_heuristic));
    repair_framework.add_heuristic(Box::new(spatial_bp_heuristic));
    
    let repair_stats = repair_framework.stats.clone();
    
    let config = EvolutionConfig {
        population_size: 100, // Balanced for fast campaign evaluation
        elite_count: 10,
        generation_limit: 30,
        mutation_rate: 0.2,
        crossover_rate: 0.8,
        seed: Some(42),
        tournament_size: Some(5),
        ..Default::default()
    };
    
    use coralys_core::Outcome;
    let engine = EvolutionEngineBuilder::new()
        .with_evaluator(evaluator)
        .with_mutator(mutator)
        .with_crossover(crossover)
        .with_factory(factory)
        .with_improvement(repair_framework)
        .build()
        .map_err(|e| format!("Engine build error: {}", e))?;
        
    let res = engine.run_ga_evolution(config).map_err(|e| format!("Evolution error: {}", e))?;
    
    let coralys_cost = res.global_best.eval.total_distance_integer;
    let runtime_sec = start.elapsed().as_secs_f64();
    
    // Validate feasibility
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
    let vehicle_count_ok = res.global_best.eval.routes.len() <= meta.vehicles;
    let feasible = customer_coverage_ok && capacity_ok && vehicle_count_ok && is_feasible;
    
    let absolute_gap = coralys_cost - meta.bks;
    let percentage_gap = (absolute_gap / meta.bks) * 100.0;
    
    let stats = {
        let stats_lock = repair_stats.lock().unwrap();
        RepairStatsSnapshot {
            total_invocations: stats_lock.total_invocations,
            successful_repairs: stats_lock.successful_repairs,
            failed_repairs: stats_lock.failed_repairs,
            total_iterations: stats_lock.total_iterations,
            violations_encountered: stats_lock.violations_encountered.clone(),
            heuristic_successes: stats_lock.heuristic_successes.clone(),
            heuristic_attempts: stats_lock.heuristic_attempts.clone(),
        }
    };

    Ok(InstanceCampaignResult {
        name: meta.name.clone(),
        family: meta.family.clone(),
        customers: meta.customers,
        vehicles: meta.vehicles,
        capacity: meta.capacity,
        bks: meta.bks,
        coralys_cost,
        absolute_gap,
        percentage_gap,
        runtime_sec,
        generations: 30,
        status: "COMPLETED".to_string(),
        feasible,
        best_permutation: res.global_best.solution().permutation.clone(),
        repair_stats: Some(stats),
    })
}

fn main() {
    println!("========================================================");
    println!("CORALYS CVRPLIB BASELINE CAMPAIGN RUNNER");
    println!("========================================================");
    
    let metadata_path = "/Users/nikhil/.gemini/antigravity/brain/262ffe5d-aed4-43c6-a002-28b6911113bc/scratch/instances.json";
    let metadata_json = fs::read_to_string(metadata_path).expect("Failed to read instances.json");
    let all_instances: Vec<InstanceMetadata> = serde_json::from_str(&metadata_json).expect("Failed to parse instances.json");
    
    let instances_dir = "/Users/nikhil/ChronoSentiment_MEGA_FINAL/adapters/cvrp/data/instances";
    let results_path = "/Users/nikhil/.gemini/antigravity/brain/262ffe5d-aed4-43c6-a002-28b6911113bc/scratch/campaign_results.json";
    
    // Load existing progress if any
    let mut completed_results: HashMap<String, InstanceCampaignResult> = if Path::new(results_path).exists() {
        let content = fs::read_to_string(results_path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        HashMap::new()
    };
    
    println!("Loaded {} completed results from previous runs.", completed_results.len());
    
    let results_mutex = Arc::new(Mutex::new(completed_results));
    
    // Set up parallel task channels
    let (tx_task, rx_task) = std::sync::mpsc::channel::<Option<InstanceMetadata>>();
    let rx_task = Arc::new(Mutex::new(rx_task));
    
    let num_workers = 8;
    let mut workers = Vec::new();
    
    for _ in 0..num_workers {
        let rx_task = rx_task.clone();
        let results_mutex = results_mutex.clone();
        let instances_dir = instances_dir.to_string();
        let results_path = results_path.to_string();
        
        let handle = thread::spawn(move || {
            loop {
                let task_opt = {
                    let lock = rx_task.lock().unwrap();
                    lock.recv().unwrap()
                };
                
                let meta = match task_opt {
                    Some(m) => m,
                    None => break, // Poison pill
                };
                
                // Check if already completed
                {
                    let lock = results_mutex.lock().unwrap();
                    if lock.contains_key(&meta.name) {
                        continue;
                    }
                }
                
                let vrp_path = format!("{}/{}.vrp", instances_dir, meta.name);
                if !Path::new(&vrp_path).exists() {
                    // Record file missing error
                    let mut lock = results_mutex.lock().unwrap();
                    lock.insert(meta.name.clone(), InstanceCampaignResult {
                        name: meta.name.clone(),
                        family: meta.family.clone(),
                        customers: meta.customers,
                        vehicles: meta.vehicles,
                        capacity: meta.capacity,
                        bks: meta.bks,
                        coralys_cost: 0.0,
                        absolute_gap: 0.0,
                        percentage_gap: 0.0,
                        runtime_sec: 0.0,
                        generations: 0,
                        status: "FAILED (File Missing)".to_string(),
                        feasible: false,
                        best_permutation: Vec::new(),
                        repair_stats: None,
                    });
                    continue;
                }
                
                let vrp_content = fs::read_to_string(&vrp_path).unwrap();
                
                // Enforce limit per instance to remain robust against XXL instances
                let meta_clone = meta.clone();
                let results_mutex_clone = results_mutex.clone();
                let results_path_clone = results_path.clone();
                
                println!("Evaluating {}...", meta_clone.name);
                let start_inst = Instant::now();
                
                // To keep it robust against huge instances (>1000 nodes), we skip instances with >1000 nodes
                if meta_clone.customers > 1000 {
                    let mut lock = results_mutex_clone.lock().unwrap();
                    lock.insert(meta_clone.name.clone(), InstanceCampaignResult {
                        name: meta_clone.name.clone(),
                        family: meta_clone.family.clone(),
                        customers: meta_clone.customers,
                        vehicles: meta_clone.vehicles,
                        capacity: meta_clone.capacity,
                        bks: meta_clone.bks,
                        coralys_cost: 0.0,
                        absolute_gap: 0.0,
                        percentage_gap: 0.0,
                        runtime_sec: 0.0,
                        generations: 0,
                        status: "SKIPPED (Scale Exceeds Limit)".to_string(),
                        feasible: false,
                        best_permutation: Vec::new(),
                        repair_stats: None,
                    });
                    continue;
                }
                
                match run_single_instance(&meta_clone, &vrp_content) {
                    Ok(res) => {
                        println!("Completed {} in {:.2}s. Cost: {}, BKS: {}", res.name, res.runtime_sec, res.coralys_cost, res.bks);
                        let mut lock = results_mutex_clone.lock().unwrap();
                        lock.insert(res.name.clone(), res);
                    }
                    Err(e) => {
                        eprintln!("Error on {}: {}", meta_clone.name, e);
                        let mut lock = results_mutex_clone.lock().unwrap();
                        lock.insert(meta_clone.name.clone(), InstanceCampaignResult {
                            name: meta_clone.name.clone(),
                            family: meta_clone.family.clone(),
                            customers: meta_clone.customers,
                            vehicles: meta_clone.vehicles,
                            capacity: meta_clone.capacity,
                            bks: meta_clone.bks,
                            coralys_cost: 0.0,
                            absolute_gap: 0.0,
                            percentage_gap: 0.0,
                            runtime_sec: start_inst.elapsed().as_secs_f64(),
                            generations: 0,
                            status: format!("FAILED ({})", e),
                            feasible: false,
                            best_permutation: Vec::new(),
                            repair_stats: None,
                        });
                    }
                }
                
                // Write incremental results database to disk
                {
                    let lock = results_mutex_clone.lock().unwrap();
                    let serialized = serde_json::to_string_pretty(&*lock).unwrap();
                    let _ = fs::write(&results_path_clone, serialized);
                }
            }
        });
        
        workers.push(handle);
    }
    
    // Dispatch tasks
    for inst in all_instances {
        tx_task.send(Some(inst)).unwrap();
    }
    
    // Poison pills
    for _ in 0..num_workers {
        tx_task.send(None).unwrap();
    }
    
    for w in workers {
        w.join().unwrap();
    }
    
    println!("Campaign execution complete.");
}
