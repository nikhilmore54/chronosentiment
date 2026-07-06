use coralys_moga::ecology::{LifecycleState, Memory, MemoryPolicy, Observation, PolicyVault, VaultEntry};
use std::collections::{HashMap, HashSet};

pub type Tag = String;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarketChain {
    pub id: u64,
    pub sequence: Vec<Tag>,
    pub hop_support: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarketObservation {
    pub id: u64,
    pub date: String,
    pub timestamp: u64,
    pub tags: Vec<Tag>,
    pub description: String,
}

pub struct ChronoPolicy;
impl MemoryPolicy<MarketChain, Vec<Tag>, MarketObservation> for ChronoPolicy {
    fn should_store(&self, _entry: &VaultEntry<MarketChain, Vec<Tag>, MarketObservation>) -> bool { true }
    fn strengthen(&self, existing: &mut VaultEntry<MarketChain, Vec<Tag>, MarketObservation>, new_obs: &VaultEntry<MarketChain, Vec<Tag>, MarketObservation>) {
        existing.support += 1;
        existing.score += new_obs.score;
        existing.timestamp = new_obs.timestamp;
    }
    fn merge(&self, entries: &[VaultEntry<MarketChain, Vec<Tag>, MarketObservation>]) -> Option<VaultEntry<MarketChain, Vec<Tag>, MarketObservation>> {
        if let Some(last) = entries.last() {
            for existing in entries.iter().take(entries.len().saturating_sub(1)) {
                let tail = existing.structure.sequence.last().unwrap();
                let head = last.structure.sequence.first().unwrap();
                if tail == head {
                    let mut new_seq = existing.structure.sequence.clone();
                    new_seq.extend(last.structure.sequence[1..].iter().cloned());
                    let merged = VaultEntry {
                        structure: MarketChain {
                            id: existing.structure.id.wrapping_add(last.structure.id.wrapping_mul(1000)),
                            sequence: new_seq,
                            hop_support: vec![1; existing.structure.sequence.len() + last.structure.sequence.len() - 1],
                        },
                        context: existing.context.clone(),
                        evidence: vec![],
                        state: LifecycleState::Strengthened,
                        support: existing.support + last.support,
                        score: existing.score + last.score,
                        timestamp: last.timestamp.max(existing.timestamp),
                    };
                    return Some(merged);
                }
            }
        }
        None
    }
    fn should_evict(&self, entry: &VaultEntry<MarketChain, Vec<Tag>, MarketObservation>) -> bool { false }
}

struct ChronoDiscovery { next_id: u64 }
impl ChronoDiscovery {
    fn discover(&mut self, tags: Vec<Tag>, current_time: u64) -> VaultEntry<MarketChain, Vec<Tag>, MarketObservation> {
        let chain = VaultEntry {
            structure: MarketChain { id: self.next_id, sequence: tags.clone(), hop_support: vec![1] },
            context: vec![tags[0].clone()],
            evidence: vec![],
            state: LifecycleState::Candidate,
            support: 1, score: 1.0, timestamp: current_time,
        };
        self.next_id += 1;
        chain
    }
}

// ==========================================
// M25: BENCHMARK SUITE
// ==========================================

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Link(pub usize, pub usize);

pub struct NetworkGraph {
    pub num_nodes: usize,
    pub capacities: HashMap<Link, usize>,
    pub adj: Vec<Vec<usize>>,
}

pub struct Demand {
    pub id: usize,
    pub src: usize,
    pub dst: usize,
    pub vol: usize,
}

pub struct Scenario {
    pub network: NetworkGraph,
    pub demands: Vec<Demand>,
    pub budget: usize,
    pub intervention: Option<Link>,
}

impl NetworkGraph {
    /// Generates a highly connected graph with random capacities
    pub fn generate_random(nodes: usize, edge_prob: f64) -> Self {
        let mut adj = vec![Vec::new(); nodes];
        let mut capacities = HashMap::new();
        // Deterministic pseudo-random for reproducible benchmarks
        let mut seed: u32 = 42;
        
        for i in 0..nodes {
            for j in 0..nodes {
                if i == j { continue; }
                seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                if (seed % 100) as f64 / 100.0 < edge_prob {
                    adj[i].push(j);
                    capacities.insert(Link(i, j), 10); // Uniform low capacity
                }
            }
        }
        // Ensure there is at least one path by injecting a backbone
        for i in 0..nodes-1 {
            adj[i].push(i+1);
            capacities.insert(Link(i, i+1), 10);
        }
        Self { num_nodes: nodes, capacities, adj }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    NaiveDFS,
    BranchAndBound,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MemoryMode {
    Disabled,
    Authoritative,
    BinaryAdvisory,
    RankedAdvisory,
}

#[derive(Clone, Debug)]
pub struct ConstraintSignal {
    pub confidence: f64,
    pub pressure: f64,
    pub velocity: f64,
    pub acceleration: f64,
    pub success_count: usize,
    pub failure_count: usize,
    pub last_seen_epoch: u64,
}

pub struct SolverState {
    pub nodes_visited: usize,
    pub false_prunes: usize,
    pub solution_found: bool,
    pub first_solution_visited: usize,
    pub best_solution_visited: usize,
    pub best_objective: usize,
    pub mode: SearchMode,
    pub memory_mode: MemoryMode,
    pub vault: PolicyVault<MarketChain, Vec<Tag>, MarketObservation, ChronoPolicy>,
    pub discovery: ChronoDiscovery,
    pub time: u64,
    pub obs_id: u64,
    pub shuffle_seed: u32,
    pub action_log: Vec<String>,
}

impl SolverState {
    fn query_coralys(&self, active_event: &str) -> ConstraintSignal {
        if self.memory_mode == MemoryMode::Disabled { 
            return ConstraintSignal { confidence: 0.0, pressure: 0.0, velocity: 0.0, acceleration: 0.0, success_count: 0, failure_count: 0, last_seen_epoch: 0 }; 
        }
        
        let mut pressure_sum = 0.0;
        let mut failures = 0;
        let mut successes = 0;

        for entry in &self.vault.entries {
            if let Some(first) = entry.structure.sequence.first() {
                if first == active_event {
                    let last = entry.structure.sequence.last().unwrap();
                    if last == "BudgetExhausted" || last == "LinkSaturated" {
                        failures += 1;
                        pressure_sum += entry.score;
                    } else if last == "Feasible" {
                        successes += 1;
                    }
                }
            }
        }
        
        let total = failures + successes;
        let pressure = if total > 0 { failures as f64 / total as f64 } else { 0.0 };

        ConstraintSignal { 
            confidence: total as f64, 
            pressure, 
            velocity: 0.0, // Future trend tracking
            acceleration: 0.0,
            success_count: successes,
            failure_count: failures,
            last_seen_epoch: self.time,
        }
    }

    fn emit_causal_edge(&mut self, cause: String, effect: String) {
        let chain = self.discovery.discover(vec![cause, effect], self.time);
        self.vault.store(chain);
        self.vault.forget();
        self.time += 1;
        self.obs_id += 1;
    }

    pub fn solve(&mut self, scenario: &Scenario, demand_idx: usize, current_allocs: &HashMap<Link, usize>, current_cost: usize, precomputed_paths: &Vec<Vec<Vec<usize>>>) {
        // B&B Pruning: If cost exceeds best found, prune immediately
        if self.mode == SearchMode::BranchAndBound && current_cost >= self.best_objective {
            return;
        }

        if demand_idx >= scenario.demands.len() {
            if !self.solution_found {
                self.solution_found = true;
                self.first_solution_visited = self.nodes_visited;
            }
            if current_cost < self.best_objective {
                self.best_objective = current_cost;
                self.best_solution_visited = self.nodes_visited;
            }
            if self.memory_mode != MemoryMode::Disabled {
                let context_tag = "Nominal".to_string(); // Simplified for full route success
                self.emit_causal_edge(context_tag, "Feasible".to_string());
            }
            return; // Backtrack to find better solutions
        }

        let demand = &scenario.demands[demand_idx];
        
        let paths = precomputed_paths[demand_idx].clone();
        let mut sorted_paths = paths;
        if self.mode == SearchMode::BranchAndBound {
            sorted_paths.sort_by_key(|p| p.len()); // Best-first
        }
        
        if self.shuffle_seed > 0 {
            let mut seed = self.shuffle_seed;
            for i in (1..sorted_paths.len()).rev() {
                seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                let j = (seed as usize) % (i + 1);
                sorted_paths.swap(i, j);
            }
        }

        let intervention_tag = match &scenario.intervention {
            Some(link) => format!("Intervention({}->{})", link.0, link.1),
            None => "Nominal".to_string(),
        };

        let mut path_risks = Vec::new();
        for path in sorted_paths {
            let context_tag = format!("Path_{:?}", path);
            let risk = self.query_coralys(&context_tag);
            path_risks.push((path, context_tag, risk));
        }

        if self.memory_mode == MemoryMode::RankedAdvisory {
            path_risks.sort_by(|a, b| {
                a.2.pressure.partial_cmp(&b.2.pressure).unwrap_or(std::cmp::Ordering::Equal)
            });
        } else if self.memory_mode == MemoryMode::BinaryAdvisory {
            let mut safe = Vec::new();
            let mut risky = Vec::new();
            for pr in path_risks {
                if pr.2.pressure > 0.0 {
                    risky.push(pr);
                } else {
                    safe.push(pr);
                }
            }
            path_risks = [safe, risky].concat();
        }

        for (path, context_tag, signal) in path_risks {
            self.nodes_visited += 1;

            if self.memory_mode == MemoryMode::Authoritative && signal.pressure > 0.5 {
                self.action_log.push(format!("AUTHORITATIVE PRUNE | Pressure: {} | Context: {}", signal.pressure, context_tag));
                self.false_prunes += 1; 
                continue;
            }

            if (self.memory_mode == MemoryMode::RankedAdvisory || self.memory_mode == MemoryMode::BinaryAdvisory) && signal.pressure > 0.0 {
                self.action_log.push(format!("ADVISORY REORDER | Pressure: {} | Context: {}", signal.pressure, context_tag));
            }

            let mut true_failure = false;
            let mut next_allocs = current_allocs.clone();
            
            for i in 0..path.len()-1 {
                let link = Link(path[i], path[i+1]);
                if let Some(inv) = &scenario.intervention {
                    if link == *inv {
                        true_failure = true;
                        break;
                    }
                }
                let used = next_allocs.entry(link.clone()).or_insert(0);
                *used += demand.vol;
                if *used > *scenario.network.capacities.get(&link).unwrap_or(&0) {
                    true_failure = true;
                    break;
                }
            }

            if true_failure {
                self.emit_causal_edge(context_tag.clone(), "LinkSaturated".to_string());
                self.emit_causal_edge("LinkSaturated".to_string(), "BudgetExhausted".to_string());
                continue;
            }

            self.solve(scenario, demand_idx + 1, &next_allocs, current_cost + path.len(), precomputed_paths);
        }
    }

    fn find_all_paths(&self, net: &NetworkGraph, src: usize, dst: usize) -> Vec<Vec<usize>> {
        let mut paths = Vec::new();
        let mut queue = vec![vec![src]];
        
        while let Some(path) = queue.pop() {
            let last = *path.last().unwrap();
            if last == dst {
                paths.push(path);
            if paths.len() > 15 { break; } // limit arbitrarily to prevent total OOM
            continue;
        }
        if path.len() > 5 { continue; } // Max depth limit
            
            for &next in &net.adj[last] {
                if !path.contains(&next) {
                    let mut new_path = path.clone();
                    new_path.push(next);
                    queue.push(new_path);
                }
            }
        }
        paths
    }
}

// ==========================================
// M25 EXPERIMENT RUNNERS
// ==========================================



fn run_m25_7_trap_corridor() {
    println!("=== M25.7A Pressure-Guided Search (Combinatorial Trap Learning Curve) ===");
    let nodes = 20;
    let net = NetworkGraph::generate_random(nodes, 0.30);

    let mut demands = Vec::new();
    for id in 1..=4 {
        demands.push(Demand { id, src: 0, dst: nodes - 1, vol: 2 });
    }

    let scenario = Scenario { network: net.clone(), demands, budget: 1000, intervention: None };

    // Baseline run (Disabled Memory)
    let mut baseline = SolverState {
        nodes_visited: 0, false_prunes: 0, solution_found: false, best_objective: usize::MAX,
        first_solution_visited: 0, best_solution_visited: 0,
        mode: SearchMode::BranchAndBound, memory_mode: MemoryMode::Disabled,
        vault: PolicyVault { entries: vec![], policy: ChronoPolicy, max_capacity: 1000 }, 
        discovery: ChronoDiscovery { next_id: 1 },
        time: 1, obs_id: 1, shuffle_seed: 0, action_log: vec![],
    };
    // Precompute paths
    let mut precomputed_paths = Vec::new();
    for d in &scenario.demands {
        let paths = baseline.find_all_paths(&scenario.network, d.src, d.dst);
        precomputed_paths.push(paths);
    }

    baseline.solve(&scenario, 0, &HashMap::new(), 0, &precomputed_paths);
    println!("Baseline (No Memory) | Best@Node: {:<8} | Total Nodes: {}", baseline.best_solution_visited, baseline.nodes_visited);
    println!("{:-<75}", "");

    // Learning Curve
    let mut coralys_vault = PolicyVault { entries: vec![], policy: ChronoPolicy, max_capacity: 1000 };
    println!("{:<6} | {:<12} | {:<10} | {:<10} | {}", "Run", "Nodes Visited", "1st@Node", "Best@Node", "Hard FP");
    println!("{:-<75}", "");

    for run in 1..=10 {
        let mut state = SolverState {
            nodes_visited: 0, false_prunes: 0, solution_found: false, best_objective: usize::MAX,
            first_solution_visited: 0, best_solution_visited: 0,
            mode: SearchMode::BranchAndBound, memory_mode: MemoryMode::RankedAdvisory,
            vault: PolicyVault { entries: coralys_vault.entries.clone(), policy: ChronoPolicy, max_capacity: 1000 }, 
            discovery: ChronoDiscovery { next_id: 1 },
            time: run as u64, obs_id: 1, shuffle_seed: 0, action_log: vec![],
        };

        state.solve(&scenario, 0, &HashMap::new(), 0, &precomputed_paths);
        
        coralys_vault = state.vault; // Accumulate memory

        let hard_fp = !state.solution_found || state.best_objective > baseline.best_objective;
        
        println!("{:<6} | {:<12} | {:<10} | {:<10} | {}", 
            run, state.nodes_visited, state.first_solution_visited, state.best_solution_visited, hard_fp);
    }
}

impl Clone for NetworkGraph {
    fn clone(&self) -> Self {
        Self {
            num_nodes: self.num_nodes,
            capacities: self.capacities.clone(),
            adj: self.adj.clone(),
        }
    }
}

impl Clone for Demand {
    fn clone(&self) -> Self {
        Self { id: self.id, src: self.src, dst: self.dst, vol: self.vol }
    }
}

fn main() {
    run_m25_7_trap_corridor();
}
