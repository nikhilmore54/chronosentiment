use roadef::evaluator::RoadefEvaluator;
use roadef::graph::Digraph;
use roadef::loader::{load_network, load_scenario, load_traffic_matrix};
use roadef::models::{Network, Scenario, Solution, SrPath, TrafficMatrix};
use std::collections::{HashMap, HashSet};

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
        } else {
            0.0
        };

        let avg_freq = if unique_contexts > 0 {
            self.observations_emitted as f64 / unique_contexts as f64
        } else {
            0.0
        };

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
    budget_usage: f64,
) -> String {
    // Layer 1/2 Ecological Mapping

    // 1. Demand Volume Category
    let vol_cat = if demand_vol < max_cap * 0.05 {
        "Vol:Low"
    } else if demand_vol < max_cap * 0.20 {
        "Vol:Med"
    } else {
        "Vol:High"
    };

    // 2. Path Length Category
    let len_cat = if path.len() <= 2 {
        "Len:Short"
    } else if path.len() <= 4 {
        "Len:Med"
    } else {
        "Len:Long"
    };

    // 3. Core Traversal (Assume nodes 0..10 are core backbone nodes heuristically)
    let crosses_core = if path.iter().any(|&n| n < 10) {
        "Core:Yes"
    } else {
        "Core:No"
    };

    // 4. Temporal / Maintenance Context
    let maint_cat = if has_interventions {
        "Maint:Active"
    } else {
        "Maint:Clear"
    };

    // 5. Budget Context
    let budget_cat = if budget_usage > 0.9 {
        "Budget:Tight"
    } else if budget_usage > 0.5 {
        "Budget:Med"
    } else {
        "Budget:Ok"
    };

    format!(
        "{}|{}|{}|{}|{}",
        vol_cat, len_cat, crosses_core, maint_cat, budget_cat
    )
}

fn main() -> anyhow::Result<()> {
    let mut tracker = AuditTracker::new();

    let stress_levels = [
        ("Normal", 1.0, 1.0),
        ("Mild", 1.1, 0.9),
        ("Moderate", 1.2, 0.85),
        ("Elevated", 1.3, 0.8),
    ];

    for (level_name, d_mult, c_mult) in &stress_levels {
        let mut net = load_network("repo/challenge-roadef-2026-main/setA/setA-01-net.json")?;
        let mut tm = load_traffic_matrix("repo/challenge-roadef-2026-main/setA/setA-01-tm.json")?;
        let scenario = load_scenario("repo/challenge-roadef-2026-main/setA/setA-01-scenario.json")?;

        for d in &mut tm.demands {
            for v in &mut d.v {
                *v *= *d_mult;
            }
        }
        for l in &mut net.links {
            l.capacity *= *c_mult;
        }

        let graph = Digraph::new(&net);
        let evaluator = RoadefEvaluator::new(&net, tm.clone(), scenario.clone());
        let max_cap = net.links.iter().map(|l| l.capacity).fold(0.0_f64, f64::max);

        let mut current_solution = Solution {
            srpaths: Vec::new(),
        };
        let mut rng_seed = 42u32;
        fn next_rand(seed: &mut u32) -> f64 {
            *seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            (*seed % 1000) as f64 / 1000.0
        }

        for _iter in 0..100 {
            current_solution.srpaths.clear();
            let t = (next_rand(&mut rng_seed) * tm.num_time_slots as f64) as usize;
            let has_interventions = scenario.interventions.iter().any(|i| i.t == t);

            for _ in 0..10 {
                let d_idx = (next_rand(&mut rng_seed) * tm.demands.len() as f64) as usize;
                let demand = &tm.demands[d_idx];

                let mut w = Vec::new();
                if next_rand(&mut rng_seed) > 0.5 {
                    let wp = (next_rand(&mut rng_seed) * graph.nodes.len() as f64) as u64;
                    w.push(wp);
                }

                current_solution.srpaths.push(SrPath {
                    d: d_idx,
                    t,
                    w: w.clone(),
                });

                let res = evaluator.evaluate_solution(&current_solution);
                let is_success = res.valid && res.obj.is_finite();

                let mut path_nodes = vec![demand.s];
                path_nodes.extend_from_slice(&w);
                path_nodes.push(demand.t);

                let tag = build_context_tag(
                    &graph,
                    &path_nodes,
                    demand.v[t],
                    max_cap,
                    t,
                    has_interventions,
                    0.5,
                );
                tracker.emit(tag, is_success);

                if !is_success {
                    current_solution.srpaths.pop();
                }
            }
        }
    }

    tracker.print_metrics();
    Ok(())
}
