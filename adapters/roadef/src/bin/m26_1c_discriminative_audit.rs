use std::collections::{HashMap, HashSet};
use roadef::loader::{load_network, load_scenario, load_traffic_matrix};
use roadef::models::{Network, Scenario, TrafficMatrix, Solution, SrPath};
use roadef::graph::Digraph;
use roadef::evaluator::RoadefEvaluator;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct Observation {
    context_tag: String,
    is_success: bool,
}

struct AuditTracker {
    observations_emitted: usize,
    contexts: HashMap<String, (usize, usize)>, // (successes, failures)
}

impl AuditTracker {
    fn new() -> Self {
        Self {
            observations_emitted: 0,
            contexts: HashMap::new(),
        }
    }

    fn emit(&mut self, tag: String, success: bool) {
        self.observations_emitted += 1;
        let entry = self.contexts.entry(tag).or_insert((0, 0));
        if success {
            entry.0 += 1;
        } else {
            entry.1 += 1;
        }
    }
    
    fn print_metrics(&self) {
        let unique_contexts = self.contexts.len();
        let mut reusable_contexts = 0;
        
        let mut freqs = Vec::new();
        
        for (tag, (s, f)) in &self.contexts {
            let total = s + f;
            if total > 1 {
                reusable_contexts += 1;
            }
            freqs.push((tag.clone(), total, *s, *f));
        }
        
        let reuse_ratio = if unique_contexts > 0 {
            (reusable_contexts as f64 / unique_contexts as f64) * 100.0
        } else { 0.0 };
        
        let avg_freq = if unique_contexts > 0 {
            self.observations_emitted as f64 / unique_contexts as f64
        } else { 0.0 };
        
        println!("=== M26.1 Observation Mapping Audit ===");
        println!("Observations Emitted : {}", self.observations_emitted);
        println!("Unique Contexts      : {}", unique_contexts);
        println!("Reusable Contexts    : {}", reusable_contexts);
        println!("Reuse Ratio          : {:.1}%", reuse_ratio);
        println!("Avg Frequency        : {:.1}", avg_freq);
        
        freqs.sort_by_key(|k| std::cmp::Reverse(k.1));
        
        println!("\nTop 10 Reused Contexts:");
        for i in 0..std::cmp::min(10, freqs.len()) {
            let (t, tot, s, f) = &freqs[i];
            println!("  {:5} | S:{:<4} F:{:<4} | {}", tot, s, f, t);
        }
        
        let mut fail_freqs: Vec<_> = freqs.iter().filter(|f| f.3 > 0).collect();
        fail_freqs.sort_by_key(|k| std::cmp::Reverse(k.3));
        
        println!("\nTop 10 Failure Contexts:");
        for i in 0..std::cmp::min(10, fail_freqs.len()) {
            let (t, tot, s, f) = &fail_freqs[i];
            println!("  {:5} | S:{:<4} F:{:<4} | {}", tot, s, f, t);
        }
    }
}

fn build_context_tag(
    graph: &Digraph, 
    path: &[u64], 
    demand_vol: f64, 
    max_cap: f64,
    time_slot: usize,
    has_interventions: bool,
    budget_usage: f64
) -> String {
    // Layer 1/2 Ecological Mapping
    
    // 1. Demand Volume Category
    let vol_cat = if demand_vol < max_cap * 0.05 { "Vol:Low" }
                  else if demand_vol < max_cap * 0.20 { "Vol:Med" }
                  else { "Vol:High" };
                  
    // 2. Path Length Category
    let len_cat = if path.len() <= 2 { "Len:Short" }
                  else if path.len() <= 4 { "Len:Med" }
                  else { "Len:Long" };
                  
    // 3. Core Traversal (Assume nodes 0..10 are core backbone nodes heuristically)
    let crosses_core = if path.iter().any(|&n| n < 10) { "Core:Yes" } else { "Core:No" };
    
    // 4. Temporal / Maintenance Context
    let maint_cat = if has_interventions { "Maint:Active" } else { "Maint:Clear" };
    
    // 5. Budget Context
    let budget_cat = if budget_usage > 0.9 { "Budget:Tight" }
                     else if budget_usage > 0.5 { "Budget:Med" }
                     else { "Budget:Ok" };

    format!("{}|{}|{}|{}|{}", vol_cat, len_cat, crosses_core, maint_cat, budget_cat)
}

fn dfs_solve(
    demand_idx: usize,
    current_solution: &mut Solution,
    evaluator: &RoadefEvaluator,
    graph: &Digraph,
    tm: &TrafficMatrix,
    max_cap: f64,
    has_interventions: bool,
    tracker: &mut AuditTracker,
    rng_seed: &mut u32,
) -> bool {
    if demand_idx == tm.demands.len().min(8) { // depth=8
        return true;
    }
    
    let demand = &tm.demands[demand_idx];
    let t = 0;
    
    let mut candidates = vec![vec![]];
    for _ in 0..3 {
        *rng_seed = rng_seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let wp = (*rng_seed % graph.nodes.len() as u32) as u64;
        candidates.push(vec![wp]);
    }
    
    let mut any_success = false;
    
    for w in candidates {
        current_solution.srpaths.push(SrPath {
            d: demand_idx,
            t,
            w: w.clone(),
        });
        
        let res = evaluator.evaluate_solution(current_solution);
        let is_success = res.valid && res.obj.is_finite();
        
        let mut path_nodes = vec![demand.s];
        path_nodes.extend_from_slice(&w);
        path_nodes.push(demand.t);
        
        let tag = build_context_tag(graph, &path_nodes, demand.v[t], max_cap, t, has_interventions, 0.5);
        tracker.emit(tag, is_success);
        
        if is_success {
            let deeper = dfs_solve(demand_idx + 1, current_solution, evaluator, graph, tm, max_cap, has_interventions, tracker, rng_seed);
            if deeper { any_success = true; }
        }
        
        current_solution.srpaths.pop();
    }
    
    any_success
}

fn main() -> anyhow::Result<()> {
    let mut tracker = AuditTracker::new();
    
    let stress_levels = [
        ("NoStress", 0.5, 2.0),
        ("Light", 0.8, 1.5),
        ("Normal", 1.0, 1.0),
        ("Mild", 1.1, 0.95),
        ("Moderate", 1.2, 0.9),
    ];
    
    for (_level_name, d_mult, c_mult) in &stress_levels {
        let mut net = load_network("repo/challenge-roadef-2026-main/setA/setA-01-net.json")?;
        let mut tm = load_traffic_matrix("repo/challenge-roadef-2026-main/setA/setA-01-tm.json")?;
        let scenario = load_scenario("repo/challenge-roadef-2026-main/setA/setA-01-scenario.json")?;
        
        for d in &mut tm.demands {
            for v in &mut d.v { *v *= *d_mult; }
        }
        for l in &mut net.links { l.capacity *= *c_mult; }
        
        let graph = Digraph::new(&net);
        let evaluator = RoadefEvaluator::new(&net, tm.clone(), scenario.clone());
        let max_cap = net.links.iter().map(|l| l.capacity).fold(0.0_f64, f64::max);
        
        let mut current_solution = Solution { srpaths: Vec::new() };
        let mut rng_seed = 42u32;
        let has_interventions = scenario.interventions.iter().any(|i| i.t == 0);
        
        // Depth 8 to allow reasonable trees, branch factor 3
        fn dfs_solve(
            demand_idx: usize,
            current_solution: &mut Solution,
            evaluator: &RoadefEvaluator,
            graph: &Digraph,
            tm: &TrafficMatrix,
            max_cap: f64,
            has_interventions: bool,
            tracker: &mut AuditTracker,
            rng_seed: &mut u32,
        ) -> bool {
            if demand_idx == tm.demands.len().min(8) { return true; }
            let demand = &tm.demands[demand_idx];
            let t = 0;
            
            let mut candidates = vec![vec![]]; // Direct path
            for _ in 0..2 { // 2 random 1-hop paths
                *rng_seed = rng_seed.wrapping_mul(1664525).wrapping_add(1013904223);
                let wp = (*rng_seed % graph.nodes.len() as u32) as u64;
                candidates.push(vec![wp]);
            }
            
            let mut any_success = false;
            for w in candidates {
                current_solution.srpaths.push(SrPath { d: demand_idx, t, w: w.clone() });
                let res = evaluator.evaluate_solution(current_solution);
                let is_success = res.valid && res.obj.is_finite();
                
                let mut path_nodes = vec![demand.s];
                path_nodes.extend_from_slice(&w);
                path_nodes.push(demand.t);
                let tag = build_context_tag(graph, &path_nodes, demand.v[t], max_cap, t, has_interventions, 0.5);
                tracker.emit(tag, is_success);
                
                if is_success {
                    let deeper = dfs_solve(demand_idx + 1, current_solution, evaluator, graph, tm, max_cap, has_interventions, tracker, rng_seed);
                    if deeper { any_success = true; }
                }
                current_solution.srpaths.pop();
            }
            any_success
        }
        
        dfs_solve(0, &mut current_solution, &evaluator, &graph, &tm, max_cap, has_interventions, &mut tracker, &mut rng_seed);
    }
    
    println!("=== M26.1C Discriminative Ecology Audit (Solver Trajectories) ===");
    
    let total_obs = tracker.observations_emitted;
    let mut entropy = 0.0;
    
    let mut contexts: Vec<String> = tracker.contexts.keys().cloned().collect();
        
    contexts.sort_by(|a, b| {
        let (s_a, f_a) = tracker.contexts.get(a).unwrap_or(&(0, 0));
        let (s_b, f_b) = tracker.contexts.get(b).unwrap_or(&(0, 0));
        
        let r_a = *f_a as f64 / (f_a + s_a) as f64;
        let r_b = *f_b as f64 / (f_b + s_b) as f64;
        
        r_b.partial_cmp(&r_a).unwrap_or(std::cmp::Ordering::Equal)
    });

    println!("{:<45} | {:<7} | {:<7} | {:<12}", "Context", "Success", "Failure", "Failure Rate");
    println!("{:-<45}-|-{:-<7}-|-{:-<7}-|-{:-<12}", "", "", "", "");
    
    for tag in &contexts {
        let (s, f) = tracker.contexts.get(tag).unwrap();
        let total = s + f;
        if total == 0 { continue; }
        
        let rate = *f as f64 / total as f64;
        let p = total as f64 / total_obs as f64;
        entropy -= p * p.log2();
        
        println!("{:<45} | {:<7} | {:<7} | {:.1}%", tag, s, f, rate * 100.0);
    }
    
    let reuse = contexts.len() as f64 / total_obs as f64;
    println!("\nObservations Emitted : {}", total_obs);
    println!("Unique Contexts      : {}", contexts.len());
    println!("Reuse Ratio          : {:.1}%", (1.0 - reuse) * 100.0);
    println!("Context Entropy      : {:.3} bits", entropy);
    
    Ok(())
}
