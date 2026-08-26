//! E-001 Dual-Path Validation Harness
//!
//! Engineering Contract E-001: evaluate_solution_cached() must produce
//! semantically identical results to evaluate_solution_timed() for every
//! solution evaluated.
//!
//! Checks per instance (empty solution):
//!   - valid flag matches exactly
//!   - obj matches within 1e-9 relative tolerance (when both valid)
//!
//! Also prints cache statistics (hit_rate, dijkstra_fraction) per instance.
//!
//! Usage:
//!   cargo run --release --bin e001_dual_path -- [instance_prefix...]
//!
//! Defaults: setA-04 setA-10
//! Exit 0 = all PASS, Exit 1 = any FAIL or load error.

use roadef::evaluator::RoadefEvaluator;
use roadef::loader::{load_network, load_scenario, load_traffic_matrix};
use roadef::models::Solution;

const REPO: &str = "adapters/roadef/repo/challenge-roadef-2026-main/setA";
const TOL: f64 = 1e-9;

fn rel_err(a: f64, b: f64) -> f64 {
    let denom = a.abs().max(b.abs()).max(1e-300);
    (a - b).abs() / denom
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut instance_names: Vec<String> = Vec::new();
    for arg in args.iter().skip(1) {
        instance_names.push(arg.clone());
    }
    if instance_names.is_empty() {
        instance_names = vec!["setA-04".to_string(), "setA-10".to_string()];
    }

    eprintln!("=== E-001 Dual-Path Validation Harness ===");
    eprintln!("Instances: {}", instance_names.join(", "));
    eprintln!("Tolerance: {:.0e}", TOL);
    eprintln!();

    let mut all_pass = true;
    let mut total = 0usize;
    let mut pass_count = 0usize;

    for name in &instance_names {
        let net_path = format!("{}/{}-net.json", REPO, name);
        let tm_path = format!("{}/{}-tm.json", REPO, name);
        let scen_path = format!("{}/{}-scenario.json", REPO, name);

        let net = match load_network(&net_path) {
            Ok(n) => n,
            Err(e) => {
                eprintln!("[SKIP] {} — net load error: {}", name, e);
                all_pass = false;
                continue;
            }
        };
        let tm = match load_traffic_matrix(&tm_path) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("[SKIP] {} — tm load error: {}", name, e);
                all_pass = false;
                continue;
            }
        };
        let scenario = match load_scenario(&scen_path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[SKIP] {} — scenario load error: {}", name, e);
                all_pass = false;
                continue;
            }
        };

        total += 1;

        // Empty solution: no SR-path overrides, all demands use default ECMP.
        // This is the worst-case for Dijkstra call count (maximum cache benefit).
        let solution = Solution { srpaths: vec![] };

        let evaluator = RoadefEvaluator::new(&net, tm, scenario);

        // --- Reference path: evaluate_solution_timed ---
        let (ref_result, _ref_t) = evaluator.evaluate_solution_timed(&solution);

        // --- Cached path: evaluate_solution_cached ---
        let (cac_result, cac_t) = evaluator.evaluate_solution_cached(&solution);

        // --- Compare ---
        let mut failures: Vec<String> = Vec::new();

        if ref_result.valid != cac_result.valid {
            failures.push(format!(
                "valid mismatch: timed={} cached={}",
                ref_result.valid, cac_result.valid
            ));
        }

        if ref_result.valid && cac_result.valid {
            let err = rel_err(ref_result.obj, cac_result.obj);
            if err > TOL {
                failures.push(format!(
                    "obj mismatch: timed={:.10e} cached={:.10e} rel_err={:.2e}",
                    ref_result.obj, cac_result.obj, err
                ));
            }
        }

        let pass = failures.is_empty();
        let status = if pass { "PASS" } else { "FAIL" };
        if pass {
            pass_count += 1;
        } else {
            all_pass = false;
        }

        eprintln!(
            "[{}] {}  valid={}  obj={:.6e}  cache_hit={:.1}%  dijkstra_frac={:.1}%  hits={}  misses={}",
            status,
            name,
            cac_result.valid,
            if cac_result.valid { cac_result.obj } else { f64::INFINITY },
            cac_t.cache_hit_rate() * 100.0,
            cac_t.dijkstra_fraction() * 100.0,
            cac_t.dijkstra_cache_hits,
            cac_t.dijkstra_cache_misses,
        );

        for f in &failures {
            eprintln!("  !! {}", f);
        }

        // Also print the full timing report for the cached path
        cac_t.print_report(&format!("{} (cached)", name));
    }

    eprintln!();
    eprintln!("E-001 Result: {}/{} instances PASS", pass_count, total);

    if total == 0 {
        eprintln!(
            "E-001 VERDICT: SKIP — no instances loaded (check data path: {})",
            REPO
        );
        std::process::exit(1);
    } else if all_pass {
        eprintln!("E-001 VERDICT: PASS — evaluate_solution_cached() is semantically equivalent to evaluate_solution_timed()");
        std::process::exit(0);
    } else {
        eprintln!("E-001 VERDICT: FAIL — see failures above");
        std::process::exit(1);
    }
}
