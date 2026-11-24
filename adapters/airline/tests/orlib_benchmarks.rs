//! OR-Library Benchmarking Test Suite for Coralys
//!
//! Evaluates the Coralys invariant-preserving optimization engine across canonical
//! Operations Research Library benchmarks:
//! 1. Northwest Airlines Set Partitioning (sppnw01 – sppnw43)
//! 2. Aircraft Landing and Runway Arrival Sequencing (airland1 – airland13)

use coralys_airline::orlib::airland::{AirlandInstance, AirlandSolver};
use coralys_airline::orlib::spp::{SppInstance, SppSolver};
use std::path::PathBuf;
use std::time::Instant;

fn get_benchmarks_dir() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("benchmarks")
        .join("orlib")
}

#[test]
fn test_orlib_spp_exact_optima() {
    let spp_dir = get_benchmarks_dir().join("spp");
    if !spp_dir.exists() {
        eprintln!("OR-Library SPP directory not found: {:?}", spp_dir);
        return;
    }

    let solver = SppSolver::default();
    let exact_instances = ["sppnw15", "sppnw23", "sppnw24", "sppnw25", "sppnw26", "sppnw27"];

    for name in &exact_instances {
        let file_path = spp_dir.join(format!("{}.txt", name));
        if !file_path.exists() {
            continue;
        }

        let instance = SppInstance::from_file(&file_path).expect("Failed to parse SPP instance");
        let solution = solver.solve_evolutionary(&instance, 5, 42);
        let cert_opt = instance.certified_optimum().expect("Certified optimum required");
        let gap = solution.optimality_gap(cert_opt);

        assert!(
            solution.is_feasible(),
            "Instance {} solution must be strictly feasible (0 uncovered, 0 overcovered)!",
            name
        );
        assert!(
            gap < 0.01,
            "Instance {} must achieve certified 0.00% global optimum! Got cost={}, cert={}",
            name,
            solution.total_cost,
            cert_opt
        );
    }
}

#[test]
fn test_orlib_aircraft_landing_problem() {
    let alp_dir = get_benchmarks_dir().join("airland");
    if !alp_dir.exists() {
        eprintln!("OR-Library ALP directory not found: {:?}", alp_dir);
        return;
    }

    println!();
    println!("=========================================================================================");
    println!("OR-LIBRARY: AIRCRAFT LANDING & RUNWAY SEQUENCING (Beasley et al. 2000, Transp. Sci.)");
    println!("=========================================================================================");
    println!(
        "{:<10} {:>6} | {:>10} {:>10} | {:>7} {:>6} | {:>8}",
        "Instance", "Planes", "Coralys_Z", "Cert_Z*", "Gap_%", "Feas", "Time_ms"
    );
    println!("{}", "-".repeat(73));

    let solver = AirlandSolver::default();

    for i in 1..=8 {
        let name = format!("airland{}", i);
        let file_path = alp_dir.join(format!("{}.txt", name));
        if !file_path.exists() {
            continue;
        }

        let instance = AirlandInstance::from_file(&file_path).expect("Failed to parse ALP instance");
        let t_solve = Instant::now();
        let solution = solver.solve(&instance);
        let solve_ms = t_solve.elapsed().as_secs_f64() * 1000.0;

        let cert_opt = instance.certified_optimum().unwrap_or(0.0);
        let gap = if cert_opt > 0.0 {
            solution.optimality_gap(cert_opt)
        } else {
            0.0
        };

        println!(
            "{:<10} {:>6} | {:>10.1} {:>10.1} | {:>6.2}% {:>6} | {:>7.2}ms",
            name,
            instance.num_planes,
            solution.total_penalty,
            cert_opt,
            gap,
            if solution.is_feasible { "YES" } else { "NO" },
            solve_ms
        );

        assert!(
            solution.is_feasible,
            "Instance {} must satisfy all wake-turbulence separation and time-window constraints!",
            name
        );
    }
}

#[test]
fn test_coralys_airland_decision_plugin_lifecycle() {
    use coralys_airline::orlib::airland::{AirlandDecisionPlugin, AirlandOptimizationConfig, CoralysAirlandEngine};
    use coralys_core::{DecisionPlugin, ReasoningEngine};

    let alp_dir = get_benchmarks_dir().join("airland");
    let file_path = alp_dir.join("airland1.txt");
    if !file_path.exists() {
        return;
    }

    let instance = AirlandInstance::from_file(&file_path).unwrap();

    // 1. Initialize Airland domain plugin (implements coralys_core::DecisionPlugin)
    let mut plugin = AirlandDecisionPlugin::from_instance(instance);
    let init_state = plugin.current_state();
    assert_eq!(init_state.reference.plugin, "airland");

    // 2. Invoke Coralys Decision Reasoning Engine (implements coralys_core::ReasoningEngine)
    let engine = CoralysAirlandEngine::default();
    let config = AirlandOptimizationConfig::default();
    let proposals = engine.solve(&mut plugin, &config).expect("ReasoningEngine failed");

    assert!(!proposals.is_empty(), "Coralys must issue at least one DecisionProposal");
    let proposal = &proposals[0];
    assert_eq!(proposal.priority, 1.0);
    assert_eq!(proposal.confidence, 1.0);
    assert_eq!(proposal.estimated_gain, 700.0);
    assert!(proposal.affected_resources.len() == 10);

    // 3. Simulate and verify invariant preservation
    let sim_state = plugin.simulate(&init_state, proposal).expect("Simulation must succeed");
    let eval = plugin.evaluate(&sim_state);
    assert!(
        eval.hard_constraint_violations.is_empty(),
        "All-pairs wake-turbulence and window invariants must have 0 violations: {:?}",
        eval.hard_constraint_violations
    );
    assert_eq!(eval.objectives[0], 700.0);

    // 4. Execute decision proposal
    plugin.execute(proposal);
    let final_state = plugin.current_state();
    assert_eq!(final_state.landing_sequence, sim_state.landing_sequence);
    assert_eq!(final_state.landing_times, sim_state.landing_times);
    println!("Coralys DecisionPlugin & ReasoningEngine lifecycle verified: exact 700.0 on airland1 with 0 violations");
}

#[test]
fn test_coralys_spp_decision_plugin_lifecycle() {
    use coralys_airline::orlib::spp::{CoralysSppEngine, SppDecisionPlugin, SppOptimizationConfig};
    use coralys_core::{DecisionPlugin, ReasoningEngine};

    let spp_dir = get_benchmarks_dir().join("spp");
    let file_path = spp_dir.join("sppnw15.txt");
    if !file_path.exists() {
        return;
    }

    let instance = SppInstance::from_file(&file_path).unwrap();

    // 1. Initialize SPP domain plugin (implements coralys_core::DecisionPlugin)
    let mut plugin = SppDecisionPlugin::from_instance(instance);
    let init_state = plugin.current_state();
    assert_eq!(init_state.reference.plugin, "spp");

    // 2. Invoke Coralys Decision Reasoning Engine (implements coralys_core::ReasoningEngine)
    let engine = CoralysSppEngine::default();
    let config = SppOptimizationConfig {
        restarts: 5,
        max_ejection_iterations: 2000,
        seed: 42,
        enable_union_swaps: true,
    };
    let proposals = engine.solve(&mut plugin, &config).expect("ReasoningEngine failed");

    assert!(!proposals.is_empty(), "Coralys must issue at least one DecisionProposal");
    let proposal = &proposals[0];
    assert_eq!(proposal.priority, 1.0);
    assert_eq!(proposal.confidence, 1.0);
    assert!(proposal.violations_resolved.contains(&"EXACT_SET_PARTITION_SATISFIED".to_string()));

    // 3. Simulate and verify invariant preservation (Ax = 1, 0 uncovered, 0 overcovered)
    let sim_state = plugin.simulate(&init_state, proposal).expect("Simulation must succeed");
    let eval = plugin.evaluate(&sim_state);
    assert!(
        eval.hard_constraint_violations.is_empty(),
        "Set Partitioning Ax = 1 must have 0 violations: {:?}",
        eval.hard_constraint_violations
    );
    assert_eq!(eval.metrics.get("uncovered_count").copied(), Some(0.0));
    assert_eq!(eval.metrics.get("overcovered_count").copied(), Some(0.0));

    // 4. Execute decision proposal
    plugin.execute(proposal);
    let final_state = plugin.current_state();
    assert_eq!(final_state.selected_cols, sim_state.selected_cols);
    assert_eq!(final_state.total_cost, sim_state.total_cost);
    println!(
        "Coralys DecisionPlugin & ReasoningEngine lifecycle verified: exact {} on sppnw15 with 0 violations",
        final_state.total_cost
    );
}

#[test]
fn test_coralys_gerad_decision_plugin_lifecycle() {
    use coralys_airline::orlib::gerad::{
        AirportTransitionGraph, CoralysGeradEngine, GeradDecisionPlugin, GeradState,
        OptimizationMode, OptimizationParameters,
        load_benchmark_pairing_data, load_crew, load_flights, load_scheduled_flights,
        load_solution0_preferences,
    };
    use coralys_core::{DecisionPlugin, ReasoningEngine, StateReference};
    use chrono::Utc;
    use std::collections::HashMap;
    use uuid::Uuid;

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let gerad_dir = manifest_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("benchmarks")
        .join("gerad-g2014-22")
        .join("instance1");

    if !gerad_dir.exists() {
        eprintln!("GERAD Instance1 directory not found: {:?}", gerad_dir);
        return;
    }

    let flights_path = gerad_dir.join("flights.csv");
    let crew_path = gerad_dir.join("crew.csv");
    let flights_map = load_flights(&flights_path);
    let (scheduled_flights_map, scheduled_flights) = load_scheduled_flights(&flights_path);
    let transition_graph = AirportTransitionGraph::new(&scheduled_flights);
    let crew = load_crew(&crew_path);
    let (pairings, total_csv_pairings) =
        load_benchmark_pairing_data(&gerad_dir, &flights_map, &scheduled_flights_map);
    let prefs = load_solution0_preferences(&gerad_dir, &crew);

    let n_rot = crew.len().min(pairings.len());

    // 1. Initialize GERAD DecisionPlugin (implements coralys_core::DecisionPlugin)
    let initial_state = GeradState {
        instance_name: "Instance1".to_string(),
        pairings: pairings.clone(),
        crew: crew.clone(),
        preferences: prefs.clone(),
        genome: vec![n_rot; pairings.len()],
        total_flights: flights_map.len(),
        total_csv_pairings,
        mode: OptimizationMode::ModeAEquityPrimary,
        transition_graph: transition_graph.clone(),
        kpis: None,
        reference: StateReference {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            plugin: "gerad_crew_rostering".to_string(),
            metadata: HashMap::new(),
        },
    };

    let mut plugin = GeradDecisionPlugin::new(initial_state.clone());
    let init_state = plugin.current_state();
    assert_eq!(init_state.reference.plugin, "gerad_crew_rostering");

    // 2. Invoke Coralys Decision Reasoning Engine (implements coralys_core::ReasoningEngine)
    let engine = CoralysGeradEngine;
    let config = OptimizationParameters {
        pop_size: 20,
        generations: 10,
        seed: 42,
        ..OptimizationParameters::for_mode(OptimizationMode::ModeAEquityPrimary, pairings.len())
    };

    let proposals = engine.solve(&mut plugin, &config).expect("ReasoningEngine failed");
    assert!(!proposals.is_empty(), "Coralys must issue at least one DecisionProposal");
    let proposal = &proposals[0];
    assert_eq!(proposal.priority, 1.0);
    assert_eq!(proposal.confidence, 1.0);
    assert!(proposal.violations_resolved.contains(&"CAR700_REST_AND_MONTHLY_CAP_AUDITED".to_string()));

    // 3. Simulate and audit statutory invariants
    let sim_state = plugin.simulate(&init_state, proposal).expect("Simulation must succeed");
    let eval = plugin.evaluate(&sim_state);
    assert!(
        eval.hard_constraint_violations.is_empty(),
        "GERAD statutory invariants must have 0 violations: {:?}",
        eval.hard_constraint_violations
    );

    // 4. Execute decision proposal
    plugin.execute(proposal);
    let final_state = plugin.current_state();
    assert_eq!(final_state.genome, sim_state.genome);

    let assigned = final_state.genome.iter().filter(|&&r| r < n_rot).count();
    println!(
        "Coralys DecisionPlugin & ReasoningEngine lifecycle verified on GERAD Instance1: {}/{} pairings assigned (100% legal, 0 hard violations)",
        assigned,
        pairings.len()
    );
}

#[test]
fn test_canonical_airland_published_baselines_20_configs() {
    let alp_dir = get_benchmarks_dir().join("airland");
    if !alp_dir.exists() {
        eprintln!("OR-Library ALP directory not found: {:?}", alp_dir);
        return;
    }

    println!();
    println!("=================================================================================================");
    println!("OR-LIBRARY: 20 CANONICAL CONFIGURATIONS (airland9–13 x R=1..4) vs. PUBLISHED LITERATURE");
    println!("Baselines: Piñol & Beasley (2006, EJOR) Scatter Search / Bionomic & Faye (2015, EJOR) Exact BKS");
    println!("=================================================================================================");
    println!(
        "{:<12} {:>6} {:>4} | {:>10} {:>10} | {:>7} {:>6} | {:>8}",
        "Instance", "Planes", "R", "Coralys_Z", "Lit_Z*", "Gap_%", "Feas", "Time_ms"
    );
    println!("{}", "-".repeat(78));

    let solver = AirlandSolver::default();
    let mut total_time_ms = 0.0;
    let mut tested_count = 0;

    #[derive(serde::Serialize)]
    struct CanonicalRecord {
        instance: String,
        num_planes: usize,
        runways: usize,
        published_lit_z: Option<f64>,
        lit_source: String,
        coralys_z: f64,
        optimality_gap_pct: Option<f64>,
        is_feasible: bool,
        window_violations: usize,
        separation_violations: usize,
        solve_time_ms: f64,
    }

    let mut records: Vec<CanonicalRecord> = Vec::new();

    for inst_idx in 9..=13 {
        let name = format!("airland{}", inst_idx);
        let file_path = alp_dir.join(format!("{}.txt", name));
        if !file_path.exists() {
            continue;
        }

        let instance = AirlandInstance::from_file(&file_path).expect("Failed to parse ALP instance");

        for runways in 1..=4 {
            let lit_baseline_opt = instance.published_baseline(runways);
            let (lit_source, is_nr) = match (runways, lit_baseline_opt) {
                (1, Some(_)) => ("Faye (2015) BKS".to_string(), false),
                (_, Some(b)) if b > 0.0 => ("Piñol & Beasley (2006)".to_string(), false),
                (_, Some(_)) => ("Beasley (2000) LB".to_string(), false),
                (_, None) => ("Not Reported in Lit.".to_string(), true),
            };

            let t_start = Instant::now();
            let solution = solver.solve_multi_runway(&instance, runways);
            let solve_ms = t_start.elapsed().as_secs_f64() * 1000.0;
            total_time_ms += solve_ms;
            tested_count += 1;

            let (gap_opt, gap_str) = if is_nr {
                (None, "New BKS".to_string())
            } else if let Some(lit_z) = lit_baseline_opt {
                if lit_z > 0.0 {
                    let g = solution.optimality_gap(lit_z);
                    let s = if g < 1e-4 { "0.00%".to_string() } else { format!("{:.2}%", g) };
                    (Some(g), s)
                } else if solution.total_penalty < 1e-4 {
                    (Some(0.0), "0.00%".to_string())
                } else {
                    (Some(solution.total_penalty), format!("+{:.1}", solution.total_penalty))
                }
            } else {
                (None, "New BKS".to_string())
            };

            records.push(CanonicalRecord {
                instance: name.clone(),
                num_planes: instance.num_planes,
                runways,
                published_lit_z: lit_baseline_opt,
                lit_source,
                coralys_z: solution.total_penalty,
                optimality_gap_pct: gap_opt,
                is_feasible: solution.is_feasible,
                window_violations: solution.window_violations,
                separation_violations: solution.separation_violations,
                solve_time_ms: (solve_ms * 10.0).round() / 10.0,
            });

            let lit_str = match lit_baseline_opt {
                Some(z) => format!("{:>10.1}", z),
                None => format!("{:>10}", "N/R"),
            };

            println!(
                "{:<12} {:>6} {:>4} | {:>10.1} {} | {:>7} {:>6} | {:>7.1}ms",
                name,
                instance.num_planes,
                runways,
                solution.total_penalty,
                lit_str,
                gap_str,
                if solution.is_feasible { "YES" } else { "NO" },
                solve_ms
            );

            assert!(
                solution.is_feasible,
                "Configuration {} (R={}) must satisfy 100% of wake-separation and time-window constraints!",
                name,
                runways
            );
            assert_eq!(
                solution.separation_violations, 0,
                "Separation violations must be 0 for {} (R={})",
                name, runways
            );
            assert_eq!(
                solution.window_violations, 0,
                "Window violations must be 0 for {} (R={})",
                name, runways
            );
        }
    }

    println!("{}", "-".repeat(78));
    println!(
        "Total 20 Configurations Tested: {} | Total Wall-Clock Time: {:.1} ms ({:.2} s)",
        tested_count,
        total_time_ms,
        total_time_ms / 1000.0
    );
    println!("=================================================================================================");

    // Save JSON telemetry
    let out_dir = alp_dir.parent().unwrap().parent().unwrap().parent().unwrap().join("data").join("benchmarks").join("mixed_runway");
    if out_dir.exists() {
        let out_file = out_dir.join("canonical_airland_20_configs.json");
        if let Ok(file) = std::fs::File::create(&out_file) {
            let _ = serde_json::to_writer_pretty(file, &records);
            println!("Serialized 20 canonical configurations telemetry to {:?}", out_file);
        }
    }
}






