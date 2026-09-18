use chronosentiment_core::ese::ExecutionEngine;
use chronosentiment_core::{MarketEvent, MarketEventType, Side};
use coralys_ecology::models::{CognitionGeometry, MemoryState};
use coralys_ecology::traits::MemoryModel;
use coralys_moga::traits::evaluator::{
    CrossoverOperator, Evaluated, FitnessEvaluator, Genome,
    GenomeFactory, MutationOperator,
};
use coralys_moga::config::EvolutionConfig;
use coralys_moga::engine::EvolutionEngine;
use rand::rngs::StdRng;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct CryptoStateRecord {
    timestamp: i64,
    price: f64,
    volume_acceleration_60m: f64,
    persistence_240m: f64,
}

#[derive(Debug, Clone)]
struct PrecomputedState {
    timestamp: i64,
    va: f64,
    persistence: f64,
    realized_pnl: f64,
    mfe: f64,
    mae: f64,
}

// ---------------------------------------------------------
// MOGA Genome & Machinery
// ---------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CandidateRelationship {
    pub va_min: f64,
    pub va_max: f64,
    pub persistence_min: f64,
    pub persistence_max: f64,
    pub use_and: bool,
}

impl Genome for CandidateRelationship {}

#[derive(Clone, Debug, PartialEq)]
pub struct DiscoveryEvaluation {
    pub fitness: f64,
    pub valid: bool,
    pub genome: CandidateRelationship,
    pub median_pnl: f64,
    pub sample_size: usize,
    pub median_mfe: f64,
    pub median_mae: f64,
}

impl Evaluated for DiscoveryEvaluation {
    type Genome = CandidateRelationship;
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

pub struct DiscoveryFactory {
    pub va_bound: (f64, f64),
    pub per_bound: (f64, f64),
}

impl GenomeFactory<CandidateRelationship> for DiscoveryFactory {
    fn create(&self, rng: &mut StdRng) -> CandidateRelationship {
        let v1 = rng.gen_range(self.va_bound.0..self.va_bound.1);
        let v2 = rng.gen_range(self.va_bound.0..self.va_bound.1);
        let p1 = rng.gen_range(self.per_bound.0..self.per_bound.1);
        let p2 = rng.gen_range(self.per_bound.0..self.per_bound.1);
        CandidateRelationship {
            va_min: v1.min(v2),
            va_max: v1.max(v2),
            persistence_min: p1.min(p2),
            persistence_max: p1.max(p2),
            use_and: rng.gen_bool(0.5),
        }
    }
}

pub struct DiscoveryMutator {
    pub va_bound: (f64, f64),
    pub per_bound: (f64, f64),
}

impl MutationOperator<CandidateRelationship> for DiscoveryMutator {
    fn mutate(&self, candidate: &mut CandidateRelationship, rng: &mut StdRng) {
        let mut_type = rng.gen_range(0..5);
        let va_range = self.va_bound.1 - self.va_bound.0;
        let p_range = self.per_bound.1 - self.per_bound.0;
        
        match mut_type {
            0 => candidate.va_min = (candidate.va_min + rng.gen_range(-0.1..0.1) * va_range).clamp(self.va_bound.0, candidate.va_max),
            1 => candidate.va_max = (candidate.va_max + rng.gen_range(-0.1..0.1) * va_range).clamp(candidate.va_min, self.va_bound.1),
            2 => candidate.persistence_min = (candidate.persistence_min + rng.gen_range(-0.1..0.1) * p_range).clamp(self.per_bound.0, candidate.persistence_max),
            3 => candidate.persistence_max = (candidate.persistence_max + rng.gen_range(-0.1..0.1) * p_range).clamp(candidate.persistence_min, self.per_bound.1),
            _ => candidate.use_and = !candidate.use_and,
        }
    }
}

#[derive(Clone)]
pub struct DiscoveryCrossover;

impl CrossoverOperator<CandidateRelationship> for DiscoveryCrossover {
    fn crossover(
        &self,
        p1: &CandidateRelationship,
        p2: &CandidateRelationship,
        rng: &mut StdRng,
    ) -> (CandidateRelationship, CandidateRelationship) {
        if rng.gen_bool(0.5) {
            (
                CandidateRelationship {
                    va_min: p1.va_min,
                    va_max: p1.va_max,
                    persistence_min: p2.persistence_min,
                    persistence_max: p2.persistence_max,
                    use_and: p1.use_and,
                },
                CandidateRelationship {
                    va_min: p2.va_min,
                    va_max: p2.va_max,
                    persistence_min: p1.persistence_min,
                    persistence_max: p1.persistence_max,
                    use_and: p2.use_and,
                }
            )
        } else {
            (
                CandidateRelationship {
                    va_min: p1.va_min,
                    va_max: p2.va_max.max(p1.va_min),
                    persistence_min: p1.persistence_min,
                    persistence_max: p2.persistence_max.max(p1.persistence_min),
                    use_and: p2.use_and,
                },
                CandidateRelationship {
                    va_min: p2.va_min,
                    va_max: p1.va_max.max(p2.va_min),
                    persistence_min: p2.persistence_min,
                    persistence_max: p1.persistence_max.max(p2.persistence_min),
                    use_and: p1.use_and,
                }
            )
        }
    }
}

pub struct DiscoveryEvaluator {
    pub precomputed_states: Vec<PrecomputedState>,
}

impl FitnessEvaluator<CandidateRelationship> for DiscoveryEvaluator {
    type Evaluation = DiscoveryEvaluation;
    fn evaluate(
        &self,
        candidate: &CandidateRelationship,
        _metrics: &coralys_moga::runtime::optimization::metric::MetricReport,
    ) -> Self::Evaluation {
        
        let mut selected_pnls = Vec::new();
        let mut selected_mfes = Vec::new();
        let mut selected_maes = Vec::new();
        
        let mut last_selected_idx: i64 = -60; // Ensure first match is valid
        
        for (idx, state) in self.precomputed_states.iter().enumerate() {
            let va_match = state.va >= candidate.va_min && state.va <= candidate.va_max;
            let per_match = state.persistence >= candidate.persistence_min && state.persistence <= candidate.persistence_max;
            
            let is_match = if candidate.use_and { va_match && per_match } else { va_match || per_match };
            
            if is_match && (idx as i64 - last_selected_idx >= 60) {
                selected_pnls.push(state.realized_pnl);
                selected_mfes.push(state.mfe);
                selected_maes.push(state.mae);
                last_selected_idx = idx as i64;
            }
        }
        
        let sample_size = selected_pnls.len();
        
        if sample_size < 30 {
            return DiscoveryEvaluation {
                fitness: 0.0,
                valid: false,
                genome: candidate.clone(),
                median_pnl: 0.0,
                sample_size,
                median_mfe: 0.0,
                median_mae: 0.0,
            };
        }
        
        selected_pnls.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        selected_mfes.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        selected_maes.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        
        let median_pnl = selected_pnls[sample_size / 2];
        let median_mfe = selected_mfes[sample_size / 2];
        let median_mae = selected_maes[sample_size / 2];
        
        // Fitness = median_pnl * penalizations. 
        // We only care about positive PnL for survival. If median_pnl <= 0, fitness is 0.
        let mut fitness = if median_pnl > 0.0 { median_pnl * 10000.0 } else { 0.0 };
        
        // Complexity penalty: an OR condition is generally less strictly bounded, but depth is bounded to 2.
        // We just evaluate the relationship.
        if !candidate.use_and {
            fitness *= 0.9; // Slight penalty for OR tree to prefer tighter bounding if possible
        }
        
        DiscoveryEvaluation {
            fitness,
            valid: true,
            genome: candidate.clone(),
            median_pnl,
            sample_size,
            median_mfe,
            median_mae,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let workspace = Path::new("/Users/nikhil/ChronoSentiment_MEGA_FINAL");
    let dataset = workspace.join("datasets/crypto_integration_trace_v0.jsonl");

    let file = File::open(&dataset)?;
    let reader = BufReader::new(file);

    let mut original_records = Vec::new();
    let mut market_events = Vec::new();
    
    println!("Reading JSONL from: {}", dataset.display());

    for line in reader.lines() {
        let line = line?;
        let record: CryptoStateRecord = serde_json::from_str(&line)?;
        
        market_events.push(MarketEvent {
            subtype: MarketEventType::Trade,
            price: record.price.round() as u64,
            quantity: 1,
            side: Some(Side::Buy),
            exchange_ts: record.timestamp as u64,
        });
        
        original_records.push(record);
    }
    
    // 1. Precompute Trajectories
    println!("Precomputing explicit trajectories...");
    let mut engine = ExecutionEngine::default();
    let mut precomputed = Vec::new();
    
    let mut min_va = f64::MAX;
    let mut max_va = f64::MIN;
    let mut min_per = f64::MAX;
    let mut max_per = f64::MIN;
    
    for (i, record) in original_records.iter().enumerate() {
        let entry_price = record.price.round() as u64;
        let tp_target = (record.price * 1.05).round() as u64;
        let sl_target = (record.price * 0.95).round() as u64;
        
        let exec_result = engine.simulate_round_trip(
            entry_price,
            tp_target,
            sl_target,
            Side::Buy,
            1,
            &market_events,
            i,
            60,
        );
        
        min_va = min_va.min(record.volume_acceleration_60m);
        max_va = max_va.max(record.volume_acceleration_60m);
        min_per = min_per.min(record.persistence_240m);
        max_per = max_per.max(record.persistence_240m);
        
        precomputed.push(PrecomputedState {
            timestamp: record.timestamp,
            va: record.volume_acceleration_60m,
            persistence: record.persistence_240m,
            realized_pnl: exec_result.realized_pnl,
            mfe: exec_result.mfe,
            mae: exec_result.mae,
        });
    }
    
    // 2. Train / Validation Split (70 / 30)
    let train_size = (precomputed.len() as f64 * 0.70) as usize;
    let train_segment = precomputed[0..train_size].to_vec();
    let val_segment = precomputed[train_size..].to_vec();
    
    println!("Train segment: {} records", train_segment.len());
    println!("Validation segment: {} records", val_segment.len());
    
    // 3. MOGA Search Setup
    println!("Initializing Coralys MOGA Evolutionary Engine...");
    
    let evaluator = DiscoveryEvaluator { precomputed_states: train_segment };
    let mutator = DiscoveryMutator { va_bound: (min_va, max_va), per_bound: (min_per, max_per) };
    let crossover = DiscoveryCrossover {};
    let factory = DiscoveryFactory { va_bound: (min_va, max_va), per_bound: (min_per, max_per) };
    
    let moga_engine = EvolutionEngine::new(evaluator, mutator, crossover, factory);
    
    let config = EvolutionConfig {
        population_size: 100,
        generation_limit: 50,
        seed: Some(42),
        ..Default::default()
    };
    
    let ga_result = moga_engine.run_ga_evolution(config)?;
    let best_train = ga_result.global_best.clone();
    
    println!("--------------------------------------------------");
    println!("TRAIN RESULTS (Best Candidate Frozen)");
    println!("--------------------------------------------------");
    println!("Genome: {:?}", best_train.genome());
    println!("Median PnL: {:.6}", best_train.median_pnl);
    println!("Median MFE: {:.6}", best_train.median_mfe);
    println!("Median MAE: {:.6}", best_train.median_mae);
    println!("Occurrences: {}", best_train.sample_size);
    println!("Fitness: {:.2}", best_train.fitness());
    
    if best_train.sample_size < 30 {
        println!("MOGA failed to find a valid relationship matching the N>=30 constraint on Train.");
        return Ok(());
    }

    // 4. Validation Gate
    println!("--------------------------------------------------");
    println!("VALIDATION GATE");
    println!("--------------------------------------------------");
    
    let val_evaluator = DiscoveryEvaluator { precomputed_states: val_segment };
    // dummy metrics report is ignored by our evaluator anyway
    let dummy_metrics = coralys_moga::runtime::optimization::metric::MetricReport::default(); 
    let val_result = val_evaluator.evaluate(best_train.genome(), &dummy_metrics);
    
    println!("Validation Median PnL: {:.6}", val_result.median_pnl);
    println!("Validation Median MFE: {:.6}", val_result.median_mfe);
    println!("Validation Median MAE: {:.6}", val_result.median_mae);
    println!("Validation Occurrences: {}", val_result.sample_size);
    
    // Acceptance rules:
    // 1. Positive sign preservation: median_pnl > 0
    // 2. val_pnl >= 0.8 * train_pnl
    // 3. Occurrences >= 10
    
    let sign_preserved = val_result.median_pnl > 0.0;
    let degradation_passed = val_result.median_pnl >= (0.80 * best_train.median_pnl);
    let sample_size_passed = val_result.sample_size >= 10;
    
    println!("Criteria Check:");
    println!(" - Sign Preserved (PnL > 0): {}", sign_preserved);
    println!(" - Metric Stability (Val >= 0.8*Train): {}", degradation_passed);
    println!(" - Sample Size (N >= 10): {}", sample_size_passed);
    
    if sign_preserved && degradation_passed && sample_size_passed {
        println!("\n>>> DISCOVERY ACCEPTED <<<");
        println!("A robust State-Behaviour Relationship was discovered and validated by MOGA!");
    } else {
        println!("\n>>> DISCOVERY REJECTED <<<");
        println!("The candidate relationship failed to survive out-of-sample validation.");
    }

    Ok(())
}
