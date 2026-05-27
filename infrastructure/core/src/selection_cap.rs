//! Shared Top-K selection config for pipeline portfolio cap and GA aggregation.
//! - `SIGNAL_TOP_K` — pipeline portfolio cap (see `resolved_signal_top_k`).
//! - `GA_SCENARIO_TOP_K` — GA-only scenario cap (see `resolved_ga_scenario_top_k`); adds selection pressure when unset.
//!   Shorthand: `GA_TOPK` (same semantics) if `GA_SCENARIO_TOP_K` is not set — matches `GA_TOPK:` log line in `ga`.
//! - `RANK_SCORE_EDGE_GAMMA` — sharpens ranking when confidence saturates (see `rank_score_edge_confidence`).
//! - `GA_WEIGHTED_SCENARIO_PNL` — weighted vs mean aggregation over scenarios in GA fitness (see `resolved_ga_weighted_scenario_pnl`).
//! - `GA_DIVERSITY_LAMBDA` — GA Top-K only: strength of execution-signature diversity (see `resolved_ga_diversity_lambda`). Default `0` = pure rank order.
//! - `GA_DIVERSITY_MODE` — `repel` (default) = reward different execution paths; `attract` = cluster like the first pick (see [`GaDiversityMode`] / `resolved_ga_diversity_mode`).
//! - `GA_MAX_TRADES_PER_SCENARIO` — cap on non-overlapping round-trips per scenario in GA `evaluate_strategy` (default `10`, clamped).
//! - `GA_TRADE_COOLDOWN` — market-event indices to skip after each exit before the next entry attempt (default `3`, clamped).
//! - `GA_PARALLELISM` — threads for **genome-level** GA population evaluation (`evaluate_population_scoped`). `1` / unset / `0` = sequential; `N≥2` = Rayon pool with `N` threads; `auto` = `available_parallelism` clamped to `[2, 64]`. **Unit tests in this crate always use 1 thread** (`cfg(test)`).

use std::env;

/// Default global cap on ranked scenarios/signals. Override with `SIGNAL_TOP_K`; set to `0` for no cap.
pub const DEFAULT_SIGNAL_TOP_K: usize = 5;

/// When `GA_SCENARIO_TOP_K` is unset, GA uses `min(SIGNAL_TOP_K, DEFAULT_GA_SCENARIO_TOP_K)` so aggregation stays tighter than a large pipeline K.
pub const DEFAULT_GA_SCENARIO_TOP_K: usize = 3;

/// Rank key for pipeline `TradeSignal.rank_score` and GA scenario ordering:
/// `confidence * max(0, edge)^gamma` with `gamma` from [`resolved_rank_score_edge_gamma`] (default `1.0` = prior linear edge weighting).
#[inline]
pub fn rank_score_edge_confidence(edge: f64, confidence: f64) -> f64 {
    rank_score_edge_confidence_with_gamma(edge, confidence, resolved_rank_score_edge_gamma())
}

#[inline]
fn rank_score_edge_confidence_with_gamma(edge: f64, confidence: f64, gamma: f64) -> f64 {
    confidence * edge.max(0.0).powf(gamma)
}

/// Exponent on edge in rank score (`confidence * edge^gamma`). Default `1.0` (unchanged behavior).
/// Values `> 1` spread ordering when raw confidence is saturated. Clamped to `[0.5, 3.0]`.
pub fn resolved_rank_score_edge_gamma() -> f64 {
    env::var("RANK_SCORE_EDGE_GAMMA")
        .ok()
        .and_then(|s| s.trim().parse::<f64>().ok())
        .unwrap_or(1.0)
        .clamp(0.5, 3.0)
}

/// Global Top-K used by pipeline (after gating) and GA (`aggregate_strategy_reports`).
/// - Unset → `DEFAULT_SIGNAL_TOP_K`.
/// - `SIGNAL_TOP_K=0` → no cap (use all ranked items).
/// - `SIGNAL_TOP_K=N` → keep top N by rank score.
pub fn resolved_signal_top_k() -> Option<usize> {
    match env::var("SIGNAL_TOP_K") {
        Ok(s) => {
            let t = s.trim();
            if t.is_empty() {
                return Some(DEFAULT_SIGNAL_TOP_K);
            }
            let k: usize = t.parse().ok()?;
            if k == 0 {
                None
            } else {
                Some(k)
            }
        }
        Err(_) => Some(DEFAULT_SIGNAL_TOP_K),
    }
}

/// Top-K for **GA aggregation only** (`aggregate_strategy_reports`), not the live pipeline.
///
/// - `GA_SCENARIO_TOP_K=0` → no cap (use all scenario evaluations).
/// - `GA_SCENARIO_TOP_K=N` → keep top N scenarios by rank score before fitness aggregation.
/// - Unset → `min(resolved_signal_top_k(), DEFAULT_GA_SCENARIO_TOP_K)` with `None` (pipeline unlimited) mapping to `DEFAULT_GA_SCENARIO_TOP_K`.
/// - If `GA_SCENARIO_TOP_K` is unset, **`GA_TOPK`** is read as an alias (same syntax as above).
pub fn resolved_ga_scenario_top_k() -> Option<usize> {
    if let Ok(s) = env::var("GA_SCENARIO_TOP_K") {
        return parse_ga_top_k_value_or_fallback(&s);
    }
    if let Ok(s) = env::var("GA_TOPK") {
        return parse_ga_top_k_value_or_fallback(&s);
    }
    ga_scenario_top_k_fallback()
}

fn parse_ga_top_k_value_or_fallback(raw: &str) -> Option<usize> {
    let t = raw.trim();
    if t.is_empty() {
        return ga_scenario_top_k_fallback();
    }
    let k: usize = t.parse().ok()?;
    if k == 0 {
        None
    } else {
        Some(k)
    }
}

fn ga_scenario_top_k_fallback() -> Option<usize> {
    match resolved_signal_top_k() {
        None => Some(DEFAULT_GA_SCENARIO_TOP_K),
        Some(k) => Some(k.min(DEFAULT_GA_SCENARIO_TOP_K)),
    }
}

/// When `true`, GA aggregates scenario `avg_pnl` (and matching variance) with weights equal to the same
/// rank score used for scenario Top-K (`edge × conf^γ`). Default **`false`**: legacy unweighted mean
/// across all scenarios so participation and sparsity behave as before.
///
/// Enable with `GA_WEIGHTED_SCENARIO_PNL=1`, `true`, `yes`, or `on`.
pub fn resolved_ga_weighted_scenario_pnl() -> bool {
    match env::var("GA_WEIGHTED_SCENARIO_PNL") {
        Ok(s) => {
            let t = s.trim().to_lowercase();
            t == "1" || t == "true" || t == "yes" || t == "on"
        }
        Err(_) => false,
    }
}

/// How GA Top-K combines rank with mean L1 distance to already-selected execution signatures (`ga` module).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GaDiversityMode {
    /// `adjusted = rank − λ * mean_dist` — prefer execution profiles similar to prior picks.
    Attract,
    /// `adjusted = rank + λ * mean_dist` — prefer different execution regimes (default).
    Repel,
}

/// - Unset or unrecognized → **`Repel`** (exploration across execution paths).
/// - `attract` / `cluster` → [`GaDiversityMode::Attract`].
/// - `repel` / `explore` / `exploration` → [`GaDiversityMode::Repel`].
pub fn resolved_ga_diversity_mode() -> GaDiversityMode {
    match env::var("GA_DIVERSITY_MODE") {
        Ok(s) => match s.trim().to_lowercase().as_str() {
            "attract" | "cluster" => GaDiversityMode::Attract,
            "repel" | "explore" | "exploration" => GaDiversityMode::Repel,
            _ => GaDiversityMode::Repel,
        },
        Err(_) => GaDiversityMode::Repel,
    }
}

/// Greedy diversity strength for GA scenario Top-K (`apply_ga_top_k_selection` in `ga`).
/// Uses **mean** L1 distance to selected signatures: `mean_dist = (Σ d(sig, sig_j)) / n_selected`.
/// Combined with [`resolved_ga_diversity_mode`]: attract → `rank − λ * mean_dist`, repel → `rank + λ * mean_dist`.
/// - Unset or invalid → **`0.0`** (same as sorting by rank score descending; no behavior change).
/// - Clamped to `[0.0, 1.0]`. Try `0.1`–`0.2` with `repel` for regime spread.
pub fn resolved_ga_diversity_lambda() -> f64 {
    env::var("GA_DIVERSITY_LAMBDA")
        .ok()
        .and_then(|s| s.trim().parse::<f64>().ok())
        .unwrap_or(0.0)
        .clamp(0.0, 1.0)
}

/// Max completed round-trips per scenario in GA evaluation. Default **10**; clamped to `[1, 64]`.
pub fn resolved_ga_max_trades_per_scenario() -> usize {
    env::var("GA_MAX_TRADES_PER_SCENARIO")
        .ok()
        .and_then(|s| s.trim().parse::<usize>().ok())
        .unwrap_or(10)
        .clamp(1, 64)
}

/// Event-index cooldown after a trade’s exit event before the next entry cursor. Default **3**; clamped to `[0, 256]`.
pub fn resolved_ga_trade_cooldown() -> usize {
    env::var("GA_TRADE_COOLDOWN")
        .ok()
        .and_then(|s| s.trim().parse::<usize>().ok())
        .unwrap_or(3)
        .clamp(0, 256)
}

/// Parse `GA_PARALLELISM` (see module doc). Used by tests; production uses [`resolved_ga_parallelism_threads`].
pub(crate) fn parse_ga_parallelism_env_value(raw: &str) -> usize {
    let t = raw.trim();
    if t.is_empty() {
        return 1;
    }
    if t.eq_ignore_ascii_case("auto") {
        return std::thread::available_parallelism()
            .map(|n| n.get().clamp(2, 64))
            .unwrap_or(4);
    }
    let n: usize = t.parse().unwrap_or(1);
    if n <= 1 {
        1
    } else {
        n.min(256)
    }
}

/// Thread count for parallel genome evaluation in GA. **Library unit tests:** always `1` (sequential).
/// Unset env → `1` (unchanged single-threaded behavior unless `GA_PARALLELISM` is set).
#[cfg(test)]
pub fn resolved_ga_parallelism_threads() -> usize {
    1
}

#[cfg(not(test))]
pub fn resolved_ga_parallelism_threads() -> usize {
    env::var("GA_PARALLELISM")
        .map(|s| parse_ga_parallelism_env_value(&s))
        .unwrap_or(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ga_diversity_lambda_resolves() {
        use std::sync::Mutex;
        static ENV_LOCK: Mutex<()> = Mutex::new(());
        let _g = ENV_LOCK.lock().unwrap();
        std::env::remove_var("GA_DIVERSITY_LAMBDA");
        assert_eq!(resolved_ga_diversity_lambda(), 0.0);
        std::env::set_var("GA_DIVERSITY_LAMBDA", "0.25");
        assert!((resolved_ga_diversity_lambda() - 0.25).abs() < 1e-12);
        std::env::set_var("GA_DIVERSITY_LAMBDA", "2");
        assert_eq!(resolved_ga_diversity_lambda(), 1.0);
        std::env::set_var("GA_DIVERSITY_LAMBDA", "-1");
        assert_eq!(resolved_ga_diversity_lambda(), 0.0);
        std::env::remove_var("GA_DIVERSITY_LAMBDA");
    }

    #[test]
    fn ga_diversity_mode_resolves() {
        use std::sync::Mutex;
        static ENV_LOCK: Mutex<()> = Mutex::new(());
        let _g = ENV_LOCK.lock().unwrap();
        std::env::remove_var("GA_DIVERSITY_MODE");
        assert_eq!(resolved_ga_diversity_mode(), GaDiversityMode::Repel);
        std::env::set_var("GA_DIVERSITY_MODE", "attract");
        assert_eq!(resolved_ga_diversity_mode(), GaDiversityMode::Attract);
        std::env::set_var("GA_DIVERSITY_MODE", "cluster");
        assert_eq!(resolved_ga_diversity_mode(), GaDiversityMode::Attract);
        std::env::set_var("GA_DIVERSITY_MODE", "repel");
        assert_eq!(resolved_ga_diversity_mode(), GaDiversityMode::Repel);
        std::env::set_var("GA_DIVERSITY_MODE", "explore");
        assert_eq!(resolved_ga_diversity_mode(), GaDiversityMode::Repel);
        std::env::set_var("GA_DIVERSITY_MODE", "bogus");
        assert_eq!(resolved_ga_diversity_mode(), GaDiversityMode::Repel);
        std::env::remove_var("GA_DIVERSITY_MODE");
    }

    #[test]
    fn ga_parallelism_parses() {
        assert_eq!(parse_ga_parallelism_env_value(""), 1);
        assert_eq!(parse_ga_parallelism_env_value("0"), 1);
        assert_eq!(parse_ga_parallelism_env_value("1"), 1);
        assert_eq!(parse_ga_parallelism_env_value("4"), 4);
        assert_eq!(parse_ga_parallelism_env_value("  8 "), 8);
        assert_eq!(parse_ga_parallelism_env_value("9999"), 256);
        let _auto = parse_ga_parallelism_env_value("AuTo");
        assert!((_auto >= 2 && _auto <= 64) || _auto == 4);
    }

    #[test]
    fn ga_multi_trade_env_resolves() {
        use std::sync::Mutex;
        static ENV_LOCK: Mutex<()> = Mutex::new(());
        let _g = ENV_LOCK.lock().unwrap();
        std::env::remove_var("GA_MAX_TRADES_PER_SCENARIO");
        std::env::remove_var("GA_TRADE_COOLDOWN");
        assert_eq!(resolved_ga_max_trades_per_scenario(), 10);
        assert_eq!(resolved_ga_trade_cooldown(), 3);
        std::env::set_var("GA_MAX_TRADES_PER_SCENARIO", "5");
        std::env::set_var("GA_TRADE_COOLDOWN", "0");
        assert_eq!(resolved_ga_max_trades_per_scenario(), 5);
        assert_eq!(resolved_ga_trade_cooldown(), 0);
        std::env::set_var("GA_MAX_TRADES_PER_SCENARIO", "999");
        assert_eq!(resolved_ga_max_trades_per_scenario(), 64);
        std::env::remove_var("GA_MAX_TRADES_PER_SCENARIO");
        std::env::remove_var("GA_TRADE_COOLDOWN");
    }

    #[test]
    fn rank_score_gamma_one_matches_linear_product() {
        let e = 0.012_f64;
        let c = 0.55_f64;
        assert!((rank_score_edge_confidence_with_gamma(e, c, 1.0) - c * e).abs() < 1e-15);
    }

    #[test]
    fn rank_score_gamma_sharpens_edge_spread() {
        let c = 1.0_f64;
        let low = 0.008_f64;
        let high = 0.012_f64;
        let g = 1.2_f64;
        let r_low = rank_score_edge_confidence_with_gamma(low, c, g);
        let r_high = rank_score_edge_confidence_with_gamma(high, c, g);
        let ratio_linear = high / low;
        let ratio_gamma = r_high / r_low;
        assert!(ratio_gamma > ratio_linear - 1e-9);
    }
}
