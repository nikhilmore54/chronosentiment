//! GERAD G2014-22 — Coralys Evolutionary Scheduler v2.0
//!
//! # Evolutionary Scheduling on Ground-Truth GERAD Benchmark Pairings
//!
//! # Objective
//!
//! Compare the Coralys-native evolutionary scheduler against the Greedy baseline
//! on the full benchmark pairing datasets from the GERAD G-2014-22 repository
//! (`pairings.csv`, `duties.csv`, `flights.csv`, `crew.csv`).
//!
//! Unlike v1.0 (`gerad_coralys.rs`), which relied on a naive single-duty generator
//! that caused search space collapse (1:1 pairings-to-crew on Instances 6 & 7,
//! and severe pairing starvation on Instances 1 & 2), v2.0 loads the benchmark's
//! multi-duty pairings and evaluates combinatorial assignment optimization across
//! the full, unconstrained solution space.

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
use std::collections::{HashMap, HashSet};
use std::path::Path;

// ── Evolutionary Algorithm Constants ─────────────────────────────────────────

const POP_SIZE: usize = 50;
const GENERATIONS: usize = 200;
const CROSSOVER_PROB: f64 = 0.80;
const TOURNAMENT_K: usize = 3;
const SEED: u64 = 42;

// ── CSV Data Structures ───────────────────────────────────────────────────────

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

#[derive(Debug, Clone)]
struct RawCrew {
    crew_id: String,
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

fn pad_airport(s: &str) -> String {
    if s.len() >= 3 {
        s[..3].to_uppercase()
    } else {
        format!("{:A<3}", s.to_uppercase())
    }
}

fn load_flights(path: &Path) -> HashMap<String, FlightLeg> {
    let mut rdr = csv::Reader::from_path(path).expect("Cannot open flights.csv");
    let mut map = HashMap::new();
    for result in rdr.deserialize::<HashMap<String, String>>() {
        let row = result.expect("CSV parse error in flights.csv");
        let fid = row["flight_id"].trim().to_string();
        let fnum = row
            .get("flight_number")
            .cloned()
            .unwrap_or_else(|| fid.clone());
        let origin = pad_airport(row["origin"].trim());
        let dest = pad_airport(row["destination"].trim());
        let dep = parse_utc(row["departure_utc"].trim());
        let arr = parse_utc(row["arrival_utc"].trim());
        let atype = row
            .get("aircraft_type")
            .cloned()
            .unwrap_or_else(|| "A320".to_string());

        let leg = FlightLeg::new(
            FlightLegId::new(&fid),
            FlightNumber::new(&fnum),
            AirportCode::new(&origin),
            AirportCode::new(&dest),
            dep,
            arr,
            AircraftType::new(&atype),
        );
        map.insert(fid, leg);
    }
    map
}

fn load_crew(path: &Path) -> Vec<RawCrew> {
    let mut rdr = csv::Reader::from_path(path).expect("Cannot open crew.csv");
    let mut crew = Vec::new();
    for result in rdr.deserialize::<HashMap<String, String>>() {
        let row = result.expect("CSV parse error in crew.csv");
        crew.push(RawCrew {
            crew_id: row["crew_id"].trim().to_string(),
        });
    }
    crew
}

fn load_benchmark_pairings(
    instance_dir: &Path,
    flights_map: &HashMap<String, FlightLeg>,
) -> (Vec<FlightLeg>, Vec<Pairing>, usize) {
    let duties_path = instance_dir.join("duties.csv");
    let pairings_path = instance_dir.join("pairings.csv");

    // 1. Load duties
    let mut rdr_d = csv::Reader::from_path(&duties_path).expect("Cannot open duties.csv");
    let mut duties_map: HashMap<String, Duty> = HashMap::new();
    for result in rdr_d.deserialize::<HashMap<String, String>>() {
        let row = result.expect("CSV parse error in duties.csv");
        let did = row["duty_id"].trim().to_string();
        let fids_raw = row["flight_ids"].trim();
        let fids: Vec<&str> = fids_raw.split(',').map(|s| s.trim()).collect();
        let mut duty_legs = Vec::new();
        for fid in fids {
            if let Some(leg) = flights_map.get(fid) {
                duty_legs.push(leg.clone());
            }
        }
        if !duty_legs.is_empty() {
            if let Ok(duty) = Duty::new(DutyId::new(&did), duty_legs) {
                duties_map.insert(did, duty);
            }
        }
    }

    // 2. Load pairings
    let mut rdr_p = csv::Reader::from_path(&pairings_path).expect("Cannot open pairings.csv");
    let mut pairings = Vec::new();
    let mut total_csv_pairings = 0usize;
    for result in rdr_p.deserialize::<HashMap<String, String>>() {
        total_csv_pairings += 1;
        let row = result.expect("CSV parse error in pairings.csv");
        let pid = row["pairing_id"].trim().to_string();
        let base = pad_airport(row["home_base"].trim());
        let dids_raw = row["duty_ids"].trim();
        let dids: Vec<&str> = dids_raw.split(',').map(|s| s.trim()).collect();

        let mut pairing_duties = Vec::new();
        for did in dids {
            if let Some(duty) = duties_map.get(did) {
                pairing_duties.push(duty.clone());
            }
        }

        if !pairing_duties.is_empty() {
            if let Ok(pairing) = Pairing::new(
                PairingId::new(&pid),
                AirportCode::new(&base),
                pairing_duties,
            ) {
                pairings.push(pairing);
            }
        }
    }

    let all_legs: Vec<FlightLeg> = flights_map.values().cloned().collect();
    (all_legs, pairings, total_csv_pairings)
}

// ── Genome Decode & Fitness ───────────────────────────────────────────────────

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

    let mut buckets: Vec<Vec<Pairing>> = vec![Vec::new(); n_rot];
    for (i, &rot_idx) in genome.iter().enumerate() {
        if rot_idx < n_rot {
            buckets[rot_idx].push(pairings[i].clone());
        }
    }

    let mut rotations = Vec::with_capacity(n_rot);
    for (i, mut bucket) in buckets.into_iter().enumerate() {
        if bucket.is_empty() {
            return None;
        }
        // Crucial: chronological sorting ensures Rotation::new order invariant
        bucket.sort_by_key(|p| p.start());
        let rot = Rotation::new(
            RotationId::new(format!("ROT{}", i + 1)),
            CrewId::new(&crew[i].crew_id),
            bucket,
        )
        .ok()?;
        rotations.push(rot);
    }

    let period = PlanningPeriod::new(period_start, period_end);
    Roster::new(RosterId::new("CORALYS_V2"), period, legs.to_vec(), rotations).ok()
}

fn fitness(
    genome: &[usize],
    pairings: &[Pairing],
    crew: &[RawCrew],
    legs: &[FlightLeg],
    period_start: DateTime<Utc>,
    period_end: DateTime<Utc>,
    obj: &WorkloadBalanceObjective,
) -> f64 {
    let n_rot = crew.len().min(pairings.len());
    if n_rot == 0 {
        return f64::INFINITY;
    }

    let mut buckets: Vec<Vec<&Pairing>> = vec![Vec::new(); n_rot];
    let mut unassigned = 0usize;
    for (i, &rot_idx) in genome.iter().enumerate() {
        if rot_idx < n_rot {
            buckets[rot_idx].push(&pairings[i]);
        } else {
            unassigned += 1;
        }
    }

    let mut overlaps = 0usize;
    let mut empty_rotations = 0usize;

    for bucket in buckets.iter_mut() {
        if bucket.is_empty() {
            empty_rotations += 1;
        } else {
            bucket.sort_by_key(|p| p.start());
            for w in bucket.windows(2) {
                if w[1].start() < w[0].end() {
                    overlaps += 1;
                }
            }
        }
    }

    if overlaps > 0 || empty_rotations > 0 {
        return 100_000.0 + (overlaps as f64) * 1_000.0 + (empty_rotations as f64) * 5_000.0;
    }

    match decode_genome(genome, pairings, crew, legs, period_start, period_end) {
        Some(roster) => {
            let base_cost = obj.evaluate(&roster);
            base_cost + (unassigned as f64) * 0.001
        }
        None => f64::INFINITY,
    }
}

fn repair(genome: &mut Vec<usize>, n_rot: usize, rng: &mut impl Rng) {
    if n_rot == 0 {
        return;
    }
    loop {
        let mut counts = vec![0usize; n_rot];
        for &g in genome.iter() {
            if g < n_rot {
                counts[g] += 1;
            }
        }
        let empty_idx = match counts.iter().position(|&c| c == 0) {
            Some(e) => e,
            None => break,
        };

        // Steal first from unassigned pool if available
        let unassigned: Vec<usize> = genome
            .iter()
            .enumerate()
            .filter(|&(_, &g)| g == n_rot)
            .map(|(i, _)| i)
            .collect();
        if let Some(&victim) = unassigned.choose(rng) {
            genome[victim] = empty_idx;
            continue;
        }

        let max_rot = counts
            .iter()
            .enumerate()
            .max_by_key(|&(_, &c)| c)
            .map(|(i, _)| i)
            .unwrap();
        if counts[max_rot] <= 1 {
            break;
        }
        let candidates: Vec<usize> = genome
            .iter()
            .enumerate()
            .filter(|&(_, &g)| g == max_rot)
            .map(|(i, _)| i)
            .collect();
        if let Some(&victim) = candidates.choose(rng) {
            genome[victim] = empty_idx;
        } else {
            break;
        }
    }
}

// ── Greedy Baseline & Seeding ─────────────────────────────────────────────────

fn run_greedy_baseline(
    pairings: &[Pairing],
    crew: &[RawCrew],
    legs: &[FlightLeg],
    period_start: DateTime<Utc>,
    period_end: DateTime<Utc>,
) -> (f64, Vec<usize>, u128) {
    let n_rot = crew.len().min(pairings.len());
    if n_rot == 0 {
        return (0.0, vec![], 0);
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

    // Extract genome from after_greedy: pairings not in roster receive unassigned index n_rot
    let mut pairing_to_rot: HashMap<&PairingId, usize> = HashMap::new();
    for (rot_idx, rot) in after_greedy.rotations().enumerate() {
        for p in rot.pairings() {
            pairing_to_rot.insert(&p.id, rot_idx);
        }
    }
    let genome: Vec<usize> = pairings
        .iter()
        .map(|p| pairing_to_rot.get(&p.id).copied().unwrap_or(n_rot))
        .collect();

    (score, genome, elapsed_ms)
}

// ── Evolutionary Algorithm ────────────────────────────────────────────────────

fn run_evolutionary(
    pairings: &[Pairing],
    crew: &[RawCrew],
    legs: &[FlightLeg],
    period_start: DateTime<Utc>,
    period_end: DateTime<Utc>,
    greedy_seed: &[usize],
    rng: &mut impl Rng,
) -> (f64, Vec<usize>, usize, u128) {
    let n_pairings = pairings.len();
    let n_rot = crew.len().min(n_pairings);
    let obj = WorkloadBalanceObjective;

    if n_pairings == 0 || n_rot == 0 {
        return (0.0, vec![], 0, 0);
    }

    let t_start = std::time::Instant::now();

    let mut population: Vec<Vec<usize>> = Vec::with_capacity(POP_SIZE);

    if !greedy_seed.is_empty() {
        population.push(greedy_seed.to_vec());
        for _ in 0..4 {
            let mut copy = greedy_seed.to_vec();
            let p1 = rng.gen_range(0..n_pairings);
            let p2 = rng.gen_range(0..n_pairings);
            copy.swap(p1, p2);
            repair(&mut copy, n_rot, rng);
            population.push(copy);
        }
    }

    while population.len() < POP_SIZE {
        let mut genome: Vec<usize> = (0..n_pairings)
            .map(|i| if i < n_rot { i } else { rng.gen_range(0..=n_rot) })
            .collect();
        repair(&mut genome, n_rot, rng);
        population.push(genome);
    }

    let mut fitnesses: Vec<f64> = population
        .iter()
        .map(|g| fitness(g, pairings, crew, legs, period_start, period_end, &obj))
        .collect();

    let mut best_score = fitnesses.iter().cloned().fold(f64::INFINITY, f64::min);
    let mut best_gen = 0usize;
    let b_idx = fitnesses
        .iter()
        .position(|&s| (s - best_score).abs() < 1e-9)
        .unwrap_or(0);
    let mut best_genome = population[b_idx].clone();

    for generation in 0..GENERATIONS {
        let mut next_pop: Vec<Vec<usize>> = Vec::with_capacity(POP_SIZE);

        while next_pop.len() < POP_SIZE {
            let pa_idx = (0..TOURNAMENT_K)
                .map(|_| rng.gen_range(0..POP_SIZE))
                .min_by(|&a, &b| fitnesses[a].partial_cmp(&fitnesses[b]).unwrap())
                .unwrap();
            let pb_idx = (0..TOURNAMENT_K)
                .map(|_| rng.gen_range(0..POP_SIZE))
                .min_by(|&a, &b| fitnesses[a].partial_cmp(&fitnesses[b]).unwrap())
                .unwrap();

            let mut child = if rng.r#gen::<f64>() < CROSSOVER_PROB {
                let point = rng.gen_range(1..n_pairings);
                let mut c = population[pa_idx][..point].to_vec();
                c.extend_from_slice(&population[pb_idx][point..]);
                c
            } else {
                population[pa_idx].clone()
            };

            let mut mutated = false;
            if rng.r#gen::<f64>() < 0.25 {
                let p1 = rng.gen_range(0..n_pairings);
                let p2 = rng.gen_range(0..n_pairings);
                child.swap(p1, p2);
                mutated = true;
            } else {
                for gene in child.iter_mut() {
                    if rng.r#gen::<f64>() < 1.0 / n_pairings as f64 {
                        *gene = rng.gen_range(0..=n_rot);
                        mutated = true;
                    }
                }
            }
            if mutated {
                repair(&mut child, n_rot, rng);
            }

            next_pop.push(child);
        }

        let next_fitnesses: Vec<f64> = next_pop
            .iter()
            .map(|g| fitness(g, pairings, crew, legs, period_start, period_end, &obj))
            .collect();

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
            let b_idx = fitnesses
                .iter()
                .position(|&s| (s - gen_best).abs() < 1e-9)
                .unwrap_or(0);
            best_genome = population[b_idx].clone();
        }
    }

    let elapsed_ms = t_start.elapsed().as_millis();
    (best_score, best_genome, best_gen, elapsed_ms)
}

// ── Operational KPIs & GERAD Alignment ────────────────────────────────────────

#[derive(Debug, Clone)]
struct OperationalKpis {
    total_flights: usize,
    covered_flights: usize,
    uncovered_flights: usize,
    uncovered_flights_pct: f64,
    total_pairings_csv: usize,
    loaded_pairings: usize,
    prefiltered_pairings: usize,
    assigned_pairings: usize,
    unassigned_loaded: usize,
    total_uncovered_pairings: usize,
    uncovered_pairings_pct: f64,
    mean_credited_hours: f64,
    min_credited_hours: f64,
    max_credited_hours: f64,
    std_credited_hours: f64,
}

fn compute_operational_kpis(
    genome: &[usize],
    pairings: &[Pairing],
    crew: &[RawCrew],
    flights_map: &HashMap<String, FlightLeg>,
    total_csv_pairings: usize,
) -> OperationalKpis {
    let n_rot = crew.len().min(pairings.len());
    let total_flights = flights_map.len();
    let loaded_pairings = pairings.len();
    let prefiltered_pairings = total_csv_pairings.saturating_sub(loaded_pairings);

    let mut covered_flights_set = HashSet::new();
    let mut assigned_pairings = 0usize;
    let mut unassigned_loaded = 0usize;
    let mut crew_flight_minutes: Vec<i64> = vec![0; n_rot];

    for (i, &rot_idx) in genome.iter().enumerate() {
        if rot_idx < n_rot {
            assigned_pairings += 1;
            let pairing = &pairings[i];
            for duty in pairing.duties() {
                crew_flight_minutes[rot_idx] += duty.total_block_time().num_minutes();
                for leg in duty.legs() {
                    covered_flights_set.insert(leg.id.as_str().to_string());
                }
            }
        } else {
            unassigned_loaded += 1;
        }
    }

    let covered_flights = covered_flights_set.len();
    let uncovered_flights = total_flights.saturating_sub(covered_flights);
    let uncovered_flights_pct = if total_flights > 0 {
        uncovered_flights as f64 / total_flights as f64 * 100.0
    } else {
        0.0
    };

    let total_uncovered_pairings = unassigned_loaded + prefiltered_pairings;
    let uncovered_pairings_pct = if total_csv_pairings > 0 {
        total_uncovered_pairings as f64 / total_csv_pairings as f64 * 100.0
    } else {
        0.0
    };

    let crew_hours: Vec<f64> = crew_flight_minutes
        .iter()
        .map(|&m| m as f64 / 60.0)
        .collect();

    let mean_credited_hours = if !crew_hours.is_empty() {
        crew_hours.iter().sum::<f64>() / crew_hours.len() as f64
    } else {
        0.0
    };

    let min_credited_hours = crew_hours
        .iter()
        .cloned()
        .fold(f64::INFINITY, f64::min);

    let max_credited_hours = crew_hours
        .iter()
        .cloned()
        .fold(0.0_f64, f64::max);

    let variance = if !crew_hours.is_empty() {
        crew_hours
            .iter()
            .map(|&h| (h - mean_credited_hours).powi(2))
            .sum::<f64>()
            / crew_hours.len() as f64
    } else {
        0.0
    };
    let std_credited_hours = variance.sqrt();

    OperationalKpis {
        total_flights,
        covered_flights,
        uncovered_flights,
        uncovered_flights_pct,
        total_pairings_csv: total_csv_pairings,
        loaded_pairings,
        prefiltered_pairings,
        assigned_pairings,
        unassigned_loaded,
        total_uncovered_pairings,
        uncovered_pairings_pct,
        mean_credited_hours,
        min_credited_hours,
        max_credited_hours,
        std_credited_hours,
    }
}

struct GeradRef {
    avg_hours: f64,
    min_hours: f64,
    max_hours: f64,
    uncovered_flights_pct: f64,
}

fn get_gerad_ref(label: &str) -> GeradRef {
    match label {
        "Instance1" => GeradRef { avg_hours: 68.75, min_hours: 23.67, max_hours: 84.90, uncovered_flights_pct: 0.00 },
        "Instance2" => GeradRef { avg_hours: 75.76, min_hours: 50.85, max_hours: 84.63, uncovered_flights_pct: 0.00 },
        "Instance3" => GeradRef { avg_hours: 72.70, min_hours: 20.33, max_hours: 85.00, uncovered_flights_pct: 0.00 },
        "Instance4" => GeradRef { avg_hours: 74.67, min_hours: 23.68, max_hours: 84.95, uncovered_flights_pct: 0.25 },
        "Instance5" => GeradRef { avg_hours: 81.12, min_hours: 10.72, max_hours: 85.00, uncovered_flights_pct: 0.64 },
        "Instance6" => GeradRef { avg_hours: 76.75, min_hours: 21.35, max_hours: 84.98, uncovered_flights_pct: 0.52 },
        "Instance7" => GeradRef { avg_hours: 81.65, min_hours:  8.88, max_hours: 85.00, uncovered_flights_pct: 1.45 },
        _ => GeradRef { avg_hours: 75.92, min_hours: 20.0, max_hours: 85.00, uncovered_flights_pct: 0.41 },
    }
}

// ── Test Execution ────────────────────────────────────────────────────────────

#[test]
fn gerad_coralys_v2_benchmark_run() {
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
    println!("GERAD G2014-22 — Coralys Evolutionary Scheduler v2.0 (Full Benchmark Pairings)");
    println!("===============================================================================");
    println!("Dataset: Real multi-duty benchmark pairings (172–1648 pairings)");
    println!("Fitness: WorkloadBalanceObjective (lower is better)");
    println!("EA params: pop={POP_SIZE} gen={GENERATIONS} xover={CROSSOVER_PROB} seed={SEED}");
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
        "{:<12} {:>8} {:>6} {:>12} {:>12} {:>12} {:>12} {:>12}",
        "Instance", "Pairings", "Crew", "Greedy", "Greedy_ms", "Coralys_v2", "Coralys_ms", "Advantage"
    );
    println!("{}", "-".repeat(95));

    for (label, dir) in &instances {
        let instance_dir = benchmark_base.join(dir);
        let flights_path = instance_dir.join("flights.csv");
        let crew_path = instance_dir.join("crew.csv");

        if !flights_path.exists() || !crew_path.exists() {
            println!("{:<12} SKIP (files not found)", label);
            continue;
        }

        let flights_map = load_flights(&flights_path);
        let crew = load_crew(&crew_path);
        let (legs, pairings, total_csv_pairings) = load_benchmark_pairings(&instance_dir, &flights_map);

        let period_start = legs.iter().map(|f| f.scheduled_departure).min().unwrap_or(Utc::now());
        let period_end = legs.iter().map(|f| f.scheduled_arrival).max().unwrap_or(Utc::now());

        let n_pairings = pairings.len();
        let n_crew = crew.len();

        if n_pairings == 0 || n_crew == 0 {
            println!("{:<12} {:>8} (no pairings loaded)", label, 0);
            continue;
        }

        // Greedy baseline
        let (greedy_score, greedy_seed, greedy_ms) =
            run_greedy_baseline(&pairings, &crew, &legs, period_start, period_end);

        // Evolutionary — Coralys v2.0
        let mut rng = StdRng::seed_from_u64(SEED);
        let (ea_score, ea_genome, ea_gen, ea_ms) = run_evolutionary(
            &pairings,
            &crew,
            &legs,
            period_start,
            period_end,
            &greedy_seed,
            &mut rng,
        );

        let adv_str = if ea_score < greedy_score {
            let pct = (greedy_score - ea_score) / greedy_score * 100.0;
            format!("+{pct:.2}% (BEATS)")
        } else if (ea_score - greedy_score).abs() < 1e-4 {
            "TIE".to_string()
        } else {
            let pct = (ea_score - greedy_score) / greedy_score * 100.0;
            format!("-{pct:.2}%")
        };

        println!(
            "{:<12} {:>8} {:>6} {:>12.4} {:>12} {:>12.4} {:>12} {:>12}",
            label, n_pairings, n_crew, greedy_score, greedy_ms, ea_score, ea_ms, adv_str
        );

        let greedy_kpi = compute_operational_kpis(&greedy_seed, &pairings, &crew, &flights_map, total_csv_pairings);
        let ea_kpi = compute_operational_kpis(&ea_genome, &pairings, &crew, &flights_map, total_csv_pairings);
        let gerad_ref = get_gerad_ref(label);

        println!();
        println!("  ┌── [{label}] Operational KPIs (GERAD G-2014-22 Comparative Alignment) ──");
        println!("  │ • Flight Coverage (Total GERAD Flights: {}):", ea_kpi.total_flights);
        println!("  │   - Coralys v2.0:  {:>5}/{} uncovered ({:.2}%)", ea_kpi.uncovered_flights, ea_kpi.total_flights, ea_kpi.uncovered_flights_pct);
        println!("  │   - Greedy:        {:>5}/{} uncovered ({:.2}%)", greedy_kpi.uncovered_flights, greedy_kpi.total_flights, greedy_kpi.uncovered_flights_pct);
        println!("  │   - GERAD Target:  uncovered target: {:.2}%", gerad_ref.uncovered_flights_pct);
        println!("  │ • Pairing Coverage (Total CSV Pairings: {}, Pre-filtered: {}):", ea_kpi.total_pairings_csv, ea_kpi.prefiltered_pairings);
        println!("  │   - Coralys v2.0:  {:>5}/{} uncovered ({:.2}%) [assigned: {}]", ea_kpi.total_uncovered_pairings, ea_kpi.total_pairings_csv, ea_kpi.uncovered_pairings_pct, ea_kpi.assigned_pairings);
        println!("  │   - Greedy:        {:>5}/{} uncovered ({:.2}%) [assigned: {}]", greedy_kpi.total_uncovered_pairings, greedy_kpi.total_pairings_csv, greedy_kpi.uncovered_pairings_pct, greedy_kpi.assigned_pairings);
        println!("  │ • Credited Flying Hours across {} Crew (Cap: <= 85.0 hrs):", n_crew);
        println!("  │   - Coralys v2.0:  Mean={:.2}h | Min={:.2}h | Max={:.2}h | StdDev={:.2}h", ea_kpi.mean_credited_hours, ea_kpi.min_credited_hours, ea_kpi.max_credited_hours, ea_kpi.std_credited_hours);
        println!("  │   - Greedy:        Mean={:.2}h | Min={:.2}h | Max={:.2}h | StdDev={:.2}h", greedy_kpi.mean_credited_hours, greedy_kpi.min_credited_hours, greedy_kpi.max_credited_hours, greedy_kpi.std_credited_hours);
        println!("  │   - GERAD Target:  Mean={:.2}h | Min={:.2}h | Max={:.2}h | Overall Target Mean: 75.92h", gerad_ref.avg_hours, gerad_ref.min_hours, gerad_ref.max_hours);
        println!("  │ • Preference Satisfaction Targets (Paper G-2014-22 Reference):");
        println!("  │   - Vacation Days Granted Target:  13.76%");
        println!("  │   - Preferred Flights Target:      29.09%");
        println!("  └────────────────────────────────────────────────────────────────────────");
        println!();

        eprintln!(
            "  [v2_result] instance={label} pairings={n_pairings} crew={n_crew} \
             greedy_score={greedy_score:.4} greedy_ms={greedy_ms} \
             ea_score={ea_score:.4} ea_gen={ea_gen} ea_ms={ea_ms} advantage={adv_str} \
             coralys_uncovered_pct={:.2}% greedy_uncovered_pct={:.2}% \
             coralys_mean_hrs={:.2}h greedy_mean_hrs={:.2}h",
            ea_kpi.uncovered_flights_pct, greedy_kpi.uncovered_flights_pct,
            ea_kpi.mean_credited_hours, greedy_kpi.mean_credited_hours
        );
    }
}
