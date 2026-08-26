/// M20 Phase 1 — Evaluator Performance Model profiler
///
/// Runs N evaluations on each specified instance using an empty solution
/// (all demands routed via default ECMP — maximum Dijkstra call count,
/// worst-case routing cost). Accumulates EvalTimings across all runs and
/// prints the Performance Model table.
///
/// Usage:
///   cargo run --release --bin eval_profiler -- [N] [instance_prefix...]
///
/// Defaults:
///   N = 20
///   instances = setA-04 setA-10
///
/// Example:
///   cargo run --release --bin eval_profiler -- 50 setA-04 setA-10 setA-17
///
/// Output goes to stderr (consistent with campaign_engine convention).
/// The binary exits 0 on success, 1 on any load error.
use roadef::evaluator::{EvalTimings, RoadefEvaluator};
use roadef::loader::{load_network, load_scenario, load_traffic_matrix};
use roadef::models::Solution;

const REPO: &str = "repo/challenge-roadef-2026-main/setA";

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Parse optional N (first arg if numeric)
    let mut n_evals: usize = 20;
    let mut instance_names: Vec<String> = Vec::new();

    for arg in args.iter().skip(1) {
        if let Ok(n) = arg.parse::<usize>() {
            n_evals = n;
        } else {
            instance_names.push(arg.clone());
        }
    }

    if instance_names.is_empty() {
        instance_names = vec!["setA-04".to_string(), "setA-10".to_string()];
    }

    eprintln!("=== M20 Phase 1 — Evaluator Performance Model ===");
    eprintln!("Evaluations per instance: {}", n_evals);
    eprintln!("Instances: {}", instance_names.join(", "));
    eprintln!();

    let mut all_ok = true;

    for name in &instance_names {
        let net_path = format!("{}/{}-net.json", REPO, name);
        let tm_path = format!("{}/{}-tm.json", REPO, name);
        let scen_path = format!("{}/{}-scenario.json", REPO, name);

        let net = match load_network(&net_path) {
            Ok(n) => n,
            Err(e) => {
                eprintln!("ERROR loading {}: {}", net_path, e);
                all_ok = false;
                continue;
            }
        };
        let tm = match load_traffic_matrix(&tm_path) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("ERROR loading {}: {}", tm_path, e);
                all_ok = false;
                continue;
            }
        };
        let scenario = match load_scenario(&scen_path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("ERROR loading {}: {}", scen_path, e);
                all_ok = false;
                continue;
            }
        };

        let num_demands = tm.demands.len();
        let num_nodes = net.nodes.len();
        let num_links = net.links.len();
        let num_slots = tm.num_time_slots;

        let evaluator = RoadefEvaluator::new(&net, tm, scenario);
        let empty_solution = Solution { srpaths: vec![] };

        // Warm-up: one run to avoid cold-start effects
        let _ = evaluator.evaluate_solution_timed(&empty_solution);

        // Accumulate timings across N runs
        let mut acc = EvalTimings::default();
        let mut valid_count = 0usize;

        for _ in 0..n_evals {
            let (result, t) = evaluator.evaluate_solution_timed(&empty_solution);
            acc.segment_check_ns += t.segment_check_ns;
            acc.budget_check_ns += t.budget_check_ns;
            acc.routing_ns += t.routing_ns;
            acc.objective_ns += t.objective_ns;
            acc.time_slots_processed += t.time_slots_processed;
            acc.demand_route_calls += t.demand_route_calls;
            acc.dijkstra_calls += t.dijkstra_calls;
            acc.dijkstra_ns += t.dijkstra_ns;
            acc.ecmp_ns += t.ecmp_ns;
            acc.dijkstra_cache_hits += t.dijkstra_cache_hits;
            acc.dijkstra_cache_misses += t.dijkstra_cache_misses;
            if result.valid {
                valid_count += 1;
            }
        }

        // Average per evaluation
        let avg = EvalTimings {
            segment_check_ns: acc.segment_check_ns / n_evals as u64,
            budget_check_ns: acc.budget_check_ns / n_evals as u64,
            routing_ns: acc.routing_ns / n_evals as u64,
            objective_ns: acc.objective_ns / n_evals as u64,
            time_slots_processed: acc.time_slots_processed / n_evals as u32,
            demand_route_calls: acc.demand_route_calls / n_evals as u64,
            dijkstra_calls: acc.dijkstra_calls / n_evals as u64,
            dijkstra_ns: acc.dijkstra_ns / n_evals as u64,
            ecmp_ns: acc.ecmp_ns / n_evals as u64,
            dijkstra_cache_hits: acc.dijkstra_cache_hits / n_evals as u64,
            dijkstra_cache_misses: acc.dijkstra_cache_misses / n_evals as u64,
        };

        let total_us = avg.total_ns() as f64 / 1_000.0;
        let dl = num_demands * num_links;

        eprintln!("--- {} ---", name);
        eprintln!(
            "  nodes={} links={} demands={} slots={} demands×links={} valid={}/{}",
            num_nodes, num_links, num_demands, num_slots, dl, valid_count, n_evals
        );
        avg.print_report(name);

        // Per-demand and per-link breakdown
        eprintln!("  Derived scaling indicators (avg per eval):");
        eprintln!(
            "    routing_us / demand        = {:.2}",
            avg.routing_ns as f64 / 1_000.0 / num_demands as f64
        );
        eprintln!(
            "    routing_us / (d×l)         = {:.4}",
            avg.routing_ns as f64 / 1_000.0 / dl as f64
        );
        eprintln!(
            "    objective_us / link        = {:.4}",
            avg.objective_ns as f64 / 1_000.0 / num_links as f64
        );
        eprintln!("    total_us                   = {:.1}", total_us);
        eprintln!("    dijkstra_calls             = {}", avg.dijkstra_calls);
        eprintln!(
            "    dijkstra_us / call         = {:.2}",
            if avg.dijkstra_calls > 0 {
                avg.routing_ns as f64 / 1_000.0 / avg.dijkstra_calls as f64
            } else {
                0.0
            }
        );
        eprintln!();
    }

    if !all_ok {
        std::process::exit(1);
    }
}
