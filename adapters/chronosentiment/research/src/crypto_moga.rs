//! Coralys MOGA Crypto Decision Support & Policy Freeze Integration
//!
//! Provides adapter-side integration connecting `coralys-moga` and `RuleListGenome`
//! to evolve and freeze Policy B (Base) and Policy C (With Exhaustion Telemetry)
//! on Binance Spot BTC/USDT 15-minute bars under disciplined value trading principles.
//!
//! RP-600 Governance Pre-Check: Reuses existing `coralys-moga` engine and `RuleListGenome`
//! representations without duplicating core optimization concepts or modifying `coralys-*` crates.

use coralys_moga::config::EvolutionConfig;
use coralys_moga::engine::EvolutionEngineBuilder;
use coralys_moga::runtime::optimization::metric::MetricReport;
use coralys_moga::traits::{
    Evaluated, FitnessEvaluator, GenomeFactory, MutationOperator,
};
use rand::rngs::StdRng;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::decision_support::policy_artifact::{
    DecisionRule, FactorPredicate,
};
use crate::decision_support::policy_genome::{RuleListCrossover, RuleListGenome};
use crate::decision_support::{DecisionAction, EvidenceFactor};

use super::crypto_replay::{
    compute_causal_features, BtcCandle15m, CausalFeatureVector, CausalReplayEngine, PolicyAction,
};

/// Registered feature predicate alphabet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FactorAlphabet {
    /// Policy B: Base market indicators only (Trend, Momentum, Volatility).
    BaseOnly,
    /// Policy C: Base market indicators plus ALL THREE RP-608 Exhaustion Telemetry predicates
    /// (PriceRangeStretch, VWAPStretch, VolumeBurst).
    WithExhaustion,
}

impl FactorAlphabet {
    pub fn allowed_concepts(&self) -> &'static [&'static str] {
        match self {
            FactorAlphabet::BaseOnly => &["Trend", "Momentum", "Volatility"],
            FactorAlphabet::WithExhaustion => &[
                "Trend",
                "Momentum",
                "Volatility",
                "PriceRangeStretch",
                "VWAPStretch",
                "VolumeBurst",
            ],
        }
    }
}

/// Generates a random predicate conforming strictly to the specified alphabet.
pub fn random_crypto_predicate(alphabet: FactorAlphabet, rng: &mut StdRng) -> FactorPredicate {
    let concepts = alphabet.allowed_concepts();
    let concept = concepts[rng.gen_range(0..concepts.len())];

    match concept {
        "Trend" => FactorPredicate {
            concept: "Trend".to_string(),
            present: Some(true),
            direction: Some(["Bullish", "Bearish", "Neutral"][rng.gen_range(0..3)].to_string()),
        },
        "Momentum" => FactorPredicate {
            concept: "Momentum".to_string(),
            present: Some(true),
            direction: Some(["Positive", "Negative", "Neutral"][rng.gen_range(0..3)].to_string()),
        },
        "Volatility" => FactorPredicate {
            concept: "Volatility".to_string(),
            present: Some(rng.gen_bool(0.7)),
            direction: None,
        },
        "PriceRangeStretch" => FactorPredicate {
            concept: "PriceRangeStretch".to_string(),
            present: Some(true),
            direction: Some(["ExhaustedHigh", "ExhaustedLow"][rng.gen_range(0..2)].to_string()),
        },
        "VWAPStretch" => FactorPredicate {
            concept: "VWAPStretch".to_string(),
            present: Some(true),
            direction: Some(["Overbought", "Oversold"][rng.gen_range(0..2)].to_string()),
        },
        "VolumeBurst" => FactorPredicate {
            concept: "VolumeBurst".to_string(),
            present: Some(true),
            direction: Some("Burst".to_string()),
        },
        _ => unreachable!(),
    }
}

pub fn random_crypto_action(rng: &mut StdRng) -> DecisionAction {
    match rng.gen_range(0..3) {
        0 => DecisionAction::Long,
        1 => DecisionAction::Short,
        _ => DecisionAction::NoTrade,
    }
}

pub fn random_crypto_rule(alphabet: FactorAlphabet, rng: &mut StdRng) -> DecisionRule {
    let n = rng.gen_range(1..=3);
    let mut when = Vec::with_capacity(n);
    for _ in 0..n {
        when.push(random_crypto_predicate(alphabet, rng));
    }
    DecisionRule {
        when,
        action: random_crypto_action(rng),
    }
}

pub struct CryptoGenomeFactory {
    pub alphabet: FactorAlphabet,
}

impl GenomeFactory<RuleListGenome> for CryptoGenomeFactory {
    fn create(&self, rng: &mut StdRng) -> RuleListGenome {
        let n = rng.gen_range(1..=5);
        let rules = (0..n)
            .map(|_| random_crypto_rule(self.alphabet, rng))
            .collect();
        let mut genome = RuleListGenome {
            rules,
            unmatched_action: DecisionAction::NoTrade,
        };
        canonicalize_crypto_genome(&mut genome, self.alphabet);
        genome
    }
}

pub struct CryptoGenomeMutation {
    pub alphabet: FactorAlphabet,
}

impl MutationOperator<RuleListGenome> for CryptoGenomeMutation {
    fn mutate(&self, genome: &mut RuleListGenome, rng: &mut StdRng) {
        match rng.gen_range(0..5) {
            0 if !genome.rules.is_empty() => {
                let i = rng.gen_range(0..genome.rules.len());
                genome.rules[i].action = random_crypto_action(rng);
            }
            1 if !genome.rules.is_empty() => {
                let i = rng.gen_range(0..genome.rules.len());
                genome.rules[i] = random_crypto_rule(self.alphabet, rng);
            }
            2 if genome.rules.len() < 8 => {
                genome.rules.push(random_crypto_rule(self.alphabet, rng));
            }
            3 if genome.rules.len() > 1 => {
                let i = rng.gen_range(0..genome.rules.len());
                genome.rules.remove(i);
            }
            _ => genome.unmatched_action = random_crypto_action(rng),
        }
        canonicalize_crypto_genome(genome, self.alphabet);
    }
}

pub fn canonicalize_crypto_genome(genome: &mut RuleListGenome, alphabet: FactorAlphabet) {
    if genome.rules.len() > 8 {
        genome.rules.truncate(8);
    }
    let allowed = alphabet.allowed_concepts();
    for rule in &mut genome.rules {
        rule.when.retain(|p| allowed.contains(&p.concept.as_str()));
        rule.when.sort_by(|a, b| a.concept.cmp(&b.concept));
        rule.when.dedup();
        rule.when.retain(|p| {
            if p.concept == "Volatility" {
                p.direction.is_none()
            } else {
                true
            }
        });
    }
}

/// Precomputed causal bar state to ensure evaluation executes in sub-millisecond time.
#[derive(Debug, Clone)]
pub struct PrecomputedBarState {
    pub bar_idx: usize,
    pub candle: BtcCandle15m,
    pub atr_14: f64,
    pub factors: Vec<EvidenceFactor>,
    pub causal_feat: CausalFeatureVector,
}

/// Converts causal features into discrete EvidenceFactor values.
pub fn features_to_evidence(feat: &CausalFeatureVector, alphabet: FactorAlphabet) -> Vec<EvidenceFactor> {
    let mut factors = Vec::with_capacity(6);

    // Trend
    let trend_dir = if feat.ret_15m_bps > 10.0 {
        "Bullish"
    } else if feat.ret_15m_bps < -10.0 {
        "Bearish"
    } else {
        "Neutral"
    };
    factors.push(EvidenceFactor {
        concept: "Trend".to_string(),
        present: true,
        direction: Some(trend_dir.to_string()),
        strength: None,
        assessment_confidence: None,
    });

    // Momentum
    let mom_dir = if feat.ret_15m_bps > 2.0 {
        "Positive"
    } else if feat.ret_15m_bps < -2.0 {
        "Negative"
    } else {
        "Neutral"
    };
    factors.push(EvidenceFactor {
        concept: "Momentum".to_string(),
        present: true,
        direction: Some(mom_dir.to_string()),
        strength: None,
        assessment_confidence: None,
    });

    // Volatility
    let vol_present = (feat.atr_14 / feat.reference_price) > 0.0035;
    factors.push(EvidenceFactor {
        concept: "Volatility".to_string(),
        present: vol_present,
        direction: None,
        strength: None,
        assessment_confidence: None,
    });

    if alphabet == FactorAlphabet::WithExhaustion {
        // Exhaustion 1: PriceRangeStretch (P_ref relative to 20-bar Donchian channel)
        if let Some(pos) = feat.price_range_pos_20 {
            let (present, dir) = if pos > 0.90 {
                (true, Some("ExhaustedHigh".to_string()))
            } else if pos < 0.10 {
                (true, Some("ExhaustedLow".to_string()))
            } else {
                (false, None)
            };
            factors.push(EvidenceFactor {
                concept: "PriceRangeStretch".to_string(),
                present,
                direction: dir,
                strength: None,
                assessment_confidence: None,
            });
        }

        // Exhaustion 2: VWAPStretch (P_ref distance to rolling 96-bar VWAP)
        if let Some(dist) = feat.dist_vwap_bps {
            let (present, dir) = if dist > 60.0 {
                (true, Some("Overbought".to_string()))
            } else if dist < -60.0 {
                (true, Some("Oversold".to_string()))
            } else {
                (false, None)
            };
            factors.push(EvidenceFactor {
                concept: "VWAPStretch".to_string(),
                present,
                direction: dir,
                strength: None,
                assessment_confidence: None,
            });
        }

        // Exhaustion 3: VolumeBurst (Volume / 20-bar SMA(V))
        if let Some(burst) = feat.vol_burst_20 {
            let present = burst > 1.8;
            factors.push(EvidenceFactor {
                concept: "VolumeBurst".to_string(),
                present,
                direction: if present { Some("Burst".to_string()) } else { None },
                strength: None,
                assessment_confidence: None,
            });
        }
    }

    factors
}

/// Precomputes all bar states for a candle dataset.
pub fn precompute_bar_states(
    candles: &[BtcCandle15m],
    alphabet: FactorAlphabet,
) -> Vec<PrecomputedBarState> {
    let mut states = Vec::with_capacity(candles.len());
    for i in 0..candles.len() {
        let feat = compute_causal_features(candles, i).expect("causal features computed");
        let factors = features_to_evidence(&feat, alphabet);
        states.push(PrecomputedBarState {
            bar_idx: i,
            candle: candles[i].clone(),
            atr_14: feat.atr_14,
            factors,
            causal_feat: feat,
        });
    }
    states
}

/// Evaluates a FactorPredicate against current EvidenceFactors.
fn predicate_matches(pred: &FactorPredicate, factors: &[EvidenceFactor]) -> bool {
    let factor = factors.iter().find(|f| f.concept == pred.concept);
    match (pred.present, factor) {
        (Some(false), None) => pred.direction.is_none(),
        (Some(false), Some(f)) => !f.present && pred.direction.is_none(),
        (Some(true), Some(f)) if f.present => match &pred.direction {
            None => true,
            Some(dir) => f.direction.as_deref() == Some(dir.as_str()),
        },
        (Some(true), _) => false,
        (None, Some(f)) => match &pred.direction {
            None => true,
            Some(dir) => f.present && f.direction.as_deref() == Some(dir.as_str()),
        },
        (None, None) => pred.direction.is_none(),
    }
}

/// First-match evaluation of RuleListGenome.
pub fn evaluate_crypto_rules(
    genome: &RuleListGenome,
    factors: &[EvidenceFactor],
) -> DecisionAction {
    for rule in &genome.rules {
        if rule.when.iter().all(|p| predicate_matches(p, factors)) {
            return rule.action;
        }
    }
    genome.unmatched_action
}

/// Itemized MOGA Multi-Objective Evaluation Record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MogaObjectiveVector {
    /// Objective 1: Net Cumulative Return in USDT after all monetary friction.
    pub net_return_usdt: f64,
    /// Objective 1: Net Cumulative Return in basis points on reference capital.
    pub net_return_bps: f64,
    /// Objective 2: Maximum Percentage Drawdown from continuous mark-to-market equity curve.
    pub max_drawdown_pct: f64,
    /// Objective 2: Maximum Drawdown in USDT.
    pub max_drawdown_usdt: f64,
    /// Objective 3: Total Turnover Friction in USDT (itemized fees + slippage).
    pub turnover_friction_usdt: f64,
    /// Objective 3: Total round-trip transactions completed.
    pub trade_count: usize,
}

/// Causal snapshot summary at decision time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalTelemetrySummary {
    pub reference_price: f64,
    pub ret_15m_bps: f64,
    pub atr_14: f64,
    pub donchian_breakout: bool,
    pub price_range_pos: Option<f64>,
    pub dist_vwap_bps: Option<f64>,
    pub vol_burst: Option<f64>,
}

/// Decision-Level Economic and Discipline Telemetry Record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionTelemetryRecord {
    pub bar_idx: usize,
    pub timestamp_ms: i64,
    pub causal_inputs: CausalTelemetrySummary,
    pub proposed_action: String,
    pub execution_disposition: String,
    pub position_active: bool,
    pub realized_gross_pnl_usdt: f64,
    pub net_pnl_usdt: f64,
    pub cumulative_equity_usdt: f64,
    pub current_drawdown_pct: f64,
    pub bars_held: usize,
    pub invariants_passed: bool,
}

/// Evaluation record produced by the crypto evaluator implementing `Evaluated`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoGenomeEvaluation {
    pub genome: RuleListGenome,
    pub fitness: f64,
    pub objectives: MogaObjectiveVector,
    pub calmar_ratio: f64,
    pub win_rate: f64,
    pub rejected_entries: usize,
    pub valid: bool,
    #[serde(skip)]
    pub telemetry_log: Vec<DecisionTelemetryRecord>,
}

impl Evaluated for CryptoGenomeEvaluation {
    type Genome = RuleListGenome;
    fn fitness(&self) -> f64 {
        self.fitness
    }
    fn is_valid(&self) -> bool {
        self.valid
    }
    fn genome(&self) -> &Self::Genome {
        &self.genome
    }
}

/// Fitness evaluator implementing Coralys `FitnessEvaluator<RuleListGenome>`.
pub struct CryptoMogaEvaluator {
    pub precomputed_states: Vec<PrecomputedBarState>,
    pub reference_capital_usdt: f64,
    pub warmup_bars: usize,
}

impl CryptoMogaEvaluator {
    pub fn new(
        candles: &[BtcCandle15m],
        alphabet: FactorAlphabet,
        reference_capital_usdt: f64,
    ) -> Self {
        let precomputed_states = precompute_bar_states(candles, alphabet);
        Self {
            precomputed_states,
            reference_capital_usdt,
            warmup_bars: 96, // 24 hours warmup
        }
    }

    /// Evaluates a genome across the entire precomputed dataset using the exact replay engine.
    pub fn evaluate_genome(&self, genome: &RuleListGenome) -> CryptoGenomeEvaluation {
        let candles: Vec<BtcCandle15m> = self
            .precomputed_states
            .iter()
            .map(|s| s.candle.clone())
            .collect();
        let mut engine = CausalReplayEngine::new(self.reference_capital_usdt, candles);
        let mut telemetry = Vec::new();

        // Step through each bar chronologically
        for i in self.warmup_bars..self.precomputed_states.len() {
            let state = &self.precomputed_states[i];
            let action = evaluate_crypto_rules(genome, &state.factors);

            // Discipline & Candidate Eligibility Semantics:
            // 1. Flat and Long -> Propose EnterLong at next bar open
            // 2. Flat and Short/NoTrade -> StandAside (remains flat)
            // 3. Long and Short -> Propose ExitPosition at next bar open
            // 4. Long and Long -> Propose EnterLong -> REJECTED by capacity gate (stays in 1-unit position)
            // 5. Long and NoTrade -> StandAside (stays in position under trailing stop & time-exit)
            let policy_action = match action {
                DecisionAction::Long => PolicyAction::EnterLong,
                DecisionAction::Short => PolicyAction::ExitPosition,
                DecisionAction::NoTrade => PolicyAction::HoldOrFlat,
            };

            let was_in_position = engine.position.is_some();
            engine.step(i, policy_action, state.atr_14);

            // Record Decision-Level Telemetry
            let bar_rec = engine.bar_records.last().cloned();
            let is_in_position = engine.position.is_some();

            let disposition = match (was_in_position, policy_action, is_in_position) {
                (false, PolicyAction::EnterLong, true) => "ENTER_LONG",
                (false, _, false) => "STAND_ASIDE_FLAT",
                (true, PolicyAction::EnterLong, true) => "REJECTED_CAPACITY",
                (true, PolicyAction::ExitPosition, false) => "EXIT_POSITION",
                (true, PolicyAction::HoldOrFlat, true) => "STAND_ASIDE_HOLDING",
                (true, _, false) => "STOP_OR_TIME_EXIT",
                _ => "NEUTRAL",
            };

            if let Some(br) = bar_rec {
                telemetry.push(DecisionTelemetryRecord {
                    bar_idx: i,
                    timestamp_ms: state.candle.open_time_ms,
                    causal_inputs: CausalTelemetrySummary {
                        reference_price: state.causal_feat.reference_price,
                        ret_15m_bps: state.causal_feat.ret_15m_bps,
                        atr_14: state.causal_feat.atr_14,
                        donchian_breakout: state.causal_feat.donchian_breakout_20,
                        price_range_pos: state.causal_feat.price_range_pos_20,
                        dist_vwap_bps: state.causal_feat.dist_vwap_bps,
                        vol_burst: state.causal_feat.vol_burst_20,
                    },
                    proposed_action: format!("{:?}", action),
                    execution_disposition: disposition.to_string(),
                    position_active: is_in_position,
                    realized_gross_pnl_usdt: br.gross_pnl_usdt,
                    net_pnl_usdt: br.net_pnl_usdt,
                    cumulative_equity_usdt: 0.0, // Calculated below
                    current_drawdown_pct: 0.0,   // Calculated below
                    bars_held: engine.position.as_ref().map(|p| p.bars_held).unwrap_or(0),
                    invariants_passed: true,
                });
            }
        }

        // Close any remaining open position at end of data
        engine.close_at_end_of_data();

        // Calculate Continuous Mark-to-Market Equity Curve and Drawdowns
        let mut peak_equity = self.reference_capital_usdt;
        let mut current_equity = self.reference_capital_usdt;
        let mut max_drawdown_usdt = 0.0;
        let mut max_drawdown_pct = 0.0;

        for rec in &mut telemetry {
            current_equity += rec.net_pnl_usdt;
            if current_equity > peak_equity {
                peak_equity = current_equity;
            }
            let dd_usdt = peak_equity - current_equity;
            let dd_pct = if peak_equity > 0.0 {
                (dd_usdt / peak_equity) * 100.0
            } else {
                0.0
            };
            if dd_usdt > max_drawdown_usdt {
                max_drawdown_usdt = dd_usdt;
            }
            if dd_pct > max_drawdown_pct {
                max_drawdown_pct = dd_pct;
            }
            rec.cumulative_equity_usdt = current_equity;
            rec.current_drawdown_pct = dd_pct;
        }

        let trade_count = engine.completed_trades.len();
        let mut total_net_usdt = 0.0;
        let mut total_friction_usdt = 0.0;
        let mut wins = 0;

        for trade in &engine.completed_trades {
            total_net_usdt += trade.net_pnl_usdt;
            total_friction_usdt += trade.base_friction.base_total_usdt();
            if trade.net_pnl_usdt > 0.0 {
                wins += 1;
            }
        }

        let win_rate = if trade_count > 0 {
            (wins as f64) / (trade_count as f64)
        } else {
            0.0
        };

        let calmar_ratio = if max_drawdown_usdt > 0.0 {
            total_net_usdt / max_drawdown_usdt
        } else if total_net_usdt > 0.0 {
            10.0
        } else {
            0.0
        };

        let objectives = MogaObjectiveVector {
            net_return_usdt: total_net_usdt,
            net_return_bps: (total_net_usdt / self.reference_capital_usdt) * 10_000.0,
            max_drawdown_pct,
            max_drawdown_usdt,
            turnover_friction_usdt: total_friction_usdt,
            trade_count,
        };

        // Governed Multi-Objective Fitness Evaluation:
        // 1. Inactive policies (0 trades) receive honest 0.0 fitness (inactivity proves no trading edge).
        // 2. Under-trading (< 5 trades) receives a penalty.
        // 3. Active policies maximize cost-adjusted economic value:
        //    (Net Return - 2.0 * Turnover Friction) / Max Drawdown
        //    This severely penalizes account churning (overtrading) and rewards high-expectancy trade selection.
        let fitness = if trade_count == 0 {
            0.0 // Honest zero: stand-aside alone is not alpha
        } else if trade_count < 5 {
            -500.0 + (trade_count as f64) * 20.0
        } else {
            let cost_penalized_return = total_net_usdt - (2.0 * total_friction_usdt);
            let risk_denom = max_drawdown_usdt.max(500.0);
            cost_penalized_return / risk_denom
        };

        CryptoGenomeEvaluation {
            genome: genome.clone(),
            fitness,
            objectives,
            calmar_ratio,
            win_rate,
            rejected_entries: engine.rejected_entries,
            valid: true,
            telemetry_log: telemetry,
        }
    }
}

impl FitnessEvaluator<RuleListGenome> for CryptoMogaEvaluator {
    type Evaluation = CryptoGenomeEvaluation;

    fn evaluate(&self, genome: &RuleListGenome, _metrics: &MetricReport) -> Self::Evaluation {
        self.evaluate_genome(genome)
    }
}

/// Evolution output artifact containing the frozen champion and search lineage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrozenChampionArtifact {
    pub policy_id: String,
    pub alphabet: FactorAlphabet,
    pub seed: u64,
    pub population_size: usize,
    pub generation_limit: usize,
    pub champion_genome: RuleListGenome,
    pub genome_hash: String,
    pub fitness: f64,
    pub objectives: MogaObjectiveVector,
    pub calmar_ratio: f64,
    pub win_rate: f64,
    pub rejected_entries: usize,
    pub dataset_sha256: String,
    pub frozen_timestamp_utc: String,
}

/// Executes Coralys MOGA evolution using public `EvolutionEngineBuilder` interfaces.
pub fn evolve_crypto_policy(
    alphabet: FactorAlphabet,
    candles: &[BtcCandle15m],
    dataset_sha256: &str,
    seed: u64,
    population_size: usize,
    generation_limit: usize,
) -> Result<FrozenChampionArtifact, String> {
    let evaluator = CryptoMogaEvaluator::new(candles, alphabet, 100_000.0);
    let factory = CryptoGenomeFactory { alphabet };
    let mutator = CryptoGenomeMutation { alphabet };
    let crossover = RuleListCrossover;

    let engine = EvolutionEngineBuilder::new()
        .with_evaluator(evaluator)
        .with_factory(factory)
        .with_mutator(mutator)
        .with_crossover(crossover)
        .build()
        .map_err(|e| format!("Failed to build EvolutionEngine: {}", e))?;

    let config = EvolutionConfig {
        population_size,
        mutation_rate: 0.20,
        crossover_rate: 0.80,
        elite_count: 2,
        generation_limit,
        seed: Some(seed),
        tournament_size: Some(3),
        termination_policy: None,
    };

    let result = engine.run_ga_evolution(config)?;
    let champion_eval = result.global_best;

    let policy_id = match alphabet {
        FactorAlphabet::BaseOnly => "Policy_B_Champion",
        FactorAlphabet::WithExhaustion => "Policy_C_Champion",
    };

    let genome_bytes = serde_json::to_vec(&champion_eval.genome).expect("serializes");
    let genome_hash = format!("{:x}", Sha256::digest(&genome_bytes));

    Ok(FrozenChampionArtifact {
        policy_id: policy_id.to_string(),
        alphabet,
        seed,
        population_size,
        generation_limit,
        champion_genome: champion_eval.genome,
        genome_hash,
        fitness: champion_eval.fitness,
        objectives: champion_eval.objectives,
        calmar_ratio: champion_eval.calmar_ratio,
        win_rate: champion_eval.win_rate,
        rejected_entries: champion_eval.rejected_entries,
        dataset_sha256: dataset_sha256.to_string(),
        frozen_timestamp_utc: chrono::Utc::now().to_rfc3339(),
    })
}
