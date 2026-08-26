use coralys_moga::ecology::{
    LifecycleState, Memory, MemoryPolicy, Observation, PolicyVault, VaultEntry,
};
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
    fn should_store(&self, _entry: &VaultEntry<MarketChain, Vec<Tag>, MarketObservation>) -> bool {
        true
    }

    fn strengthen(
        &self,
        existing: &mut VaultEntry<MarketChain, Vec<Tag>, MarketObservation>,
        new_obs: &VaultEntry<MarketChain, Vec<Tag>, MarketObservation>,
    ) {
        existing.support += 1;
        existing.score += new_obs.score;
        existing.timestamp = new_obs.timestamp;
    }

    fn merge(
        &self,
        entries: &[VaultEntry<MarketChain, Vec<Tag>, MarketObservation>],
    ) -> Option<VaultEntry<MarketChain, Vec<Tag>, MarketObservation>> {
        if let Some(last) = entries.last() {
            for existing in entries.iter().take(entries.len().saturating_sub(1)) {
                let tail = existing.structure.sequence.last().unwrap();
                let head = last.structure.sequence.first().unwrap();
                if tail == head {
                    let mut new_seq = existing.structure.sequence.clone();
                    new_seq.extend(last.structure.sequence[1..].iter().cloned());
                    let mut ev = existing.evidence.clone();
                    for e in &last.evidence {
                        if !ev.iter().any(|x| x.id == e.id) {
                            ev.push(e.clone());
                        }
                    }
                    let merged = VaultEntry {
                        structure: MarketChain {
                            id: existing
                                .structure
                                .id
                                .wrapping_add(last.structure.id.wrapping_mul(1000)),
                            sequence: new_seq,
                            hop_support: vec![
                                1;
                                existing.structure.sequence.len()
                                    + last.structure.sequence.len()
                                    - 1
                            ],
                        },
                        context: existing.context.clone(),
                        evidence: ev,
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

    fn should_evict(&self, entry: &VaultEntry<MarketChain, Vec<Tag>, MarketObservation>) -> bool {
        entry.state == LifecycleState::Expired
    }
}

struct ChronoDiscovery {
    next_id: u64,
}

impl ChronoDiscovery {
    fn discover(
        &mut self,
        tags: Vec<Tag>,
        current_time: u64,
        obs_id: u64,
    ) -> VaultEntry<MarketChain, Vec<Tag>, MarketObservation> {
        let obs = MarketObservation {
            id: obs_id,
            date: current_time.to_string(),
            timestamp: current_time,
            tags: tags.clone(),
            description: "Explicit Causal Edge".to_string(),
        };
        let chain = VaultEntry {
            structure: MarketChain {
                id: self.next_id,
                sequence: tags.clone(),
                hop_support: vec![1],
            },
            context: vec![tags[0].clone()],
            evidence: vec![obs],
            state: LifecycleState::Candidate,
            support: 1,
            score: 1.0,
            timestamp: current_time,
        };
        self.next_id += 1;
        chain
    }
}

// ==========================================
// TINY ROADEF SOLVER
// ==========================================

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Link(String, String);

struct ToyNetwork {
    capacities: HashMap<Link, usize>,
}

#[derive(Clone, Debug)]
struct ToyDemand {
    id: usize,
    src: String,
    dst: String,
    vol: usize,
}

struct SolverState {
    nodes_visited: usize,
    false_prunes: usize,
    network: ToyNetwork,
    demands: Vec<ToyDemand>,
    use_coralys: bool,
    vault: PolicyVault<MarketChain, Vec<Tag>, MarketObservation, ChronoPolicy>,
    discovery: ChronoDiscovery,
    time: u64,
    obs_id: u64,
    found_solution: bool,
}

impl SolverState {
    fn query_coralys(&self, active_event: &str) -> bool {
        if !self.use_coralys {
            return false;
        }
        // Look for chains starting with active_event that lead to BudgetExhausted
        for entry in &self.vault.entries {
            if entry.structure.sequence.first().unwrap() == active_event {
                if entry.structure.sequence.last().unwrap() == "BudgetExhausted" {
                    return true; // Predicts failure!
                }
            }
        }
        false
    }

    fn emit_causal_edge(&mut self, cause: String, effect: String) {
        let chain = self
            .discovery
            .discover(vec![cause, effect], self.time, self.obs_id);
        self.vault.store(chain);
        self.vault.forget(); // Apply merges
        self.time += 1;
        self.obs_id += 1;
    }

    fn get_all_paths(&self, src: &str, dst: &str) -> Vec<Vec<String>> {
        // Hardcoded toy paths for simplicity
        if src == "A" && dst == "D" {
            vec![
                vec!["A".to_string(), "B".to_string(), "D".to_string()],
                vec!["A".to_string(), "C".to_string(), "D".to_string()],
                vec![
                    "A".to_string(),
                    "B".to_string(),
                    "C".to_string(),
                    "D".to_string(),
                ],
            ]
        } else {
            vec![]
        }
    }

    fn solve(
        &mut self,
        demand_idx: usize,
        current_allocations: &HashMap<Link, usize>,
        intervention: &str,
        budget: usize,
    ) -> bool {
        self.nodes_visited += 1;

        if budget == 0 {
            return false;
        }

        if demand_idx >= self.demands.len() {
            self.found_solution = true;
            return true;
        }

        let demand = self.demands[demand_idx].clone();
        let paths = self.get_all_paths(&demand.src, &demand.dst);

        for path in paths {
            let decision_tag = format!("Route(D{}, {:?})", demand.id, path);
            let context_tag = if demand_idx == 0 {
                format!("Intervention({})", intervention)
            } else {
                decision_tag.clone()
            };

            // 1. CORALYS PRUNING CHECK
            let will_fail = self.query_coralys(&context_tag);

            // Check true failure to track false prunes
            let mut true_failure = false;
            let mut next_allocs = current_allocations.clone();
            for i in 0..path.len() - 1 {
                let link = Link(path[i].clone(), path[i + 1].clone());
                if link == Link("A".to_string(), "C".to_string()) && intervention == "A->C" {
                    true_failure = true;
                    break;
                }
                let used = next_allocs.entry(link.clone()).or_insert(0);
                *used += demand.vol;
                if *used > *self.network.capacities.get(&link).unwrap_or(&0) {
                    true_failure = true;
                    break;
                }
            }

            if will_fail {
                if !true_failure {
                    self.false_prunes += 1; // It was pruned but could have succeeded!
                }
                continue; // Pruned by memory!
            }

            // 2. SIMULATE
            if true_failure {
                // Record failure explicitly
                self.emit_causal_edge(context_tag.clone(), "LinkSaturated".to_string());
                self.emit_causal_edge("LinkSaturated".to_string(), "BudgetExhausted".to_string());
                continue; // Backtrack
            }

            // 3. RECURSE
            if self.solve(demand_idx + 1, &next_allocs, intervention, budget - 1) {
                return true;
            }
        }

        false
    }
}

fn main() {
    let mut caps = HashMap::new();
    caps.insert(Link("A".to_string(), "B".to_string()), 15);
    caps.insert(Link("A".to_string(), "C".to_string()), 15);
    caps.insert(Link("B".to_string(), "D".to_string()), 10);
    caps.insert(Link("C".to_string(), "D".to_string()), 10);
    caps.insert(Link("B".to_string(), "C".to_string()), 10);

    let network = ToyNetwork { capacities: caps };
    let demands = vec![
        ToyDemand {
            id: 1,
            src: "A".to_string(),
            dst: "D".to_string(),
            vol: 7,
        },
        ToyDemand {
            id: 2,
            src: "A".to_string(),
            dst: "D".to_string(),
            vol: 7,
        },
        ToyDemand {
            id: 3,
            src: "A".to_string(),
            dst: "D".to_string(),
            vol: 7,
        },
    ]; // Total demand 21. Max throughput A->D with A->C down is 15. So it's heavily constrained.

    let mut state = SolverState {
        nodes_visited: 0,
        false_prunes: 0,
        network,
        demands,
        use_coralys: false,
        vault: PolicyVault::new(1000, ChronoPolicy),
        discovery: ChronoDiscovery { next_id: 1 },
        time: 1,
        obs_id: 1,
        found_solution: false,
    };

    println!("=== BASELINE SOLVER (No Memory) ===");
    let allocs = HashMap::new();
    state.solve(0, &allocs, "A->C", 100);
    let baseline_visited = state.nodes_visited as f64;
    println!("Nodes Visited: {}", baseline_visited);
    println!("Solution Found: {}", state.found_solution);
    println!("Vault Entries Created: {}", state.vault.entries.len());
    println!();

    println!("=== CORALYS MEMORY-GUIDED SOLVER ===");
    // Reset state but KEEP the vault!
    state.nodes_visited = 0;
    state.false_prunes = 0;
    state.found_solution = false;
    state.use_coralys = true;

    state.solve(0, &allocs, "A->C", 100);
    println!("Nodes Visited: {}", state.nodes_visited);
    println!("False Prunes: {}", state.false_prunes);
    println!("Solution Found: {}", state.found_solution);

    if baseline_visited > 0.0 {
        let reduction = 100.0 * (1.0 - (state.nodes_visited as f64 / baseline_visited));
        println!("Search Node Reduction: {:.1}%", reduction);
    }
}
