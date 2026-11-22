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

use coralys_airline::domain::duty::{Duty, DutyId};
use coralys_airline::domain::flight::{
    AircraftType, AirportCode, FlightLeg, FlightLegId, FlightNumber,
};
use coralys_airline::domain::pairing::{Pairing, PairingId};

use chrono::{DateTime, Utc};
use rand::prelude::*;
use std::collections::{HashMap, HashSet};
use std::path::Path;

// ── Evolutionary Algorithm Constants ─────────────────────────────────────────

const POP_SIZE: usize = 50;
const GENERATIONS: usize = 100;
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
    base: String,
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
            base: row["base"].trim().to_string(),
        });
    }
    crew
}

fn load_benchmark_pairings(
    instance_dir: &Path,
    flights_map: &HashMap<String, FlightLeg>,
) -> (Vec<FlightLeg>, Vec<Pairing>, Vec<String>, usize) {
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
    let mut pairing_bases = Vec::new();
    let mut total_csv_pairings = 0usize;
    for result in rdr_p.deserialize::<HashMap<String, String>>() {
        total_csv_pairings += 1;
        let row = result.expect("CSV parse error in pairings.csv");
        let pid = row["pairing_id"].trim().to_string();
        let raw_base = row["home_base"].trim().to_string();
        let base = pad_airport(&raw_base);
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
                pairing_bases.push(raw_base);
            }
        }
    }

    let all_legs: Vec<FlightLeg> = flights_map.values().cloned().collect();
    (all_legs, pairings, pairing_bases, total_csv_pairings)
}

// ── Precomputed Pairing Data & Constrained Domain ────────────────────────────

#[derive(Clone)]
struct PrecomputedPairingData {
    base: String,
    minutes: i64,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    flight_ids: Vec<String>,
}

fn precompute_pairing_data(pairings: &[Pairing], bases: &[String]) -> Vec<PrecomputedPairingData> {
    pairings
        .iter()
        .zip(bases.iter())
        .map(|(p, raw_base)| {
            let minutes: i64 = p
                .duties()
                .iter()
                .map(|d| d.total_block_time().num_minutes())
                .sum();
            let flight_ids: Vec<String> = p
                .duties()
                .iter()
                .flat_map(|d| d.legs().iter().map(|l| l.id.as_str().to_string()))
                .collect();
            PrecomputedPairingData {
                base: raw_base.clone(),
                minutes,
                start: p.start(),
                end: p.end(),
                flight_ids,
            }
        })
        .collect()
}

fn build_valid_rotations(
    pairings_data: &[PrecomputedPairingData],
    crew: &[RawCrew],
    n_rot: usize,
) -> Vec<Vec<usize>> {
    let mut crew_by_base: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, c) in crew.iter().take(n_rot).enumerate() {
        crew_by_base.entry(c.base.clone()).or_default().push(i);
    }

    pairings_data
        .iter()
        .map(|p| {
            let mut valid = crew_by_base.get(&p.base).cloned().unwrap_or_default();
            valid.push(n_rot); // unassigned reserve pool is always permitted
            valid
        })
        .collect()
}

// ── Multi-Parameter Objective Function ────────────────────────────────────────

fn fitness(
    genome: &[usize],
    data: &[PrecomputedPairingData],
    crew: &[RawCrew],
    n_rot: usize,
    total_flights: usize,
) -> f64 {
    let mut rot_minutes = vec![0i64; n_rot];
    let mut rot_pairing_indices: Vec<Vec<usize>> = vec![Vec::new(); n_rot];
    let mut unassigned_count = 0usize;
    let mut base_mismatches = 0usize;

    for (i, &rot_idx) in genome.iter().enumerate() {
        if rot_idx < n_rot {
            rot_pairing_indices[rot_idx].push(i);
            rot_minutes[rot_idx] += data[i].minutes;
            if data[i].base != crew[rot_idx].base {
                base_mismatches += 1;
            }
        } else {
            unassigned_count += 1;
        }
    }

    let mut overlaps = 0usize;
    let mut empty_rotations = 0usize;
    let mut covered_flights_set = HashSet::new();

    for (_rot_idx, indices) in rot_pairing_indices.iter_mut().enumerate() {
        if indices.is_empty() {
            empty_rotations += 1;
        } else {
            indices.sort_by_key(|&idx| data[idx].start);
            for w in indices.windows(2) {
                if data[w[1]].start < data[w[0]].end {
                    overlaps += 1;
                }
            }
            for &idx in indices.iter() {
                for fid in &data[idx].flight_ids {
                    covered_flights_set.insert(fid.as_str());
                }
            }
        }
    }

    // Hard feasibility constraint 1: Max 85.0 flying hours cap (FAA / GERAD limit)
    let mut overtime_hours = 0.0_f64;
    for &mins in &rot_minutes {
        let hrs = mins as f64 / 60.0;
        if hrs > 85.0 {
            overtime_hours += hrs - 85.0;
        }
    }

    // Feasibility penalties
    if overlaps > 0 || empty_rotations > 0 || base_mismatches > 0 || overtime_hours > 0.0 {
        return 1_000_000.0
            + (overlaps as f64) * 20_000.0
            + (empty_rotations as f64) * 20_000.0
            + (base_mismatches as f64) * 50_000.0
            + overtime_hours * 50_000.0
            + (unassigned_count as f64) * 100.0;
    }

    // Feasible candidate: evaluate Coverage + Workload Balance
    let uncovered_flights = total_flights.saturating_sub(covered_flights_set.len());
    let hours: Vec<f64> = rot_minutes.iter().map(|&m| m as f64 / 60.0).collect();
    let mean_hrs = hours.iter().sum::<f64>() / hours.len() as f64;
    let variance_hrs =
        hours.iter().map(|&h| (h - mean_hrs).powi(2)).sum::<f64>() / hours.len() as f64;

    // Parameter Priorities:
    // 1. Uncovered flight penalty = 1000.0 per uncovered flight (dominant priority)
    // 2. Unassigned pairing penalty = 500.0 per unassigned pairing
    // 3. Workload variance = 1.0 (fine-grained equity tie-breaker)
    (uncovered_flights as f64) * 1000.0 + (unassigned_count as f64) * 500.0 + variance_hrs
}

// ── Smart Feasibility & Packing Repair ────────────────────────────────────────

fn repair(
    genome: &mut Vec<usize>,
    data: &[PrecomputedPairingData],
    crew: &[RawCrew],
    valid_rotations: &[Vec<usize>],
    n_rot: usize,
    rng: &mut impl Rng,
) {
    if n_rot == 0 {
        return;
    }

    // 1. Enforce base matching, non-overlap, and 85.0h cap per rotation
    let mut rot_intervals: Vec<Vec<(DateTime<Utc>, DateTime<Utc>)>> = vec![Vec::new(); n_rot];
    let mut rot_minutes: Vec<i64> = vec![0; n_rot];

    let mut rot_pairings: Vec<Vec<usize>> = vec![Vec::new(); n_rot];
    for p_idx in 0..genome.len() {
        let r = genome[p_idx];
        if r < n_rot {
            if data[p_idx].base == crew[r].base {
                rot_pairings[r].push(p_idx);
            } else {
                genome[p_idx] = n_rot;
            }
        }
    }

    for r in 0..n_rot {
        rot_pairings[r].sort_by_key(|&p_idx| data[p_idx].start);
        let mut last_end = DateTime::<Utc>::MIN_UTC;
        let mut cum_mins = 0i64;

        for &p_idx in &rot_pairings[r] {
            let p = &data[p_idx];
            let overlaps = p.start < last_end;
            let exceeds_85h = cum_mins + p.minutes > 85 * 60;

            if overlaps || exceeds_85h {
                genome[p_idx] = n_rot;
            } else {
                last_end = p.end;
                cum_mins += p.minutes;
                rot_intervals[r].push((p.start, p.end));
                rot_minutes[r] += p.minutes;
            }
        }
    }

    // 2. Ensure no empty rotations
    for r in 0..n_rot {
        if rot_intervals[r].is_empty() {
            let candidate_unassigned: Vec<usize> = genome
                .iter()
                .enumerate()
                .filter(|&(p_idx, &g)| {
                    g == n_rot && data[p_idx].base == crew[r].base && data[p_idx].minutes <= 85 * 60
                })
                .map(|(p_idx, _)| p_idx)
                .collect();
            if let Some(&p_idx) = candidate_unassigned.choose(rng) {
                genome[p_idx] = r;
                rot_intervals[r].push((data[p_idx].start, data[p_idx].end));
                rot_minutes[r] += data[p_idx].minutes;
            }
        }
    }

    // 3. Greedy fill-in of unassigned pairings to maximize coverage
    let mut unassigned: Vec<usize> = genome
        .iter()
        .enumerate()
        .filter(|&(_, &g)| g == n_rot)
        .map(|(p_idx, _)| p_idx)
        .collect();
    unassigned.shuffle(rng);

    for p_idx in unassigned {
        let p = &data[p_idx];
        let mut best_rot = None;
        let mut min_mins = i64::MAX;

        for &r in &valid_rotations[p_idx] {
            if r >= n_rot {
                continue;
            }
            if rot_minutes[r] + p.minutes > 85 * 60 {
                continue;
            }
            let mut overlaps = false;
            for &(s, e) in &rot_intervals[r] {
                if !(p.end <= s || p.start >= e) {
                    overlaps = true;
                    break;
                }
            }
            if !overlaps && rot_minutes[r] < min_mins {
                min_mins = rot_minutes[r];
                best_rot = Some(r);
            }
        }

        if let Some(r) = best_rot {
            genome[p_idx] = r;
            rot_intervals[r].push((p.start, p.end));
            rot_minutes[r] += p.minutes;
        }
    }
}

// ── Randomized Feasible Individual Generator ──────────────────────────────────

fn generate_random_greedy(
    data: &[PrecomputedPairingData],
    crew: &[RawCrew],
    valid_rotations: &[Vec<usize>],
    n_rot: usize,
    rng: &mut impl Rng,
) -> Vec<usize> {
    let n_pairings = data.len();
    let mut genome = vec![n_rot; n_pairings];

    let mut rot_intervals: Vec<Vec<(DateTime<Utc>, DateTime<Utc>)>> = vec![Vec::new(); n_rot];
    let mut rot_minutes = vec![0i64; n_rot];

    // Seed each rotation with 1 random base-compatible pairing
    let mut assigned = HashSet::new();
    let mut crew_indices: Vec<usize> = (0..n_rot).collect();
    crew_indices.shuffle(rng);

    for &r in &crew_indices {
        let mut candidates: Vec<usize> = (0..n_pairings)
            .filter(|&p_idx| {
                !assigned.contains(&p_idx)
                    && data[p_idx].base == crew[r].base
                    && data[p_idx].minutes <= 85 * 60
            })
            .collect();
        candidates.shuffle(rng);
        if let Some(&p_idx) = candidates.first() {
            genome[p_idx] = r;
            assigned.insert(p_idx);
            rot_intervals[r].push((data[p_idx].start, data[p_idx].end));
            rot_minutes[r] += data[p_idx].minutes;
        }
    }

    // Consider remaining pairings in random order
    let mut remaining: Vec<usize> = (0..n_pairings).filter(|p| !assigned.contains(p)).collect();
    remaining.shuffle(rng);

    for p_idx in remaining {
        let p = &data[p_idx];
        let mut eligible_rots: Vec<usize> = Vec::new();

        for &r in &valid_rotations[p_idx] {
            if r >= n_rot {
                continue;
            }
            if rot_minutes[r] + p.minutes > 85 * 60 {
                continue;
            }
            let mut overlaps = false;
            for &(s, e) in &rot_intervals[r] {
                if !(p.end <= s || p.start >= e) {
                    overlaps = true;
                    break;
                }
            }
            if !overlaps {
                eligible_rots.push(r);
            }
        }

        if !eligible_rots.is_empty() {
            eligible_rots.sort_by_key(|&r| rot_minutes[r]);
            let choice = if eligible_rots.len() > 1 && rng.gen_bool(0.3) {
                eligible_rots[1]
            } else {
                eligible_rots[0]
            };
            genome[p_idx] = choice;
            rot_intervals[choice].push((p.start, p.end));
            rot_minutes[choice] += p.minutes;
        }
    }

    genome
}

// ── Base-Aware, 85h-Capped Greedy Baseline ────────────────────────────────────

fn run_greedy_baseline(
    data: &[PrecomputedPairingData],
    crew: &[RawCrew],
    valid_rotations: &[Vec<usize>],
    n_rot: usize,
    total_flights: usize,
) -> (f64, Vec<usize>, u128) {
    let t_start = std::time::Instant::now();
    let n_pairings = data.len();
    let mut genome = vec![n_rot; n_pairings];

    let mut assigned_pairings = HashSet::new();
    let mut rot_intervals: Vec<Vec<(DateTime<Utc>, DateTime<Utc>)>> = vec![Vec::new(); n_rot];
    let mut rot_minutes = vec![0i64; n_rot];

    let mut sorted_pairing_indices: Vec<usize> = (0..n_pairings).collect();
    sorted_pairing_indices.sort_by_key(|&i| data[i].start);

    // 1. Seed each crew rotation with one base-compatible pairing
    for (rot_idx, c) in crew.iter().take(n_rot).enumerate() {
        for &p_idx in &sorted_pairing_indices {
            if !assigned_pairings.contains(&p_idx)
                && data[p_idx].base == c.base
                && data[p_idx].minutes <= 85 * 60
            {
                genome[p_idx] = rot_idx;
                assigned_pairings.insert(p_idx);
                rot_intervals[rot_idx].push((data[p_idx].start, data[p_idx].end));
                rot_minutes[rot_idx] += data[p_idx].minutes;
                break;
            }
        }
    }

    // 2. Greedily assign remaining pairings to base-compatible, non-overlapping rotations under 85.0h
    for &p_idx in &sorted_pairing_indices {
        if assigned_pairings.contains(&p_idx) {
            continue;
        }
        let p = &data[p_idx];
        let mut best_rot = None;
        let mut min_resulting_minutes = i64::MAX;

        for &rot_idx in &valid_rotations[p_idx] {
            if rot_idx >= n_rot {
                continue;
            }

            // Check 85.0h cap (85 * 60 = 5100 minutes)
            if rot_minutes[rot_idx] + p.minutes > 85 * 60 {
                continue;
            }

            // Check temporal overlap
            let mut overlaps = false;
            for &(s, e) in &rot_intervals[rot_idx] {
                if !(p.end <= s || p.start >= e) {
                    overlaps = true;
                    break;
                }
            }

            if !overlaps && rot_minutes[rot_idx] < min_resulting_minutes {
                min_resulting_minutes = rot_minutes[rot_idx];
                best_rot = Some(rot_idx);
            }
        }

        if let Some(r) = best_rot {
            genome[p_idx] = r;
            rot_intervals[r].push((p.start, p.end));
            rot_minutes[r] += p.minutes;
            assigned_pairings.insert(p_idx);
        }
    }

    let elapsed_ms = t_start.elapsed().as_millis();
    let score = fitness(&genome, data, crew, n_rot, total_flights);
    (score, genome, elapsed_ms)
}

// ── Evolutionary Algorithm ────────────────────────────────────────────────────

fn run_evolutionary(
    data: &[PrecomputedPairingData],
    crew: &[RawCrew],
    valid_rotations: &[Vec<usize>],
    n_rot: usize,
    total_flights: usize,
    greedy_seed: &[usize],
    rng: &mut impl Rng,
) -> (f64, Vec<usize>, usize, u128) {
    let n_pairings = data.len();
    if n_pairings == 0 || n_rot == 0 {
        return (0.0, vec![], 0, 0);
    }

    let t_start = std::time::Instant::now();
    let mut population: Vec<Vec<usize>> = Vec::with_capacity(POP_SIZE);

    if !greedy_seed.is_empty() {
        population.push(greedy_seed.to_vec());
    }

    while population.len() < POP_SIZE {
        let ind = generate_random_greedy(data, crew, valid_rotations, n_rot, rng);
        population.push(ind);
    }

    let mut fitnesses: Vec<f64> = population
        .iter()
        .map(|g| fitness(g, data, crew, n_rot, total_flights))
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

            // Targeted mutations
            if rng.r#gen::<f64>() < 0.40 {
                let n_mut = rng.gen_range(1..=3);
                for _ in 0..n_mut {
                    let p1 = rng.gen_range(0..n_pairings);
                    let valid = &valid_rotations[p1];
                    child[p1] = *valid.choose(rng).unwrap();
                }
            }

            repair(&mut child, data, crew, valid_rotations, n_rot, rng);
            next_pop.push(child);
        }

        let next_fitnesses: Vec<f64> = next_pop
            .iter()
            .map(|g| fitness(g, data, crew, n_rot, total_flights))
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
    base_mismatches: usize,
    overtime_violations: usize,
}

fn compute_operational_kpis(
    genome: &[usize],
    pairings: &[Pairing],
    pairing_bases: &[String],
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
    let mut base_mismatches = 0usize;

    for (i, &rot_idx) in genome.iter().enumerate() {
        if rot_idx < n_rot {
            assigned_pairings += 1;
            if pairing_bases[i] != crew[rot_idx].base {
                base_mismatches += 1;
            }
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
    let overtime_violations = crew_hours.iter().filter(|&&h| h > 85.0 + 1e-4).count();

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
        base_mismatches,
        overtime_violations,
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
    println!("GERAD G2014-22 — Coralys Evolutionary Scheduler v2.0 (4-Parameter Multi-Objective)");
    println!("===============================================================================");
    println!("Optimization Goals:");
    println!("  1. Flight & Pairing Coverage: maximize coverage (uncovered flights -> 0%)");
    println!("  2. Legal Cap: strictly <= 85.0 monthly credited flying hours");
    println!("  3. Base Matching: 100% station base match (pairing.base == crew.base)");
    println!("  4. Workload Equity: minimize credited hours variance across crew");
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
        let (_legs, pairings, pairing_bases, total_csv_pairings) =
            load_benchmark_pairings(&instance_dir, &flights_map);

        let n_pairings = pairings.len();
        let n_crew = crew.len();

        if n_pairings == 0 || n_crew == 0 {
            println!("{:<12} {:>8} (no pairings loaded)", label, 0);
            continue;
        }

        let n_rot = n_crew.min(n_pairings);
        let total_flights = flights_map.len();
        let pairing_data = precompute_pairing_data(&pairings, &pairing_bases);
        let valid_rotations = build_valid_rotations(&pairing_data, &crew, n_rot);

        // Greedy baseline
        let (greedy_score, greedy_seed, greedy_ms) =
            run_greedy_baseline(&pairing_data, &crew, &valid_rotations, n_rot, total_flights);

        // Evolutionary — Coralys v2.0
        let mut rng = StdRng::seed_from_u64(SEED);
        let (ea_score, ea_genome, ea_gen, ea_ms) = run_evolutionary(
            &pairing_data,
            &crew,
            &valid_rotations,
            n_rot,
            total_flights,
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

        let greedy_kpi = compute_operational_kpis(
            &greedy_seed,
            &pairings,
            &pairing_bases,
            &crew,
            &flights_map,
            total_csv_pairings,
        );
        let ea_kpi = compute_operational_kpis(
            &ea_genome,
            &pairings,
            &pairing_bases,
            &crew,
            &flights_map,
            total_csv_pairings,
        );
        let gerad_ref = get_gerad_ref(label);

        println!();
        println!("  ┌── [{label}] Operational KPIs (GERAD G-2014-22 Comparative Alignment) ──");
        println!("  │ • Flight Coverage (Total GERAD Flights: {}):", ea_kpi.total_flights);
        println!(
            "  │   - Coralys v2.0:  {:>5}/{} uncovered ({:.2}%)",
            ea_kpi.uncovered_flights, ea_kpi.total_flights, ea_kpi.uncovered_flights_pct
        );
        println!(
            "  │   - Greedy:        {:>5}/{} uncovered ({:.2}%)",
            greedy_kpi.uncovered_flights, greedy_kpi.total_flights, greedy_kpi.uncovered_flights_pct
        );
        println!(
            "  │   - GERAD Target:  uncovered target: {:.2}%",
            gerad_ref.uncovered_flights_pct
        );
        println!(
            "  │ • Pairing Coverage (Total CSV Pairings: {}, Pre-filtered: {}):",
            ea_kpi.total_pairings_csv, ea_kpi.prefiltered_pairings
        );
        println!(
            "  │   - Coralys v2.0:  {:>5}/{} uncovered ({:.2}%) [assigned: {}]",
            ea_kpi.total_uncovered_pairings,
            ea_kpi.total_pairings_csv,
            ea_kpi.uncovered_pairings_pct,
            ea_kpi.assigned_pairings
        );
        println!(
            "  │   - Greedy:        {:>5}/{} uncovered ({:.2}%) [assigned: {}]",
            greedy_kpi.total_uncovered_pairings,
            greedy_kpi.total_pairings_csv,
            greedy_kpi.uncovered_pairings_pct,
            greedy_kpi.assigned_pairings
        );
        println!(
            "  │ • Credited Flying Hours across {} Crew (Cap: <= 85.0 hrs):",
            n_crew
        );
        println!(
            "  │   - Coralys v2.0:  Mean={:.2}h | Min={:.2}h | Max={:.2}h | StdDev={:.2}h",
            ea_kpi.mean_credited_hours,
            ea_kpi.min_credited_hours,
            ea_kpi.max_credited_hours,
            ea_kpi.std_credited_hours
        );
        println!(
            "  │   - Greedy:        Mean={:.2}h | Min={:.2}h | Max={:.2}h | StdDev={:.2}h",
            greedy_kpi.mean_credited_hours,
            greedy_kpi.min_credited_hours,
            greedy_kpi.max_credited_hours,
            greedy_kpi.std_credited_hours
        );
        println!(
            "  │   - GERAD Target:  Mean={:.2}h | Min={:.2}h | Max={:.2}h | Overall Target Mean: 75.92h",
            gerad_ref.avg_hours, gerad_ref.min_hours, gerad_ref.max_hours
        );
        println!("  │ • Feasibility & Legality Compliance:");
        println!(
            "  │   - Base Mismatches: Coralys={} | Greedy={}",
            ea_kpi.base_mismatches, greedy_kpi.base_mismatches
        );
        println!(
            "  │   - 85.0h Cap Violations: Coralys={} | Greedy={}",
            ea_kpi.overtime_violations, greedy_kpi.overtime_violations
        );
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
             coralys_mean_hrs={:.2}h greedy_mean_hrs={:.2}h \
             coralys_base_mismatches={} greedy_base_mismatches={} \
             coralys_cap_violations={} greedy_cap_violations={}",
            ea_kpi.uncovered_flights_pct,
            greedy_kpi.uncovered_flights_pct,
            ea_kpi.mean_credited_hours,
            greedy_kpi.mean_credited_hours,
            ea_kpi.base_mismatches,
            greedy_kpi.base_mismatches,
            ea_kpi.overtime_violations,
            greedy_kpi.overtime_violations
        );
    }
}
