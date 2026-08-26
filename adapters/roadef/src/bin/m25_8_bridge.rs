use std::collections::{HashMap, HashSet};

// --- Core Architecture Primitives ---
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Link(pub usize, pub usize);

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Demand {
    pub id: usize,
    pub src: usize,
    pub dst: usize,
    pub vol: usize,
}

#[derive(Clone, Debug)]
pub struct NetworkGraph {
    pub num_nodes: usize,
    pub capacities: HashMap<Link, usize>,
    pub adj: Vec<Vec<usize>>,
}

#[derive(Clone, Debug)]
pub struct Scenario {
    pub network: NetworkGraph,
    pub demands: Vec<Demand>,
    pub budget: usize,
}

#[derive(Clone, Debug)]
pub struct ConstraintSignal {
    pub confidence: f64,
    pub pressure: f64,
    pub trend: f64, // Positive = worsening, Negative = improving
    pub success_count: usize,
    pub failure_count: usize,
    pub last_seen_epoch: u64,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SearchPreference {
    StronglyDefer,
    Defer,
    Neutral,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MemoryMode {
    Disabled,
    RankedAdvisory,
}

// --- Solver State ---
pub struct SolverState {
    pub nodes_visited: usize,
    pub false_prunes: usize,
    pub solution_found: bool,
    pub best_objective: usize,
    pub first_solution_visited: usize,
    pub best_solution_visited: usize,
    pub memory_mode: MemoryMode,

    pub time: u64,
    pub successes: HashMap<String, usize>,
    pub failures: HashMap<String, usize>,
    pub prev_pressures: HashMap<String, f64>,

    // Tracking Metrics
    pub memory_queries: usize,
    pub memory_hits: usize,
    pub link_failure_counts: HashMap<Link, usize>,
}

impl SolverState {
    fn query_coralys(&mut self, context_tag: &str) -> ConstraintSignal {
        self.memory_queries += 1;
        if self.memory_mode == MemoryMode::Disabled {
            return ConstraintSignal {
                confidence: 0.0,
                pressure: 0.0,
                trend: 0.0,
                success_count: 0,
                failure_count: 0,
                last_seen_epoch: 0,
            };
        }

        let succ = *self.successes.get(context_tag).unwrap_or(&0);
        let fail = *self.failures.get(context_tag).unwrap_or(&0);
        let total = succ + fail;

        if total == 0 {
            return ConstraintSignal {
                confidence: 0.0,
                pressure: 0.5,
                trend: 0.0,
                success_count: 0,
                failure_count: 0,
                last_seen_epoch: 0,
            };
        }

        if self.prev_pressures.contains_key(context_tag) {
            self.memory_hits += 1; // It was based on existing memory from a prior epoch
        }

        let pressure = fail as f64 / total as f64;
        let prev = *self.prev_pressures.get(context_tag).unwrap_or(&0.5);
        let trend = pressure - prev; // Positive if pressure increased

        ConstraintSignal {
            confidence: total as f64,
            pressure,
            trend,
            success_count: succ,
            failure_count: fail,
            last_seen_epoch: self.time,
        }
    }

    fn derive_preference(&self, signal: &ConstraintSignal) -> SearchPreference {
        if signal.pressure > 0.8 {
            SearchPreference::StronglyDefer
        } else if signal.pressure > 0.4 {
            SearchPreference::Defer
        } else {
            SearchPreference::Neutral
        }
    }

    fn emit_causal_edge(&mut self, context_tag: String, is_success: bool) {
        if is_success {
            *self.successes.entry(context_tag).or_insert(0) += 1;
        } else {
            *self.failures.entry(context_tag).or_insert(0) += 1;
        }
    }

    pub fn snapshot_pressures(&mut self) {
        for tag in self.successes.keys().chain(self.failures.keys()) {
            let succ = *self.successes.get(tag).unwrap_or(&0);
            let fail = *self.failures.get(tag).unwrap_or(&0);
            let total = succ + fail;
            if total > 0 {
                self.prev_pressures
                    .insert(tag.clone(), fail as f64 / total as f64);
            }
        }
    }

    pub fn solve(
        &mut self,
        scenario: &Scenario,
        demand_idx: usize,
        current_allocs: &HashMap<Link, usize>,
        current_cost: usize,
        precomputed_paths: &Vec<Vec<Vec<usize>>>,
    ) -> bool {
        if current_cost > scenario.budget {
            return false;
        }
        if current_cost >= self.best_objective {
            return false;
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
            return true;
        }

        let demand = &scenario.demands[demand_idx];
        let mut paths = precomputed_paths[demand_idx].clone();

        // Pseudo-random baseline heuristic to simulate an imperfect solver
        let mut seed = demand_idx as u32 + 1;
        for i in (1..paths.len()).rev() {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            let j = (seed as usize) % (i + 1);
            paths.swap(i, j);
        }

        let mut path_risks = Vec::new();
        for path in &paths {
            let context_tag = format!("Path_{:?}", path);
            let risk = self.query_coralys(&context_tag);
            path_risks.push((path.clone(), context_tag, risk));
        }

        if self.memory_mode == MemoryMode::RankedAdvisory {
            path_risks.sort_by(|a, b| {
                a.2.pressure
                    .partial_cmp(&b.2.pressure)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }

        let mut any_success = false;

        for (path, context_tag, _) in path_risks {
            self.nodes_visited += 1;

            let mut true_failure = false;
            let mut failed_link = None;
            let mut next_allocs = current_allocs.clone();

            for i in 0..path.len() - 1 {
                let link = Link(path[i], path[i + 1]);
                let used = next_allocs.entry(link.clone()).or_insert(0);
                *used += demand.vol;
                if *used > *scenario.network.capacities.get(&link).unwrap_or(&0) {
                    true_failure = true;
                    failed_link = Some(link.clone());
                    break;
                }
            }

            if true_failure {
                if let Some(fl) = failed_link {
                    *self.link_failure_counts.entry(fl.clone()).or_insert(0) += 1;
                    let link_context = format!("Link_{:?}", fl);
                    if self.memory_mode != MemoryMode::Disabled {
                        self.emit_causal_edge(link_context, false);
                    }
                }
                if self.memory_mode != MemoryMode::Disabled {
                    self.emit_causal_edge(context_tag, false);
                }
                continue;
            }

            let success = self.solve(
                scenario,
                demand_idx + 1,
                &next_allocs,
                current_cost + path.len(),
                precomputed_paths,
            );
            if self.memory_mode != MemoryMode::Disabled {
                self.emit_causal_edge(context_tag, success);
                if success {
                    for i in 0..path.len() - 1 {
                        let link = Link(path[i], path[i + 1]);
                        self.emit_causal_edge(format!("Link_{:?}", link), true);
                    }
                }
            }
            if success {
                any_success = true;
            }
        }
        any_success
    }

    fn find_all_paths(&self, net: &NetworkGraph, src: usize, dst: usize) -> Vec<Vec<usize>> {
        let mut paths = Vec::new();
        let mut queue = vec![vec![src]];
        while let Some(path) = queue.pop() {
            let last = *path.last().unwrap();
            if last == dst {
                paths.push(path);
                if paths.len() >= 3 {
                    break;
                } // Bounded combinatorics
                continue;
            }
            if path.len() > 6 {
                continue;
            }
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
// M25.8 Realistic Validation Run
// ==========================================
fn main() {
    println!("=== M25.8 Realistic Solver Validation (Bridge to ROADEF) ===");
    let nodes = 30;

    // Create base network
    let mut base_net = NetworkGraph {
        num_nodes: nodes,
        capacities: HashMap::new(),
        adj: vec![Vec::new(); nodes],
    };
    let mut seed: u32 = 42;
    for i in 0..nodes {
        for j in 0..nodes {
            if i == j {
                continue;
            }
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            if (seed % 100) as f64 / 100.0 < 0.20 {
                base_net.adj[i].push(j);
                base_net.capacities.insert(Link(i, j), 8);
            }
        }
    }
    // High capacity backbone
    for i in 0..nodes - 1 {
        base_net.adj[i].push(i + 1);
        base_net.capacities.insert(Link(i, i + 1), 20);
    }

    // Generate 10 demands
    let mut demands = Vec::new();
    for id in 1..=10 {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let src = (seed as usize) % (nodes / 2);
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let dst = nodes / 2 + (seed as usize) % (nodes / 2);
        demands.push(Demand {
            id,
            src,
            dst,
            vol: 2,
        });
    }

    let mut coralys = SolverState {
        nodes_visited: 0,
        false_prunes: 0,
        solution_found: false,
        best_objective: usize::MAX,
        first_solution_visited: 0,
        best_solution_visited: 0,
        memory_mode: MemoryMode::RankedAdvisory,
        time: 1,
        successes: HashMap::new(),
        failures: HashMap::new(),
        prev_pressures: HashMap::new(),
        memory_queries: 0,
        memory_hits: 0,
        link_failure_counts: HashMap::new(),
    };

    println!(
        "{:<5} | {:<12} | {:<12} | {:<12} | {:<12} | {:<8} | {:<10} | {:<16}",
        "Epoch",
        "State",
        "Baseline Obj",
        "Coralys Obj",
        "Base Best@N",
        "Cor Best@N",
        "Hard FP",
        "Memory Reuse %"
    );
    println!("{:-<100}", "");

    for epoch in 1..=10 {
        coralys.time = epoch;
        coralys.snapshot_pressures();

        let mut epoch_net = base_net.clone();
        let mut epoch_demands = demands.clone();
        let mut state_name = "Normal";

        // Dynamic Epoch Adjustments
        if epoch >= 4 && epoch <= 6 {
            state_name = "Maintenance";
            epoch_net.capacities.insert(Link(10, 11), 2); // Severely drop capacity
            epoch_net.capacities.insert(Link(11, 12), 2);
        } else if epoch >= 7 && epoch <= 8 {
            state_name = "TrafficSpike";
            for d in epoch_demands.iter_mut() {
                d.vol = 4;
            }
        } else if epoch == 9 {
            state_name = "LinkFailure";
            epoch_net.capacities.insert(Link(14, 15), 0);
        } else if epoch == 10 {
            state_name = "Recovery";
        }

        let scenario = Scenario {
            network: epoch_net,
            demands: epoch_demands,
            budget: 150,
        };

        let mut precomputed_paths = Vec::new();
        for d in &scenario.demands {
            precomputed_paths.push(coralys.find_all_paths(&scenario.network, d.src, d.dst));
        }

        let mut baseline = SolverState {
            nodes_visited: 0,
            false_prunes: 0,
            solution_found: false,
            best_objective: usize::MAX,
            first_solution_visited: 0,
            best_solution_visited: 0,
            memory_mode: MemoryMode::Disabled,
            time: epoch,
            successes: HashMap::new(),
            failures: HashMap::new(),
            prev_pressures: HashMap::new(),
            memory_queries: 0,
            memory_hits: 0,
            link_failure_counts: HashMap::new(),
        };
        baseline.solve(&scenario, 0, &HashMap::new(), 0, &precomputed_paths);

        coralys.nodes_visited = 0;
        coralys.solution_found = false;
        coralys.best_objective = usize::MAX;
        coralys.first_solution_visited = 0;
        coralys.best_solution_visited = 0;
        coralys.memory_queries = 0;
        coralys.memory_hits = 0;
        coralys.link_failure_counts.clear();

        // Forecast Constraints
        let mut predicted_risks = Vec::new();
        let tags: Vec<String> = coralys.failures.keys().cloned().collect();
        for tag in tags {
            if tag.starts_with("Link_") {
                let sig = coralys.query_coralys(&tag);
                predicted_risks.push((tag.clone(), sig.pressure));
            }
        }
        predicted_risks.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let top_forecast: Vec<_> = predicted_risks
            .iter()
            .take(3)
            .map(|(t, _)| t.clone())
            .collect();

        // Pre-Recovery Snapshot
        let recovery_link = "Link_Link(14, 15)";
        let pre_recovery_sig = coralys.query_coralys(recovery_link);

        // Solve
        coralys.solve(&scenario, 0, &HashMap::new(), 0, &precomputed_paths);

        // Post-Recovery Snapshot
        let post_recovery_sig = coralys.query_coralys(recovery_link);

        // Metrics
        let reuse_rate = if coralys.memory_queries > 0 {
            (coralys.memory_hits as f64 / coralys.memory_queries as f64) * 100.0
        } else {
            0.0
        };

        let hard_fp = (baseline.solution_found && !coralys.solution_found)
            || (baseline.solution_found && coralys.best_objective > baseline.best_objective);

        let base_best = baseline.best_solution_visited;
        let cor_best = coralys.best_solution_visited;

        let mut actual_failures: Vec<_> = coralys.link_failure_counts.iter().collect();
        actual_failures.sort_by_key(|&(_, count)| std::cmp::Reverse(count));
        let top_actual: Vec<_> = actual_failures
            .iter()
            .take(3)
            .map(|(fl, _)| format!("Link_{:?}", fl))
            .collect();

        // Precision
        let mut hits = 0;
        for tf in &top_forecast {
            if top_actual.contains(tf) {
                hits += 1;
            }
        }
        let precision = if top_forecast.len() > 0 {
            (hits as f64 / top_forecast.len() as f64) * 100.0
        } else {
            0.0
        };

        println!(
            "{:<5} | {:<12} | {:<12} | {:<12} | {:<12} | {:<8} | {:<10} | {:>5.1}%",
            epoch,
            state_name,
            if baseline.solution_found {
                baseline.best_objective.to_string()
            } else {
                "Unsat".to_string()
            },
            if coralys.solution_found {
                coralys.best_objective.to_string()
            } else {
                "Unsat".to_string()
            },
            base_best,
            cor_best,
            hard_fp,
            reuse_rate
        );

        println!("  ├─ Top Forecast: {:?}", top_forecast);
        println!("  ├─ Top Actual  : {:?}", top_actual);
        println!("  └─ Precision   : {:.1}%", precision);

        if epoch == 10 {
            println!("\n=== Recovery Epoch Kinematics ({}) ===", recovery_link);
            println!(
                "Epoch 9  | Pressure: {:.2} | Confidence: {} | Trend: {:+.2} | Pref: {:?}",
                pre_recovery_sig.pressure,
                pre_recovery_sig.confidence,
                pre_recovery_sig.trend,
                coralys.derive_preference(&pre_recovery_sig)
            );
            println!(
                "Epoch 10 | Pressure: {:.2} | Confidence: {} | Trend: {:+.2} | Pref: {:?}",
                post_recovery_sig.pressure,
                post_recovery_sig.confidence,
                post_recovery_sig.trend,
                coralys.derive_preference(&post_recovery_sig)
            );
        }
    }
}
