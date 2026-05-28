use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::domain::Strategy as Candidate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetEvoState {
    pub symbol: String,
    pub max_log_queue: f64,
    pub prev_max_log_queue: f64,
    pub delta_log_q: f64,
    pub stability_streak: u32,
    pub trade_density: f64,
    pub fill_rate: f64,
    pub last_smoothed_fill: f64,
    pub mutation_scale: f64,
    pub last_weight: f64,
    pub stagnation_counter: u32,
    pub last_best_fitness: f64,
    pub rolling_variance: f64,
    pub initial_diversity: f64,
    pub current_diversity: f64,
    pub selection_pressure: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GlobalEvoState {
    pub expansion_bias: f64,
    pub agreement_ema: f64,
    pub stability_ema: f64,
    pub prev_stability_ema: f64,
    pub progress_ema: f64,
    pub energy_ema: f64,
    pub energy_ema_prev: f64,
    pub peak_energy_ema: f64,
    pub global_max_log_q: f64,
    pub frontier_velocity_ema: f64,
    pub peak_velocity_ema: f64,
    pub velocity_history: Vec<f64>,
    pub last_expansion_gen: usize,
    pub prev_converged_assets: usize,
    pub post_strike_cooldown: u32,
    pub agreement_streak: u32,
    pub soft_expansion_active: bool,
    pub alignment_anchor: Option<Candidate>,
    pub global_mean: Option<Candidate>,
    pub pull_strength: f64,
    pub asset_states: HashMap<String, AssetEvoState>,
}
