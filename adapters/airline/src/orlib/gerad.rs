//! GERAD G-2014-22 Crew Rostering Module for Coralys.
//!
//! Implements the canonical [`coralys_core::DecisionPlugin`] and [`coralys_core::ReasoningEngine`]
//! contracts for monthly airline crew rostering, evaluating against the benchmark datasets from
//! the GERAD G-2014-22 repository (Kasirzadeh et al., 2017 and Racette et al., 2025/2026).
//!
//! # Core Architectural Components
//! - **Statutory Legality Invariant Auditing**: Evaluates 100% home base matching, CAR 700 12.0h rest,
//!   hard 85.00h monthly flying cap, <= 6 consecutive work days, >= 10 days off, and AST vacation non-overlap.
//! - **Multi-Pass Disjoint Ejection Chain Search**: Rescues unassigned pairings via direct packing
//!   and 1-swap ejection cascades to maximize flight coverage.
//! - **Duty Rebundling at Base Touches**: Decomposes monolithic multi-duty pairings at hub visits
//!   into closed out-and-back turns to eliminate uncovered flight legs.
//! - **Multi-Objective Preference Evaluator**: Balances flying equity (workload standard deviation sigma)
//!   against crew bids (PAL preferred flights and vacation days granted).
//! - **Cross-Base Deadhead Routing (Mode C)**: Solves network capacity deficits via point-to-point and
//!   2-hop hub connecting deadhead trajectories with dynamic CAR 700 50% credit accounting.

use crate::domain::flight::{AircraftType, AirportCode, FlightLeg, FlightLegId, FlightNumber};

use chrono::{DateTime, Datelike, Duration, Utc};
use coralys_core::models::decision_lineage::LineageNode;
use coralys_core::{
    DecisionLineage, DecisionPlugin, DecisionProposal, EvaluationResult,
    ReasoningEngine, SimulationResult, StateReference, Violation,
};
use rand::prelude::*;
use rand::rngs::StdRng;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Mutex;
use uuid::Uuid;

// ── Statutory & Regulatory Working Rule Constants ────────────────────────────

pub const MIN_REST_MINUTES: i64 = 720; // 12.0 hours minimum rest between pairings (CAR 700 / GERAD)
pub const MAX_CONSEC_WORK_DAYS: usize = 6; // <= 6 consecutive work days
pub const MIN_DAYS_OFF: usize = 10; // >= 10 days off per 31-day month

// ── Domain Data Structures ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawCrew {
    pub crew_id: String,
    pub base: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScheduledFlight {
    pub flight_id: String,
    pub origin: String,
    pub destination: String,
    pub departure: DateTime<Utc>,
    pub arrival: DateTime<Utc>,
    pub block_minutes: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrecomputedPairingData {
    pub id: String,
    pub base: String,
    pub origin: String,
    pub dest: String,
    pub minutes: i64,
    pub acft_minutes: i64, // block minutes + 75m duty allowance (briefing/debriefing)
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub calendar_days: u32, // bitmask of calendar days (1..31)
    pub flight_ids: Vec<String>,
    pub duty_ids: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrecomputedDutyData {
    pub did: String,
    pub minutes: i64,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub calendar_days: u32,
    pub flight_ids: Vec<String>,
    pub origin: String,
    pub destination: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PilotPreference {
    pub crew_id: String,
    pub base: String,
    pub preferred_flights: HashSet<String>,
    pub vacation_days: HashSet<usize>,
    pub preassigned_days_off: HashSet<usize>, // Q^k (AST vacations: pre-assigned activities)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptimizationMode {
    ModeAEquityPrimary,
    ModeBPreferenceAware,
    ModeCCrossBaseDeadhead,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationParameters {
    pub pop_size: usize,
    pub generations: usize,
    pub crossover_prob: f64,
    pub tournament_k: usize,
    pub seed: u64,
    pub unassigned_pairing_penalty: f64,
    pub uncovered_flight_penalty: f64,
    pub preference_score_weight: f64,
    pub workload_variance_weight: f64,
    pub deadhead_leg_penalty: f64,
    pub deadhead_credit_hour_penalty: f64,
    pub home_base_bonus: f64,
}

impl OptimizationParameters {
    pub fn for_mode(mode: OptimizationMode, n_pairings: usize) -> Self {
        let generations = if n_pairings > 1000 { 150 } else { 100 };
        match mode {
            OptimizationMode::ModeAEquityPrimary => Self {
                pop_size: 50,
                generations,
                crossover_prob: 0.80,
                tournament_k: 3,
                seed: 42,
                unassigned_pairing_penalty: 1_000_000.0,
                uncovered_flight_penalty: 1_000_000.0,
                preference_score_weight: 0.0,
                workload_variance_weight: 0.01,
                deadhead_leg_penalty: 0.0,
                deadhead_credit_hour_penalty: 0.0,
                home_base_bonus: 0.0,
            },
            OptimizationMode::ModeBPreferenceAware => Self {
                pop_size: 50,
                generations,
                crossover_prob: 0.80,
                tournament_k: 3,
                seed: 42,
                unassigned_pairing_penalty: 1_000_000.0,
                uncovered_flight_penalty: 1_000_000.0,
                preference_score_weight: 1.0,
                workload_variance_weight: 0.01,
                deadhead_leg_penalty: 0.0,
                deadhead_credit_hour_penalty: 0.0,
                home_base_bonus: 0.0,
            },
            OptimizationMode::ModeCCrossBaseDeadhead => Self {
                pop_size: 50,
                generations,
                crossover_prob: 0.80,
                tournament_k: 3,
                seed: 42,
                unassigned_pairing_penalty: 1_000_000.0,
                uncovered_flight_penalty: 1_000_000.0,
                preference_score_weight: 1.0,
                workload_variance_weight: 0.01,
                deadhead_leg_penalty: 500.0,
                deadhead_credit_hour_penalty: 20.0,
                home_base_bonus: 10_000.0,
            },
        }
    }
}

impl Default for OptimizationParameters {
    fn default() -> Self {
        Self::for_mode(OptimizationMode::ModeAEquityPrimary, 500)
    }
}

// ── Airport Transition Graph & Point-to-Point Deadhead Routing ────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeadheadTrajectory {
    pub legs: Vec<ScheduledFlight>,
    pub departure: DateTime<Utc>,
    pub arrival: DateTime<Utc>,
    pub total_block_minutes: i64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AirportTransitionGraph {
    pub direct_lookup: HashMap<(String, String), Vec<ScheduledFlight>>,
    pub connecting_hubs: HashMap<(String, String), Vec<String>>,
}

impl AirportTransitionGraph {
    pub fn new(flights: &[ScheduledFlight]) -> Self {
        let mut direct_lookup: HashMap<(String, String), Vec<ScheduledFlight>> = HashMap::new();
        let mut airports = HashSet::new();

        for f in flights {
            airports.insert(f.origin.clone());
            airports.insert(f.destination.clone());
            direct_lookup
                .entry((f.origin.clone(), f.destination.clone()))
                .or_default()
                .push(f.clone());
        }

        for list in direct_lookup.values_mut() {
            list.sort_by(|a, b| {
                a.departure
                    .cmp(&b.departure)
                    .then_with(|| a.block_minutes.cmp(&b.block_minutes))
            });
        }

        let mut connecting_hubs: HashMap<(String, String), Vec<String>> = HashMap::new();
        for u in &airports {
            for v in &airports {
                if u == v {
                    continue;
                }
                let mut hubs = Vec::new();
                for h in &airports {
                    if h == u || h == v {
                        continue;
                    }
                    if let (Some(l1), Some(l2)) = (
                        direct_lookup.get(&(u.clone(), h.clone())),
                        direct_lookup.get(&(h.clone(), v.clone())),
                    ) {
                        let total_flights = l1.len() + l2.len();
                        hubs.push((h.clone(), total_flights));
                    }
                }
                if !hubs.is_empty() {
                    hubs.sort_by_key(|&(_, count)| std::cmp::Reverse(count));
                    hubs.truncate(5);
                    connecting_hubs.insert(
                        (u.clone(), v.clone()),
                        hubs.into_iter().map(|(h, _)| h).collect(),
                    );
                }
            }
        }

        Self {
            direct_lookup,
            connecting_hubs,
        }
    }

    pub fn find_inter_duty_deadhead(
        &self,
        origin: &str,
        destination: &str,
        p1_end: DateTime<Utc>,
        p2_start: DateTime<Utc>,
    ) -> Option<DeadheadTrajectory> {
        if origin == destination {
            return None;
        }

        let min_rest = Duration::hours(12);
        if p2_start < p1_end + min_rest {
            return None;
        }

        let min_connect = Duration::minutes(30);
        let mut best: Option<DeadheadTrajectory> = None;

        if let Some(flights) = self.direct_lookup.get(&(origin.to_string(), destination.to_string())) {
            let start_idx = flights.partition_point(|f| f.departure < p1_end);
            for f in &flights[start_idx..] {
                if f.departure > p2_start {
                    break;
                }
                let case1 = f.departure >= p1_end + min_rest && f.arrival <= p2_start;
                let case2 = f.departure >= p1_end && f.arrival + min_rest <= p2_start;
                if case1 || case2 {
                    if best.as_ref().map_or(true, |b| {
                        f.block_minutes < b.total_block_minutes
                            || (f.block_minutes == b.total_block_minutes
                                && (f.arrival - f.departure) < (b.arrival - b.departure))
                    }) {
                        best = Some(DeadheadTrajectory {
                            legs: vec![f.clone()],
                            departure: f.departure,
                            arrival: f.arrival,
                            total_block_minutes: f.block_minutes,
                        });
                    }
                }
            }
        }

        if best.is_some() {
            return best;
        }

        if let Some(hubs) = self.connecting_hubs.get(&(origin.to_string(), destination.to_string())) {
            for h in hubs {
                let leg1_candidates = match self.direct_lookup.get(&(origin.to_string(), h.clone())) {
                    Some(list) => list,
                    None => continue,
                };
                let leg2_candidates = match self.direct_lookup.get(&(h.clone(), destination.to_string())) {
                    Some(list) => list,
                    None => continue,
                };

                let start_idx1 = leg1_candidates.partition_point(|f| f.departure < p1_end);
                for f1 in &leg1_candidates[start_idx1..] {
                    if f1.departure > p2_start {
                        break;
                    }
                    let min_dep2 = f1.arrival + min_connect;
                    if min_dep2 > p2_start {
                        continue;
                    }
                    let start_idx2 = leg2_candidates.partition_point(|f| f.departure < min_dep2);
                    for f2 in &leg2_candidates[start_idx2..] {
                        if f2.departure > p2_start {
                            break;
                        }
                        if f2.arrival > p2_start {
                            continue;
                        }

                        let case1 = f1.departure >= p1_end + min_rest && f2.arrival <= p2_start;
                        let case2 = f1.departure >= p1_end && f2.arrival + min_rest <= p2_start;
                        if case1 || case2 {
                            let total_block = f1.block_minutes + f2.block_minutes;
                            if best.as_ref().map_or(true, |b| {
                                total_block < b.total_block_minutes
                                    || (total_block == b.total_block_minutes
                                        && (f2.arrival - f1.departure) < (b.arrival - b.departure))
                            }) {
                                best = Some(DeadheadTrajectory {
                                    legs: vec![f1.clone(), f2.clone()],
                                    departure: f1.departure,
                                    arrival: f2.arrival,
                                    total_block_minutes: total_block,
                                });
                            }
                        }
                    }
                }
            }
        }

        best
    }

    pub fn find_outbound_positioning(
        &self,
        base: &str,
        destination: &str,
        latest_arr: DateTime<Utc>,
    ) -> Option<DeadheadTrajectory> {
        if base == destination {
            return None;
        }
        let earliest_dep = latest_arr - Duration::hours(48);
        let min_connect = Duration::minutes(30);
        let mut best: Option<DeadheadTrajectory> = None;

        if let Some(flights) = self.direct_lookup.get(&(base.to_string(), destination.to_string())) {
            let start_idx = flights.partition_point(|f| f.departure < earliest_dep);
            for f in &flights[start_idx..] {
                if f.departure > latest_arr {
                    break;
                }
                if f.arrival <= latest_arr {
                    if best.as_ref().map_or(true, |b| f.block_minutes < b.total_block_minutes) {
                        best = Some(DeadheadTrajectory {
                            legs: vec![f.clone()],
                            departure: f.departure,
                            arrival: f.arrival,
                            total_block_minutes: f.block_minutes,
                        });
                    }
                }
            }
        }
        if best.is_some() {
            return best;
        }

        if let Some(hubs) = self.connecting_hubs.get(&(base.to_string(), destination.to_string())) {
            for h in hubs {
                if let (Some(l1), Some(l2)) = (
                    self.direct_lookup.get(&(base.to_string(), h.clone())),
                    self.direct_lookup.get(&(h.clone(), destination.to_string())),
                ) {
                    let start_idx1 = l1.partition_point(|f| f.departure < earliest_dep);
                    for f1 in &l1[start_idx1..] {
                        if f1.departure > latest_arr {
                            break;
                        }
                        let min_dep2 = f1.arrival + min_connect;
                        if min_dep2 > latest_arr {
                            continue;
                        }
                        let start_idx2 = l2.partition_point(|f| f.departure < min_dep2);
                        for f2 in &l2[start_idx2..] {
                            if f2.departure > latest_arr {
                                break;
                            }
                            if f2.arrival <= latest_arr {
                                let blk = f1.block_minutes + f2.block_minutes;
                                if best.as_ref().map_or(true, |b| blk < b.total_block_minutes) {
                                    best = Some(DeadheadTrajectory {
                                        legs: vec![f1.clone(), f2.clone()],
                                        departure: f1.departure,
                                        arrival: f2.arrival,
                                        total_block_minutes: blk,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        best
    }

    pub fn find_inbound_positioning(
        &self,
        origin: &str,
        base: &str,
        earliest_dep: DateTime<Utc>,
    ) -> Option<DeadheadTrajectory> {
        if origin == base {
            return None;
        }
        let latest_arr = earliest_dep + Duration::hours(48);
        let min_connect = Duration::minutes(30);
        let mut best: Option<DeadheadTrajectory> = None;

        if let Some(flights) = self.direct_lookup.get(&(origin.to_string(), base.to_string())) {
            let start_idx = flights.partition_point(|f| f.departure < earliest_dep);
            for f in &flights[start_idx..] {
                if f.departure > latest_arr {
                    break;
                }
                if f.arrival <= latest_arr {
                    if best.as_ref().map_or(true, |b| f.block_minutes < b.total_block_minutes) {
                        best = Some(DeadheadTrajectory {
                            legs: vec![f.clone()],
                            departure: f.departure,
                            arrival: f.arrival,
                            total_block_minutes: f.block_minutes,
                        });
                    }
                }
            }
        }
        if best.is_some() {
            return best;
        }

        if let Some(hubs) = self.connecting_hubs.get(&(origin.to_string(), base.to_string())) {
            for h in hubs {
                if let (Some(l1), Some(l2)) = (
                    self.direct_lookup.get(&(origin.to_string(), h.clone())),
                    self.direct_lookup.get(&(h.clone(), base.to_string())),
                ) {
                    let start_idx1 = l1.partition_point(|f| f.departure < earliest_dep);
                    for f1 in &l1[start_idx1..] {
                        if f1.departure > latest_arr {
                            break;
                        }
                        let min_dep2 = f1.arrival + min_connect;
                        if min_dep2 > latest_arr {
                            continue;
                        }
                        let start_idx2 = l2.partition_point(|f| f.departure < min_dep2);
                        for f2 in &l2[start_idx2..] {
                            if f2.departure > latest_arr {
                                break;
                            }
                            if f2.arrival <= latest_arr {
                                let blk = f1.block_minutes + f2.block_minutes;
                                if best.as_ref().map_or(true, |b| blk < b.total_block_minutes) {
                                    best = Some(DeadheadTrajectory {
                                        legs: vec![f1.clone(), f2.clone()],
                                        departure: f1.departure,
                                        arrival: f2.arrival,
                                        total_block_minutes: blk,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        best
    }
}

// ── Calendar Bitmask & Invariant Helpers ─────────────────────────────────────

#[inline]
pub fn calendar_mask_for_interval(start: DateTime<Utc>, end: DateTime<Utc>) -> u32 {
    let mut mask = 0u32;
    let s_day = start.date_naive().day() as usize;
    let e_day = end.date_naive().day() as usize;
    for d in s_day..=e_day {
        if (1..=31).contains(&d) {
            mask |= 1u32 << (d - 1);
        }
    }
    mask
}

#[inline(always)]
pub fn violates_working_rules(calendar_mask: u32) -> bool {
    if calendar_mask.count_ones() as usize > (31 - MIN_DAYS_OFF) {
        return true;
    }
    (calendar_mask
        & (calendar_mask >> 1)
        & (calendar_mask >> 2)
        & (calendar_mask >> 3)
        & (calendar_mask >> 4)
        & (calendar_mask >> 5)
        & (calendar_mask >> 6))
        != 0
}

#[inline(always)]
pub fn violates_preassigned(preassigned: &HashSet<usize>, calendar_mask: u32) -> bool {
    for &d in preassigned {
        if (1..=31).contains(&d) && (calendar_mask & (1u32 << (d - 1))) != 0 {
            return true;
        }
    }
    false
}

#[derive(Clone, Debug, Default)]
pub struct RotationEvaluation {
    pub total_acft_minutes: i64,
    pub raw_block_minutes: i64,
    pub calendar_mask: u32,
    pub deadhead_count: usize,
    pub deadhead_block_minutes: i64,
    pub deadhead_credit_minutes: i64,
}

pub fn evaluate_rotation_feasibility(
    pairings: &[usize],
    r: usize,
    data: &[PrecomputedPairingData],
    crew: &[RawCrew],
    prefs: &[PilotPreference],
    mode: OptimizationMode,
    graph: &AirportTransitionGraph,
) -> Option<RotationEvaluation> {
    if pairings.is_empty() {
        return Some(RotationEvaluation::default());
    }

    let min_rest = Duration::minutes(MIN_REST_MINUTES);
    let mut sorted_pairings = pairings.to_vec();
    sorted_pairings.sort_by_key(|&idx| data[idx].start);

    for w in sorted_pairings.windows(2) {
        if data[w[1]].start < data[w[0]].end {
            return None;
        }
    }

    let base = &crew[r].base;

    match mode {
        OptimizationMode::ModeAEquityPrimary | OptimizationMode::ModeBPreferenceAware => {
            let mut total_acft = 0i64;
            let mut raw_block = 0i64;
            let mut cal_mask = 0u32;

            for &p_idx in &sorted_pairings {
                let p = &data[p_idx];
                if p.base != *base {
                    return None;
                }
                total_acft += p.acft_minutes;
                raw_block += p.minutes;
                cal_mask |= p.calendar_days;
            }

            if total_acft > 85 * 60 {
                return None;
            }

            for w in sorted_pairings.windows(2) {
                if data[w[1]].start < data[w[0]].end + min_rest {
                    return None;
                }
            }

            if violates_working_rules(cal_mask) {
                return None;
            }
            if violates_preassigned(&prefs[r].preassigned_days_off, cal_mask) {
                return None;
            }

            Some(RotationEvaluation {
                total_acft_minutes: total_acft,
                raw_block_minutes: raw_block,
                calendar_mask: cal_mask,
                deadhead_count: 0,
                deadhead_block_minutes: 0,
                deadhead_credit_minutes: 0,
            })
        }
        OptimizationMode::ModeCCrossBaseDeadhead => {
            let mut total_acft = 0i64;
            let mut raw_block = 0i64;
            let mut cal_mask = 0u32;
            let mut dh_count = 0usize;
            let mut dh_block = 0i64;

            for &p_idx in &sorted_pairings {
                let p = &data[p_idx];
                total_acft += p.acft_minutes;
                raw_block += p.minutes;
                cal_mask |= p.calendar_days;
            }

            if total_acft > 85 * 60 {
                return None;
            }
            if violates_working_rules(cal_mask) {
                return None;
            }
            if violates_preassigned(&prefs[r].preassigned_days_off, cal_mask) {
                return None;
            }

            let first_p = &data[sorted_pairings[0]];
            if first_p.origin != *base {
                match graph.find_outbound_positioning(base, &first_p.origin, first_p.start) {
                    Some(traj) => {
                        dh_count += traj.legs.len();
                        dh_block += traj.total_block_minutes;
                        cal_mask |= calendar_mask_for_interval(traj.departure, traj.arrival);
                    }
                    None => return None,
                }
            }

            for w in sorted_pairings.windows(2) {
                let p_prev = &data[w[0]];
                let p_next = &data[w[1]];

                if p_prev.dest == p_next.origin {
                    if p_next.start < p_prev.end + min_rest {
                        return None;
                    }
                } else {
                    match graph.find_inter_duty_deadhead(&p_prev.dest, &p_next.origin, p_prev.end, p_next.start) {
                        Some(traj) => {
                            dh_count += traj.legs.len();
                            dh_block += traj.total_block_minutes;
                            cal_mask |= calendar_mask_for_interval(traj.departure, traj.arrival);
                        }
                        None => return None,
                    }
                }
            }

            let last_p = &data[sorted_pairings[sorted_pairings.len() - 1]];
            if last_p.dest != *base {
                match graph.find_inbound_positioning(&last_p.dest, base, last_p.end) {
                    Some(traj) => {
                        dh_count += traj.legs.len();
                        dh_block += traj.total_block_minutes;
                        cal_mask |= calendar_mask_for_interval(traj.departure, traj.arrival);
                    }
                    None => return None,
                }
            }

            let dh_credit = (dh_block + 1) / 2;
            total_acft += dh_credit;
            raw_block += dh_block;

            if total_acft > 85 * 60 {
                return None;
            }
            if violates_working_rules(cal_mask) {
                return None;
            }
            if violates_preassigned(&prefs[r].preassigned_days_off, cal_mask) {
                return None;
            }

            Some(RotationEvaluation {
                total_acft_minutes: total_acft,
                raw_block_minutes: raw_block,
                calendar_mask: cal_mask,
                deadhead_count: dh_count,
                deadhead_block_minutes: dh_block,
                deadhead_credit_minutes: dh_credit,
            })
        }
    }
}

#[inline]
pub fn evaluate_assignment_feasibility(
    p_idx: usize,
    r: usize,
    data: &[PrecomputedPairingData],
    crew: &[RawCrew],
    prefs: &[PilotPreference],
    mode: OptimizationMode,
    rot_pairings: &[usize],
    cur_mins: i64,
    cur_cal: u32,
    graph: &AirportTransitionGraph,
) -> Option<(i64, u32, bool)> {
    let p = &data[p_idx];

    if cur_mins + p.acft_minutes > 85 * 60 {
        return None;
    }
    if mode != OptimizationMode::ModeCCrossBaseDeadhead && p.base != crew[r].base {
        return None;
    }
    let new_cal = cur_cal | p.calendar_days;
    if violates_working_rules(new_cal) {
        return None;
    }
    if violates_preassigned(&prefs[r].preassigned_days_off, new_cal) {
        return None;
    }

    for &existing_idx in rot_pairings {
        let ep = &data[existing_idx];
        if ep.start < p.end && p.start < ep.end {
            return None;
        }
    }

    let base = &crew[r].base;
    let is_pure_home_base = p.base == *base
        && p.origin == *base
        && p.dest == *base
        && rot_pairings.iter().all(|&idx| {
            let ep = &data[idx];
            ep.base == *base && ep.origin == *base && ep.dest == *base
        });

    if mode != OptimizationMode::ModeCCrossBaseDeadhead || is_pure_home_base {
        let min_rest = Duration::minutes(MIN_REST_MINUTES);
        for &existing_idx in rot_pairings {
            let ep = &data[existing_idx];
            if p.start >= ep.end && p.start < ep.end + min_rest {
                return None;
            }
            if ep.start >= p.end && ep.start < p.end + min_rest {
                return None;
            }
        }
        let added_acft = p.acft_minutes;
        return Some((added_acft, new_cal, false));
    }

    let mut cand = rot_pairings.to_vec();
    cand.push(p_idx);
    let eval = evaluate_rotation_feasibility(&cand, r, data, crew, prefs, mode, graph)?;
    let added_acft = eval.total_acft_minutes - cur_mins;
    let is_dh = eval.deadhead_count > 0 || p.base != crew[r].base;
    Some((added_acft, eval.calendar_mask, is_dh))
}

// ── Preference KPI & Conflict Matrix ─────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PreferenceKpi {
    pub total_pal_requested: usize,
    pub total_pal_granted: usize,
    pub pal_granted_pct: f64,
    pub total_vac_requested: usize,
    pub total_vac_granted: usize,
    pub vac_granted_pct: f64,
    pub preference_score_s: f64,
}

pub fn evaluate_preferences(
    genome: &[usize],
    data: &[PrecomputedPairingData],
    prefs: &[PilotPreference],
    n_rot: usize,
    uncovered_flights: usize,
) -> PreferenceKpi {
    let mut total_pal_req = 0usize;
    let mut total_pal_grant = 0usize;
    let mut total_vac_req = 0usize;
    let mut total_vac_grant = 0usize;

    for r in 0..n_rot {
        let pref = &prefs[r];
        total_pal_req += pref.preferred_flights.len();
        total_vac_req += pref.vacation_days.len();

        let mut pilot_flights = HashSet::new();
        let mut pilot_calendar = 0u32;

        for (p_idx, &g_rot) in genome.iter().enumerate() {
            if g_rot == r {
                pilot_calendar |= data[p_idx].calendar_days;
                for fid in &data[p_idx].flight_ids {
                    pilot_flights.insert(fid.as_str());
                }
            }
        }

        for p_fid in &pref.preferred_flights {
            if pilot_flights.contains(p_fid.as_str()) {
                total_pal_grant += 1;
            }
        }

        for &v_day in &pref.vacation_days {
            if (1..=31).contains(&v_day) {
                let worked = (pilot_calendar & (1u32 << (v_day - 1))) != 0;
                if !worked {
                    total_vac_grant += 1;
                }
            }
        }
    }

    let pal_pct = if total_pal_req > 0 {
        total_pal_grant as f64 / total_pal_req as f64 * 100.0
    } else {
        0.0
    };
    let vac_pct = if total_vac_req > 0 {
        total_vac_grant as f64 / total_vac_req as f64 * 100.0
    } else {
        0.0
    };

    let score_s = (total_pal_grant as f64 * 100.0) + (total_vac_grant as f64 * 50.0)
        - (uncovered_flights as f64 * 100.0);

    PreferenceKpi {
        total_pal_requested: total_pal_req,
        total_pal_granted: total_pal_grant,
        pal_granted_pct: pal_pct,
        total_vac_requested: total_vac_req,
        total_vac_granted: total_vac_grant,
        vac_granted_pct: vac_pct,
        preference_score_s: score_s,
    }
}

pub fn build_valid_rotations(
    pairings_data: &[PrecomputedPairingData],
    crew: &[RawCrew],
    n_rot: usize,
    mode: OptimizationMode,
) -> Vec<Vec<usize>> {
    let mut crew_by_base: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, c) in crew.iter().take(n_rot).enumerate() {
        crew_by_base.entry(c.base.clone()).or_default().push(i);
    }

    pairings_data
        .iter()
        .map(|p| {
            let mut valid = crew_by_base.get(&p.base).cloned().unwrap_or_default();
            if mode == OptimizationMode::ModeCCrossBaseDeadhead {
                for (other_base, pilots) in &crew_by_base {
                    if other_base != &p.base {
                        for &c_idx in pilots {
                            if !valid.contains(&c_idx) {
                                valid.push(c_idx);
                            }
                        }
                    }
                }
            }
            valid.push(n_rot);
            valid
        })
        .collect()
}

#[derive(Clone, Debug)]
pub struct ConflictMatrix {
    pub bits: Vec<u64>,
    pub words_per_row: usize,
}

impl ConflictMatrix {
    pub fn new(pairings: &[PrecomputedPairingData]) -> Self {
        let n = pairings.len();
        let words_per_row = (n + 63) / 64;
        let mut bits = vec![0u64; n * words_per_row];
        let min_rest = Duration::minutes(MIN_REST_MINUTES);

        for i in 0..n {
            let pi = &pairings[i];
            let row_offset = i * words_per_row;
            for j in 0..n {
                if i == j {
                    continue;
                }
                let pj = &pairings[j];
                let no_conflict = pj.start >= pi.end + min_rest || pi.start >= pj.end + min_rest;
                if !no_conflict {
                    let word_idx = row_offset + (j / 64);
                    bits[word_idx] |= 1u64 << (j % 64);
                }
            }
        }

        Self { bits, words_per_row }
    }
}

// ── Multi-Objective Fitness Evaluator ─────────────────────────────────────────

pub fn fitness(
    genome: &[usize],
    data: &[PrecomputedPairingData],
    crew: &[RawCrew],
    prefs: &[PilotPreference],
    n_rot: usize,
    total_flights: usize,
    mode: OptimizationMode,
    graph: &AirportTransitionGraph,
    params: &OptimizationParameters,
) -> f64 {
    let mut rot_minutes = vec![0i64; n_rot];
    let mut rot_pairing_indices: Vec<Vec<usize>> = vec![Vec::new(); n_rot];
    let mut unassigned_count = 0usize;
    let mut deadhead_count = 0usize;

    for (i, &rot_idx) in genome.iter().enumerate() {
        if rot_idx < n_rot {
            rot_pairing_indices[rot_idx].push(i);
        } else {
            unassigned_count += 1;
        }
    }

    let mut empty_rotations = 0usize;
    let mut infeasible_rotations = 0usize;
    let mut covered_flights_set = HashSet::new();
    let mut total_deadhead_credit_minutes = 0i64;

    for (rot_idx, indices) in rot_pairing_indices.iter().enumerate() {
        if indices.is_empty() {
            empty_rotations += 1;
        } else {
            match evaluate_rotation_feasibility(indices, rot_idx, data, crew, prefs, mode, graph) {
                Some(eval) => {
                    rot_minutes[rot_idx] = eval.total_acft_minutes;
                    deadhead_count += eval.deadhead_count;
                    total_deadhead_credit_minutes += eval.deadhead_credit_minutes;
                    for &idx in indices {
                        for fid in &data[idx].flight_ids {
                            covered_flights_set.insert(fid.as_str());
                        }
                    }
                }
                None => {
                    infeasible_rotations += 1;
                }
            }
        }
    }

    if infeasible_rotations > 0 || empty_rotations > 0 {
        return 100_000_000.0
            + (infeasible_rotations as f64) * 1_000_000.0
            + (empty_rotations as f64) * 500_000.0
            + (unassigned_count as f64) * 10_000.0;
    }

    let uncovered_flights = total_flights.saturating_sub(covered_flights_set.len());
    let hours: Vec<f64> = rot_minutes.iter().map(|&m| m as f64 / 60.0).collect();
    let mean_hrs = hours.iter().sum::<f64>() / hours.len() as f64;
    let variance_hrs =
        hours.iter().map(|&h| (h - mean_hrs).powi(2)).sum::<f64>() / hours.len() as f64;

    match mode {
        OptimizationMode::ModeAEquityPrimary => {
            (uncovered_flights as f64) * params.uncovered_flight_penalty
                + (unassigned_count as f64) * params.unassigned_pairing_penalty
                + params.workload_variance_weight * variance_hrs
        }
        OptimizationMode::ModeBPreferenceAware => {
            let pref_kpi = evaluate_preferences(genome, data, prefs, n_rot, uncovered_flights);
            (uncovered_flights as f64) * params.uncovered_flight_penalty
                + (unassigned_count as f64) * params.unassigned_pairing_penalty
                + params.workload_variance_weight * variance_hrs
                - params.preference_score_weight * pref_kpi.preference_score_s
        }
        OptimizationMode::ModeCCrossBaseDeadhead => {
            let pref_kpi = evaluate_preferences(genome, data, prefs, n_rot, uncovered_flights);
            (uncovered_flights as f64) * params.uncovered_flight_penalty
                + (unassigned_count as f64) * params.unassigned_pairing_penalty
                + params.workload_variance_weight * variance_hrs
                - params.preference_score_weight * pref_kpi.preference_score_s
                + (deadhead_count as f64) * params.deadhead_leg_penalty
                + (total_deadhead_credit_minutes as f64 / 60.0) * params.deadhead_credit_hour_penalty
        }
    }
}

// ── Smart Feasibility & Packing Repair ────────────────────────────────────────

pub fn repair(
    genome: &mut Vec<usize>,
    data: &[PrecomputedPairingData],
    crew: &[RawCrew],
    prefs: &[PilotPreference],
    valid_rotations: &[Vec<usize>],
    n_rot: usize,
    _conflict_matrix: &ConflictMatrix,
    mode: OptimizationMode,
    rng: &mut impl Rng,
    graph: &AirportTransitionGraph,
    params: &OptimizationParameters,
) {
    if n_rot == 0 {
        return;
    }

    let mut rot_minutes: Vec<i64> = vec![0; n_rot];
    let mut rot_calendar: Vec<u32> = vec![0; n_rot];
    let mut rot_pairings: Vec<Vec<usize>> = vec![Vec::new(); n_rot];

    for p_idx in 0..genome.len() {
        let r = genome[p_idx];
        if r < n_rot {
            let is_home = data[p_idx].base == crew[r].base;
            let is_valid_dh = mode == OptimizationMode::ModeCCrossBaseDeadhead
                && valid_rotations[p_idx].contains(&r);
            if is_home || is_valid_dh {
                rot_pairings[r].push(p_idx);
            } else {
                genome[p_idx] = n_rot;
            }
        }
    }

    for r in 0..n_rot {
        rot_pairings[r].sort_by_key(|&p_idx| data[p_idx].start);
        let mut cum_mins = 0i64;
        let mut cum_cal = 0u32;
        let mut kept = Vec::with_capacity(rot_pairings[r].len());

        for &p_idx in &rot_pairings[r] {
            if let Some((added_acft, new_cal, _)) = evaluate_assignment_feasibility(
                p_idx,
                r,
                data,
                crew,
                prefs,
                mode,
                &kept,
                cum_mins,
                cum_cal,
                graph,
            ) {
                cum_mins += added_acft;
                cum_cal = new_cal;
                kept.push(p_idx);
            } else {
                genome[p_idx] = n_rot;
            }
        }
        rot_pairings[r] = kept;
        rot_minutes[r] = cum_mins;
        rot_calendar[r] = cum_cal;
    }

    // Ensure no empty rotations
    for r in 0..n_rot {
        if rot_pairings[r].is_empty() {
            let mut candidate_unassigned: Vec<usize> = genome
                .iter()
                .enumerate()
                .filter(|&(p_idx, &g)| {
                    g == n_rot
                        && evaluate_assignment_feasibility(
                            p_idx,
                            r,
                            data,
                            crew,
                            prefs,
                            mode,
                            &rot_pairings[r],
                            rot_minutes[r],
                            rot_calendar[r],
                            graph,
                        )
                        .is_some()
                })
                .map(|(p_idx, _)| p_idx)
                .collect();
            candidate_unassigned.sort_by_key(|&p| if data[p].base == crew[r].base { 0 } else { 1 });
            let home_count = candidate_unassigned
                .iter()
                .take_while(|&&p| data[p].base == crew[r].base)
                .count();
            let chosen = if home_count > 0 {
                candidate_unassigned[..home_count].choose(rng).copied()
            } else {
                candidate_unassigned.choose(rng).copied()
            };
            if let Some(p_idx) = chosen {
                if let Some((added_acft, new_cal, _)) = evaluate_assignment_feasibility(
                    p_idx,
                    r,
                    data,
                    crew,
                    prefs,
                    mode,
                    &rot_pairings[r],
                    rot_minutes[r],
                    rot_calendar[r],
                    graph,
                ) {
                    genome[p_idx] = r;
                    rot_pairings[r].push(p_idx);
                    rot_minutes[r] += added_acft;
                    rot_calendar[r] = new_cal;
                }
            }
        }
    }

    // Greedy fill-in of unassigned pairings to maximize coverage
    let mut unassigned: Vec<usize> = genome
        .iter()
        .enumerate()
        .filter(|&(_, &g)| g == n_rot)
        .map(|(p_idx, _)| p_idx)
        .collect();
    unassigned.sort_by_key(|&p_idx| {
        (valid_rotations[p_idx].len(), data[p_idx].start)
    });

    for p_idx in unassigned {
        let mut best_rot = None;
        let mut best_score = if mode == OptimizationMode::ModeBPreferenceAware
            || mode == OptimizationMode::ModeCCrossBaseDeadhead
        {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        };

        let has_feasible_home = if mode == OptimizationMode::ModeCCrossBaseDeadhead {
            valid_rotations[p_idx].iter().any(|&r| {
                r < n_rot
                    && crew[r].base == data[p_idx].base
                    && evaluate_assignment_feasibility(
                        p_idx,
                        r,
                        data,
                        crew,
                        prefs,
                        mode,
                        &rot_pairings[r],
                        rot_minutes[r],
                        rot_calendar[r],
                        graph,
                    )
                    .is_some()
            })
        } else {
            false
        };

        for &r in &valid_rotations[p_idx] {
            if r >= n_rot {
                continue;
            }
            if has_feasible_home && crew[r].base != data[p_idx].base {
                continue;
            }
            if let Some((added_acft, new_cal, is_dh)) = evaluate_assignment_feasibility(
                p_idx,
                r,
                data,
                crew,
                prefs,
                mode,
                &rot_pairings[r],
                rot_minutes[r],
                rot_calendar[r],
                graph,
            ) {
                match mode {
                    OptimizationMode::ModeAEquityPrimary => {
                        let score = (rot_minutes[r] + added_acft) as f64;
                        if score < best_score {
                            best_score = score;
                            best_rot = Some((r, added_acft, new_cal));
                        }
                    }
                    OptimizationMode::ModeBPreferenceAware => {
                        let pref = &prefs[r];
                        let pal_gain = data[p_idx]
                            .flight_ids
                            .iter()
                            .filter(|f| pref.preferred_flights.contains(f.as_str()))
                            .count() as f64;
                        let mut vac_conflicts = 0.0_f64;
                        for &v_day in &pref.vacation_days {
                            if (1..=31).contains(&v_day)
                                && (data[p_idx].calendar_days & (1u32 << (v_day - 1))) != 0
                            {
                                vac_conflicts += 1.0;
                            }
                        }
                        let score = (pal_gain * 50.0) - (vac_conflicts * 30.0)
                            - (rot_minutes[r] as f64 / 60.0 * 2.0);
                        if score > best_score {
                            best_score = score;
                            best_rot = Some((r, added_acft, new_cal));
                        }
                    }
                    OptimizationMode::ModeCCrossBaseDeadhead => {
                        let pref = &prefs[r];
                        let pal_gain = data[p_idx]
                            .flight_ids
                            .iter()
                            .filter(|f| pref.preferred_flights.contains(f.as_str()))
                            .count() as f64;
                        let mut vac_conflicts = 0.0_f64;
                        for &v_day in &pref.vacation_days {
                            if (1..=31).contains(&v_day)
                                && (data[p_idx].calendar_days & (1u32 << (v_day - 1))) != 0
                            {
                                vac_conflicts += 1.0;
                            }
                        }
                        let home_bonus = if data[p_idx].base == crew[r].base {
                            params.home_base_bonus
                        } else {
                            0.0
                        };
                        let dh_penalty = if is_dh {
                            (params.deadhead_leg_penalty * 3.0)
                                + ((added_acft - data[p_idx].acft_minutes).max(0) as f64 / 60.0)
                                    * params.deadhead_credit_hour_penalty
                        } else {
                            0.0
                        };
                        let score = home_bonus + (pal_gain * 50.0) - (vac_conflicts * 30.0)
                            - (rot_minutes[r] as f64 / 60.0 * 20.0)
                            - dh_penalty;
                        if score > best_score {
                            best_score = score;
                            best_rot = Some((r, added_acft, new_cal));
                        }
                    }
                }
            }
        }

        if let Some((r, added_acft, new_cal)) = best_rot {
            genome[p_idx] = r;
            rot_pairings[r].push(p_idx);
            rot_minutes[r] += added_acft;
            rot_calendar[r] = new_cal;
        }
    }
}

// ── Randomized Feasible Individual Generator ──────────────────────────────────

pub fn generate_random_greedy(
    data: &[PrecomputedPairingData],
    crew: &[RawCrew],
    prefs: &[PilotPreference],
    valid_rotations: &[Vec<usize>],
    n_rot: usize,
    _conflict_matrix: &ConflictMatrix,
    mode: OptimizationMode,
    rng: &mut impl Rng,
    graph: &AirportTransitionGraph,
    params: &OptimizationParameters,
) -> Vec<usize> {
    let n_pairings = data.len();
    let mut genome = vec![n_rot; n_pairings];

    let mut rot_pairings: Vec<Vec<usize>> = vec![Vec::new(); n_rot];
    let mut rot_minutes = vec![0i64; n_rot];
    let mut rot_calendar = vec![0u32; n_rot];

    let mut assigned = HashSet::new();
    let mut crew_indices: Vec<usize> = (0..n_rot).collect();
    crew_indices.shuffle(rng);

    for &r in &crew_indices {
        if !rot_pairings[r].is_empty() {
            continue;
        }
        let mut candidates: Vec<usize> = (0..n_pairings)
            .filter(|&p_idx| {
                !assigned.contains(&p_idx)
                    && data[p_idx].base == crew[r].base
                    && data[p_idx].acft_minutes <= 85 * 60
                    && !violates_working_rules(data[p_idx].calendar_days)
                    && !violates_preassigned(
                        &prefs[r].preassigned_days_off,
                        data[p_idx].calendar_days,
                    )
            })
            .collect();
        candidates.shuffle(rng);
        if let Some(&p_idx) = candidates.first() {
            genome[p_idx] = r;
            assigned.insert(p_idx);
            rot_pairings[r].push(p_idx);
            rot_minutes[r] += data[p_idx].acft_minutes;
            rot_calendar[r] |= data[p_idx].calendar_days;
        }
    }

    let mut remaining: Vec<usize> = (0..n_pairings).filter(|p| !assigned.contains(p)).collect();
    remaining.sort_by_key(|&p_idx| {
        (valid_rotations[p_idx].len(), data[p_idx].start)
    });

    for p_idx in remaining {
        let mut eligible_rots: Vec<(usize, i64, u32, bool)> = Vec::new();

        for &r in &valid_rotations[p_idx] {
            if r >= n_rot {
                continue;
            }
            if let Some((added_acft, new_cal, is_dh)) = evaluate_assignment_feasibility(
                p_idx,
                r,
                data,
                crew,
                prefs,
                mode,
                &rot_pairings[r],
                rot_minutes[r],
                rot_calendar[r],
                graph,
            ) {
                eligible_rots.push((r, added_acft, new_cal, is_dh));
            }
        }

        if !eligible_rots.is_empty() {
            match mode {
                OptimizationMode::ModeAEquityPrimary => {
                    eligible_rots.sort_by_key(|&(r, _, _, _)| rot_minutes[r]);
                    let choice = if eligible_rots.len() > 1 && rng.gen_bool(0.3) {
                        eligible_rots[1]
                    } else {
                        eligible_rots[0]
                    };
                    genome[p_idx] = choice.0;
                    rot_pairings[choice.0].push(p_idx);
                    rot_minutes[choice.0] += choice.1;
                    rot_calendar[choice.0] = choice.2;
                }
                OptimizationMode::ModeBPreferenceAware => {
                    eligible_rots.sort_by(|&a, &b| {
                        let pref_a = &prefs[a.0];
                        let pal_a = data[p_idx]
                            .flight_ids
                            .iter()
                            .filter(|f| pref_a.preferred_flights.contains(f.as_str()))
                            .count() as f64;
                        let vac_a = pref_a
                            .vacation_days
                            .iter()
                            .filter(|&&d| (1..=31).contains(&d) && (data[p_idx].calendar_days & (1 << (d - 1))) != 0)
                            .count() as f64;
                        let score_a = (pal_a * 50.0) - (vac_a * 30.0) - (rot_minutes[a.0] as f64 / 60.0 * 2.0);

                        let pref_b = &prefs[b.0];
                        let pal_b = data[p_idx]
                            .flight_ids
                            .iter()
                            .filter(|f| pref_b.preferred_flights.contains(f.as_str()))
                            .count() as f64;
                        let vac_b = pref_b
                            .vacation_days
                            .iter()
                            .filter(|&&d| (1..=31).contains(&d) && (data[p_idx].calendar_days & (1 << (d - 1))) != 0)
                            .count() as f64;
                        let score_b = (pal_b * 50.0) - (vac_b * 30.0) - (rot_minutes[b.0] as f64 / 60.0 * 2.0);

                        score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
                    });
                    let choice = if eligible_rots.len() > 1 && rng.gen_bool(0.2) {
                        eligible_rots[1]
                    } else {
                        eligible_rots[0]
                    };
                    genome[p_idx] = choice.0;
                    rot_pairings[choice.0].push(p_idx);
                    rot_minutes[choice.0] += choice.1;
                    rot_calendar[choice.0] = choice.2;
                }
                OptimizationMode::ModeCCrossBaseDeadhead => {
                    let home_rots: Vec<_> = eligible_rots
                        .iter()
                        .filter(|&&(r, _, _, is_dh)| !is_dh && crew[r].base == data[p_idx].base)
                        .copied()
                        .collect();
                    let mut candidates = if !home_rots.is_empty() {
                        home_rots
                    } else {
                        eligible_rots
                    };

                    candidates.sort_by(|&a, &b| {
                        let pref_a = &prefs[a.0];
                        let pal_a = data[p_idx]
                            .flight_ids
                            .iter()
                            .filter(|f| pref_a.preferred_flights.contains(f.as_str()))
                            .count() as f64;
                        let vac_a = pref_a
                            .vacation_days
                            .iter()
                            .filter(|&&d| (1..=31).contains(&d) && (data[p_idx].calendar_days & (1 << (d - 1))) != 0)
                            .count() as f64;
                        let home_a = if data[p_idx].base == crew[a.0].base { params.home_base_bonus } else { 0.0 };
                        let dh_pen_a = if a.3 {
                            (params.deadhead_leg_penalty * 3.0) + ((a.1 - data[p_idx].acft_minutes).max(0) as f64 / 60.0) * params.deadhead_credit_hour_penalty
                        } else {
                            0.0
                        };
                        let score_a = home_a + (pal_a * 50.0) - (vac_a * 30.0) - (rot_minutes[a.0] as f64 / 60.0 * 2.0) - dh_pen_a;

                        let pref_b = &prefs[b.0];
                        let pal_b = data[p_idx]
                            .flight_ids
                            .iter()
                            .filter(|f| pref_b.preferred_flights.contains(f.as_str()))
                            .count() as f64;
                        let vac_b = pref_b
                            .vacation_days
                            .iter()
                            .filter(|&&d| (1..=31).contains(&d) && (data[p_idx].calendar_days & (1 << (d - 1))) != 0)
                            .count() as f64;
                        let home_b = if data[p_idx].base == crew[b.0].base { params.home_base_bonus } else { 0.0 };
                        let dh_pen_b = if b.3 {
                            (params.deadhead_leg_penalty * 3.0) + ((b.1 - data[p_idx].acft_minutes).max(0) as f64 / 60.0) * params.deadhead_credit_hour_penalty
                        } else {
                            0.0
                        };
                        let score_b = home_b + (pal_b * 50.0) - (vac_b * 30.0) - (rot_minutes[b.0] as f64 / 60.0 * 2.0) - dh_pen_b;

                        score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
                    });
                    let choice = if candidates.len() > 1 && rng.gen_bool(0.2) {
                        candidates[1]
                    } else {
                        candidates[0]
                    };
                    genome[p_idx] = choice.0;
                    rot_pairings[choice.0].push(p_idx);
                    rot_minutes[choice.0] += choice.1;
                    rot_calendar[choice.0] = choice.2;
                }
            }
        }
    }

    genome
}

// ── Base-Aware Greedy Baseline Scheduler ──────────────────────────────────────

pub fn run_greedy_baseline(
    data: &[PrecomputedPairingData],
    crew: &[RawCrew],
    prefs: &[PilotPreference],
    valid_rotations: &[Vec<usize>],
    n_rot: usize,
    total_flights: usize,
    _conflict_matrix: &ConflictMatrix,
    mode: OptimizationMode,
    graph: &AirportTransitionGraph,
    params: &OptimizationParameters,
) -> (f64, Vec<usize>, u128) {
    let t_start = std::time::Instant::now();
    let n_pairings = data.len();
    let mut genome = vec![n_rot; n_pairings];

    let mut assigned_pairings = HashSet::new();
    let mut rot_pairings: Vec<Vec<usize>> = vec![Vec::new(); n_rot];
    let mut rot_minutes = vec![0i64; n_rot];
    let mut rot_calendar = vec![0u32; n_rot];

    let mut sorted_pairing_indices: Vec<usize> = (0..n_pairings).collect();
    sorted_pairing_indices.sort_by_key(|&i| {
        (valid_rotations[i].len(), data[i].start)
    });

    for (rot_idx, c) in crew.iter().take(n_rot).enumerate() {
        if !rot_pairings[rot_idx].is_empty() {
            continue;
        }
        for &p_idx in &sorted_pairing_indices {
            if !assigned_pairings.contains(&p_idx)
                && data[p_idx].base == c.base
                && data[p_idx].acft_minutes <= 85 * 60
                && !violates_working_rules(data[p_idx].calendar_days)
                && !violates_preassigned(
                    &prefs[rot_idx].preassigned_days_off,
                    data[p_idx].calendar_days,
                )
            {
                genome[p_idx] = rot_idx;
                assigned_pairings.insert(p_idx);
                rot_pairings[rot_idx].push(p_idx);
                rot_minutes[rot_idx] += data[p_idx].acft_minutes;
                rot_calendar[rot_idx] |= data[p_idx].calendar_days;
                break;
            }
        }
    }

    for p_idx in sorted_pairing_indices {
        if assigned_pairings.contains(&p_idx) {
            continue;
        }

        let mut eligible_rots: Vec<(usize, i64, u32, bool)> = Vec::new();
        for &r in &valid_rotations[p_idx] {
            if r >= n_rot {
                continue;
            }
            if let Some((added_acft, new_cal, is_dh)) = evaluate_assignment_feasibility(
                p_idx,
                r,
                data,
                crew,
                prefs,
                mode,
                &rot_pairings[r],
                rot_minutes[r],
                rot_calendar[r],
                graph,
            ) {
                eligible_rots.push((r, added_acft, new_cal, is_dh));
            }
        }

        if !eligible_rots.is_empty() {
            match mode {
                OptimizationMode::ModeAEquityPrimary => {
                    eligible_rots.sort_by_key(|&(r, added_acft, _, _)| rot_minutes[r] + added_acft);
                    let (best_r, added_acft, new_cal, _) = eligible_rots[0];
                    genome[p_idx] = best_r;
                    assigned_pairings.insert(p_idx);
                    rot_pairings[best_r].push(p_idx);
                    rot_minutes[best_r] += added_acft;
                    rot_calendar[best_r] = new_cal;
                }
                OptimizationMode::ModeBPreferenceAware => {
                    eligible_rots.sort_by(|&a, &b| {
                        let pref_a = &prefs[a.0];
                        let pal_a = data[p_idx]
                            .flight_ids
                            .iter()
                            .filter(|f| pref_a.preferred_flights.contains(f.as_str()))
                            .count() as f64;
                        let vac_a = pref_a
                            .vacation_days
                            .iter()
                            .filter(|&&d| (1..=31).contains(&d) && (data[p_idx].calendar_days & (1 << (d - 1))) != 0)
                            .count() as f64;
                        let score_a = (pal_a * 50.0) - (vac_a * 30.0) - (rot_minutes[a.0] as f64 / 60.0 * 2.0);

                        let pref_b = &prefs[b.0];
                        let pal_b = data[p_idx]
                            .flight_ids
                            .iter()
                            .filter(|f| pref_b.preferred_flights.contains(f.as_str()))
                            .count() as f64;
                        let vac_b = pref_b
                            .vacation_days
                            .iter()
                            .filter(|&&d| (1..=31).contains(&d) && (data[p_idx].calendar_days & (1 << (d - 1))) != 0)
                            .count() as f64;
                        let score_b = (pal_b * 50.0) - (vac_b * 30.0) - (rot_minutes[b.0] as f64 / 60.0 * 2.0);

                        score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
                    });
                    let (best_r, added_acft, new_cal, _) = eligible_rots[0];
                    genome[p_idx] = best_r;
                    assigned_pairings.insert(p_idx);
                    rot_pairings[best_r].push(p_idx);
                    rot_minutes[best_r] += added_acft;
                    rot_calendar[best_r] = new_cal;
                }
                OptimizationMode::ModeCCrossBaseDeadhead => {
                    let home_rots: Vec<_> = eligible_rots
                        .iter()
                        .filter(|&&(r, _, _, is_dh)| !is_dh && crew[r].base == data[p_idx].base)
                        .copied()
                        .collect();
                    let mut candidates = if !home_rots.is_empty() {
                        home_rots
                    } else {
                        eligible_rots
                    };

                    candidates.sort_by(|&a, &b| {
                        let pref_a = &prefs[a.0];
                        let pal_a = data[p_idx]
                            .flight_ids
                            .iter()
                            .filter(|f| pref_a.preferred_flights.contains(f.as_str()))
                            .count() as f64;
                        let vac_a = pref_a
                            .vacation_days
                            .iter()
                            .filter(|&&d| (1..=31).contains(&d) && (data[p_idx].calendar_days & (1 << (d - 1))) != 0)
                            .count() as f64;
                        let home_a = if data[p_idx].base == crew[a.0].base { params.home_base_bonus } else { 0.0 };
                        let dh_pen_a = if a.3 {
                            (params.deadhead_leg_penalty * 3.0) + ((a.1 - data[p_idx].acft_minutes).max(0) as f64 / 60.0) * params.deadhead_credit_hour_penalty
                        } else {
                            0.0
                        };
                        let score_a = home_a + (pal_a * 50.0) - (vac_a * 30.0) - (rot_minutes[a.0] as f64 / 60.0 * 2.0) - dh_pen_a;

                        let pref_b = &prefs[b.0];
                        let pal_b = data[p_idx]
                            .flight_ids
                            .iter()
                            .filter(|f| pref_b.preferred_flights.contains(f.as_str()))
                            .count() as f64;
                        let vac_b = pref_b
                            .vacation_days
                            .iter()
                            .filter(|&&d| (1..=31).contains(&d) && (data[p_idx].calendar_days & (1 << (d - 1))) != 0)
                            .count() as f64;
                        let home_b = if data[p_idx].base == crew[b.0].base { params.home_base_bonus } else { 0.0 };
                        let dh_pen_b = if b.3 {
                            (params.deadhead_leg_penalty * 3.0) + ((b.1 - data[p_idx].acft_minutes).max(0) as f64 / 60.0) * params.deadhead_credit_hour_penalty
                        } else {
                            0.0
                        };
                        let score_b = home_b + (pal_b * 50.0) - (vac_b * 30.0) - (rot_minutes[b.0] as f64 / 60.0 * 2.0) - dh_pen_b;

                        score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
                    });
                    let (best_r, added_acft, new_cal, _) = candidates[0];
                    genome[p_idx] = best_r;
                    assigned_pairings.insert(p_idx);
                    rot_pairings[best_r].push(p_idx);
                    rot_minutes[best_r] += added_acft;
                    rot_calendar[best_r] = new_cal;
                }
            }
        }
    }

    let fit = fitness(
        &genome,
        data,
        crew,
        prefs,
        n_rot,
        total_flights,
        mode,
        graph,
        params,
    );
    let ms = t_start.elapsed().as_millis();
    (fit, genome, ms)
}

// ── Multi-Pass Disjoint Ejection Chain Search ─────────────────────────────────

pub fn rescue_unassigned_ejection_chain(
    genome: &mut [usize],
    data: &[PrecomputedPairingData],
    crew: &[RawCrew],
    prefs: &[PilotPreference],
    valid_rotations: &[Vec<usize>],
    n_rot: usize,
    mode: OptimizationMode,
    graph: &AirportTransitionGraph,
) -> usize {
    if n_rot == 0 || data.is_empty() {
        return 0;
    }

    let mut rot_pairings: Vec<Vec<usize>> = vec![Vec::new(); n_rot];
    let mut rot_minutes: Vec<i64> = vec![0; n_rot];
    let mut rot_calendar: Vec<u32> = vec![0; n_rot];

    for (p_idx, &r) in genome.iter().enumerate() {
        if r < n_rot {
            rot_pairings[r].push(p_idx);
        }
    }

    for r in 0..n_rot {
        rot_pairings[r].sort_by_key(|&p| data[p].start);
        if let Some(eval) = evaluate_rotation_feasibility(&rot_pairings[r], r, data, crew, prefs, mode, graph) {
            rot_minutes[r] = eval.total_acft_minutes;
            rot_calendar[r] = eval.calendar_mask;
        }
    }

    let mut total_rescued = 0;

    for _pass in 0..8 {
        let mut rescued_in_pass = 0;
        let mut unassigned: Vec<usize> = genome
            .iter()
            .enumerate()
            .filter(|&(_, &r)| r == n_rot)
            .map(|(p, _)| p)
            .collect();
        unassigned.sort_by_key(|&p| (data[p].acft_minutes, data[p].start));

        for p_idx in unassigned {
            if genome[p_idx] < n_rot {
                continue;
            }

            let mut cand_r_direct: Vec<usize> = valid_rotations[p_idx]
                .iter()
                .copied()
                .filter(|&r| r < n_rot)
                .collect();
            cand_r_direct.sort_by_key(|&r| {
                let is_home = crew[r].base == data[p_idx].base;
                (if is_home { 0 } else { 1 }, rot_minutes[r])
            });

            let mut inserted = false;
            for &r in &cand_r_direct {
                if let Some((added_acft, new_cal, _)) = evaluate_assignment_feasibility(
                    p_idx,
                    r,
                    data,
                    crew,
                    prefs,
                    mode,
                    &rot_pairings[r],
                    rot_minutes[r],
                    rot_calendar[r],
                    graph,
                ) {
                    genome[p_idx] = r;
                    rot_pairings[r].push(p_idx);
                    rot_pairings[r].sort_by_key(|&p| data[p].start);
                    rot_minutes[r] += added_acft;
                    rot_calendar[r] = new_cal;
                    rescued_in_pass += 1;
                    total_rescued += 1;
                    inserted = true;
                    break;
                }
            }
            if inserted {
                continue;
            }

            // 1-swap ejection chain
            for &r1 in &cand_r_direct {
                for v_pos in 0..rot_pairings[r1].len() {
                    let victim_idx = rot_pairings[r1][v_pos];
                    let mut cand_r1: Vec<usize> = rot_pairings[r1]
                        .iter()
                        .copied()
                        .filter(|&p| p != victim_idx)
                        .collect();
                    cand_r1.push(p_idx);
                    cand_r1.sort_by_key(|&p| data[p].start);

                    if let Some(eval1) = evaluate_rotation_feasibility(&cand_r1, r1, data, crew, prefs, mode, graph) {
                        let mut cand_r2_list: Vec<usize> = valid_rotations[victim_idx]
                            .iter()
                            .copied()
                            .filter(|&r| r < n_rot && r != r1)
                            .collect();
                        cand_r2_list.sort_by_key(|&r| {
                            let is_home = crew[r].base == data[victim_idx].base;
                            (if is_home { 0 } else { 1 }, rot_minutes[r])
                        });

                        for &r2 in cand_r2_list.iter().take(40) {
                            if let Some((added_acft2, new_cal2, _)) = evaluate_assignment_feasibility(
                                victim_idx,
                                r2,
                                data,
                                crew,
                                prefs,
                                mode,
                                &rot_pairings[r2],
                                rot_minutes[r2],
                                rot_calendar[r2],
                                graph,
                            ) {
                                genome[p_idx] = r1;
                                rot_pairings[r1] = cand_r1.clone();
                                rot_minutes[r1] = eval1.total_acft_minutes;
                                rot_calendar[r1] = eval1.calendar_mask;

                                genome[victim_idx] = r2;
                                rot_pairings[r2].push(victim_idx);
                                rot_pairings[r2].sort_by_key(|&p| data[p].start);
                                rot_minutes[r2] += added_acft2;
                                rot_calendar[r2] = new_cal2;

                                rescued_in_pass += 1;
                                total_rescued += 1;
                                break;
                            }
                        }
                    }
                }
            }
        }

        if rescued_in_pass == 0 {
            break;
        }
    }

    total_rescued
}

// ── Duty Rebundling at Base Touches ───────────────────────────────────────────

pub fn rescue_unassigned_flights_via_duty_rebundling(
    genome: &[usize],
    pairing_data: &[PrecomputedPairingData],
    duties_map: &HashMap<String, PrecomputedDutyData>,
    crew: &[RawCrew],
    prefs: &[PilotPreference],
    n_rot: usize,
    total_flights: usize,
    mode: OptimizationMode,
    graph: &AirportTransitionGraph,
) -> (usize, usize, f64) {
    let mut covered_flights_set = HashSet::new();
    let mut rot_intervals: Vec<Vec<(DateTime<Utc>, DateTime<Utc>)>> = vec![Vec::new(); n_rot];
    let mut rot_minutes = vec![0i64; n_rot];
    let mut rot_calendar = vec![0u32; n_rot];
    let mut rot_pairings: Vec<Vec<usize>> = vec![Vec::new(); n_rot];

    for (p, &r) in genome.iter().enumerate() {
        if r < n_rot {
            rot_pairings[r].push(p);
            for fid in &pairing_data[p].flight_ids {
                covered_flights_set.insert(fid.clone());
            }
        }
    }

    for r in 0..n_rot {
        rot_pairings[r].sort_by_key(|&p| pairing_data[p].start);
        if let Some(eval) = evaluate_rotation_feasibility(&rot_pairings[r], r, pairing_data, crew, prefs, mode, graph) {
            rot_minutes[r] = eval.total_acft_minutes;
            rot_calendar[r] = eval.calendar_mask;
            for &p in &rot_pairings[r] {
                rot_intervals[r].push((pairing_data[p].start, pairing_data[p].end));
            }
        }
    }

    let min_rest = Duration::minutes(MIN_REST_MINUTES);

    let unassigned_pairings: Vec<usize> = genome
        .iter()
        .enumerate()
        .filter(|&(_, &r)| r >= n_rot)
        .map(|(p, _)| p)
        .collect();

    for p_idx in unassigned_pairings {
        let p = &pairing_data[p_idx];
        let home_base = &p.base;
        let mut current_duties: Vec<&PrecomputedDutyData> = Vec::new();

        for did in &p.duty_ids {
            if let Some(d) = duties_map.get(did) {
                current_duties.push(d);
                let ends_at_base = d.destination == *home_base;
                if ends_at_base {
                    let sub_start = current_duties[0].start;
                    let sub_end = current_duties.last().unwrap().end;
                    let sub_mins: i64 = current_duties.iter().map(|cd| cd.minutes).sum();
                    let sub_acft: i64 = sub_mins + (current_duties.len() as i64) * 75;
                    let mut sub_cal = 0u32;
                    let mut sub_fids = Vec::new();
                    for cd in &current_duties {
                        sub_cal |= cd.calendar_days;
                        sub_fids.extend(cd.flight_ids.clone());
                    }

                    let mut cand_pilots: Vec<usize> = (0..n_rot)
                        .filter(|&r| crew[r].base == *home_base)
                        .collect();
                    cand_pilots.sort_by_key(|&r| rot_minutes[r]);

                    for &r in &cand_pilots {
                        if rot_minutes[r] + sub_acft > 85 * 60 {
                            continue;
                        }

                        let mut rest_conflict = false;
                        for &(s, e) in &rot_intervals[r] {
                            if !(sub_start >= e + min_rest || sub_end + min_rest <= s) {
                                rest_conflict = true;
                                break;
                            }
                        }
                        if rest_conflict {
                            continue;
                        }

                        let new_cal = rot_calendar[r] | sub_cal;
                        if 31 - new_cal.count_ones() < MIN_DAYS_OFF as u32 {
                            continue;
                        }

                        let mut max_c = 0;
                        let mut cur_c = 0;
                        for day in 0..31 {
                            if (new_cal & (1u32 << day)) != 0 {
                                cur_c += 1;
                                if cur_c > max_c {
                                    max_c = cur_c;
                                }
                            } else {
                                cur_c = 0;
                            }
                        }
                        if max_c > MAX_CONSEC_WORK_DAYS {
                            continue;
                        }

                        if violates_preassigned(&prefs[r].preassigned_days_off, new_cal) {
                            continue;
                        }

                        // Success: assign decomposed sub-pairing
                        rot_minutes[r] += sub_acft;
                        rot_calendar[r] = new_cal;
                        rot_intervals[r].push((sub_start, sub_end));

                        for fid in &sub_fids {
                            covered_flights_set.insert(fid.clone());
                        }
                        break;
                    }

                    current_duties.clear();
                }
            }
        }
    }

    let covered_flights = covered_flights_set.len();
    let uncovered_flights = total_flights.saturating_sub(covered_flights);
    let uncovered_flights_pct = if total_flights > 0 {
        uncovered_flights as f64 / total_flights as f64 * 100.0
    } else {
        0.0
    };

    (covered_flights, uncovered_flights, uncovered_flights_pct)
}

// ── Workload Equity Polish ───────────────────────────────────────────────────

pub fn rebalance_equity_polish(
    genome: &mut [usize],
    data: &[PrecomputedPairingData],
    crew: &[RawCrew],
    prefs: &[PilotPreference],
    n_rot: usize,
    _conflict_matrix: &ConflictMatrix,
    mode: OptimizationMode,
    graph: &AirportTransitionGraph,
) -> usize {
    if n_rot <= 1 || data.is_empty() {
        return 0;
    }

    let mut rot_pairings: Vec<Vec<usize>> = vec![Vec::new(); n_rot];
    let mut rot_minutes: Vec<i64> = vec![0; n_rot];

    for (p_idx, &r) in genome.iter().enumerate() {
        if r < n_rot {
            rot_pairings[r].push(p_idx);
        }
    }

    for r in 0..n_rot {
        if let Some(eval) = evaluate_rotation_feasibility(&rot_pairings[r], r, data, crew, prefs, mode, graph) {
            rot_minutes[r] = eval.total_acft_minutes;
        }
    }

    let total_mins: i64 = rot_minutes.iter().sum();
    let mean_mins = total_mins as f64 / n_rot as f64;
    let mut total_moves = 0;

    for _pass in 0..20 {
        let mut improved_in_pass = false;

        let mut overloaded: Vec<usize> = (0..n_rot)
            .filter(|&r| (rot_minutes[r] as f64) > mean_mins + 5.0)
            .collect();
        overloaded.sort_by_key(|&r| -rot_minutes[r]);

        let mut underloaded: Vec<usize> = (0..n_rot)
            .filter(|&r| (rot_minutes[r] as f64) < mean_mins - 5.0)
            .collect();
        underloaded.sort_by_key(|&r| rot_minutes[r]);

        for &r_over in &overloaded {
            if (rot_minutes[r_over] as f64) <= mean_mins {
                continue;
            }
            let h_over = rot_minutes[r_over] as f64;

            for &r_under in &underloaded {
                if (rot_minutes[r_under] as f64) >= mean_mins {
                    continue;
                }
                if crew[r_over].base != crew[r_under].base {
                    continue;
                }
                let h_under = rot_minutes[r_under] as f64;
                if h_over <= h_under {
                    continue;
                }

                let cur_sq = (h_over - mean_mins).powi(2) + (h_under - mean_mins).powi(2);
                let mut best_move: Option<(usize, Option<usize>, i64, i64)> = None;
                let mut best_delta_sq = 0.0_f64;

                let pairs_over = &rot_pairings[r_over];
                let pairs_under = &rot_pairings[r_under];

                // 1. Single transfer: move p from r_over to r_under
                for &p in pairs_over {
                    let mut cand_under = pairs_under.clone();
                    cand_under.push(p);
                    if let Some(eval_u) = evaluate_rotation_feasibility(&cand_under, r_under, data, crew, prefs, mode, graph) {
                        let cand_over: Vec<usize> = pairs_over.iter().copied().filter(|&x| x != p).collect();
                        if cand_over.is_empty() {
                            continue;
                        }
                        if let Some(eval_o) = evaluate_rotation_feasibility(&cand_over, r_over, data, crew, prefs, mode, graph) {
                            let new_sq = (eval_o.total_acft_minutes as f64 - mean_mins).powi(2)
                                + (eval_u.total_acft_minutes as f64 - mean_mins).powi(2);
                            let delta = cur_sq - new_sq;
                            if delta > best_delta_sq {
                                best_delta_sq = delta;
                                best_move = Some((p, None, eval_o.total_acft_minutes, eval_u.total_acft_minutes));
                            }
                        }
                    }
                }

                // 2. 1-for-1 Swap: swap p_o with p_u
                for &p_o in pairs_over {
                    for &p_u in pairs_under {
                        if data[p_o].acft_minutes <= data[p_u].acft_minutes {
                            continue;
                        }
                        let mut cand_over: Vec<usize> = pairs_over.iter().copied().filter(|&x| x != p_o).collect();
                        cand_over.push(p_u);
                        let mut cand_under: Vec<usize> = pairs_under.iter().copied().filter(|&x| x != p_u).collect();
                        cand_under.push(p_o);

                        if let Some(eval_o) = evaluate_rotation_feasibility(&cand_over, r_over, data, crew, prefs, mode, graph) {
                            if let Some(eval_u) = evaluate_rotation_feasibility(&cand_under, r_under, data, crew, prefs, mode, graph) {
                                let new_sq = (eval_o.total_acft_minutes as f64 - mean_mins).powi(2)
                                    + (eval_u.total_acft_minutes as f64 - mean_mins).powi(2);
                                let delta = cur_sq - new_sq;
                                if delta > best_delta_sq {
                                    best_delta_sq = delta;
                                    best_move = Some((p_o, Some(p_u), eval_o.total_acft_minutes, eval_u.total_acft_minutes));
                                }
                            }
                        }
                    }
                }

                if let Some((p_o, p_u_opt, mins_o, mins_u)) = best_move {
                    if let Some(p_u) = p_u_opt {
                        genome[p_o] = r_under;
                        genome[p_u] = r_over;
                        rot_pairings[r_over].retain(|&x| x != p_o);
                        rot_pairings[r_over].push(p_u);
                        rot_pairings[r_under].retain(|&x| x != p_u);
                        rot_pairings[r_under].push(p_o);
                    } else {
                        genome[p_o] = r_under;
                        rot_pairings[r_over].retain(|&x| x != p_o);
                        rot_pairings[r_under].push(p_o);
                    }
                    rot_minutes[r_over] = mins_o;
                    rot_minutes[r_under] = mins_u;
                    improved_in_pass = true;
                    total_moves += 1;
                    break;
                }
            }
        }

        if !improved_in_pass {
            break;
        }
    }

    total_moves
}

// ── Evolutionary Algorithm Core ───────────────────────────────────────────────

pub fn run_evolutionary(
    data: &[PrecomputedPairingData],
    crew: &[RawCrew],
    prefs: &[PilotPreference],
    valid_rotations: &[Vec<usize>],
    n_rot: usize,
    total_flights: usize,
    greedy_seed: &[usize],
    conflict_matrix: &ConflictMatrix,
    mode: OptimizationMode,
    rng: &mut impl Rng,
    graph: &AirportTransitionGraph,
    params: &OptimizationParameters,
) -> (f64, Vec<usize>, usize, u128) {
    let n_pairings = data.len();
    if n_pairings == 0 || n_rot == 0 {
        return (0.0, vec![], 0, 0);
    }

    let t_start = std::time::Instant::now();
    let mut population: Vec<Vec<usize>> = Vec::with_capacity(params.pop_size);

    if !greedy_seed.is_empty() {
        let mut seed_ind = greedy_seed.to_vec();
        rescue_unassigned_ejection_chain(
            &mut seed_ind,
            data,
            crew,
            prefs,
            valid_rotations,
            n_rot,
            mode,
            graph,
        );
        population.push(seed_ind);
    }

    while population.len() < params.pop_size {
        let ind = generate_random_greedy(
            data,
            crew,
            prefs,
            valid_rotations,
            n_rot,
            conflict_matrix,
            mode,
            rng,
            graph,
            params,
        );
        population.push(ind);
    }

    let mut fitnesses: Vec<f64> = population
        .par_iter()
        .map(|g| fitness(g, data, crew, prefs, n_rot, total_flights, mode, graph, params))
        .collect();

    let mut best_score = fitnesses.iter().cloned().fold(f64::INFINITY, f64::min);
    let mut best_gen = 0usize;
    let b_idx = fitnesses
        .iter()
        .position(|&s| (s - best_score).abs() < 1e-9)
        .unwrap_or(0);
    let mut best_genome = population[b_idx].clone();

    for generation in 0..params.generations {
        let mut next_pop: Vec<Vec<usize>> = Vec::with_capacity(params.pop_size);

        while next_pop.len() < params.pop_size {
            let pa_idx = (0..params.tournament_k)
                .map(|_| rng.gen_range(0..params.pop_size))
                .min_by(|&a, &b| fitnesses[a].partial_cmp(&fitnesses[b]).unwrap())
                .unwrap();
            let pb_idx = (0..params.tournament_k)
                .map(|_| rng.gen_range(0..params.pop_size))
                .min_by(|&a, &b| fitnesses[a].partial_cmp(&fitnesses[b]).unwrap())
                .unwrap();

            let mut child = if rng.r#gen::<f64>() < params.crossover_prob {
                let point = rng.gen_range(1..n_pairings);
                let mut c = population[pa_idx][..point].to_vec();
                c.extend_from_slice(&population[pb_idx][point..]);
                c
            } else {
                population[pa_idx].clone()
            };

            if rng.r#gen::<f64>() < 0.40 {
                let n_mut = rng.gen_range(1..=3);
                for _ in 0..n_mut {
                    let p1 = rng.gen_range(0..n_pairings);
                    let valid = &valid_rotations[p1];
                    let home_count = valid
                        .iter()
                        .take_while(|&&r| r < n_rot && crew[r].base == data[p1].base)
                        .count();
                    child[p1] = if home_count > 0 && rng.r#gen::<f64>() < 0.95 {
                        *valid[..home_count].choose(rng).unwrap()
                    } else {
                        *valid.choose(rng).unwrap()
                    };
                }
            }

            repair(
                &mut child,
                data,
                crew,
                prefs,
                valid_rotations,
                n_rot,
                conflict_matrix,
                mode,
                rng,
                graph,
                params,
            );
            next_pop.push(child);
        }

        let next_fitnesses: Vec<f64> = next_pop
            .par_iter()
            .map(|g| fitness(g, data, crew, prefs, n_rot, total_flights, mode, graph, params))
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

        let prev_best_genome = population[prev_best_idx].clone();
        let prev_best_fitness = fitnesses[prev_best_idx];

        population = next_pop;
        fitnesses = next_fitnesses;
        population[new_worst_idx] = prev_best_genome;
        fitnesses[new_worst_idx] = prev_best_fitness;

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

        if (generation + 1) % 25 == 0 {
            rescue_unassigned_ejection_chain(
                &mut best_genome,
                data,
                crew,
                prefs,
                valid_rotations,
                n_rot,
                mode,
                graph,
            );
            best_score = fitness(
                &best_genome,
                data,
                crew,
                prefs,
                n_rot,
                total_flights,
                mode,
                graph,
                params,
            );
        }
    }

    // Final multi-pass ejection chain rescue
    rescue_unassigned_ejection_chain(
        &mut best_genome,
        data,
        crew,
        prefs,
        valid_rotations,
        n_rot,
        mode,
        graph,
    );

    // Final equity rebalancing polish
    rebalance_equity_polish(
        &mut best_genome,
        data,
        crew,
        prefs,
        n_rot,
        conflict_matrix,
        mode,
        graph,
    );

    let final_fitness = fitness(
        &best_genome,
        data,
        crew,
        prefs,
        n_rot,
        total_flights,
        mode,
        graph,
        params,
    );
    let ms = t_start.elapsed().as_millis();
    (final_fitness, best_genome, best_gen, ms)
}

// ── Operational KPIs Evaluator ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationalKpis {
    pub total_flights: usize,
    pub covered_flights: usize,
    pub uncovered_flights: usize,
    pub uncovered_flights_pct: f64,
    pub total_pairings_csv: usize,
    pub loaded_pairings: usize,
    pub assigned_pairings: usize,
    pub unassigned_pairings: usize,
    pub uncovered_pairings_pct: f64,
    pub raw_mean_hours: f64,
    pub raw_min_hours: f64,
    pub raw_max_hours: f64,
    pub raw_std_hours: f64,
    pub acft_mean_hours: f64,
    pub acft_min_hours: f64,
    pub acft_max_hours: f64,
    pub acft_std_hours: f64,
    pub min_days_off: usize,
    pub max_consec_work_days: usize,
    pub total_preassigned_days: usize,
    pub preassigned_violations: usize,
    pub zero_hour_pilots_count: usize,
    pub total_pal_requested: usize,
    pub total_pal_granted: usize,
    pub pal_granted_pct: f64,
    pub total_vac_requested: usize,
    pub total_vac_granted: usize,
    pub vac_granted_pct: f64,
    pub preference_score_s: f64,
    pub base_mismatches: usize,
    pub deadhead_pairings_count: usize,
    pub deadhead_credit_hours: f64,
    pub rest_violations: usize,
    pub consec_work_violations: usize,
    pub days_off_violations: usize,
    pub overtime_violations: usize,
}

pub fn compute_operational_kpis(
    genome: &[usize],
    data: &[PrecomputedPairingData],
    crew: &[RawCrew],
    prefs: &[PilotPreference],
    flights_map: &HashMap<String, FlightLeg>,
    total_csv_pairings: usize,
    mode: OptimizationMode,
    graph: &AirportTransitionGraph,
) -> OperationalKpis {
    let n_rot = crew.len().min(data.len());
    let total_flights = flights_map.len();
    let loaded_pairings = data.len();

    let mut covered_flights_set = HashSet::new();
    let mut assigned_pairings = 0usize;
    let mut unassigned_pairings = 0usize;
    let mut raw_crew_minutes: Vec<i64> = vec![0; n_rot];
    let mut acft_crew_minutes: Vec<i64> = vec![0; n_rot];
    let mut rot_pairing_indices: Vec<Vec<usize>> = vec![Vec::new(); n_rot];
    let mut rot_calendar: Vec<u32> = vec![0; n_rot];
    let mut base_mismatches = 0usize;
    let mut deadhead_pairings_count = 0usize;
    let mut total_deadhead_credit_minutes = 0i64;

    for (i, &rot_idx) in genome.iter().enumerate() {
        if rot_idx < n_rot {
            assigned_pairings += 1;
            rot_pairing_indices[rot_idx].push(i);
            for fid in &data[i].flight_ids {
                covered_flights_set.insert(fid.clone());
            }
        } else {
            unassigned_pairings += 1;
        }
    }

    let mut rest_violations = 0usize;
    let mut consec_work_violations = 0usize;
    let mut days_off_violations = 0usize;
    let mut min_days_off = usize::MAX;
    let mut max_consec_work_days = 0usize;

    for (rot_idx, indices) in rot_pairing_indices.iter().enumerate() {
        if indices.is_empty() {
            continue;
        }
        match evaluate_rotation_feasibility(indices, rot_idx, data, crew, prefs, mode, graph) {
            Some(eval) => {
                acft_crew_minutes[rot_idx] = eval.total_acft_minutes;
                raw_crew_minutes[rot_idx] = eval.raw_block_minutes;
                rot_calendar[rot_idx] = eval.calendar_mask;
                deadhead_pairings_count += eval.deadhead_count;
                total_deadhead_credit_minutes += eval.deadhead_credit_minutes;
            }
            None => {
                for &idx in indices {
                    if data[idx].base != crew[rot_idx].base {
                        base_mismatches += 1;
                    }
                    acft_crew_minutes[rot_idx] += data[idx].acft_minutes;
                    raw_crew_minutes[rot_idx] += data[idx].minutes;
                    rot_calendar[rot_idx] |= data[idx].calendar_days;
                }
                rest_violations += 1;
            }
        }

        let cal = rot_calendar[rot_idx];
        let work_days = cal.count_ones() as usize;
        let days_off = 31 - work_days;
        if days_off < min_days_off {
            min_days_off = days_off;
        }
        if days_off < MIN_DAYS_OFF {
            days_off_violations += 1;
        }

        let mut cur_consec = 0usize;
        let mut max_c = 0usize;
        for d in 0..31 {
            if (cal & (1u32 << d)) != 0 {
                cur_consec += 1;
                if cur_consec > max_c {
                    max_c = cur_consec;
                }
            } else {
                cur_consec = 0;
            }
        }
        if max_c > max_consec_work_days {
            max_consec_work_days = max_c;
        }
        if max_c > MAX_CONSEC_WORK_DAYS {
            consec_work_violations += 1;
        }
    }

    if min_days_off == usize::MAX {
        min_days_off = 31;
    }

    let mut total_preassigned_days = 0usize;
    let mut preassigned_violations = 0usize;
    for r in 0..n_rot {
        total_preassigned_days += prefs[r].preassigned_days_off.len();
        if violates_preassigned(&prefs[r].preassigned_days_off, rot_calendar[r]) {
            preassigned_violations += 1;
        }
    }
    let zero_hour_pilots_count = acft_crew_minutes.iter().filter(|&&m| m == 0).count();

    let covered_flights = covered_flights_set.len();
    let uncovered_flights = total_flights.saturating_sub(covered_flights);
    let uncovered_flights_pct = if total_flights > 0 {
        uncovered_flights as f64 / total_flights as f64 * 100.0
    } else {
        0.0
    };

    let uncovered_pairings_pct = if total_csv_pairings > 0 {
        unassigned_pairings as f64 / total_csv_pairings as f64 * 100.0
    } else {
        0.0
    };

    let raw_hours: Vec<f64> = raw_crew_minutes.iter().map(|&m| m as f64 / 60.0).collect();
    let raw_mean_hours = if !raw_hours.is_empty() {
        raw_hours.iter().sum::<f64>() / raw_hours.len() as f64
    } else {
        0.0
    };
    let raw_min_hours = raw_hours.iter().cloned().fold(f64::INFINITY, f64::min);
    let raw_max_hours = raw_hours.iter().cloned().fold(0.0_f64, f64::max);
    let raw_variance = if !raw_hours.is_empty() {
        raw_hours
            .iter()
            .map(|&h| (h - raw_mean_hours).powi(2))
            .sum::<f64>()
            / raw_hours.len() as f64
    } else {
        0.0
    };
    let raw_std_hours = raw_variance.sqrt();

    let acft_hours: Vec<f64> = acft_crew_minutes.iter().map(|&m| m as f64 / 60.0).collect();
    let acft_mean_hours = if !acft_hours.is_empty() {
        acft_hours.iter().sum::<f64>() / acft_hours.len() as f64
    } else {
        0.0
    };
    let acft_min_hours = acft_hours.iter().cloned().fold(f64::INFINITY, f64::min);
    let acft_max_hours = acft_hours.iter().cloned().fold(0.0_f64, f64::max);
    let acft_variance = if !acft_hours.is_empty() {
        acft_hours
            .iter()
            .map(|&h| (h - acft_mean_hours).powi(2))
            .sum::<f64>()
            / acft_hours.len() as f64
    } else {
        0.0
    };
    let acft_std_hours = acft_variance.sqrt();
    let overtime_violations = acft_hours.iter().filter(|&&h| h > 85.0 + 1e-4).count();

    let pref_kpi = evaluate_preferences(genome, data, prefs, n_rot, uncovered_flights);
    let deadhead_credit_hours = total_deadhead_credit_minutes as f64 / 60.0;

    OperationalKpis {
        total_flights,
        covered_flights,
        uncovered_flights,
        uncovered_flights_pct,
        total_pairings_csv: total_csv_pairings,
        loaded_pairings,
        assigned_pairings,
        unassigned_pairings,
        uncovered_pairings_pct,
        raw_mean_hours,
        raw_min_hours,
        raw_max_hours,
        raw_std_hours,
        acft_mean_hours,
        acft_min_hours,
        acft_max_hours,
        acft_std_hours,
        min_days_off,
        max_consec_work_days,
        total_preassigned_days,
        preassigned_violations,
        zero_hour_pilots_count,
        total_pal_requested: pref_kpi.total_pal_requested,
        total_pal_granted: pref_kpi.total_pal_granted,
        pal_granted_pct: pref_kpi.pal_granted_pct,
        total_vac_requested: pref_kpi.total_vac_requested,
        total_vac_granted: pref_kpi.total_vac_granted,
        vac_granted_pct: pref_kpi.vac_granted_pct,
        preference_score_s: pref_kpi.preference_score_s,
        base_mismatches,
        deadhead_pairings_count,
        deadhead_credit_hours,
        rest_violations,
        consec_work_violations,
        days_off_violations,
        overtime_violations,
    }
}

// ── File Loaders & Parsers ───────────────────────────────────────────────────

pub fn parse_utc(s: &str) -> DateTime<Utc> {
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

pub fn pad_airport(s: &str) -> String {
    if s.len() >= 3 {
        s[..3].to_uppercase()
    } else {
        format!("{:A<3}", s.to_uppercase())
    }
}

pub fn load_flights(path: &Path) -> HashMap<String, FlightLeg> {
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

pub fn load_scheduled_flights(path: &Path) -> (HashMap<String, ScheduledFlight>, Vec<ScheduledFlight>) {
    let mut rdr = csv::Reader::from_path(path).expect("Cannot open flights.csv");
    let mut map = HashMap::new();
    let mut vec = Vec::new();
    for result in rdr.deserialize::<HashMap<String, String>>() {
        let row = result.expect("CSV parse error in flights.csv");
        let fid = row["flight_id"].trim().to_string();
        let orig = row["origin"].trim().to_string();
        let dest = row["destination"].trim().to_string();
        let dep = parse_utc(row["departure_utc"].trim());
        let arr = parse_utc(row["arrival_utc"].trim());
        let block_minutes: i64 = row
            .get("block_minutes")
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(|| (arr - dep).num_minutes());

        let sf = ScheduledFlight {
            flight_id: fid.clone(),
            origin: orig,
            destination: dest,
            departure: dep,
            arrival: arr,
            block_minutes,
        };
        map.insert(fid, sf.clone());
        vec.push(sf);
    }
    (map, vec)
}

pub fn load_crew(path: &Path) -> Vec<RawCrew> {
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

pub fn load_benchmark_pairing_data(
    instance_dir: &Path,
    flights_map: &HashMap<String, FlightLeg>,
    scheduled_flights_map: &HashMap<String, ScheduledFlight>,
) -> (Vec<PrecomputedPairingData>, usize) {
    let duties_path = instance_dir.join("duties.csv");
    let pairings_path = instance_dir.join("pairings.csv");

    let mut rdr_d = csv::Reader::from_path(&duties_path).expect("Cannot open duties.csv");
    let mut duties_map: HashMap<String, (i64, DateTime<Utc>, DateTime<Utc>, u32, Vec<String>, String, String)> =
        HashMap::new();
    for result in rdr_d.deserialize::<HashMap<String, String>>() {
        let row = result.expect("CSV parse error in duties.csv");
        let did = row["duty_id"].trim().to_string();
        let fids_raw = row["flight_ids"].trim();
        let fids: Vec<String> = fids_raw
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let mut duty_legs: Vec<&FlightLeg> = Vec::new();
        let mut d_days = 0u32;
        for fid in &fids {
            if let Some(leg) = flights_map.get(fid) {
                duty_legs.push(leg);
                let dep_day = leg.scheduled_departure.date_naive().day() as usize;
                let arr_day = leg.scheduled_arrival.date_naive().day() as usize;
                for d in dep_day..=arr_day {
                    if (1..=31).contains(&d) {
                        d_days |= 1u32 << (d - 1);
                    }
                }
            }
        }
        if !duty_legs.is_empty() {
            let start = duty_legs
                .iter()
                .map(|l| l.scheduled_departure)
                .min()
                .unwrap();
            let end = duty_legs
                .iter()
                .map(|l| l.scheduled_arrival)
                .max()
                .unwrap();
            let minutes: i64 = duty_legs
                .iter()
                .map(|l| (l.scheduled_arrival - l.scheduled_departure).num_minutes())
                .sum();
            let duty_orig = scheduled_flights_map
                .get(&fids[0])
                .map(|f| f.origin.clone())
                .unwrap_or_default();
            let duty_dest = scheduled_flights_map
                .get(&fids[fids.len() - 1])
                .map(|f| f.destination.clone())
                .unwrap_or_default();
            duties_map.insert(did, (minutes, start, end, d_days, fids, duty_orig, duty_dest));
        }
    }

    let mut rdr_p = csv::Reader::from_path(&pairings_path).expect("Cannot open pairings.csv");
    let mut pairing_data = Vec::new();
    let mut total_csv_pairings = 0usize;
    for result in rdr_p.deserialize::<HashMap<String, String>>() {
        total_csv_pairings += 1;
        let row = result.expect("CSV parse error in pairings.csv");
        let pid = row["pairing_id"].trim().to_string();
        let raw_base = row["home_base"].trim().to_string();
        let dids_raw = row["duty_ids"].trim();
        let dids: Vec<&str> = dids_raw.split(',').map(|s| s.trim()).collect();

        let mut minutes = 0i64;
        let mut start: Option<DateTime<Utc>> = None;
        let mut end: Option<DateTime<Utc>> = None;
        let mut calendar_days = 0u32;
        let mut flight_ids = Vec::new();

        for did in &dids {
            if let Some(&(dmins, dstart, dend, ddays, ref dfids, _, _)) = duties_map.get(*did) {
                minutes += dmins;
                start = Some(start.map_or(dstart, |s| s.min(dstart)));
                end = Some(end.map_or(dend, |e| e.max(dend)));
                calendar_days |= ddays;
                flight_ids.extend(dfids.clone());
            }
        }

        if let (Some(s), Some(e)) = (start, end) {
            let acft_minutes = minutes + (dids.len() as i64) * 75;
            let pairing_orig = if !dids.is_empty() {
                duties_map.get(dids[0]).map(|d| d.5.clone()).unwrap_or_else(|| raw_base.clone())
            } else {
                raw_base.clone()
            };
            let pairing_dest = if !dids.is_empty() {
                duties_map.get(dids[dids.len() - 1]).map(|d| d.6.clone()).unwrap_or_else(|| raw_base.clone())
            } else {
                raw_base.clone()
            };

            pairing_data.push(PrecomputedPairingData {
                id: pid,
                base: raw_base,
                origin: pairing_orig,
                dest: pairing_dest,
                minutes,
                acft_minutes,
                start: s,
                end: e,
                calendar_days,
                flight_ids,
                duty_ids: dids.iter().map(|s| s.to_string()).collect(),
            });
        }
    }

    (pairing_data, total_csv_pairings)
}

pub fn load_benchmark_duties_map(
    instance_dir: &Path,
    flights_map: &HashMap<String, FlightLeg>,
    scheduled_flights_map: &HashMap<String, ScheduledFlight>,
) -> HashMap<String, PrecomputedDutyData> {
    let duties_path = instance_dir.join("duties.csv");
    let mut rdr = csv::Reader::from_path(&duties_path).expect("Cannot open duties.csv");
    let mut map = HashMap::new();

    for result in rdr.deserialize::<HashMap<String, String>>() {
        let row = result.expect("CSV parse error in duties.csv");
        let did = row["duty_id"].trim().to_string();
        let fids_raw = row["flight_ids"].trim();
        let fids: Vec<String> = fids_raw
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let mut duty_legs: Vec<&FlightLeg> = Vec::new();
        let mut d_days = 0u32;
        for fid in &fids {
            if let Some(leg) = flights_map.get(fid) {
                duty_legs.push(leg);
                let dep_day = leg.scheduled_departure.date_naive().day() as usize;
                let arr_day = leg.scheduled_arrival.date_naive().day() as usize;
                for d in dep_day..=arr_day {
                    if (1..=31).contains(&d) {
                        d_days |= 1u32 << (d - 1);
                    }
                }
            }
        }

        if !duty_legs.is_empty() {
            let start = duty_legs.iter().map(|l| l.scheduled_departure).min().unwrap();
            let end = duty_legs.iter().map(|l| l.scheduled_arrival).max().unwrap();
            let minutes: i64 = duty_legs.iter().map(|l| (l.scheduled_arrival - l.scheduled_departure).num_minutes()).sum();
            let duty_orig = scheduled_flights_map.get(&fids[0]).map(|f| f.origin.clone()).unwrap_or_default();
            let duty_dest = scheduled_flights_map.get(&fids[fids.len() - 1]).map(|f| f.destination.clone()).unwrap_or_default();

            map.insert(did.clone(), PrecomputedDutyData {
                did,
                minutes,
                start,
                end,
                calendar_days: d_days,
                flight_ids: fids,
                origin: duty_orig,
                destination: duty_dest,
            });
        }
    }
    map
}

pub fn load_solution0_preferences(instance_dir: &Path, crew: &[RawCrew]) -> Vec<PilotPreference> {
    let inst_name = instance_dir
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let parent = match instance_dir.parent() {
        Some(p) => p,
        None => return vec![PilotPreference::default(); crew.len()],
    };
    let candidate1 = parent
        .join("raw")
        .join(&inst_name)
        .join(&inst_name)
        .join("solution_0");
    let candidate2 = parent.join("raw").join(&inst_name).join("solution_0");
    let candidate3 = instance_dir.join("solution_0");
    let sol_path = if candidate1.exists() {
        candidate1
    } else if candidate2.exists() {
        candidate2
    } else {
        candidate3
    };

    let mut prefs = vec![PilotPreference::default(); crew.len()];
    if !sol_path.exists() {
        return prefs;
    }

    let content = match std::fs::read_to_string(&sol_path) {
        Ok(c) => c,
        Err(_) => return prefs,
    };

    let mut sol_by_base: HashMap<String, Vec<(HashSet<String>, HashSet<usize>, HashSet<usize>)>> =
        HashMap::new();

    for line in content.split(';') {
        let trimmed = line.trim();
        if !trimmed.starts_with("schedule") {
            continue;
        }
        let parts: Vec<&str> = trimmed.splitn(2, ':').collect();
        if parts.len() < 2 {
            continue;
        }
        let header = parts[0];
        let body = parts[1];

        let base = if let (Some(b_start), Some(b_end)) = (header.find('('), header.find(')')) {
            header[b_start + 1..b_end].trim().to_string()
        } else {
            continue;
        };

        let tokens: Vec<&str> = body.split("--->").map(|s| s.trim()).collect();
        let mut pal_flights = HashSet::new();
        let mut vac_days = HashSet::new();
        let mut ast_days = HashSet::new();

        for (i, &t) in tokens.iter().enumerate() {
            if t.starts_with("PAL_LEG_") {
                pal_flights.insert(t[4..].to_string());
            } else if t.starts_with("AST_VACATION_") {
                let mut p_day = 1usize;
                for prev_t in tokens[..i].iter().rev() {
                    if prev_t.starts_with("LEG_") || prev_t.starts_with("PAL_LEG_") || prev_t.starts_with("TDH_") {
                        if let Some(pos) = prev_t.rfind('_') {
                            let prefix = &prev_t[..pos];
                            if let Some(pos2) = prefix.rfind('_') {
                                if let Ok(d) = prefix[pos2 + 1..].parse::<usize>() {
                                    p_day = d + 1;
                                    break;
                                }
                            }
                        }
                    }
                }
                let mut n_day = 31usize;
                for next_t in tokens[i + 1..].iter() {
                    if next_t.starts_with("LEG_") || next_t.starts_with("PAL_LEG_") || next_t.starts_with("TDH_") {
                        if let Some(pos) = next_t.rfind('_') {
                            let prefix = &next_t[..pos];
                            if let Some(pos2) = prefix.rfind('_') {
                                if let Ok(d) = prefix[pos2 + 1..].parse::<usize>() {
                                    n_day = if d > 1 { d - 1 } else { 1 };
                                    break;
                                }
                            }
                        }
                    }
                }
                for d in p_day..=n_day {
                    if (1..=31).contains(&d) {
                        ast_days.insert(d);
                        vac_days.insert(d);
                    }
                }
            }
        }

        let mut cur_day = 1usize;
        for &t in &tokens {
            if t.starts_with("LEG_") || t.starts_with("PAL_LEG_") || t.starts_with("TDH_") {
                if let Some(pos) = t.rfind('_') {
                    let prefix = &t[..pos];
                    if let Some(pos2) = prefix.rfind('_') {
                        if let Ok(d) = prefix[pos2 + 1..].parse::<usize>() {
                            cur_day = d + 1;
                        }
                    }
                }
            } else if t == "VACATION" {
                if (1..=31).contains(&cur_day) && !ast_days.contains(&cur_day) {
                    vac_days.insert(cur_day);
                }
                cur_day += 1;
            }
        }

        sol_by_base
            .entry(base)
            .or_default()
            .push((pal_flights, vac_days, ast_days));
    }

    let mut base_cursor: HashMap<String, usize> = HashMap::new();
    for (c_idx, c) in crew.iter().enumerate() {
        let cursor = base_cursor.entry(c.base.clone()).or_insert(0);
        if let Some(sols) = sol_by_base.get(&c.base) {
            if *cursor < sols.len() {
                let (ref pals, ref vacs, ref ast) = sols[*cursor];
                prefs[c_idx] = PilotPreference {
                    crew_id: c.crew_id.clone(),
                    base: c.base.clone(),
                    preferred_flights: pals.clone(),
                    vacation_days: vacs.clone(),
                    preassigned_days_off: ast.clone(),
                };
                *cursor += 1;
            }
        }
    }

    prefs
}

// ── Coralys Platform Contract: State, Action, Plugin, & ReasoningEngine ───────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GeradAction {
    ApplyRosterAssignment {
        genome: Vec<usize>,
        mode: OptimizationMode,
        notes: String,
    },
    ApplyEjectionChainRescue {
        rescued_pairings: Vec<usize>,
    },
    ApplyDutyRebundling {
        rescued_flights: usize,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeradState {
    pub instance_name: String,
    pub pairings: Vec<PrecomputedPairingData>,
    pub crew: Vec<RawCrew>,
    pub preferences: Vec<PilotPreference>,
    pub genome: Vec<usize>,
    pub total_flights: usize,
    pub total_csv_pairings: usize,
    pub mode: OptimizationMode,
    pub transition_graph: AirportTransitionGraph,
    pub kpis: Option<OperationalKpis>,
    pub reference: StateReference,
}

pub struct GeradDecisionPlugin {
    pub state: Mutex<GeradState>,
}

impl GeradDecisionPlugin {
    pub fn new(initial_state: GeradState) -> Self {
        Self {
            state: Mutex::new(initial_state),
        }
    }
}

impl DecisionPlugin for GeradDecisionPlugin {
    type State = GeradState;
    type Evaluation = EvaluationResult;

    fn current_state(&self) -> Self::State {
        self.state.lock().unwrap().clone()
    }

    fn evaluate(&self, state: &Self::State) -> Self::Evaluation {
        let n_rot = state.crew.len().min(state.pairings.len());
        let mut hard_violations = Vec::new();
        let soft_violations = Vec::new();

        let mut rot_pairing_indices: Vec<Vec<usize>> = vec![Vec::new(); n_rot];
        for (p, &r) in state.genome.iter().enumerate() {
            if r < n_rot {
                rot_pairing_indices[r].push(p);
            }
        }

        for (r, indices) in rot_pairing_indices.iter().enumerate() {
            if indices.is_empty() {
                continue;
            }
            match evaluate_rotation_feasibility(
                indices,
                r,
                &state.pairings,
                &state.crew,
                &state.preferences,
                state.mode,
                &state.transition_graph,
            ) {
                Some(eval) => {
                    if eval.total_acft_minutes > 85 * 60 {
                        hard_violations.push(Violation {
                            constraint_id: "CAR700_MONTHLY_CAP".to_string(),
                            severity: "ERROR".to_string(),
                            value: Some(eval.total_acft_minutes as f64 / 60.0),
                            expected: "85.0h".to_string(),
                            actual: format!("{:.2}h", eval.total_acft_minutes as f64 / 60.0),
                            description: format!("Pilot {} exceeds 85.0h monthly cap", state.crew[r].crew_id),
                            penalty: 100_000,
                        });
                    }
                }
                None => {
                    hard_violations.push(Violation {
                        constraint_id: "CAR700_REST_OR_BASE_VIOLATION".to_string(),
                        severity: "ERROR".to_string(),
                        value: None,
                        expected: "FEASIBLE_ROTATION".to_string(),
                        actual: "INFEASIBLE".to_string(),
                        description: format!("Pilot {} rotation violates statutory rest or home base match", state.crew[r].crew_id),
                        penalty: 100_000,
                    });
                }
            }
        }

        let assigned_count = state.genome.iter().filter(|&&r| r < n_rot).count();
        let total_pairings = state.pairings.len();
        let pairing_cov_pct = if total_pairings > 0 {
            assigned_count as f64 / total_pairings as f64 * 100.0
        } else {
            100.0
        };

        let mut metrics = HashMap::new();
        metrics.insert("assigned_pairings".to_string(), assigned_count as f64);
        metrics.insert("total_pairings".to_string(), total_pairings as f64);
        metrics.insert("pairing_coverage_pct".to_string(), pairing_cov_pct);
        metrics.insert("hard_violations".to_string(), hard_violations.len() as f64);

        EvaluationResult {
            objectives: vec![pairing_cov_pct, -(hard_violations.len() as f64) * 10_000.0],
            hard_constraint_violations: hard_violations,
            soft_constraint_violations: soft_violations,
            metrics,
        }
    }

    fn simulate(
        &self,
        state: &Self::State,
        proposal: &DecisionProposal,
    ) -> SimulationResult<Self::State> {
        let action: GeradAction = serde_json::from_value(proposal.payload.clone())
            .map_err(|e| format!("Failed to deserialize GeradAction: {}", e))?;

        let mut next_state = state.clone();

        match action {
            GeradAction::ApplyRosterAssignment { genome, mode, .. } => {
                next_state.genome = genome;
                next_state.mode = mode;
            }
            GeradAction::ApplyEjectionChainRescue { rescued_pairings: _ } => {
                // Already recorded in proposal
            }
            GeradAction::ApplyDutyRebundling { rescued_flights: _ } => {
                // Already recorded in proposal
            }
        }

        next_state.reference = StateReference {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            plugin: "gerad_crew_rostering".to_string(),
            metadata: {
                let mut m = HashMap::new();
                m.insert("parent_id".to_string(), state.reference.id.to_string());
                m.insert("confidence".to_string(), format!("{:.4}", proposal.confidence));
                m
            },
        };

        Ok(next_state)
    }

    fn execute(&mut self, proposal: &DecisionProposal) {
        let cur = self.current_state();
        if let Ok(next) = self.simulate(&cur, proposal) {
            *self.state.lock().unwrap() = next;
        }
    }
}

/// Coralys Primary Decision Engine for Crew Rostering.
#[derive(Debug, Clone, Default)]
pub struct CoralysGeradEngine;

impl ReasoningEngine for CoralysGeradEngine {
    type Plugin = GeradDecisionPlugin;
    type Config = OptimizationParameters;

    fn solve(
        &self,
        plugin: &mut Self::Plugin,
        config: &Self::Config,
    ) -> Result<Vec<DecisionProposal>, String> {
        let state = plugin.current_state();
        let n_rot = state.crew.len().min(state.pairings.len());
        if n_rot == 0 || state.pairings.is_empty() {
            return Ok(Vec::new());
        }

        let conflict_matrix = ConflictMatrix::new(&state.pairings);
        let valid_rotations = build_valid_rotations(&state.pairings, &state.crew, n_rot, state.mode);

        let mut rng = StdRng::seed_from_u64(config.seed);

        let (_, greedy_seed, _) = run_greedy_baseline(
            &state.pairings,
            &state.crew,
            &state.preferences,
            &valid_rotations,
            n_rot,
            state.total_flights,
            &conflict_matrix,
            state.mode,
            &state.transition_graph,
            config,
        );

        let (_, ea_genome, _, _) = run_evolutionary(
            &state.pairings,
            &state.crew,
            &state.preferences,
            &valid_rotations,
            n_rot,
            state.total_flights,
            &greedy_seed,
            &conflict_matrix,
            state.mode,
            &mut rng,
            &state.transition_graph,
            config,
        );

        let action = GeradAction::ApplyRosterAssignment {
            genome: ea_genome.clone(),
            mode: state.mode,
            notes: format!("Optimized schedule for {}", state.instance_name),
        };

        let payload = serde_json::to_value(&action)
            .map_err(|e| format!("Failed to serialize proposal action: {}", e))?;

        let proposal = DecisionProposal {
            priority: 1.0,
            estimated_gain: ea_genome.iter().filter(|&&r| r < n_rot).count() as f64,
            affected_resources: state.crew.iter().map(|c| c.crew_id.clone()).collect(),
            violations_resolved: vec!["CAR700_REST_AND_MONTHLY_CAP_AUDITED".to_string()],
            confidence: 1.0,
            payload,
        };

        Ok(vec![proposal])
    }
}

/// High-Level Evaluator Facade for GERAD Crew Rostering Benchmarks and Production Callers.
#[derive(Debug, Clone, Default)]
pub struct GeradCrewRosteringEvaluator;

impl GeradCrewRosteringEvaluator {
    pub fn new() -> Self {
        Self
    }

    pub fn solve_instance(
        &self,
        instance_dir: &Path,
        mode: OptimizationMode,
        config: &OptimizationParameters,
    ) -> Result<(OperationalKpis, DecisionLineage), String> {
        let inst_name = instance_dir
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Instance".to_string());

        let flights_path = instance_dir.join("flights.csv");
        let crew_path = instance_dir.join("crew.csv");

        if !flights_path.exists() || !crew_path.exists() {
            return Err("flights.csv or crew.csv not found".to_string());
        }

        let flights_map = load_flights(&flights_path);
        let (scheduled_flights_map, scheduled_flights) = load_scheduled_flights(&flights_path);
        let transition_graph = AirportTransitionGraph::new(&scheduled_flights);
        let crew = load_crew(&crew_path);
        let (pairing_data, total_csv_pairings) =
            load_benchmark_pairing_data(instance_dir, &flights_map, &scheduled_flights_map);
        let prefs = load_solution0_preferences(instance_dir, &crew);

        let initial_state = GeradState {
            instance_name: inst_name.clone(),
            pairings: pairing_data.clone(),
            crew: crew.clone(),
            preferences: prefs.clone(),
            genome: vec![crew.len().min(pairing_data.len()); pairing_data.len()],
            total_flights: flights_map.len(),
            total_csv_pairings,
            mode,
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
        let initial_eval = plugin.evaluate(&initial_state);

        let root_node = LineageNode {
            id: initial_state.reference.id,
            parent_id: None,
            proposal: None,
            evaluation: initial_eval,
        };
        let mut nodes = HashMap::new();
        nodes.insert(root_node.id, root_node);

        let mut lineage = DecisionLineage {
            nodes,
            root_id: initial_state.reference.id,
            current_id: initial_state.reference.id,
        };

        let engine = CoralysGeradEngine;
        let proposals = engine.solve(&mut plugin, config)?;

        for proposal in proposals {
            let cur_state = plugin.current_state();
            let new_state = plugin.simulate(&cur_state, &proposal)?;
            let eval = plugin.evaluate(&new_state);
            let node_id = new_state.reference.id;
            let parent_id = lineage.current_id;

            plugin.execute(&proposal);

            let node = LineageNode {
                id: node_id,
                parent_id: Some(parent_id),
                proposal: Some(proposal),
                evaluation: eval,
            };
            lineage.nodes.insert(node_id, node);
            lineage.current_id = node_id;
        }

        let final_state = plugin.current_state();
        let kpis = compute_operational_kpis(
            &final_state.genome,
            &pairing_data,
            &crew,
            &prefs,
            &flights_map,
            total_csv_pairings,
            mode,
            &transition_graph,
        );

        Ok((kpis, lineage))
    }
}
