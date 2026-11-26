//! OR-Library Set Partitioning Problem (SPP) Engine
//!
//! Models and solves canonical 0-1 Set Partitioning instances:
//! \min \sum_{j=1}^n c_j x_j
//! s.t. \sum_{j : i \in R_j} x_j = 1, \forall i \in \{1, \dots, m\}
//!      x_j \in \{0, 1\}
//!
//! Implements canonical Coralys Decision Architecture:
//! - [`SppDecisionPlugin`]: Platform [`coralys_core::DecisionPlugin`] auditing row coverage invariants
//!   and 256-bit SIMD-style disjointness masks without domain side-effects.
//! - [`CoralysSppEngine`]: Primary Decision Engine implementing [`coralys_core::ReasoningEngine`]
//!   with multi-start randomized disjoint ejection chains and post-feasibility union-bitmask
//!   column swaps (1-for-1, 2-for-1, 1-for-2, 2-for-2, 3-for-1).

use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::sync::Mutex;

use chrono::Utc;
use coralys_core::{
    DecisionPlugin, DecisionProposal, EvaluationResult, ReasoningEngine, SimulationResult,
    StateReference, Violation,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Deterministic, zero-dependency LCG Pseudo-Random Number Generator.
#[derive(Clone)]
struct SimpleRng(u64);

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self(seed.wrapping_add(1))
    }

    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0
    }

    fn next_usize(&mut self) -> usize {
        (self.next_u64() >> 33) as usize
    }

    fn gen_range(&mut self, max: usize) -> usize {
        if max == 0 {
            0
        } else {
            self.next_usize() % max
        }
    }
}

/// Column in a Set Partitioning instance (representing an airline flight pairing).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SppColumn {
    pub id: usize,
    pub cost: u64,
    pub rows: Vec<usize>, // 0-indexed row indices
    pub mask: [u64; 4],   // 256-bit fixed stack bitmask (covers up to 256 rows with zero heap alloc)
}

impl SppColumn {
    /// Checks if this column is disjoint from another column using 256-bit bitmasks.
    #[inline(always)]
    pub fn is_disjoint(&self, other: &SppColumn) -> bool {
        ((self.mask[0] & other.mask[0])
            | (self.mask[1] & other.mask[1])
            | (self.mask[2] & other.mask[2])
            | (self.mask[3] & other.mask[3]))
            == 0
    }

    /// Checks if this column collides with an accumulated row occupancy bitmask.
    #[inline(always)]
    pub fn collides_with_mask(&self, occupancy: &[u64; 4]) -> bool {
        ((self.mask[0] & occupancy[0])
            | (self.mask[1] & occupancy[1])
            | (self.mask[2] & occupancy[2])
            | (self.mask[3] & occupancy[3]))
            != 0
    }
}

/// A parsed Set Partitioning Problem instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SppInstance {
    pub name: String,
    pub num_rows: usize, // e.g. flights
    pub num_cols: usize, // e.g. candidate pairings
    pub columns: Vec<SppColumn>,
    pub row_to_cols: Vec<Vec<usize>>, // Inverted index: for each row, list of column indices covering it
    pub mask_words: usize,
}

impl SppInstance {
    /// Parses an OR-Library SPP file (e.g., sppnw01.txt).
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
                if let Ok(val) = token.parse::<usize>() {
                    tokens.push(val);
                }
            }
        }

        if tokens.len() < 2 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Insufficient tokens in SPP file",
            ));
        }

        let num_rows = tokens[0];
        let num_cols = tokens[1];
        let mask_words = 4;

        let mut columns = Vec::with_capacity(num_cols);
        let mut row_to_cols: Vec<Vec<usize>> = vec![Vec::new(); num_rows];

        let mut idx = 2;
        let mut col_id = 0;

        while idx < tokens.len() && col_id < num_cols {
            let cost = tokens[idx] as u64;
            let num_cov = tokens[idx + 1];
            idx += 2;

            let mut rows = Vec::with_capacity(num_cov);
            let mut mask = [0u64; 4];

            for _ in 0..num_cov {
                if idx < tokens.len() {
                    let row_1indexed = tokens[idx];
                    idx += 1;
                    if row_1indexed >= 1 && row_1indexed <= num_rows {
                        let r = row_1indexed - 1; // 0-indexed
                        rows.push(r);
                        let word = r / 64;
                        let bit = r % 64;
                        if word < 4 {
                            mask[word] |= 1u64 << bit;
                        }
                        row_to_cols[r].push(col_id);
                    }
                }
            }

            rows.sort_unstable();
            columns.push(SppColumn {
                id: col_id,
                cost,
                rows,
                mask,
            });
            col_id += 1;
        }

        // Sort each row's candidate column list by unit cost (cost / rows.len())
        for r in 0..num_rows {
            row_to_cols[r].sort_by(|&a, &b| {
                let unit_a = columns[a].cost as f64 / columns[a].rows.len() as f64;
                let unit_b = columns[b].cost as f64 / columns[b].rows.len() as f64;
                unit_a.partial_cmp(&unit_b).unwrap_or(std::cmp::Ordering::Equal)
            });
        }

        Ok(Self {
            name,
            num_rows,
            num_cols: columns.len(),
            columns,
            row_to_cols,
            mask_words,
        })
    }

    /// Known certified global integer optimal objective value from the literature.
    pub fn certified_optimum(&self) -> Option<u64> {
        get_certified_optimum(&self.name)
    }
}

/// Evaluated solution for an SPP instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SppSolution {
    pub selected_cols: Vec<usize>,
    pub total_cost: u64,
    pub row_coverage: Vec<u16>,
    pub uncovered_count: usize,
    pub overcovered_count: usize,
}

impl SppSolution {
    #[inline(always)]
    pub fn is_feasible(&self) -> bool {
        self.uncovered_count == 0 && self.overcovered_count == 0
    }

    /// Calculates optimality gap in percentage relative to proven global optimum:
    /// gap = (Z - Z*) / Z* * 100%
    pub fn optimality_gap(&self, certified_opt: u64) -> f64 {
        if certified_opt == 0 {
            return 0.0;
        }
        ((self.total_cost as f64 - certified_opt as f64) / certified_opt as f64) * 100.0
    }
}

/// Operational domain state managed by the Coralys SPP Decision Plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SppState {
    pub instance: SppInstance,
    pub selected_cols: Vec<usize>,
    pub total_cost: u64,
    pub occupancy_mask: [u64; 4],
    pub reference: StateReference,
}

impl SppState {
    pub fn new(instance: SppInstance, selected_cols: Vec<usize>) -> Self {
        let mut occupancy_mask = [0u64; 4];
        let mut total_cost = 0;
        for &col_id in &selected_cols {
            if col_id < instance.columns.len() {
                let col = &instance.columns[col_id];
                total_cost += col.cost;
                for w in 0..4 {
                    occupancy_mask[w] |= col.mask[w];
                }
            }
        }
        let reference = StateReference {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            plugin: "spp".to_string(),
            metadata: HashMap::new(),
        };
        Self {
            instance,
            selected_cols,
            total_cost,
            occupancy_mask,
            reference,
        }
    }
}

/// Operational actions carried in `DecisionProposal.payload` for set partitioning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SppAction {
    /// Apply an evaluated column partition solution.
    ApplyPartition {
        selected_cols: Vec<usize>,
        total_cost: u64,
    },
    /// Atomic column replacement (ejection and insertion).
    ReplaceColumns {
        drop_cols: Vec<usize>,
        add_cols: Vec<usize>,
    },
}

/// SPP Domain Plugin implementing the platform [`coralys_core::DecisionPlugin`].
pub struct SppDecisionPlugin {
    pub instance: SppInstance,
    pub current_state: Mutex<SppState>,
}

impl SppDecisionPlugin {
    pub fn from_instance(instance: SppInstance) -> Self {
        let initial_state = SppState::new(instance.clone(), Vec::new());
        Self {
            instance,
            current_state: Mutex::new(initial_state),
        }
    }

    /// Computes full row coverage histogram across all rows.
    pub fn compute_row_coverage(instance: &SppInstance, selected_cols: &[usize]) -> Vec<u16> {
        let mut row_cov = vec![0u16; instance.num_rows];
        for &col_id in selected_cols {
            if col_id < instance.columns.len() {
                for &r in &instance.columns[col_id].rows {
                    if r < instance.num_rows {
                        row_cov[r] += 1;
                    }
                }
            }
        }
        row_cov
    }
}

impl DecisionPlugin for SppDecisionPlugin {
    type State = SppState;
    type Evaluation = EvaluationResult;

    fn current_state(&self) -> Self::State {
        self.current_state.lock().unwrap().clone()
    }

    fn evaluate(&self, state: &Self::State) -> Self::Evaluation {
        let mut hard_violations = Vec::new();
        let soft_violations = Vec::new();
        let row_cov = Self::compute_row_coverage(&state.instance, &state.selected_cols);
        let mut uncov = 0;
        let mut overcov = 0;

        for (r, &cov) in row_cov.iter().enumerate() {
            if cov == 0 {
                uncov += 1;
                hard_violations.push(Violation {
                    constraint_id: format!("ROW_UNCOVERED_{}", r),
                    severity: "Hard".to_string(),
                    value: Some(0.0),
                    expected: "== 1.0".to_string(),
                    actual: "0.0".to_string(),
                    description: format!("Row {} is uncovered (must be exactly 1)", r),
                    penalty: 10_000,
                });
            } else if cov > 1 {
                overcov += (cov - 1) as usize;
                hard_violations.push(Violation {
                    constraint_id: format!("ROW_OVERCOVERED_{}", r),
                    severity: "Hard".to_string(),
                    value: Some(cov as f64),
                    expected: "== 1.0".to_string(),
                    actual: format!("{:.1}", cov),
                    description: format!("Row {} is covered {} times (must be exactly 1)", r, cov),
                    penalty: 20_000 * (cov as i32 - 1),
                });
            }
        }

        let mut metrics = HashMap::new();
        metrics.insert("total_cost".to_string(), state.total_cost as f64);
        metrics.insert("uncovered_count".to_string(), uncov as f64);
        metrics.insert("overcovered_count".to_string(), overcov as f64);
        metrics.insert("hard_violations_count".to_string(), hard_violations.len() as f64);

        EvaluationResult {
            objectives: vec![state.total_cost as f64],
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
        let action: SppAction = serde_json::from_value(proposal.payload.clone())
            .map_err(|e| format!("Failed to parse SppAction payload: {}", e))?;

        match action {
            SppAction::ApplyPartition {
                selected_cols,
                total_cost,
            } => {
                let mut occupancy = [0u64; 4];
                let mut computed_cost = 0;
                for &col_id in &selected_cols {
                    if col_id >= state.instance.columns.len() {
                        return Err(format!("Column index {} out of bounds", col_id));
                    }
                    let col = &state.instance.columns[col_id];
                    computed_cost += col.cost;
                    for w in 0..4 {
                        occupancy[w] |= col.mask[w];
                    }
                }

                let mut new_state = state.clone();
                new_state.selected_cols = selected_cols;
                new_state.total_cost = if total_cost > 0 { total_cost } else { computed_cost };
                new_state.occupancy_mask = occupancy;
                new_state.reference = StateReference {
                    id: Uuid::new_v4(),
                    timestamp: Utc::now(),
                    plugin: "spp".to_string(),
                    metadata: HashMap::new(),
                };
                Ok(new_state)
            }
            SppAction::ReplaceColumns {
                drop_cols,
                add_cols,
            } => {
                let mut new_cols: Vec<usize> = state
                    .selected_cols
                    .iter()
                    .filter(|c| !drop_cols.contains(c))
                    .copied()
                    .collect();
                new_cols.extend(add_cols);
                new_cols.sort_unstable();
                new_cols.dedup();

                let mut occupancy = [0u64; 4];
                let mut computed_cost = 0;
                for &col_id in &new_cols {
                    if col_id >= state.instance.columns.len() {
                        return Err(format!("Column index {} out of bounds", col_id));
                    }
                    let col = &state.instance.columns[col_id];
                    computed_cost += col.cost;
                    for w in 0..4 {
                        occupancy[w] |= col.mask[w];
                    }
                }

                let mut new_state = state.clone();
                new_state.selected_cols = new_cols;
                new_state.total_cost = computed_cost;
                new_state.occupancy_mask = occupancy;
                new_state.reference = StateReference {
                    id: Uuid::new_v4(),
                    timestamp: Utc::now(),
                    plugin: "spp".to_string(),
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

/// Configuration for the Coralys SPP Decision Reasoning Engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SppOptimizationConfig {
    pub restarts: usize,
    pub max_ejection_iterations: usize,
    pub seed: u64,
    pub enable_union_swaps: bool,
}

impl Default for SppOptimizationConfig {
    fn default() -> Self {
        Self {
            restarts: 20,
            max_ejection_iterations: 3000,
            seed: 42,
            enable_union_swaps: true,
        }
    }
}

/// Coralys Primary Decision Engine for Crew Set Partitioning.
///
/// Implements [`coralys_core::ReasoningEngine`], generating, evaluating, and issuing
/// operational [`coralys_core::DecisionProposal`] actions with machine-level invariant preservation.
#[derive(Debug, Clone, Default)]
pub struct CoralysSppEngine;

impl ReasoningEngine for CoralysSppEngine {
    type Plugin = SppDecisionPlugin;
    type Config = SppOptimizationConfig;

    fn solve(
        &self,
        plugin: &mut Self::Plugin,
        config: &Self::Config,
    ) -> Result<Vec<DecisionProposal>, String> {
        let instance = &plugin.instance;
        let m = instance.num_rows;
        let n = instance.num_cols;

        if m == 0 || n == 0 {
            return Ok(Vec::new());
        }

        // Pre-index global minimum cost per unique 256-bit row mask
        let mut mask_to_best_col: HashMap<[u64; 4], (u64, usize)> =
            HashMap::with_capacity(n);
        for col in &instance.columns {
            mask_to_best_col
                .entry(col.mask)
                .and_modify(|(best_cost, best_id)| {
                    if col.cost < *best_cost {
                        *best_cost = col.cost;
                        *best_id = col.id;
                    }
                })
                .or_insert((col.cost, col.id));
        }

        let mut best_selected: Vec<usize> = Vec::new();
        let mut best_cost = u64::MAX;
        let mut best_uncovered = m;
        let mut best_feasible = false;

        let restarts = config.restarts.max(6);

        for r_idx in 0..restarts {
            let seed = config.seed.wrapping_add(r_idx as u64 * 997 + 1);
            let mut rng = SimpleRng::new(seed);

            let (mut selected, uncov) = Self::run_ejection_search(
                instance,
                config.max_ejection_iterations,
                r_idx,
                &mut rng,
            );

            if uncov == 0 {
                // Feasible partition found! Execute post-feasibility union-bitmask column swaps
                if config.enable_union_swaps {
                    Self::optimize_union_swaps(instance, &mut selected, &mask_to_best_col);
                }

                let cur_cost: u64 = selected.iter().map(|&c| instance.columns[c].cost).sum();
                if !best_feasible || cur_cost < best_cost {
                    best_cost = cur_cost;
                    best_selected = selected;
                    best_uncovered = 0;
                    best_feasible = true;
                }
            } else if !best_feasible {
                if uncov < best_uncovered {
                    best_uncovered = uncov;
                    best_selected = selected;
                    best_cost = best_selected.iter().map(|&c| instance.columns[c].cost).sum();
                }
            }
        }

        best_selected.sort_unstable();

        let action = SppAction::ApplyPartition {
            selected_cols: best_selected.clone(),
            total_cost: best_cost,
        };

        let payload = serde_json::to_value(&action)
            .map_err(|e| format!("Failed to serialize proposal action: {}", e))?;

        let proposal = DecisionProposal {
            priority: 1.0,
            estimated_gain: best_cost as f64,
            affected_resources: best_selected.iter().map(|c| format!("Col_{}", c)).collect(),
            violations_resolved: if best_feasible {
                vec!["EXACT_SET_PARTITION_SATISFIED".to_string()]
            } else {
                vec![format!("PARTIAL_PARTITION_UNCOVERED_{}", best_uncovered)]
            },
            confidence: if best_feasible { 1.0 } else { 0.5 },
            payload,
        };

        Ok(vec![proposal])
    }
}

impl CoralysSppEngine {
    /// Disjoint Ejection Chain Search to find feasible (zero uncovered, zero overcovered) solutions.
    fn run_ejection_search(
        instance: &SppInstance,
        max_iterations: usize,
        strategy: usize,
        rng: &mut SimpleRng,
    ) -> (Vec<usize>, usize) {
        let m = instance.num_rows;
        let mut in_solution = vec![false; instance.num_cols];
        let mut selected = Vec::new();
        let mut col_covering_row: Vec<Option<usize>> = vec![None; m];
        let mut occupancy = [0u64; 4];
        let mut uncovered_count = m;

        // Phase 1: Randomized Greedy Disjoint Seeding using Minimum Remaining Values (MRV)
        for _ in 0..m {
            let mut best_r = None;
            let mut min_score = u64::MAX;

            for r in 0..m {
                if col_covering_row[r].is_none() {
                    let cand_count = instance.row_to_cols[r].len() as u64;
                    let score = cand_count * 100 + (rng.next_u64() % 40);
                    if score < min_score {
                        min_score = score;
                        best_r = Some(r);
                    }
                }
            }

            let r = match best_r {
                Some(row) => row,
                None => break,
            };

            // Find best disjoint column using strategy-driven ranking + RCL
            let mut scored_cands: Vec<(usize, f64)> = Vec::with_capacity(16);

            for &cand_id in &instance.row_to_cols[r] {
                let col = &instance.columns[cand_id];
                if !col.collides_with_mask(&occupancy) {
                    let score = match strategy % 6 {
                        0 => col.cost as f64 / col.rows.len().max(1) as f64,
                        1 => {
                            let sum_inv: f64 = col
                                .rows
                                .iter()
                                .map(|&cr| 1.0 / (instance.row_to_cols[cr].len().max(1) as f64).sqrt())
                                .sum();
                            col.cost as f64 / sum_inv.max(1e-6)
                        }
                        2 => col.cost as f64 / (col.rows.len().max(1) as f64).powf(1.4),
                        3 => col.cost as f64,
                        4 => {
                            let jitter = 1.0 + ((rng.next_u64() % 1000) as f64 / 4000.0);
                            (col.cost as f64 / col.rows.len().max(1) as f64) * jitter
                        }
                        _ => {
                            let sum_inv: f64 = col
                                .rows
                                .iter()
                                .map(|&cr| 1.0 / instance.row_to_cols[cr].len().max(1) as f64)
                                .sum();
                            col.cost as f64 / sum_inv.max(1e-6)
                        }
                    };
                    scored_cands.push((cand_id, score));
                    if scored_cands.len() >= 12 {
                        break;
                    }
                }
            }

            scored_cands.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

            let best_col_id = if !scored_cands.is_empty() {
                let limit = scored_cands.len().min(3);
                let pick = match rng.gen_range(100) {
                    0..=65 => 0,
                    66..=89 => 1.min(limit - 1),
                    _ => 2.min(limit - 1),
                };
                Some(scored_cands[pick].0)
            } else {
                None
            };

            if let Some(col_id) = best_col_id {
                in_solution[col_id] = true;
                selected.push(col_id);
                let col = &instance.columns[col_id];
                for &cov_r in &col.rows {
                    col_covering_row[cov_r] = Some(col_id);
                    uncovered_count -= 1;
                }
                for w in 0..4 {
                    occupancy[w] |= col.mask[w];
                }
            }
        }

        if uncovered_count == 0 {
            return (selected, 0);
        }

        // Phase 2: Disjoint Ejection Chain Search with Tabu Avoidance & Dynamic Perturbations
        let mut tabu_until = vec![0usize; instance.num_cols];
        let mut best_uncovered = uncovered_count;
        let mut best_selected = selected.clone();

        let iters = max_iterations.min(if m > 100 { 800 } else { 400 });
        let mut stagnant_steps = 0;

        for _iter in 0..iters {
            if uncovered_count == 0 {
                return (selected, 0);
            }

            if stagnant_steps > 50 {
                break;
            }

            // Pick uncovered row with minimum candidates (most constrained)
            let mut best_r = None;
            let mut min_candidates = usize::MAX;

            for r in 0..m {
                if col_covering_row[r].is_none() {
                    let cand_count = instance.row_to_cols[r].len();
                    if cand_count < min_candidates {
                        min_candidates = cand_count;
                        best_r = Some(r);
                    }
                }
            }

            let r = match best_r {
                Some(row) => row,
                None => break,
            };

            let candidates = &instance.row_to_cols[r];
            if candidates.is_empty() {
                break;
            }

            let mut best_move: Option<(usize, Vec<usize>, isize)> = None;
            let mut best_eval = i64::MAX;

            // When close to feasibility, evaluate more thoroughly
            let sample_limit = if uncovered_count <= 4 {
                candidates.len().min(4000)
            } else if uncovered_count <= 8 {
                candidates.len().min(1500)
            } else {
                candidates.len().min(400)
            };
            let offset = rng.gen_range(candidates.len());

            for s in 0..sample_limit {
                let cand_id = candidates[(offset + s) % candidates.len()];
                if in_solution[cand_id] {
                    continue;
                }

                let col = &instance.columns[cand_id];

                let mut colliding: Vec<usize> = Vec::with_capacity(col.rows.len());
                for &cov_r in &col.rows {
                    if let Some(exist_id) = col_covering_row[cov_r] {
                        if !colliding.contains(&exist_id) {
                            colliding.push(exist_id);
                        }
                    }
                }

                let newly_covered = col
                    .rows
                    .iter()
                    .filter(|&&cr| col_covering_row[cr].is_none())
                    .count();

                let mut lost_rows = 0;
                for &e_id in &colliding {
                    let e_col = &instance.columns[e_id];
                    for &e_r in &e_col.rows {
                        if !col.rows.contains(&e_r) {
                            lost_rows += 1;
                        }
                    }
                }

                let delta_uncovered = lost_rows as isize - newly_covered as isize;
                let target_uncovered = (uncovered_count as isize + delta_uncovered).max(0) as usize;

                // Tabu check with aspiration criterion (if achieves 0 uncovered or improves historical best)
                let is_tabu = tabu_until[cand_id] > _iter;
                if is_tabu && target_uncovered > 0 && target_uncovered >= best_uncovered {
                    continue;
                }

                // Pure feasibility score: prioritize reducing uncovered rows with minimal collisions
                let collision_count = colliding.len() as i64;
                let rand_jitter = (rng.next_u64() % 100) as i64;
                let eval = (delta_uncovered as i64 * 10_000) + (collision_count * 100) + rand_jitter;

                if eval < best_eval {
                    best_eval = eval;
                    best_move = Some((cand_id, colliding, delta_uncovered));
                }
            }

            if let Some((add_col, drop_cols, delta_uncov)) = best_move {
                for &drop_id in &drop_cols {
                    in_solution[drop_id] = false;
                    let drop_col = &instance.columns[drop_id];
                    for &dr in &drop_col.rows {
                        col_covering_row[dr] = None;
                    }
                    for w in 0..4 {
                        occupancy[w] &= !drop_col.mask[w];
                    }
                    // Apply randomized tabu tenure
                    tabu_until[drop_id] = _iter + 12 + rng.gen_range(12);
                }
                selected.retain(|&c| !drop_cols.contains(&c));

                in_solution[add_col] = true;
                selected.push(add_col);
                let col = &instance.columns[add_col];
                for &ar in &col.rows {
                    col_covering_row[ar] = Some(add_col);
                }
                for w in 0..4 {
                    occupancy[w] |= col.mask[w];
                }

                uncovered_count = (uncovered_count as isize + delta_uncov) as usize;

                if uncovered_count < best_uncovered {
                    best_uncovered = uncovered_count;
                    best_selected = selected.clone();
                    stagnant_steps = 0;
                } else {
                    stagnant_steps += 1;
                }

                // Dynamic perturbation if stagnant: kick a random candidate
                if stagnant_steps > 80 && uncovered_count > 0 {
                    stagnant_steps = 0;
                    let rand_cand = candidates[rng.gen_range(candidates.len())];
                    if !in_solution[rand_cand] {
                        let rand_col = &instance.columns[rand_cand];
                        let mut colliding: Vec<usize> = Vec::new();
                        for &cov_r in &rand_col.rows {
                            if let Some(exist_id) = col_covering_row[cov_r] {
                                if !colliding.contains(&exist_id) {
                                    colliding.push(exist_id);
                                }
                            }
                        }
                        for &drop_id in &colliding {
                            in_solution[drop_id] = false;
                            let drop_col = &instance.columns[drop_id];
                            for &dr in &drop_col.rows {
                                col_covering_row[dr] = None;
                            }
                            for w in 0..4 {
                                occupancy[w] &= !drop_col.mask[w];
                            }
                        }
                        selected.retain(|&c| !colliding.contains(&c));

                        in_solution[rand_cand] = true;
                        selected.push(rand_cand);
                        for &ar in &rand_col.rows {
                            col_covering_row[ar] = Some(rand_cand);
                        }
                        for w in 0..4 {
                            occupancy[w] |= rand_col.mask[w];
                        }
                        uncovered_count = col_covering_row.iter().filter(|c| c.is_none()).count();
                    }
                }
            } else {
                // Random walk step
                let rand_cand = candidates[rng.gen_range(candidates.len())];
                if !in_solution[rand_cand] {
                    let col = &instance.columns[rand_cand];
                    let mut colliding: Vec<usize> = Vec::new();
                    for &cov_r in &col.rows {
                        if let Some(exist_id) = col_covering_row[cov_r] {
                            if !colliding.contains(&exist_id) {
                                colliding.push(exist_id);
                            }
                        }
                    }

                    for &drop_id in &colliding {
                        in_solution[drop_id] = false;
                        let drop_col = &instance.columns[drop_id];
                        for &dr in &drop_col.rows {
                            col_covering_row[dr] = None;
                        }
                        for w in 0..4 {
                            occupancy[w] &= !drop_col.mask[w];
                        }
                    }
                    selected.retain(|&c| !colliding.contains(&c));

                    in_solution[rand_cand] = true;
                    selected.push(rand_cand);
                    for &ar in &col.rows {
                        col_covering_row[ar] = Some(rand_cand);
                    }
                    for w in 0..4 {
                        occupancy[w] |= col.mask[w];
                    }

                    uncovered_count = col_covering_row.iter().filter(|c| c.is_none()).count();
                }
            }
        }

        if uncovered_count == 0 {
            (selected, 0)
        } else {
            (best_selected, best_uncovered)
        }
    }

    /// Post-Feasibility Union-Bitmask Column Swap Optimization.
    ///
    /// Preserves exact 100% feasibility (sum a_{ij} x_j = 1) at every single step:
    /// - 1-for-1 swaps: mask_B == mask_A and c_B < c_A
    /// - 2-for-1 swaps: mask_C == mask_A | mask_B and c_C < c_A + c_B
    /// - 1-for-2 swaps: mask_B | mask_C == mask_A (disjoint) and c_B + c_C < c_A
    /// - 2-for-2 swaps: mask_C | mask_D == mask_A | mask_B (disjoint) and c_C + c_D < c_A + c_B
    /// - 3-for-1 swaps: mask_D == mask_A | mask_B | mask_C and c_D < c_A + c_B + c_C
    pub fn optimize_union_swaps(
        instance: &SppInstance,
        selected: &mut Vec<usize>,
        mask_to_best_col: &HashMap<[u64; 4], (u64, usize)>,
    ) {
        let mut improved = true;
        let mut polish_rounds = 0;

        while improved && polish_rounds < 30 {
            improved = false;
            polish_rounds += 1;

            // 1. Pass: 1-for-1 Swaps (replace column with cheaper identical-mask column)
            for idx in 0..selected.len() {
                let cur_id = selected[idx];
                let cur_mask = instance.columns[cur_id].mask;
                if let Some(&(best_cost, best_id)) = mask_to_best_col.get(&cur_mask) {
                    if best_cost < instance.columns[cur_id].cost && !selected.contains(&best_id) {
                        selected[idx] = best_id;
                        improved = true;
                    }
                }
            }

            // 2. Pass: 2-for-1 Swaps (merge two disjoint columns into one cheaper column)
            let mut pair_to_merge = None;
            'outer_2_1: for i in 0..selected.len().saturating_sub(1) {
                let id_a = selected[i];
                let col_a = &instance.columns[id_a];
                for j in (i + 1)..selected.len() {
                    let id_b = selected[j];
                    let col_b = &instance.columns[id_b];
                    let union_mask = [
                        col_a.mask[0] | col_b.mask[0],
                        col_a.mask[1] | col_b.mask[1],
                        col_a.mask[2] | col_b.mask[2],
                        col_a.mask[3] | col_b.mask[3],
                    ];
                    let combined_cost = col_a.cost + col_b.cost;
                    if let Some(&(cand_cost, cand_id)) = mask_to_best_col.get(&union_mask) {
                        if cand_cost < combined_cost && !selected.contains(&cand_id) {
                            pair_to_merge = Some((i, j, cand_id));
                            break 'outer_2_1;
                        }
                    }
                }
            }
            if let Some((i, j, cand_id)) = pair_to_merge {
                selected.remove(j); // remove larger index first
                selected[i] = cand_id;
                improved = true;
                continue;
            }

            // 3. Pass: 1-for-2 Swaps (split one expensive column into two cheaper disjoint columns)
            let mut split_move = None;
            'outer_1_2: for i in 0..selected.len() {
                let id_a = selected[i];
                let col_a = &instance.columns[id_a];
                if col_a.rows.len() < 2 {
                    continue;
                }
                let r1 = col_a.rows[0];
                let cands = &instance.row_to_cols[r1];
                for &id_b in cands.iter().take(600) {
                    if id_b == id_a || selected.contains(&id_b) {
                        continue;
                    }
                    let col_b = &instance.columns[id_b];
                    if col_b.cost >= col_a.cost {
                        continue;
                    }
                    let is_submask = ((col_b.mask[0] & !col_a.mask[0])
                        | (col_b.mask[1] & !col_a.mask[1])
                        | (col_b.mask[2] & !col_a.mask[2])
                        | (col_b.mask[3] & !col_a.mask[3])) == 0;
                    if !is_submask || col_b.mask == col_a.mask {
                        continue;
                    }
                    let rem_mask = [
                        col_a.mask[0] & !col_b.mask[0],
                        col_a.mask[1] & !col_b.mask[1],
                        col_a.mask[2] & !col_b.mask[2],
                        col_a.mask[3] & !col_b.mask[3],
                    ];
                    if let Some(&(cost_c, id_c)) = mask_to_best_col.get(&rem_mask) {
                        if col_b.cost + cost_c < col_a.cost && id_c != id_b && !selected.contains(&id_c) {
                            split_move = Some((i, id_b, id_c));
                            break 'outer_1_2;
                        }
                    }
                }
            }
            if let Some((i, id_b, id_c)) = split_move {
                selected[i] = id_b;
                selected.push(id_c);
                improved = true;
                continue;
            }

            // 4. Pass: 2-for-2 Swaps (trade two columns for two cheaper disjoint columns)
            let mut swap_move = None;
            'outer_2_2: for i in 0..selected.len().saturating_sub(1) {
                let id_a = selected[i];
                let col_a = &instance.columns[id_a];
                for j in (i + 1)..selected.len() {
                    let id_b = selected[j];
                    let col_b = &instance.columns[id_b];
                    let union_mask = [
                        col_a.mask[0] | col_b.mask[0],
                        col_a.mask[1] | col_b.mask[1],
                        col_a.mask[2] | col_b.mask[2],
                        col_a.mask[3] | col_b.mask[3],
                    ];
                    let combined_cost = col_a.cost + col_b.cost;
                    let r1 = col_a.rows[0];
                    for &id_c in instance.row_to_cols[r1].iter().take(400) {
                        if id_c == id_a || id_c == id_b || selected.contains(&id_c) {
                            continue;
                        }
                        let col_c = &instance.columns[id_c];
                        if col_c.cost >= combined_cost {
                            continue;
                        }
                        let is_submask = ((col_c.mask[0] & !union_mask[0])
                            | (col_c.mask[1] & !union_mask[1])
                            | (col_c.mask[2] & !union_mask[2])
                            | (col_c.mask[3] & !union_mask[3])) == 0;
                        if !is_submask
                            || col_c.mask == col_a.mask
                            || col_c.mask == col_b.mask
                            || col_c.mask == union_mask
                        {
                            continue;
                        }
                        let rem_mask = [
                            union_mask[0] & !col_c.mask[0],
                            union_mask[1] & !col_c.mask[1],
                            union_mask[2] & !col_c.mask[2],
                            union_mask[3] & !col_c.mask[3],
                        ];
                        if let Some(&(cost_d, id_d)) = mask_to_best_col.get(&rem_mask) {
                            if col_c.cost + cost_d < combined_cost
                                && id_d != id_c
                                && !selected.contains(&id_d)
                            {
                                swap_move = Some((i, j, id_c, id_d));
                                break 'outer_2_2;
                            }
                        }
                    }
                }
            }
            if let Some((i, j, id_c, id_d)) = swap_move {
                selected.remove(j);
                selected[i] = id_c;
                selected.push(id_d);
                improved = true;
                continue;
            }

            // 5. Pass: 3-for-1 Swaps (merge three disjoint columns into one cheaper column)
            if selected.len() >= 3 && selected.len() <= 60 {
                let mut tri_to_merge = None;
                'outer_3_1: for i in 0..selected.len().saturating_sub(2) {
                    let col_a = &instance.columns[selected[i]];
                    for j in (i + 1)..selected.len().saturating_sub(1) {
                        let col_b = &instance.columns[selected[j]];
                        let ab_cost = col_a.cost + col_b.cost;
                        let ab_0 = col_a.mask[0] | col_b.mask[0];
                        let ab_1 = col_a.mask[1] | col_b.mask[1];
                        let ab_2 = col_a.mask[2] | col_b.mask[2];
                        let ab_3 = col_a.mask[3] | col_b.mask[3];

                        for k in (j + 1)..selected.len() {
                            let col_c = &instance.columns[selected[k]];
                            let tri_mask = [
                                ab_0 | col_c.mask[0],
                                ab_1 | col_c.mask[1],
                                ab_2 | col_c.mask[2],
                                ab_3 | col_c.mask[3],
                            ];
                            let tri_cost = ab_cost + col_c.cost;
                            if let Some(&(cand_cost, cand_id)) = mask_to_best_col.get(&tri_mask) {
                                if cand_cost < tri_cost && !selected.contains(&cand_id) {
                                    tri_to_merge = Some((i, j, k, cand_id));
                                    break 'outer_3_1;
                                }
                            }
                        }
                    }
                }
                if let Some((i, j, k, cand_id)) = tri_to_merge {
                    selected.remove(k);
                    selected.remove(j);
                    selected[i] = cand_id;
                    improved = true;
                    continue;
                }
            }
        }
    }
}

/// 64-bit, 128-bit, and 256-bit Bitmask Set Partitioning Evaluator and Invariant Solver.
#[derive(Debug, Clone)]
pub struct SetPartitioningEvaluator {
    pub penalty_uncovered: u64,
    pub penalty_overcovered: u64,
}

impl Default for SetPartitioningEvaluator {
    fn default() -> Self {
        Self {
            penalty_uncovered: 1_000_000,
            penalty_overcovered: 2_000_000,
        }
    }
}

impl SetPartitioningEvaluator {
    /// 64-bit bitwise intersection check (returns true if any collision).
    #[inline(always)]
    pub fn has_overlap_64(a: u64, b: u64) -> bool {
        (a & b) != 0
    }

    /// 128-bit bitwise intersection check (returns true if any collision).
    #[inline(always)]
    pub fn has_overlap_128(a: u128, b: u128) -> bool {
        (a & b) != 0
    }

    /// 256-bit bitwise intersection check (returns true if any collision).
    #[inline(always)]
    pub fn has_overlap_256(a: &[u64; 4], b: &[u64; 4]) -> bool {
        ((a[0] & b[0]) | (a[1] & b[1]) | (a[2] & b[2]) | (a[3] & b[3])) != 0
    }

    /// Evaluates vector-wide bitmask disjointness between two columns.
    #[inline(always)]
    pub fn are_columns_disjoint(c1: &SppColumn, c2: &SppColumn) -> bool {
        c1.is_disjoint(c2)
    }

    /// Solves an SPP instance through the platform Coralys Decision Engine
    /// and domain DecisionPlugin adapter.
    pub fn solve(&self, instance: &SppInstance, restarts: usize, seed: u64) -> SppSolution {
        self.solve_evolutionary(instance, restarts, seed)
    }

    /// Solves an SPP instance using greedy search with 1 restart.
    pub fn solve_greedy(&self, instance: &SppInstance) -> SppSolution {
        self.solve_evolutionary(instance, 1, 42)
    }

    /// Solves an SPP instance using disjoint ejection chains with specified iterations.
    pub fn solve_disjoint_ejection(
        &self,
        instance: &SppInstance,
        max_iterations: usize,
        seed: u64,
    ) -> SppSolution {
        let mut plugin = SppDecisionPlugin::from_instance(instance.clone());
        let engine = CoralysSppEngine::default();
        let config = SppOptimizationConfig {
            restarts: 1,
            max_ejection_iterations: max_iterations,
            seed,
            enable_union_swaps: true,
        };

        let proposals = engine
            .solve(&mut plugin, &config)
            .expect("Coralys Decision Engine solve failed");

        if let Some(best) = proposals.first() {
            plugin.execute(best);
        }

        let final_state = plugin.current_state();
        let row_cov = SppDecisionPlugin::compute_row_coverage(
            &final_state.instance,
            &final_state.selected_cols,
        );
        let uncov = row_cov.iter().filter(|&&c| c == 0).count();
        let overcov = row_cov.iter().filter(|&&c| c > 1).count();

        SppSolution {
            selected_cols: final_state.selected_cols,
            total_cost: final_state.total_cost,
            row_coverage: row_cov,
            uncovered_count: uncov,
            overcovered_count: overcov,
        }
    }

    /// Evolutionary Set Partitioning Solver delegating to [`CoralysSppEngine`].
    pub fn solve_evolutionary(
        &self,
        instance: &SppInstance,
        restarts: usize,
        seed: u64,
    ) -> SppSolution {
        let mut plugin = SppDecisionPlugin::from_instance(instance.clone());
        let engine = CoralysSppEngine::default();
        let config = SppOptimizationConfig {
            restarts: restarts.max(1),
            max_ejection_iterations: if instance.num_rows > 100 { 4000 } else { 2000 },
            seed,
            enable_union_swaps: true,
        };

        let proposals = engine
            .solve(&mut plugin, &config)
            .expect("Coralys Decision Engine solve failed");

        if let Some(best) = proposals.first() {
            plugin.execute(best);
        }

        let final_state = plugin.current_state();
        let row_cov = SppDecisionPlugin::compute_row_coverage(
            &final_state.instance,
            &final_state.selected_cols,
        );
        let uncov = row_cov.iter().filter(|&&c| c == 0).count();
        let overcov = row_cov.iter().filter(|&&c| c > 1).count();

        SppSolution {
            selected_cols: final_state.selected_cols,
            total_cost: final_state.total_cost,
            row_coverage: row_cov,
            uncovered_count: uncov,
            overcovered_count: overcov,
        }
    }
}

/// Type alias for backward compatibility.
pub type SppSolver = SetPartitioningEvaluator;

/// Lookup table of certified exact integer global optimal objective values
/// published in the literature (Hoffman & Padberg 1993, Chu & Beasley 1998, OR-Library).
pub fn get_certified_optimum(instance_name: &str) -> Option<u64> {
    let name = instance_name.to_lowercase();
    match name.as_str() {
        "sppnw01" => Some(114_852),
        "sppnw02" => Some(105_444),
        "sppnw03" => Some(24_492),
        "sppnw04" => Some(16_862),
        "sppnw05" => Some(132_360),
        "sppnw06" => Some(7_810),
        "sppnw07" => Some(5_476),
        "sppnw08" => Some(35_894),
        "sppnw09" => Some(67_760),
        "sppnw10" => Some(68_271),
        "sppnw11" => Some(116_256),
        "sppnw12" => Some(14_118),
        "sppnw13" => Some(50_146),
        "sppnw14" => Some(61_844),
        "sppnw15" => Some(67_743),
        "sppnw16" => Some(1_181_590),
        "sppnw17" => Some(11_115),
        "sppnw18" => Some(340_160),
        "sppnw19" => Some(10_898),
        "sppnw20" => Some(16_812),
        "sppnw21" => Some(7_408),
        "sppnw22" => Some(6_984),
        "sppnw23" => Some(12_534),
        "sppnw24" => Some(6_314),
        "sppnw25" => Some(5_960),
        "sppnw26" => Some(6_796),
        "sppnw27" => Some(9_933),
        "sppnw28" => Some(8_298),
        "sppnw29" => Some(4_274),
        "sppnw30" => Some(3_942),
        "sppnw31" => Some(8_038),
        "sppnw32" => Some(14_877),
        "sppnw33" => Some(6_678),
        "sppnw34" => Some(10_488),
        "sppnw35" => Some(7_216),
        "sppnw36" => Some(7_314),
        "sppnw37" => Some(10_068),
        "sppnw38" => Some(5_558),
        "sppnw39" => Some(10_080),
        "sppnw40" => Some(10_809),
        "sppnw41" => Some(11_307),
        "sppnw42" => Some(7_656),
        "sppnw43" => Some(8_904),
        "sppaa01" => Some(50_558),
        "sppaa02" => Some(44_777),
        "sppaa03" => Some(47_878),
        "sppaa04" => Some(57_087),
        "sppaa05" => Some(54_909),
        "sppaa06" => Some(62_544),
        _ => None,
    }
}
