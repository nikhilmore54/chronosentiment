use crate::Side;
use serde::{Deserialize, Serialize};
use crate::ga::{StrategyEvaluation};
use std::collections::HashMap;

/// [V4.1.0] Final structured result of the Recommendation Engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationResult {
    /// High-confidence trade signal surviving all gates.
    Trade(Recommendation),
    /// Emerging consensus or decent execution, but not meeting the "Trade" threshold.
    WeakSignal(Recommendation),
    /// Explicit refusal to trade with a structured reason.
    NoTrade {
        reason: NoTradeReason,
        metrics: RecoMetrics,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NoTradeReason {
    LowConsensus,
    LowStability,
    PoorExecution,
    LowCapture,
    EmptyCandidatePool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub symbol: String,
    pub action: Side,
    pub confidence: ConfidenceDecomposition,
    pub expected_move: f64,
    pub capture_efficiency: f64,
    pub execution: ExecutionSummary,
    pub consensus: ConsensusSummary,
    /// Population / consensus basis used to build this recommendation (for live gating and audits).
    #[serde(default)]
    pub ensemble_metrics: RecoMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceDecomposition {
    pub total: f64,
    pub consensus: f64,
    pub execution: f64,
    pub capture: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSummary {
    pub score: f64,
    pub fill_probability: f64,
    pub latency_impact: f64,
    pub slippage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusSummary {
    pub dominant_axes: (u8, u8, u8, u8),
    pub agreement_score: f64,
    pub stability_score: f64,
    pub energy: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecoMetrics {
    pub agreement: f64,
    pub agreement_global: f64,
    pub agreement_local: f64,
    pub stability: f64,
    pub cohesion: f64,
    pub execution_score: f64,
    pub capture_efficiency: f64,
    /// Medoid genome fitness after clustering; 0 if no medoid was selected.
    #[serde(default)]
    pub medoid_fitness: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoConfig {
    pub min_agreement: f64,
    pub min_stability: f64,
    pub min_execution_score: f64,
    pub min_capture_efficiency: f64,
    pub sim_horizon_events: usize,
    pub min_pool_size: usize,
}

impl Default for RecoConfig {
    fn default() -> Self {
        Self {
            min_agreement: 0.4,
            min_stability: 0.5,
            min_execution_score: 0.3,
            min_capture_efficiency: 0.65,
            sim_horizon_events: 50,
            min_pool_size: 5,
        }
    }
}

pub struct RecommendationEngine;

impl RecommendationEngine {
    /// [V4.1.0] Main entry point for the recommendation decision layer.
    pub fn process(
        population: &[StrategyEvaluation],
        _market: &[crate::MarketEvent],
        config: &RecoConfig,
        symbol: &str,
    ) -> RecommendationResult {
        // 1. Calculate population stats and filter candidates
        let (mean, std) = crate::ga::calculate_population_stats(population);
        let mut candidates: Vec<&StrategyEvaluation> = population
            .iter()
            .filter(|e| e.fitness > mean + 0.5 * std)
            .collect();

        // 2. Safeguard: if the strict tail is too small, expand the pool by fitness rank.
        // IMPORTANT: do not use only ceil(20% × n) — for n=5 that is 1 genome, which forces
        // identical agreement_global (1/n) and agreement_local (1) for every asset in the dashboard.
        if candidates.len() < config.min_pool_size {
            let mut all: Vec<&StrategyEvaluation> = population.iter().collect();
            all.sort_by(|a, b| b.fitness.total_cmp(&a.fitness));
            let n = population.len();
            let k_tail = (n as f64 * 0.2).ceil() as usize;
            let k_need = config.min_pool_size.min(n).max(k_tail).max(1);
            candidates = all.into_iter().take(k_need).collect();
        }

        if candidates.is_empty() {
            return RecommendationResult::NoTrade {
                reason: NoTradeReason::EmptyCandidatePool,
                metrics: RecoMetrics::default(),
            };
        }

        if std::env::var("POOL_DEBUG").map_or(false, |v| {
            !v.is_empty() && v != "0" && v.to_lowercase() != "false"
        }) {
            let fs: Vec<f64> = candidates.iter().map(|e| e.fitness).collect();
            let mm = fs.iter().sum::<f64>() / fs.len() as f64;
            let var = fs.iter().map(|f| (f - mm).powi(2)).sum::<f64>() / fs.len() as f64;
            let sd = var.sqrt();
            println!(
                "POOL_DEBUG sym={} pool_size={} fitness_mean={:.6} fitness_std={:.6} values={:?}",
                symbol,
                candidates.len(),
                mm,
                sd,
                fs
            );
        }

        // 3. Cluster by behavioral axes
        let mut clusters: std::collections::HashMap<(u8, u8, u8, u8), Vec<&StrategyEvaluation>> = std::collections::HashMap::new();
        for &e in &candidates {
            clusters.entry(e.behavioral_signature.axes).or_insert(vec![]).push(e);
        }

        // 4. Score clusters and select dominant
        let mut best_cluster_axes = None;
        let mut max_energy = -1.0;
        let mut best_metrics = RecoMetrics::default();

        let pool_size = candidates.len() as f64;

        let total_pop_size = population.len() as f64;

        for (axes, members) in &clusters {
            let size = members.len() as f64;
            let avg_fitness = members.iter().map(|e| e.fitness).sum::<f64>() / size;
            
            // Stability: cluster fitness dispersion; if the cluster is a singleton (or numerically flat),
            // use population-wide fitness std so different assets get different scores.
            let cluster_std = (members.iter().map(|e| (e.fitness - avg_fitness).powi(2)).sum::<f64>() / size).sqrt();
            let stability = if size >= 2.0 && cluster_std > 1e-9 {
                (avg_fitness / (cluster_std + 1e-6)).min(5.0)
            } else {
                let pn = population.len().max(1) as f64;
                let pop_mean = population.iter().map(|e| e.fitness).sum::<f64>() / pn;
                let pop_std = (population
                    .iter()
                    .map(|e| (e.fitness - pop_mean).powi(2))
                    .sum::<f64>()
                    / pn)
                    .sqrt();
                (avg_fitness / (pop_std + 1e-6)).min(5.0)
            };

            // Coherence: 1.0 - Normalized Genomic Variance
            let mut sum_var = 0.0;
            if size > 1.0 {
                for i in 0..members.len() {
                    for j in i + 1..members.len() {
                        sum_var += crate::ga::calculate_genotype_distance_normalized(&members[i].strategy, &members[j].strategy);
                    }
                }
                sum_var = (sum_var * 2.0) / (size * (size - 1.0));
            }
            let cohesion = 1.0 - sum_var;

            // Consensus Energy formula (V4.1.0 Locked)
            let energy = (1.0 + size).ln() * avg_fitness * stability * cohesion;

            // Adaptive Dual-Awareness Consensus: Smooth Regime Interpolation
            let agreement_global = size / total_pop_size;
            let agreement_local = if pool_size > 0.0 { size / pool_size } else { 0.0 };
            
            // Continuous alpha blend: 0 -> Discovery (elite), 1 -> Confirmation (collective)
            let alpha = ((agreement_global - 0.2) / 0.2).clamp(0.0, 1.0);
            
            // Interpolate weights: G (0.4 -> 0.7), L (0.6 -> 0.3)
            let weight_global = 0.4 + alpha * (0.7 - 0.4);
            let weight_local  = 0.6 - alpha * (0.6 - 0.3);
            
            let combined_agreement = weight_global * agreement_global + weight_local * agreement_local;

            if energy > max_energy {
                max_energy = energy;
                best_cluster_axes = Some(*axes);
                best_metrics = RecoMetrics {
                    agreement: combined_agreement,
                    agreement_global,
                    agreement_local,
                    stability,
                    cohesion,
                    execution_score: 0.0,
                    capture_efficiency: 0.0,
                    medoid_fitness: 0.0,
                };
            }
        }

        // --- PHASE D.1.22: ENTROPY CALCULATION (Pool-wide) ---
        let mut axis_counts = HashMap::new();
        for eval in &candidates {
            *axis_counts.entry(eval.behavioral_signature.axes).or_insert(0) += 1;
        }
        let total_cands = candidates.len() as f64;
        let mut entropy = 0.0;
        for &count in axis_counts.values() {
            let p = count as f64 / total_cands;
            if p > 0.0 {
                entropy -= p * p.ln();
            }
        }
        let max_entropy = (axis_counts.len() as f64).ln().max(1e-6);
        let normalized_entropy = (entropy / max_entropy).clamp(0.0, 1.0);
        let entropy_penalty = 1.0 - normalized_entropy;

        let axes = match best_cluster_axes {
            Some(a) => a,
            None => return RecommendationResult::NoTrade { reason: NoTradeReason::LowConsensus, metrics: best_metrics },
        };

        // 5. Select Medoid (Strategy closest to cluster center with tie-break)
        let cluster_members = clusters.get(&axes).unwrap();
        let medoid_eval = match Self::calculate_medoid(cluster_members) {
            Some(m) => m,
            None => return RecommendationResult::NoTrade { reason: NoTradeReason::LowConsensus, metrics: best_metrics },
        };

        // 7. Derive execution quality from GA's proven ExecutionMetrics (not shadow sim).
        //    The shadow sim fires on every market event without a real signal trigger, so its
        //    fill_count is always 0 → disconnected from reality.  We use what the GA proved.
        let exec_summary = Self::build_execution_summary_from_ga(medoid_eval);
        best_metrics.execution_score = exec_summary.score;

        // 9. Rolling Capture Efficiency (Using strategy's proven capture).
        best_metrics.capture_efficiency = medoid_eval.execution_metrics.capture_efficiency;
        best_metrics.medoid_fitness = medoid_eval.fitness;

        // 10. Gating Logic
        let action = if medoid_eval.avg_conviction > 0.0 { Side::Buy } else { Side::Sell };
        
        // 🔥 Consensus Tension: Context-Aware Maturity
        let g = best_metrics.agreement_global;
        let l = best_metrics.agreement_local;
        let stability = best_metrics.stability;
        let cohesion = best_metrics.cohesion;
        
        // Maturity Signal: Blend broad popularity with internal consistency & variance
        let stab_norm = (stability / 5.0).clamp(0.0, 1.0); // 2.0-5.0 range
        let variance_penalty = cohesion.clamp(0.5, 1.0); // Cohesion is 1.0 - genomic variance
        let stability_signal = stab_norm.sqrt() * variance_penalty;
        
        let maturity = (0.6 * g + 0.4 * stability_signal).clamp(0.0, 1.0);
        
        // Continuous alpha blend: 0 -> Discovery (elite), 1 -> Confirmation (collective)
        let alpha = ((maturity - 0.2) / 0.2).clamp(0.0, 1.0);
        
        // Interpolate weights: G (0.4 -> 0.7), L (0.6 -> 0.3)
        let weight_global = 0.4 + alpha * (0.7 - 0.4);
        let weight_local  = 0.6 - alpha * (0.6 - 0.3);
        
        let agreement = weight_global * g + weight_local * l;
        let raw_consensus = agreement * agreement;
        
        // Final Consensus Confidence with Entropy Penalty (Phase D.1.22)
        let diversity_factor = (stability / 2.0).clamp(0.7, 1.0);
        let mut consensus_conf = (raw_consensus * diversity_factor).clamp(0.1, 0.95);
        
        // Apply Entropy Penalty: punish unoriginal/homogeneous consensus
        consensus_conf *= 1.0 - 0.3 * entropy_penalty;
        
        let execution_conf = best_metrics.execution_score.min(0.95);
        let capture_conf = ((best_metrics.capture_efficiency + 1.0) / 2.0).clamp(0.0, 1.0);
        
        let total_conf = 0.45 * consensus_conf + 0.35 * execution_conf + 0.20 * capture_conf;

        // Optional: very chatty; enable with RECO_DEBUG=1 when tuning the reco layer.
        if std::env::var("RECO_DEBUG").map_or(false, |v| {
            !v.is_empty() && v != "0" && v.to_lowercase() != "false"
        }) {
            println!(
                "CONF_BREAKDOWN sym={} → G={:.3} L={:.3} STAB={:.3} MAT={:.3} α={:.3} H={:.3} FINAL={:.3} | E={:.3} R={:.3} TOTAL={:.3}",
                symbol, g, l, stability, maturity, alpha, normalized_entropy, consensus_conf,
                execution_conf, capture_conf, total_conf
            );
        }

        // `ensemble_metrics` == `best_metrics` for this tick: same winning cluster, same medoid (S/G/F coherent).
        let rec = Recommendation {
            symbol: symbol.to_string(),
            action,
            confidence: ConfidenceDecomposition {
                total: total_conf,
                consensus: consensus_conf,
                execution: execution_conf,
                capture: capture_conf,
            },
            expected_move: medoid_eval.avg_pnl,
            capture_efficiency: best_metrics.capture_efficiency,
            execution: exec_summary,
            consensus: ConsensusSummary {
                dominant_axes: axes,
                agreement_score: best_metrics.agreement,
                stability_score: best_metrics.stability,
                energy: max_energy,
            },
            ensemble_metrics: best_metrics.clone(),
        };

        // Emit final Decision
        if best_metrics.agreement >= config.min_agreement 
            && best_metrics.stability >= config.min_stability
            && best_metrics.execution_score >= config.min_execution_score
            && best_metrics.capture_efficiency >= config.min_capture_efficiency 
        {
            RecommendationResult::Trade(rec)
        } else if best_metrics.agreement >= config.min_agreement * 0.8 && total_conf > 0.6 {
            RecommendationResult::WeakSignal(rec)
        } else {
            let reason = if best_metrics.agreement < config.min_agreement { NoTradeReason::LowConsensus }
                        else if best_metrics.stability < config.min_stability { NoTradeReason::LowStability }
                        else if best_metrics.execution_score < config.min_execution_score { NoTradeReason::PoorExecution }
                        else { NoTradeReason::LowCapture };
            RecommendationResult::NoTrade { reason, metrics: best_metrics }
        }
    }

    fn calculate_medoid<'a>(members: &[&'a StrategyEvaluation]) -> Option<&'a StrategyEvaluation> {
        if members.is_empty() { return None; }
        if members.len() == 1 { return Some(members[0]); }

        let mut min_total_dist = f64::MAX;
        let mut best_member = members[0];

        for i in 0..members.len() {
            let mut total_dist = 0.0;
            for j in 0..members.len() {
                if i == j { continue; }
                total_dist += crate::ga::calculate_genotype_distance_normalized(&members[i].strategy, &members[j].strategy);
            }

            if total_dist < min_total_dist {
                min_total_dist = total_dist;
                best_member = members[i];
            } else if (total_dist - min_total_dist).abs() < 1e-9 {
                // Tie-breaker: Higher fitness wins
                if members[i].fitness > best_member.fitness {
                    best_member = members[i];
                }
            }
        }

        Some(best_member)
    }

    /// Build an ExecutionSummary directly from the GA's proven execution metrics.
    ///
    /// The old shadow-sim approach fired an intent on *every* market event regardless
    /// of whether the strategy would actually signal at that point, so fill_count was
    /// always 0 and the reco layer always saw fill_probability = 0.00.  This function
    /// reads the fields the GA already computed during its full round-trip evaluation.
    fn build_execution_summary_from_ga(eval: &StrategyEvaluation) -> ExecutionSummary {
        let em = &eval.execution_metrics;

        // fill_probability: prefer the GA's measured fill_rate; if zero but trades
        // actually happened, treat execution as proven (1.0) so the reco layer
        // doesn't veto something the GA already demonstrated works.
        let fill_probability = if eval.trade_count > 0 && em.fill_rate == 0.0 {
            1.0  // GA recorded real trades; old fill_rate was not updated — trust trades
        } else {
            em.fill_rate as f64  // Measured fill rate from GA round-trip
        };

        let latency_impact = (em.latency_impact).clamp(0.0, 1.0);
        let avg_slippage   = em.avg_slippage.clamp(0.0, 1.0);

        // Replicate the same weighting formula so the score is comparable
        let score = 0.5 * fill_probability
            + 0.3 * (1.0 - latency_impact).max(0.0)
            + 0.2 * (1.0 - avg_slippage);

        ExecutionSummary {
            score,
            fill_probability,
            latency_impact,
            slippage: avg_slippage,
        }
    }
}
