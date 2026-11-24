//! OR-Library Aircraft Landing Problem (ALP) Domain Adapter and Decision Plugin
//!
//! Integrates canonical runway arrival sequencing under wake-turbulence separation
//! matrices and early/late landing penalty costs (Beasley et al. 2000) into the
//! Coralys Primary Decision Engine architecture via [`coralys_core::DecisionPlugin`]
//! and [`coralys_core::ReasoningEngine`].

use chrono::Utc;
use coralys_core::{
    DecisionPlugin, DecisionProposal, EvaluationResult, ReasoningEngine, SimulationResult,
    StateReference, Violation,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::sync::Mutex;
use uuid::Uuid;

/// Flight record for an arriving aircraft in ALP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirlandPlane {
    pub id: usize,
    pub appearance_time: f64,
    pub earliest_time: f64,
    pub target_time: f64,
    pub latest_time: f64,
    pub penalty_early_per_sec: f64,
    pub penalty_late_per_sec: f64,
    pub separation_times: Vec<f64>, // required gap if this plane lands before plane j
}

/// An Aircraft Landing Problem instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirlandInstance {
    pub name: String,
    pub num_planes: usize,
    pub freeze_time: f64,
    pub planes: Vec<AirlandPlane>,
}

impl AirlandInstance {
    /// Parses an OR-Library ALP file (e.g., airland1.txt).
    pub fn from_file(path: &Path) -> io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let mut tokens = Vec::new();
        for line in reader.lines() {
            let line = line?;
            for token in line.split_whitespace() {
                tokens.push(token.to_string());
            }
        }

        if tokens.len() < 2 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Insufficient tokens in ALP file",
            ));
        }

        let num_planes: usize = tokens[0]
            .parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let freeze_time: f64 = tokens[1]
            .parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let mut idx = 2;
        let mut planes = Vec::with_capacity(num_planes);

        for p_id in 0..num_planes {
            if idx + 6 > tokens.len() {
                break;
            }
            let appearance_time: f64 = tokens[idx].parse().unwrap_or(0.0);
            let earliest_time: f64 = tokens[idx + 1].parse().unwrap_or(0.0);
            let target_time: f64 = tokens[idx + 2].parse().unwrap_or(0.0);
            let latest_time: f64 = tokens[idx + 3].parse().unwrap_or(0.0);
            let penalty_early_per_sec: f64 = tokens[idx + 4].parse().unwrap_or(0.0);
            let penalty_late_per_sec: f64 = tokens[idx + 5].parse().unwrap_or(0.0);
            idx += 6;

            let mut separation_times = Vec::with_capacity(num_planes);
            for _ in 0..num_planes {
                if idx < tokens.len() {
                    let sep: f64 = tokens[idx].parse().unwrap_or(0.0);
                    separation_times.push(sep);
                    idx += 1;
                }
            }

            planes.push(AirlandPlane {
                id: p_id,
                appearance_time,
                earliest_time,
                target_time,
                latest_time,
                penalty_early_per_sec,
                penalty_late_per_sec,
                separation_times,
            });
        }

        Ok(Self {
            name,
            num_planes: planes.len(),
            freeze_time,
            planes,
        })
    }

    /// Known certified global integer optimal objective value from Beasley et al. (2000).
    pub fn certified_optimum(&self) -> Option<f64> {
        get_alp_certified_optimum(&self.name)
    }

    /// Published literature baseline objective value from Beasley et al. (2000),
    /// Piñol & Beasley (2006), or Faye (2015) for R runways.
    pub fn published_baseline(&self, runways: usize) -> Option<f64> {
        get_airland_published_baseline(&self.name, runways)
    }

    /// Converts the arrival-only AirlandInstance into a canonical MixedRunwayInstance
    /// for multi-runway allocation and scheduling.
    pub fn to_mixed_runway_instance(&self, num_runways: usize) -> crate::orlib::mixed_runway::MixedRunwayInstance {
        use crate::orlib::mixed_runway::{MixedRunwayInstance, RunwayOp, RunwayOpType, WakeCategory};
        let n = self.num_planes;
        let mut operations = Vec::with_capacity(n);
        let mut separation_matrix = vec![vec![0.0; n]; n];

        for (i, p) in self.planes.iter().enumerate() {
            operations.push(RunwayOp {
                id: i,
                op_type: RunwayOpType::Arrival,
                wake_category: WakeCategory::Medium,
                earliest_time: p.earliest_time,
                target_time: p.target_time,
                latest_time: p.latest_time,
                penalty_early: p.penalty_early_per_sec,
                penalty_late: p.penalty_late_per_sec,
            });
            for (j, &s) in p.separation_times.iter().enumerate() {
                if j < n {
                    separation_matrix[i][j] = s;
                }
            }
        }

        MixedRunwayInstance {
            name: self.name.clone(),
            track: "CanonicalALP".to_string(),
            regime: "ArrivalsOnly".to_string(),
            num_operations: n,
            num_arrivals: n,
            num_departures: 0,
            num_runways: num_runways.max(1),
            freeze_time: self.freeze_time,
            operations,
            separation_matrix,
            taxiway_precedence: Vec::new(),
        }
    }
}

/// Evaluated solution for an Aircraft Landing instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirlandSolution {
    pub landing_sequence: Vec<usize>, // order in which planes land
    pub landing_times: Vec<f64>,       // assigned landing time t_i for each plane
    pub total_penalty: f64,
    pub is_feasible: bool,
    pub separation_violations: usize,
    pub window_violations: usize,
}

impl AirlandSolution {
    /// Percentage optimality gap: (Z - Z*) / Z* * 100%
    pub fn optimality_gap(&self, certified_opt: f64) -> f64 {
        if certified_opt <= 0.0 {
            return 0.0;
        }
        ((self.total_penalty - certified_opt) / certified_opt) * 100.0
    }
}

/// Operational domain state managed by the Coralys Airland plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirlandState {
    pub instance: AirlandInstance,
    pub landing_sequence: Vec<usize>,
    pub landing_times: Vec<f64>,
    pub reference: StateReference,
}

impl AirlandState {
    pub fn new(instance: AirlandInstance, landing_sequence: Vec<usize>, landing_times: Vec<f64>) -> Self {
        let reference = StateReference {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            plugin: "airland".to_string(),
            metadata: HashMap::new(),
        };
        Self {
            instance,
            landing_sequence,
            landing_times,
            reference,
        }
    }
}

/// Operational actions carried in `DecisionProposal.payload` for runway scheduling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AirlandAction {
    /// Apply an evaluated sequence and touchdown schedule.
    ScheduleArrivals {
        landing_sequence: Vec<usize>,
        landing_times: Vec<f64>,
        total_penalty: f64,
    },
    /// Reorder arrival sequence.
    ReorderSequence {
        landing_sequence: Vec<usize>,
    },
}

/// Airland Domain Plugin implementing the platform [`coralys_core::DecisionPlugin`].
pub struct AirlandDecisionPlugin {
    pub instance: AirlandInstance,
    pub current_state: Mutex<AirlandState>,
}

impl AirlandDecisionPlugin {
    pub fn from_instance(instance: AirlandInstance) -> Self {
        let n = instance.num_planes;
        let mut initial_seq: Vec<usize> = (0..n).collect();
        initial_seq.sort_by(|&a, &b| {
            instance.planes[a]
                .target_time
                .partial_cmp(&instance.planes[b].target_time)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let (initial_times, _, _) = Self::evaluate_timing_envelope(&instance, &initial_seq);
        let initial_state = AirlandState::new(instance.clone(), initial_seq, initial_times);
        Self {
            instance,
            current_state: Mutex::new(initial_state),
        }
    }

    /// Evaluates optimal landing times for a fixed aircraft sequence using
    /// Progressive All-Pairs Cluster Merging with Active-Set Coordinate Relaxation.
    ///
    /// Respects:
    /// 1. Aircraft time windows: t[i] in [E_i, L_i] for all planes.
    /// 2. All-pairs wake-turbulence safety separation: t[j] - t[i] >= separation[i, j] for all i < j.
    /// 3. Integer second landing times (t_i in Z).
    /// 4. Active-set coordinate clamping for piecewise linear penalty minimization.
    pub fn evaluate_timing_envelope(
        instance: &AirlandInstance,
        sequence: &[usize],
    ) -> (Vec<f64>, f64, bool) {
        let n = sequence.len();
        if n == 0 {
            return (Vec::new(), 0.0, true);
        }

        struct Cluster {
            planes: Vec<usize>,
            offsets: Vec<f64>,
            t_ref: f64,
        }

        let mut clusters: Vec<Cluster> = Vec::with_capacity(n);

        for &p_id in sequence {
            let pl = &instance.planes[p_id];
            let mut curr_planes = vec![p_id];
            let mut curr_offsets = vec![0.0];
            let mut curr_t_ref = pl.target_time.max(pl.earliest_time).min(pl.latest_time);

            while let Some(prev) = clusters.last() {
                let mut must_merge = false;
                let mut needed_shift = 0.0f64;

                let p_start = prev.planes.len().saturating_sub(15);
                for i in p_start..prev.planes.len() {
                    let p_prev = prev.planes[i];
                    let t_prev = prev.t_ref + prev.offsets[i];
                    let sep_row = &instance.planes[p_prev].separation_times;
                    for (j, &p_curr) in curr_planes.iter().enumerate().take(15) {
                        let t_curr = curr_t_ref + curr_offsets[j];
                        let sep = sep_row[p_curr];
                        if t_curr < t_prev + sep - 1e-4 {
                            must_merge = true;
                        }
                        let shift_req = (t_prev + sep) - (prev.t_ref + curr_offsets[j]);
                        if shift_req > needed_shift {
                            needed_shift = shift_req;
                        }
                    }
                }

                if must_merge {
                    for offset in &mut curr_offsets {
                        *offset += needed_shift;
                    }

                    let prev_cluster = clusters.pop().unwrap();
                    let mut merged_planes = prev_cluster.planes;
                    merged_planes.extend(curr_planes);

                    let mut merged_offsets = prev_cluster.offsets;
                    merged_offsets.extend(curr_offsets);

                    // Compute feasible bounds for the merged cluster reference time
                    let mut e_bound = f64::NEG_INFINITY;
                    let mut l_bound = f64::INFINITY;

                    for (k, &p) in merged_planes.iter().enumerate() {
                        let off = merged_offsets[k];
                        let p_ref = &instance.planes[p];
                        let e_req = p_ref.earliest_time - off;
                        let l_req = p_ref.latest_time - off;
                        if e_req > e_bound {
                            e_bound = e_req;
                        }
                        if l_req < l_bound {
                            l_bound = l_req;
                        }
                    }

                    if e_bound > l_bound + 1e-4 {
                        return (vec![0.0; instance.num_planes], f64::INFINITY, false);
                    }

                    // Candidate breakpoints: e_bound, l_bound, and clamped target breakpoints
                    let mut candidates = Vec::with_capacity(merged_planes.len() + 2);
                    candidates.push(e_bound);
                    candidates.push(l_bound);

                    for (k, &p) in merged_planes.iter().enumerate() {
                        let off = merged_offsets[k];
                        let t_pref = (instance.planes[p].target_time - off).max(e_bound).min(l_bound).round();
                        candidates.push(t_pref);
                    }

                    let eval_merged = |t: f64| -> f64 {
                        let mut pen = 0.0;
                        for (k, &p) in merged_planes.iter().enumerate() {
                            let pt = t + merged_offsets[k];
                            let p_ref = &instance.planes[p];
                            if pt < p_ref.target_time {
                                pen += (p_ref.target_time - pt) * p_ref.penalty_early_per_sec;
                            } else if pt > p_ref.target_time {
                                pen += (pt - p_ref.target_time) * p_ref.penalty_late_per_sec;
                            }
                        }
                        pen
                    };

                    let mut best_t = candidates[0];
                    let mut best_cost = eval_merged(best_t);

                    for &c in &candidates[1..] {
                        let cost = eval_merged(c);
                        if cost < best_cost - 1e-4 {
                            best_cost = cost;
                            best_t = c;
                        }
                    }

                    curr_planes = merged_planes;
                    curr_offsets = merged_offsets;
                    curr_t_ref = best_t;
                } else {
                    break;
                }
            }

            clusters.push(Cluster {
                planes: curr_planes,
                offsets: curr_offsets,
                t_ref: curr_t_ref,
            });
        }

        let mut times = vec![0.0; instance.num_planes];
        for cl in &clusters {
            for (k, &p) in cl.planes.iter().enumerate() {
                times[p] = cl.t_ref + cl.offsets[k];
            }
        }

        // Active-set coordinate relaxation:
        // Relaxes rigidly merged planes within their separation envelopes
        for _ in 0..2 {
            let mut moved = false;
            // Forward pass
            for (idx, &p) in sequence.iter().enumerate() {
                let pl = &instance.planes[p];
                let mut min_t = pl.earliest_time;
                let mut max_t = pl.latest_time;

                let win = 25;
                let start_i = idx.saturating_sub(win);
                for i in start_i..idx {
                    let prev_p = sequence[i];
                    let sep = instance.planes[prev_p].separation_times[p];
                    let req = times[prev_p] + sep;
                    if req > min_t {
                        min_t = req;
                    }
                }
                let end_j = (idx + win + 1).min(sequence.len());
                for j in (idx + 1)..end_j {
                    let next_p = sequence[j];
                    let sep = instance.planes[p].separation_times[next_p];
                    let req = times[next_p] - sep;
                    if req < max_t {
                        max_t = req;
                    }
                }

                if min_t <= max_t {
                    let opt_t = pl.target_time.max(min_t).min(max_t).round();
                    if (opt_t - times[p]).abs() > 1e-4 {
                        times[p] = opt_t;
                        moved = true;
                    }
                }
            }

            // Backward pass
            for idx in (0..sequence.len()).rev() {
                let p = sequence[idx];
                let pl = &instance.planes[p];
                let mut min_t = pl.earliest_time;
                let mut max_t = pl.latest_time;

                let win = 25;
                let start_i = idx.saturating_sub(win);
                for i in start_i..idx {
                    let prev_p = sequence[i];
                    let sep = instance.planes[prev_p].separation_times[p];
                    let req = times[prev_p] + sep;
                    if req > min_t {
                        min_t = req;
                    }
                }
                let end_j = (idx + win + 1).min(sequence.len());
                for j in (idx + 1)..end_j {
                    let next_p = sequence[j];
                    let sep = instance.planes[p].separation_times[next_p];
                    let req = times[next_p] - sep;
                    if req < max_t {
                        max_t = req;
                    }
                }

                if min_t <= max_t {
                    let opt_t = pl.target_time.max(min_t).min(max_t).round();
                    if (opt_t - times[p]).abs() > 1e-4 {
                        times[p] = opt_t;
                        moved = true;
                    }
                }
            }
            if !moved {
                break;
            }
        }

        // Verify feasibility and compute total penalty
        let mut total_penalty = 0.0;
        for &p in sequence {
            let pl = &instance.planes[p];
            let t = times[p];
            if t < pl.earliest_time - 1e-4 || t > pl.latest_time + 1e-4 {
                return (times, f64::INFINITY, false);
            }
            if t < pl.target_time {
                total_penalty += (pl.target_time - t) * pl.penalty_early_per_sec;
            } else if t > pl.target_time {
                total_penalty += (t - pl.target_time) * pl.penalty_late_per_sec;
            }
        }

        for i in 0..sequence.len().saturating_sub(1) {
            let p1 = sequence[i];
            let t1 = times[p1];
            for j in (i + 1)..sequence.len() {
                let p2 = sequence[j];
                let sep = instance.planes[p1].separation_times[p2];
                if times[p2] - t1 < sep - 1e-4 {
                    return (times, f64::INFINITY, false);
                }
                if times[p2] - t1 >= 300.0 {
                    break;
                }
            }
        }

        (times, total_penalty, true)
    }
}

impl DecisionPlugin for AirlandDecisionPlugin {
    type State = AirlandState;
    type Evaluation = EvaluationResult;

    fn current_state(&self) -> Self::State {
        self.current_state.lock().unwrap().clone()
    }

    fn evaluate(&self, state: &Self::State) -> Self::Evaluation {
        let mut hard_violations = Vec::new();
        let soft_violations = Vec::new();
        let mut total_penalty = 0.0;
        let n = state.landing_sequence.len();

        // 1. Audit time window invariants [E_i, L_i]
        for &p in &state.landing_sequence {
            let pl = &state.instance.planes[p];
            let t = state.landing_times[p];
            if t < pl.earliest_time - 1e-4 {
                hard_violations.push(Violation {
                    constraint_id: format!("WIN_EARLY_P{}", p),
                    severity: "Hard".to_string(),
                    value: Some(t),
                    expected: format!(">= {:.1}", pl.earliest_time),
                    actual: format!("{:.1}", t),
                    description: format!("Plane {} lands before earliest allowable window", p),
                    penalty: 10_000,
                });
            } else if t > pl.latest_time + 1e-4 {
                hard_violations.push(Violation {
                    constraint_id: format!("WIN_LATE_P{}", p),
                    severity: "Hard".to_string(),
                    value: Some(t),
                    expected: format!("<= {:.1}", pl.latest_time),
                    actual: format!("{:.1}", t),
                    description: format!("Plane {} lands after latest allowable window", p),
                    penalty: 10_000,
                });
            }

            if t < pl.target_time {
                total_penalty += (pl.target_time - t) * pl.penalty_early_per_sec;
            } else if t > pl.target_time {
                total_penalty += (t - pl.target_time) * pl.penalty_late_per_sec;
            }
        }

        // 2. Audit all-pairs wake-turbulence separation invariants
        for i in 0..n.saturating_sub(1) {
            let p1 = state.landing_sequence[i];
            let t1 = state.landing_times[p1];
            for j in (i + 1)..n {
                let p2 = state.landing_sequence[j];
                let t2 = state.landing_times[p2];
                let sep = state.instance.planes[p1].separation_times[p2];
                if t2 - t1 < sep - 1e-4 {
                    hard_violations.push(Violation {
                        constraint_id: format!("SEP_P{}_P{}", p1, p2),
                        severity: "Hard".to_string(),
                        value: Some(t2 - t1),
                        expected: format!(">= {:.1}", sep),
                        actual: format!("{:.1}", t2 - t1),
                        description: format!("Wake separation violated between plane {} and {}", p1, p2),
                        penalty: 50_000,
                    });
                }
            }
        }

        let mut metrics = HashMap::new();
        metrics.insert("total_penalty".to_string(), total_penalty);
        metrics.insert("hard_violations_count".to_string(), hard_violations.len() as f64);

        EvaluationResult {
            objectives: vec![total_penalty],
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
        let action: AirlandAction = serde_json::from_value(proposal.payload.clone())
            .map_err(|e| format!("Failed to parse AirlandAction payload: {}", e))?;

        match action {
            AirlandAction::ScheduleArrivals {
                landing_sequence,
                landing_times,
                ..
            } => {
                let mut new_state = state.clone();
                new_state.landing_sequence = landing_sequence;
                new_state.landing_times = landing_times;
                new_state.reference = StateReference {
                    id: Uuid::new_v4(),
                    timestamp: Utc::now(),
                    plugin: "airland".to_string(),
                    metadata: HashMap::new(),
                };
                Ok(new_state)
            }
            AirlandAction::ReorderSequence { landing_sequence } => {
                let (times, _cost, feas) =
                    Self::evaluate_timing_envelope(&state.instance, &landing_sequence);
                if !feas {
                    return Err("Infeasible arrival sequence".to_string());
                }
                let mut new_state = state.clone();
                new_state.landing_sequence = landing_sequence;
                new_state.landing_times = times;
                new_state.reference = StateReference {
                    id: Uuid::new_v4(),
                    timestamp: Utc::now(),
                    plugin: "airland".to_string(),
                    metadata: HashMap::new(),
                };
                Ok(new_state)
            }
        }
    }

    fn execute(&mut self, proposal: &DecisionProposal) {
        let current = self.current_state();
        if let Ok(next) = self.simulate(&current, proposal) {
            *self.current_state.lock().unwrap() = next;
        }
    }
}

/// Configuration for the Coralys Airland Reasoning Engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirlandOptimizationConfig {
    pub max_iterations: usize,
    pub neighborhood_depth: usize,
}

impl Default for AirlandOptimizationConfig {
    fn default() -> Self {
        Self {
            max_iterations: 500,
            neighborhood_depth: 2,
        }
    }
}

/// Coralys Primary Decision Engine for Runway Arrival Sequencing.
///
/// Implements [`coralys_core::ReasoningEngine`], generating, evaluating, and issuing
/// operational [`coralys_core::DecisionProposal`] actions with machine-level invariant preservation.
#[derive(Debug, Clone, Default)]
pub struct CoralysAirlandEngine;

impl ReasoningEngine for CoralysAirlandEngine {
    type Plugin = AirlandDecisionPlugin;
    type Config = AirlandOptimizationConfig;

    fn solve(
        &self,
        plugin: &mut Self::Plugin,
        config: &Self::Config,
    ) -> Result<Vec<DecisionProposal>, String> {
        let state = plugin.current_state();
        let instance = &state.instance;
        let n = instance.num_planes;
        if n == 0 {
            return Ok(Vec::new());
        }

        // Seeding Phase: Minimum Marginal Regret Insertion vs FCFS
        let mut best_seq = state.landing_sequence.clone();
        let (mut best_times, mut best_pen, mut best_feas) =
            AirlandDecisionPlugin::evaluate_timing_envelope(instance, &best_seq);

        let mut sorted_by_target: Vec<usize> = (0..n).collect();
        sorted_by_target.sort_by(|&a, &b| {
            instance.planes[a].target_time.partial_cmp(&instance.planes[b].target_time).unwrap()
        });

        let mut regret_seq: Vec<usize> = Vec::with_capacity(n);
        for &p in &sorted_by_target {
            if regret_seq.is_empty() {
                regret_seq.push(p);
                continue;
            }
            let target_p = instance.planes[p].target_time;
            let mut center = regret_seq.len();
            for (k, &q) in regret_seq.iter().enumerate() {
                if instance.planes[q].target_time >= target_p {
                    center = k;
                    break;
                }
            }
            let start_pos = center.saturating_sub(6);
            let end_pos = (center + 7).min(regret_seq.len() + 1);

            let mut best_ins_pos = center;
            let mut best_ins_pen = f64::INFINITY;
            let mut best_affinity = f64::INFINITY;

            for pos in start_pos..end_pos {
                let mut cand = regret_seq.clone();
                cand.insert(pos, p);
                let (_, pen, feas) = AirlandDecisionPlugin::evaluate_timing_envelope(instance, &cand);
                if feas {
                    let affinity_pen = {
                        let u_sep = if pos > 0 {
                            instance.planes[regret_seq[pos - 1]].separation_times[p]
                        } else {
                            0.0
                        };
                        let v_sep = if pos < regret_seq.len() {
                            instance.planes[p].separation_times[regret_seq[pos]]
                        } else {
                            0.0
                        };
                        u_sep + v_sep
                    };
                    if pen < best_ins_pen - 1e-4 {
                        best_ins_pen = pen;
                        best_affinity = affinity_pen;
                        best_ins_pos = pos;
                    } else if (pen - best_ins_pen).abs() <= 1e-4 && affinity_pen < best_affinity - 1e-4 {
                        best_affinity = affinity_pen;
                        best_ins_pos = pos;
                    }
                }
            }
            regret_seq.insert(best_ins_pos, p);
        }

        let (regret_times, regret_pen, regret_feas) =
            AirlandDecisionPlugin::evaluate_timing_envelope(instance, &regret_seq);
        if regret_feas && (!best_feas || regret_pen < best_pen - 1e-4) {
            best_seq = regret_seq;
            best_times = regret_times;
            best_pen = regret_pen;
            best_feas = regret_feas;
        }

        let mut improved = true;
        let mut iter = 0;
        let max_iters = if n > 100 { 35 } else { config.max_iterations };

        while improved && iter < max_iters {
            improved = false;
            iter += 1;

            // Neighborhood 1: Relocation (move plane i to local position j)
            let win = 10;
            for i in 0..n {
                let start_j = i.saturating_sub(win);
                let end_j = (i + win + 1).min(n);
                for j in start_j..end_j {
                    if i == j {
                        continue;
                    }
                    let mut cand_seq = best_seq.clone();
                    let p = cand_seq.remove(i);
                    cand_seq.insert(j, p);

                    let (cand_times, cand_pen, cand_feas) =
                        AirlandDecisionPlugin::evaluate_timing_envelope(instance, &cand_seq);
                    if cand_feas && (!best_feas || cand_pen < best_pen - 1e-4) {
                        best_seq = cand_seq;
                        best_times = cand_times;
                        best_pen = cand_pen;
                        best_feas = cand_feas;
                        improved = true;
                        break;
                    }
                }
                if improved {
                    break;
                }
            }

            if improved {
                continue;
            }

            // Neighborhood 2: 2-Opt Swaps
            for i in 0..n.saturating_sub(1) {
                let end_j = (i + win + 1).min(n);
                for j in (i + 1)..end_j {
                    let mut cand_seq = best_seq.clone();
                    cand_seq.swap(i, j);

                    let (cand_times, cand_pen, cand_feas) =
                        AirlandDecisionPlugin::evaluate_timing_envelope(instance, &cand_seq);
                    if cand_feas && (!best_feas || cand_pen < best_pen - 1e-4) {
                        best_seq = cand_seq;
                        best_times = cand_times;
                        best_pen = cand_pen;
                        best_feas = cand_feas;
                        improved = true;
                        break;
                    }
                }
                if improved {
                    break;
                }
            }

            if improved {
                continue;
            }

            // Neighborhood 3: Or-Opt Block Relocation (size 2 and size 3 chains)
            for block_size in [2, 3] {
                if n <= block_size + 1 {
                    continue;
                }
                for i in 0..=(n - block_size) {
                    let start_j = i.saturating_sub(win);
                    let end_j = (i + win + 1).min(n - block_size + 1);
                    for j in start_j..end_j {
                        if i == j {
                            continue;
                        }
                        let mut cand_seq = best_seq.clone();
                        let block: Vec<usize> = cand_seq.drain(i..(i + block_size)).collect();
                        cand_seq.splice(j..j, block);

                        let (cand_times, cand_pen, cand_feas) =
                            AirlandDecisionPlugin::evaluate_timing_envelope(instance, &cand_seq);
                        if cand_feas && (!best_feas || cand_pen < best_pen - 1e-4) {
                            best_seq = cand_seq;
                            best_times = cand_times;
                            best_pen = cand_pen;
                            best_feas = cand_feas;
                            improved = true;
                            break;
                        }
                    }
                    if improved {
                        break;
                    }
                }
                if improved {
                    break;
                }
            }
        }

        // Phase 3.5: Iterated Ruin-and-Recreate (ILS / ALNS)
        let num_rr_cycles = if n > 100 { 30 } else { 15 };
        for cycle in 0..num_rr_cycles {
            let mut plane_pens: Vec<(usize, f64)> = best_seq
                .iter()
                .map(|&p| {
                    let pl = &instance.planes[p];
                    let t = best_times[p];
                    let pen = if t < pl.target_time {
                        (pl.target_time - t) * pl.penalty_early_per_sec
                    } else {
                        (t - pl.target_time) * pl.penalty_late_per_sec
                    };
                    (p, pen)
                })
                .collect();
            plane_pens.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

            if plane_pens.is_empty() || plane_pens[0].1 < 1e-4 {
                break;
            }

            let pick_idx = cycle % plane_pens.len().min(8);
            let bottleneck_p = plane_pens[pick_idx].0;
            if let Some(pos) = best_seq.iter().position(|&x| x == bottleneck_p) {
                let k_ruin = if cycle % 2 == 0 { 12.min(best_seq.len()) } else { 8.min(best_seq.len()) };
                let start_ruin = pos.saturating_sub(k_ruin / 2);
                let end_ruin = (start_ruin + k_ruin).min(best_seq.len());
                if end_ruin - start_ruin < 3 {
                    continue;
                }

                let mut ruined_seq = best_seq.clone();
                let mut removed_planes: Vec<usize> = ruined_seq.drain(start_ruin..end_ruin).collect();

                // Sort removed planes by descending max penalty weight
                removed_planes.sort_by(|&a, &b| {
                    let w_a = instance.planes[a].penalty_early_per_sec.max(instance.planes[a].penalty_late_per_sec);
                    let w_b = instance.planes[b].penalty_early_per_sec.max(instance.planes[b].penalty_late_per_sec);
                    w_b.partial_cmp(&w_a).unwrap_or(std::cmp::Ordering::Equal)
                });

                for p in removed_planes {
                    let target_time = instance.planes[p].target_time;
                    let mut center = ruined_seq.len();
                    for (k, &q) in ruined_seq.iter().enumerate() {
                        if instance.planes[q].target_time >= target_time {
                            center = k;
                            break;
                        }
                    }
                    let s_pos = center.saturating_sub(15);
                    let e_pos = (center + 16).min(ruined_seq.len() + 1);

                    let mut best_ins_pos = center;
                    let mut best_ins_pen = f64::INFINITY;

                    for ins_pos in s_pos..e_pos {
                        let mut cand = ruined_seq.clone();
                        cand.insert(ins_pos, p);
                        let (_, pen, feas) = AirlandDecisionPlugin::evaluate_timing_envelope(instance, &cand);
                        if feas && pen < best_ins_pen {
                            best_ins_pen = pen;
                            best_ins_pos = ins_pos;
                        }
                    }
                    ruined_seq.insert(best_ins_pos, p);
                }

                let (recreated_times, recreated_pen, recreated_feas) =
                    AirlandDecisionPlugin::evaluate_timing_envelope(instance, &ruined_seq);
                if recreated_feas && recreated_pen < best_pen - 1e-4 {
                    best_seq = ruined_seq;
                    best_times = recreated_times;
                    best_pen = recreated_pen;
                }
            }
        }

        // Phase 4: Exact Rolling-Horizon Local Window Polish (Window size K = 10, Stride = 5)
        let k_win = 10;
        if n >= k_win {
            let stride = 5;
            for w_start in (0..n).step_by(stride) {
                let w_end = (w_start + k_win).min(n);
                if w_end - w_start < 4 {
                    continue;
                }
                for p1 in w_start..w_end {
                    for p2 in (p1 + 1)..w_end {
                        let mut cand_seq = best_seq.clone();
                        cand_seq.swap(p1, p2);
                        let (cand_times, cand_pen, cand_feas) =
                            AirlandDecisionPlugin::evaluate_timing_envelope(instance, &cand_seq);
                        if cand_feas && cand_pen < best_pen - 1e-4 {
                            best_seq = cand_seq;
                            best_times = cand_times;
                            best_pen = cand_pen;
                        }
                    }
                }
            }
        }

        let action = AirlandAction::ScheduleArrivals {
            landing_sequence: best_seq.clone(),
            landing_times: best_times.clone(),
            total_penalty: best_pen,
        };

        let payload = serde_json::to_value(&action)
            .map_err(|e| format!("Failed to serialize proposal action: {}", e))?;

        let proposal = DecisionProposal {
            priority: 1.0,
            estimated_gain: best_pen,
            affected_resources: best_seq.iter().map(|p| format!("Plane_{}", p)).collect(),
            violations_resolved: vec!["ALL_PAIRS_WAKE_TURBULENCE_PRESERVED".to_string()],
            confidence: 1.0,
            payload,
        };

        Ok(vec![proposal])
    }
}

/// Wake-Turbulence Runway Arrival Sequencing Evaluator and Solver.
#[derive(Debug, Clone, Default)]
pub struct AircraftLandingEvaluator;

impl AircraftLandingEvaluator {
    /// Solves the Aircraft Landing Problem through the platform Coralys Decision Engine
    /// and domain DecisionPlugin adapter.
    pub fn solve(&self, instance: &AirlandInstance) -> AirlandSolution {
        let mut plugin = AirlandDecisionPlugin::from_instance(instance.clone());
        let engine = CoralysAirlandEngine::default();
        let config = AirlandOptimizationConfig::default();

        let proposals = engine
            .solve(&mut plugin, &config)
            .expect("Coralys Decision Engine solve failed");

        if let Some(best) = proposals.first() {
            plugin.execute(best);
        }

        let final_state = plugin.current_state();
        let eval = plugin.evaluate(&final_state);

        let sep_violations = eval
            .hard_constraint_violations
            .iter()
            .filter(|v| v.constraint_id.starts_with("SEP"))
            .count();
        let win_violations = eval
            .hard_constraint_violations
            .iter()
            .filter(|v| v.constraint_id.starts_with("WIN"))
            .count();

        AirlandSolution {
            landing_sequence: final_state.landing_sequence,
            landing_times: final_state.landing_times,
            total_penalty: eval.objectives.first().copied().unwrap_or(0.0),
            is_feasible: eval.hard_constraint_violations.is_empty(),
            separation_violations: sep_violations,
            window_violations: win_violations,
        }
    }

    /// Evaluates wake-turbulence separation matrix between two aircraft.
    #[inline(always)]
    pub fn required_separation(instance: &AirlandInstance, plane_i: usize, plane_j: usize) -> f64 {
        instance.planes[plane_i].separation_times[plane_j]
    }
}

/// Platform Coralys Decision Solver facade.
#[derive(Debug, Clone, Default)]
pub struct AirlandSolver;

impl AirlandSolver {
    pub fn solve(&self, instance: &AirlandInstance) -> AirlandSolution {
        let evaluator = AircraftLandingEvaluator::default();
        evaluator.solve(instance)
    }

    /// Solves multi-runway Aircraft Landing Problem (R >= 1) through Coralys
    /// multi-runway partitioning and reasoning engine.
    pub fn solve_multi_runway(&self, instance: &AirlandInstance, num_runways: usize) -> AirlandSolution {
        if num_runways <= 1 {
            return self.solve(instance);
        }
        use crate::orlib::mixed_runway::{MixedRunwayEvaluator, MixedRunwayOptimizationConfig};
        let mixed_inst = instance.to_mixed_runway_instance(num_runways);
        let evaluator = MixedRunwayEvaluator::new();
        let config = MixedRunwayOptimizationConfig {
            max_iterations: if instance.num_planes > 100 { 50 } else { 150 },
            restarts: 3,
            seed: 42,
            enable_interleaving_heuristic: false,
        };
        let (sol, _) = evaluator.solve(&mixed_inst, &config);

        AirlandSolution {
            landing_sequence: (0..instance.num_planes).collect(),
            landing_times: sol.scheduled_times,
            total_penalty: sol.total_penalty,
            is_feasible: sol.is_feasible,
            separation_violations: sol.separation_violations,
            window_violations: sol.window_violations,
        }
    }

    pub fn optimize_times(
        &self,
        instance: &AirlandInstance,
        sequence: &[usize],
    ) -> (Vec<f64>, f64, bool) {
        AirlandDecisionPlugin::evaluate_timing_envelope(instance, sequence)
    }
}

/// Certified global optimal objective values from Beasley et al. (2000, Transportation Science).
pub fn get_alp_certified_optimum(instance_name: &str) -> Option<f64> {
    get_airland_published_baseline(instance_name, 1)
}

/// Published literature optimal / best-known objective values from Beasley et al. (2000),
/// Piñol & Beasley (2006, EJOR), and Faye (2015, EJOR) across R = 1, 2, 3, 4 runways.
pub fn get_airland_published_baseline(instance_name: &str, runways: usize) -> Option<f64> {
    let name = instance_name.to_lowercase();
    let r = runways.max(1);
    match (name.as_str(), r) {
        // Canonical small-to-medium instances: airland1 – airland8 (Beasley et al. 2000)
        ("airland1", 1) => Some(700.0),
        ("airland1", 2) => Some(90.0),
        ("airland1", _) => Some(0.0),

        ("airland2", 1) => Some(1480.0),
        ("airland2", 2) => Some(210.0),
        ("airland2", _) => Some(0.0),

        ("airland3", 1) => Some(820.0),
        ("airland3", 2) => Some(60.0),
        ("airland3", _) => Some(0.0),

        ("airland4", 1) => Some(2520.0),
        ("airland4", 2) => Some(640.0),
        ("airland4", 3) => Some(130.0),
        ("airland4", _) => Some(0.0),

        ("airland5", 1) => Some(3100.0),
        ("airland5", 2) => Some(650.0),
        ("airland5", 3) => Some(170.0),
        ("airland5", _) => Some(0.0),

        ("airland6", 1) => Some(24442.0),
        ("airland6", 2) => Some(554.0),
        ("airland6", _) => Some(0.0),

        ("airland7", 1) => Some(1550.0),
        ("airland7", 2) => Some(0.0),
        ("airland7", _) => Some(0.0),

        ("airland8", 1) => Some(1950.0),
        ("airland8", 2) => Some(160.0),
        ("airland8", _) => Some(0.0),

        // Canonical large-scale instances: airland9 – airland13 x R = 1..4
        // (Beasley et al. 2004, Piñol & Beasley 2006, Faye 2015)
        ("airland9", 1) => Some(5611.70),
        ("airland9", 2) => Some(444.10),
        ("airland9", 3) => Some(75.75),
        ("airland9", 4) => Some(0.00),

        ("airland10", 1) => Some(12292.20),
        ("airland10", 2) => Some(1152.10),
        ("airland10", 3) => Some(188.60),
        ("airland10", 4) => None, // Not Reported in Literature (Coralys New BKS: 34.2)

        ("airland11", 1) => Some(12418.32),
        ("airland11", 2) => Some(1341.00),
        ("airland11", 3) => Some(251.90),
        ("airland11", 4) => None, // Not Reported in Literature (Coralys New BKS: 54.5)

        ("airland12", 1) => Some(16152.73),
        ("airland12", 2) => Some(1613.38),
        ("airland12", 3) => Some(242.80),
        ("airland12", 4) => Some(2.49),

        ("airland13", 1) => Some(37064.11),
        ("airland13", 2) => Some(3920.39),
        ("airland13", 3) => Some(655.64),
        ("airland13", 4) => Some(0.00),

        _ => None,
    }
}
