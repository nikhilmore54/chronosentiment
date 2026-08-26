//! GERAD G2014-22 — Coralys-Native Evolutionary Scheduler
//!
//! # Baseline Freeze — GERAD Coralys v1.0 (2026-08-01)
//!
//! This file is frozen as the Section 2.17 reference baseline.
//! Do not modify the algorithm, parameters, or output format.
//! Future experiments must reference this file as the v1.0 baseline.
//! Any improvements belong in new test files (e.g. `gerad_coralys_v2.rs`).
//!
//! Baseline parameters: pop=50, gen=200, xover=0.80, tournament_k=3, seed=42
//! Baseline result: see Section 2.17 of UltraCrew_Pairing_Topology_Mutation_Evaluation.md
//!
//! # Objective
//!
//! Compare a Coralys-native evolutionary optimizer against the baseline
//! Greedy + LocalSearch pipeline (`gerad_e2e.rs`) on the same 7 GERAD
//! instances, using the same domain infrastructure (flight parser, duty
//! builder, pairing builder, constraint engine, objective function).
//!
//! # Architecture
//!
//! The only difference from `gerad_e2e.rs` is the assignment engine:
//!
//! ```text
//! gerad_e2e.rs:
//!   flights.csv → pairings → GreedyScheduler → LocalSearch → roster
//!
//! gerad_coralys.rs:
//!   flights.csv → pairings → EvolutionaryScheduler → roster
//! ```
//!
//! # Genome
//!
//! A genome is a `Vec<usize>` of length P (number of pairings).
//! `genome[i]` is the rotation index (0..R) assigned to pairing i.
//! All rotations must receive at least one pairing (feasibility constraint).
//!
//! # Evolutionary Algorithm
//!
//! - Population size: 50
//! - Generations: 200
//! - Selection: tournament (k=3)
//! - Crossover: one-point (probability 0.8)
//! - Mutation: random gene reassignment (probability 1/P per gene)
//! - Fitness: WorkloadBalanceObjective (lower is better)
//! - Repair: if any rotation is empty after crossover/mutation, reassign
//!   one pairing from the most-loaded rotation to the empty one.
//!
//! # Initialization strategies compared
//!
//! - Round-robin: pairing i → rotation (i % R)
//! - Random: pairing i → uniform random rotation
//!
//! Both are run and reported. The greedy baseline from gerad_e2e.rs is
//! reported for comparison (re-run here to avoid cross-test dependency).
//!
//! # Modeling Assumptions
//!
//! Identical to gerad_e2e.rs (see that file for full documentation).
//! The pairing construction pipeline is shared verbatim.

#![allow(non_snake_case, dead_code)]

use coralys_airline::domain::crew::CrewId;
use coralys_airline::domain::duty::{Duty, DutyId};
use coralys_airline::domain::flight::{
    AircraftType, AirportCode, FlightLeg, FlightLegId, FlightNumber,
};
use coralys_airline::domain::pairing::{Pairing, PairingId};
use coralys_airline::domain::roster::{PlanningPeriod, Roster, RosterId};
use coralys_airline::domain::rotation::{Rotation, RotationId};
use coralys_airline::optimization::cost::CostEvaluator;
use coralys_airline::optimization::metrics::OptimizationMetrics;
use coralys_airline::optimization::objective::{SchedulingObjective, WorkloadBalanceObjective};
use coralys_airline::optimization::search::greedy::GreedyScheduler;

use chrono::{DateTime, Utc};
use rand::prelude::*;
use std::collections::HashMap;
use std::path::Path;

// ── Constants ─────────────────────────────────────────────────────────────────

const DEFAULT_BASE: &str = "YUL";
const LAYOVER_REST_HOURS: f64 = 8.0;

// Evolutionary algorithm parameters
const POP_SIZE: usize = 50;
const GENERATIONS: usize = 200;
const CROSSOVER_PROB: f64 = 0.80;
const TOURNAMENT_K: usize = 3;
const SEED: u64 = 42;

// ── CSV parsing (shared with gerad_e2e.rs) ───────────────────────────────────

#[derive(Debug, Clone)]
struct RawFlight {
    flight_id: String,
    flight_number: String,
    origin: String,
    destination: String,
    departure_utc: DateTime<Utc>,
    arrival_utc: DateTime<Utc>,
    aircraft_type: String,
}

fn parse_utc(s: &str) -> DateTime<Utc> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return dt.into();
    }
    let with_z = if s.ends_with('Z') {
        s.to_string()
    } else {
        format!("{s}Z")
    };
    DateTime::parse_from_rfc3339(&with_z)
        .unwrap_or_else(|_| panic!("Cannot parse UTC timestamp: {s}"))
        .into()
}

fn load_flights(path: &Path) -> Vec<RawFlight> {
    let mut rdr = csv::Reader::from_path(path).expect("Cannot open flights.csv");
    let mut flights = Vec::new();
    for result in rdr.deserialize::<HashMap<String, String>>() {
        let row = result.expect("CSV parse error in flights.csv");
        let origin = pad_airport(row["origin"].trim());
        let dest = pad_airport(row["destination"].trim());
        flights.push(RawFlight {
            flight_id: row["flight_id"].clone(),
            flight_number: row
                .get("flight_number")
                .cloned()
                .unwrap_or_else(|| row["flight_id"].clone()),
            origin,
            destination: dest,
            departure_utc: parse_utc(row["departure_utc"].trim()),
            arrival_utc: parse_utc(row["arrival_utc"].trim()),
            aircraft_type: row
                .get("aircraft_type")
                .cloned()
                .unwrap_or_else(|| "B737".to_string()),
        });
    }
    flights.sort_by_key(|f| f.departure_utc);
    flights
}

fn pad_airport(s: &str) -> String {
    if s.len() >= 3 {
        s[..3].to_uppercase()
    } else {
        format!("{:A<3}", s.to_uppercase())
    }
}

#[derive(Debug, Clone)]
struct RawCrew {
    crew_id: String,
}

fn load_crew(path: &Path) -> Vec<RawCrew> {
    let mut rdr = csv::Reader::from_path(path).expect("Cannot open crew.csv");
    let mut crew = Vec::new();
    for result in rdr.deserialize::<HashMap<String, String>>() {
        let row = result.expect("CSV parse error in crew.csv");
        crew.push(RawCrew {
            crew_id: row["crew_id"].clone(),
        });
    }
    crew
}

// ── Pairing construction (identical to gerad_e2e.rs) ─────────────────────────

fn build_pairings_from_flights(
    raw_flights: &[RawFlight],
    layover_rest_hours: f64,
) -> (Vec<FlightLeg>, Vec<Pairing>) {
    let legs: Vec<FlightLeg> = raw_flights
        .iter()
        .map(|f| {
            FlightLeg::new(
                FlightLegId::new(&f.flight_id),
                FlightNumber::new(&f.flight_number),
                AirportCode::new(&f.origin),
                AirportCode::new(&f.destination),
                f.departure_utc,
                f.arrival_utc,
                AircraftType::new(&f.aircraft_type),
            )
        })
        .collect();

    let mut duties: Vec<Duty> = Vec::new();
    let mut current_duty_legs: Vec<FlightLeg> = Vec::new();
    let mut duty_counter = 0usize;
    let mut duty_rejections = 0usize;

    let mut flush_duty = |counter: &mut usize,
                          rejections: &mut usize,
                          duties: &mut Vec<Duty>,
                          legs: Vec<FlightLeg>| {
        if legs.is_empty() {
            return;
        }
        *counter += 1;
        match Duty::new(DutyId::new(format!("D{counter}")), legs) {
            Ok(d) => duties.push(d),
            Err(_) => {
                *rejections += 1;
            }
        }
    };

    for leg in &legs {
        if current_duty_legs.is_empty() {
            current_duty_legs.push(leg.clone());
        } else {
            let last = current_duty_legs.last().unwrap();
            let ground_h =
                (leg.scheduled_departure - last.scheduled_arrival).num_seconds() as f64 / 3600.0;
            let airports_connect = last.destination == leg.origin;
            let temporal_overlap = leg.scheduled_departure < last.scheduled_arrival;
            if ground_h >= layover_rest_hours || !airports_connect || temporal_overlap {
                let batch = std::mem::take(&mut current_duty_legs);
                flush_duty(&mut duty_counter, &mut duty_rejections, &mut duties, batch);
                current_duty_legs = vec![leg.clone()];
            } else {
                current_duty_legs.push(leg.clone());
            }
        }
    }
    if !current_duty_legs.is_empty() {
        let batch = std::mem::take(&mut current_duty_legs);
        flush_duty(&mut duty_counter, &mut duty_rejections, &mut duties, batch);
    }

    let mut pairings: Vec<Pairing> = Vec::new();
    let mut pairing_counter = 0usize;
    for duty in duties {
        let pairing_base = duty.report_station().clone();
        pairing_counter += 1;
        if let Ok(p) = Pairing::new(
            PairingId::new(format!("P{pairing_counter}")),
            pairing_base,
            vec![duty],
        ) {
            pairings.push(p);
        }
    }

    (legs, pairings)
}

// ── Genome → Roster conversion ────────────────────────────────────────────────

/// Decode a genome into a Roster.
///
/// `genome[i]` = rotation index for pairing i.
/// All rotations must have at least one pairing (enforced by repair).
fn decode_genome(
    genome: &[usize],
    pairings: &[Pairing],
    crew: &[RawCrew],
    legs: &[FlightLeg],
    period_start: DateTime<Utc>,
    period_end: DateTime<Utc>,
) -> Option<Roster> {
    let n_rot = crew.len().min(pairings.len());
    if n_rot == 0 {
        return None;
    }

    // Group pairings by rotation index
    let mut buckets: Vec<Vec<Pairing>> = vec![Vec::new(); n_rot];
    for (i, &rot_idx) in genome.iter().enumerate() {
        buckets[rot_idx % n_rot].push(pairings[i].clone());
    }

    // Build rotations — each bucket must be non-empty (enforced by repair)
    let mut rotations = Vec::with_capacity(n_rot);
    for (i, bucket) in buckets.into_iter().enumerate() {
        if bucket.is_empty() {
            return None;
        } // repair should prevent this
        let rot = Rotation::new(
            RotationId::new(format!("ROT{}", i + 1)),
            CrewId::new(&crew[i].crew_id),
            bucket,
        )
        .ok()?;
        rotations.push(rot);
    }

    let period = PlanningPeriod::new(period_start, period_end);
    Roster::new(RosterId::new("CORALYS"), period, legs.to_vec(), rotations).ok()
}

/// Evaluate fitness of a genome (lower is better).
fn fitness(
    genome: &[usize],
    pairings: &[Pairing],
    crew: &[RawCrew],
    legs: &[FlightLeg],
    period_start: DateTime<Utc>,
    period_end: DateTime<Utc>,
    obj: &WorkloadBalanceObjective,
) -> f64 {
    match decode_genome(genome, pairings, crew, legs, period_start, period_end) {
        Some(roster) => obj.evaluate(&roster),
        None => f64::INFINITY,
    }
}

/// Repair: ensure every rotation has at least one pairing.
/// If a rotation is empty, steal one pairing from the most-loaded rotation.
fn repair(genome: &mut Vec<usize>, n_rot: usize, rng: &mut impl Rng) {
    if n_rot == 0 {
        return;
    }
    loop {
        // Count pairings per rotation
        let mut counts = vec![0usize; n_rot];
        for &g in genome.iter() {
            counts[g % n_rot] += 1;
        }
        // Find first empty rotation
        let empty = counts.iter().position(|&c| c == 0);
        let empty_idx = match empty {
            Some(e) => e,
            None => break, // all rotations have at least one pairing
        };
        // Find most-loaded rotation
        let max_rot = counts
            .iter()
            .enumerate()
            .max_by_key(|&(_, &c)| c)
            .map(|(i, _)| i)
            .unwrap();
        // Collect indices of pairings in max_rot
        let candidates: Vec<usize> = genome
            .iter()
            .enumerate()
            .filter(|&(_, &g)| g % n_rot == max_rot)
            .map(|(i, _)| i)
            .collect();
        // Reassign a random one to empty_idx
        if let Some(&victim) = candidates.choose(rng) {
            genome[victim] = empty_idx;
        } else {
            break;
        }
    }
}

// ── Evolutionary algorithm ────────────────────────────────────────────────────

/// Run the evolutionary scheduler. Returns (best_score, generations_to_best, elapsed_ms).
fn run_evolutionary(
    pairings: &[Pairing],
    crew: &[RawCrew],
    legs: &[FlightLeg],
    period_start: DateTime<Utc>,
    period_end: DateTime<Utc>,
    init_strategy: &str, // "round_robin" | "random"
    rng: &mut impl Rng,
) -> (f64, usize, u128) {
    let n_pairings = pairings.len();
    let n_rot = crew.len().min(n_pairings);
    let obj = WorkloadBalanceObjective;

    if n_pairings == 0 || n_rot == 0 {
        return (0.0, 0, 0);
    }

    let t_start = std::time::Instant::now();

    // ── Initialise population ─────────────────────────────────────────────────
    let mut population: Vec<Vec<usize>> = (0..POP_SIZE)
        .map(|_| {
            let mut genome: Vec<usize> = match init_strategy {
                "round_robin" => (0..n_pairings).map(|i| i % n_rot).collect(),
                _ => (0..n_pairings).map(|_| rng.gen_range(0..n_rot)).collect(),
            };
            repair(&mut genome, n_rot, rng);
            genome
        })
        .collect();

    let mut fitnesses: Vec<f64> = population
        .iter()
        .map(|g| fitness(g, pairings, crew, legs, period_start, period_end, &obj))
        .collect();

    let mut best_score = fitnesses.iter().cloned().fold(f64::INFINITY, f64::min);
    let mut best_gen = 0usize;

    // ── Evolution loop ────────────────────────────────────────────────────────
    for generation in 0..GENERATIONS {
        let mut next_pop: Vec<Vec<usize>> = Vec::with_capacity(POP_SIZE);

        while next_pop.len() < POP_SIZE {
            // Tournament selection — parent A
            let pa_idx = (0..TOURNAMENT_K)
                .map(|_| rng.gen_range(0..POP_SIZE))
                .min_by(|&a, &b| fitnesses[a].partial_cmp(&fitnesses[b]).unwrap())
                .unwrap();
            // Tournament selection — parent B
            let pb_idx = (0..TOURNAMENT_K)
                .map(|_| rng.gen_range(0..POP_SIZE))
                .min_by(|&a, &b| fitnesses[a].partial_cmp(&fitnesses[b]).unwrap())
                .unwrap();

            let mut child = if rng.r#gen::<f64>() < CROSSOVER_PROB {
                // One-point crossover
                let point = rng.gen_range(1..n_pairings);
                let mut c = population[pa_idx][..point].to_vec();
                c.extend_from_slice(&population[pb_idx][point..]);
                c
            } else {
                population[pa_idx].clone()
            };

            // Mutation: each gene flipped with probability 1/n_pairings
            let mut mutated = false;
            for gene in child.iter_mut() {
                if rng.r#gen::<f64>() < 1.0 / n_pairings as f64 {
                    *gene = rng.gen_range(0..n_rot);
                    mutated = true;
                }
            }
            if mutated {
                repair(&mut child, n_rot, rng);
            }

            next_pop.push(child);
        }

        // Evaluate new population
        let next_fitnesses: Vec<f64> = next_pop
            .iter()
            .map(|g| fitness(g, pairings, crew, legs, period_start, period_end, &obj))
            .collect();

        // Elitism: keep best from previous generation
        let prev_best_idx = fitnesses
            .iter()
            .enumerate()
            .min_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(i, _)| i)
            .unwrap();
        let new_worst_idx = next_fitnesses
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(i, _)| i)
            .unwrap();

        population = next_pop;
        fitnesses = next_fitnesses;
        population[new_worst_idx] = population[prev_best_idx].clone();
        fitnesses[new_worst_idx] = fitnesses[prev_best_idx];

        let gen_best = fitnesses.iter().cloned().fold(f64::INFINITY, f64::min);
        if gen_best < best_score {
            best_score = gen_best;
            best_gen = generation + 1;
        }
    }

    let elapsed_ms = t_start.elapsed().as_millis();
    (best_score, best_gen, elapsed_ms)
}

// ── Greedy baseline (re-run for direct comparison) ───────────────────────────

fn run_greedy_baseline(
    pairings: &[Pairing],
    crew: &[RawCrew],
    legs: &[FlightLeg],
    period_start: DateTime<Utc>,
    period_end: DateTime<Utc>,
) -> (f64, u128) {
    let n_rot = crew.len().min(pairings.len());
    if n_rot == 0 {
        return (0.0, 0);
    }

    let period = PlanningPeriod::new(period_start, period_end);
    let mut remaining = pairings.to_vec();
    let seeds: Vec<Pairing> = remaining.drain(..n_rot).collect();
    let rotations: Vec<Rotation> = crew
        .iter()
        .take(n_rot)
        .enumerate()
        .map(|(i, c)| {
            Rotation::new(
                RotationId::new(format!("ROT{}", i + 1)),
                CrewId::new(&c.crew_id),
                vec![seeds[i].clone()],
            )
            .expect("Seeded rotation must be valid")
        })
        .collect();
    let baseline = Roster::new(RosterId::new("GERAD"), period, legs.to_vec(), rotations)
        .expect("Roster construction failed");

    let mut evaluator = CostEvaluator::new();
    evaluator.add_objective(Box::new(WorkloadBalanceObjective));
    let greedy = GreedyScheduler::new(&evaluator, vec![1.0]);
    let mut metrics = OptimizationMetrics::new();

    let t_start = std::time::Instant::now();
    let after_greedy = greedy.assign(&baseline, remaining, &mut metrics);
    let elapsed_ms = t_start.elapsed().as_millis();

    let obj = WorkloadBalanceObjective;
    let score = obj.evaluate(&after_greedy);
    (score, elapsed_ms)
}

// ── Test ──────────────────────────────────────────────────────────────────────

/// Coralys-native evolutionary scheduler vs Greedy baseline.
///
/// Runs both schedulers on all 7 GERAD instances and reports:
/// - Greedy baseline score and runtime
/// - Evolutionary (round-robin init) score and runtime
/// - Evolutionary (random init) score and runtime
///
/// See `UltraCrew_Pairing_Topology_Mutation_Evaluation.md` Section 2.16–2.17.
#[test]
fn gerad_coralys_vs_greedy() {
    let workspace_root = {
        let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
        std::path::PathBuf::from(&manifest)
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf()
    };

    let benchmark_base = workspace_root.join("benchmarks").join("gerad-g2014-22");

    println!();
    println!("GERAD G2014-22 — Coralys-Native Evolutionary Scheduler vs Greedy Baseline");
    println!("============================================================================");
    println!("Genome: pairing_id → rotation_id (Vec<usize>, length P)");
    println!("Fitness: WorkloadBalanceObjective (lower is better)");
    println!(
        "EA params: pop={POP_SIZE} gen={GENERATIONS} xover={CROSSOVER_PROB} tournament_k={TOURNAMENT_K} seed={SEED}"
    );
    println!("Layover threshold: {LAYOVER_REST_HOURS}h (Condition A)");
    println!();

    let instances = [
        ("Instance1", "instance1"),
        ("Instance2", "instance2"),
        ("Instance3", "instance3"),
        ("Instance4", "instance4"),
        ("Instance5", "instance5"),
        ("Instance6", "instance6"),
        ("Instance7", "instance7"),
    ];

    println!(
        "{:<12} {:>8} {:>12} {:>12} {:>12} {:>12} {:>12} {:>12}",
        "Instance", "Pairings", "Greedy", "Greedy_ms", "EA_rr", "EA_rr_ms", "EA_rand", "EA_rand_ms"
    );
    println!("{}", "-".repeat(100));

    for (label, dir) in &instances {
        let instance_dir = benchmark_base.join(dir);
        let flights_path = instance_dir.join("flights.csv");
        let crew_path = instance_dir.join("crew.csv");

        if !flights_path.exists() || !crew_path.exists() {
            println!("{:<12} SKIP (files not found)", label);
            continue;
        }

        let raw_flights = load_flights(&flights_path);
        let crew = load_crew(&crew_path);

        let (legs, pairings) = build_pairings_from_flights(&raw_flights, LAYOVER_REST_HOURS);

        let period_start = raw_flights
            .first()
            .map(|f| f.departure_utc)
            .unwrap_or(Utc::now());
        let period_end = raw_flights
            .last()
            .map(|f| f.arrival_utc)
            .unwrap_or(Utc::now());

        let n_pairings = pairings.len();
        let n_rot = crew.len().min(n_pairings);

        if n_pairings == 0 || n_rot == 0 {
            println!(
                "{:<12} {:>8} (no pairings or rotations — skipping EA)",
                label, 0
            );
            continue;
        }

        // Greedy baseline
        let (greedy_score, greedy_ms) =
            run_greedy_baseline(&pairings, &crew, &legs, period_start, period_end);

        // Evolutionary — round-robin initialisation
        let mut rng_rr = StdRng::seed_from_u64(SEED);
        let (ea_rr_score, ea_rr_gen, ea_rr_ms) = run_evolutionary(
            &pairings,
            &crew,
            &legs,
            period_start,
            period_end,
            "round_robin",
            &mut rng_rr,
        );

        // Evolutionary — random initialisation
        let mut rng_rand = StdRng::seed_from_u64(SEED + 1);
        let (ea_rand_score, ea_rand_gen, ea_rand_ms) = run_evolutionary(
            &pairings,
            &crew,
            &legs,
            period_start,
            period_end,
            "random",
            &mut rng_rand,
        );

        println!(
            "{:<12} {:>8} {:>12.4} {:>12} {:>12.4} {:>12} {:>12.4} {:>12}",
            label,
            n_pairings,
            greedy_score,
            greedy_ms,
            ea_rr_score,
            ea_rr_ms,
            ea_rand_score,
            ea_rand_ms
        );

        eprintln!(
            "  [coralys] instance={label} pairings={n_pairings} rotations={n_rot} \
             greedy_score={greedy_score:.4} greedy_ms={greedy_ms} \
             ea_rr_score={ea_rr_score:.4} ea_rr_gen={ea_rr_gen} ea_rr_ms={ea_rr_ms} \
             ea_rand_score={ea_rand_score:.4} ea_rand_gen={ea_rand_gen} ea_rand_ms={ea_rand_ms}"
        );
    }

    println!();
    println!("Legend:");
    println!("  Greedy     = GreedyScheduler score (WorkloadBalanceObjective)");
    println!("  EA_rr      = Evolutionary, round-robin initialisation");
    println!("  EA_rand    = Evolutionary, random initialisation");
    println!("  *_ms       = wall-clock milliseconds");
    println!("  gen        = generation at which best score was first achieved");
}
