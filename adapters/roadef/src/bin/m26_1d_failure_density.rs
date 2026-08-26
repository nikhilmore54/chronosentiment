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
    path_nodes: &[u64],
    vol: f64,
    max_cap: f64,
    _time_slot: usize,
    _has_interventions: bool,
    avg_congestion: f64,
) -> String {
    let vol_cat = if vol > max_cap * 0.5 {
        "Vol:High"
    } else if vol > max_cap * 0.1 {
        "Vol:Med"
    } else {
        "Vol:Low"
    };
    let len_cat = if path_nodes.len() > 6 {
        "Len:Long"
    } else if path_nodes.len() > 3 {
        "Len:Med"
    } else {
        "Len:Short"
    };

    let crosses_core = if path_nodes.iter().any(|&n| n < 10) {
        "Core:Yes"
    } else {
        "Core:No"
    };

    let cong_cat = if avg_congestion > 0.8 {
        "Cong:High"
    } else if avg_congestion > 0.5 {
        "Cong:Med"
    } else {
        "Cong:Low"
    };

    format!("{}|{}|{}|{}", vol_cat, len_cat, crosses_core, cong_cat)
}

fn next_rand(seed: &mut u32) -> f64 {
    *seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
    (*seed % 1000) as f64 / 1000.0
}

fn main() -> anyhow::Result<()> {
    let mut tracker = AuditTracker::new();

    let stress_levels = [
        ("S1", 1.0, 1.5),
        ("S2", 1.0, 1.2),
        ("S3", 1.0, 1.0),
        ("S4", 1.0, 0.9),
        ("S5", 1.0, 0.8),
    ];

    let mut net_base = load_network("repo/challenge-roadef-2026-main/setA/setA-01-net.json")?;
    let mut tm_base = load_traffic_matrix("repo/challenge-roadef-2026-main/setA/setA-01-tm.json")?;
    let scenario = load_scenario("repo/challenge-roadef-2026-main/setA/setA-01-scenario.json")?;

    let mut rng_seed = 42u32;

    for (level, d_mult, c_mult) in &stress_levels {
        let mut net = net_base.clone();
        let mut tm = tm_base.clone();

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

        for _ in 0..10000 {
            let mut current_solution = Solution {
                srpaths: Vec::new(),
            };

            // Build a random partial solution of length 0 to 1 to simulate light background load
            let num_demands = (next_rand(&mut rng_seed) * 1.5) as usize;
            for _ in 0..num_demands {
                let d_idx = (next_rand(&mut rng_seed) * tm.demands.len() as f64) as usize;
                let mut w = Vec::new();
                if next_rand(&mut rng_seed) > 0.5 {
                    let wp = (next_rand(&mut rng_seed) * graph.nodes.len() as f64) as u64;
                    w.push(wp);
                }
                current_solution.srpaths.push(SrPath { d: d_idx, t: 0, w });
            }

            // Now pick a random target path to evaluate
            let target_d_idx = (next_rand(&mut rng_seed) * tm.demands.len() as f64) as usize;
            let demand = &tm.demands[target_d_idx];
            let mut w = Vec::new();
            if next_rand(&mut rng_seed) > 0.5 {
                let wp = (next_rand(&mut rng_seed) * graph.nodes.len() as f64) as u64;
                w.push(wp);
            }

            current_solution.srpaths.push(SrPath {
                d: target_d_idx,
                t: 0,
                w: w.clone(),
            });

            let res = evaluator.evaluate_solution(&current_solution);
            let is_success = res.valid && res.obj.is_finite();

            let mut path_nodes = vec![demand.s];
            path_nodes.extend_from_slice(&w);
            path_nodes.push(demand.t);

            let avg_congestion = if *c_mult < 1.0 {
                0.9
            } else if *c_mult < 1.5 {
                0.6
            } else {
                0.3
            };

            let tag = build_context_tag(
                &graph,
                &path_nodes,
                demand.v[0],
                max_cap,
                0,
                false,
                avg_congestion,
            );
            tracker.emit(tag, is_success);
        }
    }

    println!("=== M26.1D Failure Density Audit (Monte Carlo Tree Sample) ===");

    let total_obs = tracker.observations_emitted;
    let mut entropy = 0.0;
    let mut contexts: Vec<String> = tracker.contexts.keys().cloned().collect();

    contexts.sort_by(|a, b| {
        let (s_a, f_a) = tracker.contexts.get(a).unwrap_or(&(0, 0));
        let (s_b, f_b) = tracker.contexts.get(b).unwrap_or(&(0, 0));
        let r_a = *f_a as f64 / (f_a + s_a) as f64;
        let r_b = *f_b as f64 / (f_b + s_b) as f64;
        r_a.partial_cmp(&r_b).unwrap_or(std::cmp::Ordering::Equal)
    });

    println!(
        "{:<45} | {:<7} | {:<7} | {:<12}",
        "Context", "Success", "Failure", "Failure Rate"
    );
    println!("{:-<45}-|-{:-<7}-|-{:-<7}-|-{:-<12}", "", "", "", "");

    for tag in &contexts {
        let (s, f) = tracker.contexts.get(tag).unwrap();
        let total = s + f;
        if total < 50 {
            continue;
        } // Filter out noise

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
