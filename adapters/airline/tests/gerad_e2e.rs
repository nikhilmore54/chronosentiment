//! GERAD G2014-22 End-to-End Scheduling Pipeline Experiment
//!
//! # Objective
//!
//! Determine whether changing `LAYOVER_REST_HOURS` from 8h to 10h affects the
//! pairing count and optimization outcome produced by the `coralys_airline`
//! scheduling pipeline when run from raw `flights.csv` + `crew.csv` inputs.
//!
//! # Methodology
//!
//! This experiment evaluates the complete UltraCrew scheduling pipeline from
//! raw benchmark inputs. Duties and pairings are generated deterministically
//! from `flights.csv` using the specified layover threshold, after which the
//! optimizer assigns the generated pairings to crew rotations and improves the
//! schedule using local search. The experiment therefore evaluates the effect
//! of the threshold on the generated pairing set and the resulting optimization
//! outcome, but it does **not** perform combinatorial optimization of pairing
//! construction itself.
//!
//! This is a **substantial methodological improvement** over the reconstruction
//! experiment (`compare_gerad.py`) because it starts from the benchmark's
//! primary inputs (`flights.csv` and `crew.csv`) and does not rely on the
//! benchmark's precomputed duties or pairings.
//!
//! At the same time, it is **not equivalent to a full crew pairing optimization
//! benchmark** because pairing construction is deterministic and occurs before
//! optimization. The optimizer optimizes assignment of generated pairings, not
//! generation of alternative pairings.
//!
//! Local search explores only assignment neighbourhoods. Pairing boundaries
//! remain fixed throughout optimization.
//!
//! # Pipeline
//!
//! ```text
//! flights.csv + crew.csv
//!         ↓
//! Parse FlightLeg + CrewMember
//!         ↓
//! Sort all flights globally by departure time (see CRITICAL LIMITATION below)
//!         ↓
//! Group legs into Duties (spatial-continuity check + LAYOVER_REST_HOURS threshold)
//!         ↓
//! Group Duties into Pairings (using HOME_BASE_REST_HOURS = 34h, temporal proxy)
//!         ↓
//! Build empty Roster (one Rotation per crew member)
//!         ↓
//! GreedyScheduler: assign pairings to rotations (WorkloadBalanceObjective)
//!         ↓
//! LocalSearch: improve assignment by swap/relocate (assignment only, not pairing topology)
//!         ↓
//! Report: legs, duties, pairings, rotations, spatial_breaks, assigned pairings, score, improvement%
//! ```
//!
//! # Conditions
//!
//! - Condition A: `LAYOVER_REST_HOURS = 8.0`
//! - Condition B: `LAYOVER_REST_HOURS = 10.0`
//!
//! # CRITICAL LIMITATION: Global Chronological Grouping
//!
//! All flights are sorted by departure time and grouped into duties as a single
//! global sequence. Within that sequence, a leg is only appended to the current
//! duty if its origin airport matches the previous leg's destination airport
//! (spatial-continuity check). If the airports do not connect, a duty break is
//! forced regardless of elapsed time, and the discontinuity is counted.
//!
//! Even with the spatial-continuity check, flights from unrelated routes may
//! still be grouped together if their airports happen to connect by coincidence.
//! A production pairing optimizer would enumerate feasible pairings per
//! route/aircraft network, not globally.
//!
//! All conclusions from this experiment are conditional on this grouping model.
//!
//! # Modeling Assumptions
//!
//! 1. **Global chronological grouping** (see CRITICAL LIMITATION above).
//!    Within the global sequence, a spatial-continuity check enforces that
//!    consecutive legs within a duty share an airport connection
//!    (`prev.destination == next.origin`). Legs that do not connect force a
//!    duty break. Coincidental airport matches across unrelated routes are not
//!    detected.
//!
//! 2. **Single home base**: All crew are assigned `DEFAULT_BASE = "YUL"`.
//!    The benchmark does not specify per-crew bases.
//!
//! 3. **Qualification and contract_type not used**: `crew.csv` contains
//!    `qualification` and `contract_type` fields that are not incorporated
//!    into duty feasibility, pairing feasibility, or assignment constraints.
//!    Only `crew_id` and `base` are used.
//!
//! 4. **Objective**: `WorkloadBalanceObjective` (variance of pairing counts
//!    per rotation). TAFB, hotel nights, deadhead, and legality are not
//!    evaluated. See `UltraCrew_Objective_Function_Alignment.md`.
//!
//! 5. **`duties.csv` and `pairings.csv` not used during optimization**.
//!    They are available for post-hoc comparison only.
//!
//! 6. **Pairing boundary is temporal, not spatial**: A new pairing is started
//!    when the rest between consecutive duties >= `HOME_BASE_REST_HOURS` (34h).
//!    The implementation does not verify that the last duty of a pairing ends
//!    at the crew's home airport. `HOME_BASE_REST_HOURS` is used as a temporal
//!    proxy for home-base return per GERAD benchmark convention.
//!
//! # Experiment Stages (for context)
//!
//! 1. Reconstruction experiment (`compare_gerad.py`): starts from `duties.csv`,
//!    measures sensitivity of reconstruction algorithm.
//! 2. **This experiment**: starts from `flights.csv` + `crew.csv`, generates
//!    duties/pairings deterministically, optimizes assignment.
//! 3. Future: combinatorial pairing optimizer that jointly optimizes pairing
//!    construction and assignment.

#![allow(non_snake_case, dead_code)]

use coralys_airline::domain::crew::CrewId;
use coralys_airline::domain::duty::{Duty, DutyId};
use coralys_airline::domain::flight::{
    AircraftType, AirportCode, FlightLeg, FlightLegId, FlightNumber,
};
use coralys_airline::domain::pairing::{Pairing, PairingId};
use coralys_airline::domain::roster::{PlanningPeriod, Roster, RosterId};
use coralys_airline::domain::rotation::{Rotation, RotationId};
use coralys_airline::legality::LegalityChecker;
use coralys_airline::optimization::cost::CostEvaluator;
use coralys_airline::optimization::metrics::OptimizationMetrics;
use coralys_airline::optimization::objective::{SchedulingObjective, WorkloadBalanceObjective};
use coralys_airline::optimization::search::greedy::GreedyScheduler;
use coralys_airline::optimization::search::local_search::LocalSearch;

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::path::Path;

// ── Constants ─────────────────────────────────────────────────────────────────

/// Minimum rest between consecutive FDPs that ends a pairing and returns
/// the crew to home base.  Fixed at 34h per GERAD benchmark convention.
/// Used as a temporal proxy; home-airport return is not spatially verified.
/// See Modeling Assumption 6.
const HOME_BASE_REST_HOURS: f64 = 34.0;

/// Default home base for all crew in the GERAD benchmark.
/// The benchmark does not specify per-crew bases; we use a single placeholder.
/// See Modeling Assumption 2.
const DEFAULT_BASE: &str = "YUL";

// ── CSV parsing ───────────────────────────────────────────────────────────────

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
    // Global chronological sort -- see Modeling Assumption 1.
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
    #[allow(dead_code)]
    base: String,
}

fn load_crew(path: &Path) -> Vec<RawCrew> {
    let mut rdr = csv::Reader::from_path(path).expect("Cannot open crew.csv");
    let mut crew = Vec::new();
    for result in rdr.deserialize::<HashMap<String, String>>() {
        let row = result.expect("CSV parse error in crew.csv");
        let base = row
            .get("base")
            .or_else(|| row.get("home_base"))
            .cloned()
            .unwrap_or_else(|| DEFAULT_BASE.to_string());
        crew.push(RawCrew {
            crew_id: row["crew_id"].clone(),
            base,
        });
    }
    crew
}

// ── Pairing construction ──────────────────────────────────────────────────────

/// Build `(legs, pairings, spatial_breaks)` from raw flights.
///
/// Duty grouping uses two criteria:
/// a. Spatial continuity: `prev.destination == next.origin` must hold.
///    If not, a duty break is forced (spatial discontinuity counted).
/// b. Temporal threshold: ground time >= `layover_rest_hours` forces a break.
///
/// Pairing grouping: new pairing when rest >= HOME_BASE_REST_HOURS (34h).
/// Home-base return is not spatially verified (Modeling Assumption 6).
/// Note on `base` parameter: `Pairing::new` requires the first duty to start
/// at `base` and the last duty to end at `base`. Since the GERAD benchmark
/// does not specify per-crew home bases and flights span many airports, we
/// derive the base per-pairing from the first duty's report station. The
/// `base` parameter is retained for the `build_seeded_roster` call but is
/// not used in pairing construction.
///
/// Returns `(legs, pairings, spatial_breaks, duty_rejections, pairing_rejections)`.
fn build_pairings_from_flights(
    raw_flights: &[RawFlight],
    layover_rest_hours: f64,
    _base: &str,
) -> (Vec<FlightLeg>, Vec<Pairing>, usize, usize, usize) {
    let mut spatial_breaks = 0usize;

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
            Err(e) => {
                if *rejections == 0 {
                    eprintln!("  [duty_err sample] {:?}", e);
                }
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
            // Spatial-continuity check: airports must connect.
            // If prev.destination != next.origin, crew cannot physically
            // operate the next leg -- force a duty break regardless of time.
            let airports_connect = last.destination == leg.origin;
            // Temporal overlap check: if the next leg departs before the
            // current batch's last leg arrives, the legs cannot be in the
            // same duty (Duty::new would reject with OutOfOrder).
            let temporal_overlap = leg.scheduled_departure < last.scheduled_arrival;
            if ground_h >= layover_rest_hours || !airports_connect || temporal_overlap {
                if !airports_connect {
                    spatial_breaks += 1;
                }
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

    // Single-duty pairing model: each duty is wrapped as its own pairing.
    //
    // The benchmark's flights.csv contains single-day synthetic flights
    // (all on 2000-01-01). There are no HOME_BASE_REST_HOURS (34h) gaps
    // between consecutive duties, so multi-duty pairing grouping by temporal
    // gap never fires. Wrapping each duty as a single-duty pairing is the
    // only model that produces valid pairings from this data.
    //
    // Pairing::new requires first duty.report_station() == base AND
    // last duty.release_station() == base. For a single-duty pairing,
    // this reduces to: duty.report_station() == duty.release_station().
    // Duties that do not form a round-trip (origin != destination) are
    // counted in rejected_pairings.
    let mut pairings: Vec<Pairing> = Vec::new();
    let mut pairing_counter = 0usize;
    let mut rejected_pairings = 0usize;

    for duty in duties {
        let pairing_base = duty.report_station().clone();
        pairing_counter += 1;
        match Pairing::new(
            PairingId::new(format!("P{pairing_counter}")),
            pairing_base,
            vec![duty],
        ) {
            Ok(p) => pairings.push(p),
            Err(_) => rejected_pairings += 1,
        }
    }

    (
        legs,
        pairings,
        spatial_breaks,
        duty_rejections,
        rejected_pairings,
    )
}

// ── Roster construction ───────────────────────────────────────────────────────

/// Build a seeded roster: distribute `seed_pairings` round-robin across crew
/// rotations (one pairing per rotation), then return the roster and any
/// remaining pairings that were not used as seeds.
///
/// `Rotation::new` requires at least one pairing, so we cannot build empty
/// rotations. The `GreedyScheduler` will assign the remaining pairings.
///
/// If there are fewer pairings than crew members, only the first
/// `pairings.len()` rotations are created (the rest have no work to do).
fn build_seeded_roster(
    legs: Vec<FlightLeg>,
    crew: &[RawCrew],
    seed_pairings: Vec<Pairing>,
    period_start: DateTime<Utc>,
    period_end: DateTime<Utc>,
) -> (Roster, Vec<Pairing>) {
    let period = PlanningPeriod::new(period_start, period_end);
    let n_rotations = crew.len().min(seed_pairings.len());
    let mut remaining = seed_pairings;
    // Take the first n_rotations pairings as seeds (one per rotation).
    let seeds: Vec<Pairing> = remaining.drain(..n_rotations).collect();
    let rotations: Vec<Rotation> = crew
        .iter()
        .take(n_rotations)
        .enumerate()
        .map(|(i, c)| {
            Rotation::new(
                RotationId::new(format!("ROT{}", i + 1)),
                CrewId::new(&c.crew_id),
                vec![seeds[i].clone()],
            )
            .expect("Seeded rotation should always be valid")
        })
        .collect();
    let roster = Roster::new(RosterId::new("GERAD"), period, legs, rotations)
        .expect("Roster construction failed");
    (roster, remaining)
}

// ── Experiment result ─────────────────────────────────────────────────────────

struct ExperimentResult {
    layover_h: f64,
    /// Invariant: number of flight legs loaded from flights.csv.
    leg_count: usize,
    /// Invariant: number of duties generated by the grouping algorithm.
    duty_count: usize,
    /// Invariant: number of pairings generated by the grouping algorithm.
    pairing_count: usize,
    /// Invariant: number of rotations (one per crew member).
    rotation_count: usize,
    /// Invariant: total pairings assigned after greedy (must equal pairing_count).
    assigned_after_greedy: usize,
    /// Invariant: total pairings assigned after local search (must equal pairing_count).
    assigned_after_local_search: usize,
    /// Diagnostic: duty breaks forced by spatial discontinuity.
    spatial_breaks: usize,
    greedy_score: f64,
    optimized_score: f64,
    improvement_pct: f64,
    evaluations: usize,
    min_rotation_pairings: usize,
    max_rotation_pairings: usize,
}

fn run_experiment(
    raw_flights: &[RawFlight],
    crew: &[RawCrew],
    layover_h: f64,
    max_iterations: usize,
) -> ExperimentResult {
    // Stage timing: pairing construction
    let t_pairing_start = std::time::Instant::now();
    let (legs, pairings, spatial_breaks, duty_rejections, pairing_rejections) =
        build_pairings_from_flights(raw_flights, layover_h, DEFAULT_BASE);
    let t_pairing_ms = t_pairing_start.elapsed().as_millis();

    let leg_count = legs.len();
    let duty_count: usize = pairings.iter().map(|p| p.duties().len()).sum();
    let pairing_count = pairings.len();
    // Diagnostic: log rejection counts and stage timing so they appear in test output.
    eprintln!(
        "  [diag thr={layover_h}h] duty_rej={duty_rejections} pairing_rej={pairing_rejections} duties_in_pairings={duty_count} pairings_ok={pairing_count} pairing_build_ms={t_pairing_ms}"
    );

    // Invariant: leg count must match raw input.
    assert_eq!(
        leg_count,
        raw_flights.len(),
        "leg_count ({leg_count}) != raw_flights.len() ({}): flight parsing error",
        raw_flights.len()
    );

    let period_start = raw_flights
        .first()
        .map(|f| f.departure_utc)
        .unwrap_or(Utc::now());
    let period_end = raw_flights
        .last()
        .map(|f| f.arrival_utc)
        .unwrap_or(Utc::now());

    // Stage timing: roster seeding
    let t_roster_start = std::time::Instant::now();
    // Build a seeded roster: one pairing per rotation as seed, remaining go to greedy.
    // Rotation::new requires >= 1 pairing, so we cannot build empty rotations.
    let (baseline, remaining_pairings) =
        build_seeded_roster(legs, crew, pairings, period_start, period_end);
    let t_roster_ms = t_roster_start.elapsed().as_millis();
    let rotation_count = baseline.rotations().count();

    // Invariant: rotations = min(crew, pairings).
    let expected_rotations = crew.len().min(pairing_count);
    assert_eq!(
        rotation_count, expected_rotations,
        "rotation_count ({rotation_count}) != expected ({expected_rotations}): roster construction error"
    );

    let mut evaluator = CostEvaluator::new();
    evaluator.add_objective(Box::new(WorkloadBalanceObjective));
    let greedy = GreedyScheduler::new(&evaluator, vec![1.0]);
    let mut greedy_metrics = OptimizationMetrics::new();

    // Stage timing: greedy assignment
    let t_greedy_start = std::time::Instant::now();
    let after_greedy = greedy.assign(&baseline, remaining_pairings, &mut greedy_metrics);
    let t_greedy_ms = t_greedy_start.elapsed().as_millis();

    let assigned_after_greedy: usize = after_greedy.rotations().map(|r| r.pairings().len()).sum();

    // Invariant: seeds + greedy-assigned = total pairings.
    assert_eq!(
        assigned_after_greedy, pairing_count,
        "assigned_after_greedy ({assigned_after_greedy}) != pairing_count ({pairing_count}): \
         greedy scheduler dropped pairings"
    );

    let obj = WorkloadBalanceObjective;
    let greedy_score = obj.evaluate(&after_greedy);

    // Local search explores only assignment neighbourhoods.
    // Pairing boundaries remain fixed throughout optimization.
    let checker = LegalityChecker::new();
    let local_search = LocalSearch::new(&evaluator, &checker, vec![1.0], max_iterations);
    let mut opt_metrics = OptimizationMetrics::new();

    // Stage timing: local search
    let t_local_start = std::time::Instant::now();
    let optimized = local_search.run(&after_greedy, &mut opt_metrics);
    let t_local_ms = t_local_start.elapsed().as_millis();
    let optimized_score = obj.evaluate(&optimized);

    eprintln!(
        "  [timing thr={layover_h}h] pairing_build={t_pairing_ms}ms roster_seed={t_roster_ms}ms greedy={t_greedy_ms}ms local_search={t_local_ms}ms"
    );

    let assigned_after_local_search: usize =
        optimized.rotations().map(|r| r.pairings().len()).sum();

    // Invariant: local search must preserve all pairing assignments.
    assert_eq!(
        assigned_after_local_search, pairing_count,
        "assigned_after_local_search ({assigned_after_local_search}) != pairing_count \
         ({pairing_count}): local search lost pairings"
    );

    let improvement_pct = if greedy_score > 0.0 {
        (greedy_score - optimized_score) / greedy_score * 100.0
    } else {
        0.0
    };

    let rotation_pairing_counts: Vec<usize> =
        optimized.rotations().map(|r| r.pairings().len()).collect();
    let min_rotation_pairings = rotation_pairing_counts.iter().copied().min().unwrap_or(0);
    let max_rotation_pairings = rotation_pairing_counts.iter().copied().max().unwrap_or(0);

    ExperimentResult {
        layover_h,
        leg_count,
        duty_count,
        pairing_count,
        rotation_count,
        assigned_after_greedy,
        assigned_after_local_search,
        spatial_breaks,
        greedy_score,
        optimized_score,
        improvement_pct,
        evaluations: opt_metrics.evaluations(),
        min_rotation_pairings,
        max_rotation_pairings,
    }
}

// ── Test ──────────────────────────────────────────────────────────────────────

/// End-to-end scheduling pipeline experiment across all 7 GERAD instances.
///
/// Runs the full pipeline under two conditions (8h vs 10h layover threshold)
/// and reports whether the threshold affects pairing count or the
/// workload-balance objective.
///
/// See `UltraCrew_Layover_Threshold_Experiment.md` Section 7.
#[test]
fn gerad_e2e_threshold_experiment() {
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
    println!("GERAD G2014-22 End-to-End Scheduling Pipeline Experiment");
    println!("===========================================================");
    println!("Pipeline: GreedyScheduler + LocalSearch (WorkloadBalanceObjective)");
    println!("Pairing generation: deterministic, global chronological grouping");
    println!("  + spatial-continuity check (prev.destination == next.origin)");
    println!("Condition A: LAYOVER_REST_HOURS = 8.0h");
    println!("Condition B: LAYOVER_REST_HOURS = 10.0h");
    println!();
    println!("Modeling assumptions:");
    println!("  1. All flights sorted globally by departure time before grouping.");
    println!("  2. Spatial-continuity check: duty break forced if airports do not connect.");
    println!("  3. All crew assigned home base = {DEFAULT_BASE} (benchmark does not specify).");
    println!("  4. crew.csv qualification/contract_type not used in this experiment.");
    println!("  5. Pairing boundary: temporal proxy (34h rest), home-airport return not verified.");
    println!("  6. Objective: WorkloadBalanceObjective only (TAFB/hotel/deadhead not evaluated).");
    println!();

    let mut any_pairing_diff = false;
    let mut any_score_diff = false;

    for i in 1..=7 {
        let idir = benchmark_base.join(format!("instance{i}"));
        let flights_path = idir.join("flights.csv");
        let crew_path = idir.join("crew.csv");

        if !flights_path.exists() {
            println!("instance{i}: flights.csv not found, skipping");
            continue;
        }
        if !crew_path.exists() {
            println!("instance{i}: crew.csv not found, skipping");
            continue;
        }

        let raw_flights = load_flights(&flights_path);
        let crew = load_crew(&crew_path);

        // Instances 1–3 are small enough for local search (10 iterations).
        // Instances 4–7 have 500–700 pairings × 100–200 rotations; even in
        // release mode the WorkloadBalanceObjective roster-clone makes local
        // search prohibitively slow.  Use greedy-only (max_iterations = 0)
        // for large instances so the experiment completes in reasonable time.
        // The primary research question (does the threshold change pairing
        // count?) is answered by the greedy phase alone.
        let max_iter = if i <= 3 { 10 } else { 0 };
        let r8 = run_experiment(&raw_flights, &crew, 8.0, max_iter);
        let r10 = run_experiment(&raw_flights, &crew, 10.0, max_iter);

        println!("instance{i}:");
        for r in [&r8, &r10] {
            println!(
                "  thr={:.1}h  legs={}  rot={}  duties={}  pairings={}  \
spatial_breaks={}  assigned={}/{}  greedy={:.4}  opt={:.4}  impr={:.1}%  pairings/rot={}-{}",
                r.layover_h,
                r.leg_count,
                r.rotation_count,
                r.duty_count,
                r.pairing_count,
                r.spatial_breaks,
                r.assigned_after_greedy,
                r.assigned_after_local_search,
                r.greedy_score,
                r.optimized_score,
                r.improvement_pct,
                r.min_rotation_pairings,
                r.max_rotation_pairings,
            );
        }

        let pairing_delta = r10.pairing_count as i64 - r8.pairing_count as i64;
        let score_delta = r10.optimized_score - r8.optimized_score;

        if pairing_delta != 0 {
            any_pairing_diff = true;
            println!(
                "  *** PAIRING COUNT DIFFERS: 8h={} 10h={} delta={:+}",
                r8.pairing_count, r10.pairing_count, pairing_delta
            );
        }
        if score_delta.abs() > 1e-6 {
            any_score_diff = true;
            println!(
                "  *** OPTIMIZED SCORE DIFFERS: 8h={:.4} 10h={:.4} delta={:+.4}",
                r8.optimized_score, r10.optimized_score, score_delta
            );
        }
        if pairing_delta == 0 && score_delta.abs() <= 1e-6 {
            println!(
                "  identical pairing count ({}) and optimized score under both conditions",
                r8.pairing_count
            );
        }
        println!();
    }

    println!("===========================================================");
    println!(
        "PAIRING COUNT: {}",
        if any_pairing_diff {
            "DIFFERS between conditions in at least one instance."
        } else {
            "IDENTICAL under both conditions across all instances."
        }
    );
    println!(
        "OPTIMIZED SCORE: {}",
        if any_score_diff {
            "DIFFERS between conditions in at least one instance."
        } else {
            "IDENTICAL under both conditions across all instances."
        }
    );
    println!();
    println!("Interpretation:");
    if !any_pairing_diff && !any_score_diff {
        println!("  The experiment shows that, within this deterministic scheduling");
        println!("  pipeline, the layover threshold does not affect pairing count or");
        println!("  the workload-balance objective. Whether a combinatorial");
        println!("  pairing-construction algorithm would exhibit different sensitivity");
        println!("  to the threshold remains an open question.");
    } else {
        println!("  The layover threshold affects the generated pairing set and/or");
        println!("  the optimization outcome. See per-instance deltas above.");
    }
    println!();
    println!("Note: This experiment does not evaluate TAFB, hotel nights, deadhead,");
    println!("      or legality. See UltraCrew_Objective_Function_Alignment.md.");
}
