use std::collections::HashMap;

use roadef::evaluator::RoadefEvaluator;
use roadef::graph::Digraph;
use roadef::loader::{load_network, load_scenario, load_traffic_matrix};
use roadef::models::{Solution, SrPath};

struct AuditTracker {
    observations_emitted: usize,
    contexts: HashMap<String, ContextStats>,
}

struct ContextStats {
    success: usize,
    failure: usize,
    survival_depths: Vec<usize>,
}

impl AuditTracker {
    fn new() -> Self {
        Self {
            observations_emitted: 0,
            contexts: HashMap::new(),
        }
    }

    fn emit_failure(&mut self, context: String) {
        self.observations_emitted += 1;
        let entry = self.contexts.entry(context).or_insert(ContextStats {
            success: 0,
            failure: 0,
            survival_depths: Vec::new(),
        });
        entry.failure += 1;
        entry.survival_depths.push(0);
    }

    fn emit_success(&mut self, context: String, depth_survived: usize) {
        self.observations_emitted += 1;
        let entry = self.contexts.entry(context).or_insert(ContextStats {
            success: 0,
            failure: 0,
            survival_depths: Vec::new(),
        });
        entry.success += 1;
        entry.survival_depths.push(depth_survived);
    }
}

fn build_context_tag(
    graph: &Digraph,
    path_nodes: &[u64],
    vol: f64,
    max_cap: f64,
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

    for (_level, d_mult, c_mult) in &stress_levels {
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

        for _ in 0..5000 {
            let mut current_solution = Solution {
                srpaths: Vec::new(),
            };
            let mut trajectory = Vec::new();
            let mut failed = false;
            let mut fail_depth = 0;

            let total_demands = tm.demands.len().min(40);

            for d_idx in 0..total_demands {
                let demand = &tm.demands[d_idx];

                let mut w = Vec::new();
                if next_rand(&mut rng_seed) > 0.5 {
                    let wp = (next_rand(&mut rng_seed) * graph.nodes.len() as f64) as u64;
                    w.push(wp);
                }

                current_solution.srpaths.push(SrPath {
                    d: d_idx,
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
                let tag =
                    build_context_tag(&graph, &path_nodes, demand.v[0], max_cap, avg_congestion);

                if is_success {
                    trajectory.push(tag);
                } else {
                    tracker.emit_failure(tag);
                    failed = true;
                    fail_depth = d_idx;
                    break;
                }
            }

            let final_depth = if failed { fail_depth } else { total_demands };
            for (i, tag) in trajectory.into_iter().enumerate() {
                let survival = final_depth.saturating_sub(i);
                tracker.emit_success(tag, survival);
            }
        }
    }

    println!("=== M26.1E Survival Curve Audit (Monte Carlo Rollouts) ===");

    let mut contexts: Vec<String> = tracker.contexts.keys().cloned().collect();
    contexts.sort_by(|a, b| {
        let stats_a = tracker.contexts.get(a).unwrap();
        let stats_b = tracker.contexts.get(b).unwrap();
        let r_a = stats_a.failure as f64 / (stats_a.failure + stats_a.success) as f64;
        let r_b = stats_b.failure as f64 / (stats_b.failure + stats_b.success) as f64;
        r_a.partial_cmp(&r_b).unwrap_or(std::cmp::Ordering::Equal)
    });

    println!(
        "{:<45} | {:<7} | {:<7} | {:<12} | {:<10} | {:<10} | {:<10}",
        "Context", "Success", "Failure", "Failure Rate", "Mean Depth", "Med Depth", "P95 Depth"
    );
    println!(
        "{:-<45}-|-{:-<7}-|-{:-<7}-|-{:-<12}-|-{:-<10}-|-{:-<10}-|-{:-<10}",
        "", "", "", "", "", "", ""
    );

    for tag in &contexts {
        let stats = tracker.contexts.get(tag).unwrap();
        let total = stats.success + stats.failure;
        if total < 50 {
            continue;
        }

        let rate = stats.failure as f64 / total as f64;

        let mut sorted_depths = stats.survival_depths.clone();
        sorted_depths.sort_unstable();

        let mean = if sorted_depths.is_empty() {
            0.0
        } else {
            sorted_depths.iter().sum::<usize>() as f64 / sorted_depths.len() as f64
        };

        let median = if sorted_depths.is_empty() {
            0.0
        } else {
            sorted_depths[sorted_depths.len() / 2] as f64
        };

        let p95 = if sorted_depths.is_empty() {
            0.0
        } else {
            let idx = (sorted_depths.len() as f64 * 0.95) as usize;
            sorted_depths[idx.min(sorted_depths.len() - 1)] as f64
        };

        println!(
            "{:<45} | {:<7} | {:<7} | {:>5.1}%       | {:>10.1} | {:>10.1} | {:>10.1}",
            tag,
            stats.success,
            stats.failure,
            rate * 100.0,
            mean,
            median,
            p95
        );
    }

    Ok(())
}
